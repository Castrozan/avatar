import dataclasses
import json
import os
import queue
import signal
import sys
import threading
import time
from pathlib import Path

from barge_in import decision
from transcription import cloud_transcriber
import listener_runtime
import speaker_identity
from pending_speech import PendingSpeech
import source_capture
import transcript_echo
import transcript_log
import turn_detector

TRANSCRIBER_RETRY_SECONDS = 1
TRANSCRIBER_ATTEMPTS = 30
PENDING_SEGMENT_LIMIT = 16
SPEECH_IN_PROGRESS_MARKER = "speech-in-progress"
EMBEDDINGS_FILE = "embeddings.jsonl"
HEARD_FILE = "heard"


def every_pause_ends_the_turn(audio):
    return True


@dataclasses.dataclass
class Listening:
    segments: queue.Queue
    stopping: threading.Event
    pending: PendingSpeech
    barge_in: object = decision.NoBargeIn
    capture_command: object = source_capture.capture_command
    language: str | None = None
    identify_speakers: bool = False
    turn_finished: object = every_pause_ends_the_turn


def transcribe_when_ready(segment, language):
    for _ in range(TRANSCRIBER_ATTEMPTS):
        try:
            return cloud_transcriber.transcribe_with_local_fallback(segment, language)
        except OSError:
            time.sleep(TRANSCRIBER_RETRY_SECONDS)
    raise TimeoutError("No transcriber answered")


@dataclasses.dataclass
class TranscriptRecorder:
    path: Path
    listening: Listening
    echoes: transcript_echo.EchoFilter
    identifier: object

    def record(self, source, segment, turn_finished, heard):
        transcription_started = time.time()
        text = transcribe_when_ready(segment, self.listening.language)
        if not text or self.echoes.is_echo(source, text, time.time()):
            return
        speaker, embedding = self.identifier.identify(segment)
        entry = transcript_log.append(
            self.path,
            source,
            text,
            speaker,
            turn_finished=turn_finished,
            speech_ended=heard.speech_ended,
            segment_closed=heard.segment_closed,
            transcription_started=transcription_started,
        )
        if embedding:
            speaker_identity.append_embedding(
                self.path.with_name(EMBEDDINGS_FILE), entry["sequence"], embedding
            )


def write_transcript(path, listening):
    recorder = TranscriptRecorder(
        path,
        listening,
        transcript_echo.EchoFilter(),
        speaker_identity.configured_identifier(listening.identify_speakers),
    )
    try:
        while not listening.stopping.is_set():
            try:
                source, segment, turn_finished, heard = listening.segments.get(
                    timeout=0.5
                )
            except queue.Empty:
                continue
            try:
                recorder.record(source, segment, turn_finished, heard)
            except Exception as error:
                print(f"Dropped a {source} segment: {error}", file=sys.stderr)
            finally:
                listening.pending.finish()
    finally:
        listening.pending.clear()


def capture_command_of_this_machine():
    if sys.platform == "darwin":
        import mac_seat_client

        return source_capture.capture_command_by_device(mac_seat_client.capture_command)
    return source_capture.capture_command_by_device(source_capture.capture_command)


def barge_in_over_replies(stopping):
    from barge_in import reply_link, voice_activity

    reply = reply_link.ReplyLink(stopping).start()
    model = os.environ[voice_activity.MODEL_ENVIRONMENT_VARIABLE]
    return lambda: decision.BargeIn(reply, voice_activity.SileroVoiceActivity(model))


def main():
    capture_command = capture_command_of_this_machine()
    directory = listener_runtime.runtime_directory()
    configuration = json.loads((directory / "listener.json").read_text())
    detector = turn_detector.SmartTurnDetector(
        os.environ[turn_detector.MODEL_ENVIRONMENT_VARIABLE]
    )
    stopping = threading.Event()
    listening = Listening(
        segments=queue.Queue(maxsize=PENDING_SEGMENT_LIMIT),
        stopping=stopping,
        pending=PendingSpeech(directory / SPEECH_IN_PROGRESS_MARKER),
        barge_in=barge_in_over_replies(stopping)
        if configuration.get("barge_in")
        else decision.NoBargeIn,
        capture_command=capture_command,
        language=configuration.get("language"),
        identify_speakers=configuration.get("identify_speakers", False),
        turn_finished=detector.finished,
    )
    signal.signal(signal.SIGTERM, lambda *_: listening.stopping.set())
    for source, device in configuration["sources"].items():
        threading.Thread(
            target=source_capture.capture_source,
            args=(source, device, listening),
            daemon=True,
        ).start()
    write_transcript(directory / "transcript.jsonl", listening)


if __name__ == "__main__":
    main()
