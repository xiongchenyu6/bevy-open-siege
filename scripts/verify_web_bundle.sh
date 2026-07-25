#!/usr/bin/env bash

set -euo pipefail

EXPECTED_VERSION=""
EXPECTED_COMMIT=""
EXPECTED_REF=""
LIST_FILES=()
MANIFEST_FILES=()

cleanup() {
  if [[ ${#LIST_FILES[@]} -gt 0 ]]; then
    rm -f -- "${LIST_FILES[@]}"
  fi
  if [[ ${#MANIFEST_FILES[@]} -gt 0 ]]; then
    rm -f -- "${MANIFEST_FILES[@]}"
  fi
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage:
  ./scripts/verify_web_bundle.sh <bundle-dir> [options]

Options:
  --expected-version <version>
  --expected-commit <40-character SHA>
  --expected-ref <git-ref>

Verifies both wasm-bindgen backends, bundle metadata, asset inventory, and every
file listed in SHA256SUMS. If Node.js or wasm-validate is available, the WASM
binaries are also parsed by that runtime/tool.
EOF
}

fail() {
  echo "web bundle verification failed: $*" >&2
  exit 1
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

BUNDLE_DIR="$(realpath -e "$1")"
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
    --expected-version)
      [[ $# -ge 2 ]] || fail "--expected-version requires a value"
      EXPECTED_VERSION="$2"
      shift 2
      ;;
    --expected-commit)
      [[ $# -ge 2 ]] || fail "--expected-commit requires a value"
      EXPECTED_COMMIT="$2"
      shift 2
      ;;
    --expected-ref)
      [[ $# -ge 2 ]] || fail "--expected-ref requires a value"
      EXPECTED_REF="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown option: $1"
      ;;
  esac
done

[[ -d "$BUNDLE_DIR" ]] || fail "bundle directory does not exist: $BUNDLE_DIR"
command -v sha256sum >/dev/null 2>&1 || fail "sha256sum is required"
command -v od >/dev/null 2>&1 || fail "od is required"

required_files=(
  "index.html"
  "favicon.png"
  "bevy_open_siege_webgl2.js"
  "bevy_open_siege_webgl2_bg.wasm"
  "bevy_open_siege_webgpu.js"
  "bevy_open_siege_webgpu_bg.wasm"
  "web-build-info.txt"
  "SHA256SUMS"
  "assets/manifest.ron"
  "assets/data/levels.ron"
  "assets/i18n/en.ron"
  "assets/i18n/zh.ron"
)

for relative_path in "${required_files[@]}"; do
  [[ -f "$BUNDLE_DIR/$relative_path" ]] || fail "missing required file: $relative_path"
  [[ -s "$BUNDLE_DIR/$relative_path" ]] || fail "required file is empty: $relative_path"
done

if find "$BUNDLE_DIR" -type l -print -quit | grep -q .; then
  fail "symbolic links are not allowed in the Pages artifact"
fi
if find "$BUNDLE_DIR" -mindepth 1 -name '.*' -print -quit | grep -q .; then
  fail "hidden files are not allowed because Pages artifact uploads omit them"
fi

actual_wasm="$(
  find "$BUNDLE_DIR" -maxdepth 1 -type f -name '*_bg.wasm' -printf '%f\n' \
    | LC_ALL=C sort
)"
expected_wasm=$'bevy_open_siege_webgl2_bg.wasm\nbevy_open_siege_webgpu_bg.wasm'
[[ "$actual_wasm" == "$expected_wasm" ]] || fail "bundle must contain exactly the WebGL2 and WebGPU WASM binaries"

for backend in webgl2 webgpu; do
  wasm_path="$BUNDLE_DIR/bevy_open_siege_${backend}_bg.wasm"
  js_path="$BUNDLE_DIR/bevy_open_siege_${backend}.js"
  wasm_magic="$(od -An -tx1 -N8 "$wasm_path" | tr -d '[:space:]')"
  wasm_size="$(wc -c < "$wasm_path" | tr -d '[:space:]')"
  js_size="$(wc -c < "$js_path" | tr -d '[:space:]')"

  [[ "$wasm_magic" == "0061736d01000000" ]] || fail "$backend binary has an invalid WASM header"
  (( wasm_size >= 1000000 )) || fail "$backend binary is unexpectedly small: $wasm_size bytes"
  (( js_size >= 1000 )) || fail "$backend JavaScript loader is unexpectedly small: $js_size bytes"

  if command -v wasm-validate >/dev/null 2>&1; then
    wasm-validate \
      --enable-bulk-memory \
      --enable-nontrapping-float-to-int \
      "$wasm_path"
  fi
done

backend_template="\${backend}"
grep -Fq "bevy_open_siege_${backend_template}.js" "$BUNDLE_DIR/index.html" \
  || fail "index.html does not select the backend JavaScript loader"
grep -Fq "bevy_open_siege_${backend_template}_bg.wasm" "$BUNDLE_DIR/index.html" \
  || fail "index.html does not select the backend WASM binary"
grep -Fq '"webgl2"' "$BUNDLE_DIR/index.html" || fail "index.html does not expose the WebGL2 fallback"
grep -Fq '"webgpu"' "$BUNDLE_DIR/index.html" || fail "index.html does not expose the WebGPU path"
grep -Fq 'return "webgl2";' "$BUNDLE_DIR/index.html" || fail "index.html does not default to WebGL2"

read_info() {
  local key="$1"
  local count
  local value

  count="$(grep -c "^${key}=" "$BUNDLE_DIR/web-build-info.txt" || true)"
  [[ "$count" == "1" ]] || fail "web-build-info.txt must contain exactly one $key entry"
  value="$(awk -F= -v key="$key" '$1 == key {sub(/^[^=]*=/, ""); print; exit}' "$BUNDLE_DIR/web-build-info.txt")"
  [[ -n "$value" ]] || fail "web-build-info.txt has an empty $key value"
  printf '%s\n' "$value"
}

product="$(read_info product)"
version="$(read_info version)"
source_commit="$(read_info source_commit)"
source_ref="$(read_info source_ref)"
backends="$(read_info backends)"
profile="$(read_info profile)"

[[ "$product" == "bevy_open_siege" ]] || fail "unexpected product metadata: $product"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] \
  || fail "invalid version metadata: $version"
[[ "$source_commit" =~ ^[0-9a-f]{40}$ ]] || fail "source_commit is not a full lowercase Git SHA"
[[ "$backends" == "webgl2,webgpu" ]] || fail "unexpected backend metadata: $backends"
[[ "$profile" == "web-release" ]] || fail "unexpected build profile: $profile"

if [[ -n "$EXPECTED_VERSION" && "$version" != "$EXPECTED_VERSION" ]]; then
  fail "version mismatch: expected $EXPECTED_VERSION, found $version"
fi
if [[ -n "$EXPECTED_COMMIT" && "$source_commit" != "$EXPECTED_COMMIT" ]]; then
  fail "commit mismatch: expected $EXPECTED_COMMIT, found $source_commit"
fi
if [[ -n "$EXPECTED_REF" && "$source_ref" != "$EXPECTED_REF" ]]; then
  fail "ref mismatch: expected $EXPECTED_REF, found $source_ref"
fi

listed_files="$(mktemp)"
actual_files="$(mktemp)"
LIST_FILES+=("$listed_files" "$actual_files")

while IFS= read -r checksum_line || [[ -n "$checksum_line" ]]; do
  checksum="${checksum_line:0:64}"
  separator="${checksum_line:64:2}"
  relative_path="${checksum_line:66}"

  [[ "$checksum" =~ ^[0-9a-f]{64}$ ]] || fail "malformed SHA256SUMS digest: $checksum_line"
  [[ "$separator" == "  " ]] || fail "malformed SHA256SUMS separator: $checksum_line"
  [[ "$relative_path" =~ ^\./[A-Za-z0-9_./-]+$ ]] \
    || fail "unsafe or unsupported SHA256SUMS path: $relative_path"
  [[ "$relative_path" != *"/../"* && "$relative_path" != "../"* ]] \
    || fail "SHA256SUMS path escapes the bundle: $relative_path"
  printf '%s\n' "$relative_path" >> "$listed_files"
done < "$BUNDLE_DIR/SHA256SUMS"

LC_ALL=C sort -o "$listed_files" "$listed_files"
if [[ -n "$(uniq -d "$listed_files")" ]]; then
  fail "SHA256SUMS contains duplicate paths"
fi

(
  cd "$BUNDLE_DIR"
  find . -type f ! -name SHA256SUMS -printf '%p\n' | LC_ALL=C sort
) > "$actual_files"

if ! cmp -s "$listed_files" "$actual_files"; then
  diff -u "$listed_files" "$actual_files" >&2 || true
  fail "SHA256SUMS does not cover every bundle file exactly once"
fi

(
  cd "$BUNDLE_DIR"
  sha256sum -c SHA256SUMS
)

if command -v node >/dev/null 2>&1; then
  node - \
    "$BUNDLE_DIR/bevy_open_siege_webgl2_bg.wasm" \
    "$BUNDLE_DIR/bevy_open_siege_webgpu_bg.wasm" <<'NODE'
const fs = require("node:fs");

for (const wasmPath of process.argv.slice(2)) {
  const bytes = fs.readFileSync(wasmPath);
  if (!WebAssembly.validate(bytes)) {
    throw new Error(`Node.js rejected WASM binary: ${wasmPath}`);
  }
}
NODE
fi

asset_count="$(find "$BUNDLE_DIR/assets" -type f | wc -l | tr -d '[:space:]')"
bundle_bytes="$(find "$BUNDLE_DIR" -type f -printf '%s\n' | awk '{total += $1} END {print total + 0}')"
echo "Web bundle verification passed"
echo "  version: $version"
echo "  source: $source_ref @ $source_commit"
echo "  backends: $backends"
echo "  assets: $asset_count"
echo "  bytes: $bundle_bytes"
