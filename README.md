# Eva by WearMeds

**Team Sanjo-X · NASA Space Apps Challenge 2026 · Rosario, Argentina**
Challenge: *Create Health Monitoring Software for Astronauts on Space Missions*

Eva is a **Super Intelligence health copilot for everyone on board**: crew and commercial passengers on flights in space, where there is no doctor and Earth can be minutes away. She lives on a rugged two-module wristband and talks to the ship's emergency station. Eva **guides, records and warns**. She gives diagnostic support only; **the AED decides the shock** and the crew lead decides treatment.

## See it live

| | Link |
|---|---|
| Landing | https://eva-wearmeds.vercel.app |
| Final pitch (6 slides, arrow keys, `N` for notes) | https://eva-wearmeds.vercel.app/pitch/ |
| 30-second video (EN/ES) | https://eva-wearmeds.vercel.app/video/ |
| App demo: every view full screen, live Eva chat, voice, photo | https://eva-wearmeds.vercel.app/app/ |
| Android APK: online demo, offline demo or connect to a ship hub | https://eva-wearmeds.vercel.app/download/ |
| Research hub (sources and decisions) | https://eva-wearmeds.vercel.app/hub/ |

## What Eva does

- **Monitors** heart rate, SpO₂, skin temperature, EDA and movement against *your own* baseline. It also runs a 30 s ECG on demand: two fingers of the same hand on the two metal pads.
- **Cardiac arrest:** no pulse, no movement and no answer. In the same second Eva sends the nearest passenger to start CPR and paces every compression, while a second person is sent to the station, which lights the AED compartment. Rule: within **10 seconds of Eva calling**, everyone knows what to do, where and how.
- **Solar particle events:** guides each person to the shelter turn by turn, re-routing when a corridor is blocked. Space weather comes from NASA DONKI and NOAA GOES.
- **Everyday care:** a journal by voice, mood check-ins, sleep, water and food. If someone says "I feel alone", Eva answers, logs it and suggests a concrete step.
- **Suit mode:** the screen module lifts off with one hand, and the sensor base stays on the skin under the suit. The screen snaps back on top with magnets through the fabric, and the base keeps sending on its own battery.
- **Records everything:** each event is signed on board (an Ed25519 hash chain). The ship hub stores and forwards. Earth receives and acknowledges the signed records.

## How it links

- **The ship hub hosts the Super Intelligence and stores everything.** Wristbands, phones and the station are its clients.
- Wristbands talk **LoRa** and relay for each other as a **mesh**.
- If the hub is down, the alarm still goes **straight from the wristband to the station**, and the station's red button alerts every wristband.
- The Super Intelligence is built from **MedGemma**, Google's open medical model, plus our space-medicine layer and a strict topic guard: space and health only.

## Scalability

Eva is the **control interface and assistant**. More modules plug into it: new health devices and sensors, emergency and catastrophe response, information, entertainment and learning. It is the computer of space, from tourist flights to commercial stations and future deep-space bases.

## Repository map

| Path | What |
|---|---|
| `app/` | The Eva app: Rust + axum + askama + htmx, one binary. It covers the passenger PWA, wristband mode, the crew dashboard, the station, the ground instance and the Eva Link protocol. |
| `android/` | Android wrapper (APK) with online, offline and hub modes. |
| `supabase/` | Hosted backend schema, row-level security tests, and the `eva-chat` edge function. |
| `landing/` | Public landing page (EN/ES). |
| `hub/` | Research hub. |
| `docs/` | Architecture decisions (`docs/app/ADR-*`), protocol, running, testing, edge cases, station. |
| `sim/` | Simulation scenarios. |
| `proto/`, `wire/`, `design/` | Early prototype, wireframes and design work. |
| `video/` | Video story and animation sources. |

### Run it locally

```bash
cd app && cargo build --release && ./target/release/eva demo
```

Open the printed URL. Eva's chat needs `GROQ_API_KEY` in your environment or in a `.env` file; everything else works offline. See `docs/app/RUNNING.md` and `docs/app/DEMO.md`.

## NASA data and sources

- Inspiration4 biomedical data on **NASA OSDR**: OSD-575 (metabolic panel), OSD-569 (blood count).
- **NASA DONKI** and **NOAA GOES** proton flux for space weather.
- NASA Human Health and Performance data sharing: https://www.nasa.gov/directorates/esdmd/hhp/human-health-and-performance-data-sharing/
- Russell et al., *The value of a spaceflight clinical decision support system for Earth-independent medical operations*, npj Microgravity 9:46 (2023), https://doi.org/10.1038/s41526-023-00284-1
- Prior work that inspired us: NASA's ARCHeR study on Artemis II, with the ActLumus wrist actigraph by Condor Instruments.

## Disclosure

Eva is a concept for NASA Space Apps Challenge 2026 and **is not a medical device**. Concept images and video were made by the team with Super Intelligence; they show no real people and no NASA insignia. Details are in `landing/a/AI-NOTICE.md` and `app/README.md`. No NASA endorsement is implied.

## Team

Sanjo-X: Lean, Tomi, Hazzi, Naza and Joaquín, from Rosario.
