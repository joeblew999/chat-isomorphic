# 02 — LLM layer: rig-core vs anthropic-ai-sdk

**Verdict:** ADOPT `rig-core` for the Anthropic provider. Keep `anthropic-ai-sdk` on the DEFER shelf only for Batch / Files APIs if/when we need them.
**Date:** 2026-05-18
**Repos:**
- [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig), `rig-core` v0.37.0 on crates.io (842k downloads)
- [katsuhirohonda/anthropic-sdk-rs](https://github.com/katsuhirohonda/anthropic-sdk-rs), `anthropic-ai-sdk` v0.2.27 on crates.io (38k downloads)

## Feature coverage (Anthropic-specific)

Source: greps of each repo's `src/` for the relevant identifiers, 2026-05-18.

| Feature | rig-core | anthropic-ai-sdk |
|---|---|---|
| Messages API | ✅ | ✅ |
| Streaming (SSE) | ✅ | ✅ |
| Tool use | ✅ (`ToolUse`, `ToolDefinition`, `ToolResultContent`) | ✅ |
| Vision (images) | ✅ (`ImageSource`, `ImageFormat`) | needs verification |
| Documents (PDF) | ✅ (`DocumentSource`, `DocumentFormat`) | needs verification |
| **Prompt caching** (`cache_control`) | ✅ (`CacheControl`, `CacheTtl`, applied at system/message/tool/image/document level) | **❌** (0 grep hits across src) |
| **Extended thinking** | ✅ (`ThinkingDelta`, `ThinkingState`, `Content::Thinking` with signature) | ❌ (0 grep hits) |
| **Current 4.x model aliases** (opus-4-6/4-7, sonnet-4-6, haiku-4-5) | ✅ (constants in [`completion.rs:25-31`](https://github.com/0xPlaygrounds/rig/blob/main/crates/rig-core/src/providers/anthropic/completion.rs)) | ❌ (no `claude-sonnet-4` or `claude-opus-4` hits) |
| Batch API | needs verification (probably no) | ✅ (`message_batches.rs`) |
| Files API | needs verification (probably no) | ✅ (`files.rs`) |
| Admin API | ❌ | ✅ (`admin_client.rs`) |
| Multi-provider abstraction | ✅ (Anthropic + OpenAI + Cohere + Azure + Ollama + …) | ❌ (Anthropic-only) |

For a chat agent driven by Claude, **caching + extended thinking + current models** are the load-bearing features. rig has them; anthropic-ai-sdk doesn't.

## Operational quality

| Metric | rig-core | anthropic-ai-sdk |
|---|---|---|
| Stars | 7,315 | 17 |
| Last push | 2026-05-16 | 2026-01-11 |
| Last crate publish | 2026-05-13 (v0.37.0) | 2026-01-11 (v0.2.27) |
| Commits last 90 days | 65+ across 8+ humans + dependabot | **0** |
| Active committers (90d) | gold_silver_copper (43), Bigto Chan (4), Tim Gühnemann (4), Brad Cannon (3), isSerge (3), Fay Carsons (2), … | none |

anthropic-ai-sdk has effectively stalled — no commits in 90 days and doesn't yet support the 4.x model family or prompt caching despite Anthropic shipping both. rig moves with Anthropic's release cadence.

## API fit

rig's `rig::providers::anthropic::Client` builds via `Client::new(api_key)` and yields `client.completion_model(CLAUDE_SONNET_4_6)`. The standard `AgentBuilder<M>::new(model).preamble().tools().build()` then composes the agent — the same pattern xdb-agent uses, so we have a working reference for the wire-up.

`Content` variants include `Text`, `Image`, `Document`, `ToolUse`, `ToolResult`, `Thinking`. Streaming emits `StreamingEvent::{MessageStart, ContentBlockStart, ContentBlockDelta, MessageDelta, MessageStop}`, plus a `ThinkingDelta` content sub-event with explicit signature accumulation. Cache markers attach via `cache_control: Option<CacheControl>` fields throughout.

For chat-isomorphic this is enough. Our `Backend` trait passes inbound messages into an `Agent`, gets back a response (text or tool-use loop), sends the response back via the messenger backend. Per-conversation history persists in our own SQLite — `rig::completion::Message` is `serde::Serialize`, so storage is mechanical.

## Footguns to carry forward

1. **rig is opinionated**, not a thin SDK. It models prompts, agents, RAG, vector stores. We use the Anthropic provider directly; we don't need to adopt the agent/RAG layers wholesale.
2. **rig API surface moves**. 65 commits in 90 days = some churn. Pin to v0.37.0 in our `Cargo.toml`, bump deliberately.
3. **Batch / Files APIs absent from rig**. If we ever need either (we currently don't), we either contribute upstream or drop down to raw HTTP / anthropic-ai-sdk for those specific paths.
4. **No official Anthropic Rust SDK exists.** Should one appear, re-evaluate.

## What this changes in HYPOTHESES.md

- **H2**: HYPOTHESIS → AUDITED.
  - Primary: `rig-core` v0.37.0 — confirmed.
  - Fallback: `anthropic-ai-sdk` is **not** a viable fallback — it's behind on cache + thinking + 4.x models. Replace with "drop to raw HTTP via `reqwest` for Anthropic features rig hasn't wrapped yet" if/when needed.
