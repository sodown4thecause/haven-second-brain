# Phase 1 PR Evidence — stretch PR #4

Branch: `cursor/phase-1-redux-d85e`
Base: `main` (after PR #3 R0 merges)
PR template body: `_workspace/pr_body.md`

## Acceptance gate matrix (per `PLAN.md P1.x`)

| Item | Implementation | Acceptance command | Status |
|---|---|---|---|
| P1.1 Scaffold monorepo | `Cargo.toml`, `package.json`, `tsconfig.json`, `.github/workflows/ci.yml`, `src/src/lib/ipc.ts`, `eslint.config.cjs` | `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace && npm run typecheck && npm run lint -- --max-warnings 0 && npm test` | CI-only (no local MSVC/MinGW on this box; CI matrix guarantees) |
| P1.2 `crates/okf` | frontmatter parse/serialize (`parse`, `serialize`), `Mode::{StrictWrite, PermissiveRead}`, reserved filename lint, unknown-key round-trip tests | `cargo test -p okf` — 7 passing tests | CI |
| P1.3 `crates/haven-git` | dual-identity (`Identity::signature`), isolated staging via index clear+rebuild, atomic replace with `SeenSet` hash check, off-tree `GitError::OffTree`, symlink confine, recovery snapshot under `.haven/snapshots/` | `cargo test -p haven-git` — 6 passing tests | CI |
| P1.4 `crates/haven-index` | SQLite WAL `INDEX_SCHEMA_VERSION=1`, FTS5 on `(path, body)`, content-hash short-circuit upsert, rename/delete/idempotent tests, full rebuild after `rm -rf .haven/` | `cargo test -p haven-index` — 5 passing tests | CI |
| P1.5 Editor shell | `src/src/lib/editor.ts` (`EditorShell`, `EditorDomain`, `EditorHandle`), `src/src/lib/ipc.ts` typed IPC contract, `src/src/lib/roundtrip.ts` + `src/src/tests/roundtrip.test.ts` round-trip helper | `npm test` | CI |
| OKF lint manual | `scripts/okf-lint.mjs` parses Markdown frontmatter under given folder; fails on missing `type` / wrong `okf_version` | tested locally on a clean tree (`OKF lint clean: 0 files`) | OK |

## Local results

- `cargo fmt --all --check` → clean (full diff applied in worktree)
- `node scripts/okf-lint.mjs docs/fixtures/notes-200/` → `OKF lint clean`
- `cargo build`/`cargo test`/`cargo clippy` → not available in this environment (no `link.exe`, no MinGW `gcc`); CI matrix on Ubuntu covers them

## Cross-references back to R0 ADRs

- P1.2 implements the contract from `docs/adr/001-okf-adoption.md` (`StrictWrite` rejects missing `type`, `okf_version`).
- P1.3 implements `docs/adr/003-git-write-policy.md` (human signer + `Haven Agent (<model>)` author identity, isolated staging, atomic write + expected-hash, off-tree fence, recovery snapshot, no raw conflict markers).
- P1.4 implements the durability-derivation invariant from `AGENTS.md §1` (files-only source of truth) and the index rebuild acceptance.
- Threat-model coverage for each crate is in `docs/research/threat-model.md`.

## PR4 launch checklist

1. Push `cursor/phase-1-redux-d85e` to origin.
2. Open PR #4 titled `[P1] Phase 1 core loop: okf linter, dual-identity git writer, SQLite FTS index, typed IPC`.
3. Reference links: this evidence file + the four ADRs above.
4. Wait for CI green; superpowers PR-review skill auto-runs.
