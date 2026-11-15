import { Avatar, Badge, Bars, Body, Button, Card, CardHeader, CardTitle, Eyebrow, HalfGauge, Headline, IconTile, PillSelect, Ring, Segmented, SplitBar, Stat, StatusDot } from "@/ds"
import { AppWindow, DataTable, DeckPlan, InfoChip, Legend, MiniMeter, Pipeline, Readout } from "@/ds/dashboard-extra"
import { Users, Siren, Satellite, Radiation, Magnet, Shirt, Sparkles, ArrowUpRight, Phone, Timer, MapPin, Zap } from "lucide-react"
import { Citation, Rail, TopBar } from "./shared"
import { crewLead, roster, syncQueue, weekAlerts, weekLabels } from "./data"

/* ---------- lookups (static class strings only) ---------- */
const statusBadge = {
  care: { tone: "critical", label: "In care" },
  helping: { tone: "info", label: "Helping Ana" },
  aed: { tone: "info", label: "Brought the AED" },
  ok: { tone: "ok", label: "Normal" },
  battery: { tone: "warn", label: "Battery low" },
  suit: { tone: "neutral", label: "Suit drill" },
  ecg: { tone: "warn", label: "ECG review" },
}
const avatarTone = { care: "critical", helping: "info", aed: "info", ok: "neutral", battery: "warn", suit: "neutral", ecg: "warn" }
const queueBadge = { saved: { tone: "neutral", label: "Saved" }, sent: { tone: "info", label: "Sent" }, ack: { tone: "ok", label: "Acknowledged" } }

/* ---------- KPI card ---------- */
const Kpi = ({ icon: I, ...stat }) => (
  <Card className="flex items-start justify-between">
    <Stat {...stat} />
    <span className="grid size-10 place-items-center rounded-full bg-raised text-muted"><I size={18} /></span>
  </Card>
)

/* ---------- roster ---------- */
const columns = [
  { key: "seat", label: "Seat" },
  { key: "who", label: "Passenger" },
  { key: "status", label: "Status" },
  { key: "hr", label: "Heart bpm", align: "right" },
  { key: "spo2", label: "Oxygen", align: "right" },
  { key: "skin", label: "Skin °C", align: "right" },
  { key: "ecg", label: "Wrist ECG" },
  { key: "band", label: "Band · battery" },
]

const rows = roster.map(p => {
  const b = statusBadge[p.status]
  const care = p.status === "care"
  return {
    id: p.seat,
    tone: care ? "critical" : "default",
    cells: {
      seat: <span className="font-mono text-xs text-muted">{p.seat}</span>,
      who: <span className="flex items-center gap-3"><Avatar name={p.name} tone={avatarTone[p.status]} size="sm" /><span className="text-sm text-ink">{p.name}</span></span>,
      status: <Badge tone={b.tone}>{care && <StatusDot tone="critical" pulse />}{b.label}</Badge>,
      hr: p.hr == null ? <Readout value="No pulse" tone="critical" /> : <Readout value={p.hr} tone={care ? "critical" : "ink"} />,
      spo2: <Readout value={p.spo2 == null ? "—" : `${p.spo2}%`} tone={care ? "critical" : "ink"} />,
      skin: <Readout value={p.skin.toFixed(1)} />,
      ecg: !p.ecg
        ? <span className="text-xs text-faint">None today</span>
        : p.ecg.result === "regular"
          ? <span className="text-xs text-muted"><span className="font-mono">{p.ecg.t}</span> · regular</span>
          : <span className="text-xs text-warn"><span className="font-mono">{p.ecg.t}</span> · irregular, review</span>,
      band: (
        <span className="inline-flex items-center gap-2">
          {p.band === "joined"
            ? <span className="inline-flex w-14 items-center gap-1 text-xs text-muted"><Magnet size={13} />Joined</span>
            : <span className="inline-flex w-14 items-center gap-1 text-xs text-ink"><Shirt size={13} />Suit</span>}
          <MiniMeter value={p.battery} tone={p.battery < 20 ? "warn" : "ok"} />
        </span>
      ),
    },
  }
})

const Roster = () => (
  <Card>
    <CardHeader>
      <div className="flex flex-col gap-1">
        <CardTitle>Passengers now</CardTitle>
        <Body muted>15 on board · live from the mesh · 3 s ago · oxygen is an estimate, best at rest</Body>
      </div>
      <div className="w-72 shrink-0"><Segmented items={["All", "Needs help", "Suit mode"]} active="All" /></div>
    </CardHeader>
    <DataTable columns={columns} rows={rows} />
  </Card>
)

