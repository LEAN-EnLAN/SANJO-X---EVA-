import { Phone, Body, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Ring, Sparkline } from "@/ds"
import { PhoneTop, Label, WhyNext, ecgTrace } from "@/ds/mobile-extra"
import { ChevronLeft, Info, X } from "lucide-react"

export function EcgCheck() {
  return (
    <Phone number="03b" caption="ECG check. A 30-second, one-lead check on demand: one finger of each hand on the two metal pads on top of the wristband's screen part. It doesn't need the wrist skin, so it works over the suit if the hands are bare. Eva asks for it; it never runs on its own.">
      <PhoneTop left={<IconButton label="Stop"><X size={18} /></IconButton>} title="ECG check" right={<IconButton label="How it works"><Info size={18} /></IconButton>} />
      <div className="flex flex-col gap-2">
        <p className="text-3xl leading-tight tracking-tight text-ink">Touch both metal pads, one finger of each hand</p>
        <p className="text-base text-muted">Stay still · 30 s</p>
      </div>
      <div className="flex justify-center py-2">
        <Ring value={40} size={176} thickness={10} tone="info">
          <span className="flex flex-col items-center">
            <span className="text-6xl leading-none tracking-tight text-ink">18</span>
            <span className="mt-1 text-xs uppercase tracking-label text-muted">seconds left</span>
          </span>
        </Ring>
      </div>
      <Card className="py-4">
        <CardHeader className="mb-2 items-center">
          <CardTitle>Live trace</CardTitle>
          <span className="flex items-center gap-2 text-xs text-muted"><StatusDot tone="ok" />Good contact</span>
        </CardHeader>
        <Sparkline data={ecgTrace(5)} tone="ink" height={64} />
      </Card>
      <div className="mt-auto">
        <WhyNext why="Your pulse was uneven twice this morning" next="The hub reads the trace · result in about 1 min" how="Optical pulse at the wrist, at rest" />
      </div>
    </Phone>
  )
}

export function EcgResult() {
  return (
    <Phone number="03c" caption="ECG result. One line in plain words, the trace, where it went, and what this check can and can't do.">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="ECG result" right={<span className="size-11" />} />
      <div className="flex flex-col gap-3">
        <Badge tone="ok" className="self-start">Normal for you</Badge>
        <p className="text-5xl leading-none tracking-tight text-ink">Regular rhythm</p>
        <p className="text-sm text-muted">72 bpm · steady beat for 30 s</p>
      </div>
      <Card className="py-4">
        <Sparkline data={ecgTrace(5)} tone="ink" height={64} />
        <p className="mt-3 text-xs text-muted">Today 10:04 · wristband, two pads, one lead, 30 s</p>
      </Card>
      <div className="flex items-center gap-3">
        <StatusDot tone="info" />
        <p className="text-sm text-ink">Saved and sent to the hub</p>
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>What to do</Label>
        <Body>Nothing. Eva may ask for another check in a few days.</Body>
      </div>
      <div className="mt-auto flex flex-col gap-1.5 rounded-tile bg-surface px-4 py-3">
        <Label>Good to know</Label>
        <Body muted>This check does not replace a hospital ECG. It only works while you are still. It never decides a shock: in an emergency, the AED from the emergency station checks the heart.</Body>
      </div>
    </Phone>
  )
}
