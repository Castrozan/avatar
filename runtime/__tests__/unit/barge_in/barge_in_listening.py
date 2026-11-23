import queue
import struct
import threading

import listener
import source_capture
import speech_segmenter
from barge_in import decision

FRAME_SAMPLES = int(speech_segmenter.SAMPLE_RATE * speech_segmenter.FRAME_SECONDS)
SPEECH_SAMPLE = 3000
NOISE_SAMPLE = 3001
QUIET_SPEECH_SAMPLE = 100
SPEECH_PROBABILITY_BY_SAMPLE = {SPEECH_SAMPLE: 0.97, QUIET_SPEECH_SAMPLE: 0.97}
NOISE_PROBABILITY = 0.05


def frame_of(sample):
    return struct.pack(f"<{FRAME_SAMPLES}h", *([sample] * FRAME_SAMPLES))


SPEECH = frame_of(SPEECH_SAMPLE)
NOISE = frame_of(NOISE_SAMPLE)
QUIET_SPEECH = frame_of(QUIET_SPEECH_SAMPLE)
SILENCE = frame_of(0)
TRAILING_SILENCE_FRAMES = speech_segmenter.frames_for(
    speech_segmenter.TRAILING_SILENCE_SECONDS
)


class SimulatedClock:
    def __init__(self):
        self.seconds = 0.0

    def __call__(self):
        return self.seconds

    def advance_one_frame(self):
        self.seconds += speech_segmenter.FRAME_SECONDS


class Reply:
    def __init__(self, clock, speaking=True):
        self.clock = clock
        self.is_speaking = speaking
        self.stops = []

    def speaking(self):
        return self.is_speaking

    def stop(self):
        self.stops.append(self.clock())


class FrameVoiceActivity:
    def __init__(self):
        self.judged = 0
        self.resets = 0

    def reset(self):
        self.resets += 1

    def speech_probability(self, frame):
        self.judged += 1
        sample = struct.unpack_from("<h", frame)[0]
        return SPEECH_PROBABILITY_BY_SAMPLE.get(sample, NOISE_PROBABILITY)


class HeardReply:
    def __init__(self, tmp_path, speaking=True):
        self.clock = SimulatedClock()
        self.reply = Reply(self.clock, speaking)
        self.voice_activity = FrameVoiceActivity()
        listening = listener.Listening(
            segments=queue.Queue(),
            stopping=threading.Event(),
            pending=listener.PendingSpeech(tmp_path / "marker"),
            barge_in=lambda: decision.BargeIn(self.reply, self.voice_activity),
        )
        self.speech = source_capture.SourceSpeech("human", listening)

    def hear(self, frames):
        for frame in frames:
            self.clock.advance_one_frame()
            self.speech.feed(frame)
        return self.reply.stops
