import { Phone, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Ring, Bars } from "@/ds"
import { PhoneTop, Label, Meter, Glasses, StepRow, PhotoTile } from "@/ds/mobile-extra"
import { ChevronLeft, Plus, Share2 } from "lucide-react"
import { Tabs } from "./shared"

const meals = [
  { name: "Breakfast", food: "Oats, dried fruit, tea", kcal: "520", src: "img/icons/meal-breakfast.webp" },
  { name: "Lunch", food: "Lentil curry, rice", kcal: "780", src: "img/icons/meal-lunch.webp" },
  { name: "Snack", food: "Nut bar", kcal: "340", src: "img/icons/meal-snack.webp" },
  { name: "Dinner", food: "Pasta, beans · 19:30", kcal: null, src: "img/icons/meal-dinner.webp" },
]

export function Food() {
  return (
    <Phone number="05" caption="Food" tabbar={<Tabs active="Food" />}>
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
        <Label className="pb-1">Meals</Label>
        {meals.map(m => (
          <div key={m.name} className="flex items-center justify-between gap-3 border-b border-line py-1 last:border-b-0">
            <div className="flex min-w-0 items-center gap-2.5">
              <PhotoTile src={m.src} alt={m.name} size="xs" />
              <span className="flex min-w-0 flex-col"><span className="text-sm text-ink">{m.name}</span><span className="truncate text-xs text-muted">{m.food}</span></span>
            </div>
            {m.kcal ? <span className="text-sm text-ink">{m.kcal}<span className="ml-1 text-xs text-muted">kcal</span></span> : <span className="text-xs text-muted">Planned</span>}
          </div>
        ))}
      </div>
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between"><span className="flex items-center gap-2"><PhotoTile src="img/icons/water.webp" alt="Water" size="xs" /><Label>Water</Label></span><span className="text-xs text-muted">1.2 of 2 L · 3 to go</span></div>
        <Glasses filled={5} total={8} />
      </div>
      <div className="flex flex-col gap-2">
        <Label>Bones</Label>
        <div className="grid grid-cols-2 gap-2">
          <div className="flex items-center gap-2.5 rounded-tile bg-surface px-2.5 py-2">
            <PhotoTile src="img/icons/calcium.webp" alt="Calcium" size="xs" />
            <span className="flex min-w-0 flex-col gap-0.5">
              <span className="flex items-center gap-2 text-sm text-ink"><StatusDot tone="warn" />Calcium</span>
              <span className="text-xs text-muted">Low · 620 mg of 1,000</span>
            </span>
          </div>
          <div className="flex items-center gap-2.5 rounded-tile bg-surface px-2.5 py-2">
            <PhotoTile src="img/icons/vitamin-d.webp" alt="Vitamin D" size="xs" />
            <span className="flex min-w-0 flex-col gap-0.5">
              <span className="flex items-center gap-2 text-sm text-ink"><StatusDot tone="ok" />Vitamin D</span>
              <span className="text-xs text-muted">Taken · good</span>
            </span>
          </div>
        </div>
      </div>
    </Phone>
  )
}

export function Review() {
  return (
    <Phone number="06" caption="Weekly review">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Week 13" right={<IconButton label="Share"><Share2 size={18} /></IconButton>} />
      <div className="flex flex-col gap-2">
        <Badge tone="ok" className="self-start">Good week</Badge>
        <p className="text-3xl leading-tight tracking-tight text-ink">Calcium low on 3 days</p>
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
        <StepRow n="1" title="Add a milk pouch to breakfast" read="+300 mg a day" />
        <StepRow n="2" title="Swap the nut bar for cheese crackers" read="Friday · +200 mg" />
      </div>
      <div className="mt-auto flex items-center gap-2 rounded-tile bg-surface px-3 py-3">
        <StatusDot tone="info" />
        <p className="text-xs text-muted">Crew lead · read 09:12</p>
      </div>
    </Phone>
  )
}
