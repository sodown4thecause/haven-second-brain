# PLAN.md — Haven Build Plan

Haven: an open-source (Apache-2.0 app / AGPL-3.0 server), local-first,
git-native second brain for **agent-native knowledge workers** who already live
in Markdown/Git and want offline AI + MCP without SaaS lock-in.

**Alpha promise:** open an existing Markdown/Git vault safely, make it fast to
search and ask with citations, then let approved agents propose durable edits
with visible Git provenance. Do not ship alpha as a generic PKM suite.

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

### Founder-first operating mode

Haven is built for the founder's real vault first and promoted more broadly
only after it survives daily use. The first proof is not signups: it is 30
consecutive days of using Haven for capture, cited recall, voice notes, and
agent-proposed edits without returning to a parallel system for those jobs.
Optimize the first-run defaults for one local user on a 16 GB laptop. Keep the
file formats, permissions, and extension boundaries general enough for later
users, but do not build teams, tenancy, marketplaces, or generalized workflow
builders before the founder workflow is dependable.

### Positioning

> **The agent-native second brain: your knowledge is a Git repo any local model
> or MCP client can use — offline by default, no lock-in.**

Win on: file ownership, provenance, MCP/skills interop, offline AI that is
*built in* (not a plugin maze). Do **not** try to out-Notion Notion, out-plugin
Obsidian, or out-supertag Tana in v1.

Launch wedge: Haven is the safest way for local models and coding agents to
read and propose changes to a user's long-lived knowledge repo. "Agent-native"
must mean a working workflow on day one, not a later marketplace promise.

### Non-goals for launch

- Live multiplayer / Yjs co-editing
- Canvas / whiteboard parity
- Full object-database parity with Notion/Capacities/Anytype
- Skill marketplace / revenue share
- Arbitrary skill script execution before the core vault workflow is trusted
- Always-on Slack/Discord companions (eve)
- Firecrawl / heavy crawl engines
- Plugin marketplace
- Parity with Capacities object types or Tana supertags

### Pricing implication

Free forever: local app, local AI, MCP, Git, import, search. Paid: E2EE sync
relay, multi-device conflict UX, optional hosted companions, team bundles.
Never gate editor, search, or local chat on a network check.

Treat this as a hypothesis, not a business model already proved. Before sync
implementation, test willingness to pay with a landing page and 15-20 ICP
interviews. Price the paid tier against its real cost drivers (relay storage,
egress, support, and mobile maintenance), offer a time-limited sync trial, and
avoid promising unlimited attachment storage before usage is measured.

### Onboarding principle

Day-0 path must produce value in <10 minutes without teaching OKF, MCP, or
skills. Sequence: open/create vault -> compatibility report -> write or select
a note -> search finds it -> ask the brain one question with citations ->
optional agent connection proposes a diff the user can approve. Skills and OKF
jargon appear after the core loop feels good.

Existing-vault rule: opening an Obsidian, Logseq, or plain Git vault starts in
read/index mode. Haven must never mass-rewrite user files to make them OKF-pure.
Strict OKF applies to files Haven creates or files the user explicitly migrates.

## Prior-art first rule (hard features)

Do not start high-complexity features from a blank page. Before a design PR for
any item below, create or update an ADR/prior-art note with: repos reviewed,
license compatibility, portability/security fit, candidate reusable modules, and
why Haven should adopt, fork, adapt, or reimplement. Favor reuse when it
preserves the product invariants; reject code that requires remote services,
proprietary formats, broad filesystem access, open TCP ports, or non-rebuildable
state.

Review these projects before building the hard parts:

- Lossless Markdown editor and property UX: [CodeMirror Markdown][gh-codemirror-markdown],
  [Milkdown][gh-milkdown], [SilverBullet][gh-silverbullet].
- Importers and migration: [Obsidian Importer][gh-obsidian-importer] before
  writing Notion/Evernote/OneNote/Google Keep converters.
- Web clipper/readability: [Obsidian Web Clipper][gh-obsidian-clipper].
- Sync/conflict/local-first collaboration: [Automerge][gh-automerge],
  [Yjs][gh-yjs], [Electric][gh-electric], [any-sync][gh-any-sync], and
  [Anytype client][gh-anytype-ts].
- MCP notes/agent bridge: [MCP servers][gh-mcp-servers],
  [zettelkasten-mcp][gh-zettelkasten-mcp], [brain.md][gh-brain-md], and
  [Obsidian MCP Server][gh-obsidian-mcp-server].
