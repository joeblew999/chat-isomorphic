# 01a — presage live verification on this machine

**Date:** 2026-05-18
**Machine:** macOS arm64 (Apple Silicon), Tahoe
**Rust:** 1.95.0
**presage rev:** `6793c3e8c30b807bddc7134c77049889bc57cf09`
**Target Signal account:** linked secondary device against an iOS primary

Real end-to-end verification on the operator's own Signal install. Every claim
in [01-signal-presage.md](01-signal-presage.md) that depended on "presage
appears to work" is either confirmed or qualified here.

## Build environment

- `protoc` is **mandatory** at build time. `spqr` (Sparse Post-Quantum Ratchet,
  pulled in transitively by `libsignal-service` → `libsignal`) panics in its
  `build.rs` if `protoc` is missing. Bundled descriptors are not used.
  On macOS: `brew install protobuf` (we landed on `libprotoc 34.1`).
- Cold cargo build: ~2 min for dep graph fetch + ~3-4 min for compilation.
  Incremental rebuild after `protoc` install: **46 seconds**, 19 MB binary.
- `[patch.crates-io]` matched the eval: curve25519-dalek, rusqlite (cdsi only),
  quickcheck_async (dev-only). No surprises.

## End-to-end capabilities — observed

| Capability | Result | Detail |
|---|---|---|
| `link-device` (QR provisioning) | ✅ | Provisioning URL has ~60-90 s lifetime; if the user isn't ready when the QR pops, the handshake errors with `ProvisioningError(MissingMessage)`. Operationally: render the QR PNG immediately after the URL is logged, then alert the user, then wait. |
| Linked device on iOS primary | ✅ | Visible in Signal → Settings → Linked Devices as "chat-isomorphic-verify" |
| `whoami` | ✅ | Returned ACI / PNI / E.164 |
| `stats` | ✅ | Pre-keys and Kyber pre-keys populated |
| `sync-contacts` request | ⚠️ | `manager.request_contacts()` returns OK (the SyncMessage::Request goes out), but the wait loop for `Received::Contacts` **never resolves** — sat for 6+ min the first run, 45 s the second, no events arrived. Repeated 2026-05-18 via our own binary (`signal request-contacts` + `signal events`): same result. Conclusion: iOS does **not** push contact-sync back to a fresh presage-linked secondary. Mirrors the inverse problem in upstream presage issue #328 (presage primary fails to push to Signal-Desktop secondary). The list-contacts / list-groups verbs work — the store just stays empty because nothing writes to it. **Workaround for chat-isomorphic:** derive contacts opportunistically from observed message traffic (every `Inbound` event carries an ACI we could `retrieve_profile_by_uuid` on and cache). Deferred until we need contact names in the MCP surface. |
| `send` to own ACI (Note-to-Self) | ✅ | `manager.send_message(own_aci, DataMessage{body=…}, ts)` succeeded, "message sent!" with exit 0. Confirmed visible on iOS. |
| `sync -q` (`stop_after_empty_queue`) | ✅ | **`Manager::receive_messages()` works.** Stream emits `Received::QueueEmpty` cleanly and the loop exits. |

## Findings that matter for our wrapper

1. **`Manager::receive_messages()` is the right primitive.** Verified live.
   Use it directly; treat `Received::QueueEmpty` as "backlog drained, switch
   to live mode," not as a stop signal. (xdb-agent's pattern stands.)

2. **Don't replicate presage-cli's `sync-contacts` semantics.** Requesting a
   contact sync and *waiting for the response on the stream* is fragile —
   the iOS primary's behavior toward a freshly linked secondary may or may
   not deliver a `Received::Contacts`. Treat contact sync as best-effort.

3. **QR provisioning has a short window.** Bake into the wrapper: emit the
   provisioning URL → render PNG → surface to the user → only *then* wait
   on the linking future. Our `link` UX must minimize the gap.

4. **Attachments default to a tempdir.** presage-cli sets
   `/var/folders/.../presage-attachmentsXXXX` per invocation. Our wrapper
   needs an explicit, configurable attachment path with cleanup semantics.

5. **`presage-cli sync` defaults to a forever-live loop.** The `-q` flag is
   opt-in. Our wrapper has no equivalent concern — we'll always be in
   live mode — but anyone scripting against presage-cli should know.

6. **Identity TOFU.** On first contact with our own primary, we saw
   `WARN presage_store_sqlite::protocol: trusting new identity address=…`.
   That's `OnNewIdentity::Trust` doing what it says. Acceptable for a
   personal bot; revisit if/when the wrapper supports running on behalf
   of other users.

## Bugs from [01-signal-presage.md](01-signal-presage.md) — status against this run

| Bug | Saw it? |
|---|---|
| Receive loop drops `DecryptionErrorMessage` | Not exercised — we didn't receive any DEMs |
| PNI cipher first-contact failure | Not exercised — no first-contact inbound |
| `presage-cli register` websocket failure | Not exercised — we used `link-device` instead |

Three known footguns remain unverified live. Plan: add focused probes for
each once the wrapper is in place and we can drive scenarios.

## What this changes in HYPOTHESES.md

- **H1**: stays AUDITED. Adds:
  - `protoc` is a mandatory build dependency
  - QR-window timing constraint (~60-90 s)
  - `sync-contacts` semantics are CLI-only — don't bake into the wrapper
- **H5** (architecture): receive loop using `Manager::receive_messages` +
  `Received::QueueEmpty` as a mode-flip is now AUDITED, not just borrowed
  from gurk/xdb-agent.
