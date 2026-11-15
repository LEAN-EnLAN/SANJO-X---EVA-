// Eva, the health copilot: talk, multimodal chat, likely causes, daily log.
import { Phone, IconButton, Card, CardHeader, CardTitle, Badge, StatusDot, Ring, Sparkline } from "@/ds"
import { PhoneTop, Label, PhotoTile, WhyNext, Glasses } from "@/ds/mobile-extra"
import { EvaMascot, Bubble, VoiceNote, MediaBubble, Chip, ProbRow, Trail, Orb } from "@/ds/eva-extra"
import { Mic, History, Keyboard, ChevronLeft, MoreHorizontal, Camera, Video, Droplet, Info, CalendarDays, Share2, Activity, HeartPulse, CircleHelp, NotebookPen, Frown, CalendarRange } from "lucide-react"
import { Tabs } from "./shared"

/** live reading from the band: icon, value, plain read-out */
const Live = ({ src, value, unit, read }) => (
  <div className="flex flex-1 items-center gap-2">
    <PhotoTile src={src} size="xs" />
    <span className="flex flex-col leading-tight">
      <span className="text-base tracking-tight text-ink">{value}<span className="ml-0.5 text-xs text-muted">{unit}</span></span>
      <span className="text-xs text-muted">{read}</span>
    </span>
  </div>
)

export function EvaHome() {
  return (
    <Phone number="E1" caption="Talk to Eva" tabbar={<Tabs active="Eva" />}>
      <PhoneTop left={<IconButton label="Past talks"><History size={18} /></IconButton>} title="Eva" right={<IconButton label="Type instead"><Keyboard size={18} /></IconButton>} />
      <div className="flex flex-col items-center gap-2">
        <EvaMascot mood="wink" size="xl" glow labelled />
        <p className="text-center text-3xl leading-tight tracking-tight text-ink">Hi Ana. How can I help?</p>
        <p className="flex items-center gap-2 text-sm text-muted"><StatusDot tone="info" pulse />I'm reading your wristband live</p>
      </div>
      <div className="flex gap-2 rounded-tile bg-surface px-3 py-2.5">
        <Live src="img/icons/heart.webp" value="72" unit="bpm" read="Calm" />
        <Live src="img/icons/oxygen.webp" value="98" unit="%" read="Good" />
        <Live src="img/icons/skin-temp.webp" value="36.4" unit="°C" read="Normal" />
      </div>
      <div className="grid grid-cols-2 gap-2">
        <Chip size="sm" icon={CircleHelp}>How am I?</Chip>
        <Chip size="sm" icon={NotebookPen}>Log my day</Chip>
        <Chip size="sm" icon={Frown}>I feel unwell</Chip>
        <Chip size="sm" icon={CalendarRange}>Show my week</Chip>
      </div>
      <div className="mt-auto flex items-center justify-center gap-6">
        <IconButton label="Send a photo"><Camera size={18} /></IconButton>
        <span className="relative grid size-22 place-items-center">
          <span className="absolute inset-0 rounded-full bg-info/10 ring-1 ring-info/30" />
          <span className="absolute inset-0 animate-ping rounded-full bg-info/10 motion-reduce:animate-none" />
          <IconButton tone="light" size="xl" label="Hold to talk"><Mic size={30} /></IconButton>
        </span>
        <IconButton label="Send a video"><Video size={18} /></IconButton>
      </div>
      <p className="-mt-2 text-center text-xs text-muted">Hold to talk</p>
    </Phone>
  )
}

/** a small card Eva drops into the chat: three readings from the band */
const DataCard = () => (
  <div className="flex flex-col gap-2 rounded-tile bg-background px-3 py-2.5 ring-1 ring-line">
    <span className="flex items-center gap-1.5 text-xs text-muted"><Activity size={12} className="text-info" />From your band · now</span>
    <div className="grid grid-cols-3 gap-2">
      {[["78", "bpm", "A bit up"], ["97", "%", "Good"], ["36.9", "°C", "Normal"]].map(([v, u, r]) => (
        <span key={u} className="flex flex-col leading-tight"><span className="text-lg tracking-tight text-ink">{v}<span className="ml-0.5 text-xs text-muted">{u}</span></span><span className="text-xs text-muted">{r}</span></span>
      ))}
    </div>
  </div>
)

export function EvaChat() {
  return (
    <Phone number="E2" caption="Voice, photo, video">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Eva · live" right={<IconButton label="More"><MoreHorizontal size={18} /></IconButton>} />
      <div className="flex min-h-0 flex-1 flex-col justify-end gap-2 overflow-hidden">
        <Bubble from="me">
          <VoiceNote data={[30, 55, 80, 45, 90, 60, 35, 70, 95, 50, 40, 75, 55, 30, 20, 45]} length="0:08" />
          <span className="text-xs text-muted">"Dry cough since lunch."</span>
        </Bubble>
        <Bubble mood="care">
          <span>Your band looks fine.</span>
          <DataCard />
          <span>Can you film one cough?</span>
        </Bubble>
        <MediaBubble video src="img/icons/lungs.webp" alt="Video of a cough" length="0:10" caption="Cough" />
        <Bubble mood="happy">Dry cough. Breathing sounds clear.</Bubble>
        <MediaBubble src="img/icons/meal-lunch.webp" alt="Photo of lunch" caption="Lunch" />
        <Bubble>Lentil curry · ~780 kcal · logged.</Bubble>
      </div>
      <div className="flex items-center gap-1.5 rounded-full bg-raised p-1.5">
        <IconButton tone="ghost" size="sm" label="Camera"><Camera size={17} /></IconButton>
        <IconButton tone="ghost" size="sm" label="Video"><Video size={17} /></IconButton>
        <span className="min-w-0 flex-1 truncate px-1 text-sm text-faint">Message Eva</span>
        <IconButton tone="light" label="Hold to talk"><Mic size={18} /></IconButton>
      </div>
    </Phone>
  )
}

