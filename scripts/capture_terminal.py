#!/usr/bin/env python3
"""Run the real gallery in a Linux PTY and retain an asciicast-v2 event stream.
This is a terminal-protocol smoke capture, not a terminal-emulator/font review.
"""
import codecs
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import termios
import time

def fresh_source_hash(root):
    """Regenerate consumer cells; never trust a previous capture's source stamp."""
    subprocess.run(
        ['cargo', 'run', '-p', 'xtask', '--locked', '--', 'render', 'segmented-comet'],
        cwd=root, check=True, stdout=subprocess.PIPE, text=True,
    )
    source = json.loads((root / 'previews/segmented-comet.cells.json').read_text())['source_sha256']
    if not isinstance(source, str) or len(source) != 64 or any(c not in '0123456789abcdef' for c in source):
        raise RuntimeError('capture has an invalid source fingerprint')
    return source


def gallery_executable(messages):
    """Use Cargo's artifact location, including custom target directories."""
    executables = set()
    for line in messages.splitlines():
        message = json.loads(line)
        target = message.get('target', {})
        if (message.get('reason') == 'compiler-artifact'
                and target.get('name') == 'spark-gallery'
                and 'bin' in target.get('kind', []) and message.get('executable')):
            executables.add(Path(message['executable']))
    if len(executables) != 1:
        raise RuntimeError('cargo did not report exactly one gallery executable')
    return executables.pop()


def prepare_capture(root):
    source = fresh_source_hash(root)
    build = subprocess.run(
        ['cargo', 'build', '-p', 'spark-gallery', '--bin', 'spark-gallery',
         '--locked', '--message-format=json'],
        cwd=root, check=True, stdout=subprocess.PIPE, text=True,
    )
    binary = gallery_executable(build.stdout)
    if not binary.is_absolute():
        binary = root / binary
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    verify_capture_inputs(root, binary, source, digest)
    return binary, source, digest


def verify_capture_inputs(root, binary, source, digest):
    if fresh_source_hash(root) != source:
        raise RuntimeError('source changed during capture preparation or recording; rerun')
    if hashlib.sha256(binary.read_bytes()).hexdigest() != digest:
        raise RuntimeError('gallery binary changed during recording; rerun')


class OutputRecorder:
    """Preserve UTF-8 sequences that straddle arbitrary PTY read boundaries."""
    def __init__(self, events, start):
        self.events = events
        self.start = start
        self.decoder = codecs.getincrementaldecoder('utf-8')(errors='replace')

    def record(self, chunk, final=False):
        output = self.decoder.decode(chunk, final=final)
        if output:
            self.events.append([round(time.monotonic() - self.start, 6), 'o', output])


def main():
    root = Path(__file__).resolve().parents[1]
    binary, source_sha256, binary_sha256 = prepare_capture(root)
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 32, 100, 0, 0))
    before = termios.tcgetattr(slave)
    process = subprocess.Popen([str(binary)], stdin=slave, stdout=slave, stderr=slave,
                               cwd=root, env={**os.environ, 'TERM':'xterm-256color', 'COLORTERM':'truecolor'})
    start = time.monotonic()
    keys = [(0.5,b'2'),(0.9,b'-'),(1.2,b'+'),(1.5,b'u'),(1.8,b'u'),
            (2.1,b'3'),(2.5,b'v'),(2.9,b'n'),(3.3,b'4'),(3.8,b'5'),
            (4.3,b'6'),(4.6,b' '),(4.9,b']'),(5.2,b'['),(5.5,b'.'),(5.8,b','),
            (6.1,b' '),(6.5,b'r'),(7.0,b'c'),(7.4,b'a'),(7.8,b'm'),(8.2,b'm'),
            (8.6,b's'),(9.0,b's'),(9.4,b'p'),(9.8,b'v'),(10.2,b'1'),
            (10.6,b'\x1b[C'),(10.9,b'\x1b[A'),(11.3,b'q')]
    sent_keys = []
    events = []
    recorder = OutputRecorder(events, start)
    try:
        while process.poll() is None:
            elapsed = time.monotonic()-start
            if elapsed > 20:
                process.kill()
                raise RuntimeError('gallery did not exit within bounded PTY smoke window')
            while keys and elapsed >= keys[0][0]:
                _, value = keys.pop(0)
                os.write(master, value)
                sent_keys.append(value.decode("ascii"))
                events.append([round(time.monotonic()-start,6), "i", value.decode("ascii")])
            if select.select([master], [], [], 0.02)[0]:
                chunk = os.read(master, 65536)
                if chunk:
                    recorder.record(chunk)
        while select.select([master], [], [], 0)[0]:
            chunk = os.read(master, 65536)
            if not chunk:
                break
            recorder.record(chunk)
        recorder.record(b'', final=True)
        after = termios.tcgetattr(slave)
    finally:
        if process.poll() is None:
            process.kill()
        process.wait()
        os.close(master)
        os.close(slave)
    if process.returncode != 0 or before != after or keys:
        raise RuntimeError(
            f'exit={process.returncode}; terminal settings restored={before == after}; '
            f'unsent control events={len(keys)}'
        )
    verify_capture_inputs(root, binary, source_sha256, binary_sha256)
    output = root/'previews/gallery-six-pty.cast'
    header = {'version':2,'width':100,'height':32,'title':'Spark six-design native gallery: real Linux PTY protocol output',
              'env':{'TERM':'xterm-256color','SHELL':'direct executable'},
              'duration':round(events[-1][0],6),
              'source_sha256':source_sha256,
              'binary_sha256':binary_sha256}
    output.write_text('\n'.join(json.dumps(record,ensure_ascii=False) for record in [header,*events])+'\n')
    report = {'schema':1,'scope':'Real Linux PTY protocol/control smoke; no emulator/font qualification',
              'status':'passed','exit_code':process.returncode,'terminal_settings_restored':before==after,
              'duration_seconds':round(events[-1][0],6),'keys_sent':sent_keys,
              'source_sha256':header['source_sha256'],'binary_sha256':header['binary_sha256'],
              'cast':'previews/gallery-six-pty.cast','cast_sha256':hashlib.sha256(output.read_bytes()).hexdigest(),
              'terminal_environment':header['env'],'dimensions':[100,32],
              'limitations':['PTY protocol smoke does not qualify a named emulator/font','Displayed cell semantics and all-six geometry are checked separately in TestBackend tests']}
    (root/'qualification/local').mkdir(parents=True,exist_ok=True)
    (root/'qualification/local/gallery-six-pty.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'PASS: all six gallery selections, controls, exit=0, terminal restored; {len(events)} recorded events')


if __name__ == '__main__':
    main()
