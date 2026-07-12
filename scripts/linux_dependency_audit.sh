#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <extracted-package-dir>" >&2
  exit 2
fi

PACKAGE_DIR="$1"
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "linux dependency audit package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

BINARY="$PACKAGE_DIR/bevy_open_siege"
PAYLOAD="$BINARY"
WRAPPER_MODE="no"
if [[ ! -x "$BINARY" ]]; then
  echo "linux dependency audit missing executable entrypoint: bevy_open_siege" >&2
  exit 1
fi

if ! command -v file >/dev/null 2>&1; then
  echo "linux dependency audit requires file" >&2
  exit 1
fi
if ! command -v ldd >/dev/null 2>&1; then
  echo "linux dependency audit requires ldd" >&2
  exit 1
fi

ENTRYPOINT_SUMMARY="$(file -b "$BINARY" | sed 's/, BuildID\\[[^]]*\\]=[^,]*//')"
if [[ "$ENTRYPOINT_SUMMARY" == *"shell script"* || "$ENTRYPOINT_SUMMARY" == *"text executable"* ]]; then
  WRAPPER_MODE="yes"
  PAYLOAD="$PACKAGE_DIR/bevy_open_siege.bin"
  if [[ ! -x "$PAYLOAD" ]]; then
    echo "linux dependency audit wrapper entrypoint is missing payload: bevy_open_siege.bin" >&2
    exit 1
  fi
  if [[ ! -x "$PACKAGE_DIR/lib/ld-linux-x86-64.so.2" ]]; then
    echo "linux dependency audit wrapper entrypoint is missing bundled loader: lib/ld-linux-x86-64.so.2" >&2
    exit 1
  fi
  grep -q "ld-linux-x86-64.so.2" "$BINARY"
  grep -q "bevy_open_siege.bin" "$BINARY"
fi

FILE_SUMMARY="$(file -b "$PAYLOAD" | sed 's/, BuildID\\[[^]]*\\]=[^,]*//')"
if [[ "$FILE_SUMMARY" != *"ELF 64-bit"* ]]; then
  echo "linux dependency audit expected an ELF 64-bit binary: $FILE_SUMMARY" >&2
  exit 1
fi

INTERPRETER="unknown"
if command -v readelf >/dev/null 2>&1; then
  INTERPRETER="$(readelf -l "$PAYLOAD" \
    | sed -n 's/.*Requesting program interpreter: \(.*\)]/\1/p' \
    | head -n 1)"
  INTERPRETER="${INTERPRETER:-unknown}"
fi

if [[ "$WRAPPER_MODE" == "yes" ]]; then
  LDD_OUTPUT="$("$PACKAGE_DIR/lib/ld-linux-x86-64.so.2" --library-path "$PACKAGE_DIR/lib" --list "$PAYLOAD")"
else
  LDD_OUTPUT="$(ldd "$PAYLOAD")"
fi
if grep -q "not found" <<< "$LDD_OUTPUT"; then
  echo "linux dependency audit found missing dynamic libraries" >&2
  grep "not found" <<< "$LDD_OUTPUT" >&2
  exit 1
fi

SONAMES="$(
  awk '
    /=>/ { n=$1; sub(/^.*\//, "", n); print n; next }
    /^[[:space:]]*[^[:space:]]+\\.so/ { print $1; next }
    /^[[:space:]]*\// { n=$1; sub(/^.*\//, "", n); print n; next }
  ' <<< "$LDD_OUTPUT" \
    | sed 's/[[:space:]]//g' \
    | sort -u
)"

for required in libc.so.6 libm.so.6 libgcc_s.so.1 libasound.so.2; do
  if ! grep -qx "$required" <<< "$SONAMES"; then
    echo "linux dependency audit missing expected linked soname: $required" >&2
    exit 1
  fi
done

nix_store_refs="no"
if grep -q "/nix/store" <<< "$LDD_OUTPUT" || grep -q "/nix/store" <<< "$INTERPRETER" || strings "$BINARY" | grep -q "/nix/store"; then
  nix_store_refs="yes"
fi

payload_nix_store_refs="no"
if strings "$PAYLOAD" | grep -q "/nix/store"; then
  payload_nix_store_refs="yes"
fi

interpreter_nix_store="no"
if [[ "$INTERPRETER" == /nix/store/* ]]; then
  interpreter_nix_store="yes"
fi

dep_count="$(wc -l <<< "$SONAMES" | tr -d ' ')"
bundled_lib_count=0
if [[ -d "$PACKAGE_DIR/lib" ]]; then
  bundled_lib_count="$(find "$PACKAGE_DIR/lib" -maxdepth 1 -type f -name '*.so*' | wc -l | tr -d ' ')"
fi

echo "linux dependency audit ok"
echo "entrypoint: bevy_open_siege"
echo "entrypoint_file: $ENTRYPOINT_SUMMARY"
echo "wrapper_mode: $WRAPPER_MODE"
echo "payload: ${PAYLOAD#$PACKAGE_DIR/}"
echo "payload_file: $FILE_SUMMARY"
echo "interpreter: $INTERPRETER"
echo "bundled_loader: ${WRAPPER_MODE}"
echo "bundled_library_files: $bundled_lib_count"
echo "linked sonames: $dep_count"
while IFS= read -r soname; do
  [[ -n "$soname" ]] || continue
  echo "soname: $soname"
done <<< "$SONAMES"
echo "missing dependencies: none"
echo "nix_store_references: $nix_store_refs"
echo "payload_nix_store_references: $payload_nix_store_refs"
echo "interpreter_nix_store: $interpreter_nix_store"
if [[ "$WRAPPER_MODE" == "yes" && "$interpreter_nix_store" == "no" ]]; then
  echo "portability review: bundled loader wrapper is active; confirm on a clean non-Nix Linux QA machine before final approval"
else
  echo "portability review: confirm target storefront runtime includes these libraries or rebuild in a portable Linux environment before final approval"
fi
