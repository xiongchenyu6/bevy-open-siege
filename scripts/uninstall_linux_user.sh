#!/usr/bin/env bash
set -euo pipefail

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
BIN_HOME="${XDG_BIN_HOME:-$HOME/.local/bin}"
STATE_DIR="$DATA_HOME/bevy_open_siege"
APP_DIR="$STATE_DIR/app"
PURGE=0

if [[ "${1:-}" == "--purge" ]]; then
  PURGE=1
elif [[ $# -gt 0 ]]; then
  echo "usage: $0 [--purge]" >&2
  exit 2
fi

rm -rf "$APP_DIR"
rm -f "$BIN_HOME/bevy_open_siege"
rm -f "$DATA_HOME/applications/bevy-open-siege.desktop"
rm -f "$DATA_HOME/metainfo/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
rm -f "$DATA_HOME/icons/hicolor/512x512/apps/bevy-open-siege.png"

if [[ "$PURGE" == "1" ]]; then
  rm -rf "$STATE_DIR"
  echo "Uninstalled Bevy Open Siege user files and save data"
else
  echo "Uninstalled Bevy Open Siege user files; save data preserved in $STATE_DIR"
fi