/* ---------- Eva assistant ---------- */
const caseSteps = [
  { t: "14:02", label: "No pulse at wrist", tone: "critical" },
  { t: "14:03", label: "Leo arrived, CPR", tone: "info" },
  { t: "14:04", label: "AED attached, shock advised by AED", tone: "critical" },
]

// Eva explains itself: what to do now · why · how Eva decided · what comes next
const evaPlan = [
  { k: "What to do now", v: "Keep CPR going. Stand clear when the AED says so." },
  { k: "Why", v: "Ana's heart is not pumping. CPR and the AED give her the best chance." },
  { k: "How Eva decided", v: "Wristband: no pulse, no movement, no answer. Her history: no heart disease." },
  { k: "What comes next", v: "Follow the AED's voice. Then check breathing and move her to the med bay." },
]

const EvaCard = () => (
  <Card tone="info" className="col-span-5">
    <CardHeader>
      <div className="flex items-center gap-3">
        <span className="grid size-11 place-items-center rounded-full bg-info text-on-light"><Sparkles size={20} /></span>
        <div className="flex flex-col"><CardTitle>Eva</CardTitle><Body muted>Guiding Leo through Ana's case</Body></div>
      </div>
      <Badge tone="info"><StatusDot tone="info" />Guiding</Badge>
    </CardHeader>
    <div className="flex flex-col gap-4 rounded-tile border border-critical/50 bg-critical/10 p-4">
      <div className="flex items-center gap-3">
        <StatusDot tone="critical" pulse />
        <p className="text-sm text-ink">Ana Ruiz · seat 4B · galley</p>
        <span className="ml-auto text-xs text-muted">case open 4 min</span>
      </div>
      <ol className="grid grid-cols-3 gap-2">
        {caseSteps.map(s => (
          <li key={s.t} className="flex flex-col gap-1 rounded-tile bg-background/60 p-3">
            <span className="font-mono text-xs text-muted">{s.t}</span>
            <span className="text-xs leading-snug text-ink">{s.label}</span>
          </li>
        ))}
      </ol>
      <dl className="grid grid-cols-2 gap-x-4 gap-y-3">
        {evaPlan.map(b => (
          <div key={b.k} className="flex flex-col gap-1">
            <dt className="text-xs uppercase tracking-label text-muted">{b.k}</dt>
            <dd className="text-sm leading-snug text-ink">{b.v}</dd>
          </div>
        ))}
      </dl>
      <div className="flex items-center gap-2 rounded-tile bg-background/60 px-3 py-2">
        <Timer size={15} className="text-info" />
        <span className="text-sm text-ink">Ana · reassess in 30 min</span>
        <span className="ml-auto font-mono text-xs text-muted">14:34</span>
      </div>
      <div className="flex items-center gap-2">
        <Button size="sm">Open case<ArrowUpRight size={14} /></Button>
        <Button size="sm" variant="outline"><Phone size={14} />Call Leo</Button>
      </div>
    </div>
    <div className="mt-4 flex items-center gap-3 rounded-tile bg-background/60 p-3">
      <StatusDot tone="warn" />
      <div className="flex min-w-0 flex-1 flex-col">
        <p className="text-sm text-ink">ECG check requested · Priya Nair</p>
        <p className="text-xs text-muted">Wrist ECG at 12:40 looked irregular · sent to Earth</p>
      </div>
      <Badge tone="warn">Review</Badge>
    </div>
    <div className="mt-4 flex items-center gap-3">
      <IconTile size="sm"><Zap size={20} className="text-ink" /></IconTile>
      <Body muted>Shocks are decided by the AED at the emergency station, never by Eva. The AED reads the heart through its own pads. Eva guides, records and warns.</Body>
    </div>
  </Card>
)

/* ---------- triage gauge ---------- */
const Triage = () => (
  <Card className="col-span-4">
    <CardHeader><CardTitle>Alert triage</CardTitle><PillSelect>This week</PillSelect></CardHeader>
    <div className="flex flex-col items-center gap-5">
      <div className="relative">
        <HalfGauge size={280} thickness={38} segments={[{ value: 75, tone: "ink" }, { value: 8, tone: "info" }, { value: 17, hatched: true }]} />
        <div className="absolute inset-x-0 bottom-0 flex flex-col items-center">
          <span className="text-4xl tracking-tight text-ink">12</span>
          <span className="text-xs text-muted">alerts this week</span>
        </div>
      </div>
      <InfoChip tone="info">Filtered by Eva: 23 minor events today</InfoChip>
      <div className="w-full border-t border-line pt-4">
        <Legend vertical items={[
          { label: "Resolved on board", tone: "ink", value: 9 },
          { label: "In progress", tone: "info", value: 1 },
          { label: "Waiting for Earth", hatched: true, value: 2 },
        ]} />
      </div>
    </div>
  </Card>
)

