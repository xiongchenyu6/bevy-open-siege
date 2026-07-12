#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 --plan <extracted-package-dir> | --init <extracted-package-dir> <session-dir>" >&2
}

require_package() {
  local package_dir="$1"
  if [[ ! -d "$package_dir" ]]; then
    echo "package directory not found: $package_dir" >&2
    exit 1
  fi
  if [[ ! -x "$package_dir/bevy_open_siege" && ! -x "$package_dir/bevy_open_siege.exe" ]]; then
    echo "platform package plan requires an extracted Bevy Open Siege package" >&2
    exit 1
  fi
}

package_version() {
  local package_dir="$1"
  local package_name version
  package_name="$(basename "$package_dir")"
  version="${package_name#bevy_open_siege-}"
  version="${version%-linux-x86_64}"
  version="${version%-windows-x86_64}"
  version="${version%-macos-universal}"
  echo "$version"
}

print_plan() {
  local package_dir="$1"
  local version
  version="$(package_version "$package_dir")"

  cat <<EOF
platform package plan ok
product: Bevy Open Siege
version: $version
current package: linux-x86_64

required platform packages:
- linux-x86_64: dist/bevy_open_siege-$version-linux-x86_64.tar.gz
- windows-x86_64: dist/bevy_open_siege-$version-windows-x86_64.zip
- macos-universal: dist/bevy_open_siege-$version-macos-universal.tar.gz

windows build host:
- OS: Windows 10 or newer
- Rust target: x86_64-pc-windows-msvc
- Package command: powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1
- Build: cargo build --release --target x86_64-pc-windows-msvc
- Binary: target/x86_64-pc-windows-msvc/release/bevy_open_siege.exe
- Smoke: bevy_open_siege.exe --validate-data
- Smoke: bevy_open_siege.exe --audit-audio
- Smoke: bevy_open_siege.exe --release-readiness
- Window smoke: run packaged runtime smoke equivalent and confirm a window opens without panic signatures
- Audio smoke: run with --audio and confirm a window opens without panic signatures before speaker/headphone listening QA
- Archive evidence: SHA256SUMS, release-info.txt, runtime-smoke.txt, audio-smoke.txt, privacy-audit.txt, release-provenance-audit.txt, platform-package-plan.txt

macOS build host:
- OS: macOS 13 or newer
- Rust targets: x86_64-apple-darwin and aarch64-apple-darwin
- Package command: scripts/package_macos.sh
- Build: cargo build --release --target x86_64-apple-darwin
- Build: cargo build --release --target aarch64-apple-darwin
- Package: create a universal binary with lipo or ship architecture-specific archives if universal signing is unavailable
- Smoke: ./bevy_open_siege --validate-data
- Smoke: ./bevy_open_siege --audit-audio
- Smoke: ./bevy_open_siege --release-readiness
- Window smoke: run packaged runtime smoke equivalent and confirm a window opens without panic signatures
- Audio smoke: run with --audio and confirm a window opens without panic signatures before speaker/headphone listening QA
- Archive evidence: SHA256SUMS, release-info.txt, runtime-smoke.txt, audio-smoke.txt, privacy-audit.txt, release-provenance-audit.txt, platform-package-plan.txt

cross-platform acceptance:
- All packages contain README.md, LICENSE, CREDITS.md, RELEASE_NOTES.md, QA_SIGNOFF.md, PRIVACY.md, SUPPORT.md, BUILD_PROVENANCE.md, VERSION.ron, assets/data, assets/i18n, assets/art, assets/audio, release reports, and integrity hashes.
- Packaged binary output for --validate-data, --audit-assets, --audit-audio, --audit-controls, --audit-input-flow, --audit-localization, --audit-layout, --audit-visual, --audit-accessibility, --audit-performance, --audit-privacy, --audit-release-provenance, --audit-marketing, --audit-ip, --audit-save, --audit-playthrough, --simulate-campaign, and --release-readiness matches the included reports.
- Manual QA rows for Windows package QA and macOS package QA are completed or explicitly scoped out before final release approval.
EOF
}

write_platform_template() {
  local path="$1"
  local title="$2"
  local target="$3"
  cat > "$path" <<EOF
# $title

Status: Pending
Owner:
Date:
Target: $target
Package:
SHA256:

## Build Host

- OS:
- Rust version:
- Cargo target:

## Commands

- [ ] Build:
- [ ] Package:
- [ ] Integrity check:
- [ ] Validate data:
- [ ] Runtime smoke:
- [ ] Audio startup smoke:

## Evidence

- [ ] release-info.txt
- [ ] release-readiness.txt
- [ ] runtime-smoke.txt
- [ ] audio-smoke.txt
- [ ] SHA256SUMS

## Findings

- Pass/Fail:
- Issues:
- Follow-up required:

## Signoff

Approved: No
Approver:
Notes:
EOF
}

init_session() {
  local package_dir="$1"
  local session_dir="$2"
  mkdir -p "$session_dir"
  print_plan "$package_dir" > "$session_dir/platform-package-plan.txt"
  write_platform_template "$session_dir/windows-package-qa.md" "Windows Package QA" "x86_64-pc-windows-msvc"
  write_platform_template "$session_dir/macos-package-qa.md" "macOS Package QA" "x86_64-apple-darwin + aarch64-apple-darwin"
  echo "platform package session initialized"
  echo "session directory: $session_dir"
  echo "created: $session_dir/platform-package-plan.txt"
  echo "created: $session_dir/windows-package-qa.md"
  echo "created: $session_dir/macos-package-qa.md"
}

case "${1:-}" in
  --plan)
    if [[ $# -ne 2 ]]; then
      usage
      exit 2
    fi
    require_package "$2"
    print_plan "$2"
    ;;
  --init)
    if [[ $# -ne 3 ]]; then
      usage
      exit 2
    fi
    require_package "$2"
    init_session "$2" "$3"
    ;;
  *)
    usage
    exit 2
    ;;
esac
