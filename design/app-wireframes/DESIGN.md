# Eva app · design system

Dark, pure-black UI for two audiences that must both understand it at a glance: passengers with no medical training, and the crew lead / Earth flight surgeon. Visual DNA: the soft rounded cards and big friendly greeting of a consumer health tracker, the KPI-card + chart grid of a hospital operations dashboard, and the restraint of a launch-company interface (black, white, uppercase tracked labels, one accent at a time).

## Rules (enforced by `npm run lint`, @shadcn/lint)
- Colours only from `src/theme.css` tokens: `background surface raised elevated line ink muted faint brand critical warn ok info on-light`. No Tailwind palette (`bg-pink-500`), no hex.
- No arbitrary values (`p-[13px]`), no inline styles. Dynamic sizes use a CSS custom property: `style={{"--pct":"40%"}}` + `w-(--pct)`.
- Screens compose components from `@/ds` and may only add layout and spacing classes to them (`CardTitle`, `Eyebrow`, `Headline` also accept typography).
- No dynamic class strings. Map tones with static lookup objects.

## Meaning of colour
- `critical`: someone may be in danger now. Only for real emergencies.
- `warn`: needs attention soon (battery, sensor quality, dose rising).
- `ok`: normal for this person.
- `info`: Eva is guiding / data / links.
- `brand`: WearMeds identity, used sparingly.
- Hatched fill (`bg-hatch`, `bg-hatch-info`): "remaining", "other", or "not yet".

## Writing
Plain words, one idea per line, verbs first ("Drink a glass of water", "Go to the shelter"). Numbers always with a plain-language read-out next to them ("72 bpm · normal for you").
