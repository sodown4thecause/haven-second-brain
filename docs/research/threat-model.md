# Haven Unified Threat Model (R0)

This document cross-references every `AGENTS.md` invariant (§1 — §10) to
specific threats, mitigations, and acceptance evidence. R0 records the
model; later phases fill in the implementation tests. The model is the
durable reference for any future "add this remote service or new ingest
path" review.

## Invariant mapping

| AGENTS.md invariant | Threat class | Mitigation | Acceptance |
| --- | --- | --- | --- |
| §1 Files are the only source of truth | "second source of truth" introduced silently | Bakeoffs reject DBs/services that don't rebuild from files. Bases-lite registers saved views as files (ADR-010) so removing a view leaves the vault intact. | Per ADR row cites this map. **M1**: `crates/haven-git/tests/safe_existing_vault.rs` proves zero mutations before opt-in; `src/src/tests/bases-lite.test.ts` proves round-trip-deletion of a saved view restores the pre-create file set. |
| §2 OKF-conformant writes | hostile whitelist or unknown-key escapes | Strict-write linter; permissive read. Saved views (`ADR-009` + `ADR-010`) parse `view_filter` via `crates/okf::parse`. | `crates/okf` tests. **M1**: `crates/haven-git/src/safe.rs` reports per-file frontmatter status; saved-view frontmatter round-trips through the strict-write linter path documented in `ADR-001`. |
| §3 Local-model default | silent cloud calls | URL validator + cloud preflight | `crates/haven-engine cloud_preflight.rs` |
| §4 Internet is first-class | sync queue corruption on long offline | Async queue + reconcile on reconnect | `crates/haven-sync` reconcile tests |
| §5 No inbound network ports | local TCP server | Loopback bind + CSP + URL gate. Tauri 2 host binding captured in `ADR-008` keeps the no-open-port invariant; `crates/haven-git::safe::dirty_worktree_detect` uses `git2` revwalk rather than spawning `git` so no child process opens a TCP port either. | `crates/haven-engine loopback_only.rs`. **M1**: the safe-existing-vault compatibility report and the dirty-worktree detector stay inside the writer's process. |
| §6 License posture / no lock-in | proprietary file mutations | OKF strict-write; raw Markdown escape hatch (`ADR-002`). Saved views as `notes/views/*.md` keep the file shape portable. | `crates/okf` permissivity tests. **M1**: the safe-existing-vault compatibility report enumerates file count and frontmatter shape so a migrated Notion or Evernote vault can be inspected before opt-in. |
| §7 Provenance sacred | batched human+AI commits | Dual-identity commit policy. Saved views are human-authored only in M1; the deletion tombstone carried by `BasesViewStub.delete` goes through `VaultWriteRequest::authorKind='human'`. The opt-in marker for the safe-existing-vault mode is always human-authored (`ADR-009 §Opt-in` carrier) so fake-agent "opt-ins" cannot occur. | `crates/haven-git identity.rs` tests. **M1**: `crates/haven-git/tests/safe_existing_vault.rs` opts in with the founder-default identity; the test asserts `Identity::agent_*` is never on the opt-in commit. |
| §8 Relay cannot decrypt | relay sees plaintext | Envelope encryption; rotation gates new sharing | `crates/haven-relay` two-device test |
| §9 Models get only typed tools | fetched content invokes tools | Static tool registry; selector cannot call directly | `crates/haven-research router.rs` tests |
| §10 Offline queue + no inbound port | secret leakage | OS keychain; logs redacted; telemetry opt-in | CI redaction tests |

## Adversary inventory

### Hostile note author

- **Assets at risk**: training input, prompt context, citation rank.
- **Threats**: prompt injection in note body; malicious hidden HTML
  attributes; large binary embedded as base64.
- **Mitigations**: size cap on retrieval input; sanitization before
  prompt inclusion; classifier-only use of citation rank.
- **Test**: `crates/haven-retrieval prompt_injection.rs` (M2).

### Malicious collaborator

- **Assets at risk**: shared-space confidentiality, key space.
- **Threats**: post-revocation decrypt attempts; replay of old envelopes;
  re-shared content as a side channel.
- **Mitigations**: rotation gates new sharing; replay protection;
  per-device grants; recovery key path documented.
- **Test**: `crates/haven-sync rotation_failure_blocks_sharing.rs`,
  `crates/haven-sync replay.rs`.

### Malicious MCP client

- **Assets at risk**: vault path, keyspace.
- **Threats**: path traversal; oversized payloads; rapid-fire requests.
- **Mitigations**: path confinement; size caps; rate limit per client.
- **Test**: `crates/haven-mcp path_traversal.rs` and rate limit tests.

