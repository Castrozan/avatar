import difflib

import name_hearing
from hypothesis import given, settings
from strategies import texts, words
from transcription import ordinary_words
from transcription.spoken_names import WORD

EXAMPLES = settings(max_examples=300, deadline=None)


def code_point_end(text, byte_end):
    return len(text.encode()[:byte_end].decode())


@EXAMPLES
@given(texts)
def test_folded_matches_python(rust, text):
    assert rust.call("folded", text=text) == ordinary_words.folded(text)


@EXAMPLES
@given(words)
def test_sound_key_matches_python(rust, word):
    assert rust.call("sound_key", word=word) == name_hearing.sound_key(word)


@EXAMPLES
@given(texts, texts)
def test_ratio_matches_python(rust, left, right):
    expected = difflib.SequenceMatcher(None, left, right).ratio()
    assert rust.call("ratio", left=left, right=right) == expected


@EXAMPLES
@given(texts)
def test_word_matches_match_python(rust, text):
    expected = [(found.group(), found.end()) for found in WORD.finditer(text)]
    found = rust.call("word_matches", text=text)
    actual = [(item["text"], code_point_end(text, item["end"])) for item in found]
    assert actual == expected
