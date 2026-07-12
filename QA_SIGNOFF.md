# Bevy Open Siege QA Signoff

This form records the manual approvals required after automated release gates pass.

## Release Candidate

- Product: Bevy Open Siege
- Version: 0.1.0
- Package: linux-x86_64 release candidate
- Automated evidence to attach: `release-readiness.txt`, `balance-audit.txt`, `asset-audit.txt`, `audio-audit.txt`, `controls-audit.txt`, `input-flow-audit.txt`, `localization-audit.txt`, `layout-audit.txt`, `visual-readability-audit.txt`, `accessibility-audit.txt`, `performance-audit.txt`, `privacy-audit.txt`, `release-provenance-audit.txt`, `marketing-audit.txt`, `ip-audit.txt`, `save-audit.txt`, `playthrough-audit.txt`, `campaign-simulation.txt`, `runtime-smoke.txt`, `visual-smoke.txt`, `audio-smoke.txt`, `store-asset-audit.txt`, `content-rating-audit.txt`, `linux-package-audit.txt`, `linux-install-smoke.txt`, `linux-dependency-audit.txt`, `linux-portability-smoke.txt`, `linux-clean-distro-smoke.txt`, `linux-metadata-audit.txt`, `manual-qa-plan.txt`, `platform-package-plan.txt`, `final-signoff-plan.txt`, `release-manifest.json`, `package_windows.ps1`, `package_macos.sh`, `qa_evidence_summary.sh`, `verify_release.sh`, `support_diagnostics.sh`, `signoff_bundle.sh`, `create_candidate_evidence.sh`, `create_store_submission_pack.sh`, `store_asset_audit.sh`, `content_rating_audit.sh`, `linux_install_smoke.sh`, `linux_dependency_audit.sh`, `linux_portability_smoke.sh`, `linux_clean_distro_smoke.sh`, `linux_metadata_audit.sh`, `manual_qa_observations.sh`, `PRIVACY.md`, `SUPPORT.md`, `TROUBLESHOOTING.md`, `CONTENT_RATING.md`, `BUILD_PROVENANCE.md`, `STORE_SCREENSHOTS.md`, `store_screenshot_check.sh`, `release-info.txt`, `SHA256SUMS`

## Manual Approval Required

Complete every row before treating a build as final release approved.

