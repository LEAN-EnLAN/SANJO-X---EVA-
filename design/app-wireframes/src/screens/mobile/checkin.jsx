import { Phone, Body, IconButton } from "@/ds"
import { PhoneTop, Label } from "@/ds/mobile-extra"
import { X, ArrowRight } from "lucide-react"

const answers = [
  { title: "Yes, better", next: "Eva closes it in your logbook" },
  { title: "The same", next: "Rest 30 min more · Eva asks again at 11:05" },
  { title: "Worse", next: "Eva tells the crew lead now" },
]

export function CheckIn() {
  return (
    <Phone number="07b" caption="Check-in, 30 min later. After any advice, Eva asks again and the answer decides the next step. Each choice says what happens next.">
      <PhoneTop left={<IconButton label="Later"><X size={18} /></IconButton>} title="Check-in" />
      <div className="flex flex-col gap-2">
        <Label>30 min later · 10:35</Label>
        <p className="text-5xl leading-none tracking-tight text-ink">Do you feel better?</p>
      </div>
      <Body muted>At 10:05 you said you had a headache. Eva suggested a glass of water and 20 min of rest.</Body>
      <div className="flex flex-col gap-3">
        {answers.map(a => (
          <button key={a.title} className="flex items-center gap-4 rounded-card border border-line bg-surface px-5 py-4 text-left hover:bg-raised">
            <span className="flex flex-1 flex-col gap-0.5">
              <span className="text-lg text-ink">{a.title}</span>
              <span className="text-xs text-muted">{a.next}</span>
            </span>
            <ArrowRight size={18} className="text-muted" />
          </button>
        ))}
      </div>
      <p className="mt-auto text-xs text-faint">Your answer is saved on board. Eva guides, records and warns; it does not diagnose.</p>
    </Phone>
  )
}
