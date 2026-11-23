import sys
import threading

import speech_segmenter
import transcript_echo

BARGE_IN_SECONDS = 0.6
BARGE_IN_FRAMES = speech_segmenter.frames_for(BARGE_IN_SECONDS)
CONFIRMATION_SECONDS = 2.0
CHECKS_PER_SEGMENT = 3


def interrupts(heard, replies):
    words = transcript_echo.normalized(heard)
    if not words:
        return False
    return not any(
        transcript_echo.echoes_part_of(words, transcript_echo.normalized(reply))
        for reply in replies
    )


def start_daemon_thread(target, *arguments):
    threading.Thread(target=target, args=arguments, daemon=True).start()


class SegmentBargeIn:
    def __init__(self, listening):
        self.listening = listening
        self.next_check_frames = BARGE_IN_FRAMES
        self.checks = 0
        self.checking = threading.Event()
        self.interrupted = threading.Event()

    def due(self, speech_frames):
        if self.checking.is_set() or self.interrupted.is_set():
            return False
        if self.checks >= CHECKS_PER_SEGMENT:
            return False
        return speech_frames >= self.next_check_frames

    def hear(self, segmenter):
        if not self.due(segmenter.speech_frames):
            return
        self.next_check_frames = segmenter.speech_frames + BARGE_IN_FRAMES
        replies = self.listening.speaking_replies()
        if not replies:
            return
        self.checks += 1
        self.checking.set()
        self.listening.start_confirmation(
            self.confirm, segmenter.audio, replies, self.listening.clock()
        )

    def confirm(self, audio, replies, requested):
        try:
            heard = self.listening.recognize_words(audio)
            self.decide(heard, replies, requested)
        except Exception as error:
            print(f"Could not confirm a barge-in: {error}", file=sys.stderr)
        finally:
            self.checking.clear()

    def decide(self, heard, replies, requested):
        if self.listening.clock() - requested > CONFIRMATION_SECONDS:
            return
        if interrupts(heard, replies):
            self.interrupted.set()
            self.listening.interrupt_speaker()
