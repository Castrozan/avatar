import conversation_addressee
from floor_support import (
    ADDRESSEE,
    OPTIONAL_ADDRESSEE,
    PANES,
    SETTINGS,
    as_json,
    compare,
    with_file,
)
from hypothesis import given
from hypothesis import strategies as st


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
