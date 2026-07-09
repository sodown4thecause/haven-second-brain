# ADR-001: Tauri 2 Monorepo with Rust Crates

**Date:** 2026-07-09
**Status:** Accepted

## Context

Phase 1 requires scaffolding a Tauri 2 desktop app with three Rust crates
(`okf`, `haven-git`, `haven-index`) and a TypeScript/BlockSuite frontend.
We must decide:

1. Cargo workspace vs standalone crate
2. Frontend tooling (Vite vs Next.js)
3. How the editor binds to the file pipeline
4. `.haven/` index location and disposal guarantees

## Decision

### Cargo workspace

The repo uses a Cargo workspace at the root with four members:
`crates/okf`, `crates/haven-git`, `crates/haven-index`, `src-tauri/`.
This keeps compilation of all Rust code unified, shares dependencies, and
aligns with the monorepo convention from AGENTS.md.

### Frontend: Vite + vanilla TypeScript

BlockSuite is editor-only; there is no SSR requirement. Vite is the
lightest path to a Tauri 2 webview. A simple `src/` dir with `index.html`
and typed IPC client in `src/lib/ipc.ts` suffices. If complexity grows,
Svelte or Solid can be adopted, but the AGENTS.md mandates no framework
lock-in. The editor is swappable.

### Editor binding

The editor shell (P1.5) will use BlockSuite page editor bound to the file
system. The save pipeline is:

1. User hits save → BlockSuite serializes to Markdown
2. Frontend wraps in YAML frontmatter (OKF)
3. `create_document` Tauri command calls `okf::parser::parse()` + linter
4. `haven-git` commits the file with dual-identity (human author)
5. `haven-index` incrementally indexes the new content

### `.haven/` location

The `.haven/` directory lives at the root of the opened knowledge bundle.
It contains:

- `index.db` — SQLite database (WAL mode)
- `vectors.db` — vector store (Phase 2)
- `cache/` — transient caches

This is fully disposable: deleting `.haven/` and re-running index rebuild
restores all derived state. The CI invariant test verifies this.

## Consequences

- **Positive:** Clear separation of concerns between crates; each is
  independently testable. The disposable index simplifies schema evolution.
- **Negative:** Three crates instead of one means more Cargo.toml maintenance.
  Mitigated by workspace-level dependency versioning.
- **Risk:** `sqlite-vec` may have platform-specific build issues on Windows.
  We use the bundled SQLite feature to minimize this.
