import { Phone, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Ring, Sparkline } from "@/ds"
import { PhoneTop, WhyNext, IconLine, Hero, ecgTrace } from "@/ds/mobile-extra"
import { ChevronLeft, Info, X, Check, Hand, Hospital, Zap, Send } from "lucide-react"

export function EcgCheck() {
  return (
    <Phone number="03b" caption="ECG check">
      <PhoneTop left={<IconButton label="Stop"><X size={18} /></IconButton>} title="ECG check" right={<IconButton label="How it works"><Info size={18} /></IconButton>} />
      <div className="flex flex-col gap-2">
        <p className="text-3xl leading-tight tracking-tight text-ink">Touch both metal pads</p>
        <div className="flex flex-wrap gap-2"><Badge><Hand size={12} />1 finger each hand</Badge><Badge>Stay still</Badge><Badge>30 s</Badge></div>
      </div>
      <Hero src="img/wrist/wrist-ecg.webp" alt="One finger of each hand on the wristband's two metal pads">
        <span className="absolute top-3 left-3 flex items-center gap-2 rounded-full bg-background/80 py-1 pr-3 pl-1">
          <Ring value={40} size={44} thickness={4} tone="info"><span className="text-sm text-ink">18</span></Ring>
          <span className="text-xs uppercase tracking-label text-muted">seconds left</span>
        </span>
      </Hero>
      <Card className="py-4">
        <CardHeader className="mb-2 items-center">
          <CardTitle>Live trace</CardTitle>
          <span className="flex items-center gap-2 text-xs text-muted"><StatusDot tone="ok" />Good contact</span>
        </CardHeader>
        <Sparkline data={ecgTrace(5)} tone="ink" height={64} />
      </Card>
      <div className="mt-auto">
        <WhyNext why="Pulse uneven twice today" next="Result in ~1 min" how="Wrist pulse, at rest" />
      </div>
    </Phone>
  )
}

export function EcgResult() {
  return (
    <Phone number="03c" caption="ECG result">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="ECG result" right={<span className="size-11" />} />
      <div className="flex flex-col gap-3">
        <Badge tone="ok" className="self-start">Normal for you</Badge>
        <p className="text-5xl leading-none tracking-tight text-ink">Regular rhythm</p>
        <p className="text-sm text-muted">72 bpm · steady 30 s</p>
      </div>
      <Card className="py-4">
        <Sparkline data={ecgTrace(5)} tone="ink" height={64} />
        <p className="mt-3 text-xs text-muted">10:04 · 1 lead · 30 s</p>
      </Card>
      <div className="flex flex-col gap-3">
        <IconLine icon={Send} tone="info">Sent to the hub</IconLine>
        <IconLine icon={Check} tone="ok">Nothing to do</IconLine>
      </div>
      <div className="mt-auto flex flex-col gap-2.5 rounded-tile bg-surface px-4 py-3">
        <IconLine icon={Hospital}>Not a hospital ECG</IconLine>
        <IconLine icon={Hand}>Only works while still</IconLine>
        <IconLine icon={Zap}>Shocks: the AED decides</IconLine>
      </div>
    </Phone>
  )
}
