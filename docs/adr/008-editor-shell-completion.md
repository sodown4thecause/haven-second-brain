# ADR-008: Editor shell completion — Tauri 2 host binds CodeMirror 6 with a raw Markdown escape hatch

Status: accepted (M1)
Date: 2026-07-11
Driver: `haven-m1-orchestrator`
Reviewer: `haven-architect`
Supersedes: `PLAN.md P1.5` stub status (commit message of 00c2b71)
Linked ADR: `docs/adr/002-editor-roundtrip.md`, `docs/adr/003-git-write-policy.md`
Linked launch workflow: `docs/superpowers/specs/launch-workflows.md` Workflow A step 5
Linked spec: `docs/superpowers/specs/safe-existing-vault.md`

## Context

`ADR-002` chose CodeMirror 6 as the editor default with a raw Markdown
escape hatch; `PLAN.md P1.5` requires the editor shell to be **bound to
files** through the OKF linter, Git dual-identity commit, and incremental
index update. The current branch ships
`src/src/lib/{editor.ts, ipc.ts, roundtrip.ts}` and a vitest roundtrip test
but no host binding — i.e. the editor surface is a contract without a
process. Without a host, a save flow cannot run the linter or land a
commit.

This ADR picks the host and restates the typed IPC contract. Code changes
land in a follow-up PR; this ADR is decision-only ship now so subsequent
PRs do not re-litigate the host choice.

## Decision

**Host:** Tauri 2.x (Rust core + TypeScript webview).

**Editor binding:** CodeMirror 6 (`@codemirror/lang-markdown`,
`@codemirror/state`, `@codemirror/view`) plus a typed `EditorShell` interface
in `src/src/lib/editor.ts`. `EditorCapabilities` declares whether the
active document round-trips losslessly; documents that fail the round-trip
test in `tests/roundtrip.test.ts` are pinned to the raw Markdown escape
hatch automatically.

**IPC transport:** typed `VaultIpc` channel defined in `src/src/lib/ipc.ts`
(already on disk). Save flow: `EditorShell -> VaultIpc.vault_write ->
linter (crates/okf) -> writer (crates/haven-git) -> indexer
incremental (crates/haven-index) -> commit`. Each stage returns a typed
`Receipt` so the UI can surface stage failures without burying them in a
toast.

**Lifecycle:** a single writer thread in `crates/haven-git` (already
present via `VaultWriter`). The webview is purely read-only against the
filesystem; every durable write is the writer's responsibility.

**Editor surface fallbacks (out of scope for this ADR, captured for the
follow-up PR):**

- Block/Milkdown are explicitly deferred per `ADR-002`. They are
  re-evaluated only when CodeMirror 6 fails lossless round-trip on a
  fixture.
- The raw Markdown hatch renders as a `<textarea>` until the round-trip
  test passes for the target document; once it passes, the rich surface
  replaces the textarea on save.

## Alternatives considered

- **Browser-only PWA without a Tauri host**: rejected. Without a host, the
  filesystem write path depends on File System Access API and exposes the
  user to silent writes the OKF linter can't reject. Tauri gives us a
  single writer on the Rust side, the existing `crates/haven-git`
  invariants apply unchanged.
- **Electron**: rejected. Mass dist weight (≥ 80 MB), and the dual-process
  shape re-implements what Tauri 2 already provides with the system
  webview.
- **Server-side rendering the editor in the webview alone (no native
  writer)**: rejected. The active editor is a privileged user-agent; we
  do not run the dual-identity pipeline in the DOM.
- **CodeMirror 6 only, no escape hatch**: rejected. `ADR-002` already
  accepted the hatch.

## Consequences

- The follow-up PR (`src-tauri/` scaffolding) lands `tauri.conf.json` and
  a Rust `src-tauri/src/main.rs` that wires `EditorShell` to
  `VaultIpc.vault_write`. Tauri version pins to: `tauri = "2.0"`,
  `tauri-plugin-fs = "2.0"`, `tauri-plugin-shell = "2.0"`.
- The CLI alternative mentioned in `PLAN.md P7.2` (`cargo run -p
  haven-cli --`) continues to use the same `crates/haven-git` writer
  without a Tauri host binding. The CLI is the lower-trust surface for
  automation; the Tauri host is for human editing flows.
- `src/src/lib/roundtrip.ts` is the gate: a document passing the lossless
  round-trip test gets the rich surface; a failing document is pinned to
  the hatch until the user requests a migration.

## Reversibility

- Switching from Tauri 2 to Tauri 3 (when stable) is a `Cargo.toml`
  bump + window-builder API change; the writer pipeline below the host
  does not move.
- Switching from CodeMirror 6 to BlockSuite/Milkdown requires the
  `tests/roundtrip.test.ts` fixture to pass for the new implementation;
  the existing safe-existing-vault integration test is unrelated and
  stays green.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §1 (lossless editor and property
  UX)`: CodeMirror 6, Milkdown, and SilverBullet are reviewed. This ADR
  inherits CodeMirror 6's review from `ADR-002`.

## Threat-model cross-reference

- `docs/research/threat-model.md` P1 section (added in commit 11 of this
  PR): the editor host holds the same off-tree fence as the writer
  (subprocess boundary is a defense-in-depth layer).
- AGENTS §5 invariants (no inbound network ports) are preserved because
  the Tauri host binds to loopback only and the webview never opens a
  TCP port.

## Acceptance evidence (the gated follow-up PR proves these)

- `cargo test -p haven-git --test safe_existing_vault` green (already in
  PR).
- `npm run typecheck && npm run lint -- --max-warnings 0 && npm test`
  green.
- `src/src/tests/roundtrip.test.ts` proves lossless round-trip for the
  200-note fixture.
- The follow-up PR's `cargo test --workspace` must remain green; this PR
  does not yet ship `src-tauri/`.

## Out of scope (deliberately deferred)

- Tauri host scaffolding (`src-tauri/`). The decision to use Tauri is
  captured here so the follow-up PR does not re-litigate the host choice,
  but no Tauri code lands in this PR.
- BlockSuite/Milkdown evaluation. Held until CodeMirror 6 round-trip
  fails on a non-fixable document.
- Window-builder UX (zoom levels, OS-keychain wiring, multi-window).
