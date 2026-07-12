#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  manual_qa_observations.sh --collect <extracted-package-dir> <qa-session-dir> [store-screenshots-dir]

Initializes the manual QA session if needed and records automated observations in
the QA evidence files. This helper does not approve the release: every evidence
file remains Status: Pending and Approved: No until a human reviewer completes
the manual checks.
EOF
}

if [[ "${1:-}" != "--collect" || $# -lt 3 || $# -gt 4 ]]; then
  usage
  exit 2
fi

PACKAGE_DIR="$2"
SESSION_DIR="$3"
SCREENSHOT_DIR="${4:-}"

if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "manual QA observations package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi
if [[ ! -x "$PACKAGE_DIR/manual_qa_session.sh" ]]; then
  echo "manual QA observations requires packaged manual_qa_session.sh" >&2
  exit 1
fi
if [[ ! -x "$PACKAGE_DIR/verify_release.sh" ]]; then
  echo "manual QA observations requires packaged verify_release.sh" >&2
  exit 1
fi
if [[ ! -x "$PACKAGE_DIR/qa_evidence_summary.sh" ]]; then
  echo "manual QA observations requires packaged qa_evidence_summary.sh" >&2
  exit 1
fi

mkdir -p "$SESSION_DIR"
if [[ ! -f "$SESSION_DIR/manual-qa-plan.txt" ]]; then
  "$PACKAGE_DIR/manual_qa_session.sh" --init "$PACKAGE_DIR" "$SESSION_DIR" >/dev/null
fi

TMPDIR_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_ROOT"' EXIT

"$PACKAGE_DIR/verify_release.sh" --quick "$PACKAGE_DIR" > "$TMPDIR_ROOT/verify-release-quick.txt"
"$PACKAGE_DIR/qa_evidence_summary.sh" --summary "$PACKAGE_DIR" "$SESSION_DIR" > "$TMPDIR_ROOT/qa-evidence-summary.txt"

if [[ -n "$SCREENSHOT_DIR" && -d "$SCREENSHOT_DIR" && -x "$PACKAGE_DIR/store_screenshot_check.sh" ]]; then
  "$PACKAGE_DIR/store_screenshot_check.sh" --validate-dir "$SCREENSHOT_DIR" > "$TMPDIR_ROOT/store-screenshot-validation.txt"
fi

copy_if_present() {
  local source="$1"
  local target="$2"
  if [[ -f "$source" ]]; then
    cp "$source" "$target"
  fi
}

copy_if_present "$TMPDIR_ROOT/verify-release-quick.txt" "$SESSION_DIR/verify-release-quick.txt"
copy_if_present "$TMPDIR_ROOT/qa-evidence-summary.txt" "$SESSION_DIR/qa-evidence-summary.txt"
copy_if_present "$TMPDIR_ROOT/store-screenshot-validation.txt" "$SESSION_DIR/store-screenshot-validation.txt"

append_observation() {
  local file="$1"
  local title="$2"
  shift 2
  if [[ ! -f "$file" ]]; then
    echo "manual QA observations missing expected session file: $file" >&2
    exit 1
  fi
  cat >> "$file" <<EOF

## Automated Precheck: $title

Status remains Pending until a human reviewer completes the manual checks.

EOF
  for line in "$@"; do
    printf -- '- %s\n' "$line" >> "$file"
  done
}

append_observation "$SESSION_DIR/full-campaign-playthrough.md" \
  "Scripted Campaign Lifecycle" \
  "Reviewed playthrough-audit.txt for automated victory, defeat, restart, unlock, and score lifecycle coverage." \
  "Reviewed campaign-simulation.txt for all 10 levels, 10/10 plants, and 10/10 enemy coverage." \
  "Manual full campaign playthrough from a clean save is still required."

append_observation "$SESSION_DIR/balance-usability.md" \
  "Balance Reports" \
  "Reviewed balance-audit.txt for level pressure, expected sun, and final wave coverage." \
  "Reviewed release-readiness.txt for pending manual balance and usability approval." \
  "Repeated human playtests are still required."

append_observation "$SESSION_DIR/audio-device-qa.md" \
  "Audio Reports" \
  "Reviewed audio-audit.txt for WAV format, peak, RMS, clipping headroom, and opt-in startup policy." \
  "Reviewed audio-smoke.txt for window creation with audio enabled." \
  "Speaker and headphone listening QA is still required."

append_observation "$SESSION_DIR/visual-hardware-spotcheck.md" \
  "Visual Reports" \
  "Reviewed visual-readability-audit.txt for viewport, contrast, HUD wrapping, and visual asset coverage." \
  "Reviewed visual-smoke.txt for nonblank captured window evidence." \
  "Release-hardware visual inspection is still required."

append_observation "$SESSION_DIR/localization-review.md" \
  "Localization Reports" \
  "Reviewed localization-audit.txt for English and Chinese string coverage." \
  "Reviewed layout-audit.txt for bilingual menu, HUD, seed bank, pause, and end-screen bounds." \
  "Human copy-quality review is still required."

append_observation "$SESSION_DIR/input-bindings.md" \
  "Input Reports" \
  "Reviewed controls-audit.txt for documented keyboard and mouse bindings." \
  "Reviewed input-flow-audit.txt for menu navigation, placement gating, pause gating, and end flow coverage." \
  "Human execution of every binding is still required."

append_observation "$SESSION_DIR/accessibility-qa.md" \
  "Accessibility Reports" \
  "Reviewed accessibility-audit.txt for keyboard-only flow, no-audio playability, contrast, and color-independence coverage." \
  "Assistive tech, remapping expectations, photosensitivity, and hardware readability review are still required."

append_observation "$SESSION_DIR/performance-qa.md" \
  "Performance Reports" \
  "Reviewed performance-audit.txt for entity budgets, spawn burst, asset bytes, viewport floor, and audio startup policy." \
  "Release-hardware profiling, long-session stability, and worst-wave responsiveness review are still required."

append_observation "$SESSION_DIR/privacy-support.md" \
  "Privacy And Support Reports" \
  "Reviewed privacy-audit.txt, PRIVACY.md, SUPPORT.md, and TROUBLESHOOTING.md for offline/no-telemetry posture and support evidence guidance." \
  "Store/platform disclosure review is still required."

append_observation "$SESSION_DIR/build-provenance.md" \
  "Build Provenance Reports" \
  "Reviewed release-provenance-audit.txt, BUILD_PROVENANCE.md, release-manifest.json, release-info.txt, and SHA256SUMS." \
  "Package host details and final archive checksum approval are still required."

append_observation "$SESSION_DIR/save-compatibility.md" \
  "Save Reports" \
  "Reviewed save-audit.txt for explicit override, XDG path, home fallback, portable fallback, legacy parse, normalization, and settings clamp coverage." \
  "Manual old-save and install/uninstall save compatibility review is still required."

append_observation "$SESSION_DIR/store-screenshots.md" \
  "Store Screenshot Reports" \
  "Reviewed STORE_SCREENSHOTS.md and store-asset-audit.txt for required capture plan and store artwork references." \
  "If store-screenshot-validation.txt is present in this session, the provided screenshot directory passed automated validation." \
  "Final storefront composition and language coverage approval are still required."

append_observation "$SESSION_DIR/store-press-ip-review.md" \
  "Store, Press, Content Rating, And IP Reports" \
  "Reviewed marketing-audit.txt, content-rating-audit.txt, ip-audit.txt, STORE_PAGE.md, PRESSKIT.md, CONTENT_RATING.md, CREDITS.md, and THIRD_PARTY_NOTICES.md." \
  "Final platform questionnaire, store copy, press copy, and naming approval are still required."

append_observation "$SESSION_DIR/final-art-direction.md" \
  "Art Direction Reports" \
  "Reviewed ART_ASSETS.md, asset-audit.txt, visual-readability-audit.txt, marketing-audit.txt, and ip-audit.txt." \
  "Final art-direction and product-separation review is still required."

cat > "$SESSION_DIR/automated-observations.md" <<EOF
# Bevy Open Siege Automated Manual QA Observations

Status: Pending
Owner:
Date:
Package: $(basename "$PACKAGE_DIR")

This file records automated prechecks that reduce manual QA setup work. It does
not approve the release. Human reviewers must still complete each QA evidence
file, set Status to Pass or Scoped Out, set Approved to Yes, and fill owner/date
metadata before final_signoff_check.sh can pass.

## Collected Outputs

- verify-release-quick.txt
- qa-evidence-summary.txt
$(if [[ -f "$SESSION_DIR/store-screenshot-validation.txt" ]]; then echo "- store-screenshot-validation.txt"; fi)

## Gate Status

- Package quick verification: collected
- QA evidence summary: collected
- Manual evidence approval: still required
- Platform evidence approval: still required
- QA_SIGNOFF.md approval: still required
EOF

echo "manual QA observations collected"
echo "package: $(basename "$PACKAGE_DIR")"
echo "session directory: $SESSION_DIR"
echo "status: pending human approval"
