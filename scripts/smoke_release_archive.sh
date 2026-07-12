#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <release-archive.tar.gz>" >&2
  exit 2
fi

ARCHIVE="$1"
if [[ ! -f "$ARCHIVE" ]]; then
  echo "release archive not found: $ARCHIVE" >&2
  exit 1
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

tar -xzf "$ARCHIVE" -C "$TMPDIR"

PACKAGE_DIR="$(find "$TMPDIR" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
if [[ -z "$PACKAGE_DIR" ]]; then
  echo "release archive does not contain a top-level package directory" >&2
  exit 1
fi
PACKAGE_NAME="$(basename "$PACKAGE_DIR")"
PACKAGE_VERSION="${PACKAGE_NAME#bevy_open_siege-}"
PACKAGE_VERSION="${PACKAGE_VERSION%-linux-x86_64}"

complete_evidence_file() {
  local path="$1"
  local status="$2"
  local owner="$3"
  local date="$4"
  local notes="$5"
  sed -i \
    -e "s/^Status:.*/Status: $status/" \
    -e "s/^Owner:.*/Owner: $owner/" \
    -e "s/^Date:.*/Date: $date/" \
    -e "s/^Approved:.*/Approved: Yes/" \
    -e "s/^Approver:.*/Approver: $owner/" \
    -e "s/^Notes:.*/Notes: $notes/" \
    "$path"
  if grep -q '^Follow-up required:' "$path"; then
    sed -i 's/^Follow-up required:.*/Follow-up required: No/' "$path"
  else
    printf '\nFollow-up required: No\n' >> "$path"
  fi
}

required_files=(
  "bevy_open_siege"
  "bevy_open_siege.bin"
  "README.md"
  "LICENSE"
  "CREDITS.md"
  "ART_ASSETS.md"
  "THIRD_PARTY_NOTICES.md"
  "THIRD_PARTY_LICENSES.md"
  "STORE_PAGE.md"
  "STORE_SCREENSHOTS.md"
  "CONTENT_RATING.md"
  "PRESSKIT.md"
  "RELEASE_CHECKLIST.md"
  "RELEASE_NOTES.md"
  "QA_SIGNOFF.md"
  "PRIVACY.md"
  "SUPPORT.md"
  "TROUBLESHOOTING.md"
  "BUILD_PROVENANCE.md"
  "VERSION.ron"
  "release-info.txt"
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
  "release-readiness.txt"
  "runtime-smoke.txt"
  "visual-smoke.txt"
  "audio-smoke.txt"
  "store-asset-audit.txt"
  "content-rating-audit.txt"
  "linux-package-audit.txt"
  "linux-install-smoke.txt"
  "linux-dependency-audit.txt"
  "linux-portability-smoke.txt"
  "linux-clean-distro-smoke.txt"
  "linux-metadata-audit.txt"
  "manual-qa-plan.txt"
  "platform-package-plan.txt"
  "final-signoff-plan.txt"
  "release-manifest.json"
  "SHA256SUMS"
  "install_linux_user.sh"
  "uninstall_linux_user.sh"
  "audio_smoke.sh"
  "runtime_smoke.sh"
  "visual_smoke.sh"
  "store_screenshot_check.sh"
  "store_asset_audit.sh"
  "content_rating_audit.sh"
  "linux_package_audit.sh"
  "linux_install_smoke.sh"
  "linux_dependency_audit.sh"
  "linux_portability_smoke.sh"
  "linux_clean_distro_smoke.sh"
  "linux_metadata_audit.sh"
  "manual_qa_session.sh"
  "manual_qa_observations.sh"
  "platform_package_plan.sh"
  "qa_evidence_summary.sh"
  "qa_signoff_prepare.sh"
  "final_signoff_check.sh"
  "verify_release.sh"
  "support_diagnostics.sh"
  "signoff_bundle.sh"
  "create_candidate_evidence.sh"
  "create_store_submission_pack.sh"
  "package_windows.ps1"
  "package_macos.sh"
  "assets/manifest.ron"
  "assets/art/plants-sheet.png"
  "assets/art/monsters-sheet.png"
  "assets/art/effects/pea.png"
  "assets/art/effects/frost-pod.png"
  "assets/art/effects/cabbage.png"
  "assets/art/effects/sun.png"
  "assets/art/effects/fire.png"
  "assets/art/effects/explosion.png"
  "assets/art/environment/lawn-base.png"
  "assets/art/environment/lane-grass.png"
  "assets/art/environment/soil-border.png"
  "assets/art/ui/menu-panel.png"
  "assets/art/ui/hud-panel.png"
  "assets/art/ui/end-panel.png"
  "assets/models/plants/sprout-slinger.glb"
  "assets/models/plants/sunbloom.glb"
  "assets/models/plants/bark-bulwark.glb"
  "assets/models/plants/frost-sprout.glb"
  "assets/models/plants/twin-pod.glb"
  "assets/models/plants/leaf-lobber.glb"
  "assets/models/plants/briar-mat.glb"
  "assets/models/plants/blast-berry.glb"
  "assets/models/plants/ember-stump.glb"
  "assets/models/plants/scent-root.glb"
  "assets/models/monsters/walker.glb"
  "assets/models/monsters/conehead.glb"
  "assets/models/monsters/runner.glb"
  "assets/models/monsters/buckethead.glb"
  "assets/models/monsters/brute.glb"
  "assets/models/monsters/healer.glb"
  "assets/models/monsters/jumper.glb"
  "assets/models/monsters/digger.glb"
  "assets/models/monsters/frostbite.glb"
  "assets/models/monsters/gargantuar.glb"
  "assets/audio/music-loop.wav"
  "assets/audio/plant-place.wav"
  "assets/audio/shoot.wav"
  "assets/audio/sun-collect.wav"
  "assets/audio/monster-down.wav"
  "assets/audio/victory.wav"
  "assets/audio/defeat.wav"
  "assets/branding/icon.svg"
  "assets/branding/capsule.svg"
  "assets/branding/generated/app-icon.png"
  "assets/branding/generated/store-capsule.png"
  "assets/data/levels.ron"
  "assets/i18n/en.ron"
  "assets/i18n/zh.ron"
  "assets/linux/bevy-open-siege.desktop"
  "assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$PACKAGE_DIR/$file" ]]; then
    echo "release archive missing required file: $file" >&2
    exit 1
  fi
