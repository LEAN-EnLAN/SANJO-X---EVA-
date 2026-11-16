# Space Apps 2026 submission: Eva by WearMeds (team Sanjo-X, Rosario)

Paste-ready text for each field of the project page on spaceappschallenge.org. It is consistent with the final report (`docs/report/`), the site, the app and the video.

---

## Project name
Eva by WearMeds

## Challenge
Create Health Monitoring Software for Astronauts on Space Missions

## High-level project summary
Eva by WearMeds is a Super Intelligence health copilot that helps astronauts and commercial passengers monitor their wellbeing and coordinate care when immediate support from Earth is not available. Team Sanjo-X combined a working software prototype with a proposed modular wristband and an onboard medical station.

The prototype brings together health-monitoring screens, conversational assistance, voice interaction, wellness logging, emergency simulations and digitally signed records. Its architecture separates immediate, rule-based alerts from the conversational Super Intelligence, so essential responses never depend on generating an answer. Records are stored on board and forwarded to Earth when the link allows.

Eva answers the challenge by turning health signals into clear guidance and coordinated action. Its intended impact is to reduce cognitive workload, support local response in the first minutes, and keep continuity of care during space missions.

## Link to project demo
https://eva-wearmeds.vercel.app/video/ (30 s video, EN/ES)

## Link to final project
https://github.com/LEAN-EnLAN/SANJO-X---EVA- (the code, the final report in `docs/report/`, and links to everything live)

---

## Project details

```
WHAT IT DOES
Eva by WearMeds is a Super Intelligence health copilot for everyone on board a spaceflight, crew and commercial passengers alike. It monitors each person's vital signs against their own baseline, talks with them by voice or text, and keeps a wellness journal: if someone says "I feel alone", Eva answers, logs it and suggests a concrete step. In an emergency it turns health signals into one clear action per screen. During a solar particle event it guides each person to the shelter. When someone stops responding, Eva alerts the crew and calls the nearest person to assist, guides them through CPR, and the onboard medical station lights the compartment with the AED. Every step is recorded in a signed log that is stored on board and forwarded to Earth, which receives and acknowledges it. Eva gives diagnostic support only: the AED decides the shock and the crew lead decides treatment.

HOW IT WORKS
Eva runs on a proposed two-module wristband. A sensor base on the skin reads pulse, SpO2, skin temperature and movement. A detachable screen module, held by magnets, provides voice, touch and a help button; in suit mode the base stays on the skin and keeps measuring. A proposed on-demand ECG uses two upper pads touched with the index and little finger of the free hand. The ship hub hosts the Super Intelligence and stores everything; wristbands, phones and the medical station connect to it over LoRa for alerts and summaries, Wi-Fi for voice and BLE for the sensor link. Two layers keep it safe: deterministic rules raise critical alerts instantly, and a conversational layer explains and guides behind a topic guard (space and health only, never treatment or shock decisions). Records use an Ed25519 signed hash chain.

WHAT WE BUILT
A working prototype as a single Rust binary: passenger app (PWA), wristband-screen mode, crew dashboard, medical-station screen and a ground instance, covered by 152 automated tests. A hosted demo with live chat, voice input, photo and video understanding and spoken answers; an Android APK with online, offline and connect-to-hub modes; a hosted backend with row-level security; plus a landing page, research hub, pitch, 30-second video and a 28-page final report in English and Spanish.

BENEFITS AND INTENDED IMPACT
In the first minutes of an emergency there may be no doctor on board and Earth may be far away. Eva reduces cognitive workload, makes sure the nearest person knows what to do, where and how, and keeps continuity of care with a complete, ordered record for Earth. Long term, Eva is the control interface and assistant that more modules plug into (other health devices, emergency and catastrophe response, information, entertainment and learning), from tourist flights to commercial stations and future deep-space bases.

TOOLS, LANGUAGES, HARDWARE AND SOFTWARE
Rust (axum, askama, htmx), TypeScript (Vercel functions), Java (Android), SQL with row-level security (Supabase), HTML/CSS/JavaScript. Demo models through Groq: gpt-oss (chat), Whisper large v3 turbo (speech-to-text), Orpheus (speech) and Qwen (vision). MedGemma is our candidate model for a mission version, to be specialised and validated for space medicine. Proposed hardware: nRF54L15, MAX30101, MAX30208, BMI270, MAX30003 (ECG), SX1262 LoRa at 915 MHz and an ESP32-S3 screen module. NASA data: OSDR (Inspiration4, OSD-575 and OSD-569), DONKI, and HRP resources.

HOW IT IS CREATIVE
Most wearables record and analyse later. Eva acts in the moment and coordinates people and equipment: it connects a person's own baseline, the nearest helper and the station's lit compartment into a single guided response. Two ideas come from life on Earth. First, a space weather forecast: just as everyday weather forecasts use upstream signals to warn people before a storm arrives, Eva turns complex heliophysics data (NASA DONKI solar flare and particle events) into a simple, actionable warning with a route to the shelter. Second, ship-wide emergency takeover: inspired by civil wireless emergency alerts, like AMBER Alerts, which push one unmistakable, high-priority message to every phone in an area at once, a serious alert takes over every wristband and screen on the ship's LoRa mesh at the same time, so nobody misses it. The wristband splits into two modules so it keeps working under a spacesuit, and the critical path never depends on generating an answer.

FACTORS WE CONSIDERED
Earth-independent medical operations and communication delay; care in microgravity; privacy and encryption of medical data on every link; energy budgets and battery life; usability for people with no medical training; and honesty about scope. Eva is a concept, not a medical device, and our report separates decisions, implementation, published data, calculations and proposals.
```

