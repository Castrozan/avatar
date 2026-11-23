import argparse
import json
import subprocess
import time

import conversation_seat
from meetings import virtual_meeting
from meetings.readiness import audio_ready, camera_ready, renderer_ready
from conversation_seat import Seat

MEETING_TARGET = "avatar.target"
CALL_CHANNELS = ("agent", "human")
AUDIO_COMMAND_TIMEOUT_SECONDS = 30
SYSTEMCTL_QUERY_TIMEOUT_SECONDS = 10
AUDIO_REPAIR_TIMEOUT_SECONDS = 60
READINESS_TIMEOUT_SECONDS = 45
CONVERSATION_TARGET = "avatar-conversation.target"
WINDOW_TARGET = "avatar-window.target"
TARGET_UNITS = {
    MEETING_TARGET: (
        "avatar-audio",
        "avatar-control-server",
        "avatar-renderer",
        "avatar-browser",
        "avatar-camera",
    ),
    CONVERSATION_TARGET: (
        "avatar-audio",
        "avatar-control-server",
        "avatar-room-microphone",
    ),
    WINDOW_TARGET: (
        "avatar-control-server",
        "avatar-renderer",
        "avatar-browser",
    ),
}


def service_failed(units):
    failed = subprocess.run(
        ["systemctl", "--user", "is-failed", *units],
        capture_output=True,
        text=True,
        timeout=SYSTEMCTL_QUERY_TIMEOUT_SECONDS,
    )
    return "failed" in failed.stdout.splitlines()


def wait_until_ready(units, readiness_checks, timeout=READINESS_TIMEOUT_SECONDS):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if service_failed(units):
            raise RuntimeError(
                "An avatar service failed; inspect systemctl --user status avatar-*"
            )
        if all(check() for check in readiness_checks):
            return
        time.sleep(0.25)
    raise TimeoutError(
        f"{', '.join(units)} did not become ready within {timeout} seconds"
    )


def bring_up(target, readiness_checks):
    try:
        if not audio_ready():
            subprocess.run(
                ["avatar-audio", "start"],
                check=True,
                stdout=subprocess.DEVNULL,
                timeout=AUDIO_REPAIR_TIMEOUT_SECONDS,
            )
        subprocess.run(
            ["systemctl", "--user", "start", target],
            check=True,
            timeout=AUDIO_REPAIR_TIMEOUT_SECONDS,
        )
        wait_until_ready(TARGET_UNITS[target], readiness_checks)
        if not audio_ready():
            raise RuntimeError("The avatar audio graph did not become ready")
    except BaseException:
        subprocess.run(
            ["systemctl", "--user", "stop", target],
            check=False,
            timeout=AUDIO_COMMAND_TIMEOUT_SECONDS,
        )
        raise


def parse_arguments(arguments=None):
    parser = argparse.ArgumentParser(prog="avatar-start")
    parser.add_argument("--seat", **conversation_seat.MEETING_SEAT_OPTIONS)
    parser.add_argument(
        "--virtual-only",
        action="store_true",
        help="Start fresh virtual meeting devices without physical capture; refuse an existing runtime.",
    )
    return parser.parse_args(arguments)


def open_call_channels():
    for channel in CALL_CHANNELS:
        subprocess.run(
            ["avatar-audio", "mute", channel, "0"],
            check=True,
            stdout=subprocess.DEVNULL,
            timeout=AUDIO_COMMAND_TIMEOUT_SECONDS,
        )


def main(seat=Seat.ROOM, virtual_only=False):
    if virtual_only:
        if seat is not Seat.ROOM:
            raise RuntimeError(
                "Virtual-only preparation requires the local meeting seat"
            )
        units = [
            MEETING_TARGET,
            CONVERSATION_TARGET,
            "avatar-listener.service",
            *TARGET_UNITS[MEETING_TARGET],
        ]
        virtual_meeting.require_fresh_runtime(
            units, audio_ready, SYSTEMCTL_QUERY_TIMEOUT_SECONDS
        )
        try:
            bring_up(MEETING_TARGET, (renderer_ready, camera_ready))
        except BaseException:
            subprocess.run(
                ["avatar-audio", "stop"],
                check=False,
                timeout=AUDIO_COMMAND_TIMEOUT_SECONDS,
            )
            raise
        print(
            json.dumps(
                {
                    "ready": True,
                    "owner": "avatar.target",
                    "camera": "Avatar Cam",
                    "microphone": "AvatarMicrophone",
                    "physical_capture": False,
                    "client_device_selection": "required",
                }
            )
        )
        return
    bring_up(MEETING_TARGET, (renderer_ready, camera_ready))
    if seat is Seat.REMOTE:
        conversation_seat.prepare(seat)
        open_call_channels()
        print("Avatar ready with her voice and the microphone at the remote seat")
        return
    subprocess.run(
        ["avatar-audio", "open"],
        check=True,
        stdout=subprocess.DEVNULL,
        timeout=AUDIO_COMMAND_TIMEOUT_SECONDS,
    )
    print(
        "Avatar ready with her voice and the default microphone open. Check levels with avatar-audio status. Switch video with avatar-command mode"
    )


if __name__ == "__main__":
    arguments = parse_arguments()
    main(arguments.seat, arguments.virtual_only)
