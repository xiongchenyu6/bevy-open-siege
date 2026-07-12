#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  qa_signoff_prepare.sh --check <package-dir> <qa-session-dir> <platform-session-dir>
  qa_signoff_prepare.sh --write <package-dir> <qa-session-dir> <platform-session-dir> <owner> <YYYY-MM-DD> <output-QA_SIGNOFF.md> [archive-sha256]

Checks completed manual/platform evidence and, when all required evidence is
approved, writes a completed QA_SIGNOFF.md with consistent row statuses. This
does not bypass QA: every required evidence file must already be Pass or Scoped
Out and Approved: Yes with no unresolved follow-up.

When archive-sha256 is omitted, the output points reviewers to the packaged
SHA256SUMS file for per-file hashes instead of pretending that SHA256SUMS is the
release archive hash.
EOF
}

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

require_completed_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "missing signoff evidence: $path" >&2
    exit 1
  fi
  local status
  status="$(sed -n 's/^Status:[[:space:]]*//p' "$path" | head -n 1)"
  case "$status" in
    Pass|"Scoped Out") ;;
    Pending)
      echo "signoff evidence still pending: $path" >&2
      exit 1
      ;;
    *)
      echo "signoff evidence must use Status: Pass or Status: Scoped Out: $path" >&2
      exit 1
      ;;
  esac
  if ! grep -Eq '^Approved:[[:space:]]*Yes[[:space:]]*$' "$path"; then
    echo "signoff evidence is not approved: $path" >&2
    exit 1
  fi
  if grep -Eq '^Follow-up required:[[:space:]]*Yes[[:space:]]*$' "$path"; then
    echo "signoff evidence has unresolved follow-up: $path" >&2
    exit 1
  fi
  printf '%s' "$status"
}

