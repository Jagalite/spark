#!/usr/bin/env python3
"""Optional Unix PTY regression, no dependencies or persistent captures.

Build the gallery, then run from the repository root:
  python3 apps/gallery/tests/terminal.py target/debug/spark-gallery
This checks terminal lifecycle, not native-font visual qualification.
"""
import errno
import fcntl
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import sys
import termios
import time


def check(binary, export, tiny):
    master, slave = pty.openpty()
    process = None
    captured = bytearray()
    try:
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 42, 140, 0, 0))
        original = termios.tcgetattr(slave)
        process = subprocess.Popen(
            [str(binary), "--paused", "--clip-nanos", "750000000"],
            stdin=slave, stdout=slave, stderr=slave, close_fds=True,
        )

        def drain(seconds):
            deadline = time.monotonic() + seconds
            while time.monotonic() < deadline:
                if select.select([master], [], [], max(0, deadline - time.monotonic()))[0]:
                    try:
                        captured.extend(os.read(master, 65536))
                    except OSError as error:
                        if error.errno == errno.EIO:
                            return
                        raise

        def send(keys):
            os.write(master, keys)
            drain(0.08)

        # Wait for real terminal setup before sending input; bound startup to 5s.
        deadline = time.monotonic() + 5
        while b"\x1b[?1049h" not in captured and time.monotonic() < deadline:
            drain(0.05)
        assert b"\x1b[?1049h" in captured, "gallery never entered alternate screen"
        for _ in range(4):
            send(b"i")
        send(b"i")
        send(b"]")
        send(b"[")
        send(b"\x1b[6~")  # PageDown
        send(b"\x1b[H")  # Home
        if tiny:
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 3, 7, 0, 0))
            os.kill(process.pid, signal.SIGWINCH)
            drain(0.1)
            send(b"i")  # Back to ordinary tiny view.
        send(b"e" if export else b"q")
        process.wait(timeout=5)
        drain(0.05)
        assert process.returncode == 0, "gallery failed"
        assert termios.tcgetattr(slave) == original, "raw terminal settings leaked"
        leave = b"\x1b[?1049l"
        assert leave in captured, "alternate screen not restored"
        tail = captured[captured.rindex(leave) + len(leave):]
        marker = b"// Spark playground:"
        if export:
            assert marker in tail, "source missing after terminal restoration"
            source = tail[tail.index(marker):]
            assert b"\x1b" not in source, "terminal escape leaked into exported Rust"
            assert b"ClipTime::from_nanos(750000000)" in source, "paused seek drifted"
            assert b"playback.pause(host)" in source, "pause state lost"
            if tiny:
                assert b"terminal 7x3" in source, "export ignored resize"
            else:
                assert b"inspector=true" in source, "inspector unexpectedly closed"
        else:
            assert marker not in tail, "quit unexpectedly exported source"
        print(f"PTY {'export' if export else 'quit'} / {'tiny' if tiny else 'normal'}: passed")
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: terminal.py /path/to/spark-gallery")
    executable = Path(sys.argv[1]).resolve(strict=True)
    for export in (False, True):
        for tiny in (False, True):
            check(executable, export, tiny)