- Vector/RAG index: [sqlite-vec][gh-sqlite-vec] before custom vector storage.
- PDF/Zotero/research lane: [Obsidian Zotero Integration][gh-obsidian-zotero],
  [Zotero Better Notes][gh-zotero-better-notes], and
  [Zotero Reader][gh-zotero-reader].
- Long-term agent memory: [Hindsight][gh-hindsight],
  [Graphiti][gh-graphiti], and [Mem0][gh-mem0].
  Hindsight is the primary behavior/evaluation reference for retain, recall,
  temporal memory, and reflect; Graphiti is the reference for temporal validity,
  supersession, episodes, and provenance. Do not adopt their Python,
  graph-database, Docker, or HTTP deployment shapes as Haven's default without
  proving file truth, offline operation, and the no-open-port invariant.
- Offline voice transcription: [whisper.cpp][gh-whisper-cpp],
  [sherpa-onnx][gh-sherpa-onnx], and [OpenSW][gh-opensw]. Prefer adapting
  whisper.cpp through a Rust binding or stdio sidecar; OpenSW is useful Tauri
  prior art, not an automatic wholesale fork.

License warning: MIT/Apache-2.0 projects are easiest to reuse. GPL/AGPL projects
can inform UX and data-model choices, but code reuse needs an explicit ADR/legal
review because the desktop app is Apache-2.0.

[gh-any-sync]: https://github.com/anyproto/any-sync
[gh-anytype-ts]: https://github.com/anyproto/anytype-ts
[gh-automerge]: https://github.com/automerge/automerge
[gh-brain-md]: https://github.com/mi4uu/brain.md
[gh-codemirror-markdown]: https://github.com/codemirror/lang-markdown
[gh-electric]: https://github.com/electric-sql/electric
[gh-graphiti]: https://github.com/getzep/graphiti
[gh-mcp-servers]: https://github.com/modelcontextprotocol/servers
[gh-milkdown]: https://github.com/Milkdown/milkdown
[gh-hindsight]: https://github.com/vectorize-io/hindsight
[gh-mem0]: https://github.com/mem0ai/mem0
[gh-obsidian-clipper]: https://github.com/obsidianmd/obsidian-clipper
[gh-obsidian-importer]: https://github.com/obsidianmd/obsidian-importer
[gh-obsidian-mcp-server]: https://github.com/smith-and-web/obsidian-mcp-server
[gh-obsidian-zotero]: https://github.com/obsidian-community/obsidian-zotero-integration
[gh-opensw]: https://github.com/liebe-magi/OpenSW
[gh-sherpa-onnx]: https://github.com/k2-fsa/sherpa-onnx
[gh-silverbullet]: https://github.com/silverbulletmd/silverbullet
[gh-sqlite-vec]: https://github.com/asg017/sqlite-vec
[gh-whisper-cpp]: https://github.com/ggml-org/whisper.cpp
[gh-yjs]: https://github.com/yjs/yjs
[gh-zettelkasten-mcp]: https://github.com/entanglr/zettelkasten-mcp
[gh-zotero-better-notes]: https://github.com/windingwind/zotero-better-notes
[gh-zotero-reader]: https://github.com/zotero/reader

---

## Phase 0 — Product spine (before more architecture)

Goal: lock decisions that prevent building the wrong product.
**Timebox: 2 weeks.** Only P0.2 (editor spike) blocks Phase 1 scaffolding;
everything else can finish in parallel with early P1 work.

- [ ] P0.1 Write one-page ICP + 3 launch workflows: safe existing-vault open,
      research Q&A with citations, and "Cursor proposes a note patch via MCP
      and the human approves the diff."
- [ ] P0.2 Editor spike: BlockSuite Markdown round-trip vs CodeMirror 6 /
      Milkdown on a 200-note fixture. Default to CodeMirror 6 unless a richer
      editor proves lossless round-trip for frontmatter, wikilinks, block refs,
      tables, embedded HTML, comments, and unknown Markdown. ADR required.
- [ ] P0.3 Git UX policy ADR: when to commit (explicit save vs debounced
      autosave vs session checkpoint); human vs agent identity; never commit
      on every keystroke; optional squash of noisy human checkpoints; agent
      writes are always atomic and separately authored.
