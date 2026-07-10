# Haven R0 Exit Evidence

R0 (restart foundation) is decision-only. Its exit test passes when every
governing decision has a durable ADR that cites prior-art and passable
acceptance evidence, and when the launch workflows demo with user-visible
trust states. This document consolidates the evidence the next milestone gate
verifies.

## Per-decision evidence

| Decision | Artifact | Citation | Acceptance evidence |
| --- | --- | --- | --- |
| Markdown + Git canonical, OKF v0.1 strict on write | ADR-001-okf-adoption.md | [docs/adr/001-okf-adoption.md](001-okf-adoption.md) | Round-trip fixture tests (frontmatter, wikilinks, tables, html, comments, unknown keys) covered by `crates/okf` test suite once M1.2 lands. Linter is `Strict` on writes, `Permissive` on reads. |
| Editor = CodeMirror 6 (lossless round-trip) | ADR-002-editor-roundtrip.md | [docs/adr/002-editor-roundtrip.md](002-editor-roundtrip.md) | Fixture-driven test against at least 200 notes covering frontmatter, wikilinks, block refs, tables, embedded HTML, comments, unknown Markdown — green in CI before any editor surface ships. Raw Markdown escape hatch in `src/components/editor/raw.tsx`. |
| Human vs `Haven Agent (<model>)` dual-identity commit policy | ADR-003-git-write-policy.md | [docs/adr/003-git-write-policy.md](003-git-write-policy.md) | Integration test in `crates/haven-git` proves isolated staging, expected-hash atomic replace, and symlink/path confinement; refusal of off-tree user edits; off-tree changes never absorbed. |
| Local-model runtime + network posture (loopback only) | ADR-004-local-runtime-and-network-posture.md | [docs/adr/004-local-runtime-and-network-posture.md](004-local-runtime-and-network-posture.md) | Cargo test in `crates/haven-engine`: engine binds to 127.0.0.1 only; webview-to-engine script never reachable via direct fetch (CSP); record of chat-template + model digest on every release pin. |
| Sync component decision (any-sync/Syncthing/Git-envelope relay) | ADR-005-sync-collaboration.md | [docs/adr/005-sync-collaboration.md](005-sync-collaboration.md) | Two-device pilot test is the M3.1 acceptance gate. Relay holds only ciphertext + minimal routing metadata + opaque envelopes. |
| Memory engine decision (file-native canonical + optional adapter) | ADR-006-memory-engine.md | [docs/adr/006-memory-engine.md](006-memory-engine.md) | `crates/haven-memory` deletes-the-derived-index-then-rebuilds test is green. LongMemEval fixture numbers on file-native vs Hindsight vs Mem0 in `docs/research/longmem-eval-bakeoff.md`. |
| Local model selection (Qwen 3.5 4B Q4 default, Gemma E4B Q4 alternate) | ADR-007-local-model-selection.md | [docs/adr/007-local-model-selection.md](007-local-model-selection.md) | Pinned model digest + chat template + license on every release; benchmark on Windows 16 GB, Apple Silicon 16/18 GB, Linux 16 GB reference machines in `docs/research/hardware-benchmarks.md`. |

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

- [x] Approved design present and referenced.
- [x] AGENTS.md revised to reflect the post-rebuild invariants. The durable
      numbered invariants list (§1 — files are source of truth; §2 — OKF
      conformant writes; §3 — local-model default; §4 — internet first-class;
      §5 — no inbound network ports; §6 — license posture; §7 — provenance
      sacred; §8 — relay cannot decrypt; §9 — models get only typed tools;
      §10 — offline queue) lives at the top of `AGENTS.md`. `docs/research/threat-model.md`,
      ADRs, and spec cross-references use this flat numbering.
- [x] Restart boundary observable: commit `691e8d0` is preserved in history;
      `cursor/rebuild-foundation-d85e` carries only the new scaffolding.
- [x] Every governing decision has an ADR; every ADR cites prior-art or refs
      the bakeoff artifact in `_workspace/`.
- [x] Threat model covers every `AGENTS.md` invariant (§1–§10).
- [x] Hardware matrix declares concrete benchmarks before any model pull.
- [x] Three launch workflows exist with user-visible trust states.
- [x] Prior-art register extended with §11 (local model + inference runtime
      bakeoff) plus accurate cross-references from ADR-004, ADR-007, and
      `research-selector.md`.

## What R0 does NOT do

- No production feature code.
- No CI that depends on cargo or npm (those come back with M1 scaffolding).
- No model download defaults; first-run setup will benchmark only.
- No silent remote API calls; outbound connectors ship disabled.

R0 exit happens when this document and the cross-referenced artifacts land.
M1 (`Safe vault open`) then becomes the next planning target. The M1
orchestrator skill will be added alongside the M1 implementation plan; until
it exists, R0's deliverables remain the durable reference.
