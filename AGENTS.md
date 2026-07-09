# AGENTS.md — Engineering Guidelines for Haven

Haven is a git-native, agent-native "second brain": a local-first Tauri desktop app
whose data layer is a plain-text Git repository conforming to the Open Knowledge
Format (OKF v0.1), with fully offline AI (Gemma 4 via Ollama/llama.cpp), Agent
Skills (`SKILL.md`), and built-in MCP interop. See `PLAN.md` for the phased build
plan and architecture decisions.

This file is the contract for every agent (and human) working in this repo.
Read it fully before making changes. When a rule here conflicts with your general
habits, this file wins.

---

## 1. Product invariants (never violate)

1. **Files are the only source of truth.** User data is Markdown + YAML frontmatter
   in a Git repo. The SQLite index, vector store, and caches under `.haven/` are
   derived and must always be rebuildable from the files alone. Never store user
   data that exists only in the index.
2. **Local-first, offline-first.** Every free-tier feature must work with the
   network cable unplugged. No feature may silently call a remote endpoint.
   Cloud calls happen only through explicit, user-visible configuration.
3. **No lock-in.** Output formats follow published open standards: OKF v0.1 for
   documents, Agent Skills for skills, MCP for interop, plain Git for history.
   Never invent proprietary syntax when a standard field or convention exists.
4. **Provenance is sacred.** Every write to the user's knowledge repo goes through
   the Git pipeline with the correct author identity — human author for user
   edits, `Haven Agent (<model>) <agent@haven.local>` for AI-generated commits.
   Never batch human and AI changes into one commit.
5. **Privacy by architecture.** No telemetry without explicit opt-in. No open
   TCP ports (browser extension uses Native Messaging). E2EE keys never leave
   the user's devices except via the folder-scoped disclosure-key flow.

## 2. Repository conventions

- **Branches:** `cursor/<descriptive-name>-d85e` for agent work; short-lived,
  one concern per branch.
- **Commits:** one logical change per commit, imperative mood
  ("Add OKF frontmatter linter", not "Added" / "misc fixes"). Reference the
  plan item when applicable (`[P1.3]`).
- **PRs:** small and reviewable. The description states what changed, why, and
  how it was verified. CI must be green before requesting review.
- **Monorepo layout (once scaffolded):**
  - `src-tauri/` — Rust core (Tauri app, indexer, git pipeline, MCP server)
  - `src/` — frontend (TypeScript, BlockSuite editor shell)
  - `crates/` — shared Rust crates (`okf`, `haven-index`, `haven-git`)
  - `packages/` — shared TS packages if needed
  - `docs/` — architecture notes and ADRs
- **ADRs:** any decision that changes an invariant, a dependency, or a data
  format gets an Architecture Decision Record in `docs/adr/NNN-title.md`.

## 3. Rust guidelines (Tauri core, crates)

- **Toolchain:** stable Rust, pinned via `rust-toolchain.toml`. Code must be
  clean under `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`.
- **Error handling:** libraries (`crates/*`) define error enums with `thiserror`;
  the app binary may use `anyhow` at the edges. Never `unwrap()`/`expect()` in
  production paths — reserve them for tests and genuinely impossible states,
  with a comment explaining why the state is impossible.
- **No `unsafe`** without an ADR and a `// SAFETY:` comment proving the invariant.
- **Async:** tokio only. Never block the async runtime — wrap filesystem-heavy or
  CPU-heavy work (indexing, embedding, git scans) in `spawn_blocking` or a
  dedicated worker thread. No `async` in trait signatures without checking
  object-safety needs.
- **Concurrency:** prefer message passing (channels) over shared mutable state.
  If you must share, use `parking_lot` mutexes and hold locks across the
  narrowest possible scope; never hold a lock across an `.await`.
- **Data types:** parse, don't validate — convert raw input into typed structs
  (`serde`) at the boundary and pass types inward. Newtypes for ids and paths
  (`DocPath`, `SessionId`) instead of bare `String`.
- **Testing:** unit tests co-located (`#[cfg(test)]`), integration tests in
  `tests/`. Property tests (`proptest`) for parsers (frontmatter, links,
  importers). Every bugfix lands with a regression test.

## 4. Tauri 2 guidelines

- **IPC:** expose functionality via `#[tauri::command]` with typed,
  serde-deserializable arguments. Commands are thin — they validate, then call
  into `crates/` logic. No business logic in command handlers.
- **Capabilities/permissions:** follow least privilege. Each window gets its own
  capability set; never grant `fs` or `shell` scope wider than needed. New
  plugin permissions require justification in the PR description.
- **Security:** strict CSP; never load remote JS/HTML into the webview; no
  `dangerousRemoteDomainIpcAccess`. All external content (scraped pages,
  imported HTML) is sanitized before rendering.
- **Sidecars (Ollama/llama.cpp):** managed as Tauri sidecars with health checks,
  graceful shutdown, and version pinning. Never assume a sidecar is running —
  every call path handles "engine unavailable" with a user-actionable error.
- **State:** long-lived state lives in `tauri::State` behind typed structs.
  Background workers (git watcher, indexer) communicate with the UI via events
  (`emit`), never by polling from the frontend.

## 5. TypeScript & frontend guidelines

- **Strictness:** `"strict": true`, `noUncheckedIndexedAccess`, no `any`
  (use `unknown` + narrowing). ESLint + Prettier enforced in CI.
- **Imports at top of module only.** No inline `import()` in function bodies
  except documented lazy-loading of heavy UI chunks.
- **Exhaustiveness:** every `switch` over a union/enum has a `default` branch
  performing a `never` check so new variants fail compilation:

  ```ts
  default: {
    const _exhaustive: never = value;
    throw new Error(`Unhandled variant: ${_exhaustive}`);
  }
  ```

