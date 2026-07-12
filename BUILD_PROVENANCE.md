# Bevy Open Siege Build Provenance

This document records how a Bevy Open Siege release candidate is built, checked, packaged, and traced.

## Source Inputs

- `Cargo.toml` declares the package name, version, and Rust dependencies.
- `Cargo.lock` locks exact Rust dependency versions used for release builds.
- `VERSION.ron` declares the player-facing product name, release channel, supported languages, supported platforms, and content-rating note.
- `assets/manifest.ron` lists production art, audio, branding, metadata, gameplay data, and localization assets embedded into release audits.

## Build Entry Points

- Linux x86_64 package: `scripts/package_release.sh`
- Windows x86_64 package: `scripts/package_windows.ps1`
- macOS universal package: `scripts/package_macos.sh`
- Candidate evidence handoff: `scripts/create_candidate_evidence.sh`
- Store submission handoff: `scripts/create_store_submission_pack.sh`
- Release gate before packaging: `scripts/release_check.sh`

The Linux package script builds `target/release/bevy_open_siege`, generates deterministic audit reports, runs runtime/visual/audio smoke tests, writes `release-manifest.json`, writes `SHA256SUMS`, creates `dist/bevy_open_siege-0.1.0-linux-x86_64.tar.gz`, and runs `scripts/smoke_release_archive.sh` against the archive.

## Integrity Evidence

- `SHA256SUMS` records hashes for files inside the extracted package.
- `release-manifest.json` records the package name, version, platform, binary path, required evidence files, final approval gate, file roles, file sizes, and file hashes.
- `scripts/smoke_release_archive.sh` verifies `sha256sum -c SHA256SUMS`.
- `release-info.txt` is generated from `./bevy_open_siege --print-release-info`.
- `release-readiness.txt` is generated from `./bevy_open_siege --release-readiness`.
- `release-provenance-audit.txt` is generated from `./bevy_open_siege --audit-release-provenance`.

## Dependency License Evidence

- `scripts/generate_third_party_licenses.py` uses `cargo metadata --locked --filter-platform x86_64-unknown-linux-gnu`.
- `THIRD_PARTY_LICENSES.md` is generated during packaging and included in the archive.
- `scripts/smoke_release_archive.sh` fails if the generated dependency license report contains `UNKNOWN_LICENSE`.

## Final Approval

Automated release evidence is necessary but not sufficient for final release. The package remains a release candidate until:

- `manual_qa_session.sh --init . qa-session` evidence is completed.
- `platform_package_plan.sh --init . platform-session` evidence is completed or explicitly scoped.
- `final_signoff_check.sh --check . qa-session platform-session` passes.
- `QA_SIGNOFF.md` says `Release approved: Yes`.
