#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR=""
REMOTE="origin"
VERIFY_PAGES="1"

cleanup_tmpdir() {
  if [[ -n "${TMP_DIR:-}" && -d "$TMP_DIR" ]]; then
    rm -rf -- "$TMP_DIR"
  fi
}
trap cleanup_tmpdir EXIT

show_usage() {
  cat <<'EOF'
Usage:
  ./scripts/verify_github_release.sh <release-tag> [options]

Options:
  --remote <name>  Git remote used to resolve tags (default: origin)
  --skip-pages     Skip the Pages workflow and deployed-file verification

Examples:
  ./scripts/verify_github_release.sh v0.1.0
  ./scripts/verify_github_release.sh v0.1.0 --remote origin

The default check downloads every release asset, verifies its checksum, and
deeply validates both WASM bundles. Stable releases also check tag/latest
provenance, require a successful Pages workflow, and compare every deployed
Pages file with the release.
EOF
}

fail() {
  echo "GitHub release verification failed: $*" >&2
  exit 1
}

resolve_ref_commit() {
  local ref="$1"
  local target
  local commit_line

  if [[ "$ref" =~ ^[0-9a-fA-F]{40}$ ]]; then
    printf '%s\n' "${ref,,}"
    return 0
  fi

  for target in "refs/tags/${ref}^{}" "refs/tags/${ref}"; do
    if commit_line="$(git ls-remote --exit-code --quiet "$REMOTE" "$target" | awk 'NF {print $1; exit}')"; then
      printf '%s\n' "$commit_line"
      return 0
    fi
  done

  if commit_line="$(git ls-remote --exit-code --quiet "$REMOTE" "refs/heads/${ref}" | awk 'NF {print $1; exit}')"; then
    printf '%s\n' "$commit_line"
    return 0
  fi

  return 1
}

if [[ $# -lt 1 ]]; then
  show_usage
  exit 1
fi

RELEASE_TAG="$1"
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote)
      [[ $# -ge 2 ]] || fail "--remote requires a remote name"
      REMOTE="$2"
      shift 2
      ;;
    --skip-pages)
      VERIFY_PAGES="0"
      shift
      ;;
    -h|--help)
      show_usage
      exit 0
      ;;
    *)
      fail "unknown option: $1"
      ;;
  esac
done

for command_name in gh git jq sha256sum tar unzip; do
  command -v "$command_name" >/dev/null 2>&1 || fail "$command_name is required"
done

