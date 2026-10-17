# Authoring policy

The local foundation now has six reference implementations, two authoring paths,
and explicit host-owned timing. This is an implementation/review workflow, not a
claim that every release gate, terminal, or platform is qualified. Keep entry
status and evidence in the design brief, manifest, and qualification reports.
The reviewed F01–F06 contract is recorded in [API review](api-review.md); subsequent
public changes still require review and regression checks.

## Start with a concrete brief

Copy [the design brief template](../templates/design-brief.md). Describe the
composition, motion, driver, supported placements, sizes, fit policy, palette and
profile behavior, static fallback, and realistic embedding. Distinguish proposed,
implemented, independently reviewed, and measured/qualified work. Neither a
working constructor nor an attractive capture proves all four.

A palette or speed variant is not a new design. Aim for a recognizable silhouette
and motion idea. Do not copy existing characters or pass stock-spinner variants
off as distinct reference art. Host labels and authoritative information remain
visually primary.

The six studies exercise different needs:

- R01 Segmented Comet: repeating trail, bounded and declared-width inline output
- R02 Assembly Progress: caller-owned known/unknown progress, with no invented estimate
- R03 Border Runner: style-only decoration, explicit title/content exclusions
- R04 Telemetry Embers: bounded analytic particles driven by a caller-supplied level
- R05 Pocket Mascot: complete authored ASCII frames with unequal holds
- R06 Prism Reveal: finite text treatment and a glyph-preserving fallback

Use direct typed APIs. A new design does not justify a public dynamic registry,
new package, arbitrary script language, or additional dependency by itself.

## Runtime and host boundaries

- Store reusable immutable configuration; borrow temporary samples where practical
- Use explicit clip time and explicit seed/input values. No clock reads, IO,
  network, timers, threads, global mutable state, or elapsed-time-linear replay
- Validate configuration before paint and cap art work by dimensions and counts
- Use the public scoped writer with exactly the required rights. Do not expose
  its mutable buffer or widen opaque, masked, or style-only permissions
- Preserve requested coordinates under clipping; empty, tiny, offset, and fully
  out-of-buffer requests are ordinary inputs
- Keep `Keep`, styled `Erase`, and full-cell `Replace` distinct. Spaces erase;
  they are not transparency
- Preserve protected rectangles and whole existing wide-glyph footprints. A
  one-column art repertoire does not make a split host glyph safe to overwrite
- Keep task state, percentages, completion, data acquisition, stable identity,
  and operation-generation replacement in the host
- Resolve roles through the supplied palette/profile without assuming an unknown
  background is black. Quiet and reduced profiles need intentional presentations
- Declare allocation and work bounds precisely. An art cap does not cap arbitrary
  host row-prefix scans, exclusions, or text-conversion costs
- Repaint hints advise the host. Return no animation-only request for static,
  hidden/paused, completed, zero-input, or compact cases where applicable; continue
  painting their current/final appearance on unrelated host redraws

Finite effects must render the terminal frame before time-only wakeups cease.
Playback helpers map explicit host time to clip time; they do not own a thread,
input loop, data cadence, or business operation. Keep completion/query helpers
separate from callbacks or application mutation.

## Procedural and authored paths

Procedural designs use bounded pure calculations and direct seeking. Share helpers
only after a concrete design demonstrates the need. Keep a small amount of bespoke
Rust when it produces a better composition than a broad recipe abstraction.

R05 demonstrates the authored path in
[`catalog/pocket-mascot/frames.json`](../catalog/pocket-mascot/frames.json). The
versioned source is tooling-only. Runtime consumers compile ordinary static Rust
arrays and perform a binary lookup into cumulative frame-end times; they neither
parse JSON nor replay earlier frames.

### Schema v1

The exact top-level fields are:

- `schema`: currently `1`
- `design`: `pocket-mascot` for the current single-asset generator
- `visual_revision`: positive integer, distinct from crate/runtime API version
- `width`, `height`: positive native cell dimensions
- `poster`: in-range quiet/static frame index
- `glyphs`: unique printable non-space single-column ASCII glyphs
- `cells`: unique printable ASCII keys with explicit `keep`, `erase`, or `replace`
  operations; erase/replace select a palette role, replace references a glyph index
- `frames`: unique lowercase names, positive integer `hold_ms`, and complete row
  strings of exactly `width` keys, with exactly `height` rows

Palette roles are `foreground`, `muted`, `accent`, and `highlight`. Keep has no
palette/glyph field. Unknown or missing fields, roles, operations, controls,
Unicode/combining glyphs, invalid indices, incomplete rows, and invalid durations
are rejected. Spaces may be cell-table keys, but their meaning is the explicit
operation in that table. Delta frames, compression, remote assets, and arbitrary
Unicode strings are outside schema v1.

