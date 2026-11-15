import { StatusDot, Sparkline, Ring } from "@/ds"
import { BandScreenRect, Label, StepBars, ecgTrace } from "@/ds/mobile-extra"
import { ArrowUpLeft, ArrowLeft, Fingerprint } from "lucide-react"

/** top strip of the band screen: a tracked label on the left, a short status on the right */
const Strip = ({ left, right }) => <div className="flex items-center justify-between gap-3 text-xs">{left}{right}</div>

const Column = ({ label, value, unit, read, first }) => (
  <div className={first ? "flex flex-col gap-1" : "flex flex-col gap-1 border-l border-line pl-4"}>
    <Label>{label}</Label>
    <p className="text-4xl leading-none tracking-tight text-ink">{value}<span className="ml-1 text-sm text-muted">{unit}</span></p>
    <p className="text-xs text-muted">{read}</p>
  </div>
)

export function BandIdle() {
  return (
    <BandScreenRect number="B1" caption="Wristband, idle. The main screen: three columns, each a number with its read-out. Time and battery stay small on top.">
      <Strip left={<span className="text-ink">9:41 · Day 94</span>} right={<span className="flex items-center gap-2 text-muted"><StatusDot tone="ok" />Joined · 64%</span>} />
      <div className="grid flex-1 grid-cols-3 content-center gap-4">
        <Column first label="Pulse" value="72" unit="bpm" read="Normal for you" />
        <Column label="SpO₂" value="98" unit="%" read="Good" />
        <Column label="Skin temp" value="36.4" unit="°C" read="Normal" />
      </div>
    </BandScreenRect>
  )
}

export function BandEcg() {
  return (
    <BandScreenRect number="B2" caption="Wristband, ECG check, on demand. One finger of each hand on the two metal pads on top of the screen part, still, for 30 s. It doesn't need the wrist skin, so it works over the suit with bare hands.">
      <Strip left={<span className="uppercase tracking-label text-info">ECG check</span>} right={<span className="text-muted">Signal good</span>} />
      <div className="flex flex-1 items-center gap-4">
        <Ring value={40} size={64} thickness={6} tone="info"><span className="text-lg text-ink">18</span></Ring>
        <Sparkline data={ecgTrace(4)} tone="ink" width={260} height={56} className="flex-1" />
      </div>
      <p className="flex items-center gap-2 text-sm text-ink"><Fingerprint size={16} />Touch both metal pads · stay still · 18 s left</p>
    </BandScreenRect>
  )
}

export function BandHelper() {
  return (
    <BandScreenRect tone="critical" number="B3" caption="Wristband, second helper. Eva sends this person to the emergency station for the AED, with an arrow to follow and one press to say &quot;I'm going&quot;.">
      <Strip left={<span className="uppercase tracking-label text-ink">Emergency · bring the AED</span>} right={<span className="text-ink">4 m</span>} />
      <div className="flex flex-1 items-center gap-4">
        <span className="grid size-14 shrink-0 place-items-center rounded-full bg-ink text-on-light"><ArrowUpLeft size={28} /></span>
        <div className="flex flex-col gap-1">
          <p className="text-3xl leading-none tracking-tight text-ink">Get the AED</p>
          <p className="text-sm text-ink">Emergency station · corridor B · then Ana, seat 4B</p>
        </div>
      </div>
      <div className="flex items-center justify-between">
        <span className="rounded-full bg-ink px-4 py-1.5 text-sm text-on-light">Press · I'm going</span>
        <span className="text-xs text-ink">Hold 2 s · I can't</span>
      </div>
    </BandScreenRect>
  )
}

export function BandCpr() {
  return (
    <BandScreenRect number="B4" caption="Wristband, CPR step. The ring pulses at 110 a minute and the band vibrates with it, so the helper can follow without looking. The AED, not Eva, decides any shock.">
      <Strip left={<span className="uppercase tracking-label text-critical">Step 2 of 4</span>} right={<div className="w-28"><StepBars step={2} total={4} tone="critical" /></div>} />
      <div className="flex flex-1 items-center gap-4">
        <span className="relative grid size-20 shrink-0 place-items-center">
          <span className="absolute inset-0 animate-ping rounded-full border-2 border-critical/40" />
          <span className="absolute inset-0 rounded-full border-2 border-critical/70" />
          <span className="relative flex flex-col items-center"><span className="text-2xl text-ink">110</span><span className="text-xs text-muted">a minute</span></span>
        </span>
        <p className="text-xl leading-tight tracking-tight text-ink">Push hard and fast in the middle of the chest</p>
      </div>
      <p className="flex items-center gap-2 text-xs text-muted"><StatusDot tone="ok" /><span className="text-ink">Good depth · keep going</span>· AED checks in 0:40</p>
    </BandScreenRect>
  )
}

export function BandShelter() {
  return (
    <BandScreenRect number="B5" caption="Wristband, shelter. Countdown, the next turn and how many people are already safe.">
      <Strip left={<span className="uppercase tracking-label text-critical">Solar storm</span>} right={<span className="text-muted">9/15 in shelter</span>} />
      <div className="flex flex-1 items-center justify-between gap-4">
        <div className="flex flex-col gap-1">
          <p className="font-mono text-5xl leading-none tracking-tight text-ink">08:40</p>
          <p className="text-xs text-muted">to reach the shelter</p>
        </div>
        <p className="flex items-center gap-2 text-2xl tracking-tight text-ink"><ArrowLeft size={26} />Turn left</p>
      </div>
      <p className="text-xs text-muted">Then follow the floor arrows to Deck B · 40 m</p>
    </BandScreenRect>
  )
}
