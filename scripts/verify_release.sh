#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  verify_release.sh [--quick|--full] [package-dir]

--quick verifies archive integrity, release metadata, and deterministic text audit reports.
--full also runs runtime, visual, audio, and Linux portability startup smoke helpers.
EOF
}

MODE="--quick"
PACKAGE_DIR="."
if [[ $# -gt 0 ]]; then
  case "$1" in
    --quick|--full)
      MODE="$1"
      shift
      ;;
  esac
fi
if [[ $# -gt 0 ]]; then
  PACKAGE_DIR="$1"
  shift
fi
if [[ $# -ne 0 ]]; then
  usage
  exit 2
fi

if [[ "$MODE" != "--quick" && "$MODE" != "--full" ]]; then
  usage
  exit 2
fi
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

cd "$PACKAGE_DIR"
PACKAGE_ROOT="$(pwd)"

if [[ -x ./bevy_open_siege.exe ]]; then
  BINARY="./bevy_open_siege.exe"
elif [[ -x ./bevy_open_siege ]]; then
  BINARY="./bevy_open_siege"
else
  echo "release package binary is not executable" >&2
  exit 1
fi

required_files=(
  "README.md"
  "LICENSE"
  "RELEASE_NOTES.md"
  "QA_SIGNOFF.md"
  "CONTENT_RATING.md"
  "TROUBLESHOOTING.md"
  "SHA256SUMS"
  "release-info.txt"
  "release-readiness.txt"
  "balance-audit.txt"
  "asset-audit.txt"
  "audio-audit.txt"
  "controls-audit.txt"
  "input-flow-audit.txt"
  "localization-audit.txt"
  "layout-audit.txt"
  "visual-readability-audit.txt"
  "accessibility-audit.txt"
  "performance-audit.txt"
  "privacy-audit.txt"
  "release-provenance-audit.txt"
  "marketing-audit.txt"
  "ip-audit.txt"
  "save-audit.txt"
  "playthrough-audit.txt"
  "campaign-simulation.txt"
  "store-asset-audit.txt"
  "content-rating-audit.txt"
  "assets/models/plants/sprout-slinger.glb"
  "assets/models/monsters/walker.glb"
  "manual_qa_session.sh"
  "platform_package_plan.sh"
  "qa_evidence_summary.sh"
  "qa_signoff_prepare.sh"
  "final_signoff_check.sh"
  "store_asset_audit.sh"
  "content_rating_audit.sh"
  "support_diagnostics.sh"
  "signoff_bundle.sh"
  "create_candidate_evidence.sh"
  "create_store_submission_pack.sh"
  "release-manifest.json"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "release verification missing required file: $file" >&2
    exit 1
  fi
done

grep -q "Bevy Open Siege Troubleshooting" TROUBLESHOOTING.md
grep -q "verify_release.sh --quick" TROUBLESHOOTING.md
grep -q "runtime_smoke.sh" TROUBLESHOOTING.md
grep -q "audio_smoke.sh" TROUBLESHOOTING.md
grep -q '"schema": "bevy-open-siege-release-manifest-v1"' release-manifest.json
grep -q '"package": "' release-manifest.json
grep -q '"binary": "' release-manifest.json
grep -q '"final_approval_required": true' release-manifest.json
grep -q '"path": "release-readiness.txt"' release-manifest.json
grep -q '"path": "TROUBLESHOOTING.md"' release-manifest.json
grep -q "support diagnostics collected" support_diagnostics.sh
grep -q "no save files, screenshots, recordings, or personal files" support_diagnostics.sh
grep -q "store asset audit ok" store_asset_audit.sh
grep -q "assets/models/plants/sprout-slinger.glb" asset-audit.txt
grep -q "assets/models/monsters/walker.glb" asset-audit.txt
grep -q "glb assets: 20" asset-audit.txt
grep -q "content rating audit ok" content_rating_audit.sh
grep -q "signoff bundle created" signoff_bundle.sh
grep -q "final_signoff_check.sh --check" signoff_bundle.sh
grep -q "candidate evidence created" create_candidate_evidence.sh
grep -q "release candidate, not final approved" create_candidate_evidence.sh
grep -q "store submission pack created" create_store_submission_pack.sh
grep -q "screenshot_status:" create_store_submission_pack.sh
grep -q "qa signoff prepare check ok" qa_signoff_prepare.sh
grep -q "does not bypass QA" qa_signoff_prepare.sh
grep -q "See packaged SHA256SUMS for per-file hashes" qa_signoff_prepare.sh
grep -q "archive-sha256 must be a 64-character SHA256 hex digest" qa_signoff_prepare.sh
if [[ "$(basename "$PACKAGE_ROOT")" == *"-linux-x86_64" ]]; then
  for file in bevy_open_siege.bin lib/ld-linux-x86-64.so.2 linux-dependency-audit.txt linux_dependency_audit.sh linux-portability-smoke.txt linux_portability_smoke.sh linux-clean-distro-smoke.txt linux_clean_distro_smoke.sh linux-metadata-audit.txt linux_metadata_audit.sh; do
    if [[ ! -f "$file" ]]; then
      echo "release verification missing required Linux package file: $file" >&2
      exit 1
    fi
  done
  grep -q "linux dependency audit ok" linux_dependency_audit.sh
  grep -q "linux portability smoke ok" linux_portability_smoke.sh
  grep -q "linux clean distro smoke ok" linux_clean_distro_smoke.sh
  grep -q "linux metadata audit ok" linux_metadata_audit.sh
fi

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum -c SHA256SUMS >/dev/null
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 -c SHA256SUMS >/dev/null
else
  echo "release verification requires sha256sum or shasum" >&2
  exit 1
fi

TMPDIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMPDIR"
}
trap cleanup EXIT

"$BINARY" --validate-data >/dev/null

compare_report() {
  local flag="$1"
  local expected="$2"
  local actual="$TMPDIR/${expected}.actual"
  "$BINARY" "$flag" > "$actual"
  if ! diff -u "$expected" "$actual" > "$TMPDIR/${expected}.diff"; then
    echo "release verification report mismatch: $expected" >&2
    cat "$TMPDIR/${expected}.diff" >&2
    exit 1
  fi
}

compare_report --audit-balance balance-audit.txt
compare_report --audit-assets asset-audit.txt
compare_report --audit-audio audio-audit.txt
compare_report --audit-controls controls-audit.txt
compare_report --audit-input-flow input-flow-audit.txt
compare_report --audit-localization localization-audit.txt
compare_report --audit-layout layout-audit.txt
compare_report --audit-visual visual-readability-audit.txt
compare_report --audit-accessibility accessibility-audit.txt
compare_report --audit-performance performance-audit.txt
compare_report --audit-privacy privacy-audit.txt
compare_report --audit-release-provenance release-provenance-audit.txt
compare_report --audit-marketing marketing-audit.txt
compare_report --audit-ip ip-audit.txt
compare_report --audit-save save-audit.txt
compare_report --audit-playthrough playthrough-audit.txt
compare_report --simulate-campaign campaign-simulation.txt
compare_report --release-readiness release-readiness.txt
compare_report --print-release-info release-info.txt

./store_asset_audit.sh "$PACKAGE_ROOT" > "$TMPDIR/store-asset-audit.actual.txt"
if ! diff -u store-asset-audit.txt "$TMPDIR/store-asset-audit.actual.txt" > "$TMPDIR/store-asset-audit.diff"; then
  echo "release verification report mismatch: store-asset-audit.txt" >&2
  cat "$TMPDIR/store-asset-audit.diff" >&2
  exit 1
fi

./content_rating_audit.sh "$PACKAGE_ROOT" > "$TMPDIR/content-rating-audit.actual.txt"
if ! diff -u content-rating-audit.txt "$TMPDIR/content-rating-audit.actual.txt" > "$TMPDIR/content-rating-audit.diff"; then
  echo "release verification report mismatch: content-rating-audit.txt" >&2
  cat "$TMPDIR/content-rating-audit.diff" >&2
  exit 1
fi

if [[ "$(basename "$PACKAGE_ROOT")" == *"-linux-x86_64" ]]; then
  ./linux_dependency_audit.sh "$PACKAGE_ROOT" > "$TMPDIR/linux-dependency-audit.actual.txt"
  if ! diff -u linux-dependency-audit.txt "$TMPDIR/linux-dependency-audit.actual.txt" > "$TMPDIR/linux-dependency-audit.diff"; then
    echo "release verification report mismatch: linux-dependency-audit.txt" >&2
    cat "$TMPDIR/linux-dependency-audit.diff" >&2
    exit 1
  fi
  if [[ "$MODE" == "--full" ]]; then
    ./linux_portability_smoke.sh "$PACKAGE_ROOT" 12 > "$TMPDIR/linux-portability-smoke.actual.txt"
    if ! diff -u linux-portability-smoke.txt "$TMPDIR/linux-portability-smoke.actual.txt" > "$TMPDIR/linux-portability-smoke.diff"; then
      echo "release verification report mismatch: linux-portability-smoke.txt" >&2
      cat "$TMPDIR/linux-portability-smoke.diff" >&2
      exit 1
    fi
  else
    grep -q "linux portability smoke ok" linux-portability-smoke.txt
    grep -q "sanitized_env: LD_LIBRARY_PATH unset, Nix variables omitted" linux-portability-smoke.txt
    grep -q "clean_distro_qa: still required" linux-portability-smoke.txt
  fi
  ./linux_clean_distro_smoke.sh "$PACKAGE_ROOT" > "$TMPDIR/linux-clean-distro-smoke.actual.txt"
  if ! diff -u linux-clean-distro-smoke.txt "$TMPDIR/linux-clean-distro-smoke.actual.txt" > "$TMPDIR/linux-clean-distro-smoke.diff"; then
    echo "release verification report mismatch: linux-clean-distro-smoke.txt" >&2
    cat "$TMPDIR/linux-clean-distro-smoke.diff" >&2
    exit 1
  fi
  ./linux_metadata_audit.sh "$PACKAGE_ROOT" > "$TMPDIR/linux-metadata-audit.actual.txt"
  if ! diff -u linux-metadata-audit.txt "$TMPDIR/linux-metadata-audit.actual.txt" > "$TMPDIR/linux-metadata-audit.diff"; then
    echo "release verification report mismatch: linux-metadata-audit.txt" >&2
    cat "$TMPDIR/linux-metadata-audit.diff" >&2
    exit 1
  fi
fi

./manual_qa_session.sh --plan "$PACKAGE_ROOT" > "$TMPDIR/manual-qa-plan.actual.txt"
if ! diff -u manual-qa-plan.txt "$TMPDIR/manual-qa-plan.actual.txt" > "$TMPDIR/manual-qa-plan.diff"; then
  echo "release verification report mismatch: manual-qa-plan.txt" >&2
  cat "$TMPDIR/manual-qa-plan.diff" >&2
  exit 1
fi
./platform_package_plan.sh --plan "$PACKAGE_ROOT" > "$TMPDIR/platform-package-plan.actual.txt"
if ! diff -u platform-package-plan.txt "$TMPDIR/platform-package-plan.actual.txt" > "$TMPDIR/platform-package-plan.diff"; then
  echo "release verification report mismatch: platform-package-plan.txt" >&2
  cat "$TMPDIR/platform-package-plan.diff" >&2
  exit 1
fi
./final_signoff_check.sh --plan "$PACKAGE_ROOT" > "$TMPDIR/final-signoff-plan.actual.txt"
if ! diff -u final-signoff-plan.txt "$TMPDIR/final-signoff-plan.actual.txt" > "$TMPDIR/final-signoff-plan.diff"; then
  echo "release verification report mismatch: final-signoff-plan.txt" >&2
  cat "$TMPDIR/final-signoff-plan.diff" >&2
  exit 1
fi
./qa_evidence_summary.sh --summary "$PACKAGE_ROOT" > "$TMPDIR/qa-evidence-summary.actual.txt"
grep -q "qa evidence summary ok" "$TMPDIR/qa-evidence-summary.actual.txt"
BEVY_OPEN_SIEGE_DIAGNOSTICS_SKIP_VERIFY=1 ./support_diagnostics.sh "$PACKAGE_ROOT" "$TMPDIR/support-diagnostics" > "$TMPDIR/support-diagnostics.out"
grep -q "support diagnostics collected" "$TMPDIR/support-diagnostics.out"
test -f "$TMPDIR/support-diagnostics/release-info.txt"
test -f "$TMPDIR/support-diagnostics/validate-data.txt"
test -f "$TMPDIR/support-diagnostics/sha256sum.txt"
grep -q "quick verification skipped" "$TMPDIR/support-diagnostics/verify-release-quick.txt"

if [[ "$MODE" == "--full" ]]; then
  ./runtime_smoke.sh "$BINARY" 12 > "$TMPDIR/runtime-smoke.actual.txt"
  if ! diff -u runtime-smoke.txt "$TMPDIR/runtime-smoke.actual.txt" > "$TMPDIR/runtime-smoke.diff"; then
    echo "release verification report mismatch: runtime-smoke.txt" >&2
    cat "$TMPDIR/runtime-smoke.diff" >&2
    exit 1
  fi
  ./visual_smoke.sh "$BINARY" 15 > "$TMPDIR/visual-smoke.actual.txt"
  if ! diff -u visual-smoke.txt "$TMPDIR/visual-smoke.actual.txt" > "$TMPDIR/visual-smoke.diff"; then
    echo "release verification report mismatch: visual-smoke.txt" >&2
    cat "$TMPDIR/visual-smoke.diff" >&2
    exit 1
  fi
  ./audio_smoke.sh "$BINARY" 12 > "$TMPDIR/audio-smoke.actual.txt"
  if ! diff -u audio-smoke.txt "$TMPDIR/audio-smoke.actual.txt" > "$TMPDIR/audio-smoke.diff"; then
    echo "release verification report mismatch: audio-smoke.txt" >&2
    cat "$TMPDIR/audio-smoke.diff" >&2
    exit 1
  fi
fi

echo "release package verification ok"
echo "mode: ${MODE#--}"
echo "binary: $BINARY"
echo "integrity: SHA256SUMS passed"
echo "deterministic reports: matched"
echo "release manifest: present"
if [[ "$MODE" == "--full" ]]; then
  echo "window smoke: runtime, visual, audio matched"
else
  echo "window smoke: skipped"
fi
echo "final approval: still requires manual/platform QA and final_signoff_check.sh --check"
