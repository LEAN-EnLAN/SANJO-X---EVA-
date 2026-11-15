# Eva explainer (Manim)

Vector explainer in the style of a math-animation video, built with Manim Community (the community-maintained fork of the 3Blue1Brown engine; it renders on CPU with Cairo, so no GPU or OpenGL is needed).

- `eva.py`: 7 scenes (S1_Title … S7_Close). Every asset is drawn in code.
- `sfx/`: sound effects rendered from the earshot bank (MIT, synthesised from code) with `earshot sfx render <id>`.

Render one scene: `manim -r 1920,1080 --fps 30 eva.py S2_Distance`

Facts used on screen:
- radio delay from orbital geometry (circular orbits, labelled as simplified; real range about 3–22 min each way);
- survival after cardiac arrest falls about 7–10% per minute without defibrillation;
- NOAA S-scale proton thresholds (S1 ≥10 pfu, S2 ≥100, S3 ≥1000, ≥10 MeV).
