// Primitives for Eva, the health copilot (passenger side), and for journaling. Tokens from theme.css only.
import { cn } from "./cn"
import { Play } from "lucide-react"

/* ---------- the mascot ---------- */
/** Eva's plush mascot as a sticker on black. mood: neutral | wink | happy | care | snack.
 *  AI-generated: pass `labelled` where it appears large; small avatars are covered by the page-level note. */
const mascotSize = { xs: "size-8", sm: "size-10", md: "size-14", lg: "size-24", xl: "size-44", hero: "size-56" }
export const EvaMascot = ({ mood = "neutral", size = "md", labelled, glow, alt = "Eva, the health copilot", className }) => (
  <span className={cn("relative inline-grid shrink-0 place-items-center", mascotSize[size], className)}>
    {glow && <span className="absolute -inset-1/4 rounded-full bg-eva-glow" />}
    <img src={`img/eva/eva-${mood}.webp`} alt={alt} className="relative size-full object-contain" />
    {labelled && <span className="absolute -right-2 bottom-1 rounded-full bg-background/80 px-2 py-0.5 text-xs text-muted">AI-generated</span>}
  </span>
)

/** Eva's eyes alone, for the wristband screen: two cyan ovals on the black visor. mode: open | happy */
const eyeSize = { sm: "h-3 w-5", md: "h-5 w-8", lg: "h-7 w-11" }
const eyeGap = { sm: "gap-2 px-3 py-2", md: "gap-3 px-4 py-3", lg: "gap-4 px-5 py-4" }
export const EvaEyes = ({ mode = "open", size = "md" }) => (
  <span className={cn("inline-flex shrink-0 items-center rounded-full bg-surface ring-1 ring-line", eyeGap[size])}>
    {[0, 1].map(i => mode === "happy"
      ? <span key={i} className={cn("rounded-t-full border-4 border-b-0 border-info", eyeSize[size])} />
      : <span key={i} className={cn("rounded-full bg-info shadow-lg shadow-info/40", eyeSize[size])} />)}
  </span>
)

/* ---------- conversation ---------- */
/** one chat bubble. from: eva | me. Eva's bubbles carry her small avatar. */
export const Bubble = ({ from = "eva", mood = "neutral", children, className }) => from === "eva" ? (
  <div className={cn("flex max-w-full items-end gap-2 self-start", className)}>
    <EvaMascot mood={mood} size="xs" alt="Eva" />
    <div className="flex min-w-0 flex-col gap-2 rounded-tile rounded-bl-sm bg-raised px-3.5 py-2.5 text-sm leading-snug text-ink">{children}</div>
  </div>
) : (
  <div className={cn("flex max-w-64 flex-col gap-2 self-end rounded-tile rounded-br-sm bg-elevated px-3.5 py-2.5 text-sm leading-snug text-ink", className)}>{children}</div>
)

/** a recorded voice message: play button, waveform, length */
const waveTone = { ink: "bg-ink", info: "bg-info", muted: "bg-faint" }
export const Waveform = ({ data, tone = "ink", className }) => (
  <span className={cn("flex h-7 flex-1 items-center gap-0.5", className)} aria-hidden="true">
    {data.map((v, i) => <span key={i} className={cn("h-(--h) w-1 shrink-0 rounded-full", waveTone[tone])} style={{ "--h": `${v}%` }} />)}
  </span>
)
export const VoiceNote = ({ data, length }) => (
  <span className="flex items-center gap-2.5">
    <span className="grid size-8 shrink-0 place-items-center rounded-full bg-ink text-on-light"><Play size={14} className="translate-x-px" /></span>
    <Waveform data={data} />
    <span className="font-mono text-xs text-muted">{length}</span>
  </span>
)

/** an image or a short video the person sends; video shows play + length; the caption sits on the image */
export const MediaBubble = ({ src, alt = "", video, length, caption }) => (
  <div className="relative h-24 w-40 shrink-0 self-end overflow-hidden rounded-tile rounded-br-sm bg-surface ring-1 ring-line">
    <img src={src} alt={alt} className="size-full scale-90 object-contain" />
    {video && <span className="absolute inset-0 grid place-items-center"><span className="grid size-9 place-items-center rounded-full bg-background/70 text-ink"><Play size={15} className="translate-x-px" /></span></span>}
    <span className="absolute inset-x-2 bottom-1.5 flex items-center justify-between gap-2 text-xs">
      <span className="rounded-full bg-background/70 px-2 py-0.5 text-ink">{caption}</span>
      {video && <span className="rounded-full bg-background/70 px-2 py-0.5 font-mono text-ink">{length}</span>}
    </span>
  </div>
)