/* ---------- supplies with stowage ---------- */
const supplies = [
  { name: "AEDs", count: "2/2", where: "At stations · corridor B, corridor D", icon: Zap, used: "Corridor B's AED is with Ana · 14:04" },
  { name: "AED pads", count: "5/6", where: "Emergency station · corridor B", icon: Zap, used: "1 set used · Ana · 14:04" },
  { name: "Spare sensor parts", count: "6/6", where: "Med bay · drawer B2", src: "img/inv-sensor.webp" },
  { name: "Spare band screens", count: "4/4", where: "Med bay · drawer B3", src: "img/inv-screen.webp" },
]
const SupplyRow = ({ s }) => (
  <li className={s.used ? "flex items-center gap-3 rounded-tile border border-info/40 bg-info/5 p-2.5" : "flex items-center gap-3 rounded-tile bg-raised p-2.5"}>
    {s.icon ? <IconTile size="sm"><s.icon size={20} className="text-ink" /></IconTile> : <IconTile src={s.src} alt="" size="sm" />}
    <div className="flex min-w-0 flex-1 flex-col">
      <span className="flex items-baseline gap-2 text-sm text-ink">{s.name}<span className="tabular-nums text-muted">{s.count}</span></span>
      <span className="flex items-center gap-1 text-xs text-muted"><MapPin size={12} />{s.where}</span>
      {s.used && <span className="text-xs text-info">{s.used}</span>}
    </div>
  </li>
)
const SuppliesCard = () => (
  <Card>
    <CardHeader>
      <div className="flex flex-col gap-1"><CardTitle>Supplies</CardTitle><Body muted>What is left, and where it is stowed</Body></div>
      <Badge>Restock 1 · AED pads</Badge>
    </CardHeader>
    <ul className="grid grid-cols-2 gap-2">{supplies.map(s => <SupplyRow key={s.name} s={s} />)}</ul>
  </Card>
)

/* ---------- solar storm watch ---------- */
const stormRows = [
  { src: "NOAA GOES", text: "Sun's X-rays: low", tone: "ok" },
  { src: "NASA DONKI", text: "No solar blast coming", tone: "ok" },
  { src: "Shelter", text: "Ready · room for all 15", tone: "ok" },
]
const StormCard = () => (
  <Card className="flex items-center gap-5">
    <IconTile src="img/ps-sun.webp" alt="The Sun" size="lg" />
    <div className="flex min-w-0 flex-1 flex-col gap-3">
      <div className="flex items-center justify-between"><CardTitle>Solar storm watch</CardTitle><Badge tone="ok">Quiet</Badge></div>
      <ul className="flex flex-col gap-1.5">
        {stormRows.map(r => (
          <li key={r.src} className="flex items-center gap-2 text-xs"><StatusDot tone={r.tone} /><span className="w-20 shrink-0 text-faint">{r.src}</span><span className="text-ink">{r.text}</span></li>
        ))}
      </ul>
    </div>
  </Card>
)

/* ---------- deck plan ---------- */
const rooms = [
  { x: 120, y: 70, w: 330, h: 120, label: "Cabins 1–4" },
  { x: 120, y: 230, w: 330, h: 120, label: "Cabins 5–8" },
  { x: 480, y: 70, w: 170, h: 120, label: "Galley" },
  { x: 480, y: 230, w: 170, h: 120, label: "Storm shelter", kind: "shelter" },
  { x: 680, y: 70, w: 160, h: 120, label: "Med bay" },
  { x: 680, y: 230, w: 160, h: 120, label: "Suit-up" },
  { x: 860, y: 160, w: 80, h: 100, label: "Bridge" },
]
const people = [
  { id: "1A", x: 170, y: 120, tone: "ok" }, { id: "1B", x: 636, y: 168, tone: "info" }, { id: "3A", x: 330, y: 110, tone: "ok" }, { id: "4A", x: 405, y: 160, tone: "ok" },
  { id: "5A", x: 175, y: 295, tone: "warn" }, { id: "5B", x: 255, y: 270, tone: "ok" }, { id: "6A", x: 340, y: 315, tone: "ok" }, { id: "7A", x: 410, y: 280, tone: "ok" },
  { id: "4B", x: 615, y: 128, tone: "critical" }, { id: "3B", x: 535, y: 150, tone: "info" },
  { id: "7B", x: 805, y: 112, tone: "ok" },
  { id: "2A", x: 720, y: 290, tone: "ok" }, { id: "2B", x: 770, y: 310, tone: "ok" }, { id: "6B", x: 800, y: 270, tone: "ok" },
  { id: "8A", x: 900, y: 225, tone: "ok" },
]
const stations = [
  { x: 465, y: 290, ly: 373, label: "AED · B" },
  { x: 665, y: 290, ly: 373, label: "AED · D" },
]
const deckLabels = [
  { x: 555, y: 66, w: 120, text: "Ana · 4B", tone: "critical" },
  { x: 470, y: 184, w: 120, text: "Leo · CPR" },
  { x: 658, y: 170, w: 150, text: "Tomás · AED" },
]

