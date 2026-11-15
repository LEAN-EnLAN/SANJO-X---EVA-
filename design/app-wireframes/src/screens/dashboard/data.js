// Wireframe data for the crew and Earth dashboards. Day 96 of a 7-month trip, ship time 14:06.

export const crewLead = { name: "Mara Okoye", role: "Crew lead" }
export const surgeon = { name: "Sam Patel", role: "Flight surgeon, MD · Houston" }

// band: joined (both parts together, pins powering sensors) | suit (suit between the parts)
// status: care | helping | ok | battery | suit | ecg
// skin: skin temperature °C · hr/spo2: null when the band finds no pulse
// ecg: last on-demand 1-lead wrist ECG (30 s, one finger of each hand on the two metal pads); null if none today
export const roster = [
  { seat: "1A", name: "Mara Okoye", status: "ok", hr: 68, spo2: 98, band: "joined", battery: 82, skin: 33.8, ecg: { t: "08:04", result: "regular" } },
  { seat: "1B", name: "Tomás Rey", status: "aed", hr: 118, spo2: 97, band: "joined", battery: 64, skin: 34.1, ecg: { t: "07:58", result: "regular" } },
  { seat: "2A", name: "Yuki Sato", status: "suit", hr: 88, spo2: 98, band: "suit", battery: 46, skin: 33.2, ecg: { t: "13:10", result: "regular" } },
  { seat: "2B", name: "Omar Haddad", status: "suit", hr: 91, spo2: 97, band: "suit", battery: 51, skin: 33.4, ecg: null },
  { seat: "3A", name: "Lena Vogel", status: "ok", hr: 62, spo2: 99, band: "joined", battery: 77, skin: 33.9, ecg: { t: "08:10", result: "regular" } },
  { seat: "3B", name: "Leo Brandt", status: "helping", hr: 112, spo2: 98, band: "joined", battery: 70, skin: 34.6, ecg: { t: "08:02", result: "regular" } },
  { seat: "4A", name: "Priya Nair", status: "ecg", hr: 70, spo2: 98, band: "joined", battery: 58, skin: 34.0, ecg: { t: "12:40", result: "irregular" } },
  { seat: "4B", name: "Ana Ruiz", status: "care", hr: null, spo2: null, band: "joined", battery: 66, skin: 32.9, ecg: { t: "08:06", result: "regular" } },
  { seat: "5A", name: "Diego Luna", status: "battery", hr: 76, spo2: 97, band: "joined", battery: 14, skin: 34.2, ecg: { t: "08:12", result: "regular" } },
  { seat: "5B", name: "Chloé Martin", status: "ok", hr: 66, spo2: 98, band: "joined", battery: 88, skin: 33.7, ecg: { t: "08:01", result: "regular" } },
  { seat: "6A", name: "Kwame Mensah", status: "ok", hr: 72, spo2: 97, band: "joined", battery: 61, skin: 34.0, ecg: { t: "08:15", result: "regular" } },
  { seat: "6B", name: "Sofia Rossi", status: "suit", hr: 84, spo2: 98, band: "suit", battery: 49, skin: 33.3, ecg: null },
  { seat: "7A", name: "Ivan Petrov", status: "ok", hr: 64, spo2: 98, band: "joined", battery: 73, skin: 33.6, ecg: { t: "08:09", result: "regular" } },
  { seat: "7B", name: "Hana Kim", status: "ok", hr: 69, spo2: 99, band: "joined", battery: 90, skin: 33.9, ecg: { t: "08:03", result: "regular" } },
  { seat: "8A", name: "Noah Clark", status: "ok", hr: 71, spo2: 97, band: "joined", battery: 55, skin: 34.1, ecg: { t: "08:11", result: "regular" } },
]

// Alerts this week (Mon..Sun). Today is Saturday.
export const weekAlerts = [1, 2, 1, 2, 3, 3, 0]
export const weekLabels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Today", "Sun"]

// Records waiting / travelling / confirmed by Earth
export const syncQueue = [
  { what: "Ana Ruiz · wristband alarm: no pulse, no movement", ship: "14:02", state: "sent", note: "Earth gets it ~14:15" },
  { what: "Ana Ruiz · AED log, shock advised by the AED", ship: "14:04", state: "saved", note: "Next link slot · 14:07" },
  { what: "Leo Brandt, Tomás Rey · helper and CPR steps", ship: "14:03", state: "saved", note: "Next link slot · 14:07" },
  { what: "Priya Nair · wrist ECG, irregular · review asked", ship: "12:40", state: "ack", note: "Confirmed 12:53 · review asked" },
  { what: "Daily vitals · 15 passengers", ship: "08:00", state: "ack", note: "Confirmed 08:26" },
  { what: "Cabin dose log · morning", ship: "09:30", state: "ack", note: "Confirmed 09:56" },
]

// Earth view: case timeline for Ana (Earth is 12 min 40 s behind)
export const anaCase = [
  { ship: "14:02:08", earth: "14:14:48", title: "No pulse at Ana's wrist", detail: "Wristband: no pulse, no movement, no answer · Eva sounded the alarm", tone: "critical" },
  { ship: "14:02:20", earth: "14:15:00", title: "Eva sent two helpers", detail: "Leo Brandt (3B, 6 m) to Ana · Tomás Rey (1B) to get the AED from the emergency station, corridor B", tone: "info" },
  { ship: "14:03:05", earth: "14:15:45", title: "Leo arrived, CPR started", detail: "Eva guiding CPR on his wristband · 110 a minute", tone: "info" },
  { ship: "14:03:40", earth: "14:16:20", title: "Crew lead Mara notified", detail: "Hub opened the case and started recording", tone: "neutral" },
  { ship: "14:04:05", earth: "14:16:45", title: "AED attached, shock advised by the AED", detail: "The AED read the heart through its own pads · Eva only guides", tone: "critical" },
  { ship: "14:04:20", earth: "~14:17:00", title: "Not received yet", detail: "What happened after the AED's advice is still on its way", pending: true },
]

export const trends = [
  { name: "Ana Ruiz", seat: "4B", metric: "Heart", value: "No pulse", unit: null, note: "fast and uneven, then no pulse at the wrist · 14:02", data: [66, 68, 65, 67, 70, 66, 112, 178, 0], band: [58, 78], tone: "critical" },
  { name: "Leo Brandt", seat: "3B", metric: "Heart", value: "112", unit: "bpm", note: "high · he is helping", data: [64, 66, 65, 63, 67, 70, 98, 108, 112], band: [56, 76], tone: "info" },
  { name: "Tomás Rey", seat: "1B", metric: "Heart", value: "118", unit: "bpm", note: "high · he brought the AED", data: [70, 72, 74, 71, 73, 72, 96, 120, 118], band: [60, 82], tone: "info" },
  { name: "Yuki Sato", seat: "2A", metric: "Heart", value: "88", unit: "bpm", note: "suit drill · expected", data: [70, 72, 71, 74, 80, 86, 89, 87, 88], band: [60, 82], tone: "info" },
  { name: "Diego Luna", seat: "5A", metric: "Heart", value: "76", unit: "bpm", note: "normal for him", data: [72, 74, 75, 73, 77, 74, 75, 78, 76], band: [62, 84], tone: "info" },
  { name: "Priya Nair", seat: "4A", metric: "Heart", value: "70", unit: "bpm", note: "rate normal · wrist ECG uneven, see above", data: [68, 71, 69, 72, 70, 74, 71, 69, 70], band: [60, 80], tone: "info" },
]
