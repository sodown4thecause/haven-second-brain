---
name: haven-threat-model
description: Unified threat model specialist. Use when the R0 orchestrator asks for `docs/research/threat-model.md` or when a new dependency, ingest path, or remote service needs to be cross-referenced against it. Covers hostile note content, prompt injection, oversized/binary inputs, secret exfiltration, optional cloud egress, malicious MCP clients, compromised relay responses, hostile scraped pages, and skill script execution. Always read-only — does not write code.
---

# Threat Model

## When to use

Use when the orchestrator asks for the unified R0 threat model. Also use
when:

- A new ADRs wants to add a remote service, a new prompt input source, or a
  new desktop capability.
- A new ingest path (browser extension, importer, MCP client) is proposed.
- A new skill category that may execute scripts is proposed.

Do not use for general security review outside R0. The threat model is the
single durable reference; later threats append to it.

## Required inputs

1. Every approved product invariant in `AGENTS.md §1`.
2. Every ADR under `docs/adr/`.
3. The approved design's threat-related sections (network and content safety,
   unified context, the prior-art bakeoff rules).

## Adversary inventory

Adversaries, each with a STRIDE-style table:

- Hostile note author.
- Malicious collaborator (after key rotation, before revocation, etc.).
- Malicious MCP client.
- Compromised relay.
- Poisoned scraper target (SSRF / DNS rebinding).
- Poisoned model file.
- Side-loaded imported vault.
- Skill author with malicious `scripts/`.

Each entry rows: asset at risk, threat, mitigation, test reference.

## Output shape

`docs/research/threat-model.md` with a table per surface and a top-level
matrix that maps each `AGENTS.md §1` invariant to one or more specific
threats and mitigations.
