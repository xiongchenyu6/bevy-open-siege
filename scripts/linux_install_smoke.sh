#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  linux_install_smoke.sh <extracted-package-dir> [runtime_seconds] [visual_seconds]

Installs the Linux package into temporary XDG user directories, starts the
installed launcher through runtime and visual smoke tests, verifies uninstall
preserves save data, and verifies --purge removes save data.
EOF
}

if [[ $# -lt 1 || $# -gt 3 ]]; then
  usage
  exit 2
fi

PACKAGE_DIR="$1"
RUNTIME_SECONDS="${2:-12}"
VISUAL_SECONDS="${3:-15}"

if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "linux install smoke package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

for file in \
  bevy_open_siege \
  install_linux_user.sh \
  uninstall_linux_user.sh \
  runtime_smoke.sh \
  visual_smoke.sh; do
  if [[ ! -x "$PACKAGE_DIR/$file" ]]; then
    echo "linux install smoke missing executable: $file" >&2
    exit 1
  fi
done

bash -n "$PACKAGE_DIR/install_linux_user.sh"
bash -n "$PACKAGE_DIR/uninstall_linux_user.sh"
bash -n "$PACKAGE_DIR/runtime_smoke.sh"
bash -n "$PACKAGE_DIR/visual_smoke.sh"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

INSTALL_HOME="$TMPDIR/home"
INSTALL_DATA_HOME="$TMPDIR/xdg-data"
INSTALL_BIN_HOME="$TMPDIR/bin"
mkdir -p "$INSTALL_HOME" "$INSTALL_DATA_HOME" "$INSTALL_BIN_HOME"

export HOME="$INSTALL_HOME"
export XDG_DATA_HOME="$INSTALL_DATA_HOME"
export XDG_BIN_HOME="$INSTALL_BIN_HOME"

"$PACKAGE_DIR/install_linux_user.sh" > "$TMPDIR/install.log"

INSTALLED_LAUNCHER="$INSTALL_BIN_HOME/bevy_open_siege"
INSTALLED_APP_DIR="$INSTALL_DATA_HOME/bevy_open_siege/app"
INSTALLED_SAVE="$INSTALL_DATA_HOME/bevy_open_siege/bevy_open_siege_save.ron"

test -x "$INSTALLED_LAUNCHER"
test -x "$INSTALLED_APP_DIR/bevy_open_siege"

"$INSTALLED_LAUNCHER" --validate-data >/dev/null
"$PACKAGE_DIR/runtime_smoke.sh" "$INSTALLED_LAUNCHER" "$RUNTIME_SECONDS" > "$TMPDIR/runtime-smoke.txt"
"$PACKAGE_DIR/visual_smoke.sh" "$INSTALLED_LAUNCHER" "$VISUAL_SECONDS" > "$TMPDIR/visual-smoke.txt"

echo "install-smoke-save" > "$INSTALLED_SAVE"
"$PACKAGE_DIR/uninstall_linux_user.sh" > "$TMPDIR/uninstall.log"

test ! -e "$INSTALLED_APP_DIR"
test ! -e "$INSTALLED_LAUNCHER"
test -f "$INSTALLED_SAVE"

"$PACKAGE_DIR/uninstall_linux_user.sh" --purge > "$TMPDIR/purge.log"
test ! -e "$INSTALL_DATA_HOME/bevy_open_siege"

echo "linux install smoke ok"
echo "checked install root: temporary XDG user directories"
echo "checked installed launcher: --validate-data"
echo "checked installed runtime smoke: ${RUNTIME_SECONDS}s"
echo "checked installed visual smoke: ${VISUAL_SECONDS}s"
echo "checked uninstall: app files removed and save preserved"
echo "checked purge: save data removed"
