import { Phone, Headline, Button, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Progress, Sparkline, ListRow, IconTile, Check, Segmented } from "@/ds"
import { PhoneTop, Label, Reading, Tile, Legend, PhotoTile, Thumb, Hero, IconLine } from "@/ds/mobile-extra"
import { Check as CheckIcon, Bell, LayoutGrid, ChevronLeft, Info, ArrowRight, CalendarDays, Plus, Compass, NotebookPen, BellRing, CircleCheck } from "lucide-react"
import { Tabs } from "./shared"

export function Welcome() {
  return (
    <Phone number="01" caption="Welcome">
      <PhoneTop left={<Label>Eva by WearMeds</Label>} right={<Badge>Offline ready</Badge>} />
      <Hero src="img/wrist/wrist-hero.webp" alt="The Eva wristband worn on the wrist like a watch" size="lg" />
      <div className="grid grid-cols-2 gap-3">
        {[{ src: "img/wrist/band-interface.webp", label: "Screen" }, { src: "img/wrist/band-base.webp", label: "Sensor base" }].map(p => (
          <div key={p.label} className="flex items-center gap-2.5">
            <PhotoTile src={p.src} alt={p.label} size="sm" />
            <span className="flex flex-col"><span className="text-sm text-ink">{p.label}</span><span className="text-xs text-ok">Paired</span></span>
          </div>
        ))}
      </div>
      <h1 className="mt-2 text-5xl leading-none tracking-tight text-ink">Care for the whole trip to Mars.</h1>
      <div className="flex flex-wrap gap-2">
        <Badge tone="info"><Compass size={12} />Guides</Badge><Badge tone="info"><NotebookPen size={12} />Records</Badge><Badge tone="info"><BellRing size={12} />Warns</Badge>
      </div>
      <div className="mt-auto flex items-center gap-3">
        <Button size="lg" className="flex-1">Get started</Button>
        <IconButton tone="light" size="lg" label="Wristband paired"><CheckIcon size={22} strokeWidth={2.5} /></IconButton>
      </div>
    </Phone>
  )
}

const BandPart = ({ src, name, pct, read }) => (
  <div className="flex items-center gap-2.5">
    <PhotoTile src={src} alt={name} size="xs" />
    <div className="flex min-w-0 flex-1 flex-col gap-1.5">
      <span className="flex items-baseline justify-between gap-2"><span className="text-sm text-ink">{name}</span><span className="truncate text-xs text-muted">{read}</span></span>
      <Progress value={pct} tone="ink" marker={false} size="sm" />
    </div>
  </div>
)

const Vital = ({ src, value, unit, read }) => (
  <div className="flex flex-col items-start gap-3">
    <PhotoTile src={src} size="sm" />
    <Reading value={value} unit={unit} read={read} size="sm" />
  </div>
)

export function Home() {
  return (
    <Phone number="02" caption="Home" tabbar={<Tabs active="Home" />}>
      <PhoneTop left={<IconButton label="Menu"><LayoutGrid size={18} /></IconButton>} right={<IconButton label="Alerts"><Bell size={18} /></IconButton>} />
      <div className="flex flex-col gap-1">
        <Label>Day 94 of 212</Label>
        <Headline as="h1" className="text-5xl leading-none">Good morning, Ana</Headline>
      </div>
      <Card>
        <CardHeader className="items-center">
          <CardTitle>Today: all good</CardTitle>
          <Badge tone="ok">Normal for you</Badge>
        </CardHeader>
        <div className="grid grid-cols-3 gap-3">
          <Vital src="img/icons/heart.webp" value="72" unit="bpm" read="Pulse · calm" />
          <Vital src="img/icons/lungs.webp" value="98" unit="%" read="Oxygen · good" />
          <Vital src="img/icons/skin-temp.webp" value="36.4" unit="°C" read="Skin · normal" />
        </div>
      </Card>
      <Card className="py-4">
        <CardHeader className="mb-3 items-center">
          <CardTitle>Your wristband</CardTitle>
          <span className="flex items-center gap-2 text-xs text-ink"><StatusDot tone="ok" />Joined</span>
        </CardHeader>
        <div className="flex items-center gap-4">
          <Thumb src="img/wrist/band-top.webp" alt="The wristband, screen part on top" size="band" />
          <div className="flex min-w-0 flex-1 flex-col gap-2">
            <BandPart src="img/wrist/band-interface.webp" name="Screen" pct={64} read="64%" />
            <BandPart src="img/wrist/band-base.webp" name="Sensor" pct={100} read="100%" />
          </div>
        </div>
      </Card>
      <div className="flex items-center gap-3 rounded-full bg-surface py-2 pr-2 pl-5">
        <div className="flex flex-1 flex-col"><span className="text-xs text-muted">Next · 10:30</span><span className="text-sm text-ink">Drink a glass of water</span></div>
        <IconButton tone="light" label="Open plan"><ArrowRight size={18} /></IconButton>
      </div>
    </Phone>
  )
}

