import unicodedata
from unittest.mock import Mock

import pytest
from hypothesis import HealthCheck, given, settings
from hypothesis import strategies as st

from scribe_answers import SEGMENT, detected, requested_languages, stub_scribe_answers
from transcription import cloud_transcriber
from transcription import spoken_languages
from transcription import transcriber

SPOKEN_PINS = ("pt", "en", "ja", "es")
SPOKEN_CODES = ("pt", "por", "en", "eng", "ja", "jpn", "es", "spa")
FOREIGN_CODES = ("rus", "zho", "heb", "ita", "pol", "srp", "mkd", "kan", "ko", "ru")
LANGUAGE_CODES = st.one_of(
    st.none(), st.text(max_size=4), st.sampled_from(SPOKEN_CODES + FOREIGN_CODES)
)
SCRIPT_SAMPLES = (
    "pode ser público",
    "Привет! Как твои дела сегодня?",
    "早晨好，中国",
    "こんにちは、元気？",
    "שלום",
    "안녕하세요",
    "ça va, señor",
)
SPOKEN_TEXT = st.one_of(st.text(max_size=12), st.sampled_from(SCRIPT_SAMPLES))
SPOKEN_SCRIPTS = (
    "LATIN",
    "HIRAGANA",
    "KATAKANA",
    "CJK",
    "IDEOGRAPHIC",
    "ORDINAL INDICATOR",
)
PROBABILITIES = st.floats(min_value=0, max_value=1)
NO_FIXTURE_CHECK = settings(
    suppress_health_check=[HealthCheck.function_scoped_fixture], max_examples=200
)


@pytest.fixture(autouse=True)
def fresh_recent_language(monkeypatch):
    monkeypatch.delenv("AVATAR_SERVANT_ROSTER", raising=False)
    monkeypatch.setattr(
        cloud_transcriber, "recent_language", spoken_languages.RecentLanguage()
    )


def test_russian_that_scribe_keeps_writing_under_a_spanish_pin_is_dropped(
    monkeypatch,
):
    urlopen = stub_scribe_answers(
        monkeypatch,
        detected("¿Y tú qué opinas?", "spa"),
        detected("Привет! Как твои дела сегодня?", "rus", 0.9505199790000916),
        detected("Привет! Как твои дела сегодня?", "es", 1.0),
    )
    texts = [cloud_transcriber.transcribe(SEGMENT) for _ in range(2)]
    assert texts == ["¿Y tú qué opinas?", ""]
    assert requested_languages(urlopen) == [None, None, "es"]


def test_chinese_that_scribe_keeps_writing_under_an_english_pin_is_dropped(
    monkeypatch,
):
    stub_scribe_answers(
        monkeypatch,
        detected("早晨好，中国，我现在我有冰淇淋", "zho", 0.9999053478240967),
        detected("早晨好，中国，我现在我有冰淇淋", "en", 1.0),
    )
    assert cloud_transcriber.transcribe(SEGMENT) == ""


def test_portuguese_mislabelled_as_hebrew_survives_the_portuguese_pin(monkeypatch):
    stub_scribe_answers(
        monkeypatch,
        detected("בלזה, קואלקר אאוטרה קויזה", "heb", 0.9993641376495361),
        detected("Beleza, qualquer outra coisa?", "pt", 1.0),
    )
    assert cloud_transcriber.transcribe(SEGMENT) == "Beleza, qualquer outra coisa?"


def test_cyrillic_labelled_as_a_spoken_language_is_dropped(monkeypatch):
    stub_scribe_answers(monkeypatch, detected("Ааа, влом", "por", 0.4))
    assert cloud_transcriber.transcribe(SEGMENT) == ""


def test_a_dropped_segment_does_not_become_the_pinned_language(monkeypatch):
    urlopen = stub_scribe_answers(
        monkeypatch,
        detected("hello", "eng"),
        detected("Ааа, влом", "spa"),
        detected("שלום", "heb"),
        detected("hello again", "en"),
    )
    texts = [cloud_transcriber.transcribe(SEGMENT) for _ in range(3)]
    assert texts == ["hello", "", "hello again"]
    assert requested_languages(urlopen) == [None, None, None, "en"]


def test_a_session_pinned_outside_the_spoken_languages_never_reaches_scribe(
    monkeypatch,
):
    urlopen = stub_scribe_answers(monkeypatch)
    assert cloud_transcriber.transcribe_with_local_fallback(SEGMENT, "ru") == ""
    assert urlopen.call_count == 0


def test_foreign_script_from_the_local_fallback_is_dropped(monkeypatch):
    stub_scribe_answers(monkeypatch, OSError("offline"))
    monkeypatch.setattr(transcriber, "transcribe", Mock(return_value="Ааа, влом"))
    assert cloud_transcriber.transcribe_with_local_fallback(SEGMENT) == ""


def is_written_in_a_spoken_script(character):
    name = unicodedata.name(character, "")
    return not character.isalpha() or any(script in name for script in SPOKEN_SCRIPTS)


def is_spoken(text):
    return all(is_written_in_a_spoken_script(character) for character in text)


@NO_FIXTURE_CHECK
@given(
    history=st.lists(st.tuples(SPOKEN_TEXT, LANGUAGE_CODES), max_size=4),
    session_language=st.one_of(st.none(), st.sampled_from(SPOKEN_CODES + ("ru",))),
    code=LANGUAGE_CODES,
    probability=PROBABILITIES,
    first_text=SPOKEN_TEXT,
    pinned_text=SPOKEN_TEXT,
)
def test_only_text_written_in_a_spoken_language_under_a_spoken_pin_is_heard(
    monkeypatch, history, session_language, code, probability, first_text, pinned_text
):
    monkeypatch.setattr(
        cloud_transcriber, "recent_language", spoken_languages.RecentLanguage()
    )
    for earlier_text, earlier_code in history:
        stub_scribe_answers(
            monkeypatch,
            detected(earlier_text, earlier_code),
            detected(earlier_text, "pt"),
        )
        cloud_transcriber.transcribe(SEGMENT)
    urlopen = stub_scribe_answers(
        monkeypatch,
        detected(first_text, code, probability),
        detected(pinned_text, code, probability),
    )
    text = cloud_transcriber.transcribe(SEGMENT, session_language)
    pins = [pin for pin in requested_languages(urlopen) if pin is not None]
    assert set(pins) <= set(SPOKEN_PINS)
    assert text == "" or is_spoken(text)
    assert cloud_transcriber.recent_language.language in (
        spoken_languages.SPOKEN_LANGUAGES
    )
