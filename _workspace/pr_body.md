## Phase 1 — Core loop on top of R0

Re-launches Phase 1 against the R0 ADRs (`docs/adr/001`–`003`).

### Branches / lifecycle

- Branch: `cursor/phase-1-redux-d85e` (off `main` after PR #3 R0 merges).
- Stack: `[P1.2] crates/okf` → `[P1.1] Scaffold TS shell + CI` → `[P1] PR evidence`.

### Acceptance gate matrix

| Phase 1 item | Implementation | Acceptance test |
|---|---|---|
| P1.1 monorepo scaffold | `Cargo.toml`, `package.json`, `tsconfig.json`, `eslint.config.cjs`, `.github/workflows/ci.yml`, `src/src/lib/ipc.ts` typed IPC contract | `cargo fmt --check && cargo clippy -D warnings && cargo test && npm run typecheck && npm run lint -- --max-warnings 0 && npm test` (CI) |
| P1.2 `crates/okf` | frontmatter parse/serialize, `Mode::{StrictWrite, PermissiveRead}`, reserved-filename lint, unknown-key round-trip tests | `cargo test -p okf` |
| P1.3 `crates/haven-git` | dual signer (`Identity::signature` human / `Haven Agent (<model>)`), isolated staging, atomic replace + `SeenSet` expected-hash, off-tree fence, symlink/path confine, recovery snapshot under `.haven/snapshots/` | `cargo test -p haven-git` |
| P1.4 `crates/haven-index` | SQLite WAL + FTS5 + edges, `INDEX_SCHEMA_VERSION=1`, content-hash short-circuit upsert, rename/delete, full rebuild after `.haven/` deletion | `cargo test -p haven-index` |
| P1.5 typed editor shell | `EditorShell` interface + `EditorHandle` + `EditorDomain`; raw-Markdown hatch in the `EDITOR_CAPABILITIES` flag set | `npm test` |

### Local verification

- `cargo fmt --all --check` clean.
- `node scripts/okf-lint.mjs docs/fixtures/notes-200/` `OKF lint clean`.
- `cargo test`/`cargo build`/`cargo clippy` cannot run here (no MSVC link.exe / MinGW gcc on this Windows box) — CI on Ubuntu is the build/run gate (see `.github/workflows/ci.yml`).

### Cross-references back to R0

- `crates/okf` realizes the contract from `docs/adr/001-okf-adoption.md` (strict-write `okf_version`, `type`, permissive-read).
- `crates/haven-git` realizes `docs/adr/003-git-write-policy.md` (human + `Haven Agent (<model>)` identity; isolated staging; atomic replace with expected hash; off-tree fence).
- `crates/haven-index` satisfies the `AGENTS.md §1` files-only-source-of-truth invariant: deleting `.haven/` leaves canonical files intact and the next reconcile rebuilds.
- `docs/research/threat-model.md` already enumerates the relevant threats (off-tree absorption; raw conflict markers; oversized binaries; etc.).

### Evidence file

Full acceptance evidence: `_workspace/10_phase1_pr_evidence.md`.

### Reviewer checklist (superpowers style)

- [ ] `crates/okf` covers OKF v0.1 strict-write + permissive-read invariants from ADR-001.
- [ ] `crates/haven-git` integrates dual-identity, isolated staging, atomic replace, and off-tree fence from ADR-003.
- [ ] `crates/haven-index` rebuilds from canonical files after `rm -rf .haven/`.
- [ ] Editor contract in `src/src/lib/editor.ts` matches ADR-002's "lossless raw-Markdown hatch" requirement.
- [ ] IPC contract in `src/src/lib/ipc.ts` is fully typed.
- [ ] CI matrix runs fmt + clippy + tests + tsc + eslint + script-linters on PR.
