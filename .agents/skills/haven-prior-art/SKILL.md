---
name: haven-prior-art
description: Prior-art and license register specialist. Use when compiling or updating `docs/research/prior-art-register.md`. Records every "hard feature" category from the approved design with reviewed repositories, license verdicts, candidate reusable modules, and an explicit adopt/fork/reimplement decision. Read-only on doc creation — does not write code.
---

# Prior Art Clerk

## When to use

Use when the orchestrator asks the prior-art phase to produce
`docs/research/prior-art-register.md`, or when a new "hard feature" category
appears (e.g., a new importer, a new sync backend, a new diff surface).

Do not use for general research. This skill is scoped to populate a durable
register that ADRs cite.

## Required inputs

1. The hard-feature list in `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md`.
2. The pre-approved prior-art URL block in `PLAN.md`.
3. The license policy in `AGENTS.md §1.6` (Apache-2.0 app + AGPL-3.0 server;
   revisit per code reuse).

## Output shape

A markdown table per category with columns:

| Project | Repo | License | Reusable modules | Decision: adopt / fork / adapt / reimplement | Evidence link |

Rules:

- Apache-2.0 / MIT projects: adoption is the default unless reuse creates a
  remote-service, proprietary-format, broad-fs, open-port, or
  non-rebuildable-state dependency.
- GPL / AGPL projects: UX and data-model choices only; code reuse requires
  an explicit ADR/legal review because the desktop app is Apache-2.0.
- For every decision, cite the specific issue, PR, or commit that supports
  the call. If there is no evidence yet, write `evidence: pending` and route
  the resolution to `haven-architect`.

## Validation

Every category in the approved design must cover:

- Lossless Markdown editor and property UX
- Importers and migration
- Web clipper/readability
- Sync/conflict/local-first collaboration
- MCP notes/agent bridge
- Vector/RAG index
- PDF/Zotero/research lane
- Long-term agent memory
- Offline voice transcription

If a category is missing, raise a blocker back to the orchestrator.
