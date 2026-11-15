import { useState } from "react"
import { Avatar, Badge, Bars, Button, Card, CardHeader, CardTitle, Eyebrow, HalfGauge, Headline, IconTile, PillSelect, Ring, Segmented, SplitBar, Stat, StatusDot } from "@/ds"
import { AppWindow, BandMode, EvaDock, EvaMessage, InfoChip, Legend, Metric, PartBatteries, PassengerChip, Pipeline, PulseTrend, ReplyChip, RosterGroup, RosterPerson, RosterRow } from "@/ds/dashboard-extra"
import { Users, Siren, Eye, Satellite, Radiation, Magnet, Shirt, ArrowUpRight, Phone, Timer, MapPin, Droplet, Utensils } from "lucide-react"
import { DeckMap, DeckMapLegend } from "@/ds/deck-map"
import { Citation, Rail, TopBar } from "./shared"
import * as deck from "./deck-data"
import { crewLead, roster, syncQueue, weekAlerts, weekLabels } from "./data"

/* ---------- lookups (static class strings only) ---------- */
const statusBadge = {
  care: { tone: "critical", label: "In care" },
  ecg: { tone: "warn", label: "ECG review" },
  battery: { tone: "warn", label: "Battery low" },
  warm: { tone: "warn", label: "Skin warm" },
}
const queueBadge = { saved: { tone: "neutral", label: "Saved" }, sent: { tone: "info", label: "Sent" }, ack: { tone: "ok", label: "Acknowledged" } }

/* ---------- status strip: one calm row of KPIs ---------- */
const strip = [
  { icon: Users, label: "Passengers OK", value: "14", unit: "/ 15", delta: "Ana in care", deltaTone: "critical" },
  { icon: Siren, label: "Open alerts", value: "1", delta: "No pulse", deltaTone: "critical", hint: "AED on" },
  { icon: Eye, label: "Watching", value: "3", delta: "ECG · battery · skin", deltaTone: "warn" },
  { icon: Satellite, label: "Earth delay", value: "12", unit: "min 40 s", delta: "Reply ≈ 25 min", deltaTone: "neutral" },
  { icon: Radiation, label: "Cabin dose", value: "98", unit: "% of plan", delta: "As expected", deltaTone: "ok" },
]
const StatusStrip = () => (
  <Card className="px-0 py-5">
    <div className="grid grid-cols-5 divide-x divide-line">
    {strip.map(({ icon: I, ...stat }) => (
      <div key={stat.label} className="flex items-start justify-between gap-3 px-6">
        <Stat {...stat} />
        <span className="grid size-9 shrink-0 place-items-center rounded-full bg-raised text-muted"><I size={16} /></span>
      </div>
    ))}
    </div>
  </Card>
)

/* ---------- roster: grouped by what matters ---------- */
const needsHelp = roster.filter(p => p.group === "help")
const watching = roster.filter(p => p.group === "watch")
const allGood = roster.filter(p => p.group === "ok")
const helpers = roster.filter(p => p.status === "helping" || p.status === "aed")
const suitCount = roster.filter(p => p.band === "suit").length
const attentionCount = needsHelp.length + watching.length

const rosterColumns = { status: "Status", pulse: "Pulse · last 2 h", spo2: "SpO₂", skin: "Skin temp", ecg: "Wrist ECG", band: "Band", battery: "Battery" }
const helperRole = { helping: "CPR", aed: "AED" }

const richCells = p => {
  const care = p.status === "care"
  const b = statusBadge[p.status]
  return {
    status: <Badge tone={b.tone}>{care && <StatusDot tone="critical" pulse />}{b.label}</Badge>,
    pulse: care
      ? <PulseTrend value="No pulse" unit={null} word={p.hrWord} tone="critical" wordTone="critical" data={p.pulse} usual={p.usual} line="critical" />
      : <PulseTrend value={p.hr} word={p.hrWord} data={p.pulse} usual={p.usual} />,
    spo2: p.spo2 == null
      ? <Metric value="—" word="no reading" tone="critical" wordTone="critical" />
      : <Metric value={p.spo2} unit="%" word={p.spo2 >= 95 ? "normal" : "low"} wordTone={p.spo2 >= 95 ? "muted" : "warn"} />,
    skin: <Metric value={p.skin.toFixed(1)} unit="°C" word={p.skinWord} tone={p.status === "warm" ? "warn" : "ink"} wordTone={p.status === "warm" ? "warn" : "muted"} />,
    ecg: <Metric mono value={p.ecg.t} word={p.ecg.result} tone={p.ecg.result === "review" ? "warn" : "ink"} wordTone={p.ecg.result === "review" ? "warn" : "muted"} />,
    band: <BandMode mode={p.band} />,
    battery: <PartBatteries base={p.battery.base} screen={p.battery.screen} />,
  }
}

