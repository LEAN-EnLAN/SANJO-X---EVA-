# Eva PWA · development steps and why

One short section per step. Each says what changed and why, so the reasoning lives next to the code.

## Step 1 · Nothing hides behind the tab bar

**What.** Screens reserve `7rem + env(safe-area-inset-bottom)` at the bottom (`pb-dock`). The tab bar floats `0.75rem` above the home indicator (`bottom-tabbar`). Eva's composer and hold-to-talk sit in a sticky dock just above the tab bar (`bottom-dock`), and the chat scrolls underneath it.

**Why.** The old padding was a fixed `pb-32`, smaller than the tab bar plus its gap on a 390 × 844 phone. So the home footer was cut off and the hold-to-talk button sat under the tab bar. The safe-area inset matters once the app is installed: in standalone mode iOS draws the home indicator over the page. On the Eva screen, the person must always be able to reach the microphone without scrolling, so it is docked.

## Step 2 · Dashboard rail and Eva chip

**What.** The left rail is now a full-height column (it stretches with the page) with a sticky inner list. The floating "Eva · Guiding a case" chip moved from `position: fixed` into the greeting row.

**Why.** A `sticky` element with `h-dvh` stops at one viewport height, so on a long page the rail's background ended at about 760 px. The fixed chip covered the roster's battery column at 1440 px. Content that matters for triage must never sit under decoration.

## Step 3 · Tabs follow the wireframe

**What.** The tabs are now Home, Plan, Eva (centre), Food and Logbook. Help moved to a red button in the home header and stays in the emergency banner and the tools sheet. Journal opens from Home, Check-in and Eva.

**Why.** These are the wireframe's five tabs, and they are the places a passenger goes every day. Help still needs only one tap from Home, and an emergency pushes the screen by itself over SSE.

## Step 4 · New screens from the wireframe, fed by live state

**What.** `/app/plan` (day plan with progress, water, vitamin D, sleep, check-in and reminders), `/app/food` (kcal ring, estimated macros, meals with their photos, water glasses, logging a meal), `/app/checkin` (better / same / worse), `/app/ecg` (instructions → 30 s countdown with a live trace → "Regular rhythm" or "Uneven · sent to the crew lead for review", plus "Not continuous. Never decides a shock.") and `/app/band` (the two parts, each battery with time left, suit mode, signal quality).

**Why.** Each screen reads from the same world state the dashboard uses: water, kcal, meals, reminders, journal, vitals, battery and the ECG window. So what the passenger sees matches what the crew lead sees. The ECG window belongs to the server, which ends it after 30 s on the reflex tick. The page only draws the trace and then asks for the result, so a reload or a second phone shows the same state.

## Step 5 · Every safe tool is a button, on the same endpoint

**What.** A tools sheet (grid button on Home, "All tools" chip on Eva) is built at request time from `eva_agent::tools::registry()`. Known tools get a label, an icon and their arguments. A tool added later with no required arguments shows up as a plain button. Every button and form posts to `POST /api/eva/tool`, as the screens' own buttons do (+250 ml, log a meal, start an ECG, reminders, check-in answers, read my band, send to the ground, prepare my report, send for the AED).

**Why.** The person and Eva use one path, so each tap is validated, executed and written to the hash chain with `origin: user`, exactly like a model call. There is no second, unlogged way to change the record. Forms name their arguments `arg.<key>`, and `eva.js` folds them into the `args` JSON. Templates stay plain HTML and the endpoint is unchanged.

## Step 6 · Motion: springs, interruptible, iOS-like

**What.** Screens push in from the right, go back from the left and cross-fade between tabs. All of these motions run on springs that keep their velocity, so a new navigation mid-flight retargets instead of jumping. Swiping from the left edge drags the screen with the finger, and letting go completes the back move or springs it home. The tab pill slides and stretches to the active tab. The tools sheet opens on the gentle spring, follows the finger, rubber-bands upward and closes with the release velocity. Rings and bars fill once when their screen appears. The tab bar's Eva is a live mascot. Tapping Eva makes her hop.

**Why.** ADR-009: motion that behaves like a physical object is easy to learn. With `prefers-reduced-motion`, every spring jumps to its target, the halo and trace draw stop, and the ECG countdown updates once a second.

## Step 7 · Installable and offline

**What.** The service worker cache is now `eva-shell-v3` (bumped whenever CSS or JS change, because static files are served cache-first). It precaches every app screen twice: once as a full page, for a cold start offline, and once as the htmx fragment, for tab navigation offline. It also precaches the hero photos and meal images. Fragment requests are cached separately, so offline navigation can never nest a whole page inside `#screen`. The manifest gained shortcuts (Plan, 30 s ECG) and a fuller description. Icons stay 192 / 512 / 512 maskable, with the mascot inside the maskable safe zone.

