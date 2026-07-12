# Bevy Open Siege

A Bevy lane-defense strategy game: place greenhouse defenders, manage sun income, and survive escalating undead waves.

## Controls

- `Enter`: start or restart
- `Arrow keys`: move the grid cursor
- `Mouse left`: move cursor to a lawn tile and plant the selected seed
- `Mouse right`: move cursor to a lawn tile and shovel that tile
- `Tab`: cycle the selected level on the menu
- `1`: select Sprout Slinger, steady single-lane shots
- `2`: select Sunbloom, creates sun
- `3`: select Bark Bulwark, high-health blocker
- `4`: select Frost Sprout, slows enemies
- `5`: select Twin Pod, rapid double damage
- `6`: select Leaf Lobber, heavy splash shots
- `7`: select Briar Mat, damages walkers on its tile
- `8`: select Blast Berry, delayed lane explosion
- `9`: select Ember Stump, burns nearby enemies
- `0`: select Scent Root, redirects attackers to another lane
- `Space`: plant the selected seed
- `Backspace`: shovel the selected tile
- `C`: collect sun on the selected tile
- `L`: switch language between English and Chinese
- `P`: pause or resume in game
- `F`: toggle fullscreen
- `+` / `-`: adjust saved master volume
- `R`: retry after victory or defeat

## Gameplay

- Ten plant types cover economy, blockers, single-target fire, slows, burst damage, ground traps, lane control, and close-range fire.
- Planting uses per-seed cooldowns, shown directly in the seed bank.
- Ten undead enemy types escalate across waves: walkers, cone guards, runners, bucket guards, brutes, healers, jumpers, diggers, frostbiters, and giants.
- Enemies stop to attack plants, but some can jump, dig past early defenses, heal, freeze plant timers, or soak extra damage.
- Enemies breach the greenhouse if they cross the left edge.
- Survive the final wave to win.

## Localization

- English and Chinese are supported in menu, HUD, plant names, plant descriptions, and end screens.
- Press `L` at any time to switch language.
- Localization data lives in `assets/i18n/en.ron` and `assets/i18n/zh.ron`.

## Levels And Progress

- The campaign currently contains 10 data-driven levels.
- Level tuning data lives in `assets/data/levels.ron`.
- The menu can preview every level with `Tab`, arrow keys, or `1-0`; locked levels cannot be started until unlocked.
- Winning a level records the best score and unlocks the next level.
- Progress, language, volume, and fullscreen settings are saved to the user data directory. On Linux this defaults to `$XDG_DATA_HOME/bevy_open_siege/bevy_open_siege_save.ron` or `~/.local/share/bevy_open_siege/bevy_open_siege_save.ron`.
- Older saves named `bevy_open_siege_save.ron` in the working directory are still read for compatibility if the user data save does not exist.
- Set `BEVY_OPEN_SIEGE_SAVE_PATH=/path/to/bevy_open_siege_save.ron` to override the save location for QA or portable builds.

## Pause And Settings

- Press `P` in game to freeze waves, combat, projectiles, cooldowns, and sun collection.
- While paused, the settings panel shows fullscreen and volume controls.

## Asset Direction

The current build ships Blender-generated 3D GLB models for runtime plants and monsters, plus production PNG art for projectiles, pickups, board textures, UI chrome, app icon, store capsule, and reference sheets. The remaining art pass is focused on release-hardware visual QA and any final polish requested after playtesting.

