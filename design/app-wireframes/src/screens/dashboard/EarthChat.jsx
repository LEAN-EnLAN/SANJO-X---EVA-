// Earth view side chat with Eva, the ship's assistant, rendered with OpenUI.
// Every message body is OpenUI Lang. @openuidev/react-lang parses it and draws it with the
// component library below, built from Eva tokens and @/ds primitives (no default OpenUI theme).
// @openuidev/react-headless keeps the thread. There is no live model in a static wireframe:
// Eva's newest packet is pre-written and replayed in chunks, as if it were streaming in from the ship.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react"
import { action, createLibrary, defineComponent, Renderer, useIsStreaming, useTriggerAction } from "@openuidev/react-lang"
import { ChatProvider, useThread } from "@openuidev/react-headless"
import { z } from "zod/v4"
import { ArrowUp, Check, Clock, HeartPulse, Radio, Satellite, Zap } from "lucide-react"
import { Avatar, Badge, Button, Progress, StatusDot } from "@/ds"
import { cn } from "@/ds/cn"

/* ---------- clocks: the Earth console sees the ship 12 min 40 s late ---------- */
const DELAY_S = 12 * 60 + 40
const EARTH_NOW = "14:16:52"
const toS = t => t.split(":").reduce((a, v) => a * 60 + Number(v), 0)
const toClock = s => [Math.floor(s / 3600) % 24, Math.floor(s / 60) % 60, s % 60].map(v => String(v).padStart(2, "0")).join(":")
const reachesShip = sent => toClock(toS(sent) + DELAY_S)
const minutesLeft = sent => Math.max(1, Math.round((toS(sent) + DELAY_S - toS(EARTH_NOW)) / 60))

/* ---------- tone lookups (static, per DESIGN.md) ---------- */
const toneText = { critical: "text-critical", warn: "text-warn", ok: "text-ok", info: "text-info", ink: "text-ink", muted: "text-muted" }
const toneDot = { critical: "bg-critical", warn: "bg-warn", ok: "bg-ok", info: "bg-info", neutral: "bg-faint" }
const toneStroke = { critical: "stroke-critical", warn: "stroke-warn", ok: "stroke-ok", info: "stroke-info", ink: "stroke-ink" }
const vitalSkin = { critical: "border-critical/30 bg-critical/10", warn: "border-warn/25 bg-warn/5", ok: "border-line bg-surface", info: "border-line bg-surface" }
const tones = z.enum(["critical", "warn", "ok", "info", "neutral"])

/* which bubble a component is drawn in, and which message it belongs to */
const BubbleCtx = createContext({ id: null, out: false })
const ChatCtx = createContext({ chosen: {} })

/* ---------- OpenUI component library, mapped to the Eva design system ---------- */

const Stack = defineComponent({
  name: "Stack",
  description: "Vertical list of blocks that make one chat message.",
  props: z.object({ children: z.array(z.any()) }),
  component: ({ props, renderNode }) => <div className="flex flex-col gap-3">{renderNode(props.children)}</div>,
})

const Text = defineComponent({
  name: "Text",
  description: "One short paragraph in plain words.",
  props: z.object({ text: z.string(), tone: z.enum(["default", "muted"]).optional() }),
  component: ({ props }) => {
    const { out } = useContext(BubbleCtx)
    if (out) return <p className="self-end rounded-card rounded-tr-md bg-ink px-4 py-3 text-sm leading-relaxed text-on-light">{props.text}</p>
    return <p className={cn("text-sm leading-relaxed", props.tone === "muted" ? "text-muted" : "text-ink")}>{props.text}</p>
  },
})

const Alert = defineComponent({
  name: "Alert",
  description: "A one-line alarm with a short detail underneath.",
  props: z.object({ title: z.string(), detail: z.string().optional(), tone: tones.optional() }),
  component: ({ props }) => (
    <div className="flex items-center gap-3 rounded-tile border border-critical/40 bg-critical/10 px-3.5 py-3">
      <span className="grid size-9 shrink-0 place-items-center rounded-full bg-critical text-ink"><HeartPulse size={17} /></span>
      <span className="flex min-w-0 flex-col leading-tight">
        <span className="text-sm text-ink">{props.title}</span>
        {props.detail && <span className="text-xs text-muted">{props.detail}</span>}
      </span>
    </div>
  ),
})

