import pathlib
import re
import string

import pytest
from hypothesis import given
from hypothesis import strategies as st

from name_hearing import japanese_sound_key
from transcription.ordinary_words import folded
from wake_phrase import (
    JAPANESE_GREETINGS,
    LiveSession,
    addressed_session,
    named_in_turn,
    sessions_heard_anywhere,
    spoken_names_of,
)

ROSTER = pathlib.Path(__file__).parents[4] / "cast" / "servant-characters.txt"
SERVANTS = [line.split(" | ")[0] for line in ROSTER.read_text().splitlines() if line]
ORDINARY_WORDS_FILE = pathlib.Path(__file__).parents[1] / "ordinary-words.txt"
CYRILLIC_LETTERS = "абвгдеёжзийклмнопрстуфхцчшщъыьэюяґєіїў"
SPOKEN_UNITS = re.compile(r"sch|sh|ch|th|ph|zh|ts|tz|qu|[aeiouy]|.")
VOWEL_UNITS = "aeiouy"
CALLED_IN_JAPANESE = [
    LiveSession(servant, f"wW:pJ{index}")
    for index, servant in enumerate(
        ["Ishtar", "Ereshkigal", "Ozymandias", "Quetzalcoatl", "Astolfo"]
        + ["Okita Souji", "Anastasia", "Tamamo Cat"]
    )
]
EVERYDAY_JAPANESE = (
    "kyou wa ii tenki desu chotto matte kudasai nani shiteru arigatou gozaimasu "
    "sumimasen daijoubu oishii kore mite doko iku gohan tabeta mada owatte nai "
    "sugoi yappari shigoto kaimono denwa ashita ame nemui samui atsui eiga "
    "ongaku densha wakarimashita ganbatte hontou genki ohayou oyasumi sou yo"
).split()


@pytest.fixture(autouse=True)
def ordinary_vocabulary(monkeypatch):
    monkeypatch.setenv("AVATAR_ORDINARY_WORDS", str(ORDINARY_WORDS_FILE))


def closes_a_syllable(unit, following):
    return unit not in VOWEL_UNITS and unit != "n" and following[:1] not in VOWEL_UNITS


def said_the_japanese_way(name):
    units = SPOKEN_UNITS.findall(name)
    spoken = []
    for unit, following in zip(units, [*units[1:], ""], strict=True):
        spoken.append(unit)
        if closes_a_syllable(unit, following):
            spoken.append("o" if unit in "td" else "u")
    return "".join(spoken).replace("l", "r")


@given(heard=st.text(alphabet=CYRILLIC_LETTERS + CYRILLIC_LETTERS.upper()))
def test_a_transcript_heard_in_cyrillic_folds_into_latin_letters(heard):
    assert set(folded(heard)) <= set(string.ascii_lowercase)


@given(servant=st.sampled_from(SERVANTS))
def test_japanese_vowels_and_r_for_l_keep_the_sound_of_every_name(servant):
    for name in spoken_names_of(servant):
        said = said_the_japanese_way(name)
        assert japanese_sound_key(said) == japanese_sound_key(name), said


@pytest.mark.parametrize(
    ("heard", "servant"),
    [
        ("Хэй, Истер.", "Ishtar"),
        ("Ne, issu tardu.", "Ishtar"),
        ("Eresyuki garu, onegaigaruno!", "Ereshkigal"),
        ("Nene, ojimandias, kore mite.", "Ozymandias"),
        ("Ketzarukwatoru, Genki.", "Quetzalcoatl"),
        ("Hey, Asturfall.", "Astolfo"),
        ("Oi okitasan okiteru.", "Okita Souji"),
    ],
)
def test_a_name_said_the_japanese_way_reaches_its_servant(heard, servant):
    assert named_in_turn(heard, CALLED_IN_JAPANESE).session.servant == servant


@given(greeting=st.sampled_from(sorted(JAPANESE_GREETINGS)))
def test_a_japanese_greeting_opens_a_wake_and_leaves_the_message(greeting):
    match = addressed_session(f"{greeting}, Tamamo, kore mite", CALLED_IN_JAPANESE)
    assert match.session.servant == "Tamamo Cat"
    assert match.message == "kore mite"


@given(
    greeting=st.sampled_from(sorted(JAPANESE_GREETINGS)),
    words=st.lists(st.sampled_from(EVERYDAY_JAPANESE), min_size=1, max_size=6),
)
def test_ordinary_japanese_after_a_japanese_greeting_wakes_nobody(greeting, words):
    assert named_in_turn(" ".join([greeting, *words]), CALLED_IN_JAPANESE) is None


@pytest.mark.parametrize(
    ("heard", "servant"),
    [
        ("Nista, Mamo!", "Ishtar"),
        ("Hããã, vaiii-- usa a skill", "Abigail Williams"),
        ("e comer a minhaaa, minha janta", "Minamoto-no-Raikou"),
        ("Ky Lastia, eu quero", "Katsushika Hokusai"),
        ("Archon", "Arjuna"),
        ("ne, tenki ii ne", "Enkidu"),
        ("ne, ii ne", "Li Shuwen"),
        ("ne, kudasai", "Katsushika Hokusai"),
    ],
)
def test_a_word_only_loosely_like_a_name_names_nobody(heard, servant):
    assert sessions_heard_anywhere(heard, [LiveSession(servant, "wW:pX")]) == []
