#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <bevy_open_siege_binary> [duration_seconds]" >&2
  exit 2
fi

BINARY="$1"
DURATION_SECONDS="${2:-${BEVY_OPEN_SIEGE_RUNTIME_SMOKE_SECONDS:-12}}"
STARTUP_TIMEOUT_SECONDS="${BEVY_OPEN_SIEGE_RUNTIME_STARTUP_TIMEOUT_SECONDS:-30}"
POST_WINDOW_STABILITY_SECONDS="${BEVY_OPEN_SIEGE_RUNTIME_STABILITY_SECONDS:-3}"

if [[ ! -x "$BINARY" ]]; then
  echo "runtime smoke binary is not executable: $BINARY" >&2
  exit 1
fi
if ! [[ "$DURATION_SECONDS" =~ ^[0-9]+$ ]] || [[ "$DURATION_SECONDS" -lt 3 ]]; then
  echo "runtime smoke duration must be an integer of at least 3 seconds" >&2
  exit 2
fi
if ! [[ "$STARTUP_TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || [[ "$STARTUP_TIMEOUT_SECONDS" -lt 5 ]]; then
  echo "runtime smoke startup timeout must be an integer of at least 5 seconds" >&2
  exit 2
fi
if ! [[ "$POST_WINDOW_STABILITY_SECONDS" =~ ^[0-9]+$ ]] || [[ "$POST_WINDOW_STABILITY_SECONDS" -lt 1 ]]; then
  echo "runtime smoke stability duration must be an integer of at least 1 second" >&2
  exit 2
fi
if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
  export WINIT_UNIX_BACKEND="${WINIT_UNIX_BACKEND:-wayland}"
fi

LOG_FILE="$(mktemp)"
PID=""
cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" >/dev/null 2>&1; then
    kill "$PID" >/dev/null 2>&1 || true
    for _ in {1..20}; do
      if ! kill -0 "$PID" >/dev/null 2>&1; then
        break
      fi
      sleep 0.1
    done
    if kill -0 "$PID" >/dev/null 2>&1; then
      kill -KILL "$PID" >/dev/null 2>&1 || true
    fi
    wait "$PID" >/dev/null 2>&1 || true
  fi
  rm -f "$LOG_FILE"
}
trap cleanup EXIT

panic_found() {
  grep -Eiq 'panicked at|Encountered a panic|thread .* panicked|error\[B0001\]' "$LOG_FILE"
}

report_early_exit() {
  local status=0
  wait "$PID" || status=$?
  PID=""
  echo "runtime smoke failed: game exited with status $status" >&2
  cat "$LOG_FILE" >&2
  exit 1
}

"$BINARY" --no-audio > "$LOG_FILE" 2>&1 &
PID=$!
START_SECONDS=$SECONDS
STARTUP_DEADLINE=$((START_SECONDS + STARTUP_TIMEOUT_SECONDS))

while [[ "$SECONDS" -lt "$STARTUP_DEADLINE" ]]; do
  if panic_found; then
    echo "runtime smoke failed: panic signature found during startup" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi
  if grep -q "Creating new window Bevy Open Siege" "$LOG_FILE"; then
    break
  fi
  if ! kill -0 "$PID" >/dev/null 2>&1; then
    report_early_exit
  fi
  sleep 0.25
done

if ! grep -q "Creating new window Bevy Open Siege" "$LOG_FILE"; then
  echo "runtime smoke failed: window creation log not found within ${STARTUP_TIMEOUT_SECONDS}s" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi

FINISH_DEADLINE=$((START_SECONDS + DURATION_SECONDS))
STABILITY_DEADLINE=$((SECONDS + POST_WINDOW_STABILITY_SECONDS))
if [[ "$STABILITY_DEADLINE" -gt "$FINISH_DEADLINE" ]]; then
  FINISH_DEADLINE="$STABILITY_DEADLINE"
fi
while [[ "$SECONDS" -lt "$FINISH_DEADLINE" ]]; do
  if panic_found; then
    echo "runtime smoke failed: panic signature found after window creation" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi
  if ! kill -0 "$PID" >/dev/null 2>&1; then
    report_early_exit
  fi
  sleep 0.25
done

if panic_found; then
  echo "runtime smoke failed: panic signature found" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi
if ! kill -0 "$PID" >/dev/null 2>&1; then
  report_early_exit
fi

cat <<EOF
runtime startup smoke ok
duration_seconds: ${DURATION_SECONDS}
startup_timeout_seconds: ${STARTUP_TIMEOUT_SECONDS}
post_window_stability_seconds: ${POST_WINDOW_STABILITY_SECONDS}
audio: disabled
window: created
panic_scan: clean
exit_status: killed_after_stability_window
EOF
