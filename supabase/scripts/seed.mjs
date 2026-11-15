#!/usr/bin/env node
// Generate the DEMO seed for Supabase from the app's fictional seed (app/src/seed.rs), ported from
// the Firebase kit's scripts/seed.mjs: day 96 of 212, fifteen people, Ana Ruiz in 4B, Leo Brandt 3B,
// Tomás Rey 1B, Priya's ECG review, Diego's low sensor battery, Kwame running warm, a suit drill,
// and a sample cardiac case.
//
// Writes:
//   supabase/seed.sql                 SQL run as the service role (records are service-role-only)
//   <verify-dir>/chain-<ship>.jsonl   the same chain in chain::Record JSONL
//   <verify-dir>/keys/<ship>.pub      the demo public key, so `eva verify --data <verify-dir>` checks it
//
// Records are hashed exactly like app/src/chain.rs and Ed25519-signed with a fresh demo key that is
// never written anywhere (only its public half is pinned on the ship row). Node 18+, no dependencies.
//
//   node supabase/scripts/seed.mjs [--ship demo-ship] [--out supabase/seed.sql] [--verify-dir DIR]

import { parseArgs } from "node:util";
import { createHash, generateKeyPairSync, sign, verify } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const { values: a } = parseArgs({
  options: {
    ship: { type: "string", default: "demo-ship" },
    out: { type: "string", default: join(here, "..", "seed.sql") },
    "verify-dir": { type: "string" },
  },
});
const SHIP = a.ship;
if (!/^[a-z0-9-]{1,40}$/.test(SHIP)) throw new Error("ship id: lowercase letters, digits and dashes");

// ---------------------------------------------------------------- data (from app/src/seed.rs)

const DAY_MS = 86_400_000;
const DAY0 = (96 - 1) * DAY_MS;
const h = (hh, mm) => DAY0 + hh * 3_600_000 + mm * 60_000;

const ROSTER = [
  ["4B", "Ana Ruiz", null, false, "ok"],
  ["4A", "Priya Nair", null, false, "ecg"],
  ["5A", "Diego Luna", null, false, "battery"],
  ["6A", "Kwame Mensah", null, false, "warm"],
  ["1A", "Mara Okoye", "Crew lead", true, "ok"],
  ["1B", "Tomás Rey", "Flight engineer", true, "ok"],
  ["3B", "Leo Brandt", null, true, "ok"],
  ["2A", "Yuki Sato", null, true, "suit"],
  ["2B", "Omar Haddad", null, true, "suit"],
  ["6B", "Sofia Rossi", null, false, "suit"],
  ["3A", "Lena Vogel", null, true, "ok"],
  ["5B", "Chloé Martin", null, false, "ok"],
  ["7A", "Ivan Petrov", null, false, "ok"],
  ["7B", "Hana Kim", null, false, "ok"],
  ["8A", "Noah Clark", null, false, "ok"],
];
const GROUP = { ok: "ok", suit: "ok", ecg: "watch", battery: "watch", warm: "watch" };
const label = (name) => {
  const [first, ...rest] = name.split(" ");
  return rest.length ? `${first} ${rest[rest.length - 1][0]}.` : first;
};

const RECORDS = [
  [h(8, 0), "vitals_summary", "ship", { text: "Daily vitals · 15 passengers" }],
  [h(9, 30), "log", "ship", { text: "Cabin dose log · morning" }],
  [h(11, 15), "alert", "5A", { text: "Diego Luna · sensor base battery low" }],
  [h(12, 40), "ecg", "4A", { text: "Priya Nair · wrist ECG, irregular · review asked" }],
  [h(14, 0), "alarm", "4B", { text: "Ana Ruiz · no pulse, no movement, no answer" }],
  [h(14, 0), "case", "4B", { text: "Case opened · CPR: Leo Brandt (3B, 3 m) · AED: Tomás Rey (1B)" }],
  [h(14, 0) + 41_000, "station", "ship", { text: "Station · door 01 opened" }],
  [h(14, 0) + 47_000, "station", "ship", { text: "AED taken at the station · 00:47" }],
  [h(14, 2), "case", "4B", { text: "AED attached · shock advised by the AED" }],
  [h(14, 9), "case", "4B", { text: "Case closed · crew lead took over" }],
];

