# ADR 0004: Keep licensing unpublished pending owner approval

## Superseded on 2026-10-17

The owner selected MIT and authorized source publication to Jagalite/spark.
[LICENSE](../../LICENSE) and [NOTICE](../../NOTICE.md) now apply to that source
distribution. Registry packages, binaries, tags and deployment are not authorized
by this decision. The original decision below is historical.

- Status: Provisional; license selection requires the repository owner's decision
- Date: 2026-10-13

## Context

The supplied implementation brief requires code and artwork provenance. It does not select a license or authorize a package release; the owner separately authorized repository commits. Choosing a permissive license on the owner's behalf would grant rights beyond this implementation task.

## Decision

Do not select a project code, documentation, or artwork license in this slice. Keep packages unpublished and original project material without a project license grant. Do not label it MIT, Apache-2.0, Creative Commons, public domain, or open source. Packages must remain non-publishable until the owner chooses terms and approves publication.

This is an operational policy, not a representation that every generated element necessarily has copyright protection. Do not invent a rights holder or contributor assignment. Dependencies retain their own licenses; the absence of a project license does not replace them.

Track code, procedural art, authored assets, and generated previews separately in [PROVENANCE](../../PROVENANCE.md). Record AI assistance, external sources, transformations, and unresolved rights accurately. Do not claim originality or clearance from a generation process alone.

## Publication gate

Before a release, the owner must select the terms for code, documentation, and artwork; approve any differing asset terms; resolve external attribution/redistribution requirements; add the actual license files/metadata; and explicitly approve publishing. A test pass or package version never substitutes for that decision.

## Contributor policy

Submissions identify the contributor, source material, AI/tool assistance, intended terms, and known restrictions. Proposed terms are not automatically adopted. Do not copy third-party artwork, characters, snippets, fonts, or asset packs without an auditable source and permission compatible with the eventual distribution.

An independent review is required for changes to licensing, provenance records, runtime rights, dependencies, and golden outputs. See [authoring](../authoring.md).