### Compromised relay

- **Assets at risk**: ciphertext metadata; delivery latency.
- **Threats**: server impersonation; replay of routing metadata; replay of
  old envelopes.
- **Mitigations**: cryptographic envelope signing; quote verification;
  replay protection.
- **Test**: `crates/haven-relay envelope_sig.rs`, `crates/haven-relay replay.rs`.

### Poisoned scraper target

- **Assets at risk**: scape-time DNS, malicious redirects, oversized
  resources.
- **Threats**: DNS rebinding; SSRF on private IPs; robots.txt bypass;
  malicious JS in interactive pages.
- **Mitigations**: re-resolve every redirect; private/loopback/link-local
  blocklist; respect robots.txt default; sandboxed browser without
  vault/keychain access.
- **Test**: `crates/haven-research ssrf.rs`, `crates/haven-research robots.rs`.

### Poisoned model file

- **Assets at risk**: model load time, inference output.
- **Threats**: tampered weights; covert channel in chat template.
- **Mitigations**: digest verification before load; pinned chat template;
  release manifest with license + quantization + digest + context cap.
- **Test**: `crates/haven-engine model_manifest_pin.rs`.

### Side-loaded imported vault

- **Assets at risk**: index entries, prompt context.
- **Threats**: oversized import with adversarial wikilink graph; markdown
  with prompt injection; markdown with embedded `<script>` or `<iframe>`
  tokens that hijack the editor surface; a third-party vault with
  pre-existing dirty worktree that Haven might absorb on first save.
- **Mitigations**: safe-existing-vault mode opens the folder in
  `read-only` state and surfaces a compatibility report before any
  write can happen (`ADR-008`, `ADR-009`); the compatibility report
  enumerates `[frontmatter, links, syntax, ignoredFiles, indexCoverage,
  dirtyWorktree, findings]`; the dirty-worktree detector never stages
  pre-existing changes; the opt-in marker is human-authored only.
- **Tests**:
  - `crates/haven-git/tests/safe_existing_vault.rs` — opens the
    canonical Obsidian-shaped fixture under
    `docs/fixtures/obsidian-readonly/`, runs `safe::safe_open`, hashes
    every file in the vault before and after, asserts byte-equal;
    flips to `WriteEnabled` only after `write_opt_in_marker`.
  - `crates/haven-importers/import_quality.rs` (phase 3).
  - `crates/haven-git/src/safe.rs` unit tests cover `OffTree` path
    confinement for paths outside the canonicalized vault root.

### Skill author with malicious scripts

- **Assets at risk**: filesystem, keychain, network.
- **Threats**: arbitrary code execution under Skill Scripts.
- **Mitigations**: per-script `allowed-tools` declaration; sandbox; P4
  ships authoring/lint/export before arbitrary execution.
- **Test**: `crates/haven-skills script_allowlist.rs`.

## Cross-cutting threats

### Prompt injection via notes

- **Surface**: any retrieval path that includes note text.
- **Mitigation**: classify note text as **evidence**, not **instructions**;
  the chat template hardcodes the user request has precedence over
  note text.

### Secrets entering prompts

- **Surface**: note frontmatter, body, attachments.
- **Mitigation**: pre-prompt secret detection; explicit redaction list
  configurable per vault.

### Adversarial imported HTML

- **Surface**: Obsidian Web Clipper output; Notion export conversion.
- **Mitigation**: HTML sanitizer before any prompt inclusion or rendered
  display; `dangerouslySetInnerHTML` is forbidden in the editor surface.

### Compromised team bridge

- **Surface**: paid eve / Slack-Discord bridge (P3, P5, P7).
- **Mitigation**: bridge input is treated as untrusted; per-bridge user
  grant; no shared-memory ingestion without explicit memory scope.

## Why this is the durable reference

Every new ADR that introduces a remote service, a new prompt input source,
or a new desktop capability must:

1. Add a row to the invariant mapping table above.
2. Add an adversary if the new surface creates one.
3. Add a test reference for the new mitigation.

R0 records the model. M1+ expands the test rows.

## P1 (Safe vault open) expansions

The M1 milestone (Safe vault open) adds surface where the existing
adversary row "Side-loaded imported vault" already lives. The
expansions here are M1-specific and link to the new artifacts shipped
in this milestone:

### Byte-identical safe-open invariant