---

## NASA data
- **NASA Open Science Data Repository (OSDR):** Inspiration4 metabolic panel OSD-575 and complete blood count OSD-569. We used them to compare each person's values against their own baseline instead of a generic lab range.
  - https://osdr.nasa.gov/bio/repo/data/studies/OSD-575
  - https://osdr.nasa.gov/bio/repo/data/studies/OSD-569
- **NASA DONKI** (solar energetic particles and solar flares), for the solar-event flow, simulated in the demo: https://ccmc.gsfc.nasa.gov/tools/DONKI/ and https://api.nasa.gov/
- **NASA Human Research Program**: hazards, risks and ongoing research. These are the challenge's resources and shaped which risks Eva covers.
  - https://www.nasa.gov/hrp/hazards/
  - https://www.nasa.gov/hrp/risks/
  - https://www.nasa.gov/hrp/ongoing-hrp-research/
- **Human Research Roadmap and Evidence Reports:** https://humanresearchroadmap.nasa.gov/
- **Human System Risk Board:** https://www.nasa.gov/directorates/esdmd/hhp/human-system-risk-board/
- **Integrated Medical Model:** https://www.nasa.gov/nasa-integrated-medical-model-imm-overview/
- **NASA-STD-3001**, Volumes 1 and 2: https://www.nasa.gov/reference/nasa-std-3001v1/ and https://www.nasa.gov/reference/nasa-std-3001v2/
- **NASA Human Health and Performance data sharing:** https://www.nasa.gov/directorates/esdmd/hhp/human-health-and-performance-data-sharing/
- **NASA Life Sciences Portal:** https://www.nasa.gov/hrp/nlsp/

## Space agency partner and other data
- **CSA, Monitoring Radiation for Astronaut Health** (challenge resource): https://www.asc-csa.gc.ca/eng/news/articles/2026/2026-02-26-monitoring-radiation-astronaut-health.asp
- **CSA, Why Do We Conduct Science Experiments in Space?** (challenge resource): https://www.asc-csa.gc.ca/eng/iss/science/why-do-we-conduct-science-experiments-in-space.asp
- **NOAA SWPC GOES proton flux:** https://www.swpc.noaa.gov/products/goes-proton-flux
- **NASA CCMC HESPERIA REleASE v3:** https://ccmc.gsfc.nasa.gov/models/HESPERIA_REleASE~v3/
- **DLR, AstroRad (MARE):** https://www.dlr.de/en/latest/news/2026/astrorad-vest-can-significantly-reduce-radiation-exposure-for-humans-in-space