/** tappable pill: quick asks, emotion labels, "what's affecting you" */
export const Chip = ({ active, icon: Icon, children, size = "md" }) => (
  <span className={cn("inline-flex items-center gap-1.5 rounded-full border", size === "sm" ? "h-8 px-3 text-xs" : "h-10 px-4 text-sm", active ? "border-transparent bg-ink text-on-light" : "border-line bg-surface text-ink")}>
    {Icon && <Icon size={size === "sm" ? 13 : 15} className={active ? "text-on-light" : "text-info"} />}{children}
  </span>
)

/* ---------- diagnostic support ---------- */
/** one likely cause with its probability as a bar. The "other" row is hatched. Values in wireframes are illustrative. */
const probFill = { info: "bg-info", ink: "bg-ink", muted: "bg-faint" }
export const ProbRow = ({ label, pct, tone = "ink", other, lead }) => (
  <div className="flex flex-col gap-1.5">
    <div className="flex items-baseline justify-between gap-3">
      <span className={cn("text-sm", lead ? "text-ink" : "text-muted")}>{label}</span>
      <span className={cn("text-sm tabular-nums", lead ? "text-ink" : "text-muted")}>{pct}%</span>
    </div>
    <div className="h-2.5 w-full overflow-hidden rounded-full bg-raised">
      <div className={cn("h-full w-(--pct) rounded-full", other ? "bg-hatch" : probFill[tone])} style={{ "--pct": `${pct}%` }} />
    </div>
  </div>
)

/** who has seen this record: a step is done (filled), waiting (outline) or not yet (faint) */
const trailDot = { done: "bg-ok text-on-light", wait: "border border-info text-info", todo: "border border-line text-faint" }
export const Trail = ({ steps }) => (
  <ol className="flex items-start">
    {steps.map((s, i) => (
      <li key={s.label} className="flex flex-1 flex-col items-center gap-1.5 text-center">
        <span className="flex w-full items-center">
          <span className={cn("h-px flex-1", i === 0 ? "bg-transparent" : s.state === "todo" ? "bg-line" : "bg-ok/60")} />
          <span className={cn("grid size-6 shrink-0 place-items-center rounded-full text-xs", trailDot[s.state])}>{i + 1}</span>
          <span className={cn("h-px flex-1", i === steps.length - 1 ? "bg-transparent" : steps[i + 1].state === "todo" ? "bg-line" : "bg-ok/60")} />
        </span>
        <span className="text-xs leading-tight text-ink">{s.label}</span>
        {s.time && <span className="font-mono text-xs text-muted">{s.time}</span>}
      </li>
    ))}
  </ol>
)

/* ---------- journaling ---------- */
/** glowing State of Mind orb; valence 1 (very unpleasant) to 5 (very pleasant) sets colour, halo and shape */
const orbTone = { 1: "orb-1", 2: "orb-2", 3: "orb-3", 4: "orb-4", 5: "orb-5" }
const orbSize = { dot: "size-3.5 orb-sm", sm: "size-9 orb-sm", md: "size-24", lg: "size-52" }
export const Orb = ({ valence = 3, size = "lg", breathe, className }) => (
  <span className={cn("inline-block shrink-0 orb", orbTone[valence], orbSize[size], breathe && "animate-orb-breathe", className)} aria-hidden="true" />
)
export const moodWord = { 1: "Very unpleasant", 2: "Unpleasant", 3: "Neutral", 4: "Pleasant", 5: "Very pleasant" }

/** the valence slider: one gradient track, a thumb at value (0-100), the two ends named */
export const ValenceSlider = ({ value }) => (
  <div className="flex flex-col gap-2">
    <div className="relative h-3 rounded-full bg-mood-scale">
      <span className="absolute top-1/2 left-(--pct) size-7 -translate-1/2 rounded-full border-4 border-background bg-ink shadow-lg" style={{ "--pct": `${value}%` }} />
    </div>
    <div className="flex justify-between text-xs text-muted"><span>Very unpleasant</span><span>Very pleasant</span></div>
  </div>
)

/** one week of moods: a dot per entry, higher and warmer = more pleasant */
const moodRow = { 1: "row-start-5", 2: "row-start-4", 3: "row-start-3", 4: "row-start-2", 5: "row-start-1" }
export const MoodWeek = ({ days, today }) => (
  <div className="flex flex-col gap-2">
    <div className="relative grid grid-cols-7 gap-x-1">
      <span className="pointer-events-none absolute inset-x-0 top-1/2 border-t border-dashed border-line" />
      {days.map(d => (
        <div key={d.day} className="grid h-16 grid-rows-5 place-items-center">
          {d.moods.map((v, i) => <span key={i} className={cn("col-start-1", moodRow[v])}><Orb valence={v} size="dot" /></span>)}
        </div>
      ))}
    </div>
    <div className="grid grid-cols-7 gap-x-1 text-center text-xs">
      {days.map(d => <span key={d.day} className={d.day === today ? "text-ink" : "text-muted"}>{d.day}</span>)}
    </div>
  </div>
)
