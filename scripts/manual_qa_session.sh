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
    echo "manual QA session requires an extracted Bevy Open Siege package" >&2
    exit 1
  fi
}

package_binary() {
  local package_dir="$1"
  if [[ -x "$package_dir/bevy_open_siege.exe" ]]; then
    echo "./bevy_open_siege.exe"
  else
    echo "./bevy_open_siege"
  fi
}

manual_files=(
  "full-campaign-playthrough.md"
  "balance-usability.md"
  "audio-device-qa.md"
  "visual-hardware-spotcheck.md"
  "localization-review.md"
  "input-bindings.md"
  "accessibility-qa.md"
  "performance-qa.md"
  "privacy-support.md"
  "build-provenance.md"
  "save-compatibility.md"
  "store-screenshots.md"
  "store-press-ip-review.md"
  "final-art-direction.md"
)

print_plan() {
  local package_dir="$1"
  local package_name binary
  package_name="$(basename "$package_dir")"
  binary="$(package_binary "$package_dir")"

  cat <<EOF
manual QA session plan ok
package: $package_name
session directory: qa-session
isolated save: qa-session/bevy_open_siege_save.ron
command prefix: BEVY_OPEN_SIEGE_SAVE_PATH=qa-session/bevy_open_siege_save.ron
binary: $binary

required setup:
- Create qa-session before testing.
- Run all game commands from the extracted package root.
- Keep audio opt-in unless audio-device QA approves changing default startup.
- Attach this plan, QA_SIGNOFF.md, release-readiness.txt, and generated session notes to final approval.

automated evidence to review before manual testing:
- release-readiness.txt
- balance-audit.txt
- asset-audit.txt
- audio-audit.txt
- controls-audit.txt
- input-flow-audit.txt
- localization-audit.txt
- layout-audit.txt
- visual-readability-audit.txt
- accessibility-audit.txt
- performance-audit.txt
- privacy-audit.txt
- release-provenance-audit.txt
- marketing-audit.txt
- ip-audit.txt
- save-audit.txt
- playthrough-audit.txt
- campaign-simulation.txt
- runtime-smoke.txt
- visual-smoke.txt
- audio-smoke.txt
- store-asset-audit.txt
- content-rating-audit.txt
- linux-package-audit.txt
- linux-install-smoke.txt
- linux-dependency-audit.txt
- linux-portability-smoke.txt
- linux-clean-distro-smoke.txt
- linux-metadata-audit.txt
- SHA256SUMS

manual evidence files to produce:
- qa-session/full-campaign-playthrough.md
- qa-session/balance-usability.md
- qa-session/audio-device-qa.md
- qa-session/visual-hardware-spotcheck.md
- qa-session/localization-review.md
- qa-session/input-bindings.md
- qa-session/accessibility-qa.md
- qa-session/performance-qa.md
- qa-session/privacy-support.md
- qa-session/build-provenance.md
- qa-session/save-compatibility.md
- qa-session/store-screenshots.md
- qa-session/store-press-ip-review.md
- qa-session/final-art-direction.md

manual run commands:
- mkdir -p qa-session
- ./manual_qa_session.sh --init . qa-session
- BEVY_OPEN_SIEGE_SAVE_PATH=qa-session/bevy_open_siege_save.ron ./bevy_open_siege
- BEVY_OPEN_SIEGE_SAVE_PATH=qa-session/bevy_open_siege_save.ron ./bevy_open_siege --audio
- BEVY_OPEN_SIEGE_SAVE_PATH=qa-session/bevy_open_siege_save.ron ./bevy_open_siege --print-save-summary
- ./runtime_smoke.sh ./bevy_open_siege 12
- ./visual_smoke.sh ./bevy_open_siege 15
- ./audio_smoke.sh ./bevy_open_siege 12
- ./store_asset_audit.sh .
- ./content_rating_audit.sh .
- ./store_screenshot_check.sh --plan .
- ./store_screenshot_check.sh --capture-pack . screenshots 10
- ./store_screenshot_check.sh --capture-startup . screenshots 01-title-menu.png 8
- ./store_screenshot_check.sh --validate-dir screenshots
- ./linux_package_audit.sh .
- ./linux_clean_distro_smoke.sh .
- ./linux_metadata_audit.sh .
- sha256sum -c SHA256SUMS

manual acceptance checklist:
- Full campaign playthrough: finish all 10 levels from a clean isolated save, confirm unlock progression, victory, defeat, restart, and score recording.
- Balance and usability: repeat-play all 10 levels, confirm seed costs, cooldowns, wave pacing, score pacing, difficulty spikes, and input ergonomics are acceptable.
- Audio device QA: test music and sound effects on speakers and headphones with --audio, confirm no missing output, harsh clipping, distracting balance, or startup hang.
- Visual hardware spot-check: inspect title, level select, HUD, board, plants, monsters, effects, pause, victory, and defeat screens on release hardware.
- Localization review: manually verify English and Chinese copy quality in menu, HUD, level select, gameplay, settings, pause, and end screens.
- Keyboard and mouse QA: review input-flow-audit.txt, then execute every binding from controls-audit.txt, including mouse placement, shovel, pause, fullscreen, volume, language, and restart.
- Accessibility QA: review accessibility-audit.txt, then manually check assistive tech behavior, input remapping expectations, photosensitivity risk, and readability on release hardware.
- Performance QA: review performance-audit.txt, then profile release hardware for frame pacing, long-session stability, memory growth, and worst-wave responsiveness.
- Privacy and support QA: review privacy-audit.txt, PRIVACY.md, SUPPORT.md, uninstall preservation, explicit --purge behavior, and bug-report evidence guidance.
- Content rating review: confirm CONTENT_RATING.md and content-rating-audit.txt match the final platform questionnaire answers.
- Build provenance QA: review release-provenance-audit.txt, BUILD_PROVENANCE.md, SHA256SUMS, release-info.txt, Cargo.lock usage, and package host details.
- Save compatibility: verify old working-directory saves, XDG user-data saves, settings persistence, and BEVY_OPEN_SIEGE_SAVE_PATH override behavior.
- Store screenshot review: use store_screenshot_check.sh with STORE_SCREENSHOTS.md to capture or review the required 1920x1080 screenshot set, including English and Chinese coverage.
- Store and press review: confirm STORE_PAGE.md, PRESSKIT.md, branding art, capsule art, credits, notices, and release notes are final.
- IP and art-direction review: confirm names, visible art direction, mechanics, and store materials remain separated from similar commercial games.

final decision rule:
- Mark every QA_SIGNOFF.md table row as Pass or Scoped Out with Owner and YYYY-MM-DD Date filled; add Notes for scoped-out rows.
- Mark QA_SIGNOFF.md Release approved as Yes only after every manual evidence file is complete and Windows/macOS package QA rows are either completed or explicitly scoped out of the release.
EOF
}

