import { Phone, Button, Badge, Progress, StatusDot } from "@/ds"
import { Label, StepBars, BeatRing, StepRow, WhyNext, Stowed, Thumb } from "@/ds/mobile-extra"
import { ArrowUpLeft, Zap } from "lucide-react"

export function Helper() {
  return (
    <Phone tone="critical" number="09" caption="First helper">
      <div className="flex items-center justify-between pt-2">
        <p className="text-xs uppercase tracking-label text-ink">Emergency · near you</p>
        <span className="grid size-11 place-items-center rounded-full bg-background/20 text-sm text-ink">AR</span>
      </div>
      <div className="mt-2 flex flex-col gap-3">
        <p className="text-6xl leading-none tracking-tight text-ink">Ana needs help</p>
        <div className="flex items-end justify-between gap-3">
          <p className="text-xl text-ink">Seat 4B · 6 m away</p>
          <Thumb src="img/wrist/wrist-alert.webp" alt="Your band shows the way too" />
        </div>
      </div>
      <div className="flex items-center gap-4 rounded-card bg-background/15 p-4">
        <span className="grid size-12 shrink-0 place-items-center rounded-full bg-ink text-on-light"><ArrowUpLeft size={24} /></span>
        <p className="text-base text-ink">Go back 2 rows. She is on your left.</p>
      </div>
      <WhyNext onCritical why="No pulse, no movement" next="CPR on your band · AED coming" how="Wristband sensors" />
      <Stowed onCritical item="AED" place="station, corridor B" light="blinking" />
      <div className="mt-auto flex flex-col gap-3">
        <Button size="xl" className="w-full">I'm going</Button>
        <Button variant="secondary" size="lg" className="w-full">I can't</Button>
      </div>
    </Phone>
  )
}

export function Cpr() {
  return (
    <Phone number="10" caption="CPR">
      <div className="flex items-center justify-between pt-2">
        <Badge tone="critical">Emergency · Ana, 4B</Badge>
        <span className="text-xs text-muted">Step 2 of 4</span>
      </div>
      <StepBars step={2} total={4} />
      <p className="text-3xl leading-tight tracking-tight text-ink">Push hard and fast in the middle of the chest</p>
      <div className="flex justify-center">
        <BeatRing rate={110} />
      </div>
      <div className="flex items-center gap-3 rounded-tile bg-ok/10 p-2 pr-4">
        <Thumb src="img/wrist/wrist-cpr.webp" alt="Hands on the chest, the band counting the pushes" className="h-12 w-18" />
        <span className="flex flex-1 flex-col gap-0.5">
          <span className="flex items-center gap-2 text-sm text-ink"><StatusDot tone="ok" />Good depth · keep going</span>
          <span className="text-xs text-muted">108 a minute · your band counts</span>
        </span>
      </div>
      <div className="flex items-center gap-3 rounded-tile bg-info/10 px-4 py-3">
        <Zap size={18} className="shrink-0 text-info" />
        <p className="flex-1 text-sm text-ink">AED on · Tomás</p>
        <Badge tone="info">AED decides shocks</Badge>
      </div>
      <WhyNext why="Your pushes move her blood" next="0:40 · AED checks · don't touch" how="No pulse, no movement" />
    </Phone>
  )
}

export function Storm() {
  return (
    <Phone number="11" caption="Solar storm">
      <div className="flex items-center justify-between pt-2">
        <Badge tone="critical">Solar storm</Badge>
        <span className="text-xs text-muted">Shelter · Deck B</span>
      </div>
      <p className="text-5xl leading-none tracking-tight text-ink">Go to the shelter now</p>
      <div className="flex items-center gap-3">
        <p className="font-mono text-4xl tracking-tight text-ink">08:40</p>
        <Badge>Walk, don't run</Badge>
      </div>
      <div className="flex flex-col gap-3">
        <Label>Your route · 40 m</Label>
        <StepRow n="1" title="Leave cabin 4 and turn left" done />
        <StepRow n="2" title="Follow the floor arrows to Deck B" />
        <StepRow n="3" title="Go in and close the hatch" />
      </div>
      <WhyNext why="Water walls block the radiation" next="Stay ~6 h, until Eva says" how="Ship radiation sensors" />
      <div className="mt-auto flex flex-col gap-2">
        <div className="flex items-baseline justify-between">
          <p className="text-sm text-ink">Crew in shelter 9/15</p>
          <p className="text-xs text-muted">Mira is there</p>
        </div>
        <Progress value={60} tone="ink" marker={false} />
      </div>
      <Button size="lg" className="w-full">I'm in the shelter</Button>
    </Phone>
  )
}