## Verification

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --validate-data
cargo run -- --audit-balance
cargo run -- --audit-assets
cargo run -- --audit-audio
cargo run -- --audit-controls
cargo run -- --audit-input-flow
cargo run -- --audit-localization
cargo run -- --audit-layout
cargo run -- --audit-visual
cargo run -- --audit-accessibility
cargo run -- --audit-performance
cargo run -- --audit-marketing
cargo run -- --audit-ip
cargo run -- --audit-save
cargo run -- --audit-playthrough
cargo run -- --simulate-campaign
cargo run -- --release-readiness
```

Or run the bundled release gate:

```bash
./scripts/release_check.sh
```

The release gate reuses the local Cargo target directory by default. Set `BEVY_OPEN_SIEGE_USE_NIX=1` to run it through the Nix dev shell, or `BEVY_OPEN_SIEGE_ISOLATED_TARGET=1` to force an isolated target directory for clean release-candidate checks.

## Packaging

Create a Linux x86_64 release archive:

```bash
./scripts/package_release.sh
```

Set `BEVY_OPEN_SIEGE_USE_NIX=1` to build the package through the Nix dev shell.

The package includes the binary, gameplay data, localization data, Blender-generated 3D GLB unit models, production PNG art, fallback SVG branding, press kit, README, privacy notice, content rating notes, support guide, troubleshooting runbook, support diagnostics helper, signoff evidence bundle helper, candidate evidence helper, store submission pack helper, machine-readable release manifest, build provenance notes, license, credits, art asset notes, third-party notices, generated Cargo dependency license report, store-page copy, store screenshot checklist, version metadata, release checklist, audio mix audit report, privacy/support audit report, release provenance audit report, store asset audit report, content rating audit report, Linux package audit report, Linux installed-launcher smoke report, Linux dependency audit report, Linux sanitized-environment portability smoke report, Linux clean-distro container smoke report, Linux desktop metadata audit report, manual QA session plan, manual QA observations helper, cross-platform package plan, QA evidence summary helper, package verification helper, final signoff plan, Windows/macOS package scripts, runtime, visual, and audio startup smoke reports, and manual QA signoff template.

The package script runs the release binary with `--validate-data`, `--audit-balance`, `--audit-assets`, `--audit-audio`, `--audit-controls`, `--audit-input-flow`, `--audit-localization`, `--audit-layout`, `--audit-visual`, `--audit-accessibility`, `--audit-performance`, `--audit-privacy`, `--audit-release-provenance`, `--audit-marketing`, `--audit-ip`, `--audit-save`, `--audit-playthrough`, `--simulate-campaign`, and `--release-readiness`, then runs `scripts/runtime_smoke.sh`, `scripts/visual_smoke.sh`, `scripts/audio_smoke.sh`, `scripts/store_asset_audit.sh`, `scripts/content_rating_audit.sh`, `scripts/linux_package_audit.sh`, `scripts/linux_install_smoke.sh`, `scripts/linux_dependency_audit.sh`, `scripts/linux_portability_smoke.sh`, `scripts/linux_clean_distro_smoke.sh`, and `scripts/linux_metadata_audit.sh` against the release package before creating the archive. It writes `release-manifest.json` and `SHA256SUMS` for package inventory and integrity, includes visual/audio/runtime/Linux package smoke helpers for QA, then runs `scripts/smoke_release_archive.sh` against the generated tarball.

After extracting the archive, `./verify_release.sh --quick .` verifies SHA256 integrity, `release-manifest.json`, release metadata, deterministic audit reports, Linux portability evidence, and generated QA plans without opening a window. Use `./verify_release.sh --full .` on a QA machine with display/audio support to include runtime, visual, audio, and sanitized-environment Linux portability startup smoke checks.

The archive also includes Linux desktop integration metadata under `assets/linux/`: a `.desktop` launcher and AppStream metainfo XML for downstream package managers.

After extracting the archive, Linux users can run `./install_linux_user.sh` for a user-local install. It installs the app under the user data directory, creates a `bevy_open_siege` launcher in `~/.local/bin` or `$XDG_BIN_HOME`, and installs desktop/AppStream metadata. `./uninstall_linux_user.sh` removes the app and launcher while preserving save data; use `./uninstall_linux_user.sh --purge` to remove saves too.

`PRIVACY.md` documents the offline/no-telemetry posture, local save fields, default save path, `BEVY_OPEN_SIEGE_SAVE_PATH` override, and explicit `--purge` deletion behavior. `SUPPORT.md` documents the issue-report evidence to collect without attaching unrelated personal files. `TROUBLESHOOTING.md` gives players, support, and QA a command-oriented runbook for startup, rendering, audio, save, package, and final evidence problems. `privacy-audit.txt` is generated from `--audit-privacy` and rechecked by archive smoke tests.

`./support_diagnostics.sh . support-diagnostics` creates a local diagnostics folder with release metadata, package integrity output, save path/summary output, privacy audit output, quick package verification output, and the release manifest. It does not copy save files, screenshots, recordings, crash dumps, or unrelated personal files.

`BUILD_PROVENANCE.md` documents release source inputs, package entry points, integrity evidence, dependency license evidence, and final approval rules. `release-provenance-audit.txt` is generated from `--audit-release-provenance` and rechecked by archive smoke tests.

`./linux_package_audit.sh .` verifies the extracted package's user-local install path, launcher wrapper, desktop/AppStream/icon metadata, save-preserving uninstall, and explicit purge behavior. The packaged `linux-package-audit.txt` is generated from this script and rechecked by archive smoke tests.

`./linux_metadata_audit.sh .` verifies Linux storefront and desktop integration metadata: `.desktop` identity, categories, keywords, AppStream ID, launchable, binary, icon PNG, version, localization claims, and optional `desktop-file-validate`/AppStream validator output when those host tools are installed. The packaged `linux-metadata-audit.txt` is generated from this script and rechecked by archive smoke tests.

`./linux_install_smoke.sh .` installs the package into temporary XDG user directories, starts the installed launcher through runtime and visual smoke tests, verifies uninstall preserves save data, and verifies `--purge` removes save data. The packaged `linux-install-smoke.txt` records this installed-launcher evidence for Linux package QA.

`./linux_dependency_audit.sh .` checks the packaged ELF binary, dynamic linker, linked sonames, and `ldd` missing-library status, then records whether build-environment paths such as `/nix/store` are embedded. The packaged `linux-dependency-audit.txt` must be reviewed before final Linux storefront approval.

`./linux_portability_smoke.sh .` starts the bundled Linux package with a sanitized environment, no `LD_LIBRARY_PATH`, and Nix-specific variables omitted. The packaged `linux-portability-smoke.txt` records this evidence, but a clean non-Nix Linux QA machine is still required before final storefront approval.

`./linux_clean_distro_smoke.sh .` runs the package inside a clean Ubuntu container with networking disabled, validates data loading, and checks bundled dependency resolution without host or Nix library search paths. The packaged `linux-clean-distro-smoke.txt` records this evidence; window, GPU, audio, install, and input review still require release hardware.

`./manual_qa_session.sh --plan .` prints the packaged manual QA session plan. `./manual_qa_session.sh --init . qa-session` creates the manual evidence templates listed in the plan, with isolated-save commands and required evidence filenames for the final signoff pass.

`./manual_qa_observations.sh --collect . qa-session screenshots` initializes the manual QA session if needed and appends automated precheck observations to each QA evidence file. It also writes `qa-session/automated-observations.md`, `verify-release-quick.txt`, and `qa-evidence-summary.txt`. It intentionally keeps each evidence file Pending and unapproved until a human reviewer completes the manual checks.

`STORE_SCREENSHOTS.md` defines the required storefront screenshot set. The package includes `store_screenshot_check.sh` to print the capture plan, capture the current game window, capture deterministic store scenes with `--capture-pack`, capture the startup title menu, and validate the final `screenshots/` directory. The manual QA session creates `qa-session/store-screenshots.md` so the final store captures, dimensions, language coverage, and composition review are recorded before signoff.

`./store_asset_audit.sh .` verifies the packaged app icon, store capsule, store page references, press kit references, art notes, and screenshot capture checklist before store submission. The packaged `store-asset-audit.txt` is generated from this script and rechecked by archive smoke tests.

`CONTENT_RATING.md` records storefront rating questionnaire notes for fantasy non-realistic combat, no blood/gore, no purchases/ads/gambling, no online interaction, no data collection, and sensitive-content exclusions. `./content_rating_audit.sh .` verifies those notes against store, press, privacy, and QA signoff materials, then writes `content-rating-audit.txt` for final review.

`./platform_package_plan.sh --plan .` prints the packaged Windows/macOS package plan. `./platform_package_plan.sh --init . platform-session` creates platform QA templates for Windows and macOS package evidence.

`./final_signoff_check.sh --plan .` prints the packaged final signoff plan. After manual QA and platform QA are complete, `./final_signoff_check.sh --check . qa-session platform-session` verifies that all manual evidence files are approved, all `QA_SIGNOFF.md` table rows are completed with owner/date metadata, and `QA_SIGNOFF.md` is marked approved.

`./qa_evidence_summary.sh --summary . qa-session platform-session` prints the current automated, manual, and platform evidence status before final signoff. It is a preflight summary only; `final_signoff_check.sh --check` remains the release approval gate.

`./qa_signoff_prepare.sh --check . qa-session platform-session` verifies that every required manual and platform evidence file is `Pass` or `Scoped Out`, `Approved: Yes`, and has no unresolved follow-up. After a human reviewer has completed that evidence, `./qa_signoff_prepare.sh --write . qa-session platform-session "Reviewer Name" YYYY-MM-DD QA_SIGNOFF.md [archive-sha256]` prepares a completed `QA_SIGNOFF.md` from the packaged template. Omitting `archive-sha256` records that per-file hashes are in packaged `SHA256SUMS`; it does not treat `SHA256SUMS` itself as the release archive hash.

`./create_candidate_evidence.sh . candidate-evidence` initializes the manual QA templates, platform QA templates, support diagnostics, and a candidate signoff bundle for handoff. It preserves Pending status and does not mark the build final approved.

`./create_store_submission_pack.sh . store-submission-pack [screenshots]` creates a store handoff folder with store copy, press materials, branding PNGs, art sheets, content-rating notes, and audit reports. If a screenshot directory is supplied, it validates the final 16:9 screenshot set before copying it; otherwise the pack is marked screenshot pending.

`./signoff_bundle.sh --plan .` prints the final evidence archive plan. Use `./signoff_bundle.sh --create --allow-candidate . qa-session platform-session signoff-bundle` for release-candidate evidence dry runs. Use `./signoff_bundle.sh --create . qa-session platform-session signoff-bundle` only after final evidence is complete; it requires `final_signoff_check.sh --check` to pass before writing the archive.

On a Windows build host, run `powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1` to create `dist/bevy_open_siege-0.1.0-windows-x86_64.zip`. On a macOS build host, run `scripts/package_macos.sh` to create `dist/bevy_open_siege-0.1.0-macos-universal.tar.gz`.

## CLI Utilities

```bash
cargo run -- --validate-data
cargo run -- --audit-balance
cargo run -- --audit-assets
cargo run -- --audit-audio
cargo run -- --audit-controls
cargo run -- --audit-input-flow
cargo run -- --audit-localization
cargo run -- --audit-layout
cargo run -- --audit-visual
cargo run -- --audit-accessibility
cargo run -- --audit-performance
cargo run -- --audit-privacy
cargo run -- --audit-release-provenance
cargo run -- --audit-marketing
cargo run -- --audit-ip
cargo run -- --audit-save
cargo run -- --audit-playthrough
cargo run -- --simulate-campaign
cargo run -- --release-readiness
cargo run -- --print-release-info
cargo run -- --print-save-path
cargo run -- --print-save-summary
cargo run -- --audio
```

`--simulate-campaign` is a headless release QA pass. It verifies the 10-level unlock chain, expected sun access to all 10 plants, and coverage of all 10 zombie rules without opening a game window.

`--audit-assets` writes a deterministic manifest-derived report of embedded PNG dimensions, WAV duration/format, fallback SVG assets, metadata files, and total embedded asset bytes for release review.

`--audit-audio` verifies embedded WAV music and sound effects for PCM format, duration bounds, non-silence, clipping headroom, mix loudness ranges, and default opt-in startup policy, then writes a deterministic audio-mix report for QA. Speaker/headphone listening review remains manual.

`--audit-controls` verifies that the documented keyboard and mouse bindings are covered by README controls and English/Chinese menu help, then writes a deterministic control-map report for QA.

`--audit-input-flow` verifies deterministic menu navigation, locked-level start gating, global settings shortcuts, cursor clamping, all 10 seed-selection keys, planting blocks, pause gating, and end-flow coverage, then writes a deterministic input-flow report for QA.

`--audit-localization` verifies English and Chinese UI strings, HUD labels, unit names, plant descriptions, and level titles for release coverage, then writes a deterministic localization report for QA.

`--audit-layout` verifies generated menu, HUD, seed-bank, pause, and end-screen text against release text bounds in both supported languages, then writes a deterministic layout-readability report for QA.

`--audit-visual` verifies common desktop and handheld viewport budgets, HUD wrapping, text contrast, and representative visual asset dimensions, then writes a deterministic visual-readability report for QA.

`--audit-accessibility` verifies keyboard-only flow coverage, keyboard alternatives for mouse actions, no-audio playability, bilingual text bounds, text contrast, and textual status cues, then writes a deterministic accessibility report for QA. Assistive-technology, remapping, and photosensitivity review remain manual.

`--audit-performance` verifies deterministic entity-count budgets, spawn burst bounds, minimum runtime spawn interval, embedded asset byte budget, compact viewport floor, and audio startup policy, then writes a deterministic performance-budget report for QA. Release-hardware profiling and long-session soak remain manual.

`--audit-privacy` verifies `PRIVACY.md`, `SUPPORT.md`, release docs, local-save disclosure, no-network/no-telemetry posture, and uninstall/purge data deletion language, then writes a deterministic privacy/support report for QA. Store/platform disclosure review remains manual.

`--audit-release-provenance` verifies `BUILD_PROVENANCE.md`, Cargo version lock evidence, release script entry points, SHA256 integrity checks, third-party license generation, package smoke verification, and final signoff rules, then writes a deterministic release-provenance report for QA.

`--audit-marketing` verifies store-page, press-kit, release-note, art-note, notice, and credit documents contain required sections, release claims, and media references, then writes a deterministic marketing-material report for QA.

`--audit-ip` verifies release-facing docs, localization, control labels, player-facing unit labels, and asset manifest paths avoid source-material names and old plant sprite filenames, then writes a deterministic naming-separation report for QA.

`--audit-save` verifies save-path resolution, legacy save parsing, score-slot normalization, default settings, and settings clamping, then writes a deterministic save-compatibility report for QA.

`--audit-playthrough` verifies a scripted clean-save campaign lifecycle across all 10 levels, including victory unlock progression, best-score recording, defeat non-advancement, and restart-state reset behavior.

`--release-readiness` summarizes automated release-candidate evidence and explicitly lists the remaining manual approvals before final release.

`scripts/runtime_smoke.sh target/release/bevy_open_siege` starts the real windowed game with audio disabled, waits for a fixed duration, verifies that the window is created, and fails if panic signatures such as Bevy query conflicts appear in the log.

`scripts/visual_smoke.sh target/release/bevy_open_siege` starts the real windowed game with audio disabled, captures the game window on X11 with ImageMagick or the active Wayland output with `grim`, and fails if the screenshot is missing, blank, or paired with panic signatures. This is an automated black-screen guard; final visual review still requires release hardware.

`scripts/audio_smoke.sh target/release/bevy_open_siege` starts the real windowed game with `--audio`, waits for a fixed duration, verifies that the window is created, and fails if panic signatures appear in the log. The audio backend initializes on a background thread so a slow or unavailable output device cannot block the window startup path. Final speaker/headphone listening review is still manual.

`scripts/linux_package_audit.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies install, launcher, desktop metadata, uninstall, and purge behavior for an extracted Linux package without touching the real user profile.

