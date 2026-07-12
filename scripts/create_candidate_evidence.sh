#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  create_candidate_evidence.sh <package-dir> <output-dir>

Initializes manual QA templates, platform package QA templates, support diagnostics,
and a candidate signoff bundle for release-candidate handoff. This does not mark a
build final approved.
EOF
}

if [[ $# -ne 2 ]]; then
  usage
  exit 2
fi

package_dir="$(cd "$1" && pwd)"
output_dir="$2"
candidate_root="$output_dir/$(basename "$package_dir")-candidate-evidence"
qa_session="$candidate_root/qa-session"
platform_session="$candidate_root/platform-session"
diagnostics="$candidate_root/support-diagnostics"
bundle_output="$candidate_root/signoff-output"

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "candidate evidence requires packaged file: $path" >&2
    exit 1
  fi
}

if [[ ! -d "$package_dir" ]]; then
  echo "package directory not found: $package_dir" >&2
  exit 1
fi
if [[ ! -x "$package_dir/bevy_open_siege" && ! -x "$package_dir/bevy_open_siege.exe" ]]; then
  echo "candidate evidence requires an extracted Bevy Open Siege package" >&2
  exit 1
fi

for file in \
  manual_qa_session.sh \
  manual_qa_observations.sh \
  platform_package_plan.sh \
  support_diagnostics.sh \
  signoff_bundle.sh \
  qa_signoff_prepare.sh \
  qa_evidence_summary.sh \
  final_signoff_check.sh \
  QA_SIGNOFF.md \
  release-manifest.json \
  SHA256SUMS; do
  require_file "$package_dir/$file"
done

rm -rf "$candidate_root"
mkdir -p "$candidate_root"

"$package_dir/manual_qa_session.sh" --init "$package_dir" "$qa_session" > "$candidate_root/manual-qa-init.txt"
"$package_dir/manual_qa_observations.sh" --collect "$package_dir" "$qa_session" > "$candidate_root/manual-qa-observations.txt"
"$package_dir/platform_package_plan.sh" --init "$package_dir" "$platform_session" > "$candidate_root/platform-package-init.txt"
BEVY_OPEN_SIEGE_DIAGNOSTICS_SKIP_VERIFY=1 \
  "$package_dir/support_diagnostics.sh" "$package_dir" "$diagnostics" > "$candidate_root/support-diagnostics.txt"
"$package_dir/qa_evidence_summary.sh" --summary "$package_dir" "$qa_session" "$platform_session" > "$candidate_root/qa-evidence-summary.txt"
if "$package_dir/qa_signoff_prepare.sh" --check "$package_dir" "$qa_session" "$platform_session" > "$candidate_root/qa-signoff-prepare.txt" 2>&1; then
  echo "qa_signoff_prepare.sh unexpectedly accepted pending candidate evidence" >&2
  exit 1
fi
"$package_dir/signoff_bundle.sh" --create --allow-candidate "$package_dir" "$qa_session" "$platform_session" "$bundle_output" > "$candidate_root/signoff-bundle.txt"

cat > "$candidate_root/README.txt" <<EOF
Bevy Open Siege candidate evidence handoff
package: $(basename "$package_dir")
status: release candidate, not final approved

Contents:
- qa-session/: manual QA templates initialized with Pending status and automated observations.
- platform-session/: Windows/macOS package QA templates initialized with Pending status.
- support-diagnostics/: package metadata and diagnostics without copied personal files.
- signoff-output/: candidate signoff bundle created with --allow-candidate.
- qa-evidence-summary.txt: current evidence status for this candidate handoff.
- manual-qa-observations.txt: automated observation collection log.
- qa-signoff-prepare.txt: expected rejection while candidate evidence remains pending.

Final release still requires:
- Complete manual/platform evidence files as Pass or Scoped Out.
- Complete every QA_SIGNOFF.md table row with owner and YYYY-MM-DD date.
- Set QA_SIGNOFF.md Release approved: Yes.
- Run final_signoff_check.sh --check.
EOF

(
  cd "$candidate_root"
  find . -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum \
    | sed 's#  ./#  #' > SHA256SUMS
)

archive="$output_dir/$(basename "$package_dir")-candidate-evidence.tar.gz"
tar -C "$output_dir" -czf "$archive" "$(basename "$candidate_root")"

echo "candidate evidence created"
echo "candidate: $candidate_root"
echo "archive: $archive"
echo "final approval: still requires manual/platform QA and final_signoff_check.sh --check"
