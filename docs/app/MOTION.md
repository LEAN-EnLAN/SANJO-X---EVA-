# Motion in Eva

Every move in the app is a **spring**: it has mass, stiffness and damping, it can be interrupted and sent somewhere else mid-flight without a jump, it follows your finger while you touch it, and it stops when you ask your device for reduced motion. Why: motion that behaves like physical objects is easy to learn and feels calm (ADR-009).

## Tokens (defined once)

In `app/styles/app.css`, `:root`:

| token | mass | stiffness | damping | settles in | use |
|---|---|---|---|---|---|
| `snappy` | 1 | 400 | 30 | ~450 ms | presses, small toggles, panel close |
| `gentle` | 1 | 170 | 26 | ~725 ms | default: bars, screens, panel open, chat bubbles (`animate-eva-rise`) |
| `bouncy` | 1 | 300 | 14 | ~960 ms | release after a press, Eva's happy hop, toasts |
| `slow` | 1 | 90 | 20 | ~1.1 s | Eva's idle breathing, concerned state |
| `swift` | 1 | 520 | 40 | ~420 ms | page push / back / tab, shared-element morph, emergency takeover, stagger |
| `glide` | 1 | 60 | 16 | ~1.3 s | band spin coast (`{stiffness 60, damping 16}`), soft returns |

`motion.js` reads these numbers at load and writes `--ease-spring-<name>` (a CSS `linear()` curve sampled from the spring) and `--dur-spring-<name>`. Fallback curves in the CSS are precomputed from the same numbers.

**The rule: springs for anything that moves in response to the user; fixed curves only for ambient loops and opacity fades.** Loops (aurora drift, breathing, the heartbeat, the shimmer, spinners) use `ease-in-out` or `linear`; fades use `--ease-fade` (`cubic-bezier(.2, 0, 0, 1)`, 180–260 ms). Everything else: `var(--dur-spring-gentle) var(--ease-spring-gentle)` or a JS spring.

Utilities: `.spring-w` / `.spring-h` (width/height glide, e.g. battery bars), `.press` (spring press feedback), `transition-eva` (wireframe utility, now springy).

## JavaScript API (`window.EvaMotion`)

```js
const s = EvaMotion.varSpring(el, "--x", 0, "gentle"); // drives a CSS custom property
s.set(100);                    // animate; calling set() again mid-flight retargets, keeping velocity
s.track(fingerX, dt);          // follow a gesture 1:1, remembers velocity
s.set(0, "bouncy");            // release: continues from the finger's velocity
EvaMotion.setMood("thinking"); // every Eva mascot on the page
```

## Eva's states

Driven by the agent (`EvaMood` in `model.rs`), pushed over SSE as the `eva` event; a few short ones come from the page itself (eva.js). Every state has its own image from the Codex kit (`static/img/eva/eva-<state>.webp`, 512 px, AI-generated). A change of state cross-fades the old and new image on the gentle spring and squashes the body (scale 1.07 × 0.90) for the bouncy spring to release. The tab-bar avatar is `eva-face` (`data-eva-fixed`): it moves with the state but keeps its image.

