# ADR 0002: Rendering samples explicit clip time

- Status: Accepted for the F01–F06 implementation scope
- Date: 2026-10-13

## Context

Advancing an animation inside `render()` ties motion to incidental redraws, makes seeking costly, and resets lightweight widgets constructed in a draw function. Long stalls must not produce a queue of old frames.

## Decision

Use `ClipTime` for a requested visual position and a validated, non-zero `Period` for repeating art. Both use `u64` nanoseconds; the fallible duration conversion rejects overflow, while an explicitly named saturating conversion supports deliberate host policy. A renderer computes the requested pose directly from configuration, seed, time, geometry, and palette/profile. It must support repeated, backwards, shuffled, and large timestamps without replaying history.

Store reusable `SegmentedComet` configuration in the application; render a temporary borrowed `CometSample`. Recreating the sample must not reset or advance any state. Caches, if eventually needed, cannot change output for identical semantic inputs.

Reduce repeat time before lossy numerical conversions. Validate bounded configuration once. The equality claim is exact cells for the same qualified build/target and semantic inputs, not universal terminal or cross-target floating-point equality.

Host time and clip time are distinct concepts. Only clip sampling is implemented in this slice. The comet has a simple size/profile-dependent recommended frame interval for F06's host. Optional playback mapping, rates, pause/seek commands, finite lifecycle, and general repaint/deadline planning will be designed later without changing ownership of the host's clock.

## Consequences

- Pausing requires the host to keep supplying a pose time; frozen art still paints.
- Static motion can have an intentionally authored pose rather than whichever frame was paused.
- Seeking costs depend on output area and bounded design work, not elapsed time.
- Completion of a future finite visual cannot imply completion of the underlying business operation.
- History-dependent smoothing needs explicit history or a separately named stateful helper.

## Verification

Test equal/reordered timestamps, two redraws at one time, time zero, both sides of a loop boundary, extreme time values, and seed reproducibility. Do not infer these properties from a live demo alone.
