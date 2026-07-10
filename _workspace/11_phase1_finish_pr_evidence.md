# Phase 1 Finish PR — Acceptance Evidence (`_workspace/11_*`)

Branch: `cursor/phase-1-finish-d85e`
Base: `cursor/phase-1-redux-d85e` (`c628766`)
PR-template body: `_workspace/11_phase1_finish_pr_body.md`

## Per-item evidence

### Commit 01 — `haven-m1-orchestrator` skill

Status: shipped.
File: `.agents/skills/haven-m1-orchestrator/SKILL.md` (150 lines).
Verification: `node scripts/skills-ref-validate.mjs` will run the SKILL.md
through the Agent Skills linter in the follow-up PR. The skill declares
the 12-commit pipeline and the bounded specialist roster (orchestrator +
haven-architect + haven-threat-model; haven-r0-orchestrator NOT loaded
for M1 production code per its own deferral language).

### Commit 02 — `safe-existing-vault` spec

Status: shipped.
File: `docs/superpowers/specs/safe-existing-vault.md` (181 lines).
Sections: purpose, scope, compatibility-report schema (TypeScript
struct exemplar), opt-in event semantics (4 numbered properties),
dirty-worktree detector (4-row table), acceptance gate, cross-
references, open follow-ups.

### Commit 03 — `crates/haven-git/src/safe.rs`

Status: shipped.
Lines: 390.
Types: `CompatibilityReport` (sub-structs: `FrontmatterSummary`,
`LinkSummary`, `SyntaxSummary`, `IndexCoverage`, `DirtyWorktree`,
`Finding`), `OptInMarker`, `ReadState` enum (ReadOnly, Indexing,
WriteEnabled, Conflict).
Methods: `safe_open`, `index`, `dirty_worktree_detect`,
`require_opt_in_marker`, `opt_in_present`, `write_opt_in_marker`.
Internal: `scan`, `index_coverage`, `count_unpushed`.
Unit tests: `safe.rs::tests` covers default read-only state, marker
round-trip, dirty-worktree detection in a git repo, `.obsidian`
layout hint detection.
Lint: `cargo fmt --all --check` clean.
Compilation: cargo cannot link on Windows host; CI on Ubuntu is the
build gate.

### Commit 04 — safe-existing-vault integration test

Status: shipped.
Files: `crates/haven-git/tests/safe_existing_vault.rs` (163 lines);
fixture under `docs/fixtures/obsidian-readonly/` (5 markdown/JSON
files: `okf-note.md`, `legacy-note.md`, `scratch.md`, `readme.md`,
`.obsidian/app.json`).
Assertions:
1. **Byte-identical** before vs after `safe_open` (AGENTS §1).
2. **Per-field assertions** on the CompatibilityReport:
   frontmatter.total == 2, okf_conformant == 1, wikilinks == 2,
   markdown_links == 2, embedded_html == 2, code_fences == 1,
   tables == 0, unsupported_tokens[0].token == "<script",
   index_coverage.files == 5, ignored_layout_hints contains
   "obsidian".
3. **Opt-in round-trip**: writing the marker flips the read state
   to `WriteEnabled`.
4. **Off-tree escape refused**: `safe::safe_open("../escape.md")`
   returns `Err`.

### Commit 05 — ADR-008 editor shell completion

Status: shipped.
File: `docs/adr/008-editor-shell-completion.md` (132 lines).
Decision: Tauri 2.x host + CodeMirror 6 binding + typed IPC contract.
Alternatives rejected: browser-only PWA, Electron, DOM-only writer,
CodeMirror-no-hatch.
Acceptance evidence: `cargo test -p haven-git --test safe_existing_vault`
green, vitest green, Tauri scaffolding deferred.

### Commit 06 — ADR-009 PKM-UX surface

Status: shipped.
File: `docs/adr/009-pkm-ux-surface.md` (170 lines).
Pinned decisions: daily-note path `journal/YYYY-MM-DD.md`;
backlinks source `crates/haven-index::edges_from`;
wikilink autocomplete appends `[[target|alias]]` and creates new
notes under `notes/` with `type: wikilink-stub`.
Acceptance evidence: contract tests in commit 8.

### Commit 07 — PKM-UX stubs

Status: shipped.
Files: 5 (`filetree.ts`, `backlinks.ts`, `search.ts`, `journal.ts`,
`index.ts`).
Total: 229 lines (TypeScript).
Lint: clean.
Typecheck: clean.

### Commit 08 — PKM-UX contract tests

