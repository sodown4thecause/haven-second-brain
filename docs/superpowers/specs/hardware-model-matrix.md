# Haven Hardware + Model Support Matrix (R0)

The matrix that drives the first-run setup and per-tier quality choices.
Per `ADR-007`, each tier carries a pinned default chat model, embedding
model, context cap, and a baseline 16 GB budget. The full benchmark table
is published in `docs/research/hardware-benchmarks.md` once measurements are
captured; this document is the schema and tier policy.

## Tier policy and defaults

| Tier | Hardware assumption | Chat default | Embedding model | Context cap | Notes |
| --- | --- | --- | --- | --- | --- |
| Floor | 8 GB RAM, CPU/iGPU | none (retrieval-only valid) | nomic-embed-text | 4k | Search-first; retrieval-only is OK |
| Default | 16 GB RAM, CPU or ≤8 GB VRAM | qwen3.5:4b Q4 | qwen3-embedding:0.6b or nomic-embed-text | 8k | 16 GB median laptop target |
| Quality | 16–32 GB RAM + ≥8 GB VRAM / Apple Silicon 18+ GB | gemma4:e4b Q4 or qwen3.5:9b Q4 | qwen3-embedding:0.6b or qwen3-embedding:4b | 16k | Gemma E4B Q4 weights ~4.5 GB before KV/runtime |
| Headroom | ≥16 GB VRAM / 32 GB+ unified | gemma4:12b Q4 (only after benchmark) | same as Quality | 32k | Never the download default |

## 16 GB runtime budget

Optimize for **sequential** local work, not resident pile:

- Qwen 3.5 4B Q4 runs at 8k default context. Queued generation jobs.
- Whisper.cpp models unload after transcription: ~388 MB `base`, ~852 MB
  `small` (per upstream).
- Embedding batches pause during interactive chat and recording.
- Microservice-style "always-resident" models are forbidden.

Release gates measure peak working-set and swap activity on:

- Windows 16 GB.
- Apple Silicon 16/18 GB.
- Linux 16 GB.

## First-run setup

- Benchmark the device before any pull. The benchmark screen shows the
  measured first-token latency (cold), prompt throughput, and peak RSS.
- The user picks a tier; the default recommendation is "Default" on 16 GB
  laptops.
- Search, editor, and MCP read work without a model being installed.

## Fallbacks (in order)

1. Smaller local model from the same tier table.
2. User-configured cloud provider (explicit opt-in, per `ADR-004`).
3. Retrieval-only answer with extractive snippets.

## Honesty bar

Small local models fail multi-step agent loops. Skill and MCP tool use must
assume failure: validate schemas, cap loops, show diffs before applying
writes, and keep retrieval-only answers available. The chat surface must
never pretend a small model is reliable for plan-execute tasks.

## Cross-references

- Per-tier benchmark numbers: `docs/research/hardware-benchmarks.md` (filled
  in during M1/M2).
- Engine binding + network posture: [ADR-004](../adr/004-local-runtime-and-network-posture.md).
- Local model selection rationale: [ADR-007](../adr/007-local-model-selection.md).
