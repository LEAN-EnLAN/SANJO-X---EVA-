// Eva on the wristband's own screen: reading you live, and asking how you feel.
import { StatusDot } from "@/ds"
import { BandScreenRect, Label } from "@/ds/mobile-extra"
import { EvaEyes, Waveform } from "@/ds/eva-extra"
import { Mic } from "lucide-react"

const Column = ({ label, value, unit, read, first }) => (
  <div className={first ? "flex flex-col gap-1" : "flex flex-col gap-1 border-l border-line pl-4"}>
    <Label>{label}</Label>
    <p className="text-3xl leading-none tracking-tight text-ink">{value}<span className="ml-1 text-sm text-muted">{unit}</span></p>
    <p className="text-xs text-muted">{read}</p>
  </div>
)

export function BandEvaLive() {
  return (
    <BandScreenRect number="E7" caption="Band · Eva live">
      <div className="flex items-center justify-between gap-3">
        <span className="flex items-center gap-3"><EvaEyes size="sm" /><span className="text-sm text-ink">Eva · reading you live</span></span>
        <StatusDot tone="info" pulse />
      </div>
      <div className="grid flex-1 grid-cols-3 content-center gap-4">
        <Column first label="Pulse" value="72" unit="bpm" read="Normal for you" />
        <Column label="SpO₂" value="98" unit="%" read="Good" />
        <Column label="Skin temp" value="36.4" unit="°C" read="Normal" />
      </div>
    </BandScreenRect>
  )
}

export function BandEvaAsk() {
  return (
    <BandScreenRect number="E7b" caption="Band · Eva asks">
      <div className="flex flex-1 items-center gap-5">
        <EvaEyes size="lg" mode="happy" />
        <div className="flex flex-col gap-1">
          <Label>Eva · 10:35</Label>
          <p className="text-3xl leading-none tracking-tight text-ink">How do you feel?</p>
        </div>
      </div>
      <div className="flex items-center gap-3">
        <span className="flex items-center gap-2 rounded-full bg-ink py-1.5 pr-4 pl-3 text-sm text-on-light"><Mic size={16} />Hold to talk</span>
        <Waveform tone="info" data={[20, 35, 60, 40, 75, 50, 30, 55, 80, 45, 30, 50, 25, 15, 30, 20, 10, 15]} />
      </div>
    </BandScreenRect>
  )
}
