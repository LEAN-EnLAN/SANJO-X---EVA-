import { Eyebrow, Headline, Body, Button, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Stat, Progress, SplitBar, Ring, HalfGauge, Sparkline, Bars, Avatar, ListRow, IconTile, Check, PillSelect, Segmented } from "@/ds"
import { Bell, Plus, HeartPulse } from "lucide-react"
import MobileScreens from "@/screens/mobile"
import { CrewDashboard, EarthView } from "@/screens/dashboard"

const Section = ({ id, eyebrow, title, children }) => (
  <section id={id} className="flex flex-col gap-8 border-t border-line px-6 py-16 lg:px-12">
    <div className="flex max-w-3xl flex-col gap-3"><Eyebrow>{eyebrow}</Eyebrow><Headline>{title}</Headline></div>
    {children}
  </section>
)

function DesignSystem() {
  const sw = [["background", "bg-background"], ["surface", "bg-surface"], ["raised", "bg-raised"], ["elevated", "bg-elevated"], ["line", "bg-line"], ["ink", "bg-ink"], ["muted", "bg-muted"], ["brand", "bg-brand"], ["critical", "bg-critical"], ["warn", "bg-warn"], ["ok", "bg-ok"], ["info", "bg-info"]]
  return (
    <div className="grid gap-6 lg:grid-cols-3">
      <Card>
        <CardHeader><CardTitle>Colour tokens</CardTitle><Badge tone="info">theme.css</Badge></CardHeader>
        <div className="grid grid-cols-4 gap-3">{sw.map(([n, c]) => <div key={n} className="flex flex-col gap-1"><span className={`h-12 rounded-tile border border-line ${c}`} /><span className="text-xs text-muted">{n}</span></div>)}</div>
      </Card>
      <Card>
        <CardHeader><CardTitle>Type</CardTitle><Badge>Inter</Badge></CardHeader>
        <div className="flex flex-col gap-3"><Eyebrow>Eyebrow · tracked label</Eyebrow><Headline as="p">Hello Ana</Headline><Body>Body · plain words, one idea per line.</Body><Body muted>Muted · the read-out next to a number.</Body></div>
      </Card>
      <Card>
        <CardHeader><CardTitle>Actions</CardTitle></CardHeader>
        <div className="flex flex-wrap items-center gap-3"><Button>Primary</Button><Button variant="secondary">Secondary</Button><Button variant="outline"><Plus size={16} />Check-in</Button><Button variant="danger">Go to shelter</Button><IconButton label="Alerts"><Bell size={18} /></IconButton><IconButton tone="light" label="Vitals"><HeartPulse size={18} /></IconButton></div>
      </Card>
      <Card>
        <CardHeader><CardTitle>Status</CardTitle></CardHeader>
        <div className="flex flex-wrap gap-2"><Badge tone="ok">Normal for you</Badge><Badge tone="warn">Battery 18%</Badge><Badge tone="critical">Emergency</Badge><Badge tone="info">Eva is guiding</Badge><Badge tone="brand">WearMeds</Badge></div>
        <div className="mt-4 flex items-center gap-4"><StatusDot tone="ok" /><StatusDot tone="warn" /><StatusDot tone="critical" pulse /><StatusDot tone="info" /></div>
      </Card>
      <Card>
        <CardHeader><CardTitle>Progress · split · gauge</CardTitle></CardHeader>
        <div className="flex flex-col gap-4"><Progress value={64} /><SplitBar a={70} b={30} /><div className="flex items-center gap-4"><Ring value={72} size={96}><span className="text-xl font-medium text-ink">72%</span></Ring><HalfGauge size={180} thickness={26} segments={[{ value: 55, tone: "info" }, { value: 15, tone: "ink" }, { value: 30, hatched: true }]} /></div></div>
      </Card>
      <Card>
        <CardHeader><CardTitle>Charts and rows</CardTitle><PillSelect>This week</PillSelect></CardHeader>
        <Sparkline data={[70, 72, 71, 74, 73, 78, 76, 72]} band={[62, 80]} />
        <div className="mt-4 flex flex-col gap-1"><ListRow leading={<Avatar name="Ana Ruiz" tone="ok" />} title="Ana Ruiz" subtitle="Seat 4B · all good" trailing={<Check done />} /><ListRow leading={<IconTile src="img/icons/water.webp" alt="Water" size="sm" />} title="Drink 1 glass of water" subtitle="32 min left" trailing={<Check />} /></div>
        <div className="mt-4"><Segmented items={["Day", "Week", "Trip"]} active="Week" /></div>
      </Card>
      <Card className="lg:col-span-3">
        <CardHeader><CardTitle>Bars · highlighted day</CardTitle><Stat value="7" unit="days" /></CardHeader>
        <Bars data={[3, 4, 3, 5, 9, 4, 3]} labels={["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]} highlight={4} tagFormat={v => `${v} alerts`} />
      </Card>
    </div>
  )
}

export default function App() {
  return (
    <main className="mx-auto flex max-w-screen-2xl flex-col">
      <header className="flex flex-col gap-6 px-6 pt-16 pb-12 lg:px-12">
        <Eyebrow>Eva by WearMeds · app wireframes</Eyebrow>
        <h1 className="max-w-5xl text-hero tracking-tight">Two views of the same care: the passenger's and the crew's.</h1>
        <p className="max-w-3xl text-xl text-muted"><span className="text-ink">Eva</span> — the health copilot for your missions, explorations and daily life.</p>
        <nav className="flex flex-wrap gap-2 text-sm"><a href="#passenger"><Badge>Passenger app</Badge></a><a href="#crew"><Badge>Crew dashboard</Badge></a><a href="#earth"><Badge>Earth view</Badge></a><a href="#system"><Badge>Design system</Badge></a></nav>
      </header>
      <Section id="passenger" eyebrow="View 1 · passenger" title="Passenger app and wristband">
        <MobileScreens />
      </Section>
      <Section id="crew" eyebrow="View 2 · crew on board" title="Crew dashboard">
        <CrewDashboard />
      </Section>
      <Section id="earth" eyebrow="View 2b · flight surgeon on Earth" title="Earth view, minutes behind">
        <EarthView />
      </Section>
      <Section id="system" eyebrow="Design system" title="One system, enforced by a linter">
        <DesignSystem />
      </Section>
      <footer className="border-t border-line px-6 py-10 text-xs text-muted lg:px-12">Wireframes, not medical software · AI-generated concept images · Eva guides, records and warns · shocks: the AED decides, never Eva</footer>
    </main>
  )
}