`scripts/linux_metadata_audit.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies Linux `.desktop`, AppStream, icon, version, category, and launch metadata before storefront submission.

`scripts/linux_install_smoke.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies an installed Linux launcher can start and render from temporary XDG user directories before manual clean-profile package QA.

`scripts/linux_dependency_audit.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies runtime dependency resolution and highlights non-portable build-environment runtime paths for Linux package QA.

`scripts/linux_portability_smoke.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies the packaged Linux wrapper can validate data and create a game window under a sanitized environment without `LD_LIBRARY_PATH` or Nix-specific variables.

`scripts/linux_clean_distro_smoke.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies packaged data loading and bundled dependency resolution inside a clean Ubuntu container with networking disabled.

`scripts/manual_qa_observations.sh --collect dist/bevy_open_siege-0.1.0-linux-x86_64 qa-session dist/store-screenshots` pre-fills manual QA evidence files with automated observations while preserving Pending status for human signoff.

`scripts/store_asset_audit.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies packaged store artwork dimensions, document references, and screenshot-plan coverage before final store review.

`scripts/content_rating_audit.sh dist/bevy_open_siege-0.1.0-linux-x86_64` verifies content rating notes, sensitive-content disclosures, monetization disclosures, online/data disclosures, and store/press references before platform questionnaire review.