const HelperPill = ({ p }) => (
  <span className="inline-flex h-8 items-center gap-2 rounded-full bg-background/60 pr-3 pl-1">
    <Avatar name={p.name} tone="info" size="sm" />
    <span className="text-xs text-ink">{p.name.split(" ")[0]} · {helperRole[p.status]}</span>
    <span className="text-xs text-muted tabular-nums">{p.hr} bpm</span>
  </span>
)

const Roster = () => (
  <Card className="flex flex-col gap-6">
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-6">
        <div className="flex items-center gap-3">
          <CardTitle className="text-xl">Passengers now</CardTitle>
          <Badge>15 on board</Badge>
          <span className="flex items-center gap-1.5 text-xs text-muted"><StatusDot tone="ok" />live · 3 s ago</span>
        </div>
        <div className="w-112 shrink-0"><Segmented items={["All · 15", `Needs attention · ${attentionCount}`, `Suit mode · ${suitCount}`]} active="All · 15" /></div>
      </div>
      <div className="flex items-center justify-between gap-6 border-b border-line pb-4">
        <Legend items={[{ label: "Needs help now", tone: "critical" }, { label: "Watching", tone: "warn" }, { label: "All good", tone: "ok" }]} />
        <span className="flex items-center gap-5 text-xs text-muted">
          <span className="flex items-center gap-2"><span className="h-3 w-5 rounded-full bg-ok/20 ring-1 ring-ok/40" />Usual pulse range</span>
          <span className="flex items-center gap-1.5"><Magnet size={13} />Joined</span>
          <span className="flex items-center gap-1.5"><Shirt size={13} />Suit mode</span>
        </span>
      </div>
    </div>

    <RosterGroup tone="critical" title="Needs help now" count={needsHelp.length} columns={rosterColumns}>
      {needsHelp.map(p => (
        <RosterRow key={p.seat} tone="critical" person={<RosterPerson name={p.name} seat={p.seat} tone="critical" />} cells={richCells(p)} eva={p.eva}
          aside={<span className="flex shrink-0 items-center gap-2">{helpers.map(h => <HelperPill key={h.seat} p={h} />)}<Button size="sm">Open case<ArrowUpRight size={14} /></Button></span>} />
      ))}
    </RosterGroup>

    <RosterGroup tone="warn" title="Watching" count={watching.length} hint="not in danger · Eva is following up">
      {watching.map(p => <RosterRow key={p.seat} person={<RosterPerson name={p.name} seat={p.seat} tone="warn" />} cells={richCells(p)} eva={p.eva} />)}
    </RosterGroup>

    <RosterGroup tone="ok" title="All good" count={allGood.length} hint="pulse and SpO₂ normal for each person">
      <div className="grid grid-cols-6 gap-2">
        {allGood.map(p => <PassengerChip key={p.seat} name={p.name} seat={p.seat} hr={p.hr} spo2={p.spo2} note={p.note} noteTone={helperRole[p.status] ? "info" : "muted"} mode={p.band} />)}
      </div>
    </RosterGroup>
  </Card>
)

/* ---------- deck plan ---------- */
const Deck = () => (
  <Card>
    <CardHeader>
      <CardTitle>Deck plan</CardTitle>
      <Badge tone="ok"><StatusDot tone="ok" />Mesh 15/15</Badge>
    </CardHeader>
    <DeckMap deck={deck} />
    <DeckMapLegend className="mt-3" />
  </Card>
)

