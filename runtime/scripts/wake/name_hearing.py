import difflib
import re

from transcription import ordinary_words

SPELLED_ALIKE = 2.0
SHORT_KEY_LENGTH = 4
MEDIUM_KEY_LENGTH = 5
SHORT_NAME_SIMILARITY = 1.0
MEDIUM_NAME_SIMILARITY = 0.8
LONG_NAME_SIMILARITY = 0.72
SHORTEST_JOINED_WORD = 3
TOLERATED_MISSING_SOUNDS = 1
LONGEST_JAPANESE_WINDOW = 2
VOWELS = "aeiou"
REPEATED_SOUND = re.compile(r"(.)\1+")
BRAZILIAN_INITIAL_R = re.compile(r"^r")
SOUND_RULES = (
    ("sch", "S"),
    ("sh", "S"),
    ("ch", "S"),
    ("x", "S"),
    ("ph", "f"),
    ("ce", "se"),
    ("ci", "si"),
    ("c", "k"),
    ("q", "k"),
    ("y", "i"),
    ("z", "s"),
    ("h", ""),
)
JAPANESE_SPELLINGS = (
    ("qu", "k"),
    ("sy", "sh"),
    ("tz", "z"),
    ("ts", "z"),
    ("l", "r"),
    ("v", "b"),
    ("w", ""),
)
EPENTHETIC_VOWEL = re.compile(r"(?<=[^aeiou])u(?=[^aeiou]|$)|(?<=[td])o(?=[^aeiou]|$)")
HONORIFIC_SUFFIX = re.compile(r"(?<=\w{3})(?:san|chan|kun|sama|dono)$")


def sound_key(word):
    word = BRAZILIAN_INITIAL_R.sub("h", word)
    for spelling, sound in SOUND_RULES:
        word = word.replace(spelling, sound)
    return REPEATED_SOUND.sub(r"\1", word)


def japanese_sound_key(word):
    for spelling, sound in JAPANESE_SPELLINGS:
        word = word.replace(spelling, sound)
    key = EPENTHETIC_VOWEL.sub("", sound_key(word)).replace("S", "s")
    return REPEATED_SOUND.sub(r"\1", key)


def ordinary_vocabulary():
    return ordinary_words.frequency_ranks()


def gains_a_leading_consonant(heard_key, name_key):
    return name_key[:1] in VOWELS and heard_key[:1] not in VOWELS


def required_similarity(heard_key, name_key):
    if len(name_key) <= SHORT_KEY_LENGTH:
        return SHORT_NAME_SIMILARITY
    if len(name_key) == MEDIUM_KEY_LENGTH or gains_a_leading_consonant(
        heard_key, name_key
    ):
        return MEDIUM_NAME_SIMILARITY
    return LONG_NAME_SIMILARITY


def joins_fragments(words):
    return len(words) > 1 and any(len(word) < SHORTEST_JOINED_WORD for word in words)


def holds_an_ordinary_word(words, vocabulary):
    return vocabulary is None or any(word in vocabulary for word in words)


def sounds_alike(heard_key, name_key):
    return heard_key == name_key and len(name_key) > SHORT_KEY_LENGTH


def sounded_similarity(words, heard_key, name_key, vocabulary):
    if holds_an_ordinary_word(words, vocabulary) and not sounds_alike(
        heard_key, name_key
    ):
        return None
    similarity = difflib.SequenceMatcher(None, heard_key, name_key).ratio()
    return (
        similarity if similarity >= required_similarity(heard_key, name_key) else None
    )


def heard_similarity(words, name, vocabulary):
    if joins_fragments(words):
        return None
    heard = "".join(words)
    if heard == name:
        return SPELLED_ALIKE
    return sounded_similarity(words, sound_key(heard), sound_key(name), vocabulary)


def sounds_like_one_japanese_name(words):
    return not joins_fragments(words) and (
        len(set(words)) == len(words) <= LONGEST_JAPANESE_WINDOW
    )


def keys_comparable(heard_key, name_key):
    if len(name_key) <= SHORT_KEY_LENGTH:
        return False
    if len(heard_key) < len(name_key) - TOLERATED_MISSING_SOUNDS:
        return False
    return (heard_key[:1] in VOWELS) == (name_key[:1] in VOWELS)


def japanese_similarity(words, name, vocabulary):
    if not sounds_like_one_japanese_name(words):
        return None
    heard_key = japanese_sound_key(HONORIFIC_SUFFIX.sub("", "".join(words)))
    name_key = japanese_sound_key(name)
    if not keys_comparable(heard_key, name_key):
        return None
    return sounded_similarity(words, heard_key, name_key, vocabulary)


def names_heard_by(similarity_of, windows, names, vocabulary):
    return [
        (similarity, name, len(words))
        for words in windows
        for name in names
        if (similarity := similarity_of(words, name, vocabulary)) is not None
    ]


def heard_name(windows, names, vocabulary):
    for similarity_of in (heard_similarity, japanese_similarity):
        heard = names_heard_by(similarity_of, windows, names, vocabulary)
        if heard:
            return clearest_name(heard)
    return None


def clearest_name(heard):
    clearest = max(similarity for similarity, _, _ in heard)
    leaders = [
        (name, size) for similarity, name, size in heard if similarity == clearest
    ]
    if len({name for name, _ in leaders}) > 1:
        return None
    return min(leaders, key=lambda leader: leader[1])
