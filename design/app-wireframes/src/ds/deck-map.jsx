// Deck map: a rendered floor plan (Blender, tools/deck-base.blender.py) with a live SVG overlay in the same
// coordinates. The overlay is computed, not hand-placed:
//   · LoRa mesh  = Delaunay neighbours of the wristbands and the hub (d3-delaunay), long links dropped
//   · AED route  = smooth curve through the walking points (d3-shape, Catmull-Rom)
//   · callouts   = settled by a small force simulation (d3-force) with a rectangle collision force,
//                  so labels never overlap each other, the dots, or the fixed room labels
// Tokens only: every colour is a fill-* / stroke-* class.
import { useEffect, useMemo, useRef, useState } from "react"
import { Delaunay } from "d3-delaunay"
import { forceSimulation, forceX, forceY } from "d3-force"
import { line, curveCatmullRom } from "d3-shape"
import { cn } from "./cn"

/* ---------- text metrics (Inter, rough but stable) ---------- */
const glyph = ch => (ch === " " ? 0.28 : ch === "·" ? 0.32 : /[A-Z]/.test(ch) ? 0.66 : /[0-9]/.test(ch) ? 0.6 : /[mw]/.test(ch) ? 0.82 : /[ijlt.,']/.test(ch) ? 0.3 : 0.54)
const textWidth = (s, size, tracking = 0) => [...s].reduce((w, ch) => w + glyph(ch) * size + tracking * size, 0)

// type sizes in viewBox units, designed for a map ~700 px wide (26 units ≈ 13 px on screen).
// k > 1 when the map is drawn narrower, so text stays readable; the finest detail drops out instead.
const DESIGN_WIDTH = 700
const typeScale = k => ({ title: 26 * k, note: 21 * k, room: 20 * k, sub: 17 * k, seat: 16 * k, pad: 16 * k, h1: 42 * k, h2: 64 * k, k })

/* ---------- geometry helpers ---------- */
const overlap1d = (a0, a1, b0, b1) => Math.min(a1, b1) - Math.max(a0, b0)

/** collision force for axis-aligned boxes centred on (x, y); pushes along the smaller overlap */
function forceRectCollide(pad = 8) {
  let nodes = []
  const force = alpha => {
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i], b = nodes[j]
        const ox = overlap1d(a.x - a.w / 2 - pad, a.x + a.w / 2 + pad, b.x - b.w / 2, b.x + b.w / 2)
        const oy = overlap1d(a.y - a.h / 2 - pad, a.y + a.h / 2 + pad, b.y - b.h / 2, b.y + b.h / 2)
        if (ox <= 0 || oy <= 0) continue
        const k = alpha
        if (ox < oy) { const s = a.x < b.x ? -1 : 1; a.vx += s * ox * k; b.vx -= s * ox * k }
        else { const s = a.y < b.y ? -1 : 1; a.vy += s * oy * k; b.vy -= s * oy * k }
      }
    }
  }
  force.initialize = n => { nodes = n }
  return force
}

/** keeps boxes off fixed obstacles: circles { x, y, r } and boxes { x, y, w, h } (centre based) */
function forceAvoid(obstacles, pad = 6) {
  let nodes = []
  const force = alpha => {
    for (const n of nodes) {
      for (const o of obstacles) {
        const ow = o.r != null ? o.r * 2 : o.w, oh = o.r != null ? o.r * 2 : o.h
        const ox = overlap1d(n.x - n.w / 2 - pad, n.x + n.w / 2 + pad, o.x - ow / 2, o.x + ow / 2)
        const oy = overlap1d(n.y - n.h / 2 - pad, n.y + n.h / 2 + pad, o.y - oh / 2, o.y + oh / 2)
        if (ox <= 0 || oy <= 0) continue
        if (ox < oy) n.vx += (n.x < o.x ? -1 : 1) * ox * alpha * 2
        else n.vy += (n.y < o.y ? -1 : 1) * oy * alpha * 2
      }
    }
  }
  force.initialize = n => { nodes = n }
  return force
}