/* ---------- Eva: floating assistant ---------- */
const caseSteps = [
  { t: "14:02", label: "No pulse", tone: "critical" },
  { t: "14:03", label: "CPR started", tone: "info" },
  { t: "14:04", label: "AED attached", tone: "critical" },
]
const stepDot = { critical: "bg-critical", info: "bg-info" }

const EvaAssistant = ({ state, onState }) => (
  <EvaDock state={state} onState={onState} subtitle="Guiding Leo through Ana's case" pill="Guiding Ana's case" unread={3} placeholder="Ask Eva or write to Leo"
    replies={[
      <ReplyChip key="open" tone="primary">Open case<ArrowUpRight size={13} /></ReplyChip>,
      <ReplyChip key="leo"><Phone size={13} />Call Leo</ReplyChip>,
      <ReplyChip key="earth"><Satellite size={13} />Message Earth</ReplyChip>,
    ]}
    pinned={<div className="flex flex-col gap-2 rounded-tile border border-critical/50 bg-critical/10 p-3">
      <div className="flex items-center gap-2"><StatusDot tone="critical" pulse /><span className="text-sm text-ink">Ana Ruiz · seat 4B · cabin</span><span className="ml-auto text-xs text-muted">4 min</span></div>
      <ol className="grid grid-cols-3 gap-1.5">
        {caseSteps.map(st => (
          <li key={st.t} className="flex flex-col gap-0.5 rounded-tile bg-background/60 px-2.5 py-1.5">
            <span className="flex items-center gap-1.5 font-mono text-xs text-muted"><span className={`size-1.5 rounded-full ${stepDot[st.tone]}`} />{st.t}</span>
            <span className="text-xs text-ink">{st.label}</span>
          </li>
        ))}
      </ol>
    </div>}>
    <EvaMessage label="What to do now" labelTone="critical">Keep CPR going. Stand clear when the AED says so.</EvaMessage>
    <EvaMessage label="Why">Ana's heart is not pumping. CPR and the AED give her the best chance.</EvaMessage>
    <EvaMessage from="person" name="Leo" time="14:04">Pads on. AED says shock advised.</EvaMessage>
    <EvaMessage label="How Eva decided">Her band: no pulse, no movement, no answer. No heart history. The AED decides shocks.</EvaMessage>
    <EvaMessage label="What comes next" time="reassess 14:34">Follow the AED's voice. Then check breathing and move her to the med bay.</EvaMessage>
  </EvaDock>
)

/* ---------- alerts: triage + the week + today, one card ---------- */
const todayAlerts = [
  { t: "14:02", who: "Ana Ruiz", what: "No pulse at the wrist", tone: "critical", state: "Open" },
  { t: "12:40", who: "Priya Nair", what: "Wrist ECG uneven", tone: "warn", state: "Review" },
  { t: "11:15", who: "Diego Luna", what: "Sensor base battery low", tone: "warn", state: "Watching" },
]
const AlertsCard = () => (
  <Card>
    <CardHeader><CardTitle>Alerts</CardTitle><PillSelect>This week</PillSelect></CardHeader>
    <div className="grid grid-cols-12 gap-6">
      <div className="col-span-5 flex flex-col items-center gap-4">
        <div className="relative">
          <HalfGauge size={240} thickness={32} segments={[{ value: 75, tone: "ink" }, { value: 8, tone: "info" }, { value: 17, hatched: true }]} />
          <div className="absolute inset-x-0 bottom-0 flex flex-col items-center">
            <span className="text-4xl tracking-tight text-ink">12</span>
            <span className="text-xs text-muted">this week</span>
          </div>
        </div>
        <div className="w-full"><Legend vertical items={[
          { label: "Solved on board", tone: "ink", value: 9 },
          { label: "In progress", tone: "info", value: 1 },
          { label: "Waiting for Earth", hatched: true, value: 2 },
        ]} /></div>
      </div>
      <div className="col-span-7 flex flex-col gap-4 border-l border-line pl-6">
        <div className="flex flex-wrap items-center gap-2">
                    <Badge><Droplet size={12} />Mostly water, sleep, battery</Badge>
          <InfoChip tone="info">Eva filtered 23 minor events</InfoChip>
        </div>
        <div className="pt-6"><Bars data={weekAlerts} labels={weekLabels} highlight={5} max={4} tagFormat={v => v} /></div>
      </div>
    </div>
    <div className="mt-5 flex flex-col gap-2 border-t border-line pt-4">
      <Eyebrow>Today</Eyebrow>
      {todayAlerts.map(a => (
        <div key={a.t} className="flex items-center gap-4 rounded-tile bg-raised px-4 py-2.5">
          <span className="font-mono text-sm text-muted">{a.t}</span>
          <StatusDot tone={a.tone} pulse={a.tone === "critical"} />
          <span className="w-32 text-sm text-ink">{a.who}</span>
          <span className="min-w-0 flex-1 truncate text-sm text-muted">{a.what}</span>
          <Badge tone={a.tone}>{a.state}</Badge>
        </div>
      ))}
    </div>
  </Card>
)

