import dataclasses

import speech_segmenter

HEARING_SLACK_SECONDS = 3


@dataclasses.dataclass(frozen=True)
class HeardSegment:
    audio: bytes
    heard_at: float

    def began_seconds_ago(self, now):
        audio_seconds = len(self.audio) / (2 * speech_segmenter.SAMPLE_RATE)
        return audio_seconds + now - self.heard_at


def reason_to_leave(boundaries, dry_run, session, heard):
    discord_host = boundaries.discord_call_host()
    if discord_host:
        return f"the Discord call on {discord_host}"
    room = boundaries.room_that_heard(
        heard.began_seconds_ago(boundaries.now()) + HEARING_SLACK_SECONDS
    )
    if room:
        return f"the conversation on {room}"
    if dry_run or boundaries.claim_wake(session):
        return None
    return "another room"


def holds_the_room(boundaries, dry_run, session, heard):
    reason = reason_to_leave(boundaries, dry_run, session, heard)
    if reason:
        boundaries.report(f"left {session.servant} to {reason}")
    return reason is None
