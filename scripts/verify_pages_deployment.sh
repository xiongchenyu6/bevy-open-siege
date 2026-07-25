#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
  ./scripts/verify_pages_deployment.sh <pages-base-url> <expected-bundle-dir>

Downloads every deployed file, compares it with the expected bundle's
SHA256SUMS, checks runtime MIME types, and reruns the local bundle verifier.
EOF
}

fail() {
  echo "GitHub Pages verification failed: $*" >&2
  exit 1
}

if [[ $# -ne 2 ]]; then
  usage
  exit 1
fi

BASE_URL="${1%/}/"
EXPECTED_DIR="$(realpath -e "$2")"
MANIFEST_ATTEMPTS="${PAGES_VERIFY_ATTEMPTS:-24}"
FILE_ATTEMPTS="${PAGES_FILE_VERIFY_ATTEMPTS:-6}"
RETRY_DELAY="${PAGES_VERIFY_RETRY_DELAY:-5}"
QUERY_TOKEN="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-1}-$(date +%s)-${RANDOM}"

[[ "$BASE_URL" != *"?"* && "$BASE_URL" != *"#"* ]] || fail "base URL must not contain a query or fragment"
if [[ ! "$BASE_URL" =~ ^https:// && ! "$BASE_URL" =~ ^http://(127\.0\.0\.1|localhost)(:[0-9]+)?/ ]]; then
  fail "base URL must use HTTPS, except localhost integration tests"
fi
[[ "$MANIFEST_ATTEMPTS" =~ ^[1-9][0-9]*$ ]] || fail "PAGES_VERIFY_ATTEMPTS must be a positive integer"
[[ "$FILE_ATTEMPTS" =~ ^[1-9][0-9]*$ ]] || fail "PAGES_FILE_VERIFY_ATTEMPTS must be a positive integer"
[[ "$RETRY_DELAY" =~ ^[0-9]+$ ]] || fail "PAGES_VERIFY_RETRY_DELAY must be a non-negative integer"

for command_name in curl sha256sum; do
  command -v "$command_name" >/dev/null 2>&1 || fail "$command_name is required"
done

"$ROOT/scripts/verify_web_bundle.sh" "$EXPECTED_DIR"

TMP_DIR="$(mktemp -d)"
REMOTE_DIR="$TMP_DIR/remote"
mkdir -p "$REMOTE_DIR"

fetch_url() {
  local relative_path="$1"
  local output_path="$2"

  curl \
    --fail \
    --silent \
    --show-error \
    --location \
    --connect-timeout 20 \
    --max-time 300 \
    --header "Cache-Control: no-cache" \
    --output "$output_path" \
    "${BASE_URL}${relative_path}?release_verifier=${QUERY_TOKEN}"
}

manifest_ready="0"
for ((attempt = 1; attempt <= MANIFEST_ATTEMPTS; attempt++)); do
  candidate_manifest="$TMP_DIR/SHA256SUMS.candidate"
  if fetch_url "SHA256SUMS" "$candidate_manifest" \
    && cmp -s "$EXPECTED_DIR/SHA256SUMS" "$candidate_manifest"; then
    cp "$candidate_manifest" "$REMOTE_DIR/SHA256SUMS"
    manifest_ready="1"
    break
  fi

  echo "Waiting for the expected Pages manifest ($attempt/$MANIFEST_ATTEMPTS)"
  if (( attempt < MANIFEST_ATTEMPTS )); then
    sleep "$RETRY_DELAY"
  fi
done

[[ "$manifest_ready" == "1" ]] || fail "deployed SHA256SUMS never matched the expected release bundle"

verified_count=0
verified_bytes=0
while IFS= read -r checksum_line || [[ -n "$checksum_line" ]]; do
  expected_checksum="${checksum_line:0:64}"
  manifest_path="${checksum_line:66}"
  relative_path="${manifest_path#./}"
  destination="$REMOTE_DIR/$relative_path"
  mkdir -p "$(dirname "$destination")"

  file_verified="0"
  for ((attempt = 1; attempt <= FILE_ATTEMPTS; attempt++)); do
    if fetch_url "$relative_path" "$destination"; then
      actual_checksum="$(sha256sum "$destination" | awk '{print $1}')"
      if [[ "$actual_checksum" == "$expected_checksum" ]]; then
        file_verified="1"
        break
      fi
    fi

    if (( attempt < FILE_ATTEMPTS )); then
      sleep "$RETRY_DELAY"
    fi
  done

  [[ "$file_verified" == "1" ]] || fail "deployed file does not match release bundle: $relative_path"
  ((verified_count += 1))
  file_bytes="$(wc -c < "$destination" | tr -d '[:space:]')"
  ((verified_bytes += file_bytes))
done < "$EXPECTED_DIR/SHA256SUMS"

assert_content_type() {
  local relative_path="$1"
  local expected_pattern="$2"
  local headers
  local content_type

  headers="$(
    curl \
      --fail \
      --silent \
      --show-error \
      --location \
      --head \
      --header "Cache-Control: no-cache" \
      "${BASE_URL}${relative_path}?release_verifier=${QUERY_TOKEN}"
  )"
  content_type="$(
    printf '%s\n' "$headers" \
      | tr -d '\r' \
      | awk -F': *' 'tolower($1) == "content-type" {value = tolower($2)} END {print value}'
  )"

  [[ "$content_type" =~ $expected_pattern ]] \
    || fail "$relative_path has unexpected Content-Type: ${content_type:-missing}"
}

assert_content_type "index.html" '^text/html([;]|$)'
assert_content_type "bevy_open_siege_webgl2.js" '^(application|text)/(javascript|x-javascript)([;]|$)'
assert_content_type "bevy_open_siege_webgpu.js" '^(application|text)/(javascript|x-javascript)([;]|$)'
assert_content_type "bevy_open_siege_webgl2_bg.wasm" '^application/wasm([;]|$)'
assert_content_type "bevy_open_siege_webgpu_bg.wasm" '^application/wasm([;]|$)'

"$ROOT/scripts/verify_web_bundle.sh" "$REMOTE_DIR"

echo "GitHub Pages verification passed"
echo "  URL: $BASE_URL"
echo "  files: $verified_count"
echo "  bytes: $verified_bytes"
