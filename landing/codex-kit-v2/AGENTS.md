# Eva · Codex asset kit v2 (black and white variants, Apple-style motion layers)

You are generating the visual assets for **Eva by WearMeds**, a health copilot for passengers on a commercial trip to Mars (NASA Space Apps Challenge 2026, team Sanjo-X, Rosario). Eva is a proper name: write "Eva", never "EVA". A human team composites these assets into a landing page with simple, scroll-driven, Apple-style motion. Work only inside this folder.

## What changed since v1
- Every page-facing asset now comes in two versions: on **pure black `#000000`** and on **pure white `#FFFFFF`**. The v1 plum and purple backgrounds are retired.
- **Team decision: the shared cabin medical station is eliminated.** Personal equipment is now only:
  1. a **wristband** with a large, flexible, curved touch screen;
  2. a body **harness** carrying sensors, a defibrillation subsystem (electrode pads) and an injectable compartment (autoinjector). A small amber light marks the item to use.

  Do not draw the station or its cuff, oximeter or thermometer. v1 S2, F1–F3 and L3–L6 are obsolete. v1 L1, L2, L12 and L13 show the old devices, so they are replaced here.
- Motion uses **separate transparent layers** moved with CSS transforms, plus a 5-frame crossfade turntable. There are no long frame sequences, because the image model cannot keep many frames consistent.
- Not image assets (do not generate them): the Earth–Mars **signal pulse/arc** (the team draws it in SVG), the solar **particles** in the motion spec (canvas; `PT1-*` are optional), the **onboard computing hub** (an abstract SVG node if needed), and all UI and text (HTML).

## Models
- Orchestrator: **gpt-6.1-terra** (text only, cannot draw).
- Images: **GPT Image 2.5**, released 2026-09-08, which accepts text and image input:
  - `gpt-image-2.5-flare`: fast drafts (`--model flare`, writes to `drafts/`).
  - `gpt-image-2.5-sunburst`: quality and edits, used for finals (`--model sunburst`, writes to `out/`).
- **Before the first call, read the current OpenAI Images API docs.** Check model ids, allowed sizes, `background`, `quality`, `input_fidelity` and the multi-image edit syntax. Fix `prompts.json → api/models` first and `generate.py` second. Never change a prompt's intent.
- If the API allows sizes close to an asset's `target`, set `size` to the target. Otherwise generate at the largest allowed size; `--upscale` resizes to the target as a stopgap (a real upscaler is better).
- If your environment has a **built-in image tool** instead, run `python generate.py --dry-run --emit prompts-final.jsonl`. Generate each record with the tool, attaching the listed `refs` in order, and save the image to `out/_raw/<file>`. Then run `--post-only`. Record the model that **actually** ran in the manifest; v1 could not, so say so honestly if you cannot.

## How to run
```
pip install openai pillow
export OPENAI_API_KEY=...
python generate.py --model flare                         # 1. drafts of everything -> drafts/
#    review drafts/contact sheets, fix prompts if a whole family fails
python generate.py --model sunburst                      # 2. finals -> out/
python generate.py --ids HX2 --force --note "retry: pad drifted 20px from anchor"   # 3. targeted retries
python generate.py --post-only --upscale --qa --derive --sheet --zip                 # 4. finish
```
`--variant black|white|both` (default both) filters page variants; shared transparent layers are always included. `--ids` takes ids or bases (`S1` means `S1-B` and `S1-W`). Existing files are skipped unless you pass `--force`.

**Order matters.** Assets run in `prompts.json` order, because later assets use earlier ones as reference images:
1. Design masters: wristband `OR0`, then harness `HX0`.
2. Sets edited from the masters: orbit, exploded parts, glows, figures.
3. Planets, flare, spacecraft.
4. Scenes. Black first; each white scene is an edit of its black twin.
5. Cabin layers.
6. Shadows.

If a master is redone, redo everything that references it.

