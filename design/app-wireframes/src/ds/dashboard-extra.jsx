// Dashboard primitives for the crew and Earth views: app window, icon rail, search, table,
// deck-plan map, dual-clock timeline, pipeline, legend and small meters. Tokens only.
import { cn } from "./cn"
import { Search } from "lucide-react"

const fill = { ok: "bg-ok", warn: "bg-warn", critical: "bg-critical", info: "bg-info", brand: "bg-brand", ink: "bg-ink", faint: "bg-faint" }
const textTone = { ok: "text-ok", warn: "text-warn", critical: "text-critical", info: "text-info", brand: "text-brand", ink: "text-ink", muted: "text-muted" }

/* ---------- app chrome ---------- */

/** desktop app window: rounded, raised border, a slim title bar */
export const AppWindow = ({ title, meta, footer, children, className }) => (
  <div className={cn("w-full overflow-hidden rounded-card border-4 border-raised bg-background ring-1 ring-line", className)}>
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

/* ---------- deck plan ---------- */
const dotFill = { ok: "fill-ok", warn: "fill-warn", critical: "fill-critical", info: "fill-info", neutral: "fill-faint" }
const dotHalo = { ok: "fill-ok/15", warn: "fill-warn/20", critical: "fill-critical/25", info: "fill-info/20", neutral: "fill-faint/20" }

/**
 * top-down deck plan of the ship. Geometry in a 1000x420 viewBox.
 * rooms: [{ x, y, w, h, label, kind: "room" | "shelter" }]
 * people: [{ id, x, y, tone, label, ring }]   hub: { x, y, label }
 */
export const DeckPlan = ({ rooms, people, hub, labels = [] }) => (
  <svg viewBox="0 0 1000 420" className="h-auto w-full" role="img" aria-label="Ship deck plan">
    <defs>
      <pattern id="deck-hatch" width="9" height="9" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="2" height="9" className="fill-info/50" /></pattern>
    </defs>
    {/* hull: a long rounded capsule with a nose */}
    <path d="M 120 40 L 800 40 Q 975 40 975 210 Q 975 380 800 380 L 120 380 Q 25 380 25 210 Q 25 40 120 40 Z" className="fill-surface stroke-line" strokeWidth="2" />
    <path d="M 70 210 L 940 210" className="stroke-line" strokeWidth="1" strokeDasharray="4 8" />
    {rooms.map(r => (
      <g key={r.label}>
        <rect x={r.x} y={r.y} width={r.w} height={r.h} rx="18" fill={r.kind === "shelter" ? "url(#deck-hatch)" : undefined} className={r.kind === "shelter" ? "stroke-info/60" : "fill-raised stroke-line"} strokeWidth="1.5" />
        <text x={r.x + 16} y={r.y + 32} className={r.kind === "shelter" ? "fill-info text-xl" : "fill-muted text-xl"}>{r.label}</text>
      </g>
    ))}
    {/* faint LoRa mesh lines from each wearable to the hub */}
    {people.map(p => <line key={`m-${p.id}`} x1={hub.x} y1={hub.y} x2={p.x} y2={p.y} className={p.tone === "critical" ? "stroke-critical/40" : "stroke-ink/10"} strokeWidth="1" strokeDasharray={p.tone === "critical" ? undefined : "2 5"} />)}
    {/* hub */}
    <g>
      <circle cx={hub.x} cy={hub.y} r="26" className="fill-ink/10" />
      <rect x={hub.x - 13} y={hub.y - 13} width="26" height="26" rx="7" className="fill-ink" />
      <circle cx={hub.x} cy={hub.y} r="4" className="fill-on-light" />
      <text x={hub.x} y={hub.y + 56} textAnchor="middle" className="fill-ink text-xl">{hub.label}</text>
    </g>
    {people.map(p => (
      <g key={p.id}>
        {(p.tone === "critical" || p.tone === "info") && <circle cx={p.x} cy={p.y} r={p.tone === "critical" ? 24 : 18} className={dotHalo[p.tone]} />}
        {p.tone === "critical" && <circle cx={p.x} cy={p.y} r="15" className="animate-pulse fill-critical/30" />}
        <circle cx={p.x} cy={p.y} r={p.tone === "ok" ? 7 : 9} className={cn(dotFill[p.tone], "stroke-surface")} strokeWidth="3" />
      </g>
    ))}
    {labels.map(l => (
      <g key={l.text}>
        <rect x={l.x} y={l.y - 21} width={l.w} height="34" rx="17" className={l.tone === "critical" ? "fill-critical" : "fill-ink"} />
        <text x={l.x + l.w / 2} y={l.y + 3} textAnchor="middle" className={l.tone === "critical" ? "fill-ink text-xl" : "fill-on-light text-xl"}>{l.text}</text>
      </g>
    ))}
  </svg>
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
