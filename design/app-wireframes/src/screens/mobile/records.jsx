import { Phone, Body, IconButton, Card, Badge, Progress, IconTile, StatusDot } from "@/ds"
import { PhoneTop, Label, SyncState, SuitDiagram, StepRow } from "@/ds/mobile-extra"
import { CalendarDays, SlidersHorizontal, ChevronLeft, Info } from "lucide-react"
import { Tabs } from "./shared"

const entries = [
  { time: "11:20", title: "Exercise done · 2 h", sub: "Treadmill and bands", step: 0 },
  { time: "09:12", title: "Check-in · all good", sub: "Heart 72 · oxygen 98%", step: 1 },
  { time: "08:30", title: "Vitamin D taken", sub: "With breakfast", step: 2 },
  { time: "06:35", title: "Slept 7 h 10 min", sub: "Woke once at 03:10", step: 2 },
  { time: "Yesterday", title: "Mild headache", sub: "Eva said: drink water, rest", step: 2 },
]

export function Logbook() {
  return (
    <Phone number="07" caption="Logbook. Everything is saved on board first. Each entry shows if it is still on the ship, on its way, or received on Earth." tabbar={<Tabs active="Logbook" />}>
      <PhoneTop left={<IconButton label="Pick a day"><CalendarDays size={18} /></IconButton>} title="Logbook" right={<IconButton label="Filter"><SlidersHorizontal size={18} /></IconButton>} />
      <Card className="flex items-center gap-4 py-4">
        <IconTile src="img/chip-earth-b.jpg" alt="Earth" size="sm" />
        <div className="flex flex-col gap-0.5">
          <p className="text-lg text-ink">Earth is 14 min away</p>
          <Body muted>Messages take 14 min each way. Your log is safe on board.</Body>
        </div>
      </Card>
      <div className="flex flex-col">
        <div className="flex items-baseline justify-between pb-1"><Label>Latest</Label><span className="text-xs text-muted">5 entries</span></div>
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
      <div className="mt-auto flex items-center justify-between rounded-tile bg-surface px-4 py-3 text-xs text-muted">
        <span>Ship</span><span className="h-px flex-1 bg-line" /><span className="px-2">14 min</span><span className="h-px flex-1 bg-line" /><span>Earth</span>
      </div>
    </Phone>
  )
}

export function SuitMode() {
  return (
    <Phone number="08" caption="Suit mode. The suit fabric sits between the two parts: the magnets still hold, the pins don't touch, so the sensors run on their own small battery.">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Wristband" right={<IconButton label="How it works"><Info size={18} /></IconButton>} />
      <div className="flex flex-col gap-2">
        <Badge tone="info" className="self-start">Suit mode</Badge>
        <p className="text-3xl leading-tight tracking-tight text-ink">Your suit is between the parts.</p>
      </div>
      <Card>
        <SuitDiagram />
      </Card>
      <div className="flex flex-col gap-2">
        <div className="flex items-baseline justify-between">
          <p className="text-sm text-ink">Sensors on their own battery</p>
          <p className="text-sm text-ink">38%</p>
        </div>
        <Progress value={38} tone="info" />
        <p className="text-xs text-muted">About 9 h left · Eva keeps measuring and saving</p>
      </div>
      <div className="flex flex-col gap-3">
        <Label>When to re-join</Label>
        <StepRow n="1" title="Suit off? Snap the parts together" read="The pins touch and the sensors charge again" />
        <StepRow n="2" title="Re-join before the battery reaches 10%" read="Eva warns you about 1 h before" />
      </div>
      <div className="mt-auto flex items-center gap-3 rounded-tile bg-surface px-4 py-3">
        <StatusDot tone="neutral" />
        <p className="text-sm text-ink">Wrist ECG unavailable<span className="block text-xs text-muted">Back when the parts are re-joined</span></p>
      </div>
    </Phone>
  )
}