- [ ] P0.4 Hardware matrix + model auto-select table (see LLM stack below).
- [ ] P0.5 Success metrics for alpha: D1/D7 retention proxy (notes created,
      searches, chats with at least one citation click), safe-vault-open
      completion rate, approved patch acceptance rate, model setup completion
      rate, import completion rate, MCP connect success rate.
- [ ] P0.6 Local runtime privacy ADR: Ollama/llama.cpp integration, loopback-only
      port policy, no silent model pulls, explicit cloud-provider opt-in, and
      failure mode when no engine is installed.
- [ ] P0.7 Hard-feature prior-art ADR: record the OSS projects reviewed,
      license/build/runtime fit, reusable modules, and adopt/fork/reimplement
      decision before implementing sync, rich editor surfaces, browser clipper,
      PDF/Zotero, MCP writes, long-term memory, voice transcription,
      arbitrary skill execution, or major importers.
- [ ] P0.8 Run a 30-day founder dogfood on the real vault, plus 5-8 external
      target-user sessions before public beta. Set numeric gates: zero
      destructive or silent rewrites, median time-to-first-cited-answer under
      10 minutes, >=80% safe-vault-open completion for external testers, and
      the founder uses Haven for >=80% of intended capture/recall workflows by
      day 30. Interview non-activated testers; event counts alone will not
      explain failure.

**Exit test:** ADRs accepted; editor choice locked; the three alpha workflows
are demoable on paper with UI sketches and user-visible trust states.

## Phase 1 — Core loop (files ⇄ index ⇄ editor)

Goal: open or create a vault; edit documents in the app; every Haven durable
write is OKF-conformant and provenance-correct in Git; search works; the index
is provably disposable.

- [ ] P1.1 Scaffold Tauri 2 workspace: `src-tauri/` (Rust), `src/` (TS +
      chosen editor), `crates/` (`okf`, `haven-index`, `haven-git`),
      CI (fmt, clippy `-D warnings`, tests, tsc, eslint).
- [ ] P1.2 `crates/okf`: frontmatter parse/serialize with unknown-key
      round-trip, conformance linter (strict-write / permissive-read),
      `index.md` / `log.md` handling, inline-link extraction. Property tests.
      Treat OKF as an *interop/export contract*, not the entire UX metaphor.
      Never rewrite existing user files for OKF conformance unless the user
      chooses an explicit migration.
- [ ] P1.3 `crates/haven-git`: repo init/open, staged writes, dual-identity
      commits (human vs `Haven Agent (<model>)`), `status --porcelain` scan,
      debounced FS watcher. Implement P0.3 commit policy and automatic conflict
      resolution strategy (e.g., writing offline conflicts to side-by-side
      files or device-specific branches instead of inserting raw git conflict
      markers that break the editor parser). Add an activity log, restore
      affordance, and conflict inbox so Git state is user-visible.
      Stage only Haven-owned paths using an isolated index or equivalent exact
      path API; never absorb a user's unrelated staged changes. Use atomic
      replace, expected-content hashes, symlink/path confinement, and a
      pre-write recovery snapshot. Git history is not a backup until a remote
      exists.
- [ ] P1.4 `crates/haven-index`: SQLite (WAL, single-writer worker) with FTS5
      + `edge` table; full rebuild from bundle; `INDEX_SCHEMA_VERSION` gate.
      Benchmark a 10k-document fixture first; add bounded parallel parsing only
      if the measured bottleneck justifies it. Correctness requires a content
      hash/revision per file, idempotent watcher reconciliation, rename/delete
      handling, and recovery after crashes or event loss. **Defer `sqlite-vec`
      until Phase 2** — ship keyword search first.
- [ ] P1.5 Editor shell bound to files; save pipeline → linter → git commit →
      incremental index update; typed IPC client. Include a raw Markdown escape
      hatch for any document the rich surface cannot safely round-trip.
- [ ] P1.6 Minimum PKM UX users expect on day one: file tree, backlinks panel,
      global search, daily note (`journal/YYYY-MM-DD.md`), wikilink
      autocomplete `[[...]]` that writes OKF-legal inline links on disk.
- [ ] P1.7 Safe existing-vault mode: open Obsidian/Logseq/plain-Git folders in
      read/index mode first; show compatibility report (frontmatter, links,
      unsupported syntax, ignored files, index coverage); require explicit
      opt-in before Haven writes.
- [ ] P1.8 Bases-lite property views: query YAML frontmatter and backlinks into
      simple table/list views with saved filters. This is not full Notion
      database parity; it is the minimum object/property workflow users now
      expect.