done

if [[ ! -x "$PACKAGE_DIR/bevy_open_siege" || ! -x "$PACKAGE_DIR/bevy_open_siege.bin" ]]; then
  echo "release entrypoint and payload must be executable" >&2
  exit 1
fi
if [[ ! -x "$PACKAGE_DIR/lib/ld-linux-x86-64.so.2" ]]; then
  echo "release archive missing bundled Linux loader" >&2
  exit 1
fi
if [[ ! -x "$PACKAGE_DIR/install_linux_user.sh" || ! -x "$PACKAGE_DIR/uninstall_linux_user.sh" ]]; then
  echo "install and uninstall scripts must be executable" >&2
  exit 1
fi
if [[ ! -x "$PACKAGE_DIR/audio_smoke.sh" || ! -x "$PACKAGE_DIR/runtime_smoke.sh" || ! -x "$PACKAGE_DIR/visual_smoke.sh" || ! -x "$PACKAGE_DIR/store_screenshot_check.sh" || ! -x "$PACKAGE_DIR/store_asset_audit.sh" || ! -x "$PACKAGE_DIR/content_rating_audit.sh" || ! -x "$PACKAGE_DIR/linux_package_audit.sh" || ! -x "$PACKAGE_DIR/linux_install_smoke.sh" || ! -x "$PACKAGE_DIR/linux_dependency_audit.sh" || ! -x "$PACKAGE_DIR/linux_portability_smoke.sh" || ! -x "$PACKAGE_DIR/linux_clean_distro_smoke.sh" || ! -x "$PACKAGE_DIR/linux_metadata_audit.sh" || ! -x "$PACKAGE_DIR/manual_qa_session.sh" || ! -x "$PACKAGE_DIR/manual_qa_observations.sh" || ! -x "$PACKAGE_DIR/platform_package_plan.sh" || ! -x "$PACKAGE_DIR/qa_evidence_summary.sh" || ! -x "$PACKAGE_DIR/qa_signoff_prepare.sh" || ! -x "$PACKAGE_DIR/final_signoff_check.sh" || ! -x "$PACKAGE_DIR/verify_release.sh" || ! -x "$PACKAGE_DIR/support_diagnostics.sh" || ! -x "$PACKAGE_DIR/signoff_bundle.sh" || ! -x "$PACKAGE_DIR/create_candidate_evidence.sh" || ! -x "$PACKAGE_DIR/create_store_submission_pack.sh" || ! -x "$PACKAGE_DIR/package_macos.sh" ]]; then
  echo "audio, runtime, visual, screenshot, store asset, content rating, linux package, linux install smoke, linux dependency, linux portability, clean distro smoke, linux metadata, manual QA, manual QA observations, platform package, QA signoff prepare, final signoff, support diagnostics, signoff bundle, candidate evidence, store submission, and macOS package helper scripts must be executable" >&2
  exit 1
fi

bash -n "$PACKAGE_DIR/install_linux_user.sh"
bash -n "$PACKAGE_DIR/uninstall_linux_user.sh"
bash -n "$PACKAGE_DIR/audio_smoke.sh"
bash -n "$PACKAGE_DIR/runtime_smoke.sh"
bash -n "$PACKAGE_DIR/visual_smoke.sh"
bash -n "$PACKAGE_DIR/store_screenshot_check.sh"
bash -n "$PACKAGE_DIR/store_asset_audit.sh"
bash -n "$PACKAGE_DIR/content_rating_audit.sh"
bash -n "$PACKAGE_DIR/linux_package_audit.sh"
bash -n "$PACKAGE_DIR/linux_install_smoke.sh"
bash -n "$PACKAGE_DIR/linux_dependency_audit.sh"
bash -n "$PACKAGE_DIR/linux_portability_smoke.sh"
bash -n "$PACKAGE_DIR/linux_clean_distro_smoke.sh"
bash -n "$PACKAGE_DIR/linux_metadata_audit.sh"
bash -n "$PACKAGE_DIR/manual_qa_session.sh"
bash -n "$PACKAGE_DIR/manual_qa_observations.sh"
bash -n "$PACKAGE_DIR/platform_package_plan.sh"
bash -n "$PACKAGE_DIR/qa_evidence_summary.sh"
bash -n "$PACKAGE_DIR/qa_signoff_prepare.sh"
bash -n "$PACKAGE_DIR/final_signoff_check.sh"
bash -n "$PACKAGE_DIR/verify_release.sh"
bash -n "$PACKAGE_DIR/support_diagnostics.sh"
bash -n "$PACKAGE_DIR/signoff_bundle.sh"
bash -n "$PACKAGE_DIR/create_candidate_evidence.sh"
bash -n "$PACKAGE_DIR/create_store_submission_pack.sh"
bash -n "$PACKAGE_DIR/package_macos.sh"
grep -q "x86_64-pc-windows-msvc" "$PACKAGE_DIR/package_windows.ps1"
grep -q "Compress-Archive" "$PACKAGE_DIR/package_windows.ps1"
grep -q "Get-FileHash -Algorithm SHA256" "$PACKAGE_DIR/package_windows.ps1"
grep -q "aarch64-apple-darwin" "$PACKAGE_DIR/package_macos.sh"
grep -q "lipo -create" "$PACKAGE_DIR/package_macos.sh"

(
  cd "$PACKAGE_DIR"
  sha256sum -c SHA256SUMS >/dev/null
)