const Vital = defineComponent({
  name: "Vital",
  description: "One vital sign at the moment of the alert: label, value and a plain read-out.",
  props: z.object({ label: z.string(), value: z.string(), read: z.string().optional(), tone: tones.optional() }),
  component: ({ props }) => (
    <div className={cn("flex flex-col gap-1 rounded-tile border px-3 py-2.5 transition duration-500 ease-out starting:translate-y-1 starting:opacity-0 motion-reduce:transition-none", vitalSkin[props.tone ?? "info"])}>
      <span className="text-xs uppercase tracking-label text-muted">{props.label}</span>
      <span className={cn("text-lg leading-none tracking-tight", toneText[props.tone === "critical" ? "critical" : "ink"])}>{props.value}</span>
      {props.read && <span className="text-xs text-muted">{props.read}</span>}
    </div>
  ),
})

const Step = defineComponent({
  name: "Step",
  description: "One step of the case timeline, in ship time.",
  props: z.object({ ship: z.string(), title: z.string(), tone: tones.optional() }),
  component: ({ props }) => (
    <li className="relative flex gap-3 pb-3 transition duration-500 ease-out last:pb-0 starting:translate-x-1 starting:opacity-0 motion-reduce:transition-none">
      <span className="relative flex w-2.5 shrink-0 justify-center pt-1.5">
        <span className={cn("size-2.5 rounded-full ring-4 ring-raised", toneDot[props.tone ?? "neutral"])} />
      </span>
      <span className="w-16 shrink-0 font-mono text-xs leading-5 text-muted tabular-nums">{props.ship}</span>
      <span className="min-w-0 flex-1 text-sm leading-5 text-ink">{props.title}</span>
    </li>
  ),
})

const CaseSummary = defineComponent({
  name: "CaseSummary",
  description: "Case card: who, status, the vitals at the moment of the alert and the timeline so far.",
  props: z.object({
    name: z.string(), seat: z.string(), status: z.string(),
    vitals: z.array(Vital.ref), steps: z.array(Step.ref), footnote: z.string().optional(),
  }),
  component: ({ props, renderNode }) => (
    <article className="flex flex-col gap-4 rounded-tile border border-line bg-raised p-4">
      <header className="flex items-center gap-3">
        <Avatar name={props.name} tone="critical" size="sm" />
        <span className="flex min-w-0 flex-1 flex-col leading-tight">
          <span className="truncate text-sm text-ink">{props.name} · seat {props.seat}</span>
          <span className="truncate text-xs text-muted">{props.status}</span>
        </span>
        <Badge tone="critical"><StatusDot tone="critical" pulse />Open</Badge>
      </header>
      {props.vitals?.length > 0 && (
        <section className="flex flex-col gap-2">
          <span className="text-xs uppercase tracking-label text-faint">Vitals at the alert · 14:02:08</span>
          <div className="grid grid-cols-2 gap-2">{renderNode(props.vitals)}</div>
        </section>
      )}
      {props.steps?.length > 0 && (
        <section className="flex flex-col gap-2.5">
          <span className="text-xs uppercase tracking-label text-faint">Timeline · ship time</span>
          <ol className="relative flex flex-col before:absolute before:top-2 before:bottom-2 before:left-1 before:w-px before:bg-line">{renderNode(props.steps)}</ol>
        </section>
      )}
      {props.footnote && <p className="flex items-center gap-2 border-t border-line pt-3 text-xs text-muted"><Zap size={13} className="shrink-0 text-info" />{props.footnote}</p>}
    </article>
  ),
})

/* one beat as [dt seconds, amplitude]: P wave, QRS spike, T wave */
const beat = [[0, 0], [0.06, 0.08], [0.1, 0.12], [0.14, 0.08], [0.18, 0], [0.22, 0], [0.24, -0.12], [0.27, 1], [0.3, -0.28], [0.33, 0], [0.42, 0.04], [0.5, 0.22], [0.58, 0.24], [0.66, 0.04], [0.72, 0]]
const ECG_W = 480, ECG_H = 88, ECG_S = 10

