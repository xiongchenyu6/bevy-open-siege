#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  qa_evidence_summary.sh --summary <extracted-package-dir> [qa-session-dir] [platform-session-dir]
EOF
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
  "create_store_submission_pack.sh"
  "SHA256SUMS"
)

require_package() {
  local package_dir="$1"
  if [[ ! -d "$package_dir" ]]; then
    echo "package directory not found: $package_dir" >&2
    exit 1
  fi
  if [[ ! -f "$package_dir/QA_SIGNOFF.md" ]]; then
    echo "QA evidence summary requires packaged QA_SIGNOFF.md" >&2
    exit 1
  fi
}

count_status() {
  local dir="$1"
  shift
  local total=0 missing=0 pending=0 pass=0 scoped_out=0 approved=0 follow_up=0 other=0
  local file path status
  for file in "$@"; do
    total=$((total + 1))
    path="$dir/$file"
    if [[ ! -f "$path" ]]; then
      missing=$((missing + 1))
      continue
    fi
    status="$(sed -n 's/^Status:[[:space:]]*//p' "$path" | head -n 1)"
    case "$status" in
      Pending) pending=$((pending + 1)) ;;
      Pass) pass=$((pass + 1)) ;;
      "Scoped Out") scoped_out=$((scoped_out + 1)) ;;
      *) other=$((other + 1)) ;;
    esac
    if grep -Eq '^Approved:[[:space:]]*Yes[[:space:]]*$' "$path"; then
      approved=$((approved + 1))
    fi
    if grep -Eq '^Follow-up required:[[:space:]]*Yes[[:space:]]*$' "$path"; then
      follow_up=$((follow_up + 1))
    fi
  done
  echo "total=${total} missing=${missing} pending=${pending} pass=${pass} scoped_out=${scoped_out} approved=${approved} follow_up=${follow_up} other=${other}"
}

list_missing_or_pending() {
  local label="$1"
  local dir="$2"
  shift 2
  local file path status
  for file in "$@"; do
    path="$dir/$file"
    if [[ ! -f "$path" ]]; then
      echo "${label}: missing ${file}"
      continue
    fi
    status="$(sed -n 's/^Status:[[:space:]]*//p' "$path" | head -n 1)"
    if [[ "$status" == "Pending" ]]; then
      echo "${label}: pending ${file}"
    elif [[ "$status" != "Pass" && "$status" != "Scoped Out" ]]; then
      echo "${label}: invalid-status ${file} (${status:-empty})"
    fi
    if ! grep -Eq '^Approved:[[:space:]]*Yes[[:space:]]*$' "$path"; then
      echo "${label}: unapproved ${file}"
    fi
    if grep -Eq '^Follow-up required:[[:space:]]*Yes[[:space:]]*$' "$path"; then
      echo "${label}: follow-up ${file}"
    fi
  done
}

trim_cell() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

count_qa_signoff_rows() {
  local signoff="$1"
  local row area status owner date notes
  local total=0 pending=0 pass=0 scoped_out=0 owner_missing=0 date_missing=0 scoped_notes_missing=0 other=0
  while IFS= read -r row; do
    [[ "$row" == \|* ]] || continue
    IFS='|' read -r _ area _ status owner date notes _ <<< "$row"
    area="$(trim_cell "${area:-}")"
    status="$(trim_cell "${status:-}")"
    owner="$(trim_cell "${owner:-}")"
    date="$(trim_cell "${date:-}")"
    notes="$(trim_cell "${notes:-}")"
    [[ "$area" != "Area" ]] || continue
    [[ "$area" != "---" ]] || continue
    [[ -n "$area" ]] || continue
    total=$((total + 1))
    case "$status" in
      Pending) pending=$((pending + 1)) ;;
      Pass) pass=$((pass + 1)) ;;
      "Scoped Out") scoped_out=$((scoped_out + 1)) ;;
      *) other=$((other + 1)) ;;
    esac
    if [[ -z "$owner" ]]; then
      owner_missing=$((owner_missing + 1))
    fi
    if [[ ! "$date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
      date_missing=$((date_missing + 1))
    fi
    if [[ "$status" == "Scoped Out" && -z "$notes" ]]; then
      scoped_notes_missing=$((scoped_notes_missing + 1))
    fi
  done < "$signoff"
  echo "total=${total} pending=${pending} pass=${pass} scoped_out=${scoped_out} owner_missing=${owner_missing} date_missing=${date_missing} scoped_notes_missing=${scoped_notes_missing} other=${other}"
}

print_summary() {
  local package_dir="$1"
  local qa_session="${2:-}"
  local platform_session="${3:-}"
  require_package "$package_dir"

  local automated_missing=0 file
  for file in "${automated_files[@]}"; do
    if [[ ! -f "$package_dir/$file" ]]; then
      automated_missing=$((automated_missing + 1))
    fi
  done

  echo "qa evidence summary ok"
  echo "package: $(basename "$package_dir")"
  echo "automated evidence: total=${#automated_files[@]} missing=${automated_missing}"
  if grep -q "automated evidence: pass" "$package_dir/release-readiness.txt" 2>/dev/null; then
    echo "release-readiness: automated pass"
  else
    echo "release-readiness: not verified"
  fi
  if grep -q "Release approved: Yes" "$package_dir/QA_SIGNOFF.md"; then
    echo "qa-signoff: approved"
  else
    echo "qa-signoff: not approved"
  fi
  echo "qa-signoff rows: $(count_qa_signoff_rows "$package_dir/QA_SIGNOFF.md")"

  if [[ -n "$qa_session" ]]; then
    if [[ -d "$qa_session" ]]; then
      echo "manual evidence: $(count_status "$qa_session" "${manual_files[@]}")"
      list_missing_or_pending "manual evidence" "$qa_session" "${manual_files[@]}"
    else
      echo "manual evidence: session missing"
    fi
  else
    echo "manual evidence: session not provided"
  fi

  if [[ -n "$platform_session" ]]; then
    if [[ -d "$platform_session" ]]; then
      echo "platform evidence: $(count_status "$platform_session" "${platform_files[@]}")"
      list_missing_or_pending "platform evidence" "$platform_session" "${platform_files[@]}"
    else
      echo "platform evidence: session missing"
    fi
  else
    echo "platform evidence: session not provided"
  fi

  echo "final decision: run final_signoff_check.sh --check only after all manual/platform evidence is Pass or Scoped Out and Approved: Yes"
}

case "${1:-}" in
  --summary)
    if [[ $# -lt 2 || $# -gt 4 ]]; then
      usage
      exit 2
    fi
    print_summary "$2" "${3:-}" "${4:-}"
    ;;
  *)
    usage
    exit 2
    ;;
esac
