import io
import json
import re
from unittest.mock import MagicMock, Mock

from transcription import cloud_transcriber

SEGMENT = b"\x00\x00" * 1600
LANGUAGE_FIELD = re.compile(rb'name="language_code"\r\n\r\n(.*?)\r\n')


def scribe_response(payload):
    response = MagicMock()
    response.__enter__.return_value = io.BytesIO(json.dumps(payload).encode())
    return response


def stub_scribe_answers(monkeypatch, *answers):
    outcomes = [
        answer if isinstance(answer, Exception) else scribe_response(answer)
        for answer in answers
    ]
    urlopen = Mock(side_effect=outcomes)
    monkeypatch.setattr(cloud_transcriber.urllib.request, "urlopen", urlopen)
    monkeypatch.setenv("ELEVENLABS_API_KEY", "test-key")
    return urlopen


def detected(text, language_code, language_probability=0.99):
    return {
        "text": text,
        "language_code": language_code,
        "language_probability": language_probability,
    }


def requested_languages(urlopen):
    fields = [
        LANGUAGE_FIELD.search(call.args[0].data) for call in urlopen.call_args_list
    ]
    return [field and field.group(1).decode() for field in fields]
