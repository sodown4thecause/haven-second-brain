# ADR-005: Sync and collaboration — encrypted Git-envelope relay over any-sync-influenced spaces

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-sync`
Reviewer: `haven-architect`
Linked design: `docs/superpowers/specs/2026-07-10-haven-clean-rebuild-design.md §E2EE Sync and Collaboration`

## Context

We need multi-device sync and asynchronous collaboration for personal vaults
(P5) and E2EE shared spaces (P7). The design's bakeoff listed:

- any-sync (Apache-2.0, full sync stack with spaces, permissions, history,
  self-host).
- Syncthing (MPL-2.0, mature file sync with strong encryption).
- Automerge / Loro (live CRDTs; reference only — live co-editing is a
  non-goal in v1).
- git-remote-gcrypt (AGPL-3.0; protocol prior art only).
- A minimal encrypted Git-envelope relay (native implementation).
- Seafile (AGPL-3.0; interoperability only).

Product invariants:

- Files are the only source of truth (`AGENTS.md §1.1`).
- Relay cannot decrypt user content, filenames, memories, comments, or
  attachments (`AGENTS.md §1.8`).
- No inbound network ports (`AGENTS.md §1.5`).

## Decision

Adopt a **minimal encrypted Git-envelope relay** with **Syncthing-influenced
conflict semantics** and **any-sync-style spaces**: we ship a native Rust
relay under `crates/haven-relay/` for opaque-object delivery, and we use
`crates/haven-sync/` on the client to encrypt envelopes before upload.

Why not adopt any-sync directly: any-sync is an excellent reference, but its
native component graph and CRDT-flavored object store go beyond the file /
Git truth model we want to keep canonical. Adapting it would force us to
maintain a custom object graph; a thinner relay focused on Git-compatible
envelopes stays inside our invariants and still imports any-sync's permission
and key-management semantics.

Why not Syncthing: Syncthing is canonical file synchronization, but our
envelope shape is also a Git commit + a typed object metadata payload. We
borrow the conflict-side-by-side semantics only.

## Components

- **Client (`crates/haven-sync`)**:
  - Device identity key in OS keychain.
  - Per-vault or per-space content key.
  - Invitations grant the space key to a device identity.
  - Revocation rotates future keys so revoked devices cannot decrypt
    new content.
  - Recovery key (user-controlled, never server-known) lets a user regain
    key access without a server backdoor.
- **Relay (`crates/haven-relay`)**:
  - Holds ciphertext + minimal routing metadata + opaque envelopes.
  - Cannot decrypt, recover keys, or grant itself content access.
  - Two-device and three-device integration tests fail when the relay
    misuses any plaintext field.
- **Reconciliation**:
  - Async Git-compatible reconciliation — async first, CRDT only as a
    fallback.
  - Clean changes produce human or agent Git commits.
  - Divergent changes preserve every version in the conflict inbox.

## Key model

- Each device has an identity key stored in the OS keychain (per
  `AGENTS.md §10`).
- Each shared space has a content-encryption key.
- Invitations grant the space key to authorized device identities.
- Revocation rotates future keys. **Failed rotation blocks new sharing
  rather than falling back to plaintext.**
- User-controlled recovery key (offline, vault-printed, verbatim in
  per-device-keychain recovery document) is the deterministic backdoor
  we will not have on a server.

## Alternatives considered

- **Adopt any-sync wholesale**: rejected; CRDT/object-graph baggage.
- **Adopt Syncthing wholesale**: rejected; envelope shape is Git object,
  not raw file.
- **git-remote-gcrypt**: rejected as a primary dependency; useful protocol
  prior art only.

## Consequences

- Implementation is split: `crates/haven-sync` (client), `crates/haven-relay`
  (AGPL-3.0 server; self-hostable).
- Mobile capture companion (`P5.2`) targets the same protocol surface.
- Billing and entitlements (`P5.3`) live outside sync but share the device
  identity.

## Reversibility

- The relay is intentionally minimal. Replacement with a federated variant
  (e.g., a complete any-sync DAG after a v1 learning phase) is bounded to
  the relay and the client edge adapters.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §4`.
- `threat-model.md` for key compromise and revocation paths.

## Acceptance evidence

- Two-device and three-device integration tests prove end-to-end encrypt,
  decrypt, and reconcile without the relay seeing plaintext.
- Rotation failure blocks new sharing — exercised in the rotation test
  harness.
- Delete derived index, memory, and research-cache state, then rebuild
  from canonical files passes.
