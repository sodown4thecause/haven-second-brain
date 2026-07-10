# Haven Static Research Selector Spec (R0)

The local model accesses the internet only through typed tools registered
statically at build time. Per `AGENTS.md §9`, fetched content is
untrusted evidence, not tool instructions.

## Tool inventory

Each tool is implemented in Rust (`crates/haven-research/`) with a typed
wrapper, a per-tool budget, and an allowlist policy. None can dynamically
register new tools at request time.

| Tool | Purpose | Default budget per request |
| --- | --- | --- |
| `web_search` | Search result pages | 10 results, 8s wall-clock |
| `fetch_page` | Single static page | 5 MiB, 5 redirects, 5s |
| `crawl_site` | Domain-finite crawl | 50 pages, 100 MiB, 60s, depth 3 |
| `extract_structured` | Schema-bound extract over a known page | 1 schema, 1 MiB extracted, 30s |
| `interact_with_page` | Sandboxed browser | 30s, 50 commands, JS disabled by default |

Each tool refuses private, loopback, link-local, and cloud-metadata targets.
The URL validator re-resolves redirects before each fetch.

## ResearchIntent enum (selector output)

A small local selector model receives the user intent + compact context
metadata and emits a schema-validated enum:

```ts
type ResearchIntent =
  | { kind: "search"; query: string }
  | { kind: "fetch"; url: string }
  | { kind: "crawl"; url: string; limits: { maxPages, maxDepth } }
  | { kind: "extract"; url: string; schema: Schema }
  | { kind: "interact"; url: string; task: string }
  | { kind: "no_web" };
```

The selector is a single tool call from the user's reasoning model. The
selector **cannot call tools directly**. The deterministic policy router
(`crates/haven-research/src/router.rs`) validates the selector output
against the schema, checks permissions and budgets, then invokes the
matching tool wrapper.

## Policy router

- A selector output that violates the schema falls back to bounded search
  or `no_web`.
- Low-confidence selector output falls back to `no_web`.
- Domain policies (e.g., "never query `*.gov`" or "always re-resolve before
  any fetch") are loaded from the explicit config file the user agreed to.

## Context assembly

Once tool evidence is collected, the reasoning model receives only:

- The user's request.
- Selected vault passages.
- Approved memories.
- Normalized evidence (URLs, snippets, claim labels).
- Provenance and retrieval diagnostics — never per-provider schemas.

The reasoning model is never asked to choose a tool at request time.

## Knowledge Diff

After web extraction, classification against the local vault + approved
memories is a separate typed step:

- **Novel**: no materially equivalent claim found in the retrieved local
  corpus.
- **Corroborating**: web evidence independently supports an existing local
  claim.
- **Conflicting**: web evidence disagrees with or supersedes a local claim.
- **Uncertain**: retrieval coverage or evidence quality is insufficient
  for classification.

A Knowledge Diff is a reviewable OKF document with `type: research-diff`,
source URLs, access timestamps, compared local concept links, and
claim-level citations. The agent proposes updates as separate Git diffs; it
never silently merges crawled claims into the vault.

## Safety

- Every redirect is re-resolved before each fetch (DNS rebinding defense).
- Tool outputs are typed JSON; HTML is sanitized for the chat view.
- Interactive-page tools run in a sandboxed browser without vault,
  keychain, sync, or filesystem access.
- Fetched content cannot trigger write, key, sync, memory-approval, or
  collaboration tools.

## Cross-references

- Local runtime + network posture: [ADR-004](../adr/004-local-runtime-and-network-posture.md).
- Provider bakeoff verdict: `docs/research/prior-art-register.md §11`
  (Ollama, llama.cpp, SearXNG, Firecrawl, Crawl4AI, Playwright); §3 covers
  the Readability.js clipper-side path; §4 covers encrypted delivery of
  research results in shared spaces.
- Knowledge Diff fixtures: [knowledge-diff-fixtures.md](../../research/knowledge-diff-fixtures.md).
- Threat model: [threat-model.md](../../research/threat-model.md).
