# Art Assets

Bevy Open Siege uses original AI-assisted raster art for release branding, character reference sheets, runtime unit sprites, gameplay effects, board textures, and UI chrome.

## Production Art

| File | Use | Source |
| --- | --- | --- |
| `assets/art/plants-sheet.png` | Plant roster character sheet and crop source | Generated for Bevy Open Siege with imagegen on 2026-06-19 |
| `assets/art/monsters-sheet.png` | Monster roster character sheet and crop source | Generated for Bevy Open Siege with imagegen on 2026-06-19 |
| `assets/models/plants/*.glb` | 10 runtime plant models | Generated in-project with Blender via `scripts/generate_3d_models.py` |
| `assets/models/monsters/*.glb` | 10 runtime monster models | Generated in-project with Blender via `scripts/generate_3d_models.py` |
| `assets/art/sprites/plants/*.png` | 10 plant sprite/reference crops from the plant sheet | Derived in-project with ImageMagick |
| `assets/art/sprites/monsters/*.png` | 10 monster sprite/reference crops from the monster sheet | Derived in-project with ImageMagick |
| `assets/art/effects/*.png` | Runtime projectile, sun pickup, fire, and explosion sprites | Created in-project with ImageMagick vector drawing |
| `assets/art/environment/*.png` | Runtime board lawn, lane, and soil-border textures | Created in-project with ImageMagick vector drawing |
| `assets/art/ui/*.png` | Runtime menu, HUD, pause, and end-screen chrome panels | Created in-project with Python standard-library PNG generation |
| `assets/branding/generated/store-capsule.png` | Store capsule, press kit header, marketing key art | Generated for Bevy Open Siege with imagegen on 2026-06-19 |
| `assets/branding/generated/app-icon.png` | Launcher icon and store avatar source | Generated for Bevy Open Siege with imagegen on 2026-06-19 |
| `assets/audio/*.wav` | Background loop and gameplay sound effects | Synthesized in-project with Python standard-library WAV generation |

## Generation Notes

The user requested local Comify/Comfy generation. No `comify`, `comfy`, or `comfyui` command was available in PATH during generation, and no ComfyUI HTTP endpoint responded at `127.0.0.1:8188`. The checked `/home/freeman.xiong/cuda/main.py` file is a wallet parsing GUI, not an image generator.

To avoid blocking release progress, the assets above were generated with the available imagegen tool and copied into the project. Original generated files remain under:

`/home/freeman.xiong/.codex/generated_images/019edcdc-bfb7-7f03-9fa1-653e12427f67/`

## Prompt Summary

- Plant sheet: original family-friendly stylized garden defense plant characters in a 5 by 2 roster sheet, no text, no direct likeness to existing games.
- Monster sheet: original family-friendly whimsical undead garden invaders in a 5 by 2 roster sheet, non-gory, no text, no direct likeness to existing games.
- Store capsule: wide original garden battlefield key art, plants defending against whimsical undead invaders, no title text.
- App icon: centered sprout shield emblem with pea-cannon leaf silhouette, no text.
- UI chrome: rounded green/gold translucent panels for menus, HUD bars, pause, and end screens.
- Audio: simple synthesized PCM WAV tones for background loop, placement, shooting, collection, monster defeat, victory, and defeat cues.

## Replacement Policy

These PNG files are tracked as `production_art` in `assets/manifest.ron` and are required by the release archive smoke test. If the local Comify pipeline becomes available, regenerate the same deliverables and replace these files with equal or higher resolution PNGs, then run:

```bash
./scripts/release_check.sh
```