`scripts/manual_qa_session.sh --plan dist/bevy_open_siege-0.1.0-linux-x86_64` emits the deterministic manual QA session plan that is packaged as `manual-qa-plan.txt`. Use `--init <package-dir> <session-dir>` to create the manual QA evidence templates.

`scripts/platform_package_plan.sh --plan dist/bevy_open_siege-0.1.0-linux-x86_64` emits the deterministic Windows/macOS package plan that is packaged as `platform-package-plan.txt`. Use `--init <package-dir> <session-dir>` to create platform package QA templates.

`scripts/final_signoff_check.sh --plan dist/bevy_open_siege-0.1.0-linux-x86_64` emits the deterministic final signoff plan that is packaged as `final-signoff-plan.txt`. Use `--check <package-dir> <qa-session-dir> <platform-session-dir>` after manual evidence is complete to block final approval when any required QA file is pending, unapproved, has unresolved follow-up, or when any `QA_SIGNOFF.md` row is still pending, missing owner/date metadata, or scoped out without notes.

`scripts/package_windows.ps1` and `scripts/package_macos.sh` are the host-specific package entry points for Windows x86_64 and macOS universal builds. They build the release binary for the target platform, generate deterministic audit reports, copy the same release documents and assets as the Linux package, write `SHA256SUMS`, and produce the platform archive for QA signoff.