export function EvaCauses() {
  return (
    <Phone number="E3" caption="Diagnostic support">
      <PhoneTop left={<IconButton label="Back"><ChevronLeft size={20} /></IconButton>} title="Eva · check" right={<IconButton label="How this works"><Info size={18} /></IconButton>} />
      <div className="flex flex-wrap gap-2"><Badge>Headache</Badge><Badge>Nausea</Badge><Badge>Since 09:40</Badge></div>
      <p className="text-3xl leading-tight tracking-tight text-ink">Most likely: not enough water</p>
      <Card className="py-4">
        <CardHeader className="mb-3 items-center">
          <CardTitle>Diagnostic support · likely causes</CardTitle>
          <span className="text-xs text-faint">Illustrative</span>
        </CardHeader>
        <div className="flex flex-col gap-3">
          <ProbRow lead label="Dehydration" pct={46} tone="info" />
          <ProbRow label="Space adaptation" pct={31} tone="muted" />
          <ProbRow label="Migraine" pct={15} tone="muted" />
          <ProbRow label="Other" pct={8} other />
        </div>
      </Card>
      <div className="flex items-center gap-3 rounded-tile bg-info/10 px-4 py-3">
        <Droplet size={20} className="shrink-0 text-info" />
        <span className="flex flex-col"><span className="text-xs uppercase tracking-label text-info">Do now</span><span className="text-sm text-ink">Drink 500 ml · rest 20 min</span></span>
      </div>
      <WhyNext why="Water 0.9 of 2 L · pulse +12" next="Eva asks again in 30 min" how="band data + your answers + your normal" />
      <div className="mt-auto flex flex-wrap gap-2">
        <Badge tone="ok"><StatusDot tone="ok" />Shared with crew lead</Badge>
        <Badge tone="info">Queued for Earth</Badge>
        <Badge>Diagnostic support · the crew lead confirms</Badge>
      </div>
    </Phone>
  )
}

/** one tile of the day summary: icon, label, value, read-out vs the person's normal */
const DayTile = ({ src, label, children, read, tone = "muted" }) => (
  <div className="flex flex-col gap-2 rounded-tile bg-surface p-3">
    <span className="flex items-center gap-2"><PhotoTile src={src} size="xs" /><span className="text-xs uppercase tracking-label text-muted">{label}</span></span>
    {children}
    <span className={tone === "ok" ? "text-xs text-ok" : tone === "warn" ? "text-xs text-warn" : "text-xs text-muted"}>{read}</span>
  </div>
)

export function DailyLog() {
  return (
    <Phone number="E4" caption="Daily log">
      <PhoneTop left={<IconButton label="Pick a day"><CalendarDays size={18} /></IconButton>} title="Day 94 · log" right={<IconButton label="Share"><Share2 size={18} /></IconButton>} />
      <div className="flex items-center gap-3">
        <EvaMascot mood="happy" size="md" />
        <div className="flex flex-col">
          <Label>Written by Eva · 14:00</Label>
          <p className="text-xl leading-tight tracking-tight text-ink">A good day. Drink a bit more.</p>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <DayTile src="img/icons/sleep.webp" label="Sleep" read="Normal for you" tone="ok">
          <div className="flex items-center gap-3"><Ring value={90} size={44} thickness={5} tone="info" /><p className="text-2xl tracking-tight text-ink">7 h 10</p></div>
        </DayTile>
        <DayTile src="img/icons/exercise.webp" label="Activity" read="Goal met" tone="ok">
          <div className="flex items-center gap-3"><Ring value={100} size={44} thickness={5} tone="ok" /><p className="text-2xl tracking-tight text-ink">2 h</p></div>
        </DayTile>
        <DayTile src="img/icons/water.webp" label="Water" read="1.2 of 2 L · drink more" tone="warn">
          <Glasses filled={5} total={8} />
        </DayTile>
        <DayTile src="img/icons/food.webp" label="Food" read="3 meals · on plan" tone="ok">
          <p className="text-2xl tracking-tight text-ink">1,640<span className="ml-1 text-xs text-muted">kcal</span></p>
        </DayTile>
        <DayTile src="img/icons/heart.webp" label="Vitals" read="Inside your normal" tone="ok">
          <Sparkline data={[64, 61, 58, 62, 74, 88, 76, 72]} band={[58, 82]} tone="ink" height={44} />
        </DayTile>
        <DayTile src="img/icons/eva-orb.webp" label="Mood" read="Pleasant · calm">
          <div className="flex items-center gap-3"><Orb valence={4} size="sm" /><p className="text-2xl tracking-tight text-ink">Calm</p></div>
        </DayTile>
      </div>
      <Card className="mt-auto py-4">
        <CardHeader className="mb-3 items-center">
          <CardTitle>Sent to mission control</CardTitle>
          <HeartPulse size={16} className="text-ok" />
        </CardHeader>
        <Trail steps={[{ label: "Signed on board", time: "14:02", state: "done" }, { label: "Sent", time: "14:07", state: "done" }, { label: "Acknowledged", time: "14:21", state: "done" }]} />
      </Card>
    </Phone>
  )
}
