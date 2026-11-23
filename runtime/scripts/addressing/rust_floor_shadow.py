import dataclasses
import tempfile
from functools import partial
from pathlib import Path

import conversation_addressee
import transcript_log
from rust_floor_recording import RecordingFloorBoundaries, RecordingQuietBoundaries
from rust_shadow import RustUnavailable, answer, configured_binary

FLOOR_ENVIRONMENT = "AVATAR_RUST_FLOOR"
CLAIM = "floor.claim"
CLOSE_WHEN_QUIET = "quiet.close_when_quiet"
CLOSE_WHEN_THE_CALL_ENDED = "quiet.close_when_the_call_ended"


def as_json(addressee):
    return dataclasses.asdict(addressee) if addressee else None


def addressee_text(directory):
    path = directory / conversation_addressee.ADDRESSEE_FILE
    try:
        return path.read_text() if path.exists() else None
    except OSError:
        return None


def reply_field(reply, key):
    if not isinstance(reply, dict) or key not in reply:
        raise RustUnavailable(f"reply has no {key}")
    return reply[key]


def difference_line(operation, expected, actual):
    return f"Rust floor differs on {operation}: python={expected!r} rust={actual!r}"


def claim_request(pane, recording):
    sessions = [dataclasses.asdict(session) for session in recording.sessions_read]
    return {
        "op": CLAIM,
        "pane": pane,
        "listening": recording.listening_read,
        "sessions": sessions,
    }


def quiet_expectation(closed, recording):
    return {
        "closed": closed,
        "reports": recording.reports,
        "close_calls": recording.close_calls,
    }


def answer_in_scratch(binary, addressee_snapshot, request):
    with tempfile.TemporaryDirectory() as scratch:
        if addressee_snapshot is not None:
            path = Path(scratch, conversation_addressee.ADDRESSEE_FILE)
            path.write_text(addressee_snapshot)
        return answer(binary, {**request, "directory": scratch})


def claimed_holder(binary, addressee_snapshot, pane, recording):
    request = claim_request(pane, recording)
    reply = answer_in_scratch(binary, addressee_snapshot, request)
    return reply_field(reply, "holder")


class RustFloorShadow:
    def __init__(self, report=transcript_log.report_to_log, binary=None):
        self.report = report
        self.binary = configured_binary(FLOOR_ENVIRONMENT) if binary is None else binary

    def check(self, operation, expected, ask):
        if not self.binary:
            return
        try:
            actual = ask()
        except (RustUnavailable, OSError, TypeError) as error:
            self.report(f"Rust floor unavailable: {error}")
            return
        if actual != expected:
            self.report(difference_line(operation, expected, actual))

    def claim(self, decide, directory, pane, boundaries):
        if not self.binary:
            return decide(directory, pane, boundaries)
        recording = RecordingFloorBoundaries(boundaries)
        snapshot = addressee_text(directory)
        holder = decide(directory, pane, recording)
        ask = partial(claimed_holder, self.binary, snapshot, pane, recording)
        self.check(CLAIM, as_json(holder), ask)
        return holder

    def close_when_quiet(self, decide, boundaries, dry_run):
        return self.quiet_decision(CLOSE_WHEN_QUIET, decide, boundaries, dry_run)

    def close_when_the_call_ended(self, decide, boundaries, dry_run):
        operation = CLOSE_WHEN_THE_CALL_ENDED
        return self.quiet_decision(operation, decide, boundaries, dry_run)

    def quiet_decision(self, operation, decide, boundaries, dry_run):
        if not self.binary:
            return decide(boundaries, dry_run)
        recording = RecordingQuietBoundaries(boundaries)
        closed = decide(recording, dry_run)
        request = {"op": operation, "dry_run": dry_run, **recording.observed}
        expected = quiet_expectation(closed, recording)
        self.check(operation, expected, lambda: answer(self.binary, request))
        return closed
