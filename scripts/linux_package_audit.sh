#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <extracted-package-dir>" >&2
  exit 2
fi

PACKAGE_DIR="$1"
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

for file in bevy_open_siege install_linux_user.sh uninstall_linux_user.sh; do
  if [[ ! -x "$PACKAGE_DIR/$file" ]]; then
    echo "linux package audit missing executable: $file" >&2
    exit 1
  fi
done

for file in \
  assets/linux/bevy-open-siege.desktop \
  assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml \
  assets/branding/generated/app-icon.png; do
  if [[ ! -f "$PACKAGE_DIR/$file" ]]; then
    echo "linux package audit missing file: $file" >&2
    exit 1
  fi
done

bash -n "$PACKAGE_DIR/install_linux_user.sh"
bash -n "$PACKAGE_DIR/uninstall_linux_user.sh"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

INSTALL_HOME="$TMPDIR/home"
INSTALL_DATA_HOME="$INSTALL_HOME/share"
INSTALL_BIN_HOME="$INSTALL_HOME/bin"
mkdir -p "$INSTALL_HOME"

HOME="$INSTALL_HOME" \
XDG_DATA_HOME="$INSTALL_DATA_HOME" \
XDG_BIN_HOME="$INSTALL_BIN_HOME" \
  "$PACKAGE_DIR/install_linux_user.sh" >/dev/null

"$INSTALL_BIN_HOME/bevy_open_siege" --validate-data >/dev/null
test -x "$INSTALL_BIN_HOME/bevy_open_siege"
test -x "$INSTALL_DATA_HOME/bevy_open_siege/app/bevy_open_siege"
test -f "$INSTALL_DATA_HOME/applications/bevy-open-siege.desktop"
test -f "$INSTALL_DATA_HOME/metainfo/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
test -f "$INSTALL_DATA_HOME/icons/hicolor/512x512/apps/bevy-open-siege.png"
grep -q "Exec=$INSTALL_BIN_HOME/bevy_open_siege" \
  "$INSTALL_DATA_HOME/applications/bevy-open-siege.desktop"
grep -q "Icon=bevy-open-siege" "$INSTALL_DATA_HOME/applications/bevy-open-siege.desktop"

echo "smoke-save" > "$INSTALL_DATA_HOME/bevy_open_siege/bevy_open_siege_save.ron"

HOME="$INSTALL_HOME" \
XDG_DATA_HOME="$INSTALL_DATA_HOME" \
XDG_BIN_HOME="$INSTALL_BIN_HOME" \
  "$PACKAGE_DIR/uninstall_linux_user.sh" >/dev/null

test ! -e "$INSTALL_DATA_HOME/bevy_open_siege/app"
test ! -e "$INSTALL_BIN_HOME/bevy_open_siege"
test ! -e "$INSTALL_DATA_HOME/applications/bevy-open-siege.desktop"
test ! -e "$INSTALL_DATA_HOME/metainfo/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
test ! -e "$INSTALL_DATA_HOME/icons/hicolor/512x512/apps/bevy-open-siege.png"
test -f "$INSTALL_DATA_HOME/bevy_open_siege/bevy_open_siege_save.ron"

HOME="$INSTALL_HOME" \
XDG_DATA_HOME="$INSTALL_DATA_HOME" \
XDG_BIN_HOME="$INSTALL_BIN_HOME" \
  "$PACKAGE_DIR/uninstall_linux_user.sh" --purge >/dev/null

test ! -e "$INSTALL_DATA_HOME/bevy_open_siege"

echo "linux package audit ok"
echo "checked install target: user-local XDG directories"
echo "checked launcher: bin wrapper validates release data"
echo "checked desktop metadata: launcher, AppStream, icon"
echo "checked uninstall: app files removed and save preserved"
echo "checked purge: save data removed only with --purge"
