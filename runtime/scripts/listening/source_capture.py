import dataclasses
import subprocess
import sys
import queue
import time

import speech_segmenter
from barge_in import SegmentBargeIn
from turn_detector import TurnTracker

FRAME_BYTES = int(speech_segmenter.SAMPLE_RATE * speech_segmenter.FRAME_SECONDS) * 2
CAPTURE_RETRY_SECONDS = 1
DISCORD_DEVICE = "discord"
DISCORD_CAPTURE_COMMAND = ["discord-voice", "capture"]


@dataclasses.dataclass(frozen=True)
class HeardMoments:
    speech_ended: float
    segment_closed: float


def capture_command(device):
    return [
        "parec",
        f"--device={device}",
        "--client-name=avatar-listener",
        "--property=node.dont-reconnect=true",
        "--property=node.dont-fallback=true",
        "--raw",
        "--format=s16le",
        f"--rate={speech_segmenter.SAMPLE_RATE}",
        "--channels=1",
        "--latency-msec=100",
    ]


def capture_command_by_device(machine_capture_command):
    def capture_command(device):
        if device == DISCORD_DEVICE:
            return list(DISCORD_CAPTURE_COMMAND)
        return machine_capture_command(device)

    return capture_command


class SourceSpeech:
    def __init__(self, source, listening):
        self.source = source
        self.listening = listening
        self.segmenter = speech_segmenter.SpeechSegmenter()
        self.turn = TurnTracker(listening.turn_finished)
        self.barge_in = SegmentBargeIn(listening)

    def feed(self, frame):
        was_active = self.segmenter.active
        segment = self.segmenter.feed(frame)
        if self.segmenter.active:
            if not was_active:
                self.begin_segment()
            self.barge_in.hear(self.segmenter)
        elif was_active:
            self.hand_over(segment)

    def begin_segment(self):
        self.listening.pending.begin()
        self.barge_in = SegmentBargeIn(self.listening)

    def hand_over(self, segment):
        if segment is None:
            self.listening.pending.finish()
            return
        closed = time.time()
        heard = HeardMoments(closed - self.segmenter.closing_silence_seconds, closed)
        turn_finished = self.turn.add(segment)
        if not enqueue_segment(
            self.listening.segments, self.source, segment, turn_finished, heard
        ):
            self.listening.pending.finish()

    def abandon(self):
        if self.segmenter.active:
            self.listening.pending.finish()


def capture_source(source, device, listening):
    while not listening.stopping.is_set():
        capture = subprocess.Popen(
            listening.capture_command(device), stdout=subprocess.PIPE
        )
        speech = SourceSpeech(source, listening)
        while not listening.stopping.is_set() and (
            frame := capture.stdout.read(FRAME_BYTES)
        ):
            speech.feed(frame)
        speech.abandon()
        capture.terminate()
        capture.wait()
        listening.stopping.wait(CAPTURE_RETRY_SECONDS)


def enqueue_segment(segments, source, segment, turn_finished, heard):
    try:
        segments.put_nowait((source, segment, turn_finished, heard))
        return True
    except queue.Full:
        print(f"Dropped a {source} segment: transcription is behind", file=sys.stderr)
        return False
