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

Driven by the agent (`EvaMood` in `model.rs`), pushed over SSE as the `eva` event.

| state | when | motion (placeholder) | art |
|---|---|---|---|
| idle | nothing happening | slow breathing: rises 3 px and grows 2.5% every 2.6 s | neutral |
| listening | recording your voice | leans in (scale 1.06, tilt −4°), glow pulses | wink |
| thinking | waiting for the model / tools | sways ±6° | neutral |
| speaking | answer shown or spoken | quick small pulses | happy |
| happy | a tool succeeded | hop up 10 px with the bouncy spring, settles | happy |
| concerned | something to watch | sinks 2 px, smaller, warm glow | care |
| alert | emergency | fast pulse, big red glow | care |

New simplified art is being commissioned: keep the file names in `static/img/eva/` or update `ART` in `motion.js`.

## Reduced motion

`prefers-reduced-motion: reduce` → springs jump straight to their targets, Eva's loops stop, CSS spring transitions drop to 1 ms, the CPR metronome and orb breathing stop (wireframe rules).