/** compact 10 s wrist trace drawn at chat size, so the grid and labels stay legible */
const EcgTrace = ({ rr, tone }) => {
  const x = t => (t / ECG_S) * ECG_W, mid = ECG_H * 0.64, amp = ECG_H * 0.52
  const pts = [[0, mid]]
  let t = 0.25, i = 0
  while (rr.length && t < ECG_S - 0.75) {
    beat.forEach(([dt, a]) => pts.push([x(t + dt), mid - a * amp]))
    t += Math.max(0.35, rr[i % rr.length]); i++
  }
  pts.push([ECG_W, mid])
  const secs = Array.from({ length: ECG_S + 1 }, (_, s) => s)
  return (
    <svg viewBox={`0 0 ${ECG_W} ${ECG_H + 18}`} className="h-auto w-full" role="img" aria-label="10 second wrist ECG trace">
      {secs.map(s => <line key={s} x1={x(s)} x2={x(s)} y1="0" y2={ECG_H} className={s % 5 === 0 ? "stroke-line" : "stroke-line/50"} strokeWidth="1" />)}
      {[0.25, 0.5, 0.75].map(f => <line key={f} x1="0" x2={ECG_W} y1={ECG_H * f} y2={ECG_H * f} className="stroke-line/40" strokeWidth="1" />)}
      <polyline points={pts.map(p => p.join(",")).join(" ")} fill="none" strokeWidth="1.75" strokeLinejoin="round" strokeLinecap="round" className={toneStroke[tone] ?? "stroke-ink"} />
      {[0, 5, 10].map(s => <text key={s} x={Math.min(x(s) + 3, ECG_W - 26)} y={ECG_H + 14} className="fill-faint text-xs">{s} s</text>)}
    </svg>
  )
}

const resultTone = { ok: "ok", warn: "warn", critical: "critical", info: "info", neutral: "neutral" }

const WristEcg = defineComponent({
  name: "WristEcg",
  description: "A 1-lead wrist ECG strip (two metal pads, one finger of each hand). Never a 12-lead ECG.",
  props: z.object({
    title: z.string(), result: z.string(), rr: z.array(z.number()),
    tone: tones.optional(), note: z.string().optional(),
  }),
  component: ({ props }) => {
    const streaming = useIsStreaming()
    const rr = props.rr ?? []
    return (
      <article className="flex flex-col gap-3 rounded-tile border border-line bg-raised p-4">
        <header className="flex items-start justify-between gap-3">
          <span className="flex min-w-0 flex-col leading-tight">
            <span className="text-sm text-ink">{props.title}</span>
            <span className="text-xs text-muted">1-lead wrist ECG · two pads · not a 12-lead</span>
          </span>
          <Badge tone={resultTone[props.tone ?? "neutral"]}>{props.result}</Badge>
        </header>
        <div className="rounded-tile bg-background px-3 pt-3 pb-1">
          {rr.length > 1 || !streaming ? <EcgTrace rr={rr} tone="ink" /> : <div className="h-24 animate-pulse rounded-tile bg-surface" />}
        </div>
        {props.note && <p className="text-xs text-muted">{props.note}</p>}
      </article>
    )
  },
})

const replyStyle = { primary: "primary", secondary: "secondary" }

const QuickReply = defineComponent({
  name: "QuickReply",
  description: "A suggested answer the flight surgeon can send with one tap.",
  props: z.object({ label: z.string(), action: action().optional(), variant: z.enum(["primary", "secondary"]).optional() }),
  component: ({ props }) => {
    const trigger = useTriggerAction()
    const streaming = useIsStreaming()
    const { id } = useContext(BubbleCtx)
    const { chosen } = useContext(ChatCtx)
    const picked = chosen[id]
    const isPicked = picked === props.label
    return (
      <span className={picked && !isPicked ? "opacity-40 transition-opacity duration-500" : "transition-opacity duration-500"}>
        <Button
          size="sm"
          variant={isPicked ? "primary" : picked ? "outline" : replyStyle[props.variant ?? "secondary"]}
          disabled={streaming || Boolean(picked)}
          aria-pressed={isPicked}
          onClick={() => trigger(props.label, undefined, props.action)}
        >
          {isPicked && <Check size={14} />}{props.label}
        </Button>
      </span>
    )
  },
})

