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
| `sync-contacts` (await `Received::Contacts`) | **iOS primary does not push contacts back to a fresh presage secondary.** Repeated 2026-05-18: request goes out fine, no Contacts variant ever arrives. Same gap, opposite direction, as upstream presage issue [#328](https://github.com/whisperfish/presage/issues/328). Workaround: derive contacts from received-message traffic (deferred). | AUDITED |
| Pinned transitive | `libsignal-service` is git-pinned in presage's own `Cargo.toml` — bumping presage = bumping libsignal-service | AUDITED |
| Linking model | Both supported: phone-number register (SMS/voice) **and** secondary-device link. `presage-cli register` currently broken on websocket upgrade — secondary-device link is the safer onboarding path until that's fixed. | AUDITED |
| Manager `Send`-ness | **Confirmed `!Send`** — requires `LocalSet` containment | AUDITED |
| Open bugs to design around | Decryption-loop session-archive bug; PNI cipher first-contact decrypt failure; `presage-cli register` websocket failure. See eval doc. | AUDITED |

## H2 — LLM / agent layer  *(DEFERRED — see note below)*

**Reframed 2026-05-18:** chat-isomorphic is an MCP server, so the LLM lives
in the *client* (Claude Desktop, Claude Code, custom agents) — not embedded
in this binary. The audited rig-core / Anthropic findings in
[02-llm-layer.md](docs/due-diligence/02-llm-layer.md) stand as research; they
just no longer apply to this repo. They become relevant again only if we
ever ship an embedded-LLM mode (e.g. for autonomous bot scenarios). Until
then this row is parked.

| Field | Value | Status |
|---|---|---|
| LLM SDK inside chat-isomorphic | Not needed — MCP client owns the LLM | DEFERRED |
| If we ever re-embed | rig-core v0.37.0 (Anthropic provider) is still the right pick per the eval | DEFERRED |

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
| Matrix | [matrix-org/matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) | **AUDITED — see [03-matrix.md](docs/due-diligence/03-matrix.md). ADOPT.** crate `matrix-sdk` v0.17.0, 2.1k★, Element-sponsored, no patches needed, sanctioned protocol. Trait stress test: `link` (QR) is Signal-shaped — Matrix uses password/OAuth, so auth stays backend-specific. |
| Discord | [serenity-rs/serenity](https://github.com/serenity-rs/serenity) or [twilight-rs/twilight](https://github.com/twilight-rs/twilight) | Both alive |
| Slack | [abdolence/slack-morphism-rust](https://github.com/abdolence/slack-morphism-rust) | Web/Events/Socket Mode |
| Bluesky | [sugyan/atrium](https://github.com/sugyan/atrium) | ATProto |
| XMPP | [prose-im/prose-core-client](https://github.com/prose-im/prose-core-client) | Small but alive |
| IRC | [aatxe/irc](https://github.com/aatxe/irc) | Stable, low churn |
| iMessage | [ReagentX/imessage-exporter](https://github.com/ReagentX/imessage-exporter) | Read-only — send-side requires AppleScript shellout |
| RCS | — | No Rust option |

## H6 — Outer shell is an MCP server

| Question | Current belief | Status |
|---|---|---|
| Primary distribution shape | An MCP server. Tools = backend operations (`signal.send_text`, `signal.list_threads`, `signal.receive_recent`, etc.). LLM lives in the client. | HYPOTHESIS |
| Transport — v1 | **stdio only.** Matches Claude Desktop / Claude Code launching us as a subprocess. Simplest to verify. | HYPOTHESIS |
| Transport — later | HTTP. Long-running native server somewhere (not Cloudflare Workers — presage can't run there). | HYPOTHESIS |
| MCP SDK | Official [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) crate. Audit when we actually take the dep, not before. | HYPOTHESIS |
| Build order | Backend trait implemented against presage first, MCP wrapping second. Cheaper iteration; wrap is mechanical once tools are real. | HYPOTHESIS |
| Auth model | None for v1 (local subprocess, trusts the launching client). Revisit when HTTP transport lands. | HYPOTHESIS |

## H8 — Matrix backend  *(AUDITED — see [03-matrix.md](docs/due-diligence/03-matrix.md) + homeserver eval [04-matrix-self-host.md](docs/due-diligence/04-matrix-self-host.md))*

| Field | Value | Status |
|---|---|---|
| Client crate | [`matrix-sdk` v0.17.0](https://crates.io/crates/matrix-sdk) | AUDITED |
| Store crate | `matrix-sdk-sqlite` (workspace member) with at-rest encryption | AUDITED |
| Required patches | **none** — clean Cargo.toml, no fork wrangling | AUDITED |
| Build prereqs | none additional (no protoc, no boringSSL gymnastics) | AUDITED |
| Auth model | password / OAuth2 / SSO — **not** QR-linkable. The trait's `link` verb does not generalize across backends; auth stays per-backend. | AUDITED |
| Homeserver — dev | `matrix.org` (zero setup) | AUDITED |
| Homeserver — self-host | **[Continuwuity](https://forgejo.ellis.link/continuwuation/continuwuity)** v0.5.9 (community continuation of Conduit/conduwuit). Single binary, RocksDB-backed, Linux pre-builds, active 5+ committers. Conduit itself is stalled (1 commit in 90 days). Maelstrom is the scale-oriented alt, deferred. | AUDITED |
| AppService bridges (strategic) | **AUDITED — see [05-mautrix-bridges.md](docs/due-diligence/05-mautrix-bridges.md).** Bridge ecosystem alive: 14+ active Go bridges under [`mautrix/*`](https://github.com/mautrix), all pushed within days of each other, shared `mautrix/go` framework. Top stars: WhatsApp 1,775★ / Telegram 1,684★ / Signal 648★ / Discord 459★ / iMessage 438★ / Meta 371★. Single-maintainer-coordinated (tulir) — strength (lockstep) and risk (bus factor). Not for v1; meaningful Phase-2 lever. | AUDITED |
| Live verification | **deferred** — eval is static (gh + crates.io + forgejo). No `signal:verify`-equivalent against a real homeserver yet. |  |

## H7 — On-disk state lives in `.data/`

| Question | Current belief | Status |
|---|---|---|
| Where do per-backend stores live? | `.data/<backend>/` inside the repo. Currently `.data/signal/signal.db` (relocated from `/tmp/chat-isomorphic-verify/` and verified to still load — same ACI/PNI). | AUDITED |
| Committed? | No. `.data/` is gitignored — contains real Signal identity keys, message history, attachments. | AUDITED |
| Cross-machine? | No. Relink per machine. Cheap because the verify dance is one QR scan. | AUDITED |

## H5 — Architecture shape

| Question | Current belief | Status |
|---|---|---|
| One binary or one-per-backend? | One binary, backends gated behind Cargo features | AUDITED (scaffold compiles, `cargo check --workspace` green) |
| Workspace layout | `crates/core` (trait) + `crates/backend-<x>` (one per backend) + `bin/chat-isomorphic` (feature-flagged deps) | AUDITED |
| Backend on/off — compile time | Cargo features on the binary crate: `--features signal`, etc. Default features = none. | AUDITED |
| Backend on/off — runtime | Deferred until 2+ backends exist; placeholder for `--backend <name>` CLI flag | HYPOTHESIS |
| Async runtime | tokio current-thread in bin scaffold; will need a Signal-dedicated LocalSet thread with 8 MiB stack once presage is wired (verified `!Send`) | AUDITED |
| State persistence | SQLite, separate DB from any backend's internal store (jean-claude side-table pattern) | HYPOTHESIS |
