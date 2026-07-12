#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  linux_clean_distro_smoke.sh <extracted-package-dir> [container-image]

Runs the Linux package inside a clean non-Nix container image with networking
disabled. This verifies package data loading and bundled runtime dependency
resolution without relying on the host shell, Nix store paths, or host library
search paths. Window, GPU, and audio QA still require a release QA machine.
EOF
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 2
fi

PACKAGE_DIR="$1"
IMAGE="${2:-docker.io/library/ubuntu:24.04}"

if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "linux clean distro smoke package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi
for file in bevy_open_siege bevy_open_siege.bin lib/ld-linux-x86-64.so.2; do
  if [[ ! -x "$PACKAGE_DIR/$file" ]]; then
    echo "linux clean distro smoke missing executable: $file" >&2
    exit 1
  fi
done

ENGINE=""
if command -v podman >/dev/null 2>&1; then
  ENGINE="podman"
elif command -v docker >/dev/null 2>&1; then
  ENGINE="docker"
else
  echo "linux clean distro smoke requires podman or docker" >&2
  exit 1
fi

PACKAGE_ABS="$(cd "$PACKAGE_DIR" && pwd)"
TMPDIR_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_ROOT"' EXIT

cat > "$TMPDIR_ROOT/check.sh" <<'EOF'
#!/bin/sh
set -eu
cd /pkg
./bevy_open_siege --validate-data > /tmp/validate-data.txt
lib/ld-linux-x86-64.so.2 --library-path lib --list ./bevy_open_siege.bin > /tmp/ld-list.txt
if grep -q "not found" /tmp/ld-list.txt; then
  cat /tmp/ld-list.txt >&2
  exit 1
fi
if grep -q "/nix/store" bevy_open_siege; then
  echo "entrypoint references /nix/store" >&2
  exit 1
fi
printf '%s\n' "validate_data: pass"
printf '%s\n' "dependency_resolution: pass"
printf '%s\n' "missing_dependencies: none"
printf '%s\n' "entrypoint_nix_store_references: no"
printf '%s\n' "window_smoke: not_run_in_container"
printf '%s\n' "audio_smoke: not_run_in_container"
EOF
chmod +x "$TMPDIR_ROOT/check.sh"

RUN_OUTPUT="$(
  "$ENGINE" run --rm --network none \
    -v "$PACKAGE_ABS:/pkg:ro" \
    -v "$TMPDIR_ROOT/check.sh:/check.sh:ro" \
    "$IMAGE" /check.sh
)"

cat <<EOF
linux clean distro smoke ok
engine: $ENGINE
container_image: $IMAGE
package: $(basename "$PACKAGE_ABS")
$RUN_OUTPUT
clean_distro_scope: validate-data and bundled dependency resolution
manual_clean_machine_qa: still required for window, GPU, audio, install, and input review
EOF
