# Caller-fed progress, task, and stream adapters

`spark_catalog::adapters` translates values the host already has into catalogue
inputs. It does not fetch, poll, read a clock, start a task, deliver a callback, or
decide whether work finished. Its types are optional and require no extra dependency.
The host owns the adapter values, source identity, labels, lifecycle, and redraws.

```sh
cargo run -p spark-catalog --example adapters
cargo run -p spark-catalog --example adapters -- --smoke
```

The example is deterministic and offscreen. Its data is illustrative; it starts no
background operation and does not initialize a terminal. It renders existing
Assembly Progress and Telemetry Embers widgets and rejects a stale source event.

## Counted progress

```rust
use spark_catalog::adapters::CountedProgress;

let counts = CountedProgress::new(7, Some(10))?;
let progress = counts.progress(); // Pass to AssemblyProgress::sample.
// Display counts.completed() and counts.total() in an authoritative host label.
# Ok::<(), spark_catalog::adapters::AdapterError>(())
```

- `completed > total` is rejected when the total is known; it is not clamped
- An unknown total is `None`, even when some units have already completed
- The valid empty count `0 / 0` yields unknown progress, because there is no ratio
- For a positive total, the normalized value is the largest binary `f64` no
  greater than the exact integer ratio. `u64::MAX - 1` out of `u64::MAX` remains
  strictly below one; only equal, nonzero counts produce one
- Preserve integer counts for labels. Conservative conversion plus whole-cell
  quantization can leave a cell unfilled at a rational boundary that is not exactly
  representable. Do not use a rounded percentage or a visual cell count as the
  authoritative completed count

Each snapshot stands alone. Changing total, replacing an operation, or moving back
to unknown progress retains no previous target or interpolation state. This type
does not establish update ordering or own an operation identifier.

## Explicit task snapshots

```rust
use spark_catalog::{Progress, adapters::{TaskSnapshot, TaskStatus}};

// All bytes may be received while host verification is still running.
let task = TaskSnapshot::new(TaskStatus::Running, Progress::known(1.0)?);
assert!(!task.status().is_terminal());

// Only an authoritative host event supplies a terminal status.
let task = TaskSnapshot::new(TaskStatus::Succeeded, Progress::unknown());
assert!(task.status().is_terminal());
# Ok::<(), spark_catalog::InputError>(())
```

`TaskStatus` is `Queued`, `Running`, `Succeeded`, `Failed`, or `Cancelled`.
`is_terminal()` only classifies that declared status. `TaskSnapshot` is an immutable
pair of status and progress; it does not validate business transitions or infer one
field from the other. Failed/cancelled work can retain partial progress. A succeeded
task can have unknown progress. Painting, seeking, completing a finite visual, and
activity decaying to zero cannot change the task.

The host must resolve task identity and reject obsolete task updates before making
a snapshot. A status-theme mapping is also host policy: an empty result is not the
same as a queued or cancelled operation. Choose `status::StatusKind` and write the
appropriate explanatory label explicitly rather than automatically converting all
task statuses into presentation states.

## Stream activity with explicit observation time

```rust
use spark_catalog::adapters::{StreamActivity, StreamEvent};
use spark_runtime::{HostTime, Period};
use std::time::Duration;

let mut stream = StreamActivity::new(
    12, HostTime::ZERO, Period::new(Duration::from_secs(1))?, 8,
)?;
stream.record(StreamEvent {
    generation: 12, sequence: 0, at: HostTime::ZERO, units: 4,
})?;
let level = stream.level(HostTime::from_nanos(250_000_000))?;
assert_eq!(level.value(), 0.25); // Pass to TelemetryEmbers::sample.
# Ok::<(), Box<dyn std::error::Error>>(())
```

Capacity is a positive impulse cap, and decay is a positive `Period`. An event adds
`units / capacity` to the current level, capped at one. Between events the level
falls by `elapsed / decay`, stopping at zero. A full level empties over one decay
period; a half level empties in half that time. Excess activity is intentionally
discarded at the cap, so a huge burst does not extend the full-level duration.
Zero-unit events are valid observations and still consume their sequence number.

This is an activity cue, not a rate measurement, ETA, completed count, health check,
or proof a task is running. The host decides what a unit represents. Quiet streams
may be stalled or waiting, and a just-completed operation can still have visual
activity. Neither condition changes `TaskStatus`.

Only `record` and `reset` change stored visual credit. Decay uses exact integer
credit: `units * decay_ns`, subtracting `elapsed_ns * capacity`. Products use
`u128`; additions saturate before applying the cap. `level` converts the retained
fraction conservatively into `Level`. Dense or sparse queries do not change later
results, and zero-unit observations do not accumulate rounding error.

### Ordering and replacement

- A record must match the current generation exactly
- Within a generation, sequence numbers must strictly increase. The first may be
  any `u64`, including zero. Duplicates and out-of-order records are errors
- Accepted event timestamps are nondecreasing. Equal-time events are allowed;
  a higher-sequence event with an older observation timestamp is still rejected
- Every rejected record or reset leaves all fields unchanged, including ordering
- `level(now)` may repeat or shuffle queries at/after the most recent accepted
  event/reset. Earlier queries fail: this fixed-size adapter has no event history
- A future query is only a read, so it does not block a later event with an earlier
  timestamp than that query, provided the event meets the event-order rules
- `reset(new_generation, now)` clears credit and sequence tracking. It requires a
  strictly greater generation and a timestamp at/after the last event/reset

On source replacement or cancellation, retire the old generation in host state.
Old queued events must retain their original generation. Do not retag them with
the new value. An event from an old generation is rejected even if its timestamp
or sequence is very large. New-generation sequences can start at zero. The adapter
does not cancel the source itself or accept further events by guessing its state.

Generations and sequences never wrap. At `u64::MAX`, retire the exhausted lifecycle
and route a separately identified replacement in the host rather than reusing an
identifier for which old events might still arrive. A saturated `HostTime::MAX`
cannot express further elapsed time; the adapter does not invent it.

### Scheduling and bounds

Use monotonic caller observation time for activity, separate from a seekable or
paused visual clip position. Feed events on host-owned input/data updates and
sample the current level during host-owned draws. Invalidate on new data and
replacement. If the host wants visible decay, it owns presentation scheduling;
ordinary design hints can be used for a moving view, but a static profile does
not schedule activity updates for you. No adapter adds a timer or wakes the host.

These adapters allocate no memory and retain no history. Each stream operation
uses fixed-size values and bounded arithmetic. Ratio conversion uses at most 180
binary-division steps, independent of source lifetime, elapsed time, or prior
event count. This is a bound, not a measured performance claim. Host labels and
widgets may have separate allocation and rendering costs.

The small shared conversion routine deliberately avoids casting both integers to
`f64` and dividing: distinct large integers can round to the same floating-point
value, falsely making incomplete work look complete. Count ratios need up to
64-bit denominators; exact stream credit can need 128-bit denominators. Binary
long division finds the leading bit and emits 52 fraction bits without rounding.
It uses a subtraction comparison before doubling the remainder, avoiding an
overflowing 129-bit intermediate. Tests compare returned count ratios and the
next representable float against exact integer fractions independently of that
algorithm.

Focused public-API tests cover invalid/zero totals, exact integer rounding,
extreme arithmetic, explicit terminal states, first sequence zero, duplicate and
late records, atomic errors, same-time bursts, source replacement, shuffled reads,
and fractional-decay invariance. These are synthetic adapter checks, not a claim
that an external stream service or application integration has been qualified.
