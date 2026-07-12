#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  linux_portability_smoke.sh <extracted-package-dir> [duration_seconds]

Runs the packaged Linux launcher in a sanitized environment without LD_LIBRARY_PATH
or Nix environment variables. This is not a substitute for clean-distro QA, but it
verifies the bundled runtime wrapper is not relying on the current shell's Nix
runtime paths.
EOF
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 2
fi

PACKAGE_DIR="$1"
DURATION_SECONDS="${2:-12}"
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "linux portability smoke package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi
if ! [[ "$DURATION_SECONDS" =~ ^[0-9]+$ ]] || [[ "$DURATION_SECONDS" -lt 3 ]]; then
  echo "linux portability smoke duration must be an integer of at least 3 seconds" >&2
  exit 2
fi

for file in bevy_open_siege bevy_open_siege.bin lib/ld-linux-x86-64.so.2; do
  if [[ ! -x "$PACKAGE_DIR/$file" ]]; then
    echo "linux portability smoke missing executable: $file" >&2
    exit 1
  fi
done

if grep -q "/nix/store" "$PACKAGE_DIR/bevy_open_siege"; then
  echo "linux portability smoke entrypoint should not reference /nix/store" >&2
  exit 1
fi
if strings "$PACKAGE_DIR/bevy_open_siege.bin" | grep -q "/nix/store"; then
  echo "linux portability smoke payload should not reference /nix/store" >&2
  exit 1
fi

TMPDIR_ROOT="$(mktemp -d)"
LOG_FILE="$TMPDIR_ROOT/runtime.log"
trap 'rm -rf "$TMPDIR_ROOT"' EXIT

SANITIZED_PATH="${BEVY_OPEN_SIEGE_SANITIZED_PATH:-/usr/bin:/bin:/run/current-system/sw/bin}"
ENV_ARGS=(
  -i
  "PATH=$SANITIZED_PATH"
  "HOME=$TMPDIR_ROOT/home"
  "TMPDIR=$TMPDIR_ROOT/tmp"
  "XDG_DATA_HOME=$TMPDIR_ROOT/xdg-data"
  "LANG=${LANG:-C.UTF-8}"
)
mkdir -p "$TMPDIR_ROOT/home" "$TMPDIR_ROOT/tmp" "$TMPDIR_ROOT/xdg-data"

for name in DISPLAY WAYLAND_DISPLAY XAUTHORITY XDG_RUNTIME_DIR; do
  if [[ -n "${!name:-}" ]]; then
    ENV_ARGS+=("$name=${!name}")
  fi
done

env "${ENV_ARGS[@]}" "$PACKAGE_DIR/bevy_open_siege" --validate-data >/dev/null

set +e
timeout "${DURATION_SECONDS}s" env "${ENV_ARGS[@]}" "$PACKAGE_DIR/bevy_open_siege" --no-audio > "$LOG_FILE" 2>&1
STATUS=$?
set -e

if [[ "$STATUS" -eq 0 ]]; then
  echo "linux portability smoke failed: game exited before ${DURATION_SECONDS}s" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi
if [[ "$STATUS" -ne 124 ]]; then
  echo "linux portability smoke failed: game exited with status $STATUS" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi
if grep -Eiq 'panicked at|Encountered a panic|thread .* panicked|error\[B0001\]' "$LOG_FILE"; then
  echo "linux portability smoke failed: panic signature found" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi
if ! grep -q "Creating new window Bevy Open Siege" "$LOG_FILE"; then
  echo "linux portability smoke failed: window creation log not found" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi

cat <<EOF
linux portability smoke ok
duration_seconds: ${DURATION_SECONDS}
entrypoint: bevy_open_siege
payload: bevy_open_siege.bin
sanitized_env: LD_LIBRARY_PATH unset, Nix variables omitted
path: ${SANITIZED_PATH}
window: created
panic_scan: clean
exit_status: timeout_after_duration
clean_distro_qa: still required
EOF