const Deck = () => (
  <Card>
    <CardHeader>
      <div className="flex flex-col gap-1"><CardTitle>Deck plan</CardTitle><Body muted>Where everyone is, from their band's mesh signal</Body></div>
      <Badge tone="ok"><StatusDot tone="ok" />Mesh 15/15</Badge>
    </CardHeader>
    <DeckPlan rooms={rooms} people={people} hub={{ x: 72, y: 210, label: "Hub" }} labels={deckLabels} stations={stations} />
    <div className="mt-3 flex flex-wrap items-center justify-between gap-y-2">
      <Legend items={[{ label: "OK", tone: "ok" }, { label: "Low battery", tone: "warn" }, { label: "Helper", tone: "info" }, { label: "In care", tone: "critical" }, { label: "Storm shelter", hatched: true }]} />
      <span className="flex items-center gap-2 text-xs text-faint"><Zap size={13} className="text-ink" />AED at emergency station · dotted lines: LoRa mesh to the hub</span>
    </div>
  </Card>
)

/* ---------- alerts this week ---------- */
const WeekBars = () => (
  <Card className="col-span-4">
    <CardHeader>
      <div className="flex flex-col gap-1"><CardTitle>Alerts this week</CardTitle><Body muted>Most were small: water, sleep, battery</Body></div>
    </CardHeader>
    <div className="flex items-baseline gap-3"><span className="text-4xl tracking-tight text-ink">12</span><span className="text-sm text-muted">alerts · 3 today · 9 solved on board</span></div>
    <div className="pt-10"><Bars data={weekAlerts} labels={weekLabels} highlight={5} max={4} tagFormat={v => v} /></div>
  </Card>
)

/* ---------- wristbands split ---------- */
const BandsCard = () => (
  <Card className="flex flex-col gap-5">
    <div className="flex flex-col gap-1"><CardTitle>Wristbands</CardTitle><Body muted>Both parts joined, or the suit between them</Body></div>
    <SplitBar a={80} b={20} aLabel="Joined 80%" bLabel="" />
    <div className="grid grid-cols-2 gap-4">
      <div className="flex flex-col gap-1"><span className="flex items-center gap-1.5 text-sm text-ink"><Magnet size={14} />12 joined</span><span className="text-xs text-muted">Big battery powers the sensors</span></div>
      <div className="flex flex-col gap-1"><span className="flex items-center gap-1.5 text-sm text-ink"><Shirt size={14} />3 suit mode · 20%</span><span className="text-xs text-muted">Sensors run on their small battery</span></div>
    </div>
    <p className="text-xs text-muted">Suit drill until 14:30: Yuki, Omar, Sofia</p>
    <div className="flex flex-col gap-2 border-t border-line pt-4">
      <Eyebrow>Signal quality</Eyebrow>
      <Legend vertical items={[
        { label: "Good signal", tone: "ok", value: 13 },
        { label: "Weak · band loose (Omar, suit)", tone: "warn", value: 1 },
        { label: "Battery low (Diego, 14%)", tone: "warn", value: 1 },
      ]} />
    </div>
    <p className="text-xs text-faint">Each band watches all the time: optical pulse (uneven rhythm, loss of pulse), SpO₂, breathing estimate, skin temperature, EDA and movement. ECG is a 30 s check on demand. Both parts have their own battery and LoRa radio, so the sensor base keeps reporting if the screen is off.</p>
  </Card>
)

