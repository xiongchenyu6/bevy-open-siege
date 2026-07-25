#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RAW_OUTPUT="${1:-web-dist}"

if [[ "$RAW_OUTPUT" = /* ]]; then
  OUTPUT_DIR="$(realpath -m "$RAW_OUTPUT")"
else
  OUTPUT_DIR="$(realpath -m "$ROOT/$RAW_OUTPUT")"
fi

if [[ "$OUTPUT_DIR" == "/" || "$OUTPUT_DIR" == "$ROOT" ]]; then
  echo "refusing to replace unsafe web output directory: $OUTPUT_DIR" >&2
  exit 1
fi

for command_name in cargo wasm-bindgen sha256sum; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "$command_name is required to build the Web bundle" >&2
    exit 1
  fi
done

LOCKED_WASM_BINDGEN_VERSION="$(
  awk '
    $0 == "name = \"wasm-bindgen\"" { found = 1; next }
    found && /^version = / {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' "$ROOT/Cargo.lock"
)"
INSTALLED_WASM_BINDGEN_VERSION="$(wasm-bindgen --version | awk '{print $2}')"

if [[ -z "$LOCKED_WASM_BINDGEN_VERSION" ]]; then
  echo "could not resolve wasm-bindgen version from Cargo.lock" >&2
  exit 1
fi
if [[ "$INSTALLED_WASM_BINDGEN_VERSION" != "$LOCKED_WASM_BINDGEN_VERSION" ]]; then
  echo "wasm-bindgen version mismatch: Cargo.lock=$LOCKED_WASM_BINDGEN_VERSION installed=$INSTALLED_WASM_BINDGEN_VERSION" >&2
  exit 1
fi

VERSION="${WEB_BUILD_VERSION:-$(awk -F'"' '/^version = / {print $2; exit}' "$ROOT/Cargo.toml")}"
SOURCE_COMMIT="${WEB_BUILD_COMMIT:-$(git -C "$ROOT" rev-parse HEAD)}"
SOURCE_REF="${WEB_BUILD_REF:-$(git -C "$ROOT" describe --tags --exact-match 2>/dev/null || git -C "$ROOT" branch --show-current)}"
WASM_OPT="${WASM_OPT_BIN:-$(command -v wasm-opt || true)}"
REQUIRE_WASM_OPT="${WEB_REQUIRE_WASM_OPT:-0}"

[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] \
  || { echo "invalid WEB_BUILD_VERSION: $VERSION" >&2; exit 1; }
[[ "$SOURCE_COMMIT" =~ ^[0-9a-f]{40}$ ]] \
  || { echo "WEB_BUILD_COMMIT must be a full lowercase Git SHA: $SOURCE_COMMIT" >&2; exit 1; }
[[ -n "$SOURCE_REF" && "$SOURCE_REF" != *$'\n'* ]] \
  || { echo "WEB_BUILD_REF must be a non-empty single line" >&2; exit 1; }

if [[ "$REQUIRE_WASM_OPT" == "1" && -z "$WASM_OPT" ]]; then
  echo "WEB_REQUIRE_WASM_OPT=1 but wasm-opt was not found" >&2
  exit 1
fi
if [[ -n "$WASM_OPT" && ! -x "$WASM_OPT" ]]; then
  echo "WASM_OPT_BIN is not executable: $WASM_OPT" >&2
  exit 1
fi

rm -rf -- "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

cd "$ROOT"

echo "Building WebGL2 WASM"
cargo build --locked --profile web-release --target wasm32-unknown-unknown
wasm-bindgen \
  --no-typescript \
  --target web \
  --out-dir "$OUTPUT_DIR" \
  --out-name bevy_open_siege_webgl2 \
  target/wasm32-unknown-unknown/web-release/bevy_open_siege.wasm

echo "Building WebGPU WASM"
cargo build --locked --profile web-release --target wasm32-unknown-unknown --features webgpu
wasm-bindgen \
  --no-typescript \
  --target web \
  --out-dir "$OUTPUT_DIR" \
  --out-name bevy_open_siege_webgpu \
  target/wasm32-unknown-unknown/web-release/bevy_open_siege.wasm

optimizer_info="skipped"
if [[ -n "$WASM_OPT" ]]; then
  optimizer_info="$("$WASM_OPT" --version | head -n 1)"
  for wasm_path in "$OUTPUT_DIR"/*_bg.wasm; do
    before_bytes="$(wc -c < "$wasm_path" | tr -d '[:space:]')"
    "$WASM_OPT" \
      -Oz \
      --enable-bulk-memory \
      --enable-nontrapping-float-to-int \
      "$wasm_path" \
      -o "$wasm_path.opt"
    mv "$wasm_path.opt" "$wasm_path"
    after_bytes="$(wc -c < "$wasm_path" | tr -d '[:space:]')"
    echo "$(basename "$wasm_path"): $before_bytes -> $after_bytes bytes"
  done
fi

install -m 0644 "$ROOT/web/index.html" "$ROOT/web/favicon.png" "$OUTPUT_DIR/"
cp -R "$ROOT/assets" "$OUTPUT_DIR/assets"

{
  printf 'product=bevy_open_siege\n'
  printf 'version=%s\n' "$VERSION"
  printf 'source_commit=%s\n' "$SOURCE_COMMIT"
  printf 'source_ref=%s\n' "$SOURCE_REF"
  printf 'profile=web-release\n'
  printf 'backends=webgl2,webgpu\n'
  printf 'wasm_bindgen=%s\n' "$LOCKED_WASM_BINDGEN_VERSION"
  printf 'wasm_optimizer=%s\n' "$optimizer_info"
} > "$OUTPUT_DIR/web-build-info.txt"

(
  cd "$OUTPUT_DIR"
  while IFS= read -r -d '' file_path; do
    sha256sum "$file_path"
  done < <(find . -type f ! -name SHA256SUMS -print0 | LC_ALL=C sort -z)
) > "$OUTPUT_DIR/SHA256SUMS"

"$ROOT/scripts/verify_web_bundle.sh" \
  "$OUTPUT_DIR" \
  --expected-version "$VERSION" \
  --expected-commit "$SOURCE_COMMIT" \
  --expected-ref "$SOURCE_REF"

echo "Web bundle assembled at $OUTPUT_DIR"
