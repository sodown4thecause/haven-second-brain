---
name: haven-m1-orchestrator
description: Orchestration skill for Haven's M1 (Safe vault open) milestone. Use when the user is working P1.5–P1.8 deliverables, safe-existing-vault ADRs, PKM-UX surface contracts, Bases-lite derived-view ADRs, or any M1 acceptance gate. Owns the `_workspace/11_*.md` handoff contract for this milestone and the M1 exit evidence. Supersedes `haven-r0-orchestrator` once R0 is exit-passed.
---

# Haven M1 Orchestrator

## When to use

Use this skill whenever the user is asking for any **M1 (Safe vault open)**
deliverable on top of the already exit-passed R0:

- Spec for the safe-existing-vault mode
  (`docs/superpowers/specs/safe-existing-vault.md`).
- ADRs 008 / 009 / 010 (`editor-shell-completion`, `pkm-ux-surface`,
  `bases-lite-derived-view`) and any future M1 ADRs up to the M1 exit gate.
- The compatibility-report schema, the opt-in event semantics, and the
  dirty-worktree detector.
- The PKM-UX stub surface (file tree, backlinks, search, daily-note /
  journal) and its typed IPC contract tests.
- The Bases-lite derived-view contract and its round-trip-deletion test.
- The M1 section of `docs/research/threat-model.md` (side-loaded-vault
  adversary rows + safe-mode opt-in test).
- M1 exit evidence (`_workspace/11_phase1_finish_pr_evidence.md`).

**Do NOT use this skill** when the orchestrator should still be in R0 — i.e.
before `_workspace/09_r0_exit_evidence.md` carries the `## Review` block marked
`Status: reviewed-pass`. In that case load `haven-r0-orchestrator` instead.

## Required inputs

1. The approved design:
   `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`.
2. `AGENTS.md` (repo-wide invariants §1 — §10).
3. `docs/harness/haven/team-spec.md` (role topology unchanged from R0).
4. `PLAN.md` (Phase 1 P1.5 — P1.8 with their acceptance blocks).
5. The current contents of `_workspace/00_objective.md` (a sentence is enough).
6. R0 durable cross-references: `docs/adr/000-r0-exit-evidence.md` and
   `_workspace/04_adrs_index.md`.

## Pipeline (M1)

```
Load context
  └─ Confirm M1 is the safe-vault milestone; refuse requests for sync,
     memory, or local-model implementation (those are M2+).
Commit 01 — Skills
  └─ Add this SKILL.md so the "no production code lands until
     haven-m1-orchestrator exists" precondition from R0 is observable.
Commit 02 — Spec
  └─ docs/superpowers/specs/safe-existing-vault.md cites ADR-001, ADR-003,
     launch-workflows.md Workflow A, and the threat-model P1 row.
Commit 03 — Code
  └─ crates/haven-git/src/safe.rs implements the spec using existing
     OwnedAllowlist + OffTree + Identity primitives.
Commit 04 — Tests
  └─ crates/haven-git/tests/safe_existing_vault.rs opens a real
     Obsidian-shaped fixture in docs/fixtures/obsidian-readonly/ and
     proves zero mutations before opt-in.
Commit 05 — ADR
  └─ docs/adr/008-editor-shell-completion.md is decision-only; no Tauri
     code lands in this milestone.
Commit 06 — ADR
  └─ docs/adr/009-pkm-ux-surface.md carries the daily-note path policy
     (`journal/YYYY-MM-DD.md` per crates/okf reserved-filename list).
Commit 07 — Stubs
  └─ src/src/components/editor/{filetree,backlinks,search,journal}.ts
     implement the typed IPC contracts only; no real UI rendering.
Commit 08 — Tests
  └─ vitest contract tests for filetree / backlinks / search / journal
     run offline; one test per stub asserts the typed contract.
Commit 09 — ADR
  └─ docs/adr/010-bases-lite-derived-view.md pins the
     "delete-view-leaves-vault-intact" invariant from PLAN P1.8.
Commit 10 — Stubs + tests
  └─ src/src/components/bases/{view,query}.ts stubs and the
     round-trip-deletion vitest.
Commit 11 — Threat model + cross-link
  └─ docs/research/threat-model.md: add the P1 section covering
     side-loaded-vault adversary + safe-mode opt-in test.
    docs/superpowers/specs/alpha-success-metrics.md: cross-link the new
    ADRs so the safe-vault-open completion rate has a governing ADR.
Commit 12 — PR
  └─ _workspace/11_phase1_finish_pr_body.md + 11_phase1_finish_pr_evidence.md.
```

