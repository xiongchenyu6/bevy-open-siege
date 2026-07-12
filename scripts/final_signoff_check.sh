#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 --plan <extracted-package-dir> | --check <extracted-package-dir> <qa-session-dir> <platform-session-dir>" >&2
}

require_package() {
  local package_dir="$1"
  if [[ ! -d "$package_dir" ]]; then
    echo "package directory not found: $package_dir" >&2
    exit 1
  fi
  for file in \
    QA_SIGNOFF.md \
    release-readiness.txt \
    manual-qa-plan.txt \
    platform-package-plan.txt; do
    if [[ ! -f "$package_dir/$file" ]]; then
      echo "final signoff check requires packaged evidence file: $file" >&2
      exit 1
    fi
  done
}

require_integrity_file() {
  local package_dir="$1"
  if [[ ! -f "$package_dir/SHA256SUMS" ]]; then
    echo "final signoff check requires packaged evidence file: SHA256SUMS" >&2
    exit 1
  fi
  if [[ ! -f "$package_dir/release-manifest.json" ]]; then
    echo "final signoff check requires packaged evidence file: release-manifest.json" >&2
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

print_plan() {
  local package_dir="$1"
  local package_name
  package_name="$(basename "$package_dir")"
  cat <<EOF
final signoff plan ok
package: $package_name
required manual session: qa-session
required platform session: platform-session

before final approval:
- Run ./manual_qa_session.sh --init . qa-session and complete every generated manual evidence file.
- Run ./platform_package_plan.sh --init . platform-session and complete Windows/macOS package QA files, or explicitly scope a platform out in the corresponding file.
- Set every completed evidence file to Status: Pass or Status: Scoped Out.
- Set every completed evidence file to Approved: Yes.
- Set every QA_SIGNOFF.md table row to Pass or Scoped Out with Owner and YYYY-MM-DD Date filled.
- Add Notes for every Scoped Out QA_SIGNOFF.md row.
- Set QA_SIGNOFF.md Release approved: Yes only after evidence files and QA_SIGNOFF.md rows are complete.

final check command:
- ./final_signoff_check.sh --check . qa-session platform-session

required manual evidence:
- qa-session/full-campaign-playthrough.md
- qa-session/balance-usability.md
- qa-session/audio-device-qa.md
- qa-session/visual-hardware-spotcheck.md
- qa-session/localization-review.md
- qa-session/input-bindings.md
- qa-session/accessibility-qa.md
- qa-session/performance-qa.md
- qa-session/privacy-support.md
- qa-session/build-provenance.md
- qa-session/save-compatibility.md
- qa-session/store-screenshots.md
- qa-session/store-press-ip-review.md
- qa-session/final-art-direction.md

required platform evidence:
- platform-session/windows-package-qa.md
- platform-session/macos-package-qa.md

decision:
- Current package is a release candidate until this check passes with completed evidence.
EOF
}

require_completed_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "missing final signoff evidence: $path" >&2
    exit 1
  fi
  if grep -Eq '^Status:[[:space:]]*Pending[[:space:]]*$' "$path"; then
    echo "evidence still pending: $path" >&2
    exit 1
  fi
  if ! grep -Eq '^Status:[[:space:]]*(Pass|Scoped Out)[[:space:]]*$' "$path"; then
    echo "evidence must use Status: Pass or Status: Scoped Out: $path" >&2
    exit 1
  fi
  if ! grep -Eq '^Approved:[[:space:]]*Yes[[:space:]]*$' "$path"; then
    echo "evidence is not approved: $path" >&2
    exit 1
  fi
  if grep -Eq '^Follow-up required:[[:space:]]*Yes[[:space:]]*$' "$path"; then
    echo "evidence has unresolved follow-up: $path" >&2
    exit 1
  fi
}

trim_cell() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

check_qa_signoff_rows() {
  local signoff="$1"
  local row area evidence status owner date notes
  local checked=0
  while IFS= read -r row; do
    [[ "$row" == \|* ]] || continue
    IFS='|' read -r _ area evidence status owner date notes _ <<< "$row"
    area="$(trim_cell "${area:-}")"
    status="$(trim_cell "${status:-}")"
    owner="$(trim_cell "${owner:-}")"
    date="$(trim_cell "${date:-}")"
    notes="$(trim_cell "${notes:-}")"

    [[ "$area" != "Area" ]] || continue
    [[ "$area" != "---" ]] || continue
    [[ -n "$area" ]] || continue

    checked=$((checked + 1))
    case "$status" in
      Pass|"Scoped Out") ;;
      Pending)
        echo "QA_SIGNOFF.md row still pending: $area" >&2
        exit 1
        ;;
      *)
        echo "QA_SIGNOFF.md row must use Status Pass or Scoped Out: $area" >&2
        exit 1
        ;;
    esac
    if [[ -z "$owner" ]]; then
      echo "QA_SIGNOFF.md row missing owner: $area" >&2
      exit 1
    fi
    if [[ ! "$date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
      echo "QA_SIGNOFF.md row date must use YYYY-MM-DD: $area" >&2
      exit 1
    fi
    if [[ "$status" == "Scoped Out" && -z "$notes" ]]; then
      echo "QA_SIGNOFF.md scoped-out row requires notes: $area" >&2
      exit 1
    fi
  done < "$signoff"

  if [[ "$checked" -eq 0 ]]; then
    echo "QA_SIGNOFF.md contains no approval rows" >&2
    exit 1
  fi
}

check_signoff() {
  local package_dir="$1"
  local qa_session="$2"
  local platform_session="$3"
  require_package "$package_dir"
  require_integrity_file "$package_dir"
  if [[ ! -d "$qa_session" ]]; then
    echo "manual QA session directory not found: $qa_session" >&2
    exit 1
  fi
  if [[ ! -d "$platform_session" ]]; then
    echo "platform QA session directory not found: $platform_session" >&2
    exit 1
  fi

  (
    cd "$package_dir"
    sha256sum -c SHA256SUMS >/dev/null
  )

  for file in "${manual_files[@]}"; do
    require_completed_file "$qa_session/$file"
  done
  for file in "${platform_files[@]}"; do
    require_completed_file "$platform_session/$file"
  done

  if grep -q "Release approved: No" "$package_dir/QA_SIGNOFF.md"; then
    echo "QA_SIGNOFF.md is not approved" >&2
    exit 1
  fi
  if ! grep -q "Release approved: Yes" "$package_dir/QA_SIGNOFF.md"; then
    echo "QA_SIGNOFF.md must contain Release approved: Yes" >&2
    exit 1
  fi
  check_qa_signoff_rows "$package_dir/QA_SIGNOFF.md"

  echo "final signoff check passed"
  echo "package: $(basename "$package_dir")"
  echo "manual evidence: ${#manual_files[@]} files approved"
  echo "platform evidence: ${#platform_files[@]} files approved or scoped out"
  echo "qa signoff rows: complete"
}

case "${1:-}" in
  --plan)
    if [[ $# -ne 2 ]]; then
      usage
      exit 2
    fi
    require_package "$2"
    print_plan "$2"
    ;;
  --check)
    if [[ $# -ne 4 ]]; then
      usage
      exit 2
    fi
    check_signoff "$2" "$3" "$4"
    ;;
  *)
    usage
    exit 2
    ;;
esac
