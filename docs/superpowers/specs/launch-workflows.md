# Haven Launch Workflows (R0)

Three launch workflows demonstrate that the alpha promise holds: open an
existing Markdown/Git vault safely, get a cited answer with no extra apps,
and let an approved agent propose durable edits via MCP the human can
approve. Each workflow defines user-visible trust states and measurable
acceptance criteria.

These workflows are the same `PLAN.md P0.1` three workflows updated for the
R0 cleaner invariants and the harness team topology.

## Workflow A — Safe existing-vault open

User want: "I have years of notes in Obsidian. I want to try Haven without
losing anything."

Sequence:

1. **User selects a folder.** Haven shows the compatibility report:
   frontmatter status, link inventory, unsupported syntax, ignored files,
   index coverage. Status: **read-only** by default; nothing is written.
2. **Index runs in read-only mode.** Status: **indexing** — the SQLite
   derived index runs against `.haven/index.db`, but the vault root is
   never modified.
3. **User opens the first note.** Status: **read-only**.
4. **User opts into write.** Status: **explicit opt-in**. A separate
   confirmation is required; the same compatibility report surfaces any
   constraints Haven will keep (e.g., wikilinks must remain unwrapped
   until migration).
5. **User saves an edit.** Status: **writing via OKF linter + dual-identity
   Git pipeline**. The `git log` shows a human-authored commit.

User-visible trust states:

- "**Read-only** — Haven will not modify your files."
- "**Indexing** — Haven is scanning your notes; your files are unchanged."
- "**Write enabled** — your edits go through Haven's pipeline."
- "**Conflict** — Haven found divergent edits and saved both versions
  side-by-side."

Acceptance criteria:

- Median **time-to-first-compatible-read** under 5 minutes for a real 1k-page
  vault; never more than 30 minutes for any 10k-note fixture.
- Zero file mutations before opt-in; verified by the
  `crates/haven-git/tests/safe_existing_vault.rs` integration test (added
  in M1 — see [ADR-003-git-write-policy.md §Acceptance evidence](../../adr/003-git-write-policy.md)).
- 100% of `git status --porcelain` diffs are explained in the UI.

## Workflow B — Cited recall with retrieval

User want: "What did I decide about X last quarter?"

Sequence:

1. **User asks a question in the chat pane.** Status: **idle**.
2. **Retrieval runs (BM25 + vector via RRF).** Status: **retrieving**.
3. **Generation (local model) produces an answer with citations.** Status:
   **drafting**.
4. **User clicks a citation.** Status: **opens the source note at the cited
   line**; the chat pane shows the snippet.
5. **Optionally save the answer.** Status: **durable transcript** — lives as
   a local file, not as an ephemeral SQLite row.

User-visible trust states:

- "**Cited** — this answer is grounded in your notes; here are the sources."
- "**No web used**" / "**Web used**" — provenance disclosure toggle.
- "**Engine unavailable**" — chat still shows a retrieval-only answer with
  snippets.

Acceptance criteria:

- Retrieval eval harness returns the cited source for at least the founder's
  tested questions.
- Every answer has at least one citation; UI enforces citation-click before
  answer copy.
- Airplane-mode demo: same answers in airplane mode as online.

## Workflow C — Approved agent patch via MCP

User want: "I asked Cursor to add a reference to my notes. How does that
land?"

Sequence:

1. **User installs the Haven MCP server.** Status: **read-only MCP** —
   the server exposes `search_brain` and `read_document` only.
2. **Cursor asks Haven for a search.** Status: **tool call received** —
   rate-limited, sanitized.
3. **Cursor drafts a patch using the search results.** Status: **patch in
   client**.
4. **Cursor calls `propose_document_patch`.** Status: **proposal local**.
   No write yet.
5. **User opens Haven and reviews the proposal as a diff.** Status:
   **awaiting human approval**.
6. **User approves.** Status: **writing via OKF linter + agent-author Git
   commit**. The commit shows `Haven Agent (<model>)` as author.

User-visible trust states:

- "**Read MCP** active."
- "**Patch proposed — review the diff**."
- "**Agent-authored commit landed** — see provenance in `git log`."
- "**Grant expires**" — per-client grants are revocable.

Acceptance criteria:

- Every MCP-mediated write produces a previewed diff before any commit.
- 100% of `git log` from MCP writes shows the agent-author identity.
- Per-client grants are persisted, revocable, and visible in the UI.
- Path traversal and oversized payloads are refused at the boundary.