**Exit test:** create/edit docs in the app; `git log` shows clean OKF files
with correct authors; delete `.haven/`, reindex, and search/backlinks fully
reconstruct (permanent CI test). Open a real Obsidian/Logseq fixture with zero
writes until opt-in. Daily note, backlinks, and Bases-lite views work without
AI.

## Phase 2 — Local AI (retrieval that earns trust)

Goal: chat with retrieval over the bundle, fully offline, citations users
click.

- [ ] P2.1 Engine manager: Ollama detection + automated installer/launcher UX,
      model pull UX, health checks, local benchmark, **hardware auto-select**
      (see stack). Never pull a model silently. **Defer custom llama.cpp
      sidecar** to avoid multi-platform compilation and distribution overhead
      in v1.
      Pin and verify runtime/model artifacts, bind only to loopback, document
      the webview-to-engine transport and CSP/CORS boundary, and make model
      deletion/storage location visible. A model tag is not a reproducible
      release unless its digest and chat template are recorded.
- [ ] P2.2 Embedding pipeline: default to `qwen3-embedding:0.6b` when the
      hardware benchmark passes; fall back to `nomic-embed-text` for low-memory
      devices or fastest setup. Model-id stamped vectors; batch + incremental
      indexing off the hot path. Reindex on model change. Pin exact model tags
      in release docs.
- [ ] P2.3 Retrieval eval harness **before** recall tuning: ~30
      question/gold-document pairs from fixture vaults; every recall change
      must show a win here. This harness is the gate for adding graph or
      temporal recall later.
- [ ] P2.4 Two-way recall for v1: vector + BM25 fused with RRF + explicit
      token budget. **Defer** 2-hop graph CTE and temporal filter until the
      eval harness shows citation quality has plateaued.
- [ ] P2.5 Chat UI with citations. Durable transcripts are user data and live
      as local files with explicit retention controls; SQLite may cache them
      but is never their only copy. Ephemeral chats are clearly labeled and may
      remain memory-only. Add a 48 h open-session resumption prompt.
- [ ] P2.6 Graceful degradation: if no model / insufficient RAM, search +
      editor still fully usable; clear CTA to pull a smaller model.
- [ ] P2.7 MCP server v0 (stdio, **read-only**): `search_brain`,
      `read_document`. No grants UI needed — nothing can write. This is the
      adoption hook for the Cursor/Claude Code ICP; applied writes and skills
      wait for Phase 4.
      Ship it as a versioned headless executable with an explicit vault path,
      stable JSON schemas, bounded responses, and useful behavior when the GUI
      is closed. Do not make external clients depend on a hidden Tauri window.
- [ ] P2.8 Approved patch proposal MCP: `propose_document_patch` and
      `create_note_draft` may prepare a local diff, but cannot apply it without
      human approval in Haven. This gives the agent-native workflow without
      opening the full write surface yet.
- [ ] P2.9 Citation trust workbench: show retrieved snippets, source paths,
      ranking signals, and "why this citation" details during alpha debugging.
      Keep the user-facing chat simple, but make retrieval quality auditable.
- [ ] P2.10 Threat model the complete retrieval path: hostile note content,
      prompt injection, oversized/binary files, secrets entering prompts,
      optional-cloud egress, malicious MCP clients, and poisoned model files.
      Cloud use gets a per-provider disclosure and a preflight showing which
      note excerpts leave the device.
- [ ] P2.11 Ship a focused 16 GB Qwen feature pack using `qwen3.5:4b` Q4:
      cited vault Q&A; summarize selection/current note; schema-constrained
      title, tags, links, and frontmatter proposals; daily/weekly review;
      duplicate or contradictory-note suggestions; approved patch generation;
      and voice-transcript cleanup. Every durable mutation is a previewed diff.
      Default to one generation job at a time, an 8k context cap, and bounded
      retrieval. Do not load chat, embedding, and transcription models
      concurrently when that would push the machine into swap.

**Exit test:** airplane-mode demo — question answered only from bundle
content; citations resolve; restart resumes session; app usable with AI
engine stopped; Cursor connected via MCP can search/read the vault and propose
a patch that appears as a human-reviewable diff.

## Phase 3 — Capture + migration (retention milestone)

Goal: get real vaults in; make Haven the place thoughts land.