/* ---------- nutrition ring ---------- */
const NutritionCard = () => (
  <Card className="flex flex-col gap-4">
    <CardTitle>Nutrition plan</CardTitle>
    <div className="flex items-center gap-5">
      <Ring value={73} size={128} thickness={14} tone="ok"><span className="flex flex-col"><span className="text-2xl tracking-tight text-ink">11/15</span><span className="text-xs text-muted">on track</span></span></Ring>
      <ul className="flex flex-col gap-2 text-xs text-muted">
        <li><span className="text-ink">3</span> need more water</li>
        <li><span className="text-ink">1</span> skipped lunch</li>
      </ul>
    </div>
    <p className="text-xs text-muted">Next meal 18:00 · Eva reminds each person on their band</p>
  </Card>
)

/* ---------- Earth sync queue ---------- */
const SyncCard = () => (
  <Card className="col-span-8 flex flex-col gap-4">
    <div className="flex items-start justify-between gap-3">
      <div className="flex items-center gap-3">
        <IconTile src="img/pe-earth.webp" alt="Earth" size="sm" />
        <div className="flex flex-col"><CardTitle>Earth sync queue</CardTitle><Body muted>Collected, collated and sent automatically</Body></div>
      </div>
      <Badge tone="info">Link open</Badge>
    </div>
    <Pipeline steps={[
      { label: "Saved", count: 2, hint: "On the hub, waiting for a link slot", style: "raised" },
      { label: "Sent", count: 1, hint: "Travelling · about 13 min", style: "soft" },
      { label: "Acknowledged", count: 41, hint: "Earth confirmed today", style: "ink" },
    ]} />
    <ul className="flex flex-col divide-y divide-line">
      {syncQueue.map(r => (
        <li key={r.what} className="flex items-center gap-3 py-2 text-xs">
          <span className="font-mono text-muted">{r.ship}</span>
          <span className="min-w-0 flex-1 truncate text-ink">{r.what}</span>
          <span className="text-faint">{r.note}</span>
          <Badge tone={queueBadge[r.state].tone}>{queueBadge[r.state].label}</Badge>
        </li>
      ))}
    </ul>
  </Card>
)

/* ---------- screen ---------- */
export function CrewDashboard() {
  return (
    <AppWindow title="Eva · ship hub · offline" meta="Day 96 of 212 · ship time 14:06" footer={<Citation />}>
      <div className="flex">
        <Rail active="Overview" />
        <div className="flex min-w-0 flex-1 flex-col">
          <TopBar search="Search people, alerts, records" actions={["Check-in", "Note", "Drill", "Message Earth"]} person={crewLead} />
          <main className="flex flex-col gap-5 p-6">
            <div className="flex items-end justify-between gap-6">
              <div className="flex flex-col gap-2">
                <Eyebrow>Saturday · day 96 of 212 · ship time 14:06</Eyebrow>
                <Headline as="h1" className="text-4xl">Good afternoon, Mara</Headline>
                <Body muted>One person needs help now. Everyone else is fine.</Body>
              </div>
              <div className="flex flex-wrap justify-end gap-2">
                <InfoChip tone="ok">Hub online · works offline</InfoChip>
                <InfoChip tone="ok">Mesh · 15 of 15 bands</InfoChip>
                <InfoChip tone="info">Earth link · next slot 14:07</InfoChip>
              </div>
            </div>

            <div className="grid grid-cols-4 gap-5">
              <Kpi icon={Users} label="Passengers OK" value="14" unit="/ 15" delta="−1 since 14:02" deltaTone="neutral" hint="Ana is in care" />
              <Kpi icon={Siren} label="Open alerts" value="1" delta="No pulse" deltaTone="critical" hint="CPR · AED attached" />
              <Kpi icon={Satellite} label="Earth delay" value="12" unit="min 40 s" delta="+9 s today" deltaTone="neutral" hint="Reply ≈ 25 min" />
              <Kpi icon={Radiation} label="Cabin dose today" value="98" unit="% of plan" delta="As expected" deltaTone="ok" hint="No shelter needed" />
            </div>

            <div className="grid grid-cols-12 gap-5">
              <EvaCard />
              <div className="col-span-7 flex flex-col gap-5">
                <Deck />
                <SuppliesCard />
              </div>
            </div>

            <Roster />

            <div className="grid grid-cols-12 gap-5">
              <Triage />
              <WeekBars />
              <div className="col-span-4 flex flex-col gap-5">
                <StormCard />
                <NutritionCard />
              </div>
            </div>

            <div className="grid grid-cols-12 gap-5">
              <SyncCard />
              <div className="col-span-4 flex flex-col"><BandsCard /></div>
            </div>
          </main>
        </div>
      </div>
    </AppWindow>
  )
}