write_template() {
  local path="$1"
  local title="$2"
  local focus="$3"
  cat > "$path" <<EOF
# $title

Status: Pending
Owner:
Date:
Package:
Save path: qa-session/bevy_open_siege_save.ron

## Focus

$focus

## Evidence Reviewed

- [ ] release-readiness.txt
- [ ] relevant audit or smoke report
- [ ] in-game behavior observed on release hardware

## Steps Performed

- [ ] Step 1:
- [ ] Step 2:
- [ ] Step 3:

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
  print_plan "$package_dir" > "$session_dir/manual-qa-plan.txt"
  cat > "$session_dir/README.md" <<'EOF'
# Bevy Open Siege Manual QA Session

Use these files to record the manual evidence required before final release approval. Run commands from the extracted package root with:

`BEVY_OPEN_SIEGE_SAVE_PATH=qa-session/bevy_open_siege_save.ron`

Do not mark `QA_SIGNOFF.md` approved until every required evidence file is complete.
EOF

  write_template "$session_dir/full-campaign-playthrough.md" \
    "Full Campaign Playthrough" \
    "Finish all 10 levels from a clean isolated save and confirm unlock progression, victory, defeat, restart, and score recording."
  write_template "$session_dir/balance-usability.md" \
    "Balance And Usability" \
    "Repeat-play all 10 levels and judge difficulty, seed costs, cooldowns, wave pacing, score pacing, and input ergonomics."
  write_template "$session_dir/audio-device-qa.md" \
    "Audio Device QA" \
    "Test music and sound effects on speakers and headphones with --audio after reviewing audio-audit.txt and audio-smoke.txt."
  write_template "$session_dir/visual-hardware-spotcheck.md" \
    "Visual Hardware Spotcheck" \
    "Inspect title, level select, HUD, board, units, effects, pause, victory, and defeat screens on release hardware after reviewing visual-smoke.txt."
  write_template "$session_dir/localization-review.md" \
    "Localization Review" \
    "Manually verify English and Chinese copy quality across menu, HUD, level select, gameplay, settings, pause, and end screens."
  write_template "$session_dir/input-bindings.md" \
    "Input Bindings" \
    "Review input-flow-audit.txt, then execute every binding from controls-audit.txt, including mouse placement, shovel, pause, fullscreen, volume, language, and restart."
  write_template "$session_dir/accessibility-qa.md" \
    "Accessibility QA" \
    "Review accessibility-audit.txt, then manually check assistive tech behavior, input remapping expectations, photosensitivity risk, and readability on release hardware."
  write_template "$session_dir/performance-qa.md" \
    "Performance QA" \
    "Review performance-audit.txt, then profile release hardware for frame pacing, long-session stability, memory growth, and worst-wave responsiveness."
  write_template "$session_dir/privacy-support.md" \
    "Privacy And Support QA" \
    "Review privacy-audit.txt, PRIVACY.md, SUPPORT.md, uninstall preservation, explicit --purge behavior, and bug-report evidence guidance."
  write_template "$session_dir/build-provenance.md" \
    "Build Provenance QA" \
    "Review release-provenance-audit.txt, BUILD_PROVENANCE.md, SHA256SUMS, release-info.txt, Cargo.lock usage, and package host details."
  write_template "$session_dir/save-compatibility.md" \
    "Save Compatibility" \
    "Verify old working-directory saves, XDG user-data saves, settings persistence, and BEVY_OPEN_SIEGE_SAVE_PATH override behavior."
  write_template "$session_dir/store-screenshots.md" \
    "Store Screenshot Review" \
    "Use store_screenshot_check.sh with STORE_SCREENSHOTS.md to capture or review the required 1920x1080 screenshot set, including English and Chinese coverage, real gameplay, HUD readability, and final storefront composition."
  write_template "$session_dir/store-press-ip-review.md" \
    "Store Press And IP Review" \
    "Confirm STORE_PAGE.md, PRESSKIT.md, branding art, capsule art, credits, notices, release notes, names, and store materials are final."
  write_template "$session_dir/final-art-direction.md" \
    "Final Art Direction" \
    "Review visible art direction and mechanics for product quality and separation from similar commercial games."

  echo "manual QA session initialized"
  echo "session directory: $session_dir"
  for file in "${manual_files[@]}"; do
    echo "created: $session_dir/$file"
  done
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
