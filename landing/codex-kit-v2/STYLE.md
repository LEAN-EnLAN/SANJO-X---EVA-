# Style v2: "Keynote studio"

Premium product photography, as clean as an Apple keynote. Soft studio light, real materials, lots of empty space. Every asset ships for two page colours, **black `#000000`** and **white `#FFFFFF`**. Its edges must melt into the page, so no box or seam ever shows.

v1's plum, wine and forest stages and its violet light blooms are **retired**. Do not use purple or coloured ambient light anywhere.

## One studio for everything
- **Key light:** large softbox, upper left (10 o'clock, about 45° up), slightly in front of the subject.
- **Black variant:** add strip lights behind the subject for a **crisp white rim** on the right and top edges. No fill light, so shadows fall into pure black. No floor line, no haze, no gradient.
- **White variant:** add a large overhead softbox and keep the image high-key. Use **only a soft contact shadow** directly under objects. No cast shadow on the background, no grey floor gradient, no grey corners.
- **Transparent layers:** use the same key from the upper left, with a gentle neutral rim from the right. The layer has no shadow; shadows are separate PNGs in `white/shadows/`.
- Planets and the Sun follow the same rule: **light comes from the left** and the Sun sits off-frame to the left. Particles and flares flow **left to right**, from the Sun toward the ship.

## Palette
- Page colours: `#000000` and `#FFFFFF`, exact.
- Hardware: matte warm white `#F2F1EE`, soft grey `#C9C9CC`, graphite `#2A2A2D`.
- Screens: **`#000000`**, flat, with nothing displayed. The UI is composited later in HTML.
- Eva amber (the "item to use" light): core `#FFB547`, halo `#FFCB77`. On the white page use a more saturated `#FF9F1C`.
- Small accents only: signal blue `#288BFF`, OK green `#47DA84`, alert red `#F64137`.
- Particles on white: deep amber `#C46A00` and graphite `#3A3A3C`. Light particles disappear on white.

## Devices (Eva by WearMeds)
- **Wristband:** a wide forearm band (about 9 cm) with a **large flexible touch screen curved around the forearm**. The active area is about 11 × 6 cm, with the long side along the arm. Thin bezel, one tiny green LED, screen pure black. The design master is `OR0`.
- **Harness:** a slim modular body harness (webbing and rounded white clips). It has a sternum **core module** (ECG and breathing sensors), **two defibrillation pads** in low-profile pockets and a sealed **injectable compartment** (autoinjector, closed, nothing visible). Each item has a tiny **status light**; Eva lights the item to use in amber. The design master is `HX0`.
- The look is believable 2030s consumer-medical design: calm and soft-touch, not military and not sci-fi armour.
- The shared cabin medical station **no longer exists**. Never draw it.

## Composition
- **Scenes:** the subject sits in a pool of light and falls off into the page colour toward all four edges. The outer 6% of the frame is exact page colour (`generate.py` enforces this). Each prompt names the side kept empty for the headline.
- **Layers:** centred, whole object visible, about 8% padding.
- **Sets** (orbit, exploded, glows) share **one camera**, stated in `prompts.json → cameras`. Each part is drawn in its assembled position on a shared canvas, so translate(0,0) gives back the assembled image.
- **Lenses:** 85–100 mm for products, 70 mm for figures, 135 mm for the ship side view. No fisheye and no dramatic perspective.

## People
- Faceless only. In scenes, show people from behind or cropped above the shoulders, with heads turned away. Figures are smooth mannequin-like editorial figures with no facial features.
- v1's S4 showed faces. Do not repeat that.

## Never
Logos, insignia (including NASA's), flags, patches, brands, text, numbers, fake UI, recognizable faces, needles, blood, gore, neon cyberpunk, lens-flare streaks, purple haze, clutter.
