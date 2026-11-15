// Eva design system primitives. Screens compose these; they never restyle them beyond layout and spacing.
import { cva } from "class-variance-authority"
import { cn } from "./cn"
import { Check as CheckIcon, ChevronDown } from "lucide-react"

/* ---------- type ---------- */
export const Eyebrow = ({ className, ...p }) => <p className={cn("text-xs uppercase tracking-label text-muted", className)} {...p} />
export const Headline = ({ className, as: As = "h2", ...p }) => <As className={cn("text-display tracking-tight text-ink", className)} {...p} />
export const Body = ({ className, muted, ...p }) => <p className={cn("text-sm leading-snug", muted ? "text-muted" : "text-ink", className)} {...p} />

/* ---------- buttons ---------- */
const button = cva("inline-flex items-center justify-center gap-2 rounded-full font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info", {
  variants: {
    variant: {
      primary: "bg-ink text-on-light hover:bg-muted",
      secondary: "border border-line bg-raised text-ink hover:bg-elevated",
      ghost: "text-ink hover:bg-raised",
      danger: "bg-critical text-ink hover:bg-brand",
      outline: "border border-line text-ink hover:bg-raised",
    },
    size: { sm: "h-8 px-3 text-xs", md: "h-11 px-5 text-sm", lg: "h-14 px-7 text-base", xl: "h-20 px-8 text-xl" },
  },
  defaultVariants: { variant: "primary", size: "md" },
})
export const Button = ({ className, variant, size, ...p }) => <button className={cn(button({ variant, size }), className)} {...p} />

const iconButton = cva("inline-flex shrink-0 items-center justify-center rounded-full transition-colors", {
  variants: {
    tone: { default: "bg-raised text-ink hover:bg-elevated", light: "bg-ink text-on-light", critical: "bg-critical text-ink", ghost: "text-muted hover:text-ink" },
    size: { sm: "size-8", md: "size-11", lg: "size-14", xl: "size-20" },
  },
  defaultVariants: { tone: "default", size: "md" },
})
export const IconButton = ({ className, tone, size, label, children, ...p }) => <button aria-label={label} className={cn(iconButton({ tone, size }), className)} {...p}>{children}</button>

/* ---------- surfaces ---------- */
const card = cva("rounded-card border p-5", {
  variants: {
    tone: { default: "border-line bg-surface", raised: "border-line bg-raised", critical: "border-critical bg-critical/10", info: "border-info/40 bg-info/5", light: "border-transparent bg-ink text-on-light" },
  },
  defaultVariants: { tone: "default" },
})
export const Card = ({ className, tone, ...p }) => <section className={cn(card({ tone }), className)} {...p} />
export const CardHeader = ({ className, ...p }) => <header className={cn("mb-4 flex items-start justify-between gap-3", className)} {...p} />
export const CardTitle = ({ className, ...p }) => <h3 className={cn("text-base text-ink", className)} {...p} />

/* ---------- status ---------- */
const badge = cva("inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-medium", {
  variants: {
    tone: {
      neutral: "border-line text-muted", ok: "border-ok/40 bg-ok/10 text-ok", warn: "border-warn/40 bg-warn/10 text-warn",
      critical: "border-critical/50 bg-critical/15 text-critical", info: "border-info/40 bg-info/10 text-info", brand: "border-brand/40 bg-brand/10 text-brand",
    },
  },
  defaultVariants: { tone: "neutral" },
})
export const Badge = ({ className, tone, ...p }) => <span className={cn(badge({ tone }), className)} {...p} />

const dot = { ok: "bg-ok", warn: "bg-warn", critical: "bg-critical", info: "bg-info", neutral: "bg-faint", brand: "bg-brand" }
export const StatusDot = ({ tone = "ok", className, pulse }) => (
  <span className={cn("relative inline-flex size-2.5 shrink-0 rounded-full", dot[tone], className)}>
    {pulse && <span className={cn("absolute inset-0 animate-ping rounded-full", dot[tone])} />}
  </span>
)

export const Stat = ({ label, value, unit, delta, deltaTone = "ok", hint, className }) => (
  <div className={cn("flex flex-col gap-2", className)}>
    {label && <Eyebrow>{label}</Eyebrow>}
    <p className="text-4xl tracking-tight text-ink">{value}{unit && <span className="ml-1 text-base text-muted">{unit}</span>}</p>
    {(delta || hint) && <p className="flex items-center gap-2 text-xs text-muted">{delta && <Badge tone={deltaTone}>{delta}</Badge>}{hint}</p>}
  </div>
)

/* ---------- data viz (all sizes and positions come from CSS custom properties or SVG geometry) ---------- */
const fill = { ok: "bg-ok", warn: "bg-warn", critical: "bg-critical", info: "bg-info", brand: "bg-brand", ink: "bg-ink" }
const stroke = { ok: "stroke-ok", warn: "stroke-warn", critical: "stroke-critical", info: "stroke-info", brand: "stroke-brand", ink: "stroke-ink", line: "stroke-line" }
const text = { ok: "text-ok", warn: "text-warn", critical: "text-critical", info: "text-info", brand: "text-brand", ink: "text-ink" }

