import json
import pathlib
import tempfile

import remote_agent
import wake_claim
from claim_differential.world import (
    CONVERSATIONS,
    DIRECTORY,
    HOST,
    NO_FIXTURE_CHECK,
    PANES,
    SCENES,
    script_world,
    seed,
)
from hypothesis import given
from hypothesis import strategies as st


@NO_FIXTURE_CHECK
@given(scene=SCENES)
def test_active_conversation_agrees_with_python(rust, monkeypatch, scene):
    script_world(monkeypatch, scene)
    with tempfile.TemporaryDirectory() as root:
        directory = pathlib.Path(root)
        seed(directory, scene)
        expected = wake_claim.active_conversation(directory)
        fields = {
            "directory": str(directory),
            "listening": scene["listening"],
            "host": HOST,
        }
        assert rust.call("claim.active_conversation", **fields) == expected
        assert rust.call("claim.conversation_report", **fields) == json.dumps(expected)


@NO_FIXTURE_CHECK
@given(pane=PANES)
def test_claim_path_agrees_with_python(rust, pane):
    expected = wake_claim.claim_path(pathlib.Path(DIRECTORY), pane)
    relative = str(expected.relative_to(DIRECTORY))
    assert rust.call("claim.claim_path", directory=DIRECTORY, pane=pane) == relative


@NO_FIXTURE_CHECK
@given(
    conversation=st.one_of(st.none(), CONVERSATIONS),
    pane=PANES,
    same=st.booleans(),
)
def test_matches_session_agrees_with_python(
    rust, monkeypatch, conversation, pane, same
):
    monkeypatch.setenv(remote_agent.HOST_ENVIRONMENT_VARIABLE, HOST)
    if conversation is not None and same:
        pane = conversation["pane"]
    expected = wake_claim.matches_session(conversation, pane)
    fields = {"conversation": conversation, "pane": pane, "host": HOST}
    assert rust.call("claim.matches_session", **fields) == expected


@NO_FIXTURE_CHECK
@given(
    conversation=st.one_of(
        st.none(),
        CONVERSATIONS,
        st.text(max_size=5),
        st.fixed_dictionaries({"host": PANES, "pane": st.integers()}),
        st.dictionaries(st.text(max_size=3), st.text(max_size=3), max_size=3),
    )
)
def test_names_a_session_agrees_with_python(rust, conversation):
    expected = wake_claim.names_a_session(conversation)
    assert rust.call("claim.names_a_session", conversation=conversation) == expected
