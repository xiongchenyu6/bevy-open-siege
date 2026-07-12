#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <bevy_open_siege_binary> [duration_seconds]" >&2
  exit 2
fi

BINARY="$1"
DURATION_SECONDS="${2:-${BEVY_OPEN_SIEGE_VISUAL_SMOKE_SECONDS:-15}}"

if [[ ! -x "$BINARY" ]]; then
  echo "visual smoke binary is not executable: $BINARY" >&2
  exit 1
fi
if ! [[ "$DURATION_SECONDS" =~ ^[0-9]+$ ]] || [[ "$DURATION_SECONDS" -lt 4 ]]; then
  echo "visual smoke duration must be an integer of at least 4 seconds" >&2
  exit 2
fi
if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  echo "visual smoke requires DISPLAY or WAYLAND_DISPLAY for screenshot capture" >&2
  exit 1
fi
if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
  export WINIT_UNIX_BACKEND="${WINIT_UNIX_BACKEND:-wayland}"
fi
CAPTURE_BACKEND="x11"
if [[ "${WINIT_UNIX_BACKEND:-}" == "wayland" && -n "${WAYLAND_DISPLAY:-}" ]]; then
  CAPTURE_BACKEND="wayland"
fi
if [[ "$CAPTURE_BACKEND" == "wayland" ]]; then
  if ! command -v grim >/dev/null 2>&1 || ! command -v magick >/dev/null 2>&1; then
    echo "visual smoke requires grim and magick commands for Wayland screenshot capture" >&2
    exit 1
  fi
elif ! command -v import >/dev/null 2>&1 || ! command -v magick >/dev/null 2>&1; then
  echo "visual smoke requires ImageMagick import and magick commands for X11 screenshot capture" >&2
  exit 1
fi

LOG_FILE="$(mktemp)"
SCREENSHOT_FILE="$(mktemp --suffix=.png)"
PID=""
cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" >/dev/null 2>&1; then
    kill "$PID" >/dev/null 2>&1 || true
    wait "$PID" >/dev/null 2>&1 || true
  fi
  rm -f "$LOG_FILE" "$SCREENSHOT_FILE"
}
trap cleanup EXIT

"$BINARY" --no-audio > "$LOG_FILE" 2>&1 &
PID=$!

deadline=$((SECONDS + DURATION_SECONDS))
while [[ "$SECONDS" -lt "$deadline" ]]; do
  if ! kill -0 "$PID" >/dev/null 2>&1; then
    echo "visual smoke failed: game exited before screenshot capture" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi
  if grep -q "Creating new window Bevy Open Siege" "$LOG_FILE"; then
    break
  fi
  sleep 0.25
done

if ! grep -q "Creating new window Bevy Open Siege" "$LOG_FILE"; then
  echo "visual smoke failed: window creation log not found within ${DURATION_SECONDS}s" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi

sleep 1
if [[ "$CAPTURE_BACKEND" == "wayland" ]]; then
  grim "$SCREENSHOT_FILE"
else
  import -window "Bevy Open Siege" "$SCREENSHOT_FILE"
fi

if grep -Eiq 'panicked at|Encountered a panic|thread .* panicked|error\[B0001\]' "$LOG_FILE"; then
  echo "visual smoke failed: panic signature found" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi
if [[ ! -s "$SCREENSHOT_FILE" ]]; then
  echo "visual smoke failed: screenshot was not captured" >&2
  exit 1
fi

stddev="$(magick "$SCREENSHOT_FILE" -colorspace Gray -format '%[fx:standard_deviation]' info:)"
if ! awk -v value="$stddev" 'BEGIN { exit !(value > 0.005) }'; then
  echo "visual smoke failed: screenshot appears blank or nearly solid" >&2
  echo "screenshot_stddev: $stddev" >&2
  exit 1
fi

cat <<EOF
visual startup smoke ok
duration_seconds: ${DURATION_SECONDS}
audio: disabled
window: created
screenshot: nonblank
capture_backend: ${CAPTURE_BACKEND}
panic_scan: clean
exit_status: killed_after_capture
EOF
