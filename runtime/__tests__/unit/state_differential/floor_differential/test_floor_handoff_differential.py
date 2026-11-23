from unittest import mock

import remote_agent
import room_holder
import servant_character
import speaking_floor
from floor_support import (
    ADDRESSEE,
    HEARD,
    HOSTS,
    NAMES,
    OPTIONAL_ADDRESSEE,
    PANES,
    SETTINGS,
    as_json,
    compare,
    with_file,
)
from hypothesis import given
from hypothesis import strategies as st
from remote_agent import AgentPane


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
