// Dashboard primitives for the crew and Earth views: app window, icon rail, search, table,
// dual-clock timeline, pipeline, legend and small meters. Tokens only.
import { cn } from "./cn"
import { Avatar, StatusDot } from "./index"
import { Search, Magnet, Shirt, Sparkles, Minus, PanelBottom, Maximize2, SendHorizontal } from "lucide-react"

const fill = { ok: "bg-ok", warn: "bg-warn", critical: "bg-critical", info: "bg-info", brand: "bg-brand", ink: "bg-ink", faint: "bg-faint" }
const textTone = { ok: "text-ok", warn: "text-warn", critical: "text-critical", info: "text-info", brand: "text-brand", ink: "text-ink", muted: "text-muted" }

/* ---------- app chrome ---------- */

/** desktop app window: rounded, raised border, a slim title bar */
export const AppWindow = ({ title, meta, footer, children, className }) => (
  <div className={cn("w-full overflow-clip rounded-card border-4 border-raised bg-background ring-1 ring-line", className)}>
    <div className="flex h-10 items-center gap-2 border-b border-line bg-surface px-4">
      <span className="size-3 rounded-full bg-elevated" /><span className="size-3 rounded-full bg-elevated" /><span className="size-3 rounded-full bg-elevated" />
      <span className="ml-4 text-xs text-muted">{title}</span>
      {meta && <span className="ml-auto text-xs text-faint">{meta}</span>}
    </div>
    {children}
    {footer && <div className="flex items-center gap-2 border-t border-line bg-surface px-4 py-2.5 text-xs text-faint">{footer}</div>}
  </div>
)

/** slim vertical icon rail; items: [{ icon, label }] */
export const IconRail = ({ items, active, footer }) => (
  <nav className="flex w-18 shrink-0 flex-col items-center gap-2 border-r border-line bg-surface py-5">
    {items.map(({ icon: I, label, dot }) => (
      <span key={label} aria-label={label} title={label} className={cn("relative grid size-11 place-items-center rounded-full", label === active ? "bg-ink text-on-light" : "text-muted")}>
        <I size={19} />
        {dot && <span className={cn("absolute top-2 right-2 size-2 rounded-full ring-2 ring-surface", fill[dot])} />}
      </span>
    ))}
    <div className="mt-auto flex flex-col items-center gap-2">
      {footer?.map(({ icon: I, label }) => <span key={label} aria-label={label} title={label} className="grid size-11 place-items-center rounded-full text-muted"><I size={19} /></span>)}
    </div>
  </nav>
)

/** product pill: brand mark + name */
export const LogoPill = ({ name = "Eva", by = "by WearMeds" }) => (
  <span className="inline-flex h-11 shrink-0 items-center gap-2 rounded-full bg-ink pr-4 pl-1.5 text-on-light">
    <span className="grid size-8 place-items-center rounded-full bg-on-light"><span className="size-3 rounded-full bg-brand" /></span>
    <span className="text-sm font-semibold tracking-tight">{name}</span>
    <span className="text-xs text-faint">{by}</span>
  </span>
)

/** wide search field (static wireframe) */
export const SearchField = ({ placeholder, shortcut = "⌘K", className }) => (
  <label className={cn("flex h-11 min-w-0 flex-1 items-center gap-3 rounded-full border border-line bg-surface px-4 text-sm text-faint", className)}>
    <Search size={17} className="shrink-0 text-muted" />
    <span className="truncate">{placeholder}</span>
    {shortcut && <span className="ml-auto rounded-full border border-line px-2 py-0.5 text-xs text-faint">{shortcut}</span>}
  </label>
)

/* ---------- table ---------- */
const align = { left: "text-left", right: "text-right", center: "text-center" }
const rowTone = { default: "", critical: "bg-critical/10", info: "bg-info/5", warn: "" }

/** columns: [{ key, label, align }]; rows: [{ id, tone, cells: { key: node } }] */
export const DataTable = ({ columns, rows }) => (
  <table className="w-full border-separate border-spacing-y-1 text-sm">
    <thead>
      <tr>{columns.map(c => <th key={c.key} className={cn("whitespace-nowrap px-2 pb-2 text-xs font-normal uppercase tracking-label text-faint", align[c.align ?? "left"])}>{c.label}</th>)}</tr>
    </thead>
    <tbody>
      {rows.map(r => (
        <tr key={r.id} className={cn(rowTone[r.tone ?? "default"])}>
          {columns.map((c, i) => (
            <td key={c.key} className={cn("whitespace-nowrap px-2 py-1.5 text-ink tabular-nums", align[c.align ?? "left"], i === 0 && "rounded-l-tile", i === columns.length - 1 && "rounded-r-tile")}>{r.cells[c.key]}</td>
          ))}
        </tr>
      ))}
    </tbody>
  </table>
)

