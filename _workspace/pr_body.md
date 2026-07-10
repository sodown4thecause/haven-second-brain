## Summary

R0 (restart foundation) is decision-only. This PR delivers the
evidence-linked bundle required to plan M1 (`Safe vault open`).

Follows the approved design at
`docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`.

## Restart boundary

- `cursor/finish-phase-1-d85e` is preserved as the pre-rebuild
  reference (commit `691e8d0`).
- `cursor/rebuild-foundation-d85e` carries only the new R0 scaffolding.
- The dirty worktree was inventoried before reset; no unrelated user
  changes were absorbed.

## What changed

### Repo-wide invariants

`AGENTS.md` now carries the durable numbered invariants list (§1 —
files are the only source of truth; §2 — OKF v0.1 conformant writes
on the desktop, permissive on read; §3 — local-model default; §4 —
internet first-class; §5 — no inbound network ports; §6 — license
posture with Apache-2.0 desktop app and AGPL-3.0 self-hostable relay;
§7 — provenance sacred; §8 — relay cannot decrypt; §9 — models get
only typed tools; §10 — offline-only edits queue and reconcile on
reconnect). `docs/research/threat-model.md` and the ADR/spec
cross-references now use this flat numbering.

### Plan

- `PLAN.md` slimmed to the milestone map with `[tier:X]` directives
  and acceptance blocks per task.
- Old implementation, manifests, lockfiles, and CI removed.

### Harness scaffolding

- `docs/harness/haven/team-spec.md` — role topology, handoff
  contract, failure policy, and R0 acceptance gates.
- `.agents/skills/haven-r0-orchestrator/SKILL.md` — orchestrator for
  the R0 pipeline.
- `.agents/skills/haven-{architect,prior-art,sync,localmodel,memory,
  research-selector,threat-model}/SKILL.md` — specialist skills.

### ADRs (`docs/adr/`)

- `001-okf-adoption.md` — OKF v0.1 strict-write, permissive-read.
- `002-editor-roundtrip.md` — CodeMirror 6 default with raw Markdown
  escape hatch.
- `003-git-write-policy.md` — dual-identity commits, isolated
  staging, atomic replace, expected-hash, side-by-side conflicts.
- `004-local-runtime-and-network-posture.md` — Ollama loopback only,
  no silent cloud calls, no inbound ports, URL allowlist.
- `005-sync-collaboration.md` — encrypted Git-envelope relay, native
  Rust relay, Syncthing-style conflicts, any-sync-style spaces.
- `006-memory-engine.md` — file-native canonical + optional
  Hindsight/Mem0/Graphiti adapters.
- `007-local-model-selection.md` — Qwen 3.5 4B Q4 default, Gemma E4B
  Q4 quality alternate, embedding fallback matrix.
- `000-r0-exit-evidence.md` — consolidates the evidence; R0 gate
  checklist.

### Specs (`docs/superpowers/specs/`)

- `launch-workflows.md` — three alpha workflows (safe-existing-vault,
  cited recall, approved MCP patch) with user-visible trust states.
- `hardware-model-matrix.md` — floor/default/quality/headroom tiers,
  16 GB runtime budget, honesty bar.
- `alpha-success-metrics.md` — activation, setup, interop gates.
- `research-selector.md` — static ResearchIntent enum, deterministic
  policy router, Knowledge Diff classification, safety contracts.

### Research (`docs/research/`)

- `prior-art-register.md` — every hard-feature category with license
  verdicts, reusable modules, and adopt/fork/adapt/reimplement
  decisions. New §11 covers Ollama + llama.cpp + cloud providers +
  SearXNG/Firecrawl/Crawl4AI/Playwright bakeoff.
- `threat-model.md` — invariant-to-threat mapping, adversary
  inventory, cross-cutting prompts-injection / secret / imported-HTML
  defenses.
- `knowledge-diff-fixtures.md` — fixture shape for novel,
  corroborating, conflicting, and uncertain truth cases.
- `longmem-eval-bakeoff.md` — companion artifact to ADR-006; numeric
  results fill in once M6 is plan-eligible.

### Orchestrator scratchpad (`_workspace/`)

- `00_objective.md` — problem brief and acceptance contract.
- `04_adrs_index.md` — cross-reference: ADR ↔ prior-art row ↔ spec ↔
  threat-model row.
- `09_r0_exit_evidence.md` — orchestrator's working copy of the gate
  evidence plus status trackers.
- `pr_body.md` — this file.

## Acceptance gate

R0 exit test green when:

- [x] Approved design referenced from this PR.
- [x] Every governing decision has an ADR; each ADR cites the
      prior-art register and the relevant spec.
- [x] `AGENTS.md` carries the durable numbered invariants list and
      everything else cross-references them by number.
- [x] Threat-model invariant mapping row exists for every
      `AGENTS.md` invariant (§1 — §10).
- [ ] Hardware matrix declares concrete benchmarks before any model
      pull. (R0 ships the schema and tier policy only — the per-tier
      benchmark table lives in
      `docs/research/hardware-benchmarks.md`, which is created and filled
      during M1. M1 is the gate that opens the first model pull.)
- [x] Three launch workflows exist with user-visible trust states.
- [x] Restart boundary observed; old code preserved in git history.

## What's next

After this lands, the next planning target is **M1 (Safe vault
open)**. The M1 implementation plan will live under
`docs/superpowers/plans/m1-safe-vault-open.md` and the orchestrator
skill `haven-m1-orchestrator` (not yet added; will be added when M1
is approved). Loading `haven-r0-orchestrator` after this lands gets
the answer "do not use this skill once R0 is exit-passed."

## Out of scope for this PR

- No production feature code.
- No CI that depends on cargo or npm yet (M1 scaffolding re-introduces
  those).
- No model downloads; first-run setup will benchmark before any pull.
- No silent remote API calls; outbound connectors ship disabled.

## Review checklist

- [ ] Every ADR's "Acceptance evidence" links to a real test or
      fixture path in this repo or M1 scaffolding.
- [ ] Every specialist skill's `Required inputs` matches the team-spec
      handoff contract.
- [ ] Threat-model invariant mapping row exists for every
      `AGENTS.md` invariant (§1 — §10).
- [ ] No ghost files / context-restored duplicates remain in the
      diff.
