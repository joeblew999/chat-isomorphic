#!/usr/bin/env bash
# Link this machine as a Signal secondary device.
#
# Spawns presage-cli link-device in the background, waits for it to emit
# the sgnl:// provisioning URL, renders that URL to a QR PNG, opens the
# PNG in Preview, then blocks until presage-cli completes (or 5 min).
#
# The provisioning URL has a ~60-90 s lifetime. Have your phone open on
# Signal → Settings → Linked Devices BEFORE running this.
#
# Env (set by mise.toml):
#   PRESAGE_CLI        path to the presage-cli binary
#   SIGNAL_STORE_PATH  sqlite db path
#   SIGNAL_DEVICE_NAME label that shows up under iOS Linked Devices
set -euo pipefail

: "${PRESAGE_CLI:?set PRESAGE_CLI to the presage-cli binary}"
: "${SIGNAL_STORE_PATH:?set SIGNAL_STORE_PATH}"
: "${SIGNAL_DEVICE_NAME:=chat-isomorphic-dev}"

[[ -x "$PRESAGE_CLI" ]] || { echo "PRESAGE_CLI not executable: $PRESAGE_CLI" >&2; exit 1; }
command -v qrencode >/dev/null || { echo "qrencode missing — brew install qrencode" >&2; exit 1; }

LOG="$(dirname "$SIGNAL_STORE_PATH")/link.log"
PNG="$(dirname "$SIGNAL_STORE_PATH")/qr.png"
mkdir -p "$(dirname "$SIGNAL_STORE_PATH")"
: > "$LOG"

"$PRESAGE_CLI" --sqlite-db-path "$SIGNAL_STORE_PATH" \
  link-device --device-name "$SIGNAL_DEVICE_NAME" \
  > "$LOG" 2>&1 &
PRESAGE_PID=$!
trap 'kill $PRESAGE_PID 2>/dev/null || true' EXIT

# Wait up to 30 s for the provisioning URL to appear.
for _ in $(seq 1 60); do
  grep -qE 'sgnl://linkdevice' "$LOG" 2>/dev/null && break
  sleep 0.5
done

URL=$(grep -oE 'sgnl://linkdevice[^[:space:]]*' "$LOG" | head -1)
[[ -n "$URL" ]] || { echo "no provisioning URL emitted; presage-cli log:" >&2; cat "$LOG" >&2; exit 1; }

qrencode -o "$PNG" -s 10 -m 4 "$URL"
echo "QR rendered: $PNG"
echo "URL fallback: $URL"
open "$PNG" 2>/dev/null || true
echo "Scan from your phone: Signal → Settings → Linked Devices → +"

# Wait for presage-cli to finish (success or ProvisioningError).
wait $PRESAGE_PID
trap - EXIT

if grep -q 'ProvisioningError' "$LOG"; then
  echo "link failed (likely URL expired before scan):" >&2
  tail -3 "$LOG" >&2
  exit 1
fi

echo "linked successfully"