- [ ] P3.1 Stage-1 deterministic importers and compatibility adapters:
      Obsidian/Logseq open-in-place first, optional copy/migrate second
      (wikilink normalize only on explicit migration), Notion zip with UUID
      strip and database rows mapped into YAML properties + Bases-lite views.
      Fixture-driven tests. **Tana JSON can wait** unless an alpha user needs
      it.
- [ ] P3.2 Browser extension via Native Messaging: readability → Markdown →
      optional summarize/tag → OKF doc with `resource:` URL.
- [ ] P3.3 Quick-capture palette (global hotkey) → inbox note or daily note.
      **Pull forward Mobile Capture MVP:** ship a lightweight web-capture PWA
      or platform Shortcut/share action that appends through an explicitly
      configured sync target. Do not imply that a PWA can safely write to an
      arbitrary Git working tree. Solve the mobile gap early without waiting
      for the full companion.
- [ ] P3.4 Import quality dashboard: show imported/skipped files, unsupported
      syntax, broken links, attachment misses, property mapping warnings, and
      one-click revert instructions. Optional LLM normalization
      (`generateObject` + Zod) stays behind a toggle; one commit per imported
      file or deterministic batch for easy revert.
- [ ] P3.5 Conflict-aware "opened existing Git/Obsidian vault" onboarding:
      detect dirty worktrees, unpushed commits, ignored files, sync artifacts,
      and common mobile sync layouts. Do not force re-import theater.
- [ ] P3.6 Offline voice capture and transcription: adapt `whisper.cpp`
      (MIT) through an audited Rust binding or a stdio sidecar, never an HTTP
      server. Start with `base` for fast capture and `small` for the 16 GB
      quality default; both remain below ~1 GB runtime memory according to the
      upstream project. Add VAD, timestamps, language choice, microphone
      selection, model download/digest verification, and a transcript review
      screen. Raw audio is deleted after successful transcription by default;
      keeping it is an explicit per-recording choice. Qwen cleanup or
      extraction runs only after the raw transcript is saved and remains
      reversible.

**Exit test:** open a real 1k-page Obsidian/Logseq vault without writes; import
a real Notion export with database rows visible in Bases-lite views; zero
non-conformant writes from Haven; `git revert` of one import commit removes
exactly that document; capture hotkey works offline; a five-minute recording
is transcribed offline, reviewed, and saved without retaining audio by default.

## Phase 4 — Skills + MCP (differentiation milestone)

Goal: notes that are skills; a brain any agent can use. **Launch only after
Phases 1-3 feel sticky and the P2 approved-diff workflow is trusted.**

- [ ] P4.1 Skill authoring: "New Skill" template, Agent Skills linter
      (mirrors `skills-ref validate`), skills browser.
- [ ] P4.2 Skill execution with progressive disclosure and `allowed-tools`
      enforcement. Sandbox `scripts/` as hostile input. Ship authoring,
      linting, and export before arbitrary script execution if sandboxing is
      not ready.
- [ ] P4.3 Extend the P2 MCP server from approved patch proposals to granted
      writes: `write_document`, `list_skills`, `read_skill`, `recall_memory`;
      read-only remains the default, per-client write grants are persisted and
      revocable, and all writes go through linter + provenance pipeline with
      diff preview.
- [ ] P4.4 MCP client: attach external servers per persona; tool-permission UI.
- [ ] P4.5 "Export to Claude/Cursor" one-click for `skills/`.
- [ ] P4.6 Security review: path traversal, prompt injection via notes,
      skill script execution policy, MCP rate limits.

**Exit test:** a skill authored in Haven runs unmodified in Claude Code;
Cursor via MCP can search and (with grant) write a note as an agent-authored
Git commit.

## Phase 5 — Multi-device sync (first paid wedge)

Interim mobile story (free, day one, document it): the vault is plain
Markdown + Git, so users can pair existing mobile editors (Obsidian Mobile,
GitJournal, a-Shell + git) with their own sync. Haven's paid tier competes on
being the *boring, reliable* version of that: clear conflict state, selective
sync, version recovery, and agent-provenance audit.

- [ ] P5.1 E2EE sync relay (AGPL, self-hostable): encrypted objects, device
      key exchange, relay never holds plaintext. Prefer **async Git-compatible
      sync** before inventing a second CRDT truth. Ship conflict inbox,
      restore snapshots, sync activity log, and selective attachment sync as
      part of the paid wedge, not polish.
