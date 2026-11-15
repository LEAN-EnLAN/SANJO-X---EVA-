# Eva landing page

The public landing page for Eva by WearMeds, team Sanjo-X, NASA Space Apps Challenge 2026. It is a single HTML page (`index.html`) with its assets in `a/` and the team rail in `team-rail.css` and `team-rail.js`. It is published as a claude.ai artifact.

## Sections
1. Hero: Eva, the Super Intelligence copilot, over a looping film cut from the team's 30 s video (see "Hero film" below).
2. Statement: wristband, emergency station, ship hub.
3. Sensors record, Eva acts: the Artemis II ARCHeR actigraph against Eva's 10-second target.
4. What Eva does: twelve functions.
5. Wristband v2: two magnetic modules, suit mode, ECG on demand.
6. Case A and Case B scrollytelling.
7. Emergency station: numbered compartments (configurable per ship), compartment 01 lit, door and item sensors.
8. Inside the station: the English compartment legend.
9. How it links: one host (the ship hub), clients, the hub-down alarm path and the near-space link.
10. Solar alert, Earth link, arrival, why it matters, kit, business, team and contact.

## Hero film
The hero plays short muted loops cut from the team's 30 s video: `a/hero-{window,detach,journal,flare,route-en,route-es,alert}.mp4`, each with a `.jpg` still of its first frame. On every load the scene list is shuffled (and the opening scene avoids the last three a viewer saw, via `localStorage`), then two stacked `<video>`s cross-fade through it (~600 ms). Only the current clip and the next one are fetched; the opening still is the poster. Screens up to 760 px wide load the `-m` files (608 × 720 crops). The route scene follows the page language. Reduced motion shows one random still and does not autoplay; the round pause / play button can still start it. The film pauses when the hero is off-screen or the tab is hidden. The clips were cut from the film sources with ffmpeg; `a/AI-NOTICE.md` lists the scenes and the processing.

## Language (EN / ES)
English is the default and is what the HTML contains. The EN / ES switch in the top bar swaps to Spanish in place, with no reload; the choice is saved per viewer, and `?lang=es` (or `?lang=en`) forces a language. The Spanish text is the `ES` dictionary in the script after the footer, keyed by the English text, so **when you change an English string, update its key in `ES` too** (an English string with no entry stays in English). Elements marked `data-i18n` swap their whole inner HTML; strings written by the page script go through `I18N.tr()`. Spanish is Rioplatense with "vos", like the station screen ("Retirá el DEA"). The formal citation stays in English (`translate="no"`).

## Wristband renders and the remaining interim visuals
The wristband uses the Codex renders (1536 × 1024, 3:2): `a/band2-hero-white.webp`, `a/band2-exploded.webp` and `a/band2-base-alone.webp` for the pinned steps, `a/band2-turntable-000.webp` … `-315.webp` (8 frames, 1024 × 1024) for the turntable in the kit slab, `a/band2-on-wrist.webp` as a hero chip and `a/band2-cabin.webp` in Business. Hotspots are positioned in % of the 1536 × 1024 frame and were measured on the renders; the screen overlay (`.bscr .ui`, `matrix3d`) is fitted to the glass of `band2-hero-white`. If a render is re-framed, re-measure its hotspots. The suit-mode step stays an inline SVG cross-section.

The station visuals are still interim team references. To replace them, overwrite the files and keep the names and aspect ratios:
- `a/station-wide.webp`: 1400 × 987. The label tags, the screen overlay and the compartment 01 glow are positioned in % of this frame.
- `a/station-front.webp`, `a/station-doors-open.webp`: any 4:3 crop.

## Super Intelligence image notice
The page uses images generated with Super Intelligence. Team decision (Sun 15/11): the declaration lives in this README, in [`a/AI-NOTICE.md`](a/AI-NOTICE.md) and in the project description, not as a mark on each image. The page footer keeps one short line that points here.

- The Codex images (icons, wrist scenes, cabin and space scenes) were made with the built-in OpenAI ImageGen tool in Codex from prompts by team Sanjo-X written with Claude.
- The v2 wristband renders (`band2-*`) were made with the built-in OpenAI ImageGen tool in Codex. The station images are concept images generated with Super Intelligence that the team used as design references.
- The diagrams, the suit cross-section and the screen overlays are inline SVG and HTML, not images.
- No image contains NASA insignia or branding. The 3D Sun is NASA's free model from NASA 3D Resources.

See `a/AI-NOTICE.md` for the full list and the post-processing.

## Product rules
Eva gives diagnostic support and never decides a shock; the AED does. No harness and no wearable injector. Product text is in English, with a full Spanish version behind the EN / ES switch. Main source: Russell et al., *npj Microgravity* 9, 46 (2023).
