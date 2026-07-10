# ADR-004: Local runtime and network posture (Ollama + IPC pipe, no silent cloud, no desktop TCP listener)

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-architect`
Reviewer: orchestrator review-board
Linked design: `PLAN.md P0.6` and `AGENTS.md §3, §4, §5, §9, §10`

## Context

Haven uses local models for chat, embeddings, and (eventually) voice
transcription. The desktop app **must not open an inbound TCP port** (per
`AGENTS.md §5` and §10). The webview must still reach the inference
subprocess for chat and embedding requests without exposing that surface to
other local processes or to the open web. We must also support a future
cloud-provider opt-in path without pretending cloud calls are silent.

## Decision

**Inference engine:** **Ollama** as the default inference subprocess.

- The Tauri Rust side gives Ollama an explicit Unix domain socket on Linux/
  macOS, a named pipe on Windows, or `127.0.0.1` only when the OS IPC
  primitive is unavailable on the target platform.
- The desktop app itself **never opens a TCP listener**. The IPC primitive
  is owned by a Tauri-managed process; only the spawned engine and the
  Tauri Rust side can read or write it.
- The Tauri webview communicates with the engine via a Tauri IPC handler,
  not via direct `fetch`. CSP forbids `connect-src` to `127.0.0.1`,
  `localhost`, the engine IPC path, or any private/loopback/link-local
  address.
- On platforms where Ollama is forced to listen on TCP (loopback), the
  bind is `127.0.0.1:<random>` only, randomized per launch, scoped to the
  engine process lifetime, and never advertised. Loopback is the worst-case
  fallback, not the default posture; the test gate (below) fails M1 if
  IPC pipe mode is not used on the documented platforms.

**Model selection:** Default to `qwen3.5:4b` Q4 (per `ADR-007`); `nomic-embed-text`
as the fallback embedding. Pinned digest + chat template + license + context
cap are stored in the release manifest.

**Cloud provider opt-in:** any cloud provider (OpenAI, Anthropic, etc.)
requires an explicit user entry in the engine settings. The first call to a
cloud provider must show a per-provider disclosure and a preflight of which
note excerpts leave the device. **No silent fallback.**

**Egress policy:** outbound traffic is restricted to a typed allowlist:

- Configured sync relay (HTTPS).
- Configured web search and extract providers (HTTPS only, plaintext domains
  rejected).
- Configured memory engine (HTTPS only).
- Configured matrix/team bridges (per `P5.2`).

Loopback addresses other than `127.0.0.1`/`::1`, link-local, and the
`169.254.169.254` cloud-metadata range are rejected by the URL validator
for both egress and any "fetch" from scraped-content evidence.

**Web research erosion:** fetched content cannot trigger tool execution
(`threat-model.md` §Network and content safety).

**Engine lifecycle:** the engine is launched on demand and shut down after
30 s of idle. A user-visible "engine state" status is always shown so the
user knows whether inference is available.

## Alternatives considered

- **Loopback TCP only** (the rejected prior posture): a `127.0.0.1:<port>`
  listener is still an inbound network port — it allows any local process
  (browser extension, malware, another user on the same machine) to talk
  to the engine. Rejected: violates §5 and §10.
- **Custom llama.cpp sidecar** with built-in IPC: deferred. Multi-platform
  packaging cost is out of v1 scope; revisit when Ollama IPC support misses
  a model we need.
- **Cloud-only default**: rejected; the user-paid model lives on the user's
  machine and must keep working when the network is down (§4, §10).
- **Mandatory model download**: rejected; search/editor/MCP must work
  before any model is installed.

## Consequences

- Implementation lives in `crates/haven-engine`. It owns spawn/health/shutdown.
- On Linux/macOS the default engine transport is a Unix domain socket; on
  Windows it is a named pipe. TCP-loopback is a documented platform-only
  fallback when no IPC primitive exists for that pair.
- No telemetry leaves the device without opt-in.
- The engine's storage location is documented and easy to inspect.
- Deletion is reversible from inside the app (no hidden caches).

## Reversibility

- The engine transport is a single typed contract; swapping in llama.cpp later
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

## Acceptance evidence (M1 scaffolding, no production code in R0)

- `crates/haven-engine` test `ipc_pipe_only.rs` proves IPC mode (UDS on
  Unix, named pipe on Windows) is the default and the engine never opens
  a desktop TCP listener on any reference platform.
- `crates/haven-engine` test `cloud_preflight.rs` simulates a cloud call
  with a recorded fixture and asserts the disclosure shows the excerpts
  flagged for transmission.
- The model-id stamp test (`crates/haven-index/tests/embedding_versioning.rs`)
  proves a model change triggers a reindex and never mixed-model similarity.