- **Threat**: a third-party Obsidian/Logseq vault is opened in Haven
  and Haven's "indexer" drifts the working tree in unannounced ways
  (e.g., normalize EOL, rename conflicts, write `.haven/index.db`
  beside the user's vault root).
- **Mitigation**: `crates/haven-git::safe::safe_open` reads only;
  derived indexes live under `.haven/`, never inside the user's
  vault root. The dirty-worktree detector enumerates
  `git status --porcelain` before opt-in and surfaces pending changes
  to the user.
- **Test**: `crates/haven-git/tests/safe_existing_vault.rs` covers
  four invariants (zero mutations before opt-in per AGENTS §1;
  per-field assertions on the compatibility report; opt-in
  round-trip flips the read state to `WriteEnabled`; off-tree path
  escape is refused). The test pre-hashes every file in the
  fixture, runs `safe_open`, and asserts multiset equality of the
  hashes.

### Compatibility-report schema

- **Trust boundary**: the report enumerates frontmatter, link
  inventory, syntax tokens, ignored files, index coverage, dirty
  worktree, and findings. Each field is a typed string so the UI
  cannot interpret a hostile log line as an instruction (AGENTS §9).
- **Mitigation**: hostile HTML tokens in note bodies are surfaced in
  `syntax.unsupported_tokens` and the editor shell rendering rejects
  them at the saved-Markdown parse stage. Adversarial embedded HTML
  in a saved-view's body is impossible because saved-view bodies
  render in the same editor surface.
- **Test**: the integration test asserts
  `syntax.unsupported_tokens` includes `<script` for the
  `scratch.md` fixture.

### Opt-in event forensic

- **Threat**: silently raised "the user opted in" without their
  consent; mis-attributed opt-in to an agent identity.
- **Mitigation**: `OptInMarker` carries a `sha256(human_name|human_email)`
  hash so the marker verifies the OS-keychain identity. The marker
  is human-authored in Git (per `ADR-009 §Opt-in`).
- **Test**: unit tests in `crates/haven-git/src/safe.rs` cover the
  marker round-trip and verify
  `Identity::founder_default("qwen3.5:4b")` produces the same hash
  on both ends.

### Bases-lite deletion invariant

- **Threat**: a saved view becomes an attacking surface (e.g.,
  encoded filter with `<script` in `view_filter`); or worse, deletion
  of a view mutates other notes, breaking AGENTS §1's file-truth.
- **Mitigation**: saved views live as `notes/views/<name>.md` with
  strict-write frontmatter parsing through `crates/okf`; deletion
  is a regular `VaultWriteRequest::authorKind=human` note
  removal.
- **Test**: `src/src/tests/bases-lite.test.ts` round-trip-deletion
  test creates a saved view file in a tmpdir seeded with
  `notes/`, `journal/`, `skills/`, `memory/`, `inputs/`, hashes the
  entire tree, deletes the view file via raw `fs.rmSync`, rehashes,
  and asserts deletion restored the pre-create baseline
  byte-for-byte.

### PKM-UX surface contract

- **Threat**: the daily-note command writes outside the canonical
  `journal/` folder; the wikilink autocomplete writes to
  unanticipated paths; backlinks source is per-document scan that
  reads every file twice.
- **Mitigation**: `JournalCommandStub.createIfMissing` writes a
  `VaultWriteRequest` whose `path` ends in
  `journal/YYYY-MM-DD.md` and the body begins with the OKF
  `type=note` header so strict-write accepts it; `BacklinksPanelStub`
  queries `VaultIpc.search` with the target as a literal substring
  so SQLite FTS handles the lookup correctly without regex
  escaping; `SearchPanelStub` wraps the typed IPC contract rather
  than implementing its own indexer.
- **Test**: `src/src/tests/editor-surface.test.ts` covers ten cases
  against a mock `VaultIpc`; the contract is observable by readers
  so the eventual Tauri implementation can swap in without
  re-litigating the contract.

### Tauri 2 host binding (decision only)

- **Threat**: prior mock hosts (Electron, browser PWA, multi-process
  Rust host) reintroduce the analog of "second source of truth"
  for active editor state.
- **Mitigation**: `ADR-008` records the host pick (Tauri 2) as a
  decision; no Tauri code lands in the M1 milestone. The
  acceptance evidence for the host-binding decision is deferred
  to the follow-up PR.
- **Test**: deferred to the follow-up PR; no current test row.

## Standing items reset at every M+ gate

The M1 expansions above are the captured edges. After M+ merges, the
model still applies: every new ingest path, remote service, or
scripting surface adds a row here and cites its integration test.
P2 will add rows for retrieval-path threats; P5 will add rows for
sync-relay threats.
