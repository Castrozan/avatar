import json
import os
import subprocess
import sys
from unittest.mock import Mock

import pytest
from rust_shadow import (
    ADDRESSED_SESSION,
    BINARY_ENVIRONMENT,
    OPENS_WITH_GREETING,
    SESSIONS_HEARD_ANYWHERE,
    RustShadow,
)
from wake_phrase import (
    LiveSession,
    addressed_session,
    opens_with_greeting,
    sessions_heard_anywhere,
)

ROSTER = [LiveSession("Astolfo", "%1"), LiveSession("Ibaraki-dōji", "%2")]
UTTERANCES = ["Hey Astolfo, are you there?", "Astolfo, check the build", "hello there"]


def fake_binary(tmp_path, reply):
    output = json.dumps(reply) + "\n"
    script = tmp_path / "fake-avatar-routing"
    script.write_text(
        f"#!{sys.executable}\nimport sys\nsys.stdin.read()\n"
        f"sys.stdout.write({output!r})\n"
    )
    script.chmod(0o755)
    return str(script)


def python_answers(text, sessions):
    return (
        (OPENS_WITH_GREETING, (), opens_with_greeting(text)),
        (SESSIONS_HEARD_ANYWHERE, sessions, sessions_heard_anywhere(text, sessions)),
        (ADDRESSED_SESSION, sessions, addressed_session(text, sessions)),
    )


def test_flag_off_never_runs_the_binary(monkeypatch):
    monkeypatch.delenv(BINARY_ENVIRONMENT, raising=False)
    monkeypatch.setattr(subprocess, "run", Mock(side_effect=AssertionError("ran")))
    report = Mock()
    RustShadow(report).compare(OPENS_WITH_GREETING, "hey", (), True)
    RustShadow(report, binary="").compare(OPENS_WITH_GREETING, "hey", (), True)
    report.assert_not_called()


def test_matching_answer_reports_nothing(tmp_path):
    report = Mock()
    shadow = RustShadow(report, fake_binary(tmp_path, {"ok": True}))
    shadow.compare(OPENS_WITH_GREETING, "hey", (), True)
    report.assert_not_called()


def test_differing_answer_reports_one_line_with_the_operation(tmp_path):
    report = Mock()
    shadow = RustShadow(report, fake_binary(tmp_path, {"ok": False}))
    shadow.compare(OPENS_WITH_GREETING, "hey Astolfo", (), True)
    (line,) = [call.args[0] for call in report.call_args_list]
    assert "opens_with_greeting" in line and "hey Astolfo" in line


def test_addressed_session_is_compared_in_its_wire_shape(tmp_path):
    report = Mock()
    match = addressed_session("Hey Astolfo, check", ROSTER)
    reply = {
        "ok": {
            "session": {"servant": "Astolfo", "pane": "%1", "host": None},
            "message": "check",
        }
    }
    shadow = RustShadow(report, fake_binary(tmp_path, reply))
    shadow.compare(ADDRESSED_SESSION, "Hey Astolfo, check", ROSTER, match)
    report.assert_not_called()


def test_one_json_line_with_a_two_second_timeout(monkeypatch):
    run = Mock(return_value=subprocess.CompletedProcess([], 0, '{"ok": true}\n', ""))
    monkeypatch.setattr(subprocess, "run", run)
    RustShadow(Mock(), "/bin/routing").compare(
        SESSIONS_HEARD_ANYWHERE, "Astolfo", ROSTER, [ROSTER[0]]
    )
    assert run.call_args.args == (["/bin/routing"],)
    assert run.call_args.kwargs["timeout"] == 2
    assert json.loads(run.call_args.kwargs["input"]) == {
        "op": SESSIONS_HEARD_ANYWHERE,
        "text": "Astolfo",
        "sessions": [
            {"servant": "Astolfo", "pane": "%1", "host": None},
            {"servant": "Ibaraki-dōji", "pane": "%2", "host": None},
        ],
    }


def assert_reports_unavailable_once(report):
    (line,) = [call.args[0] for call in report.call_args_list]
    assert line.startswith("Rust routing unavailable: ")


def test_missing_binary_reports_unavailable_once(tmp_path):
    report = Mock()
    RustShadow(report, str(tmp_path / "missing")).compare(
        OPENS_WITH_GREETING, "hey", (), True
    )
    assert_reports_unavailable_once(report)


def test_timeout_reports_unavailable_once(monkeypatch):
    timeout = subprocess.TimeoutExpired("avatar-routing", 2)
    monkeypatch.setattr(subprocess, "run", Mock(side_effect=timeout))
    report = Mock()
    RustShadow(report, "/bin/routing").compare(OPENS_WITH_GREETING, "hey", (), True)
    assert_reports_unavailable_once(report)


@pytest.mark.parametrize("reply", [{"error": "bad op"}, {"neither": 1}, [1, 2]])
def test_error_or_unexpected_reply_reports_unavailable_once(tmp_path, reply):
    report = Mock()
    shadow = RustShadow(report, fake_binary(tmp_path, reply))
    shadow.compare(OPENS_WITH_GREETING, "hey", (), True)
    assert_reports_unavailable_once(report)


@pytest.mark.parametrize("output", ["not json\n", "", "{}\n{}\n"])
def test_unparseable_output_reports_unavailable_once(monkeypatch, output):
    completed = subprocess.CompletedProcess([], 0, output, "")
    monkeypatch.setattr(subprocess, "run", Mock(return_value=completed))
    report = Mock()
    RustShadow(report, "/bin/routing").compare(OPENS_WITH_GREETING, "hey", (), True)
    assert_reports_unavailable_once(report)


@pytest.mark.skipif(
    not os.environ.get("AVATAR_ROUTING_BINARY"),
    reason="AVATAR_ROUTING_BINARY names the Rust avatar-routing binary",
)
def test_real_binary_agrees_with_python_on_the_roster():
    report = Mock()
    shadow = RustShadow(report, os.environ["AVATAR_ROUTING_BINARY"])
    for text in UTTERANCES:
        for operation, sessions, result in python_answers(text, ROSTER):
            shadow.compare(operation, text, sessions, result)
    report.assert_not_called()