/** tiny horizontal meter, e.g. battery; value 0-100 */
export const MiniMeter = ({ value, tone = "ok", label }) => (
  <span className="inline-flex items-center gap-2">
    <span className="relative h-2 w-10 overflow-hidden rounded-full bg-hatch">
      <span className={cn("absolute inset-y-0 left-0 w-(--pct) rounded-full", fill[tone])} style={{ "--pct": `${value}%` }} />
    </span>
    <span className={cn("w-9 text-right text-xs tabular-nums", tone === "warn" ? "text-warn" : "text-muted")}>{label ?? `${value}%`}</span>
  </span>
)

/** a value with its plain read-out underneath */
export const Readout = ({ value, unit, note, tone = "ink" }) => (
  <span className="inline-flex flex-col leading-tight">
    <span className={cn("text-sm tabular-nums", textTone[tone])}>{value}{unit && <span className="ml-0.5 text-xs text-muted">{unit}</span>}</span>
    {note && <span className="text-xs text-faint">{note}</span>}
  </span>
)

/** legend: items [{ label, tone, hatched, value }] */
export const Legend = ({ items, vertical }) => (
  <ul className={cn("flex gap-x-5 gap-y-2 text-xs text-muted", vertical ? "flex-col" : "flex-wrap")}>
    {items.map(it => (
      <li key={it.label} className="flex items-center gap-2">
        <span className={cn("size-3 shrink-0 rounded-full", it.hatched ? "bg-hatch-info ring-1 ring-info/40" : fill[it.tone ?? "ink"])} />
        <span>{it.label}</span>
        {it.value != null && <span className="ml-auto pl-2 text-ink tabular-nums">{it.value}</span>}
      </li>
    ))}
  </ul>
)

/* ---------- timeline with two clocks ---------- */
const tlDot = { ok: "bg-ok", warn: "bg-warn", critical: "bg-critical", info: "bg-info", neutral: "bg-faint" }

/** items: [{ ship, earth, title, detail, tone, pending }]; first two columns are clocks */
export const Timeline = ({ items, shipLabel = "Ship time", earthLabel = "Earth received" }) => (
  <div className="flex flex-col">
    <div className="grid grid-cols-12 gap-4 px-3 pb-3 text-xs uppercase tracking-label text-faint">
      <span className="col-span-2">{shipLabel}</span><span className="col-span-3">{earthLabel}</span><span className="col-span-7 pl-7">What happened</span>
    </div>
    <ol className="flex flex-col">
      {items.map((it, i) => (
        <li key={it.ship + it.title} className={cn("grid grid-cols-12 items-start gap-4 rounded-tile px-3 py-3", it.pending ? "bg-hatch" : it.tone === "critical" ? "bg-critical/10" : "")}>
          <span className={cn("col-span-2 font-mono text-sm tabular-nums", it.pending ? "text-muted" : "text-ink")}>{it.ship}</span>
          <span className={cn("col-span-3 font-mono text-sm tabular-nums", it.pending ? "text-muted" : "text-info")}>{it.earth}</span>
          <div className="relative col-span-7 flex gap-4">
            <span className="relative flex w-3 shrink-0 justify-center pt-1.5">
              <span className={cn("size-3 rounded-full ring-4 ring-background", it.pending ? "border border-faint bg-background" : tlDot[it.tone ?? "neutral"])} />
              {i < items.length - 1 && <span className="absolute top-6 -bottom-6 w-px bg-faint" />}
            </span>
            <div className="flex min-w-0 flex-col gap-0.5">
              <p className={cn("text-sm", it.pending ? "text-muted" : "text-ink")}>{it.title}</p>
              {it.detail && <p className="text-xs text-muted">{it.detail}</p>}
            </div>
          </div>
        </li>
      ))}
    </ol>
  </div>
)

/* ---------- pipeline (saved -> sent -> acknowledged) ---------- */
const stepFill = { ink: "bg-ink text-on-light", info: "bg-info text-on-light", soft: "border border-info/40 bg-info/10 text-info", raised: "bg-raised text-ink" }

