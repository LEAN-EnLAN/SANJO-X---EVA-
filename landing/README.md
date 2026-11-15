# Eva landing page

The public landing page for Eva by WearMeds, team Sanjo-X, NASA Space Apps Challenge 2026. It is a single HTML page (`index.html`) with its assets in `a/` and the team rail in `team-rail.css` and `team-rail.js`. It is published as a claude.ai artifact.

## Sections
1. Hero: Eva, the AI copilot.
2. Statement: wristband, emergency station, ship hub.
3. Sensors record, Eva acts: the Artemis II ARCHeR actigraph against Eva's 10-second target.
4. What Eva does: twelve functions.
5. Wristband v2: two magnetic modules, suit mode, ECG on demand.
6. Case A and Case B scrollytelling.
7. Emergency station: 12 compartments, compartment 01 lit, door and item sensors.
8. Inside the station: the English compartment legend.
9. How it links: one host (the ship hub), clients, the hub-down alarm path and the near-space link.
10. Solar alert, Earth link, Mars arrival, why it matters, kit, business, team and contact.

## Swapping the interim visuals
The wristband and station visuals are team reference images. To replace them with renders, overwrite the files and keep the names and aspect ratios:
- `a/band2-joined.webp`, `a/band2-stacked.webp`, `a/band2-parts.webp`: 1400 × 933 (3:2). Hotspots, the ECG pads and the screen overlay are positioned in % of this frame. If the framing changes, update their `left`/`top` values and the `matrix3d` in `.bscr .ui`.
- `a/station-wide.webp`: 1400 × 987. The English label tags, the screen overlay and the compartment 01 glow are positioned in % of this frame.
- `a/station-front.webp`, `a/station-doors-open.webp`: any 4:3 crop.

## AI notice
The page uses AI-generated images. Team decision (Sun 15/11): the AI declaration lives in this README, in [`a/AI-NOTICE.md`](a/AI-NOTICE.md) and in the project description, not as a mark on each image. The page footer keeps one short line that points here.

- The Codex images (icons, wrist scenes, cabin, space and Mars scenes) were made with the built-in OpenAI ImageGen tool in Codex from prompts by team Sanjo-X written with Claude.
- The v2 wristband and station images are AI-generated concept images that the team used as design references.
- The diagrams, the suit cross-section and the screen overlays are inline SVG and HTML, not images.
- No image contains NASA insignia or branding. The 3D Sun is NASA's free model from NASA 3D Resources.

See `a/AI-NOTICE.md` for the full list and the post-processing.

## Product rules
Eva gives diagnostic support and never decides a shock; the AED does. No harness and no wearable injector. Product text is in English. Main source: Russell et al., *npj Microgravity* 9, 46 (2023).
