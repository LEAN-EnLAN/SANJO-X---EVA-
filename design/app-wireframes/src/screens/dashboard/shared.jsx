import { Avatar, Button, IconButton } from "@/ds"
import { IconRail, LogoPill, SearchField } from "@/ds/dashboard-extra"
import { Bell, Settings, Plus, House, Users, HeartPulse, Map, Sun, Satellite, FileText, LifeBuoy } from "lucide-react"

export const railItems = [
  { icon: House, label: "Overview" },
  { icon: Users, label: "Passengers" },
  { icon: HeartPulse, label: "Cases", dot: "critical" },
  { icon: Map, label: "Deck plan" },
  { icon: Sun, label: "Solar storms" },
  { icon: Satellite, label: "Earth link" },
  { icon: FileText, label: "Records" },
]
export const railFooter = [{ icon: LifeBuoy, label: "Help" }]

const RUSSELL_DOI = "https://doi.org/10.1038/s41526-023-00284-1"

/** subtle source line shown at the foot of each dashboard window */
export const Citation = () => (
  <p>Decision-support functions after Russell et al., npj Microgravity 9:46 (2023) · <a href={RUSSELL_DOI} className="text-info hover:underline">doi:10.1038/s41526-023-00284-1</a></p>
)

export const Rail = ({ active }) => <IconRail items={railItems} active={active} footer={railFooter} />

/** top bar: logo pill, wide search, outlined + actions, round bell + settings, avatar */
export const TopBar = ({ search, actions, person, bellDot = true }) => (
  <header className="flex items-center gap-3 border-b border-line px-6 py-4">
    <LogoPill />
    <SearchField placeholder={search} className="mx-2" />
    <div className="flex items-center gap-2">
      {actions.map(a => <Button key={a} variant="outline" size="sm"><Plus size={14} />{a}</Button>)}
    </div>
    <div className="ml-2 flex items-center gap-2">
      <span className="relative">
        <IconButton label="Alerts"><Bell size={18} /></IconButton>
        {bellDot && <span className="absolute top-2 right-2.5 size-2 rounded-full bg-critical ring-2 ring-raised" />}
      </span>
      <IconButton label="Settings"><Settings size={18} /></IconButton>
      <span className="ml-1 flex items-center gap-2.5">
        <Avatar name={person.name} tone="info" />
        <span className="flex flex-col leading-tight"><span className="text-sm text-ink">{person.name}</span><span className="text-xs text-muted">{person.role}</span></span>
      </span>
    </div>
  </header>
)