/** steps: [{ label, count, hint, style: "ink" | "info" | "soft" | "raised" }] */
export const Pipeline = ({ steps }) => (
  <ol className="grid grid-cols-3 gap-2">
    {steps.map((s, i) => (
      <li key={s.label} className="flex flex-col gap-3">
        <div className={cn("relative flex h-20 flex-col justify-between rounded-tile px-4 py-3", stepFill[s.style ?? "raised"])}>
          <span className="text-xs uppercase tracking-label">{s.label}</span>
          <span className="text-2xl tracking-tight tabular-nums">{s.count}</span>
          {i < steps.length - 1 && <span className="absolute top-1/2 -right-2 z-10 grid size-4 -translate-y-1/2 place-items-center rounded-full bg-background text-xs text-muted">›</span>}
        </div>
        <span className="text-xs text-muted">{s.hint}</span>
      </li>
    ))}
  </ol>
)

/** mock text area for a static composer */
export const TextBox = ({ children, placeholder }) => (
  <div className="min-h-28 rounded-tile border border-line bg-surface p-4 text-sm leading-relaxed text-ink">
    {children ?? <span className="text-faint">{placeholder}</span>}
    <span className="ml-0.5 inline-block h-4 w-px translate-y-0.5 animate-pulse bg-info" />
  </div>
)

/** small key/value chip with a leading status dot or image */
export const InfoChip = ({ tone = "ok", src, children }) => (
  <span className="inline-flex h-9 items-center gap-2 rounded-full border border-line bg-surface pr-3.5 pl-1.5 text-xs text-muted">
    {src ? <img src={src} alt="" className="size-6 rounded-full object-cover" /> : <span className="grid size-6 place-items-center"><span className={cn("size-2 rounded-full", fill[tone])} /></span>}
    {children}
  </span>
)

/* ---------- 1-lead ECG strip ---------- */
const ecgStroke = { ok: "stroke-ok", warn: "stroke-warn", critical: "stroke-critical", info: "stroke-info", ink: "stroke-ink" }

/** one beat's shape as [dt seconds, amplitude] pairs: P wave, QRS spike, T wave */
const beatShape = [[0, 0], [0.06, 0.08], [0.1, 0.12], [0.14, 0.08], [0.18, 0], [0.22, 0], [0.24, -0.12], [0.27, 1], [0.3, -0.28], [0.33, 0], [0.42, 0.04], [0.5, 0.22], [0.58, 0.24], [0.66, 0.04], [0.72, 0]]

/**
 * a wrist ECG strip: rr = seconds between beats (uneven = irregular). Gridlines every second,
 * stronger every 5 s with a label. Geometry only; colour comes from the tone lookup.
 */
export const EcgStrip = ({ rr, seconds = 30, tone = "ink", height = 120 }) => {
  const width = 1200, mid = height * 0.62, amp = height * 0.5, x = t => (t / seconds) * width
  const pts = [[0, mid]]
  let t = 0.2, i = 0
  while (t < seconds - 0.75) {
    beatShape.forEach(([dt, a]) => pts.push([x(t + dt), mid - a * amp]))
    t += rr[i % rr.length]; i++
  }
  pts.push([width, mid])
  const ticks = Array.from({ length: seconds + 1 }, (_, s) => s)
  return (
    <svg viewBox={`0 0 ${width} ${height + 22}`} className="h-auto w-full" preserveAspectRatio="none" role="img" aria-label={`${seconds} second ECG strip`}>
      {ticks.map(s => <line key={s} x1={x(s)} x2={x(s)} y1="0" y2={height} className={s % 5 === 0 ? "stroke-line" : "stroke-line/40"} strokeWidth="1" />)}
      <line x1="0" x2={width} y1={mid} y2={mid} className="stroke-line/60" strokeWidth="1" />
      <polyline points={pts.map(p => p.join(",")).join(" ")} fill="none" strokeWidth="2" strokeLinejoin="round" strokeLinecap="round" className={ecgStroke[tone]} />
      {ticks.filter(s => s % 5 === 0).map(s => <text key={s} x={Math.min(x(s) + 4, width - 30)} y={height + 18} className="fill-faint text-sm">{s} s</text>)}
    </svg>
  )
}