const COMPARTMENTS = ["AED", "Breathing", "Diagnostics", "Ultrasound", "Wounds", "Restraint", "Oral meds", "Injectables", "IV access", "Eyes and dental", "Airway", "Hygiene"];

// ---------------------------------------------------------------- hashing (same as chain.rs)

const GENESIS = "0".repeat(64);
const SIG_DOMAIN = Buffer.from("eva-record-v1\0", "latin1");

/** Canonical JSON like serde_json::Value without preserve_order: keys sorted, compact. */
function canonical(v) {
  if (Array.isArray(v)) return `[${v.map(canonical).join(",")}]`;
  if (v && typeof v === "object") return `{${Object.keys(v).sort().map((k) => `${JSON.stringify(k)}:${canonical(v[k])}`).join(",")}}`;
  return JSON.stringify(v);
}
function u64(n) {
  const b = Buffer.alloc(8);
  b.writeBigUInt64BE(BigInt(n));
  return b;
}
export function computeHash(prev, seq, ts, kind, subject, origin, body) {
  const hh = createHash("sha256");
  hh.update(Buffer.from(prev, "hex"));
  hh.update(u64(seq));
  hh.update(u64(ts));
  for (const part of [kind, subject, origin]) {
    hh.update(Buffer.from(part, "utf8"));
    hh.update(Buffer.from([0]));
  }
  hh.update(Buffer.from(canonical(body), "utf8"));
  return hh.digest("hex");
}

const { publicKey, privateKey } = generateKeyPairSync("ed25519");
const pubHex = Buffer.from(publicKey.export({ format: "jwk" }).x, "base64url").toString("hex");
const signHash = (hashHex) => sign(null, Buffer.concat([SIG_DOMAIN, Buffer.from(hashHex, "hex")]), privateKey).toString("hex");

let prev = GENESIS;
const chain = RECORDS.map(([ts, kind, subject, body], seq) => {
  const hash = computeHash(prev, seq, ts, kind, subject, SHIP, body);
  const sig = signHash(hash);
  if (!verify(null, Buffer.concat([SIG_DOMAIN, Buffer.from(hash, "hex")]), publicKey, Buffer.from(sig, "hex"))) throw new Error(`self-check failed at seq ${seq}`);
  const r = { seq, ts_ms: ts, kind, subject, body, prev, hash, sig };
  prev = hash;
  return r;
});

// ---------------------------------------------------------------- SQL

const q = (v) => (v === null || v === undefined ? "null" : typeof v === "boolean" || typeof v === "number" ? String(v) : `'${String(v).replace(/'/g, "''")}'`);
const j = (v) => `${q(JSON.stringify(v))}::jsonb`;
const now = Date.now();

