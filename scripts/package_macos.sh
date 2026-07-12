#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)"
PACKAGE="bevy_open_siege-${VERSION}-macos-universal"
DIST="$ROOT/dist/$PACKAGE"
BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
X86_TARGET="x86_64-apple-darwin"
ARM_TARGET="aarch64-apple-darwin"
X86_BINARY="$BUILD_TARGET_DIR/$X86_TARGET/release/bevy_open_siege"
ARM_BINARY="$BUILD_TARGET_DIR/$ARM_TARGET/release/bevy_open_siege"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS package must be built on macOS" >&2
  exit 1
fi

cargo build --release --target "$X86_TARGET"
cargo build --release --target "$ARM_TARGET"

if [[ ! -x "$X86_BINARY" || ! -x "$ARM_BINARY" ]]; then
  echo "macOS target binaries were not produced" >&2
  exit 1
fi

mkdir -p "$ROOT/dist"
rm -rf "$DIST"
mkdir -p "$DIST/assets"
lipo -create "$X86_BINARY" "$ARM_BINARY" -output "$DIST/bevy_open_siege"
chmod +x "$DIST/bevy_open_siege"
RELEASE_BINARY="$DIST/bevy_open_siege"

"$RELEASE_BINARY" --validate-data
"$RELEASE_BINARY" --audit-balance > "$ROOT/dist/balance-audit.txt"
"$RELEASE_BINARY" --audit-assets > "$ROOT/dist/asset-audit.txt"
"$RELEASE_BINARY" --audit-audio > "$ROOT/dist/audio-audit.txt"
"$RELEASE_BINARY" --audit-controls > "$ROOT/dist/controls-audit.txt"
"$RELEASE_BINARY" --audit-input-flow > "$ROOT/dist/input-flow-audit.txt"
"$RELEASE_BINARY" --audit-localization > "$ROOT/dist/localization-audit.txt"
"$RELEASE_BINARY" --audit-layout > "$ROOT/dist/layout-audit.txt"
"$RELEASE_BINARY" --audit-visual > "$ROOT/dist/visual-readability-audit.txt"
"$RELEASE_BINARY" --audit-accessibility > "$ROOT/dist/accessibility-audit.txt"
"$RELEASE_BINARY" --audit-performance > "$ROOT/dist/performance-audit.txt"
"$RELEASE_BINARY" --audit-privacy > "$ROOT/dist/privacy-audit.txt"
"$RELEASE_BINARY" --audit-release-provenance > "$ROOT/dist/release-provenance-audit.txt"
"$RELEASE_BINARY" --audit-marketing > "$ROOT/dist/marketing-audit.txt"
"$RELEASE_BINARY" --audit-ip > "$ROOT/dist/ip-audit.txt"
"$RELEASE_BINARY" --audit-save > "$ROOT/dist/save-audit.txt"
"$RELEASE_BINARY" --audit-playthrough > "$ROOT/dist/playthrough-audit.txt"
"$RELEASE_BINARY" --simulate-campaign > "$ROOT/dist/campaign-simulation.txt"
"$RELEASE_BINARY" --release-readiness > "$ROOT/dist/release-readiness.txt"
"$RELEASE_BINARY" --print-release-info > "$ROOT/dist/release-info.txt"
python3 "$ROOT/scripts/generate_third_party_licenses.py" > "$ROOT/dist/THIRD_PARTY_LICENSES.md"

