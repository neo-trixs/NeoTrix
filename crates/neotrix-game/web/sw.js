/* NeoTrix PWA Service Worker: cache-first + versioned cache. */
const CACHE_VERSION = 'neotrix-pwa-v4';
const CACHE_NAME = CACHE_VERSION;
const CORE_ASSETS = [
  './',
  './index.html',
  './manifest.webmanifest',
  './icon.svg',
  './gl.js',
  './pkg/neotrix-game.wasm'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(CORE_ASSETS)).then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k))))
      .then(() => self.clients.claim())
  );
});

async function cacheFirst(request) {
  const cached = await caches.match(request);
  if (cached) return cached;
  try {
    const resp = await fetch(request);
    if (resp && resp.ok) {
      const copy = resp.clone();
      const cache = await caches.open(CACHE_NAME);
      cache.put(request, copy);
    }
    return resp;
  } catch (e) {
    if (request.mode === 'navigate') {
      const fallback = await caches.match('./index.html');
      if (fallback) return fallback;
    }
    throw e;
  }
}

self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET') return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  // wasm/图片按前缀缓存，不硬编码文件名
  if (url.pathname.includes('/pkg/')) {
    event.respondWith(cacheFirst(request));
    return;
  }
  event.respondWith(cacheFirst(request));
});
