import { Avatar, Badge, Body, Button, Card, CardHeader, CardTitle, Eyebrow, IconTile, Progress, Segmented, Sparkline, StatusDot } from "@/ds"
import { AppWindow, EcgStrip, InfoChip, Readout, TextBox, Timeline } from "@/ds/dashboard-extra"
import { Clock, Send, Rocket, ShieldCheck } from "lucide-react"
import { Citation, Rail, TopBar } from "./shared"
import { anaCase, surgeon, trends } from "./data"

const DelayBanner = () => (
  <Card tone="info" className="flex items-center gap-8 px-6">
    <span className="grid size-14 shrink-0 place-items-center rounded-full bg-info text-on-light"><Clock size={26} /></span>
    <div className="flex min-w-0 flex-1 flex-col gap-1">
      <p className="text-2xl tracking-tight text-ink">You are seeing the ship as it was 12 min 40 s ago</p>
      <Body muted>Last packet: ship time 14:04:12, received here 14:16:52. Anything newer is still on its way.</Body>
      <p className="text-xs text-muted">The crew does not wait for you: Eva guides them now. Without decision support, a ground answer can take 45 min to several hours to reach the crew (Russell et al., 2023).</p>
    </div>
    <div className="flex w-96 shrink-0 items-center gap-3">
      <span className="flex flex-col items-center gap-1"><span className="grid size-12 place-items-center rounded-full bg-raised text-ink"><Rocket size={20} /></span><span className="text-xs text-muted">Ship</span></span>
      <div className="flex flex-1 flex-col gap-2">
        <Progress value={68} size="sm" />
        <span className="text-center text-xs text-muted">Next packet arrives in about 4 min</span>
      </div>
      <span className="flex flex-col items-center gap-1"><IconTile src="img/pe-earth.webp" alt="Earth" size="sm" /><span className="text-xs text-muted">You</span></span>
    </div>
  </Card>
)

const CaseCard = () => (
  <Card className="col-span-7">
    <CardHeader>
      <div className="flex items-center gap-3">
        <Avatar name="Ana Ruiz" tone="critical" />
        <div className="flex flex-col"><CardTitle>Ana Ruiz · seat 4B</CardTitle><Body muted>Case 0414 · no pulse at wrist · opened 14:02 ship time · no heart history</Body></div>
      </div>
      <Badge tone="critical"><StatusDot tone="critical" pulse />Open on the ship</Badge>
    </CardHeader>
    <Timeline items={anaCase} />
    <div className="mt-4 flex items-center justify-between border-t border-line pt-4">
      <span className="flex items-center gap-2 text-xs text-muted"><ShieldCheck size={15} />Record chain checked · 5 of 5 steps intact</span>
      <Button size="sm" variant="outline">Acknowledge 5 records</Button>
    </div>
  </Card>
)

const Composer = () => (
  <Card className="col-span-5 flex flex-col gap-4">
    <div className="flex items-start justify-between gap-3">
      <div className="flex flex-col gap-1"><CardTitle>Message the ship</CardTitle><Body muted>They will read it after the next link slot</Body></div>
      <Segmented items={["Text", "Voice"]} active="Text" />
    </div>
    <div className="flex flex-wrap items-center gap-2">
      <Eyebrow>To</Eyebrow>
      <InfoChip tone="info">Mara · crew lead</InfoChip>
      <InfoChip tone="info">Leo · helper</InfoChip>
      <InfoChip tone="info">Tomás · AED</InfoChip>
    </div>
    <div className="flex flex-col gap-2">
      <Eyebrow>Last from the ship</Eyebrow>
      <div className="flex items-start gap-3 rounded-tile bg-raised p-3">
        <Avatar name="Mara Okoye" size="sm" />
        <div className="flex min-w-0 flex-col gap-0.5">
          <p className="text-sm text-ink">Ana collapsed in the galley, no pulse. Leo is doing CPR, Tomás brought the AED. Eva is guiding.</p>
          <p className="font-mono text-xs text-muted">ship 14:03:50 · received 14:16:30</p>
        </div>
      </div>
    </div>
    <TextBox>Mara, Leo, Tomás: you did the right thing. Keep CPR going and follow the AED's voice. Check her breathing every 2 minutes and send me the AED log.</TextBox>
    <div className="flex flex-wrap gap-2">
      <Button size="sm" variant="secondary">Follow the AED</Button>
      <Button size="sm" variant="secondary">Send AED log</Button>
      <Button size="sm" variant="secondary">Rest in med bay</Button>
    </div>
    <div className="mt-auto flex items-center gap-4 rounded-tile bg-raised p-4">
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <p className="flex items-center gap-2 text-base text-ink"><Clock size={16} className="shrink-0 text-info" />Your message arrives in about 13 min</p>
        <p className="text-xs text-muted">Earliest reply: about 26 min from now</p>
      </div>
      <Button size="sm">Send<Send size={14} /></Button>
    </div>
  </Card>
)

