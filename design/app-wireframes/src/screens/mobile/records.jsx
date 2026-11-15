import { Phone, IconButton, Card, Badge, Progress, IconTile, StatusDot } from "@/ds"
import { PhoneTop, Label, SyncState, SuitDiagram, StepRow, Hero, PhotoTile } from "@/ds/mobile-extra"
import { CalendarDays, SlidersHorizontal, ChevronLeft, Info } from "lucide-react"
import { Tabs } from "./shared"

const entries = [
  { time: "11:20", title: "Exercise done · 2 h", sub: "Treadmill and bands", step: 0 },
  { time: "09:12", title: "Check-in · all good", sub: "Heart 72 · oxygen 98%", step: 1 },
  { time: "08:30", title: "Vitamin D taken", sub: "With breakfast", step: 2 },
  { time: "06:35", title: "Slept 7 h 10 min", sub: "Woke once at 03:10", step: 2 },
  { time: "Yesterday", title: "Mild headache", sub: "Water and rest", step: 2 },
]

export function Logbook() {
  return (
    <Phone number="07" caption="Logbook" tabbar={<Tabs active="Logbook" />}>
      <PhoneTop left={<IconButton label="Pick a day"><CalendarDays size={18} /></IconButton>} title="Logbook" right={<IconButton label="Filter"><SlidersHorizontal size={18} /></IconButton>} />
      <Card className="flex items-center gap-3 py-4">
        <span className="flex flex-col items-center gap-1"><IconTile src="img/icons/logbook.webp" alt="Logbook" size="sm" /><span className="text-xs text-muted">On board</span></span>
        <div className="flex flex-1 flex-col items-center gap-1.5 pb-5">
          <span className="text-lg text-ink">Earth · 14 min</span>
          <span className="w-full border-t border-dashed border-info/60" />
        </div>
        <span className="flex flex-col items-center gap-1"><IconTile src="img/icons/earth-link.webp" alt="Earth link" size="sm" /><span className="text-xs text-muted">Earth</span></span>
      </Card>
      <div className="flex flex-col">
        <Label className="pb-1">Latest</Label>
        {entries.map(e => (
          <div key={e.time} className="flex items-start gap-3 border-b border-line py-3 last:border-b-0">
            <span className="w-16 shrink-0 pt-0.5 text-xs text-muted">{e.time}</span>
            <div className="flex min-w-0 flex-1 flex-col">
              <span className="text-sm text-ink">{e.title}</span>
              <span className="text-xs text-muted">{e.sub}</span>
            </div>
            <SyncState step={e.step} />
          </div>
        ))}
      </div>
    </Phone>
  )
}

export function SuitMode() {
  return (
    <Phone number="08" caption="Suit mode">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Wristband" right={<IconButton label="How it works"><Info size={18} /></IconButton>} />
      <Hero src="img/wrist/wrist-suit.webp" alt="Screen part on the suit sleeve, sensor base under it on the wrist" size="sm">
        <span className="absolute top-3 left-3 flex rounded-full bg-background/80"><Badge tone="info">Suit mode</Badge></span>
      </Hero>
      <p className="text-xl leading-tight tracking-tight text-ink">Your suit is between the parts.</p>
      <Card className="py-4">
        <SuitDiagram />
      </Card>
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <p className="flex items-center gap-2.5 text-sm text-ink"><PhotoTile src="img/icons/battery.webp" alt="Battery" size="xs" />Sensor battery<span className="text-xs text-muted">~9 h left</span></p>
          <p className="text-sm text-ink">38%</p>
        </div>
        <Progress value={38} tone="info" />
      </div>
      <div className="flex flex-col gap-3">
        <StepRow n="1" title="Suit off? Snap the parts together" read="Pins touch · sensors charge" />
        <StepRow n="2" title="Re-join before 10%" read="Eva warns 1 h before" />
      </div>
      <div className="mt-auto flex items-center gap-3 rounded-tile bg-surface px-4 py-3">
        <StatusDot tone="ok" />
        <p className="text-sm text-ink">ECG works · bare hands on pads</p>
      </div>
    </Phone>
  )
}
