# LongMemEval Bakeoff Artifact

Companion to [ADR-006](../adr/006-memory-engine.md). The bakeoff compares
four memory approaches on a small HavenLongMemEval fixture built from the
founder's workflows.

This document is the artifact that backs the ADR verdict. Numeric results
are filled in during M6 once the file-native canonical path and the
external adapters are runnable. Until then the artifact captures the
**shape** of the comparison so the ADR can sit on solid cross-reference.

## Comparison shape

Each row is a memory engine under test. Each column is a measured
quality.

| Engine | License | Runtime shape | Second source of truth? | Temporal recall | Contradiction handling | Evidence attribution | Rebuild-from-files | Latency / peak RAM |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| File-native canonical + SQL derived | n/a | in-process Rust | No | TBD | supersession rule | per-observation | Yes (delete `.haven/` and reindex) | TBD |
| Hindsight | Apache-2.0 | stdio sidecar; user-hosted service optional | No if used as adapter only | TBD | TBD | yes (per observation) | yes from canonical files | TBD |
| Mem0 | Apache-2.0 | stdio sidecar | No | TBD | limited | yes | yes from canonical files | TBD |
| Graphiti-derived behavior | Apache-2.0 | re-implemented | No | TBD | supersession + contradiction rules | yes | yes from canonical files | TBD |

The numbers come from `experiments/longmem-eval/`. They are written down

once the bakeoff runs. The decision in [ADR-006](../adr/006-memory-engine.md)
is that file-native is canonical and the others are optional adapters; this
document is why.

## Why file-native is canonical

Several reasons stack:

1. **File-truth invariant (`AGENTS.md §1`)** — only a file-native path
   keeps the canonical state on disk.
2. **Rebuild-from-files** — the file set is the load-bearing canonical
   form; any swap-in adapter is disposable.
3. **Second-source-of-truth reward** — we deliberately avoid the cost of
   keeping two synchronized stores.
4. **Local-only default** — the desktop app does not require Docker,
   Python, or an HTTP service the user has to manage.

## Notes on adapters

Where a candidate is useful as an **adapter**, the adapter must:

- Speak to a chosen host via in-process or stdio. No additional HTTP
  service.
- Refuse to accept writes that bypass the file-native canonical path.
- Reindex from the canonical file set on cold start.

The benchmark for "file-native vs Hindsight" is the recall accuracy on the
founder workflow fixture. The benchmark for "file-native vs Mem0" is the
contradiction/supersession quality on the same fixture.

## Open questions

These are deliberate remaining uncertainties; they move forward as
experiments, not as silent invariants.

- Will a "bidirectional" mental-model layer be needed? Out of scope until
  evidence shows it would help.
- Will consolidation proposals review themselves when memory scope changes?
  Reviewed during the proposed phase, never silent.

## Cross-references

- Spec and decision: [ADR-006](../adr/006-memory-engine.md).
- File-backed retain implementation: `crates/haven-memory` (M6).
- Threat-model mapping: [threat-model.md](threat-model.md).