if [[ ! "$RELEASE_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  fail "release tag must be semantic, such as v0.1.0 or v0.1.0-rc.1: $RELEASE_TAG"
fi

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
VERSION="${RELEASE_TAG#v}"
BASE_VERSION="${VERSION%%-*}"

git ls-remote --exit-code --heads "$REMOTE" >/dev/null 2>&1 \
  || fail "remote '$REMOTE' is not reachable"
git ls-remote --exit-code --tags "$REMOTE" "refs/tags/$RELEASE_TAG" >/dev/null 2>&1 \
  || fail "tag $RELEASE_TAG is not present on remote '$REMOTE'"

release_tsv="$(
  gh api "repos/$REPO/releases/tags/$RELEASE_TAG" \
    --jq '[.id, .tag_name, .draft, .prerelease, .target_commitish, (.assets | length), .html_url] | @tsv' \
    2>/dev/null || true
)"
[[ -n "$release_tsv" ]] || fail "published release not found for $RELEASE_TAG"

IFS=$'\t' read -r RELEASE_ID API_TAG RELEASE_DRAFT RELEASE_PRERELEASE RELEASE_REF ASSET_COUNT RELEASE_URL \
  <<< "$release_tsv"
[[ -n "$RELEASE_ID" && "$RELEASE_ID" != "null" ]] || fail "release API returned no ID"
[[ "$API_TAG" == "$RELEASE_TAG" ]] || fail "release API tag mismatch: $API_TAG"
[[ "$RELEASE_DRAFT" == "false" ]] || fail "release $RELEASE_TAG is still a draft"

required_assets=(
  "bevy_open_siege-${BASE_VERSION}-linux-x86_64.tar.gz"
  "bevy_open_siege-${BASE_VERSION}-linux-x86_64.tar.gz.sha256"
  "bevy_open_siege-${BASE_VERSION}-web.tar.gz"
  "bevy_open_siege-${BASE_VERSION}-web.tar.gz.sha256"
  "bevy_open_siege-${BASE_VERSION}-windows-x86_64.zip"
  "bevy_open_siege-${BASE_VERSION}-windows-x86_64.zip.sha256"
  "bevy_open_siege-${BASE_VERSION}-macos-universal.tar.gz"
  "bevy_open_siege-${BASE_VERSION}-macos-universal.tar.gz.sha256"
  "bevy_open_siege-${BASE_VERSION}-release-metadata.txt"
)

asset_names="$(gh api "repos/$REPO/releases/$RELEASE_ID/assets" --paginate --jq '.[].name')"
for expected_asset in "${required_assets[@]}"; do
  grep -Fxq "$expected_asset" <<< "$asset_names" \
    || fail "release is missing required asset: $expected_asset"
done

TMP_DIR="$(mktemp -d)"
for expected_asset in "${required_assets[@]}"; do
  gh release download \
    "$RELEASE_TAG" \
    --repo "$REPO" \
    --dir "$TMP_DIR" \
    --pattern "$expected_asset" \
    >/dev/null
  [[ -f "$TMP_DIR/$expected_asset" ]] || fail "downloaded asset is missing: $expected_asset"
done

shopt -s nullglob
checksums=("$TMP_DIR"/*.sha256)
[[ ${#checksums[@]} -eq 4 ]] || fail "expected four downloaded checksum files"
for checksum_path in "${checksums[@]}"; do
  checksum_name="$(basename "$checksum_path")"
  archive_name="${checksum_name%.sha256}"
  mapfile -t checksum_lines < "$checksum_path"
  [[ ${#checksum_lines[@]} -eq 1 ]] \
    || fail "$checksum_name must contain exactly one checksum line"
  checksum_line="${checksum_lines[0]%$'\r'}"
  [[ "${checksum_line:0:64}" =~ ^[0-9a-f]{64}$ ]] \
    || fail "$checksum_name contains an invalid SHA256 digest"
  [[ "${checksum_line:64:2}" == "  " && "${checksum_line:66}" == "$archive_name" ]] \
    || fail "$checksum_name must reference only $archive_name"
  [[ -f "$TMP_DIR/$archive_name" ]] || fail "checksum target is missing: $archive_name"
  (
    cd "$TMP_DIR"
    sha256sum -c "$checksum_name"
  ) || fail "checksum verification failed for $checksum_name"
done

for platform in linux-x86_64 windows-x86_64 macos-universal; do
  case "$platform" in
    linux-x86_64)
      platform_archive="$TMP_DIR/bevy_open_siege-${BASE_VERSION}-linux-x86_64.tar.gz"
      ;;
    windows-x86_64)
      platform_archive="$TMP_DIR/bevy_open_siege-${BASE_VERSION}-windows-x86_64.zip"
      ;;
    macos-universal)
      platform_archive="$TMP_DIR/bevy_open_siege-${BASE_VERSION}-macos-universal.tar.gz"
      ;;
  esac
  "$ROOT/scripts/verify_platform_archive.sh" "$platform_archive" "$platform"
done

release_target="$(gh release view "$RELEASE_TAG" --repo "$REPO" --json targetCommitish --jq .targetCommitish)"
[[ -n "$release_target" ]] || fail "release target commit is empty"
[[ "$RELEASE_REF" == "$release_target" ]] \
  || fail "release target mismatch: API=$RELEASE_REF CLI=$release_target"

release_commit="$(resolve_ref_commit "$release_target")" \
  || fail "could not resolve release target: $release_target"
tag_commit="$(resolve_ref_commit "$RELEASE_TAG")" \
  || fail "could not resolve release tag: $RELEASE_TAG"

[[ "$release_commit" == "$tag_commit" ]] \
  || fail "release target $release_commit does not match tag commit $tag_commit"

latest_commit=""
if [[ "$RELEASE_PRERELEASE" == "false" ]]; then
  latest_commit="$(resolve_ref_commit latest)" \
    || fail "remote latest tag not found on $REMOTE"
  [[ "$latest_commit" == "$release_commit" ]] \
    || fail "latest tag $latest_commit does not match release commit $release_commit"
fi

metadata_path="$TMP_DIR/bevy_open_siege-${BASE_VERSION}-release-metadata.txt"
grep -Fxq "tag=$RELEASE_TAG" "$metadata_path" || fail "release metadata tag is incorrect"
grep -Fxq "version=$BASE_VERSION" "$metadata_path" || fail "release metadata version is incorrect"
grep -Fxq "commit=$release_commit" "$metadata_path" || fail "release metadata commit is incorrect"

web_archive="$TMP_DIR/bevy_open_siege-${BASE_VERSION}-web.tar.gz"
tar -tzf "$web_archive" > "$TMP_DIR/web-archive-files.txt"
if grep -Eq '(^/|(^|/)\.\.(/|$))' "$TMP_DIR/web-archive-files.txt"; then
  fail "Web release archive contains an unsafe path"
fi
web_bundle="$TMP_DIR/web-bundle"
mkdir -p "$web_bundle"
tar -xzf "$web_archive" -C "$web_bundle"
"$ROOT/scripts/verify_web_bundle.sh" \
  "$web_bundle" \
  --expected-version "$BASE_VERSION" \
  --expected-commit "$release_commit" \
  --expected-ref "$RELEASE_TAG"

PAGES_URL=""
PAGES_RUN_URL=""
if [[ "$VERIFY_PAGES" == "1" && "$RELEASE_PRERELEASE" == "false" ]]; then
  run_prefix="Deploy Pages release ${RELEASE_TAG} (release run "
  pages_run_json="$(
    gh run list \
      --repo "$REPO" \
      --workflow deploy-pages.yml \
      --event workflow_dispatch \
      --limit 100 \
      --json databaseId,displayTitle,status,conclusion,createdAt,url \
      | jq -c --arg prefix "$run_prefix" \
          '[.[] | select(.displayTitle | startswith($prefix))] | sort_by(.createdAt) | reverse | .[0] // empty'
  )"
  [[ -n "$pages_run_json" ]] || fail "no release Pages workflow found for $RELEASE_TAG"

  pages_status="$(jq -r .status <<< "$pages_run_json")"
  pages_conclusion="$(jq -r .conclusion <<< "$pages_run_json")"
  PAGES_RUN_URL="$(jq -r .url <<< "$pages_run_json")"
  [[ "$pages_status" == "completed" && "$pages_conclusion" == "success" ]] \
    || fail "release Pages workflow is not successful: status=$pages_status conclusion=$pages_conclusion"

  PAGES_URL="$(gh api "repos/$REPO/pages" --jq .html_url)"
  [[ -n "$PAGES_URL" && "$PAGES_URL" != "null" ]] || fail "repository has no GitHub Pages URL"
  "$ROOT/scripts/verify_pages_deployment.sh" "$PAGES_URL" "$web_bundle"
elif [[ "$RELEASE_PRERELEASE" == "true" ]]; then
  echo "Skipping public Pages equality check for prerelease $RELEASE_TAG"
fi

echo "GitHub release verification passed for $RELEASE_TAG"
echo "  target: $release_commit"
echo "  assets: $ASSET_COUNT"
echo "  release: $RELEASE_URL"
if [[ -n "$latest_commit" ]]; then
  echo "  latest: $latest_commit"
fi
if [[ -n "$PAGES_URL" ]]; then
  echo "  Pages: $PAGES_URL"
  echo "  Pages workflow: $PAGES_RUN_URL"
fi
