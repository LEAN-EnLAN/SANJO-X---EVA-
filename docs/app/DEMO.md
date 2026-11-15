# Eva · live demo script (3–4 minutes)

One laptop is the **ship hub**, three phones are passengers, a tablet (or a second laptop) is the ship's **medical station**, a second laptop (or a second port) is the **ground**. Everything runs from the one `eva` binary, offline-capable. The story: a normal afternoon in low Earth orbit, then a cardiac arrest: three phones change screen at the same moment and the station lights the AED compartment.

Words to keep (team rules): Eva gives **diagnostic support**, guides, records and warns. **The ship's AED decides the shock**, never Eva. Near space: the ground is **seconds** away, links have **contact gaps**.

## Setup (10 minutes before)

### Machines

| who | device | opens |
|---|---|---|
| **Ship hub** | laptop A, on the projector | `https://<A-ip>:8080/dash` (crew dashboard, Mara, crew lead) |
| **Ana** · 4B · the patient | phone 1 | `https://<A-ip>:8080/app?seat=4B` |
| **Leo** · 3B · nearest trained passenger | phone 2 | `https://<A-ip>:8080/app?seat=3B` |
| **Tomás** · 1B · nearest to the AED station | phone 3 | `https://<A-ip>:8080/app?seat=1B` |
| **Medical station** · corridor B | tablet in landscape (or laptop B in full screen, F11) | `https://<A-ip>:8080/station` |
| **Ground** · flight surgeon | laptop B (or a second browser tab) | `http://<B-ip>:8081/dash` |

### Start

Two machines (the real split):

```bash
# laptop B (ground)
cd app && ./target/release/eva serve --role ground --port 8081 --data data-ground
# laptop A (ship hub), phones need HTTPS to install the PWA
cd app && set -a; . /path/outside/the/repo/.eva-demo.env; set +a     # optional: GROQ_API_KEY
./target/release/eva serve --role ship --port 8080 --peer http://<B-ip>:8081 --tls --data data-ship --station
```

One machine (ground on a second port, same process):

```bash
cd app && ./target/release/eva demo --tls          # ship https://<ip>:8080 (+ /station) · ground http://<ip>:8081
```

The terminal prints the LAN address. On each phone: open the address, trust the certificate once (iOS: install the profile, then Settings → General → About → Certificate Trust), Share → **Add to Home Screen**. Pick the seat in the app header. Detail: RUNNING.md and PWA-STEPS.md.

### Checklist

- [ ] All three phones show "Good afternoon, <name>" and a green **Mesh** dot in the header.
- [ ] Dashboard: roster live (pulse lines moving), mesh card shows 5+ nodes (the station is one).
- [ ] Station screen: **Hub online**, 12 compartments, all full; the dashboard's station tile says **Online** with the battery. Tablet: auto-lock off, brightness up, landscape.
- [ ] Ground dashboard: **Ship link · On**, "Records received" counting.
- [ ] Eva badge reads **Groq online** or **offline rules**; both demo fine. With a key, "browser voice (TTS terms pending)" means Groq's voice is not enabled for the org yet: phones speak with their own voice.
- [ ] Phones: volume up, auto-lock off, brightness up. Laptop: notifications off.
- [ ] Open the dev panel once to check it (↑ ↑ ↓ ↓ ← → ← → B A on the laptop; on a phone tap the clock 5 times), then close it (Esc).

## The run

