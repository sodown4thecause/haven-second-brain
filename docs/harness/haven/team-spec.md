---
name: haven-haven-team-spec
description: Role topology, handoff contract, and failure policy for Haven's R0 restart foundation. Loaded by no agent directly — it is the durable reference for `.agents/skills/haven-r0-orchestrator` and each specialist skill. Update only when a role boundary or handoff contract changes.
---

# Haven R0 Team Spec

## 1. Purpose

Approved design:
`docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`

R0 (restart foundation) is decision-only. Its deliverable is the set of
**evidence-linked** ADRs, specs, bakeoffs, threat model, hardware matrix, and
prior-art register that allow M1 (Safe vault open) to be planned without making
architecture decisions during coding.

## 2. Architecture Pattern

Pipeline with a bounded Fan-out/Fan-in for the three bakeoffs.

- Pipeline stages run in order: **Prior-art → Bakeoffs → ADRs → Specs → Skills
  → Threat model → R0 exit evidence.**
- Fan-out applies inside the **Bakeoffs** stage: `sync`, `local-model`,
  `memory` bakeoffs run in parallel branches; the Fan-in is the corresponding
  ADR.
- Producer-Reviewer applies to every ADR: the writer is the bakeoff
  specialist; the reviewer is `haven-architect`.

Why this shape: each M1-up design must cite its bakeoff; each bakeoff cites
its prior-art review. A flat pipeline keeps that dependency chain visible and
keeps each specialist's context budget small.

## 3. Roles

| Role | Skill | Pipeline stage | Input | Output | Reviewer |
| --- | --- | --- | --- | --- | --- |
| Orchestrator | `haven-r0-orchestrator` | whole pipeline | spec, team spec | phase handoffs, R0 exit evidence | human |
| Prior-art clerk | `haven-prior-art` | Prior-art | spec, GitHub links | `docs/research/prior-art-register.md` | `haven-architect` |
| Sync architect | `haven-sync` | Bakeoffs | prior-art register | `docs/adr/005-sync-collaboration.md` | `haven-architect` |
| Local-model architect | `haven-localmodel` | Bakeoffs | prior-art register, hardware matrix | `docs/adr/004-local-runtime-and-network-posture.md`, `docs/adr/007-local-model-selection.md`, `docs/superpowers/specs/research-selector.md` | `haven-architect` |
| Memory architect | `haven-memory` | Bakeoffs | prior-art register | `docs/adr/006-memory-engine.md` | `haven-architect` |
| Research-selector spec author | `haven-research-selector` | Specs | local-model ADR | `docs/superpowers/specs/research-selector.md` | `haven-architect` |
| Architect / ADR reviewer | `haven-architect` | ADRs + Specs | all bakeoffs | signed ADRs under `docs/adr/` | orchestrator |
| Threat-modeler | `haven-threat-model` | Threat model | all ADRs, prior-art | `docs/research/threat-model.md` | `haven-architect` |
| Hardware matrix owner | (folded into `haven-architect`) | Prior-art | prior-art register | `docs/superpowers/specs/hardware-model-matrix.md` | orchestrator |

Role briefs (the durable portable view) already exist for every specialist
through their `SKILL.md`; this team spec does not duplicate them. Add a brief
under `docs/harness/haven/roles/` only if a role needs a stable narrative that
does not fit a skill.

## 4. Handoff Contract

Every phase lands its **durable artifact** in the canonical path
(`docs/adr/`, `docs/superpowers/specs/`, `docs/research/`, or
`.agents/skills/`). The orchestrator's scratchpad in `_workspace/` carries the
running pipeline state — objective brief, ADR index, status trackers, PR
body draft — not the durable content.

```
_workspace/                            # orchestrator scratchpad only
  00_objective.md                       # problem brief; orchestrator owns.
  04_adrs_index.md                      # cross-reference of docs/adr/.
  09_r0_exit_evidence.md                # status tracker for the gate.
  pr_body.md                            # PR description draft.
docs/adr/                               # durable ADRs (incl. 000-r0-exit-evidence.md)
docs/superpowers/specs/                 # durable specs.
docs/research/                          # durable research artifacts.
.agents/skills/haven-*/SKILL.md        # durable specialist skills.
```

Stage transitions require the producer to mark `Status: ready-for-review`
on the durable artifact and the reviewer to mark `Status: reviewed-pass` or
`Status: reviewed-fix`. The orchestrator only advances when the durable
artifact (not a scratchpad stub) is `reviewed-pass`, and it records the
status in `_workspace/09_r0_exit_evidence.md`.

## 5. Failure Policy

| Failure | Action |
| --- | --- |
| Bakeoff candidate cannot satisfy the no-open-port invariant | reject the candidate to adoption; record reasoning; ADR inherits the rejection. |
| Bakeoff recommends adding a Docker/Python service or second canonical DB | override the bakeoff; record the override reason in the ADR; treat the recommendation as advisory, not authoritative. |
| ADR contradicts an existing ADR | writer must reconcile or mark one superseded; reviewer blocks advancement until reconciled. |
| Threat model surfaces a risk that the chosen candidate cannot mitigate | threat-modeler raises a blocker; orchestrator must either choose a different candidate or add a tracked residual-risk item in the ADR. |
| Specialist exceeds context budget on a stage | orchestrator splits the stage and revises `_workspace/` filenames; never silently pads the prompt. |
| Reviewer cannot reach a verdict | orchestrator schedules a tie-breaker specialist with explicit scope. |

Rippable harness rule: model-specific retries and recovery heuristics live in
the orchestrator skill or `.agents/skills/*/references/*`. They are easy to
delete without rewriting this team spec.

## 6. Acceptance Gates

R0 exit requires all of the following to be `Status: reviewed-pass` and
referenced from `_workspace/09_r0_exit_evidence.md`:

- `docs/adr/000-r0-exit-evidence.md` exists and lists every governing
  decision.
- `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md` (approved
  design) is referenced as the source.
- The OKF v0.1, editor round-trip, Git write policy, and local runtime / network
  posture ADRs (`docs/adr/001..004`) carry acceptance evidence.
- The sync, local-model, and memory bakeoffs each produce an ADR with a
  concrete adopt / fork / reimplement decision.
- Static research selector and Knowledge Diff evaluation spec exists.
- Threat model exists and links to every stated product invariant.
- Hardware + model support matrix exists with schema + tier policy;
      per-platform reference measurements live in
      `docs/research/hardware-benchmarks.md` (created in M1, gated on
      the first model pull).
- Prior-art and license register covers every "hard feature" category from the
  approved design.
- Old implementation code is removed from this branch but preserved in git
  history (commit `691e8d0`).
- Three launch workflows exist with user-visible trust states and measurable
  acceptance criteria.

## 7. Restart Boundary

Preserve commit `691e8d0` and the pre-rebuild branch as references. Inventory
the dirty worktree before any reset. Never silently absorb unrelated user
changes. Disposable experiments live under `experiments/` and may not be
imported by production code.
