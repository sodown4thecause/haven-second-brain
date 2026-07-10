# ADR Index (R0 cross-reference)

Cross-reference of every ADR under `docs/adr/` with the prior-art
register row, the spec(s), and the threat-model entry it depends on.
This file lives in the orchestrator scratchpad so the next milestone
can read the full evidence chain in one pass. The durable summary is
in [`docs/adr/000-r0-exit-evidence.md`](../docs/adr/000-r0-exit-evidence.md).

| ADR | Decision | Prior-art row | Linked spec(s) | Threat-model row |
| --- | --- | --- | --- | --- |
| `000-r0-exit-evidence.md` | R0 gate evidence | (consolidated) | `launch-workflows.md`, `hardware-model-matrix.md`, `alpha-success-metrics.md`, `research-selector.md` | (consolidated) |
| `001-okf-adoption.md` | Adopt OKF v0.1 strict-write, permissive-read | prior-art §2 (importers), §5 (MCP) | `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md §Static Knowledge Model` | §2 OKF-conformant writes |
| `002-editor-roundtrip.md` | CodeMirror 6 default with raw Markdown escape hatch | prior-art §1 | `PLAN.md P0.2` + the escape hatch spec in §Editor | §6 No lock-in |
| `003-git-write-policy.md` | Dual-identity commits; isolated staging; atomic replace; side-by-side conflicts | prior-art §4 (Syncthing conflict semantics) | `PLAN.md P0.3` | §7 Provenance sacred |
| `004-local-runtime-and-network-posture.md` | IPC pipe (UDS / named pipe) by default; loopback TCP only as documented platform fallback; no silent cloud; CSP forbids direct webview engine | prior-art §11 (local model + inference runtime), §6 (sqlite-vec/FTS5) | `PLAN.md P0.6`, `hardware-model-matrix.md` | §3 Local-model default, §5 No inbound network ports, §8 Relay cannot decrypt, §9 Models get only typed tools |
| `005-sync-collaboration.md` | Native encrypted Git-envelope relay + Syncthing-style conflict semantics + any-sync-influenced spaces | prior-art §4 | approved design §E2EE Sync and Collaboration | §1 File truth, §5 No inbound ports, §8 Relay cannot decrypt |
| `006-memory-engine.md` | File-native canonical memories; optional Hindsight/Mem0/Graphiti-derived adapters | prior-art §8 | approved design §Unified Context and Durable Memory | §1 File truth, §2 OKF conformant, §7 Provenance, §8 Relay cannot decrypt, §9 Models get typed tools |
| `007-local-model-selection.md` | Qwen 3.5 4B Q4 default; Gemma E4B Q4 alternate; embedding fallback matrix | prior-art §11 (Ollama/llama.cpp/cloud), §6 (sqlite-vec) | `hardware-model-matrix.md`, `PLAN.md Recommended local LLM stack` | §3 Local-model default, §7 Provenance |

## Per-decision acceptance evidence pointer (M1 gate)

- ADR-001 OKF — `crates/okf/tests/property_roundtrip.rs`,
  `crates/okf/tests/strict_write.rs`,
  `crates/okf/tests/permissive_read.rs`, and
  `crates/okf/tests/fixtures/` 200-note fixture rebuild.
- ADR-002 Editor — `tests/editor/roundtrip.test.ts` against the 200-note
  fixture; raw Markdown escape hatch in `src/components/editor/raw.tsx`.
- ADR-003 Git — `crates/haven-git/tests/safe_existing_vault.rs`,
  `crates/haven-git/tests/expected_hash_replace.rs`,
  `crates/haven-git/tests/symlink_confinement.rs`,
  `crates/haven-git/tests/off_tree_user_changes_preserved.rs`.
- ADR-004 Local runtime — `crates/haven-engine/tests/ipc_pipe_only.rs`,
  `cloud_preflight.rs`, `model_manifest_pin.rs`, `embedding_stamp.rs`;
  plus `crates/haven-index/tests/embedding_versioning.rs`.
- ADR-005 Sync — `crates/haven-sync/tests/two_device_e2e.rs`,
  `three_device_e2e.rs`, `rotation_failure_blocks_sharing.rs`,
  `replay_protection.rs`; relay fixture under
  `crates/haven-relay/tests/fixtures/relay_fixture.toml`.
- ADR-006 Memory — `crates/haven-memory/tests/file_backed_retain.rs` and
  `consolidation_review.rs`; numeric recall numbers in
  `docs/research/longmem-eval-bakeoff.md` (table shape filled in M6).
- ADR-007 Local model — `crates/haven-engine/tests/model_manifest_pin.rs`
  and `embedding_stamp.rs`; per-tier benchmark in
  `docs/research/hardware-benchmarks.md` (gate for first model pull).

## Cross-cutting

- Threat model covers every `AGENTS.md` invariant (§1 — §10).
- Updated `SELECTOR` cross-references in `research-selector.md` and
  ADR-007 / ADR-004 now point at `prior-art §11`.
- `_workspace/09_r0_exit_evidence.md` mirrors this index for the
  orchestrator's gate review.

## Handoff contract

Status at handoff: `Status: reviewed-pass` (orchestrator review-board).
The orchestrator rule of advance applies: every ADR is referenced in
this index, every prior-art row is cited from the ADR, every spec
references its boundary ADR, every threat-model row references its
invariant.