const COMPASS = { e: [1, 0], w: [-1, 0], s: [0, 1], n: [0, -1], se: [1, 1], sw: [-1, 1], ne: [1, -1], nw: [-1, -1] }
const boxOverlap = (a, b) => Math.max(0, overlap1d(a.x - a.w / 2, a.x + a.w / 2, b.x - b.w / 2, b.x + b.w / 2)) * Math.max(0, overlap1d(a.y - a.h / 2, a.y + a.h / 2, b.y - b.h / 2, b.y + b.h / 2))
const asBox = o => (o.r != null ? { x: o.x, y: o.y, w: o.r * 2, h: o.r * 2 } : o)

/**
 * greedy start: for each callout (in priority order) try its preferred side, then the other seven,
 * and keep the spot that covers the least of the obstacles and the callouts already placed
 */
function pickSides(items, obstacles, bounds) {
  const placed = []
  return items.map(it => {
    const order = [it.side, ...Object.keys(COMPASS).filter(k => k !== it.side)]
    let best = null
    order.forEach((side, rank) => {
      const [ux, uy] = COMPASS[side]
      const dx = ux * (it.w / 2 + it.gap), dy = uy * (it.h / 2 + it.gap)
      const b = { x: it.ax + dx, y: it.ay + dy, w: it.w, h: it.h }
      const out = Math.max(0, bounds.x - (b.x - b.w / 2)) + Math.max(0, b.x + b.w / 2 - bounds.x - bounds.w) + Math.max(0, bounds.y - (b.y - b.h / 2)) + Math.max(0, b.y + b.h / 2 - bounds.y - bounds.h)
      const cost = obstacles.reduce((c, o) => c + boxOverlap(b, asBox(o)), 0) + placed.reduce((c, p) => c + 3 * boxOverlap(b, p), 0) + out * it.h * 4 + rank * 300
      if (!best || cost < best.cost) best = { cost, dx, dy, b }
    })
    placed.push(best.b)
    return { ...it, dx: best.dx, dy: best.dy }
  })
}

/** settle callouts near their chosen spot; returns boxes with x, y as the centre */
function placeCallouts(items, obstacles, bounds) {
  const nodes = pickSides(items, obstacles, bounds).map(c => ({ ...c, x: c.ax + c.dx, y: c.ay + c.dy }))
  const sim = forceSimulation(nodes)
    .velocityDecay(0.35)
    .force("x", forceX(d => d.ax + d.dx).strength(0.06))
    .force("y", forceY(d => d.ay + d.dy).strength(0.06))
    .force("collide", forceRectCollide(10))
    .force("avoid", forceAvoid(obstacles, 6))
    .stop()
  const clamp = () => {
    for (const n of nodes) {
      n.x = Math.max(bounds.x + n.w / 2, Math.min(bounds.x + bounds.w - n.w / 2, n.x))
      n.y = Math.max(bounds.y + n.h / 2, Math.min(bounds.y + bounds.h - n.h / 2, n.y))
    }
  }
  for (let i = 0; i < 360; i++) { sim.tick(); clamp() }
  // final pass without the pull towards the anchors: only the collision forces, until nothing overlaps
  sim.force("x", null).force("y", null).alpha(0.3)
  for (let i = 0; i < 80; i++) { sim.tick(); clamp() }
  // hard guarantee: callouts never overlap each other. Earlier items win, later ones step aside.
  for (let pass = 0; pass < 24; pass++) {
    let moved = false
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i], b = nodes[j], pad = 8
        const ox = overlap1d(a.x - a.w / 2 - pad, a.x + a.w / 2 + pad, b.x - b.w / 2, b.x + b.w / 2)
        const oy = overlap1d(a.y - a.h / 2 - pad, a.y + a.h / 2 + pad, b.y - b.h / 2, b.y + b.h / 2)
        if (ox <= 0 || oy <= 0) continue
        if (ox < oy) b.x += b.x < a.x ? -ox : ox
        else b.y += b.y < a.y ? -oy : oy
        moved = true
      }
    }
    clamp()
    if (!moved) break
  }
  return nodes
}