const QuickReplies = defineComponent({
  name: "QuickReplies",
  description: "A row of quick replies at the end of a message.",
  props: z.object({ replies: z.array(QuickReply.ref), hint: z.string().optional() }),
  component: ({ props, renderNode }) => (
    <div className="flex flex-col gap-2 pt-1">
      {props.hint && <span className="text-xs uppercase tracking-label text-faint">{props.hint}</span>}
      <div className="flex flex-wrap gap-2">{renderNode(props.replies)}</div>
    </div>
  ),
})

const Delivery = defineComponent({
  name: "Delivery",
  description: "Delivery status of a message to the ship: queued, on its way or delivered.",
  props: z.object({ status: z.enum(["queued", "on-the-way", "delivered"]), minutes: z.number().optional(), arrives: z.string().optional() }),
  component: ({ props }) => {
    const left = props.minutes ?? 13
    const done = props.status === "delivered"
    return (
      <span className="inline-flex items-center gap-2 self-end rounded-full border border-info/30 bg-info/10 py-1 pr-3 pl-1.5 text-xs text-info">
        <span className="grid size-5 place-items-center rounded-full bg-info text-on-light">{done ? <Check size={12} strokeWidth={3} /> : <Clock size={12} strokeWidth={2.5} />}</span>
        <span className="whitespace-nowrap">{done ? "Delivered to the ship" : `On its way · arrives in ~${left} min`}</span>
        {!done && <span className="w-10"><Progress value={Math.round(((13 - left) / 13) * 100) + 4} size="sm" marker={false} /></span>}
        {props.arrives && <span className="whitespace-nowrap font-mono text-info/70 tabular-nums">ship ~{props.arrives}</span>}
      </span>
    )
  },
})

const evaLibrary = createLibrary({
  root: "Stack",
  components: [Stack, Text, Alert, Vital, Step, CaseSummary, WristEcg, QuickReply, QuickReplies, Delivery],
})

/* ---------- the conversation, written in OpenUI Lang ---------- */
const q = s => JSON.stringify(s)

/** an outgoing message from the flight surgeon, with its delivery chip */
const outgoing = (text, sent) => [
  "root = Stack([body, status])",
  `body = Text(${q(text)})`,
  `status = Delivery("on-the-way", ${minutesLeft(sent)}, ${q(reachesShip(sent))})`,
].join("\n")

const alarm = `root = Stack([intro, alert])
intro = Text("Alarm for Ana Ruiz, seat 4B. I sent Leo to start CPR and Tomás to bring the AED from the emergency station.")
alert = Alert("No pulse at the wrist", "No movement · no answer · 14:02:08 ship time", "critical")`

const mara = `root = Stack([t])
t = Text("Ana collapsed at her seat in the cabin, no pulse. Leo is doing CPR, Tomás brought the AED. Eva is guiding.")`

/* Ana's last wrist ECG, this morning: steady gaps, about 66 a minute */
const anaRR = [0.91, 0.9, 0.92, 0.89, 0.91, 0.9, 0.92, 0.9, 0.91, 0.9, 0.89]

/** Eva's newest packet: root first, so the card shell draws at once and fills in as lines arrive */
const evaUpdate = `root = Stack([intro, summary, ecg, plan, replies])
intro = Text("Case update for Ana Ruiz. CPR is going and the AED has advised a shock.")
summary = CaseSummary("Ana Ruiz", "4B", "Cardiac arrest · 2 min since the alarm", [v1, v2, v3, v4], [s1, s2, s3, s4], "The AED reads the heart through its own pads and decides shocks. I only guide.")
v1 = Vital("Pulse", "None", "at the wrist", "critical")
v2 = Vital("Movement", "None", "no answer to me", "critical")
v3 = Vital("Just before", "178 bpm", "fast and uneven", "warn")
v4 = Vital("Skin", "32.9 °C", "falling", "warn")
s1 = Step("14:02:08", "No pulse, no movement, no answer", "critical")
s2 = Step("14:02:20", "Leo sent to Ana · Tomás to the AED", "info")
s3 = Step("14:03:05", "CPR started · I guide 110 a minute", "info")
s4 = Step("14:04:05", "AED attached · the AED advised a shock", "critical")
ecg = WristEcg("Her last wrist ECG · today 08:06", "Regular · 66 bpm", [${anaRR.join(", ")}], "ok", "Her baseline from this morning. The AED log comes in the next packet.")
plan = Text("Plan on board: keep CPR going, follow the AED's voice, check breathing every 2 minutes.")
replies = QuickReplies([r1, r2, r3], "Reply to the crew")
r1 = QuickReply("Agree with plan", @ToAssistant("Agree with the plan. Keep CPR going and follow the AED's voice."), "primary")
r2 = QuickReply("Ask for a repeat check", @ToAssistant("Please repeat the pulse and breathing check after the next AED analysis, and send it to me."))
r3 = QuickReply("Call me when stable", @ToAssistant("Call me on the next link slot when Ana is stable. I am staying on this case."))`

