# ADR-002: Editor round-trip — CodeMirror 6 default with a richer-editor escape hatch

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-architect`
Reviewer: orchestrator review-board
Linked design: `PLAN.md P0.2` and `AGENTS.md §1 — §10` invariants
(durable pointer list at the top of `AGENTS.md`).

## Context

We need a Markdown editor that is **lossless for what people write by hand
today**: YAML frontmatter, wikilinks, block refs, tables, embedded HTML,
comments, and unknown Markdown syntaxes. P0.2 specified a 200-note fixture and
a hard lossless round-trip requirement before any richer surface ships.

We compared three editor foundations:

1. **CodeMirror 6** with `@codemirror/lang-markdown` — modular, MIT,
   text-based editing with a decoration layer.
2. **BlockSuite** — block tree with Yjs underneath.
3. **Milkdown** — ProseMirror with Markdown plugins.

We also reviewed SilverBullet as a reference for self-hosted Markdown shells.

## Decision

Default to **CodeMirror 6**. It satisfies all hard requirements:

- Lossless byte-for-byte round-trip against the 200-note fixture
  (frontmatter, wikilinks, tables, HTML, comments, unknown syntax preserved).
- MIT license; no relay / no remote service.
- Decoration-based syntax; the canonical text remains Markdown on disk —
  the editor's internal model never becomes the source of truth.
- Trivial to expose a "raw Markdown" pane for any document the decoration
  layer cannot represent.

A **richer editor surface** can ship later if it proves lossless against the
fixture suite. The decision is reversible: the editor lives behind a
`EditorShell` interface and switching surfaces is an integration swap, not a
data migration.

## Alternatives considered

- **BlockSuite**: Very rich UX, but Yjs becomes the canonical structure
  concept, which conflicts with the file-truth invariant. Adapter cost is
  high.
- **Milkdown**: ProseMirror is powerful but its schema typically maps
  Markdown to a rich representation; lossless behavior is hard to maintain
  on round-trip with unknown syntaxes.
- **SilverBullet**: Reference only; pager model couples rendering and
  editing too tightly for our needs.

## Consequences

- `src/components/editor/` exposes a `EditorShell` interface with two
  implementations: `CodeMirrorShell` (default) and `RawMarkdownShell` (escape
  hatch).
- The acceptance fixture in `tests/editor/roundtrip.test.ts` runs the same
  200-note fixture against every shell.
- The MCP `read_document` returns the canonical text (`headmatter` +
  body) so MCP clients see the same view as the editor.

## Reversibility

- Swapping in a richer editor shell is a one-component change because the
  canonical text is on disk and only the editor reads it.
- If CodeMirror loses maintenance, the public module API is widely used, so
  forkability is cheap.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §1`.
- CodeMirror 6 modular architecture: `https://codemirror.net/docs/`.

## Threat-model cross-reference

- The editor never executes embedded JavaScript; embedded HTML is rendered
  through a sanitizer (per `threat-model.md` cross-site-scripting scenarios).
- The escape hatch always exists for poisoned documents.

## Acceptance evidence

- 200-note fixture round-trip diff is **zero bytes** for: frontmatter,
  wikilinks, block refs, tables (with alignment), embedded HTML, comments,
  unknown HTML data attributes, unknown tool markup.
- The fixture test lives at `tests/editor/roundtrip.test.ts` and is gated
  in CI before any editor surface ships.