/** point where the segment from a box centre towards (px, py) leaves the box */
function boxEdge(b, px, py) {
  const dx = px - b.x, dy = py - b.y
  if (!dx && !dy) return [b.x, b.y]
  const sx = dx ? (b.w / 2) / Math.abs(dx) : Infinity, sy = dy ? (b.h / 2) / Math.abs(dy) : Infinity
  const s = Math.min(sx, sy, 1)
  return [b.x + dx * s, b.y + dy * s]
}

/** Delaunay neighbours as unique [i, j] pairs, dropping links longer than maxLen */
function meshLinks(points, maxLen) {
  const d = Delaunay.from(points)
  const seen = new Set(), out = []
  for (let e = 0; e < d.triangles.length; e++) {
    const i = d.triangles[e], j = d.triangles[e % 3 === 2 ? e - 2 : e + 1]
    const key = i < j ? `${i}-${j}` : `${j}-${i}`
    if (seen.has(key)) continue
    seen.add(key)
    const [ax, ay] = points[i], [bx, by] = points[j]
    if (Math.hypot(ax - bx, ay - by) <= maxLen) out.push([i, j])
  }
  return out
}

const smooth = line().curve(curveCatmullRom.alpha(0.5))

/* ---------- lookups (static class strings) ---------- */
const dot = {
  critical: { r: 13, fill: "fill-critical", halo: "fill-critical/20", haloR: 34 },
  helper: { r: 11, fill: "fill-info", halo: "fill-info/20", haloR: 22 },
  ok: { r: 7.5, fill: "fill-ok", halo: null, haloR: 0 },
}
const pill = {
  critical: { box: "fill-critical", title: "fill-ink", note: "fill-ink/80", lead: "stroke-critical" },
  helper: { box: "fill-info", title: "fill-on-light", note: "fill-on-light/70", lead: "stroke-info" },
  station: { box: "", title: "fill-ink", note: "fill-muted", lead: "stroke-muted/50" },
}

/* ---------- small marks ---------- */
const AedMark = ({ x, y, out }) => (
  <g transform={`translate(${x} ${y})`}>
    <rect x="-17" y="-17" width="34" height="34" rx="9" className={out ? "fill-background stroke-info" : "fill-ink stroke-background"} strokeWidth="3" strokeDasharray={out ? "5 4" : undefined} />
    <path d="M0 11 C-17 1 -12 -12 0 -5 C12 -12 17 1 0 11 Z" className={out ? "fill-info/25" : "fill-on-light"} />
    <path d="M2 -7 L-4 1 L1 1 L-2 8 L5 -1 L0 -1 Z" className={out ? "fill-info" : "fill-ink"} />
  </g>
)

const HubMark = ({ x, y }) => (
  <g transform={`translate(${x} ${y})`}>
    <circle r="44" className="fill-info/5 stroke-info/30" strokeWidth="1.5" strokeDasharray="3 6" />
    <rect x="-16" y="-20" width="32" height="40" rx="7" className="fill-ink stroke-background" strokeWidth="3" />
    <path d="M-9 -9 H9 M-9 0 H9 M-9 9 H9" className="stroke-on-light" strokeWidth="2.5" strokeLinecap="round" />
    <circle cx="6" cy="-9" r="2.2" className="fill-ok" />
  </g>
)

/** pill callout: title + optional note on a second line */
const Callout = ({ b, tone, t }) => {
  const s = pill[tone]
  const twoLine = Boolean(b.note)
  // stations read as floor markings: the corridor's name in tracked capitals, no box
  if (tone === "station") return (
    <g>
      <text x={b.x - b.w / 2} y={twoLine ? b.y - 4 * t.k : b.y + 7 * t.k} className={cn(s.title, "uppercase tracking-label")} fontSize={t.room}>{b.title}</text>
      {twoLine && <text x={b.x - b.w / 2} y={b.y + 18 * t.k} className={s.note} fontSize={t.sub}>{b.note}</text>}
    </g>
  )
  return (
    <g>
      <rect x={b.x - b.w / 2} y={b.y - b.h / 2} width={b.w} height={b.h} rx={twoLine ? 16 * t.k : b.h / 2} className={s.box} strokeWidth="1.5" />
      <text x={b.x - b.w / 2 + t.pad} y={twoLine ? b.y - 4 * t.k : b.y + 9 * t.k} className={cn(s.title, "font-medium")} fontSize={t.title}>{b.title}</text>
      {twoLine && <text x={b.x - b.w / 2 + t.pad} y={b.y + 21 * t.k} className={s.note} fontSize={t.note}>{b.note}</text>}
    </g>
  )
}

