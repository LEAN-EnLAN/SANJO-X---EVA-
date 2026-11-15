// Passenger deck of the transit ship, top-down. One source of truth for two consumers:
//   1. tools/deck-base.blender.py renders the floor plan (walls, furniture) to public/img/deck-base.png
//   2. DeckMap (src/ds/deck-map.jsx) draws the live overlay on top, in the same coordinates.
// Regenerate the image after changing geometry: see the header of tools/deck-base.blender.py.
// Units: viewBox pixels, 1400 x 800, 50 px = 1 m (deck is 26 m x 14 m inside the hull).
// x grows forward (towards the fwd hatch), y grows to starboard.

export const DECK = { width: 1400, height: 800, pxPerMetre: 50, image: "img/deck-base.png" }

/* ---------- structure (Blender reads all of this) ---------- */

// pressure hull: outer and inner rounded rectangles; the hatches are gaps at both ends of the corridor
export const hull = {
  outer: { x: 28, y: 38, w: 1344, h: 724, r: 84 },
  inner: { x: 50, y: 60, w: 1300, h: 680, r: 62 },
  hatches: [
    { x: 28, y: 372, w: 22, h: 56, label: "Aft hatch" },
    { x: 1350, y: 372, w: 22, h: 56, label: "Fwd hatch" },
  ],
}

// floor zones (also the room label anchors). kind tints the floor in the render.
export const rooms = [
  { id: "cabin", label: "Cabin", sub: "15 seats", x: 50, y: 60, w: 740, h: 290, kind: "cabin", lx: 746, ly: 132, anchor: "middle" },
  { id: "galley", label: "Galley", x: 800, y: 60, w: 270, h: 290, kind: "galley", lx: 935, ly: 330, anchor: "middle" },
  { id: "hygiene", label: "Hygiene", x: 1080, y: 60, w: 270, h: 290, kind: "wet", lx: 1215, ly: 236, anchor: "middle" },
  { id: "corridor", label: "", x: 50, y: 360, w: 1300, h: 80, kind: "corridor" },
  { id: "exercise", label: "Exercise", x: 50, y: 450, w: 390, h: 290, kind: "exercise", lx: 245, ly: 606, anchor: "middle" },
  { id: "shelter", label: "Storm shelter", sub: "water walls · room for 15", x: 470, y: 470, w: 220, h: 240, kind: "shelter", lx: 580, ly: 548, anchor: "middle" },
  { id: "hub", label: "Ship hub", sub: "computer", x: 720, y: 450, w: 180, h: 290, kind: "hub", lx: 800, ly: 482, anchor: "middle" },
  { id: "medbay", label: "Med bay", x: 910, y: 450, w: 440, h: 290, kind: "med", lx: 1130, ly: 520, anchor: "middle" },
]

// interior walls (rectangles, 0.2 m thick). Door gaps below are cut out of them.
export const walls = [
  // corridor, north side (cabin, galley, hygiene)
  { x: 50, y: 350, w: 1300, h: 10 },
  // corridor, south side (exercise; the shelter's thick wall takes over from x 440 to 720)
  { x: 50, y: 440, w: 390, h: 10 },
  { x: 720, y: 440, w: 630, h: 10 },
  // north partitions
  { x: 790, y: 60, w: 10, h: 290 },
  { x: 1070, y: 60, w: 10, h: 290 },
  // south partitions
  { x: 900, y: 450, w: 10, h: 290 },
  // hygiene stalls
  { x: 1080, y: 180, w: 250, h: 6 },
  { x: 1162, y: 60, w: 6, h: 120 },
  { x: 1244, y: 60, w: 6, h: 120 },
]

// the storm shelter: 0.6 m of stored water on every side, built from tank cells
export const shelter = {
  outer: { x: 440, y: 440, w: 280, h: 300 },
  inner: { x: 470, y: 470, w: 220, h: 240 },
  thickness: 30,
  door: { x: 550, y: 440, w: 60, h: 30 },
}

// door openings (cut through any wall they overlap)
export const doors = [
  { x: 100, y: 350, w: 60, h: 10 }, // cabin aft
  { x: 520, y: 350, w: 60, h: 10 }, // cabin fwd
  { x: 880, y: 350, w: 60, h: 10 }, // galley
  { x: 1110, y: 350, w: 60, h: 10 }, // hygiene
  { x: 280, y: 440, w: 60, h: 10 }, // exercise
  { x: 780, y: 440, w: 60, h: 10 }, // hub
  { x: 1100, y: 440, w: 60, h: 10 }, // med bay
  { x: 1092, y: 180, w: 56, h: 6 }, // stall 1
  { x: 1176, y: 180, w: 56, h: 6 }, // stall 2
  { x: 1258, y: 180, w: 56, h: 6 }, // shower
  { x: 600, y: 350, w: 50, h: 10 }, // station B niche
  { x: 1000, y: 350, w: 50, h: 10 }, // station D niche
]

