import json
import subprocess
import types

import listener_runtime
import remote_agent
from arbitration import wake_claim
from hypothesis import HealthCheck, settings
from hypothesis import strategies as st

HOST = "thishost"
DIRECTORY = "/srv/avatar-claims"
ROOM_NAMES = [HOST, "kira", "chise room", "wsl:é"]
NO_FIXTURE_CHECK = settings(
    suppress_health_check=[HealthCheck.function_scoped_fixture],
    max_examples=150,
    deadline=None,
)
PANES = st.text(
    st.characters(blacklist_characters="\x00", blacklist_categories=["Cs"]),
    min_size=1,
    max_size=10,
)
TIMES = st.floats(1e9, 1e9 + 30, allow_nan=False, allow_infinity=False)
CONVERSATIONS = st.fixed_dictionaries(
    {"host": st.sampled_from([HOST, "kira"]), "pane": PANES}
)
ADDRESSEES = st.fixed_dictionaries(
    {
        "pane": PANES,
        "servant": st.text(max_size=6),
        "heard_through": st.integers(0, 100),
        "host": st.one_of(st.none(), st.just(""), st.sampled_from([HOST, "kira"])),
    }
)
ANSWERS = st.one_of(
    st.none(),
    st.just("error"),
    CONVERSATIONS,
    st.fixed_dictionaries({"host": st.just(1), "pane": PANES}),
    st.sampled_from(["", "not json", "[1]", '{"host": 1}']),
)
ROOMS = st.fixed_dictionaries({name: ANSWERS for name in ROOM_NAMES})
ROOM_OF_RECORD = st.one_of(st.none(), st.sampled_from(ROOM_NAMES + [""]))
RECORD_OBJECTS = st.one_of(
    st.fixed_dictionaries({"room": ROOM_OF_RECORD, "claimed_at": TIMES}),
    st.fixed_dictionaries({"room": ROOM_OF_RECORD}),
    st.fixed_dictionaries({"claimed_at": TIMES}),
    st.just({}),
)
RECORDS = st.one_of(
    RECORD_OBJECTS.map(json.dumps), st.sampled_from(["", "{", "garbage é"])
)
SCENES = st.fixed_dictionaries(
    {
        "pane": PANES,
        "room": st.sampled_from(ROOM_NAMES),
        "now": TIMES,
        "listening": st.booleans(),
        "rooms": ROOMS,
        "addressee": st.one_of(st.none(), ADDRESSEES),
        "claim": st.one_of(st.none(), RECORDS),
    }
)


def answer_process(answer):
    if answer == "error":
        raise subprocess.CalledProcessError(1, "ssh")
    text = answer if isinstance(answer, str) else json.dumps(answer)
    return types.SimpleNamespace(stdout=text)


def script_world(monkeypatch, scene):
    monkeypatch.setenv(remote_agent.HOST_ENVIRONMENT_VARIABLE, HOST)
    monkeypatch.setattr(listener_runtime, "listening", lambda: scene["listening"])
    monkeypatch.setattr(
        subprocess,
        "run",
        lambda command, **_options: answer_process(scene["rooms"][command[-2]]),
    )


def seed(directory, scene):
    directory.mkdir(parents=True, exist_ok=True)
    if scene["addressee"] is not None:
        (directory / "addressee.json").write_text(
            json.dumps(scene["addressee"]), encoding="utf-8"
        )
    if scene["claim"] is not None:
        path = wake_claim.claim_path(directory, scene["pane"])
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(scene["claim"], encoding="utf-8")


def files_of(directory, pane):
    paths = [directory / "addressee.json", wake_claim.claim_path(directory, pane)]
    return {
        str(path.relative_to(directory)): path.read_text(encoding="utf-8")
        for path in paths
        if path.exists()
    }


def python_outcome(directory, scene):
    try:
        result = wake_claim.claim(directory, scene["pane"], scene["room"], scene["now"])
    except Exception:
        return {"error": True}
    return {"result": result, "files": files_of(directory, scene["pane"])}


def rust_outcome(rust, directory, scene):
    try:
        return rust.call(
            "claim.claim",
            directory=str(directory),
            pane=scene["pane"],
            room=scene["room"],
            now=scene["now"],
            listening=scene["listening"],
            host=HOST,
            rooms=scene["rooms"],
        )
    except AssertionError:
        return {"error": True}