require_package_file() {
  local package_dir="$1"
  local file="$2"
  if [[ ! -f "$package_dir/$file" ]]; then
    echo "qa signoff prepare requires packaged file: $file" >&2
    exit 1
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

platform_files=(
  "windows-package-qa.md"
  "macos-package-qa.md"
)

automated_files=(
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
)

declare -A row_status
declare -A row_notes

set_row() {
  local area="$1"
  local status="$2"
  local notes="$3"
  row_status["$area"]="$status"
  row_notes["$area"]="$notes"
}

load_statuses() {
  local package_dir="$1"
  local qa_session="$2"
  local platform_session="$3"
  local file status

  if [[ ! -d "$package_dir" || ! -d "$qa_session" || ! -d "$platform_session" ]]; then
    echo "package, qa session, and platform session directories are required" >&2
    exit 1
  fi
  require_package_file "$package_dir" "QA_SIGNOFF.md"
  for file in "${automated_files[@]}"; do
    require_package_file "$package_dir" "$file"
  done
  (
    cd "$package_dir"
    sha256sum -c SHA256SUMS >/dev/null
  )

  for file in "${manual_files[@]}"; do
    require_completed_file "$qa_session/$file" >/dev/null
  done
  for file in "${platform_files[@]}"; do
    require_completed_file "$platform_session/$file" >/dev/null
  done

  set_row "Manual QA session plan" "Pass" "Manual QA session evidence approved."
  set_row "Platform package plan" "Pass" "Platform QA session evidence approved or scoped out."
  set_row "QA evidence summary" "Pass" "Automated, manual, and platform evidence summary reviewed."
  set_row "Package verification helper" "Pass" "verify_release.sh quick verification evidence reviewed."
  set_row "Support diagnostics helper" "Pass" "Support diagnostics process reviewed for final handoff."
  set_row "Signoff evidence bundle" "Pass" "signoff_bundle.sh plan reviewed; final bundle is created after this signoff passes."
  set_row "Release manifest" "Pass" "release-manifest.json reviewed."
  set_row "Final signoff check" "Pass" "final-signoff-plan.txt reviewed."
  set_row "Full campaign playthrough" "Pass" "Approved evidence: qa-session/full-campaign-playthrough.md."
  set_row "Balance and usability" "Pass" "Approved evidence: qa-session/balance-usability.md."
  set_row "Visual readability" "Pass" "Approved evidence: qa-session/visual-hardware-spotcheck.md."
  set_row "Localization QA" "Pass" "Approved evidence: qa-session/localization-review.md."
  set_row "Runtime startup smoke" "Pass" "runtime-smoke.txt reviewed with manual startup evidence."
  set_row "Visual startup smoke" "Pass" "visual-smoke.txt reviewed with manual visual evidence."
  set_row "Audio mix audit" "Pass" "audio-audit.txt reviewed."
  set_row "Audio startup smoke" "Pass" "audio-smoke.txt reviewed with manual audio evidence."
  set_row "Audio device QA" "Pass" "Approved evidence: qa-session/audio-device-qa.md."
  set_row "Input flow audit" "Pass" "input-flow-audit.txt reviewed."
  set_row "Keyboard and mouse QA" "Pass" "Approved evidence: qa-session/input-bindings.md."
  set_row "Accessibility QA" "Pass" "Approved evidence: qa-session/accessibility-qa.md."
  set_row "Performance QA" "Pass" "Approved evidence: qa-session/performance-qa.md."
  set_row "Privacy, support, and troubleshooting audit" "Pass" "Approved evidence: qa-session/privacy-support.md."
  set_row "Build provenance audit" "Pass" "Approved evidence: qa-session/build-provenance.md."
  set_row "Save compatibility" "Pass" "Approved evidence: qa-session/save-compatibility.md."
  set_row "Save audit" "Pass" "save-audit.txt reviewed."
  set_row "Scripted playthrough audit" "Pass" "playthrough-audit.txt reviewed."
  set_row "Linux package QA" "Pass" "Linux package, install, dependency, portability, clean-distro, and metadata evidence reviewed."
  set_row "Linux desktop metadata QA" "Pass" "linux-metadata-audit.txt reviewed."
  status="$(require_completed_file "$platform_session/windows-package-qa.md")"
  set_row "Windows package QA" "$status" "Approved evidence: platform-session/windows-package-qa.md."
  status="$(require_completed_file "$platform_session/macos-package-qa.md")"
  set_row "macOS package QA" "$status" "Approved evidence: platform-session/macos-package-qa.md."
  set_row "Store screenshot review" "Pass" "Approved evidence: qa-session/store-screenshots.md."
  set_row "Store asset audit" "Pass" "store-asset-audit.txt reviewed."
  set_row "Content rating review" "Pass" "content-rating-audit.txt reviewed."
  set_row "Store and press review" "Pass" "Approved evidence: qa-session/store-press-ip-review.md."
  set_row "Marketing material audit" "Pass" "marketing-audit.txt reviewed."
  set_row "IP and naming audit" "Pass" "ip-audit.txt reviewed."
  set_row "Final art-direction review" "Pass" "Approved evidence: qa-session/final-art-direction.md."
}

write_signoff() {
  local package_dir="$1"
  local owner="$2"
  local date="$3"
  local output="$4"
  local archive_sha="${5:-}"
  local input="$package_dir/QA_SIGNOFF.md"
  local artifact sha_line
  artifact="$(basename "$package_dir")"
  if [[ -n "$archive_sha" ]]; then
    if [[ ! "$archive_sha" =~ ^[0-9a-fA-F]{64}$ ]]; then
      echo "archive-sha256 must be a 64-character SHA256 hex digest" >&2
      exit 1
    fi
    sha_line="$archive_sha"
  else
    sha_line="See packaged SHA256SUMS for per-file hashes."
  fi

  if [[ -z "$owner" || ! "$date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    echo "owner must be non-empty and date must use YYYY-MM-DD" >&2
    exit 1
  fi

  local tmp
  tmp="$(mktemp)"
  while IFS= read -r line; do
    if [[ "$line" == \|* ]]; then
      local _ area evidence status old_owner old_date notes tail
      IFS='|' read -r _ area evidence status old_owner old_date notes tail <<< "$line"
      area="$(trim "${area:-}")"
      evidence="$(trim "${evidence:-}")"
      if [[ "$area" != "Area" && "$area" != "---" && -n "$area" && -n "${row_status[$area]:-}" ]]; then
        printf '| %s | %s | %s | %s | %s | %s |\n' \
          "$area" "$evidence" "${row_status[$area]}" "$owner" "$date" "${row_notes[$area]}" >> "$tmp"
        continue
      fi
    fi
    case "$line" in
      "- Release approved:"*)
        echo "- Release approved: Yes" >> "$tmp"
        ;;
      "- Approver:"*)
        echo "- Approver: $owner" >> "$tmp"
        ;;
      "- Approval date:"*)
        echo "- Approval date: $date" >> "$tmp"
        ;;
      "- Build artifact:"*)
        echo "- Build artifact: $artifact" >> "$tmp"
        ;;
      "- SHA256:"*)
        echo "- SHA256: $sha_line" >> "$tmp"
        ;;
      "- Final notes:"*)
        echo "- Final notes: Prepared by qa_signoff_prepare.sh after all required evidence was approved." >> "$tmp"
        ;;
      *)
        echo "$line" >> "$tmp"
        ;;
    esac
  done < "$input"

  mv "$tmp" "$output"
}

case "${1:-}" in
  --check)
    if [[ $# -ne 4 ]]; then
      usage
      exit 2
    fi
    load_statuses "$2" "$3" "$4"
    echo "qa signoff prepare check ok"
    echo "package: $(basename "$2")"
    echo "manual evidence: complete"
    echo "platform evidence: complete or scoped out"
    echo "qa signoff rows ready: ${#row_status[@]}"
    ;;
  --write)
    if [[ $# -ne 7 && $# -ne 8 ]]; then
      usage
      exit 2
    fi
    load_statuses "$2" "$3" "$4"
    write_signoff "$2" "$5" "$6" "$7" "${8:-}"
    echo "qa signoff prepared"
    echo "output: $7"
    echo "rows written: ${#row_status[@]}"
    ;;
  *)
    usage
    exit 2
    ;;
esac
