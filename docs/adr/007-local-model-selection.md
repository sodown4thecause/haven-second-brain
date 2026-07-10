# ADR-007: Local model selection — Qwen 3.5 4B Q4 default, Gemma E4B Q4 quality alternate, EmbeddingGemma / Qwen3-Embedding fallback matrix

Status: accepted (R0)
Date: 2026-07-10
Driver: `haven-localmodel`
Reviewer: `haven-architect`
Linked design: `PLAN.md Recommended local LLM stack` and `AGENTS.md §1.3, §7`

## Context

Haven runs chat and embeddings on a local model. The 16 GB laptop is the
median target. We need a default model and an embedding model with
documented fallback paths so search, editor, and MCP read remain usable on
smaller hardware for users who cannot or will not download a multi-GB
artifact.

We compared the candidates listed in the `PLAN.md` table:

- Qwen 3.5 4B Q4 — Apache-2.0; multimodal; designed for tool use.
- Gemma 4 E4B Q4 — Google documents weights around ~4.5 GB pre-runtime.
- Qwen 3.5 2B Q4 — smaller Qwen variant.
- EmbeddingGemma — Google embedder.
- Qwen3-Embedding 0.6B — small embedder.
- nomic-embed-text — smallest, widely used fallback.

## Decision

**Chat default (16 GB median target):** `qwen3.5:4b` Q4.

- Apache-2.0 license friendly.
- Supports text, vision, and tool use; future-proofs the chat surface.
- Casts acceptable on a 16 GB Windows/macOS/Linux laptop with 8k context.

**Embedding default:** `qwen3-embedding:0.6b` when the hardware benchmark
passes; fall back to `nomic-embed-text` on low-memory devices or when
fastest-setup wins.

**Quality alternate:** `gemma4:e4b` Q4 or `qwen3.5:9b` Q4 on machines with
16+ GB VRAM or Apple Silicon 18+ GB. Headroom tier: `gemma4:12b` Q4 only
on explicitly headroom hardware.

**Floor tier (8 GB):** no default pull. Optional `gemma4:e2b` or verified
Qwen 3.5 2B Q4 — retrieval-only answers are valid.

**Pin each default model by:** digest, quantization, chat template,
license, context cap, and a baseline benchmark number.

**Honesty bar:** small local models fail multi-step agent loops. The agent
loop validates schemas, caps loops, shows diffs before writes, and keeps
retrieval-only answers available.

## Alternatives considered

- **Universal default pull of one large chat model**: rejected; would
  push the 8 GB tier out of the product.
- **Gemma 3 27B / larger**: deferred; release gates use measured peak
  working-set and swap activity on Windows 16 GB / Apple Silicon 16/18 GB
  / Linux 16 GB reference machines.

## Consequences

- `crates/haven-engine` exposes per-tier selection driven by the hardware
  matrix (`docs/superpowers/specs/hardware-model-matrix.md`).
- Release manifest carries: digest, quantization, chat template, license,
  context cap, baseline benchmark number. A mutable Ollama tag is never the
  only provenance.
- First-run setup benchmarks the device, explains the tradeoff, and lets
  the user choose; the founder workflow defaults are documented so the
  technical ICP has a recommended path.

## Reversibility

- Tier policies are config; switching the default chat model for 16 GB
  is a one-line release manifest change.
- A future llama.cpp sidecar path replaces Ollama via the engine adapter
  (`crates/haven-engine/src/runtime.rs`), without changing call sites.

## Prior-art cross-reference

- `docs/research/prior-art-register.md §6, §9`.
- `docs/superpowers/specs/hardware-model-matrix.md` carries the per-tier
  measurement table.

## Threat-model cross-reference

- Model files are verified by digest before load (defends against poisoned
  pulls).
- Cloud-provider fallback is opt-in only and shows a per-provider
  disclosure.

## Acceptance evidence

- `crates/haven-engine` test `model_manifest_pin.rs` confirms the runtime
  refuses to load a model whose digest does not match the pinned manifest.
- `crates/haven-engine` test `embedding_stamp.rs` confirms every embedded
  vector row stores model id + dimensions + chunking policy.
- `docs/research/hardware-benchmarks.md` records the first-token, prompt
  throughput, and peak working-set numbers per tier.