cp "$ROOT/README.md" "$DIST/"
cp "$ROOT/LICENSE" "$DIST/"
cp "$ROOT/CREDITS.md" "$DIST/"
cp "$ROOT/ART_ASSETS.md" "$DIST/"
cp "$ROOT/THIRD_PARTY_NOTICES.md" "$DIST/"
cp "$ROOT/STORE_PAGE.md" "$DIST/"
cp "$ROOT/STORE_SCREENSHOTS.md" "$DIST/"
cp "$ROOT/CONTENT_RATING.md" "$DIST/"
cp "$ROOT/PRESSKIT.md" "$DIST/"
cp "$ROOT/RELEASE_CHECKLIST.md" "$DIST/"
cp "$ROOT/RELEASE_NOTES.md" "$DIST/"
cp "$ROOT/QA_SIGNOFF.md" "$DIST/"
cp "$ROOT/PRIVACY.md" "$DIST/"
cp "$ROOT/SUPPORT.md" "$DIST/"
cp "$ROOT/TROUBLESHOOTING.md" "$DIST/"
cp "$ROOT/BUILD_PROVENANCE.md" "$DIST/"
cp "$ROOT/VERSION.ron" "$DIST/"
cp "$ROOT/dist/THIRD_PARTY_LICENSES.md" "$DIST/"
cp "$ROOT/dist/release-info.txt" "$DIST/"
cp "$ROOT/dist/balance-audit.txt" "$DIST/"
cp "$ROOT/dist/asset-audit.txt" "$DIST/"
cp "$ROOT/dist/audio-audit.txt" "$DIST/"
cp "$ROOT/dist/controls-audit.txt" "$DIST/"
cp "$ROOT/dist/input-flow-audit.txt" "$DIST/"
cp "$ROOT/dist/localization-audit.txt" "$DIST/"
cp "$ROOT/dist/layout-audit.txt" "$DIST/"
cp "$ROOT/dist/visual-readability-audit.txt" "$DIST/"
cp "$ROOT/dist/accessibility-audit.txt" "$DIST/"
cp "$ROOT/dist/performance-audit.txt" "$DIST/"
cp "$ROOT/dist/privacy-audit.txt" "$DIST/"
cp "$ROOT/dist/release-provenance-audit.txt" "$DIST/"
cp "$ROOT/dist/marketing-audit.txt" "$DIST/"
cp "$ROOT/dist/ip-audit.txt" "$DIST/"
cp "$ROOT/dist/save-audit.txt" "$DIST/"
cp "$ROOT/dist/playthrough-audit.txt" "$DIST/"
cp "$ROOT/dist/campaign-simulation.txt" "$DIST/"
cp "$ROOT/dist/release-readiness.txt" "$DIST/"
cp "$ROOT/scripts/store_asset_audit.sh" "$DIST/"
cp "$ROOT/scripts/content_rating_audit.sh" "$DIST/"
cp "$ROOT/scripts/runtime_smoke.sh" "$DIST/"
cp "$ROOT/scripts/visual_smoke.sh" "$DIST/"
cp "$ROOT/scripts/store_screenshot_check.sh" "$DIST/"
cp "$ROOT/scripts/audio_smoke.sh" "$DIST/"
cp "$ROOT/scripts/manual_qa_session.sh" "$DIST/"
cp "$ROOT/scripts/platform_package_plan.sh" "$DIST/"
cp "$ROOT/scripts/qa_evidence_summary.sh" "$DIST/"
cp "$ROOT/scripts/final_signoff_check.sh" "$DIST/"
cp "$ROOT/scripts/verify_release.sh" "$DIST/"
cp "$ROOT/scripts/support_diagnostics.sh" "$DIST/"
cp "$ROOT/scripts/signoff_bundle.sh" "$DIST/"
cp "$ROOT/scripts/create_candidate_evidence.sh" "$DIST/"
cp "$ROOT/scripts/create_store_submission_pack.sh" "$DIST/"
cp "$ROOT/scripts/package_windows.ps1" "$DIST/"
cp "$ROOT/scripts/package_macos.sh" "$DIST/"
chmod +x "$DIST/runtime_smoke.sh" "$DIST/visual_smoke.sh" "$DIST/store_screenshot_check.sh" "$DIST/store_asset_audit.sh" "$DIST/content_rating_audit.sh" "$DIST/audio_smoke.sh" "$DIST/manual_qa_session.sh" "$DIST/platform_package_plan.sh" "$DIST/qa_evidence_summary.sh" "$DIST/final_signoff_check.sh" "$DIST/verify_release.sh" "$DIST/support_diagnostics.sh" "$DIST/signoff_bundle.sh" "$DIST/create_candidate_evidence.sh" "$DIST/create_store_submission_pack.sh" "$DIST/package_macos.sh"
cp "$ROOT/assets/manifest.ron" "$DIST/assets/"
cp -R "$ROOT/assets/art" "$DIST/assets/"
cp -R "$ROOT/assets/audio" "$DIST/assets/"
cp -R "$ROOT/assets/branding" "$DIST/assets/"
cp -R "$ROOT/assets/data" "$DIST/assets/"
cp -R "$ROOT/assets/i18n" "$DIST/assets/"
cp -R "$ROOT/assets/models" "$DIST/assets/"

"$ROOT/scripts/runtime_smoke.sh" "$RELEASE_BINARY" 12 > "$DIST/runtime-smoke.txt"
"$ROOT/scripts/visual_smoke.sh" "$RELEASE_BINARY" 15 > "$DIST/visual-smoke.txt"
"$ROOT/scripts/audio_smoke.sh" "$RELEASE_BINARY" 12 > "$DIST/audio-smoke.txt"
"$ROOT/scripts/store_asset_audit.sh" "$DIST" > "$DIST/store-asset-audit.txt"
"$ROOT/scripts/content_rating_audit.sh" "$DIST" > "$DIST/content-rating-audit.txt"
"$ROOT/scripts/manual_qa_session.sh" --plan "$DIST" > "$DIST/manual-qa-plan.txt"
"$ROOT/scripts/platform_package_plan.sh" --plan "$DIST" > "$DIST/platform-package-plan.txt"
"$ROOT/scripts/final_signoff_check.sh" --plan "$DIST" > "$DIST/final-signoff-plan.txt"
python3 "$ROOT/scripts/generate_release_manifest.py" "$DIST" "macos-universal" > "$DIST/release-manifest.json"

(
  cd "$DIST"
  find . -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 shasum -a 256 \
    | sed 's#  ./#  #' > SHA256SUMS
)

tar -C "$ROOT/dist" -czf "$ROOT/dist/${PACKAGE}.tar.gz" "$PACKAGE"
echo "Created dist/${PACKAGE}.tar.gz"
