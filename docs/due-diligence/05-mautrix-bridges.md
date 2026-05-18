# 05 — mautrix bridge ecosystem (sanity check)

**Verdict:** Strategic claim in [H8](../../HYPOTHESES.md) **stands**. The mautrix bridge ecosystem is uniformly active, single-maintainer-coordinated, and covers every messenger we'd plausibly want to reach via Matrix.
**Date:** 2026-05-18
**Source:** `gh api orgs/mautrix/repos` 2026-05-18.

Reminder on scope: bridges are **operator-deployed Matrix AppServices** running alongside a homeserver. They are *not* in our build, our deps, or our trait. This eval validates whether the "Matrix + bridges = indirect coverage of N networks" framing is real or wishful thinking.

## The active bridges (Go, current)

Every entry below was pushed within 4 days of 2026-05-17. Same primary maintainer ([@tulir](https://github.com/tulir)), shared framework (`mautrix/go`).

| Bridge | Repo | Stars | Last push | Notes |
|---|---|---|---|---|
| **WhatsApp** | [mautrix/whatsapp](https://github.com/mautrix/whatsapp) | **1,775** | 2026-05-16 | Highest-starred bridge — the canonical reason people deploy mautrix |
| **Telegram** | [mautrix/telegram](https://github.com/mautrix/telegram) | **1,684** | 2026-05-16 | Go rewrite landed (old Python version retired) |
| **Signal** | [mautrix/signal](https://github.com/mautrix/signal) | 648 | 2026-05-16 | Go-native (was `signalgo`, now merged) |
| **Discord** | [mautrix/discord](https://github.com/mautrix/discord) | 459 | 2026-05-13 | |
| **iMessage** | [mautrix/imessage](https://github.com/mautrix/imessage) | 438 | 2026-05-14 | macOS-host bridge; works on Mac running BlueBubbles or native |
| **Meta (FB Messenger + Instagram)** | [mautrix/meta](https://github.com/mautrix/meta) | 371 | 2026-05-16 | Single bridge covers both; replaces archived mautrix/facebook + mautrix/instagram |
| **Google Messages (RCS)** | [mautrix/gmessages](https://github.com/mautrix/gmessages) | 147 | 2026-05-16 | RCS-over-bridge |
| **Twitter DMs** | [mautrix/twitter](https://github.com/mautrix/twitter) | 111 | 2026-05-16 | |
| **Slack** | [mautrix/slack](https://github.com/mautrix/slack) | 97 | 2026-05-16 | |
| **Bluesky DMs** | [mautrix/bluesky](https://github.com/mautrix/bluesky) | 40 | 2026-05-16 | |
| **Google Voice** | [mautrix/gvoice](https://github.com/mautrix/gvoice) | 31 | 2026-05-16 | |
| **LinkedIn** | [mautrix/linkedin](https://github.com/mautrix/linkedin) | 25 | 2026-05-16 | |
| **IRC** | [mautrix/irc](https://github.com/mautrix/irc) | 13 | 2026-05-16 | (Distinct from the older matrix-appservice-irc) |
| **Zulip** | [mautrix/zulip](https://github.com/mautrix/zulip) | 8 | 2026-05-16 | |

Plus the shared framework: [mautrix/go](https://github.com/mautrix/go) — 621★, last push 2026-05-17. Single point of leverage means a bug fix or protocol change here lands in every bridge nearly simultaneously.

## Older / Python-flavored (still alive but slower)

| Bridge | Stars | Last push | Status |
|---|---|---|---|
| [mautrix/python](https://github.com/mautrix/python) | 240 | 2026-04-12 | Python framework for the still-Python bridges below |
| [mautrix/googlechat](https://github.com/mautrix/googlechat) | 122 | 2026-04-22 | Python; not yet Go-rewritten |

## Archived (cleanly retired)

These are properly archived with merge/replacement context — not abandoned.

| Repo | Reason |
|---|---|
| mautrix/facebook | Replaced by [mautrix/meta](https://github.com/mautrix/meta) (2024-03-02) |
| mautrix/instagram | Replaced by [mautrix/meta](https://github.com/mautrix/meta) (2024-03-02) |
| mautrix/telegramgo | Merged into [mautrix/telegram](https://github.com/mautrix/telegram) (2026-03-28) |
| mautrix/signalgo | Merged into [mautrix/signal](https://github.com/mautrix/signal) (2023-12-18) |
| mautrix/syncproxy | Superseded by direct websockets (2023-06-14) |
| mautrix/deltachat | Abandoned WIP (2023-04-05) |

The Python → Go consolidation is the dominant pattern. The old Python projects are gone or going.

## What this means for chat-isomorphic

- **The "Matrix + bridges" multiplier is real.** Operator who runs Continuwuity can drop in any of these 14 active bridges and our Matrix backend gets that network's traffic — appearing as Matrix rooms.
- **Single-maintainer-coordination is a strength here.** Tulir keeps the protocol-tracking work centralized in `mautrix/go`; bridges share a release cadence; a Meta protocol change reaches every Meta-touching bridge in days.
- **All bridges are Go.** Not our language, but **not our problem** — we don't link them. They're separate processes the operator deploys.
- **Each bridge is its own deploy + config** — registering with the homeserver, persistent SQLite or PostgreSQL per bridge, credentials for the foreign network. ~10-30 min per bridge, then runs untouched.

## Footguns to flag (for operators, not for our code)

1. **Encryption story weakens.** A bridge is a man-in-the-middle by design — it sees plaintext to translate between Matrix events and the other protocol. For Signal-strict E2EE workflows, the bridge is plaintext-on-the-bridge-host. Operator must trust the bridge host equivalently to their primary account.
2. **TOS / ban risk on the wrapped network.** mautrix/whatsapp uses whatsmeow (reverse-engineered WhatsApp). Same risk profile as oxidezap/whatsapp-rust we noted earlier. Meta could ban; protocol churn is expected.
3. **Per-bridge state surface.** Each bridge stores its own session credentials, contacts, encryption keys for its own network. Backing up / migrating means N moving parts.
4. **iMessage is macOS-only.** mautrix/imessage either drives a real macOS Messages.app via the host (BlueBubbles pattern) or runs Apple's protocol natively on Mac. Won't help anyone on Linux.
5. **Single-maintainer bus factor.** Whole ecosystem leans on tulir. If they get hit by a bus, churn slows substantially. Mitigated by source availability + Element's larger backing of Matrix overall.

## What this changes in HYPOTHESES.md

- **H8 "AppService bridges (strategic)" row:** promoted from HYPOTHESIS to **AUDITED** with the verdict that the bridge ecosystem is alive and uniformly maintained. Phrase remains "not for v1" — we still ship Signal direct + Matrix native first.
- **H4 row entries** for WhatsApp, Telegram, Discord, Slack, Bluesky, iMessage can each note "indirect access via mautrix-* AppService" as an alternative deployment path to a native Rust integration. Native Rust still wins on latency, plaintext-on-host concerns, and not depending on tulir's roadmap; bridges win on speed-to-coverage and not having to maintain N protocol clients ourselves.
