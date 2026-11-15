// Eva service worker: offline shell + queued posts.
// - Static assets: cache first (they are embedded in the binary and versioned by CACHE).
// - Pages (/app, /dash): network first, cached copy when offline.
// - SSE (/sse), /link and /api GETs: network only (live data must not be stale).
// - POSTs to journal / Eva tools / chat while offline: stored in IndexedDB and replayed
//   when the page says it is back online (or on the next successful request).
const CACHE = "eva-shell-v1";
const SHELL = [
  "/app", "/dash", "/manifest.webmanifest",
  "/static/css/app.css", "/static/js/htmx.min.js", "/static/js/htmx-sse.js",
  "/static/js/motion.js", "/static/js/eva.js", "/static/js/link.js",
  "/static/icons/icon-192.png", "/static/icons/icon-512.png",
  "/static/img/eva/eva-neutral.webp", "/static/img/eva/eva-happy.webp", "/static/img/eva/eva-wink.webp", "/static/img/eva/eva-care.webp",
  "/static/img/icons/heart.webp", "/static/img/icons/lungs.webp", "/static/img/icons/skin-temp.webp", "/static/img/icons/oxygen.webp",
  "/static/img/wrist/band-top.webp", "/static/img/wrist/band-base.webp", "/static/img/wrist/band-interface.webp"
];
const QUEUED = ["/api/journal", "/api/eva/tool", "/api/eva/chat", "/api/shelter", "/api/help"];

self.addEventListener("install", (e) => {
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(SHELL)).then(() => self.skipWaiting()));
});
self.addEventListener("activate", (e) => {
  e.waitUntil(caches.keys().then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))).then(() => self.clients.claim()));
});

function db() {
  return new Promise((resolve, reject) => {
    const r = indexedDB.open("eva-outbox", 1);
    r.onupgradeneeded = () => r.result.createObjectStore("posts", { keyPath: "id", autoIncrement: true });
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
}
async function enqueue(req) {
  const body = await req.clone().text();
  const d = await db();
  await new Promise((res, rej) => {
    const tx = d.transaction("posts", "readwrite");
    tx.objectStore("posts").add({ url: req.url, body, type: req.headers.get("content-type") || "application/x-www-form-urlencoded", at: Date.now() });
    tx.oncomplete = res; tx.onerror = () => rej(tx.error);
  });
}
async function replay() {
  const d = await db();
  const items = await new Promise((res) => { const out = []; const tx = d.transaction("posts", "readonly"); tx.objectStore("posts").openCursor().onsuccess = (e) => { const c = e.target.result; if (c) { out.push(c.value); c.continue(); } else res(out); }; });
  let sent = 0;
  for (const it of items) {
    try {
      const r = await fetch(it.url, { method: "POST", body: it.body, headers: { "content-type": it.type, "x-eva-replayed": "1" } });
      if (r.ok || r.status < 500) {
        await new Promise((res) => { const tx = d.transaction("posts", "readwrite"); tx.objectStore("posts").delete(it.id); tx.oncomplete = res; });
        sent++;
      }
    } catch (_) { break; }
  }
  if (sent) (await self.clients.matchAll()).forEach((c) => c.postMessage({ type: "replayed", count: sent }));
}
self.addEventListener("message", (e) => { if (e.data === "replay") e.waitUntil(replay()); });

self.addEventListener("fetch", (e) => {
  const req = e.request, url = new URL(req.url);
  if (url.origin !== location.origin) return;
  if (req.method === "POST") {
    if (!QUEUED.some((p) => url.pathname === p)) return;
    e.respondWith(fetch(req.clone()).then((r) => { replay(); return r; }).catch(async () => {
      await enqueue(req);
      return new Response('<p class="toast pointer-events-auto rounded-full border border-warn/40 bg-warn/10 px-4 py-2 text-sm text-warn" data-toast>Saved on this device · will send when the hub is back</p>', { status: 202, headers: { "content-type": "text/html" } });
    }));
    return;
  }
  if (req.method !== "GET") return;
  if (url.pathname.startsWith("/sse") || url.pathname.startsWith("/link") || url.pathname.startsWith("/api") || url.pathname.startsWith("/export")) return;
  if (url.pathname.startsWith("/static/")) {
    e.respondWith(caches.match(req).then((hit) => hit || fetch(req).then((r) => { const copy = r.clone(); caches.open(CACHE).then((c) => c.put(req, copy)); return r; })));
    return;
  }
  // pages and htmx fragments: network first, cache as fallback
  e.respondWith(fetch(req).then((r) => {
    if (r.ok && !req.headers.get("hx-request")) { const copy = r.clone(); caches.open(CACHE).then((c) => c.put(url.pathname, copy)); }
    return r;
  }).catch(() => caches.match(url.pathname).then((hit) => hit || caches.match("/app"))));
});
