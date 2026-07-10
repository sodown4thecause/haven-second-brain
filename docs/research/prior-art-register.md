# Prior-Art and License Register (R0)

This register supplies evidence for every "hard feature" governance decision
in R0. Each row cites the licensed repository, the verdict relative to
Haven's product invariants, a documented set of reusable modules, and an
explicit adopt / fork / adapt / reimplement call with linked evidence.

License policy (`AGENTS.md §1.6`):

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
| Milkdown | [Milkdown](https://github.com/Milkdown/milkdown) | MIT | ProseMirror core, plugin model, rich block layer | Adapt (reviewed, deferred) | P0.2 spike notes |
| SilverBullet | [silverbullet](https://github.com/silverbulletmd/silverbullet) | MIT | Pager-based MD shell, expression language | Reimplement (model too coupled) | spike notes |
| BlockSuite | block-suite/blocksuite | Apache-2.0 | Block tree, Yjs-backed editing | Reimplement (Yjs baggage) | P0.2 spike notes |
| Mermaid/KaTeX (third-party renderers) | per project | MIT | Diagram and math rendering | Adopt (out-of-process render only) | P2.2 retrieval spec |

CodeMirror 6 wins because the round-trip ADR runs in-place edits against the
canonical text and proves lossless behavior for frontmatter, wikilinks,
tables, embedded HTML, and unknown syntax.

## 2. Importers and migration

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Obsidian Importer | [obsidianmd/obsidian-importer](https://github.com/obsidianmd/obsidian-importer) | AGPL-3.0 | Format adapters for Notion/Evernote/etc. | UX + adapter patterns only | ADR-003 (open-in-place rule) |
| jyyzsed/Notion-export-cleaner | community | MIT | Notion zip normalization | Reuse via opinionated adapter (LGPL reviewed) | P3.1 importer plan |
| Any-Block / Logseq block extractor | community | AGPL-3.0 | Block identity heuristics | UX + data model only | P3.1 importer plan |
| Tana JSON loader | community | MIT | Tana export parsing | Defer until alpha user demand | P3.1 (Tana can wait) |

The desktop app is Apache-2.0; importing AGPL importers directly would require
either licensing review or adaptation of format knowledge only.

## 3. Web clipper/readability

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Obsidian Web Clipper | [obsidian-md/obsidian-web-clipper](https://github.com/obsidianmd/obsidian-clipper) | AGPL-3.0 | Native Messaging protocol, readability profile | Borrow UX and Native Messaging flow only | P3.2 browser extension plan |
| Mozilla Readability.js | [mozilla/readability](https://github.com/mozilla/readability) | Apache-2.0 | Article extractor | Adopt via Native Messaging sidecar | P3.2 |
| Mozilla PDF.js | [mozilla/pdf.js](https://github.com/mozilla/pdf.js) | Apache-2.0 | PDF text extraction in sidecar | Adopt (later) | threat-model.md |

Native Messaging is the only secure browser -> desktop channel; HTTPS /
remote API / websocket-from-page approaches are explicitly disallowed by the
no-inbound-port invariant.

## 4. Sync / E2EE collaboration

See `docs/adr/005-sync-collaboration.md` for the verdict row. Bakeoff evidence
lives under `_workspace/03a_sync_bakeoff.md` (artifact) and is summarized here:

| Project | Repo | License | Status | Comment |
| --- | --- | --- | --- | --- |
| any-sync | [anyproto/any-sync](https://github.com/anyproto/any-sync) | Apache-2.0 | Strong reference, no fork | Spaces + permissions + history + self-host port |
| Syncthing | [syncthing/syncthing](https://github.com/syncthing/syncthing) | MPL-2.0 | Reference | Mature file sync; encryption model is canonical |
| Automerge / Loro | automerge / loro | MIT | Reference only (not adopted) | Live co-editing is non-goal; reconcile-always stays primary |
| Minimal encrypted Git-envelope relay | new | Apache-2.0 | Adapt: client + relay per ADR | Sync native envelopes over BORQ-style opaque store |
| Seafile | haiwen/seafile | AGPL-3.0 | Interoperability only | Not an embedded foundation |
| git-remote-gcrypt | AGPL-3.0 | Reference only | Useful protocol precedent; not default |

Decision: adopt (`Syncthing`-influenced conflict semantics + `any-sync`-style
spaces + Git-compatible envelopes + native Rust relay under `crates/haven-relay/`).

## 5. MCP notes / agent bridge

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| modelcontextprotocol/servers | [mcp/servers](https://github.com/modelcontextprotocol/servers) | MIT | Reference TypeScript server skeleton | Adapt to Rust crate | ADR-MCP-tool-shape |
| zettelkasten-mcp | [entanglr/zettelkasten-mcp](https://github.com/entanglr/zettelkasten-mcp) | MIT | Vault search/read tool shape | UX only | P2.7 stdio plan |
| brain.md | [mi4uu/brain.md](https://github.com/mi4uu/brain.md) | MIT | Conversation-to-doc recipe | UX only | P2.5 transcripts |
| Obsidian MCP Server | [smith-and-web/obsidian-mcp-server](https://github.com/smith-and-web/obsidian-mcp-server) | MIT | Read-only tool shapes | Adapt | P2.7 stdio plan |

The haven-mcp server is a versioned headless executable with a stable JSON
schema (per AGENTS.md `§9`). External clients must not depend on a hidden
Tauri window.

## 6. Vector / RAG index

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| sqlite-vec | [asg017/sqlite-vec](https://github.com/asg017/sqlite-vec) | Apache-2.0 + commercial dual | SQLite extension for vectors | Adopt via vendored loadable extension | P2.2 embed pipeline (deferred to Phase 2) |
| SQLite's own FTS5 | bundled | public-domain | Full-text search | Adopt | P1.4 indexer |
| Lance, hnswlib, faiss, qdrant — server based | per project | various | Reject | Aspirational comparison: each opens a TCP port or adds Docker | threat model |

The chosen pipeline stays single-writer SQLite via FTS5 then `sqlite-vec`.
WAL discipline + content-hash revisions keep rebuild-from-files faithful
(see `crates/haven-index/`).

## 7. PDF / Zotero / research lane

| Project | Repo | License | Reusable modules | Decision | Evidence |
| --- | --- | --- | --- | --- | --- |
| Obsidian Zotero Integration | [community plugin](https://github.com/obsidian-community/obsidian-zotero-integration) | MIT | Annotation extraction patterns | Reimplement (not adopt) | P-PDF (out of R0 scope; design concern noted) |
| Zotero Reader | [zotero/reader](https://github.com/zotero/reader) | AGPL-3.0 | Reader UI patterns | UX only | P-PDF |
| Zotero Better Notes | [community plugin](https://github.com/windingwind/zotero-better-notes) | MIT | Markdown export profile | UX only | P-PDF |

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
| whisper.cpp | [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | Self-contained C++ transcription library | Adopt via audited Rust binding or stdio sidecar | P3.6 voice capture plan |
| sherpa-onnx | [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) | Apache-2.0 | Streaming ASR runtime | Adapt as fallback if whisper.cpp path falters | P3.6 |
| OpenSW | [liebe-magi/OpenSW](https://github.com/liebe-magi/OpenSW) | MIT | Tauri + whisper.cpp prototype | UX and packaging | P3.6 |

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
