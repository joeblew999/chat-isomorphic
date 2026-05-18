# 04 — Self-hosting Matrix in Rust (homeserver choice)

**Verdict:** ADOPT **Continuwuity** as the recommended self-host. REJECT Conduit (stalled). DEFER Maelstrom. matrix.org is the zero-friction default for getting-started; self-host is for parity with Signal's "no third party in the data path" ethos.
**Date:** 2026-05-18

For a chat-isomorphic Matrix backend, the deployment shape is: our binary speaks the Matrix Client-Server API (via `matrix-sdk`) to **a homeserver**. The homeserver is where federation happens, where rooms are persisted, and where AppService bridges plug in. If we want the same "we control the data path" property that presage gives us with Signal, the homeserver has to be self-hosted in Rust.

## Candidates

| Project | Repo (canonical) | Stars | Last active | Status | Notes |
|---|---|---|---|---|---|
| **Conduit** | [gitlab.com/famedly/conduit](https://gitlab.com/famedly/conduit) | 940 | 2026-05-12 push, **1 commit in last 90 days** | ❌ **Stalled** | The original Rust homeserver. Author paused. Don't build on top. |
| **conduwuit** (original) | [girlbossceo/conduwuit](https://github.com/girlbossceo/conduwuit) | — | archived | ❌ **Archived** | Project was shut down after maintainer drama. Fork lineage: Conduit → conduwuit → continuwuity. |
| **Continuwuity** | [forgejo.ellis.link/continuwuation/continuwuity](https://forgejo.ellis.link/continuwuation/continuwuity) (mirror: [continuwuity/continuwuity](https://github.com/continuwuity/continuwuity), 782★) | 90 forgejo / 782 GitHub mirror | **v0.5.9 released 2026-05-07** | ✅ **ADOPT** | Community continuation of conduwuit. 30+ commits/recent batch across 5+ humans, Renovate bot tracking deps. |
| **Maelstrom** | [maelstrom-rs/maelstrom](https://github.com/maelstrom-rs/maelstrom) | 254 | 2026-04-14 | 🟡 **DEFER** | "The Matrix homeserver for serious scale" — clusterable, HA. Single dominant maintainer (12 commits/90d). Interesting if/when we need horizontal scale. |

## Continuwuity — build viability

- **Workspace**: `src/{admin,api,core,database,main,router,service,...}`. Single binary `conduwuit` (legacy name).
- **Storage**: **RocksDB** — file-backed, no external DB process. Embedded.
- **Release artifacts** (per v0.5.9): pre-built statically-linked binaries for Linux amd64 (79 MiB) + arm64 (71 MiB), plus `-maxperf` variants (54-64 MiB) built with march-tuning. **No macOS binary** — compile from source if running on dev boxes.
- License: Apache-2.0.
- Mirrored to Forgejo (primary), GitHub, GitLab, Codeberg. The Forgejo instance is where issues/PRs live.

## Operational quality

- v0.5.9 released 2026-05-07 (12 days ago). v0.5.8 → 0.5.7 → 0.5.9 weekly-to-fortnightly release cadence — healthy.
- Recent commits: stratself (6), Ginger (4), timedout (3), 31a05b9c (3), Jade (2), Renovate Bot (12 — dep tracking automation). **5+ active humans.**
- Documentation tree (`docs/`): `appservices.mdx`, `calls.mdx`, `configuration.mdx`, `deploying.mdx` + `deploying/` subdir, `maintenance.mdx`, `reference/`, `security.mdx`, `troubleshooting.mdx`. Comprehensive.

## What this gets us — vs running matrix.org

| Property | matrix.org (public) | Self-hosted Continuwuity |
|---|---|---|
| Setup cost | zero — sign up, log in | TLS cert + DNS + reverse-proxy + open inbound 8448 |
| Operator sees federation traffic | yes (matrix.org sees who you talk to) | no (you see it) |
| Account portability | matrix.org-bound | bound to your homeserver's domain |
| Federation | full (default Matrix network) | full (same) |
| Cost | free | hardware/colo for a small VPS |
| AppService bridges | not under your control | install any: `mautrix-whatsapp`, `mautrix-telegram`, `mautrix-signal`, `mautrix-imessage`, … |
| Aligns with "Signal-style no-third-party" | no | **yes** |

For development / first integration: point at matrix.org, get going fast.
For the "we control the data path" deployment: spin up Continuwuity.

## Operational footguns

1. **No macOS binary** — for dev on Mac you compile from source (RocksDB pulls in C/C++ deps; build is slower than presage's). Production deployment is Linux.
2. **Open inbound port 8448 + valid TLS** — Matrix federation requires an https-reachable server. Behind home NAT this means port forwarding or a reverse-proxy + tunnel (Cloudflare Tunnel works; many self-hosters use it). Documented but real.
3. **DNS — `.well-known/matrix/server`** — federation discovery uses this if you can't run on port 443 directly. One more thing to set up correctly.
4. **First-party-only client model**: Continuwuity (and Conduit lineage in general) targets `matrix-rust-sdk` / Element clients. Some niche Matrix clients depend on Synapse-specific extensions. Not our problem with our own client.
5. **AppService bridges are a separate deploy** — `mautrix-whatsapp` is a Go binary you run alongside Continuwuity, register via `appservices.mdx`. Each bridge is its own ops surface.
6. **Single binary, single RocksDB** — good for "I run one server"; concerning at scale. If you grow past a few users + heavy bridging, look at Maelstrom or Synapse-Postgres.

## Effect on chat-isomorphic shape

- Our **Matrix backend** ([crates/backend-matrix](#)) needs a `homeserver_url` config — defaulting to `https://matrix.org` for development. Operators self-hosting set it to `https://matrix.yourdomain.tld`.
- Login: username + password (or OAuth2). Different from Signal's QR — confirms the trait-level finding that `link` is Signal-shaped and auth stays per-backend.
- Once authed, the rest (`whoami`, `events`, `send`, `list-rooms`, ...) is homeserver-agnostic — matrix-sdk handles the protocol, the homeserver routes the bytes.

## What this changes in HYPOTHESES.md

- **H8** (Matrix backend) gets a new row: *Deployment model — point at matrix.org for dev (zero setup), self-host Continuwuity for production / Signal-parity. Both work with the same `matrix-sdk` client code.*
- New optional row: *AppService bridges — strategic but not for v1. Add `mautrix-whatsapp` etc. only if/when we want indirect coverage of other networks via Matrix.*
