#!/usr/bin/env node
// Re-verify the cloud copy without trusting Supabase: read a ship's mirrored records and its pinned
// public key, write them in the app's chain file format, then run the app's own verifier:
//
//   node supabase/scripts/export-chain.mjs --ship demo-ship --out /tmp/eva-cloud
//   app/target/release/eva verify --data /tmp/eva-cloud
//
// Reads through the REST API as a signed-in user who may read records (crew_lead, mission_control,
// or team on a demo ship): SUPABASE_URL, SUPABASE_ANON_KEY and EVA_ACCESS_TOKEN (that user's JWT).
// Or offline: --rows rows.json --pubkey <hex> (rows as returned by `select ... from records`).

import { parseArgs } from "node:util";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const { values: a } = parseArgs({
  options: {
    ship: { type: "string", default: "demo-ship" },
    epoch: { type: "string", default: "1" },
    out: { type: "string" },
    rows: { type: "string" },
    pubkey: { type: "string" },
  },
});
if (!a.out) {
  console.error("usage: export-chain.mjs --ship <id> --out <dir> [--epoch 1] [--rows rows.json --pubkey <hex>]");
  process.exit(2);
}

async function rest(path) {
  const url = process.env.SUPABASE_URL, key = process.env.SUPABASE_ANON_KEY, token = process.env.EVA_ACCESS_TOKEN;
  if (!url || !key || !token) throw new Error("set SUPABASE_URL, SUPABASE_ANON_KEY and EVA_ACCESS_TOKEN (or use --rows/--pubkey)");
  const r = await fetch(`${url}/rest/v1/${path}`, { headers: { apikey: key, authorization: `Bearer ${token}` } });
  if (!r.ok) throw new Error(`REST ${r.status}`);
  return r.json();
}

let rows, pubkey;
if (a.rows) {
  rows = JSON.parse(readFileSync(a.rows, "utf8"));
  pubkey = a.pubkey;
} else {
  const ship = encodeURIComponent(a.ship);
  [{ pubkey }] = await rest(`ships?id=eq.${ship}&select=pubkey`);
  rows = await rest(`records?ship=eq.${ship}&epoch=eq.${Number(a.epoch)}&select=ship,epoch,seq,ts,kind,subject,hash,prev,signature,body&order=seq.asc`);
}
if (!/^[0-9a-f]{64}$/.test(pubkey ?? "")) throw new Error("ship public key missing");

rows.sort((x, y) => x.seq - y.seq);
const lines = [JSON.stringify({ t: "key", origin: a.ship, public: pubkey, from_seq: 0 })];
for (const r of rows) {
  lines.push(JSON.stringify({ t: "rec", seq: r.seq, ts_ms: r.ts, kind: r.kind, subject: r.subject, body: r.body, prev: r.prev, hash: r.hash, sig: r.signature, state: "acked", sent_ms: null, acked_ms: null, origin: a.ship }));
}
mkdirSync(join(a.out, "keys"), { recursive: true });
writeFileSync(join(a.out, `chain-${a.ship}.jsonl`), lines.join("\n") + "\n");
writeFileSync(join(a.out, "keys", `${a.ship}.pub`), pubkey + "\n");
console.log(`exported ${rows.length} records of ${a.ship} (epoch ${a.epoch}) → ${a.out}; now: eva verify --data ${a.out}`);
