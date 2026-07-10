# R0 Exit Evidence (orchestrator scratchpad)

This is the orchestrator's working copy of the R0 exit gate artifact.
The durable copy is [`docs/adr/000-r0-exit-evidence.md`](../docs/adr/000-r0-exit-evidence.md).
This scratchpad exists so the next orchestrator run starts from a
checkpoint without having to re-derive the cross-reference graph.

## Status

- Prior-art register: `Status: reviewed-pass` (driver `haven-prior-art`, reviewer `haven-architect`).
- Sync bakeoff → ADR-005: `Status: reviewed-pass` (driver `haven-sync`, reviewer `haven-architect`).
- Local-model bakeoff → ADR-004 + ADR-007 + `research-selector.md`: `Status: reviewed-pass`
  (driver `haven-localmodel`, reviewer `haven-architect`).
- Memory bakeoff → ADR-006: `Status: reviewed-pass`
  (driver `haven-memory`, reviewer `haven-architect`).
- Three launch workflows → `launch-workflows.md`: `Status: reviewed-pass`
  (driver `haven-architect`, reviewer orchestrator review-board).
- Hardware model matrix (tier policy + schema): `Status: reviewed-pass`;
  per-tier benchmark numbers: `Status: deferred-to-M1` —
  `docs/research/hardware-benchmarks.md` is created and populated in M1
  and is the gate that allows the first model pull.
- Alpha success metrics: `Status: reviewed-pass`.
- Threat model: `Status: reviewed-pass` (driver `haven-threat-model`, reviewer `haven-architect`).
- Skills under `.agents/skills/haven-*/`: `Status: reviewed-pass`.

## Acceptance for R0 → M1

- [x] Approved design present (`docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`).
- [x] `AGENTS.md` carries the durable invariants list (§1 — §10) at the top.
- [x] Restart boundary observable: pre-rebuild commit `691e8d0` preserved in
      history; rebuild branch carries only the new scaffolding.
- [x] Every ADR cites prior-art and the relevant spec.
- [x] Every spec cites its boundary ADR.
- [x] Threat model covers every `AGENTS.md` invariant (§1 — §10).
- [x] Three launch workflows exist with user-visible trust states.
- [x] Specialist skills reinforce the team spec at
      `docs/harness/haven/team-spec.md`.
- [ ] Per-tier benchmark numbers in `docs/research/hardware-benchmarks.md`
      populated for Floor / Default / Quality / Headroom on Windows 16 GB,
      Apple Silicon 16/18 GB, Linux 16 GB reference machines. **M1 gate.**

## Open work after R0 lands

- The `haven-m1-orchestrator` skill is deliberately not yet created.
  Loading `haven-r0-orchestrator` after this commit gets the answer
  "do not use this skill once R0 is exit-passed." The next planning
  target is **M1 — Safe vault open**. The M1 orchestrator skill will
  be added alongside the M1 implementation plan under
  `docs/superpowers/plans/m1-safe-vault-open.md` (see
  [_workspace/pr_body.md](./pr_body.md) — "What's next").

## Review block (orchestrator review-board)

## Review

- Status: `reviewed-pass`. Every governing decision has an ADR with a
  link to the prior-art register and the relevant spec. Threat model
  is mapped to every `AGENTS.md` invariant. The R0 phase gate can
  open. M1 planning may begin, but no production code lands until
  the `haven-m1-orchestrator` skill is added.

## Sign-off

- Reviewed by: orchestrator review-board (on behalf of `haven-architect`).
- Date: 2026-07-11.
- Decision: open PR for human review; do not auto-merge (founder policy).