Audio is opt-in while device compatibility is being hardened. The default startup path avoids initializing the platform audio backend; use `--audio` or `BEVY_OPEN_SIEGE_AUDIO=1` to enable the background loop and sound effects.

## Branding And Store Materials

- Asset manifest: `assets/manifest.ron`
- Production app icon: `assets/branding/generated/app-icon.png`
- Production store capsule: `assets/branding/generated/store-capsule.png`
- Plant character sheet: `assets/art/plants-sheet.png`
- Monster character sheet: `assets/art/monsters-sheet.png`
- Runtime plant models: `assets/models/plants/`
- Runtime monster models: `assets/models/monsters/`
- Plant sprite/reference crops: `assets/art/sprites/plants/`
- Monster sprite/reference crops: `assets/art/sprites/monsters/`
- Runtime projectile and pickup sprites: `assets/art/effects/`
- Runtime board environment textures: `assets/art/environment/`
- Runtime UI chrome panels: `assets/art/ui/`
- Music and sound effects: `assets/audio/`
- Linux desktop and AppStream metadata: `assets/linux/`
- Fallback app icon SVG: `assets/branding/icon.svg`
- Fallback store capsule SVG: `assets/branding/capsule.svg`
- Press kit: `PRESSKIT.md`
- Store page copy: `STORE_PAGE.md`
- Release notes: `RELEASE_NOTES.md`
- Manual QA signoff template: `QA_SIGNOFF.md`
- Credits: `CREDITS.md`
- Generated dependency license report in release archives: `THIRD_PARTY_LICENSES.md`
- Art asset notes: `ART_ASSETS.md`
