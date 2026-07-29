#!/usr/bin/env bash

set -euo pipefail

TMP_DIR=""

cleanup() {
  if [[ -n "${TMP_DIR:-}" && -d "$TMP_DIR" ]]; then
    rm -rf -- "$TMP_DIR"
  fi
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage:
  ./scripts/verify_platform_archive.sh <archive> <platform>

Platforms:
  linux-x86_64
  windows-x86_64
  macos-universal

Verifies archive paths, package layout, release metadata, the internal
SHA256SUMS inventory, and binary format without executing the package.
EOF
}

fail() {
  echo "platform archive verification failed: $*" >&2
  exit 1
}

if [[ $# -ne 2 ]]; then
  usage
  exit 2
fi

ARCHIVE="$(realpath -e "$1")"
PLATFORM="$2"
ARCHIVE_NAME="$(basename "$ARCHIVE")"

case "$PLATFORM" in
  linux-x86_64)
    PACKAGE_NAME="${ARCHIVE_NAME%.tar.gz}"
    [[ "$ARCHIVE_NAME" == "$PACKAGE_NAME.tar.gz" ]] || fail "Linux archive must use .tar.gz"
    [[ "$PACKAGE_NAME" == bevy_open_siege-*-linux-x86_64 ]] || fail "unexpected Linux archive name: $ARCHIVE_NAME"
    ARCHIVE_KIND="tar"
    BINARY_PATH="bevy_open_siege.bin"
    BINARY_MAGIC="7f454c46"
    ;;
  windows-x86_64)
    PACKAGE_NAME="${ARCHIVE_NAME%.zip}"
    [[ "$ARCHIVE_NAME" == "$PACKAGE_NAME.zip" ]] || fail "Windows archive must use .zip"
    [[ "$PACKAGE_NAME" == bevy_open_siege-*-windows-x86_64 ]] || fail "unexpected Windows archive name: $ARCHIVE_NAME"
    ARCHIVE_KIND="zip"
    BINARY_PATH="bevy_open_siege.exe"
    BINARY_MAGIC="4d5a"
    ;;
  macos-universal)
    PACKAGE_NAME="${ARCHIVE_NAME%.tar.gz}"
    [[ "$ARCHIVE_NAME" == "$PACKAGE_NAME.tar.gz" ]] || fail "macOS archive must use .tar.gz"
    [[ "$PACKAGE_NAME" == bevy_open_siege-*-macos-universal ]] || fail "unexpected macOS archive name: $ARCHIVE_NAME"
    ARCHIVE_KIND="tar"
    BINARY_PATH="bevy_open_siege"
    BINARY_MAGIC="cafebabe"
    ;;
  *)
    fail "unsupported platform: $PLATFORM"
    ;;
esac

for command_name in find jq od realpath sha256sum sort; do
  command -v "$command_name" >/dev/null 2>&1 || fail "$command_name is required"
done
if [[ "$ARCHIVE_KIND" == "tar" ]]; then
  command -v tar >/dev/null 2>&1 || fail "tar is required"
else
  command -v unzip >/dev/null 2>&1 || fail "unzip is required"
fi

TMP_DIR="$(mktemp -d)"
ARCHIVE_LIST="$TMP_DIR/archive-files.txt"
EXTRACT_DIR="$TMP_DIR/extracted"
mkdir -p "$EXTRACT_DIR"

if [[ "$ARCHIVE_KIND" == "tar" ]]; then
  tar -tzf "$ARCHIVE" > "$ARCHIVE_LIST"
else
  unzip -Z1 "$ARCHIVE" > "$ARCHIVE_LIST"
fi

[[ -s "$ARCHIVE_LIST" ]] || fail "archive is empty"
if grep -Eq '(^/|(^|/)\.\.(/|$)|\\)' "$ARCHIVE_LIST"; then
  fail "archive contains an unsafe path"
fi

while IFS= read -r archive_path || [[ -n "$archive_path" ]]; do
  normalized="${archive_path#./}"
  [[ "$normalized" == "$PACKAGE_NAME" || "$normalized" == "$PACKAGE_NAME/"* ]] \
    || fail "archive entry is outside expected root $PACKAGE_NAME: $archive_path"
done < "$ARCHIVE_LIST"

if [[ "$ARCHIVE_KIND" == "tar" ]]; then
  tar -xzf "$ARCHIVE" -C "$EXTRACT_DIR"
else
  unzip -q "$ARCHIVE" -d "$EXTRACT_DIR"
fi

PACKAGE_DIR="$EXTRACT_DIR/$PACKAGE_NAME"
[[ -d "$PACKAGE_DIR" ]] || fail "archive did not contain package root $PACKAGE_NAME"
if find "$PACKAGE_DIR" -type l -print -quit | grep -q .; then
  fail "symbolic links are not allowed in release packages"
