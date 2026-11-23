import queue
import threading
from unittest.mock import Mock

import barge_in
import listener
import source_capture
import speech_segmenter

SOUND_FRAME = b"\xb8\x0b" * 480
SILENT_FRAME = bytes(960)
FRAMES_BEFORE_BARGE_IN = barge_in.BARGE_IN_FRAMES
LONGEST_SEGMENT_FRAMES = speech_segmenter.frames_for(
    speech_segmenter.MAXIMUM_SEGMENT_SECONDS
)


def run_now(target, *arguments):
    target(*arguments)


class FakeClock:
    def __init__(self):
        self.now = 1000.0

    def __call__(self):
        return self.now


def listening_with(tmp_path, heard, replies, recognition_seconds=0.0):
    clock = FakeClock()

    def recognize_words(audio):
        clock.now += recognition_seconds
        return heard

    return listener.Listening(
        segments=queue.Queue(maxsize=4),
        stopping=threading.Event(),
        pending=listener.PendingSpeech(tmp_path / "marker"),
        interrupt_speaker=Mock(),
        speaking_replies=Mock(return_value=list(replies)),
        recognize_words=Mock(side_effect=recognize_words),
        start_confirmation=run_now,
        clock=clock,
    )


def hear_sound(listening, frames):
    speech = source_capture.SourceSpeech("human", listening)
    for _ in range(frames):
        speech.feed(SOUND_FRAME)
    return speech
