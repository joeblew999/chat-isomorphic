#!/usr/bin/env bash
# Composite read/write validation against the linked Signal device.
# Exits 0 only if every step succeeds.
set -euo pipefail

: "${PRESAGE_CLI:?set PRESAGE_CLI}"
: "${SIGNAL_STORE_PATH:?set SIGNAL_STORE_PATH}"

scripts_dir="$(cd "$(dirname "$0")" && pwd)"

echo "── whoami ─────────────────────────────────────────"
"$PRESAGE_CLI" --sqlite-db-path "$SIGNAL_STORE_PATH" whoami | grep -v 'attachments will be stored'

echo
echo "── stats ──────────────────────────────────────────"
"$PRESAGE_CLI" --sqlite-db-path "$SIGNAL_STORE_PATH" stats | grep -v 'attachments will be stored'

echo
echo "── sync -q (drain backlog, read path) ─────────────"
"$PRESAGE_CLI" --sqlite-db-path "$SIGNAL_STORE_PATH" sync -q 2>&1 | tail -2

echo
echo "── send to self (write path) ──────────────────────"
"$scripts_dir/send-self.sh" "[chat-isomorphic verify] composite-run $(date '+%Y-%m-%dT%H:%M:%S')"

echo
echo "all checks passed — read + write paths are healthy"
