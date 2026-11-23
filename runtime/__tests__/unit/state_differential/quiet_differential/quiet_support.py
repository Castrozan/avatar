from conversation_seat import Seat
from hypothesis import strategies as st
from listener_session import ListenerPurpose

SECONDS = st.integers(0, 40_000).map(lambda tenths: tenths / 10)


class RecordingBoundaries:
    def __init__(self, quiet_seconds, purpose, seat, call_ended):
        self.seconds = quiet_seconds
        self.purpose = purpose
        self.seat = seat
        self.ended = call_ended
        self.reports = []
        self.close_calls = 0

    def quiet_seconds(self):
        return self.seconds

    def listening_purpose(self):
        return ListenerPurpose(self.purpose)

    def open_seat(self):
        return Seat(self.seat)

    def discord_call_ended(self):
        return self.ended

    def report(self, message):
        self.reports.append(message)

    def close_conversation(self):
        self.close_calls += 1

    def record(self):
        return {"reports": self.reports, "close_calls": self.close_calls}
