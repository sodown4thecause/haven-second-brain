# PLAN.md — Haven Build Plan

Haven: an open-source (Apache-2.0 app / AGPL-3.0 server), local-first,
git-native second brain for **agent-native knowledge workers** who already live
in Markdown/Git and want offline AI + MCP without SaaS lock-in.

Data layer is plain Markdown + YAML-frontmatter in a Git repo, OKF v0.1
conformant on write. AI is offline by default (hardware-selected local LLM via
Ollama/llama.cpp). Skills follow the Agent Skills standard (`SKILL.md`).
Interop via a built-in MCP server. Monetized through an optional cloud layer
(E2EE sync, always-on companions, team spaces) that the free local app never
depends on.

Read `AGENTS.md` before contributing. Phases are dependency-ordered; each has
an exit test that must pass before the next phase starts.

---

## Product framing (do not skip)

### ICP (primary)

Technical knowledge workers who:

1. Already keep notes as Markdown (Obsidian/Logseq/plain Git), **or**
2. Use Cursor/Claude Code daily and want a durable personal knowledge store
   those agents can read/write via MCP.

Secondary later: privacy-maximalists leaving Notion/Anytype; researchers who
want provenance. **Not** the primary ICP for v1: non-technical Notion teams,
meeting-transcription buyers (Tana), or "pretty object database" users
(Capacities).

### Positioning

> **The agent-native second brain: your knowledge is a Git repo any local model
> or MCP client can use — offline by default, no lock-in.**

Win on: file ownership, provenance, MCP/skills interop, offline AI that is
*built in* (not a plugin maze). Do **not** try to out-Notion Notion, out-plugin
Obsidian, or out-supertag Tana in v1.

### Non-goals for launch

- Live multiplayer / Yjs co-editing
- Skill marketplace / revenue share
- Always-on Slack/Discord companions (eve)
- Firecrawl / heavy crawl engines
- Plugin marketplace
- Parity with Capacities object types or Tana supertags

### Pricing implication

Free forever: local app, local AI, MCP, Git, import, search. Paid: E2EE sync
relay, multi-device conflict UX, optional hosted companions, team bundles.
Never gate editor, search, or local chat on a network check.

### Onboarding principle

Day-0 path must produce value in <10 minutes without teaching OKF, MCP, or
skills. Sequence: open/create vault → write a note → search finds it → ask the
brain one question → (optional) connect Cursor via MCP. Skills and OKF jargon
appear after the core loop feels good.

---

## Phase 0 — Product spine (before more architecture)

Goal: lock decisions that prevent building the wrong product.

- [ ] P0.1 Write one-page ICP + 3 launch workflows (daily capture, research
      Q&A with citations, "Cursor writes a note via MCP").
- [ ] P0.2 Editor spike: BlockSuite Markdown round-trip vs CodeMirror 6 /
      Milkdown on a 200-note fixture. Kill BlockSuite if round-trip corrupts
      frontmatter, wikilinks, or tables. ADR required.
- [ ] P0.3 Git UX policy ADR: when to commit (explicit save vs debounced
      autosave vs session checkpoint); human vs agent identity; never commit
      on every keystroke; optional squash of noisy human checkpoints.
- [ ] P0.4 Hardware matrix + model auto-select table (see LLM stack below).
- [ ] P0.5 Success metrics for alpha: D1/D7 retention proxy (notes created,
      searches, chats with ≥1 citation click), import completion rate,
      MCP connect success rate.

**Exit test:** ADRs accepted; editor choice locked; three workflows demoable
on paper with UI sketches.

## Phase 1 — Core loop (files ⇄ index ⇄ editor)

Goal: edit documents in the app; every durable write is OKF-conformant and
provenance-correct in Git; search works; the index is provably disposable.

- [ ] P1.1 Scaffold Tauri 2 workspace: `src-tauri/` (Rust), `src/` (TS +
      chosen editor), `crates/` (`okf`, `haven-index`, `haven-git`),
      CI (fmt, clippy `-D warnings`, tests, tsc, eslint).
- [ ] P1.2 `crates/okf`: frontmatter parse/serialize with unknown-key
      round-trip, conformance linter (strict-write / permissive-read),
      `index.md` / `log.md` handling, inline-link extraction. Property tests.
      Treat OKF as an *interop/export contract*, not the entire UX metaphor.
- [ ] P1.3 `crates/haven-git`: repo init/open, staged writes, dual-identity
      commits (human vs `Haven Agent (<model>)`), `status --porcelain` scan,
      debounced FS watcher. Implement P0.3 commit policy.