/* ---------- passenger roster ---------- */
// one column layout shared by the rich rows and their header, so every value lines up
const rosterCol = {
  person: "min-w-0 flex-1", status: "w-32 shrink-0", pulse: "w-52 shrink-0", spo2: "w-20 shrink-0",
  skin: "w-28 shrink-0", ecg: "w-24 shrink-0", band: "w-24 shrink-0", battery: "w-36 shrink-0",
}
const rosterKeys = ["status", "pulse", "spo2", "skin", "ecg", "band", "battery"]
const wordTone = { muted: "text-muted", ok: "text-ok", warn: "text-warn", critical: "text-critical", info: "text-info", ink: "text-ink" }

/** a group of passengers: dot + uppercase title + count; `columns` labels the rich-row columns on the same line */
export const RosterGroup = ({ tone = "ok", title, count, hint, columns, children }) => (
  <section className="flex flex-col gap-2">
    <header className="flex items-center gap-4 px-4">
      <span className={cn(rosterCol.person, "flex items-center gap-2")}>
        <StatusDot tone={tone} />
        <span className="text-xs uppercase tracking-label text-ink">{title}</span>
        <span className="text-xs text-muted tabular-nums">{count}</span>
        {hint && <span className="truncate text-xs text-muted">· {hint}</span>}
      </span>
      {columns && rosterKeys.map(k => <span key={k} className={cn(rosterCol[k], "text-xs uppercase tracking-label text-muted")}>{columns[k]}</span>)}
    </header>
    {children}
  </section>
)

const rowSkin = { critical: "border-critical/50 bg-critical/10", default: "border-transparent bg-raised" }
const rowRule = { critical: "border-critical/30", default: "border-line" }

/** rich passenger row: person + the shared columns, then one plain line from Eva with optional trailing actions */
export const RosterRow = ({ tone = "default", person, cells, eva, aside }) => (
  <article className={cn("flex flex-col rounded-tile border", rowSkin[tone])}>
    <div className="flex items-center gap-4 px-4 py-3">
      <div className={rosterCol.person}>{person}</div>
      {rosterKeys.map(k => <div key={k} className={rosterCol[k]}>{cells[k]}</div>)}
    </div>
    {eva && (
      <div className={cn("flex items-center gap-3 border-t px-4 py-2.5", rowRule[tone])}>
        <span className="grid size-6 shrink-0 place-items-center rounded-full bg-info/15 text-info"><Sparkles size={13} /></span>
        <p className="min-w-0 flex-1 truncate text-sm text-ink">{eva}</p>
        {aside}
      </div>
    )}
  </article>
)

/** avatar, name and seat */
export const RosterPerson = ({ name, seat, tone = "neutral", size = "md" }) => (
  <span className="flex min-w-0 items-center gap-3">
    <Avatar name={name} tone={tone} size={size} />
    <span className="flex min-w-0 flex-col leading-tight">
      <span className="truncate text-sm text-ink">{name}</span>
      <span className="text-xs text-muted">Seat {seat}</span>
    </span>
  </span>
)

/** a number with its plain word underneath */
export const Metric = ({ value, unit, word, tone = "ink", wordTone: wt = "muted", mono }) => (
  <span className="flex flex-col gap-0.5 leading-tight">
    <span className={cn("tabular-nums", mono ? "font-mono text-sm" : "text-base", textTone[tone])}>{value}{unit && <span className="ml-1 text-xs text-muted">{unit}</span>}</span>
    {word && <span className={cn("truncate text-xs", wordTone[wt])}>{word}</span>}
  </span>
)

/** pulse read-out + a tiny trend with the person's usual range shaded */
export const PulseTrend = ({ value, unit = "bpm", word, tone = "ink", wordTone: wt = "muted", data, usual, line = "info" }) => (
  <span className="flex items-center gap-3">
    <span className="w-24 shrink-0"><Metric value={value} unit={unit} word={word} tone={tone} wordTone={wt} /></span>
    <span className="w-24 shrink-0"><TrendLine data={data} usual={usual} tone={line} /></span>
  </span>
)

/** band mode as an icon chip */
export const BandMode = ({ mode }) => mode === "suit"
  ? <span className="inline-flex h-7 items-center gap-1.5 rounded-full border border-line bg-elevated px-2.5 text-xs text-ink"><Shirt size={13} />Suit</span>
  : <span className="inline-flex h-7 items-center gap-1.5 rounded-full border border-line px-2.5 text-xs text-muted"><Magnet size={13} />Joined</span>

