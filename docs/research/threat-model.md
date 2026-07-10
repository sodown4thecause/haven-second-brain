# Haven Unified Threat Model (R0)

This document cross-references every `AGENTS.md` invariant (§1 — §10) to
specific threats, mitigations, and acceptance evidence. R0 records the
model; later phases fill in the implementation tests. The model is the
durable reference for any future "add this remote service or new ingest
path" review.

## Invariant mapping

| AGENTS.md invariant | Threat class | Mitigation | Acceptance |
| --- | --- | --- | --- |
| §1 Files are the only source of truth | "second source of truth" introduced silently | Bakeoffs reject DBs/services that don't rebuild from files | Per ADR row cites this map |
| §2 OKF-conformant writes | hostile whitelist or unknown-key escapes | Strict-write linter; permissive read | `crates/okf` tests |
| §3 Local-model default | silent cloud calls | URL validator + cloud preflight | `crates/haven-engine cloud_preflight.rs` |
| §4 Internet is first-class | sync queue corruption on long offline | Async queue + reconcile on reconnect | `crates/haven-sync` reconcile tests |
| §5 No inbound network ports | local TCP server | NAMED PIPE / UNIX DOMAIN SOCKET IPC between webview and engine; no desktop TCP listener ever. Webview CSP forbids direct fetch of any `127.0.0.1` URL. Loopback TCP is a documented platform-only fallback when no IPC primitive exists; it is never the default. Outbound-only connectors go through the URL gate. | `crates/haven-engine ipc_pipe_only.rs` |
| §6 License posture / no lock-in | proprietary file mutations | OKF strict-write; raw Markdown escape hatch | `crates/okf` permissivity tests |
| §7 Provenance sacred | batched human+AI commits | Dual-identity commit policy | `crates/haven-git identity.rs` tests |
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
  with prompt injection.
- **Mitigations**: import quality dashboard; one commit per imported file
  with revert; size cap per import.
- **Test**: `crates/haven-importers import_quality.rs`.

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
