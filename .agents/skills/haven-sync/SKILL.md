---
name: haven-sync
description: Sync / E2EE collaboration bakeoff specialist. Use when the R0 orchestrator asks for an E2EE sync + collaboration bakeoff and ADR. Compares any-sync, Syncthing, Automerge, Loro, a minimal encrypted Git-envelope relay, Seafile, and git-remote-gcrypt. Records license, metadata leakage, key handling, conflict semantics, mobile fit, deployment complexity, recovery, performance, and "does this create a second source of truth" verdict. Read-only — does not write code.
---

# Sync / E2EE Collaboration Bakeoff

## When to use

Use when the orchestrator calls for the R0 sync bakeoff or for an ADR
revision prompted by a new collaborator / conflict / rotation requirement.

Do not use to design or implement E2EE code. R0 is decision-only.

## Required inputs

1. The approved design, especially §E2EE Sync and Collaboration and the
   Prior-art Bakeoff list.
2. `docs/research/prior-art-register.md` for license verdicts.
3. `AGENTS.md` invariants 8 (relay cannot decrypt), 4 (no inbound ports), and
   1 (files are the only source of truth).

## Bakeoff dimensions

Each candidate is scored on:

- **License:** Apache-2.0/MIT vs GPL/AGPL/legal review needed.
- **Metadata leakage:** what the server can infer from envelopes vs filenames
  vs IPs.
- **Key handling:** identity keys, space keys, rotation, revocation, recovery.
- **Conflict semantics:** async Git-compatible vs CRDT vs side-store inbox.
- **Mobile fit:** smallest sensible mobile library / footprint.
- **Deployment complexity:** docker, native, server requirements.
- **Recovery:** server compromise, lost device, rotated key impact.
- **Performance:** envelope size, sync latency, bandwidth.
- **Second source of truth:** does this candidate create a parallel state
  we cannot rebuild from canonical Markdown + Git?

## Output shape

`_workspace/03a_sync_bakeoff.md` followed by an ADR sketch (the
`haven-architect` reviews and writes the final ADR).

The ADR must record the chosen candidate and the evidence-based reasons.
If the choice is "build a native relay", the ADR must also explain why no
existing candidate satisfies the invariants.
