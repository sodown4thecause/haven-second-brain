# ADR-004: Local runtime and network posture (Ollama loopback, no silent cloud)

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-architect`
Reviewer: orchestrator review-board
Linked design: `PLAN.md P0.6` and `AGENTS.md §3, §4, §5, §9`

## Context

Haven uses local models for chat, embeddings, and (eventually) voice
transcription. The desktop app must remain free of inbound network services
while still functioning as a first-class local AI client. We must also
support a future cloud-provider opt-in path without pretending cloud calls
are silent.

## Decision

**Inference engine:** **Ollama** as the default inference subprocess.
- Binds to `127.0.0.1:<dynamic>` — never `0.0.0.0`. The port is randomized
  per launch and never exposed in any public interface.
- The Tauri webview communicates with Ollama over a Tauri-managed Rust-side
  client; no direct webview-to-engine script occurs.
- The CSP forbids direct webview-to-engine `connect-src`.

**Model selection:** Default to `qwen3.5:4b` Q4 (per `ADR-007`); `nomic-embed-text`
as the fallback embedding. Pinned digest + chat template + license + context
cap are stored in the release manifest.

**Cloud provider opt-in:** any cloud provider (OpenAI, Anthropic, etc.)
requires an explicit user entry in the engine settings. The first call to a
cloud provider must show a per-provider disclosure and a preflight of which
note excerpts leave the device. **No silent fallback.**

**Egress policy:** outbound traffic is restricted to a typed allowlist:

- Configured Ollama endpoint (loopback only by default).
- Configured sync relay (HTTPS).
- Configured web search and extract providers (HTTPS only, plaintext domains
  rejected).
- Configured memory engine (HTTPS only).
- Configured matrix/team bridges (per `P5.2`).

Loopback addresses other than `127.0.0.1`/`::1`, link-local, and the
`169.254.169.254` cloud-metadata range are blocked at the URL validator.

**Web research erosion:** fetched content cannot trigger tool execution
(`threat-model.md` §Network and content safety).

**Engine lifecycle:** the engine is launched on demand and shut down after
30 s of idle. A user-visible "engine state" status is always shown so the
user knows whether inference is available.

## Alternatives considered

- **Custom llama.cpp sidecar**: deferred. Multi-platform packaging cost is
  out of v1 scope; revisit when Ollama support misses a model we need.
- **Mandatory model download**: rejected; search/editor/MCP must work
  before any model is installed.
- **Open engine binding**: rejected; violates `AGENTS.md §5`.

## Consequences

- Implementation lives in `crates/haven-engine`. It owns spawn/health/shutdown.
- No telemetry leaves the device without opt-in.
- The engine's storage location is documented and easy to inspect.
- Deletion is reversible from inside the app (no hidden caches).

## Reversibility

- The engine binding is a single typed contract; swapping in llama.cpp later
  is local to that crate.
- Deleting `.haven/` does not delete Ollama's model store; that is an
  explicit user action with a documented consequence.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §11` (local model + inference
  runtime bakeoff: Ollama, llama.cpp, cloud providers, SearXNG/Firecrawl/
  Crawl4AI/Playwright) and §6 (sqlite-vec/FTS5 for embedding rows).
- Ollama's published API surface; Qwen 3.5 chat template + Qwen3-Embedding
  dimensions.

## Threat-model cross-reference

- The network posture is enforced at the URL validator, the engine spawn,
  and the CSP — defense in depth.
- DNS rebinding is defeated by re-resolving redirects before each fetch.

## Acceptance evidence

- `crates/haven-engine` test `loopback_only.rs` proves the engine refuses any
  bind address outside `127.0.0.1`/`::1`.
- `crates/haven-engine` test `cloud_preflight.rs` simulates a cloud call
  with a recorded fixture and asserts the disclosure shows the excerpts
  flagged for transmission.
- The model-id stamp test (`crates/haven-index tests/embedding_versioning.rs`)
  proves a model change triggers a reindex and never mixed-model similarity.