## Use of artificial intelligence
This form calls them artificial intelligence; in our product we call them Super Intelligence. Our answer, one item per checkbox:

- **Ideation: No.** The idea, the use cases, the wristband and station concept, and the scalability vision came from the team.
- **Information gathering: Yes, partly.** The team found the sources one link at a time: NASA pages, OSDR studies, papers and the challenge resources. Assistants (Claude, Gemini) then helped filter and summarise long papers and documents so we could tell what each one said.
- **Analysis: Yes, with team review.** Assistants drafted analyses from our own material, for example worked versions of our original acute cases and the project justification from the scientific papers. The team critiqued, corrected and decided every one.
- **Coding: Yes.** Coding assistants helped write and test the app, the Android wrapper, the backend and the website. The team reviewed and directed them, and the code has 152 automated tests.
- **Content creation: Yes.** Concept images, the video footage and the voices were generated with image, video and text-to-speech models, then edited by the team. They show no real people and no NASA insignia, and are declared in `landing/a/AI-NOTICE.md`.
- **Did we build or train a model? No.** The demo uses existing open-weight models through Groq: gpt-oss (chat), Whisper (speech-to-text), Orpheus (speech) and Qwen (vision). They sit behind our own topic guard, with deterministic rules for safety-critical alerts. For a mission version we *propose* specialising MedGemma for space medicine; that training has not been done.
- **Other project-related use:** EN/ES translation, drafting and layout of the report and the pitch deck, and consistency checks across the deliverables. The team shared context from its Google Drive and did peer cross-review. Every claim and number was checked by the team against its source.

---

## References

**Research**
- Russell BK et al., *The value of a spaceflight clinical decision support system for Earth-independent medical operations*, npj Microgravity 9:46 (2023). https://doi.org/10.1038/s41526-023-00284-1
- NASA Ames, *A Clinical Decision Support System for Earth-Independent Medical Operations*: https://www.nasa.gov/centers-and-facilities/ames/ames-science/ames-space-biosciences/a-clinical-decision-support-system-for-earth-independent-medical-operations/
- NASA, *The Human Body in Space*: https://www.nasa.gov/humans-in-space/the-human-body-in-space/
- NASA SANS briefing (NTRS 20230006218): https://ntrs.nasa.gov/citations/20230006218
- Khan et al. 2025, astronaut sinonasal events: https://pmc.ncbi.nlm.nih.gov/articles/PMC12322577/
- American Heart Association, Sudden Cardiac Arrest fact sheet: https://olis.oregonlegislature.gov/liz/2013R1/Downloads/CommitteeMeetingDocument/4759

**Prior work that inspired us**
- NASA ARCHeR study on Artemis II, with the ActLumus wrist actigraph (Condor Instruments):
  - https://www.nasa.gov/image-detail/amf-jsc2025e075542_alt/
  - https://pesquisaparainovacao.fapesp.br/4031
- NASA HRP Artemis II Data Methodology Challenge: https://hrpdatachallenge.org/
- Space Apps 2018, DeltaProtocol (Rosario), a LoRa sensor network and crew wristband concept:
  - http://2018.spaceappschallenge.org/challenges/can-you-build/make-sense-out-mars/teams/deltaprotocol/
  - https://github.com/juantinazzo/SpaceApps2018
- NASA/JPL Hermes, commanding and telemetry framework, studied for the link to Earth: https://github.com/nasa/hermes

**Other**
- MedGemma, Google Health AI Developer Foundations (candidate mission model): https://developers.google.com/health-ai-developer-foundations/medgemma
- Live deliverables:
  - landing https://eva-wearmeds.vercel.app
  - pitch https://eva-wearmeds.vercel.app/pitch/
  - research hub https://eva-wearmeds.vercel.app/hub/
  - app https://eva-wearmeds.vercel.app/app/
  - APK https://eva-wearmeds.vercel.app/download/

---

*Eva is a concept for NASA Space Apps Challenge 2026 and is not a medical device. No NASA endorsement is implied.*