[`xtask/src/assets.rs`](../xtask/src/assets.rs) owns the central limits:

| Bound | Schema-v1 maximum |
| --- | ---: |
| Source bytes | 1,048,576 |
| Width / height | 128 / 64 |
| Frames | 256 |
| Expanded frame cells | 262,144 |
| Glyphs / cell keys | 94 / 95 |
| Sum of frame holds | 86,400,000,000,000 ns (24 hours) |

The reader takes at most the byte ceiling plus one byte so oversized inputs can be
rejected before parsing. JSON parsing keeps its default depth guard. Dimensions,
frame counts, checked products, checked duration multiplication/sums, references,
and row shapes are all validated before frame expansion. Runtime `FrameSequence`
independently validates its bounded, strictly increasing cumulative ends.

Generate and verify with:

```sh
cargo xtask generate-assets
cargo xtask generate-assets --check
```

The generated output is
[`crates/catalog/src/generated/mascot.rs`](../crates/catalog/src/generated/mascot.rs).
Never hand-edit it as the source of truth. The check command fails on drift without
rewriting anything. Review changed artwork and timing as well as the generated
diff; a successful regeneration proves consistency, not visual acceptance.

## Entry workflow and evidence

1. Write or update `catalog/<id>/brief.md`, its manifest, and code/asset sources
2. Keep stable ID and visual revision consistent. Manifests describe capabilities,
   geometry, capture configuration, source policy, and evidence locations. A
   path in an evidence field does not prove that a current successful report exists
3. Test the public API directly: repeated/shuffled sampling, direct seeking, input
   replacement, loop/finite boundaries, palette/profile fallback, and truthful data
4. Exercise empty/tiny/offset/clipped/extreme geometry, protected neighbors, wide
   glyphs, and preservation of every unapproved cell/style field
5. For authored assets test positive unequal holds and seams, independent expected
   cells, keep-versus-erase semantics, malformed source, limits before expansion,
   checked duration arithmetic, and byte-for-byte deterministic generation
6. Run the workspace checks and inspect failures. Do not disable tests, loosen
   rights, update unrelated goldens, or manufacture expected qualification data
7. Capture through the same Rust consumer code, with source/configuration hashes,
   time or input traces, size, seed if any, palette/profile, and capture settings
8. Request independent visual and API review; record reviewer, evidence, and limits

Useful commands in the current toolchain include:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo xtask validate
cargo xtask snapshots --check
cargo xtask render pocket-mascot
cargo xtask record pocket-mascot
cargo xtask qualify pocket-mascot
cargo xtask package-audit
```

`validate` includes dependency/renderer boundaries, manifests, and authored-data
drift checks. Existing R01 exact goldens remain a separate regression check.
Qualification reports must distinguish passed, failed, and unmeasured checks.
Bounded mutation tests are useful malformed-input coverage, not exhaustive fuzzing.
The native gallery, synthetic embeddings, consumer fixtures, and host examples
exercise different surfaces; none substitutes for every other check.

## Independent review

Inspect full-speed and slower playback, every authored key pose, loop seams or
finite completion, quiet poster, smallest supported/fallback size, and realistic
embedding. Review silhouette, pacing, legibility, distraction, theme fit, and
clipping. Actual captured cells prove the preview came from the implementation;
a fixed font preview does not qualify every terminal/font or backend.

Shared runtime changes revalidate dependent designs. Expected-image changes and
new paint rights require an independent reviewer. Content contributors cannot
approve their own widened permissions, add arbitrary dependencies/build scripts,
or overwrite established goldens to silence a failure. Keep proposed API changes
separate from art-only changes where practical.

Report numerical performance with target, toolchain, build settings, requested
geometry, source/configuration hashes, and measured scope. Do not infer allocation
freedom from pure functions or claim package elimination from a dependency list.
Publish only measured support claims after the remaining release gates pass.

## Provenance and publication

Record sources actually used, creator if known, license/permission evidence,
AI/tool assistance, transformations, and unresolved restrictions. Keep code and
artwork provenance separate; previews inherit their source restrictions. See the
[provenance template](../templates/provenance-entry.md) and
[project record](../PROVENANCE.md).

The owner selected MIT and authorized source publication to Jagalite/spark on
2026-10-17. Keep third-party terms and notices intact. Crates.io publication,
release tags, binary distribution and deployment need separate approval. AI assistance is an authoring tool, never a runtime service or an
automatic legal clearance. `forbid(unsafe_code)` is a source guardrail, not a
sandbox or permission for autonomous publication.