"$PACKAGE_DIR/bevy_open_siege" --validate-data >/dev/null
"$PACKAGE_DIR/bevy_open_siege" --audit-balance > "$TMPDIR/balance-audit.actual.txt"
diff -u "$PACKAGE_DIR/balance-audit.txt" "$TMPDIR/balance-audit.actual.txt"
grep -q "balance audit ok" "$TMPDIR/balance-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-assets > "$TMPDIR/asset-audit.actual.txt"
diff -u "$PACKAGE_DIR/asset-audit.txt" "$TMPDIR/asset-audit.actual.txt"
grep -q "asset audit ok" "$TMPDIR/asset-audit.actual.txt"
grep -q "png assets: 36" "$TMPDIR/asset-audit.actual.txt"
grep -q "wav assets: 7" "$TMPDIR/asset-audit.actual.txt"
grep -q "glb assets: 20" "$TMPDIR/asset-audit.actual.txt"
grep -q "production art assets: 56" "$TMPDIR/asset-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-audio > "$TMPDIR/audio-audit.actual.txt"
diff -u "$PACKAGE_DIR/audio-audit.txt" "$TMPDIR/audio-audit.actual.txt"
grep -q "audio audit ok" "$TMPDIR/audio-audit.actual.txt"
grep -q "checked music loops: 1" "$TMPDIR/audio-audit.actual.txt"
grep -q "checked sound effects: 6" "$TMPDIR/audio-audit.actual.txt"
grep -q "checked startup policy: audio remains opt-in by default" "$TMPDIR/audio-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-controls > "$TMPDIR/controls-audit.actual.txt"
diff -u "$PACKAGE_DIR/controls-audit.txt" "$TMPDIR/controls-audit.actual.txt"
grep -q "control audit ok: 24 bindings" "$TMPDIR/controls-audit.actual.txt"
grep -q "Mouse left | gameplay | move cursor and plant selected seed" "$TMPDIR/controls-audit.actual.txt"
grep -q "menu localization: en/zh menu_help covered" "$TMPDIR/controls-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-input-flow > "$TMPDIR/input-flow-audit.actual.txt"
diff -u "$PACKAGE_DIR/input-flow-audit.txt" "$TMPDIR/input-flow-audit.actual.txt"
grep -q "input flow audit ok" "$TMPDIR/input-flow-audit.actual.txt"
grep -q "gameplay seed selection: 10/10 plants covered" "$TMPDIR/input-flow-audit.actual.txt"
grep -q "pause gating: planting blocked while paused" "$TMPDIR/input-flow-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-localization > "$TMPDIR/localization-audit.actual.txt"
diff -u "$PACKAGE_DIR/localization-audit.txt" "$TMPDIR/localization-audit.actual.txt"
grep -q "localization audit ok" "$TMPDIR/localization-audit.actual.txt"
grep -q "checked languages: en, zh" "$TMPDIR/localization-audit.actual.txt"
grep -q "checked plants: 10" "$TMPDIR/localization-audit.actual.txt"
grep -q "checked zombies: 10" "$TMPDIR/localization-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-layout > "$TMPDIR/layout-audit.actual.txt"
diff -u "$PACKAGE_DIR/layout-audit.txt" "$TMPDIR/layout-audit.actual.txt"
grep -q "layout audit ok" "$TMPDIR/layout-audit.actual.txt"
grep -q "checked languages: en, zh" "$TMPDIR/layout-audit.actual.txt"
grep -q "checked plants in HUD: 10" "$TMPDIR/layout-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-visual > "$TMPDIR/visual-readability-audit.actual.txt"
diff -u "$PACKAGE_DIR/visual-readability-audit.txt" "$TMPDIR/visual-readability-audit.actual.txt"
grep -q "visual readability audit ok" "$TMPDIR/visual-readability-audit.actual.txt"
grep -q "checked viewports: 6" "$TMPDIR/visual-readability-audit.actual.txt"
grep -q "checked HUD wrapping: en/zh status and seed bank" "$TMPDIR/visual-readability-audit.actual.txt"
grep -q "checked visual assets: plant, monster, effect, environment, ui chrome" "$TMPDIR/visual-readability-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-accessibility > "$TMPDIR/accessibility-audit.actual.txt"
diff -u "$PACKAGE_DIR/accessibility-audit.txt" "$TMPDIR/accessibility-audit.actual.txt"
grep -q "accessibility audit ok" "$TMPDIR/accessibility-audit.actual.txt"
grep -q "keyboard-only flow: menu, gameplay, pause, settings, and end screens covered" "$TMPDIR/accessibility-audit.actual.txt"
grep -q "no-audio playability: audio remains opt-in and HUD/end text carry state" "$TMPDIR/accessibility-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-performance > "$TMPDIR/performance-audit.actual.txt"
diff -u "$PACKAGE_DIR/performance-audit.txt" "$TMPDIR/performance-audit.actual.txt"
grep -q "performance budget audit ok" "$TMPDIR/performance-audit.actual.txt"
grep -q "estimated dynamic entities:" "$TMPDIR/performance-audit.actual.txt"
grep -q "manual performance QA still required" "$TMPDIR/performance-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-privacy > "$TMPDIR/privacy-audit.actual.txt"
diff -u "$PACKAGE_DIR/privacy-audit.txt" "$TMPDIR/privacy-audit.actual.txt"
grep -q "privacy audit ok" "$TMPDIR/privacy-audit.actual.txt"
grep -q "checked network posture: no telemetry" "$TMPDIR/privacy-audit.actual.txt"
grep -q "manual privacy QA still required" "$TMPDIR/privacy-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-release-provenance > "$TMPDIR/release-provenance-audit.actual.txt"
diff -u "$PACKAGE_DIR/release-provenance-audit.txt" "$TMPDIR/release-provenance-audit.actual.txt"
grep -q "release provenance audit ok" "$TMPDIR/release-provenance-audit.actual.txt"
grep -q "checked integrity evidence: release-manifest.json, SHA256SUMS" "$TMPDIR/release-provenance-audit.actual.txt"
grep -q "manual provenance QA still required" "$TMPDIR/release-provenance-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-marketing > "$TMPDIR/marketing-audit.actual.txt"
diff -u "$PACKAGE_DIR/marketing-audit.txt" "$TMPDIR/marketing-audit.actual.txt"
grep -q "marketing audit ok" "$TMPDIR/marketing-audit.actual.txt"
grep -q "STORE_SCREENSHOTS.md token | screenshots/01-title-menu.png" "$TMPDIR/marketing-audit.actual.txt"
grep -q "checked documents: store, presskit, screenshots, content rating, release notes, art, notices, credits" "$TMPDIR/marketing-audit.actual.txt"
grep -q "checked media references: app icon, store capsule, plant sheet, monster sheet, screenshot plan" "$TMPDIR/marketing-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-ip > "$TMPDIR/ip-audit.actual.txt"
diff -u "$PACKAGE_DIR/ip-audit.txt" "$TMPDIR/ip-audit.actual.txt"
grep -q "ip audit ok" "$TMPDIR/ip-audit.actual.txt"
grep -q "checked release-facing blocks:" "$TMPDIR/ip-audit.actual.txt"
grep -q "checked asset manifest paths for renamed plant and frost assets" "$TMPDIR/ip-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-save > "$TMPDIR/save-audit.actual.txt"
diff -u "$PACKAGE_DIR/save-audit.txt" "$TMPDIR/save-audit.actual.txt"
grep -q "save audit ok" "$TMPDIR/save-audit.actual.txt"
grep -q "checked paths: explicit, xdg, home, portable" "$TMPDIR/save-audit.actual.txt"
grep -q "checked compatibility: legacy save, normalization, settings clamp" "$TMPDIR/save-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --audit-playthrough > "$TMPDIR/playthrough-audit.actual.txt"
diff -u "$PACKAGE_DIR/playthrough-audit.txt" "$TMPDIR/playthrough-audit.actual.txt"
grep -q "playthrough audit ok: 10 levels" "$TMPDIR/playthrough-audit.actual.txt"
grep -q "checked lifecycle: victories 10, defeats 10, restarts 10, score saves 10" "$TMPDIR/playthrough-audit.actual.txt"
grep -q "checked failure handling: defeat does not advance unlocks" "$TMPDIR/playthrough-audit.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --simulate-campaign > "$TMPDIR/campaign-simulation.actual.txt"
diff -u "$PACKAGE_DIR/campaign-simulation.txt" "$TMPDIR/campaign-simulation.actual.txt"
grep -q "campaign simulation ok" "$TMPDIR/campaign-simulation.actual.txt"
grep -q "covered plants: 10/10" "$TMPDIR/campaign-simulation.actual.txt"
grep -q "covered zombies: 10/10" "$TMPDIR/campaign-simulation.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --release-readiness > "$TMPDIR/release-readiness.actual.txt"
diff -u "$PACKAGE_DIR/release-readiness.txt" "$TMPDIR/release-readiness.actual.txt"
grep -q "release readiness: manual approval required" "$TMPDIR/release-readiness.actual.txt"
grep -q "automated evidence: pass" "$TMPDIR/release-readiness.actual.txt"
grep -q "ship status: release candidate, not final approval" "$TMPDIR/release-readiness.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --print-release-info > "$TMPDIR/release-info.actual.txt"
diff -u "$PACKAGE_DIR/release-info.txt" "$TMPDIR/release-info.actual.txt"
grep -q "Bevy Open Siege" "$TMPDIR/release-info.actual.txt"
"$PACKAGE_DIR/runtime_smoke.sh" "$PACKAGE_DIR/bevy_open_siege" 12 > "$TMPDIR/runtime-smoke.actual.txt"
diff -u "$PACKAGE_DIR/runtime-smoke.txt" "$TMPDIR/runtime-smoke.actual.txt"
grep -q "runtime startup smoke ok" "$TMPDIR/runtime-smoke.actual.txt"
grep -q "window: created" "$TMPDIR/runtime-smoke.actual.txt"
grep -q "panic_scan: clean" "$TMPDIR/runtime-smoke.actual.txt"
"$PACKAGE_DIR/visual_smoke.sh" "$PACKAGE_DIR/bevy_open_siege" 15 > "$TMPDIR/visual-smoke.actual.txt"
diff -u "$PACKAGE_DIR/visual-smoke.txt" "$TMPDIR/visual-smoke.actual.txt"
grep -q "visual startup smoke ok" "$TMPDIR/visual-smoke.actual.txt"
grep -q "screenshot: nonblank" "$TMPDIR/visual-smoke.actual.txt"
grep -q "panic_scan: clean" "$TMPDIR/visual-smoke.actual.txt"
"$PACKAGE_DIR/store_screenshot_check.sh" --plan "$PACKAGE_DIR" > "$TMPDIR/store-screenshot-plan.actual.txt"
grep -q "store screenshot workflow ok" "$TMPDIR/store-screenshot-plan.actual.txt"
grep -q "screenshots/01-title-menu.png" "$TMPDIR/store-screenshot-plan.actual.txt"
grep -q -- "--capture-startup . screenshots 01-title-menu.png 8" "$TMPDIR/store-screenshot-plan.actual.txt"
grep -q -- "--validate-dir screenshots" "$TMPDIR/store-screenshot-plan.actual.txt"
"$PACKAGE_DIR/store_asset_audit.sh" "$PACKAGE_DIR" > "$TMPDIR/store-asset-audit.actual.txt"
diff -u "$PACKAGE_DIR/store-asset-audit.txt" "$TMPDIR/store-asset-audit.actual.txt"
grep -q "store asset audit ok" "$TMPDIR/store-asset-audit.actual.txt"
grep -q "app icon: assets/branding/generated/app-icon.png" "$TMPDIR/store-asset-audit.actual.txt"
grep -q "store capsule: assets/branding/generated/store-capsule.png" "$TMPDIR/store-asset-audit.actual.txt"
grep -q "checked screenshot plan: five captures" "$TMPDIR/store-asset-audit.actual.txt"
"$PACKAGE_DIR/content_rating_audit.sh" "$PACKAGE_DIR" > "$TMPDIR/content-rating-audit.actual.txt"
diff -u "$PACKAGE_DIR/content-rating-audit.txt" "$TMPDIR/content-rating-audit.actual.txt"
grep -q "content rating audit ok" "$TMPDIR/content-rating-audit.actual.txt"
grep -q "checked gameplay content: fantasy non-realistic undead combat" "$TMPDIR/content-rating-audit.actual.txt"
grep -q "checked online/data: no accounts" "$TMPDIR/content-rating-audit.actual.txt"
"$PACKAGE_DIR/audio_smoke.sh" "$PACKAGE_DIR/bevy_open_siege" 12 > "$TMPDIR/audio-smoke.actual.txt"
diff -u "$PACKAGE_DIR/audio-smoke.txt" "$TMPDIR/audio-smoke.actual.txt"
grep -q "audio startup smoke ok" "$TMPDIR/audio-smoke.actual.txt"
grep -q "window: created" "$TMPDIR/audio-smoke.actual.txt"
grep -q "panic_scan: clean" "$TMPDIR/audio-smoke.actual.txt"
"$PACKAGE_DIR/linux_package_audit.sh" "$PACKAGE_DIR" > "$TMPDIR/linux-package-audit.actual.txt"
diff -u "$PACKAGE_DIR/linux-package-audit.txt" "$TMPDIR/linux-package-audit.actual.txt"
grep -q "linux package audit ok" "$TMPDIR/linux-package-audit.actual.txt"
grep -q "checked uninstall: app files removed and save preserved" "$TMPDIR/linux-package-audit.actual.txt"
grep -q "checked purge: save data removed only with --purge" "$TMPDIR/linux-package-audit.actual.txt"
"$PACKAGE_DIR/linux_dependency_audit.sh" "$PACKAGE_DIR" > "$TMPDIR/linux-dependency-audit.actual.txt"
diff -u "$PACKAGE_DIR/linux-dependency-audit.txt" "$TMPDIR/linux-dependency-audit.actual.txt"
grep -q "linux dependency audit ok" "$TMPDIR/linux-dependency-audit.actual.txt"
grep -q "missing dependencies: none" "$TMPDIR/linux-dependency-audit.actual.txt"
grep -q "portability review:" "$TMPDIR/linux-dependency-audit.actual.txt"
"$PACKAGE_DIR/linux_portability_smoke.sh" "$PACKAGE_DIR" 12 > "$TMPDIR/linux-portability-smoke.actual.txt"
diff -u "$PACKAGE_DIR/linux-portability-smoke.txt" "$TMPDIR/linux-portability-smoke.actual.txt"
grep -q "linux portability smoke ok" "$TMPDIR/linux-portability-smoke.actual.txt"
grep -q "sanitized_env: LD_LIBRARY_PATH unset, Nix variables omitted" "$TMPDIR/linux-portability-smoke.actual.txt"
grep -q "window: created" "$TMPDIR/linux-portability-smoke.actual.txt"
grep -q "panic_scan: clean" "$TMPDIR/linux-portability-smoke.actual.txt"
grep -q "clean_distro_qa: still required" "$TMPDIR/linux-portability-smoke.actual.txt"
"$PACKAGE_DIR/linux_clean_distro_smoke.sh" "$PACKAGE_DIR" > "$TMPDIR/linux-clean-distro-smoke.actual.txt"
diff -u "$PACKAGE_DIR/linux-clean-distro-smoke.txt" "$TMPDIR/linux-clean-distro-smoke.actual.txt"
grep -q "linux clean distro smoke ok" "$TMPDIR/linux-clean-distro-smoke.actual.txt"
grep -q "validate_data: pass" "$TMPDIR/linux-clean-distro-smoke.actual.txt"
grep -q "dependency_resolution: pass" "$TMPDIR/linux-clean-distro-smoke.actual.txt"
grep -q "missing_dependencies: none" "$TMPDIR/linux-clean-distro-smoke.actual.txt"
grep -q "manual_clean_machine_qa: still required" "$TMPDIR/linux-clean-distro-smoke.actual.txt"
"$PACKAGE_DIR/linux_metadata_audit.sh" "$PACKAGE_DIR" > "$TMPDIR/linux-metadata-audit.actual.txt"
diff -u "$PACKAGE_DIR/linux-metadata-audit.txt" "$TMPDIR/linux-metadata-audit.actual.txt"
grep -q "linux metadata audit ok" "$TMPDIR/linux-metadata-audit.actual.txt"
grep -q "appstream id: io.github.bevy_open_siege.BevyOpenSiege" "$TMPDIR/linux-metadata-audit.actual.txt"
grep -q "desktop entry: bevy-open-siege.desktop" "$TMPDIR/linux-metadata-audit.actual.txt"
"$PACKAGE_DIR/manual_qa_session.sh" --plan "$PACKAGE_DIR" > "$TMPDIR/manual-qa-plan.actual.txt"
diff -u "$PACKAGE_DIR/manual-qa-plan.txt" "$TMPDIR/manual-qa-plan.actual.txt"
grep -q "manual QA session plan ok" "$TMPDIR/manual-qa-plan.actual.txt"
grep -q "qa-session/full-campaign-playthrough.md" "$TMPDIR/manual-qa-plan.actual.txt"
grep -q "qa-session/privacy-support.md" "$TMPDIR/manual-qa-plan.actual.txt"
grep -q "qa-session/build-provenance.md" "$TMPDIR/manual-qa-plan.actual.txt"
grep -q "qa-session/store-screenshots.md" "$TMPDIR/manual-qa-plan.actual.txt"
grep -q "final decision rule:" "$TMPDIR/manual-qa-plan.actual.txt"
"$PACKAGE_DIR/manual_qa_session.sh" --init "$PACKAGE_DIR" "$TMPDIR/qa-session" > "$TMPDIR/manual-qa-init.txt"
grep -q "manual QA session initialized" "$TMPDIR/manual-qa-init.txt"
test -f "$TMPDIR/qa-session/README.md"
test -f "$TMPDIR/qa-session/manual-qa-plan.txt"
test -f "$TMPDIR/qa-session/full-campaign-playthrough.md"
test -f "$TMPDIR/qa-session/audio-device-qa.md"
test -f "$TMPDIR/qa-session/accessibility-qa.md"
test -f "$TMPDIR/qa-session/performance-qa.md"
test -f "$TMPDIR/qa-session/privacy-support.md"
test -f "$TMPDIR/qa-session/build-provenance.md"
test -f "$TMPDIR/qa-session/store-screenshots.md"
test -f "$TMPDIR/qa-session/final-art-direction.md"
grep -q "accessibility-audit.txt" "$TMPDIR/qa-session/accessibility-qa.md"
grep -q "performance-audit.txt" "$TMPDIR/qa-session/performance-qa.md"
grep -q "privacy-audit.txt" "$TMPDIR/qa-session/privacy-support.md"
grep -q "release-provenance-audit.txt" "$TMPDIR/qa-session/build-provenance.md"
grep -q "STORE_SCREENSHOTS.md" "$TMPDIR/qa-session/store-screenshots.md"
grep -q "store_screenshot_check.sh" "$TMPDIR/qa-session/store-screenshots.md"
grep -q "Status: Pending" "$TMPDIR/qa-session/full-campaign-playthrough.md"
grep -q "Approved: No" "$TMPDIR/qa-session/final-art-direction.md"
"$PACKAGE_DIR/manual_qa_observations.sh" --collect "$PACKAGE_DIR" "$TMPDIR/qa-observations" > "$TMPDIR/manual-qa-observations.txt"
grep -q "manual QA observations collected" "$TMPDIR/manual-qa-observations.txt"
test -f "$TMPDIR/qa-observations/automated-observations.md"
test -f "$TMPDIR/qa-observations/verify-release-quick.txt"
test -f "$TMPDIR/qa-observations/qa-evidence-summary.txt"
grep -q "Status remains Pending" "$TMPDIR/qa-observations/full-campaign-playthrough.md"
grep -q "Approved: No" "$TMPDIR/qa-observations/final-art-direction.md"
"$PACKAGE_DIR/platform_package_plan.sh" --plan "$PACKAGE_DIR" > "$TMPDIR/platform-package-plan.actual.txt"
diff -u "$PACKAGE_DIR/platform-package-plan.txt" "$TMPDIR/platform-package-plan.actual.txt"
grep -q "platform package plan ok" "$TMPDIR/platform-package-plan.actual.txt"
grep -q "windows-x86_64" "$TMPDIR/platform-package-plan.actual.txt"
grep -q "macos-universal" "$TMPDIR/platform-package-plan.actual.txt"
"$PACKAGE_DIR/platform_package_plan.sh" --init "$PACKAGE_DIR" "$TMPDIR/platform-session" > "$TMPDIR/platform-package-init.txt"
grep -q "platform package session initialized" "$TMPDIR/platform-package-init.txt"
test -f "$TMPDIR/platform-session/windows-package-qa.md"
test -f "$TMPDIR/platform-session/macos-package-qa.md"
grep -q "x86_64-pc-windows-msvc" "$TMPDIR/platform-session/windows-package-qa.md"
grep -q "aarch64-apple-darwin" "$TMPDIR/platform-session/macos-package-qa.md"
grep -q "package_windows.ps1" "$TMPDIR/platform-package-plan.actual.txt"
grep -q "package_macos.sh" "$TMPDIR/platform-package-plan.actual.txt"
"$PACKAGE_DIR/qa_evidence_summary.sh" --summary "$PACKAGE_DIR" "$TMPDIR/qa-session" "$TMPDIR/platform-session" > "$TMPDIR/qa-evidence-summary.actual.txt"
grep -q "qa evidence summary ok" "$TMPDIR/qa-evidence-summary.actual.txt"
grep -q "automated evidence: total=35 missing=0" "$TMPDIR/qa-evidence-summary.actual.txt"
grep -q "qa-signoff rows: total=37 pending=37" "$TMPDIR/qa-evidence-summary.actual.txt"
grep -q "manual evidence: total=14 missing=0 pending=14" "$TMPDIR/qa-evidence-summary.actual.txt"
grep -q "platform evidence: total=2 missing=0 pending=2" "$TMPDIR/qa-evidence-summary.actual.txt"
grep -q "qa-signoff: not approved" "$TMPDIR/qa-evidence-summary.actual.txt"
if "$PACKAGE_DIR/qa_signoff_prepare.sh" --check "$PACKAGE_DIR" "$TMPDIR/qa-session" "$TMPDIR/platform-session" > "$TMPDIR/qa-signoff-prepare.unexpected.txt" 2>&1; then
  echo "qa_signoff_prepare.sh should reject pending candidate evidence" >&2
  cat "$TMPDIR/qa-signoff-prepare.unexpected.txt" >&2
  exit 1
