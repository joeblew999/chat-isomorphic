# 01 — Signal via whisperfish/presage

**Verdict:** ADOPT WITH CAVEATS
**Date:** 2026-05-18
**Repo @ rev:** [whisperfish/presage](https://github.com/whisperfish/presage) @ `6793c3e8c30b807bddc7134c77049889bc57cf09` (2026-05-17, "Fix opening the Sqlite store without a passphrase")

## Build viability

Workspace: `presage`, `presage-cli`, `presage-store-sqlite`. `presage-store-sled` is gone — sqlite is the only first-party store now.

Mandatory `[patch.crates-io]` (from `Cargo.toml`):
- `curve25519-dalek` → `signalapp/curve25519-dalek` tag `signal-curve25519-4.1.3`
- `libsqlite3-sys` → `whisperfish/rusqlite` rev `2a42b3354c9194700d08aa070f70a131a470e7dc` (only required if enabling `cdsi` feature, otherwise optional)
- `quickcheck_async` → `nytopop/quickcheck_async` rev `76b13525c…` (dev-dep only)

`libsignal-service` is a **git-pinned dep** (`rev = "bd585acc…"` on `whisperfish/libsignal-service-rs`), not crates.io. Bumping presage means accepting whatever libsignal-service rev they pinned to.

`cdsi` feature (contact discovery from phone numbers) brings in `libsignal-net` which depends on `boringSSL`. Conflicts with sqlcipher's OpenSSL ≥3 unless using the whisperfish rusqlite fork. Carry this constraint forward.

Not run: actual `cargo check` on a clean toolchain. Building presage in the wild is documented as slow but reliable — gurk-rs, xdb-agent, jean-claude all succeed.

## API fit

[`presage/src/manager/registered.rs`](https://github.com/whisperfish/presage/blob/main/presage/src/manager/registered.rs) exposes a clean async surface that maps to our trait:

| Our trait method | presage method |
|---|---|
| `link` | `Manager::link_secondary` + `register` (SMS / voice) — both registration paths exist |
| `receive_stream` | `Manager::receive_messages` (returns `Stream<Received>`) |
| `send` | `Manager::send_message(service_id, DataMessage, ts)` |
| `send_group` | `Manager::send_message_to_group(master_key, DataMessage, ts)` |
| `attachment_upload` | `Manager::upload_attachment(s)` |
| `attachment_download` | `Manager::get_attachment` |
| `profile` | `Manager::retrieve_profile`, `update_profile` |
| `groups` / `contacts` | iterated via `Store` trait |
| `thread_title` | `Manager::thread_title(&Thread)` |
| `reactions / edits / receipts` | DataMessage fields (no first-class method) |
| `typing indicator` | not in API surface |

Manager is generic over `S: Store`. **Believed `!Send`** — no `unsafe impl Send` in source, and gurk-rs/xdb-agent/jean-claude all confirm it via the `LocalSet`/`LocalPool` pattern. Plan accordingly.

Both registration models supported: phone-number (SMS / voice) **and** secondary-device linking. README claims registration works; one open issue currently blocks `presage-cli register` with a "Websocket upgrade failed" — likely fixed in a recent rev, verify before assuming.

## Operational quality

- Stars: 237, forks: 54, open issues: 37, not archived
- 90-day commit activity: Gabriel Féron (10), Schmiddiii (8), thandal (2), Taras Glek (1), boxdot (1). One dominant maintainer; effective bus factor 2.
- Tests: 3 files with `#[test]` (`presage/src/serde.rs`, `presage/src/manager/mod.rs`, `presage-store-sqlite/src/protocol.rs`). **Sparse.** Don't trust the test suite as a regression net.
- CI: GitHub Actions "Build" workflow is green.
- Downstream consumers: ~30 found via GitHub code search (gurk-rs, xdb-agent, foghorn, signal-tauri, signal-gatewayd, …). Real production usage.

## Platform risk

Unsanctioned client of Signal's network. Risk that Signal changes the protocol and breaks presage; mitigation is the active upstream maintaining libsignal-service-rs in lockstep. Historical precedent: presage has tracked Signal's changes successfully for years.

## Repo ergonomics

- README is thorough; gives the exact `[patch.crates-io]` snippet to copy.
- `presage-cli` is a near-complete reference (register, link, unlink, devices, whoami, profile, receive, send, groups, contacts, stickers). Read this as the canonical example.
- API docs published at `whisperfish.github.io/presage/presage`.
- No `examples/` directory; presage-cli serves that role.
- No `llms.txt`.

## Footguns to carry forward

1. **Decryption-loop bug.** Open issue: *"Receive loop drops DecryptionErrorMessage without archiving sessions; peers stuck in decryption loop."* Real and load-bearing. We will need a workaround (intercept the message, archive the session manually) until upstream fixes it.
2. **PNI cipher first-contact failure.** Open issue: *"PNI cipher fails to decrypt first-contact messages from new senders."* Means initial messages from unknown senders may silently fail. Test this scenario explicitly.
3. **`presage-cli register` websocket error.** Track upstream; for now plan around secondary-device linking as the primary onboarding path.
4. **Manager is `!Send`.** Custom `LocalPool` with 8 MiB stack required (post-quantum crypto blows default stack — gurk-rs comment).
5. **Receive stream must be continuously polled** for sends to make progress (presage uses a duplex WebSocket). xdb-agent's `QueueEmpty` gate + pump-to-mpsc pattern is the right shape.
6. **Bumping presage = bumping libsignal-service** (git-pinned). Major bumps are breaking; gurk's CHANGELOG flags every one.
7. **Test sparsity.** When we wrap presage in our `Backend` trait, we test our wrapper, not the wrapped client.

## What this changes in HYPOTHESES.md

- **H1**: HYPOTHESIS → AUDITED. Updates:
  - `presage-store-sled` was listed as alternative; remove — sled is gone, sqlite is the only path.
  - Linking model: not "secondary only" — phone-number registration works too (with one open bug).
  - Mark `Manager` is `!Send` from "Believed" to "Confirmed".
  - Add the three open-bug footguns explicitly.
- **H5** (architecture): the LocalPool + 8 MiB stack pattern carried from gurk is now confirmed necessary, not aspirational.
