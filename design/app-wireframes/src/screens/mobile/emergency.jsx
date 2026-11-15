import { Phone, Button, Badge, Progress, StatusDot } from "@/ds"
import { Label, StepBars, BeatRing, StepRow, WhyNext, Stowed } from "@/ds/mobile-extra"
import { ArrowUpLeft } from "lucide-react"

export function Helper() {
  return (
    <Phone tone="critical" number="09" caption="Emergency, helper. The closest passenger is asked first: where to go, why, what comes next, and where the pads are. Two answers only; if they can't, Eva asks the next person.">
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
      <WhyNext onCritical why="Her heart rhythm looks wrong and she doesn't answer" next="Eva guides you step by step when you reach her" how="Harness ECG, no reply to voice" />
      <Stowed onCritical item="Defibrillation pads" place="harness front pocket" light="blinking" />
      <div className="mt-auto flex flex-col gap-3">
        <Button size="xl" className="w-full">I'm going</Button>
        <Button variant="secondary" size="lg" className="w-full">I can't</Button>
      </div>
    </Phone>
  )
}

export function Cpr() {
  return (
    <Phone number="10" caption="CPR, step 2 of 4. Only what the helper needs now: the instruction, the beat, feedback on the pushes, and why and what comes next. The harness does the heart checks and any shock.">
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
      <WhyNext why="No pulse detected by the harness" next="The harness checks the heart in 0:40 · don't touch then" how="Harness ECG, no reply" />
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