const sql = [];
sql.push(`-- Eva demo seed (fictional data from app/src/seed.rs). Generated by supabase/scripts/seed.mjs.`);
sql.push(`-- Demo Ed25519 public key ${pubHex}; the private key was discarded after signing.`);
sql.push(`-- Idempotent: an existing ship keeps its pinned key and records (append-only); rows conflict-skip.`);
sql.push(`begin;`);
sql.push(`set local role service_role;`);
sql.push(`set local request.jwt.claims = '{"role":"service_role"}';`);
sql.push(`insert into public.ships (id, display_name, demo, pubkey, epoch, head_seq, head_hash, last_ingest_at, notes) values (${q(SHIP)}, 'Eva demo ship (fictional data)', true, ${q(pubHex)}, 1, ${chain.length - 1}, ${q(prev)}, now(), 'Seeded from app/src/seed.rs · day 96 of 212') on conflict (id) do nothing;`);
sql.push(`insert into public.records (ship, epoch, seq, ts, kind, subject, hash, prev, signature, body, ground_received_ms) values`);
sql.push(chain.map((r) => `  (${q(SHIP)}, 1, ${r.seq}, ${r.ts_ms}, ${q(r.kind)}, ${q(r.subject)}, ${q(r.hash)}, ${q(r.prev)}, ${q(r.sig)}, ${j(r.body)}, ${now})`).join(",\n") + `\non conflict do nothing;`);
sql.push(`insert into public.passengers (ship, seat, label, role, trained, grp, status) values`);
sql.push(ROSTER.map(([seat, name, role, trained, status]) => `  (${q(SHIP)}, ${q(seat)}, ${q(label(name))}, ${q(role)}, ${trained}, ${q(GROUP[status])}, ${q(status)})`).join(",\n") +
  `\non conflict (ship, seat) do update set label = excluded.label, role = excluded.role, trained = excluded.trained, grp = excluded.grp, status = excluded.status, updated_at = now();`);
const steps = chain.filter((r) => r.seq >= 4).map((r) => ({ ts_ms: r.ts_ms, title: r.body.text, seq: r.seq }));
sql.push(`insert into public.cases (ship, epoch, case_no, seat, kind, open, opened_ms, closed_ms, helpers, steps, now, why, how, next, last_seq) values (${q(SHIP)}, 1, 1, '4B', 'cardiac', false, ${h(14, 0)}, ${h(14, 9)}, ${j([{ seat: "3B", task: "CPR" }, { seat: "1B", task: "AED" }])}, ${j(steps)}, 'Crew lead has taken over.', 'No pulse, no movement and no answer on the band.', 'Layer 1 reflex rule (three signs), no model involved.', 'Ground follows the signed records; the AED decided the shock.', ${chain.length - 1}) on conflict do nothing;`);
sql.push(`insert into public.station_state (ship, mode, compartments, screen, last_event) values (${q(SHIP)}, 'online', ${j(COMPARTMENTS.map((name, i) => ({ n: i + 1, name, state: "idle", count: i === 0 ? 1 : null })))}, 'EVA · All clear', 'Restocked after case 1') on conflict (ship) do update set mode = excluded.mode, compartments = excluded.compartments, screen = excluded.screen, last_event = excluded.last_event, updated_at = now();`);
sql.push(`insert into public.team_docs (id, title, body, kind, tags, author_id) values ('00000000-0000-4000-8000-00000000c0de', 'Welcome to the Eva team space', 'Reports, notes and asset links for team Sanjo-X. Health data lives in the ship tables, never here.', 'note', '{readme}', null) on conflict (id) do nothing;`);
sql.push(`commit;`);
mkdirSync(dirname(a.out), { recursive: true });
writeFileSync(a.out, sql.join("\n") + "\n");

// ---------------------------------------------------------------- JSONL for `eva verify`

if (a["verify-dir"]) {
  const dir = a["verify-dir"];
  mkdirSync(join(dir, "keys"), { recursive: true });
  // chain.rs file format: a "key" line, then one tagged "rec" line per record
  const lines = [JSON.stringify({ t: "key", origin: SHIP, public: pubHex, from_seq: 0 })].concat(
    chain.map((r) => JSON.stringify({ t: "rec", seq: r.seq, ts_ms: r.ts_ms, kind: r.kind, subject: r.subject, body: r.body, prev: r.prev, hash: r.hash, sig: r.sig, state: "acked", sent_ms: null, acked_ms: null, origin: SHIP })),
  );
  writeFileSync(join(dir, `chain-${SHIP}.jsonl`), lines.join("\n") + "\n");
  writeFileSync(join(dir, "keys", `${SHIP}.pub`), pubHex + "\n");
}
console.log(`seed: ${chain.length} signed records for ${SHIP} (head ${prev.slice(0, 12)}…), ${ROSTER.length} passengers, 1 case, station, team note → ${a.out}`);
