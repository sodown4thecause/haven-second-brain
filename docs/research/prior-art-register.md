# Prior-Art and License Register (R0)

This register supplies evidence for every "hard feature" governance decision
in R0. Each row cites the licensed repository, the verdict relative to
Haven's product invariants, a documented set of reusable modules, and an
explicit adopt / fork / adapt / reimplement call with linked evidence.

License policy (`AGENTS.md §6`):

- Desktop app: **Apache-2.0**.
- Sync relay: **AGPL-3.0** (so users can self-host and copyleft improvements).
- Reuse default for permissive licenses (Apache-2.0, MIT, BSD).
- GPL/AGPL projects: UX and data-model choices only; code reuse requires
  an explicit ADR/legal review because the desktop app is Apache-2.0.
- Reject code that requires remote services, proprietary formats, broad
  filesystem access, open TCP ports, or non-rebuildable state.

Evidence convention: each row's `Evidence link` is either an upstream repo
URL or the local ADR that records the verdict.

## 1. Lossless Markdown editor and property UX

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| CodeMirror 6 + `@codemirror/lang-markdown` | [codemirror/lang-markdown](https://github.com/codemirror/lang-markdown) | MIT | Markdown language, decoration-based syntax, modular extension API | Adopt | [ADR-002](../adr/002-editor-roundtrip.md) |
| Milkdown | [Milkdown](https://github.com/Milkdown/milkdown) | MIT | ProseMirror core, plugin model, rich block layer | Adapt (reviewed, deferred) | CodeMirror 6 wins the bakeoff — see [ADR-002](../adr/002-editor-roundtrip.md) §Decision; no separate spike notes retained post-rebuild |
| SilverBullet | [silverbullet](https://github.com/silverbulletmd/silverbullet) | MIT | Pager-based MD shell, expression language | Reimplement (model too coupled) | [ADR-002](../adr/002-editor-roundtrip.md) §Alternatives considered |
| BlockSuite | block-suite/blocksuite | Apache-2.0 | Block tree, Yjs-backed editing | Reimplement (Yjs baggage) | [ADR-002](../adr/002-editor-roundtrip.md) §Alternatives considered |
| Mermaid/KaTeX (third-party renderers) | per project | MIT | Diagram and math rendering | Adopt (out-of-process render only) | [ADR-002](../adr/002-editor-roundtrip.md) §Decision; rendering lives in the editor surface, not in the canonical text |

CodeMirror 6 wins because the round-trip ADR runs in-place edits against the
canonical text and proves lossless behavior for frontmatter, wikilinks,
tables, embedded HTML, and unknown syntax.

## 2. Importers and migration

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Obsidian Importer | [obsidianmd/obsidian-importer](https://github.com/obsidianmd/obsidian-importer) | AGPL-3.0 | Format adapters for Notion/Evernote/etc. | UX + adapter patterns only | [ADR-003-git-write-policy.md §Cradle reset / open-in-place](../adr/003-git-write-policy.md); importer plan surfaces in M3 (see `docs/superpowers/specs/launch-workflows.md`) |
| jyyzsed/Notion-export-cleaner | community | MIT | Notion zip normalization | Reuse via opinionated adapter (LGPL reviewed) | M3 importer plan — no separate plan file in R0 |
| Any-Block / Logseq block extractor | community | AGPL-3.0 | Block identity heuristics | UX + data model only | M3 importer plan — no separate plan file in R0 |
| Tana JSON loader | community | MIT | Tana export parsing | Defer until alpha user demand | M3 importer plan — no separate plan file in R0 |

The desktop app is Apache-2.0; importing AGPL importers directly would require
either licensing review or adaptation of format knowledge only.

## 3. Web clipper/readability

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Obsidian Web Clipper | [obsidian-md/obsidian-web-clipper](https://github.com/obsidianmd/obsidian-clipper) | AGPL-3.0 | Native Messaging protocol, readability profile | Borrow UX and Native Messaging flow only | [threat-model.md §Adversary — Poisoned scraper target](threat-model.md); browser-extension plan surfaces in M3 |
| Mozilla Readability.js | [mozilla/readability](https://github.com/mozilla/readability) | Apache-2.0 | Article extractor | Adopt via Native Messaging sidecar | [docs/superpowers/specs/research-selector.md](../superpowers/specs/research-selector.md) — static tool registry |
| Mozilla PDF.js | [mozilla/pdf.js](https://github.com/mozilla/pdf.js) | Apache-2.0 | PDF text extraction in sidecar | Adopt (later) | [threat-model.md §Adversary — Side-loaded imported vault](threat-model.md) |

Native Messaging is the only secure browser -> desktop channel; HTTPS /
remote API / websocket-from-page approaches are explicitly disallowed by the
no-inbound-port invariant.

## 4. Sync / E2EE collaboration

See `docs/adr/005-sync-collaboration.md` for the verdict row. The
orchestrator scratchpad in `_workspace/04_adrs_index.md` cross-references
the durable ADR; no separate `_workspace/03a_sync_bakeoff.md` stub is
retained post-rebuild.

| Project | Repo | License | Status | Comment |
| --- | --- | --- | --- | --- |
| any-sync | [anyproto/any-sync](https://github.com/anyproto/any-sync) | Apache-2.0 | Strong reference, no fork | Spaces + permissions + history + self-host port |
| Syncthing | [syncthing/syncthing](https://github.com/syncthing/syncthing) | MPL-2.0 | Reference | Mature file sync; encryption model is canonical |
| Automerge / Loro | automerge / loro | MIT | Reference only (not adopted) | Live co-editing is non-goal; reconcile-always stays primary |
| Minimal encrypted Git-envelope relay | new | AGPL-3.0 (per `AGENTS.md §6`) | Adapt: client + relay per ADR | Sync native envelopes over a BORQ-style opaque store. Server is AGPL-3.0 so users can self-host and copyleft improvements; the desktop client stays Apache-2.0. |
| Seafile | haiwen/seafile | AGPL-3.0 | Interoperability only | Not an embedded foundation |
| git-remote-gcrypt | AGPL-3.0 | Reference only | Useful protocol precedent; not default |

Decision: adopt (`Syncthing`-influenced conflict semantics + `any-sync`-style
spaces + Git-compatible envelopes + native Rust relay under `crates/haven-relay/`).

## 5. MCP notes / agent bridge

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| modelcontextprotocol/servers | [mcp/servers](https://github.com/modelcontextprotocol/servers) | MIT | Reference TypeScript server skeleton | Adapt to Rust crate (`crates/haven-mcp`) | [threat-model.md §Adversary — Malicious MCP client](threat-model.md); tool-shape ADR surfaces in M2/P2 (no separate file in R0) |
| zettelkasten-mcp | [entanglr/zettelkasten-mcp](https://github.com/entanglr/zettelkasten-mcp) | MIT | Vault search/read tool shape | UX only | [docs/superpowers/specs/research-selector.md](../superpowers/specs/research-selector.md) — typed tool registry |
| brain.md | [mi4uu/brain.md](https://github.com/mi4uu/brain.md) | MIT | Conversation-to-doc recipe | UX only | [ADR-006](../adr/006-memory-engine.md) §Memory inbox UX — reviewable proposals |
| Obsidian MCP Server | [smith-and-web/obsidian-mcp-server](https://github.com/smith-and-web/obsidian-mcp-server) | MIT | Read-only tool shapes | Adapt | [docs/superpowers/specs/research-selector.md](../superpowers/specs/research-selector.md) — typed tool registry |

The haven-mcp server is a versioned headless executable with a stable JSON
schema (per AGENTS.md `§9`). External clients must not depend on a hidden
Tauri window.

## 6. Vector / RAG index

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| sqlite-vec | [asg017/sqlite-vec](https://github.com/asg017/sqlite-vec) | Apache-2.0 + commercial dual | SQLite extension for vectors | Adopt via vendored loadable extension | [ADR-007](../adr/007-local-model-selection.md) §Embedding default; in-process extension load — no second canonical store |
| SQLite's own FTS5 | bundled | public-domain | Full-text search | Adopt | [ADR-001](../adr/001-okf-adoption.md) + [ADR-007](../adr/007-local-model-selection.md); rebuild-from-files is the FTS5 + derived FTS rebuild |
| Lance, hnswlib, faiss, qdrant — server based | per project | various | Reject | Aspirational comparison: each opens a TCP port or adds Docker | [threat-model.md §Adversary — Compromised relay / second source of truth](threat-model.md) |

## 7. PDF / Zotero / research lane

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Obsidian Zotero Integration | [community plugin](https://github.com/obsidian-community/obsidian-zotero-integration) | MIT | Annotation extraction patterns | Reimplement (not adopt) | M3+ PDF/Zotero ADR (not yet written); R0 records the lane only |
| Zotero Reader | [zotero/reader](https://github.com/zotero/reader) | AGPL-3.0 | Reader UI patterns | UX only | M3+ PDF/Zotero ADR (not yet written); R0 records the lane only |
| Zotero Better Notes | [community plugin](https://github.com/windingwind/zotero-better-notes) | MIT | Markdown export profile | UX only | M3+ PDF/Zotero ADR (not yet written); R0 records the lane only |

R0 records the lane but does not commit license-pinning code. The
follow-on ADR for PDF/Zotero must decide whether citations come from a
local-only sidecar or a Zotero-friendly API the founder can self-host.

## 8. Long-term agent memory

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Hindsight | [vectorize-io/hindsight](https://github.com/vectorize-io/hindsight) | Apache-2.0 | Retain / recall / reflect behavior; temporal reasoning | Adopt as a behavior + eval reference only; **not** default embedded dependency | ADR-006-memory-engine |
| Graphiti | [getzep/graphiti](https://github.com/getzep/graphiti) | Apache-2.0 | Temporal validity, supersession, episodes | Adopt as a reference design; not embedded dependency | ADR-006-memory-engine |
| Mem0 | [mem0ai/mem0](https://github.com/mem0ai/mem0) | Apache-2.0 | Add / search / feedback APIs | Adapt as optional adapter | ADR-006-memory-engine |
| File-native baseline | n/a | n/a | Markdown observations, sqlite derived index | Adopt as canonical path | ADR-006-memory-engine + P6.2 |

Bakeoff verdict: file-native canonical memories + an optional adapter for
Hindsight or Mem0 behind a small in-process or stdio boundary. Service-DB
candidates never become second sources of truth.

## 9. Offline voice transcription

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| whisper.cpp | [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | Self-contained C++ transcription library | Adopt via audited Rust binding or stdio sidecar | [ADR-004](../adr/004-local-runtime-and-network-posture.md) §Outbound / typing posture; voice-capture plan surfaces in M3 |
| sherpa-onnx | [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) | Apache-2.0 | Streaming ASR runtime | Adapt as fallback if whisper.cpp path falters | ADR-004 §Engine lifecycle + 30 s idle shutdown |
| OpenSW | [liebe-magi/OpenSW](https://github.com/liebe-magi/OpenSW) | MIT | Tauri + whisper.cpp prototype | UX and packaging | Tauri sidecar shape per ADR-004 |

R0 verdict: whisper.cpp via `--stdio-server` style wrapper. Never an HTTP
service. Memory budget keeps whisper base under ~388 MB and small under
~852 MB on the 16 GB reference machine; unload after transcription; raw
audio deleted by default.

## 10. Cross-cutting: license register

This section exists so ADR authors never have to ask the legal question
twice.

| Verification topic | Decision |
| --- | --- |
| Does a candidate rely on a remote service? | Reject. |
| Does a candidate require a proprietary format? | Reject. |
| Does a candidate open TCP ports on the desktop? | Reject. |
| Does a candidate require Docker/Python on the user's machine? | Reject for embedded paths; allow only when isolated as a sidecar with explicit user opt-in. |
| Does a candidate allow second canonical state outside the file tree? | Reject; derived indexes, caches, and external engines must be rebuildable from the canonical files. |

Anything that violates the above triggers an R0 re-review.

## 11. Local model + inference runtime (LLM, embedding, transcription)

See `docs/adr/004-local-runtime-and-network-posture.md` for the IPC-pipe
+ network posture and `docs/adr/007-local-model-selection.md` for the
per-tier chatter/embedding matrix. This row covers the runtime side of the
same decision: subprocesses, sidecars, and binding contracts.

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Ollama | [ollama/ollama](https://github.com/ollama/ollama) | MIT | Loopback REST subprocesses, model manifest pull, telemetry opt-out | Adopt (default in v1) | [ADR-004](../adr/004-local-runtime-and-network-posture.md) |
| llama.cpp | [ggerganov/llama.cpp](https://github.com/ggerganov/llama.cpp) | MIT | Self-contained inference; GGUF/GGML formats | Defer custom sidecar to post-v1; revisit when Ollama misses a needed model | [ADR-004](../adr/004-local-runtime-and-network-posture.md) |
| OpenAI / Anthropic / Gemini HTTP clients | vendor SDKs | proprietary | Optional cloud providers | Adapt (explicit opt-in, per-call disclosure) | [ADR-004](../adr/004-local-runtime-and-network-posture.md) |
| AI SDK | [vercel/ai](https://github.com/vercel/ai) | Apache-2.0 | Provider-agnostic TS tool-calling surface | Adapt for cloud provider selection (per `docs/superpowers/specs/research-selector.md`) | [ADR-004](../adr/004-local-runtime-and-network-posture.md) |
| SearXNG | [searxng/searxng](https://github.com/searxng/searxng) | AGPL-3.0 | Self-hosted meta-search | UX + provider policy only (data-model only; no code embedding because desktop is Apache-2.0) | research-selector.md |
| Firecrawl | [mendable/firecrawl](https://github.com/mendable/firecrawl) | AGPL-3.0 (hosted SaaS) | Hosted scrape/crawl/extract | Optional provider via explicit opt-in; no embedded dependency | research-selector.md |
| Crawl4AI | [unclecode/crawl4ai](https://github.com/unclecode/crawl4ai) | Apache-2.0 | Self-hosted extract + crawl | Adapt as optional self-hosted provider | research-selector.md |
| Playwright (browser) | [microsoft/playwright](https://github.com/microsoft/playwright) | Apache-2.0 | Sandboxed browser automation | Adapt (interactive pages only; vault/keychain/sync access disabled) | threat-model.md §Adversary — Poisoned scraper target |

Hard requirements referenced from `AGENTS.md §3 / §5 / §9`: IPC pipe (Unix
domain socket / Windows named pipe) is the default engine transport; loopback
TCP exists only as a documented platform fallback when no IPC primitive is
available, no inbound port ever opens by default, static tool registration,
fetched content cannot trigger privileged tools. R0 record survives even
when an Ollama release rebases; the digest + chat template + license +
quantization pin is the release contract.
