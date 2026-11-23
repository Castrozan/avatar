import contextlib
import json
import threading

from websockets.exceptions import WebSocketException
from websockets.sync.client import connect

CONTROL_SERVER_URL = "ws://127.0.0.1:8765"
RECONNECT_SECONDS = 1
IDENTIFICATION = json.dumps({"type": "identify", "role": "agent"})
FOLLOW_SPEECH = json.dumps({"type": "followSpeech"})
STOP_SPEAKING = json.dumps({"type": "stopSpeaking"})
SPEECH_ACTIVITY = "speechActivity"


def open_loopback_connection(url):
    return connect(url, proxy=None)


class ReplyLink:
    def __init__(
        self, stopping, url=CONTROL_SERVER_URL, open_connection=open_loopback_connection
    ):
        self.stopping = stopping
        self.url = url
        self.open_connection = open_connection
        self.connection = None
        self.avatar_speaking = threading.Event()

    def speaking(self):
        return self.avatar_speaking.is_set()

    def stop(self):
        connection = self.connection
        if connection is None:
            return
        with contextlib.suppress(WebSocketException):
            connection.send(STOP_SPEAKING)

    def follow(self):
        while not self.stopping.is_set():
            with contextlib.suppress(OSError, WebSocketException):
                self.follow_connection()
            self.connection = None
            self.avatar_speaking.clear()
            self.stopping.wait(RECONNECT_SECONDS)

    def follow_connection(self):
        with self.open_connection(self.url) as connection:
            connection.send(IDENTIFICATION)
            connection.send(FOLLOW_SPEECH)
            self.connection = connection
            for message in connection:
                self.hear_notice(json.loads(message))

    def hear_notice(self, notice):
        if notice.get("type") != SPEECH_ACTIVITY:
            return
        if notice["speaking"]:
            self.avatar_speaking.set()
        else:
            self.avatar_speaking.clear()

    def start(self):
        threading.Thread(target=self.follow, daemon=True).start()
        return self
