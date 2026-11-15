// Passenger app and wristband screen wireframes for Eva by WearMeds.
import { Eyebrow, Body } from "@/ds"
import { Welcome, Home, HomeGrid, Heart, Plan } from "./everyday"
import { Food, Review } from "./food"
import { Logbook, SuitMode } from "./records"
import { Helper, Cpr, Storm } from "./emergency"
import { EcgCheck, EcgResult } from "./ecg"
import { CheckIn } from "./checkin"
import { BandIdle, BandEcg, BandHelper, BandCpr, BandShelter } from "./band"

const Group = ({ eyebrow, intro, children }) => (
  <div className="flex flex-col gap-6">
    <div className="flex max-w-2xl flex-col gap-2"><Eyebrow>{eyebrow}</Eyebrow>{intro && <Body muted>{intro}</Body>}</div>
    <div className="flex flex-wrap gap-10">{children}</div>
  </div>
)

export default function MobileScreens() {
  return (
    <div className="flex flex-col gap-16">
      <p className="-mt-4 max-w-2xl text-xs text-faint">Interaction principles from Russell et al., npj Microgravity 9:46 (2023) · doi:10.1038/s41526-023-00284-1</p>
      <Group eyebrow="Every day" intro="Start, check in, follow the plan. Calm screens, one accent at a time. Photos are AI-generated concept renders.">
        <Welcome /><Home /><HomeGrid /><Heart /><EcgCheck /><EcgResult /><Plan /><Food /><Review /><Logbook /><CheckIn /><SuitMode />
      </Group>
      <Group eyebrow="When something happens" intro="Red only when someone may be in danger now. One order per screen, verbs first. Every guided step says what to do, why, how Eva decided and what comes next, with feedback on whether it is working.">
        <Helper /><Cpr /><Storm />
      </Group>
      <Group eyebrow="Wristband screen" intro="The rugged module's flat 2:1 screen. Same words as the phone, bigger, for a glance mid-task.">
        <BandIdle /><BandEcg /><BandHelper /><BandCpr /><BandShelter />
      </Group>
    </div>
  )
}