/**
 * DeckMap: the passenger deck with who is where.
 * deck: the module from screens/dashboard/deck-data.js (DECK, hull, rooms, shelter,
 *       stations, hub, seatRows, people, aedRoute, meshMaxLink)
 */
export const DeckMap = ({ deck, className }) => {
  const { DECK, rooms, shelter, stations, hub, seatRows, people, aedRoute, meshMaxLink } = deck
  const { width: W, height: H } = DECK

  // track the drawn width so type can be scaled up on narrow cards
  const ref = useRef(null)
  const [drawn, setDrawn] = useState(DESIGN_WIDTH)
  useEffect(() => {
    const el = ref.current
    if (!el || typeof ResizeObserver === "undefined") return
    const ro = new ResizeObserver(([e]) => setDrawn(Math.round(e.contentRect.width / 20) * 20 || DESIGN_WIDTH))
    ro.observe(el)
    return () => ro.disconnect()
  }, [])
  const k = Math.min(1.6, Math.max(1, DESIGN_WIDTH / drawn))
  const T = useMemo(() => typeScale(k), [k])
  const fine = k < 1.3 // seat numbers and sub-labels only when there is room

  const layout = useMemo(() => {
    // mesh: every wristband plus the hub as the gateway
    const nodes = [...people.map(p => [p.x, p.y]), [hub.x, hub.y]]
    const links = meshLinks(nodes, meshMaxLink).map(([i, j]) => ({ a: nodes[i], b: nodes[j], ana: people[i]?.tone === "critical" || people[j]?.tone === "critical" }))

    // route
    const route = smooth(aedRoute.points)

    // fixed things the callouts must not cover
    const roomBoxes = rooms.filter(r => r.label).map(r => {
      const sub = r.sub && fine
      const w = textWidth(r.label.toUpperCase(), T.room, 0.14), h = sub ? T.room + T.sub + 12 : T.room + 6
      const cx = r.anchor === "end" ? r.lx - w / 2 : r.anchor === "start" ? r.lx + w / 2 : r.lx
      return { x: cx, y: r.ly - T.room * 0.35 + (sub ? (T.sub + 6) / 2 : 0), w, h }
    })
    const dots = people.map(p => ({ x: p.x, y: p.y, r: dot[p.tone].haloR || dot[p.tone].r + 4 }))
    const marks = [...stations.map(s => ({ x: s.x, y: s.y, r: 20 })), { x: hub.x, y: hub.y, w: 40, h: 48 }]
    // sample the route so labels stay off it
    const routeDots = aedRoute.points.slice(0, -1).flatMap((p, i) => {
      const q = aedRoute.points[i + 1]
      return [0, 0.5].map(t => ({ x: p[0] + (q[0] - p[0]) * t, y: p[1] + (q[1] - p[1]) * t, r: 6 }))
    })

    const box = (title, note) => {
      const w = Math.max(textWidth(title, T.title), note ? textWidth(note, T.note) : 0) + T.pad * 2
      return { title, note, w, h: note ? T.h2 : T.h1 }
    }
    // each callout starts on its preferred compass side (data: side), the solver moves it if that side is busy
    const items = [
      ...people.filter(p => p.callout).map(p => ({ id: p.id, tone: p.tone, ax: p.x, ay: p.y, side: p.side ?? "e", gap: dot[p.tone].haloR + 12, ...(fine ? box(p.callout, p.note) : box(p.short ?? p.callout, null)) })),
      ...stations.map(s => ({ id: `st-${s.id}`, tone: "station", ax: s.x, ay: s.y, side: s.side ?? "se", gap: 22, title: s.label, note: fine ? s.note : null, w: Math.max(textWidth(s.label.toUpperCase(), T.room, 0.14), fine ? textWidth(s.note, T.sub) : 0), h: fine ? T.room + T.sub + 10 : T.room + 6 })),
    ]
    const callouts = placeCallouts(items, [...dots, ...marks, ...roomBoxes, ...routeDots], { x: 6, y: 6, w: W - 12, h: H - 12 })
    return { links, route, callouts }
  }, [people, hub, meshMaxLink, aedRoute, rooms, stations, W, H, k, T, fine])

  const sOuter = shelter.outer, sInner = shelter.inner
  const shelterRing = `M${sOuter.x} ${sOuter.y}h${sOuter.w}v${sOuter.h}h${-sOuter.w}Z M${sInner.x} ${sInner.y}v${sInner.h}h${sInner.w}v${-sInner.h}Z`
  const stationById = Object.fromEntries(stations.map(s => [`st-${s.id}`, s]))
  const personById = Object.fromEntries(people.map(p => [p.id, p]))

  return (
    <svg ref={ref} viewBox={`0 0 ${W} ${H}`} className={cn("h-auto w-full select-none", className)} role="img" aria-label="Passenger deck: Ana Ruiz at seat 4B has no pulse, Leo is giving CPR, Tomás brought the AED from station B">
      <defs>
        <pattern id="deckmap-water" width="10" height="10" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
          <rect width="3" height="10" className="fill-info/45" />
        </pattern>
        <marker id="deckmap-arrow" viewBox="0 0 10 10" refX="7" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
          <path d="M0 0 L10 5 L0 10 Z" className="fill-info" />
        </marker>
      </defs>

      {/* base: rendered floor plan */}
      <image href={DECK.image} x="0" y="0" width={W} height={H} preserveAspectRatio="none" />

      {/* storm shelter: water walls */}
      <path d={shelterRing} fill="url(#deckmap-water)" fillRule="evenodd" />
      <rect x={sOuter.x} y={sOuter.y} width={sOuter.w} height={sOuter.h} rx="4" className="fill-none stroke-info/60" strokeWidth="2" />

      {/* room and seat labels */}
      {rooms.filter(r => r.label).map(r => (
        <g key={r.id}>
          <text x={r.lx} y={r.ly} textAnchor={r.anchor} fontSize={T.room} className={cn("uppercase tracking-label", r.kind === "shelter" ? "fill-info" : "fill-muted")}>{r.label}</text>
          {r.sub && fine && <text x={r.lx} y={r.ly + T.sub + 6} textAnchor={r.anchor} fontSize={T.sub} className={r.kind === "shelter" ? "fill-info/75" : "fill-muted/70"}>{r.sub}</text>}
        </g>
      ))}
      {fine && (
        <g fontSize={T.seat} textAnchor="middle" className="fill-faint font-mono">
          {seatRows.map(s => <text key={s.n} x={s.x + 2} y="88">{s.n}</text>)}
          <text x="66" y="131">A</text>
          <text x="66" y="196">B</text>
        </g>
      )}

      {/* LoRa mesh between wristbands (and the hub) */}
      <g strokeLinecap="round">
        {layout.links.map(l => (
          <line key={`${l.a}-${l.b}`} x1={l.a[0]} y1={l.a[1]} x2={l.b[0]} y2={l.b[1]} className={l.ana ? "stroke-critical/35" : "stroke-info/20"} strokeWidth="1.5" strokeDasharray="1 7" />
        ))}
      </g>

      {/* hub and emergency stations */}
      <HubMark x={hub.x} y={hub.y} />
      {stations.map(s => <AedMark key={s.id} x={s.x} y={s.y} out={s.state === "out"} />)}

      {/* AED route from station B to Ana */}
      <path d={layout.route} className="fill-none stroke-info/15" strokeWidth="14" strokeLinecap="round" strokeLinejoin="round" />
      <path d={layout.route} className="fill-none stroke-info" strokeWidth="3.5" strokeLinecap="round" strokeLinejoin="round" strokeDasharray="10 7" markerEnd="url(#deckmap-arrow)" />

      {/* people */}
      {people.map(p => {
        const d = dot[p.tone]
        return (
          <g key={p.id}>
            {d.halo && <circle cx={p.x} cy={p.y} r={d.haloR} className={d.halo} />}
            {p.tone === "critical" && <circle cx={p.x} cy={p.y} r="18" className="origin-center animate-ping fill-none stroke-critical transform-fill motion-reduce:animate-none" strokeWidth="3" />}
            <circle cx={p.x} cy={p.y} r={d.r} className={cn(d.fill, "stroke-background")} strokeWidth="3" />
          </g>
        )
      })}

      {/* callouts with leader lines */}
      {layout.callouts.map(b => {
        const anchor = stationById[b.id] ?? personById[b.id]
        const [ex, ey] = boxEdge(b, anchor.x, anchor.y)
        const len = Math.hypot(ex - anchor.x, ey - anchor.y)
        const r0 = b.tone === "station" ? 20 : dot[anchor.tone].r + 3
        const sx = anchor.x + ((ex - anchor.x) / (len || 1)) * r0, sy = anchor.y + ((ey - anchor.y) / (len || 1)) * r0
        return (
          <g key={b.id}>
            {len > r0 + 4 && <line x1={sx} y1={sy} x2={ex} y2={ey} className={pill[b.tone].lead} strokeWidth="2" />}
            <Callout b={b} tone={b.tone} t={T} />
          </g>
        )
      })}
    </svg>
  )
}

