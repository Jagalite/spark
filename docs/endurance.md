# Bounded host qualification

This source-only repository retains the canonical workload script at
`scripts/qualify_host_endurance.py`, host lifecycle tests and seven harness guard
cases. Historical process recordings, machine/resource details, archived inputs,
binaries and verification reports are private archival evidence under
[ARTIFACTS](../ARTIFACTS.md), not files shipped in this checkout.

## Scope of the earlier local work

The October 14 candidate completed a bounded Linux PTY workload across synchronous,
async-event and already-paced host examples. It exercised input storms, source
replacement, resize, pause/hidden/static states, finite completion, recovery from
stalls, exit and terminal-state restoration. Independent deterministic tests also
cover the same lifecycle invariants. This is historical, fixed-workload evidence;
it is not a fresh endurance run of the licensed publication checkout.

## Reproduce on a suitable Linux host

```sh
python3 scripts/qualify_host_endurance.py --help
python3 scripts/qualify_host_endurance.py
python3 qualification/endurance-linux-x86_64/test_harness_guards.py
```

Inspect the script's acceptance criteria and output location before running.
A new run creates new evidence; retain its exact source/configuration, actual
outcome and explicit omissions. Do not reuse an old pass for a changed source.

## Remaining qualification

PTY protocol output does not qualify a physical emulator's font shaping, display
latency or CPU cost. macOS/Windows native hosts, terminal/font/tmux/SSH combinations,
browser-WASM playback/parity and multi-hour/day behavior remain separate checks.
No universal resource budget or zero-allocation guarantee follows from the bounded
workload. The current source checks are recorded in [publication](publication.md).
