import json
import queue
import threading

import pytest
from websockets.sync.server import serve

from barge_in import reply_link

WAIT_SECONDS = 5


class ControlServer:
    def __init__(self):
        self.received = queue.Queue()
        self.notices = queue.Queue()
        self.server = serve(self.answer, "127.0.0.1", 0)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.url = f"ws://127.0.0.1:{self.server.socket.getsockname()[1]}"

    def answer(self, connection):
        threading.Thread(
            target=self.forward_notices, args=(connection,), daemon=True
        ).start()
        for message in connection:
            self.received.put(json.loads(message))

    def forward_notices(self, connection):
        while (notice := self.notices.get()) is not None:
            connection.send(json.dumps(notice))
        connection.close()

    def next_message(self):
        return self.received.get(timeout=WAIT_SECONDS)


class SpeakingWatch:
    def __init__(self, link):
        self.link = link
        self.changed = threading.Event()
        self.link.avatar_speaking = self

    def set(self):
        self.value = True
        self.changed.set()

    def clear(self):
        self.value = False
        self.changed.set()

    def is_set(self):
        return getattr(self, "value", False)

    def wait_for(self, value):
        while self.is_set() != value:
            assert self.changed.wait(WAIT_SECONDS)
            self.changed.clear()


@pytest.fixture
def server():
    control = ControlServer()
    yield control
    control.notices.put(None)
    control.server.shutdown()


def following(server):
    stopping = threading.Event()
    link = reply_link.ReplyLink(stopping, url=server.url)
    watch = SpeakingWatch(link)
    link.start()
    assert server.next_message() == {"type": "identify", "role": "agent"}
    assert server.next_message() == {"type": "followSpeech"}
    return link, watch, stopping


def test_the_link_knows_when_the_avatar_speaks_and_stops_it_over_one_connection(
    server,
):
    link, watch, stopping = following(server)
    server.notices.put({"type": "speechActivity", "speaking": True})
    watch.wait_for(True)
    link.stop()
    assert server.next_message() == {"type": "stopSpeaking"}
    server.notices.put({"type": "speechActivity", "speaking": False})
    watch.wait_for(False)
    stopping.set()


def test_other_server_messages_leave_the_speaking_state_alone(server):
    link, watch, stopping = following(server)
    server.notices.put({"type": "speechActivity", "speaking": True})
    server.notices.put({"type": "characterChanged", "name": "Voyager"})
    server.notices.put({"type": "stopSpeakingAck", "status": "stopped"})
    watch.wait_for(True)
    assert link.speaking()
    stopping.set()


def test_a_lost_server_means_nothing_is_speaking(server):
    link, watch, stopping = following(server)
    server.notices.put({"type": "speechActivity", "speaking": True})
    watch.wait_for(True)
    server.notices.put(None)
    watch.wait_for(False)
    stopping.set()


def test_a_link_never_connected_sends_nothing_when_asked_to_stop():
    link = reply_link.ReplyLink(threading.Event())
    link.stop()
    assert not link.speaking()
