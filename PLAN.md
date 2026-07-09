# PLAN.md — Haven Build Plan

Haven: an open-source (Apache-2.0 app / AGPL-3.0 server), local-first,
git-native second brain. Data layer is a plain Markdown + YAML-frontmatter Git
repo conforming to OKF v0.1; AI is 100% offline by default (Gemma 4 via
Ollama/llama.cpp, EmbeddingGemma for vectors); skills follow the Agent Skills
standard (`SKILL.md`); interop via a built-in MCP server and client. Monetized
through a cloud layer (E2EE sync relay, always-on Vercel eve companions, team
spaces, skill marketplace) that the free local app never depends on.

Read `AGENTS.md` before contributing. Phases are dependency-ordered; each has
an exit test that must pass before the next phase starts.

---

## Phase 1 — Core loop (files ⇄ index ⇄ editor)

Goal: edit documents in the app; every save is an OKF-conformant file and a
provenance-correct Git commit; search works; the index is provably disposable.

- [ ] P1.1 Scaffold Tauri 2 workspace: `src-tauri/` (Rust), `src/` (TS +
      BlockSuite editor shell), `crates/` (`okf`, `haven-index`, `haven-git`),
      CI (fmt, clippy `-D warnings`, tests, tsc, eslint).
- [ ] P1.2 `crates/okf`: frontmatter parse/serialize with unknown-key
      round-trip, conformance linter (strict-write / permissive-read),
      `index.md` / `log.md` handling, inline-link extraction. Property tests.
- [ ] P1.3 `crates/haven-git`: repo init/open, staged writes, dual-identity
      commits (human vs `Haven Agent (<model>)`), `status --porcelain` scan,
      debounced FS watcher.
- [ ] P1.4 `crates/haven-index`: SQLite (WAL, single-writer worker) with FTS5 +
      `sqlite-vec` + `edge` table; full rebuild from bundle;
      `INDEX_SCHEMA_VERSION` gate.
- [ ] P1.5 Editor shell: BlockSuite page editor bound to files, save pipeline
      → linter → git commit → incremental index update; typed IPC client.

**Exit test:** create/edit docs in the app; `git log` shows clean OKF files
with correct authors; delete `.haven/`, reindex, and search/backlinks fully
reconstruct (this becomes a permanent CI test).

## Phase 2 — Local AI

Goal: chat with retrieval over the bundle, fully offline.

- [ ] P2.1 Engine manager: Ollama detection + llama.cpp sidecar fallback,
      model pull UX, health checks. Default Gemma 4 E4B; configurable to
      12B/26B-A4B/31B.
- [ ] P2.2 Embedding pipeline: EmbeddingGemma 768-dim, model-id stamped
      vectors, batch + incremental indexing off the hot path.
- [ ] P2.3 Four-way recall: vector + BM25 + 2-hop graph CTE + temporal filter,
      fused with reciprocal rank fusion, explicit token budget.
- [ ] P2.4 Chat UI with citations to source documents; `chat_session` /
      `chat_message` persistence; 48 h open-session resumption prompt on start.

**Exit test:** airplane-mode demo — ask a question answered only by bundle
content; response cites the right documents; restart resumes the open session.

## Phase 3 — Skills + MCP (launch milestone)

Goal: the differentiated story — notes that are skills, a brain any agent can use.

- [ ] P3.1 Skill authoring: "New Skill" template, Agent Skills linter
      (mirrors `skills-ref validate`), skills browser in-app.
- [ ] P3.2 Skill execution with progressive disclosure (metadata → SKILL.md →
      references/scripts on demand) and `allowed-tools` enforcement.
- [ ] P3.3 MCP server (stdio): `search_brain`, `read_document`,
      `write_document`, `list_skills`, `read_skill`, `recall_memory`;
      read-only default, per-client write grants; all writes through
      linter + provenance pipeline.
- [ ] P3.4 MCP client: attach external servers per persona config;
      tool-permission UI.
- [ ] P3.5 "Export to Claude/Cursor" one-click for the `skills/` directory.

**Exit test:** a skill authored in Haven runs unmodified in Claude Code;
Cursor connected via MCP can search and (with grant) write a note that appears
as an agent-authored Git commit.

## Phase 4 — Ingestion + capture

- [ ] P4.1 Stage-1 deterministic importers: Notion zip (UUID strip), Tana
      JSON, Logseq/Obsidian wikilink normalization. Fixture-driven tests.
- [ ] P4.2 Stage-2 LLM normalization: `generateObject` + Zod → OKF docs;
      one commit per imported file for easy revert.
- [ ] P4.3 Browser extension via Native Messaging (no TCP port): readability
      extraction → Markdown → summarize/tag → OKF doc with `resource:` URL.
- [ ] P4.4 Optional Firecrawl engine (self-hosted URL or API key) for JS-heavy
      crawls.

**Exit test:** import a real 1k-page Notion export; zero non-conformant files;
`git revert` of one import commit cleanly removes exactly that document.

## Phase 5 — Memory depth

- [ ] P5.1 retain: post-session fact extraction to `memory/observations/`
      (`type: Observation`, confidence + evidence links), consolidation that
      updates rather than overwrites.
- [ ] P5.2 reflect: agentic loop querying observations before raw notes;
      persona documents with directives (hard rules) + disposition.
- [ ] P5.3 Proactive git-watcher ingestion prompts ("you added X outside the
      app — parse and link it?").

**Exit test:** a fact told in session 1 is recalled in session 5 with its
provenance visible as a Git diff in `memory/`.

## Phase 6 — Cloud tier (paid)

- [ ] P6.1 E2EE sync relay (AGPL, self-hostable): encrypted object store,
      device key exchange, relay never holds plaintext.
- [ ] P6.2 Companion provisioning: deploy `agents/` dir as a Vercel eve app;
      Slack/Discord/Teams via eve integrations; model calls via AI Gateway;
      folder-scoped disclosure keys with visible-scope UI and revocation.
- [ ] P6.3 Billing + entitlements (app never gates local features on network).

## Phase 7 — Team + marketplace

- [ ] P7.1 Shared bundles: Yjs live co-editing + Git async merge handlers,
      role-based folder permissions.
- [ ] P7.2 Provenance audit exports (which model, which prompt, per commit).
- [ ] P7.3 Skill & bundle registry: browse/install/publish, inspect-before-
      install, 80/20 creator revenue share.

---

## Standing risks (watch continuously)

| Risk | Mitigation |
|---|---|
| BlockSuite churn (coupled to AFFiNE roadmap) | Pin versions; OKF serializer owns the data, editor is swappable |
| OKF v0.1 spec evolution (published 2026-06) | Declare `okf_version`; permissive reader per spec §9 |
| eve immaturity (launched 2026-06) | Paid tier only; fallback = Vercel functions + Chat SDK |
| Small-model quality on ingestion | Deterministic stage 1 + schema-constrained stage 2; recommend 12B where hardware allows |
