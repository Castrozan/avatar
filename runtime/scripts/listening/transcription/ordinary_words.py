import functools
import os
import pathlib
import unicodedata

VOCABULARY_ENVIRONMENT_VARIABLE = "AVATAR_ORDINARY_WORDS"
CYRILLIC_IN_LATIN = str.maketrans(
    {
        **dict(
            zip(
                "абвгдезийклмнопрстуфхыэёіїєґў",
                "abvgdeziiklmnoprstufhieeiiegu",
                strict=True,
            )
        ),
        **dict(zip("жцчшщюя", ("zh", "ts", "ch", "sh", "sh", "iu", "ia"), strict=True)),
        "ъ": "",
        "ь": "",
    }
)


def folded(text):
    latin = text.casefold().translate(CYRILLIC_IN_LATIN)
    decomposed = unicodedata.normalize("NFD", latin)
    return "".join(
        character for character in decomposed if unicodedata.category(character) != "Mn"
    )


@functools.cache
def ranks_in(paths):
    if not paths:
        return None
    ranks = {}
    try:
        for path in paths.split(os.pathsep):
            lines = pathlib.Path(path).read_text().splitlines()
            for rank, line in enumerate(line for line in lines if line.strip()):
                word = folded(line.split()[0])
                ranks[word] = min(rank, ranks.get(word, rank))
    except OSError:
        return None
    return ranks


def frequency_ranks():
    return ranks_in(os.environ.get(VOCABULARY_ENVIRONMENT_VARIABLE, ""))
