# chat-isomorphic

A Rust binary that abstracts over many chat / messenger backends behind one
trait, driven by an LLM agent.

The bet: chats are structurally equivalent — accounts, threads, messages,
attachments, reactions — and a single isomorphic layer can route between them.

## Status

Scaffolding only. Nothing has been built yet. See [HYPOTHESES.md](HYPOTHESES.md)
for current beliefs about the stack and [docs/due-diligence/](docs/due-diligence/)
for evaluations that turn beliefs into decisions.

## Planned backends

Signal first (via [whisperfish/presage](https://github.com/whisperfish/presage)).
Others — WhatsApp, Telegram, Matrix, Discord, Slack, Bluesky, XMPP, IRC,
iMessage — are candidates pending due diligence, not commitments.
