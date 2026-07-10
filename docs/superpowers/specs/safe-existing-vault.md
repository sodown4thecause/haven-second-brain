# Safe Existing Vault Mode — Spec (M1)

Status: accepted (M1)
Date: 2026-07-11
Driver: `haven-m1-orchestrator`
Reviewer: `haven-architect`
Linked ADRs: `docs/adr/003-git-write-policy.md`, `docs/adr/008-editor-shell-completion.md`,
`docs/adr/009-pkm-ux-surface.md`, `docs/adr/010-bases-lite-derived-view.md`
Linked launch workflow: `docs/superpowers/specs/launch-workflows.md` Workflow A
Linked metrics: `docs/superpowers/specs/alpha-success-metrics.md` (safe-vault-open completion rate)
Linked threat-model rows: `docs/research/threat-model.md` P1 section (side-loaded-vault adversary)

## Purpose

`PLAN.md P1.7` requires Haven to open an existing Obsidian / Logseq / plain-Git
vault **without modifying a single user file** until the user performs an
explicit opt-in event. This spec is the durable contract that the
`crates/haven-git/safe.rs` skeleton, the safe-existing-vault integration test,
and the M1 UX states all bind to. It also bound the M1 PR so reviewers see one
contract per stage rather than re-litigating the surface area.

## Scope

In:

- The four user-visible trust states (`read-only`, `indexing`, `write-enabled`,
  `conflict`) carried from `launch-workflows.md:33-39`.
- The compatibility-report schema (what gets enumerated; what gets hidden
  from the user by default).
- The opt-in event semantics: how the user enters `write-enabled`, what
  durable artifact the opt-in produces, and how the OS-keychain identity binds
  to it.
- The dirty-worktree detector: how Haven enumerates unpushed commits, ignored
  files, sync artifacts, and other layout hints without mutating the working
  tree.
- The acceptance gate the integration test must prove: reopen the same
  fixture after `git status --porcelain` and assert zero diffs.

Out of scope (later milestones):

- Live notification when the user edits the file in another process (Phase 5
  sync wedge).
- Editor-surface rendering (P1.5 finishes the host binding; UI styling is a
  follow-up PR).
- Import dashboards and one-click revert instructions (Phase 3 per
  `PLAN.md P3.4`).
- Cross-device opt-in propagation (Phase 5 per `PLAN.md P5.1`).

## Compatibility report schema

The compatibility report enumerates the user-facing state of a third-party
vault on first open. Every entry follows the shape used by the integration
test's JSON snapshot under `docs/fixtures/obsidian-readonly/`:

```ts
// conceptual; the canonical struct lives in crates/haven-git/src/safe.rs.
interface CompatibilityReport {
  vaultRoot: string;            // absolute, canonicalized
  observedAt: string;           // ISO-8601 captured at indexer start
  frontmatter: {
    total: number;              // count of files starting with `---\n`
    okfConformant: number;       // files passing crates/okf StrictWrite
    nonConformant: number;       // files that permissive-read accepts but strict-write rejects
    sample: Array<{ path: string; reason: string }>;
  };
  links: {
    wikilinks: number;           // [[...]] count
    markdownLinks: number;       // [text](url) count
    brokenRefs: number;          // links whose target is absent
    sample: Array<{ source: string; target: string; kind: "wikilink" | "markdown" }>;
  };
  syntax: {
    embeddedHtml: number;        // files containing inline HTML
    codeFences: number;          // ```...``` regions
    tables: number;              // pipe-tables
    unsupportedTokens: Array<{ path: string; token: string }>;
  };
  ignoredFiles: string[];        // .gitignore + .havenignore matches
  indexCoverage: {
    files: number;               // total files under vault root
    indexed: number;             // files written to .haven/index.db
    skipped: number;             // files excluded by the indexer policy
  };
  dirtyWorktree: {
    uncommittedChanges: number;  // git status --porcelain count
    unpushedCommits: number;     // git rev-list HEAD ^@{u} count (best-effort)
    ignoredLayoutHints: Array<"obsidian" | "logseq" | "syncthing" | "icloud" | "dropbox">;
  };
  findings: Array<{ severity: "info" | "warn"; code: string; message: string }>;
}
```

The report is **a derived projection**. Deleting it does not lose information
about the vault; the vault root is the source of truth (AGENTS §1).

## Opt-in event semantics

