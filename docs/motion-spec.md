# Motion spec: reference landing → Eva by WearMeds

Reference: `https://accountant-website-template-for-cursor-free-html-t-v1.21st.app/` (a template for an accounting firm called "Tallyhouse").
Captured 2026-11-15 with Playwright/Chromium at 1440x900 (DPR 1), plus one capture at 390 px wide. Full page height at 1440: 17,612 px.
Screenshots are in this folder: `load-*.png` (top of page, 100 ms → 4.5 s), `shot-00..58.png` (every 300 px of scroll), `mobile-390-*.png`.
Note: a few shots (e.g. shot-26, shot-29) show text doubled up. That happened during capture: the page's own smooth-scroll and the animations caught up after `scrollTo`. It is not a design effect.

**This spec describes behaviour only. Nothing below copies their code. Every recipe is a fresh description for us to implement.**

---

## 0. How the page is built (confirmed from source)

- **No animation libraries.** It ships no GSAP, ScrollTrigger, Lenis, Framer, Three.js, AOS or Swiper, and has no canvas, WebGL, video or image sequences. `window` exposes none of these globals and the DOM has no `<canvas>` or `<video>`. The only external script is the Cloudflare analytics beacon.
- **Everything is one HTML file** with inline `<style>` blocks grouped into `@layer tokens, base, atoms, layout, sections, scenes`, plus about 13 small inline scripts. Images are Unsplash AVIFs.
- **Main technique: native CSS scroll-driven animations.** Each section sets `view-timeline: --name block` and children use `animation-timeline: --name` with `animation-range: cover <start> cover <end>`. `document.getAnimations()` reported 109 animations, most on a `ViewTimeline`.
- **The gate:** a script adds a `data-8b-live` attribute on `<html>` only when `prefers-reduced-motion: no-preference` AND `CSS.supports('animation-timeline: view()')`. Every scene rule is scoped under `[data-8b-live]`. Without the attribute the page stays static, which today covers Firefox and older Safari.
- **A small JS "motion kit" (about 120 lines) handles what CSS cannot:**
  1. It measures geometry (offsets, sizes, travel distances) and writes the results to CSS custom properties. It re-measures on `ResizeObserver`, `fonts.ready` and `load`.
  2. It numbers children so CSS can stagger them: `--i` and `--n` on items of a `[data-8b-items]` group.
  3. It runs a one-shot "enter" class: an element gets `.is-in` once its top passes 88% of the viewport height, and CSS keyframes then play by time, not by scroll.
  4. It provides a scroll-progress helper (`track(el, "top 58%", "bottom 58%", cb)`) for step logic.
  5. Its own **smooth wheel scrolling** runs only on `pointer: fine` devices.
  6. An IntersectionObserver pauses infinite ambient animations while they are off-screen.
- **`@property`** registers animatable custom properties (`<number>` and `<length>`). This lets one keyframed number drive a whole `clip-path` formula or a gradient size.
- **Easing vocabulary** (named by feel):
  - "power3 in-out": a very slow start and end, written as `linear(...)` stops.
  - "power2 in-out": written as `linear(...)` stops.
  - "expo-ish in-out" for wipes: `cubic-bezier(.76,0,.24,1)`.
  - "cubic out" for settling: `cubic-bezier(.33,1,.67,1)`.
  - "snappy out" for load-ins: `cubic-bezier(.2,.7,.1,1)`.
  - Hover fills: `cubic-bezier(.7,0,.2,1)`.

**Converting `cover` ranges to plain JS (needed for our fallback).** Take a section whose top is at document Y `T`, with viewport height `V`. The CSS point `cover X` (X in px, where `100vh = V`) is reached when `scrollY = T − V + X`. The progress of a range `[a, b]` is therefore `p = clamp((scrollY − (T − V + a)) / (b − a), 0, 1)`. Every recipe below uses this formula.

---

## 1. Effect inventory

