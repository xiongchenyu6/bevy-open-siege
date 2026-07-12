#!/usr/bin/env python3
import argparse
import json
import subprocess
import sys
from datetime import date


def cargo_metadata(platform: str) -> dict:
    command = [
        "cargo",
        "metadata",
        "--locked",
        "--format-version",
        "1",
        "--filter-platform",
        platform,
    ]
    result = subprocess.run(command, check=True, text=True, capture_output=True)
    return json.loads(result.stdout)


def package_sort_key(package: dict) -> tuple[str, str]:
    return (package["name"].lower(), package["version"])


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate a third-party dependency license report from Cargo metadata."
    )
    parser.add_argument(
        "--platform",
        default="x86_64-unknown-linux-gnu",
        help="Cargo target platform to report, default: x86_64-unknown-linux-gnu",
    )
    args = parser.parse_args()

    metadata = cargo_metadata(args.platform)
    packages = sorted(
        (package for package in metadata["packages"] if package.get("source")),
        key=package_sort_key,
    )
    missing = [
        f"{package['name']} {package['version']}"
        for package in packages
        if not (package.get("license") or package.get("license_file"))
    ]
    if missing:
        print("dependencies missing license metadata:", file=sys.stderr)
        for package in missing:
            print(f"- {package}", file=sys.stderr)
        return 1

    print("# Third-Party Dependency Licenses")
    print()
    print(
        f"Generated from `cargo metadata --locked --filter-platform {args.platform}` on {date.today().isoformat()}."
    )
    print()
    print(f"Dependency count: {len(packages)}")
    print()
    print("| Package | Version | License | Source |")
    print("| --- | --- | --- | --- |")
    for package in packages:
        license_text = package.get("license") or f"license file: {package['license_file']}"
        source = package.get("source") or "local"
        print(
            f"| `{package['name']}` | `{package['version']}` | {license_text} | `{source}` |"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