fi

required_files=(
  "README.md"
  "LICENSE"
  "RELEASE_NOTES.md"
  "VERSION.ron"
  "release-info.txt"
  "release-readiness.txt"
  "release-manifest.json"
  "SHA256SUMS"
  "runtime-smoke.txt"
  "visual-smoke.txt"
  "audio-smoke.txt"
  "assets/manifest.ron"
  "assets/data/levels.ron"
  "assets/i18n/en.ron"
  "assets/i18n/zh.ron"
  "assets/models/plants/sprout-slinger.glb"
  "assets/models/monsters/walker.glb"
  "$BINARY_PATH"
)

for relative_path in "${required_files[@]}"; do
  [[ -s "$PACKAGE_DIR/$relative_path" ]] || fail "missing or empty package file: $relative_path"
done

if [[ "$PLATFORM" == "linux-x86_64" ]]; then
  [[ -x "$PACKAGE_DIR/bevy_open_siege" ]] || fail "Linux launcher is not executable"
  [[ -x "$PACKAGE_DIR/bevy_open_siege.bin" ]] || fail "Linux binary is not executable"
  [[ -s "$PACKAGE_DIR/lib/ld-linux-x86-64.so.2" ]] || fail "Linux loader is missing"
elif [[ "$PLATFORM" == "macos-universal" ]]; then
  [[ -x "$PACKAGE_DIR/bevy_open_siege" ]] || fail "macOS binary is not executable"
fi

actual_magic="$(od -An -tx1 -N$(( ${#BINARY_MAGIC} / 2 )) "$PACKAGE_DIR/$BINARY_PATH" | tr -d '[:space:]')"
[[ "$actual_magic" == "$BINARY_MAGIC" ]] \
  || fail "$BINARY_PATH has unexpected binary magic: $actual_magic"

jq -e \
  --arg package "$PACKAGE_NAME" \
  --arg platform "$PLATFORM" \
  '
    .schema == "bevy-open-siege-release-manifest-v1"
    and .product == "Bevy Open Siege"
    and .package == $package
    and .platform == $platform
    and .qa.status == "release_candidate"
    and .qa.final_approval_required == true
    and .qa.missing_evidence == []
    and .integrity.hash_file == "SHA256SUMS"
  ' "$PACKAGE_DIR/release-manifest.json" >/dev/null \
  || fail "release-manifest.json metadata is inconsistent"

listed_files="$TMP_DIR/listed-files.txt"
actual_files="$TMP_DIR/actual-files.txt"
while IFS= read -r checksum_line || [[ -n "$checksum_line" ]]; do
  # PowerShell writes the Windows manifest with CRLF line endings.
  checksum_line="${checksum_line%$'\r'}"
  checksum="${checksum_line:0:64}"
  separator="${checksum_line:64:2}"
  relative_path="${checksum_line:66}"

  [[ "$checksum" =~ ^[0-9a-f]{64}$ ]] || fail "malformed SHA256SUMS digest"
  [[ "$separator" == "  " ]] || fail "malformed SHA256SUMS separator"
  [[ -n "$relative_path" && "$relative_path" != /* && "$relative_path" != *\\* ]] \
    || fail "unsafe SHA256SUMS path: $relative_path"
  if [[ "/$relative_path/" == *"/../"* ]]; then
    fail "SHA256SUMS path escapes the package: $relative_path"
  fi
  printf '%s\n' "$relative_path" >> "$listed_files"
done < "$PACKAGE_DIR/SHA256SUMS"

LC_ALL=C sort -o "$listed_files" "$listed_files"
if [[ -n "$(uniq -d "$listed_files")" ]]; then
  fail "SHA256SUMS contains duplicate paths"
fi

(
  cd "$PACKAGE_DIR"
  find . -type f ! -name SHA256SUMS -printf '%P\n' | LC_ALL=C sort
) > "$actual_files"
if ! cmp -s "$listed_files" "$actual_files"; then
  diff -u "$listed_files" "$actual_files" >&2 || true
  fail "SHA256SUMS does not cover every package file exactly once"
fi

(
  cd "$PACKAGE_DIR"
  sha256sum -c SHA256SUMS >/dev/null
) || fail "internal SHA256SUMS verification failed"

file_count="$(wc -l < "$actual_files" | tr -d '[:space:]')"
package_bytes="$(find "$PACKAGE_DIR" -type f -printf '%s\n' | awk '{total += $1} END {print total + 0}')"
echo "Platform archive verification passed"
echo "  platform: $PLATFORM"
echo "  package: $PACKAGE_NAME"
echo "  files: $file_count"
echo "  bytes: $package_bytes"