**Why.** On the ship there is no store and the hub can drop out. Before this step, offline navigation fell back to the full `/app` page and swapped it into the screen area. Lighthouse 11.7.1 (the last version with the PWA category), headless Chromium, `/app`: **PWA 1.0, accessibility 1.0, best practices 1.0**. After Step 8, `/app`, `/app/plan`, `/app/food`, `/app/ecg` and `/app/band` all score 1.0 on PWA, accessibility and best practices, and `/dash` scores 1.0 on accessibility and best practices with the desktop preset. Card titles are `h2` under each screen's `h1`.

## Step 8 · Contrast and short copy

**What.** Body text that used `text-faint` (#57564f on black, 2.9:1) now uses `text-muted` (#8f8e8a, at least 5.5:1 on every surface). Full-bleed emergency cards use a deeper red (`critical-deep`, #b81d12), so ink text on them is 6:1 instead of 3.1:1. Toasts are solid pills. Dashboard cards use `h2` under the page's `h1`. Long helper paragraphs were cut to one line.

**Why.** WCAG AA asks for 4.5:1 for body text, and passengers read these screens tired, in dim cabins. Short lines are the wireframe's voice, and a person in a hurry reads them.

## Step 9 · Phones, tablets and wide screens

**What.** Checked at 360 × 740, 390 × 844, 430 × 932 and 820 × 1180 for `/app`, and at 1280 × 800, 1440 × 900 and 1920 × 1080 for `/dash`. None has horizontal page scroll. On a tablet the app column widens to 42 rem. Below 380 px, Eva's mascot and greeting shrink so the composer fits. Below 1400 px, the roster's suit column folds into the seat line ("Seat 2A · suit mode"), so the battery column is never clipped.

**Why.** Passengers use their own phones, so the narrowest common Android width is the floor. The crew lead's console is a laptop, so 1280 px is the floor there.

## Step 10 · Emergency tasks are full screen

**What.** When a person's screen carries an emergency task (go help, bring the AED, CPR), the tab bar is hidden and the screen keeps only the safe-area gap at the bottom (`[data-focus]` + CSS `:has()`).

**Why.** At 390 × 844, "I can't" sat under the tab bar on the helper screen. During a cardiac case the helper must see both answers without scrolling, and nothing else should compete for the tap. It works like an iOS full-screen modal. The case banner and the server's `go-home` event bring the tab bar back when the task ends.

## Step 11 · The medical station screen

**What.** `/station` is the screen of the ship's medical station (ADR-011), meant for a tablet or a second laptop mounted as the station, landscape first. On top, the Eva screen strip: the mascot, the current message ("Take the AED · Ana, seat 4B", or "Emergency · Take the AED" when the station acts on its own), the mesh status, **Hub online** or **Standalone**, the backup battery, buffered reports, and the red **Emergency** button. Below, the 12 compartments as on the cabinet: 4 × 3 in landscape (`landscape:` variant), 3 or 2 columns in portrait and on phones, each a rounded card with its number, label, contents, a 3D icon where the kit has one (AED, lungs, thermometer) and our own line icons otherwise, and its item count. A lit compartment glows teal (`--color-station`, #1FE5C4, the cabinet's light strip) and pulses on the bouncy spring; with reduced motion it is a static teal ring. Tapping a compartment simulates "door opened, item taken". The strip and the grid are separate SSE targets, so a status change never restarts the glow.

**Why.** The station is a client of the hub, not a part of the app: it has its own small computer, sensors and screen, and it must keep working when the hub is down. In the demo that computer runs inside `eva` and this page is its screen, so a tablet next to the phones shows the moment compartment 01 lights up for the AED runner. The screen is read from a few metres away in a hurry: one big line of text, one lit door, the red button always in the same corner. Every compartment is a real button with a spoken label ("Compartment 01, AED: lit, take this now, 1 of 1 in"), the message is a live region (assertive during an emergency), and nothing depends on colour alone (the lit card also says "Take now", an open one "Door open", a taken one "Taken"). Checked at 1280 × 800 (no scroll), 800 × 1280 and 390 × 844 (no horizontal scroll). The service worker caches `/station` with the shell (`eva-shell-v6`).

## Step 12 · Wristband v2 renders and the 360° turn

**What.** Every picture of the wristband now shows the v2 design (Codex renders in `static/img/band`, about 2× display size, WebP q82): the home card, the ECG photo, the emergency thumbnails, and on `/app/band` the parts (lifts off, screen part, sensor base, pins), suit mode and a 360° turn built from 8 turntable frames. Drag or swipe turns it on the gentle spring and it settles on the nearest frame; arrows and ←/→ step 45°; with reduced motion it is one still frame plus the arrows. The service worker is `eva-shell-v7`.

**Why.** A two-part band is hard to picture from one photo, and turning it with a finger shows how the parts, straps and pins fit in a second.
