import { Phone, Button, Badge, Progress, StatusDot } from "@/ds"
import { Label, StepBars, BeatRing, StepRow, WhyNext, Stowed } from "@/ds/mobile-extra"
import { ArrowUpLeft, Zap } from "lucide-react"

export function Helper() {
  return (
    <Phone tone="critical" number="09" caption="Emergency, first helper. Eva asks the closest trained passenger to go to Ana and a second helper to bring the AED from the emergency station. Where to go, why, what comes next, and where the AED is. Two answers only; if they can't, Eva asks the next person.">
      <div className="flex items-center justify-between pt-2">
        <p className="text-xs uppercase tracking-label text-ink">Emergency · near you</p>
        <span className="grid size-11 place-items-center rounded-full bg-background/20 text-sm text-ink">AR</span>
      </div>
      <div className="mt-2 flex flex-col gap-3">
        <p className="text-6xl leading-none tracking-tight text-ink">Ana needs help</p>
        <p className="text-xl text-ink">Seat 4B · 6 m away</p>
      </div>
      <div className="flex items-center gap-4 rounded-card bg-background/15 p-4">
        <span className="grid size-12 shrink-0 place-items-center rounded-full bg-ink text-on-light"><ArrowUpLeft size={24} /></span>
        <p className="text-base text-ink">Go back 2 rows. She is on your left.</p>
      </div>
      <WhyNext onCritical why="Her wristband finds no pulse. She doesn't move or answer." next="Eva guides CPR on your wristband. Tomás brings the AED." how="Wristband: no pulse, no movement, no answer" />
      <Stowed onCritical item="AED" place="emergency station, corridor B" light="light is blinking" />
      <div className="mt-auto flex flex-col gap-3">
        <Button size="xl" className="w-full">I'm going</Button>
        <Button variant="secondary" size="lg" className="w-full">I can't</Button>
      </div>
    </Phone>
  )
}

export function Cpr() {
  return (
    <Phone number="10" caption="CPR, step 2 of 4. Only what the helper needs now: the instruction, the beat, feedback on the pushes, and why and what comes next. The AED reads the heart through its own pads and decides any shock; Eva never does.">
      <div className="flex items-center justify-between pt-2">
        <Badge tone="critical">Emergency · Ana, 4B</Badge>
        <span className="text-xs text-muted">Step 2 of 4</span>
      </div>
      <StepBars step={2} total={4} />
      <p className="text-3xl leading-tight tracking-tight text-ink">Push hard and fast in the middle of the chest</p>
      <div className="flex justify-center">
        <BeatRing rate={110} />
      </div>
      <div className="flex items-center gap-3 rounded-tile bg-ok/10 px-4 py-3">
        <StatusDot tone="ok" />
        <p className="flex-1 text-sm text-ink">Good depth · keep going</p>
        <span className="text-xs text-muted">108/min</span>
      </div>
      <div className="flex items-center gap-3 rounded-tile bg-info/10 px-4 py-3">
        <Zap size={18} className="shrink-0 text-info" />
        <p className="flex-1 text-sm text-ink">AED attached by Tomás<span className="block text-xs text-muted">The AED checks the heart and tells you if a shock is needed</span></p>
      </div>
      <WhyNext why="Her heart is not pumping. Your pushes move the blood." next="In 0:40: AED is checking the heart · don't touch" how="Wristband: no pulse, no movement, no answer" />
    </Phone>
  )
}

export function Storm() {
  return (
    <Phone number="11" caption="Solar storm. One order, a countdown, three route steps, why and what comes next, and how many people are already safe.">
      <div className="flex items-center justify-between pt-2">
        <Badge tone="critical">Solar storm</Badge>
        <span className="text-xs text-muted">Shelter · Deck B</span>
      </div>
      <p className="text-5xl leading-none tracking-tight text-ink">Go to the shelter now</p>
      <div className="flex items-baseline gap-3">
        <p className="font-mono text-4xl tracking-tight text-ink">08:40</p>
        <p className="text-xs text-muted">left to get there · walk, don't run</p>
      </div>
      <div className="flex flex-col gap-3">
        <Label>Your route · 40 m</Label>
        <StepRow n="1" title="Leave cabin 4 and turn left" done />
        <StepRow n="2" title="Follow the floor arrows to Deck B" />
        <StepRow n="3" title="Go in and close the hatch" />
      </div>
      <WhyNext why="A solar storm is coming. The shelter's water walls block the radiation." next="Stay inside until Eva says it's safe, about 6 h" how="Ship radiation sensors" />
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
