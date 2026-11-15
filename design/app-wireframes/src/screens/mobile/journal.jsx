// Journaling with Eva: State of Mind check-in, past entries and one insight.
import { IconButton, Card, Badge, Phone } from "@/ds"
import { PhoneTop, Label } from "@/ds/mobile-extra"
import { EvaMascot, Chip, Orb, ValenceSlider, MoodWeek, moodWord } from "@/ds/eva-extra"
import { X, Check, Mic, ChevronLeft, Search, Lock, Users, BookOpen } from "lucide-react"

const week = [
  { day: "M", moods: [4] }, { day: "T", moods: [3] }, { day: "W", moods: [2, 3] }, { day: "T", moods: [4] },
  { day: "F", moods: [2] }, { day: "S", moods: [5] }, { day: "S", moods: [4] },
]

export function StateOfMind() {
  return (
    <Phone number="E5" caption="State of mind">
      <PhoneTop left={<IconButton label="Close"><X size={18} /></IconButton>} title="Journal · 21:10" right={<IconButton tone="light" label="Next"><Check size={18} /></IconButton>} />
      <div className="flex flex-col items-center gap-5 pt-4">
        <Orb valence={4} size="lg" breathe />
        <div className="flex flex-col items-center gap-1">
          <Label>How do you feel right now?</Label>
          <p className="text-4xl leading-none tracking-tight text-ink">{moodWord[4]}</p>
        </div>
      </div>
      <ValenceSlider value={70} />
      <div className="flex flex-col gap-2">
        <Label>You feel</Label>
        <div className="flex flex-wrap gap-1.5">
          <Chip size="sm" active>Calm</Chip><Chip size="sm">Anxious</Chip><Chip size="sm" active>Homesick</Chip>
          <Chip size="sm">Grateful</Chip><Chip size="sm">Tired</Chip><Chip size="sm">Focused</Chip>
        </div>
      </div>
      <div className="flex flex-col gap-2">
        <Label>What's affecting you?</Label>
        <div className="flex flex-wrap gap-1.5">
          <Chip size="sm">Crew</Chip><Chip size="sm">Sleep</Chip><Chip size="sm">Work</Chip><Chip size="sm" active>Family on Earth</Chip><Chip size="sm">Health</Chip>
        </div>
      </div>
      <p className="mt-auto text-center text-xs text-muted">Next · write a few lines, or just save</p>
    </Phone>
  )
}

export function WriteItDown() {
  return (
    <Phone number="E5b" caption="Write it down">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Journal · 21:12" right={<IconButton tone="light" label="Save"><Check size={18} /></IconButton>} />
      <div className="flex items-center gap-3">
        <Orb valence={4} size="sm" />
        <div className="flex flex-col"><span className="text-sm text-ink">{moodWord[4]}</span><span className="text-xs text-muted">Calm · homesick · family on Earth</span></div>
      </div>
      <div className="flex flex-1 flex-col gap-3 rounded-card bg-surface p-4">
        <div className="flex items-start gap-2.5">
          <EvaMascot mood="writing" size="xs" />
          <p className="text-sm text-info">Write for 5 minutes. What's on your mind, and why?</p>
        </div>
        <p className="text-lg leading-snug text-ink">Mum's message came through today. I miss the sea, the noise of the street. The crew helps<span className="animate-pulse text-info">|</span></p>
        <div className="mt-auto flex items-center justify-between">
          <span className="flex items-center gap-1.5 text-xs text-muted"><Lock size={12} />Only you · 2:40 of 5 min</span>
          <IconButton size="sm" label="Speak instead"><Mic size={14} /></IconButton>
        </div>
      </div>
      <Card className="py-4">
        <div className="mb-3 flex items-center justify-between"><Label>This week</Label><span className="text-xs text-muted">8 entries</span></div>
        <MoodWeek days={week} today="S" />
      </Card>
    </Phone>
  )
}

const entries = [
  { v: 4, when: "Today · 21:10", title: "Calm · homesick", text: "Mum's message came through.", shared: false },
  { v: 2, when: "Fri · 22:40", title: "Tired · anxious", text: "5 h sleep. Long shift.", shared: true },
  { v: 5, when: "Sat · 19:05", title: "Grateful", text: "Movie night with the crew.", shared: false },
]

/** sleep per night with the mood colour of that day; nights under 6 h are hatched */
const SleepMood = () => {
  const nights = [[7.2, 4], [6.8, 3], [5.4, 2], [7, 4], [5, 2], [7.6, 5], [7.1, 4]]
  return (
    <div className="flex h-20 items-end gap-2">
      {nights.map(([h, v], i) => (
        <div key={i} className="flex h-full flex-1 flex-col items-center justify-end gap-1.5">
          <Orb valence={v} size="dot" />
          <span className={h < 6 ? "h-(--h) w-full rounded-md bg-hatch ring-1 ring-line" : "h-(--h) w-full rounded-md bg-elevated"} style={{ "--h": `${(h / 8) * 70}%` }} />
        </div>
      ))}
    </div>
  )
}

export function JournalEntries() {
  return (
    <Phone number="E6" caption="Journal and insight">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Journal" right={<IconButton label="Search"><Search size={18} /></IconButton>} />
      <Card tone="info" className="flex flex-col gap-3 py-4">
        <div className="flex items-start gap-3">
          <EvaMascot mood="care" size="sm" />
          <div className="flex flex-col gap-0.5">
            <p className="text-xs uppercase tracking-label text-info">Eva noticed</p>
            <p className="text-lg leading-tight tracking-tight text-ink">Your mood dips on days with less than 6 h sleep.</p>
          </div>
        </div>
        <SleepMood />
        <p className="text-xs text-muted">Sleep · dot = mood · hatched = under 6 h</p>
      </Card>
      <div className="flex flex-col gap-2">
        <Label>Entries</Label>
        {entries.map(e => (
          <div key={e.when} className="flex items-center gap-3 rounded-tile bg-surface px-3 py-3">
            <Orb valence={e.v} size="sm" />
            <div className="flex min-w-0 flex-1 flex-col">
              <span className="text-xs text-muted">{e.when}</span>
              <span className="text-sm text-ink">{e.title}</span>
              <span className="truncate text-xs text-muted">{e.text}</span>
            </div>
            {e.shared ? <Badge tone="info"><Users size={11} />Crew lead</Badge> : <Badge><Lock size={11} />Private</Badge>}
          </div>
        ))}
      </div>
      <p className="flex items-center gap-2 text-sm text-ink"><Lock size={14} className="shrink-0 text-info" />Private: you choose what is shared.</p>
      <div className="mt-auto flex flex-col gap-2">
        <p className="flex gap-2 text-xs text-faint"><BookOpen size={12} className="mt-0.5 shrink-0" /><span>Why writing helps · Pennebaker &amp; Beall 1986; Frattaroli 2006; Lieberman et al. 2007</span></p>
      </div>
    </Phone>
  )
}