| state | when | set by | motion |
|---|---|---|---|
| idle | nothing happening | default | slow breathing: rises 3 px and grows 2.5% every 2.6 s |
| hello | Eva's screen (or About) opens | eva.js, 1.6 s | bouncy hop, settles |
| listening | recording your voice | eva.js on the mic, `/api/eva/stt` | leans in, glow pulses |
| thinking | waiting for the model (chat, photo, video) | agent, `/api/eva/vision` | sways ±6° |
| speaking | answer shown or spoken (TTS playing) | agent, `/api/eva/tts`, eva.js on audio play | quick small pulses |
| happy | a tool succeeded | `mood_after_tool`, 2.5 s | hop 10 px, settles |
| wink | a tap on Eva | eva.js, 1.4 s | small tilted hop |
| concerned | something to watch, or a tool failed | agent | sinks 2 px, warm glow |
| alert | emergency (case opened, solar storm, AED runner) | agent | fast pulse, red glow |
| calm | the CPR helper is pushing | `helper_answer` (CPR) | slow, steady glow |
| sleeping | ship night 22:30–06:30 and otherwise idle | `World::mood` | very slow breathing, dim violet glow |
| celebrate | a daily goal is met (2 L of water) | `mood_after_tool`, 4 s | bouncy jumps |
| care | check-in, "I feel worse" | check-in screen, `mood_after_tool` | slow lean, mint glow |
| reading | showing records or a summary | records screen, `daily_summary` / `show_screen`, 3.5 s | gentle page-turn sway |
| writing | journaling | journal screen, eva.js while the text box has focus | small writing wiggle |
| sending | uplink: send to the ground, report, outbox replay | `mood_after_tool` (3.5 s), eva.js on replay | quick lift, glow pulses |
| low-battery | the band's base or screen part is under 20 % and otherwise idle | `World::mood` | sinks, dim warm glow |

Short states (`EvaMood::transient`) fall back to idle by themselves (`state::pulse_mood`), unless something else changed Eva's state meanwhile. Emergency states are never overridden by a tap or a hello.

## fx.js: the "alive" layer (APP-POLISH §3, §4)

`static/js/fx.js` runs on top of `motion.js` and registers its per-frame work with `EvaMotion.onFrame` (one rAF loop for the whole app, never a second one).

| effect | how | notes |
|---|---|---|
| heartbeat ring | SVG ring, a `lub-dub` keyframe on `transform` with period `--beat` (60 000 / bpm), two pulse-wave rings | the vitals redraw every second over SSE; fx.js gives each new node a negative `animation-delay` from one global clock, so the beat continues instead of restarting |
| rolling counters | `[data-count]` is split into digit columns that spring to `translateY(-digit em)` (snappy, 30 ms stagger from the right), then flash the domain colour | previous values live in a map keyed by the counter's name, because htmx replaces the nodes; the plain value stays in `aria-label` |
| Eva's aurora | three radial-gradient blobs, `mix-blend-mode: screen`, drifting 18/23/29 s; colours per state via registered custom properties that cross-fade 600 ms | no blur, no canvas; static in lite mode and on the wristband (two blobs) |
| stagger | top-level blocks land with `swift` (16 px rise, scale .98, 45 ms apart, max 7) on push/tab, not on refresh | reads every position first, then writes |
| ripple | one reused radial ink per button / row, scale 0 → 2.2 | |
| morph | same-document View Transitions from a vital tile to its detail (`vital-hero`, `vital-value`) | only with `startViewTransition`, not in lite mode, not reduced; otherwise the spring push |
| burst | one canvas, 36 particles (12 for a mini burst), 900 ms, removed after | goals, + water, journal saved |
| skeleton | a shimmer template only if a screen takes > 160 ms | |
| takeover | a 200 vmax red disc scales in from the top (`swift`, 320 ms; 240 ms on the wristband), two shock rings, then it fades into the screen's own red | storm: amber; theme-color follows |

### Haptics map (`EvaFx.haptic(name)`, no-op without `navigator.vibrate`)

| name | pattern | used for |
|---|---|---|
| tick | 4 | tab change, band detent, wristband page change, ECG every 10 s |
| tap | 8 | buttons, mic start |
| success | [12, 30, 12] | goal reached, I'm going / I have it, ECG done, Eva back online |
| warn | [40, 60, 40] | storm, station guidance, bystander |
| emergency | [120, 60, 120, 60, 400] | takeover arrival (also the LoRa alarm in `link.js`) |
| beat | 30 | one per CPR compression, locked to the metronome's `animationiteration` |
| hold | 20 | every 400 ms while holding Help on the wristband |
| wake | [300, 150, 300, 150, 300] | patient alert on the wristband (then 80 every 10 s) |

### Lite mode and performance

