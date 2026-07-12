# Bevy Open Siege Store Screenshots

This checklist defines the screenshot set required before store submission and press distribution for the Bevy Open Siege Linux x86_64 release-candidate package.

## Required Captures

- `screenshots/01-title-menu.png`: title menu with level list, language state, and settings line visible.
- `screenshots/02-early-defense.png`: early campaign gameplay showing sun economy, at least three plant types, and multiple active lanes.
- `screenshots/03-special-enemies.png`: mid-campaign gameplay showing armored, fast, or support enemies with projectiles and effects visible.
- `screenshots/04-late-siege.png`: late campaign gameplay showing high pressure, giant threats, and the full HUD.
- `screenshots/05-victory-summary.png`: victory or end-state screen showing score and retry flow.

## Capture Requirements

- Capture from the release-candidate package, not from an editor or debug-only build.
- Use `store_screenshot_check.sh --plan .` from the extracted package root before capture.
- Use `store_screenshot_check.sh --capture-pack . screenshots 10` to capture the deterministic title, gameplay, special-enemy, late-siege, and victory store scenes from the release package.
- Use `store_screenshot_check.sh --capture-startup . screenshots 01-title-menu.png 8` for the title menu capture.
- Use `store_screenshot_check.sh --capture-current screenshots <filename>` after manually staging gameplay or end-state captures.
- Run `store_screenshot_check.sh --validate-dir screenshots` before final screenshot approval.
- Use 16:9 output at 1920x1080 or higher for store uploads.
- Keep UI language coverage: at least one English screenshot and one Chinese screenshot.
- Keep audio disabled unless the capture workflow also needs audio-device QA.
- Do not crop out the HUD, seed bank, score, or visible lane state.
- Do not submit screenshots with debug overlays, terminal windows, external desktop chrome, or unreadable text.

## QA Requirements

- Compare captures against `visual-smoke.txt` and `visual-readability-audit.txt`.
- Confirm the screenshot set shows real gameplay, not only branding art or character sheets.
- Confirm plant, monster, projectile, effect, board, HUD, menu, and end-state visuals are represented across the set.
- Attach the final screenshot filenames and dimensions to `qa-session/store-screenshots.md`.
- Store review remains manual because the final image selection must be judged for composition, readability, platform rules, and current storefront requirements.
