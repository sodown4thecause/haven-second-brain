# R0 — Restart foundation objective

Status: ready-for-review.
Phase: R0 (decision-only).
Branch: `cursor/rebuild-foundation-d85e`.
Driver: orchestrator (`haven-r0-orchestrator`).
Producer ledger: this file plus `_workspace/04_adrs_index.md` and
`_workspace/09_r0_exit_evidence.md`. Final artifacts live under
`docs/adr/`, `docs/superpowers/specs/`, and `docs/research/`.

## Problem brief

The pre-rebuild branch (`cursor/finish-phase-1-d85e`, commit `691e8d0`)
predates several governing decisions: editor selection, safe
existing-vault behavior, Git write policy, E2EE collaboration trust,
memory model, and web-research boundary. Phase-1 was implemented
against an earlier plan and is treated as a prototype, not as floor.

R0 produces the **decision-only** evidence bundle that lets M1
(`Safe vault open`) start without inventing architecture during
coding. R0 prohibits production feature code; disposable experiments
under `experiments/` may inform ADRs but cannot be imported by
production paths.

## Required outputs

- Concise, evidence-linked `PLAN.md` with `[tier:X]` directives.
- Durable repo-wide invariants in `AGENTS.md`.
- Three launch workflows with user-visible trust states.
- ADRs under `docs/adr/` for: OKF, editor round-trip, Git write
  policy, local runtime + network posture, sync/collaboration, memory
  engine, local model selection.
- Specs under `docs/superpowers/specs/` for: launch workflows,
  hardware model matrix, alpha success metrics, research selector.
- Research artifacts under `docs/research/`: prior-art register,
  threat model, knowledge-diff fixtures, longmem-eval bakeoff.
- Specialist skills under `.agents/skills/haven-*/` for every R0
  pipeline role.
- Team spec under `docs/harness/haven/team-spec.md`.
- Orchestrator skill `.agents/skills/haven-r0-orchestrator/SKILL.md`.

## Acceptance gate (R0 → M1)

R0 is exit-passed when `docs/adr/000-r0-exit-evidence.md` lists every
governing decision, every ADR cites prior-art or a bakeoff artifact,
and the three launch workflows are demoable with trust states. The
acceptance run is summarized in
`_workspace/09_r0_exit_evidence.md`.

## What is explicitly out of scope

- Production feature code.
- CI that depends on cargo or npm.
- Model downloads (first-run setup will benchmark before any pull).
- Silent remote API calls.

## Cross-references

- Approved design: `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`.
- Team spec: `docs/harness/haven/team-spec.md`.
- Exit evidence: `docs/adr/000-r0-exit-evidence.md`.
- ADR index: `_workspace/04_adrs_index.md`.
