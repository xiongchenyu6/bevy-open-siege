#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  store_screenshot_check.sh --plan <package-dir>
  store_screenshot_check.sh --capture-current <output-dir> <filename>
  store_screenshot_check.sh --capture-startup <package-dir> <output-dir> <filename> [duration_seconds]
  store_screenshot_check.sh --capture-pack <package-dir> <output-dir> [duration_seconds]
  store_screenshot_check.sh --validate-dir <screenshot-dir>
EOF
}

require_package() {
  local package_dir="$1"
  if [[ ! -d "$package_dir" ]]; then
    echo "package directory not found: $package_dir" >&2
    exit 1
  fi
  if [[ ! -x "$package_dir/bevy_open_siege" && ! -x "$package_dir/bevy_open_siege.exe" ]]; then
    echo "store screenshot workflow requires an extracted Bevy Open Siege package" >&2
    exit 1
  fi
}

package_binary() {
  local package_dir="$1"
  if [[ -x "$package_dir/bevy_open_siege.exe" ]]; then
    echo "$package_dir/bevy_open_siege.exe"
  else
    echo "$package_dir/bevy_open_siege"
  fi
}

require_capture_tools() {
  if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
    echo "screenshot capture requires DISPLAY or WAYLAND_DISPLAY" >&2
    exit 1
  fi
  if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
    export WINIT_UNIX_BACKEND="${WINIT_UNIX_BACKEND:-wayland}"
  fi
  if [[ "${WINIT_UNIX_BACKEND:-}" == "wayland" && -n "${WAYLAND_DISPLAY:-}" ]]; then
    if ! command -v grim >/dev/null 2>&1 || ! command -v magick >/dev/null 2>&1; then
      echo "screenshot capture requires grim and magick commands on Wayland" >&2
      exit 1
    fi
  elif ! command -v import >/dev/null 2>&1 || ! command -v magick >/dev/null 2>&1; then
    echo "screenshot capture requires ImageMagick import and magick commands on X11" >&2
    exit 1
  fi
}

validate_png() {
  local path="$1"
  if [[ ! -s "$path" ]]; then
    echo "screenshot is missing or empty: $path" >&2
    exit 1
  fi

  local dimensions width height stddev
  dimensions="$(magick identify -format '%w %h' "$path")"
  width="${dimensions%% *}"
  height="${dimensions##* }"
  stddev="$(magick "$path" -colorspace Gray -format '%[fx:standard_deviation]' info:)"

  if [[ "$width" -lt 1920 || "$height" -lt 1080 ]]; then
    echo "screenshot must be at least 1920x1080: $path is ${width}x${height}" >&2
    exit 1
  fi
  if ! awk -v w="$width" -v h="$height" 'BEGIN { ratio = w / h; exit !(ratio > 1.76 && ratio < 1.79) }'; then
    echo "screenshot must be 16:9: $path is ${width}x${height}" >&2
    exit 1
  fi
  if ! awk -v value="$stddev" 'BEGIN { exit !(value > 0.005) }'; then
    echo "screenshot appears blank or nearly solid: $path" >&2
    echo "screenshot_stddev: $stddev" >&2
    exit 1
  fi

  echo "${path} | ${width}x${height} | stddev ${stddev}"
}

capture_window() {
  local output_dir="$1"
  local filename="$2"
  require_capture_tools
  mkdir -p "$output_dir"
  local output_path="$output_dir/$filename"
  if [[ "${WINIT_UNIX_BACKEND:-}" == "wayland" && -n "${WAYLAND_DISPLAY:-}" ]]; then
    grim "$output_path"
  else
    import -window "Bevy Open Siege" "$output_path"
  fi
  validate_png "$output_path" >/dev/null
  echo "store screenshot captured: $output_path"
  validate_png "$output_path"
}

capture_startup() {
  local package_dir="$1"
  local output_dir="$2"
  local filename="$3"
  local duration="${4:-8}"
  require_package "$package_dir"
  require_capture_tools
  if ! [[ "$duration" =~ ^[0-9]+$ ]] || [[ "$duration" -lt 4 ]]; then
    echo "capture duration must be an integer of at least 4 seconds" >&2
    exit 2
  fi

  local binary log_file pid
  binary="$(package_binary "$package_dir")"
  log_file="$(mktemp)"
  pid=""
  cleanup() {
    if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
    rm -f "$log_file"
  }
  trap cleanup EXIT

  "$binary" --no-audio > "$log_file" 2>&1 &
  pid=$!

  local deadline=$((SECONDS + duration))
  while [[ "$SECONDS" -lt "$deadline" ]]; do
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      echo "store screenshot capture failed: game exited before capture" >&2
      cat "$log_file" >&2
      exit 1
    fi
    if grep -q "Creating new window Bevy Open Siege" "$log_file"; then
      break
    fi
    sleep 0.25
  done

  if ! grep -q "Creating new window Bevy Open Siege" "$log_file"; then
    echo "store screenshot capture failed: window creation log not found within ${duration}s" >&2
    cat "$log_file" >&2
    exit 1
  fi
  if grep -Eiq 'panicked at|Encountered a panic|thread .* panicked|error\[B0001\]' "$log_file"; then
    echo "store screenshot capture failed: panic signature found" >&2
    cat "$log_file" >&2
    exit 1
  fi

  sleep 1
  capture_window "$output_dir" "$filename"
}

