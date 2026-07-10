# Haven R0 Exit Evidence

R0 (restart foundation) is decision-only. Its exit test passes when every
governing decision has a durable ADR that cites prior-art and passable
acceptance evidence, and when the launch workflows demo with user-visible
trust states. This document consolidates the evidence the next milestone gate
verifies.

## Per-decision evidence (R0 contracts; M1 verifies)

| Decision | Artifact | Citation | Acceptance evidence (contract — verified in M1) |
| --- | --- | --- | --- |
| Markdown + Git canonical, OKF v0.1 strict on write | ADR-001-okf-adoption.md | [docs/adr/001-okf-adoption.md](001-okf-adoption.md) | `crates/okf` property / strict-write / permissive-read / 200-note fixture tests assert round-trip (frontmatter, wikilinks, tables, html, comments, unknown keys). Linter is `Strict` on writes, `Permissive` on reads. **M1 gate.** |
| Editor = CodeMirror 6 (lossless round-trip) | ADR-002-editor-roundtrip.md | [docs/adr/002-editor-roundtrip.md](002-editor-roundtrip.md) | 200-note fixture-driven test at `tests/editor/roundtrip.test.ts` asserts byte-equal round-trip for frontmatter, wikilinks, block refs, tables, embedded HTML, comments, unknown Markdown — green in CI before any editor surface ships. Raw Markdown escape hatch is expected in `src/components/editor/raw.tsx` once `src/components/editor/` lands. **M1 gate.** |
| Human vs `Haven Agent (<model>)` dual-identity commit policy | ADR-003-git-write-policy.md | [docs/adr/003-git-write-policy.md](003-git-write-policy.md) | `crates/haven-git` tests (off-tree preservation, expected-hash atomic replace, symlink/path confinement) prove isolation of the writer and integrity under crash. **M1 gate.** |
| Local-model runtime + network posture (IPC pipe default; loopback TCP only as documented fallback) | ADR-004-local-runtime-and-network-posture.md | [docs/adr/004-local-runtime-and-network-posture.md](004-local-runtime-and-network-posture.md) | `crates/haven-engine` tests (ipc_pipe_only, cloud_preflight, model_manifest_pin, embedding_stamp) prove no desktop TCP listener, cloud disclosure before each call, and release-pin refusal. **M1 gate.** |
| Sync component decision (any-sync/Syncthing/Git-envelope relay) | ADR-005-sync-collaboration.md | [docs/adr/005-sync-collaboration.md](005-sync-collaboration.md) | `crates/haven-sync` tests (two-device e2e, three-device e2e, rotation_failure_blocks_sharing, replay_protection) are the M3.1 acceptance gate. Relay holds only ciphertext + minimal routing metadata + opaque envelopes. |
| Memory engine decision (file-native canonical + optional adapter) | ADR-006-memory-engine.md | [docs/adr/006-memory-engine.md](006-memory-engine.md) | `crates/haven-memory` tests (file_backed_retain, consolidation_review) and the rebuild-after-purge test. LongMemEval fixture numbers on file-native vs Hindsight vs Mem0 live in `docs/research/longmem-eval-bakeoff.md` and are filled in during M6 (R0 ships the table shape; numeric cells are `TBD`). |
| Local model selection (Qwen 3.5 4B Q4 default, Gemma E4B Q4 alternate) | ADR-007-local-model-selection.md | [docs/adr/007-local-model-selection.md](007-local-model-selection.md) | Pinned model digest + chat template + license + quantization on every release manifest. Benchmark numbers per tier per platform live in `docs/research/hardware-benchmarks.md` and are filled in during M1 — that file is the gate that allows the first model pull. |

## Cross-cutting evidence