export function HomeGrid() {
  return (
    <Phone number="02b" caption="Everything Eva watches" tabbar={<Tabs active="Home" />}>
      <PhoneTop left={<IconButton label="Menu"><LayoutGrid size={18} /></IconButton>} right={<IconButton label="Alerts"><Bell size={18} /></IconButton>} />
      <div className="flex items-center justify-between gap-3">
        <p className="text-3xl tracking-tight text-ink">8 of 9 fine</p>
        <Badge tone="warn">Water · check</Badge>
      </div>
      <div className="grid grid-cols-3 gap-x-2 gap-y-5">
        <Tile src="img/icons/heart.webp" label="Heart" read="72 · calm" />
        <Tile src="img/icons/lungs.webp" label="Oxygen" read="98% · good" />
        <Tile src="img/icons/sleep.webp" label="Sleep" read="7 h 10 · good" />
        <Tile src="img/icons/water.webp" label="Water" read="1.2 of 2 L" tone="warn" />
        <Tile src="img/icons/radiation.webp" label="Radiation" read="Low today" />
        <Tile src="img/icons/food.webp" label="Food" read="1,640 kcal" />
        <Tile src="img/icons/aed-station.webp" label="Station" read="AED · corridor B" />
        <Tile src="img/icons/crew.webp" label="Crew" read="15 · all fine" />
        <Tile src="img/icons/earth-link.webp" label="Earth link" read="14 min away" />
      </div>
      <p className="mt-auto text-center text-xs text-faint">Checked 2 min ago</p>
    </Phone>
  )
}

export function Heart() {
  return (
    <Phone number="03" caption="Heart">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Heart" right={<IconButton label="About this reading"><Info size={18} /></IconButton>} />
      <div className="flex flex-col gap-3">
        <p className="text-8xl leading-none tracking-tight text-ink">72<span className="ml-2 text-xl text-muted">bpm</span></p>
        <Badge tone="ok" className="self-start">Normal for you</Badge>
      </div>
      <Card>
        <CardHeader className="items-center">
          <CardTitle>Last 8 hours</CardTitle>
          <div className="w-36"><Segmented items={["8 h", "Week"]} active="8 h" /></div>
        </CardHeader>
        <Sparkline data={[62, 57, 54, 53, 56, 64, 79, 92, 88, 74, 72]} band={[58, 82]} tone="ink" height={96} />
        <div className="mt-2 flex justify-between text-xs text-muted"><span>Sleep</span><span>Exercise</span><span>Now</span></div>
        <div className="mt-3"><Legend>Your normal · 58 to 82 · learned in 30 days</Legend></div>
      </Card>
      <div className="flex items-center gap-3 rounded-tile bg-ok/10 px-4 py-3">
        <IconLine icon={CircleCheck} tone="ok">Nothing to do · keep your day</IconLine>
      </div>
      <div className="mt-auto flex items-center gap-3">
        <IconTile src="img/wrist/band-base.webp" alt="Sensor base" size="sm" />
        <p className="text-xs text-muted">Sensor base · 1 min ago</p>
      </div>
    </Phone>
  )
}

export function Plan() {
  const items = [
    { src: "img/icons/exercise.webp", title: "2 h exercise", sub: "Done 11:20", done: true },
    { src: "img/icons/water.webp", title: "Drink 2 L of water", sub: "1.2 of 2 L", done: false },
    { src: "img/icons/vitamin-d.webp", title: "Take vitamin D", sub: "Done · breakfast", done: true },
    { src: "img/icons/sleep.webp", title: "Sleep 22:30 to 06:30", sub: "Lights dim 22:00", done: false },
    { src: "img/icons/eva-orb.webp", title: "Check in with Eva", sub: "Done 09:12", done: true },
  ]
  return (
    <Phone number="04" caption="Today's plan" tabbar={<Tabs active="Plan" />}>
      <PhoneTop left={<IconButton label="Calendar"><CalendarDays size={18} /></IconButton>} title="Today's plan" right={<IconButton label="Add"><Plus size={18} /></IconButton>} />
      <div className="flex flex-col gap-3">
        <div className="flex items-end justify-between gap-3">
          <p className="text-7xl leading-none tracking-tight text-ink">88%</p>
          <Badge tone="ok" className="mb-2">Doing well</Badge>
        </div>
        <Progress value={88} tone="info" size="lg" />
        <p className="text-xs text-muted">3 of 5 done</p>
      </div>
      <div className="flex flex-col gap-1">
        {items.map(it => (
          <ListRow key={it.title} className="px-0" leading={<PhotoTile size="sm" src={it.src} />} title={it.title} subtitle={it.sub} trailing={<Check done={it.done} />} />
        ))}
      </div>
    </Phone>
  )
}
