// Extra primitives for the passenger app and wristband screen. Tokens from theme.css only.
import { cn } from "./cn"
import { Progress } from "./index"

const dot = { ok: "bg-ok", warn: "bg-warn", critical: "bg-critical", info: "bg-info", neutral: "bg-faint" }
const textTone = { ok: "text-ok", warn: "text-warn", critical: "text-critical", info: "text-info", ink: "text-ink", muted: "text-muted" }

/** phone header: round buttons in the top corners, optional tracked label in the middle */
export const PhoneTop = ({ left, right, title }) => (
  <div className="flex items-center justify-between gap-3 pt-1">
    {left ?? <span className="size-11" />}
    {title && <p className="text-xs uppercase tracking-label text-muted">{title}</p>}
    {right ?? <span className="size-11" />}
  </div>
)

/** small tracked label used inside phone screens */
export const Label = ({ children, className }) => <p className={cn("text-xs uppercase tracking-label text-muted", className)}>{children}</p>

/** a big number with its unit and a plain read-out underneath */
export const Reading = ({ value, unit, read, size = "md", tone = "muted" }) => (
  <div className="flex flex-col gap-1">
    <p className={cn("tracking-tight text-ink", size === "lg" ? "text-7xl" : size === "sm" ? "text-2xl" : "text-3xl")}>
      {value}{unit && <span className="ml-1 text-sm text-muted">{unit}</span>}
    </p>
    {read && <p className={cn("text-xs", textTone[tone])}>{read}</p>}
  </div>
)

/** round photo (or icon) tile with a label, a read-out and a status dot, for the 3x3 home grid */
export const Tile = ({ src, icon: Icon, label, read, tone = "ok" }) => (
  <div className="flex flex-col items-center gap-2 text-center">
    <span className="relative">
      <span className="grid size-18 place-items-center overflow-hidden rounded-full bg-raised">
        {src ? <img src={src} alt="" className="size-full scale-110 object-cover" /> : <Icon size={26} className="text-ink" />}
      </span>
      <span className={cn("absolute top-0.5 right-0.5 size-3 rounded-full ring-2 ring-background", dot[tone])} />
    </span>
    <span className="flex flex-col">
      <span className="text-sm text-ink">{label}</span>
      <span className="text-xs text-muted">{read}</span>
    </span>
  </div>
)

/** labelled meter: name, value of goal, and a bar whose remainder is hatched */
export const Meter = ({ label, value, goal, unit, tone = "info", read }) => (
  <div className="flex flex-col gap-2">
    <div className="flex items-baseline justify-between gap-2">
      <p className="text-sm text-ink">{label}</p>
      <p className="text-xs text-muted"><span className="text-ink">{value}</span> / {goal} {unit}</p>
    </div>
    <Progress value={Math.min(100, (value / goal) * 100)} tone={tone} marker={false} />
    {read && <p className="text-xs text-muted">{read}</p>}
  </div>
)

/** row of water glasses; full ones filled, the rest hatched */
export const Glasses = ({ filled, total }) => (
  <div className="flex gap-1.5">
    {Array.from({ length: total }, (_, i) => (
      <span key={i} className="flex h-11 flex-1 flex-col justify-end overflow-hidden rounded-t-sm rounded-b-lg border border-line bg-hatch">
        {i < filled && <span className="h-4/5 bg-info" />}
      </span>
    ))}
  </div>
)

/** where a log entry is: saved on board -> sent -> received on Earth */
const syncLabel = ["Saved on board", "Sent to Earth", "Received on Earth"]
const syncFill = ["bg-ink", "bg-info", "bg-ok"]
export const SyncState = ({ step }) => (
  <div className="flex shrink-0 flex-col items-end gap-1.5">
    <div className="flex gap-1">
      {[0, 1, 2].map(i => <span key={i} className={cn("h-1.5 w-4 rounded-full", i <= step ? syncFill[step] : "bg-line")} />)}
    </div>
    <p className="text-xs text-muted">{syncLabel[step]}</p>
  </div>
)

/** steps of a guided sequence as short bars ("step 2 of 4") */
export const StepBars = ({ step, total, tone = "ink" }) => (
  <div className="flex gap-1.5">
    {Array.from({ length: total }, (_, i) => (
      <span key={i} className={cn("h-1.5 flex-1 rounded-full", i < step ? (tone === "critical" ? "bg-critical" : "bg-ink") : "bg-line")} />
    ))}
  </div>
)

/** CPR rhythm target: concentric rings with the push rate in the middle */
export const BeatRing = ({ rate, label = "pushes a minute" }) => (
  <div className="relative grid size-60 place-items-center">
    <span className="absolute inset-0 rounded-full border border-critical/30" />
    <span className="absolute inset-0 animate-ping rounded-full border-2 border-critical/40" />
    <span className="absolute inset-5 rounded-full border-2 border-critical/70" />
    <span className="absolute inset-12 rounded-full bg-critical/15" />
    <div className="relative flex flex-col items-center gap-1">
      <p className="text-6xl tracking-tight text-ink">{rate}</p>
      <p className="text-xs uppercase tracking-label text-muted">{label}</p>
    </div>
  </div>
)

/** numbered step row (route steps, suggestions) */
export const StepRow = ({ n, title, read, done }) => (
  <div className="flex items-start gap-3">
    <span className={cn("grid size-8 shrink-0 place-items-center rounded-full text-sm", done ? "bg-ink text-on-light" : "border border-line text-ink")}>{n}</span>
    <div className="flex flex-col gap-0.5 pt-1">
      <p className="text-sm text-ink">{title}</p>
      {read && <p className="text-xs text-muted">{read}</p>}
    </div>
  </div>
)

