# Phase 1 Finish — PR Body

Branch: `cursor/phase-1-finish-d85e`
Base: `cursor/phase-1-redux-d85e` (`c628766`)
Worktree: `../second-brain--p1-finish`

## Summary

Finishes Phase 1 (P1.5 — P1.8) on top of the R0 foundation and the
existing Phase 1 redux crates. Follows the M1 orchestrator pattern
declared in `_workspace/00_objective.md` ("M1 planning may begin, but
no production code lands until the `haven-m1-orchestrator` skill is
added"); the first commit in this PR adds the skill and the rest
becomes admissible under the team-spec producer-reviewer rule.

Delivers decisions, skeleton code, and contract tests. The follow-up
PR will land the actual Tauri host binding and any UX polish; the
M1 PR is decision + spine only.

## Stacked-PR context

`PR #3 (R0)` and `PR #4 (Phase 1 redux)` should land first so the
M1-finish PR cross-references resolve cleanly in CI. The M1-finish
PR is additive on top of those — it does not re-litigate scaffold
choices.

## Acceptance gate matrix (per `PLAN.md P1.x`)

| Item | Implementation | Acceptance command | Evidence |
|---|---|---|---|
| P1.5 editor shell completion (decision only) | `ADR-008` | decision captured; Tauri code deferred | `docs/adr/008-editor-shell-completion.md` |
| P1.6 PKM-UX surface (file tree, backlinks, search, daily note, wikilink autocomplete) | `ADR-009` + stubs under `src/src/components/editor/` | `npm test` green (10 cases) | `src/src/tests/editor-surface.test.ts` (plus the post-PR-prep fixup that swaps the regex query for a literal FTS5 substring) |
| P1.7 Safe existing-vault mode | `ADR-009` daily-note policy + `ADR-010` view policy + spec + Rust skeleton + integration test | `cargo test -p haven-git --test safe_existing_vault` (CI) | `crates/haven-git/tests/safe_existing_vault.rs` + `docs/fixtures/obsidian-readonly/` |
| P1.8 Bases-lite property views (derived view, delete-leaves-vault-intact invariant) | `ADR-010` + stubs + vitest | `npm test -- --testPathPattern bases-lite` green (6 cases including round-trip-deletion) | `src/src/tests/bases-lite.test.ts` |

## Cross-references to R0

| M1 surface | R0 anchor ADR | R0 spec | Threat-model row |
|---|---|---|---|
| `safe.rs` skeleton | ADR-001, ADR-003 | `safe-existing-vault.md` (new in this PR) | §1, §2, §5, §7 invariants |
| Editor stubs | ADR-002, ADR-003 | `launch-workflows.md` Workflow A | §5 invariants |
| Bases-lite stubs | ADR-001 | `launch-workflows.md` Workflow A step 5 | §1 invariants |
| PR summary | `00-r0-exit-evidence.md` | — | — supersession |

## Local verification (this PR)

(The local Windows box has no MSVC `link.exe` / MinGW `gcc`; rust
build artifacts fix in CI.)

- `cargo fmt --all --check` clean (after `rustfmt` over the new files).
- `npm run lint -- --max-warnings 0` clean.
- `npm run typecheck` clean.
- `npm test` green: 19 tests across three test files (3 roundtrip +
  10 editor-surface + 6 bases-lite).
- `cargo test` cannot run locally (no MSVC linker); CI matrix on
  Ubuntu is the gate (per prior `_workspace/10_phase1_pr_evidence.md`).

## Open follow-up PRs (deliberately not in this PR)

1. Tauri 2 host scaffolding (`src-tauri/`).
2. Real Bases-lite query evaluator (phase 2).
3. Editor surface rendering (rich surface binding; this PR ship a
   typed contract only).
4. The original `R0` PR and the prior `Phase 1 redux` PR still
   need to land on `origin/main`; recommend doing that before
   pushing this PR to keep cross-references stable.

## Reviewer checklist (per `haven-architect`)

- [ ] `docs/adr/008-editor-shell-completion.md` captures the Tauri
      host pick and the reasons why the alternatives were rejected.
- [ ] `docs/adr/009-pkm-ux-surface.md` carries the daily-note path
      policy and the wikilink autocomplete contract.
- [ ] `docs/adr/010-bases-lite-derived-view.md` pins the deletion
      invariant and the storage path.
- [ ] `docs/superpowers/specs/safe-existing-vault.md` is the bound
      contract for `crates/haven-git/src/safe.rs`.
- [ ] `crates/haven-git/tests/safe_existing_vault.rs` proves zero
      mutations before opt-in against a real Obsidian-shaped
      fixture.
- [ ] `src/src/tests/bases-lite.test.ts` proves the round-trip-
      deletion invariant.
- [ ] `src/src/tests/editor-surface.test.ts` exercises the typed
      contracts against a mock IPC.
- [ ] `docs/research/threat-model.md` M1 P1 section references each
      PR-delivered test.
- [ ] No PR commits mix human and agent authorship; each commit is
      authored as `Haven Agent (minimax-m3) <agent@haven.local>` in
      line with the agent-commits rule from `docs/adr/003-git-write-policy.md`.

## Branch lifecycle

- Merge strategy: **squash-merge** is the founder's preferred mode so
  the PR title appears as one commit on `main` and the agent commits
  don't leak into the durable history.
- Release tag: none in this PR.
- Backout safety: every artifact is additive; reverting the merge
  leaves the R0 + Phase 1 redux foundation unchanged.