/** filled part in a tone, the rest hatched, with an optional marker line */
export const Progress = ({ value, tone = "info", marker = true, className, size = "md" }) => (
  <div className={cn("relative w-full overflow-hidden rounded-full bg-hatch", size === "lg" ? "h-10" : size === "sm" ? "h-2" : "h-4", className)}>
    <div className={cn("h-full w-(--pct) rounded-full", fill[tone])} style={{ "--pct": `${value}%` }} />
    {marker && <div className="absolute inset-y-0 left-(--pct) w-0.5 bg-ink" style={{ "--pct": `${value}%` }} />}
  </div>
)

/** two-part bar: solid share with label, hatched remainder with label */
export const SplitBar = ({ a, b, aLabel, bLabel, tone = "ink" }) => (
  <div className="flex h-10 w-full gap-1">
    <div className={cn("flex w-(--pct) items-center rounded-full px-4 text-sm", fill[tone], tone === "ink" ? "text-on-light" : "text-on-light")} style={{ "--pct": `${a}%` }}>{aLabel ?? `${a}%`}</div>
    <div className="flex flex-1 items-center justify-end rounded-full bg-hatch-info px-4 text-sm text-ink">{bLabel ?? `${b}%`}</div>
  </div>
)

/** donut ring; value 0-100 */
export const Ring = ({ value, tone = "info", size = 120, thickness = 12, children, className }) => {
  const r = (size - thickness) / 2, c = 2 * Math.PI * r
  return (
    <div className={cn("relative inline-grid place-items-center", className)}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="-rotate-90">
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" strokeWidth={thickness} className="stroke-line" />
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" strokeWidth={thickness} strokeLinecap="round" strokeDasharray={`${(c * value) / 100} ${c}`} className={stroke[tone]} />
      </svg>
      <div className="absolute inset-0 grid place-items-center text-center">{children}</div>
    </div>
  )
}

/** half gauge with segments [{value, tone, hatched}] summing to 100 */
export const HalfGauge = ({ segments, size = 240, thickness = 34 }) => {
  const r = (size - thickness) / 2, c = Math.PI * r
  let acc = 0
  return (
    <svg width={size} height={size / 2 + thickness / 2} viewBox={`0 0 ${size} ${size / 2 + thickness / 2}`}>
      <defs><pattern id="hatch" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="2" height="7" className="fill-info" /></pattern></defs>
      {segments.map((s, i) => {
        const dash = `${(c * s.value) / 100} ${c * 2}`, off = -(c * acc) / 100; acc += s.value
        return <path key={i} d={`M ${thickness / 2} ${size / 2} A ${r} ${r} 0 0 1 ${size - thickness / 2} ${size / 2}`} fill="none" strokeWidth={thickness} strokeLinecap={i === 0 ? "round" : "butt"} strokeDasharray={dash} strokeDashoffset={off} stroke={s.hatched ? "url(#hatch)" : undefined} className={s.hatched ? "" : stroke[s.tone]} />
      })}
    </svg>
  )
}

/** sparkline with the person's own baseline band */
export const Sparkline = ({ data, band, tone = "info", width = 280, height = 72, className }) => {
  const min = Math.min(...data, band?.[0] ?? Infinity), max = Math.max(...data, band?.[1] ?? -Infinity), pad = 6
  const y = v => height - pad - ((v - min) / (max - min || 1)) * (height - pad * 2)
  const pts = data.map((v, i) => `${(i / (data.length - 1)) * width},${y(v)}`).join(" ")
  return (
    <svg viewBox={`0 0 ${width} ${height}`} className={cn("h-auto w-full", className)} preserveAspectRatio="none">
      {band && <rect x="0" width={width} y={y(band[1])} height={y(band[0]) - y(band[1])} className="fill-ok/10" />}
      <polyline points={pts} fill="none" strokeWidth="2.5" strokeLinejoin="round" strokeLinecap="round" className={stroke[tone]} />
      <circle cx={width} cy={y(data[data.length - 1])} r="4" className={cn("fill-current", text[tone])} />
    </svg>
  )
}

/** columns; the highlighted one is solid with a hatched cap and a value tag */
export const Bars = ({ data, labels, highlight, max, tagFormat = v => v }) => {
  const m = max ?? Math.max(...data)
  return (
    <div className="flex h-56 items-end gap-2">
      {data.map((v, i) => (
        <div key={i} className="flex h-full flex-1 flex-col items-center justify-end gap-2">
          <div className="relative flex h-(--h) w-full flex-col overflow-visible rounded-tile bg-raised" style={{ "--h": `${(v / m) * 100}%` }}>
            {i === highlight && <>
              <span className="absolute -top-8 left-1/2 -translate-x-1/2 rounded-full bg-ink px-2 py-0.5 text-xs text-on-light">{tagFormat(v)}</span>
              <div className="h-1/4 rounded-t-tile bg-hatch-info" />
              <div className="flex-1 rounded-b-tile bg-elevated bg-hatch" />
            </>}
          </div>
          <span className={cn("text-xs", i === highlight ? "text-ink" : "text-muted")}>{labels?.[i]}</span>
        </div>
      ))}
    </div>
  )
}

