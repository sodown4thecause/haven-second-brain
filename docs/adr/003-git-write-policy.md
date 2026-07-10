# ADR-003: Git write policy — human checkpoint commits, agent atomic commits, off-tree preserved

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-architect`
Reviewer: orchestrator review-board
Linked design: `PLAN.md P0.3` and `AGENTS.md §1.7`

## Context

Haven's durability layer is Git. Every Haven write must land with the
correct identity, scope, and timing, and must never absorb the user's
unrelated staged changes. We need a policy that:

1. Never commits on every keystroke.
2. Lets human edits commit on save or at session checkpoints, with optional
   squash of noisy checkpoints.
3. Lets agent / import / memory / collaboration writes commit atomically
   and separately from human edits.
4. Never inserts raw Git conflict markers that break the editor parser.
5. Never writes outside the explicit Haven-owned paths.
6. Survives crashes mid-write via a pre-write recovery snapshot and an
   expected-content hash on replace.

## Decision

**Identity:**

- Human edits commit as `Haven Founder <founder@haven.local>` or whatever
  the user has set in OS-keychain identity.
- Agent-author edits commit as `Haven Agent (<model>) <agent@haven.local>`.
- Batched human+AI changes in a single commit are forbidden.

**Commit cadence:**

- Human edits: on explicit save, with an optional debounce (default 60 s) and
  session-checkpoint squash on app close.
- Agent / import / memory writes: each is one atomic commit, after the user
  approval event.

**Staging isolation:**

- Stage only `docs/*.md`, `agents/notes/*.md`, or whichever paths Haven
  owns. Use `git2`'s index API with `update_index_from_tree` on a path
  whitelist, or the equivalent exact-path API.
- Never read or rewrite the user's existing index entries.

**Atomic write:**

- Compute SHA-256 of the current canonical content for a path before
  stage. After successful atomic replace (write-temp + rename), verify the
  expected-content hash matches the post-write hash before staging.
- Pre-write: take a recovery snapshot under `.haven/snapshots/` for any file
  Haven has never seen.

**Conflict strategy:**

- On `git pull` style reconciliation with divergent edits, write
  `path.conflict-{device-name}-{timestamp}.md` side-by-side with original.
- Push the divergent pair to the conflict inbox in the UI; never insert raw
  conflict markers.

**Path confinement:**

- Canonicalize and confine all writes to the vault root with `realpath`.
- Symlinks that escape the vault are rejected at startup.

## Alternatives considered

- **Commit on every keystroke**: rejected; violates GRIP noise and slows
  watch reconciliation.
- **Auto-merge divergent edits**: rejected; violates file-truth invariant.
- **Insert raw conflict markers**: rejected; the parser would fail and
  the user loses both versions.

## Consequences

- Implementation lives in `crates/haven-git`. It owns a single writer
  thread; the UI thread talks to it over a typed channel.
- The activity log is a markdown sidebar append-only narrative.
- The conflict inbox is one route in the UI; users resolve every side-by-side
  file before it disappears.

## Reversibility

- Mode changes are a config flip; switching to a CRDT is a much larger
  invariant change and not contemplated here.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §4 (sync)`: Syncthing's
  conflict-side-by-side semantics inform our off-tree write boundary.
- Git itself is the source of truth; we do not invent a side ledger.

## Threat-model cross-reference

- Off-tree writes never propagate unintended changes into the agent's
  commit.
- Symlink/path confinement shrinks the attack surface.

## Acceptance evidence

- `crates/haven-git` integration test `safe_existing_vault.rs` opens a real
  Obsidian fixture, runs the compatibility report, asserts zero file
  mutations before opt-in.
- `crates/haven-git` test `expected_hash_replace.rs` proves atomic replace
  fails when the pre-write hash does not match — no silent overwrites.
- `crates/haven-git` test `symlink_confinement.rs` rejects symlinks that
  resolve outside the vault.
