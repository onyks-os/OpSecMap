# Copyright (c) 2026 onyks-os
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Checks password entry in a real terminal, which `cargo test` cannot do.

The binary runs in a pseudo-terminal, so it reads the password from a tty with
echo off, exactly as for a user. OPSECMAP_PASSWORD is removed. The script
checks that save asks twice and open once, that no password is echoed, that
echo comes back afterwards, that a wrong password or two different ones are
refused and leave the map unchanged, and that the file holds no map text.
Linux and macOS only; the map and the passwords are fictional.

    cargo build && python3 tests/manual/terminal.py [path/to/opsecmap]
"""

import os
import pty
import select
import shutil
import sys
import tempfile
import termios
import time

BIN = sys.argv[1] if len(sys.argv) > 1 else os.path.join("target", "debug", "opsecmap")
PASSWORD = "correct horse battery staple"
ENV = {key: value for key, value in os.environ.items() if key != "OPSECMAP_PASSWORD"}


class Session:
    """One run of the binary in a pseudo-terminal."""

    def __init__(self):
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.execve(BIN, [BIN], ENV)
        self.out = b""
        self.expect("> ")

    def expect(self, text, timeout=20):
        """Reads until `text` appears and returns everything up to it."""
        wanted = text.encode()
        end = time.time() + timeout
        while wanted not in self.out:
            left = end - time.time()
            if left <= 0:
                raise SystemExit(f"TIMEOUT waiting for {text!r}; got:\n{self.decoded()}")
            ready, _, _ = select.select([self.fd], [], [], left)
            if not ready:
                continue
            try:
                chunk = os.read(self.fd, 4096)
            except OSError:
                chunk = b""
            if not chunk:
                raise SystemExit(f"EOF waiting for {text!r}; got:\n{self.decoded()}")
            self.out += chunk
        cut = self.out.index(wanted) + len(wanted)
        seen, self.out = self.out[:cut], self.out[cut:]
        return seen.decode(errors="replace")

    def decoded(self):
        return self.out.decode(errors="replace")

    def send(self, line):
        os.write(self.fd, (line + "\n").encode())

    def run(self, line):
        """Sends a command and returns its output, up to the next prompt."""
        self.send(line)
        return self.expect("> ")

    def send_secret(self, secret, timeout=5):
        """Types `secret` once the program has turned echo off, as a person
        would after reading the prompt; typing earlier would be echoed by the
        terminal itself, before the program can hide it."""
        end = time.time() + timeout
        while termios.tcgetattr(self.fd)[3] & termios.ECHO:
            if time.time() > end:
                raise SystemExit("FAIL echo was never turned off for the password")
            time.sleep(0.01)
        self.send(secret)

    def close(self):
        """Sends `quit` and returns the exit code."""
        self.send("quit")
        try:
            while select.select([self.fd], [], [], 5)[0] and os.read(self.fd, 4096):
                pass
        except OSError:
            pass
        _, status = os.waitpid(self.pid, 0)
        return os.waitstatus_to_exitcode(status)


def map_lines(output):
    """The lines of a `list` output, without the terminal's carriage returns."""
    lines = output.replace("\r", "").splitlines()
    return [line for line in lines if line.startswith(("resource ", "route ", "coverage "))]


def check(condition, message, transcript):
    print(("OK   " if condition else "FAIL ") + message)
    if not condition:
        print(transcript)
        sys.exit(1)


def save_asks_twice_and_hides_the_password(path):
    session = Session()
    for line in [
        "resource email Test email",
        "resource phone Test phone",
        "route email-login email phone Email login",
    ]:
        session.run(line)
    listed = session.run("list")
    session.send(f"save {path}")
    session.expect("New password: ")
    session.send_secret(PASSWORD)
    first = session.expect("Repeat password: ")
    check(PASSWORD not in first, "the first password is not echoed", first)
    session.send_secret(PASSWORD)
    second = session.expect("> ")
    check(PASSWORD not in second, "the repeated password is not echoed", second)
    check("(encrypted)" in second, "save confirms (encrypted)", second)
    shown = session.run("list")
    check("list" in shown, "echo is back on: the typed command is shown", shown)
    code = session.close()
    check(code == 0, f"the session exits 0 (got {code})", "")
    return listed


def open_asks_once_and_gives_the_same_map(path, listed):
    session = Session()
    session.send(f"open {path}")
    session.expect("Password: ")
    session.send_secret(PASSWORD)
    opened = session.expect("> ")
    check(PASSWORD not in opened and "(encrypted)" in opened, "open asks once, hides it, confirms", opened)
    relisted = session.run("list")
    same = map_lines(listed) == map_lines(relisted) and map_lines(listed)
    check(same, f"list after reopening equals the original ({len(map_lines(listed))} lines)", relisted)
    session.close()


def wrong_password_keeps_the_previous_map(path):
    session = Session()
    session.run("resource extra Test extra")
    session.send(f"open {path}")
    session.expect("Password: ")
    session.send_secret("wrong password")
    refused = session.expect("> ")
    check("wrong password, or the map file is damaged or was modified" in refused, "a wrong password is refused", refused)
    check("wrong password\r\n" not in refused.split("Password: ")[-1][:20], "the wrong password is not echoed", refused)
    kept = session.run("list")
    check(map_lines(kept) == ["resource extra Test extra"], "the previous map is kept", kept)
    code = session.close()
    check(code == 1, f"a failed command makes the exit code 1 (got {code})", "")


def different_passwords_write_nothing(path):
    session = Session()
    session.run("resource phone Test phone")
    session.send(f"save {path}")
    session.expect("New password: ")
    session.send_secret("one")
    session.expect("Repeat password: ")
    session.send_secret("two")
    refused = session.expect("> ")
    written = os.path.exists(path)
    check("the two passwords do not match" in refused and not written, "different passwords are refused, nothing is written", refused)
    session.close()


def file_holds_no_map_text_or_password(path):
    with open(path, "rb") as file:
        data = file.read()
    check(data[:8] == b"OPSECMAP", f"the first 8 bytes are OPSECMAP ({data[:8]!r})", "")
    hidden = b"Test" not in data and b"email" not in data and PASSWORD.encode() not in data
    check(hidden, f"the {len(data)} bytes contain no map text and no password", "")


def main():
    work = tempfile.mkdtemp(prefix="opsecmap-terminal-")
    try:
        path = os.path.join(work, "map.opsecmap")
        listed = save_asks_twice_and_hides_the_password(path)
        open_asks_once_and_gives_the_same_map(path, listed)
        wrong_password_keeps_the_previous_map(path)
        different_passwords_write_nothing(os.path.join(work, "other.opsecmap"))
        file_holds_no_map_text_or_password(path)
    finally:
        shutil.rmtree(work)
    print("all manual checks passed")


if __name__ == "__main__":
    main()
