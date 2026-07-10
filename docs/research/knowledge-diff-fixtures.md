# Knowledge Diff Evaluation Fixtures (R0)

This file is part of the static research selector spec. It documents the
recorded-fixture set the alpha uses to evaluate Knowledge Diff
classification quality, before any wide roll-out of web-connected
features.

These fixtures live under `experiments/knowledge-diff/fixtures/` and the
evaluation harness reads them read-only. They must be reproducible on a
fresh device without network access.

## Buckets

- **Novel** — web evidence claim has no materially equivalent local claim.
- **Corroborating** — web evidence independently supports an existing local
  claim.
- **Conflicting** — web evidence disagrees with or supersedes a local
  claim.
- **Uncertain** — retrieval coverage or evidence quality is insufficient
  for classification.

## Required coverage

The minimum recorded-fixture set is:

- 4 cases of **novel**, each with a captured HTML / Markdown page and a local
  corpus that does not contain the claim.
- 4 cases of **corroborating**, each with a local claim and an independent
  web source supporting it.
- 4 cases of **conflicting**, each with a local claim and a web source
  providing a contradictory timestamp or counterexample.
- 4 cases of **uncertain**, each with sparse local coverage or low-quality
  web evidence (e.g., a 500-error response, a paywalled page, a corrupted
  HTML).

## Fixture shape

Each fixture is a folder:

```
experiments/knowledge-diff/fixtures/<bucket>/<id>/
  query.txt             # the user request, in plain text.
  local/                # the local retrieval snapshot (notes as .md).
  web/markdown/         # the captured page content, parsed offline.
  web/html/             # the original HTML the parser saw.
  expected_bucket.txt   # "novel" | "corroborating" | "conflicting" | "uncertain"
  expected_citations.tsv
```

## Acceptance rule

- The Knowledge Diff classification produced by the harness MUST equal
  `expected_bucket.txt`.
- The cited claim IDs MUST equal `expected_citations.tsv`.
- The harness MUST be deterministic: identical fixture input, identical
  output.

## Determinism guarantees

- Fixture pages are recorded as captured HTML — never re-fetched at
  evaluation time.
- The local corpus is a directory of `.md` files; the harness operates on
  the directory read-only.
- The selector model is invoked with `seed` and full precision; no
  temperature noise.

## When to add fixtures

- Adding a fixture is a typed PR with:
  - The captured page (offline-safe).
  - The local corpus (clearly marked as sanitizer-only).
  - The expected bucket and citations.
  - A note explaining the bucket decision.

## Honest limits

- A small fixture set catches regressions, not creativity. Things will go
  wrong without surfacing in these fixtures; that is expected and out of
  scope of this contract.
- The harness uses the file-native canonical memory and the SQLite-derived
  index. It does not require a remote model.

## Cross-references

- Spec: [research-selector.md](../superpowers/specs/research-selector.md).
- Local runtime: [ADR-004](../adr/004-local-runtime-and-network-posture.md).
- Threat model: [threat-model.md](threat-model.md).