- [ ] P5.2 Mobile **capture companion** (iOS/Android): inbox + daily note +
      search; full editor can lag. Do not ship desktop-only forever.
- [ ] P5.3 Billing + entitlements (app never gates local features on network).

## Phase 6 — Memory depth (only if Phase 2 citations are trusted)

- [ ] P6.1 Memory prior-art ADR and benchmark: compare Hindsight, Graphiti,
      Mem0, and a simple file-native baseline on a small HavenLongMemEval
      fixture built from the founder's workflows. Score temporal recall,
      contradiction handling, evidence attribution, latency, peak RAM, and
      recovery after the derived index is deleted. Hindsight is the primary
      behavioral reference and Graphiti the temporal-provenance reference, not
      default embedded dependencies: their Python/database/service shapes
      conflict with Haven's desktop invariants.
- [ ] P6.2 File-backed retain: write candidate world facts, experiences,
      preferences, and decisions to `memory/observations/` with timestamps,
      confidence, source links, scope, and supersession relationships. SQLite,
      vectors, entities, and mental-model caches remain derived.
- [ ] P6.3 Recall and reflect: use Hindsight-inspired temporal/entity retrieval
      and evidence-linked mental models, but run extraction/consolidation only
      during idle periods or explicit review so Qwen 4B does not compete with
      interactive chat. Consolidation proposes updates; it never silently
      overwrites source memories.
- [ ] P6.4 Memory inbox: let the user approve, correct, pin, forget, or mark a
      memory private; show why it was recalled and which source supports it.
      Deleting a memory removes its derived vectors on the next reconciliation.
- [ ] P6.5 Fork or adopt code only after P6.1. Prefer small MIT/Apache modules
      that can run in-process or over stdio. Do not bundle Docker, require a
      Python runtime, open a local TCP port, or introduce a second canonical
      database merely to claim Hindsight compatibility.

**Exit test:** a fact from session 1 is recalled in session 5 with provenance
visible as a Git diff under `memory/`.

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
| Floor | 8 GB RAM, CPU/iGPU | None by default; optional `gemma4:e2b` or verified Qwen3.5 2B quant | `nomic-embed-text` | Search-first; retrieval-only answers are valid |
| Default | 16 GB RAM, CPU or <=8 GB VRAM | `qwen3.5:4b` Q4 after benchmark | `qwen3-embedding:0.6b` or `nomic-embed-text` | Apache-2.0; strong text, vision, and tool-use fit; verify Ollama support |
| Quality | 16-32 GB RAM + >=8 GB VRAM / Apple Silicon 18+ GB | `gemma4:e4b` Q4 or `qwen3.5:9b` Q4 | `qwen3-embedding:0.6b` or `qwen3-embedding:4b` | Gemma E4B Q4 weights are ~4.5 GB before KV/runtime overhead |
| Headroom | >=16 GB VRAM / 32 GB+ unified | `gemma4:12b` Q4; larger models only after benchmark | same | Optional; never the download default |

**Default policy:** do not ship or pull one universal chat model. First-run setup
benchmarks the device, explains the tradeoff, and lets the user choose. Search,
editor, and MCP read must work before any model is installed.

**Model choice:** use `qwen3.5:4b` Q4 as the provisional recommendation for
the median 16 GB laptop: it is Apache-2.0, multimodal, and designed for tool use,
but Haven must benchmark the exact Ollama/llama.cpp build before release.
Keep `gemma4:e4b` Q4 as the quality alternate; Google documents ~4.5 GB for
weights alone, so long context and runtime overhead still make it a poor
universal 8 GB-RAM default. Pin model digest, quantization, chat template,
license, context cap, and eval result rather than only a mutable tag.

**16 GB runtime budget:** optimize for sequential local work, not a pile of
resident models. Keep Qwen 3.5 4B Q4 at an 8k default context, queue generation
jobs, unload Whisper after transcription, and pause embedding batches during
interactive chat or recording. The upstream whisper.cpp estimates are ~388 MB
for `base` and ~852 MB for `small`; both are realistic companions to a 4B
Q4 model when they are not run concurrently with reindexing. Release gates use
measured peak working-set and swap activity on Windows 16 GB, Apple Silicon
16/18 GB, and Linux 16 GB reference machines.

**Embedding choice:** prefer `qwen3-embedding:0.6b` when hardware allows, with
`nomic-embed-text` as the tiny fallback. Do not bake a fixed context-window
assumption into code; verify the selected Ollama tag at release time and store
the embedding `model_id`, dimensions, and chunking policy with vectors.