/* ---------- lists & identity ---------- */
const avatarTone = { ok: "bg-ok/15 text-ok", warn: "bg-warn/15 text-warn", critical: "bg-critical/20 text-critical", info: "bg-info/15 text-info", neutral: "bg-elevated text-ink" }
export const Avatar = ({ name, tone = "neutral", size = "md" }) => (
  <span className={cn("inline-grid shrink-0 place-items-center rounded-full font-medium", avatarTone[tone], size === "sm" ? "size-8 text-xs" : size === "lg" ? "size-14 text-lg" : "size-10 text-sm")}>
    {name.split(" ").map(w => w[0]).slice(0, 2).join("")}
  </span>
)
export const ListRow = ({ leading, title, subtitle, trailing, tone = "default", className }) => (
  <div className={cn("flex items-center gap-3 rounded-tile px-3 py-3", tone === "raised" ? "bg-raised" : tone === "critical" ? "bg-critical/10" : "", className)}>
    {leading}
    <div className="min-w-0 flex-1"><p className="truncate text-sm text-ink">{title}</p>{subtitle && <p className="truncate text-xs text-muted">{subtitle}</p>}</div>
    {trailing}
  </div>
)
export const IconTile = ({ src, alt = "", active, size = "md", className, children }) => (
  <span className={cn("inline-grid shrink-0 place-items-center overflow-hidden rounded-full", active ? "bg-elevated ring-1 ring-line" : "bg-raised", size === "sm" ? "size-12" : size === "lg" ? "size-24" : "size-16", className)}>
    {src ? <img src={src} alt={alt} className="size-full object-cover" /> : children}
  </span>
)
export const Check = ({ done }) => (
  <span className={cn("inline-grid size-7 shrink-0 place-items-center rounded-full border", done ? "border-info bg-info text-on-light" : "border-faint")}>{done && <CheckIcon size={16} strokeWidth={3} />}</span>
)
export const PillSelect = ({ children }) => (
  <span className="inline-flex h-8 items-center gap-1 rounded-full border border-line px-3 text-xs text-muted">{children}<ChevronDown size={14} /></span>
)
export const Segmented = ({ items, active }) => (
  <div className="flex rounded-full bg-raised p-1">
    {items.map(it => <span key={it} className={cn("flex-1 rounded-full px-3 py-1.5 text-center text-xs", it === active ? "bg-ink text-on-light" : "text-muted")}>{it}</span>)}
  </div>
)

/* ---------- device frames ---------- */
export const Phone = ({ children, tabbar, caption, number, tone = "default" }) => (
  <figure className="flex flex-col gap-4">
    <div className={cn("relative flex h-200 w-96 flex-col overflow-hidden rounded-device border-8 border-raised", tone === "critical" ? "bg-critical" : "bg-background")}>
      <div className={cn("flex items-center justify-between px-7 pt-4 pb-2 text-sm", tone === "critical" ? "text-ink" : "text-ink")}>
        <span>9:41</span><span className="h-6 w-24 rounded-full bg-background" /><span className="text-xs">LoRa ●</span>
      </div>
      <div className="flex flex-1 flex-col gap-4 overflow-hidden px-5 pb-4">{children}</div>
      {tabbar}
      <div className="mx-auto mb-2 h-1 w-32 rounded-full bg-faint" />
    </div>
    {caption && <figcaption className="flex max-w-96 gap-3 text-sm text-muted"><span className="text-ink">{number}</span>{caption}</figcaption>}
  </figure>
)
export const TabBar = ({ items, active }) => (
  <nav className="mx-5 mb-3 flex items-center justify-around rounded-full bg-raised py-3">
    {items.map(({ icon: I, label }) => <span key={label} aria-label={label} className={cn("grid size-11 place-items-center rounded-full", label === active ? "bg-ink text-on-light" : "text-muted")}><I size={20} /></span>)}
  </nav>
)
/** the wristband's own wide screen (interface part), drawn flat */
export const BandScreen = ({ children, tone = "default", caption, number }) => (
  <figure className="flex flex-col gap-4">
    <div className="rounded-device bg-raised p-3">
      <div className={cn("flex h-44 w-176 items-stretch gap-4 overflow-hidden rounded-card p-5", tone === "critical" ? "bg-critical text-ink" : "bg-background text-ink")}>{children}</div>
    </div>
    {caption && <figcaption className="flex max-w-176 gap-3 text-sm text-muted"><span className="text-ink">{number}</span>{caption}</figcaption>}
  </figure>
)
