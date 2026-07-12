#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: support_diagnostics.sh [package-dir] [output-dir]

Creates a local diagnostics folder with release metadata, integrity checks, save-path
summary, and package verification output. It does not copy save files, screenshots,
recordings, or unrelated personal files.
EOF
}

PACKAGE_DIR="${1:-.}"
OUTPUT_DIR="${2:-support-diagnostics}"
if [[ $# -gt 2 ]]; then
  usage
  exit 2
fi
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
PACKAGE_DIR="$(cd "$PACKAGE_DIR" && pwd)"
OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"

if [[ -x "$PACKAGE_DIR/bevy_open_siege.exe" ]]; then
  BINARY="$PACKAGE_DIR/bevy_open_siege.exe"
elif [[ -x "$PACKAGE_DIR/bevy_open_siege" ]]; then
  BINARY="$PACKAGE_DIR/bevy_open_siege"
else
  echo "release package binary is not executable" >&2
  exit 1
fi

run_capture() {
  local name="$1"
  shift
  {
    echo "$ $*"
    "$@"
  } > "$OUTPUT_DIR/$name" 2>&1 || {
    local status=$?
    echo "command failed with status $status" >> "$OUTPUT_DIR/$name"
    return "$status"
  }
}

run_optional_capture() {
  local name="$1"
  shift
  {
    echo "$ $*"
    "$@"
  } > "$OUTPUT_DIR/$name" 2>&1 || {
    local status=$?
    echo "command failed with status $status" >> "$OUTPUT_DIR/$name"
  }
}

{
  echo "Bevy Open Siege support diagnostics"
  echo "package_dir: $PACKAGE_DIR"
  echo "binary: $BINARY"
  date -u +"utc: %Y-%m-%dT%H:%M:%SZ"
  uname -a 2>/dev/null || true
  echo "shell: ${SHELL:-unknown}"
  echo "xdg_data_home: ${XDG_DATA_HOME:-unset}"
  echo "xdg_bin_home: ${XDG_BIN_HOME:-unset}"
  echo "audio_env: ${BEVY_OPEN_SIEGE_AUDIO:-unset}"
  echo "save_override: ${BEVY_OPEN_SIEGE_SAVE_PATH:-unset}"
  echo "privacy: no save files, screenshots, recordings, or personal files are copied by this script"
} > "$OUTPUT_DIR/environment.txt"

run_capture "release-info.txt" "$BINARY" --print-release-info
run_capture "validate-data.txt" "$BINARY" --validate-data
run_capture "save-path.txt" "$BINARY" --print-save-path
run_optional_capture "save-summary.txt" "$BINARY" --print-save-summary
run_capture "privacy-audit.txt" "$BINARY" --audit-privacy

if [[ -f "$PACKAGE_DIR/SHA256SUMS" ]]; then
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$PACKAGE_DIR" && run_capture "sha256sum.txt" sha256sum -c SHA256SUMS)
  elif command -v shasum >/dev/null 2>&1; then
    (cd "$PACKAGE_DIR" && run_capture "sha256sum.txt" shasum -a 256 -c SHA256SUMS)
  else
    echo "sha256sum or shasum not available" > "$OUTPUT_DIR/sha256sum.txt"
  fi
else
  echo "SHA256SUMS not found" > "$OUTPUT_DIR/sha256sum.txt"
fi

if [[ "${BEVY_OPEN_SIEGE_DIAGNOSTICS_SKIP_VERIFY:-0}" == "1" ]]; then
  echo "quick verification skipped by BEVY_OPEN_SIEGE_DIAGNOSTICS_SKIP_VERIFY=1" > "$OUTPUT_DIR/verify-release-quick.txt"
elif [[ -x "$PACKAGE_DIR/verify_release.sh" ]]; then
  run_optional_capture "verify-release-quick.txt" "$PACKAGE_DIR/verify_release.sh" --quick "$PACKAGE_DIR"
else
  echo "verify_release.sh not found or not executable" > "$OUTPUT_DIR/verify-release-quick.txt"
fi

if [[ -f "$PACKAGE_DIR/release-manifest.json" ]]; then
  cp "$PACKAGE_DIR/release-manifest.json" "$OUTPUT_DIR/release-manifest.json"
else
  echo "release-manifest.json not found" > "$OUTPUT_DIR/release-manifest.json"
fi

find "$PACKAGE_DIR" -maxdepth 2 -type f \
  | sed "s#^$PACKAGE_DIR/##" \
  | sort > "$OUTPUT_DIR/package-files.txt"

cat > "$OUTPUT_DIR/README.txt" <<'EOF'
Attach this diagnostics folder only if you intentionally choose to share it.
It contains command output and package metadata. It does not include save files,
screenshots, recordings, crash dumps, or unrelated personal files.

If the issue depends on save state, include the save file separately only after
reviewing it yourself.
EOF

echo "support diagnostics collected"
echo "output: $OUTPUT_DIR"
