# Performance, allocation and output methods

Detailed historical benchmark, process and machine reports are private archival
evidence under [ARTIFACTS](../ARTIFACTS.md), not public files in this source-only
checkout. The methods below support new measurements; no universal hardware or
release-performance guarantee is claimed.

## Reproduce

```sh
cargo xtask baseline
cargo xtask allocations
cargo xtask package-audit
cargo xtask render telemetry-embers --fixture dashboard
cargo xtask record telemetry-embers
cargo xtask qualify telemetry-embers
cargo xtask snapshots --check
cargo xtask generate-assets --check
```

`allocations` runs the allocation observation alone; `baseline` runs construction,
painting/output and allocation observations together. `qualify` includes the
workspace/dependency/renderer boundary validator as a required local check, not
just the design tests.

Final reports record source fingerprints, exact scenarios and raw command output.
The source fingerprint covers runtime/catalogue, examples/tests, gallery/tooling,
consumer fixtures, manifests/assets, exact-cell goldens and capture scripts. A
source/evidence handoff also has its own file-level checksums. Local reruns write
ignored `qualification/local/`; historical accepted evidence remains in the
separate private archive and is not silently overwritten.

## Timing stages and workloads

The foundation benchmark has 1,000 warmed samples per scenario. It separates base
preparation, painting, buffer diff and actual Crossterm encoding to a preallocated
in-memory sink. Workloads include no art, one comet, 120×40 Telemetry Embers, six mixed
designs, 24 repeated visible instances, static, hidden and completed states.
The old F06 construction measurement is historical and one-design only.

No-art/static/completed/hidden repeated measurements intentionally call stages even
when a real host should skip redundant presentations. An unchanged buffer can still
produce 13 backend reset bytes; zero changed cells does not imply zero output bytes.
The examples test avoiding animation-driven presentations when inactive. Their
producer counts are independent of visual cadence.

The tooling executable uses `stats_alloc 0.1.10` over the system allocator through
its safe wrapper. The final latency binary is instrumented too, so measurements
include that instrumentation. No unsafe block or allocator override enters the
runtime/catalogue. Do not compare instrumented and historical uninstrumented
numbers as a performance regression without controlling the measurement setup.

## Allocation observation

Allocation reports separate warmed direct painting from buffer construction,
host base layout, diff, backend encoding and owned inline conversion. Each of the
six widget paths is sampled 1,000 and 100,000 times into an existing buffer; its
configuration, protections and base setup are created outside the region.
This is an allocation observation, not fresh-base visual/output evidence.

The owned inline adapter allocates. Any warmed-paint allocation observations apply
only to their measured matrix, not every geometry, palette, string, host buffer
or future version. Inspect fresh allocation results instead of assuming an
unconditional allocation guarantee.

The metrics count requested allocations/bytes. They do not measure process RSS,
allocator fragmentation, reserved memory, full-application CPU or terminal-emulator
CPU. A 100,000-sample direct loop is not a long-lived interactive-host endurance test.

## Downstream package and binary audit

`fixtures/consumer` and `fixtures/consumer-all` are separate locked workspaces built
with the same recorded release arguments. The one-design fixture also retains its
early small TravelingMarker extension exercise; this is a comparison of these exact
fixture binaries, not a perfect isolated marginal cost of each design.

The audit reads executable paths from Cargo compiler-artifact messages, so custom
`CARGO_TARGET_DIR`/target-directory settings and native executable suffixes are
respected rather than assuming a fixture-local `target/release` path.

Reports include binary sizes/hashes, build durations, normal dependency trees,
package file lists and symbol observations. The consumers do not pull Tokio,
Crossterm, gallery, JSON, hashing, image encoders or allocation instrumentation into
their normal dependency graphs. Package listing is not package publication.

A missing mascot symbol supports the observation that no recognizable mascot symbol
is linked in the one-design executable. Constant/inlined/anonymous data may lack
recognizable symbols, so symbol absence and smaller size do not prove byte-complete
asset elimination. Binary composition can also change with compiler/linker settings.

## Rendering and host costs

Scoped writers scan existing row prefixes to preserve wide footprints, and check
host-supplied exclusion lists. The art's dimension/particle cap bounds its own work,
not every possible buffer-origin/exclusion-list cost. The compact/scene benchmark
uses actual ember fields rather than requesting a huge rectangle around a three-row
comet and calling that a full scene.

Fixed-font image rendering uses Pillow outside the Rust consumers. Its version,
font hash, dimensions, colors, frame holds and style approximations are captured
separately. The GIF pipeline uses actual consumer cells and labels finite replays
and nonperiodic observation windows honestly.

## Terminal evidence and remaining checks

The Linux PTY protocol capture exercises real backend output, six selection controls,
playback/input/view changes, exit and terminal-state restoration. It does not run a
named terminal emulator or inspect its glyph shaping. macOS/Windows interactive terminal review,
tmux/SSH, output latency, emulator CPU and native/WASM sample equality remain
unmeasured. See [compatibility](compatibility.md).

A subsequent [bounded real-host workload](endurance.md) exercises the repaired
sync/async/paced release examples in Linux PTYs for about 50 seconds each. It records
process RSS/HWM, CPU ticks, descriptors, threads, actual output, input storms,
resizing, completion, stall recovery and terminal restoration. The private archival
host-input fingerprint identifies what ran. These host observations supplement the
original drawing microbenchmarks; they do not establish multi-hour/day endurance,
fragmentation or terminal-emulator CPU/latency.