| time | do | say |
|---|---|---|
| **0:00** | Projector on the crew dashboard. | "This is a commercial spacecraft in low Earth orbit, day 96. Fifteen passengers, each with a WearMeds band on the wrist. Meet **Eva**: the health copilot that reads every band live, keeps the logs and the mood journal, gives diagnostic support, and sends signed records to the ground." |
| **0:20** | Hold up Ana's phone (4B). Tap **Eva**, ask *"How am I doing?"* (type or hold the mic). | "Ana asks Eva. The answer always has four parts: **what to do now, why, how Eva decided, what comes next**. It reads her band against *her own* usual range, not a textbook." |
| **0:45** | On Ana's phone: **+1 glass of water**; then **Journal**, slide to *Pleasant*, tick a feeling, save. | "Daily life: water, meals, sleep, mood. The journal is private; only what she chooses to share reaches the crew lead." |
| **1:05** | Ana: **30 s ECG** chip. Show the countdown and the trace. | "On demand, a 30-second wrist ECG from two metal pads on the band's screen. If something looks uneven, it goes to the ground for review." |
| **1:20** | Laptop: **↑ ↑ ↓ ↓ ← → ← → B A** → dev panel → seat 4B → **Cardiac arrest**. Close the panel. | "Now the hard part. Ana's heart stops." *(pause)* "Watch her band: the pulse races, then disappears." |
| **1:40** | Wait. ~20 s later the band reports no answer. (Short on time: dev panel **+1 min**.) | "Eva doesn't guess. Her reflex rule needs **three signs**: no pulse, no movement, and no answer to the band's 'are you OK?'. A loose band alone never triggers it. No AI model in this loop: it is instant and works with no network." |
| **2:00** | **WOW.** Hold up Leo's and Tomás's phones next to Ana's, with the station tablet behind them. All three phones turn red at the same moment and compartment **01** on the station glows teal: "Take the AED · Ana, seat 4B". The dashboard jumps Ana to *Needs help now*. | "Three phones and the medical station, same second. **Leo**, three metres away and trained, is asked to start CPR. **Tomás**, nearest to the station in corridor B, is asked to get the AED, and the station lights the exact door he needs. Mara sees the whole case on the dashboard." |
| **2:20** | Leo taps **I'm going** → CPR screen (110 a minute). Tomás taps compartment **01** on the station (door opened, AED taken), then **I have it** on his phone. Point at the case card: "AED taken at the station · 00:47". | "Leo gets guided CPR on his own band. The station's door and item sensors report the AED leaving: a signed record, on the timeline, on its way to the ground. **The AED analyses the rhythm and decides the shock, not Eva.** Eva guides, records and warns." |
| **2:40** | Point at the dashboard case card: *now / why / how Eva decided / next*, helpers, timeline. Dev panel **+1 min** twice: "CPR started", "AED attached, shock advised by the AED". | "Every step is a record, hashed into a chain and **signed with the ship's key**. Nobody can edit it afterwards without it showing." |
| **3:00** | Switch to the ground dashboard (laptop B). The case is there with **Ship time** and **Ground received** side by side; sync card goes *Saved → Sent → Acknowledged*. | "The flight surgeon on the ground sees the same case **seconds** later, and verifies every record's hash and signature on arrival. If the link drops, records wait on board and nothing is lost." |
| **3:20** | Optional, if time: dev panel → uplink **Link outage**, log something, show *Saved* piling up; back to **Near-space uplink**, watch them turn *Acknowledged*. | "Near space has contact gaps. Eva Link stores and forwards, alarms first: an alarm is ten bytes." |
| **3:30** | Optional, if time: dev panel → **Hub down**. The station turns **Standalone** within 5 s. | "And if the hub itself fails? The station listens to the wristbands' radio directly: a no-pulse alarm still lights the AED door, and its red button still reaches every wristband. When the hub returns, it uploads what happened and the chain still verifies." |
| **3:35** | Dev panel → **Close case** (01 goes dark on the station). Dashboard header → **Report** (print to PDF), **CSV**, **JSON**. | "Case closed by the crew lead. One click: the full record, for the doctors on the ground, verifiable byte by byte." |
| **3:50** | Close. | "Eva: a health copilot on the wrist of every passenger, working with or without the ground." |

## If the Wi-Fi dies

The demo needs **no internet**, only a network between the laptop and the phones.

| what broke | do |
|---|---|
| Venue Wi-Fi gone, phones lost the hub | Turn on the **laptop's hotspot** (macOS: Internet Sharing; Windows: Mobile hotspot; Linux: `nmcli dev wifi hotspot`), join the phones to it, reload. The hub IP changes: read the new address from `eva`'s start line (or `ip addr`). With `--tls`, the certificate covers `localhost` and the IP at creation time: add `--tls-host <new-ip>` and delete `data/certs/` to recreate it, or use mkcert. |
| No internet, Groq unreachable | Nothing to do: Eva falls back to her **on-board rules** by herself (badge: "offline rules"). To be sure before going on stage: restart with `--offline`. Voice is the phone's own. |
| One phone frozen or disconnected | The installed PWA still opens from its cache; actions made offline show *"Saved on this device · will send when the hub is back"* and are sent when it reconnects. Reload the page when the network is back. |
| Station tablet lost the hub | The station page reloads by itself when the network is back (SSE reconnects); the simulated station computer runs inside `eva`, so nothing is lost. Or put `/station` in a laptop window. |
| Everything broken | Run the whole story on the laptop alone: `eva demo`, open the five URLs in browser windows side by side (`/app?seat=4B`, `?seat=3B`, `?seat=1B`, `/dash`, `/station`) plus `:8081/dash`. Or show the screenshots from `scripts/e2e/run.sh` (`scripts/e2e/out/…`). |
| Turn it into a feature | Say: "This is what a contact gap looks like", switch the uplink to **Link outage**, and show the records waiting on board. |

