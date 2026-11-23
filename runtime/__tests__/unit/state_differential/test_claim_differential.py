import fcntl
import pathlib
import tempfile
import threading
import time

import conversation_addressee
import wake_claim
from claim_differential.world import (
    HOST,
    NO_FIXTURE_CHECK,
    SCENES,
    rust_outcome,
    python_outcome,
    script_world,
    seed,
)
from hypothesis import given


@NO_FIXTURE_CHECK
@given(scene=SCENES)
def test_claim_agrees_with_python(rust, monkeypatch, scene):
    script_world(monkeypatch, scene)
    with tempfile.TemporaryDirectory() as root:
        python_directory = pathlib.Path(root, "python")
        rust_directory = pathlib.Path(root, "rust")
        seed(python_directory, scene)
        seed(rust_directory, scene)
        assert rust_outcome(rust, rust_directory, scene) == python_outcome(
            python_directory, scene
        )


def timed_call(operation):
    started = time.monotonic()
    outcome = operation()
    return outcome, time.monotonic() - started


def test_neither_side_waits_for_a_held_addressee_lock(rust, tmp_path, monkeypatch):
    script_world(monkeypatch, {"listening": False, "rooms": {}})
    held = threading.Event()

    def hold_addressee_lock():
        with conversation_addressee.exclusive(tmp_path):
            held.set()
            time.sleep(0.3)

    holder = threading.Thread(target=hold_addressee_lock)
    holder.start()
    held.wait()
    rust_result, rust_elapsed = timed_call(
        lambda: rust.call(
            "claim.claim",
            directory=str(tmp_path),
            pane="w:p",
            room="kira",
            now=1e9,
            listening=False,
            host=HOST,
            rooms={},
        )
    )
    python_result, python_elapsed = timed_call(
        lambda: wake_claim.claim(tmp_path, "w:p", "kira", 1e9)
    )
    holder.join()
    assert rust_result["result"] is True and rust_elapsed < 0.25
    assert python_result is True and python_elapsed < 0.25


def test_a_held_claim_file_refuses_both_without_blocking(rust, tmp_path, monkeypatch):
    script_world(monkeypatch, {"listening": False, "rooms": {}})
    path = wake_claim.claim_path(tmp_path, "w:p")
    path.parent.mkdir(parents=True)
    held = threading.Event()

    def hold_claim_lock():
        with path.open("a+") as handle:
            fcntl.flock(handle, fcntl.LOCK_EX)
            held.set()
            time.sleep(0.3)

    holder = threading.Thread(target=hold_claim_lock)
    holder.start()
    held.wait()
    rust_result, rust_elapsed = timed_call(
        lambda: rust.call(
            "claim.claim",
            directory=str(tmp_path),
            pane="w:p",
            room="kira",
            now=1e9,
            listening=False,
            host=HOST,
            rooms={},
        )
    )
    python_result, python_elapsed = timed_call(
        lambda: wake_claim.claim(tmp_path, "w:p", "kira", 1e9)
    )
    holder.join()
    assert rust_result["result"] is False and rust_elapsed < 0.25
    assert python_result is False and python_elapsed < 0.25