The user is in `read-only` state from indexer start until they perform a typed
opt-in event. The event has these properties:

1. It is an **explicit** UI action, never implied by background activity. A
   Cmd-S / Ctrl-S in the editor does not count as opt-in.
2. It produces a **durable marker**: an empty file at
   `.haven/state.json` committed by the user's own identity
   (`Identity::human_name` from `crates/haven-git/src/lib.rs:46`). The
   commit is the audit log entry the founder can `git log` to see
   "When did I trust this app?".
3. Once the marker exists, the next editor save flows through the OKF
   linter and the dual-identity commit pipeline (per Workflow A step 5
   and ADR-003).
4. Removing the marker (deleting `.haven/state.json`) returns the vault
   to `read-only`. The next `write_atomic` call fails with
   `GitError::OffTree` for any path not in `OwnedAllowlist` until a new
   marker commit lands. The integration test asserts this round-trip.

The opt-in marker format itself is not part of this spec; it is captured in
`crates/haven-git/src/safe.rs` as a typed `OptInMarker { accepted_at,
accepting_identity_hash }` struct.

## Dirty-worktree detector

Before showing the compatibility report, Haven enumerates the working tree's
"dirty" state. This is the on-board step in `PLAN.md P3.5` shipped early
because the safe-mode user is exactly the user who would lose data to
silent absorption of pre-existing changes.

| Detector | Signal | Source |
|---|---|---|
| Uncommitted changes | `git status --porcelain` count | `git2` index lookup |
| Unpushed commits | `git rev-list HEAD ^@{u} --count` | `git2` revwalk; best-effort |
| Ignored files | matched against `.gitignore` + `.havenignore` | Rust `ignore` crate (already a dep, `crates/haven-git/Cargo.toml`) |
| Sync layout hints | filesystem-level hints (`iCloud Drive`, `.stfolder`, `.dropbox.cache`, `.obsidian`) | Rust `walkdir` traversal |

Detector runs are **read-only**: no writes, no `git add`, no `git commit`,
no `.haven/` mutation. The detector surfaces its findings inside the
compatibility report so the UI can present them in the same pass that
proposes opt-in.

## Acceptance gate (the integration test must prove)

The spine-PR test in `crates/haven-git/tests/safe_existing_vault.rs` opens
the canonical Obsidian-shaped fixture at
`docs/fixtures/obsidian-readonly/`, runs `safe_open`, and asserts:

1. **Zero file mutations before the opt-in event.** The test hashes every
   file under the vault root before and after `safe_open`; hashes must match
   byte-for-byte (per AGENTS §1 file-truth invariant).
2. **Compatibility report matches the snapshot.** A reference JSON file
   under `docs/fixtures/obsidian-readonly/expected-report.json` is compared
   against the report struct the test computed. Drift fails the test; the
   snapshot is the durable contract.
3. **Opt-in writes only Haven-owned paths.** After the typed opt-in event,
   a `write_atomic` to a non-allowlisted path returns `GitError::OffTree`,
   and a write to an allowlisted path succeeds and produces a dual-identity
   commit (`crates/haven-git/src/lib.rs:417-446`).

## Cross-references

This spec depends on:

- `docs/adr/003-git-write-policy.md` §Decision: identity, staging
  isolation, atomic write, conflict strategy, path confinement.
- `docs/adr/008-editor-shell-completion.md`: editor host binding for the
  `write-enabled` state's save pipeline.
- `docs/adr/009-pkm-ux-surface.md`: daily-note policy applied during
  `indexing` and editable in `write-enabled`.
- `docs/adr/010-bases-lite-derived-view.md`: derived views do not change
  the opt-in marker; deleting a saved view leaves the vault intact.
- `docs/superpowers/specs/launch-workflows.md` Workflow A: the user-visible
  trust states the spec enumerates.
- `docs/superpowers/specs/alpha-success-metrics.md` safe-vault-open
  completion rate (target ≥ 80%) and zero-destructive-rewrites invariant.
- `docs/research/threat-model.md` P1 section: side-loaded-vault adversary
  and safe-mode opt-in test.

## Open follow-ups (not in M1)

- Compatibility-report live refresh debouncing (Phase 2 or 3).
- Import-quality dashboard (Phase 3 per PLAN P3.4).
- Per-device opt-in propagation and a UI for revoking the marker
  (Phase 5).
