# Hypotheses

Nothing here is validated. These are starting beliefs from a screen-level
survey (stars, last-push date, README). Each row gets confirmed, refined, or
rejected by an evaluation under [docs/due-diligence/](docs/due-diligence/)
before it influences code.

Status legend: **HYPOTHESIS** (unaudited) · **AUDITED** (rubric run, see eval)
· **REJECTED** (eval found a blocker) · **LOCKED** (committed to in code).

## H1 — Signal backend  *(AUDITED — see [01-signal-presage.md](docs/due-diligence/01-signal-presage.md) + live verification [01a](docs/due-diligence/01a-presage-live-verification.md))*

| Field | Value | Status |
|---|---|---|
| Client crate | `presage` ([whisperfish/presage](https://github.com/whisperfish/presage) @ `6793c3e8`, git-only) | AUDITED |
| Store crate | `presage-store-sqlite` (same repo) — sled store is gone | AUDITED |
| Required patches | `signalapp/curve25519-dalek` @ `signal-curve25519-4.1.3`; `whisperfish/rusqlite` for `libsqlite3-sys` (only if `cdsi` feature) | AUDITED |
| Build tool prerequisite | **`protoc` required** — `spqr` (post-quantum ratchet) needs it. `brew install protobuf` on macOS | AUDITED |
| QR provisioning window | ~60-90 s — render QR PNG and surface to user *before* awaiting the linking future | AUDITED |
| `receive_messages` semantics | Stream emits `Received::QueueEmpty` cleanly; live-verified on this machine | AUDITED |
| `sync-contacts` (await `Received::Contacts`) | Hangs — iOS primary may not deliver this variant to a fresh secondary. Treat as best-effort, do not block on it | AUDITED |
| Pinned transitive | `libsignal-service` is git-pinned in presage's own `Cargo.toml` — bumping presage = bumping libsignal-service | AUDITED |
| Linking model | Both supported: phone-number register (SMS/voice) **and** secondary-device link. `presage-cli register` currently broken on websocket upgrade — secondary-device link is the safer onboarding path until that's fixed. | AUDITED |
| Manager `Send`-ness | **Confirmed `!Send`** — requires `LocalSet` containment | AUDITED |
| Open bugs to design around | Decryption-loop session-archive bug; PNI cipher first-contact decrypt failure; `presage-cli register` websocket failure. See eval doc. | AUDITED |

## H2 — LLM / agent layer  *(AUDITED — see [02-llm-layer.md](docs/due-diligence/02-llm-layer.md))*

| Field | Value | Status |
|---|---|---|
| Primary | `rig-core` v0.37.0 — Anthropic provider has prompt caching, extended thinking, tools, vision, documents, streaming, current 4.x models | AUDITED |
| Fallback for features rig misses | Drop to raw HTTP via `reqwest`. **`anthropic-ai-sdk` is not viable** — 0 commits in 90 days, no cache/thinking support, no 4.x models. | AUDITED |
| No official Anthropic Rust SDK exists | Confirmed via crates.io + GitHub search 2026-05-18 | AUDITED |
| Initial model | `CLAUDE_SONNET_4_6` constant from rig (matches our usual default) | AUDITED |

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