// passenger seats: rows 1-8 along the cabin, A by the hull, B by the aisle (8B is a stowage locker)
const rowX = n => 110 + (n - 1) * 78
export const seats = ["A", "B"].flatMap(letter =>
  Array.from({ length: 8 }, (_, i) => ({ id: `${i + 1}${letter}`, x: rowX(i + 1), y: letter === "A" ? 125 : 190 }))
).filter(s => s.id !== "8B")
export const seatRows = Array.from({ length: 8 }, (_, i) => ({ n: i + 1, x: rowX(i + 1) }))

// emergency stations with an AED, recessed into the corridor's north wall. The corridor segment they sit in
// takes their letter, so their callout is also the corridor's label.
export const stations = [
  { id: "B", label: "Corridor B", note: "AED station · in use", state: "out", side: "se", x: 625, y: 344, niche: { x: 600, y: 326, w: 50, h: 34 } },
  { id: "D", label: "Corridor D", note: "AED station · ready", state: "ready", side: "se", x: 1025, y: 344, niche: { x: 1000, y: 326, w: 50, h: 34 } },
]

// the hub (computer racks); also the mesh gateway
export const hub = { id: "hub", label: "Ship hub", x: 783, y: 640 }

// furniture blocks: x, y, w, h in px; z = height in metres; r = corner rounding (0..1); tone: render grey
export const furniture = [
  // cabin: lockers along the corridor wall, crew stowage at the fwd end
  { x: 56, y: 304, w: 40, h: 44, z: 1.1, tone: "locker" },
  { x: 166, y: 304, w: 348, h: 44, z: 1.1, tone: "locker" },
  { x: 586, y: 304, w: 10, h: 44, z: 1.1, tone: "locker" },
  { x: 656, y: 304, w: 128, h: 44, z: 1.1, tone: "locker" },
  { x: 628, y: 160, w: 56, h: 58, z: 0.9, tone: "locker" }, // 8B: stowage locker
  { x: 712, y: 192, w: 70, h: 100, z: 1.3, tone: "locker" },
  // galley
  { x: 806, y: 66, w: 258, h: 46, z: 0.92, tone: "counter" },
  { x: 830, y: 74, w: 52, h: 30, z: 0.94, tone: "inset" },
  { x: 990, y: 70, w: 64, h: 38, z: 1.0, tone: "appliance" },
  { x: 806, y: 130, w: 40, h: 150, z: 1.6, tone: "locker" },
  { x: 880, y: 196, w: 150, h: 64, z: 0.76, r: 0.5, tone: "table" },
  // hygiene: two toilets, one shower, a basin counter
  { x: 1108, y: 84, w: 26, h: 36, z: 0.42, r: 1, tone: "fixture" },
  { x: 1190, y: 84, w: 26, h: 36, z: 0.42, r: 1, tone: "fixture" },
  { x: 1262, y: 92, w: 62, h: 62, z: 0.06, tone: "tray" },
  { x: 1210, y: 296, w: 120, h: 48, z: 0.88, tone: "counter" },
  { x: 1228, y: 306, w: 30, h: 26, z: 0.9, r: 1, tone: "inset" },
  { x: 1282, y: 306, w: 30, h: 26, z: 0.9, r: 1, tone: "inset" },
  // exercise: treadmill, cycle, resistance rack, mat
  { x: 70, y: 478, w: 180, h: 62, z: 0.22, tone: "machine" },
  { x: 232, y: 478, w: 18, h: 62, z: 1.2, tone: "appliance" },
  { x: 366, y: 478, w: 54, h: 26, z: 0.9, tone: "machine" },
  { x: 380, y: 504, w: 26, h: 44, z: 0.5, tone: "machine" },
  { x: 72, y: 602, w: 130, h: 104, z: 0.05, tone: "tray" },
  { x: 80, y: 610, w: 12, h: 88, z: 1.8, tone: "appliance" },
  { x: 182, y: 610, w: 12, h: 88, z: 1.8, tone: "appliance" },
  { x: 92, y: 650, w: 90, h: 8, z: 1.2, tone: "machine" },
  { x: 236, y: 640, w: 176, h: 76, z: 0.03, tone: "mat" },
  // shelter: benches on three sides
  { x: 474, y: 500, w: 38, h: 206, z: 0.45, tone: "bench" },
  { x: 648, y: 500, w: 38, h: 206, z: 0.45, tone: "bench" },
  { x: 512, y: 668, w: 136, h: 38, z: 0.45, tone: "bench" },
  // hub: rack row on the wall, a second rack row, the crew desk and its chair
  { x: 726, y: 500, w: 40, h: 50, z: 1.5, tone: "rack" },
  { x: 726, y: 552, w: 40, h: 50, z: 1.5, tone: "rack" },
  { x: 726, y: 604, w: 40, h: 50, z: 1.5, tone: "rack" },
  { x: 726, y: 656, w: 40, h: 50, z: 1.5, tone: "rack" },
  { x: 800, y: 604, w: 40, h: 50, z: 1.5, tone: "rack" },
  { x: 800, y: 656, w: 40, h: 50, z: 1.5, tone: "rack" },
  { x: 852, y: 470, w: 42, h: 104, z: 0.76, tone: "counter" },
  { x: 818, y: 506, w: 28, h: 28, z: 0.5, r: 1, tone: "chair" },
  // med bay: exam bed, cabinets, a stool
  { x: 1046, y: 560, w: 150, h: 60, z: 0.7, r: 0.3, tone: "table" },
  { x: 1050, y: 566, w: 34, h: 48, z: 0.74, r: 0.5, tone: "inset" },
  { x: 960, y: 690, w: 300, h: 44, z: 1.0, tone: "locker" },
  { x: 1286, y: 520, w: 44, h: 160, z: 1.6, tone: "locker" },
  { x: 1214, y: 576, w: 26, h: 26, z: 0.5, r: 1, tone: "chair" },
  { x: 920, y: 470, w: 44, h: 110, z: 0.9, tone: "counter" },
]