- **State & data flow:** UI state derives from the Yjs document and Tauri events.
  Do not duplicate document state into ad-hoc stores; subscribe to the source.
- **BlockSuite/Yjs:** all document mutations go through BlockSuite transactions —
  never mutate Yjs types outside a transaction. Custom blocks live in their own
  package with schema, model, and view separated.
- **IPC hygiene:** every `invoke()` call is wrapped in a typed client
  (`src/lib/ipc.ts`) — no raw string command names scattered through components.

## 6. Data layer (SQLite index)

- **The index is disposable.** Any schema change ships with a bump to
  `INDEX_SCHEMA_VERSION` and a full-rebuild path; there are no data migrations
  for derived data — drop and reindex.
- Access SQLite from a single writer worker (WAL mode, one write connection,
  many readers). No SQL string concatenation — parameterized queries only.
- FTS5 for keyword search, `sqlite-vec` for vectors, recursive CTEs for graph
  queries. Any graph engine beyond that goes behind the `GraphEngine` trait.
- Keep index rebuild fast: target < 30 s for a 10k-document repo on a mid-range
  laptop; benchmark in CI when the indexer changes.

## 7. AI layer

- **Provider abstraction:** all model calls go through the Vercel AI SDK with a
  single provider factory. Local default is Ollama (`ai-sdk-ollama`); never
  hardcode a model name at call sites — models are configuration.
- **Structured output first:** for classification, extraction, and ingestion use
  `generateObject` with a Zod schema. Free-form generation is only for chat and
  prose drafting. Validate LLM output like untrusted user input — because it is.
- **Prompts are code:** system prompts live in versioned files
  (`src-tauri/prompts/` or `prompts/`), not string literals; changes to prompts
  get the same review as changes to code.
- **Embeddings:** EmbeddingGemma, 768-dim stored, Matryoshka truncation allowed
  for pre-filtering. Embedding versioning: store the model id alongside vectors;
  a model change triggers reindex, never mixed-model similarity search.
- **Token discipline:** retrieval and skill loading respect explicit token
  budgets. Progressive disclosure for skills (metadata → SKILL.md → resources)
  is mandatory, not optional.
- **Determinism in tests:** unit tests never call a live model. Use recorded
  fixtures or fake providers. Integration tests that need a model are opt-in
  (`HAVEN_TEST_LLM=1`) and excluded from default CI.

## 8. OKF & Agent Skills conformance

- Every document the app writes must be OKF v0.1 conformant: parseable YAML
  frontmatter with non-empty `type`; recommended fields (`title`, `description`,
  `resource`, `tags`, `timestamp`) populated when known; unknown frontmatter
  keys preserved on round-trip; cross-links as inline Markdown links with
  bundle-relative paths. Reserved files `index.md` / `log.md` follow spec §6/§7.
- **Be permissive reading, strict writing:** never reject a user's file for
  missing optional fields or broken links (spec §9); never emit a
  non-conformant file ourselves.
- Skills follow the Agent Skills spec exactly: `SKILL.md` frontmatter with
  `name` (lowercase, hyphenated, ≤64 chars) and `description` (≤1024 chars,
  says what *and when*); body under 500 lines; deeper material in `references/`;
  executables in `scripts/`. The built-in linter mirrors `skills-ref validate`.

## 9. MCP guidelines

- The MCP server exposes narrow, well-described tools (`search_brain`,
  `read_document`, `write_document`, `list_skills`, `recall_memory`, …).
  Read-only by default; write tools require a per-client user grant persisted
  and revocable.
- Every write via MCP goes through the same OKF linter + Git provenance
  pipeline as in-app edits — no side doors.
- Treat all inbound MCP tool arguments as untrusted: validate paths against the
  bundle root (no traversal), size-limit payloads, and rate-limit clients.

## 10. Security checklist (every PR touching these areas)

- No new open network ports. Browser extension traffic uses Native Messaging.
- Secrets and keys via the OS keychain (Tauri plugin), never files or env vars
  in the repo. Nothing secret in logs.
- Path handling: canonicalize and confine to the workspace/bundle root before
  any filesystem operation.
- Dependency policy: prefer well-maintained crates/packages; new dependencies
  need a sentence of justification in the PR; `cargo audit` / `npm audit` run
  in CI and block on criticals.
- Anything sandbox-adjacent (skill `scripts/`, scraped content, imported files)
  is treated as hostile input.

## 11. Testing & CI

- CI gates on: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`,
  `tsc --noEmit`, ESLint, frontend unit tests, and the OKF/skills linter run
  against fixture bundles.
- E2E: `tauri-driver`/WebDriver smoke tests for the core loop (create doc →
  git commit → search finds it → reindex reproduces state).
- The invariant test that must never be deleted: *delete `.haven/`, reindex,
  and assert search/graph/embeddings are fully reconstructed.*
- Write tests at the lowest level that can catch the bug; don't E2E-test what a
  unit test can cover.

## 12. Performance budgets

- Cold app start to interactive: < 2 s (excluding model load, which is lazy and
  shows progress).
- Keystroke-to-render in editor: no LLM or index work on the hot path; indexing
  is debounced and async.
- Local chat first-token: < 2 s with the default E4B model on 16 GB RAM.
- Memory: the app (excluding inference engine) stays under 500 MB RSS with a
  10k-document repo.

## 13. Writing style for code & docs

- Comments explain *why*, never *what*. No narration comments.
- Public Rust items get doc comments; public TS exports get TSDoc.
- User-facing strings live in a central module from day one (future i18n).
- Docs and ADRs are plain Markdown, wrapped at 100 columns, and updated in the
  same PR as the behavior they describe.
