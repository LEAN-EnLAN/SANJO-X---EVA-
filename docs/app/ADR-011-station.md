# ADR-011 · Emergency station and how everything links

- **Status:** proposed · 15/11/2026 · Claude (for Sanjo-X), awaiting team confirmation
- **Extends:** ADR-003 (Eva Link), ADR-008 (mesh through the hub), ADR-010 (signed chain)
- **Source:** `docs/station.md` on branch `claude/entorno-compartido`; Drive "06 · Concepto actual: Eva" (sections "Estación de emergencia" and "Cómo se conecta todo") and the Registro (Dom 15/11 09:22)

## Context

On Sun 15/11 the team sent the v2 references: a rugged two-module wristband and a wall-mounted emergency station for the ship (concept images generated with Super Intelligence). Lean asked: is the backend a server on a local computer (for example the laptop at the demo), or does each client host something? Is the station a host or a client?

Product rule: Eva gives **diagnostic support** and guidance. The station's AED analyses the heart rhythm and decides the shock. Eva never does.

### The station

A wall-mounted medical cabinet in each habitable module, ISS-like: beige wall panels, handrails and blue straps around it.

**Numbered compartments**, configurable for each ship (the demo cabinet's layout is data: `station::COMPARTMENTS`; no screen states a count). Each is numbered and labelled, sits behind a clear window and has its own light strip. The team images use Spanish labels; product text is English.

| # | Label | Contents |
|---|---|---|
| 01 | AED | The AED and spare pads |
| 02 | Breathing | Oxygen bottle, mask, bag-valve mask |
| 03 | Diagnostics | Monitor, thermometer, glucometer |
| 04 | Ultrasound | |
| 05 | Wounds | |
| 06 | Restraint | |
| 07 | Oral meds | |
| 08 | Injectables | Part of the station's crew medical kit. Not a wearable injector. |
| 09 | IV access | |
| 10 | Eyes and dental | |
| 11 | Airway | |
| 12 | Hygiene | |

Variant: a centre cabinet with side doors that open to reveal the second half of the compartments.

**On top of the cabinet**
- **Eva screen**: says what to take and what to do, for example "EVA · Take the AED".
- **Red emergency button** ("PRESS IN EMERGENCY"): opens a case on the hub and also broadcasts an alarm over LoRa to every wristband, so it works when the hub is down.
- **Printed backup guide** ("AED retrieval guide").

**Lights**: when Eva asks for an item, its compartment lights up teal (#1FE5C4) and the screen says what to take.

**Sensors**: one door sensor per compartment, plus an item sensor (weight or reed switch) where it matters (01 first). They report "door 01 open 00:41" and "AED taken 00:47". Those reports go into the signed log and the case timeline.

**Inside**: a small computer, a LoRa radio, a Wi-Fi or Ethernet link to the hub, and a backup battery.

## Decision (proposed)

### How everything links

```
  EARTH                               ground instance (eva serve --role ground)
    ^
    |  near-space link: the hub stores and forwards signed records; Earth acknowledges
    v
  +-----------------------------------------------------------+
  |  SHIP HUB  (the only host)                                |
  |  Eva (3 layers) . signed hash chain . dashboard           |
  |  Earth link . serves the app (PWA)                        |
  |  demo: the laptop on the table (eva demo)                 |
  +-----------------------------------------------------------+
        ^ ship Wi-Fi            ^ LoRa                 ^ Wi-Fi/Ethernet + LoRa
        | (Eva Link /link)      |                      | case events down,
        |                       |                      | door/item reports up
  +-------------+     +-------------------+     +--------------------------+
  | phone or    |     | WRISTBANDS        |     | EMERGENCY STATION        |
  | tablet app  |     | base + screen     |     | Eva screen, 12 lit       |
  | (client)    |     | module, LoRa each |     | compartments, sensors,   |
  | big screen  |     | (clients)         |     | red button, LoRa,        |
  +-------------+     | band <-> band     |     | backup battery (client)  |
                      | mesh relay        |     +--------------------------+
                      +-------------------+                  ^
                                |                            |
                                +------- direct LoRa --------+
                                  fallback when the hub is down:
                                  no-pulse alarm band -> station
                                  red-button alarm station -> bands
```

- **One host: the ship hub.** A local computer (the server) runs Eva, the signed log, the dashboard and the near-space link to Earth, and serves the app (PWA). It is the single source of truth for the log and for Eva's state.
- **Wristbands are clients.** Each part (sensor base and screen module) talks LoRa to the hub. Wristbands relay for each other (mesh) when one is out of range.
- **Phones and tablets are clients.** The app is an optional big screen for the same person, connected to the hub over the ship's Wi-Fi (Eva Link over `/link`, ADR-003 and ADR-008 in the app).
- **The station is a client, not a host.** It has its own small computer, but it hosts nothing. It subscribes to case events from the hub and reports back.
- **Earth** is a separate ground instance that receives and acknowledges the signed records over the near-space link (the ship hub is the one that stores and forwards; it hosts the Super Intelligence and stores everything). Already built: `eva serve --role ground`.
- **Demo**: one laptop is the hub (`eva demo`). Phones play the wristbands and screens; a tablet or a second browser window plays the station.

#### Why this shape
- One source of truth for the log and Eva's state.
- Clients stay cheap and simple.
- It works offline on the ship.
- The station stays on the alarm path even if the hub fails.
- One laptop is enough for the demo.

### Message flow of a cardiac case

Names marked *(exists)* are already in the app (branch `fase2/12-app`: `docs/app/PROTOCOL.md` and `docs/app/ARCHITECTURE.md`). Names marked *(proposed)* are new and are not built yet.

| t | From → to | Event | Effect |
|---|---|---|---|
| 00:00 | wristband → hub | `Vitals` *(exists)* with `hr = 0`, low motion, then the `no answer` flag after 20 s | Reflex layer sees no pulse + no movement + no answer |
| 00:20 | wristband → hub | `Alarm` code 1, no pulse *(exists)*, alarm priority class | |
| 00:20 | hub | `open_emergency` *(exists)*; `alarm` and `case` records *(exist)* | Nearest passenger for CPR, nearest to the station for the AED |
| 00:21 | hub → helpers' apps and wristbands | `go-emergency` *(exists, SSE)* | Helper screens switch to the case |
| 00:21 | hub → station | `station.request {case, compartment: 1, text: "Take the AED"}` *(proposed)* | Compartment 01 lights teal, screen shows "EVA · Take the AED", chime |
| 00:41 | station → hub | `station.door {compartment: 1, state: open}` *(proposed, record kind)* | Signed log + case timeline: "door 01 open 00:41" |
| 00:47 | station → hub | `station.item {compartment: 1, item: aed, state: taken}` *(proposed, record kind)* | Timeline: "AED taken 00:47"; the helper's screen moves to "Bring the AED" |
| ~01:30 | helper → hub | case step "AED attached" *(exists, case timeline)* | Eva guides pad placement |
| — | AED | the AED analyses the rhythm and decides | Timeline logs "shock advised by the AED". Eva never decides it |
| any | hub → ground | `Record` frames *(exist)*, signed, then `Ack` | Earth sees the case seconds later (near-space profile) |
| end | hub → station | `station.clear {case}` *(proposed)* | Lights off; the station returns to idle |

Proposed additions in the app: a station role in `Hello` (next free role id, 6), the three `station.*` messages carried as Eva Link `Text`/`Guidance` frames or as a new message type, and `station.door`, `station.item` and `station.button` as chain record kinds.

### Fallback when the hub is down

The critical path never depends on one box.

1. The wristband broadcasts its no-pulse alarm over LoRa (the same `Alarm` code 1).
2. The station listens directly, lights compartment 01 and sounds an alert. Nearby wristbands vibrate and show the alarm.
3. Pressing the station's red button broadcasts an alarm to every wristband in range.
4. The station and the wristbands keep their own records while the hub is away.
5. When the hub returns, they upload what happened. The hash chain keeps it in order, and the hub links the records to the case.

Without the hub there is no Eva reasoning (layer 2) and no Earth link. The alarm, the lit AED compartment and the AED itself still work.

## Consequences

- One source of truth: the hub keeps the signed chain and Eva's state; station reports become ordinary chain records and case-timeline steps, and reach the ground like any other record.
- The hub stays a single point of failure for reasoning, the dashboard and the Earth link (as ADR-008 already accepts), but **not** for the alarm path: wristband and station talk LoRa directly when it is down.
- New work in the app: a station role in `Hello`, the `station.request` / `station.clear` messages, the `station.door` / `station.item` / `station.button` record kinds, a station page (a tablet or second browser window in the demo) and an upload path for records written while the hub was away.
- Records written during a hub outage need ordering and trust when they are merged; per-device keys (ADR-010 future work) become more relevant.
- The demo still needs only one laptop: phones play wristbands and screens, a tablet or a second browser window plays the station.

### Open questions

- **Confirm the link architecture** with the team (hub as the only host; wristbands, phones and station as clients; LoRa mesh and fallback).
- How many stations, and where (proposal: one per habitable module, under a minute from any seat).
- How the station reaches the hub: Wi-Fi, Ethernet or LoRa only.
- LoRa range and power through module walls; whether wristband-to-wristband relay is needed or LoRa repeaters on the hub side are enough.
- Item sensors: weight or reed switch, and which compartments need them beyond 01.
- Do records written during a hub outage need their own signing key on the station and the bands (per-device keys, ADR-010 future work)?
- Demo: show the station with real LEDs or only simulated in a browser window.
- Protocol: carry `station.*` as `Text`/`Guidance` frames or add a message type to Eva Link.
