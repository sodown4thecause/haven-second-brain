# ADR-006: Memory engine — file-native canonical, optional Hindsight/Mem0/Mem0 adapter

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-memory`
Reviewer: `haven-architect`
Linked design: `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md §Unified Context and Durable Memory`

## Context

We need durable agent memory that:

1. Survives the file-truth invariant: canonical memories are in `memory/observations/`
   as Markdown with provenance, scope, confidence, supersession, and
   sensitivity.
2. Never depends on a second-source-of-truth database (no Docker/Python/
   Postgres/open-port default on the user's machine).
3. Supports temporal recall, contradictions, evidence attribution, and
   forgetting.
4. Recovers from "delete derived index" — the canonical file set alone must
   rebuild the system.

We compared four candidates:

1. **File-native canonical + SQL-derived index** (custom).
2. **Hindsight** (vectorize-io, Apache-2.0).
3. **Mem0** (mem0ai, Apache-2.0).
4. **Graphiti-derived behavior** (getzep/graphiti, Apache-2.0).

The design says Hindsight is the primary behavior/reference, Graphiti the
temporal-provenance reference, and Mem0 an alternate adapter. None should be
adopted wholesale as a service dependency.

## Decision

Adopt **file-native canonical memories** as the canonical path. The flagship
implementation is `crates/haven-memory`:

- Candidate observations live in `memory/observations/<date>-<slug>.md` with
  the schema in `AGENTS.md §7` (type, subject, scope, timestamp, confidence,
  sources, supersedes, sensitivity, status).
- A SQLite-derived index is built from approved observations only.
- External memory engines are wrapped as **optional adapters**, never as the
  canonical store.
- The memory inbox UX lives under `src/components/memory-inbox/`; users
  approve, edit, reject, forget, or pin observations before they become
  canonical.

**Adapters:**

- **Hindsight** is wired as an alternate retain/reflect provider behind a
  stdio sidecar that the user installs explicitly. Hindsight's service DB
  is never authoritative; a rebuild from files reproduces Hindsight state.
- **Mem0** is wired similarly.
- **Graphiti** is the temporal-validity reference; we re-implement the
  supersession and contradiction rules in Rust, citing the Graphiti
  papers/issues as evidence for rule choice.

**Why file-native is canonical:** it preserves Markdown + Git durability, the
file-truth invariant, and rebuild-from-files. The build-forward path is:
file set -> derive SQLite -> Hindsight/Mem0 (if configured). The
build-backward path is: Hindsight/Mem0 are disposable; canonical behaviors
must round-trip through the file set.

## Alternatives considered

- **Adopt Hindsight as service**: rejected; service DB becomes canonical
  and the desktop invariants break.
- **Adopt Mem0 wholesale**: rejected for the same reason; benefits do not
  outweigh the second-source-of-truth cost.
- **Skip memory until P6**: rejected; the file-native baseline is small and
  pays for itself with a memory-inbox on day one.

## Consequences

- `crates/haven-memory` owns file-native retain/recall, supersession,
  sensitivity, evidence attribution, and the inbox flow.
- Hindsight/Mem0 adapters run in-process or over stdio; never a network
  service on the user's machine.
- Forgetting removes derived vectors on the next reconciliation pass.

## Reversibility

- The file set is canonical. Removing every adapter leaves a working memory
  system with file-native recall only.
- Adding a new adapter is typed: implement the
  `MemoryEngineAdapter` trait (`crates/haven-memory/src/engine.rs`) and
  gate it behind an explicit user opt-in.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §8 (long-term agent memory)`.
- Hindsight paper and Graphiti papers cited inline.

## Threat-model cross-reference

- Memories have sensitivity (`public`/`private`/`shared`). Private memories
  never enter a shared space; shared memories have provenance and a shared
  space they are scoped to.

## Acceptance evidence

- `crates/haven-memory` test `file_backed_retain.rs`: deleting the SQLite
  index, the next reconcile rebuilds it from the file set.
- `crates/haven-memory` test `consolidation_review.rs`: consolidation runs
  only idle and produces reviewable proposals; never silent overwrites.
- `docs/research/longmem-eval-bakeoff.md`: numeric recall results across
  the founder workflow on file-native, Hindsight, and Mem0.
