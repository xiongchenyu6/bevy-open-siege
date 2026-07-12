# Bevy Open Siege Privacy Notice

Bevy Open Siege is an offline single-player game.

## Data Collection

Bevy Open Siege does not include telemetry, analytics, ads, online accounts, leaderboards, crash uploaders, or network services. The game does not intentionally collect, transmit, sell, or share personal information.

## Local Data

Bevy Open Siege stores local gameplay settings and progress only on the player's device. The save file can contain:

- selected language
- unlocked campaign level count
- best scores
- master volume
- fullscreen preference

On Linux the default save path is `$XDG_DATA_HOME/bevy_open_siege/bevy_open_siege_save.ron` or `~/.local/share/bevy_open_siege/bevy_open_siege_save.ron`. QA and portable builds can override this with `BEVY_OPEN_SIEGE_SAVE_PATH=/path/to/bevy_open_siege_save.ron`.

## Installation And Removal

The Linux user-local installer copies game files into the user's data directory and creates launcher metadata. `uninstall_linux_user.sh` removes installed app files while preserving save data by default. `uninstall_linux_user.sh --purge` removes save data only when explicitly requested.

## Support Requests

Support requests are player-initiated. If a player chooses to send a bug report, they should avoid sharing personal files and should only attach the requested release evidence, logs, screenshots, or save files they are comfortable disclosing.

## Changes

Privacy behavior should be reviewed before every release. Any future feature that adds networking, telemetry, crash reporting, cloud saves, accounts, or third-party services must update this notice before release.