/* ---------- live overlay (what the mesh reports now, 14:06) ---------- */

const seat = id => seats.find(s => s.id === id)

// tone: critical | helper | ok. callout/note only for the people who get a label (short: one-line version for
// narrow cards); side = preferred compass side for the label.
export const people = [
  { id: "ana", name: "Ana Ruiz", seat: "4B", ...seat("4B"), tone: "critical", callout: "Ana Ruiz · 4B", note: "No pulse", short: "Ana · no pulse", side: "w" },
  { id: "leo", name: "Leo Brandt", seat: "3B", x: 298, y: 248, tone: "helper", callout: "Leo · CPR", note: "beside her", side: "sw" },
  { id: "tomas", name: "Tomás Rey", seat: "1B", x: 396, y: 248, tone: "helper", callout: "Tomás · AED", note: "from station B", side: "ne" },
  { id: "priya", name: "Priya Nair", seat: "4A", ...seat("4A"), tone: "ok" },
  { id: "lena", name: "Lena Vogel", seat: "3A", ...seat("3A"), tone: "ok" },
  { id: "diego", name: "Diego Luna", seat: "5A", ...seat("5A"), tone: "ok" },
  { id: "ivan", name: "Ivan Petrov", seat: "7A", ...seat("7A"), tone: "ok" },
  { id: "noah", name: "Noah Clark", seat: "8A", ...seat("8A"), tone: "ok" },
  { id: "chloe", name: "Chloé Martin", seat: "5B", x: 905, y: 176, tone: "ok" },
  { id: "kwame", name: "Kwame Mensah", seat: "6A", x: 1000, y: 280, tone: "ok" },
  { id: "hana", name: "Hana Kim", seat: "7B", x: 1258, y: 252, tone: "ok" },
  { id: "mara", name: "Mara Okoye", seat: "1A", x: 832, y: 548, tone: "ok" },
  { id: "yuki", name: "Yuki Sato", seat: "2A", x: 300, y: 560, tone: "ok" },
  { id: "omar", name: "Omar Haddad", seat: "2B", x: 360, y: 600, tone: "ok" },
  { id: "sofia", name: "Sofia Rossi", seat: "6B", x: 300, y: 680, tone: "ok" },
]

// Tomás's way with the AED: out of station B, along corridor B, through the cabin's fwd door, down the aisle to Ana.
// Drawn as a smooth curve through these points; it stops at the edge of Ana's marker.
export const aedRoute = {
  from: "B",
  to: "ana",
  points: [[625, 364], [612, 402], [552, 404], [550, 344], [530, 274], [452, 258], [396, 248], [364, 213]],
}

// LoRa mesh: drawn as the Delaunay neighbours of these nodes (every wristband, plus the hub as gateway)
export const meshMaxLink = 360