/* ---------- ship today: storm · bands · food, one card in three parts ---------- */
const stormRows = [
  { src: "NOAA", text: "Sun's X-rays low" },
  { src: "NASA", text: "No solar blast coming" },
  { src: "Shelter", text: "Ready · room for 15" },
]
const ShipToday = () => (
  <Card>
    <CardHeader><CardTitle>Ship today</CardTitle></CardHeader>
    <div className="grid grid-cols-3 divide-x divide-line">
      <div className="flex flex-col gap-3 pr-5">
        <div className="flex items-center gap-3"><IconTile src="img/icons/sun.webp" alt="The Sun" size="sm" /><span className="flex flex-col gap-1"><Eyebrow>Solar storm</Eyebrow><Badge tone="ok">Quiet</Badge></span></div>
        <ul className="flex flex-col gap-1.5">
          {stormRows.map(r => <li key={r.src} className="flex items-center gap-2 text-xs"><StatusDot tone="ok" /><span className="text-ink">{r.text}</span><span className="ml-auto text-muted">{r.src}</span></li>)}
        </ul>
      </div>
      <div className="flex flex-col gap-3 px-5">
        <Eyebrow>Wristbands</Eyebrow>
        <SplitBar a={80} b={20} aLabel="12 joined" bLabel="3" />
        <div className="flex items-center gap-4 text-xs text-muted"><span className="flex items-center gap-1.5"><Magnet size={13} />Joined</span><span className="flex items-center gap-1.5"><Shirt size={13} />Suit drill · to 14:30</span></div>
        <Legend vertical items={[
          { label: "Good signal", tone: "ok", value: 13 },
          { label: "Loose · Omar", tone: "warn", value: 1 },
          { label: "Battery low · Diego", tone: "warn", value: 1 },
        ]} />
      </div>
      <div className="flex flex-col gap-3 pl-5">
        <Eyebrow>Nutrition</Eyebrow>
        <div className="flex items-center gap-4">
          <Ring value={73} size={104} thickness={12} tone="ok"><span className="flex flex-col"><span className="text-xl tracking-tight text-ink">11/15</span><span className="text-xs text-muted">on track</span></span></Ring>
          <ul className="flex flex-col gap-2 text-xs text-muted">
            <li className="flex items-center gap-1.5"><Droplet size={13} className="text-info" /><span className="text-ink">3</span> need water</li>
            <li className="flex items-center gap-1.5"><Utensils size={13} /><span className="text-ink">1</span> skipped lunch</li>
            <li className="flex items-center gap-1.5"><Timer size={13} />Next meal 18:00</li>
          </ul>
        </div>
      </div>
    </div>
  </Card>
)