const SURGEON_SENT = "14:15:30"

/* sender, ship time and Earth received time travel with each message */
const seed = [
  { id: "m1", role: "assistant", content: alarm, meta: { from: "eva", ship: "14:02:12", earth: "14:14:52" } },
  { id: "m2", role: "user", content: outgoing("Got the alarm. Keep CPR going and follow the AED's voice. I am on this case.", SURGEON_SENT), meta: { from: "surgeon", sent: SURGEON_SENT } },
  { id: "m3", role: "assistant", content: mara, meta: { from: "mara", ship: "14:03:50", earth: "14:16:30" } },
]
const LIVE = { id: "m4", role: "assistant", meta: { from: "eva", ship: "14:04:10", earth: "14:16:50" } }

/* no model to call: Eva lives on the ship, 12 min 40 s away, so nothing answers instantly */
const offlineLLM = {
  send: async () => new Response("", { status: 204 }),
  streamProtocol: { async *parse() {} },
}

/* ---------- presentation ---------- */

const reducedMotion = () => typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches

/** Eva's orb: a soft info-blue sphere with a highlight, a glow, and a ping while a packet streams in */
export const EvaOrb = ({ live, size = "md" }) => (
  <span className={cn("relative grid shrink-0 place-items-center", size === "sm" ? "size-8" : "size-12")}>
    <span className="absolute inset-0 rounded-full bg-info/25 blur-md" />
    {live && <span className="absolute inset-1 animate-ping rounded-full bg-info/30 motion-reduce:animate-none" />}
    <span className={cn("relative overflow-hidden rounded-full bg-radial from-info via-info/60 to-surface ring-1 ring-info/50", size === "sm" ? "size-7" : "size-10")}>
      <span className={cn("absolute rounded-full bg-ink/80 blur-xs", size === "sm" ? "top-1 left-1.5 size-2" : "top-1.5 left-2 size-3")} />
      <span className="absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-background/60 to-transparent" />
    </span>
  </span>
)

const people = { eva: { name: "Eva", role: "ship assistant" }, mara: { name: "Mara Okoye", role: "crew lead" }, surgeon: { name: "You", role: "flight surgeon" } }

const Meta = ({ m }) => m.from === "surgeon"
  ? <span className="font-mono text-xs text-faint tabular-nums">sent {m.sent} Earth</span>
  : <span className="font-mono text-xs text-faint tabular-nums">ship {m.ship} · here {m.earth}</span>

function MessageRow({ msg, streaming, onAction }) {
  const m = msg.meta
  const out = m.from === "surgeon"
  const ctx = useMemo(() => ({ id: msg.id, out }), [msg.id, out])
  const body = (
    <BubbleCtx.Provider value={ctx}>
      <Renderer response={msg.content} library={evaLibrary} isStreaming={streaming} onAction={onAction} />
    </BubbleCtx.Provider>
  )
  const enter = "transition duration-700 ease-out starting:translate-y-3 starting:opacity-0 motion-reduce:transition-none"

  if (out) return (
    <li className={cn("flex flex-col items-end gap-1.5 pl-12", enter)}>
      <div className="flex items-baseline gap-2"><Meta m={m} /><span className="text-xs text-muted">You</span></div>
      <div className="flex flex-col items-end">{body}</div>
    </li>
  )

  const isEva = m.from === "eva"
  return (
    <li className={cn("flex gap-3", enter)}>
      {isEva ? <EvaOrb size="sm" live={streaming} /> : <Avatar name={people[m.from].name} size="sm" />}
      <div className="flex min-w-0 flex-1 flex-col gap-1.5">
        <div className="flex flex-wrap items-baseline gap-x-2">
          <span className="text-sm text-ink">{people[m.from].name}</span>
          <span className="text-xs text-muted">{people[m.from].role}</span>
          <span className="ml-auto"><Meta m={m} /></span>
        </div>
        {isEva
          ? <div className="flex flex-col gap-3">{body}</div>
          : <div className="self-start rounded-card rounded-tl-md border border-line bg-raised px-4 py-3">{body}</div>}
      </div>
    </li>
  )
}

