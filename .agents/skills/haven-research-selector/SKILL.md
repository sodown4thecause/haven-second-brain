---
name: haven-research-selector
description: Static research-selector spec + Knowledge Diff evaluation fixture specialist. Use when the R0 orchestrator asks for the static research-selector spec or the Knowledge Diff fixtures. Records the ResearchIntent enum, the deterministic policy router, the per-tool budget, and a recorded-fixture set covering novel / corroborating / conflicting / uncertain cases. Read-only — does not write code.
---

# Research Selector Spec

## When to use

Use when the orchestrator calls for the static research-selector spec or the
Knowledge Diff evaluation fixtures.

Do not use to design or implement web-research tooling. R0 is decision-only.

## Required inputs

1. `AGENTS.md §1.9` (models get only typed tools).
2. The local-model runtime ADR.
3. `docs/research/threat-model.md`.
4. Approved design §Static research-tool routing and §Generative Citation
   Intelligence.

## Spec must include

- ResearchIntent enum: `search`, `fetch`, `crawl`, `extract`, `interact`,
  `no_web`.
- Selector model requirements (small, fast, schema-validated output).
- Policy router rules per ResearchIntent variant: which typed tool, which
  budget, which permission.
- Budget envelope (domains per request, requests per session, byte budget,
  redirect count, depth, time).
- Knowledge Diff bucket definitions and a documented mixed-posterior rule.
- A minimum recorded-fixture set:
  - 4 cases of novel (each sourced from a captured HTML / markdown).
  - 4 cases of corroborating.
  - 4 cases of conflicting.
  - 4 cases of uncertain.

## Output shape

`docs/superpowers/specs/research-selector.md` and
`docs/research/knowledge-diff-fixtures.md`.