/* ---------- supplies with stowage ---------- */
const supplies = [
  { name: "AEDs", count: "2/2", where: "Corridor B · D", src: "img/icons/aed.webp", used: "B is with Ana · 14:04" },
  { name: "AED pads", count: "5/6", where: "Emergency station B", src: "img/icons/aed-pads.webp", used: "1 set used · 14:04" },
  { name: "Sensor parts", count: "6/6", where: "Med bay · B2", src: "img/icons/sensor-base.webp" },
  { name: "Band screens", count: "4/4", where: "Med bay · B3", src: "img/icons/interface.webp" },
]
const SupplyRow = ({ s }) => (
  <li className={s.used ? "flex items-center gap-3 rounded-tile border border-info/40 bg-info/5 p-2.5" : "flex items-center gap-3 rounded-tile bg-raised p-2.5"}>
    <IconTile src={s.src} alt="" size="sm" />
    <div className="flex min-w-0 flex-1 flex-col">
      <span className="text-sm text-ink">{s.name}</span>
      <span className="flex items-center gap-1 truncate text-xs text-muted"><MapPin size={12} />{s.where}</span>
      {s.used && <span className="truncate text-xs text-info">{s.used}</span>}
    </div>
    <span className="text-base text-ink tabular-nums">{s.count}</span>
  </li>
)
const SuppliesCard = () => (
  <Card>
    <CardHeader><CardTitle>Supplies</CardTitle><Badge tone="warn">Restock · AED pads</Badge></CardHeader>
    <ul className="flex flex-col gap-2">{supplies.map(s => <SupplyRow key={s.name} s={s} />)}</ul>
  </Card>
)

/* ---------- Earth sync queue ---------- */
const SyncCard = () => (
  <Card className="flex flex-col gap-4">
    <div className="flex items-center justify-between gap-3">
      <div className="flex items-center gap-3"><IconTile src="img/icons/earth-link.webp" alt="Earth link" size="sm" /><CardTitle>Earth sync</CardTitle></div>
      <Badge tone="info">Link open</Badge>
    </div>
    <Pipeline steps={[
      { label: "Saved", count: 2, hint: "Next slot 14:07", style: "raised" },
      { label: "Sent", count: 1, hint: "≈ 13 min", style: "soft" },
      { label: "Confirmed", count: 41, hint: "Today", style: "ink" },
    ]} />
    <ul className="flex flex-col divide-y divide-line">
      {syncQueue.slice(0, 4).map(r => (
        <li key={r.what} className="flex items-center gap-3 py-2 text-xs">
          <span className="font-mono text-muted">{r.ship}</span>
          <span className="min-w-0 flex-1 truncate text-ink">{r.what}</span>
          <Badge tone={queueBadge[r.state].tone}>{queueBadge[r.state].label}</Badge>
        </li>
      ))}
    </ul>
  </Card>
)

/* ---------- screen ---------- */
export function CrewDashboard() {
  const [eva, setEva] = useState("docked")
  return (
    <AppWindow title="Eva · ship hub · offline" meta="Day 96 of 212 · ship time 14:06" footer={<Citation />}>
      <div className="flex">
        <Rail active="Overview" />
        <div className="flex min-w-0 flex-1 flex-col">
          <TopBar search="Search people, alerts, records" actions={["Check-in", "Note", "Drill", "Message Earth"]} person={crewLead} />
          <main className="flex flex-col gap-6 p-6">
            <div className="flex items-end justify-between gap-6">
              <div className="flex flex-col gap-2">
                <Eyebrow>Saturday · day 96 of 212 · ship time 14:06</Eyebrow>
                <Headline as="h1" className="text-4xl">Good afternoon, Mara</Headline>
              </div>
              <div className="flex flex-wrap justify-end gap-2">
                <InfoChip tone="ok" src="img/icons/hub.webp">Hub online · offline-ready</InfoChip>
                <InfoChip tone="ok" src="img/icons/lora.webp">Mesh · 15 of 15 bands</InfoChip>
                <InfoChip tone="info" src="img/icons/earth-link.webp">Earth link · next slot 14:07</InfoChip>
              </div>
            </div>

            <StatusStrip />

            <Roster />

            <div className="grid grid-cols-12 items-start gap-6">
              <div className="col-span-8 flex flex-col gap-6">
                <Deck />
                <AlertsCard />
                <ShipToday />
              </div>
              <div className="col-span-4 flex flex-col gap-6">
                <SuppliesCard />
                <SyncCard />
              </div>
            </div>
          </main>
          {/* Eva floats over the content, anchored bottom-right; sticky so it follows the viewer down the window */}
          <div className="pointer-events-none sticky bottom-6 z-30 h-0">
            <div className="pointer-events-auto absolute right-6 bottom-0"><EvaAssistant state={eva} onState={setEva} /></div>
          </div>
        </div>
      </div>
    </AppWindow>
  )
}
