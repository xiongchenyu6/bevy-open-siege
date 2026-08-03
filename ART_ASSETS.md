# Art Assets

Bevy Open Siege uses original AI-assisted raster art for release branding, character reference sheets, runtime unit sprites, gameplay effects, board textures, and UI chrome.

## Production Art

| File | Use | Source |
| --- | --- | --- |
| `assets/art/plants-sheet.png` | 1280x640 plant roster overview | Composited from the production character cutouts on 2026-07-30 |
| `assets/art/monsters-sheet.png` | 1280x640 monster roster overview | Composited from the production character cutouts on 2026-07-30 |
| `assets/models/plants/*.glb` | 10 packaged legacy plant reference models | Generated in-project with Blender via `scripts/generate_3d_models.py` |
| `assets/models/monsters/*.glb` | 10 packaged legacy monster reference models | Generated in-project with Blender via `scripts/generate_3d_models.py` |
| `assets/art/sprites/plants/*.png` | 10 runtime plant portraits and battlefield billboards, 512x640 RGBA | Generated with local ComfyUI Flux and cut out with local SAM3 on 2026-07-30 |
| `assets/art/sprites/monsters/*.png` | 10 runtime monster portraits and battlefield billboards, 512x640 RGBA | Generated with local ComfyUI Flux and cut out with local SAM3 on 2026-07-30 |
| `assets/art/effects/*.png` | Runtime projectile, sun pickup, fire, and explosion sprites | Created in-project with ImageMagick vector drawing |
| `assets/art/environment/*.png` | Runtime board lawn, lane, and soil-border textures | Created in-project with ImageMagick vector drawing |
| `assets/art/ui/menu-panel.png`, `hud-panel.png`, `end-panel.png` | Legacy runtime panel textures used by pause and end screens | Created in-project with Python standard-library PNG generation |
| `assets/art/ui/menu-background.png` | Illustrated main-menu garden battlefield and campaign board | Generated for Bevy Open Siege with imagegen on 2026-07-30 |
| `assets/art/ui/hud-overlay.png` | Illustrated top status frame and 10-slot seed-bank frame | Generated for Bevy Open Siege with imagegen on 2026-07-30, then chroma-keyed in-project |
| `assets/branding/generated/store-capsule.png` | Store capsule, press kit header, marketing key art | Generated for Bevy Open Siege with imagegen on 2026-06-19 |
| `assets/branding/generated/app-icon.png` | Launcher icon and store avatar source | Generated for Bevy Open Siege with imagegen on 2026-06-19 |
| `assets/audio/*.wav` | Background loop and gameplay sound effects | Synthesized in-project with Python standard-library WAV generation |

## Generation Notes

The complete runtime UI set remains packaged under `assets/art/ui/*.png`; the table above separates generated illustrations from the earlier programmatic panel textures so their provenance stays explicit.

The 2026-07-30 character pass used the local ComfyUI service at `127.0.0.1:8188`. Flux produced individual 768x960 character renders, local SAM3 produced subject masks, and ImageMagick normalized the approved results to optimized 512x640 RGBA PNGs. Runtime plants and monsters now use these same cutouts for both card portraits and animated battlefield billboards, keeping the visual language consistent.

## Prompt Summary

- Plant roster: original family-friendly greenhouse defenders with distinct weapon, support, defense, trap, frost, fire, and explosive silhouettes; premium hand-painted 3D maquette rendering; no text or direct likeness to existing games.
- Monster roster: original non-gory corrupted greenhouse invaders with a shared gray-green bark skin and amber-eye faction language, plus role-specific armor and tools; no text or direct likeness to existing games.
- Store capsule: wide original garden battlefield key art, plants defending against whimsical undead invaders, no title text.
- App icon: centered sprout shield emblem with pea-cannon leaf silhouette, no text.
- Main menu: original golden-hour garden-defense scene with an empty carved campaign board, no text or logos.
- Gameplay HUD: green, wood, and brass top status frame plus exactly 10 bottom card openings, generated over a flat chroma background with no text or icons.
- Legacy UI chrome: rounded green/gold translucent panels for pause and end screens.
- Audio: simple synthesized PCM WAV tones for background loop, placement, shooting, collection, monster defeat, victory, and defeat cues.

## Replacement Policy

These PNG files are tracked as `production_art` in `assets/manifest.ron` and are required by the release archive smoke test. Any later art pass must preserve the same paths, transparent canvas contract, and equal or higher source resolution, then run:

```bash
./scripts/release_check.sh
```
