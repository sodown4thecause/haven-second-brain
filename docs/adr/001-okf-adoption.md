# ADR-001: Adopt OKF v0.1 as the on-disk knowledge contract

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-architect`
Reviewer: orchestrator review-board
Linked design: `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md §Static Knowledge Model`

## Context

Haven's canonical knowledge is plain Markdown in a Git repo. Before launch we
must lock the on-disk contract so importers, editors, indexes, models, and
sync envelopes all agree on the same shape. We considered three candidates:

1. **Google's Open Knowledge Format v0.1 (OKF v0.1)** — YAML frontmatter with
   non-empty `type`, recommended fields `title`/`description`/`resource`/
   `tags`/`timestamp`, standard Markdown links, reserved `index.md` /
   `log.md`.
2. **An Obsidian-flavored Markdown contract** — wikilinks, tags, frontmatter
   but no fixed `type` and no reserved `index.md` / `log.md`.
3. **A custom Haven Markdown contract** — full knowledge model in our own
   dialect.

## Decision

Adopt **OKF v0.1** as the strict-write contract. Reading is permissive:
unknown `type`, unknown fields, broken links, and missing optional fields
must all parse without throwing. We will not invent inline supertags,
block-level schemas, or proprietary relationship syntax. Saved views derive
from YAML frontmatter and links.

Haven authors new documents as **OKF-conformant**. Existing user files are
never mass-rewritten for OKF conformance — `Safe Existing Vault Mode`
(`M1`) opens them read/index first and requires an explicit user opt-in
before any migration runs.

## Alternatives considered

- **Obsidian flavor**: keeps long-tail user compatibility but loses the
  reserved-file convention, which the index, sync, and memory layers need
  to keep behavior stable. OKF v0.1 inherits the same wikilink-friendliness
  with permissive reads.
- **Custom dialect**: cheapest short term but creates lock-in and forces
  importer fans to maintain one more dialect.

## Consequences

- Adopting OKF v0.1 means every Haven-written document carries an `okf_version`
  field and a non-empty `type`. The `crates/okf` library enforces this on
  writes.
- The indexer can rely on `type` to build Bases-lite property views
  (P1.8) without inventing a schema.
- The MCP read tools (`search_brain`, `read_document`) can return
  frontmatter as a typed object instead of free-key JSON.
- The sync envelopes can carry `type` as a routing hint without inspecting
  the body.

## Reversibility

OKF adoption is reversible if Google revises the spec, with these conditions:

- We track `okf_version` so we can detect a newer draft.
- Permissive reads keep us compatible with future spec changes that introduce
  new optional fields.
- If we need to abandon OKF, the strict-write rule means we only wrote
  OKF-conformant files and the lint fixture suite gives a one-line acceptance
  test.

## Prior-art cross-reference

- Google Open Knowledge Format v0.1 upstream spec.
- `docs/research/prior-art-register.md §2 (importers)` and
  `§5 (MCP notes)` share the same flat-Markdown model.
- Knowledge Diffs (ADR-007 upstream design) reuse frontmatter as the typed
  source of citation provenance.

## Threat-model cross-reference

- Frontmatter rigor reduces ambiguity for citation provenance but is not a
  security boundary.
- Permissive reads defend against hostile note authors who craft unexpected
  `type` values.

## Acceptance evidence

R0 records the evidence contract. M1 scaffolding introduces `crates/okf`
and the strict-write linter (`scripts/okf-lint.mjs` referenced in
`AGENTS.md`). R0 contains no crate sources; the references below describe
the **expected** location and behavior of each acceptance test.

- `crates/okf/tests/property_roundtrip.rs`: property tests round-trip
  unknown keys, malformed inputs, and unknown `type` values without
  exceptions.
- `crates/okf/tests/strict_write.rs`: strict-write linter refuses documents
  missing `type`, duplicates of `index.md`/`log.md`, and missing
  `okf_version`.
- `crates/okf/tests/permissive_read.rs`: permissive-read accepts unknown
  `type`, unknown optional fields, and broken links; the test fixture set
  mirrors `AGENTS.md §2` posture.
- `crates/okf/tests/fixtures/`: a 200-note OKF fixture set; the test
  rebuilds the derived `.haven/` index after `rm -rf .haven`.
