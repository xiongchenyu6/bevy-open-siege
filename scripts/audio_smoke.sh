#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <bevy_open_siege_binary> [duration_seconds]" >&2
  exit 2
fi

BINARY="$1"
DURATION_SECONDS="${2:-${BEVY_OPEN_SIEGE_AUDIO_SMOKE_SECONDS:-25}}"

if [[ ! -x "$BINARY" ]]; then
  echo "audio smoke binary is not executable: $BINARY" >&2
  exit 1
fi
if ! [[ "$DURATION_SECONDS" =~ ^[0-9]+$ ]] || [[ "$DURATION_SECONDS" -lt 5 ]]; then
  echo "audio smoke duration must be an integer of at least 5 seconds" >&2
  exit 2
fi
if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
  export WINIT_UNIX_BACKEND="${WINIT_UNIX_BACKEND:-wayland}"
fi

LOG_FILE="$(mktemp)"
trap 'rm -f "$LOG_FILE"' EXIT

set +e
timeout "${DURATION_SECONDS}s" "$BINARY" --audio > "$LOG_FILE" 2>&1
STATUS=$?
set -e

if grep -Eiq 'panicked at|Encountered a panic|thread .* panicked|error\[B0001\]' "$LOG_FILE"; then
  echo "audio startup smoke failed: panic signature found" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi
if [[ "$STATUS" -ne 124 ]]; then
  echo "audio startup smoke failed: game exited with status $STATUS before ${DURATION_SECONDS}s" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi
if ! grep -q "Creating new window Bevy Open Siege" "$LOG_FILE"; then
  echo "audio startup smoke failed: window creation log not found within ${DURATION_SECONDS}s" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi

cat <<EOF
audio startup smoke ok
duration_seconds: ${DURATION_SECONDS}
audio: enabled
window: created
panic_scan: clean
exit_status: timeout_after_duration
EOF