/** "receiving" row shown before the first line of a streamed packet lands */
const Receiving = () => (
  <li className="flex items-center gap-3 transition duration-500 ease-out starting:opacity-0 motion-reduce:transition-none">
    <EvaOrb size="sm" live />
    <span className="flex items-center gap-2 rounded-full border border-line bg-surface px-3 py-1.5 text-xs text-muted">
      <Satellite size={13} className="text-info" />Packet from the ship arriving
      <span className="flex gap-1"><span className="size-1 animate-pulse rounded-full bg-info" /><span className="size-1 animate-pulse rounded-full bg-info/70" /><span className="size-1 animate-pulse rounded-full bg-info/40" /></span>
    </span>
  </li>
)

/** thin divider with a centred label */
const Divider = ({ children }) => (
  <li className="flex items-center gap-3 text-xs text-faint">
    <span className="h-px flex-1 bg-line" /><span className="uppercase tracking-label">{children}</span><span className="h-px flex-1 bg-line" />
  </li>
)

let nextId = 10

function Chat() {
  const { messages, setMessages, appendMessages, updateMessage } = useThread()
  const [phase, setPhase] = useState("idle") // idle -> receiving -> streaming -> done
  const [chosen, setChosen] = useState({})
  const [draft, setDraft] = useState("")
  const root = useRef(null)
  const list = useRef(null)

  // seed the thread once
  useEffect(() => { setMessages(seed) }, [setMessages])

  // replay Eva's newest packet when the chat first comes into view
  const started = useRef(false)
  useEffect(() => {
    const el = root.current
    if (!el || started.current) return
    const timers = []
    const start = () => {
      started.current = true
      if (reducedMotion()) { appendMessages({ ...LIVE, content: evaUpdate }); setPhase("done"); return }
      setPhase("receiving")
      timers.push(setTimeout(() => {
        // the root line lands first, so the message shell draws at once
        let at = evaUpdate.indexOf("\n") + 1, n = 0
        setPhase("streaming")
        appendMessages({ ...LIVE, content: evaUpdate.slice(0, at) })
        const tick = () => {
          at = Math.min(evaUpdate.length, at + 9 + ((n++ * 7) % 13))
          updateMessage({ ...LIVE, content: evaUpdate.slice(0, at) })
          if (at < evaUpdate.length) timers.push(setTimeout(tick, 28))
          else setPhase("done")
        }
        timers.push(setTimeout(tick, 28))
      }, 1400))
    }
    const io = new IntersectionObserver(([e]) => { if (e.isIntersecting) { io.disconnect(); start() } }, { threshold: 0.35 })
    io.observe(el)
    return () => { io.disconnect(); timers.forEach(clearTimeout) }
  }, [appendMessages, updateMessage])

  // keep the newest message in view inside the list (never scrolls the page)
  useEffect(() => {
    const el = list.current
    if (el) el.scrollTo({ top: el.scrollHeight, behavior: reducedMotion() ? "auto" : "smooth" })
  }, [messages, phase])

  const send = useCallback(text => {
    const body = text.trim()
    if (!body) return
    appendMessages({ id: `m${nextId++}`, role: "user", content: outgoing(body, EARTH_NOW), meta: { from: "surgeon", sent: EARTH_NOW } })
  }, [appendMessages])

  const onAction = useCallback(msgId => event => {
    if (event.type !== "continue_conversation") return
    const label = event.humanFriendlyMessage
    setChosen(c => (c[msgId] ? c : { ...c, [msgId]: labelFor(label) }))
    send(label)
  }, [send])

  const submit = e => { e.preventDefault(); send(draft); setDraft("") }
  const streaming = phase === "streaming"

  return (
    <ChatCtx.Provider value={{ chosen }}>
      <section ref={root} className="absolute inset-0 flex flex-col overflow-hidden rounded-card border border-line bg-surface">
        {/* header */}
        <header className="flex flex-col gap-4 border-b border-line bg-background/40 px-5 pt-5 pb-4">
          <div className="flex items-center gap-3">
            <EvaOrb live={phase === "receiving" || streaming} />
            <div className="flex min-w-0 flex-1 flex-col leading-tight">
              <span className="flex items-center gap-2 text-base text-ink">Eva<span className="text-xs text-muted">ship assistant</span></span>
              <span className="flex items-center gap-1.5 text-sm text-info"><Clock size={14} />Ship · 12 min 40 s behind</span>
            </div>
            <Badge tone={phase === "receiving" || streaming ? "info" : "neutral"}>
              <Radio size={12} />{phase === "receiving" || streaming ? "Receiving" : "Link open"}
            </Badge>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-xs uppercase tracking-label text-faint">To</span>
            <span className="flex -space-x-1">
              {["Mara Okoye", "Leo Brandt", "Tomás Rey"].map(n => <span key={n} className="rounded-full ring-2 ring-surface"><Avatar name={n} size="sm" /></span>)}
            </span>
            <span className="truncate text-xs text-muted">Mara · Leo · Tomás</span>
            <span className="ml-auto font-mono text-xs text-faint tabular-nums">Earth {EARTH_NOW}</span>
          </div>
        </header>

        {/* messages */}
        <ol ref={list} className="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto overscroll-contain px-5 py-6">
          <Divider>Case 0414 · opened 14:02 ship time</Divider>
          {messages.map(msg => (
            <MessageRow key={msg.id} msg={msg} streaming={streaming && msg.id === LIVE.id} onAction={onAction(msg.id)} />
          ))}
          {phase === "receiving" && <Receiving />}
          <li aria-hidden className="mt-auto" />
        </ol>

        {/* composer */}
        <form onSubmit={submit} className="flex flex-col gap-3 border-t border-line bg-background/40 p-4">
          <label className="flex flex-col gap-2 rounded-tile border border-line bg-raised px-4 pt-3 pb-2 transition-colors focus-within:border-info/50">
            <span className="sr-only">Message to the ship</span>
            <textarea
              rows={2}
              value={draft}
              onChange={e => setDraft(e.target.value)}
              onKeyDown={e => { if (e.key === "Enter" && !e.shiftKey) submit(e) }}
              placeholder="Write to Mara, Leo and Tomás…"
              className="w-full resize-none bg-transparent text-sm leading-relaxed text-ink placeholder:text-faint focus:outline-none"
            />
            <span className="flex items-center gap-2 text-xs text-faint"><Satellite size={13} />Next link slot in about 4 min · earliest reply in about 26 min</span>
          </label>
          <div className="flex items-center justify-between gap-3">
            <span className="flex items-center gap-1.5 text-xs text-muted"><Zap size={13} className="text-info" />The AED decides shocks</span>
            <Button type="submit" size="sm" disabled={!draft.trim()} className="shrink-0">Send · arrives in ~13 min<ArrowUp size={14} /></Button>
          </div>
        </form>
      </section>
    </ChatCtx.Provider>
  )
}

/* quick replies send a longer sentence; the chip keeps the short label */
const replyLabels = {
  "Agree with the plan. Keep CPR going and follow the AED's voice.": "Agree with plan",
  "Please repeat the pulse and breathing check after the next AED analysis, and send it to me.": "Ask for a repeat check",
  "Call me on the next link slot when Ana is stable. I am staying on this case.": "Call me when stable",
}
const labelFor = text => replyLabels[text] ?? text

/** tall right-hand chat column; it fills the height of its grid row */
export function EarthChat({ className }) {
  return (
    <div className={cn("relative min-h-200", className)}>
      <ChatProvider llm={offlineLLM}>
        <Chat />
      </ChatProvider>
    </div>
  )
}
