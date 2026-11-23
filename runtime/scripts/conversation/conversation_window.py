import enum
import subprocess

import conversation_seat
import desktop_window
import listener_session
import user_units
from conversation_seat import Seat
from meetings import readiness, start

COMMAND_TIMEOUT_SECONDS = 120


class WindowPresence(enum.StrEnum):
    HIDDEN = "hidden"
    SHOWN = "shown"


def set_canvas_presence(presence):
    subprocess.run(
        ["avatar-command", "presence", presence],
        capture_output=True,
        check=True,
        timeout=COMMAND_TIMEOUT_SECONDS,
    )


def hide_everywhere():
    set_canvas_presence(WindowPresence.HIDDEN)
    desktop_window.hide()


def reveal_window(directory):
    start.wait_until_ready(
        start.TARGET_UNITS[start.WINDOW_TARGET], (readiness.renderer_ready,)
    )
    if conversation_seat.current(directory) is Seat.ROOM:
        desktop_window.show()
    set_canvas_presence(WindowPresence.SHOWN)


def show_window(arguments, directory):
    start.bring_up(start.WINDOW_TARGET, (readiness.renderer_ready,))


def hide_window(arguments, directory):
    subprocess.run(
        ["systemctl", "--user", "stop", start.WINDOW_TARGET],
        check=True,
        timeout=listener_session.SYSTEMCTL_TIMEOUT_SECONDS,
    )
    if user_units.unit_active(start.MEETING_TARGET):
        hide_everywhere()