| Cross-cutting concern | Artifact |
| --- | --- |
| Three launch workflows | [docs/superpowers/specs/launch-workflows.md](../superpowers/specs/launch-workflows.md) |
| Static research selector | [docs/superpowers/specs/research-selector.md](../superpowers/specs/research-selector.md) |
| Knowledge Diff evaluation fixtures | [docs/research/knowledge-diff-fixtures.md](knowledge-diff-fixtures.md) |
| Hardware + model support matrix | [docs/superpowers/specs/hardware-model-matrix.md](../superpowers/specs/hardware-model-matrix.md) |
| Alpha success metrics | [docs/superpowers/specs/alpha-success-metrics.md](../superpowers/specs/alpha-success-metrics.md) |
| Unified threat model | [docs/research/threat-model.md](threat-model.md) |
| Prior-art and license register | [docs/research/prior-art-register.md](prior-art-register.md) |

## Acceptance for R0 → M1

R0 is **decision-only**. The gate that opens M1 has two halves: the durable
artifacts (decision documents, cross-references, threat-model mapping, the
prior-art register, the team spec and skills) and the **M1 scaffolding**
(`crates/`, fixtures, benches, CI). The first half is what R0 must leave
green; the second half belongs to M1 and is referenced here only as the
gating list M1 must satisfy. R0 signs the contract — the implementation
files land in M1 and are gated separately below.

- [x] Approved design present (`docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`).
- [x] Every governing decision has an ADR (`docs/adr/001..007`); each ADR
      cites the prior-art register and acceptance evidence **contracts**
      (test paths and behavior expected at M1 — no crate sources in R0).
- [x] AGENTS.md carries the durable numbered invariants list (§1 — §10)
      and every cross-reference uses this flat numbering.
- [x] Threat model covers every `AGENTS.md` invariant (§1–§10) and each
      row links an adversary, a mitigation, and an acceptance test path
      under `crates/`.
- [x] Hardware + model support matrix declares tier policy, the 16 GB
      runtime budget, and the honesty bar. **Concrete benchmark numbers
      stay `TBD` in R0 and are filled into a new file
      `docs/research/hardware-benchmarks.md` during M1 — they are the
      gate that allows the first model pull.**
- [x] Three launch workflows exist (`launch-workflows.md`) with
      user-visible trust states and measurable acceptance criteria.
- [x] Prior-art register (`docs/research/prior-art-register.md`) covers
      every "hard feature" category from the approved design and includes
      §11 (local model + inference runtime bakeoff).
- [x] Restart boundary observable: pre-rebuild commit `691e8d0` is
      preserved in history; `cursor/rebuild-foundation-d85e` carries only
      the new scaffolding.

The following list is the **M1 gate**; it is checked off when M1 lands,
not by R0:

- [ ] `crates/okf` test suite (strict-write, permissive-read, fixture).
- [ ] `crates/okf` 200-note OKF fixture rebuild after `rm -rf .haven/`.
- [ ] `crates/haven-git` test suite (off-tree preservation, expected-hash,
      symlink confinement).
- [ ] `crates/haven-engine` test suite (IPC pipe default; loopback TCP
      only as a documented platform fallback; cloud preflight disclosure;
      model-pin refusal).
- [ ] `crates/haven-sync` test suite (two-device, three-device, rotation,
      replay).
- [ ] `crates/haven-memory` test suite (rebuild-from-files, consolidation
      review).
- [ ] `docs/research/hardware-benchmarks.md` filled with Floor / Default
      / Quality / Headroom numbers per platform.
- [ ] `scripts/skills-ref-validate.mjs` and `scripts/okf-lint.mjs` are
      restored and wired into CI.

## What R0 does NOT do

- No production feature code.
- No CI that depends on cargo or npm (those come back with M1 scaffolding).
- No model download defaults; first-run setup will benchmark only.
- No silent remote API calls; outbound connectors ship disabled.

R0 exit happens when this document and the cross-referenced artifacts land.
M1 (`Safe vault open`) then becomes the next planning target. The M1
orchestrator skill will be added alongside the M1 implementation plan; until
it exists, R0's deliverables remain the durable reference.
