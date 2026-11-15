// GET /api/status · public counts for the /status/ page (Vercel Function, Node).
// Reads the public_stats view with the public anon key (SUPABASE_URL, SUPABASE_ANON_KEY env vars).
// public_stats exposes aggregates only (no health data, names, seats, hashes or keys).
// Same-origin for the page, so the site's CSP (connect-src 'self') stays unchanged.
export default async function handler(req: any, res: any) {
  res.setHeader("x-content-type-options", "nosniff");
  if (req.method !== "GET" && req.method !== "HEAD") {
    res.setHeader("allow", "GET, HEAD");
    return res.status(405).json({ error: "method_not_allowed" });
  }
  const url = process.env.SUPABASE_URL;
  const key = process.env.SUPABASE_ANON_KEY;
  if (!url || !key) {
    res.setHeader("cache-control", "no-store");
    return res.status(503).json({ error: "backend_not_configured" });
  }
  try {
    const r = await fetch(`${url}/rest/v1/public_stats?select=ship,display_name,demo,epoch,acknowledged_records,last_ingest_at,open_cases,total_cases,passengers,station_mode&order=ship.asc`, {
      headers: { apikey: key, accept: "application/json" },
      signal: AbortSignal.timeout(8000),
    });
    if (!r.ok) throw new Error(`supabase ${r.status}`);
    const ships = await r.json();
    res.setHeader("cache-control", "public, s-maxage=30, stale-while-revalidate=120");
    return res.status(200).json({ ok: true, ships, checkedAt: new Date().toISOString() });
  } catch (e) {
    console.error("status: backend unreachable", e instanceof Error ? e.message : "unknown");
    res.setHeader("cache-control", "no-store");
    return res.status(503).json({ error: "backend_unreachable" });
  }
}
