import os
import shutil
import subprocess
from pathlib import Path

import pytest

AVATAR_STOP = (
    Path(__file__).resolve().parents[2] / "runtime" / "scripts" / "avatar-stop"
)
STOP_EVERY_OWNED_UNIT_IN_ONE_JOB = (
    "systemctl --user stop avatar-conversation.target avatar-listener.service "
    "avatar-remote-seat.service avatar.target avatar-camera.service "
    "avatar-browser.service avatar-control-server.service "
    "avatar-renderer.service avatar-audio.service"
)


@pytest.fixture
def call_log(tmp_path, monkeypatch):
    fake_commands = tmp_path / "bin"
    fake_commands.mkdir()
    log = tmp_path / "calls.log"
    for command, exit_status_variable in (
        ("systemctl", "SYSTEMCTL_EXIT_STATUS"),
        ("avatar-audio", "AVATAR_AUDIO_EXIT_STATUS"),
    ):
        fake = fake_commands / command
        fake.write_text(
            f"#!{shutil.which('bash')}\n"
            f'echo "{command} $*" >>"$CALL_LOG"\n'
            f'exit "${exit_status_variable}"\n'
        )
        fake.chmod(0o755)
    monkeypatch.setenv("PATH", f"{fake_commands}{os.pathsep}{os.environ['PATH']}")
    monkeypatch.setenv("CALL_LOG", str(log))
    monkeypatch.setenv("SYSTEMCTL_EXIT_STATUS", "0")
    monkeypatch.setenv("AVATAR_AUDIO_EXIT_STATUS", "0")
    return log


def run_avatar_stop():
    return subprocess.run(["bash", str(AVATAR_STOP)], check=False).returncode


def test_is_executable():
    assert os.access(AVATAR_STOP, os.X_OK)


def test_stops_listening_the_target_and_every_owned_service_in_one_job_before_audio_cleanup(
    call_log,
):
    assert run_avatar_stop() == 0
    calls = call_log.read_text().splitlines()
    assert STOP_EVERY_OWNED_UNIT_IN_ONE_JOB in calls
    assert calls[-1] == "avatar-audio stop"


def test_reports_the_audio_refusal_while_agent_playback_is_connected(
    call_log, monkeypatch
):
    monkeypatch.setenv("AVATAR_AUDIO_EXIT_STATUS", "3")
    assert run_avatar_stop() == 3


def test_still_cleans_audio_when_stopping_services_fails_and_keeps_that_failure(
    call_log, monkeypatch
):
    monkeypatch.setenv("SYSTEMCTL_EXIT_STATUS", "5")
    assert run_avatar_stop() == 5
    assert call_log.read_text().splitlines()[-1] == "avatar-audio stop"
