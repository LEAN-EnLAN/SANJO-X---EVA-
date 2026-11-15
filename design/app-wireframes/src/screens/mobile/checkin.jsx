import { Phone, IconButton, Badge } from "@/ds"
import { PhoneTop, Label } from "@/ds/mobile-extra"
import { X, ArrowRight, Smile, Meh, Frown } from "lucide-react"

const answers = [
  { icon: Smile, title: "Yes, better", next: "Closed in your logbook" },
  { icon: Meh, title: "The same", next: "Rest 30 min · asks again 11:05" },
  { icon: Frown, title: "Worse", next: "Crew lead told now" },
]

export function CheckIn() {
  return (
    <Phone number="07b" caption="Check-in">
      <PhoneTop left={<IconButton label="Later"><X size={18} /></IconButton>} title="Check-in" />
      <div className="flex flex-col gap-2">
        <Label>10:35</Label>
        <p className="text-5xl leading-none tracking-tight text-ink">Do you feel better?</p>
      </div>
      <div className="flex flex-wrap gap-2"><Badge>10:05 · headache</Badge><Badge tone="info">Water + 20 min rest</Badge></div>
      <div className="flex flex-col gap-3">
        {answers.map(a => (
          <button key={a.title} className="flex items-center gap-4 rounded-card border border-line bg-surface px-5 py-4 text-left hover:bg-raised">
            <a.icon size={24} className="shrink-0 text-ink" />
            <span className="flex flex-1 flex-col gap-0.5">
              <span className="text-lg text-ink">{a.title}</span>
              <span className="text-xs text-muted">{a.next}</span>
            </span>
            <ArrowRight size={18} className="text-muted" />
          </button>
        ))}
      </div>
      <p className="mt-auto text-xs text-faint">Saved on board</p>
    </Phone>
  )
}
