# ADR-009: PKM-UX surface — file tree, backlinks, search, daily note, wikilink autocomplete

Status: accepted (M1)
Date: 2026-07-11
Driver: `haven-m1-orchestrator`
Reviewer: `haven-architect`
Linked ADRs: `docs/adr/008-editor-shell-completion.md`, `docs/adr/001-okf-adoption.md`, `docs/adr/002-editor-roundtrip.md`, `docs/adr/010-bases-lite-derived-view.md`
Linked spec: `docs/superpowers/specs/safe-existing-vault.md`
Linked index crate: `crates/haven-index/src/lib.rs` (`search`, `edges_from`)
Linked AGENTS invariant: §1 (files are source of truth), §7 (provenance sacred)

## Context

`PLAN.md P1.6` requires the minimum PKM UX users expect on day one:
file tree, backlinks panel, global search, daily note creation,
wikilink autocomplete. The current branch provides typed IPC primitives
but no surface; this ADR pins the contracts so the PR stubs in commit 7
land without re-litigating them.

Three decisions are gated by this ADR:

1. The **daily-note path policy** (`journal/YYYY-MM-DD.md`).
2. The **backlinks source** (`crates/haven-index` `edges_from`, not a
   per-document scan).
3. The **wikilink autocomplete** write policy (OKF `type: wikilink` or
   equivalent).

## Decision

### File tree

The file tree surface walks the canonical vault root in
`OwnedAllowlist::default_vault()` (`crates/haven-git/src/lib.rs:82-90`).
Files under `.git/` and `.haven/` are excluded. Hidden directories are
surfaced when they appear in `OwnedAllowlist::default_vault()` (e.g.
`.obsidian/`); they are NOT surface-rewritten.

The tree updates incrementally through `crates/haven-index::upsert`
events. No client-side indexing.

### Backlinks panel

Source is `crates/haven-index::edges_from` — derived edges stored in the
SQLite `edge` table. Backlinks are recomputed on watcher events
(`haven-index::upsert` + `haven-index::delete`); the panel never reads
the user filesystem directly to compute backlinks. This keeps the
backlinks surface fast and derived.

When `crates/haven-index` is empty (e.g. immediately after `rm -rf
.haven/`), backlinks return an empty list; the UI shows a `rebuild
index` affordance. AGENTS §1 (file-truth) overrides any cached
backlinks: the index is rebuildable from canonical files at any time.

### Global search

Source is BM25 (FTS5) via `crates/haven-index::search`. Vector recall is
deferred to Phase 2 (per `PLAN.md P2.4`). The search UI ships as a
typed contract that returns `{ path, snippet, score }` records; clicking
a result navigates to the source line.

### Daily note / journal

Path: `journal/YYYY-MM-DD.md` (regex `^\d{4}-\d{2}-\d{2}\.md$`). The
folder `journal/` is allowlisted in
`OwnedAllowlist::default_vault()`. Filename `YYYY-MM-DD.md` is **not** in
OKF's reserved-filename list (`RESERVED_INDEX`, `RESERVED_LOG` per
`crates/okf/src/lib.rs:34-35`), so a daily note is `type: note` or has
no frontmatter at all.

The `journal/YYYY-MM-DD.md` file is created only if missing (idempotent;
the `journal` command is offline-safe). On create, Haven writes an OKF
header matching `crates/okf::Frontmatter::default_with_type_note()` so
strict-write accepts it. WIKILINK to daily note uses `[[YYYY-MM-DD]]`
form (no `journal/` prefix); the wikilink resolver prefixes
`journal/` from the lookup rather than the disk path.

### Wikilink autocomplete

When the user types `[[`, the autocomplete reads the file tree, the
backlinks panel, and the daily-note index. Suggested entries surface
titles, file paths, and recent-uses. On selection, the link is appended
as `[[target|alias]]` (Obsidian-compatible) and the target is created
lazily on first click if it doesn't exist.

When the user invokes `[[new-note-title]]`, Haven creates the note under
`notes/` (not in `journal/`) and writes OKF frontmatter with
`type: wikilink-stub` so strict-write accepts it and a later migration
can flatten the stub into a real link.

### Linked AGENTS invariants

- §1 (files are source of truth): every surface reads from a
  derived projection (the SQLite index) and falls back to a rebuild on
  `.haven/` deletion.
- §7 (provenance sacred): any surface-triggered file create or edit
  flows through `crates/haven-git::VaultWriter` so the human or agent
  identity is recorded in Git.

## Alternatives considered

- **Daily note at `YYYY-MM-DD.md` (vault root)**: rejected. Pollutes the
  vault root and conflicts with the file tree UX.
- **Daily note under a folder chosen per user**: rejected for v1. The
  constant `journal/` keeps Bases-lite's saved filters predictable; a
  per-user folder is a follow-up setting.
- **Backlinks from a per-document scan at panel-open**: rejected. The
  incremental index already maintains the `edge` table; doing the work
  twice is the wrong shape.
- **Wikilink as inline HTML or a custom token**: rejected. Plain
  `[[...]]` survives round-trip with `crates/okf`'s permissive-read
  path and the CodeMirror 6 Markdown language pack already recognizes
  the link.

## Consequences

- The spine stub in commit 7 implements typed contracts only: a
  `JournalCommandStub::today()`, `BacklinksPanelStub::forPath(path)`,
  `SearchPanelStub::query(q)`, `FileTreeStub::roots()`. Each returns a
  typed result; the integration tests in commit 8 verify the contract.
- The wikilink autocomplete does NOT modify the OKF spec for v1.
  `[[target]]` remains a plain link in the on-disk Markdown; the
  `type: wikilink-stub` frontmatter only appears on **newly created
  notes through autocomplete** to keep strict-write acceptance unique
  to that creation path.

## Reversibility

- Switching the journal folder name is a configuration flip plus a
  document migration tool. The ADR-009 ADR is the durable decision, not
  the code.
- Replacing backlinks source with a per-document scan is a single
  crate change in `crates/haven-index`; consumers are typed through
  `lib.rs`, so no surface needs to change.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §1 (lossless editor and property
  UX)`: CodeMirror 6's Markdown lang pack handles wikilinks via the
  link
  decoration extension; no new dependencies introduced.
- `docs/research/prior-art-register.md §2 (importers)`: Obsidian and
  Logseq daily-note paths and naming conventions are reviewed for this
  ADR.

## Threat-model cross-reference

- `docs/research/threat-model.md` P1 section: the wikilink autocomplete
  does not cause writes outside `OwnedAllowlist`; the daily-note create
  is allowlisted to `journal/`.
- AGENTS §2 (OKF strict-write permissive-read): newly created daily notes
  pass strict-write because they carry `type: note`; the wikilink-stub
  variant is documented for future migration.

## Acceptance evidence (commit 8 backbone)

- `npm run typecheck && npm run lint -- --max-warnings 0 && npm test`
  green; the contract tests exercise:
  - `journal/2026-07-11.md` is created on `JournalCommandStub::today()`
    call.
  - `SearchPanelStub::query("OKF")` returns the OKF-conformant fixture
    path with a non-empty snippet.
  - `BacklinksPanelStub::forPath("okf-note")` returns the legacy-note
    path under the fixture (since legacy-note links to a missing note).
  - `FileTreeStub::roots()` returns the canonical allowlist roots.

## Out of scope (deliberately deferred)

- Tauri host wiring (`src-tauri/`) — gated on `ADR-008` follow-up PR.
- Vector recall / hybrid (RRF) search — Phase 2 per `PLAN.md P2.4`.
- Per-user journal folder override — explicit even-lower priority.
