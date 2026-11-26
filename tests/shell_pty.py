"""Drive a clean interactive shell in a pseudoterminal."""

import os
import pty
import re
import select
import signal
import time


PROMPT = re.compile(rb"__t_test_(\d+)__ ")


class InteractiveShell:
    def __init__(self, argv, env):
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.execve(argv[0], argv, env)
        self.pending = b""

    def close(self):
        try:
            os.kill(self.pid, signal.SIGHUP)
        except ProcessLookupError:
            pass
        os.close(self.fd)
        os.waitpid(self.pid, 0)

    def command(self, line):
        os.write(self.fd, (line + "\n").encode())
        return self.read_prompt()

    def read_prompt(self):
        deadline = time.monotonic() + 10
        while True:
            if match := PROMPT.search(self.pending):
                output = self.pending[: match.start()].decode(errors="replace").replace("\r", "")
                self.pending = self.pending[match.end() :]
                return output, int(match.group(1))
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select([self.fd], [], [], remaining)[0]:
                break
            data = os.read(self.fd, 8192)
            if not data:
                break
            self.pending += data
        raise AssertionError(f"No prompt from test shell; received {self.pending!r}")
