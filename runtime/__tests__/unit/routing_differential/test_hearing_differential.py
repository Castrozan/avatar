import dataclasses
from unittest import mock

import name_hearing
import wake_phrase
from hypothesis import given, settings
from hypothesis import strategies as st
from strategies import SERVANTS, rosters, texts, utterances, vocabularies
from transcription.ordinary_words import folded

EXAMPLES = settings(max_examples=300, deadline=None)
phrases = st.one_of(texts, utterances())


def sessions_json(sessions):
    return [dataclasses.asdict(session) for session in sessions]


def match_json(match):
    if match is None:
        return None
    return {"session": dataclasses.asdict(match.session), "message": match.message}


def with_vocabulary(vocabulary):
    return mock.patch.object(name_hearing, "ordinary_vocabulary", lambda: vocabulary)


@EXAMPLES
@given(
    st.lists(st.lists(st.text(max_size=8).map(folded), min_size=1, max_size=3), max_size=5),
    st.lists(st.sampled_from(SERVANTS).map(lambda name: folded(name.split()[0])), max_size=4, unique=True),
    vocabularies,
)
def test_heard_name_matches_python(rust, windows, names, vocabulary):
    expected = name_hearing.heard_name([tuple(item) for item in windows], dict.fromkeys(names), vocabulary)
    actual = rust.call("heard_name", windows=windows, names=names, vocabulary=vocabulary)
    assert actual == (None if expected is None else {"name": expected[0], "size": expected[1]})


@EXAMPLES
@given(phrases, rosters(), vocabularies)
def test_wake_functions_match_python(rust, text, roster, vocabulary):
    request = {"text": text, "sessions": sessions_json(roster), "vocabulary": vocabulary}
    with with_vocabulary(vocabulary):
        assert rust.call("opens_with_greeting", text=text) == wake_phrase.opens_with_greeting(text)
        heard = wake_phrase.sessions_heard_anywhere(text, roster)
        assert rust.call("sessions_heard_anywhere", **request) == sessions_json(heard)
        vocative = wake_phrase.vocative_session(text, roster)
        assert rust.call("vocative_session", **request) == (vocative and dataclasses.asdict(vocative))
        assert rust.call("addressed_session", **request) == match_json(wake_phrase.addressed_session(text, roster))
        assert rust.call("named_in_turn", **request) == match_json(wake_phrase.named_in_turn(text, roster))