At first load fx.js samples 60 frames; more than 12 slower than 24 ms (or ≤ 4 cores and ≤ 2 GB) sets `html.lite`, saved in `localStorage`. About → Smooth mode switches Auto / Full / Lite. Lite removes the tab bar's `backdrop-filter` (the only one in the app), the aurora drift, the pulse waves and view transitions, and halves burst particles. Infinite loops pause when off screen (one IntersectionObserver) or when the page is hidden.

Measured with Playwright + Chrome, 390 × 844, **4× CPU throttle** (`scratchpad/polish/fps.js`), rAF deltas:

| flow | median | p95 | frames > 50 ms | long tasks > 50 ms |
|---|---|---|---|---|
| home idle 5 s (heart ring + aurora) | 16.7 ms | 16.8 ms | 0 | 0 |
| tools sheet open/close ×3 | 16.7 ms | 16.8 ms | 0 | 0 |
| band drag and flick | 16.7 ms | 16.7 ms | 0 | 0 |
| emergency takeover | 16.7 ms | 16.8 ms | 1 | 1 |
| wristband paging ×4 (640 × 360) | 16.7 ms | 16.8 ms | 0 | 0 |
| tab switches ×8 | 16.7 ms | 33.3 ms | ~1 per switch | ~1 per switch |
| pulse detail and back ×4 | 16.7 ms | 33.3 ms | ~1 per move | ~1 per move |

Every animation holds 60 fps. The one long task per navigation is the swap itself (htmx parsing a whole screen, mounting Eva and the counters) at a quarter of the CPU, before the spring starts; the move that follows runs at 16.7 ms. The measurement caught one real cost and it was fixed: the sheet used to scale the app behind it, which restyled the page every frame (median 33 ms); it keeps only its backdrop now.

## Reduced motion

`prefers-reduced-motion: reduce` → springs jump straight to their targets, Eva's loops stop, CSS spring transitions drop to 1 ms, the CPR metronome and orb breathing stop (wireframe rules). v2: no stagger translate, no morph, no heartbeat or pulse waves (the ECG ribbon is static), static aurora, counters swap instantly, the burst is replaced by Eva's celebrate state, the takeover is an instant red screen, the band's auto-turn is off, the wristband Help hold counts "2… 1…" instead of filling a ring. Haptics stay (they are not motion).

## Navigation and sheets (`motion.js`, second half)

| move | spring | detail |
|---|---|---|
| push (open a detail screen) | swift | screen enters from +28% width, opacity from 0 |
| back (header back, edge swipe, browser back) | swift | enters from −18% width, opacity from 0 |
| tab switch | swift | cross-fade with an 8 px rise (iOS tabs do not slide) |
| morph (vital tile → detail) | view transition, 420 ms on the swift curve | the screen itself stays still |
| refresh after a tool | none | the screen redraws in place |
| edge swipe | tracks the finger | strip on the left 16 px; release past 33% or faster than 700 px/s completes the back move, otherwise springs home (bouncy) |
| tab pill | gentle (x), bouncy (stretch) | a capsule tinted per tab slides under the active icon, stretches 1.35× and settles; the filled icon pops .8 → 1 and a 6 ms tick vibrates |
| band 360° | tracks the finger, then `{60, 16}` | a flick coasts up to ¾ of a turn (`v × 0.35`, max 6 frames) and eases onto a frame; a 4 ms detent per frame while dragging; one slow auto-turn on first view |
| tools sheet | gentle open, snappy close | follows the finger down 1:1, rubber-bands up; closes past 30% or above 160 %/s; navy backdrop |
| wristband pages | native scroll-snap | compositor-driven, zero JS per frame; dots and haptics from one IntersectionObserver |
| rings and bars on enter | gentle | `[data-grow]` springs `--g` from 0 to 1 once |

Links say what kind of move they are with `data-nav="push|back|tab|fade"` (the `go` macro in `templates/app/_ui.html` sets it). Scripts can call `EvaMotion.expect("refresh")` before swapping `#screen` to keep it still.
