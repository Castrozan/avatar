from hypothesis import strategies as st

import wake_phrase

LETTERS = "abcdefghijklmnopqrstuvwxyzàáâãçéêíóôõúüñßÀÉÇÑ"
CYRILLIC = "абвгдезийклмнопрстуфхыэёіїєґўжцчшщюяъь"
HONORIFICS = ("san", "chan", "kun", "sama", "dono", "")
JAPANESE_SWAPS = (("l", "r"), ("v", "b"), ("w", ""), ("ts", "tz"), ("sh", "sy"), ("c", "qu"))
KANA = "あいうえおかきくけこアストルフォ"
PUNCTUATION = " ,.;:!?-"
SERVANTS = (
    "Ishtar",
    "Astolfo",
    "Anastasia",
    "Katsushika Hokusai",
    "Ruri",
    "Ozymandias",
    "Quetzalcoatl",
    "Shuten",
)
GREETING_WORDS = ("hey", "oi", "ok", "ei", "hi", "olá", "ne", "nee", "moshi", "moshimoshi")

texts = st.text(alphabet=LETTERS + KANA + CYRILLIC + PUNCTUATION, max_size=60)
words = st.text(alphabet=LETTERS + KANA + CYRILLIC, max_size=12)
vocabularies = st.dictionaries(words, st.integers(0, 5000), max_size=12) | st.none()


@st.composite
def japanese_heard(draw, name):
    heard = name.lower()
    for spelling, swap in JAPANESE_SWAPS:
        heard = heard.replace(spelling, swap) if draw(st.booleans()) else heard
    heard = "u".join(heard) if draw(st.booleans()) else heard
    return heard + draw(st.sampled_from(HONORIFICS))


@st.composite
def utterances(draw):
    name = draw(st.sampled_from(SERVANTS)).split()[0]
    name = draw(st.one_of(st.just(name), japanese_heard(name)))
    greeting = draw(st.sampled_from(GREETING_WORDS + ("",)))
    tail = draw(texts)
    pieces = [greeting, name + draw(st.sampled_from(("", ",", "."))), tail]
    return " ".join(piece for piece in pieces if piece)


@st.composite
def rosters(draw):
    servants = draw(st.lists(st.sampled_from(SERVANTS), min_size=1, max_size=5))
    panes = [f"pane-{index}" for index in range(len(servants))]
    hosts = draw(st.lists(st.sampled_from((None, "kira")), min_size=len(servants), max_size=len(servants)))
    return [
        wake_phrase.LiveSession(servant, pane, host)
        for servant, pane, host in zip(servants, panes, hosts)
    ]
