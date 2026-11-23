import json
import os
import subprocess
from pathlib import Path
from unittest.mock import Mock

import pytest
import quiet_conversation
import speaking_floor
from conversation_addressee import Addressee, write
from listener_session import ListenerPurpose
from rust_floor_shadow import FLOOR_ENVIRONMENT, RustFloorShadow, as_json
from wake_phrase import LiveSession

ROSTER = [LiveSession("Astolfo", "%1"), LiveSession("Ibaraki-dōji", "%2")]
CLAIMED = Addressee("%1", "Astolfo", 0)
HELD = Addressee("%2", "Ibaraki-dōji", 0)
CLOSED = {"closed": True, "reports": [quiet_conversation.CLOSING], "close_calls": 1}
REAL_BINARY = os.environ.get("AVATAR_FLOOR_BINARY", "")


def ok(value):
    return json.dumps({"ok": value}) + "\n"


def answering(monkeypatch, output):
    if isinstance(output, Exception):
        run = Mock(side_effect=output)
    else:
        run = Mock(return_value=subprocess.CompletedProcess([], 0, output, ""))
    monkeypatch.setattr(subprocess, "run", run)
    return "/bin/floor"


def echoing_the_addressee_file(command, input, **options):
    path = Path(json.loads(input)["directory"], "addressee.json")
    holder = json.loads(path.read_text()) if path.exists() else None
    return subprocess.CompletedProcess([], 0, ok({"holder": holder}), "")


def claim_with(shadow, directory):
    boundaries = speaking_floor.FloorBoundaries(lambda: ROSTER, lambda: True)
    return shadow.claim(speaking_floor.claim, directory, "%1", boundaries)


def quiet_with(shadow):
    boundaries = Mock()
    boundaries.quiet_seconds.return_value = 700.0
    boundaries.listening_purpose.return_value = ListenerPurpose.CONVERSATION
    decide = quiet_conversation.close_when_quiet
    return shadow.close_when_quiet(decide, boundaries, False)


def test_flag_off_never_runs_the_binary(tmp_path, monkeypatch):
    monkeypatch.delenv(FLOOR_ENVIRONMENT, raising=False)
    monkeypatch.setattr(subprocess, "run", Mock(side_effect=AssertionError("ran")))
    report = Mock()
    for shadow in (RustFloorShadow(report), RustFloorShadow(report, binary="")):
        assert claim_with(shadow, tmp_path) == CLAIMED
        assert quiet_with(shadow) is True
    report.assert_not_called()


@pytest.mark.parametrize(
    ("output", "differs"),
    [(ok({"holder": as_json(CLAIMED)}), False), (ok({"holder": None}), True)],
)
def test_claim_reports_only_a_difference(tmp_path, monkeypatch, output, differs):
    report = Mock()
    shadow = RustFloorShadow(report, answering(monkeypatch, output))
    assert claim_with(shadow, tmp_path) == CLAIMED
    assert report.call_count == int(differs)
    assert not differs or "floor.claim" in report.call_args.args[0]


@pytest.mark.parametrize(
    ("output", "differs"), [(ok(CLOSED), False), (ok({"closed": False}), True)]
)
def test_quiet_decision_reports_only_a_difference(monkeypatch, output, differs):
    report = Mock()
    shadow = RustFloorShadow(report, answering(monkeypatch, output))
    assert quiet_with(shadow) is True
    assert report.call_count == int(differs)


def test_claim_reads_the_held_addressee_from_a_scratch_copy(tmp_path, monkeypatch):
    monkeypatch.setattr(subprocess, "run", echoing_the_addressee_file)
    write(tmp_path, HELD)
    report = Mock()
    assert claim_with(RustFloorShadow(report, "/bin/floor"), tmp_path) == HELD
    report.assert_not_called()


@pytest.mark.parametrize(
    "output",
    [
        "not json\n",
        "",
        "{}\n{}\n",
        '{"error": "bad"}\n',
        subprocess.TimeoutExpired("avatar-floor", 2),
        FileNotFoundError("avatar-floor"),
    ],
)
def test_failed_replies_keep_python_answers(tmp_path, monkeypatch, output):
    report = Mock()
    shadow = RustFloorShadow(report, answering(monkeypatch, output))
    assert claim_with(shadow, tmp_path) == CLAIMED
    assert quiet_with(shadow) is True
    assert report.call_count == 2
    assert all(
        call.args[0].startswith("Rust floor unavailable: ")
        for call in report.call_args_list
    )


@pytest.mark.skipif(not REAL_BINARY, reason="AVATAR_FLOOR_BINARY names the binary")
def test_real_binary_agrees_with_python(tmp_path):
    write(tmp_path, HELD)
    report = Mock()
    shadow = RustFloorShadow(report, REAL_BINARY)
    assert claim_with(shadow, tmp_path) == HELD
    assert quiet_with(shadow) is True
    report.assert_not_called()
