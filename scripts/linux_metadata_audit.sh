#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <extracted-linux-package-dir>" >&2
  exit 2
fi

PACKAGE_DIR="$1"
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "linux metadata audit package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

desktop_file="$PACKAGE_DIR/assets/linux/bevy-open-siege.desktop"
metainfo_file="$PACKAGE_DIR/assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
icon_file="$PACKAGE_DIR/assets/branding/generated/app-icon.png"
binary_file="$PACKAGE_DIR/bevy_open_siege"

for file in "$desktop_file" "$metainfo_file" "$icon_file" "$binary_file"; do
  if [[ ! -f "$file" ]]; then
    echo "linux metadata audit missing file: ${file#$PACKAGE_DIR/}" >&2
    exit 1
  fi
done

if [[ ! -x "$binary_file" ]]; then
  echo "linux metadata audit binary is not executable: bevy_open_siege" >&2
  exit 1
fi

package_name="$(basename "$PACKAGE_DIR")"
package_version="${package_name#bevy_open_siege-}"
package_version="${package_version%-linux-x86_64}"

desktop_require() {
  local line="$1"
  if ! grep -Fxq "$line" "$desktop_file"; then
    echo "linux metadata audit desktop entry missing exact line: $line" >&2
    exit 1
  fi
}

desktop_require "[Desktop Entry]"
desktop_require "Type=Application"
desktop_require "Name=Bevy Open Siege"
desktop_require "Exec=bevy_open_siege"
desktop_require "Icon=bevy-open-siege"
desktop_require "Terminal=false"
grep -qx "Categories=.*Game.*StrategyGame.*;" "$desktop_file" || {
  echo "linux metadata audit desktop categories must include Game and StrategyGame" >&2
  exit 1
}
grep -qx "Keywords=.*strategy.*tower-defense.*lane-defense.*;" "$desktop_file" || {
  echo "linux metadata audit desktop keywords must cover strategy, tower-defense, and lane-defense" >&2
  exit 1
}

grep -q '<component type="desktop-application">' "$metainfo_file"
grep -q '<id>io.github.bevy_open_siege.BevyOpenSiege</id>' "$metainfo_file"
grep -q '<metadata_license>CC0-1.0</metadata_license>' "$metainfo_file"
grep -q '<project_license>MIT</project_license>' "$metainfo_file"
grep -q '<name>Bevy Open Siege</name>' "$metainfo_file"
grep -q '<launchable type="desktop-id">bevy-open-siege.desktop</launchable>' "$metainfo_file"
grep -q '<binary>bevy_open_siege</binary>' "$metainfo_file"
grep -q '<category>Game</category>' "$metainfo_file"
grep -q '<category>StrategyGame</category>' "$metainfo_file"
grep -q "<release version=\"$package_version\"" "$metainfo_file"
grep -q "10-level campaign" "$metainfo_file"
grep -q "bilingual English and Chinese localization" "$metainfo_file"

if command -v file >/dev/null 2>&1; then
  file "$icon_file" | grep -q "PNG image data" || {
    echo "linux metadata audit icon is not a PNG image" >&2
    exit 1
  }
fi

desktop_validator="not installed"
if command -v desktop-file-validate >/dev/null 2>&1; then
  desktop-file-validate "$desktop_file"
  desktop_validator="passed"
fi

appstream_validator="not installed"
if command -v appstreamcli >/dev/null 2>&1; then
  appstreamcli validate --no-net "$metainfo_file" >/dev/null
  appstream_validator="passed"
elif command -v appstream-util >/dev/null 2>&1; then
  appstream-util validate-relax "$metainfo_file" >/dev/null
  appstream_validator="passed"
fi

echo "linux metadata audit ok"
echo "package: $package_name"
echo "version: $package_version"
echo "desktop entry: bevy-open-siege.desktop"
echo "appstream id: io.github.bevy_open_siege.BevyOpenSiege"
echo "launchable: bevy-open-siege.desktop"
echo "binary: bevy_open_siege"
echo "icon: assets/branding/generated/app-icon.png"
echo "categories: Game;StrategyGame;"
echo "desktop-file-validate: $desktop_validator"
echo "appstream validator: $appstream_validator"