Each phase handoff is a markdown file in `_workspace/` named `11_*` so the
team-spec contract is preserved. The orchestrator only advances to the next
commit when the previous one carries its acceptance evidence.

## Outputs

- `.agents/skills/haven-m1-orchestrator/SKILL.md` (this file).
- `docs/superpowers/specs/safe-existing-vault.md`.
- `docs/adr/008-editor-shell-completion.md`,
  `docs/adr/009-pkm-ux-surface.md`,
  `docs/adr/010-bases-lite-derived-view.md`.
- `crates/haven-git/src/safe.rs` + `crates/haven-git/tests/safe_existing_vault.rs`.
- `src/src/components/editor/{filetree,backlinks,search,journal}.ts` + tests.
- `src/src/components/bases/{view,query}.ts` + tests.
- `docs/fixtures/obsidian-readonly/` (real Obsidian-style fixture).
- `docs/research/threat-model.md` P1 section.
- `docs/superpowers/specs/alpha-success-metrics.md` cross-link update.
- `_workspace/11_phase1_finish_pr_body.md` +
  `_workspace/11_phase1_finish_pr_evidence.md`.

## Specialist roster (M1)

- `haven-architect` — reviews each new ADR (mandatory; team-spec §3 producer-
  reviewer rule).
- `haven-threat-model` — updates the threat model for safe-vault (mandatory;
  team-spec §3).
- `haven-prior-art` — only if M1 introduces a new library; CodeMirror 6
  already in §1, `crates/haven-index` already in §6, so we expect zero
  reuse-adjudication calls in M1.
- `haven-r0-orchestrator` — NOT loaded for M1 production code; its own
  skill text defers to this skill post-R0 exit.
- `haven-safe-vault` / `haven-pkm-ux` / `haven-bases-lite` — NOT created in
  M1; one orchestrator skill keeps the context budget narrow. The M1
  pipeline is intentionally bounded (5 ADRs / 1 spec / 4 stub modules).

## Rippable rules

- Pipeline-specific adjustments belong in `.agents/skills/haven-m1-orchestrator/references/`.
- Never delete this skill without also deleting the branch convention
  `cursor/phase-1-finish-d85e` references (per AGENTS.md branch rules).
- If M2 (Trustworthy human writes) opens, create `haven-m2-orchestrator`;
  do not bolt the M2 pipeline onto this skill.

## Quick checks

- Does each new ADR cite prior-art (`docs/research/prior-art-register.md`),
  the launch workflow it serves (`docs/superpowers/specs/launch-workflows.md`),
  and the AGENTS invariant it backs?
- Does `safe-existing-vault.md` cite ADR-009 (the PKM-UX ADR) for the daily
  note policy and ADR-010 (the Bases-lite ADR) for derived projections?
- Does the safe-existing-vault integration test reopen the same fixture
  after `git status --porcelain` and assert zero diffs?
- Does the threat model P1 section include the side-loaded-vault adversary
  **and** the safe-mode opt-in test, both keyed to AGENTS §1 (files are
  source of truth) and §7 (provenance sacred)?
- Does the PR evidence doc reference every gate above?

## Open pre-conditions (do not skip)

- R0 must be exit-passed; if `_workspace/09_r0_exit_evidence.md` does not
  carry `Status: reviewed-pass`, halt and load `haven-r0-orchestrator`.
- The base branch for an M1 worktree must include at least
  `cursor/phase-1-redux-d85e` (the R0 foundation + the existing Phase 1
  redux crates); basing off `origin/main` pre-R0 will not compile.
