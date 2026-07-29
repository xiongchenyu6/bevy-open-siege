#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

if [[ -n "${BEVY_OPEN_SIEGE_USE_NIX:-}" ]]; then
  USE_NIX="$BEVY_OPEN_SIEGE_USE_NIX"
elif command -v cargo >/dev/null 2>&1; then
  USE_NIX="0"
else
  USE_NIX="1"
fi
ISOLATED_TARGET="${BEVY_OPEN_SIEGE_ISOLATED_TARGET:-$USE_NIX}"

cmd=(
  bash -lc '
  set -euo pipefail
  cargo fmt --check
  cargo check
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo run -- --validate-data
  cargo run -- --audit-balance
  cargo run -- --audit-assets
  cargo run -- --audit-audio
  cargo run -- --audit-controls
  cargo run -- --audit-input-flow
  cargo run -- --audit-localization
  cargo run -- --audit-layout
  cargo run -- --audit-visual
  cargo run -- --audit-accessibility
  cargo run -- --audit-performance
  cargo run -- --audit-privacy
  cargo run -- --audit-release-provenance
  cargo run -- --audit-marketing
  cargo run -- --audit-ip
  cargo run -- --audit-save
  cargo run -- --audit-playthrough
  cargo run -- --simulate-campaign
  cargo run -- --release-readiness
'
)

if [[ "$ISOLATED_TARGET" == "1" ]]; then
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/bevy_open_siege_release_check_target}"
fi

if [[ "$USE_NIX" == "1" ]]; then
  rm -rf "$ROOT/target"
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    nix develop "path:$ROOT" --command env CARGO_TARGET_DIR="$CARGO_TARGET_DIR" "${cmd[@]}"
  else
    nix develop "path:$ROOT" --command "${cmd[@]}"
  fi
else
  "${cmd[@]}"
