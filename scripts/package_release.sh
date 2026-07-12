#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)"
PACKAGE="bevy_open_siege-${VERSION}-linux-x86_64"
DIST="$ROOT/dist/$PACKAGE"
if [[ -n "${BEVY_OPEN_SIEGE_USE_NIX:-}" ]]; then
  USE_NIX="$BEVY_OPEN_SIEGE_USE_NIX"
elif command -v cargo >/dev/null 2>&1; then
  USE_NIX="0"
else
  USE_NIX="1"
fi
BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

if [[ "$USE_NIX" == "1" ]]; then
  BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/bevy_open_siege_package_target}"
  rm -rf "$ROOT/target"
  nix develop "path:$ROOT" --command env CARGO_TARGET_DIR="$BUILD_TARGET_DIR" cargo build --release
  NIX_ENV_FILE="$(mktemp)"
  nix develop "path:$ROOT" --command bash -lc \
    'printf "export PATH=%q\nexport LD_LIBRARY_PATH=%q\n" "$PATH" "${LD_LIBRARY_PATH:-}"' \
    > "$NIX_ENV_FILE"
  # shellcheck disable=SC1090
  source "$NIX_ENV_FILE"
  rm -f "$NIX_ENV_FILE"
else
  cargo build --release
fi
RELEASE_BINARY="$BUILD_TARGET_DIR/release/bevy_open_siege"
mkdir -p "$ROOT/dist"
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
"$ROOT/scripts/runtime_smoke.sh" "$RELEASE_BINARY" 12 > "$ROOT/dist/runtime-smoke.txt"
"$ROOT/scripts/visual_smoke.sh" "$RELEASE_BINARY" 15 > "$ROOT/dist/visual-smoke.txt"
"$ROOT/scripts/audio_smoke.sh" "$RELEASE_BINARY" 12 > "$ROOT/dist/audio-smoke.txt"
python3 "$ROOT/scripts/generate_third_party_licenses.py" > "$ROOT/dist/THIRD_PARTY_LICENSES.md"

rm -rf "$DIST"
mkdir -p "$DIST/assets" "$DIST/lib"
cp "$RELEASE_BINARY" "$DIST/bevy_open_siege.bin"
if ! command -v patchelf >/dev/null 2>&1; then
  echo "package_release.sh requires patchelf to build the Linux portable runtime wrapper" >&2
  exit 1
fi
INTERPRETER="$(patchelf --print-interpreter "$RELEASE_BINARY")"
if [[ ! -f "$INTERPRETER" ]]; then
  echo "release binary interpreter not found: $INTERPRETER" >&2
  exit 1
fi
declare -A COPIED_RUNTIME_LIBS=()
copy_runtime_lib() {
  local source="$1"
  if [[ -f "$source" ]]; then
    rm -f "$DIST/lib/$(basename "$source")"
    cp -aL "$source" "$DIST/lib/"
  fi
}
copy_runtime_lib_recursive() {
  local requested_source="$1"
  local source
  source="$(readlink -f "$requested_source" 2>/dev/null || printf '%s' "$requested_source")"
  if [[ ! -f "$requested_source" || ! -f "$source" ]]; then
    return
  fi
  if [[ -n "${COPIED_RUNTIME_LIBS[$source]:-}" ]]; then
    copy_runtime_lib "$requested_source"
    return
  fi
  COPIED_RUNTIME_LIBS[$source]=1
  copy_runtime_lib "$requested_source"
  copy_runtime_lib "$source"
  while IFS= read -r nested_dependency; do
    copy_runtime_lib_recursive "$nested_dependency"
  done < <(ldd "$source" 2>/dev/null | awk '/=>[[:space:]]*\// { print $3 } /^[[:space:]]*\// { print $1 }')
}
copy_runtime_lib_recursive "$INTERPRETER"
while IFS= read -r dependency; do
  copy_runtime_lib_recursive "$dependency"
done < <(ldd "$RELEASE_BINARY" | awk '/=>[[:space:]]*\// { print $3 } /^[[:space:]]*\// { print $1 }')
while IFS= read -r lib_dir; do
  if [[ -d "$lib_dir" ]]; then
    find "$lib_dir" -maxdepth 1 -type f -name '*.so*' -print0 \
      | while IFS= read -r -d '' lib_file; do
          copy_runtime_lib_recursive "$lib_file"
        done
  fi
