import { Phone, Body, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Ring, Bars } from "@/ds"
import { PhoneTop, Label, Meter, Glasses, StepRow } from "@/ds/mobile-extra"
import { ChevronLeft, Plus, Share2 } from "lucide-react"
import { Tabs } from "./shared"

const meals = [
  { name: "Breakfast", food: "Oats, dried fruit, tea", kcal: "520" },
  { name: "Lunch", food: "Lentil curry, rice", kcal: "780" },
  { name: "Snack", food: "Nut bar", kcal: "340" },
  { name: "Dinner", food: "Pasta with beans · 19:30", kcal: null },
]

export function Food() {
  return (
    <Phone number="05" caption="Food. Calories left in a ring, macros as bars with a hatched remainder, meals, water glasses, and two flags for bones, which weaken in low gravity." tabbar={<Tabs active="Food" />}>
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Food · today" right={<IconButton label="Add a meal"><Plus size={18} /></IconButton>} />
      <div className="flex items-center gap-5">
        <Ring value={71} size={124} thickness={12} tone="info">
          <span className="flex flex-col items-center">
            <span className="text-2xl tracking-tight text-ink">660</span>
            <span className="text-xs text-muted">kcal left</span>
          </span>
        </Ring>
        <div className="flex flex-1 flex-col gap-3">
          <Meter label="Carbs" value={180} goal={290} unit="g" />
          <Meter label="Protein" value={74} goal={95} unit="g" />
          <Meter label="Fat" value={48} goal={75} unit="g" />
        </div>
      </div>
      <div className="flex flex-col">
        <div className="flex items-baseline justify-between pb-1"><Label>Meals</Label><span className="text-xs text-muted">1,640 of 2,300 kcal</span></div>
        {meals.map(m => (
          <div key={m.name} className="flex items-center justify-between border-b border-line py-1.5 last:border-b-0">
            <div className="flex flex-col"><span className="text-sm text-ink">{m.name}</span><span className="text-xs text-muted">{m.food}</span></div>
            {m.kcal ? <span className="text-sm text-ink">{m.kcal}<span className="ml-1 text-xs text-muted">kcal</span></span> : <span className="text-xs text-muted">Planned</span>}
          </div>
        ))}
      </div>
      <div className="flex flex-col gap-2">
        <div className="flex items-baseline justify-between"><Label>Water</Label><span className="text-xs text-muted">1.2 of 2 L · 3 to go</span></div>
        <Glasses filled={5} total={8} />
      </div>
      <div className="flex flex-col gap-2">
        <Label>Bones weaken in low gravity</Label>
        <div className="grid grid-cols-2 gap-2">
          <div className="flex flex-col gap-0.5 rounded-tile bg-surface px-3 py-2.5">
            <span className="flex items-center gap-2 text-sm text-ink"><StatusDot tone="warn" />Calcium</span>
            <span className="text-xs text-muted">620 of 1,000 mg · low</span>
          </div>
          <div className="flex flex-col gap-0.5 rounded-tile bg-surface px-3 py-2.5">
            <span className="flex items-center gap-2 text-sm text-ink"><StatusDot tone="ok" />Vitamin D</span>
            <span className="text-xs text-muted">Taken · good</span>
          </div>
        </div>
      </div>
    </Phone>
  )
}

export function Review() {
  return (
    <Phone number="06" caption="Weekly review. Eva explains the week in plain words, shows the one thing to fix, gives two concrete changes and says who else can see it.">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Week 13" right={<IconButton label="Share"><Share2 size={18} /></IconButton>} />
      <div className="flex flex-col gap-2">
        <Label>Eva · your week in food</Label>
        <p className="text-3xl leading-tight tracking-tight text-ink">A good week. Calcium was low on 3 days.</p>
      </div>
      <Card>
        <CardHeader className="mb-9 items-center">
          <CardTitle>Calcium per day · mg</CardTitle>
          <Badge>Goal 1,000 mg</Badge>
        </CardHeader>
        <Bars data={[980, 1040, 610, 920, 540, 700, 1010]} labels={["M", "T", "W", "T", "F", "S", "S"]} highlight={4} max={1100} tagFormat={v => v} />
      </Card>
      <div className="flex flex-col gap-4">
        <Label>Try this next week</Label>
        <StepRow n="1" title="Add a milk pouch to breakfast" read="About +300 mg calcium a day" />
        <StepRow n="2" title="Swap the nut bar for cheese crackers" read="Friday snack · about +200 mg" />
      </div>
      <div className="mt-auto flex items-center gap-2 rounded-tile bg-surface px-3 py-3">
        <StatusDot tone="info" />
        <Body muted>Shared with the crew lead · read at 09:12</Body>
      </div>
    </Phone>
  )
}