**Fallbacks:** (1) smaller local model, (2) user-configured cloud provider via
AI SDK (explicit opt-in, never silent), (3) retrieval-only answers with
extractive snippets when generation is unavailable.

**Honesty bar:** small local models will fail multi-step agent loops. Skills
and MCP tool use must assume failure: validate schemas, cap loops, show diffs
before applying writes, and keep retrieval-only answers available.

---

## Standing risks (watch continuously)

| Risk | Mitigation |
|---|---|
| Rich editor corrupts Markdown | Default to CodeMirror 6 unless P0.2 proves a richer editor is lossless; OKF serializer owns data; raw Markdown escape hatch |
| Existing-vault trust loss from mass rewrites | Safe existing-vault mode starts read/index only; strict OKF applies only to Haven-created or explicitly migrated files |
| Commit-on-save Git noise / merge pain | P0.3 policy; session checkpoints; agent commits always atomic & separate |
| OKF v0.1 still a draft (published 2026-06) | `okf_version`; permissive read; UX speaks "notes/vault" not "OKF bundle" |
| Ollama/engine violates no-open-port invariant | P0.6 local runtime ADR; loopback-only process; explicit user-visible engine state; no silent remote endpoints |
| Mandatory model download hurts onboarding | No universal default pull; benchmark first; search/editor/MCP work without a model |
| Hindsight adds a second truth or server stack | Use it as behavior/eval prior art; canonical memories stay as files; no Docker/Python/Postgres/open-port default |
| Voice capture becomes another always-running service | Adapt whisper.cpp in-process or over stdio; explicit model pull; unload after transcription; delete raw audio by default |
| Missing database/property workflows vs competitors | Bases-lite in Phase 1; full object database parity remains a non-goal |
| Four-way recall overfit | Ship BM25+vector; add graph/temporal only with eval harness |
| eve immaturity | Paid tier only; fallback = Vercel functions + Chat SDK |
| Small-model quality on ingestion | Deterministic stage 1 first; schema-constrained stage 2; recommend 12B/8B quality tier |
| No mobile → churn vs Obsidian/Anytype | Phase 5 capture companion is the sync wedge, not marketplace |
| MCP write = prompt-injection surface | Read-only default; P2 proposal-only diffs; path allowlists; human approval; dual-identity commits |
| Paid sync not differentiated from Obsidian Sync | Compete on conflict inbox, Git-native recovery, selective sync, and agent-provenance audit |
| Rebuilding hard OSS features from scratch | Prior-art ADR before implementation; adopt/fork proven libraries when license, offline behavior, and security fit |
| `sqlite-vec` Windows build pain | Bundle SQLite; CI on Win/macOS/Linux before Phase 2 exit |
| Competing with Obsidian plugin combo | Win on integrated offline AI + MCP + provenance, not plugin count |

---

## Prioritized roadmap (summary)

**Must-have (private alpha):** Phase 0; P1.1-P1.7; Phase 2; safe existing-vault
mode; editor locked by lossless round-trip tests; daily note, backlinks, global
search, hardware-aware model manager with no required model pull; retrieval
eval harness green; read MCP + approved patch proposal MCP; airplane-mode demo
with clickable citations. Bases-lite, Notion migration, and the browser
extension do not block testing the launch wedge.

**Must-have (founder dogfood):** complete the focused Qwen 3.5 4B feature pack;
global capture; offline whisper.cpp voice transcription; approved transcript
cleanup; and the Hindsight-vs-file-native memory benchmark. Use these on the
real vault for 30 days before promoting Haven broadly. Full autonomous
reflection does not block dogfood; a trustworthy memory inbox does.

**Must-have (public beta):** Bases-lite; global and mobile quick capture;
import-quality reporting for open-in-place Obsidian/Logseq vaults; one
deterministic migration path selected from actual alpha demand; recovery UX;
and measured proof that the private-alpha activation and safety gates hold.

**Should-have (post-beta):** E2EE sync with conflict inbox and mobile capture
companion before speculative memory depth; granted MCP writes and one polished
skill path; MCP client; browser extension; PDF/Zotero lane; graph/temporal
recall only if evals justify it.

**Later:** Yjs co-editing, eve companions, arbitrary skill script execution,
skill marketplace, Firecrawl, team RBAC, Tana importer, plugin API, canvas.