/* ---------- legend ---------- */
const Swatch = ({ children }) => <svg viewBox="0 0 28 16" className="h-4 w-7 shrink-0" aria-hidden="true">{children}</svg>
const legendItems = [
  { label: "No pulse · in care", mark: <Swatch><circle cx="14" cy="8" r="6" className="fill-critical" /></Swatch> },
  { label: "Helping", mark: <Swatch><circle cx="14" cy="8" r="5.5" className="fill-info" /></Swatch> },
  { label: "OK", mark: <Swatch><circle cx="14" cy="8" r="4" className="fill-ok" /></Swatch> },
  { label: "AED route", mark: <Swatch><path d="M2 8 H24" className="stroke-info" strokeWidth="2.5" strokeDasharray="5 3" /><path d="M21 4 L27 8 L21 12 Z" className="fill-info" /></Swatch> },
  { label: "AED station", mark: <Swatch><rect x="7" y="1" width="14" height="14" rx="4" className="fill-ink" /><path d="M14 12 C7 8 9 3 14 6 C19 3 21 8 14 12 Z" className="fill-on-light" /></Swatch> },
  { label: "LoRa mesh", mark: <Swatch><path d="M2 8 H26" className="stroke-info/60" strokeWidth="2" strokeDasharray="1 4" strokeLinecap="round" /></Swatch> },
  { label: "Ship hub", mark: <Swatch><rect x="8" y="1" width="12" height="14" rx="3" className="fill-ink" /><path d="M11 6 H17 M11 10 H17" className="stroke-on-light" strokeWidth="1.5" /></Swatch> },
  { label: "Storm shelter · water walls", mark: <Swatch><rect x="3" y="2" width="22" height="12" rx="2" className="fill-none stroke-info/70" strokeWidth="1.5" /><path d="M6 13 L13 3 M12 13 L19 3 M18 13 L24 5" className="stroke-info/50" strokeWidth="1.5" /></Swatch> },
]

export const DeckMapLegend = ({ className }) => (
  <ul className={cn("flex flex-wrap gap-x-5 gap-y-2 text-xs text-muted", className)}>
    {legendItems.map(it => <li key={it.label} className="flex items-center gap-1.5">{it.mark}<span>{it.label}</span></li>)}
  </ul>
)
