#!/usr/bin/env bash
set -euo pipefail

SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
BIN_HOME="${XDG_BIN_HOME:-$HOME/.local/bin}"
STATE_DIR="$DATA_HOME/bevy_open_siege"
APP_DIR="$STATE_DIR/app"
APPLICATIONS_DIR="$DATA_HOME/applications"
APPSTREAM_DIR="$DATA_HOME/metainfo"
ICON_DIR="$DATA_HOME/icons/hicolor/512x512/apps"

if [[ ! -x "$SOURCE_DIR/bevy_open_siege" ]]; then
  echo "install script must be run from an extracted Bevy Open Siege release package" >&2
  exit 1
fi

mkdir -p "$BIN_HOME" "$APPLICATIONS_DIR" "$APPSTREAM_DIR" "$ICON_DIR"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR" "$STATE_DIR"
cp -R "$SOURCE_DIR/." "$APP_DIR/"

cat > "$BIN_HOME/bevy_open_siege" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$APP_DIR"
exec ./bevy_open_siege "\$@"
EOF
chmod +x "$BIN_HOME/bevy_open_siege"

sed "s#^Exec=.*#Exec=$BIN_HOME/bevy_open_siege#" \
  "$APP_DIR/assets/linux/bevy-open-siege.desktop" \
  > "$APPLICATIONS_DIR/bevy-open-siege.desktop"
cp "$APP_DIR/assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml" \
  "$APPSTREAM_DIR/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
cp "$APP_DIR/assets/branding/generated/app-icon.png" "$ICON_DIR/bevy-open-siege.png"

echo "Installed Bevy Open Siege to $APP_DIR"
echo "Launcher: $BIN_HOME/bevy_open_siege"
echo "Save data is stored in $STATE_DIR"