done < <(
  {
    patchelf --print-rpath "$RELEASE_BINARY" | tr ':' '\n'
    strings "$RELEASE_BINARY" | grep -oE '/nix/store/[^[:space:]:]+/lib' || true
  } | sort -u
)
for soname in \
  libxkbcommon-x11.so.0 \
  libxkbcommon.so.0 \
  libX11.so.6 \
  libXi.so.6 \
  libXcursor.so.1 \
  libX11-xcb.so.1 \
  libXinerama.so.1 \
  libxcb.so.1 \
  libvulkan.so.1 \
  libGL.so.1; do
  found_lib="$(find /nix/store -maxdepth 5 \( -type f -o -type l \) -path "*/lib*/$soname" 2>/dev/null | grep -v fhsenv-rootfs | head -n 1 || true)"
  if [[ -z "$found_lib" ]]; then
    found_lib="$(find /nix/store -maxdepth 5 \( -type f -o -type l \) -path "*/lib*/$soname" 2>/dev/null | head -n 1 || true)"
  fi
  if [[ -n "$found_lib" ]]; then
    copy_runtime_lib_recursive "$found_lib"
  fi
done
patchelf --set-rpath '$ORIGIN/lib' "$DIST/bevy_open_siege.bin"
patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 "$DIST/bevy_open_siege.bin"
cat > "$DIST/bevy_open_siege" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$APP_DIR/lib"
LOADER="$LIB_DIR/ld-linux-x86-64.so.2"
if [[ ! -x "$LOADER" ]]; then
  echo "Bundled Linux loader not found: $LOADER" >&2
  exit 1
fi
export LD_LIBRARY_PATH="$LIB_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$LOADER" --library-path "$LIB_DIR" "$APP_DIR/bevy_open_siege.bin" "$@"
EOF
chmod +x "$DIST/bevy_open_siege" "$DIST/bevy_open_siege.bin"
cp "$ROOT/README.md" "$DIST/"
cp "$ROOT/LICENSE" "$DIST/"
cp "$ROOT/CREDITS.md" "$DIST/"
cp "$ROOT/ART_ASSETS.md" "$DIST/"
cp "$ROOT/THIRD_PARTY_NOTICES.md" "$DIST/"
cp "$ROOT/dist/THIRD_PARTY_LICENSES.md" "$DIST/"
cp "$ROOT/STORE_PAGE.md" "$DIST/"
cp "$ROOT/STORE_SCREENSHOTS.md" "$DIST/"
cp "$ROOT/CONTENT_RATING.md" "$DIST/"
cp "$ROOT/RELEASE_CHECKLIST.md" "$DIST/"
cp "$ROOT/RELEASE_NOTES.md" "$DIST/"
cp "$ROOT/QA_SIGNOFF.md" "$DIST/"
cp "$ROOT/PRIVACY.md" "$DIST/"
cp "$ROOT/SUPPORT.md" "$DIST/"
cp "$ROOT/TROUBLESHOOTING.md" "$DIST/"
cp "$ROOT/BUILD_PROVENANCE.md" "$DIST/"
cp "$ROOT/VERSION.ron" "$DIST/"
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
cp "$ROOT/dist/runtime-smoke.txt" "$DIST/"
cp "$ROOT/dist/visual-smoke.txt" "$DIST/"
cp "$ROOT/dist/audio-smoke.txt" "$DIST/"
cp "$ROOT/PRESSKIT.md" "$DIST/"
cp "$ROOT/scripts/install_linux_user.sh" "$DIST/"
cp "$ROOT/scripts/uninstall_linux_user.sh" "$DIST/"
cp "$ROOT/scripts/audio_smoke.sh" "$DIST/"
cp "$ROOT/scripts/runtime_smoke.sh" "$DIST/"
cp "$ROOT/scripts/visual_smoke.sh" "$DIST/"
cp "$ROOT/scripts/store_screenshot_check.sh" "$DIST/"
cp "$ROOT/scripts/store_asset_audit.sh" "$DIST/"
cp "$ROOT/scripts/content_rating_audit.sh" "$DIST/"
cp "$ROOT/scripts/linux_package_audit.sh" "$DIST/"
cp "$ROOT/scripts/linux_install_smoke.sh" "$DIST/"
cp "$ROOT/scripts/linux_dependency_audit.sh" "$DIST/"
cp "$ROOT/scripts/linux_portability_smoke.sh" "$DIST/"
cp "$ROOT/scripts/linux_clean_distro_smoke.sh" "$DIST/"
cp "$ROOT/scripts/linux_metadata_audit.sh" "$DIST/"
cp "$ROOT/scripts/manual_qa_session.sh" "$DIST/"
cp "$ROOT/scripts/manual_qa_observations.sh" "$DIST/"
cp "$ROOT/scripts/platform_package_plan.sh" "$DIST/"
cp "$ROOT/scripts/qa_evidence_summary.sh" "$DIST/"
cp "$ROOT/scripts/qa_signoff_prepare.sh" "$DIST/"
cp "$ROOT/scripts/final_signoff_check.sh" "$DIST/"
cp "$ROOT/scripts/verify_release.sh" "$DIST/"
cp "$ROOT/scripts/support_diagnostics.sh" "$DIST/"
cp "$ROOT/scripts/signoff_bundle.sh" "$DIST/"
cp "$ROOT/scripts/create_candidate_evidence.sh" "$DIST/"
cp "$ROOT/scripts/create_store_submission_pack.sh" "$DIST/"
cp "$ROOT/scripts/package_windows.ps1" "$DIST/"
cp "$ROOT/scripts/package_macos.sh" "$DIST/"
chmod +x "$DIST/audio_smoke.sh"
chmod +x "$DIST/runtime_smoke.sh"
chmod +x "$DIST/visual_smoke.sh"
chmod +x "$DIST/store_screenshot_check.sh"
chmod +x "$DIST/store_asset_audit.sh"
chmod +x "$DIST/content_rating_audit.sh"
chmod +x "$DIST/linux_package_audit.sh"
chmod +x "$DIST/linux_install_smoke.sh"
chmod +x "$DIST/linux_dependency_audit.sh"
chmod +x "$DIST/linux_portability_smoke.sh"
chmod +x "$DIST/linux_clean_distro_smoke.sh"
chmod +x "$DIST/linux_metadata_audit.sh"
chmod +x "$DIST/manual_qa_session.sh"
chmod +x "$DIST/manual_qa_observations.sh"
chmod +x "$DIST/platform_package_plan.sh"
chmod +x "$DIST/qa_evidence_summary.sh"
chmod +x "$DIST/qa_signoff_prepare.sh"
chmod +x "$DIST/final_signoff_check.sh"
chmod +x "$DIST/verify_release.sh"
chmod +x "$DIST/support_diagnostics.sh"
chmod +x "$DIST/signoff_bundle.sh"
chmod +x "$DIST/create_candidate_evidence.sh"
chmod +x "$DIST/create_store_submission_pack.sh"
chmod +x "$DIST/package_macos.sh"
cp "$ROOT/assets/manifest.ron" "$DIST/assets/"
cp -R "$ROOT/assets/art" "$DIST/assets/"
cp -R "$ROOT/assets/audio" "$DIST/assets/"
cp -R "$ROOT/assets/branding" "$DIST/assets/"
cp -R "$ROOT/assets/data" "$DIST/assets/"
cp -R "$ROOT/assets/i18n" "$DIST/assets/"
cp -R "$ROOT/assets/linux" "$DIST/assets/"
cp -R "$ROOT/assets/models" "$DIST/assets/"

