import { TabBar } from "@/ds"
import { EvaMascot } from "@/ds/eva-extra"
import { House, ListChecks, Utensils, BookOpen } from "lucide-react"

/** Eva's mascot as the centre tab */
const EvaTab = () => <EvaMascot size="sm" alt="" />

const tabs = [
  { icon: House, label: "Home" },
  { icon: ListChecks, label: "Plan" },
  { icon: EvaTab, label: "Eva", center: true },
  { icon: Utensils, label: "Food" },
  { icon: BookOpen, label: "Logbook" },
]

/** the floating pill tab bar shared by the everyday screens; Eva sits in the centre */
export const Tabs = ({ active }) => <TabBar items={tabs} active={active} />
