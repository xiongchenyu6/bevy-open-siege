# Bevy Open Siege Press Kit

## Boilerplate

Bevy Open Siege is a lane-defense strategy game about holding a greenhouse line with specialized plants, sun economy, and escalating undead waves.

## Fact Sheet

- Developer: Bevy Open Siege contributors
- Genre: Strategy / Lane Defense
- Players: Single player
- Languages: English, Chinese
- Current platform target: Linux x86_64 release-candidate package
- Engine: Bevy

## Available Media

- Production app icon: `assets/branding/generated/app-icon.png`
- Production store capsule: `assets/branding/generated/store-capsule.png`
- Plant character sheet: `assets/art/plants-sheet.png`
- Monster character sheet: `assets/art/monsters-sheet.png`
- UI chrome panels: `assets/art/ui/`
- Fallback app icon SVG: `assets/branding/icon.svg`
- Fallback store capsule SVG: `assets/branding/capsule.svg`
- Store description draft: `STORE_PAGE.md`
- Store screenshot capture checklist: `STORE_SCREENSHOTS.md`
- Store screenshot helper script: `store_screenshot_check.sh`
- Art asset notes: `ART_ASSETS.md`
- Content rating notes: `CONTENT_RATING.md`

The PNG branding, character-sheet, runtime sprite, environment, effect, and UI chrome assets are included in the release package and tracked as production art in `assets/manifest.ron`. The final store screenshot set is captured or validated with `store_screenshot_check.sh` and tracked through `STORE_SCREENSHOTS.md` and `qa-session/store-screenshots.md` because storefront composition requires manual review.

## Content Notes

- Fantasy non-realistic combat against stylized undead creatures.
- No blood or gore.
- No in-app purchases, ads, gambling, online multiplayer, chat, user-generated content, telemetry, analytics, or account system.