capture_scene() {
  local package_dir="$1"
  local output_dir="$2"
  local scene="$3"
  local filename="$4"
  local duration="${5:-10}"
  require_package "$package_dir"
  require_capture_tools
  if ! [[ "$duration" =~ ^[0-9]+$ ]] || [[ "$duration" -lt 6 ]]; then
    echo "capture duration must be an integer of at least 6 seconds" >&2
    exit 2
  fi

  local binary log_file pid
  binary="$(package_binary "$package_dir")"
  log_file="$(mktemp)"
  pid=""
  cleanup_scene() {
    if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
    rm -f "$log_file"
  }
  trap cleanup_scene EXIT

  "$binary" --no-audio --store-screenshot-scene "$scene" > "$log_file" 2>&1 &
  pid=$!

  local deadline=$((SECONDS + duration))
  while [[ "$SECONDS" -lt "$deadline" ]]; do
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      echo "store screenshot scene capture failed: game exited before capture" >&2
      cat "$log_file" >&2
      exit 1
    fi
    if grep -q "Creating new window Bevy Open Siege" "$log_file"; then
      break
    fi
    sleep 0.25
  done

  if ! grep -q "Creating new window Bevy Open Siege" "$log_file"; then
    echo "store screenshot scene capture failed: window creation log not found within ${duration}s" >&2
    cat "$log_file" >&2
    exit 1
  fi
  if grep -Eiq 'panicked at|Encountered a panic|thread .* panicked|error\[B0001\]' "$log_file"; then
    echo "store screenshot scene capture failed: panic signature found" >&2
    cat "$log_file" >&2
    exit 1
  fi

  sleep 2
  capture_window "$output_dir" "$filename"
  cleanup_scene
  trap - EXIT
}

capture_pack() {
  local package_dir="$1"
  local output_dir="$2"
  local duration="${3:-10}"
  mkdir -p "$output_dir"
  capture_scene "$package_dir" "$output_dir" "title-menu" "01-title-menu.png" "$duration"
  capture_scene "$package_dir" "$output_dir" "early-defense" "02-early-defense.png" "$duration"
  capture_scene "$package_dir" "$output_dir" "special-enemies" "03-special-enemies.png" "$duration"
  capture_scene "$package_dir" "$output_dir" "late-siege" "04-late-siege.png" "$duration"
  capture_scene "$package_dir" "$output_dir" "victory-summary" "05-victory-summary.png" "$duration"
  validate_dir "$output_dir"
}

print_plan() {
  local package_dir="$1"
  local binary
  require_package "$package_dir"
  binary="$(package_binary "$package_dir")"
  cat <<EOF
store screenshot workflow ok
package: $(basename "$package_dir")
binary: $binary
required screenshots:
- screenshots/01-title-menu.png
- screenshots/02-early-defense.png
- screenshots/03-special-enemies.png
- screenshots/04-late-siege.png
- screenshots/05-victory-summary.png
capture commands:
- ./store_screenshot_check.sh --capture-startup . screenshots 01-title-menu.png 8
- ./store_screenshot_check.sh --capture-pack . screenshots 10
- ./store_screenshot_check.sh --capture-current screenshots 02-early-defense.png
- ./store_screenshot_check.sh --capture-current screenshots 03-special-enemies.png
- ./store_screenshot_check.sh --capture-current screenshots 04-late-siege.png
- ./store_screenshot_check.sh --capture-current screenshots 05-victory-summary.png
- ./store_screenshot_check.sh --validate-dir screenshots
requirements:
- 1920x1080 or larger
- 16:9 aspect ratio
- nonblank window capture
- at least one English screenshot and one Chinese screenshot
- final notes recorded in qa-session/store-screenshots.md
EOF
}

validate_dir() {
  local dir="$1"
  if [[ ! -d "$dir" ]]; then
    echo "screenshot directory not found: $dir" >&2
    exit 1
  fi
  local required=(
    "01-title-menu.png"
    "02-early-defense.png"
    "03-special-enemies.png"
    "04-late-siege.png"
    "05-victory-summary.png"
  )
  echo "store screenshot validation ok"
  for file in "${required[@]}"; do
    validate_png "$dir/$file"
  done
  echo "checked screenshots: ${#required[@]}"
  echo "manual review still required: language coverage, composition, storefront rules, and qa-session/store-screenshots.md approval"
}

case "${1:-}" in
  --plan)
    if [[ $# -ne 2 ]]; then
      usage
      exit 2
    fi
    print_plan "$2"
    ;;
  --capture-current)
    if [[ $# -ne 3 ]]; then
      usage
      exit 2
    fi
    capture_window "$2" "$3"
    ;;
  --capture-startup)
    if [[ $# -lt 4 || $# -gt 5 ]]; then
      usage
      exit 2
    fi
    capture_startup "$2" "$3" "$4" "${5:-8}"
    ;;
  --capture-pack)
    if [[ $# -lt 3 || $# -gt 4 ]]; then
      usage
      exit 2
    fi
    capture_pack "$2" "$3" "${4:-10}"
    ;;
  --validate-dir)
    if [[ $# -ne 2 ]]; then
      usage
      exit 2
    fi
    validate_dir "$2"
    ;;
  *)
    usage
    exit 2
    ;;
esac
