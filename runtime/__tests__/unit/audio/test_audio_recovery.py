import audio_cli
import pytest
from audio_ownership import AudioPhase
from pulse_audio_fake import Pulse, audio

OWNED_SINKS = {
    "AvatarMic",
    "AvatarHuman",
    "AvatarMix",
    "AvatarLipSync",
    "AvatarRemoteMicrophone",
    "AvatarRemoteMeeting",
}


@pytest.fixture
def pulse(monkeypatch):
    backend = Pulse()
    monkeypatch.setattr(audio.pulse_audio, "run", backend.call)
    monkeypatch.setattr(audio.pulse_audio, "items", backend.items)
    return backend


def restart_the_audio_server(pulse):
    pulse.modules = [{"index": 1, "name": "unrelated", "argument": "keep"}]
    pulse.sinks = []
    pulse.sources = [{"name": "synthetic-human.monitor"}]
    pulse.calls.clear()


def phase_of(state, pulse):
    return audio.audio_ownership.observe(state, pulse.modules)


def test_recover_rebuilds_the_mix_an_audio_server_restart_took_away(tmp_path, pulse):
    path = tmp_path / "state.json"
    state = {"modules": []}
    audio.start(path, state)
    restart_the_audio_server(pulse)
    assert phase_of(state, pulse) is AudioPhase.ORPHANED
    audio.recover(path, state)
    assert phase_of(state, pulse) is AudioPhase.RUNNING_DETACHED
    assert {sink["name"] for sink in pulse.sinks} == OWNED_SINKS
    assert audio_cli.describe(state)["running"] is True


def test_recover_returns_the_inputs_muted_so_nothing_reaches_a_meeting_unasked(
    tmp_path, pulse
):
    path = tmp_path / "state.json"
    state = {"modules": []}
    audio.start(path, state)
    audio.open_meeting_inputs(path, state)
    restart_the_audio_server(pulse)
    audio.recover(path, state)
    assert not state.get("microphone")
    assert [call for call in pulse.calls if call[0] == "set-sink-mute"] == [
        ("set-sink-mute", "AvatarMic", "1"),
        ("set-sink-mute", "AvatarHuman", "1"),
    ]


def test_recover_builds_nothing_when_the_mix_was_never_started(tmp_path, pulse):
    path = tmp_path / "state.json"
    state = {"modules": []}
    audio.recover(path, state)
    assert not pulse.calls
    assert not path.exists()
    assert phase_of(state, pulse) is AudioPhase.ABSENT


def test_recover_leaves_a_running_mix_untouched(tmp_path, pulse):
    path = tmp_path / "state.json"
    state = {"modules": []}
    audio.start(path, state)
    audio.open_meeting_inputs(path, state)
    before = (list(pulse.calls), list(pulse.modules), dict(state))
    audio.recover(path, state)
    assert (list(pulse.calls), list(pulse.modules), dict(state)) == before


def test_the_command_line_offers_recover(monkeypatch):
    monkeypatch.setattr("sys.argv", ["avatar-audio", "recover"])
    assert audio_cli.parse_arguments().action == "recover"
