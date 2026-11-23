import dataclasses
import json
import os
import subprocess

BINARY_ENVIRONMENT = "AVATAR_RUST_ROUTING"
OPENS_WITH_GREETING = "opens_with_greeting"
SESSIONS_HEARD_ANYWHERE = "sessions_heard_anywhere"
ADDRESSED_SESSION = "addressed_session"
RUN_OPTIONS = {
    "capture_output": True,
    "text": True,
    "timeout": 2,
    "check": False,
}


class RustUnavailable(Exception):
    pass


def session_values(sessions):
    return [dataclasses.asdict(session) for session in sessions]


def addressed_value(match):
    if match is None:
        return None
    return {"session": dataclasses.asdict(match.session), "message": match.message}


WIRE_VALUES = {
    OPENS_WITH_GREETING: bool,
    SESSIONS_HEARD_ANYWHERE: session_values,
    ADDRESSED_SESSION: addressed_value,
}


def run_binary(binary, request):
    line = json.dumps(request, ensure_ascii=False) + "\n"
    try:
        completed = subprocess.run([binary], input=line, **RUN_OPTIONS)
    except (OSError, subprocess.SubprocessError) as error:
        raise RustUnavailable(str(error) or type(error).__name__) from error
    return completed.stdout


def parsed_reply(output):
    lines = output.splitlines()
    if len(lines) != 1:
        raise RustUnavailable(f"expected one reply line, got {len(lines)}")
    try:
        reply = json.loads(lines[0])
    except ValueError as error:
        raise RustUnavailable(f"invalid reply: {error}") from error
    if not isinstance(reply, dict):
        raise RustUnavailable("reply is not an object")
    return reply


def reply_value(output):
    reply = parsed_reply(output)
    if "error" in reply:
        raise RustUnavailable(str(reply["error"]))
    if "ok" not in reply:
        raise RustUnavailable("reply has neither ok nor error")
    return reply["ok"]


def difference_line(operation, text, expected, actual):
    return (
        f"Rust routing differs on {operation}: python={expected!r} "
        f"rust={actual!r} text={text!r}"
    )


class RustShadow:
    def __init__(self, report, binary=None):
        self.report = report
        self.binary = (
            os.environ.get(BINARY_ENVIRONMENT, "") if binary is None else binary
        )

    def compare(self, operation, text, sessions, python_result):
        if not self.binary:
            return
        expected = WIRE_VALUES[operation](python_result)
        try:
            actual = self.ask(operation, text, sessions)
        except RustUnavailable as error:
            self.report(f"Rust routing unavailable: {error}")
            return
        if actual != expected:
            self.report(difference_line(operation, text, expected, actual))

    def ask(self, operation, text, sessions):
        request = {"op": operation, "text": text, "sessions": session_values(sessions)}
        return reply_value(run_binary(self.binary, request))
