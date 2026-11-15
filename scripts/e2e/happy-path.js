// Eva happy path, end to end, in a real (headless) browser.
//
// Usage: scripts/e2e/run.sh   (builds, starts `eva demo`, runs this, stops it)
//   or:  BASE=http://127.0.0.1:18080 GROUND=http://127.0.0.1:18081 OUT=/tmp/eva-e2e node scripts/e2e/happy-path.js
//
// Story: open the app as Ana (4B), Leo (3B) and Tomás (1B), the crew dashboard, the
// medical station screen (/station) and the ground dashboard · ask Eva · log water and a
// mood · run an ECG · cardiac arrest from the dev panel (Konami code) · three phones and
// the station light up (compartment 01) · Leo gets CPR · the AED is taken at the station
// (tap on 01) and "AED taken" reaches the dashboard · Tomás brings it · the case closes and
// 01 clears · every record (station reports included) reaches the ground, verified and
// acknowledged · export CSV, JSON, report.
// Screenshots go to $OUT. Exit code 0 = pass.
"use strict";
const { chromium } = require("playwright");
const fs = require("fs");
const path = require("path");

const BASE = process.env.BASE || "http://127.0.0.1:18080";
const GROUND = process.env.GROUND || "http://127.0.0.1:18081";
const OUT = process.env.OUT || path.join(process.cwd(), "e2e-out");
const HEADED = process.env.HEADED === "1";
fs.mkdirSync(OUT, { recursive: true });

const results = [];
let shotN = 0;
const t0 = Date.now();
const log = (...a) => console.log(`[${((Date.now() - t0) / 1000).toFixed(1).padStart(5)} s]`, ...a);

async function shot(page, name) {
  shotN += 1;
  const file = path.join(OUT, `${String(shotN).padStart(2, "0")}-${name}.png`);
  await page.screenshot({ path: file, fullPage: false });
  return file;
}

async function step(name, fn) {
  const t = Date.now();
  try {
    await fn();
    results.push({ name, ok: true, ms: Date.now() - t });
    log(`PASS  ${name}`);
  } catch (e) {
    results.push({ name, ok: false, ms: Date.now() - t, error: String(e && e.message || e).split("\n")[0] });
    log(`FAIL  ${name}: ${String(e && e.message || e).split("\n")[0]}`);
    throw e;
  }
}

async function api(ctx, url, opts) {
  const r = await ctx.request.fetch(url, opts);
  if (!r.ok()) throw new Error(`${url} → ${r.status()}`);
  const ct = r.headers()["content-type"] || "";
  return ct.includes("json") ? r.json() : r.text();
}

async function until(what, fn, ms = 20000, every = 250) {
  const end = Date.now() + ms;
  let last;
  while (Date.now() < end) {
    try {
      last = await fn();
      if (last) return last;
    } catch (e) { last = e; }
    await new Promise((r) => setTimeout(r, every));
  }
  throw new Error(`timed out waiting for ${what}${last instanceof Error ? ": " + last.message : ""}`);
}

async function dev(ctx, action, form) {
  return api(ctx, `${BASE}/api/dev/${action}`, { method: "POST", form: form || {} });
}

