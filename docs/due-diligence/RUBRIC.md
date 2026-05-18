# Due-diligence rubric

Every candidate library that lands in [HYPOTHESES.md](../../HYPOTHESES.md) gets
evaluated against this rubric before any code depends on it. Output lives in
this directory as `NN-<topic>.md` with a verdict at the top.

## Verdict header

Each eval starts with one of:

- **ADOPT** — meets the bar, lock into HYPOTHESES.md
- **ADOPT WITH CAVEATS** — meets the bar but specific footguns to track
- **DEFER** — promising but not enough info yet; concrete next probe listed
- **REJECT** — found a blocker; alternative or workaround named

## Dimensions

### 1. Build viability
- Does `cargo check` succeed on a clean toolchain, today?
- `[patch.crates-io]` requirements (transitive forks, pinned revs)
- Cold-build time and dep-tree size on the target platform
- Cross-compile story if it matters (Linux/macOS/ARM)

### 2. API fit for the isomorphic trait
- Can it cleanly implement `Backend::{link, receive_stream, send, react, edit, attachment_upload, typing, read_receipt}`? Which are missing?
- Is the client/manager type `Send`? If not, what containment is needed?
- Inbound model: `Stream<Event>` vs callbacks vs polling
- Store ownership: client-owned vs BYO; how decoupled
- Group / multi-party support — first-class or bolted on

### 3. Operational quality
- Active committer count in last 90 days (bus factor)
- Open-issue count + label distribution; look for "blocker", "deadlock", "crash", "panic"
- Breaking-change cadence on the public API
- Tests + CI present and meaningful
- Real-world consumers (crates.io reverse-deps + GitHub code search)

### 4. Platform / protocol risk
- Sanctioned API (Bot API, official SDK) vs reverse-engineered (whatsmeow-style)
- Ban-risk posture; rate-limit handling
- E2EE / crypto correctness if applicable

### 5. Repo ergonomics
- Examples that actually run end-to-end
- Docs depth and currency
- `llms.txt` or other LLM-friendly hints
- Issue-tracker responsiveness from maintainers

## Eval template

```markdown
# NN — <candidate>

**Verdict:** ADOPT | ADOPT WITH CAVEATS | DEFER | REJECT
**Date:** YYYY-MM-DD
**Repo @ rev:** <url>@<rev>

## Build viability
…

## API fit
…

## Operational quality
…

## Platform risk
…

## Repo ergonomics
…

## Footguns to carry forward
- …

## What this changes in HYPOTHESES.md
- H_ updated from HYPOTHESIS → AUDITED, with: …
```
