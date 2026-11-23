import sys

from speech_segmenter import FRAME_SECONDS

SPEECH_PROBABILITY = 0.85
SPEECH_FRAMES = 5


class BargeIn:
    def __init__(self, reply, voice_activity):
        self.reply = reply
        self.voice_activity = voice_activity
        self.hearing_reply = False
        self.begin_segment()

    def begin_segment(self):
        self.segment_frames = 0
        self.speech_frames = 0
        self.stopped = False

    def hear(self, frame, loud):
        self.segment_frames += 1
        if not self.reply.speaking():
            self.hearing_reply = False
            return
        self.speech_frames += self.speech_heard(frame) and loud
        if self.speech_frames >= SPEECH_FRAMES and not self.stopped:
            self.stop_reply()

    def speech_heard(self, frame):
        if not self.hearing_reply:
            self.voice_activity.reset()
            self.hearing_reply = True
        return self.voice_activity.speech_probability(frame) >= SPEECH_PROBABILITY

    def stop_reply(self):
        self.stopped = True
        self.reply.stop()
        heard_seconds = self.segment_frames * FRAME_SECONDS
        print(
            f"Stopped the reply {heard_seconds:.3f} s after the sound began",
            file=sys.stderr,
            flush=True,
        )


class NoBargeIn:
    def begin_segment(self):
        pass

    def hear(self, frame, loud):
        pass
