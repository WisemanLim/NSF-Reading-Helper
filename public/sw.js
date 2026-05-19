// NSF Reading Helper — minimal service worker.
//
// Strategy:
//   - App shell (HTML + same-origin /assets/*) cached on install + stale-while-revalidate
//     on subsequent loads.
//   - /api/* and /ocr/*: network-first, fall through to cache for read-only GETs.
//   - cross-origin (MathJax CDN, fonts, etc.): cache-first opportunistically.

const VERSION = "nsf-cache-v1";
const SHELL = [
  "/",
  "/index.html",
  "/manifest.webmanifest",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(VERSION);
      try {
        await cache.addAll(SHELL);
      } catch (e) {
        console.warn("sw: shell precache partial", e);
      }
      self.skipWaiting();
    })(),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys();
      await Promise.all(
        keys.filter((k) => k !== VERSION).map((k) => caches.delete(k)),
      );
      await self.clients.claim();
    })(),
  );
});

function isApiRequest(url) {
  return url.pathname.startsWith("/api/") || url.pathname.startsWith("/ocr/");
}

function isNavigationRequest(req) {
  return req.mode === "navigate";
}

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;

  const url = new URL(req.url);

  // SPA navigation — fall back to cached index.html when offline.
  if (isNavigationRequest(req)) {
    event.respondWith(
      (async () => {
        try {
          return await fetch(req);
        } catch {
          const cached = await caches.match("/index.html");
          return cached ?? Response.error();
        }
      })(),
    );
    return;
  }

  // API / OCR — network first, fall through to last successful cache for read-only.
  if (isApiRequest(url)) {
    event.respondWith(
      (async () => {
        try {
          const resp = await fetch(req);
          if (resp.ok) {
            const clone = resp.clone();
            caches.open(VERSION).then((c) => c.put(req, clone)).catch(() => {});
          }
          return resp;
        } catch {
          const cached = await caches.match(req);
          if (cached) return cached;
          return new Response(
            JSON.stringify({ error: "오프라인입니다" }),
            { status: 503, headers: { "content-type": "application/json" } },
          );
        }
      })(),
    );
    return;
  }

  // Static / cross-origin — cache-first then network.
  event.respondWith(
    (async () => {
      const cached = await caches.match(req);
      if (cached) return cached;
      try {
        const resp = await fetch(req);
        // Cache successful GETs of static assets opportunistically.
        if (resp.ok && (url.origin === self.location.origin ||
                         url.host.endsWith("jsdelivr.net"))) {
          const clone = resp.clone();
          caches.open(VERSION).then((c) => c.put(req, clone)).catch(() => {});
        }
        return resp;
      } catch {
        return cached ?? Response.error();
      }
    })(),
  );
});