/* Priya's 30 s wrist ECG: uneven gaps between beats */
const priyaRR = [0.6, 0.98, 0.52, 1.12, 0.72, 0.56, 1.2, 0.64, 0.9, 0.55, 1.05, 0.7]

const WristEcg = () => (
  <Card>
    <CardHeader>
      <div className="flex items-center gap-3">
        <Avatar name="Priya Nair" tone="warn" />
        <div className="flex flex-col"><CardTitle>Priya Nair · seat 4A · wrist ECG, 30 s</CardTitle><Body muted>Ship time 12:40:10 · received 12:52:50 · one finger of each hand on the two metal pads, sitting still</Body></div>
      </div>
      <div className="flex items-center gap-2"><Badge>1-lead wrist ECG (two pads) · not a 12-lead ECG</Badge><Badge tone="warn">Irregular, review</Badge></div>
    </CardHeader>
    <div className="rounded-tile bg-raised px-4 pt-4 pb-2"><EcgStrip rr={priyaRR} tone="ink" /></div>
    <div className="mt-4 grid grid-cols-12 items-center gap-5">
      <div className="col-span-3 flex flex-col"><span className="text-sm text-ink">Pulse 58–108, uneven</span><span className="text-xs text-muted">Gaps between beats keep changing</span></div>
      <div className="col-span-5 flex flex-col"><span className="text-sm text-ink">A wrist check on demand, not continuous</span><span className="text-xs text-muted">Shocks are decided by the AED, never by Eva</span></div>
      <div className="col-span-4 flex justify-end gap-2">
        <Button size="sm" variant="outline">Ask for a repeat check</Button>
        <Button size="sm">Mark reviewed</Button>
      </div>
    </div>
  </Card>
)

const Trends = () => (
  <Card>
    <CardHeader>
      <div className="flex flex-col gap-1"><CardTitle>Trends since 08:00</CardTitle><Body muted>Shaded band = normal for that person. Line = their last 6 hours.</Body></div>
      <div className="w-80"><Segmented items={["Heart", "Oxygen", "Skin", "Sleep"]} active="Heart" /></div>
    </CardHeader>
    <div className="grid grid-cols-3 gap-3">
      {trends.map(t => (
        <div key={t.name} className="flex flex-col gap-3 rounded-tile bg-raised p-4">
          <div className="flex items-center gap-3">
            <Avatar name={t.name} size="sm" tone={t.tone === "critical" ? "critical" : "neutral"} />
            <div className="flex min-w-0 flex-1 flex-col"><span className="truncate text-sm text-ink">{t.name}</span><span className="text-xs text-muted">Seat {t.seat}</span></div>
            <Readout value={t.value} unit={t.unit} tone={t.tone === "critical" ? "critical" : "ink"} />
          </div>
          <Sparkline data={t.data} band={t.band} tone={t.tone} height={56} />
          <span className="text-xs text-muted">{t.note}</span>
        </div>
      ))}
    </div>
  </Card>
)

export function EarthView() {
  return (
    <AppWindow title="Eva · Earth console · flight surgeon" meta="Earth time 14:16:52 · ship data as of 14:04:12" footer={<Citation />}>
      <div className="flex">
        <Rail active="Cases" />
        <div className="flex min-w-0 flex-1 flex-col">
          <TopBar search="Search people, cases, records" actions={["Acknowledge", "Note", "Message ship"]} person={surgeon} />
          <main className="flex flex-col gap-5 p-6">
            <DelayBanner />
            <div className="grid grid-cols-12 items-stretch gap-5">
              <CaseCard />
              <Composer />
            </div>
            <WristEcg />
            <Trends />
          </main>
        </div>
      </div>
    </AppWindow>
  )
}
