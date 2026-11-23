import json

UNOBSERVED_DEFAULTS = {
    "quiet_seconds": 0.0,
    "purpose": "conversation",
    "seat": "room",
    "call_ended": False,
}


def closing_reason(message):
    try:
        return json.loads(message)["reason"]
    except (ValueError, KeyError, TypeError):
        return message


class RecordingFloorBoundaries:
    def __init__(self, boundaries):
        self.boundaries = boundaries
        self.sessions_read = []
        self.listening_read = False

    def sessions(self):
        self.sessions_read = self.boundaries.sessions()
        return self.sessions_read

    def listening(self):
        self.listening_read = self.boundaries.listening()
        return self.listening_read


class RecordingQuietBoundaries:
    def __init__(self, boundaries):
        self.boundaries = boundaries
        self.observed = dict(UNOBSERVED_DEFAULTS)
        self.reports = []
        self.close_calls = 0

    def observe(self, name, value):
        self.observed[name] = value
        return value

    def quiet_seconds(self):
        return self.observe("quiet_seconds", self.boundaries.quiet_seconds())

    def listening_purpose(self):
        return self.observe("purpose", self.boundaries.listening_purpose())

    def open_seat(self):
        return self.observe("seat", self.boundaries.open_seat())

    def discord_call_ended(self):
        return self.observe("call_ended", self.boundaries.discord_call_ended())

    def report(self, message):
        self.reports.append(closing_reason(message))
        self.boundaries.report(message)

    def close_conversation(self):
        self.close_calls += 1
        self.boundaries.close_conversation()
