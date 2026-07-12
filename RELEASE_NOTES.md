# Bevy Open Siege Release Notes

## 0.1.0 Release Candidate

This Linux x86_64 package contains the full 10-level Bevy Open Siege campaign, 10 playable plant types, 10 enemy types, English and Chinese localization, Blender-generated GLB runtime unit models, production PNG runtime art, synthesized WAV music and sound effects, Linux desktop integration metadata, privacy/support/troubleshooting documentation, user-local install scripts, and release smoke tests.

### Package Contents

- Playable Bevy game binary: `bevy_open_siege`
- Gameplay data: `assets/data/levels.ron`
- Localization data: `assets/i18n/en.ron` and `assets/i18n/zh.ron`
- Runtime art and branding assets under `assets/art/` and `assets/branding/`
- Runtime 3D plant and monster models under `assets/models/`
- UI chrome panels under `assets/art/ui/`
- Synthesized audio under `assets/audio/`
- Linux launcher and AppStream metadata under `assets/linux/`
- Audio and runtime smoke helper scripts: `audio_smoke.sh` and `runtime_smoke.sh`
- Documentation, credits, asset notes, store copy, press kit, build provenance, and release checklist
- Store screenshot capture checklist: `STORE_SCREENSHOTS.md`
- Store screenshot helper script: `store_screenshot_check.sh`
- Content rating notes: `CONTENT_RATING.md`
- Privacy notice, support guide, and troubleshooting runbook: `PRIVACY.md`, `SUPPORT.md`, and `TROUBLESHOOTING.md`
- Support diagnostics helper: `support_diagnostics.sh`
- Machine-readable package inventory and QA summary: `release-manifest.json`
- Build provenance notes: `BUILD_PROVENANCE.md`
- Manual QA signoff template: `QA_SIGNOFF.md`
- Generated Cargo dependency license report: `THIRD_PARTY_LICENSES.md`
- Embedded asset audit report: `asset-audit.txt`
- Audio mix and startup policy audit report: `audio-audit.txt`
- Input binding audit report: `controls-audit.txt`
- Input flow semantics audit report: `input-flow-audit.txt`
- Bilingual localization coverage report: `localization-audit.txt`
- UI text layout audit report: `layout-audit.txt`
- Visual readability audit report: `visual-readability-audit.txt`
- Accessibility audit report: `accessibility-audit.txt`
- Performance budget audit report: `performance-audit.txt`
- Privacy and support audit report: `privacy-audit.txt`
- Release provenance audit report: `release-provenance-audit.txt`
- Store and press material audit report: `marketing-audit.txt`
- IP and naming separation audit report: `ip-audit.txt`
- Save compatibility audit report: `save-audit.txt`
- Scripted playthrough lifecycle audit report: `playthrough-audit.txt`
- Headless campaign QA report: `campaign-simulation.txt`
- Release-candidate readiness snapshot: `release-readiness.txt`
- Windowed runtime startup smoke report: `runtime-smoke.txt`
- Windowed visual screenshot smoke report: `visual-smoke.txt`
- Windowed audio startup smoke report: `audio-smoke.txt`
- Store artwork and screenshot-plan audit report: `store-asset-audit.txt`
- Content rating audit report: `content-rating-audit.txt`
- Linux package install/uninstall audit report: `linux-package-audit.txt`
- Linux installed-launcher runtime and visual smoke report: `linux-install-smoke.txt`
- Linux runtime dependency audit report: `linux-dependency-audit.txt`
- Linux sanitized-environment portability smoke report: `linux-portability-smoke.txt`
- Linux clean-distro container smoke report: `linux-clean-distro-smoke.txt`
- Linux desktop metadata audit report: `linux-metadata-audit.txt`
- Manual QA session plan: `manual-qa-plan.txt`
- Manual QA observations helper: `manual_qa_observations.sh`
- Windows/macOS package plan: `platform-package-plan.txt`
- QA evidence summary helper: `qa_evidence_summary.sh`
- QA signoff preparation helper: `qa_signoff_prepare.sh`
- Package verification helper: `verify_release.sh`
- Signoff evidence bundle helper: `signoff_bundle.sh`
- Candidate evidence handoff helper: `create_candidate_evidence.sh`
- Store submission pack helper: `create_store_submission_pack.sh`
- Final signoff plan: `final-signoff-plan.txt`
- Windows/macOS package entry points: `package_windows.ps1` and `package_macos.sh`
- Archive integrity file: `SHA256SUMS`

### Known Release Review Items

- Audio remains opt-in with `--audio` or `BEVY_OPEN_SIEGE_AUDIO=1` while device compatibility is being hardened. Audio backend initialization runs asynchronously so slow or unavailable output devices do not block window startup.
- Final approval should include repeated manual playthroughs of all 10 levels for balance and usability.
- Final store submission should include the manually approved screenshot set from `STORE_SCREENSHOTS.md`.
- Use `store_screenshot_check.sh --validate-dir screenshots` before approving final screenshot uploads.
- The current packaged platform is Linux x86_64; Windows and macOS packages require separate host builds with `package_windows.ps1` and `package_macos.sh`, followed by smoke-test passes.

### Verification

Run the packaged binary with:

```bash
./bevy_open_siege --validate-data
./bevy_open_siege --audit-balance
./bevy_open_siege --audit-assets
./bevy_open_siege --audit-audio
./bevy_open_siege --audit-controls
./bevy_open_siege --audit-input-flow
./bevy_open_siege --audit-localization
./bevy_open_siege --audit-layout
./bevy_open_siege --audit-visual
./bevy_open_siege --audit-accessibility
./bevy_open_siege --audit-performance
./bevy_open_siege --audit-privacy
./bevy_open_siege --audit-release-provenance
./bevy_open_siege --audit-marketing
./bevy_open_siege --audit-ip
./bevy_open_siege --audit-save
./bevy_open_siege --audit-playthrough
./bevy_open_siege --simulate-campaign
./bevy_open_siege --release-readiness
./bevy_open_siege --print-release-info
./bevy_open_siege --print-save-summary
./audio_smoke.sh ./bevy_open_siege
./runtime_smoke.sh ./bevy_open_siege
./visual_smoke.sh ./bevy_open_siege
./store_asset_audit.sh .
./content_rating_audit.sh .
./linux_package_audit.sh .
./linux_install_smoke.sh .
./linux_dependency_audit.sh .
./linux_portability_smoke.sh .
./linux_clean_distro_smoke.sh .
./linux_metadata_audit.sh .
./manual_qa_session.sh --plan .
./manual_qa_session.sh --init . qa-session
./manual_qa_observations.sh --collect . qa-session screenshots
./platform_package_plan.sh --plan .
./platform_package_plan.sh --init . platform-session
./qa_evidence_summary.sh --summary . qa-session platform-session
./qa_signoff_prepare.sh --check . qa-session platform-session
./qa_signoff_prepare.sh --write . qa-session platform-session "Reviewer Name" YYYY-MM-DD QA_SIGNOFF.md
./verify_release.sh --quick .
./final_signoff_check.sh --plan .
./final_signoff_check.sh --check . qa-session platform-session
powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1
scripts/package_macos.sh
```

For archive-level verification, run:

```bash
scripts/smoke_release_archive.sh dist/bevy_open_siege-0.1.0-linux-x86_64.tar.gz
```
