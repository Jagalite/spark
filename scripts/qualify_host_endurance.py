#!/usr/bin/env python3
"""Bounded Linux PTY qualification of the three actual release example hosts.

Standard library only. This measures host processes, not a terminal emulator or
allocator internals. A small VT reader reconstructs ASCII status checkpoints; it
does not qualify typography, styles, wide-cell preservation, or full VT behavior.
Raw protocol output is retained so the reconstruction can be independently audited.
"""

import argparse
import codecs
import errno
import fcntl
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import pty
import re
import select
import signal
import statistics
import struct
import subprocess
import sys
import termios
import time
import unicodedata


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUT = ROOT / "qualification/endurance-linux-x86_64"
FROZEN_SOURCE = "0c9fd43a2bb9b799b1cce95f40dcdafb4d0d581e9aa353e5c26b5e725ed1eaf2"
HOSTS = ("sync_host", "async_host", "paced_host")
LAYOUTS = ("Table / stable entity rows", "Request / replace and cancel",
           "Dashboard / independent data cadence", "Reader / protected selection and media")
FRAME_SECONDS = .016
OBSERVE_SECONDS = 64 * FRAME_SECONDS
SETTLE_SECONDS = 8 * FRAME_SECONDS
CYCLE_OBSERVE_SECONDS = 16 * FRAME_SECONDS
BURST = b"ncoohhss  rllll" * 64
GEOMETRIES = ((12, 4), (40, 12), (80, 24), (160, 48), (3, 1),
              (100, 32), (120, 40), (100, 32))
CLOCK_TICKS = os.sysconf("SC_CLK_TCK")
CRITERIA = {
    "warmup_cycles": 4,
    "measured_cycles": 32,
    "burst_sequences_each_cycle": 64,
    "resize_requests_each_cycle": len(GEOMETRIES),
    "checkpoint_timeout_seconds": 5,
    "quit_timeout_seconds": 5,
    "rss_and_hwm_safety_ceiling_kib": 128 * 1024,
    "post_warmup_rss_and_hwm_envelope_kib": 1024,
    "settled_fd_and_thread_counts": "exactly equal to warmed baseline",
    "inactive_busy_spin_sentinel_cpu_fraction_one_core": .90,
    "inactive_output": "sync/async: zero after settle; paced: observation only",
    "active_and_resumed_output": "positive byte count after settle; output liveness, not direct mode identity",
    "exit_code": 0,
    "terminal_attributes": "exactly restored",
    "scope": "predeclared local observation envelopes and fault sentinels; not universal release budgets",
}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_manifest():
    """Inputs of measured release hosts, not unrelated documentation or evidence."""
    paths = [ROOT / p for p in ("Cargo.lock", "Cargo.toml", "rust-toolchain.toml",
                                ".cargo/config.toml", "crates/runtime/Cargo.toml",
                                "crates/catalog/Cargo.toml")]
    for folder in ("crates/runtime/src", "crates/catalog/src", "crates/catalog/examples"):
        paths += sorted((ROOT / folder).rglob("*.rs"))
    return {str(p.relative_to(ROOT)): sha256(p) for p in sorted(paths)}


def fingerprint(manifest):
    return hashlib.sha256(json.dumps(manifest, sort_keys=True).encode()).hexdigest()


