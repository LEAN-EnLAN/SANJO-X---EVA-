// Shared guard rails for the public Eva endpoints: same-origin (plus the Android app's bundled
// demo origin), per-IP rate limit, JSON body parsing. Keys never leave the server.
export const APP_ORIGIN = "https://appassets.androidplatform.net";

/** Returns true when the request was fully handled (preflight or rejected). */
export function preflight(req: any, res: any, methods = "POST"): boolean {
  res.setHeader("cache-control", "no-store");
  res.setHeader("x-content-type-options", "nosniff");
  const origin = req.headers.origin as string | undefined;
  if (origin === APP_ORIGIN) {
    res.setHeader("access-control-allow-origin", APP_ORIGIN);
    res.setHeader("vary", "origin");
  }
  if (req.method === "OPTIONS") {
    if (origin !== APP_ORIGIN) { res.status(403).end(); return true; }
    res.setHeader("access-control-allow-methods", methods);
    res.setHeader("access-control-allow-headers", "content-type");
    res.setHeader("access-control-max-age", "600");
    res.status(204).end();
    return true;
  }
  if (req.method !== "POST") {
    res.setHeader("allow", "POST");
    res.status(405).json({ error: "method_not_allowed" });
    return true;
  }
  const host = req.headers.host as string | undefined;
  if (origin && host && origin !== APP_ORIGIN) {
    let ok = false;
    try { ok = new URL(origin).host === host; } catch { ok = false; }
    if (!ok) { res.status(403).json({ error: "cross_origin" }); return true; }
  }
  return false;
}

const buckets = new Map<string, number[]>();
/** Seconds to wait, or 0 when allowed. */
export function limited(req: any, name: string, max: number, windowMs = 60_000): number {
  const ip = String(req.headers["x-forwarded-for"] ?? "").split(",")[0].trim() || "anon";
  const k = name + ":" + ip, now = Date.now();
  const list = (buckets.get(k) ?? []).filter((t) => now - t < windowMs);
  if (list.length >= max) { buckets.set(k, list); return Math.ceil((windowMs - (now - list[0])) / 1000); }
  list.push(now); buckets.set(k, list);
  if (buckets.size > 5000) buckets.clear();
  return 0;
}

export function jsonBody(req: any): any {
  let b: any = req.body;
  if (typeof b === "string") { try { b = JSON.parse(b); } catch { b = {}; } }
  return b && typeof b === "object" ? b : {};
}
