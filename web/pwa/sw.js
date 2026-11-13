// TURBOCURSA service worker: makes the game installable and playable offline.
const CACHE = "turbocursa-1.6.0";
const FILES = ["./", "index.html", "turbocursa.html", "manifest.webmanifest", "icon-192.png", "icon-512.png", "icon-maskable-512.png"];
const MAX_ENTRIES = 32;
// one cache entry per path: "?code=...", "?daily" and the like all share the page itself
const keyOf = u => { const x = new URL(u); return x.origin + x.pathname; };
self.addEventListener("install", e => {
  e.waitUntil(caches.open(CACHE).then(c => Promise.all(FILES.map(f => c.add(f).catch(() => {})))).then(() => self.skipWaiting()));
});
self.addEventListener("activate", e => {
  e.waitUntil(caches.keys().then(ks => Promise.all(ks.filter(k => k !== CACHE).map(k => caches.delete(k)))).then(() => self.clients.claim()));
});
self.addEventListener("fetch", e => {
  const r = e.request;
  // only plain same-origin GETs; ranges (video, partial content) go straight to the network
  if (r.method !== "GET" || r.headers.has("range") || new URL(r.url).origin !== location.origin) return;
  const key = keyOf(r.url);
  // network first (so updates arrive), cache as the offline fallback
  e.respondWith(fetch(r).then(res => {
    if (res.status === 200 && res.type === "basic") {
      const copy = res.clone();
      e.waitUntil(caches.open(CACHE).then(async c => {
        await c.put(key, copy);
        const ks = await c.keys();
        for (const k of ks.slice(0, Math.max(0, ks.length - MAX_ENTRIES))) await c.delete(k);
      }).catch(() => {}));
    }
    return res;
  }).catch(async () => (await caches.match(key)) || (await caches.match("turbocursa.html")) || (await caches.match("index.html")) || Response.error()));
});
