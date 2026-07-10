---
name: haven-r0-orchestrator
description: Orchestration skill for Haven's R0 restart foundation. Use when the user is working on R0 ADRs, bakeoffs, specs, threat model, or any R0 deliverable. Owns the `_workspace/` handoff contract and the R0 exit-gate. Not for production implementation — R0 is decision-only.
---

# Haven R0 Orchestrator

## When to use

Use this skill whenever the user is asking for any R0 deliverable:

- ADRs (editor round-trip, OKF, Git write policy, local runtime, sync, local
  model, memory, E2EE collab, skill registry).
- Bakeoffs (sync, local-model, memory).
- Specs (launch workflows, hardware matrix, success metrics, research selector).
- Threat model.
- Prior-art and license register.
- R0 exit evidence and the R0 gate review.

Do not use this skill once the orchestrator has flagged "R0 exit-passed". At
that point the work belongs to `haven-m1-orchestrator` (which does not exist
yet and is out of scope for the current restart).

## Required inputs

1. The approved design: `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`
2. `AGENTS.md` (repo-wide invariants).
3. `docs/harness/haven/team-spec.md` (role topology).
4. `PLAN.md` (milestone map with the `[tier:X]` directives and acceptance
   blocks).
5. The current contents of `_workspace/00_objective.md` (a sentence is enough).

## Pipeline

```
Load context
  └─ Confirm R0 is decision-only; refuse any production feature code request.
Phase 1 — Prior-art
  └─ haven-prior-art writes docs/research/prior-art-register.md.
Phase 2 — Bakeoffs (Fan-out)
  ├─ haven-sync-bakeoff  → docs/adr/005-...
  ├─ haven-localmodel    → docs/adr/006-... + research-selector spec
  └─ haven-memory-bakeoff → docs/adr/007-...
Phase 3 — ADRs
  └─ haven-architect reviews every ADR; cross-references prior-art and bakeoffs.
Phase 4 — Specs
  ├─ docs/superpowers/specs/launch-workflows.md
  ├─ docs/superpowers/specs/hardware-model-matrix.md
  ├─ docs/superpowers/specs/alpha-success-metrics.md
  └─ docs/superpowers/specs/research-selector.md
Phase 5 — Threat model
  └─ haven-threat-model writes docs/research/threat-model.md.
Phase 6 — Skills
  └─ Update .agents/skills/* with evidence pointers.
Phase 7 — R0 exit
  └─ docs/adr/000-r0-exit-evidence.md references every gate.
```

Each phase handoff is a markdown file in `_workspace/` named by stage. The
reviewer writes a `## Review` block before the next stage begins.

## Outputs

- One ADR per governing decision under `docs/adr/`.
- One spec per product-system surface under `docs/superpowers/specs/`.
- One threat model and one prior-art register under `docs/research/`.
- `_workspace/09_r0_exit_evidence.md` cross-references every gate.

## Rippable rules

- Model-specific retries belong in `.agents/skills/haven-r0-orchestrator/references/`.
- Never delete this skill without deleting the team spec that depends on it.

## Quick checks

- Did the specialist answer "what decision is being made, what alternatives
  were considered, what evidence supports the choice"?
- Does each ADR cite the prior-art register and the bakeoff it depends on?
- Does each spec cite the ADR that sets its boundary?
- Does the threat model cite every product invariant from `AGENTS.md §1`?
