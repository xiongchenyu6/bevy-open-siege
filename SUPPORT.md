# Bevy Open Siege Support

## Supported Release

This package is a Bevy Open Siege 0.1.0 release candidate for Linux x86_64. Windows and macOS package scripts are included for platform QA, but those builds require separate smoke-test approval before release.

## Before Reporting An Issue

Please include the package version from `./bevy_open_siege --print-release-info` and the platform where the issue occurred.

Useful local checks:

- `./bevy_open_siege --validate-data`
- `./bevy_open_siege --audit-save`
- `./runtime_smoke.sh ./bevy_open_siege 12`
- `./visual_smoke.sh ./bevy_open_siege 8`
- `./audio_smoke.sh ./bevy_open_siege 12`
- `./support_diagnostics.sh . support-diagnostics`

## Bug Report Evidence

When reporting a bug, include only files you intentionally choose to share. Useful evidence can include:

- a short description of what happened and what you expected
- reproduction steps
- the release archive filename and checksum from `SHA256SUMS`
- terminal output from the failing command
- screenshots or recordings
- the local save file, if the issue requires save-state reproduction

Do not attach unrelated personal files. The game is offline and has no automatic crash uploader, telemetry, analytics, accounts, or network services.

`support_diagnostics.sh` creates a local diagnostics folder with command output and package metadata. It does not copy save files, screenshots, recordings, crash dumps, or unrelated personal files. Review the folder before attaching it to any report.

## Save Data

Progress is stored locally. On Linux the default save path is `$XDG_DATA_HOME/bevy_open_siege/bevy_open_siege_save.ron` or `~/.local/share/bevy_open_siege/bevy_open_siege_save.ron`. Set `BEVY_OPEN_SIEGE_SAVE_PATH=/path/to/bevy_open_siege_save.ron` when testing with an isolated save.

`uninstall_linux_user.sh` preserves save data by default. Use `uninstall_linux_user.sh --purge` only when you intentionally want to remove the save file.

## Release QA Status

Final release approval still requires the manual QA evidence listed in `QA_SIGNOFF.md`, including full campaign playthrough, accessibility, audio-device, performance, platform package, and final signoff checks.
