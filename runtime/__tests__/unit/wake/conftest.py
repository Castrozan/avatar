from types import SimpleNamespace

import floor_cli
import listener_runtime
import local_seat
import pytest


@pytest.fixture
def shown(monkeypatch):
    servants = []
    monkeypatch.setattr(
        floor_cli.servant_character,
        "show_for",
        lambda servant: servants.append(servant),
    )
    return servants


@pytest.fixture
def seat_listening(monkeypatch):
    state = SimpleNamespace(listening=True)
    monkeypatch.setattr(local_seat, "conversation_listening", lambda: state.listening)
    return state


@pytest.fixture(autouse=True)
def conversation_here(monkeypatch):
    state = SimpleNamespace(listening=False)
    monkeypatch.setattr(listener_runtime, "listening", lambda: state.listening)
    return state


@pytest.fixture
def clock():
    return SimpleNamespace(now=100.0)

