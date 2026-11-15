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

`motion.js` reads these numbers at load and writes `--ease-spring-<name>` (a CSS `linear()` curve sampled from the spring) and `--dur-spring-<name>`. Fallback curves in the CSS are precomputed from the same numbers. **Never use `ease`, `linear` or cubic-bezier timing in new CSS**: use `var(--dur-spring-gentle) var(--ease-spring-gentle)`.

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

## Reduced motion

`prefers-reduced-motion: reduce` → springs jump straight to their targets, Eva's loops stop, CSS spring transitions drop to 1 ms, the CPR metronome and orb breathing stop (wireframe rules).

## Navigation and sheets (`motion.js`, second half)

| move | spring | detail |
|---|---|---|
| push (open a detail screen) | gentle | screen enters from +30% width, opacity from 0.3 |
| back (header back, edge swipe, browser back) | gentle | enters from −25% width |
| tab switch | snappy | cross-fade, scale 0.985 → 1 (iOS tabs do not slide) |
| refresh after a tool | none | the screen redraws in place |
| edge swipe | tracks the finger | strip on the left 16 px; release past 33% or faster than 700 px/s completes the back move, otherwise springs home (bouncy) |
| tab pill | gentle (x), bouncy (stretch) | slides under the active tab, stretches 1.25× and settles |
| tools sheet | gentle open, snappy close | follows the finger down 1:1, rubber-bands up; closes past 30% or above 160 %/s |
| rings and bars on enter | gentle | `[data-grow]` springs `--g` from 0 to 1 once |

Links say what kind of move they are with `data-nav="push|back|tab|fade"` (the `go` macro in `templates/app/_ui.html` sets it). Scripts can call `EvaMotion.expect("refresh")` before swapping `#screen` to keep it still.