class StatusScreen:
    """Minimal cursor/erase parser for ASCII host status, never a visual oracle."""

    def __init__(self, width=100, height=32):
        self.width, self.height = width, height
        self.cells = {}
        self.x = self.y = 0
        self.escape = ""
        self.unknown = set()

    def resize(self, width, height):
        self.width, self.height = width, height
        self.x = min(self.x, width - 1)
        self.y = min(self.y, height - 1)

    def feed(self, text):
        for char in text:
            if self.escape:
                self.escape += char
                if len(self.escape) == 2 and char != "[":
                    self.unknown.add(self.escape)
                    self.escape = ""
                elif len(self.escape) > 2 and "@" <= char <= "~":
                    self.csi(self.escape[2:-1], char)
                    self.escape = ""
                continue
            if char == "\x1b":
                self.escape = char
            elif char == "\r":
                self.x = 0
            elif char == "\n":
                self.y = min(self.y + 1, self.height - 1)
            elif char == "\b":
                self.x = max(0, self.x - 1)
            elif ord(char) >= 32:
                if unicodedata.combining(char):
                    continue
                width = 2 if unicodedata.east_asian_width(char) in ("W", "F") else 1
                if self.x >= self.width:
                    self.x = 0
                    self.y = min(self.y + 1, self.height - 1)
                self.cells[self.x, self.y] = char
                if width == 2:
                    self.cells[self.x + 1, self.y] = " "
                self.x += width

    def csi(self, args, op):
        # SGR and DEC private modes do not change ASCII status coordinates.
        if op == "m" or args.startswith("?"):
            return
        values = [int(v) if v else 0 for v in args.split(";")]
        n = (values[0] if values else 0) or 1
        if op in ("H", "f"):
            self.y = n - 1
            self.x = ((values[1] if len(values) > 1 else 0) or 1) - 1
        elif op == "A":
            self.y = max(0, self.y - n)
        elif op == "B":
            self.y = min(self.height - 1, self.y + n)
        elif op == "C":
            self.x = min(self.width - 1, self.x + n)
        elif op == "D":
            self.x = max(0, self.x - n)
        elif op == "G":
            self.x = n - 1
        elif op == "d":
            self.y = n - 1
        elif op == "J":
            mode = values[0]
            if mode in (2, 3):
                self.cells.clear()
            else:
                self.cells = {p: c for p, c in self.cells.items()
                              if (mode == 0 and (p[1], p[0]) < (self.y, self.x))
                              or (mode == 1 and (p[1], p[0]) > (self.y, self.x))}
        elif op == "K":
            mode = values[0]
            self.cells = {p: c for p, c in self.cells.items()
                          if p[1] != self.y or (mode == 0 and p[0] < self.x)
                          or (mode == 1 and p[0] > self.x)}
        else:
            self.unknown.add("CSI " + args + op)

    def row(self, y):
        return "".join(self.cells.get((x, y), " ") for x in range(self.width))

    def status(self):
        match = re.search(r"id=(\d+) gen=(\d+) (working|complete|cancelled) \| data polls=(\d+)",
                          self.row(1))
        if match:
            return {"id": int(match[1]), "generation": int(match[2]),
                    "state": match[3], "data_polls": int(match[4])}
        return None


def proc_snapshot(pid):
    root = Path("/proc") / str(pid)
    status = {}
    for line in (root / "status").read_text().splitlines():
        key, _, value = line.partition(":")
        status[key] = value.strip()
    stat = (root / "stat").read_text().rpartition(") ")[2].split()
    return {"state": status["State"],
            "rss_kib": int(status["VmRSS"].split()[0]),
            "hwm_kib": int(status["VmHWM"].split()[0]),
            "virtual_kib": int(status["VmSize"].split()[0]),
            "threads": int(status["Threads"]),
            "fd_count": len(list((root / "fd").iterdir())),
            "cpu_user_ticks": int(stat[11]), "cpu_system_ticks": int(stat[12]),
            "cpu_ticks": int(stat[11]) + int(stat[12]),
            "voluntary_context_switches": int(status["voluntary_ctxt_switches"]),
            "involuntary_context_switches": int(status["nonvoluntary_ctxt_switches"])}


