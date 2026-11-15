# Eva · Codex asset kit

You are generating the visual assets for **Eva**, a health copilot for passengers and crew on a commercial trip to Mars (NASA Space Apps Challenge 2026, team Sanjo-X, Rosario). A human team will animate these assets into a landing page and a 30-second video. Work only inside this folder.

## Model setup
- Reasoning/orchestration model: **gpt-6.1-terra** (released 2026-10-26; text output only, cannot draw).
- Image model: **gpt-image-2** (OpenAI image output model, released 2026-04-21). If your environment exposes a built-in image tool, use it with gpt-image-2. Otherwise run `python generate.py` with `OPENAI_API_KEY` set.
- Check the current OpenAI Images API docs before running: parameter names and allowed sizes may differ from `generate.py`. Fix the script if needed; do not change the prompts' intent.

## What to produce
1. Every asset listed in `prompts.json`, saved with the exact `file` name and folder.
2. `manifest.json` (see `manifest.template.json`): one entry per file with prompt used, model, size, date and any retries.
3. `contact-sheet.png`: all assets on one sheet with their ids, for quick review.
4. Zip everything as `eva-assets.zip` (folders `scenes/`, `layers/`, `frames/`, plus `manifest.json`, `contact-sheet.png`, `AI-NOTICE.md`).

## Hard rules (competition and law)
- **No NASA insignia ("meatball"), NASA worm logotype, NASA flag, mission patches, agency logos or any real brand.** NASA's insignia is legally protected and the competition forbids AI-generated NASA branding.
- **No readable text** inside images (no labels, numbers or fake UI text). The team adds all text in HTML. Device screens must be **pure black (#000000)** so UI can be composited on top.
- **No real or recognizable people.** Humans appear as stylized silhouettes or faceless editorial figures.
- **No medical gore**, no needles, no blood.
- Every file in `scenes/` must carry a small, visible watermark in the bottom-right corner reading `AI-generated` (white, ~2% of image height, 60% opacity). Files in `layers/` and `frames/` stay clean; the team watermarks them in the final composition.
- Keep `AI-NOTICE.md` in the zip unchanged.

## Visual direction
Read `STYLE.md` before generating. Consistency across assets matters more than any single image: same light direction (upper left, warm key + cool rim), same palette, same level of detail.

## Quality loop
For each asset: generate → check against the rules above and STYLE.md → regenerate at most twice if it fails (text appeared, logo appeared, wrong palette, screen not black, background not transparent for layers). Log every retry in the manifest.
