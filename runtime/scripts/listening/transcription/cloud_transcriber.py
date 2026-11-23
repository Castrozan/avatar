import functools
import json
import math
import os
import re
import sys
import urllib.request
import uuid
from pathlib import Path

from transcription import ordinary_words, spoken_languages, spoken_names, transcriber

SCRIBE_URL = "https://api.elevenlabs.io/v1/speech-to-text"
SCRIBE_MODEL = "scribe_v2"
API_KEY_FILE = Path.home() / ".secrets" / "elevenlabs-api-key"
REQUEST_TIMEOUT_SECONDS = 15
KEYTERMS_BEFORE_MINIMUM_BILLING = 100
ROSTER_SEPARATOR = " | "
PARENTHESIZED = re.compile(r"\s*\([^)]*\)")
RECOGNIZED_TERMS = (
    "ElevenLabs",
    "Hex",
    "NixOS",
    "chise",
    "Claude",
    "Hyprland",
    "dotfiles",
    "rebuild",
    "Lucas",
    "Lulu",
)
recent_language = spoken_languages.RecentLanguage()


def api_key():
    return os.environ.get("ELEVENLABS_API_KEY") or API_KEY_FILE.read_text().strip()


@functools.cache
def servant_names(roster_path):
    try:
        lines = Path(roster_path).read_text().splitlines()
    except (OSError, ValueError):
        return ()
    names = (
        PARENTHESIZED.sub("", line.split(ROSTER_SEPARATOR)[0]).strip() for line in lines
    )
    return tuple(dict.fromkeys(name for name in names if name))


def familiarity(word, frequency_ranks):
    rank = frequency_ranks.get(ordinary_words.folded(word))
    return -math.inf if rank is None else -rank


def recognized_terms():
    roster_path = os.environ.get("AVATAR_SERVANT_ROSTER")
    if not roster_path:
        return RECOGNIZED_TERMS
    frequency_ranks = ordinary_words.frequency_ranks() or {}
    answered = dict.fromkeys(
        name
        for servant in servant_names(roster_path)
        for name in spoken_names.answered_names(servant)
    )
    names = [name for name in answered if name not in RECOGNIZED_TERMS]
    names.sort(key=lambda name: familiarity(name, frequency_ranks))
    return RECOGNIZED_TERMS + tuple(
        names[: KEYTERMS_BEFORE_MINIMUM_BILLING - len(RECOGNIZED_TERMS)]
    )


def scribe_fields(language):
    fields = [("model_id", SCRIBE_MODEL), ("tag_audio_events", "false")]
    if language:
        fields.append(("language_code", language))
    return fields + [("keyterms", term) for term in recognized_terms()]


def multipart_body(boundary, fields, wav):
    parts = [
        f'--{boundary}\r\nContent-Disposition: form-data; name="{name}"\r\n\r\n'
        f"{value}\r\n".encode()
        for name, value in fields
    ]
    parts.append(
        f'--{boundary}\r\nContent-Disposition: form-data; name="file"; '
        f'filename="segment.wav"\r\nContent-Type: audio/wav\r\n\r\n'.encode()
        + wav
        + f"\r\n--{boundary}--\r\n".encode()
    )
    return b"".join(parts)


def scribe_answer(pcm, language):
    boundary = uuid.uuid4().hex
    request = urllib.request.Request(
        SCRIBE_URL,
        data=multipart_body(
            boundary, scribe_fields(language), transcriber.wav_bytes(pcm)
        ),
        headers={
            "xi-api-key": api_key(),
            "Content-Type": f"multipart/form-data; boundary={boundary}",
        },
    )
    with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_SECONDS) as response:
        return json.load(response)


def heard_text(text, languages, heard_as):
    if spoken_languages.is_written_in_one_of(text, languages):
        return text
    print(
        f"Dropped a segment heard as {heard_as}: not a spoken language", file=sys.stderr
    )
    return ""


def scribe_heard_text(answer, languages):
    heard_as = f"{answer.get('language_code')} ({answer.get('language_probability')})"
    return heard_text(answer["text"], languages, heard_as)


def pinned_text(pcm, language):
    spoken_language = spoken_languages.spoken_language(language)
    if spoken_language is None:
        return heard_text("", (), f"the session language {language}")
    answer = scribe_answer(pcm, spoken_language.pinning_code)
    return scribe_heard_text(answer, (spoken_language,))


def repinned_text(pcm, answer):
    pinning_code = recent_language.pinning_code
    print(
        f"Scribe detected {answer['language_code']} "
        f"({answer.get('language_probability')}), pinning {pinning_code}",
        file=sys.stderr,
    )
    return pinned_text(pcm, pinning_code)


def detected_text(pcm):
    answer = scribe_answer(pcm, None)
    detected_code = answer.get("language_code")
    languages = spoken_languages.languages_named_by(detected_code)
    if not languages:
        return repinned_text(pcm, answer)
    text = scribe_heard_text(answer, languages)
    if text and detected_code is not None:
        recent_language.remember(languages[0])
    return text


def spoken_text(pcm, language):
    return pinned_text(pcm, language) if language else detected_text(pcm)


def transcribe(pcm, language=None):
    return transcriber.clean_transcript(spoken_text(pcm, language))


def local_text(pcm, language):
    text = transcriber.transcribe(pcm, language)
    languages = spoken_languages.languages_named_by(language)
    return heard_text(text, languages, f"Hex in {language}")


def transcribe_with_local_fallback(pcm, language=None):
    try:
        return transcribe(pcm, language)
    except (OSError, ValueError, KeyError) as error:
        print(f"Cloud transcription failed, using Hex: {error}", file=sys.stderr)
        return local_text(pcm, language)
