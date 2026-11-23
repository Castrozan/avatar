import dataclasses
import json
import tempfile
from pathlib import Path
from types import SimpleNamespace

import conversation_addressee
import speaking_floor
from conversation_addressee import Addressee
from hypothesis import settings
from hypothesis import strategies as st

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