fi
grep -q "signoff evidence still pending" "$TMPDIR/qa-signoff-prepare.unexpected.txt"
FINAL_FIXTURE="$TMPDIR/final-signoff-fixture"
FINAL_PACKAGE="$FINAL_FIXTURE/$PACKAGE_NAME"
FINAL_QA_SESSION="$FINAL_FIXTURE/qa-session"
FINAL_PLATFORM_SESSION="$FINAL_FIXTURE/platform-session"
mkdir -p "$FINAL_FIXTURE"
cp -R "$PACKAGE_DIR" "$FINAL_PACKAGE"
cp -R "$TMPDIR/qa-session" "$FINAL_QA_SESSION"
cp -R "$TMPDIR/platform-session" "$FINAL_PLATFORM_SESSION"
for evidence in "$FINAL_QA_SESSION"/*.md; do
  complete_evidence_file "$evidence" "Pass" "Archive Smoke Fixture" "2026-06-20" "Temporary fixture approval for archive smoke test only."
done
complete_evidence_file "$FINAL_PLATFORM_SESSION/windows-package-qa.md" "Scoped Out" "Archive Smoke Fixture" "2026-06-20" "Scoped out for Linux v1 release scope; Windows package QA is not a Linux v1 blocker."
complete_evidence_file "$FINAL_PLATFORM_SESSION/macos-package-qa.md" "Scoped Out" "Archive Smoke Fixture" "2026-06-20" "Scoped out for Linux v1 release scope; macOS package QA is not a Linux v1 blocker."
"$FINAL_PACKAGE/qa_signoff_prepare.sh" --write "$FINAL_PACKAGE" "$FINAL_QA_SESSION" "$FINAL_PLATFORM_SESSION" "Archive Smoke Fixture" "2026-06-20" "$FINAL_PACKAGE/QA_SIGNOFF.md" > "$TMPDIR/qa-signoff-prepare.final.txt"
grep -q "qa signoff prepared" "$TMPDIR/qa-signoff-prepare.final.txt"
grep -q "Release approved: Yes" "$FINAL_PACKAGE/QA_SIGNOFF.md"
grep -q "SHA256: See packaged SHA256SUMS for per-file hashes." "$FINAL_PACKAGE/QA_SIGNOFF.md"
(
  cd "$FINAL_PACKAGE"
  find . -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum \
    | sed 's#  ./#  #' > SHA256SUMS
)
"$FINAL_PACKAGE/final_signoff_check.sh" --check "$FINAL_PACKAGE" "$FINAL_QA_SESSION" "$FINAL_PLATFORM_SESSION" > "$TMPDIR/final-signoff-check.fixture.txt"
grep -q "final signoff check passed" "$TMPDIR/final-signoff-check.fixture.txt"
"$PACKAGE_DIR/verify_release.sh" --quick "$PACKAGE_DIR" > "$TMPDIR/verify-release.actual.txt"
grep -q "release package verification ok" "$TMPDIR/verify-release.actual.txt"
grep -q "mode: quick" "$TMPDIR/verify-release.actual.txt"
grep -q "deterministic reports: matched" "$TMPDIR/verify-release.actual.txt"
"$PACKAGE_DIR/support_diagnostics.sh" "$PACKAGE_DIR" "$TMPDIR/support-diagnostics" > "$TMPDIR/support-diagnostics.out"
grep -q "support diagnostics collected" "$TMPDIR/support-diagnostics.out"
test -f "$TMPDIR/support-diagnostics/release-info.txt"
test -f "$TMPDIR/support-diagnostics/validate-data.txt"
test -f "$TMPDIR/support-diagnostics/save-path.txt"
test -f "$TMPDIR/support-diagnostics/privacy-audit.txt"
test -f "$TMPDIR/support-diagnostics/sha256sum.txt"
test -f "$TMPDIR/support-diagnostics/verify-release-quick.txt"
test -f "$TMPDIR/support-diagnostics/release-manifest.json"
grep -q "Bevy Open Siege support diagnostics" "$TMPDIR/support-diagnostics/environment.txt"
grep -q "does not include save files" "$TMPDIR/support-diagnostics/README.txt"
"$PACKAGE_DIR/signoff_bundle.sh" --plan "$PACKAGE_DIR" > "$TMPDIR/signoff-bundle-plan.actual.txt"
grep -q "signoff bundle plan ok" "$TMPDIR/signoff-bundle-plan.actual.txt"
"$PACKAGE_DIR/signoff_bundle.sh" --create --allow-candidate "$PACKAGE_DIR" "$TMPDIR/qa-session" "$TMPDIR/platform-session" "$TMPDIR/signoff-output" > "$TMPDIR/signoff-bundle.out"
grep -q "signoff bundle created" "$TMPDIR/signoff-bundle.out"
test -f "$TMPDIR/signoff-output/$(basename "$PACKAGE_DIR")-signoff-bundle.tar.gz"
test -f "$TMPDIR/signoff-output/$(basename "$PACKAGE_DIR")-signoff-bundle/bundle-manifest.txt"
test -f "$TMPDIR/signoff-output/$(basename "$PACKAGE_DIR")-signoff-bundle/qa-evidence-summary.txt"
test -f "$TMPDIR/signoff-output/$(basename "$PACKAGE_DIR")-signoff-bundle/final-signoff-check.txt"
test -f "$TMPDIR/signoff-output/$(basename "$PACKAGE_DIR")-signoff-bundle/SHA256SUMS"
grep -q "candidate_mode: 1" "$TMPDIR/signoff-output/$(basename "$PACKAGE_DIR")-signoff-bundle/bundle-manifest.txt"
"$PACKAGE_DIR/create_candidate_evidence.sh" "$PACKAGE_DIR" "$TMPDIR/candidate-output" > "$TMPDIR/candidate-evidence.out"
grep -q "candidate evidence created" "$TMPDIR/candidate-evidence.out"
test -f "$TMPDIR/candidate-output/$(basename "$PACKAGE_DIR")-candidate-evidence.tar.gz"
test -f "$TMPDIR/candidate-output/$(basename "$PACKAGE_DIR")-candidate-evidence/README.txt"
test -f "$TMPDIR/candidate-output/$(basename "$PACKAGE_DIR")-candidate-evidence/qa-session/full-campaign-playthrough.md"
test -f "$TMPDIR/candidate-output/$(basename "$PACKAGE_DIR")-candidate-evidence/platform-session/windows-package-qa.md"
test -f "$TMPDIR/candidate-output/$(basename "$PACKAGE_DIR")-candidate-evidence/support-diagnostics/release-info.txt"
test -f "$TMPDIR/candidate-output/$(basename "$PACKAGE_DIR")-candidate-evidence/signoff-output/$(basename "$PACKAGE_DIR")-signoff-bundle.tar.gz"
grep -q "release candidate, not final approved" "$TMPDIR/candidate-output/$(basename "$PACKAGE_DIR")-candidate-evidence/README.txt"
"$PACKAGE_DIR/create_store_submission_pack.sh" "$PACKAGE_DIR" "$TMPDIR/store-output" > "$TMPDIR/store-submission-pack.out"
grep -q "store submission pack created" "$TMPDIR/store-submission-pack.out"
grep -q "screenshot_status: pending" "$TMPDIR/store-submission-pack.out"
test -f "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack.tar.gz"
test -f "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack/README.txt"
test -f "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack/docs/STORE_PAGE.md"
test -f "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack/branding/app-icon.png"
test -f "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack/branding/store-capsule.png"
test -f "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack/reports/store-screenshot-validation.txt"
grep -q "store screenshot validation pending" "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack/reports/store-screenshot-validation.txt"
grep -q "store review not final approved" "$TMPDIR/store-output/$(basename "$PACKAGE_DIR")-store-submission-pack/README.txt"
"$PACKAGE_DIR/final_signoff_check.sh" --plan "$PACKAGE_DIR" > "$TMPDIR/final-signoff-plan.actual.txt"
diff -u "$PACKAGE_DIR/final-signoff-plan.txt" "$TMPDIR/final-signoff-plan.actual.txt"
grep -q "final signoff plan ok" "$TMPDIR/final-signoff-plan.actual.txt"
grep -q "final_signoff_check.sh --check . qa-session platform-session" "$TMPDIR/final-signoff-plan.actual.txt"
grep -q "Release approved: Yes" "$TMPDIR/final-signoff-plan.actual.txt"
"$PACKAGE_DIR/bevy_open_siege" --print-save-path | grep -q "bevy_open_siege_save.ron"
LEGACY_SAVE_HOME="$TMPDIR/legacy-home"
LEGACY_SAVE_DATA_HOME="$LEGACY_SAVE_HOME/share"
LEGACY_SAVE_CWD="$TMPDIR/legacy-cwd"
mkdir -p "$LEGACY_SAVE_HOME" "$LEGACY_SAVE_DATA_HOME" "$LEGACY_SAVE_CWD"
cat > "$LEGACY_SAVE_CWD/bevy_open_siege_save.ron" <<'EOF'
(
    version: 1,
    language: Chinese,
    unlocked_levels: 3,
    best_scores: [1250, 800],
)
EOF
(
  cd "$LEGACY_SAVE_CWD"
  HOME="$LEGACY_SAVE_HOME" \
  XDG_DATA_HOME="$LEGACY_SAVE_DATA_HOME" \
    "$PACKAGE_DIR/bevy_open_siege" --print-save-summary > "$TMPDIR/legacy-save-summary.txt"
)
grep -q "language=Chinese" "$TMPDIR/legacy-save-summary.txt"
grep -q "unlocked_levels=3" "$TMPDIR/legacy-save-summary.txt"
grep -q "best_scores=1250,800,0,0,0,0,0,0,0,0" "$TMPDIR/legacy-save-summary.txt"
grep -q "master_volume=0.80" "$TMPDIR/legacy-save-summary.txt"
grep -q "Exec=bevy_open_siege" "$PACKAGE_DIR/assets/linux/bevy-open-siege.desktop"
grep -q "Icon=bevy-open-siege" "$PACKAGE_DIR/assets/linux/bevy-open-siege.desktop"
grep -q "io.github.bevy_open_siege.BevyOpenSiege" "$PACKAGE_DIR/assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
grep -q "<release version=\"$PACKAGE_VERSION\"" "$PACKAGE_DIR/assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml"
grep -q "Generated from \`cargo metadata --locked --filter-platform x86_64-unknown-linux-gnu\`" "$PACKAGE_DIR/THIRD_PARTY_LICENSES.md"
grep -q "\`bevy\`" "$PACKAGE_DIR/THIRD_PARTY_LICENSES.md"
grep -q "\`serde\`" "$PACKAGE_DIR/THIRD_PARTY_LICENSES.md"
grep -q "\`ron\`" "$PACKAGE_DIR/THIRD_PARTY_LICENSES.md"
grep -q "Full campaign playthrough" "$PACKAGE_DIR/QA_SIGNOFF.md"
grep -q "Store screenshot review" "$PACKAGE_DIR/QA_SIGNOFF.md"
grep -q "Content rating review" "$PACKAGE_DIR/QA_SIGNOFF.md"
grep -q "Windows package QA" "$PACKAGE_DIR/QA_SIGNOFF.md"
grep -q "Release approved: No" "$PACKAGE_DIR/QA_SIGNOFF.md"
grep -q "Bevy Open Siege Troubleshooting" "$PACKAGE_DIR/TROUBLESHOOTING.md"
grep -q "verify_release.sh --quick" "$PACKAGE_DIR/TROUBLESHOOTING.md"
grep -q "runtime_smoke.sh" "$PACKAGE_DIR/TROUBLESHOOTING.md"
grep -q "audio_smoke.sh" "$PACKAGE_DIR/TROUBLESHOOTING.md"
grep -q '"schema": "bevy-open-siege-release-manifest-v1"' "$PACKAGE_DIR/release-manifest.json"
grep -q '"package": "'"$PACKAGE_NAME"'"' "$PACKAGE_DIR/release-manifest.json"
grep -q '"platform": "linux-x86_64"' "$PACKAGE_DIR/release-manifest.json"
grep -q '"final_approval_required": true' "$PACKAGE_DIR/release-manifest.json"
grep -q '"path": "privacy-audit.txt"' "$PACKAGE_DIR/release-manifest.json"
grep -q '"path": "TROUBLESHOOTING.md"' "$PACKAGE_DIR/release-manifest.json"
if grep -q "UNKNOWN_LICENSE" "$PACKAGE_DIR/THIRD_PARTY_LICENSES.md"; then
  echo "third-party license report contains an unknown license entry" >&2
  exit 1
fi

plant_sprite_count="$(find "$PACKAGE_DIR/assets/art/sprites/plants" -maxdepth 1 -type f -name '*.png' | wc -l)"
monster_sprite_count="$(find "$PACKAGE_DIR/assets/art/sprites/monsters" -maxdepth 1 -type f -name '*.png' | wc -l)"
plant_model_count="$(find "$PACKAGE_DIR/assets/models/plants" -maxdepth 1 -type f -name '*.glb' | wc -l)"
monster_model_count="$(find "$PACKAGE_DIR/assets/models/monsters" -maxdepth 1 -type f -name '*.glb' | wc -l)"
effect_sprite_count="$(find "$PACKAGE_DIR/assets/art/effects" -maxdepth 1 -type f -name '*.png' | wc -l)"
environment_texture_count="$(find "$PACKAGE_DIR/assets/art/environment" -maxdepth 1 -type f -name '*.png' | wc -l)"
ui_chrome_count="$(find "$PACKAGE_DIR/assets/art/ui" -maxdepth 1 -type f -name '*.png' | wc -l)"
audio_count="$(find "$PACKAGE_DIR/assets/audio" -maxdepth 1 -type f -name '*.wav' | wc -l)"
if [[ "$plant_sprite_count" -ne 10 ]]; then
  echo "release archive must include 10 plant sprites, found $plant_sprite_count" >&2
  exit 1
fi
if [[ "$monster_sprite_count" -ne 10 ]]; then
  echo "release archive must include 10 monster sprites, found $monster_sprite_count" >&2
  exit 1
fi
if [[ "$plant_model_count" -ne 10 ]]; then
  echo "release archive must include 10 generated plant models, found $plant_model_count" >&2
  exit 1
fi
if [[ "$monster_model_count" -ne 10 ]]; then
  echo "release archive must include 10 generated monster models, found $monster_model_count" >&2
  exit 1
fi
if [[ "$effect_sprite_count" -ne 6 ]]; then
  echo "release archive must include 6 effect sprites, found $effect_sprite_count" >&2
  exit 1
fi
if [[ "$environment_texture_count" -ne 3 ]]; then
  echo "release archive must include 3 environment textures, found $environment_texture_count" >&2
  exit 1
fi
if [[ "$ui_chrome_count" -ne 3 ]]; then
  echo "release archive must include 3 UI chrome textures, found $ui_chrome_count" >&2
  exit 1
fi
if [[ "$audio_count" -ne 7 ]]; then
  echo "release archive must include 7 audio files, found $audio_count" >&2
  exit 1
fi

echo "release archive smoke test passed: $ARCHIVE"
