import dataclasses
import re

LATIN_LETTERS = "A-Za-zªºÀ-ÖØ-öø-ɏḀ-ỿ"
JAPANESE_LETTERS = LATIN_LETTERS + "々-〇ぁ-ヿㇰ-ㇿ㐀-䶿一-鿿豈-﫿Ａ-Ｚａ-ｚｦ-ﾟ"


@dataclasses.dataclass(frozen=True)
class SpokenLanguage:
    iso_639_1: str
    iso_639_3: str
    letter: re.Pattern

    @property
    def pinning_code(self):
        return self.iso_639_1

    def is_named_by(self, code):
        return code in (self.iso_639_1, self.iso_639_3)

    def writes(self, text):
        return all(
            self.letter.fullmatch(character)
            for character in text
            if character.isalpha()
        )


LATIN_LETTER = re.compile(f"[{LATIN_LETTERS}]")
PORTUGUESE = SpokenLanguage("pt", "por", LATIN_LETTER)
ENGLISH = SpokenLanguage("en", "eng", LATIN_LETTER)
JAPANESE = SpokenLanguage("ja", "jpn", re.compile(f"[{JAPANESE_LETTERS}]"))
SPANISH = SpokenLanguage("es", "spa", LATIN_LETTER)
SPOKEN_LANGUAGES = (PORTUGUESE, ENGLISH, JAPANESE, SPANISH)


def spoken_language(code):
    return next(
        (language for language in SPOKEN_LANGUAGES if language.is_named_by(code)), None
    )


def languages_named_by(code):
    if code is None:
        return SPOKEN_LANGUAGES
    language = spoken_language(code)
    return (language,) if language else ()


def is_written_in_one_of(text, languages):
    return any(language.writes(text) for language in languages)


@dataclasses.dataclass
class RecentLanguage:
    language: SpokenLanguage = PORTUGUESE

    @property
    def pinning_code(self):
        return self.language.pinning_code

    def remember(self, language):
        self.language = language
