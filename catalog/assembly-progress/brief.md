# R02 — Assembly Progress

- Stable ID: `assembly-progress`
- Visual revision: **1**
- Status: **Implemented locally in F09; release/visual qualification pending**
- Driver: validated caller-supplied `Progress` plus explicit clip time
- Placement: owned bounded widget, normally 24×3
- Fit: full machine at 12–80 columns and at least three rows; centered 80×3 cap

## Composition and motion

A small gantry travels above a row of solid assembled modules and dotted blueprint
positions. An open-sided conveyor with spaced rollers supports the workpiece.
Known progress determines only the middle-row module count. The gantry and rollers
supply restrained mechanical motion without changing the authoritative quantity.
A completed workpiece is still. Unknown progress has a central question mark and
no solid modules, so the moving machinery never masquerades as a percentage.

The default mechanical period is 1.8 seconds. Seed changes carrier and roller
phase. The caller may provide any validated positive `Period`. Reduced motion
slows motion fourfold; static motion parks the gantry while retaining the current
known/unknown state. Sampling seeks directly and never consumes input.

## Input and lifecycle contract

`Progress::known(f64)` and `TryFrom<f64>` reject NaN, infinity, and values outside
`0..=1`. `Progress::unknown()` is distinct from zero. The private payload prevents
invalid construction. No implicit clamping or smoothing occurs.

Whole modules are conservatively quantized from the exact supplied binary-f64
value using integer arithmetic. They never exceed the target, including values
immediately below a module boundary. A decimal represented microscopically below
an exact boundary can therefore remain one whole module below it. Empty/tiny
fallbacks preserve this bound. Labels and business completion remain host-owned.

Every sample carries its current progress. No prior target, completion, operation
identity, or generation is stored by the design. A host replaces/cancels an
operation by supplying its current input; stable entity and generation management
belongs to host state rather than visual state.

## Geometry, rights, and fallback

At widths 4–11, or heights below three rows, a centered static bracketed meter
shows solid/dotted slots or a question mark. At widths 1–3, an incomplete known
operation uses a quiet dot, a complete operation a solid point, and unknown a
question mark. Empty geometry paints nothing. Unused cells inside the capped
composition are erased; cells outside it remain untouched.

Painting uses `OpaqueSurface`, preserves requested coordinates under clipping,
and honors `with_protected` and existing wide-glyph footprints. No mutable buffer
escape is used. ASCII uses `#` versus `.`; Unicode uses `■` versus `·`.
Palette roles and ANSI-16/monochrome resolution retain meaning without assuming
black behind an unknown background.

`frame_interval` is advisory only: 80 ms full or 200 ms reduced; complete, static,
empty, and tiny presentations return none. Pausing is the host holding clip time,
not a reason to omit artwork on unrelated redraws.

## Bounded work and evidence

Artwork visits at most 240 cells. No design-owned heap scratch, runtime IO,
clock, thread, or elapsed-time-linear simulation is used. This cap does not bound
host row-prefix scans or the length of caller-supplied protected lists.

`crates/catalog/tests/data_designs.rs` covers input errors/boundaries, conservative
module counts across every supported width, exact ASCII silhouette, complete
cell equality at shuffled/extreme times, unknown/replacement transitions, period
seams, quiet profiles, clipped coordinates, protected/wide neighbors, and huge
requests against small actual buffers. Independent breadth tests are separate.
Actual visual captures, isolated/embedded review, allocation measurement,
compact timing, and platform coverage must be recorded before qualification;
implementation and deterministic tests alone do not establish those results.

## Provenance

Original procedural composition and code authored for this project. No external
artwork, character, font asset, or imported frame source is embedded. Code and art
terms are [MIT](../../LICENSE); see [provenance](../../PROVENANCE.md).