/** the two parts' batteries as two labelled mini meters */
export const PartBatteries = ({ base, screen, low = 20 }) => (
  <span className="flex flex-col gap-1">
    {[["Base", base], ["Screen", screen]].map(([label, v]) => (
      <span key={label} className="flex items-center gap-2">
        <span className="w-11 text-xs text-muted">{label}</span>
        <MiniMeter value={v} tone={v < low ? "warn" : "ok"} />
      </span>
    ))}
  </span>
)

/** compact card for a passenger who is fine: who, a status dot, pulse and SpO2, one plain line */
export const PassengerChip = ({ name, seat, tone = "ok", hr, spo2, note, noteTone = "muted", mode }) => (
  <article className="flex flex-col gap-3 rounded-tile bg-raised p-3.5">
    <div className="flex items-center gap-2.5">
      <Avatar name={name} size="sm" />
      <span className="flex min-w-0 flex-1 flex-col leading-tight">
        <span className="truncate text-sm text-ink">{name}</span>
        <span className="text-xs text-muted">Seat {seat}</span>
      </span>
      <StatusDot tone={tone} />
    </div>
    <div className="grid grid-cols-2 gap-2">
      <span className="flex flex-col leading-tight"><span className="text-xs text-muted">Pulse</span><span className="text-base text-ink tabular-nums">{hr}<span className="ml-1 text-xs text-muted">bpm</span></span></span>
      <span className="flex flex-col leading-tight"><span className="text-xs text-muted">SpO₂</span><span className="text-base text-ink tabular-nums">{spo2}<span className="ml-0.5 text-xs text-muted">%</span></span></span>
    </div>
    <p className={cn("flex items-center gap-1.5 truncate text-xs", wordTone[noteTone])}>{mode === "suit" && <Shirt size={13} className="shrink-0 text-ink" aria-label="Suit mode" />}{note}</p>
  </article>
)

/* ---------- Eva floating assistant ---------- */
// three predefined positions; every move between them uses the transition-eva utility from theme.css
const dockSize = { minimised: "h-14 w-72", docked: "h-112 w-96", expanded: "h-188 w-112" }
const dockPanel = { minimised: "pointer-events-none opacity-0", docked: "opacity-100", expanded: "opacity-100" }
const dockPill = { minimised: "opacity-100", docked: "pointer-events-none opacity-0", expanded: "pointer-events-none opacity-0" }
const dockPositions = [
  { key: "minimised", label: "Minimise", icon: Minus },
  { key: "docked", label: "Dock", icon: PanelBottom },
  { key: "expanded", label: "Expand", icon: Maximize2 },
]
const posButton = { on: "bg-ink text-on-light", off: "text-muted hover:text-ink" }

/** Eva's orb: the assistant's face */
export const EvaOrb = ({ size = "md" }) => (
  <span className={cn("relative grid shrink-0 place-items-center rounded-full bg-info text-on-light", size === "sm" ? "size-8" : "size-10")}>
    <span className="absolute inset-0 animate-ping rounded-full bg-info/30 motion-reduce:animate-none" />
    <Sparkles size={size === "sm" ? 15 : 18} className="relative" />
  </span>
)

/**
 * floating assistant anchored bottom-right. state: minimised | docked | expanded.
 * title / subtitle head the panel; pinned stays under the header; children are the conversation (newest last); replies are quick chips; pill is the minimised label.
 */
