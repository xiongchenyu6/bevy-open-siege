#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  create_store_submission_pack.sh <package-dir> <output-dir> [screenshot-dir]

Creates a store-submission handoff folder with store copy, press materials,
branding assets, content-rating notes, and optional validated screenshots.
Without screenshot-dir, the pack is marked pending for final screenshot review.
EOF
}

if [[ $# -lt 2 || $# -gt 3 ]]; then
  usage
  exit 2
fi

package_dir="$(cd "$1" && pwd)"
output_dir="$2"
screenshot_dir="${3:-}"
pack_root="$output_dir/$(basename "$package_dir")-store-submission-pack"

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "store submission pack requires packaged file: $path" >&2
    exit 1
  fi
}

if [[ ! -d "$package_dir" ]]; then
  echo "package directory not found: $package_dir" >&2
  exit 1
fi

for file in \
  STORE_PAGE.md \
  PRESSKIT.md \
  STORE_SCREENSHOTS.md \
  CONTENT_RATING.md \
  ART_ASSETS.md \
  CREDITS.md \
  THIRD_PARTY_NOTICES.md \
  RELEASE_NOTES.md \
  store_asset_audit.sh \
  content_rating_audit.sh \
  store_screenshot_check.sh \
  marketing-audit.txt \
  store-asset-audit.txt \
  content-rating-audit.txt \
  assets/branding/generated/app-icon.png \
  assets/branding/generated/store-capsule.png \
  assets/art/plants-sheet.png \
  assets/art/monsters-sheet.png; do
  require_file "$package_dir/$file"
done

rm -rf "$pack_root"
mkdir -p "$pack_root/docs" "$pack_root/branding" "$pack_root/art" "$pack_root/reports"

cp "$package_dir/STORE_PAGE.md" "$pack_root/docs/"
cp "$package_dir/PRESSKIT.md" "$pack_root/docs/"
cp "$package_dir/STORE_SCREENSHOTS.md" "$pack_root/docs/"
cp "$package_dir/CONTENT_RATING.md" "$pack_root/docs/"
cp "$package_dir/ART_ASSETS.md" "$pack_root/docs/"
cp "$package_dir/CREDITS.md" "$pack_root/docs/"
cp "$package_dir/THIRD_PARTY_NOTICES.md" "$pack_root/docs/"
cp "$package_dir/RELEASE_NOTES.md" "$pack_root/docs/"
cp "$package_dir/assets/branding/generated/app-icon.png" "$pack_root/branding/"
cp "$package_dir/assets/branding/generated/store-capsule.png" "$pack_root/branding/"
cp "$package_dir/assets/art/plants-sheet.png" "$pack_root/art/"
cp "$package_dir/assets/art/monsters-sheet.png" "$pack_root/art/"

"$package_dir/store_asset_audit.sh" "$package_dir" > "$pack_root/reports/store-asset-audit.txt"
"$package_dir/content_rating_audit.sh" "$package_dir" > "$pack_root/reports/content-rating-audit.txt"
cp "$package_dir/marketing-audit.txt" "$pack_root/reports/"

if [[ -n "$screenshot_dir" ]]; then
  if [[ ! -d "$screenshot_dir" ]]; then
    echo "screenshot directory not found: $screenshot_dir" >&2
    exit 1
  fi
  "$package_dir/store_screenshot_check.sh" --validate-dir "$screenshot_dir" > "$pack_root/reports/store-screenshot-validation.txt"
  mkdir -p "$pack_root/screenshots"
  cp "$screenshot_dir"/*.png "$pack_root/screenshots/"
  screenshot_status="validated"
else
  cat > "$pack_root/reports/store-screenshot-validation.txt" <<'EOF'
store screenshot validation pending
reason: no screenshot directory was provided
required: run store_screenshot_check.sh --validate-dir screenshots after capturing the final 1920x1080 screenshot set
manual review still required: language coverage, composition, storefront rules, and qa-session/store-screenshots.md approval
EOF
  screenshot_status="pending"
fi

cat > "$pack_root/README.txt" <<EOF
Bevy Open Siege store submission pack
package: $(basename "$package_dir")
status: release candidate, store review not final approved
screenshot_status: $screenshot_status

Contents:
- docs/: store copy, press kit, content rating notes, screenshot checklist, credits, notices, and release notes.
- branding/: app icon and store capsule PNG exports.
- art/: plant and monster production character sheets.
- reports/: marketing, store-asset, content-rating, and screenshot validation evidence.
- screenshots/: included only when a screenshot directory was provided and validated.

Final store submission still requires:
- Final screenshot set validation and manual composition approval.
- Current storefront metadata/rating questionnaire review.
- Final QA_SIGNOFF.md approval and final_signoff_check.sh --check.
EOF

(
  cd "$pack_root"
  find . -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum \
    | sed 's#  ./#  #' > SHA256SUMS
)

archive="$output_dir/$(basename "$package_dir")-store-submission-pack.tar.gz"
tar -C "$output_dir" -czf "$archive" "$(basename "$pack_root")"

echo "store submission pack created"
echo "pack: $pack_root"
echo "archive: $archive"
echo "screenshot_status: $screenshot_status"
echo "final approval: still requires screenshot review, storefront review, and final_signoff_check.sh --check"