## Asset families (62 entries; full specs in `prompts.json`)
| Group | Ids | Notes |
|---|---|---|
| Wristband orbit | OR0, OR±30, OR±60 | 5 consistent yaw angles for a crossfade turntable; light fixed to the studio |
| Wristband exploded | WX0 anchor; WX1 strap, WX2 body, WX3 screen, WX4 sensor, WX5 battery/board | Same canvas, parts in assembled position |
| Wristband | WA1 on forearm (screen reads as a flat rectangle for the UI overlay), PL-B / PL-W on plinth | |
| Harness exploded | HX0 anchor; HX1 straps, HX2 defib pads, HX3 core module, HX4 injectable compartment, HX5 status lights | Same canvas, parts in assembled position |
| Harness glow | GL1 compartment lit, GL2 pads lit, each in -B and -W | Pixel-aligned to HX0 |
| Figures | FIG1 standing with harness and wristband (front 3/4), FIG2 slumped in a seat | Faceless mannequin style |
| Planets | PE / PE-G, PM / PM-G, PS / PS-G (disc plus separate glow), MS1 Mars surface | Light from the left on all |
| Space FX | FL1 prominence (origin at bottom centre), FL2 ejection streaks, PT1-B / PT1-W optional particles, ST1-B / ST2-B star fields | Stars are for the black page only |
| Spacecraft | SC1 hull, SC2 engine glow | |
| Scenes | S1–S5, each in -B and -W | Edges fall off to the page colour; watermarked |
| Cabin | CAB-BG / CAB-MID / CAB-FG, each in -B and -W | Hero parallax with 2–3 depths |
| Shadows | SH-OR0, SH-WX0, SH-HX0, SH-FIG1, SH-FIG2 | Built locally from alpha; white page only |

`--derive` builds the page-colour **chips** (1200²) and **inventory squares** (1600² / 1200²) that MOTION-SPEC asks for from finished layers. They need no model calls.

## Hard rules (competition and law)
- **No NASA insignia ("meatball"), worm logotype, flag, mission patches, agency logos or any real brand.** NASA's insignia is legally protected, and the competition forbids AI-generated NASA branding.
- **No readable text** in images: no labels, numbers or fake UI. Device screens are **pure black `#000000`** so the UI can be composited.
- **No real or recognizable people.** Use faceless figures only. In scenes, show people from behind with heads turned away or cropped. A visible face fails the image.
- **No gore, no needles, no blood.** The autoinjector stays inside its closed compartment.
- **Visible "AI-generated" watermark on full scenes only** (`S*`, `MS1`). `generate.py` stamps it bottom-right at about 2% of image height and 60% opacity: white on dark images, black on the white page. Never ask the model to draw it. Layers, plates, shadows and chips stay clean; the team watermarks the final composition.
- **Consistent light** in every asset: key from the upper left, light from the left for space, flow left to right.
- **Same camera for every layer of a set**, as stated in `cameras`: focal length, angle, centred framing, about 8% padding.
- Keep `AI-NOTICE.md` in the zip.

## Post-processing (automatic, in `generate.py`)
- Raw images go to `<root>/_raw/`, and finals are rebuilt from raw on every run, so `--post-only` can be repeated safely.
- **Edge lock** (opaque black and white assets): measures how far the outer 2% ring is from the page colour, then blends the outer 6% into the exact page colour and snaps near-black or near-white pixels. If the measured deviation is above 10, the manifest flags it. Regenerate in that case, because the model did not vignette.
- Transparent check: all four corners must have alpha = 0.
- Shadows: soft black contact or float ellipses built from the source layer's alpha, on the same canvas, to sit under the layer.
- `--qa` writes `qa/<set>.png` (anchor | parts composite | black page with glows | white page with glows) and prints an **alpha IoU** of the parts against the anchor. Aim for > 0.90.

## Quality loop (per asset; at most 2 regenerations, log each with `--note`)
Check each image for:
1. text, logos or a face;
2. a screen that is not pure black;
3. edges that are not the page colour (edge flag);
4. a background that is not transparent on layers;
5. a **coloured fringe or halo** at alpha edges (v1 layers had a purple fringe);
6. drift from the design master (wristband width, curved screen; harness parts and positions);
7. parts or glows misaligned with their anchor (IoU < 0.90, or visible offset in `qa/`);
8. wrong light side.

If exploded parts cannot be aligned after 2 retries, keep the best version, write its pixel offset relative to the anchor in `notes`, and the team will correct it in CSS. If glows drift, note it; the team can paint them in CSS instead.

## Deliverables
1. Every asset in `prompts.json` under `out/` with its exact `file` path (`black/`, `white/`, `shared/`).
2. `out/manifest.json` following `manifest.template.json`.
3. `contact-sheet-black.png` and `contact-sheet-white.png` (`--sheet`), plus `qa/`.
4. `eva-assets-v2.zip` (`--zip`): `out/` without `_raw/`, plus `AI-NOTICE.md`.
5. A short report listing which assets needed retries and why, which sizes were native and which were upscaled, and any unresolved flags.
