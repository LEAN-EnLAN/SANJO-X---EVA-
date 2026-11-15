import { Phone, Headline, Body, Button, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Progress, Sparkline, ListRow, IconTile, Check, Segmented } from "@/ds"
import { PhoneTop, Label, Reading, Tile, Hero, Legend } from "@/ds/mobile-extra"
import { Check as CheckIcon, Bell, LayoutGrid, ChevronLeft, Info, HeartPulse, Droplet, ArrowRight, Moon, Utensils, Wind, CalendarDays, Plus, Dumbbell, Sun, MessageCircle } from "lucide-react"
import { Tabs } from "./shared"

export function Welcome() {
  return (
    <Phone number="01" caption="Welcome. One promise, one button. The round check confirms the band is paired before the trip starts.">
      <PhoneTop left={<Label>Eva by WearMeds</Label>} right={<Badge>Offline ready</Badge>} />
      <Hero src="img/s4-b.jpg" alt="A passenger wearing the harness and wristband" size="lg" />
      <h1 className="mt-2 text-5xl leading-none tracking-tight text-ink">Care for the whole trip to Mars.</h1>
      <Body muted>Eva guides you, keeps your health log and warns you early. It works on board, even when Earth is far away.</Body>
      <div className="flex flex-wrap gap-2">
        <Badge>Guides</Badge><Badge>Records</Badge><Badge>Warns</Badge>
      </div>
      <div className="mt-auto flex items-center gap-3">
        <Button size="lg" className="flex-1">Get started</Button>
        <IconButton tone="light" size="lg" label="Wristband paired"><CheckIcon size={22} strokeWidth={2.5} /></IconButton>
      </div>
      <p className="text-center text-xs text-faint">Wristband paired · harness paired</p>
    </Phone>
  )
}

export function Home() {
  return (
    <Phone number="02" caption="Home. A friendly greeting, one card that answers &quot;am I OK?&quot;, and the state of both parts of the wristband." tabbar={<Tabs active="Home" />}>
      <PhoneTop left={<IconButton label="Menu"><LayoutGrid size={18} /></IconButton>} right={<IconButton label="Alerts"><Bell size={18} /></IconButton>} />
      <div className="flex flex-col gap-1">
        <Label>Day 94 of 212 · on the way</Label>
        <Headline as="h1" className="text-5xl leading-none">Good morning, Ana</Headline>
      </div>
      <Card>
        <CardHeader className="mb-5 items-center">
          <CardTitle>Today: all good</CardTitle>
          <Badge tone="ok">Normal for you</Badge>
        </CardHeader>
        <div className="grid grid-cols-3 gap-3">
          <Reading value="72" unit="bpm" read="Pulse · calm" />
          <Reading value="98" unit="%" read="Oxygen · good" />
          <Reading value="36.4" unit="°C" read="Skin · normal" />
        </div>
      </Card>
      <Card>
        <CardHeader className="items-center">
          <CardTitle>Your wristband</CardTitle>
          <span className="flex items-center gap-2 text-xs text-ink"><StatusDot tone="ok" />Joined</span>
        </CardHeader>
        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-xs"><span className="text-ink">Screen battery · 64%</span><span className="text-muted">about 2 days</span></div>
            <Progress value={64} tone="ink" marker={false} />
          </div>
          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-xs"><span className="text-ink">Sensor battery · 100%</span><span className="text-muted">charging from screen</span></div>
            <Progress value={100} tone="ink" marker={false} />
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
    <Phone number="02b" caption="Home, scrolled. Nine round tiles for everything Eva watches. Each dot is green when it is normal for you, amber when it needs a look." tabbar={<Tabs active="Home" />}>
      <PhoneTop left={<IconButton label="Menu"><LayoutGrid size={18} /></IconButton>} right={<IconButton label="Alerts"><Bell size={18} /></IconButton>} />
      <div className="flex flex-col gap-1">
        <Label>Everything Eva watches</Label>
        <p className="text-2xl tracking-tight text-ink">8 of 9 are fine. Water needs a look.</p>
      </div>
      <div className="grid grid-cols-3 gap-x-2 gap-y-5">
        <Tile icon={HeartPulse} label="Heart" read="72 · calm" />
        <Tile icon={Wind} label="Oxygen" read="98% · good" />
        <Tile icon={Moon} label="Sleep" read="7 h 10 · good" />
        <Tile icon={Droplet} label="Water" read="1.2 of 2 L" tone="warn" />
        <Tile src="img/chip-sun-b.jpg" label="Radiation" read="Low today" />
        <Tile icon={Utensils} label="Food" read="1,640 kcal" />
        <Tile src="img/chip-harness-b.jpg" label="Harness" read="Ready" />
        <Tile src="img/chip-crew-b.jpg" label="Crew" read="15 · all fine" />
        <Tile src="img/chip-earth-b.jpg" label="Earth link" read="14 min away" />
      </div>
      <p className="mt-auto text-center text-xs text-faint">Checked 2 min ago · saved on board</p>
    </Phone>
  )
}

export function Heart() {
  return (
    <Phone number="03" caption="Heart. The number, the person's own normal band, what it means and what to do, in that order.">
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
        <div className="mt-2 flex justify-between text-xs text-muted"><span>Sleep · low is normal</span><span>Exercise</span><span>Now</span></div>
        <div className="mt-3"><Legend>Your normal at rest · 58 to 82</Legend></div>
      </Card>
      <div className="flex flex-col gap-1.5">
        <Label>What this means</Label>
        <Body>Your heart beats 72 times a minute. That is inside your own normal, learned over 30 days on board.</Body>
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>What to do</Label>
        <Body>Nothing. Keep your day as planned. Eva tells you if it changes.</Body>
      </div>
      <p className="mt-auto text-xs text-faint">Inner sensor · light through the skin · 1 min ago</p>
    </Phone>
  )
}

export function Plan() {
  const items = [
    { icon: Dumbbell, title: "2 h exercise", sub: "For bones and muscles · done 11:20", done: true },
    { icon: Droplet, title: "Drink 2 L of water", sub: "1.2 L so far · 3 glasses to go", done: false },
    { icon: Sun, title: "Take vitamin D", sub: "Done with breakfast", done: true },
    { icon: Moon, title: "Sleep 22:30 to 06:30", sub: "Tonight · lights dim at 22:00", done: false },
    { icon: MessageCircle, title: "Check in with Eva", sub: "Done at 09:12 · all good", done: true },
  ]
  return (
    <Phone number="04" caption="Today's plan. One score in plain words and a short checklist. The hatched part of the bar is what is left." tabbar={<Tabs active="Plan" />}>
      <PhoneTop left={<IconButton label="Calendar"><CalendarDays size={18} /></IconButton>} title="Today's plan" right={<IconButton label="Add"><Plus size={18} /></IconButton>} />
      <div className="flex flex-col gap-3">
        <p className="text-7xl leading-none tracking-tight text-ink">88%</p>
        <p className="text-lg text-ink">You are doing well</p>
        <Progress value={88} tone="info" size="lg" />
        <p className="text-xs text-muted">3 of 5 done · water and sleep are next</p>
      </div>
      <div className="flex flex-col">
        {items.map(it => (
          <ListRow key={it.title} className="px-0" leading={<IconTile size="sm"><it.icon size={20} /></IconTile>} title={it.title} subtitle={it.sub} trailing={<Check done={it.done} />} />
        ))}
      </div>
    </Phone>
  )
}

