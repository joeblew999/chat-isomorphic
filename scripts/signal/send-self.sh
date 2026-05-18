#!/usr/bin/env bash
# Send a message to your own ACI (Note-to-Self) — write-path validation.
#
# Usage:  scripts/signal/send-self.sh "your message"
#         (default body if omitted: a timestamped verify ping)
set -euo pipefail

: "${PRESAGE_CLI:?set PRESAGE_CLI}"
: "${SIGNAL_STORE_PATH:?set SIGNAL_STORE_PATH}"

BODY="${1:-[chat-isomorphic verify] $(date '+%Y-%m-%d %H:%M:%S')}"

# whoami returns: WhoAmIResponse { aci: <uuid>, pni: <uuid>, number: ... }
ACI=$("$PRESAGE_CLI" --sqlite-db-path "$SIGNAL_STORE_PATH" whoami 2>/dev/null \
  | grep -oE 'aci: [0-9a-f-]{36}' | head -1 | awk '{print $2}')
[[ -n "$ACI" ]] || { echo "could not extract own ACI from whoami" >&2; exit 1; }

echo "→ sending to self (aci=$ACI): $BODY"
"$PRESAGE_CLI" --sqlite-db-path "$SIGNAL_STORE_PATH" \
  send --uuid "$ACI" --message "$BODY"