(async () => {
  const browser = await chromium.launch({ headless: !HEADED });
  const phone = { viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true };
  const desk = { viewport: { width: 1440, height: 900 } };
  // one browser context per device, like separate phones and laptops
  const ctx = {
    ana: await browser.newContext(phone),
    leo: await browser.newContext(phone),
    tomas: await browser.newContext(phone),
    dash: await browser.newContext({ ...desk, acceptDownloads: true }),
    // a tablet or second laptop mounted as the medical station, landscape
    station: await browser.newContext({ viewport: { width: 1280, height: 800 } }),
    ground: await browser.newContext(desk),
  };
  const page = {};
  const errors = [];
  for (const [k, c] of Object.entries(ctx)) {
    page[k] = await c.newPage();
    page[k].on("pageerror", (e) => errors.push(`${k}: ${e.message}`));
  }
  let exit = 0;
  try {
    await step("demo is up (ship and ground)", async () => {
      await until("ship /healthz", async () => (await api(ctx.dash, `${BASE}/healthz`)) === "ok");
      await until("ground /healthz", async () => (await api(ctx.dash, `${GROUND}/healthz`)) === "ok");
      // the uplink flows fast for the test; the demo default is the near-space profile
      await dev(ctx.dash, "profile", { profile: "lan" });
    });

    await step("open the app as 4B, 3B, 1B, the crew /dash, the station /station and the ground /dash", async () => {
      await page.ana.goto(`${BASE}/app?seat=4B`);
      await page.leo.goto(`${BASE}/app?seat=3B`);
      await page.tomas.goto(`${BASE}/app?seat=1B`);
      await page.dash.goto(`${BASE}/dash`);
      await page.station.goto(`${BASE}/station`);
      await page.ground.goto(`${GROUND}/dash`);
      await page.ana.getByText(/Good \w+,\s*Ana/).first().waitFor();
      await page.leo.getByText(/Good \w+,\s*Leo/).first().waitFor();
      await page.tomas.getByText(/Good \w+,\s*Tom/).first().waitFor();
      await page.dash.locator("#roster, [sse-swap=roster]").first().waitFor();
      await page.station.locator("[data-station-hub]").getByText("Hub online").waitFor({ timeout: 10000 });
      if ((await page.station.locator('[data-compartment]').count()) !== 12) throw new Error("the station shows 12 compartments");
      await page.ground.waitForLoadState("domcontentloaded");
      await new Promise((r) => setTimeout(r, 1500)); // SSE streams connect
      await page.dash.locator("[data-station-status]").getByText("Online").waitFor({ timeout: 10000 });
      for (const k of ["ana", "leo", "tomas", "dash", "station", "ground"]) await shot(page[k], `open-${k}`);
    });

    await step("ask Eva a question (Eva screen, typed)", async () => {
      await page.ana.goto(`${BASE}/app/eva?seat=4B`);
      const input = page.ana.locator("#eva-text");
      await input.fill("How am I doing?");
      await input.press("Enter");
      await page.ana.locator("#chat").getByText(/Now:|pulse|bpm/i).first().waitFor({ timeout: 15000 });
      const chat = await page.ana.locator("#chat").innerText();
      if (!/Now/.test(chat) || !/Next/.test(chat)) throw new Error("answer is not in the Now / Why / How / Next format: " + chat.slice(0, 200));
      await shot(page.ana, "eva-answer");
    });

    await step("log water (Eva chip, +1 glass)", async () => {
      const before = (await api(ctx.ana, `${BASE}/api/passengers/4B`)).passenger.water_ml;
      await page.ana.locator('[data-tool="log_water"]').first().click();
      await until("water +250 ml", async () => (await api(ctx.ana, `${BASE}/api/passengers/4B`)).passenger.water_ml === before + 250);
      await shot(page.ana, "water-logged");
    });

    await step("log a mood (journal: valence + a feeling, saved)", async () => {
      await page.ana.goto(`${BASE}/app/journal?seat=4B`);
      const form = page.ana.locator("form[data-journal]");
      await form.waitFor();
      await page.ana.locator("#valence").fill("5");
      const firstLabel = form.locator('input[name="labels"]').first();
      if (await firstLabel.count()) await firstLabel.evaluate((el) => { el.checked = true; });
      const before = (await api(ctx.ana, `${BASE}/api/chain?limit=2000`)).filter((r) => r.kind === "tool" && r.body.tool === "log_mood").length;
      await form.locator('button[type="submit"]').last().click();
      await until("log_mood on the chain", async () => (await api(ctx.ana, `${BASE}/api/chain?limit=2000`)).filter((r) => r.kind === "tool" && r.body.tool === "log_mood").length === before + 1);
      await shot(page.ana, "mood-saved");
    });

    await step("run a 30 s wrist ECG", async () => {
      await page.ana.goto(`${BASE}/app/eva?seat=4B`);
      await page.ana.locator('[data-tool="start_ecg_check"]').first().click();
      await until("ECG recording", async () => (await api(ctx.ana, `${BASE}/api/passengers/4B`)).passenger.ecg.result === "recording");
      await page.ana.goto(`${BASE}/app/ecg?seat=4B`);
      await shot(page.ana, "ecg-recording");
      await dev(ctx.ana, "ff", { minutes: "1" }); // the 30 s pass in mission time
      await until("ECG result", async () => (await api(ctx.ana, `${BASE}/api/passengers/4B`)).passenger.ecg.result === "regular");
      await page.ana.goto(`${BASE}/app/ecg?seat=4B`);
      await page.ana.getByText(/Normal for you|regular/i).first().waitFor();
      await shot(page.ana, "ecg-result");
    });

    await step("cardiac arrest from the dev panel (Konami code on the dashboard)", async () => {
      await page.dash.bringToFront();
      await page.dash.locator("body").click({ position: { x: 700, y: 450 } }).catch(() => {});
      for (const k of ["ArrowUp", "ArrowUp", "ArrowDown", "ArrowDown", "ArrowLeft", "ArrowRight", "ArrowLeft", "ArrowRight", "b", "a"]) await page.dash.keyboard.press(k);
      const panel = page.dash.locator("#dev-panel aside");
      await panel.waitFor({ state: "visible", timeout: 5000 });
      await shot(page.dash, "dev-panel");
      await panel.locator("select[name=seat]").selectOption("4B");
      await panel.getByRole("button", { name: /Cardiac arrest/ }).click();
      await panel.getByText(/Cardiac arrest started/).waitFor();
      // the band reports "no answer" 20 s after the pulse is lost: jump past it
      await panel.getByRole("button", { name: "+1 min" }).click();
      await until("the case opens", async () => (await api(ctx.dash, `${BASE}/api/state`)).cases.some((c) => c.open && c.seat === "4B"));
    });

    await step("WOW: three phones change screens and the station lights 01", async () => {
      await page.leo.getByText(/I'm going/).first().waitFor({ timeout: 15000 });
      await page.tomas.getByText(/Get the AED/).first().waitFor({ timeout: 15000 });
      await page.station.locator('[data-compartment="1"][data-state="lit"]').waitFor({ timeout: 10000 });
      await page.station.locator("[data-station-message]").getByText(/Take the AED/).waitFor({ timeout: 5000 });
      const lit = await page.station.locator('[data-state="lit"]').count();
      if (lit !== 1) throw new Error(`only compartment 01 should be lit, ${lit} are`);
      await page.ana.waitForURL(/emergency|app/, { timeout: 5000 }).catch(() => {});
      for (const k of ["ana", "leo", "tomas", "dash", "station"]) await shot(page[k], `emergency-${k}`);
      const st = await api(ctx.dash, `${BASE}/api/state`);
      const c = st.cases.find((c) => c.open && c.seat === "4B");
      const cpr = c.helpers.find((h) => h.task === "CPR"), aed = c.helpers.find((h) => h.task === "AED");
      if (!cpr || cpr.seat !== "3B") throw new Error("CPR helper should be 3B, got " + JSON.stringify(c.helpers));
      if (!aed || aed.seat !== "1B") throw new Error("AED runner should be 1B, got " + JSON.stringify(c.helpers));
    });

    await step("3B answers I'm going → CPR guidance", async () => {
      await page.leo.getByRole("button", { name: /I'm going/ }).first().click();
      await page.leo.getByText(/CPR|110|push/i).first().waitFor({ timeout: 10000 });
      await shot(page.leo, "leo-cpr");
    });

    await step("station: tap 01 (door opened, AED taken) → the dashboard shows it", async () => {
      await page.station.locator('[data-compartment="1"]').click();
      await page.station.locator('[data-compartment="1"][data-state="taken"]').waitFor({ timeout: 5000 });
      await until("AED taken on the case timeline", async () => {
        const c = (await api(ctx.dash, `${BASE}/api/state`)).cases.find((c) => c.open && c.seat === "4B");
        return c && c.steps.some((s) => /^AED taken at the station · \d\d:\d\d$/.test(s.title));
      }, 10000);
      await page.dash.locator("#case").getByText(/AED taken at the station · \d\d:\d\d/).waitFor({ timeout: 10000 });
      await page.dash.locator("[data-station-last]").getByText(/AED taken from compartment 01/).waitFor({ timeout: 10000 });
      await shot(page.station, "station-aed-taken");
      await shot(page.dash, "dash-aed-taken");
    });

    await step("1B accepts (I have it) → brings the AED", async () => {
      await page.tomas.getByRole("button", { name: /I'm going|I have it/ }).first().click();
      await page.tomas.getByText(/AED/).first().waitFor({ timeout: 10000 });
      await shot(page.tomas, "tomas-aed");
      await dev(ctx.dash, "ff", { minutes: "1" });
      await dev(ctx.dash, "ff", { minutes: "1" });
      await until("AED attached on the case timeline", async () => {
        const c = (await api(ctx.dash, `${BASE}/api/state`)).cases.find((c) => c.seat === "4B");
        return c && c.steps.some((s) => /AED attached/.test(s.title));
      });
      await page.dash.getByText(/AED/).first().waitFor();
      await shot(page.dash, "dash-case-aed");
    });

    await step("the crew lead closes the case", async () => {
      const panel = page.dash.locator("#dev-panel aside");
      if (!(await panel.isVisible())) {
        for (const k of ["ArrowUp", "ArrowUp", "ArrowDown", "ArrowDown", "ArrowLeft", "ArrowRight", "ArrowLeft", "ArrowRight", "b", "a"]) await page.dash.keyboard.press(k);
        await panel.waitFor({ state: "visible" });
      }
      await panel.locator("select[name=seat]").selectOption("4B");
      await panel.getByRole("button", { name: /Close case/ }).click();
      await until("case closed", async () => !(await api(ctx.dash, `${BASE}/api/state`)).cases.some((c) => c.open));
      await page.dash.keyboard.press("Escape");
      // the case closed: compartment 01 clears on the station
      await page.station.locator('[data-compartment="1"][data-state="idle"]').waitFor({ timeout: 10000 });
    });

    await step("every record reaches the ground: verified and acknowledged", async () => {
      const st = await until("all records acknowledged", async () => {
        const s = await api(ctx.dash, `${BASE}/api/state`);
        return s.chain.signed === 0 && s.chain.sent === 0 && s;
      }, 45000);
      const ship = await api(ctx.dash, `${BASE}/api/chain/verify`);
      // invariant (PROTOCOL.md §7): nothing pending on the ship → the ground's mirror
      // (current epoch) holds exactly the ship's chain, every record verified
      const ground = await until("ground mirror == ship chain", async () => {
        const g = await api(ctx.ground, `${GROUND}/api/chain/verify`);
        return g.mirror.records === ship.chain.records && g.mirror.verified === ship.chain.records && g;
      }, 20000);
      if (st.chain.acked !== ship.chain.records) throw new Error(`ship acked ${st.chain.acked} of ${ship.chain.records}`);
      if (!ship.chain.ok) throw new Error("ship chain does not verify: " + JSON.stringify(ship.chain));
      if (ground.mirror.failed.length) throw new Error("ground failed records: " + ground.mirror.failed);
      log(`      ship: ${ship.chain.records} records, ${ship.chain.signatures} signatures · acked ${st.chain.acked} · ground verified ${ground.mirror.verified}`);
      const g = await api(ctx.ground, `${GROUND}/api/state`);
      if (!g.cases.some((c) => c.seat === "4B" && !c.open)) throw new Error("the ground does not show the closed case");
      // the station's "AED taken" is a signed ship record the ground acknowledged and shows on its timeline
      const chain = await api(ctx.dash, `${BASE}/api/chain?limit=2000`);
      const taken = chain.find((r) => r.kind === "station" && r.body.event === "item_taken" && r.body.compartment === 1);
      if (!taken) throw new Error("no 'AED taken' station record on the ship's chain");
      if (taken.state !== "acked") throw new Error(`the station's 'AED taken' record is ${taken.state}, not acknowledged by the ground`);
      const gcase = g.cases.find((c) => c.seat === "4B");
      if (!gcase.steps.some((s) => /^AED taken at the station/.test(s.title))) throw new Error("the ground's case timeline lacks 'AED taken at the station'");
      log(`      station: 'AED taken' record #${taken.seq} acknowledged by the ground`);
      // fresh vitals on the ground again (the uplink sends a snapshot every 5 s; the
      // fast-forwards above made the last one look old), Ana's pulse back after the case
      await until("fresh vitals on the ground", async () => {
        const g = await api(ctx.ground, `${GROUND}/api/state`);
        const ana = g.passengers.find((p) => p.seat === "4B");
        return ana && ana.vitals.hr > 0 && g.ship_time_ms - ana.last_ms < 10000;
      }, 20000);
      await page.ground.reload();
      await page.ground.getByText(/Ana|4B/).first().waitFor();
      await shot(page.ground, "ground-case");
      await page.dash.reload();
      await page.dash.getByText(/Acknowledged/).first().waitFor({ timeout: 10000 });
      await shot(page.dash, "dash-acknowledged");
    });

    await step("export CSV, JSON and the report", async () => {
      // through the dashboard's own links
      const csvLink = page.dash.locator('a[href="/export/records.csv"]').first();
      const jsonLink = page.dash.locator('a[href="/export/records.json"]').first();
      for (const [link, name] of [[csvLink, "eva-records.csv"], [jsonLink, "eva-records.json"]]) {
        if (await link.isVisible()) {
          const [dl] = await Promise.all([page.dash.waitForEvent("download"), link.click()]);
          await dl.saveAs(path.join(OUT, name));
        } else {
          fs.writeFileSync(path.join(OUT, name), await api(ctx.dash, `${BASE}/export/${name.replace("eva-", "")}`).then((x) => typeof x === "string" ? x : JSON.stringify(x, null, 2)));
        }
      }
      const csv = fs.readFileSync(path.join(OUT, "eva-records.csv"), "utf8");
      if (!csv.startsWith("seq,ship_time")) throw new Error("CSV header: " + csv.slice(0, 40));
      if (!/alarm/.test(csv)) throw new Error("CSV has no alarm record");
      const json = JSON.parse(fs.readFileSync(path.join(OUT, "eva-records.json"), "utf8"));
      if (!json.verify.ok) throw new Error("JSON export says the chain does not verify");
      if (!json.records.every((r) => r.sig && r.sig.length === 128)) throw new Error("records in the JSON export are not all signed");
      const report = await ctx.dash.newPage();
      await report.goto(`${BASE}/export/report`);
      await report.getByText(/report/i).first().waitFor();
      await report.pdf({ path: path.join(OUT, "eva-report.pdf"), format: "A4" });
      await shot(report, "report");
      log(`      exports: ${csv.split("\n").length - 2} CSV rows · ${json.records.length} JSON records · report PDF ${fs.statSync(path.join(OUT, "eva-report.pdf")).size} bytes`);
    });

    if (errors.length) log(`note: ${errors.length} page error(s): ${errors.slice(0, 3).join(" | ")}`);
  } catch (e) {
    exit = 1;
    for (const k of Object.keys(page)) await shot(page[k], `failure-${k}`).catch(() => {});
  } finally {
    fs.writeFileSync(path.join(OUT, "results.json"), JSON.stringify({ base: BASE, ground: GROUND, results, pageErrors: errors }, null, 2));
    const passed = results.filter((r) => r.ok).length;
    console.log(`\n${passed}/${results.length} steps passed · ${shotN} screenshots in ${OUT} · ${((Date.now() - t0) / 1000).toFixed(1)} s`);
    await browser.close();
    process.exit(exit);
  }
})();
