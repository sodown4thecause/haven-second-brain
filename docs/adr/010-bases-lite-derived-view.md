# ADR-010: Bases-lite — derived property views (deleting a saved view leaves the vault intact)

Status: accepted (M1)
Date: 2026-07-11
Driver: `haven-m1-orchestrator`
Reviewer: `haven-architect`
Linked ADRs: `docs/adr/001-okf-adoption.md`, `docs/adr/009-pkm-ux-surface.md`, `docs/adr/008-editor-shell-completion.md`
Linked spec: `docs/superpowers/specs/safe-existing-vault.md`
Linked launch workflow: `docs/superpowers/specs/launch-workflows.md` Workflow A
Linked AGENTS invariant: §1 (files are source of truth), §2 (OKF strict-write permissive-read)

## Context

`PLAN.md P1.8` requires the minimum object/property UX users now expect:
saved filters over YAML frontmatter and backlinks surfaces as table/list
views. The cross-cutting constraint from `PLAN.md:357` is that
**deleting a saved view must leave the vault working**. Bases-lite is
**not** Notion-database parity (`PLAN.md:65`); it is a derived
projection over canonical files. This ADR pins that boundary.

Three decisions are gated here:

1. Saved views live as **notes under `notes/`** with `type: view`
   frontmatter, not as a hidden SQLite registry.
2. The view's **deletion is a vault file deletion**, not a soft delete;
   the vault re-derives from canonical files on next index.
3. Edit operations on view sources use the same `crates/haven-git`
   writer as every other edit; saved views cannot promote themselves
   into a "second source of truth".

## Decision

### Storage path

The default saved-filter note ships at `notes/views/<name>.md` with
frontmatter:

```yaml
---
okf_version: v0.1
type: view
view_kind: table        # table | list | kanban (list is the only MVP variant)
filter_yaml: |
  type = note AND tags ⊆ ["drafts"]
sort: updated_at desc
---
```

The note body may carry a Markdown table or list starter that the view
editor renders in `indexing`/`write-enabled` states. Notes under
`notes/views/` are quiet (no FTS indexing noise) but findable via the
`*` glob on the file tree.

### Derived projection

A saved view runs at view-open time. It is **not** a persistent index
table. The execution path:

1. Read the file `notes/views/<name>.md`.
2. Parse `frontmatter.filter_yaml` via `crates/okf::parse` (so OKF
   round-trip is verified for the query before evaluation).
3. Walk canonical allowlist roots via `crates/haven-index::search` +
   per-file `crates/okf::parse` for filter evaluation.
4. Render the result as a table or list, with column sorts taken from
   `frontmatter.sort`.

The view surfaces never read or write `.haven/`. They are deterministic
projections; removing one is a regular note delete that AGENTS §1
intends.

### Deletion invariant

Deleting `notes/views/<name>.md`:

1. Triggers `crates/haven-git::VaultWriter` with the human identity.
2. The view disappears from the file tree immediately (next index
   `upsert`/`delete` event).
3. No other files are touched. The vault's notes, daily journal, skills,
   memory observations, and inputs — none are modified by the deletion.

The contract test in `bases-lite.test.ts` (commit 10) creates a saved
view, snapshots the canonical allowlist file set, deletes the view,
compares the snapshot, and asserts byte-equal.

### Edit semantics

Editing the saved view's filter is a regular editor save flow:

1. `EditorShell.save()` →
   `crates/okf::lint_strict_write` (rejects invalid YAML or unknown
   keys) →
   `crates/haven-git::VaultWriter` →
   `crates/haven-index::upsert`.

The view cannot mass-edit the matched files; it is a filter, not a
bulk-edit tool. A bulk edit is the diff-preview MCP flow (Phase 2).

## Alternatives considered

- **SQLite-only saved views**: rejected. Adds a second source of truth
  for the view definition; the user can `git log` a Markdown file but not
  a SQLite row.
- **`.haven/views.json`**: rejected. Same problem as above plus the
  path is not under `OwnedAllowlist::default_vault()`.
- **Saved views as a phase-3 marketplace plugin**: rejected. The
  founder dogfood needs the surface now (per `PLAN.md:870-873`).
- **Bases-lite as full Notion DB parity (column types, formulas)**:
  rejected. `PLAN.md:65-66` lists "full object database parity with
  Notion" as a non-goal.

## Consequences

- The spine stub in commit 10 (we ship a `ViewDefinition` shape and a
  round-trip-deletion vitest) proves the contract axis. Real evaluator
  implementation lands in a follow-up PR after the OKF-extended query
  language is scoped.
- Editorial UX ties to ADR-008: the editor shell hosts the YAML
  frontmatter editor with strict-write lint at save time.
- The safe-existing-vault compatibility report stores the count of
  `notes/views/` files under `view_kind` (added in `crates/haven-git/src/safe.rs`
  in commit 11's threat-model-and-spec update). Existing-vault-open
  users see "0 saved views" until they author one.

## Reversibility

- Switching the storage path from `notes/views/` to a JSON registry is
  a migration that must honour the deletion invariant and pass the
  round-trip-deletion test once more.
- Adding `kanban` as a `view_kind` is a follow-up PR; the `view_kind`
  enum and the frontmatter parser accept any value not in `[table,
  list, kanban]` as `list` (permissive-read fallback per ADR-001).

## Prior-art cross-reference

- `docs/research/prior-art-register.md §6 (vector/RAG)`: the view
  evaluator follows the same frugal-shape approach as `sqlite-vec`
  adoption (no extra runtime when not instantiated).
- `docs/research/prior-art-register.md §2 (importers)`: the YAML
  `filter_yaml` shape mirrors Obsidian Bases' filter syntax in
  shape, but evaluation is local-only.

## Threat-model cross-reference

- `docs/research/threat-model.md` P1 section: warned-path rows
  (`<script>`, `<iframe>`) in a saved view frontmatter are surfaced in
  the M1 P1 section's threat rows; the OKF lint at save time strips
  unsupported tokens via the same path-detector that the safe-existing-
  vault compatibility report uses.
- AGENTS §2: `filter_yaml` is parsed through `crates/okf::parse` with
  permissive-read fallback; strict-write on save means unknown keys are
  rejected.

## Acceptance evidence (commit 10 backbone)

- `npm test -- --testPathPattern bases-lite` green; the round-trip-
  deletion vitest asserts no other file changes when a saved view is
  deleted.
- The plan-and-spine-PR does not yet include a real evaluator;
  `ViewDefinition.execute()` returns a `not-implemented` sentinel
  marked `requires(Phase 2)`.

## Out of scope (deliberately deferred)

- Real evaluator (full filter YAML runtime) — gated on a phase-2 PR
  that defines the query language.
- Kanban view kind.
- Cross-vault view sharing.
- View export/import as a separate file (single-file YAML round-trip is
  enough for v1).