## Reset

- **Between runs:** dev panel → **Reset demo** (ship): world, clock (day 96, 13:58) and chain back to the start; phones go home; the station restocks every compartment and turns its lights off; "Hub down" goes off. The ground starts a new mirror epoch when the ship's new records arrive (or reset it from the ground's own dev panel).
- **From scratch:** stop the processes, delete the data dirs (`data`, or `data-ship` / `data-ground`), start again. Keep `data/certs/` if the phones already trust the certificate, and keep `data/keys/` to keep the same ship key (otherwise delete the ground's `keys/peer-ship.pub` too).
- **Check the chain afterwards:** `eva verify --data data` → every file `OK`.

## Troubleshooting

| symptom | cause | fix |
|---|---|---|
| Phone: "connection not private" every time, no Add to Home Screen | certificate not trusted | Trust it once (RUNNING.md, PWA-STEPS.md) or use mkcert. Service workers refuse untrusted certificates. |
| Phone can't reach the laptop at all | different network, client isolation on venue Wi-Fi, firewall | Same Wi-Fi or the laptop hotspot; allow port 8080/8081 in the firewall; check `--host 0.0.0.0` (default). |
| Helpers' phones don't turn red | that phone is on another seat, or its live connection dropped | Check the seat in the header and the green Mesh dot; reload the page. |
| Case does not open after "Cardiac arrest" | the 20 s "no answer" window has not passed | Wait, or dev panel **+1 min**. The rule needs all three signs. |
| Station 01 does not light | "Hub down" is on (the hub's rules are off), or the station page lost its live connection | Dev panel: the Hub down button reads "Hub down" (not "tap to bring it back"); reload `/station`. |
| Station says "No station on this node" | the ship was started without the station | `eva demo` always runs one; with `eva serve --role ship` add `--station`. |
| Ground shows nothing | uplink not connected, or in a contact gap (near-space profile: 20 s every 2 min) | Ground KPI **Ship link**; on the ship start with `--peer http://<B-ip>:8081`; switch the profile to **Ship LAN** for a fast demo. |
| Ground: "different public key than the one pinned" | the ship's data dir (and key) was recreated | Delete `keys/peer-ship.pub` in the ground's data dir and restart the ground. |
| Eva badge "browser voice (TTS terms pending)" | Groq's Orpheus terms not accepted for the org | Fine for the demo (phone voice). To fix: an org admin accepts the terms in the Groq console (RUNNING.md). |
| Eva answers "offline rules" although a key is set | no internet, or `--offline` / `EVA_OFFLINE=1` | Expected fallback; answers stay in the Now / Why / How / Next format. |
| Port already in use | an old `eva` still running | `pkill eva`, or `--port` / `--ground-port`. |
| `eva verify` says TAMPERED | a chain file was edited, or signed with another key | That is the feature. For the demo, reset with fresh data dirs. |


## Wristband mode (show the band's own UI)

The same app is the wristband's screen. Three ways to show it:

1. **Rotate a phone** (auto-rotate on): the app becomes the band UI by itself, and back when you turn it upright. In the APK, *Open as → Wristband (landscape)* locks it.
2. **Laptop**: Chrome devtools → device toolbar → Responsive → **640 × 360** (also 480 × 270 or 800 × 450).
3. **Any size**: add `?mode=wristband` (`?mode=auto` to undo).

What to show: the watch face (Eva's light, the time, pulse / SpO₂ / skin), swipe to the three vitals panels (tap one to open it), Eva's quick-ask with three chips, the 30 s ECG, and **Help: hold 2 seconds** (a tap never calls the crew). Then start the cardiac arrest from the dev panel with a band-sized window on 3B: "Ana needs help" with the arrow, **I'm going** → the CPR metronome that vibrates on every compression. Closing the case brings the band back to its face.

If Eva says **On-board rules**: the hub has no key. Put `eva.env` (one line, `GROQ_API_KEY=…`) next to the `eva` program and start it again (RUNNING.md).
