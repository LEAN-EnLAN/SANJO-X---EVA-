// Passenger app and wristband screen wireframes for Eva by WearMeds.
import { Eyebrow } from "@/ds"
import { Welcome, Home, HomeGrid, Heart, Plan } from "./everyday"
import { Food, Review } from "./food"
import { Logbook, SuitMode } from "./records"
import { Helper, Cpr, Storm } from "./emergency"
import { EcgCheck, EcgResult } from "./ecg"
import { CheckIn } from "./checkin"
import { EvaHome, EvaChat, EvaCauses, DailyLog } from "./eva"
import { StateOfMind, WriteItDown, JournalEntries } from "./journal"
import { BandEvaLive, BandEvaAsk } from "./eva-band"
import { BandIdle, BandEcg, BandHelper, BandCpr, BandShelter } from "./band"

const Group = ({ eyebrow, children }) => (
  <div className="flex flex-col gap-6">
    <Eyebrow>{eyebrow}</Eyebrow>
    <div className="flex flex-wrap gap-10">{children}</div>
  </div>
)

export default function MobileScreens() {
  return (
    <div className="flex flex-col gap-16">
      <p className="-mt-4 text-xs text-faint">Russell et al., npj Microgravity 9:46 (2023)</p>
      <Group eyebrow="Eva · your health copilot">
        <EvaHome /><EvaChat /><EvaCauses /><DailyLog /><StateOfMind /><WriteItDown /><JournalEntries />
        <BandEvaLive /><BandEvaAsk />
      </Group>
      <Group eyebrow="Every day">
        <Welcome /><Home /><HomeGrid /><Heart /><EcgCheck /><EcgResult /><Plan /><Food /><Review /><Logbook /><CheckIn /><SuitMode />
      </Group>
      <Group eyebrow="When something happens">
        <Helper /><Cpr /><Storm />
      </Group>
      <Group eyebrow="Wristband screen">
        <BandIdle /><BandEcg /><BandHelper /><BandCpr /><BandShelter />
      </Group>
    </div>
  )
}
