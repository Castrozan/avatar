import dataclasses
import json
import tempfile
import threading
import time
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import conversation_addressee
import remote_agent
import room_holder
import servant_character
import speaking_floor
from conversation_addressee import Addressee
from hypothesis import given, settings
from hypothesis import strategies as st
from remote_agent import AgentPane

SETTINGS = settings(max_examples=150, deadline=None)
NAMES = st.sampled_from(["Voyager", "Kira", "Iris", ""])
PANES = st.sampled_from(["wW:pPA", "wW:pPB", "wX:pPA", "hermes:ava", "hermes:kira"])
HOSTS = st.sampled_from([None, "kira", "chise"])
HEARD = st.integers(min_value=-(10**9), max_value=10**9)
SESSIONS = st.lists(
    st.fixed_dictionaries({"servant": NAMES, "pane": PANES, "host": HOSTS}), max_size=4
)
ADDRESSEE = st.builds(
    Addressee, pane=PANES, servant=NAMES, heard_through=HEARD, host=HOSTS
)
OPTIONAL_ADDRESSEE = ADDRESSEE | st.none()


def as_json(addressee):
    return dataclasses.asdict(addressee) if addressee else None


def boundaries_of(records, listening):
    sessions = [SimpleNamespace(**record) for record in records]
    return speaking_floor.FloorBoundaries(lambda: sessions, lambda: listening)


def with_file(home, reply):
    path = Path(home) / "addressee.json"
    file = json.loads(path.read_text()) if path.exists() else None
    return {**reply, "file": file}


def compare(rust, operation, python_reply, holder=None, **fields):
    with (
        tempfile.TemporaryDirectory() as python_home,
        tempfile.TemporaryDirectory() as rust_home,
    ):
        for home in (python_home, rust_home):
            if holder is not None:
                conversation_addressee.write(Path(home), holder)
        expected = python_reply(Path(python_home))
        actual = rust.call(f"floor.{operation}", directory=rust_home, **fields)
    assert actual == expected


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


@SETTINGS
@given(ADDRESSEE, OPTIONAL_ADDRESSEE)
def test_give_the_room_to_matches_python(rust, addressee, holder):
    def python_reply(home):
        shown = []
        with mock.patch.object(servant_character, "show_for", shown.append):
            room_holder.give_the_room_to(home, addressee)
        return with_file(home, {"shown": shown})

    compare(
        rust, "give_the_room_to", python_reply, holder, addressee=as_json(addressee)
    )


@SETTINGS
@given(PANES, HOSTS, NAMES | st.none(), HEARD, OPTIONAL_ADDRESSEE)
def test_hand_to_remote_agent_matches_python(rust, pane, host, servant, heard, holder):
    def python_reply(home):
        shown = []
        agent = AgentPane(host=host, pane=pane)
        with (
            mock.patch.object(remote_agent, "servant_at", lambda _: servant),
            mock.patch.object(servant_character, "show_for", shown.append),
        ):
            handed = speaking_floor.hand_to_remote_agent(home, agent, heard)
        return with_file(home, {"handed": handed, "shown": shown})

    fields = {
        "agent": {"pane": pane, "host": host},
        "servant": servant,
        "heard_through": heard,
    }
    compare(rust, "hand_to_remote_agent", python_reply, holder, **fields)


@SETTINGS
@given(OPTIONAL_ADDRESSEE)
def test_addressee_current_matches_python(rust, holder):
    compare(
        rust,
        "addressee.current",
        lambda home: as_json(conversation_addressee.current(home)),
        holder,
    )


@SETTINGS
@given(ADDRESSEE, OPTIONAL_ADDRESSEE)
def test_addressee_hand_to_matches_python(rust, addressee, holder):
    def python_reply(home):
        conversation_addressee.hand_to(home, addressee)
        return with_file(home, {})

    compare(
        rust, "addressee.hand_to", python_reply, holder, addressee=as_json(addressee)
    )


@SETTINGS
@given(OPTIONAL_ADDRESSEE)
def test_addressee_forget_matches_python(rust, holder):
    def python_reply(home):
        conversation_addressee.forget(home)
        return with_file(home, {})

    compare(rust, "addressee.forget", python_reply, holder)


@SETTINGS
@given(OPTIONAL_ADDRESSEE, PANES | st.none())
def test_addressee_addresses_matches_python(rust, holder, pane):
    def python_reply(home):
        return conversation_addressee.addresses(home, pane)

    compare(rust, "addressee.addresses", python_reply, holder, pane=pane)


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
