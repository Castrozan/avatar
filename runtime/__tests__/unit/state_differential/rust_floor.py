import json
import os
import subprocess


class RustFloor:
    def __init__(self, binary, vocabulary_file=None):
        environment = dict(os.environ)
        environment.pop("AVATAR_ORDINARY_WORDS", None)
        self.process = subprocess.Popen(
            [binary],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            env=environment,
        )

    def call(self, operation, **fields):
        request = json.dumps({"op": operation, **fields}, ensure_ascii=False)
        self.process.stdin.write(request + "\n")
        self.process.stdin.flush()
        reply = json.loads(self.process.stdout.readline())
        if "error" in reply:
            raise AssertionError(f"{operation}: {reply['error']}")
        return reply["ok"]

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=10)
        self.process.stdout.close()