| # | Section | What moves | Trigger | Technique (confirmed) | Timing / easing | Our equivalent idea |
|---|---|---|---|---|---|---|
| 1 | Nav | The fixed 48 px bar slides up out of view, and comes back on scroll up | scroll direction | JS rAF scroll listener tracks how far you have scrolled in one direction; past 24 px the bar hides with `translateY(-100%)`. It always shows when `y<80` and on focus | .45 s `cubic-bezier(.7,0,.2,1)` | Same for our top bar on black/white |
| 2 | Opening (load) | 8 photo tiles around the hero card wipe open from the bottom | load | CSS keyframe `clip-path: inset(100% 0 0 0)` → `inset(0)` with a per-tile delay `--i*60ms`, in a hand-set order (not DOM order) | .7 s `cubic-bezier(.2,.7,.1,1)`, 60 ms stagger | Cabin "windows" / device chips open around the hero headline |
| 3 | Opening (load) | The headline block and actions rise 28 px | load | CSS keyframe `translate3d(0,28px,0)` → none; actions delayed .12 s | .8 s `cubic-bezier(.2,.7,.1,1)` | Hero type rises in |
| 4 | Opening (ambient) | The photo inside each tile drifts slowly up and down | always, paused off-screen | `img` is 128% tall, offset −14%; infinite `alternate` keyframe `translateY(-9%)`↔`9%`; odd tiles 5.2 s, even tiles 4.4 s `alternate-reverse`; an IntersectionObserver adds `.is-idle` to pause it | ease-in-out, infinite | Slow parallax drift inside square media (stars, the cabin) |
| 5 | Opening (scroll) | **The hero card's black background grows into a full-screen black canvas** | scroll 0 → ~48 vh | Sticky 100svh stage. A full-bleed `.op-ground` layer has `clip-path: inset()` written as a formula of a registered `--spread` number (0 = clip equals the card rect, 1 = full screen); a scroll timeline animates `--spread` | range `cover 102vh → 148vh` (about 18 px → 430 px of scroll at 900 px tall), power2 in-out | **Main "planet reveal" or "cabin opens" move:** the card becomes the whole universe |
| 6 | Opening (scroll) | **The tiles fly outward radially, tilt ±8–9° and shrink to .82**, hold, then come back (hidden by then) | scroll 0 → ~92 vh | JS measures each tile's unit vector from the card centre, multiplies it by half the stage size and stores the result as `--dx/--dy`. A keyframe scrubbed on the view timeline: 0%→54% fly out, 54–67% hold, →100% return. Odd/even tiles get opposite rotation | `cover 100vh → 192vh`; first segment eases in (`cubic-bezier(.33,0,.67,.33)`) | Device parts or satellites "explode" off-screen while space opens |
| 7 | Opening (scroll) | The year label "26/27" slides from the card's corner to the stage's bottom-right corner | scroll | JS computes the offset from the card corner to the stage corner and stores it in CSS vars; a scrubbed `translate` | same range as #5 | Mission clock "SOL 0 / T+…" flies into the corner |
| 8 | Opening (2nd screen) | A full-screen black panel scrolls up over the sticky stage; 3 words in the statement get a coloured highlighter sweep one after another | scroll | The 2nd screen has `z-index:2` and slides over the sticky stage. Each `mark` animates `background-size: 0%→100%` and its text colour from white to black on staggered scroll ranges (148/153/158 → 166/171/176 vh) | each sweep is 18 vh of scroll, offset 5 vh; `cubic-bezier(.76,0,.24,1)` | Key words ("solar flare", "22 min delay") get a highlighter sweep |
| 9 | Global | Thin rules draw from left to right | enter (88% viewport) | `.is-in` class → keyframe `scale: 0 1 → 1` (`transform-origin: left`), with a `--rule-delay` of 0/80/160 ms | 1.1 s, power3 in-out | Hairlines under section heads |
| 10 | Global | Photos wipe in from the bottom and settle from 1.12 to 1 scale | enter | `.is-in` → `clip-path: inset(100% 0 0)` → `inset(0)` (1.1 s `cubic-bezier(.76,0,.24,1)`) plus `scale 1.12→1` (1.4 s cubic out), delayed per item | 1.1 s / 1.4 s | Every square image enters like this |
| 11 | Global | Highlighter mark behind words | enter | `.is-in` → `background-size 0%→100%` on a linear-gradient background with `box-decoration-break: clone` so the highlight wraps across lines | .9 s `cubic-bezier(.76,0,.24,1)` | Same |
| 12 | Global | Duotone photos | static | `img{filter:grayscale(1) contrast(1.12) brightness(1.06); mix-blend-mode:multiply}` over a coloured background (`--photo-tint`) | — | **Do not use on black** (multiply goes to black). On black, use `screen` over a dark tint. On white, `multiply` works |
| 13 | Month | Sticky report sheet on the right; a coloured band slides behind whichever part of the sheet matches the step being read on the left; numbered step keys flip to filled | scroll steps | `track(step,'top 58%','bottom 58%')`: a step is "reached" once its top passes 58% of the viewport. JS moves the absolutely positioned band (`translate` Y plus `scaleY` set to the part's height) and sets `data-lit` / `data-cash` | band .6 s `cubic-bezier(.76,0,.24,1)`; keys .4 s | **Case 01 / Case 02 scrollytelling:** text steps on one side, a sticky device or station on the other with the active region lit |
| 14 | Month | 12 mini bars grow from 6% to 100% height | step reached (`data-cash`) | CSS transition on `scaleY .06→1`, `transform-origin: bottom`, `--i*35ms` stagger; heights come from `data` values normalised by JS | .8 s `cubic-bezier(.25,1,.5,1)` | Heart-rate / SpO2 sparkline bars on the wristband screen |
| 15 | Services | **Full-screen coloured slabs stack:** each slab is `position:sticky; top:0`, 100svh tall; as the next slab slides over, the previous one shrinks to 0.9 (origin top), which looks like it sinks into depth | scroll | `services-sink` scale 1→0.9 on a shared view timeline; JS writes each slab's cumulative offset (`--at`, `--next`) so each range starts where the next slab begins to cover it (+15 vh → +105 vh). Only active at ≥900 px wide and ≥680 px tall | linear scrub over 90 vh | **Product inventory:** one sticky full-screen slab per device, stacking |
| 16 | Services | **The photo "de-halftones":** big dots in the slab colour cover the photo, then shrink to a fine 6 px screen while the photo scales 1.2→1 | scroll | A `.sv-dots` overlay uses `radial-gradient(circle, slabColor r, transparent r+.7px)` tiled at `cell` size. Registered `<length>` props animate cell 46→6 px and radius 20→1.2 px. `img` scale 1.2→1 | `cover at+15vh → at+105vh`, power2 in-out | Telemetry "resolving" (a low-res signal becomes a clear image) for the Earth-link section |
| 17 | Switch | Square photo, rule draws, mark sweep | enter | Primitives #9–#11 | — | — |
| 18 | Year | **Pinned horizontal timeline:** the section pins and a 12-month track scrolls sideways under a fixed orange playhead line at 50% of the width; each chip fills with its category colour as the playhead crosses it | scroll (vertical → horizontal) | The section gets an `::after` spacer as tall as the horizontal overflow (`run = track.scrollWidth − view.clientWidth`). `.yr-pin` is `sticky; top:0`. The track translates `0 → −run` over `cover 100vh → 100vh+run`, linear. JS computes each chip's crossing interval `[a,b]` relative to the line; the chip fill (`scaleX` from `from` to 1, origin left) is scrubbed over `100vh+a → 100vh+b` | linear, 1:1 with scroll | **Earth–Mars signal timeline / mission timeline:** a playhead line at "now", with packets lighting up as they cross |
| 19 | Cases | Testimonial blocks with portrait wipes and word highlights | enter | Primitives #10 and #11 only | — | — |
| 20 | People | **Cards "deal" into a 4-column grid:** they start offset toward the grid centre (x ±23 vw / ±7.7 vw, y ±12%+80 px), rotated −7°/−4°/5°/8°, then settle to 0 | scroll | Per-card start values sit in inline CSS vars; the shared view timeline range is staggered `cover 5vh+4.06vh*i → 55.8vh+4.06vh*i` | cubic out | **Inventory grid:** oximeter, cuff, thermometer and the rest are dealt onto the table |
| 21 | FAQ | `+` icon rotates 45° on open; button background wipes up on hover | click / hover | `details[open]` plus a transition; `::before` with `clip-path inset(100% 0 0 0) → inset(0)` | .35–.4 s `cubic-bezier(.7,0,.2,1)` | (We have no buttons; reuse the wipe on link hovers) |
| 22 | Fees | Three price blocks rise like a bar chart; the price label rides on top of each block | scroll | The block height comes from the price ratio (JS: `value/max`, min .08); the block is revealed by `clip-path inset(100% 0 0) → 0`; the label `translate(0, blockHeight) → 0`; stagger `10.9vh*i` | `cover 12vh+… → 48vh+…`, cubic out | Signal-strength or radiation-dose bars; time-delay comparison bars |
| 23 | Contact | No motion beyond primitives | — | — | — | — |
| 24 | Footer | The giant wordmark rises (translate Y 35% → 0, scale 1.08 → 1 from bottom-left) and the year slides in from the left (−60% of the free width) | scroll into footer | View timeline on the footer, `cover 0% → 100%−100vh`; JS sizes the wordmark font as width / (characters × .445) so it spans exactly full width | cubic out | Huge "EVA" wordmark closing the page |
| 25 | Global | **Smooth wheel scrolling** | wheel | JS calls `preventDefault` on wheel and lerps `current` toward `target` with `k = 1 − exp(−5.3·dt)` (dt capped at 50 ms); delta ×0.95; snaps when within 0.5 px; skips nested scrollers, ctrl-zoom and horizontal wheels; disabled while the menu is open. Off on touch | lerp time constant ≈ 190 ms | Optional; worth adding because it makes scrubbed scenes feel "Apple-smooth" |
| 26 | Global | Hover fills on links, buttons and nav | hover | `background-size` 0→100% or `clip-path inset(50%)→0` | .45–.5 s `cubic-bezier(.7,0,.2,1)` | Link underline fill only (no buttons) |
| 27 | Mobile 390 | Same scenes with re-laid-out tiles (6 visible, 2 hidden), a card at 64svh, slabs not sticky, timeline months 76 vw wide | — | Media queries; the services scene switches to a per-photo timeline | — | Reduce the pinning on phones |

---

## 2. Rebuild recipes (implement from scratch in vanilla JS/CSS)

### Shared engine: `scrollScene` (write once, ~60 lines)
- Keep a registry of `{el, ranges:[{startPx,endPx}], onProgress}`. On `scroll` (passive) schedule one rAF. In it, read `scrollY` once, compute `p = clamp((scrollY − start)/(end − start))` for each range and call `onProgress(p)`. The callback writes **only** CSS custom properties (`el.style.setProperty('--p', p)`), and CSS does `transform` / `clip-path` / `opacity` from `var(--p)`.
- Range helper: `rangeFor(section, a, b)` returns the scroll positions `T − V + a` and `T − V + b`, where `a`/`b` accept "vh" or px, and `T` is the section's top measured in the document (sum of `offsetTop`s, which is not affected by transforms).
- Re-measure on `ResizeObserver(document.body)`, `document.fonts.ready` and `load`.
- Optional progressive enhancement: if `CSS.supports('animation-timeline: view()')`, scenes can use native CSS timelines instead. Keep the JS path as the default, because it works in Firefox and older Safari, and our recipes then behave the same everywhere.
- Reduced motion: if `prefers-reduced-motion: reduce`, set every `--p` to its final state and skip pinning.
- Easing helpers in JS: `easeInOutQuad`, `easeInOutCubic`, `easeOutCubic`, and `easeInOutExpo ≈ cubic-bezier(.76,0,.24,1)`. Apply them to `p` before writing the variable. Scrubbed scenes feel best with in-out easing inside each sub-range and linear across the whole scene.

### R1. Card → full-bleed canvas ("the universe opens") — from #5
- DOM: `section.hero` (height **200svh**) > `.stage` (`position:sticky; top:0; height:100svh; overflow:hidden`). Inside the stage: `.ground` (absolute, inset 0, background #000) and `.card` (absolute, centred, `--cw` × `--ch`, e.g. `min(52vw,680px)` × `min(100svh−150px,640px)`, top `--cy`).
- Clip math with `s = eased p ∈ [0,1]`, where L, R, T, B are the card's insets from the stage edges:
  `clip-path: inset(calc(T*(1-s)) calc(R*(1-s)) calc(B*(1-s)) calc(L*(1-s)))`.
  For a horizontally centred card, `L = R = (100% − cw)/2`, `T = cy` and `B = 100% − cy − ch`.
- Range: scroll 2 vh → 48 vh after the hero top; ease in-out quad.
- Content inside the card stays fixed in place (it is not scaled), so the type looks as if the walls fall away around it. Optional: animate the card's own background to transparent once `s>0.95`.
- The second screen (`z-index:2`, 100svh, black) scrolls up over the sticky stage naturally. No JS is needed for that hand-off.

### R2. Radial scatter of satellites — from #6
- For each tile, measure its centre `(tx,ty)` and the card centre `(cx,cy)`. Compute `d = (tx−cx, ty−cy)`, `u = d/|d|`, and the fly vector `F = (u.x · stageW·0.5, u.y · stageH·0.5)`. Store `--fx/--fy` in px. Recompute on resize.
- Per tile, `--turn` = −8° for odd tiles and +9° for even tiles (or alternate ±(6…10)° to vary it).
- Timeline over scroll 0 → 92 vh: segment A `p ∈ [0,.54]` → `k = easeIn(p/.54)`; segment B `[.54,.67]` → `k = 1`; segment C `[.67,1]` → `k` returns to 0 (it is hidden under the next panel anyway, so you can skip C and hold).
  `transform: translate(calc(var(--fx)*k), calc(var(--fy)*k)) rotate(calc(var(--turn)*k)) scale(calc(1 - .18*k))`.
- Layering: tiles sit **under** `.ground`, so the growing black swallows them as they fly. That combination is what makes it dramatic.
- Load-in (from #2): each tile frame has `clip-path: inset(100% 0 0 0)` → `inset(0)`, .7 s `cubic-bezier(.2,.7,.1,1)`, delay `order*60ms` in a hand-picked scatter order (not left to right).
- Ambient drift (from #4): media 128% tall, offset −14%; `translateY` ±9% alternating, 4.4 s / 5.2 s, even tiles reversed. Pause with an IntersectionObserver.

### R3. Sticky stacking slabs with depth sink — from #15
- Each slab: `position:sticky; top:0; min-height:100svh`; slabs are siblings in one list. Enable it only when `min-width:900px and min-height:680px`.
- JS: cumulative `at_i = Σ heights before i`. For slab i (not the last), `sink` progress spans scroll `[listTop + at_{i+1} − V + 0.15V, listTop + at_{i+1} − V + 1.05V]`. Simplified: "starts when slab i+1's top is 15 vh into the viewport and ends 90 vh later".
- `transform: scale(calc(1 - .1*p)); transform-origin: 50% 0`. Optional extras that suit black: `filter: brightness(1 − .5p)`, or a black overlay at opacity `.6p`, so the covered slab recedes into darkness.
- On a pure-black page, give alternate slabs `#000`/`#fff`, or let the device photo carry the colour.

### R4. Halftone resolve — from #16
- Overlay a `span.dots` over the image, `inset:0; pointer-events:none`:
  `background-image: radial-gradient(circle, var(--bg) var(--r), transparent calc(var(--r) + .7px)); background-size: var(--cell) var(--cell)`.
- Scroll-scrub `cell 46px → 6px` and `r 20px → 1.2px` (in-out quad), and the image `scale 1.2 → 1`, over the same 90 vh range as R3, or over a per-image range on mobile (from 5 vh into cover until 5 vh before the middle).
- Because the dots use the **page background colour**, the image looks as if it emerges from the page. On black: dots `#000`, image `mix-blend-mode: normal`.
- Implementation note: register `--cell`/`--r` via `CSS.registerProperty` (`syntax:'<length>'`) if animating them in CSS. In the JS engine, write the computed px directly instead.

### R5. Pinned horizontal track with playhead — from #18
- `section.timeline` > `.pin` (`position:sticky; top:0; min-height:100svh`) > `.view` (`overflow:clip`) > `.track` (flex, `width:max-content`, right padding about 52 vw so the last item can reach the centre line).
- Measure `run = track.scrollWidth − view.clientWidth`. Append a spacer after `.pin` with `height = run` px, so the pin lasts exactly `run` px of vertical scroll.
- Progress `p` over `[sectionTop, sectionTop + run]`; `track.style.transform = translateX(−p·run)` (linear, 1:1, which gives the "physical" feel).
- Playhead: a 2 px absolute line at `left:50%` of `.view`. For each item, compute its left edge `a` and right edge `b` relative to the line at `p=0`. The item's fill progress is `clamp((p·run − a)/(b − a))`, and it drives `scaleX` from `from = clamp(−a/width)` to 1, origin left. Items already past the line at start are pre-filled.
- Use it for the Earth→Mars signal: the track holds "packets" and the playhead is "now"; packets light up as they cross.

### R6. Scrollytelling with a sticky panel and a moving highlight — from #13 and #14
- Grid: steps column (each step `min-height:40svh`, with a hairline top border) and a sticky panel column (`position:sticky; top:64px`).
- A step is active when its top crosses 58% of the viewport (`getBoundingClientRect().top < 0.58·innerHeight`); the active index is the last reached step.
- Map step i to panel region i (`offsetTop`, `offsetHeight`). One absolutely positioned highlight element moves to it with `translate(0, top−6px) scaleY((h+12)/100)` (base height 100 px, origin top), transition .6 s `cubic-bezier(.76,0,.24,1)`; it fades in when the first step is reached.
- Step badges flip to filled (transition .4 s). On a later step, a group of bars animates `scaleY .06 → 1` with a 35 ms per-bar stagger, .8 s `cubic-bezier(.25,1,.5,1)`.
- Below 900 px, drop the sticky panel and trigger on the panel regions themselves.

### R7. Deal-in grid — from #20
- Each card gets start offsets that point toward the grid's centre column, so all cards begin roughly stacked in the middle: for 4 columns, x = +23 vw, +7.7 vw, −7.7 vw, −23 vw; y = ±12% + 80 px; rotation −7°, −4°, +5°, +8°.
- Progress per card i over `[gridTop − V + 5vh + 4.06vh·i, gridTop − V + 55.8vh + 4.06vh·i]`, eased out cubic. `transform: translate(calc(var(--x0)*(1-k)), calc(var(--y0)*(1-k))) rotate(calc(var(--r0)*(1-k)))`.
- Two-column mobile: alternate ±33% x and ±12% y.

### R8. Enter primitives + giant wordmark + smooth wheel — from #9–#11, #24, #25
- Enter: cache each `[data-reveal]` element's document Y. On scroll (rAF), add `.is-in` once `y < scrollY + 0.88·innerHeight`, one-shot. CSS then plays:
  - rules: `scaleX 0→1`, 1.1 s, slow in-out.
  - images: `clip-path inset(100% 0 0)→0` (1.1 s `cubic-bezier(.76,0,.24,1)`) together with `scale 1.12→1` (1.4 s `cubic-bezier(.33,1,.67,1)`).
  - word marks: `background-size 0→100%` (.9 s).
  - Stagger siblings with a `--d` of 0, 80 and 160 ms.
- Wordmark: `font-size = (containerWidth) / (chars × k)`, where k is about .445 for a bold grotesk; for our 400 weight, measure k once with a canvas. Footer progress spans from the moment the footer enters until its bottom aligns with the viewport bottom: `translateY(35%→0)`, `scale(1.08→1)` from bottom-left, cubic out.
- Smooth wheel (desktop, `pointer:fine` only):
  - On `wheel` (passive:false) call `preventDefault` and add `deltaY` (×40 for line mode, ×innerHeight for page mode, ×0.95) to `target`, clamped to the scrollable range.
  - In rAF: `current += (target−current)·(1−e^(−5.3·dt))`, dt in seconds, capped at .05; `scrollTo(0,current)`; stop when within .5 px.
  - Bail out for ctrl+wheel, mostly horizontal wheels, and nested scrollable elements that can still scroll. Expose `stop()`/`start()` for modals.

---

## 3. Eva mapping (our sections)

Ground rules for the black/white move:
- Pure `#000` sections for space (hero, solar, Earth link, exploded device, planet reveal) and pure `#fff` for clinical and product (Case 01, Case 02, inventory).
- Transitions between them always use a **clip-path wipe or a sticky cover** (R1/R3), never a gradient.
- Type: 400-weight grotesk at huge sizes; R8 marks highlight key words in one accent colour (suggested: flare orange for alerts, signal cyan for links).
- No buttons, so links use the underline-fill hover.

| Eva section | Recipe(s) | Scene description |
|---|---|---|
| **Hero (cabin)** | R1 + R2 + load-in + ambient drift | Start on white with a square black card (headline "Eva", sub-line). Around it sit 6–8 square chips: chest band, wristband, station bay, Mars, Sun, Earth, a crew portrait silhouette, a vital-sign line. On load the chips wipe open (60 ms stagger) and drift. On scroll the black card expands to the full screen (R1) while the chips fly radially off-screen and tilt (R2). As the black reaches the edges, the **cabin interior** layer fades and scales in behind the headline (scale 1.08→1, opacity 0→1 in the last 30% of R1), and the mission clock "T+212 d" flies to the bottom-right corner (#7). The second panel then slides up with the statement and 3 highlighter words: "chest band", "wristband", "station". |
| **Case 01: helper guided** | R6 | White. Left: 4 steps (alarm on chest band → wristband shows a guided instruction → crewmate arrives → station confirms). Right: a sticky square image of the wristband screen. The highlight band moves over the UI region for each step (alert banner, instruction card, timer, check), and the bars (heart rate) grow on step 3. Region UI can be HTML on top of a blank screen image, so no extra renders are needed. |
| **Case 02: station lights a bay** | R6 variant + R3 sink | White, or black for drama. Sticky square image of the 3-bay station. Each step adds `.lit` to one bay: a glow layer (separate PNG per bay, `mix-blend-mode:screen` on black) fades in at opacity 0→1 over .6 s, with a soft `scale(1.02)` pulse once. The step text says which bay and why. After the last step, the station slab sinks (R3) as the solar section covers it. |
| **Solar alert** | R1 inverse + R4 + particle canvas (our own) + R2 for the spacecraft | Black, pinned for 250 vh. 0–30%: the Sun (a separate layer) enters from the right edge, scale .6→1, with a slow ambient rotation of the corona layer (60 s infinite). 30–55%: the flare layer grows from the limb (clip-path circle radius 0→100% from the flare origin, plus scale 0.8→1). 45–85%: the particle stream travels left toward the spacecraft. Use a **tiny vanilla canvas** of ~400 points moving along a bezier from the flare to the ship, with spawn rate tied to `p` (this is not in the reference; it is our own addition). 70–100%: the spacecraft (a separate layer) gets a red rim light (a duplicate layer at opacity driven by p) and the wristband alert card slides in (`translateY 40px→0`), with the highlighter on "SHELTER NOW". |
| **Earth link** | R5 (pinned horizontal) + R4 | Black, pinned for the horizontal run. The track is a long strip: Earth at the left, Mars at the right, with a dotted path between. The playhead at 50% is "now". As you scroll, the strip moves left and signal packets (small HTML pills: "ECG 06:12", "SpO2 94%", "ACK") fill as they cross the playhead, which makes the delay legible. A counter shows "light-time 3 → 22 min" (JS interpolates the number from p, using the real min/max). Mars's photo de-halftones (R4) when it reaches the centre. |
| **Product inventory** | R3 (stacking slabs) + R7 (deal-in) | White. Three sticky full-screen slabs: chest band, wristband, cabin station. Each has a huge name (400 weight), a square photo with an R4 halftone resolve, and specs that rules draw under. Each slab sinks to 0.9 as the next arrives. Then the **accessory grid** (oximeter, cuff, thermometer, plus three repeated as a 3×2 grid or a row of 4) is dealt in from the centre with R7. |
| **Apple-style "exploded device"** (wristband, then chest band) | New scene built on R2's maths | Black, pinned for 300 vh. Square stage, device centred. 0–15%: the assembled device rotates about 8° and scales .9→1. 15–60%: layers separate along one axis (Apple-style, vertical): each part i has a target offset `y_i` (e.g. −240, −120, 0, +120, +240 px) and a small x drift; `translate(x_i·k, y_i·k) scale(1 − .04·|i−mid|·k)` with in-out cubic easing. Labels draw in with rules (R8) as each part reaches its target (k>.8). 60–80%: hold, with captions changing on step thresholds (R6 logic). 80–100%: reassemble, or let the next panel cover it. Optional pseudo-3D: add `rotateX(55deg)` on the parent while exploding, with `perspective:1200px`, so parts stack like plates. |
| **Planet reveal** (Mars approach) | R1 maths with a circle + scale + R4 | Black, pinned for 200 vh. 0–40%: a pin-point of light at the centre grows. Mars is clipped by `clip-path: circle(r at 50% 50%)` with r from 0 to 60 vmin, and the Mars layer scales 1.4→1 so the surface "settles". 40–70%: the atmosphere rim layer (a separate PNG, `screen`) fades in and the terminator shadow layer slides from right to left (`translateX 20%→0`), as if the planet rotates into light. 50–90%: a huge "MARS" wordmark sits behind (z-index below the planet) and rises 35%→0 as in #24; star layers parallax at 0.2× and 0.5× scroll. 90–100%: the clip circle grows past the viewport, so the planet fills the screen and hands off to the next section. |

---

## 4. Asset needs

General rules:
- Our image model cannot keep 24+ frames consistent, so **no frame sequences anywhere**. Every effect above runs on separate layers moved with transforms, clip-paths and opacity.
- Deliver transparent PNG (or WebP with alpha) when a layer must sit over another; JPG/WebP without alpha for full-bleed backgrounds.
- Square media is our system default: 1600×1600 for full-width squares and 1200×1200 for chips. Full-bleed backgrounds: 2880×1800 (16:10) plus a 1170×2532 crop for mobile if the composition needs it.
- Lighting must be consistent within a scene: same light direction, camera height and lens across all layers of one composition. State this explicitly in every prompt.

| Effect | Assets | Framing / size | Notes |
|---|---|---|---|
| Hero R1+R2 | (a) 6–8 square **chips**: chest band, wristband, station bay, Sun, Earth, Mars, crew silhouette, vitals detail. (b) **Cabin interior** full-bleed background (no people, or 1 crew member with a large negative-space area for the headline). (c) Optional **foreground cabin frame** (window rim / bulkhead edge, transparent) for parallax | chips 1200² JPG/WebP (opaque is fine, since they are clipped squares); cabin 2880×1800 + mobile 1170×2532; foreground 2880×1800 PNG alpha | Chips on pure black or pure white backgrounds so their edges disappear. Supply the cabin as **2 layers** (background structure and mid-ground props) if possible, to allow a 2-depth parallax |
| Case 01 R6 | (a) **Wristband with a blank, dark screen** (front 3/4, big screen facing the camera). (b) Optional: a hand/arm wearing it, separate. (c) Screen UI built in HTML/SVG, not rendered | 1600² PNG on white or with alpha | We overlay UI states in HTML, so one render covers every step. Ask for the screen to be a flat, front-facing quadrilateral (easy to position HTML over it) |
| Case 02 R6 | (a) **Station, all bays dark**. (b) **3 glow-only layers**, one per bay (just the light spill and the lit bay interior, transparent elsewhere, **same camera**). (c) Optional patient silhouette in a bay | 1600² base; glows 1600² PNG alpha, pixel-aligned with the base | The hardest consistency demand. Ask for "same image, only bay N lit". If the model drifts, generate the base once and **paint the glows procedurally** (CSS radial gradients masked to each bay's rect), which needs no extra images |
| Solar alert | (a) **Sun disc** (transparent). (b) **Corona** layer (transparent, rotatable, roughly radially symmetric). (c) **Flare/prominence** layer anchored at one limb point (transparent). (d) **Spacecraft** side view (transparent). (e) Optional red rim-light version of the spacecraft (same pose; or do it with a CSS `drop-shadow` plus `filter: hue-rotate/brightness`). (f) **Star field** bg ×2 densities | Sun 2000² PNG; corona 2400² PNG; flare 1600×1000 PNG with a documented origin point; ship 2000×1000 PNG; stars 2880×1800 JPG ×2 (one sparse, one fine) | Particles are drawn in code (canvas), so no assets are needed for them. The Sun and corona must share a centre |
| Earth link R5 | (a) **Earth** disc (transparent). (b) **Mars** disc (transparent). (c) Optional thin **orbit/path** line as SVG (made in code). (d) Small device icons for packets (SVG, ours) | planets 1600² PNG; render at least 2× display size, since they scale | The halftone (R4) is code. Packets are HTML |
| Inventory R3+R7 | One **hero square** per main device (chest band, wristband, station) and one square each for the **oximeter, cuff, thermometer**, all with the same light direction and backdrop (pure white for this section) | 1600² for the 3 slabs; 1200² for the 3 accessories | The deal-in rotates the cards, so keep a margin of ~8% around each object |
| Exploded device | Per device, **5–7 part layers**, each a transparent PNG rendered from the **same camera and lighting**, plus one **assembled** render for reference. Wristband parts: strap top, strap bottom, case shell, battery/board, sensor puck, screen glass, screen UI. Chest band: strap, electrode pad(s), pod housing, board, battery | each part 1600² PNG on a shared canvas (parts in assembled position, so zero offset = assembled) | **Key requirement: same canvas size and registration for every part.** Then explode offsets are pure translate. If the model cannot do aligned parts, render each part separately centred and assemble by hand-tuned px offsets stored in JSON |
| Planet reveal | (a) **Mars full disc**, high resolution (transparent). (b) **Atmosphere rim** glow ring (transparent). (c) **Terminator shadow** overlay (transparent black gradient shaped to the disc). (d) **Star field** ×2 depths (can reuse solar). (e) Optional **surface close-up** full-bleed for the final fill | Mars 3000² PNG (it scales 1.4→1 and then fills the screen); rim 3000² PNG; shadow 3000² PNG; surface 2880×1800 JPG | The terminator can also be a CSS radial gradient, which saves one asset |
| Global | Wordmark "EVA" set in type, no asset. Duotone or halftone treatments in CSS | — | On black use `mix-blend-mode:screen` for tints, not `multiply` |

**Keyframe images: none required.** The only "states" are the 3 bay-glow layers (Case 02) and the optional rim-lit spacecraft, each a pixel-aligned overlay, not a sequence.

---

## 5. Open points / not confirmed
- The exploded-device and planet-reveal scenes **do not exist on the reference page**. They are designed here from its primitives (sticky pin + scrubbed transforms + clip-path) to meet the "Apple-style" brief.
- The reference's particle-free, canvas-free approach is confirmed. The solar particle stream is our own addition and needs a small canvas.
- Native `animation-timeline` was not tested in Firefox or Safari. The reference turns its scenes off there. Our JS engine path avoids the dependency.
