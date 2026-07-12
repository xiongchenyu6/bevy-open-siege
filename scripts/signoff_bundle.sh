#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  signoff_bundle.sh --plan <package-dir>
  signoff_bundle.sh --create [--allow-candidate] <package-dir> <qa-session-dir> <platform-session-dir> <output-dir>

Creates a final signoff evidence archive. Without --allow-candidate, creation first
requires final_signoff_check.sh --check to pass.
EOF
}

mode="${1:-}"
if [[ "$mode" != "--plan" && "$mode" != "--create" ]]; then
  usage
  exit 2
fi
shift

allow_candidate="0"
if [[ "$mode" == "--create" && "${1:-}" == "--allow-candidate" ]]; then
  allow_candidate="1"
  shift
fi

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "signoff bundle missing required file: $path" >&2
    exit 1
  fi
}

package_name() {
  basename "$1"
}

if [[ "$mode" == "--plan" ]]; then
  if [[ $# -ne 1 ]]; then
    usage
    exit 2
  fi
  package_dir="$1"
  if [[ ! -d "$package_dir" ]]; then
    echo "package directory not found: $package_dir" >&2
    exit 1
  fi
  cat <<EOF
signoff bundle plan ok
package: $(package_name "$package_dir")
default output: signoff-bundle

candidate preflight:
- ./signoff_bundle.sh --create --allow-candidate . qa-session platform-session signoff-bundle

final approval archive:
- ./signoff_bundle.sh --create . qa-session platform-session signoff-bundle

contents:
- package automated evidence and release documents
- qa-session manual evidence
- platform-session package evidence
- qa-evidence-summary.txt
- final-signoff-check.txt
- bundle-manifest.txt
- SHA256SUMS
EOF
  exit 0
fi

if [[ $# -ne 4 ]]; then
  usage
  exit 2
fi

package_dir="$(cd "$1" && pwd)"
qa_session="$(cd "$2" && pwd)"
platform_session="$(cd "$3" && pwd)"
output_dir="$4"
package="$(package_name "$package_dir")"
bundle_root="$output_dir/${package}-signoff-bundle"

for file in \
  QA_SIGNOFF.md \
  release-readiness.txt \
  release-manifest.json \
  release-info.txt \
  SHA256SUMS \
  qa_evidence_summary.sh \
  final_signoff_check.sh; do
  require_file "$package_dir/$file"
done

if [[ ! -d "$qa_session" ]]; then
  echo "manual QA session directory not found: $qa_session" >&2
  exit 1
fi
if [[ ! -d "$platform_session" ]]; then
  echo "platform QA session directory not found: $platform_session" >&2
  exit 1
fi

rm -rf "$bundle_root"
mkdir -p "$bundle_root/package-evidence" "$bundle_root/qa-session" "$bundle_root/platform-session"

if [[ "$allow_candidate" == "1" ]]; then
  {
    echo "candidate signoff bundle"
    echo "final_signoff_check.sh --check skipped by --allow-candidate"
    "$package_dir/final_signoff_check.sh" --plan "$package_dir"
  } > "$bundle_root/final-signoff-check.txt"
else
  "$package_dir/final_signoff_check.sh" --check "$package_dir" "$qa_session" "$platform_session" > "$bundle_root/final-signoff-check.txt"
fi

"$package_dir/qa_evidence_summary.sh" --summary "$package_dir" "$qa_session" "$platform_session" > "$bundle_root/qa-evidence-summary.txt"

while IFS= read -r file; do
  cp "$package_dir/$file" "$bundle_root/package-evidence/$file"
done <<'EOF'
QA_SIGNOFF.md
release-info.txt
release-readiness.txt
release-manifest.json
SHA256SUMS
CONTENT_RATING.md
balance-audit.txt
asset-audit.txt
audio-audit.txt
controls-audit.txt
input-flow-audit.txt
localization-audit.txt
layout-audit.txt
visual-readability-audit.txt
accessibility-audit.txt
performance-audit.txt
privacy-audit.txt
release-provenance-audit.txt
marketing-audit.txt
ip-audit.txt
save-audit.txt
playthrough-audit.txt
campaign-simulation.txt
runtime-smoke.txt
visual-smoke.txt
audio-smoke.txt
store-asset-audit.txt
content-rating-audit.txt
linux-package-audit.txt
linux-install-smoke.txt
linux-dependency-audit.txt
linux-portability-smoke.txt
linux-clean-distro-smoke.txt
linux-metadata-audit.txt
manual-qa-plan.txt
platform-package-plan.txt
final-signoff-plan.txt
EOF

cp -R "$qa_session/." "$bundle_root/qa-session/"
cp -R "$platform_session/." "$bundle_root/platform-session/"

{
  echo "Bevy Open Siege signoff evidence bundle"
  echo "package: $package"
  echo "candidate_mode: $allow_candidate"
  echo "package_dir: $package_dir"
  echo "qa_session: $qa_session"
  echo "platform_session: $platform_session"
  date -u +"created_utc: %Y-%m-%dT%H:%M:%SZ"
  echo "final_gate: final_signoff_check.sh --check"
  echo "privacy: bundle contains QA notes and package evidence only; review before sharing"
} > "$bundle_root/bundle-manifest.txt"

(
  cd "$bundle_root"
  find . -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum \
    | sed 's#  ./#  #' > SHA256SUMS
)

archive="$output_dir/${package}-signoff-bundle.tar.gz"
tar -C "$output_dir" -czf "$archive" "${package}-signoff-bundle"

echo "signoff bundle created"
echo "bundle: $bundle_root"
echo "archive: $archive"
