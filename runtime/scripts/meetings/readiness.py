import json
import subprocess
import urllib.error
import urllib.request

PROBE_TIMEOUT_SECONDS = 10


def control_server_health():
    try:
        with urllib.request.urlopen(
            "http://127.0.0.1:8766/health", timeout=1
        ) as response:
            return json.load(response)
    except (urllib.error.URLError, TimeoutError):
        return {}


def control_server_ready():
    return control_server_health().get("status") == "ok"


def renderer_ready():
    return control_server_health().get("rendererConnected", False)


def camera_ready():
    capture = subprocess.run(
        ["v4l2-ctl", "-d", "/dev/video10", "--info"],
        capture_output=True,
        text=True,
        timeout=PROBE_TIMEOUT_SECONDS,
    )
    return capture.returncode == 0 and "Video Capture" in capture.stdout


def audio_ready():
    result = subprocess.run(
        ["avatar-audio", "status"],
        capture_output=True,
        text=True,
        timeout=PROBE_TIMEOUT_SECONDS,
    )
    if result.returncode:
        return False
    return json.loads(result.stdout).get("running", False)
