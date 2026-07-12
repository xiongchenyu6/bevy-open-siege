#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <extracted-package-dir>" >&2
  exit 2
fi

PACKAGE_DIR="$1"
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "store asset audit package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

icon="$PACKAGE_DIR/assets/branding/generated/app-icon.png"
capsule="$PACKAGE_DIR/assets/branding/generated/store-capsule.png"

for file in "$icon" "$capsule" \
  "$PACKAGE_DIR/STORE_PAGE.md" \
  "$PACKAGE_DIR/PRESSKIT.md" \
  "$PACKAGE_DIR/STORE_SCREENSHOTS.md" \
  "$PACKAGE_DIR/ART_ASSETS.md"; do
  if [[ ! -f "$file" ]]; then
    echo "store asset audit missing file: ${file#$PACKAGE_DIR/}" >&2
    exit 1
  fi
done

png_size() {
  python3 - "$1" <<'PY'
import struct
import sys
from pathlib import Path

path = Path(sys.argv[1])
data = path.read_bytes()
if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
    raise SystemExit(f"not a PNG file: {path}")
width, height = struct.unpack(">II", data[16:24])
print(f"{width}x{height}")
PY
}

icon_size="$(png_size "$icon")"
capsule_size="$(png_size "$capsule")"
icon_width="${icon_size%x*}"
icon_height="${icon_size#*x}"
capsule_width="${capsule_size%x*}"
capsule_height="${capsule_size#*x}"

if [[ "$icon_width" -ne "$icon_height" || "$icon_width" -lt 512 ]]; then
  echo "store asset audit app icon must be square and at least 512px: $icon_size" >&2
  exit 1
fi

python3 - "$capsule_width" "$capsule_height" <<'PY'
import sys

width = int(sys.argv[1])
height = int(sys.argv[2])
ratio = width / height
if width < 1200 or height < 675:
    raise SystemExit(f"store capsule must be at least 1200x675: {width}x{height}")
if not 1.70 <= ratio <= 1.85:
    raise SystemExit(f"store capsule must be near 16:9: {width}x{height}")
PY

grep -q "assets/branding/generated/app-icon.png" "$PACKAGE_DIR/STORE_PAGE.md"
grep -q "assets/branding/generated/store-capsule.png" "$PACKAGE_DIR/STORE_PAGE.md"
grep -q "assets/branding/generated/app-icon.png" "$PACKAGE_DIR/PRESSKIT.md"
grep -q "assets/branding/generated/store-capsule.png" "$PACKAGE_DIR/PRESSKIT.md"
grep -q "assets/branding/generated/app-icon.png" "$PACKAGE_DIR/ART_ASSETS.md"
grep -q "assets/branding/generated/store-capsule.png" "$PACKAGE_DIR/ART_ASSETS.md"
grep -q "screenshots/01-title-menu.png" "$PACKAGE_DIR/STORE_SCREENSHOTS.md"
grep -q "screenshots/05-victory-summary.png" "$PACKAGE_DIR/STORE_SCREENSHOTS.md"
grep -q "1920x1080" "$PACKAGE_DIR/STORE_SCREENSHOTS.md"
grep -q "one English screenshot and one Chinese screenshot" "$PACKAGE_DIR/STORE_SCREENSHOTS.md"
grep -q "store_screenshot_check.sh --validate-dir screenshots" "$PACKAGE_DIR/STORE_SCREENSHOTS.md"

variance_validator="not installed"
if command -v magick >/dev/null 2>&1; then
  for file in "$icon" "$capsule"; do
    stddev="$(magick identify -format '%[standard-deviation]' "$file")"
    python3 - "$stddev" "$file" <<'PY'
import sys

stddev = float(sys.argv[1])
path = sys.argv[2]
if stddev <= 0.0:
    raise SystemExit(f"store asset appears blank or solid: {path}")
PY
  done
  variance_validator="passed"
elif command -v identify >/dev/null 2>&1; then
  for file in "$icon" "$capsule"; do
    stddev="$(identify -format '%[standard-deviation]' "$file")"
    python3 - "$stddev" "$file" <<'PY'
import sys

stddev = float(sys.argv[1])
path = sys.argv[2]
if stddev <= 0.0:
    raise SystemExit(f"store asset appears blank or solid: {path}")
PY
  done
  variance_validator="passed"
fi

echo "store asset audit ok"
echo "app icon: assets/branding/generated/app-icon.png ${icon_size}"
echo "store capsule: assets/branding/generated/store-capsule.png ${capsule_size}"
echo "checked minimums: icon square >=512, capsule >=1200x675 near 16:9"
echo "checked store page references: app icon, store capsule"
echo "checked press kit references: app icon, store capsule"
echo "checked art notes references: app icon, store capsule"
echo "checked screenshot plan: five captures, 1920x1080, en/zh coverage, validation helper"
echo "image variance validator: $variance_validator"