"$ROOT/scripts/linux_package_audit.sh" "$DIST" > "$DIST/linux-package-audit.txt"
"$ROOT/scripts/linux_install_smoke.sh" "$DIST" 12 15 > "$DIST/linux-install-smoke.txt"
"$ROOT/scripts/linux_dependency_audit.sh" "$DIST" > "$DIST/linux-dependency-audit.txt"
"$ROOT/scripts/linux_portability_smoke.sh" "$DIST" 12 > "$DIST/linux-portability-smoke.txt"
"$ROOT/scripts/linux_clean_distro_smoke.sh" "$DIST" > "$DIST/linux-clean-distro-smoke.txt"
"$ROOT/scripts/linux_metadata_audit.sh" "$DIST" > "$DIST/linux-metadata-audit.txt"
"$ROOT/scripts/store_asset_audit.sh" "$DIST" > "$DIST/store-asset-audit.txt"
"$ROOT/scripts/content_rating_audit.sh" "$DIST" > "$DIST/content-rating-audit.txt"
"$ROOT/scripts/manual_qa_session.sh" --plan "$DIST" > "$DIST/manual-qa-plan.txt"
"$ROOT/scripts/platform_package_plan.sh" --plan "$DIST" > "$DIST/platform-package-plan.txt"
"$ROOT/scripts/final_signoff_check.sh" --plan "$DIST" > "$DIST/final-signoff-plan.txt"
python3 "$ROOT/scripts/generate_release_manifest.py" "$DIST" "linux-x86_64" > "$DIST/release-manifest.json"

(
  cd "$DIST"
  find . -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum \
    | sed 's#  ./#  #' > SHA256SUMS
)

tar -C "$ROOT/dist" -czf "$ROOT/dist/${PACKAGE}.tar.gz" "$PACKAGE"
"$ROOT/scripts/smoke_release_archive.sh" "$ROOT/dist/${PACKAGE}.tar.gz"
echo "Created dist/${PACKAGE}.tar.gz"
