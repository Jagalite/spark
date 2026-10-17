# Release decisions awaiting the owner

## Current decision, 2026-10-17

The owner chose MIT for original project materials and public source publication
to Jagalite/spark. [Publication](publication.md) records validation and scope.
Code/docs/assets now carry MIT metadata; third-party terms are preserved.
Registry names, package/binary distribution, tags and platform support remain
separate decisions. The questions and observations below are the historical
2026-10-14 planning record, not unresolved source-publication permissions.

Recorded 2026-10-14. This note qualifies the local candidate; it does not choose
terms, establish ownership or legal clearance, reserve a name, or authorize a
repository push or package release.

## Current state

- [PROVENANCE](../PROVENANCE.md) and [ADR 0004](decisions/0004-licensing-and-provenance.md)
  leave code, documentation and artwork terms unresolved. No project license has
  been selected or granted
- The four workspace packages are provisionally named `spark-runtime`,
  `spark-catalog`, `spark-gallery` and `xtask`. Each inherits `publish = false`
  from the root manifest; all three standalone fixture manifests also explicitly
  set `publish = false`. None declares a project `license` or `license-file`
- The root manifest's library dependencies are local paths. These manifests are
  development inputs, not completed public-release metadata
- “Spark” remains a provisional product/repository name. The manifests do not select a final
  crates.io owner or registry publication destination. Existing repository work
  and its publication approval are separate from registry ownership/name decisions
- [Foundation status](foundation-status.md) records separate technical and
  publication limits. Local test or packaging results do not resolve these decisions

## Owner questions

1. **Distribution:** Should this remain private/local, be distributed privately,
   or become a public open-source project? Which source, docs, artwork, previews,
   binaries and packages are intended to be distributed, to whom and where?
2. **Code and documentation terms:** Which license or other distribution terms
   should apply to the code and documentation, and who is authorized to approve
   that grant? No license recommendation or selection is made here
3. **Artwork and assets:** What terms should cover procedural artwork, authored
   mascot frames, generated frame tables, preview images/GIFs and later imports?
   Should any use different terms from the code? Generated derivatives need their
   source terms tracked; generation alone supplies no rights clearance
4. **Contributor permissions and provenance:** Who supplied each contribution and
   the source brief, what permissions support the intended distribution, and
   what notices or restrictions must be retained? Confirm the contribution policy
   and any required approvals without assuming an assignment, sole rights holder,
   or unrestricted rights in AI-assisted material
5. **Final names and ownership:** Keep “Spark” or choose another product/repository
   name? What are the final repository owner/name and public library/package
   names? Should the gallery be distributed, and should `xtask` and the fixtures
   remain internal? Crate lookup alone does not clear trademarks, repository names,
   domains, related projects or confusingly similar names
6. **Publication and sequence:** Should any name-claim/reservation route be
   investigated or any repository/package be published at all? Specify the
   destination/account, artifacts, revision/version and order when approving that
   work. Name reservation or placeholder publication is not authorized by this
   local continuation, and a supported reservation route has not been established

## Public crate-name observation

Read-only, unauthenticated checks ran on **2026-10-14 between 03:20:34 and
03:20:53 UTC**. These are the exact official API endpoints and returned results:

| Provisional package | Primary source | Result |
| --- | --- | --- |
| `spark-runtime` | [crates.io API](https://crates.io/api/v1/crates/spark-runtime) | HTTP 404; JSON error detail: ``crate `spark-runtime` does not exist`` |
| `spark-catalog` | [crates.io API](https://crates.io/api/v1/crates/spark-catalog) | HTTP 404; JSON error detail: ``crate `spark-catalog` does not exist`` |

**Classification: not found at lookup time for both exact names.** No existing
exact-name crate conflict was observed. This is not an availability guarantee,
reservation, ownership check, normalized/related-name audit, IP clearance or
permission to publish. Availability must be rechecked if publication is later
authorized. No other proposed package names were checked.

The web reader could not retrieve these crate pages/API responses or their
official registry-index entries. The results above instead came from direct
unauthenticated HTTPS GETs to the APIs, with the HTTP status and JSON error body
inspected. Reader/cache failures and an absence of search hits were not treated
as evidence of absence. No credentials, sign-in or remote mutation were used.

The [Cargo publishing guide](https://doc.crates.io/crates-io.html), consulted
2026-10-14, states that names are allocated first-come-first-served and an existing
name cannot be used for a different crate. It also warns that publishing is
generally permanent. A name check therefore does not authorize a test upload or
placeholder release.

## Proposed decision order, not release approval

1. Record the owner's intended distribution, names, responsible owner and chosen
   code/documentation/artwork terms; resolve contribution and source permissions
2. Review the actual shipped files and dependency notices against that scope,
   then prepare approved license files, attribution and package metadata locally
3. Recheck final names and review any renaming across imports, examples, docs,
   fixtures and release evidence. Keep publication disabled until expressly
   authorized; requalify the final source and packaged contents
4. Obtain separate, specific approval for repository publication and registry
   releases, with destination, artifacts and revision/version recorded. Remote
   CI and still-unqualified platform acceptance remain explicit release gates
5. If both libraries are approved for crates.io, publish and verify the runtime
   before its dependent catalogue, using an approved version requirement in
   addition to the development path. Verify registry resolution before downstream
   packages. The [Cargo dependency guide](https://doc.crates.io/specifying-dependencies.html)
   describes publishing a path dependency separately and specifying its version

This is a release-planning record, not legal advice. No manifests, licenses,
accounts, remotes or registry state were changed to prepare it.
