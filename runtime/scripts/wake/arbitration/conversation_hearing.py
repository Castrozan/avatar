import concurrent.futures
import json
import subprocess
import time

import listener
import listener_runtime
import transcript_log

ROOM_SOURCE = "human"
AVATAR_SPEAKER = "avatar"
REACH_ROOM = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=5"]
ASK_TIMEOUT_SECONDS = 8
ROOM_UNREADABLE = (OSError, subprocess.SubprocessError, ValueError)


def spoken_by_the_room(entry):
    return entry.get("source") == ROOM_SOURCE and entry.get("speaker") != AVATAR_SPEAKER


def speech_ended_at(entry):
    return entry.get("speech_ended") or entry["time"]


def room_spoke_within(entries, seconds, now):
    return any(
        spoken_by_the_room(entry) and now - speech_ended_at(entry) <= seconds
        for entry in entries
    )


def heard_within(directory, seconds, now):
    if not listener_runtime.listening():
        return False
    if (directory / listener.SPEECH_IN_PROGRESS_MARKER).exists():
        return True
    return room_spoke_within(
        transcript_log.read_since(directory / "transcript.jsonl", 0), seconds, now
    )


def report(seconds):
    return json.dumps(
        heard_within(listener_runtime.runtime_directory(), seconds, time.time())
    )


def heard_in_room(host, seconds, run):
    try:
        answer = run(
            [*REACH_ROOM, host, "avatar-wake", "--heard-within", f"{seconds:.1f}"],
            capture_output=True,
            text=True,
            timeout=ASK_TIMEOUT_SECONDS,
        )
        return answer.returncode == 0 and json.loads(answer.stdout) is True
    except ROOM_UNREADABLE:
        return False


def room_that_heard(hosts, seconds, run):
    if not hosts:
        return None
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(hosts)) as asking:
        answers = asking.map(lambda host: heard_in_room(host, seconds, run), hosts)
        return next(
            (host for host, heard in zip(hosts, answers, strict=True) if heard), None
        )
