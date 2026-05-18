# chat-isomorphic

A Rust binary that abstracts over many chat / messenger backends behind one
trait, driven by an LLM agent.

The bet: chats are structurally equivalent — accounts, threads, messages,
attachments, reactions — and a single isomorphic layer can route between them.

## Status

Scaffolding only. Nothing has been built yet. See [HYPOTHESES.md](HYPOTHESES.md)
for current beliefs about the stack and [docs/due-diligence/](docs/due-diligence/)
for evaluations that turn beliefs into decisions.

## Backends are opt-in at compile time

Each backend is gated behind a Cargo feature on the `chat-isomorphic` binary
crate so you only pay for what you build:

```bash
cargo build --release -p chat-isomorphic                          # no backends
cargo build --release -p chat-isomorphic --features signal        # presage compiled in
cargo build --release -p chat-isomorphic --features signal,matrix # both, once matrix exists
```

`cargo check --workspace` checks every backend crate regardless of feature
selection; it's the binary that picks which backends end up in the build.

A runtime enable/disable switch (config file or CLI flag selecting between
multiple compiled-in backends) is on the list once we have more than one.

## Planned backends

Signal first (via [whisperfish/presage](https://github.com/whisperfish/presage),
audited in [01-signal-presage.md](docs/due-diligence/01-signal-presage.md) and
[01a](docs/due-diligence/01a-presage-live-verification.md)). Others — WhatsApp,
Telegram, Matrix, Discord, Slack, Bluesky, XMPP, IRC, iMessage — are candidates
pending due diligence, not commitments.