- [ ] P1.4 `crates/haven-index`: SQLite (WAL, single-writer worker) with FTS5
      + `edge` table; full rebuild from bundle; `INDEX_SCHEMA_VERSION` gate.
      **Defer `sqlite-vec` until Phase 2** — ship keyword search first.
- [ ] P1.5 Editor shell bound to files; save pipeline → linter → git commit →
      incremental index update; typed IPC client.
- [ ] P1.6 Minimum PKM UX users expect on day one: file tree, backlinks panel,
      global search, daily note (`journal/YYYY-MM-DD.md`), wikilink
      autocomplete `[[...]]` that writes OKF-legal inline links on disk.

**Exit test:** create/edit docs in the app; `git log` shows clean OKF files
with correct authors; delete `.haven/`, reindex, and search/backlinks fully
reconstruct (permanent CI test). Daily note + backlinks work without AI.

## Phase 2 — Local AI (retrieval that earns trust)

Goal: chat with retrieval over the bundle, fully offline, citations users
click.

- [ ] P2.1 Engine manager: Ollama detection + llama.cpp sidecar fallback,
      model pull UX, health checks, **hardware auto-select** (see stack).
- [ ] P2.2 Embedding pipeline: default `nomic-embed-text` (768-d, 8k ctx) or
      EmbeddingGemma if already pulled; model-id stamped vectors; batch +
      incremental indexing off the hot path. Reindex on model change.
- [ ] P2.3 Two-way recall for v1: vector + BM25 fused with RRF + explicit
      token budget. **Defer** 2-hop graph CTE and temporal filter until
      citation quality plateaus (measure first).
- [ ] P2.4 Chat UI with citations; `chat_session` / `chat_message` as local
      files or SQLite *derived* state that can be rebuilt; 48 h open-session
      resumption prompt.
- [ ] P2.5 Graceful degradation: if no model / insufficient RAM, search +
      editor still fully usable; clear CTA to pull a smaller model.

**Exit test:** airplane-mode demo — question answered only from bundle
content; citations resolve; restart resumes session; app usable with AI
engine stopped.

## Phase 3 — Capture + migration (retention milestone)

Goal: get real vaults in; make Haven the place thoughts land.

- [ ] P3.1 Stage-1 deterministic importers: Obsidian/Logseq (wikilink
      normalize), Notion zip (UUID strip). Fixture-driven tests. **Tana JSON
      can wait** unless an alpha user needs it.
- [ ] P3.2 Browser extension via Native Messaging: readability → Markdown →
      optional summarize/tag → OKF doc with `resource:` URL.
- [ ] P3.3 Quick-capture palette (global hotkey) → inbox note or daily note.
- [ ] P3.4 Optional LLM normalization (`generateObject` + Zod) behind a
      toggle; one commit per imported file for easy revert.
- [ ] P3.5 Conflict-aware "opened existing Git/Obsidian vault" onboarding —
      do not force re-import theater.

**Exit test:** import a real 1k-page Notion or Obsidian vault; zero
non-conformant writes from Haven; `git revert` of one import commit removes
exactly that document; capture hotkey works offline.

## Phase 4 — Skills + MCP (differentiation milestone)

Goal: notes that are skills; a brain any agent can use. **Launch only after
Phases 1–3 feel sticky.**

- [ ] P4.1 Skill authoring: "New Skill" template, Agent Skills linter
      (mirrors `skills-ref validate`), skills browser.
- [ ] P4.2 Skill execution with progressive disclosure and `allowed-tools`
      enforcement. Sandbox `scripts/` as hostile input.
- [ ] P4.3 MCP server (stdio): `search_brain`, `read_document`,
      `write_document`, `list_skills`, `read_skill`, `recall_memory`;
      read-only default, per-client write grants; all writes through
      linter + provenance pipeline.
- [ ] P4.4 MCP client: attach external servers per persona; tool-permission UI.
- [ ] P4.5 "Export to Claude/Cursor" one-click for `skills/`.
- [ ] P4.6 Security review: path traversal, prompt injection via notes,
      skill script execution policy, MCP rate limits.

**Exit test:** a skill authored in Haven runs unmodified in Claude Code;
Cursor via MCP can search and (with grant) write a note as an agent-authored
Git commit.

## Phase 5 — Memory depth (only if Phase 2 citations are trusted)

- [ ] P5.1 retain: post-session fact extraction to `memory/observations/`
      with confidence + evidence links; consolidation updates rather than
      overwrites.
- [ ] P5.2 reflect: query observations before raw notes; persona docs with
      hard directives vs disposition.