export const EvaDock = ({ state = "expanded", onState, title = "Eva", subtitle, pill, unread = 0, replies = [], placeholder = "Ask Eva", pinned, children }) => (
  <div className={cn("relative overflow-hidden rounded-card border border-line bg-surface/95 ring-1 ring-info/20 backdrop-blur transition-eva", dockSize[state])}>
    <button type="button" onClick={() => onState?.("docked")} aria-label="Open Eva" className={cn("absolute inset-0 flex items-center gap-3 pr-3 pl-2 text-left transition-eva", dockPill[state])}>
      <EvaOrb size="sm" />
      <span className="flex min-w-0 flex-1 flex-col leading-tight"><span className="text-sm text-ink">{title}</span><span className="truncate text-xs text-muted">{pill}</span></span>
      {unread > 0 && <span className="grid size-6 place-items-center rounded-full bg-critical text-xs text-ink tabular-nums">{unread}</span>}
    </button>
    <div className={cn("absolute inset-0 flex flex-col transition-eva", dockPanel[state])}>
      <header className="flex items-center gap-3 border-b border-line px-4 py-3">
        <EvaOrb />
        <span className="flex min-w-0 flex-1 flex-col leading-tight">
          <span className="flex items-center gap-2 text-sm text-ink">{title}<StatusDot tone="ok" pulse /><span className="text-xs text-muted">live</span></span>
          <span className="truncate text-xs text-muted">{subtitle}</span>
        </span>
        <span className="flex shrink-0 rounded-full bg-raised p-0.5">
          {dockPositions.map(({ key, label, icon: I }) => (
            <button key={key} type="button" aria-label={label} title={label} onClick={() => onState?.(key)} className={cn("grid size-7 place-items-center rounded-full transition-eva", posButton[key === state ? "on" : "off"])}><I size={13} /></button>
          ))}
        </span>
      </header>
      {pinned && <div className="px-4 pt-3">{pinned}</div>}
      <div className="relative flex min-h-0 flex-1 flex-col justify-end gap-2.5 overflow-hidden px-4 py-3">
        {children}
        <span className="pointer-events-none absolute inset-x-0 top-0 h-16 bg-linear-to-b from-surface to-transparent" />
      </div>
      {replies.length > 0 && <div className="flex flex-wrap gap-1.5 px-4 pb-3">{replies}</div>}
      <div className="flex items-center gap-2 border-t border-line px-3 py-3">
        <span className="flex h-10 min-w-0 flex-1 items-center rounded-full border border-line bg-background px-4 text-sm text-faint">{placeholder}</span>
        <span aria-label="Send" className="grid size-10 shrink-0 place-items-center rounded-full bg-ink text-on-light"><SendHorizontal size={16} /></span>
      </div>
    </div>
  </div>
)

const msgLabel = { info: "text-info", critical: "text-critical", muted: "text-muted" }

/** one message in Eva's conversation. from: eva | person */
export const EvaMessage = ({ from = "eva", label, labelTone = "info", name, time, children }) => from === "eva" ? (
  <div className="flex max-w-sm animate-eva-rise flex-col gap-1 self-start rounded-tile bg-raised px-3.5 py-2.5 motion-reduce:animate-none">
    {label && <span className={cn("text-xs uppercase tracking-label", msgLabel[labelTone])}>{label}</span>}
    <p className="text-sm leading-snug text-ink">{children}</p>
    {time && <span className="font-mono text-xs text-muted">{time}</span>}
  </div>
) : (
  <div className="flex max-w-xs animate-eva-rise flex-col items-end gap-1 self-end motion-reduce:animate-none">
    <span className="text-xs text-muted">{name}{time && <span className="ml-2 font-mono text-faint">{time}</span>}</span>
    <p className="rounded-tile bg-elevated px-3.5 py-2.5 text-sm leading-snug text-ink">{children}</p>
  </div>
)

/** quick-reply chip under the conversation */
export const ReplyChip = ({ children, tone = "default" }) => (
  <span className={cn("inline-flex h-8 items-center gap-1.5 rounded-full border px-3 text-xs", tone === "primary" ? "border-transparent bg-ink text-on-light" : "border-line text-ink")}>{children}</span>
)

/** compact trend line with the person's usual range drawn as a clear band */
export const TrendLine = ({ data, usual, tone = "info", width = 96, height = 36 }) => {
  const lo = Math.min(...data, usual?.[0] ?? Infinity), hi = Math.max(...data, usual?.[1] ?? -Infinity), pad = 5
  const y = v => height - pad - ((v - lo) / (hi - lo || 1)) * (height - pad * 2)
  const pts = data.map((v, i) => `${(i / (data.length - 1)) * (width - 5)},${y(v)}`).join(" ")
  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="h-9 w-full" preserveAspectRatio="none" aria-hidden="true">
      {usual && <rect x="0" width={width} y={y(usual[1])} height={y(usual[0]) - y(usual[1])} rx="4" className="fill-ok/15" />}
      <polyline points={pts} fill="none" strokeWidth="2" strokeLinejoin="round" strokeLinecap="round" vectorEffect="non-scaling-stroke" className={ecgStroke[tone]} />
      <circle cx={width - 5} cy={y(data[data.length - 1])} r="3.5" className={cn("fill-current", textTone[tone])} />
    </svg>
  )
}
