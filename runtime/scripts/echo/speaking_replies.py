import contextlib
import os
import tempfile
import time
from pathlib import Path

SPEAKING_DIRECTORY = "speaking-replies"
LONGEST_REPLY_SECONDS = 310


@contextlib.contextmanager
def speaking(directory, text):
    folder = directory / SPEAKING_DIRECTORY
    folder.mkdir(mode=0o700, parents=True, exist_ok=True)
    descriptor, name = tempfile.mkstemp(dir=folder, suffix=".txt")
    with os.fdopen(descriptor, "w") as record:
        record.write(text)
    try:
        yield
    finally:
        Path(name).unlink(missing_ok=True)


def current_text(record, now):
    with contextlib.suppress(FileNotFoundError):
        if now - record.stat().st_mtime < LONGEST_REPLY_SECONDS:
            return record.read_text()
    return None


def being_spoken(directory):
    folder = directory / SPEAKING_DIRECTORY
    if not folder.exists():
        return []
    now = time.time()
    texts = (current_text(record, now) for record in folder.glob("*.txt"))
    return [text for text in texts if text is not None]