- [ ] P5.3 Proactive git-watcher prompts for external edits ("parse and
      link?").

**Exit test:** a fact from session 1 is recalled in session 5 with provenance
visible as a Git diff under `memory/`.

## Phase 6 — Multi-device sync (first paid wedge)

- [ ] P6.1 E2EE sync relay (AGPL, self-hostable): encrypted objects, device
      key exchange, relay never holds plaintext. Prefer **async Git-compatible
      sync** before inventing a second CRDT truth.
- [ ] P6.2 Mobile **capture companion** (iOS/Android): inbox + daily note +
      search; full editor can lag. Do not ship desktop-only forever.
- [ ] P6.3 Billing + entitlements (app never gates local features on network).

## Phase 7 — Team + marketplace (explicitly later)

- [ ] P7.1 Shared bundles: start with Git permissions + review workflow; add
      Yjs live co-editing only if async collaboration fails users.
- [ ] P7.2 Provenance audit exports (model, prompt hash, per commit).
- [ ] P7.3 Companion provisioning (eve or plain Vercel functions + Chat SDK
      fallback) — paid only.
- [ ] P7.4 Skill & bundle registry with inspect-before-install — after a
      critical mass of local skills exists.

---

## Recommended local LLM stack

| Tier | Hardware assumption | Chat default | Embeddings | Notes |
|---|---|---|---|---|
| Floor | 8 GB RAM, iGPU/CPU | Phi-4-mini or Gemma 4 E2B (Q4) | `nomic-embed-text` | Search-first; chat is best-effort |
| Default | 16 GB RAM, CPU or ≤8 GB VRAM | **Gemma 4 E4B (Q4_K_M)** | `nomic-embed-text` (768-d) | Fast TTFT, native tool/structured tokens; keep `num_ctx` modest (8–16k) |
| Quality | 16–32 GB RAM + ≥8 GB VRAM / Apple Silicon 18+ GB | Gemma 4 12B or Qwen3 8B (Q4/Q5) | `nomic-embed-text` or Qwen3-Embedding 0.6B | Prefer for ingestion + multi-step skills |
| Headroom | ≥24 GB VRAM / 32 GB+ unified | Gemma 4 26B-A4B MoE | same | Optional; never the download default |

**Keep Gemma 4 E4B as the shipped default** for the median laptop: Apache-2.0,
strong structured output / tool-calling for its size, fits the <2 s first-token
budget on GPU and remains usable on CPU. Do **not** default to 12B/26B — pull
size and RAM pressure kill onboarding.

**Change from prior plan:** do not default embeddings to EmbeddingGemma.
Its ~2k token context is a retrieval footgun next to `nomic-embed-text` (8k)
or Qwen3-Embedding. Keep EmbeddingGemma as an alternate for Gemma-stack fans;
always stamp `model_id` on vectors.

**Fallbacks:** (1) smaller local model, (2) user-configured cloud provider via
AI SDK (explicit opt-in, never silent), (3) retrieval-only answers with
extractive snippets when generation is unavailable.

**Honesty bar:** small local models will fail multi-step agent loops. Skills
and MCP tool use must assume failure — validate schemas, cap loops, show
diffs before applying writes.

---

## Standing risks (watch continuously)

| Risk | Mitigation |
|---|---|
| BlockSuite churn / Markdown lossy adapters | P0.2 spike; pin versions; OKF serializer owns data; editor swappable |
| Commit-on-save Git noise / merge pain | P0.3 policy; session checkpoints; agent commits always atomic & separate |
| OKF v0.1 still a draft (published 2026-06) | `okf_version`; permissive read; UX speaks "notes/vault" not "OKF bundle" |
| Four-way recall overfit | Ship BM25+vector; add graph/temporal only with eval harness |
| eve immaturity | Paid tier only; fallback = Vercel functions + Chat SDK |
| Small-model quality on ingestion | Deterministic stage 1 first; schema-constrained stage 2; recommend 12B/8B quality tier |
| No mobile → churn vs Obsidian/Anytype | Phase 6 capture companion is the sync wedge, not marketplace |
| MCP write = prompt-injection surface | Read-only default; path allowlists; show diff; dual-identity commits |
| `sqlite-vec` Windows build pain | Bundle SQLite; CI on Win/macOS/Linux before Phase 2 exit |
| Competing with Obsidian plugin combo | Win on integrated offline AI + MCP + provenance, not plugin count |

---

## Prioritized roadmap (summary)

**Must-have (alpha → public launch):** Phase 0–3 complete; Phase 4 MCP server
(read + granted write) + one polished skill path; airplane-mode demo;
Obsidian import; daily note + capture; hardware-aware local LLM.

**Should-have (post-launch):** Memory retain/reflect; MCP client; browser
extension polish; mobile capture companion; E2EE sync; graph/temporal recall
if evals justify it.

**Later:** Yjs co-editing, eve companions, skill marketplace, Firecrawl,
team RBAC, Tana importer, plugin API.