| Area | Required Evidence | Status | Owner | Date | Notes |
| --- | --- | --- | --- | --- | --- |
| Manual QA session plan | Review `manual-qa-plan.txt`, run `manual_qa_session.sh --init . qa-session`, and use its commands and evidence filenames for the manual pass | Pending |  |  |  |
| Platform package plan | Review `platform-package-plan.txt`, run `platform_package_plan.sh --init . platform-session`, and use its templates for Windows/macOS package QA | Pending |  |  |  |
| QA evidence summary | Run `qa_evidence_summary.sh --summary . qa-session platform-session` and confirm no required manual or platform evidence is missing before final signoff | Pending |  |  |  |
| Package verification helper | Run `verify_release.sh --quick .` from the extracted package and attach the output before final signoff | Pending |  |  |  |
| Support diagnostics helper | Run `support_diagnostics.sh . support-diagnostics` and confirm the folder contains metadata, integrity, save-path, privacy, verification, and manifest outputs without copied personal files | Pending |  |  |  |
| Signoff evidence bundle | Review `signoff_bundle.sh --plan .` before final signoff, then run `signoff_bundle.sh --create . qa-session platform-session signoff-bundle` after final signoff passes and attach the generated archive | Pending |  |  |  |
| Release manifest | Review `release-manifest.json` and confirm product, version, platform, binary path, required evidence, and final approval gate match the candidate package | Pending |  |  |  |
| Final signoff check | Review `final-signoff-plan.txt`, then run `final_signoff_check.sh --check . qa-session platform-session` after manual and platform evidence is complete | Pending |  |  |  |
| Full campaign playthrough | Finish all 10 levels from a clean save and confirm unlock progression, victory, defeat, restart, and score recording | Pending |  |  |  |
| Balance and usability | Repeat-play all 10 levels and confirm difficulty, seed costs, cooldowns, wave pacing, and score pacing are acceptable | Pending |  |  |  |
| Visual readability | Review `visual-readability-audit.txt`, then spot-check menu, HUD, board, plants, monsters, effects, and end screens on release hardware | Pending |  |  |  |
| Localization QA | Review `localization-audit.txt`, then manually verify English and Chinese copy quality in menu, HUD, level select, gameplay, and end screens | Pending |  |  |  |
| Runtime startup smoke | Review `runtime-smoke.txt` and confirm the windowed build starts without panic signatures on the release QA machine | Pending |  |  |  |
| Visual startup smoke | Review `visual-smoke.txt` and confirm the captured game window is nonblank before final visual spot-check | Pending |  |  |  |
| Audio mix audit | Review `audio-audit.txt` and confirm WAV format, duration, clipping headroom, non-silence, and opt-in startup policy evidence is attached | Pending |  |  |  |
| Audio startup smoke | Review `audio-smoke.txt`, rerun packaged `audio_smoke.sh`, and confirm `--audio` reaches the game window without panic signatures | Pending |  |  |  |
| Audio device QA | Test music and sound effects on speakers and headphones with `--audio`, then decide whether audio should remain opt-in or become default | Pending |  |  |  |
| Input flow audit | Review `input-flow-audit.txt` and confirm deterministic menu navigation, locked-level gating, seed selection, planting blocks, and pause gating evidence is attached | Pending |  |  |  |
| Keyboard and mouse QA | Execute every binding listed in `controls-audit.txt`, including mouse placement, shovel, pause, fullscreen, volume, language, and restart | Pending |  |  |  |
| Accessibility QA | Review `accessibility-audit.txt`, then manually check assistive tech behavior, input remapping expectations, photosensitivity risk, and readability on release hardware | Pending |  |  |  |
| Performance QA | Review `performance-audit.txt`, then profile release hardware for frame pacing, long-session stability, memory growth, and worst-wave responsiveness | Pending |  |  |  |
| Privacy, support, and troubleshooting audit | Review `privacy-audit.txt`, `PRIVACY.md`, `SUPPORT.md`, and `TROUBLESHOOTING.md`, then confirm offline/no-telemetry disclosure, local-save fields, uninstall preservation, explicit `--purge`, bug-report evidence guidance, and troubleshooting commands | Pending |  |  |  |
| Build provenance audit | Review `release-provenance-audit.txt`, `BUILD_PROVENANCE.md`, `SHA256SUMS`, `release-info.txt`, dependency license evidence, package host details, and final signoff rules | Pending |  |  |  |
| Save compatibility | Verify old working-directory saves, XDG user-data saves, settings persistence, and `BEVY_OPEN_SIEGE_SAVE_PATH` override behavior | Pending |  |  |  |
| Save audit | Review `save-audit.txt` and confirm automated save path, legacy parsing, normalization, and settings clamp evidence is attached | Pending |  |  |  |
| Scripted playthrough audit | Review `playthrough-audit.txt` and confirm automated victory, defeat, restart, unlock, and score lifecycle evidence is attached | Pending |  |  |  |
| Linux package QA | Review `linux-package-audit.txt`, `linux-install-smoke.txt`, `linux-dependency-audit.txt`, `linux-portability-smoke.txt`, and `linux-clean-distro-smoke.txt`, then install, run, uninstall, and purge the Linux x86_64 archive on a clean user profile | Pending |  |  |  |
| Linux desktop metadata QA | Review `linux-metadata-audit.txt`, then confirm `.desktop`, AppStream, icon, category, version, and launch metadata are acceptable for Linux storefront submission | Pending |  |  |  |
| Windows package QA | Complete `platform-session/windows-package-qa.md`, run `package_windows.ps1`, then build, run, smoke test, and archive a Windows package | Pending |  |  |  |
| macOS package QA | Complete `platform-session/macos-package-qa.md`, run `package_macos.sh`, then build, run, smoke test, and archive a macOS package | Pending |  |  |  |
| Store screenshot review | Complete `qa-session/store-screenshots.md`, review `STORE_SCREENSHOTS.md`, run `store_screenshot_check.sh --validate-dir screenshots`, and approve the final 1920x1080 screenshot set for store submission | Pending |  |  |  |
| Store asset audit | Review `store-asset-audit.txt` and confirm app icon, store capsule, store/press references, art notes, and screenshot-plan coverage are acceptable for store submission | Pending |  |  |  |
| Content rating review | Review `CONTENT_RATING.md` and `content-rating-audit.txt`, then confirm platform questionnaire answers for fantasy violence, data collection, online interaction, purchases, and sensitive content | Pending |  |  |  |
| Store and press review | Confirm `STORE_PAGE.md`, `PRESSKIT.md`, branding art, capsule art, credits, notices, and release notes are final | Pending |  |  |  |
| Marketing material audit | Review `marketing-audit.txt` and confirm store, press, release note, art, notice, and credit documents are structurally complete | Pending |  |  |  |
| IP and naming audit | Review `ip-audit.txt` and confirm release-facing names, labels, and manifest paths avoid source-material terms | Pending |  |  |  |
| Final art-direction review | Review art direction and mechanics for separation from similar commercial games | Pending |  |  |  |

## Final Decision

- Release approved: No
- Approver:
- Approval date:
- Build artifact:
- SHA256:
- Final notes:
