# Release Checklist

Bevy Open Siege is not yet production-ready, but this checklist defines the concrete gates for a release candidate.

## Required Before Release

- Verify runtime UI chrome sprites from `assets/art/ui/` render clearly in menu, HUD, pause, and end screens.
- Verify runtime projectile, sun pickup, fire, and explosion sprites from `assets/art/effects/` render clearly at gameplay camera distance.
- Verify runtime board environment textures from `assets/art/environment/` render clearly at gameplay camera distance.
- Verify Blender-generated runtime plant and monster models from `assets/models/` render clearly at gameplay camera distance.
- Verify plant and monster sprite/reference crops from `assets/art/sprites/` remain useful for store, press, and fallback review.
- Verify production branding assets in `assets/branding/generated/` and update store artwork exports if they are regenerated.
- Verify synthesized music and sound effects in `assets/audio/` at release volume on speakers and headphones, then decide whether `--audio` should become the default startup mode.
- Review `audio-smoke.txt`, then rerun the packaged `audio_smoke.sh` on the release QA machine and confirm `--audio` reaches the game window without panic signatures before speaker/headphone listening tests.
- Run `--audit-balance` and review `balance-audit.txt`, then balance the 10-level campaign in `assets/data/levels.ron` through repeated playtests.
- Run `--audit-assets` and review `asset-audit.txt` to confirm PNG dimensions, WAV durations, fallback SVGs, metadata assets, and production-art counts before visual/audio QA.
- Run `--audit-audio` and review `audio-audit.txt` to confirm WAV duration, clipping headroom, non-silence, mix loudness, and opt-in audio startup policy before speaker/headphone QA.
- Run `--audit-controls` and review `controls-audit.txt` to confirm documented keyboard/mouse bindings match README and bilingual menu help.
- Run `--audit-input-flow` and review `input-flow-audit.txt` to confirm deterministic menu navigation, locked-level gating, settings shortcuts, seed selection, planting blocks, and pause gating.
- Run `--audit-localization` and review `localization-audit.txt` to confirm English and Chinese UI strings, unit names, plant descriptions, and level titles have release coverage.
- Run `--audit-layout` and review `layout-audit.txt` to confirm generated menu, HUD, pause, and end-screen text stays inside release text bounds in English and Chinese.
- Run `--audit-visual` and review `visual-readability-audit.txt` to confirm common viewport budgets, text contrast, HUD wrapping, and representative visual assets meet release bounds.
- Run `--audit-accessibility` and review `accessibility-audit.txt` to confirm keyboard-only flow, no-audio playability, bilingual text bounds, contrast, and textual status cues before manual accessibility QA.
- Run `--audit-performance` and review `performance-audit.txt` to confirm spawn bursts, runtime spawn interval, estimated dynamic entities, embedded asset bytes, compact viewport floor, and audio startup policy stay within release budgets.
- Run `--audit-privacy` and review `privacy-audit.txt`, `PRIVACY.md`, `SUPPORT.md`, and `TROUBLESHOOTING.md` to confirm no telemetry/network posture, local-save disclosure, uninstall preservation, explicit `--purge` deletion, bug-report evidence guidance, and a player-facing troubleshooting path.
- Run `--audit-release-provenance` and review `release-provenance-audit.txt` and `BUILD_PROVENANCE.md` to confirm Cargo.lock use, package entry points, SHA256 integrity, third-party license generation, archive smoke verification, and final signoff rules.
- Run `--audit-marketing` and review `marketing-audit.txt` to confirm store copy, press kit, release notes, art notes, notices, credits, and media references are present before manual store review.
- Run `scripts/store_asset_audit.sh <extracted-package-dir>` and review `store-asset-audit.txt` to confirm app icon, store capsule, store/press references, art notes, and screenshot-plan coverage before manual store review.
- Run `scripts/content_rating_audit.sh <extracted-package-dir>` and review `content-rating-audit.txt` and `CONTENT_RATING.md` to confirm fantasy violence, sensitive-content, online/data, monetization, and platform questionnaire notes before manual rating review.
- Review `STORE_SCREENSHOTS.md`, run `store_screenshot_check.sh --plan .`, capture the required 1920x1080 screenshot set from the release package with `store_screenshot_check.sh --capture-pack . screenshots 10` or manual staged captures, validate it with `store_screenshot_check.sh --validate-dir screenshots`, and record final approval in `qa-session/store-screenshots.md`.
- Run `--audit-ip` and review `ip-audit.txt` to confirm release-facing docs, localization, control labels, and manifest paths avoid source-material names.
- Run `--audit-save` and review `save-audit.txt` to confirm save paths, legacy save parsing, score-slot normalization, default settings, and settings clamping are release-compatible.
- Run `--audit-playthrough` and review `playthrough-audit.txt` to confirm scripted victory, defeat, restart, unlock, and best-score lifecycle checks across all 10 levels.
- Run `--simulate-campaign` and review `campaign-simulation.txt` to confirm the unlock chain, plant affordability, and zombie rule coverage before manual playtests.
- Run `--release-readiness` and review `release-readiness.txt` to confirm automated evidence is complete and manual approvals are explicitly listed.
- Run `scripts/runtime_smoke.sh target/release/bevy_open_siege` and review `runtime-smoke.txt` to confirm the windowed game starts, creates the main window, and runs through the smoke duration without panic signatures.
- Run `scripts/visual_smoke.sh target/release/bevy_open_siege` and review `visual-smoke.txt` to confirm the game window can be captured and is not blank.
- Run `scripts/linux_package_audit.sh <extracted-package-dir>` and review `linux-package-audit.txt` to confirm user-local install, launcher, desktop metadata, uninstall, and purge behavior without touching the real user profile.
- Run `scripts/linux_install_smoke.sh <extracted-package-dir>` and review `linux-install-smoke.txt` to confirm the installed launcher opens a runtime and visual smoke window from temporary XDG user directories, then preserves saves on uninstall and removes them with `--purge`.
- Run `scripts/linux_dependency_audit.sh <extracted-package-dir>` and review `linux-dependency-audit.txt` to confirm no missing dynamic libraries and to explicitly review whether the Linux binary is tied to a build-environment runtime such as `/nix/store`.
- Run `scripts/linux_portability_smoke.sh <extracted-package-dir>` and review `linux-portability-smoke.txt` to confirm the packaged wrapper validates data and creates a window with `LD_LIBRARY_PATH` unset and Nix-specific environment variables omitted.
- Run `scripts/linux_clean_distro_smoke.sh <extracted-package-dir>` and review `linux-clean-distro-smoke.txt` to confirm package data loading and bundled dependency resolution pass inside a clean Ubuntu container with networking disabled.
- Run `scripts/linux_metadata_audit.sh <extracted-package-dir>` and review `linux-metadata-audit.txt` to confirm Linux `.desktop`, AppStream, icon, category, version, and launch metadata before storefront submission.
- Run `scripts/manual_qa_session.sh --plan <extracted-package-dir>` and review `manual-qa-plan.txt` before starting manual QA so the final pass uses an isolated save and consistent evidence filenames.
- Run `scripts/manual_qa_session.sh --init <extracted-package-dir> <session-dir>` to create manual QA evidence templates before the final pass.
- Run `scripts/manual_qa_observations.sh --collect <extracted-package-dir> <session-dir> [screenshots-dir]` to pre-fill manual QA evidence files with automated observations while preserving Pending status for human signoff.
- Run `scripts/platform_package_plan.sh --plan <extracted-package-dir>` and review `platform-package-plan.txt` before Windows/macOS package work.
- Run `scripts/platform_package_plan.sh --init <extracted-package-dir> <session-dir>` to create Windows/macOS package QA templates before cross-platform passes.
- Run `scripts/final_signoff_check.sh --plan <extracted-package-dir>` and review `final-signoff-plan.txt` before final approval.
- Run packaged `verify_release.sh --quick .` to verify SHA256 integrity, deterministic audit reports, and QA plan generation from the extracted package.
- Run packaged `support_diagnostics.sh . support-diagnostics` and confirm it creates release metadata, integrity, save-path, privacy-audit, quick verification, and manifest outputs without copying save files or personal files.
- Run packaged `create_candidate_evidence.sh . candidate-evidence` for release-candidate handoff and confirm it creates initialized QA templates, platform templates, support diagnostics, and a candidate signoff bundle without marking final approval.
- Run packaged `create_store_submission_pack.sh . store-submission-pack [screenshots]` for store handoff and confirm it includes store copy, press materials, branding assets, content-rating notes, store reports, and validated screenshots when a screenshot directory is provided.
- Review packaged `release-manifest.json` to confirm product, version, platform, binary path, required evidence files, and final approval gate are machine-readable for QA/support handoff.
- Run `scripts/qa_evidence_summary.sh --summary <extracted-package-dir> <qa-session-dir> <platform-session-dir>` to preflight manual and platform evidence status before final approval.
- Run `scripts/qa_signoff_prepare.sh --check <extracted-package-dir> <qa-session-dir> <platform-session-dir>` after human QA evidence is completed to verify all required evidence is approved and has no unresolved follow-up.
- Run `scripts/qa_signoff_prepare.sh --write <extracted-package-dir> <qa-session-dir> <platform-session-dir> "<reviewer>" <YYYY-MM-DD> <output-QA_SIGNOFF.md> [archive-sha256]` to prepare the final signoff file from completed evidence; if the archive hash is omitted, the output must point to packaged `SHA256SUMS` for per-file hashes.
- Run `scripts/signoff_bundle.sh --plan <extracted-package-dir>` and review the final evidence bundle contents before final approval.
- After manual/platform QA evidence is complete, run `scripts/final_signoff_check.sh --check <extracted-package-dir> <qa-session-dir> <platform-session-dir>` before marking release approved.
- After final signoff passes, run `scripts/signoff_bundle.sh --create <extracted-package-dir> <qa-session-dir> <platform-session-dir> <output-dir>` and archive the generated signoff evidence tarball.
- On Windows, run `powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1` and attach the resulting `dist/bevy_open_siege-0.1.0-windows-x86_64.zip` to Windows package QA.
- On macOS, run `scripts/package_macos.sh` and attach the resulting `dist/bevy_open_siege-0.1.0-macos-universal.tar.gz` to macOS package QA.
- Complete every `QA_SIGNOFF.md` table row after manual playthrough, visual, audio, platform, store, and IP review; each row must be `Pass` or `Scoped Out` with Owner and YYYY-MM-DD Date filled, and scoped-out rows must include Notes.
- Verify the title/settings/level-select menu on release hardware after reviewing `visual-readability-audit.txt`.
- Perform keyboard and mouse usability testing on every gameplay action.
- Verify save compatibility from older working-directory `bevy_open_siege_save.ron` files.
- Verify `--print-save-summary` reads old working-directory saves, applies default settings, and normalizes best-score slots to the current campaign length.
- Verify `--print-save-path` points at the user data directory, or at `BEVY_OPEN_SIEGE_SAVE_PATH` when overridden for QA.
- Verify the release archive includes `save-audit.txt` and that it matches the packaged binary output.
- Package and smoke-test Linux, Windows, and macOS builds.
- Verify the release archive includes README, license, credits, art asset notes, third-party notices, store copy, version metadata, gameplay data, and localization data.
- Verify the release archive includes generated Cargo dependency license report `THIRD_PARTY_LICENSES.md`.
- Verify the release archive includes `RELEASE_NOTES.md` and that the notes match the packaged version and platform.
- Verify the release archive includes `asset-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `audio-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `controls-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `input-flow-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `localization-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `layout-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `visual-readability-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `accessibility-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `performance-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `privacy-audit.txt`, `PRIVACY.md`, `SUPPORT.md`, and `TROUBLESHOOTING.md`, and that `privacy-audit.txt` matches the packaged binary output.
- Verify the release archive includes `release-provenance-audit.txt` and `BUILD_PROVENANCE.md`, and that `release-provenance-audit.txt` matches the packaged binary output.
- Verify the release archive includes `marketing-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `STORE_SCREENSHOTS.md` and manual QA creates `qa-session/store-screenshots.md`.
- Verify the release archive includes `store-asset-audit.txt` and executable `store_asset_audit.sh`.
- Verify the release archive includes `CONTENT_RATING.md`, `content-rating-audit.txt`, and executable `content_rating_audit.sh`.
- Verify the release archive includes `linux-install-smoke.txt` and executable `linux_install_smoke.sh`.
- Verify the release archive includes `linux-dependency-audit.txt` and executable `linux_dependency_audit.sh`.
- Verify the release archive includes `linux-portability-smoke.txt` and executable `linux_portability_smoke.sh`.
- Verify the release archive includes `linux-clean-distro-smoke.txt` and executable `linux_clean_distro_smoke.sh`.
- Verify the release archive includes `ip-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `save-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `playthrough-audit.txt` and that it matches the packaged binary output.
- Verify the release archive includes `campaign-simulation.txt` and that it matches the packaged binary output.
- Verify the release archive includes `release-readiness.txt` and that it matches the packaged binary output.
- Verify the release archive includes `runtime-smoke.txt` and that it matches the packaged binary output.
- Verify the release archive includes `visual-smoke.txt` and that it matches the packaged visual smoke output.
- Verify the release archive includes `audio-smoke.txt` and that it matches the packaged audio smoke output.
- Verify the release archive includes `audio_smoke.sh`, `runtime_smoke.sh`, and `visual_smoke.sh` as executable QA helpers.
- Verify the release archive includes executable `store_screenshot_check.sh` for storefront screenshot capture and validation.
- Verify the release archive includes `linux-package-audit.txt` and executable `linux_package_audit.sh`.
- Verify the release archive includes `linux-metadata-audit.txt` and executable `linux_metadata_audit.sh`.
- Verify the release archive includes `manual-qa-plan.txt` and executable `manual_qa_session.sh`.
- Verify the release archive includes executable `manual_qa_observations.sh`.
- Verify the release archive includes `platform-package-plan.txt` and executable `platform_package_plan.sh`.
- Verify the release archive includes executable `qa_evidence_summary.sh`.
- Verify the release archive includes executable `verify_release.sh`.
- Verify the release archive includes executable `support_diagnostics.sh`.
- Verify the release archive includes executable `signoff_bundle.sh`.
- Verify the release archive includes executable `create_candidate_evidence.sh`.
- Verify the release archive includes executable `create_store_submission_pack.sh`.
- Verify the release archive includes `final-signoff-plan.txt` and executable `final_signoff_check.sh`.
- Verify the release archive includes Windows/macOS package entry points `package_windows.ps1` and `package_macos.sh`.
- Verify the release archive includes `QA_SIGNOFF.md` with every manual approval area listed.
- Verify the release archive includes PNG production art, WAV audio, fallback SVG branding assets, and press kit materials.
- Verify the release archive includes Linux `.desktop` and AppStream metadata for downstream package managers.
- Verify AppStream release metadata version matches `VERSION.ron` and `Cargo.toml`.
- Verify the release archive includes `SHA256SUMS` and that `sha256sum -c SHA256SUMS` passes.
- Verify the release archive includes `release-manifest.json` and that `verify_release.sh --quick .` reports `release manifest: present`.
- Verify `install_linux_user.sh` installs to user-local XDG directories, `uninstall_linux_user.sh` removes app files while preserving saves, and `uninstall_linux_user.sh --purge` removes saves when explicitly requested.
- Review naming, art direction, and mechanics for IP separation from similar commercial games.

## Automated Gate

Run before every release candidate:

```bash
./scripts/release_check.sh
```

This currently checks formatting, type checking, unit tests, clippy warnings, release data validity, embedded PNG dimensions, embedded WAV properties, audio mix safety, campaign balance bounds, embedded asset inventory, documented input bindings, deterministic input flow, bilingual localization coverage, generated UI text layout bounds, visual readability bounds, accessibility coverage, performance budgets, privacy/support disclosure, build provenance, store and press material structure, store screenshot checklist coverage, IP/naming separation, save compatibility, scripted playthrough lifecycle, headless campaign progression, release-candidate readiness status, and release script syntax.
It runs the game binary in `--validate-data`, `--audit-balance`, `--audit-assets`, `--audit-audio`, `--audit-controls`, `--audit-input-flow`, `--audit-localization`, `--audit-layout`, `--audit-visual`, `--audit-accessibility`, `--audit-performance`, `--audit-privacy`, `--audit-release-provenance`, `--audit-marketing`, `--audit-ip`, `--audit-save`, `--audit-playthrough`, `--simulate-campaign`, and `--release-readiness` modes so release data can be checked without opening a window.
The default path reuses the local Cargo target directory for fast iteration. For a cleaner release-candidate pass, run:

```bash
BEVY_OPEN_SIEGE_ISOLATED_TARGET=1 ./scripts/release_check.sh
```

To run the same gate through the Nix dev shell:

```bash
BEVY_OPEN_SIEGE_USE_NIX=1 ./scripts/release_check.sh
```

## Manual Smoke Test

- Start the game from a clean directory.
- Switch language with `L` on the menu and in-game.
- Cycle levels with `Tab` or arrow keys; use `1-0` to jump to a level; confirm locked levels cannot start.
- Win level 1 and confirm level 2 unlocks after restart.
- Confirm the user data save records language, best score, unlocked levels, volume, and fullscreen.
- Plant every seed type once and confirm cooldowns appear in the HUD.
- Pause with `P`; confirm waves, projectiles, cooldowns, and sun collection freeze.
- Toggle fullscreen with `F` and adjust volume with `+` / `-`.
- Confirm defeat and victory screens can restart with `R` or `Enter`.
- Run the packaged binary with `--validate-data`, `--print-release-info`, and `--print-save-path`.
- Run the packaged binary with `--audit-balance` and confirm the output matches `balance-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-assets` and confirm the output matches `asset-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-audio` and confirm the output matches `audio-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically. Speaker/headphone listening review remains manual.
- Run the packaged binary with `--audit-controls` and confirm the output matches `controls-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-input-flow` and confirm the output matches `input-flow-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-localization` and confirm the output matches `localization-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-layout` and confirm the output matches `layout-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-visual` and confirm the output matches `visual-readability-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-accessibility` and confirm the output matches `accessibility-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-performance` and confirm the output matches `performance-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-privacy` and confirm the output matches `privacy-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-release-provenance` and confirm the output matches `release-provenance-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-marketing` and confirm the output matches `marketing-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-ip` and confirm the output matches `ip-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-save` and confirm the output matches `save-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--audit-playthrough` and confirm the output matches `playthrough-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--simulate-campaign` and confirm the output matches `campaign-simulation.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--release-readiness` and confirm the output matches `release-readiness.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged binary with `--print-release-info` and confirm the output matches `release-info.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `runtime_smoke.sh` and confirm the output matches `runtime-smoke.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `visual_smoke.sh` and confirm the output matches `visual-smoke.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `audio_smoke.sh` and confirm the output matches `audio-smoke.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically. Speaker/headphone listening review remains manual.
- Run the packaged `store_asset_audit.sh` and confirm the output matches `store-asset-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `content_rating_audit.sh` and confirm the output matches `content-rating-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `linux_package_audit.sh` and confirm the output matches `linux-package-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `linux_portability_smoke.sh` and confirm the output matches `linux-portability-smoke.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically. Clean non-Nix Linux QA remains manual.
- Run the packaged `linux_clean_distro_smoke.sh` and confirm the output matches `linux-clean-distro-smoke.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically. Release-hardware window, GPU, audio, install, and input QA remain manual.
- Run the packaged `linux_metadata_audit.sh` and confirm the output matches `linux-metadata-audit.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `manual_qa_session.sh --plan .` and confirm the output matches `manual-qa-plan.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `manual_qa_session.sh --init . qa-session` and confirm the manual evidence templates are created; `scripts/smoke_release_archive.sh` performs this check automatically with an isolated temporary directory.
- Run the packaged `platform_package_plan.sh --plan .` and confirm the output matches `platform-package-plan.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- Run the packaged `platform_package_plan.sh --init . platform-session` and confirm the Windows/macOS package QA templates are created; `scripts/smoke_release_archive.sh` performs this check automatically with an isolated temporary directory.
- Run the packaged `qa_signoff_prepare.sh --check . qa-session platform-session` and confirm it rejects pending evidence; `scripts/smoke_release_archive.sh` also verifies in an isolated temporary fixture that completed Linux v1 evidence plus scoped-out Windows/macOS evidence can produce a passing final signoff.
- Run the packaged `final_signoff_check.sh --plan .` and confirm the output matches `final-signoff-plan.txt`; `scripts/smoke_release_archive.sh` performs this comparison automatically.
- On the relevant host OS, run `scripts/package_windows.ps1` or `scripts/package_macos.sh` and attach the produced platform archive to the matching QA template.
- Run `scripts/smoke_release_archive.sh` against the final tarball.
