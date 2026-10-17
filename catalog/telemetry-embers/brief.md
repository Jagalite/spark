# R04 — Telemetry Embers

- Stable ID: `telemetry-embers`
- Visual revision: **1**
- Status: **Implemented locally in F09; release/visual qualification pending**
- Driver: validated caller-supplied `Level` plus explicit clip time and seed
- Placement: owned bounded widget, normally 24×8; larger scene uses the same view
- Fit: adaptive 8×4 through 120×40; larger requests center the 120×40 cap

## Composition and motion

Sparse embers rise through an open field above a low, dotted hearth. Three seeded
horizontal positions form a soft central plume; each ember has an independent
3.5–8 second lifetime, phase, bounded triangular sway, and age-dependent fading.
Bright embers may leave one dim trailing cell in full motion. Large quiet spaces
and varied paths avoid an equalizer or dense particle screen.

Level controls the number of active indexed particles and the availability of
bright sparks. Zero displays only the quiet hearth. Static motion displays a
stable, level-dependent constellation. Reduced motion slows the analytic time
fourfold and removes trails. All poses derive directly from the supplied time;
no missed frames, particles, or data history are accumulated.

There is intentionally no global short repeat period: independently seeded
lifetimes need not align. A recorded observation window is not a seamless-loop
claim. Inputs, seed, clip timestamps, and capture window must be recorded with
any preview.

## Input contract

`Level::new(f64)` and `TryFrom<f64>` reject NaN, infinity, and values outside
`0..=1`. `Level::ZERO` and `Level::MAX` are convenient validated endpoints. The
private payload prevents invalid construction. The application measures/fetches
its own data, decides what a level means, and supplies updates at its own cadence.
The artwork performs no audio sampling, polling, synthetic telemetry, or smoothing.

Density is a visual response, not a labeled numeric scale. The hearth is a
composition element, not a fabricated measurement. A host's numeric labels,
alerts, thresholds, and scheduling remain authoritative.

## Geometry, rights, and fallback

Below 8×4, a centered static dot/spark shows zero/low/medium/high level, using both
shape and palette roles. Empty geometry paints nothing. The full field erases
unoccupied cells inside its capped composition and leaves cells outside untouched.
`OpaqueSurface` preserves requested coordinates while clipping and honors explicit
`with_protected` regions and existing wide-glyph footprints.

Unicode sparks use a one-column repertoire; ASCII uses `.`, `*`, and `+`.
ANSI-16 and monochrome preserve sparse structure. Colors resolve through the
caller palette; no alpha blending or implicit black background is used.
`frame_interval` advises 50 ms full or 150 ms reduced. Zero-level, static, tiny,
and empty presentations return none; unrelated redraws still paint their pose.

## Bounded work and evidence

At most 64 analytic particles, each with at most one tail cell, are sampled.
Geometry contributes at most 4,800 final cell writes, and fixed 4,800-byte stack
scratch stores intensity only. Particle capacity also scales down with area
(one per 12 cells before the 64 cap). No design-owned heap scratch, runtime IO,
clock, thread, or elapsed-time-linear replay is used. Host row-prefix scanning
and protected-list costs remain separate from those art bounds.

`crates/catalog/tests/data_designs.rs` covers invalid/boundary levels, seed effects,
current-input replacement without accumulation, shuffled/extreme times, exact
complete-cell comparisons, static/zero/tiny redraw advice, clipped/offset buffers,
protected/wide host glyphs, palette fallbacks, and giant requests against a tiny
buffer. The 120×40 cap and sparse-cell bounds are directly exercised.
Independent breadth tests, actual compact/larger-scene previews, visual review,
allocation/performance measurements, and platform coverage must be recorded
separately. An implemented 120×40 scene is not a qualified performance claim.

## Provenance

Original procedural composition and code authored for this project. No external
artwork, particle asset, or imported frame source is embedded. Code and artwork
terms are [MIT](../../LICENSE); see [provenance](../../PROVENANCE.md).