class HostRun:
    def __init__(self, name, out, manifest):
        self.name, self.out = name, out
        self.binary = ROOT / "target/release/examples" / name
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 100, 0, 0))
        self.termios_before = termios.tcgetattr(self.slave)
        self.screen = StatusScreen()
        self.decoder = codecs.getincrementaldecoder("utf-8")("replace")
        self.record = gzip.open(out / f"{name}.cast.gz", "wt", encoding="utf-8")
        self.raw = gzip.open(out / f"{name}.output.bin.gz", "wb")
        self.samples_file = (out / f"{name}.samples.jsonl").open("w")
        self.log = (out / f"{name}.log").open("w")
        self.start = time.monotonic()
        self.last_sample = 0
        self.samples = []
        self.checkpoints = []
        self.windows = []
        self.cycle_points = []
        self.bytes_output = self.keys_sent = self.resize_requests = 0
        self.output_tail = b""
        self.phase = "startup"
        self.errors = []
        self.process = subprocess.Popen([str(self.binary)], stdin=self.slave,
                                        stdout=self.slave, stderr=self.slave, cwd=ROOT,
                                        start_new_session=True,
                                        env={**os.environ, "TERM": "xterm-256color",
                                             "COLORTERM": "truecolor"})
        self.record.write(json.dumps({"version": 2, "width": 100, "height": 32,
            "title": f"Spark actual release {name}: bounded Linux PTY endurance",
            "env": {"TERM": "xterm-256color", "COLORTERM": "truecolor"},
            "host_inputs_sha256": fingerprint(manifest),
            "binary_sha256": sha256(self.binary)}) + "\n")

    def elapsed(self):
        return time.monotonic() - self.start

    def event(self, kind, value):
        self.record.write(json.dumps([round(self.elapsed(), 6), kind, value],
                                     ensure_ascii=False) + "\n")

    def note(self, message):
        line = f"{self.elapsed():.6f} {self.phase}: {message}"
        self.log.write(line + "\n")
        self.log.flush()

    def sample(self, force=False):
        if not force and self.elapsed() - self.last_sample < .05:
            return None
        self.last_sample = self.elapsed()
        try:
            sample = {"seconds": round(self.elapsed(), 6), "phase": self.phase,
                      "output_bytes": self.bytes_output,
                      **proc_snapshot(self.process.pid)}
        except (FileNotFoundError, ProcessLookupError, KeyError):
            return None
        self.samples.append(sample)
        self.samples_file.write(json.dumps(sample) + "\n")
        ceiling = CRITERIA["rss_and_hwm_safety_ceiling_kib"]
        if max(sample["rss_kib"], sample["hwm_kib"]) > ceiling:
            raise RuntimeError(f"RSS/HWM exceeded declared {ceiling} KiB safety ceiling")
        return sample

    def pump(self, duration=0, allow_exit=False):
        until = time.monotonic() + duration
        while True:
            self.sample()
            ready = select.select([self.master], [], [], min(.025, max(0, until - time.monotonic())))[0]
            if ready:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
                    chunk = b""
                if chunk:
                    self.raw.write(chunk)
                    self.bytes_output += len(chunk)
                    self.output_tail = (self.output_tail + chunk)[-8192:]
                    text = self.decoder.decode(chunk)
                    self.screen.feed(text)
                    self.event("o", text)
            if self.process.poll() is not None and not allow_exit:
                raise RuntimeError(f"unexpected host exit {self.process.returncode}: {self.output_tail!r}")
            if time.monotonic() >= until:
                break

    def keys(self, value):
        self.event("i", value.decode("ascii"))
        total = 0
        while total < len(value):
            total += os.write(self.master, value[total:])
        self.keys_sent += len(value)

    def resize(self, width, height):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", height, width, 0, 0))
        self.screen.resize(width, height)
        os.kill(self.process.pid, signal.SIGWINCH)
        self.event("r", f"{width}x{height}")
        self.resize_requests += 1

    def checkpoint(self, label, generation=None, state=None, layout=None, polls=None,
                   predicate=None, timeout=5):
        started = self.elapsed()
        until = time.monotonic() + timeout
        while True:
            self.pump(.008)
            status = self.screen.status()
            if (status and status["id"] == 42
                    and (generation is None or status["generation"] == generation)
                    and (state is None or status["state"] == state)
                    and (polls is None or status["data_polls"] == polls)
                    and (layout is None or LAYOUTS[layout] in self.screen.row(0))
                    and (predicate is None or predicate(status))):
                point = {"label": label, "seconds": round(self.elapsed(), 6),
                         "response_seconds": round(self.elapsed() - started, 6),
                         "status": status, "header_row": self.screen.row(0),
                         "status_row": self.screen.row(1), "resources": self.sample(True)}
                self.checkpoints.append(point)
                self.note(f"checkpoint {label}: {status}")
                return point
            if time.monotonic() > until:
                raise RuntimeError(f"checkpoint {label} timed out; expected gen={generation}, "
                                   f"state={state}, layout={layout}, polls={polls}; "
                                   f"header={self.screen.row(0)!r}; row={self.screen.row(1)!r}")

    def window(self, label, inactive=False, duration=OBSERVE_SECONDS):
        self.phase = label
        self.pump(SETTLE_SECONDS)
        before = self.sample(True)
        output_before = self.bytes_output
        started = self.elapsed()
        self.pump(duration)
        after = self.sample(True)
        elapsed = self.elapsed() - started
        cpu_seconds = (after["cpu_ticks"] - before["cpu_ticks"]) / CLOCK_TICKS
        result = {"label": label, "seconds": round(elapsed, 6),
                  "cpu_seconds": cpu_seconds,
                  "cpu_fraction_one_core": cpu_seconds / elapsed,
                  "output_bytes": self.bytes_output - output_before,
                  "before": before, "after": after,
                  "inactive": inactive}
        self.windows.append(result)
        if inactive:
            if result["cpu_fraction_one_core"] >= .90:
                self.errors.append(f"{label}: inactive busy-spin sentinel exceeded")
            if self.name != "paced_host" and result["output_bytes"]:
                self.errors.append(f"{label}: inactive host emitted {result['output_bytes']} bytes")
        elif result["output_bytes"] == 0:
            self.errors.append(f"{label}: active/resumed output liveness check failed")
        self.note(f"observation {label}: CPU={cpu_seconds:.3f}s; output={result['output_bytes']} bytes")
        return result

    def comparison(self, prefix, generation, polls):
        self.window(prefix + "_active")
        for name, key in (("static", b"s"), ("paused", b" "), ("hidden", b"h")):
            self.keys(key)
            self.window(prefix + "_" + name, inactive=True)
            self.checkpoint(prefix + "_" + name, generation, "working", polls=polls)
            self.keys(key)
            self.pump(SETTLE_SECONDS)
        self.window(prefix + "_resumed")

    def cycle(self, number, generation, polls, measured):
        self.phase = f"{'measured' if measured else 'warmup'}_cycle_{number:02}"
        layout = 2
        for _ in range(4):
            layout = (layout + 1) % 4
            self.keys(b"l")
            self.checkpoint(self.phase + f"_layout_{layout}", generation, "working", layout, polls)
        self.keys(BURST)
        generation += 64
        for width, height in GEOMETRIES:
            self.resize(width, height)
            self.pump(.004)
        self.checkpoint(self.phase + "_storm_drained", generation, "cancelled", 2, polls)
        self.keys(b"n")
        generation += 1
        self.checkpoint(self.phase + "_replacement", generation, "working", 2, polls)
        self.pump(CYCLE_OBSERVE_SECONDS)
        self.keys(b" ")
        self.pump(SETTLE_SECONDS + CYCLE_OBSERVE_SECONDS)
        point = self.sample(True)
        point["cycle"] = number
        point["measured"] = measured
        self.cycle_points.append(point)
        self.keys(b" ")
        self.pump(SETTLE_SECONDS)
        return generation

    def run(self):
        quit_latency = None
        completed = False
        try:
            self.checkpoint("startup", 1, "working", 2)
            # Six genuine 500 ms producer updates take initial progress .25 to 1.
            self.phase = "producer_completion"
            completion = self.checkpoint("real_generation_complete", 1, "complete", 2,
                                         predicate=lambda s: s["data_polls"] >= 6)
            self.pump(1.200 + 2 * FRAME_SECONDS)
            self.checkpoint("after_finite_reveal_duration", 1, "complete", 2)
            if "Result ready. The host owns completion." not in self.screen.row(29):
                raise RuntimeError("completed reveal text not present in dashboard final row")
            self.keys(b"d")
            self.pump(SETTLE_SECONDS)
            polls = self.screen.status()["data_polls"]
            self.keys(b"n")
            generation = 2
            self.checkpoint("producer_disabled_new_generation", generation, "working", 2, polls)
            self.comparison("before_cycles", generation, polls)
            for cycle in range(1, CRITERIA["warmup_cycles"] + 1):
                generation = self.cycle(cycle, generation, polls, False)
            for cycle in range(1, CRITERIA["measured_cycles"] + 1):
                generation = self.cycle(cycle, generation, polls, True)
            self.comparison("after_cycles", generation, polls)
            # Stall for three 500 ms producer periods plus settling margin.
            # A single poll on resume is observable before its next 500 ms due time.
            self.phase = "long_stall"
            self.keys(b"d")
            self.pump(SETTLE_SECONDS)
            before_stall = self.checkpoint("before_stall", generation, "working", 2, polls)
            os.kill(self.process.pid, signal.SIGSTOP)
            self.event("m", "SIGSTOP: three producer periods, no input backlog injected")
            self.pump(3 * .500 + SETTLE_SECONDS)
            os.kill(self.process.pid, signal.SIGCONT)
            self.event("m", "SIGCONT")
            self.checkpoint("coalesced_single_data_poll_after_stall", generation, "working", 2, polls + 1)
            self.keys(b"d")
            self.pump(SETTLE_SECONDS)
            polls += 1
            self.checkpoint("producer_disabled_after_stall", generation, "working", 2, polls)
            self.phase = "quit_with_burst"
            # Queue much more than async's 32 events, then quit, then leave more
            # input queued. This exercises receiver drop before producer join.
            started = time.monotonic()
            self.keys(b"n" * 128 + b"q" + b"n" * 512)
            while self.process.poll() is None:
                self.pump(.016, allow_exit=True)
                if time.monotonic() - started > CRITERIA["quit_timeout_seconds"]:
                    raise RuntimeError("quit with queued input timed out")
            quit_latency = time.monotonic() - started
            self.pump(.050, allow_exit=True)
            completed = True
        except Exception as error:
            self.errors.append(f"{type(error).__name__}: {error}")
            self.note(self.errors[-1])
        finally:
            if self.process.poll() is None:
                os.kill(self.process.pid, signal.SIGCONT)
                self.process.kill()
                self.process.wait(timeout=5)
            self.exit_code = self.process.returncode
            self.restored = termios.tcgetattr(self.slave) == self.termios_before
            os.close(self.master)
            os.close(self.slave)
            for stream in (self.record, self.raw, self.samples_file, self.log):
                stream.close()
        measured = [s for s in self.cycle_points if s["measured"]]
        baseline = next((s for s in reversed(self.cycle_points) if not s["measured"]), None)
        resource_checks = {}
        if baseline and measured:
            for key in ("fd_count", "threads"):
                resource_checks[key] = {"baseline": baseline[key],
                    "min": min(s[key] for s in measured),
                    "max": max(s[key] for s in measured),
                    "passed": all(s[key] == baseline[key] for s in measured)}
            for key in ("rss_kib", "hwm_kib"):
                values = [baseline[key]] + [s[key] for s in measured]
                resource_checks[key] = {"baseline": baseline[key], "min": min(values),
                    "max": max(values), "span": max(values) - min(values),
                    "first_half_median": statistics.median(s[key] for s in measured[:len(measured)//2]),
                    "last_half_median": statistics.median(s[key] for s in measured[len(measured)//2:]),
                    "passed": max(values) - min(values) <= CRITERIA["post_warmup_rss_and_hwm_envelope_kib"]}
            for key, check in resource_checks.items():
                if not check["passed"]:
                    self.errors.append(f"post-warmup {key} criterion failed")
        if not completed:
            self.errors.append("workload incomplete")
        if self.exit_code != 0:
            self.errors.append(f"exit code {self.exit_code}, expected 0")
        if not self.restored:
            self.errors.append("terminal attributes not restored")
        raw_path = self.out / f"{self.name}.output.bin.gz"
        with gzip.open(raw_path, "rb") as stream:
            raw = stream.read()
        protocol = {"entered_alternate_screen": b"\x1b[?1049h" in raw,
                    "left_alternate_screen": b"\x1b[?1049l" in raw,
                    "cursor_shown_on_exit": b"\x1b[?25h" in raw[-1024:],
                    "panic_signature_absent": b"panicked at" not in raw}
        for key, passed in protocol.items():
            if not passed:
                self.errors.append("protocol check failed: " + key)
        result = {"host": self.name, "status": "passed-with-limitations" if not self.errors else "failed",
            "pid": self.process.pid, "duration_seconds": round(self.elapsed(), 6),
            "binary_sha256": sha256(self.binary), "binary_bytes": self.binary.stat().st_size,
            "exit_code": self.exit_code, "terminal_attributes_restored": self.restored,
            "protocol_checks": protocol, "quit_latency_seconds": quit_latency,
            "input_bytes_sent": self.keys_sent, "resize_requests": self.resize_requests,
            "output_bytes": self.bytes_output, "proc_samples": len(self.samples),
            "warmup_cycles_completed": len(self.cycle_points) - len(measured),
            "measured_cycles_completed": len(measured),
            "resource_checks": resource_checks,
            "peak_observed_rss_kib": max((s["rss_kib"] for s in self.samples), default=0),
            "peak_observed_hwm_kib": max((s["hwm_kib"] for s in self.samples), default=0),
            "last_observed_cpu_seconds": self.samples[-1]["cpu_ticks"] / CLOCK_TICKS if self.samples else None,
            "windows": self.windows, "checkpoints": self.checkpoints,
            "unhandled_protocol_sequences": sorted(self.screen.unknown), "errors": self.errors,
            "artifacts": {p.name: sha256(p) for p in self.out.glob(self.name + ".*")}}
        (self.out / f"{self.name}.json").write_text(json.dumps(result, indent=2) + "\n")
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--host", choices=HOSTS, action="append")
    parser.add_argument("--skip-build", action="store_true",
                        help="use existing release examples; caller is responsible for matching source")
    args = parser.parse_args()
    if sys.platform != "linux" or platform.machine() != "x86_64":
        parser.error("this qualification explicitly targets Linux x86_64 /proc and PTYs")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    manifest = source_manifest()
    plan = {"schema": 1, "criteria": CRITERIA,
            "frozen_foundation_reference_sha256": FROZEN_SOURCE,
            "host_inputs_sha256": fingerprint(manifest), "host_input_files": manifest,
            "harness_sha256": sha256(Path(__file__).resolve()),
            "source_references": ["docs/foundation-plan.md sections 9, 10.2, 11",
                                  "docs/integration.md", "crates/catalog/examples/async_host.rs"],
            "workload": {"hosts": args.host or HOSTS, "initial_geometry": [100, 32],
                "resize_geometries": GEOMETRIES, "burst_ascii": BURST.decode(),
                "observation_seconds": OBSERVE_SECONDS, "settle_seconds": SETTLE_SECONDS,
                "cycle_active_and_idle_seconds_each": CYCLE_OBSERVE_SECONDS,
                "stall_seconds": 3 * .500 + SETTLE_SECONDS,
                "async_source_channel_capacity": 32, "async_input_poll_ms": 25,
                "paced_host_frame_ms": 16},
            "machine": {"platform": platform.platform(), "uname": list(platform.uname()),
                        "python": sys.version, "proc_clock_ticks_per_second": CLOCK_TICKS,
                        "cpu": next((line for line in Path("/proc/cpuinfo").read_text().splitlines()
                                     if line.startswith("model name")), "unknown")}}
    (out / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    if not args.skip_build:
        with (out / "build.log").open("w") as log:
            subprocess.run(["cargo", "build", "--release", "--locked", "-p", "spark-catalog", "--examples"],
                           cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
    if source_manifest() != manifest:
        raise RuntimeError("host source inputs changed during build; no qualification run made")
    results = []
    for name in args.host or HOSTS:
        print(f"RUN {name}: fixed lifecycle/event/resize workload", flush=True)
        result = HostRun(name, out, manifest).run()
        results.append(result)
        print(f"{result['status'].upper()} {name}: {result['duration_seconds']}s; "
              f"RSS/HWM peak {result['peak_observed_rss_kib']}/{result['peak_observed_hwm_kib']} KiB; "
              f"exit={result['exit_code']}; restored={result['terminal_attributes_restored']}; "
              f"errors={result['errors']}", flush=True)
    source_unchanged = source_manifest() == manifest
    report = {"schema": 1, "status": "passed-with-limitations" if source_unchanged and
              all(r["status"] == "passed-with-limitations" for r in results) else "failed",
              "scope": "bounded full release host-process workload in Linux PTYs",
              "host_inputs_sha256": fingerprint(manifest), "source_unchanged_during_run": source_unchanged,
              "harness_sha256": plan["harness_sha256"], "criteria": CRITERIA,
              "hosts": [{k: v for k, v in r.items() if k not in ("windows", "checkpoints")}
                        for r in results],
              "limitations": [
                  "This completed-cycle experiment is bounded endurance evidence, not a multi-hour/day soak or a proof that no leak can ever occur.",
                  "RSS/HWM are process-level observations including host, terminal backend, and allocator retention; no allocator fragmentation or live-object census is inferred.",
                  "CPU time is quantized by /proc clock ticks; small idle costs may read zero. The 90% sentinel detects gross spinning, not a calibrated CPU performance budget.",
                  "Async input polls at 25 ms; paced host retains its 16 ms presentation clock, including inactive states. Zero animation-only wakes does not mean zero process work.",
                  "The async 32-event capacity and single input thread are source-inspected. No queue-occupancy instrumentation or universal queue bound is claimed for the OS or Crossterm internals.",
                  "Kernel SIGWINCH notifications can coalesce. Recorded resize requests are not a count of host-handled resize events.",
                  "VT reconstruction verifies ASCII status and layout labels, not exact styles, whole-wide-glyph preservation, font support, or emulator fidelity.",
                  "Control-mode flags are not exposed in PTY status. Source inspection and deterministic tests establish key-to-mode behavior; PTYs observe resulting output, not direct mode identity.",
                  "Real producer completion and post-duration reveal text are observed; exact final reveal styles/pose remain covered by separate deterministic tests.",
                  "No terminal-emulator CPU/latency, external application, macOS, Windows, tmux, SSH, or WASM execution was measured.",
                  "FD/thread checks use settled cycle checkpoints; periodic sampling can miss short-lived transients. Shared executor CPU is not isolated.",
                  "The original frozen source reference is historical; measured host sources and harness have separate content hashes and are checked unchanged across this run."]}
    (out / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["status"] == "passed-with-limitations" else 1


if __name__ == "__main__":
    sys.exit(main())
