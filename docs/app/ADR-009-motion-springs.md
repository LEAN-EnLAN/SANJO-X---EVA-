# ADR-009 · Natural motion: springs only, Eva's mascot driven by state

- **Status:** accepted · 15/11/2026 · Claude · details in [MOTION.md](MOTION.md)

## Context

The team lead asked that motion "just works" out of the box, following what Apple's animation engineers describe: motion that imitates physics is easy to learn. All UI motion must be spring-based (mass / stiffness / damping, no linear easing), interruptible and retargetable, continuous with the gesture, and honour `prefers-reduced-motion`. Eva the mascot is the friendliest element and the selling point: it needs state-driven animation (idle breathing, listening, thinking, speaking, happy, concerned, alert), with placeholders until the new simplified art arrives.

## Decision

- **Tokens once, in CSS**: `--spring-{snappy,gentle,bouncy,slow}-{mass,stiffness,damping}` in `styles/app.css`. `static/js/motion.js` reads them, and regenerates the CSS easings `--ease-spring-*` (CSS `linear()` sampled from the same springs) and `--dur-spring-*`; static fallbacks are precomputed from the same numbers. The wireframe's `--ease-eva` cubic-bezier now points at the gentle spring.
- **JS springs** (`EvaMotion.Spring`): position + velocity, one rAF loop, semi-implicit Euler at 240 Hz substeps. `set(target)` keeps the current velocity → interruptible and retargetable. `track(value, dt)` follows a finger 1:1 and remembers velocity; on release the spring starts from that velocity (dev panel drag-to-dismiss).
- **Press feedback** on every `.press` element: scale springs to 0.95 under the finger, back with the bouncy spring.
- **Eva mascot**: `[data-eva]` elements get a controller with five springs (y, scale, rotation, glow, glow scale). Each state has targets and an optional loop that alternates targets so the spring keeps it alive (idle breathing every 2.6 s, speaking pulses, thinking sway). The agent sets `EvaMood`; the server pushes it over SSE (`eva` event) and every mascot on the page retargets. Placeholder art swaps per state (`eva-neutral/wink/happy/care.webp`).
- **Reduced motion**: springs jump to targets, loops stop, CSS transitions drop to 1 ms.

## Consequences

- New art only needs the same file names (or an updated `ART` map in `motion.js`).
- No animation library dependency; ~200 lines of our own JS.
