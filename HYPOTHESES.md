# Hypotheses

Nothing here is validated. These are starting beliefs from a screen-level
survey (stars, last-push date, README). Each row gets confirmed, refined, or
rejected by an evaluation under [docs/due-diligence/](docs/due-diligence/)
before it influences code.

Status legend: **HYPOTHESIS** (unaudited) · **AUDITED** (rubric run, see eval)
· **REJECTED** (eval found a blocker) · **LOCKED** (committed to in code).

## H1 — Signal backend

| Field | Value | Status |
|---|---|---|
| Client crate | `presage` ([whisperfish/presage](https://github.com/whisperfish/presage), git-only) | HYPOTHESIS |
| Store crate | `presage-store-sqlite` (same repo) | HYPOTHESIS |
| Required patches | `signalapp/curve25519-dalek` @ `signal-curve25519-4.1.3`, `whisperfish/rusqlite`, optionally a `sqlx` fork for SQLCipher | HYPOTHESIS |
| Linking model | Secondary device via QR only (no primary registration) | HYPOTHESIS |
| Manager `Send`-ness | Believed `!Send` (drove gurk-rs's LocalPool) | HYPOTHESIS |

## H2 — LLM / agent layer

| Field | Value | Status |
|---|---|---|
| Primary candidate | `rig-core` ([0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig)) — first-class Anthropic provider | HYPOTHESIS |
| Fallback for features rig hides | `anthropic-ai-sdk` ([katsuhirohonda/anthropic-sdk-rs](https://github.com/katsuhirohonda/anthropic-sdk-rs)) | HYPOTHESIS |
| No official Anthropic Rust SDK exists | Confirmed via crates.io + GitHub search 2026-05-18 | AUDITED |

## H3 — Reference implementations

| Repo | What we hope to crib | Status |
|---|---|---|
| [boxdot/gurk-rs](https://github.com/boxdot/gurk-rs) | LocalPool + 8 MiB stack, SQLCipher store, Fibonacci backoff, `SignalManager` trait | HYPOTHESIS |
| [camperking/xdb-agent](https://github.com/camperking/xdb-agent) | presage + rig wiring in one file, `QueueEmpty` gate, unbounded mpsc decoupling | HYPOTHESIS |

## H4 — Other messenger backends (deferred)

Only audit these once Signal works end-to-end.

| Platform | Candidate | Screen-level signal |
|---|---|---|
| WhatsApp | [oxidezap/whatsapp-rust](https://github.com/oxidezap/whatsapp-rust) | Pure Rust port of whatsmeow, active |
| Telegram (bots) | [teloxide/teloxide](https://github.com/teloxide/teloxide) | Bot API, idiomatic |
| Telegram (user) | [Lonami/grammers](https://github.com/Lonami/grammers) | MTProto |
| Matrix | [matrix-org/matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) | Mature, E2EE, official |
| Discord | [serenity-rs/serenity](https://github.com/serenity-rs/serenity) or [twilight-rs/twilight](https://github.com/twilight-rs/twilight) | Both alive |
| Slack | [abdolence/slack-morphism-rust](https://github.com/abdolence/slack-morphism-rust) | Web/Events/Socket Mode |
| Bluesky | [sugyan/atrium](https://github.com/sugyan/atrium) | ATProto |
| XMPP | [prose-im/prose-core-client](https://github.com/prose-im/prose-core-client) | Small but alive |
| IRC | [aatxe/irc](https://github.com/aatxe/irc) | Stable, low churn |
| iMessage | [ReagentX/imessage-exporter](https://github.com/ReagentX/imessage-exporter) | Read-only — send-side requires AppleScript shellout |
| RCS | — | No Rust option |

## H5 — Architecture shape

| Question | Current belief | Status |
|---|---|---|
| One binary or one-per-backend? | One binary (matches gurk shape) | HYPOTHESIS |
| Workspace layout | `crates/core` (trait) + `crates/backend-signal` + `bin/chat-isomorphic` | HYPOTHESIS |
| Async runtime | tokio multi-thread, custom LocalPool with 8 MiB stack for Signal | HYPOTHESIS (carried from gurk) |
| State persistence | SQLite, separate DB from any backend's internal store | HYPOTHESIS |