/** cross-section of the wristband in suit mode; each layer sits on the same row as its label */
const Layer = ({ graphic, title, read, accent }) => (
  <div className="flex items-center gap-4">
    <div className="flex w-28 shrink-0 flex-col items-center">{graphic}</div>
    <p className={cn("text-sm", accent ? "text-info" : "text-ink")}>{title}<span className="block text-xs text-muted">{read}</span></p>
  </div>
)
export const SuitDiagram = () => (
  <div className="flex flex-col gap-3">
    <Layer title="Screen part" read="ECG pads, battery, LoRa" graphic={
      <span className="flex h-10 w-full items-center justify-center rounded-tile border border-line bg-elevated"><span className="h-4 w-16 rounded-sm bg-background" /></span>
    } />
    <Layer accent title="Suit fabric" read="Magnets hold, pins don't" graphic={<>
      <span className="flex w-20 justify-between"><span className="h-3 w-0.5 bg-muted" /><span className="h-3 w-0.5 bg-muted" /><span className="h-3 w-0.5 bg-muted" /></span>
      <span className="h-5 w-full rounded-sm bg-hatch-info" />
    </>} />
    <Layer title="Sensor base" read="Own battery and LoRa" graphic={
      <span className="flex h-7 w-24 items-center justify-center gap-1.5 rounded-full border border-line bg-raised"><span className="size-2 rounded-full bg-muted" /><span className="size-2 rounded-full bg-muted" /></span>
    } />
    <Layer title="Your skin" read="Light reads your pulse" graphic={<span className="h-2 w-full rounded-full bg-faint" />} />
  </div>
)

/** legend chip for a chart band or series */
export const Legend = ({ swatch = "band", children }) => (
  <span className="inline-flex items-center gap-2 text-xs text-muted">
    <span className={cn("h-2.5 w-4 rounded-sm", swatch === "band" ? "bg-ok/20" : swatch === "hatch" ? "bg-hatch" : "bg-info")} />
    {children}
  </span>
)

/** photo hero on black with the visible "AI-generated" label the team requires on concept renders.
 *  Pass `baked` when the image already carries its own label, so it is not shown twice. */
export const Hero = ({ src, alt = "", size = "md", fit = "cover", baked, className }) => (
  <div className={cn("relative shrink-0 overflow-hidden rounded-card bg-surface", size === "lg" ? "h-72" : size === "sm" ? "h-32" : "h-48", className)}>
    <img src={src} alt={alt} className={cn("size-full", fit === "contain" ? "object-contain" : "object-cover")} />
    {!baked && <span className="absolute right-3 bottom-3 rounded-full bg-background/70 px-2 py-0.5 text-xs text-muted">AI-generated</span>}
  </div>
)

/** the real wristband: a rugged rectangular module with a flat screen of about 2:1, drawn front-on */
export const BandScreenRect = ({ children, tone = "default", caption, number }) => (
  <figure className="flex flex-col gap-4">
    <div className="relative self-start rounded-card border border-line bg-raised p-4">
      <span className="absolute top-2 left-2 size-1.5 rounded-full bg-faint" />
      <span className="absolute top-2 right-2 size-1.5 rounded-full bg-faint" />
      <span className="absolute bottom-2 left-2 size-1.5 rounded-full bg-faint" />
      <span className="absolute right-2 bottom-2 size-1.5 rounded-full bg-faint" />
      <span className="absolute top-1/2 -right-1.5 h-10 w-1.5 -translate-y-1/2 rounded-r-sm bg-elevated" />
      <div className={cn("flex h-48 w-96 flex-col gap-3 overflow-hidden rounded-tile p-4", tone === "critical" ? "bg-critical text-ink" : "bg-background text-ink")}>{children}</div>
    </div>
    {caption && <figcaption className="flex max-w-md gap-3 text-sm text-muted"><span className="text-ink">{number}</span>{caption}</figcaption>}
  </figure>
)

/** one-lead ECG trace as plain numbers for a Sparkline: P wave, QRS spike, T wave, repeated */
export const ecgTrace = (beats = 4) => {
  const beat = [0, 0, 0.12, 0.18, 0.1, 0, 0, -0.12, 1, -0.3, 0, 0, 0.08, 0.22, 0.28, 0.18, 0.05, 0, 0, 0]
  return Array.from({ length: beats }, () => beat).flat()
}

/** every guided step answers: why this, what comes next, and (optionally) how Eva decided.
 *  Pattern from Russell et al., npj Microgravity 9:46 (2023). Use `onCritical` on a red screen. */
export const WhyNext = ({ why, next, how, onCritical }) => (
  <div className={cn("flex flex-col gap-2 rounded-tile px-4 py-3", onCritical ? "bg-background/15" : "bg-surface")}>
    <div className="flex gap-3 text-sm text-ink">
      <span className={cn("w-10 shrink-0 pt-0.5 text-xs uppercase tracking-label", onCritical ? "text-ink/70" : "text-muted")}>Why</span>
      <span>{why}</span>
    </div>
    <div className="flex gap-3 text-sm text-ink">
      <span className={cn("w-10 shrink-0 pt-0.5 text-xs uppercase tracking-label", onCritical ? "text-ink/70" : "text-muted")}>Next</span>
      <span>{next}</span>
    </div>
    {how && <p className={cn("text-xs", onCritical ? "text-ink/70" : "text-faint")}>How Eva decided · {how}</p>}
  </div>
)

/** where a device is stowed, with the light that helps find it */
export const Stowed = ({ item, place, light, onCritical }) => (
  <p className={cn("flex items-center gap-2 text-xs", onCritical ? "text-ink" : "text-muted")}>
    <span className={cn("size-2 shrink-0 animate-pulse rounded-full", onCritical ? "bg-ink" : "bg-info")} />
    <span><span className="text-ink">{item}:</span> {place} · {light}</span>
  </p>
)