fi
bash -n "$ROOT/scripts/package_release.sh"
bash -n "$ROOT/scripts/smoke_release_archive.sh"
bash -n "$ROOT/scripts/audio_smoke.sh"
bash -n "$ROOT/scripts/runtime_smoke.sh"
bash -n "$ROOT/scripts/visual_smoke.sh"
bash -n "$ROOT/scripts/store_screenshot_check.sh"
bash -n "$ROOT/scripts/store_asset_audit.sh"
bash -n "$ROOT/scripts/content_rating_audit.sh"
bash -n "$ROOT/scripts/linux_package_audit.sh"
bash -n "$ROOT/scripts/linux_install_smoke.sh"
bash -n "$ROOT/scripts/linux_dependency_audit.sh"
bash -n "$ROOT/scripts/linux_portability_smoke.sh"
bash -n "$ROOT/scripts/linux_clean_distro_smoke.sh"
bash -n "$ROOT/scripts/linux_metadata_audit.sh"
bash -n "$ROOT/scripts/manual_qa_session.sh"
bash -n "$ROOT/scripts/manual_qa_observations.sh"
bash -n "$ROOT/scripts/platform_package_plan.sh"
bash -n "$ROOT/scripts/qa_evidence_summary.sh"
bash -n "$ROOT/scripts/qa_signoff_prepare.sh"
bash -n "$ROOT/scripts/final_signoff_check.sh"
bash -n "$ROOT/scripts/verify_release.sh"
bash -n "$ROOT/scripts/support_diagnostics.sh"
bash -n "$ROOT/scripts/signoff_bundle.sh"
bash -n "$ROOT/scripts/create_candidate_evidence.sh"
bash -n "$ROOT/scripts/create_store_submission_pack.sh"
bash -n "$ROOT/scripts/package_macos.sh"
bash -n "$ROOT/scripts/verify_github_release.sh"
bash -n "$ROOT/scripts/verify_platform_archive.sh"
bash -n "$ROOT/scripts/build_web_bundle.sh"
bash -n "$ROOT/scripts/verify_web_bundle.sh"
bash -n "$ROOT/scripts/verify_pages_deployment.sh"
node --check "$ROOT/scripts/pages_smoke.spec.cjs"
python3 -m py_compile "$ROOT/scripts/generate_3d_models.py"
grep -q 'cp -R "$ROOT/assets/models"' "$ROOT/scripts/package_release.sh"
grep -q 'assets/models/plants/sprout-slinger.glb' "$ROOT/scripts/smoke_release_archive.sh"
grep -q 'glb assets: 20' "$ROOT/scripts/smoke_release_archive.sh"
grep -q 'qa_signoff_prepare.sh' "$ROOT/scripts/package_release.sh"
grep -q 'cp "$ROOT/scripts/qa_signoff_prepare.sh"' "$ROOT/scripts/package_release.sh"
grep -q 'chmod +x "$DIST/qa_signoff_prepare.sh"' "$ROOT/scripts/package_release.sh"
grep -q 'qa_signoff_prepare.sh should reject pending candidate evidence' "$ROOT/scripts/smoke_release_archive.sh"
grep -q 'final signoff check passed' "$ROOT/scripts/smoke_release_archive.sh"
grep -Fq "NIX_STORE_PATH_PATTERN='/nix/store/[0-9a-z]{32}-'" "$ROOT/scripts/linux_dependency_audit.sh"
grep -Fq "NIX_STORE_PATH_PATTERN='/nix/store/[0-9a-z]{32}-'" "$ROOT/scripts/linux_portability_smoke.sh"
grep -Fq 'grep -aEq "$NIX_STORE_PATH_PATTERN" "$PACKAGE_DIR/bevy_open_siege.bin"' "$ROOT/scripts/linux_portability_smoke.sh"
grep -Fq 'docker info >/dev/null 2>&1' "$ROOT/scripts/linux_clean_distro_smoke.sh"
grep -Fq 'container execution failed with status' "$ROOT/scripts/linux_clean_distro_smoke.sh"
grep -q 'See packaged SHA256SUMS for per-file hashes' "$ROOT/scripts/qa_signoff_prepare.sh"
grep -q 'archive-sha256 must be a 64-character SHA256 hex digest' "$ROOT/scripts/qa_signoff_prepare.sh"
grep -q "x86_64-pc-windows-msvc" "$ROOT/scripts/package_windows.ps1"
grep -q "cargo build --locked --release --target" "$ROOT/scripts/package_windows.ps1"
grep -q "Compress-Archive" "$ROOT/scripts/package_windows.ps1"
grep -q 'qa_signoff_prepare.sh' "$ROOT/scripts/package_windows.ps1"
grep -q 'cargo build --locked --release --target "$X86_TARGET"' "$ROOT/scripts/package_macos.sh"
grep -q 'lipo "$RELEASE_BINARY" -verify_arch x86_64 arm64' "$ROOT/scripts/package_macos.sh"
grep -q 'qa_signoff_prepare.sh' "$ROOT/scripts/package_macos.sh"
grep -q 'verify_platform_archive.sh' "$ROOT/.github/workflows/release.yml"
grep -q 'shasum -a 256 "${PACKAGE}.tar.gz"' "$ROOT/.github/workflows/release.yml"
grep -q 'cargo metadata --format-version 1' "$ROOT/.github/workflows/post-release-bump.yml"
grep -q 'default: false' "$ROOT/.github/workflows/post-release-bump.yml"
grep -q 'build_web_bundle.sh web-dist' "$ROOT/.github/workflows/release.yml"
grep -q 'verify_web_bundle.sh' "$ROOT/.github/workflows/release.yml"
grep -q 'release_tag=$RELEASE_TAG' "$ROOT/.github/workflows/release.yml"
grep -q 'verify_pages_deployment.sh' "$ROOT/.github/workflows/deploy-pages.yml"
grep -q 'pages_smoke.spec.cjs' "$ROOT/.github/workflows/deploy-pages.yml"
grep -q 'actions/upload-pages-artifact@v5' "$ROOT/.github/workflows/deploy-pages.yml"
grep -q 'return "webgl2";' "$ROOT/web/index.html"
python3 -m py_compile "$ROOT/scripts/generate_third_party_licenses.py"
python3 -m py_compile "$ROOT/scripts/generate_release_manifest.py"
