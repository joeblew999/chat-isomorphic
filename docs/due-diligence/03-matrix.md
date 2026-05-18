# 03 — Matrix via matrix-org/matrix-rust-sdk

**Verdict:** ADOPT (cleanly — easier than presage on every operational axis)
**Date:** 2026-05-18
**Repo @ rev:** [matrix-org/matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) @ `929551152e2e` (2026-05-15)
**Crate:** [`matrix-sdk` v0.17.0](https://crates.io/crates/matrix-sdk) on crates.io (347 771 downloads, updated 2026-05-08)

## Build viability

- **No `[patch.crates-io]` required.** Standard workspace, normal crates.io deps. `Cargo.toml` is clean — no Signal-fork curve25519, no whisperfish-rusqlite, no nytopop-quickcheck. Big difference from presage.
- Workspace resolver = "3", `rust-version = "1.93"` (we're on 1.95).
- No `protoc` requirement observed in the build graph.
- Crates we'd consume from a chat-isomorphic backend:
  - `matrix-sdk` — high-level client (the main surface)
  - `matrix-sdk-sqlite` — SQLite store with at-rest encryption (matches the pattern presage uses)
  - `matrix-sdk-crypto` — pulled in transitively for E2EE
- Stable releases on crates.io. **Pin a version (0.17.0)**, not a git rev — exactly the opposite of presage's reality.

## API fit for the isomorphic trait

`crates/matrix-sdk` exposes a `Client` + `Room` model:

| Our trait method | Matrix mapping |
|---|---|
| `whoami` | `Client::user_id()` (local), `Client::whoami()` (server round-trip) |
| `link` (auth) | **Different shape** — Matrix uses password / OAuth2 / SSO login, not QR-linking. The `link` verb on the trait won't generalize; each backend's auth stays backend-specific. The trait covers post-auth ops. |
| `events` | `Client::sync_stream(SyncSettings)` returns a stream of sync responses; rooms expose timeline events. We adapt this to `Event` variants the same way we did for presage. |
| `send` | `Room::send(AnyMessageLikeEventContent::RoomMessage(...))`. Threads = rooms; member ACIs = MXIDs (e.g. `@gerard:matrix.org`). |
| `list-devices` | `Client::encryption().get_user_devices(user_id)` lists this account's verified devices |
| `unlink` | `Client::encryption().delete_devices(...)` — *primary-or-any* unlike Signal's secondary-only restriction |
| `list-contacts` / `list-groups` | Different model — Matrix has rooms (DMs + groups + public channels). No separate contact list concept; the "contact" of a DM is the other member of a 1:1 room. Maps to "list rooms". |
| Reactions / edits / attachments | First-class Matrix event types (`m.reaction`, `m.replace`, `m.image` etc.) — same shape as DataMessage variants |

**Trait stress test:** Matrix exercises whether our trait is genuinely isomorphic or quietly Signal-shaped. Likely outcome — the `link()` method becomes backend-specific in practice, and `Inbound::Other` variants we don't fold in lose granularity. Both findings sharpen the trait.

## Operational quality

| Metric | Value |
|---|---|
| Stars | **2,124** |
| Forks | 435 |
| Open issues | 309 (large project; not all are bugs) |
| 90-day committers | Jorge Martín (26), Damir Jelić (20), Michael Goldenberg (16), Stefan Ceriu (9), Kévin Commaille (8), + 5 other humans + dependabot |
| crates.io reverse deps | **52** (vs presage's 0) — real ecosystem |
| Sponsor | **Element** (the company behind the flagship Matrix client) — strong commercial maintenance |
| CI | GitHub Actions, status badge in README. Codecov integration. |
| Examples | 15 in `examples/` — including `command_bot` (closest to our MCP-tool use case), `getting_started`, `timeline`, `login`, `oauth_cli`, `image_bot`, `autojoin` |

### Open issues that look scary on the title but don't actually bite us

- *"Android crash after removal of ring dependency"* — Android-specific
- *"iOS: 0xdead10cc crashes — SQLite WAL locks held by tokio blocking threads during background suspension"* — iOS background-suspension specific
- *"Crashes on Element X Android"* — Android-specific
- *"`media_store_integration_tests_time::test_lease_locks` intermittently fails"* — flaky test, low impact
- *"emoji_verification example need work, user to user verification process broken for multiple clients"* — feature edge case (cross-signing verification UX)

None of these are blockers for a macOS bot. The iOS WAL-lock issue is the most interesting because it reveals that SQLite + tokio blocking-pool can deadlock under specific lifecycle pressure — worth noting if/when we run multi-threaded.

## Platform / protocol risk

- Matrix is a **public, openly-specified protocol** ([spec.matrix.org](https://spec.matrix.org)). Not reverse-engineered; not subject to a vendor pulling the rug. Most importantly: **federated** — no single corporate gatekeeper can ban our account.
- E2EE via Olm/Megolm (Signal-protocol-derived double ratchet + tree-based group key agreement). First-class in `matrix-sdk-crypto`.
- The `matrix.org` flagship homeserver exists, but the operator can run their own or pick any other. Self-sovereign.

## Repo ergonomics

- README is current, well-styled, clearly identifies Element sponsorship and audience (Matrix clients, bots, full apps).
- **15 first-party examples**, each its own crate under `examples/`. `getting_started` and `command_bot` are the right entry points. Tested via CI.
- docs.rs is current (v0.17.0 published 2026-05-08).
- No `llms.txt`. Element-maintained `matrix-rust-sdk-tutorial` exists separately (out-of-tree).
- Codecov coverage tracked.

## Footguns to carry forward

1. **Bigger surface area than presage.** matrix-sdk + matrix-sdk-sqlite + matrix-sdk-crypto is a multi-crate consumer landscape. Pin to a workspace version (0.17.0) and bump deliberately.
2. **Auth is not QR-linkable.** Our `signal link` UX (QR pop, scan from phone) does not map. Matrix backend's onboarding verb is `login --username @gerard:matrix.org --password ...` or OAuth2 flow. Implementation effort lives in the matrix backend, not the trait.
3. **Rooms ≠ Threads.** Matrix has rooms (1:1 DM, group, public). Threads (within a room) are a Matrix concept too. We'll need to clarify `ThreadId` semantics for non-Signal backends.
4. **E2EE state machine.** Matrix's E2EE requires cross-device verification flows for new devices to decrypt history. A naive bot can opt out (don't request history) but should still verify itself with the user's other devices. `examples/cross_signing_bootstrap` is the reference.
5. **iOS WAL-lock issue (cross-reference for future).** If we ever run chat-isomorphic on iOS, the SQLite WAL + tokio blocking-pool interaction needs the workaround upstream is developing. Not relevant for macOS native or Linux server.

## Comparison vs presage (the audited Signal pick)

| Axis | presage | matrix-rust-sdk |
|---|---|---|
| Stars | 237 | **2,124** |
| Committers (90 days) | 5 (one dominant) | **10+ humans, 5+ very active** |
| crates.io | git-only, no version | **v0.17.0, 348k downloads** |
| `[patch.crates-io]` required | 3 patches | **0** |
| Build prereqs | `protoc` mandatory | none observed |
| Sponsor | community | **Element (commercial)** |
| Protocol | reverse-engineered Signal API | **open spec, federated** |
| Critical open bugs for our use | decryption-loop, PNI first-contact, register-WS broken | iOS-specific only |

matrix-rust-sdk is the easier, more conservative, better-maintained dependency on every operational axis. The trade-off is **userbase + ethos** — Signal users are not Matrix users; the two communities barely overlap. Adding Matrix doesn't subsume the need for Signal.

## What this changes in HYPOTHESES.md

- **New H8**: Matrix backend candidate AUDITED. Crate `matrix-sdk` v0.17.0, store `matrix-sdk-sqlite`, no patches required. Auth model is password/OAuth2/SSO (not QR), so the trait's `link` verb stays backend-specific.
- **H4 row for Matrix**: HYPOTHESIS → AUDITED.
- **H1 trait shape**: Matrix's existence sharpens the "Backend trait must be auth-mechanism-agnostic" finding — `link` is too Signal-shaped. Future refactor when we wire Matrix will likely move `link` off the trait and onto each backend as a constructor.