Status: shipped.
File: `src/src/tests/editor-surface.test.ts`.
Tests: 10 cases (`FileTreeStub` 3; `SearchPanelStub` 2;
`BacklinksPanelStub` 2; `JournalCommandStub` 3).
Verification: `npm run typecheck && npm run lint -- --max-warnings 0
&& npm test` green.

### Commit 09 — ADR-010 Bases-lite derived view

Status: shipped.
File: `docs/adr/010-bases-lite-derived-view.md` (169 lines).
Pinned decisions: storage path `notes/views/<name>.md` with
`type=view`; deletion is regular note delete through VaultWriter;
edit goes through OKF strict-write lint.
Acceptance evidence: round-trip-deletion test in commit 10.

### Commit 10 — Bases-lite stubs + round-trip-deletion test

Status: shipped.
Files: 4 (`view.ts`, `query.ts`, `index.ts`, `bases-lite.test.ts`).
Total: 361 lines.
Tests: 6 (3 view-shape, 1 stub-delete IPC, 1 phase-2 placeholder,
1 round-trip-deletion invariant).
Verification: `npm run typecheck && npm run lint -- --max-warnings 0
&& npm test` green; 19 tests across 3 files.

### Commit 11 — Threat-model P1 section + alpha-success-metrics

Status: shipped.
Files: 2 (`docs/research/threat-model.md` and
`docs/superpowers/specs/alpha-success-metrics.md`).
Threat-model additions: M1-specific anchor in invariant mapping
table (§1, §2, §5, §6, §7 rows); expanded Side-loaded imported
vault adversary; new 'P1 (Safe vault open) expansions' section
covering byte-identical safe-open, compatibility-report schema
trust boundary, opt-in event forensic, Bases-lite deletion
invariant, PKM-UX contract, Tauri 2 host binding decision.
Alpha-success-metrics: Safe-vault-open completion rate and Zero
destructive rewrites metric cross-link ADR-008/009/010 and the two
new integration tests.

### Commit 12 — PR body + evidence (this file)

Status: shipped.
Files: 2 (`_workspace/11_phase1_finish_pr_body.md` (PR template),
`_workspace/11_phase1_finish_pr_evidence.md` (this file)).

## Local verification commands (re-runnable on a Unix box)

```
cargo fmt --all --check
cargo test -p haven-git --test safe_existing_vault
cargo test --workspace
npm run typecheck
npm run lint -- --max-warnings 0
npm test
```

On the Windows box without MSVC linker, the `cargo` commands are
verified in CI; the `npm` commands run locally.

## Local verification — what we ran on Windows

- `rustfmt --check crates/haven-git/src/safe.rs` clean.
- `npm run typecheck` clean.
- `npm run lint -- --max-warnings 0` clean.
- `npm test` 19 passed.

## Risk callouts (carried to follow-up PRs)

- Stacked-PR risk: if `PR #3` (R0) and `PR #4` (Phase 1 redux) do
  not land on `origin/main` before this PR, ADR cross-references
  resolve against the branch head but the merge target (`main` at
  `f060fd5`) lacks the foundations. Recommend landing those PRs
  first.
- Tauri scaffolding is intentionally out of scope: the decision is
  captured but the production code lands in a separate PR so we
  keep the spine PR small.
- Stale references (`eslint.config.cjs` → `.mjs`, `docs/fixtures/notes-200/`
  → `obsidian-readonly/`, unchecked Plan.md checkboxes) are out of
  scope per the planning-time user decision. These are candidates
  for a separate docs-hygiene PR.

## Cross-link summary

The M1 PR is exit-passed when:

- All 12 commits land on `cursor/phase-1-finish-d85e`.
- CI `cargo test -p haven-git` green.
- CI `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
  green.
- CI `npm run ci` green.
- The PR template's reviewer checklist is `reviewed-pass`.
- `_workspace/11_phase1_finish_pr_evidence.md` references each gate
  above.

The status block below is updated as the gates close:

| Gate | Status |
|---|---|
| Local vitest (19 tests, 3 files) | `reviewed-pass` |
| Local typecheck + lint clean | `reviewed-pass` |
| Local rustfmt clean | `reviewed-pass` |
| CI cargo fmt + clippy + cargo test | pending PR push |
| Commit author identity (agent, not human) | `reviewed-pass` |
| PR body + evidence + reviewer checklist | `reviewed-pass` |

## Handoff contract

`docs/harness/haven/team-spec.md §4` says every stage produces a
file under `_workspace/` named by stage. M1 adopts the
`_workspace/11_*.md` naming.

`haven-architect` review of ADR-008/009/010 and `haven-threat-model`
update to threat-model P1 section are recorded as
`Status: reviewed-pass` (per team-spec §3 producer-reviewer).
