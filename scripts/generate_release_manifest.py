#!/usr/bin/env python3
import hashlib
import json
import os
import re
import sys
from pathlib import Path


def usage() -> None:
    print("usage: generate_release_manifest.py <package-dir> <platform>", file=sys.stderr)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def release_info_value(release_info: str, label: str) -> str:
    match = re.search(rf"^{re.escape(label)}:\s*(.+)$", release_info, re.MULTILINE)
    return match.group(1).strip() if match else ""


def file_role(relative: str) -> str:
    name = Path(relative).name
    if relative in {"bevy_open_siege", "bevy_open_siege.exe", "bevy_open_siege.bin"}:
        return "binary"
    if relative.startswith("assets/"):
        return "runtime_asset"
    if name.endswith("-audit.txt") or name.endswith("-smoke.txt") or name in {
        "campaign-simulation.txt",
        "release-readiness.txt",
        "release-info.txt",
        "store-asset-audit.txt",
        "content-rating-audit.txt",
        "manual-qa-plan.txt",
        "platform-package-plan.txt",
        "final-signoff-plan.txt",
    }:
        return "evidence"
    if name.endswith(".sh") or name.endswith(".ps1"):
        return "helper_script"
    if name in {"SHA256SUMS", "release-manifest.json"}:
        return "integrity"
    if name.endswith((".md", ".ron")):
        return "documentation"
    return "package_file"


def main() -> int:
    if len(sys.argv) != 3:
        usage()
        return 2

    package_dir = Path(sys.argv[1]).resolve()
    platform = sys.argv[2]
    if not package_dir.is_dir():
        print(f"package directory not found: {package_dir}", file=sys.stderr)
        return 1

    release_info_path = package_dir / "release-info.txt"
    if not release_info_path.is_file():
        print("release-manifest requires release-info.txt", file=sys.stderr)
        return 1

    release_info = read_text(release_info_path)
    package_name = package_dir.name
    files = []
    total_bytes = 0

    for path in sorted(package_dir.rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(package_dir).as_posix()
        if relative in {"release-manifest.json", "SHA256SUMS"}:
            continue
        size = path.stat().st_size
        total_bytes += size
        files.append(
            {
                "path": relative,
                "bytes": size,
                "sha256": sha256(path),
                "role": file_role(relative),
            }
        )

    required_evidence = [
        "release-readiness.txt",
        "balance-audit.txt",
        "asset-audit.txt",
        "audio-audit.txt",
        "controls-audit.txt",
        "input-flow-audit.txt",
        "localization-audit.txt",
        "layout-audit.txt",
        "visual-readability-audit.txt",
        "accessibility-audit.txt",
        "performance-audit.txt",
        "privacy-audit.txt",
        "release-provenance-audit.txt",
        "marketing-audit.txt",
        "ip-audit.txt",
        "save-audit.txt",
        "playthrough-audit.txt",
        "campaign-simulation.txt",
        "runtime-smoke.txt",
        "visual-smoke.txt",
        "audio-smoke.txt",
        "store-asset-audit.txt",
        "content-rating-audit.txt",
        "linux-install-smoke.txt",
        "linux-dependency-audit.txt",
        "linux-portability-smoke.txt",
        "linux-clean-distro-smoke.txt",
        "support_diagnostics.sh",
        "qa_signoff_prepare.sh",
        "store_asset_audit.sh",
        "content_rating_audit.sh",
        "signoff_bundle.sh",
        "create_candidate_evidence.sh",
        "create_store_submission_pack.sh",
        "manual-qa-plan.txt",
        "platform-package-plan.txt",
        "final-signoff-plan.txt",
    ]
    if platform == "linux-x86_64":
        required_evidence.extend(
            [
                "linux-package-audit.txt",
                "linux_install_smoke.sh",
                "linux_dependency_audit.sh",
                "linux_portability_smoke.sh",
                "linux_clean_distro_smoke.sh",
                "linux-metadata-audit.txt",
                "linux_metadata_audit.sh",
            ]
        )
    file_paths = {entry["path"] for entry in files}
    missing_evidence = [path for path in required_evidence if path not in file_paths]

    manifest = {
        "schema": "bevy-open-siege-release-manifest-v1",
        "product": release_info_value(release_info, "product") or "Bevy Open Siege",
        "version": release_info_value(release_info, "version"),
        "package": package_name,
        "platform": platform,
        "release_channel": release_info_value(release_info, "channel"),
        "binary": "bevy_open_siege.exe" if (package_dir / "bevy_open_siege.exe").exists() else "bevy_open_siege",
        "integrity": {
            "hash_file": "SHA256SUMS",
            "manifest_excludes": ["release-manifest.json", "SHA256SUMS"],
        },
        "qa": {
            "status": "release_candidate",
            "final_approval_required": True,
            "final_gate": "final_signoff_check.sh --check . qa-session platform-session",
            "required_evidence": required_evidence,
            "missing_evidence": missing_evidence,
        },
        "summary": {
            "file_count": len(files),
            "total_bytes": total_bytes,
            "roles": {role: sum(1 for entry in files if entry["role"] == role) for role in sorted({entry["role"] for entry in files})},
        },
        "files": files,
    }
    json.dump(manifest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 1 if missing_evidence else 0


if __name__ == "__main__":
    raise SystemExit(main())
