import threading
import time

import conversation_addressee
import speaking_floor
from floor_support import (
    OPTIONAL_ADDRESSEE,
    PANES,
    SESSIONS,
    SETTINGS,
    as_json,
    boundaries_of,
    compare,
    with_file,
)
from hypothesis import given
from hypothesis import strategies as st


@SETTINGS
@given(PANES, SESSIONS, st.booleans(), OPTIONAL_ADDRESSEE)
def test_claim_matches_python(rust, pane, sessions, listening, holder):
    def python_reply(home):
        claimed = speaking_floor.claim(home, pane, boundaries_of(sessions, listening))
        return with_file(home, {"holder": as_json(claimed)})

    compare(
        rust,
        "claim",
        python_reply,
        holder,
        pane=pane,
        sessions=sessions,
        listening=listening,
    )


@SETTINGS
@given(PANES, SESSIONS, OPTIONAL_ADDRESSEE)
def test_first_claim_matches_python(rust, pane, sessions, holder):
    expected = speaking_floor.first_claim(
        pane, boundaries_of(sessions, True).sessions(), holder
    )
    actual = rust.call(
        "floor.first_claim", pane=pane, sessions=sessions, holder=as_json(holder)
    )
    assert actual == as_json(expected)


@SETTINGS
@given(PANES, OPTIONAL_ADDRESSEE)
def test_may_speak_matches_python(rust, pane, holder):
    expected = speaking_floor.may_speak(holder, pane)
    assert rust.call("floor.may_speak", pane=pane, holder=as_json(holder)) == expected


def test_rust_claim_waits_for_the_python_addressee_lock(rust, tmp_path):
    held = threading.Event()

    def hold_the_lock():
        with conversation_addressee.exclusive(tmp_path):
            held.set()
            time.sleep(0.3)

    holder_thread = threading.Thread(target=hold_the_lock)
    holder_thread.start()
    held.wait(5)
    started = time.monotonic()
    sessions = [{"servant": "Voyager", "pane": "wW:pPA", "host": None}]
    reply = rust.call(
        "floor.claim",
        directory=str(tmp_path),
        pane="wW:pPA",
        sessions=sessions,
        listening=True,
    )
    elapsed = time.monotonic() - started
    holder_thread.join()
    assert elapsed >= 0.25
    assert reply["holder"]["servant"] == "Voyager"
