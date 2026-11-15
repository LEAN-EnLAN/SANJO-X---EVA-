import { TabBar } from "@/ds"
import { House, ListChecks, Utensils, BookOpen, MessageCircle } from "lucide-react"

const tabs = [
  { icon: House, label: "Home" },
  { icon: ListChecks, label: "Plan" },
  { icon: Utensils, label: "Food" },
  { icon: BookOpen, label: "Logbook" },
  { icon: MessageCircle, label: "Ask Eva" },
]

/** the floating pill tab bar shared by the everyday screens */
export const Tabs = ({ active }) => <TabBar items={tabs} active={active} />
