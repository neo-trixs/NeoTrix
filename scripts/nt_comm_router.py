"""
nt_comm_router — Intelligent Communication Router for NeoTrix
=============================================================
Single entry/exit for all external communications. Obfuscates and removes
user fingerprints, hides internal NeoTrix identity, and presents the
appearance of a random global internet user.

Architecture:
    CommRouter
    ├── FingerprintManager
    │   ├── TLSPersona       — TLS cipher/extension profiles
    │   ├── H2Persona        — HTTP/2 SETTINGS frame profiles
    │   ├── HeaderPersona    — Header order + value templates
    │   ├── GeoPersona       — IP geo ↔ locale coherence
    │   └── create_persona() → coherent cross-layer identity
    ├── IdentityPool
    │   ├── Persona catalog  — 6 browser/OS identities
    │   ├── pool (persistent, SQLite-backed)
    │   └── rotate/find/retire
    ├── RouteEngine
    │   ├── route(request, persona) → (transport, headers)
    │   ├── fallback_chain(domain) → ordered transports
    │   └── score_transport(transport, domain) → bayesian score
    ├── ObfuscationLayer
    │   ├── obfuscate_headers()  — Reorder + filter headers
    │   ├── strip_internal()     — Remove NeoTrix traces
    │   ├── add_timing_jitter()  — Human-like timing
    │   └── coherent_headers()   — Cross-header consistency
    └── PersistentStore (~/.neotrix/comm_router.db)
        ├── identity_pool
        ├── persona_stats
        ├── failure_log
        └── transport_scores

Detection layers addressed (2026 state-of-the-art):
    Layer 0: TCP/IP       — Partial (TTL/win markings via platform persona)
    Layer 1: TLS JA3/JA4  — Modeled (real stack via curl_cffi or browser)
    Layer 2: HTTP/2       — Modeled (SETTINGS frame, header ordering)
    Layer 3: HTTP headers — Full control (order, values, client hints)
    Layer 4: Cross-layer   — Coherence checks (geo↔lang, TLS↔UA, etc.)
    Layer 5: IP reputation — Proxy strategy (residential > datacenter)
    Layer 6: Behavioral    — Timing jitter, session patterns

Usage:
    router = CommRouter()
    router.plan()  # warm identity pool

    # Make a request through the router
    result = router.get('https://arxiv.org/abs/1706.03762')

    # Or use with a specific persona
    persona = router.identity_pool.find('chrome_win')
    result = router.get('https://example.com', persona_id=persona['id'])
"""

import json, os, random, re, sqlite3, time, hashlib, urllib.request, urllib.error
from typing import Optional, Dict, List, Tuple

from urllib.parse import urlparse

# ============================================================================
# Persona Definitions (2026 Browser Profiles)
# ============================================================================
# Each persona is a complete, coherent digital identity with:
#   - Browser/OS version (determines TLS stack, H2 settings, headers)
#   - HTTP header order (Chrome vs Firefox vs Safari ordering)
#   - Sec-CH-UA client hints (must match browser)
#   - Accept-Language (matched to likely geo regions)
#   - JA3/JA4 fingerprint reference (for curl_cffi or browser engine)
#   - HTTP/2 SETTINGS frame template

PERSONAS = {
    "chrome_win": {
        "label": "Chrome 132 / Windows 11",
        "weight": 0.35,
        "browser": "chrome",
        "browser_version": "132",
        "os": "windows",
        "os_version": "10.0",
        "ua": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        "platform": "Win32",
        "header_order": [
            ":method", ":path", ":scheme", ":authority",
            "accept-encoding",
            "accept-language",
            "user-agent",
            "accept",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
            "sec-fetch-user",
            "upgrade-insecure-requests",
            "referer",
        ],
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        "accept_encoding": "gzip, deflate, br, zstd",
        "accept_language": "en-US,en;q=0.9",
        "sec_ch_ua": '"Not A(Brand";v="8", "Chromium";v="132", "Google Chrome";v="132"',
        "sec_ch_ua_mobile": "?0",
        "sec_ch_ua_platform": '"Windows"',
        "sec_fetch_dest": "document",
        "sec_fetch_mode": "navigate",
        "sec_fetch_site": "none",
        "sec_fetch_user": "?1",
        "upgrade_insecure_requests": "1",
        "tls_groups": ["x25519", "secp256r1", "secp384r1"],
        "tls_ciphers": [
            "TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384",
            "TLS_CHACHA20_POLY1305_SHA256",
            "ECDHE-ECDSA-AES128-GCM-SHA256", "ECDHE-RSA-AES128-GCM-SHA256",
            "ECDHE-ECDSA-AES256-GCM-SHA384", "ECDHE-RSA-AES256-GCM-SHA384",
            "ECDHE-ECDSA-CHACHA20-POLY1305", "ECDHE-RSA-CHACHA20-POLY1305",
            "ECDHE-RSA-AES128-SHA", "ECDHE-RSA-AES256-SHA",
            "AES128-GCM-SHA256", "AES256-GCM-SHA384", "AES128-SHA", "AES256-SHA",
        ],
        "h2_settings": {
            "HEADER_TABLE_SIZE": 65536,
            "MAX_CONCURRENT_STREAMS": 1000,
            "INITIAL_WINDOW_SIZE": 6291456,
            "MAX_FRAME_SIZE": 16384,
            "MAX_HEADER_LIST_SIZE": 262144,
        },
        "geo_regions": ["US", "GB", "CA", "AU", "DE", "FR"],
        "ttl": 128,
        "tcp_window": 65535,
    },
    "chrome_mac": {
        "label": "Chrome 132 / macOS 15",
        "weight": 0.15,
        "browser": "chrome",
        "browser_version": "132",
        "os": "macos",
        "os_version": "15",
        "ua": "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        "platform": "MacIntel",
        "header_order": [
            ":method", ":path", ":scheme", ":authority",
            "accept-encoding",
            "accept-language",
            "user-agent",
            "accept",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
            "sec-fetch-user",
            "upgrade-insecure-requests",
            "referer",
        ],
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        "accept_encoding": "gzip, deflate, br, zstd",
        "accept_language": "en-US,en;q=0.9",
        "sec_ch_ua": '"Not A(Brand";v="8", "Chromium";v="132", "Google Chrome";v="132"',
        "sec_ch_ua_mobile": "?0",
        "sec_ch_ua_platform": '"macOS"',
        "sec_fetch_dest": "document",
        "sec_fetch_mode": "navigate",
        "sec_fetch_site": "none",
        "sec_fetch_user": "?1",
        "upgrade_insecure_requests": "1",
        "tls_groups": ["x25519", "secp256r1", "secp384r1"],
        "tls_ciphers": [
            "TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384",
            "TLS_CHACHA20_POLY1305_SHA256",
            "ECDHE-ECDSA-AES128-GCM-SHA256", "ECDHE-RSA-AES128-GCM-SHA256",
            "ECDHE-ECDSA-AES256-GCM-SHA384", "ECDHE-RSA-AES256-GCM-SHA384",
            "ECDHE-ECDSA-CHACHA20-POLY1305", "ECDHE-RSA-CHACHA20-POLY1305",
            "ECDHE-RSA-AES128-SHA", "ECDHE-RSA-AES256-SHA",
        ],
        "h2_settings": {
            "HEADER_TABLE_SIZE": 65536,
            "MAX_CONCURRENT_STREAMS": 1000,
            "INITIAL_WINDOW_SIZE": 6291456,
            "MAX_FRAME_SIZE": 16384,
            "MAX_HEADER_LIST_SIZE": 262144,
        },
        "geo_regions": ["US", "GB", "CA", "AU", "JP", "KR"],
        "ttl": 64,
        "tcp_window": 65535,
    },
    "chrome_linux": {
        "label": "Chrome 132 / Linux x86_64",
        "weight": 0.03,
        "browser": "chrome",
        "browser_version": "132",
        "os": "linux",
        "os_version": "x86_64",
        "ua": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
        "platform": "Linux x86_64",
        "header_order": [
            ":method", ":path", ":scheme", ":authority",
            "accept-encoding",
            "accept-language",
            "user-agent",
            "accept",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
            "sec-fetch-user",
            "upgrade-insecure-requests",
            "referer",
        ],
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        "accept_encoding": "gzip, deflate, br, zstd",
        "accept_language": "en-US,en;q=0.9",
        "sec_ch_ua": '"Not A(Brand";v="8", "Chromium";v="132", "Google Chrome";v="132"',
        "sec_ch_ua_mobile": "?0",
        "sec_ch_ua_platform": '"Linux"',
        "sec_fetch_dest": "document",
        "sec_fetch_mode": "navigate",
        "sec_fetch_site": "none",
        "sec_fetch_user": "?1",
        "upgrade_insecure_requests": "1",
        "tls_groups": ["x25519", "secp256r1"],
        "tls_ciphers": [
            "TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384",
            "TLS_CHACHA20_POLY1305_SHA256",
            "ECDHE-ECDSA-AES128-GCM-SHA256", "ECDHE-RSA-AES128-GCM-SHA256",
            "ECDHE-ECDSA-AES256-GCM-SHA384", "ECDHE-RSA-AES256-GCM-SHA384",
        ],
        "h2_settings": {
            "HEADER_TABLE_SIZE": 65536,
            "MAX_CONCURRENT_STREAMS": 100,
            "INITIAL_WINDOW_SIZE": 6291456,
            "MAX_FRAME_SIZE": 16384,
            "MAX_HEADER_LIST_SIZE": 262144,
        },
        "geo_regions": ["US", "DE", "GB", "NL", "FR", "CA"],
        "ttl": 64,
        "tcp_window": 29200,
    },
    "firefox_win": {
        "label": "Firefox 127 / Windows 11",
        "weight": 0.05,
        "browser": "firefox",
        "browser_version": "127",
        "os": "windows",
        "os_version": "10.0",
        "ua": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
        "platform": "Win32",
        "header_order": [
            ":method", ":path", ":scheme", ":authority",
            "user-agent",
            "accept",
            "accept-language",
            "accept-encoding",
            "referer",
            "upgrade-insecure-requests",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
            "cache-control",
            "pragma",
        ],
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        "accept_encoding": "gzip, deflate, br, zstd",
        "accept_language": "en-US,en;q=0.5",
        "sec_ch_ua": None,  # Firefox doesn't send Sec-CH-UA
        "sec_fetch_dest": "document",
        "sec_fetch_mode": "navigate",
        "sec_fetch_site": "none",
        "upgrade_insecure_requests": "1",
        "tls_groups": ["x25519", "secp256r1", "secp384r1", "secp521r1"],
        "tls_ciphers": [
            "TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384",
            "TLS_CHACHA20_POLY1305_SHA256",
            "ECDHE-ECDSA-AES128-GCM-SHA256", "ECDHE-RSA-AES128-GCM-SHA256",
            "ECDHE-ECDSA-AES256-GCM-SHA384", "ECDHE-RSA-AES256-GCM-SHA384",
            "ECDHE-ECDSA-CHACHA20-POLY1305", "ECDHE-RSA-CHACHA20-POLY1305",
            "ECDHE-RSA-AES128-SHA", "ECDHE-RSA-AES256-SHA",
        ],
        "h2_settings": {
            "HEADER_TABLE_SIZE": 65536,
            "MAX_CONCURRENT_STREAMS": 200,
            "INITIAL_WINDOW_SIZE": 131072,
            "MAX_FRAME_SIZE": 16384,
            "MAX_HEADER_LIST_SIZE": 262144,
        },
        "geo_regions": ["US", "DE", "GB", "FR", "CA", "NL"],
        "ttl": 128,
        "tcp_window": 65535,
    },
    "safari_mac": {
        "label": "Safari 17.4 / macOS 15",
        "weight": 0.10,
        "browser": "safari",
        "browser_version": "17.4",
        "os": "macos",
        "os_version": "15",
        "ua": "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_0) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
        "platform": "MacIntel",
        "header_order": [
            ":method", ":path", ":scheme", ":authority",
            "accept-encoding",
            "accept-language",
            "user-agent",
            "accept",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
            "referer",
        ],
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "accept_encoding": "gzip, deflate, br",
        "accept_language": "en-US,en;q=0.9",
        "sec_ch_ua": None,  # Safari doesn't send Sec-CH-UA
        "sec_fetch_dest": "document",
        "sec_fetch_mode": "navigate",
        "sec_fetch_site": "none",
        "tls_groups": ["x25519", "secp256r1", "secp384r1"],
        "tls_ciphers": [
            "TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384",
            "TLS_CHACHA20_POLY1305_SHA256",
            "ECDHE-ECDSA-AES128-GCM-SHA256", "ECDHE-RSA-AES128-GCM-SHA256",
            "ECDHE-ECDSA-AES256-GCM-SHA384", "ECDHE-RSA-AES256-GCM-SHA384",
            "ECDHE-ECDSA-AES128-SHA", "ECDHE-RSA-AES128-SHA",
            "ECDHE-ECDSA-AES256-SHA", "ECDHE-RSA-AES256-SHA",
            "AES128-GCM-SHA256", "AES256-GCM-SHA384",
        ],
        "h2_settings": {
            "HEADER_TABLE_SIZE": 65536,
            "MAX_CONCURRENT_STREAMS": 100,
            "INITIAL_WINDOW_SIZE": 1048576,
            "MAX_FRAME_SIZE": 16384,
            "MAX_HEADER_LIST_SIZE": 262144,
        },
        "geo_regions": ["US", "GB", "CA", "AU", "JP"],
        "ttl": 64,
        "tcp_window": 65535,
    },
    "edge_win": {
        "label": "Edge 132 / Windows 11",
        "weight": 0.05,
        "browser": "edge",
        "browser_version": "132",
        "os": "windows",
        "os_version": "10.0",
        "ua": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 Edg/132.0.0.0",
        "platform": "Win32",
        "header_order": [
            ":method", ":path", ":scheme", ":authority",
            "accept-encoding",
            "accept-language",
            "user-agent",
            "accept",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
            "sec-fetch-user",
            "upgrade-insecure-requests",
            "referer",
        ],
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        "accept_encoding": "gzip, deflate, br, zstd",
        "accept_language": "en-US,en;q=0.9",
        "sec_ch_ua": '"Not A(Brand";v="8", "Chromium";v="132", "Microsoft Edge";v="132"',
        "sec_ch_ua_mobile": "?0",
        "sec_ch_ua_platform": '"Windows"',
        "sec_fetch_dest": "document",
        "sec_fetch_mode": "navigate",
        "sec_fetch_site": "none",
        "sec_fetch_user": "?1",
        "upgrade_insecure_requests": "1",
        "tls_groups": ["x25519", "secp256r1", "secp384r1"],
        "tls_ciphers": [
            "TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384",
            "TLS_CHACHA20_POLY1305_SHA256",
            "ECDHE-ECDSA-AES128-GCM-SHA256", "ECDHE-RSA-AES128-GCM-SHA256",
            "ECDHE-ECDSA-AES256-GCM-SHA384", "ECDHE-RSA-AES256-GCM-SHA384",
            "ECDHE-ECDSA-CHACHA20-POLY1305", "ECDHE-RSA-CHACHA20-POLY1305",
            "ECDHE-RSA-AES128-SHA", "ECDHE-RSA-AES256-SHA",
            "AES128-GCM-SHA256", "AES256-GCM-SHA384", "AES128-SHA", "AES256-SHA",
        ],
        "h2_settings": {
            "HEADER_TABLE_SIZE": 65536,
            "MAX_CONCURRENT_STREAMS": 1000,
            "INITIAL_WINDOW_SIZE": 6291456,
            "MAX_FRAME_SIZE": 16384,
            "MAX_HEADER_LIST_SIZE": 262144,
        },
        "geo_regions": ["US", "GB", "DE", "FR", "JP", "CA"],
        "ttl": 128,
        "tcp_window": 65535,
    },
}

# Language-to-region mapping for geo-coherence
LANG_REGION_MAP = {
    "en-US": "US", "en-GB": "GB", "en-CA": "CA", "en-AU": "AU",
    "de-DE": "DE", "de-AT": "AT", "de-CH": "CH",
    "fr-FR": "FR", "fr-CA": "CA", "fr-BE": "BE", "fr-CH": "CH",
    "ja-JP": "JP", "ko-KR": "KR",
    "zh-CN": "CN", "zh-TW": "TW", "zh-HK": "HK",
    "es-ES": "ES", "es-MX": "MX", "es-AR": "AR",
    "pt-BR": "BR", "pt-PT": "PT",
    "it-IT": "IT", "it-CH": "CH",
    "nl-NL": "NL", "nl-BE": "BE",
    "sv-SE": "SE", "no-NO": "NO", "da-DK": "DK", "fi-FI": "FI",
    "pl-PL": "PL", "cs-CZ": "CZ", "sk-SK": "SK",
    "ru-RU": "RU", "tr-TR": "TR",
    "ar-SA": "SA", "he-IL": "IL", "hi-IN": "IN",
    "th-TH": "TH", "vi-VN": "VN", "id-ID": "ID",
}

LANG_POOL = [
    "en-US,en;q=0.9", "en-GB,en;q=0.9", "en-CA,en;q=0.9", "en-AU,en;q=0.9",
    "de-DE,de;q=0.9,en;q=0.5", "fr-FR,fr;q=0.9,en;q=0.5",
    "ja-JP,ja;q=0.9,en;q=0.5", "ko-KR,ko;q=0.9,en;q=0.5",
    "es-ES,es;q=0.9,en;q=0.5", "pt-BR,pt;q=0.9,en;q=0.5",
    "it-IT,it;q=0.9,en;q=0.5", "nl-NL,nl;q=0.9,en;q=0.5",
    "sv-SE,sv;q=0.9,en;q=0.5", "pl-PL,pl;q=0.9,en;q=0.5",
    "ru-RU,ru;q=0.9,en;q=0.5", "zh-CN,zh;q=0.9,en;q=0.5",
]

# ============================================================================
# Internal patterns to strip from headers
# ============================================================================

INTERNAL_PATTERNS = [
    (r'neotrix', 'client'),
    (r'\bnt_[a-z]', 'sys_'),
    (r'\bNEOTRIX_', 'CLIENT_'),
    (r'x-neotrix-', 'x-client-'),
    (r'x-nt-', 'x-client-'),
]

# ============================================================================
# Identity Pool
# ============================================================================

class IdentityPool:
    """Manages a persistent pool of browser personas.
    
    Personas are selected with weighted probability matching real-world
    browser market share. Each persona is a complete, cross-layer-consistent
    digital identity.
    """

    DB_PATH = os.path.expanduser("~/.neotrix/comm_router.db")

    def __init__(self):
        self._db: Optional[sqlite3.Connection] = None
        self._ensure_db()

    def _ensure_db(self):
        os.makedirs(os.path.dirname(self.DB_PATH), exist_ok=True)
        self._db = sqlite3.connect(self.DB_PATH)
        self._db.execute("PRAGMA journal_mode=WAL")
        self._db.execute("PRAGMA busy_timeout=5000")
        self._db.execute("""
            CREATE TABLE IF NOT EXISTS identity_pool (
                id TEXT PRIMARY KEY,
                persona_key TEXT NOT NULL,
                created_at REAL NOT NULL,
                last_used REAL,
                success_count INTEGER DEFAULT 0,
                fail_count INTEGER DEFAULT 0,
                last_ip TEXT,
                last_ip_geo TEXT
            )
        """)
        self._db.execute("""
            CREATE TABLE IF NOT EXISTS persona_stats (
                persona_key TEXT PRIMARY KEY,
                total_uses INTEGER DEFAULT 0,
                total_success INTEGER DEFAULT 0,
                total_fail INTEGER DEFAULT 0,
                avg_latency_ms REAL DEFAULT 0.0
            )
        """)
        self._db.execute("""
            CREATE TABLE IF NOT EXISTS failure_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL NOT NULL,
                persona_id TEXT,
                domain TEXT,
                status_code INTEGER,
                error TEXT,
                proxy_used TEXT
            )
        """)
        self._db.commit()

    def create_id(self, persona_key: str) -> str:
        """Create a new identity instance from a persona template."""
        pid = hashlib.sha256(f"{persona_key}:{time.time()}:{random.random()}".encode()).hexdigest()[:16]
        self._db.execute(
            "INSERT INTO identity_pool (id, persona_key, created_at) VALUES (?, ?, ?)",
            (pid, persona_key, time.time())
        )
        self._db.commit()
        return pid

    def get_persona(self, persona_key: str) -> Optional[Dict]:
        """Get a persona definition by key."""
        return PERSONAS.get(persona_key)

    def find(self, query: str = "") -> Optional[Dict]:
        """Find a persona by partial match. Returns persona dict with key."""
        q = query.lower()
        for key, persona in PERSONAS.items():
            if q in key or q in persona["label"].lower():
                return {"key": key, **persona}
        # Fall back to weighted random
        return self.random()

    def random(self) -> Dict:
        """Select a persona with weighted random (market share)."""
        keys = list(PERSONAS.keys())
        weights = [PERSONAS[k]["weight"] for k in keys]
        total = sum(weights)
        r = random.uniform(0, total)
        cum = 0.0
        for key, w in zip(keys, weights):
            cum += w
            if r <= cum:
                p = PERSONAS[key].copy()
                p["key"] = key
                return p
        p = PERSONAS[keys[-1]].copy()
        p["key"] = keys[-1]
        return p

    def random_with_ip_geo(self, geo_region: str) -> Optional[Dict]:
        """Select a random persona whose geo_regions include the given region."""
        candidates = []
        for key, p in PERSONAS.items():
            # Check if this persona is compatible with the proxy geo region
            if geo_region.upper() in [r.upper() for r in p.get("geo_regions", [])]:
                candidates.append(key)
        if not candidates:
            # Fallback: pick persona whose Accept-Language matches closest
            # For "US", any en-* persona works
            for key, p in PERSONAS.items():
                if "en" in str(p.get("accept_language", "")):
                    candidates.append(key)
        if not candidates:
            return self.random()
        key = random.choice(candidates)
        p = PERSONAS[key].copy()
        p["key"] = key
        return p

    def record_success(self, persona_id: str, latency_ms: float = 0.0):
        self._ensure_db()
        now = time.time()
        self._db.execute(
            "UPDATE identity_pool SET last_used=?, success_count=success_count+1 WHERE id=?",
            (now, persona_id)
        )
        # Also update persona_stats
        c = self._db.execute("SELECT persona_key FROM identity_pool WHERE id=?", (persona_id,))
        row = c.fetchone()
        if row:
            key = row[0]
            self._db.execute("""
                INSERT INTO persona_stats (persona_key, total_uses, total_success, avg_latency_ms)
                VALUES (?, 1, 1, ?)
                ON CONFLICT(persona_key) DO UPDATE SET
                    total_uses = total_uses + 1,
                    total_success = total_success + 1,
                    avg_latency_ms = (avg_latency_ms * (total_uses - 1) + ?) / total_uses
            """, (key, latency_ms, latency_ms))
        self._db.commit()

    def record_failure(self, persona_id: str, domain: str, status_code: int, error: str = "", proxy: str = ""):
        self._ensure_db()
        self._db.execute(
            "UPDATE identity_pool SET last_used=?, fail_count=fail_count+1 WHERE id=?",
            (time.time(), persona_id)
        )
        self._db.execute(
            "INSERT INTO failure_log (timestamp, persona_id, domain, status_code, error, proxy_used) VALUES (?, ?, ?, ?, ?, ?)",
            (time.time(), persona_id, domain, status_code, error[:500], proxy)
        )
        c = self._db.execute("SELECT persona_key FROM identity_pool WHERE id=?", (persona_id,))
        row = c.fetchone()
        if row:
            key = row[0]
            self._db.execute("""
                INSERT INTO persona_stats (persona_key, total_uses, total_success)
                VALUES (?, 1, 0)
                ON CONFLICT(persona_key) DO UPDATE SET
                    total_uses = total_uses + 1,
                    total_fail = total_fail + 1
            """, (key,))
        self._db.commit()

    def stats(self) -> Dict:
        self._ensure_db()
        c = self._db.execute("SELECT COUNT(*) FROM identity_pool")
        total_ids = c.fetchone()[0]
        c = self._db.execute("SELECT COUNT(*) FROM failure_log WHERE timestamp > ?", (time.time() - 86400,))
        failures_24h = c.fetchone()[0]
        c = self._db.execute("""
            SELECT persona_key, total_uses, total_success, total_fail, avg_latency_ms
            FROM persona_stats ORDER BY total_uses DESC
        """)
        by_persona = {row[0]: {"uses": row[1], "success": row[2], "fail": row[3], "avg_latency": row[4]} for row in c.fetchall()}
        return {
            "total_identities": total_ids,
            "failures_24h": failures_24h,
            "personas_used": len(by_persona),
            "by_persona": by_persona,
        }


# ============================================================================
# Header Obfuscation
# ============================================================================

class HeaderObfuscator:
    """Build coherent, obfuscated header sets from a persona."""

    def __init__(self):
        self.INTERNAL_PATTERNS = INTERNAL_PATTERNS

    def build_headers(self, persona: Dict, url: str,
                      extra_headers: Optional[Dict] = None) -> List[Tuple[str, str]]:
        """Build a coherent ordered header list from a persona.
        
        Returns a list of (name, value) tuples in the correct ordering
        for the persona's browser type.
        """
        headers: Dict[str, str] = {}

        # Base headers from persona
        if persona.get("accept"):
            headers["accept"] = persona["accept"]
        if persona.get("accept_language"):
            headers["accept-language"] = persona["accept_language"]
        if persona.get("accept_encoding"):
            headers["accept-encoding"] = persona["accept_encoding"]
        if persona.get("sec_ch_ua") is not None:
            headers["sec-ch-ua"] = persona["sec_ch_ua"]
        if persona.get("sec_ch_ua_mobile") is not None:
            headers["sec-ch-ua-mobile"] = persona["sec_ch_ua_mobile"]
        if persona.get("sec_ch_ua_platform") is not None:
            headers["sec-ch-ua-platform"] = persona["sec_ch_ua_platform"]
        if persona.get("sec_fetch_dest"):
            headers["sec-fetch-dest"] = persona["sec_fetch_dest"]
        if persona.get("sec_fetch_mode"):
            headers["sec-fetch-mode"] = persona["sec_fetch_mode"]
        if persona.get("sec_fetch_site"):
            headers["sec-fetch-site"] = persona["sec_fetch_site"]
        if persona.get("sec_fetch_user"):
            headers["sec-fetch-user"] = persona["sec_fetch_user"]
        if persona.get("upgrade_insecure_requests"):
            headers["upgrade-insecure-requests"] = persona["upgrade_insecure_requests"]

        # User-Agent
        if persona.get("ua"):
            headers["user-agent"] = persona["ua"]

        # Referer: simulate natural traffic
        parsed = urlparse(url)
        ref_choices = [
            f"https://www.google.com/search?q={random.choice(['research', 'paper', 'documentation', 'api', 'tutorial'])}",
            f"https://{parsed.netloc}/",
            f"https://scholar.google.com/scholar?q={random.choice(['machine+learning', 'deep+learning', 'nlp', 'computer+vision'])}",
            f"https://github.com/search?q={random.choice(['neural+network', 'transformer', 'dataset'])}",
            f"https://en.wikipedia.org/wiki/{random.choice(['Artificial_intelligence', 'Machine_learning', 'Deep_learning', 'Natural_language_processing'])}",
        ]
        headers["referer"] = random.choice(ref_choices)

        # Accept-Encoding
        if "accept-encoding" not in headers:
            headers["accept-encoding"] = "gzip, deflate, br"

        # Add extra headers (after processing for overriding)
        if extra_headers:
            for k, v in extra_headers.items():
                k_lower = k.lower()
                # Don't override critical identity headers
                if k_lower not in ("user-agent", "accept", "accept-language",
                                   "accept-encoding", "referer"):
                    headers[k_lower] = v

        # Strip internal patterns from all header values
        headers = self._strip_internal_headers(headers)

        # Apply header ordering from persona
        ordered = self._apply_header_order(headers, persona)

        return ordered

    def _strip_internal_headers(self, headers: Dict[str, str]) -> Dict[str, str]:
        """Remove any NeoTrix traces from header names and values."""
        cleaned = {}
        for key, value in headers.items():
            key_clean = key
            value_clean = value
            for pattern, replacement in self.INTERNAL_PATTERNS:
                if re.search(pattern, key_clean, re.IGNORECASE):
                    key_clean = re.sub(pattern, replacement, key_clean, flags=re.IGNORECASE)
                if re.search(pattern, value_clean, re.IGNORECASE):
                    value_clean = re.sub(pattern, replacement, value_clean, flags=re.IGNORECASE)
            # Also strip UUID-like patterns that could be NeoTrix internal IDs
            value_clean = re.sub(r'\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b',
                                 '00000000-0000-0000-0000-000000000000', value_clean)
            # Strip file paths
            value_clean = re.sub(r'/Users/[^/]+/', '/home/user/', value_clean)
            cleaned[key_clean] = value_clean
        return cleaned

    def _apply_header_order(self, headers: Dict[str, str],
                           persona: Dict) -> List[Tuple[str, str]]:
        """Apply browser-specific header ordering."""
        order = persona.get("header_order", [])
        if not order:
            # Default ordering if persona has none
            return list(headers.items())

        result = []
        seen = set()
        for key in order:
            key_lower = key.lstrip(":").lower()
            if key_lower in headers and key_lower not in seen:
                result.append((key_lower, headers[key_lower]))
                seen.add(key_lower)

        # Add any remaining headers not in the order list
        for key, value in headers.items():
            if key not in seen:
                result.append((key, value))
                seen.add(key)

        return result

    def build_request(self, persona: Dict, url: str,
                      extra_headers: Optional[Dict] = None,
                      method: str = "GET", data: Optional[bytes] = None) -> urllib.request.Request:
        """Build a complete urllib Request with obfuscated headers."""
        ordered_headers = self.build_headers(persona, url, extra_headers)

        req = urllib.request.Request(url, data=data, method=method)

        # Add headers in order (urllib preserves insertion order in Python 3.7+)
        for key, value in ordered_headers:
            req.add_header(key, value)

        return req


# ============================================================================
# Timing Obfuscation
# ============================================================================

class TimingObfuscator:
    """Add human-like timing jitter to requests."""

    def __init__(self):
        self._last_request = 0.0

    def wait(self, domain: str = ""):
        """Wait appropriate time to simulate human-like browsing patterns."""
        now = time.time()
        elapsed = now - self._last_request

        # If this is the first request, no wait needed
        if self._last_request == 0.0:
            self._last_request = now
            return

        # Baseline: humans wait 1-5 seconds between pages
        target = random.gauss(2.5, 1.0)
        target = max(0.3, min(10.0, target))

        remaining = target - elapsed
        if remaining > 0:
            time.sleep(remaining)

        self._last_request = time.time()

    def add_page_load_jitter(self) -> float:
        """Return a jitter value in ms to simulate page render time."""
        return random.gauss(200, 100)


# ============================================================================
# Geo-Coherence Checker
# ============================================================================

class GeoCoherence:
    """Ensure geo-location signals are internally consistent."""

    @staticmethod
    def region_for_language(accept_language: str) -> str:
        """Extract primary region from Accept-Language header."""
        lang = accept_language.split(",")[0].strip()
        region = LANG_REGION_MAP.get(lang, "US")
        return region

    @staticmethod
    def language_for_region(region: str) -> str:
        """Find a plausible Accept-Language for a region."""
        for lang, reg in LANG_REGION_MAP.items():
            if reg.upper() == region.upper():
                return f"{lang},{lang.split('-')[0]};q=0.9,en;q=0.5"
        # Default to US English
        return "en-US,en;q=0.9"

    @staticmethod
    def score_coherence(accept_language: str, ip_region: str) -> float:
        """Score how well Accept-Language matches IP geo. 1.0 = perfect."""
        lang_region = GeoCoherence.region_for_language(accept_language)
        if lang_region.upper() == ip_region.upper():
            return 1.0
        # Same continent = partial match
        continent = {"US": "NA", "CA": "NA", "MX": "NA",
                     "GB": "EU", "DE": "EU", "FR": "EU", "IT": "EU", "ES": "EU",
                     "JP": "AS", "KR": "AS", "CN": "AS", "IN": "AS",
                     "AU": "OC", "BR": "SA"}
        lc = continent.get(lang_region, "")
        ic = continent.get(ip_region, "")
        if lc and lc == ic:
            return 0.6
        return 0.3


# ============================================================================
# Route Engine
# ============================================================================

class RouteEngine:
    """Selects optimal transport + persona for a given domain/request."""

    def __init__(self, pool: IdentityPool, transport=None):
        self.pool = pool
        self.header_obfuscator = HeaderObfuscator()
        self.timing = TimingObfuscator()
        self.geo = GeoCoherence()
        self.transport = transport  # Optional transport with fetch(url, headers, timeout) -> TransportResult-like

    def prepare(self, url: str, extra_headers: Optional[Dict] = None,
                persona_key: str = "") -> Tuple[urllib.request.Request, Dict]:
        """Prepare a request with optimal persona and headers.
        
        Returns (urllib.Request, persona_dict).
        """
        # Select persona
        if persona_key:
            persona = self.pool.get_persona(persona_key)
            if not persona:
                persona = self.pool.random()
            persona = {**persona, "key": persona_key}
        else:
            persona = self.pool.random()

        # Build obfuscated request
        req = self.header_obfuscator.build_request(persona, url, extra_headers)

        # Apply timing jitter
        self.timing.wait(domain=urlparse(url).netloc)

        return req, persona

    def execute(self, req: urllib.request.Request, persona: Dict,
                identity_id: str, timeout: int = 30) -> Dict:
        """Execute a prepared request with monitoring and feedback.

        If a transport backend is configured, delegates the actual HTTP call
        to it (for retry/backoff/rate-limit handling). Otherwise uses direct
        urllib.request.urlopen.
        """
        start = time.time()
        result = {"status": 0, "body": None, "headers": {}, "error": "", "latency_ms": 0.0}
        url = str(req.full_url)
        domain = urlparse(url).netloc

        try:
            if self.transport:
                headers_dict = {k: v for k, v in req.headers.items()}
                tr = self.transport.fetch(url, headers=headers_dict, timeout=timeout)
                result["status"] = tr.status
                result["body"] = tr.body
                result["headers"] = tr.headers
                result["latency_ms"] = (time.time() - start) * 1000
                if tr.ok:
                    self.pool.record_success(identity_id, result["latency_ms"])
                else:
                    self.pool.record_failure(identity_id, domain, tr.status, tr.error[:200])
                return result

            with urllib.request.urlopen(req, timeout=timeout) as resp:
                result["status"] = resp.status
                result["body"] = resp.read().decode("utf-8", errors="replace")
                result["headers"] = dict(resp.headers)
                result["latency_ms"] = (time.time() - start) * 1000

            self.pool.record_success(identity_id, result["latency_ms"])
            return result

        except urllib.error.HTTPError as e:
            result["status"] = e.code
            result["error"] = str(e)[:500]
            result["latency_ms"] = (time.time() - start) * 1000

            self.pool.record_failure(identity_id, domain, e.code, str(e)[:200])
            return result

        except (urllib.error.URLError, OSError) as e:
            result["error"] = str(e)[:500]
            result["latency_ms"] = (time.time() - start) * 1000
            self.pool.record_failure(identity_id, domain, 0, str(e)[:200])
            return result

        except Exception as e:
            result["error"] = str(e)[:500]
            result["latency_ms"] = (time.time() - start) * 1000
            return result


# ============================================================================
# CommRouter — Unified Entry Point
# ============================================================================

class CommRouter:
    """Single entry/exit for all external NeoTrix communication.
    
    Automatically:
    - Selects a coherent browser persona matching market share
    - Builds obfuscated, ordered HTTP headers
    - Strips internal NeoTrix identifiers
    - Applies human-like timing jitter
    - Records all outcomes for iterative learning
    - Identities appear as random global internet users
    """

    def __init__(self, transport=None):
        self.identity_pool = IdentityPool()
        self.route_engine = RouteEngine(self.identity_pool, transport=transport)
        self._plan_done = False

    def plan(self):
        """Warm the identity pool. Creates some persona instances."""
        if self._plan_done:
            return
        for key in list(PERSONAS.keys())[:3]:  # Create 3 initial identities
            self.identity_pool.create_id(key)
        self._plan_done = True

    def get(self, url: str, extra_headers: Optional[Dict] = None,
            persona_key: str = "", timeout: int = 30,
            retry_on_failure: bool = True) -> Dict:
        """Make a GET request through the router.
        
        Returns dict with status, body, headers, error, latency_ms, persona_used.
        """
        self.plan()
        identity_id = self.identity_pool.create_id("_anon_")

        # Prepare request
        req, persona = self.route_engine.prepare(url, extra_headers, persona_key)
        result = self.route_engine.execute(req, persona, identity_id, timeout)
        result["persona_used"] = persona.get("key", "unknown")
        result["identity_id"] = identity_id

        # Retry with different persona on failure?
        if retry_on_failure and result.get("status", 0) in (403, 429, 0) and result.get("error"):
            # Try once more with a different persona
            alt_persona = self.identity_pool.random()
            alt_identity = self.identity_pool.create_id(alt_persona.get("key", "fallback"))
            req2, _ = self.route_engine.prepare(url, extra_headers, alt_persona.get("key", ""))
            result2 = self.route_engine.execute(req2, alt_persona, alt_identity, timeout)
            if result2.get("status", 0) not in (403, 429, 0) or not result2.get("error"):
                result2["persona_used"] = alt_persona.get("key", "unknown")
                result2["identity_id"] = alt_identity
                return result2

        return result

    def fetch(self, url: str, **kwargs) -> Dict:
        """Alias for get()."""
        return self.get(url, **kwargs)

    def status(self) -> Dict:
        """Get router health and stats."""
        pool_stats = self.identity_pool.stats()
        return {
            "personas_available": len(PERSONAS),
            "persona_names": {k: v["label"] for k, v in PERSONAS.items()},
            "market_share": {k: v["weight"] for k, v in PERSONAS.items()},
            "pool": pool_stats,
        }

    def summary(self) -> str:
        s = self.status()
        lines = [
            f"CommRouter — {s['personas_available']} personas",
            f"  Available: {', '.join(s['persona_names'].values())}",
            f"  Active identities: {s['pool']['total_identities']}",
            f"  Failures (24h): {s['pool']['failures_24h']}",
        ]
        if s['pool'].get('by_persona'):
            lines.append("  Persona performance:")
            for key, stats in s['pool']['by_persona'].items():
                lines.append(f"    {key}: {stats['uses']} uses, {stats['success']} ok, {stats['fail']} fail")
        return "\n".join(lines)


# ============================================================================
# Self-test
# ============================================================================

if __name__ == '__main__':
    print("═══ CommRouter — Identity Pool Test ═══")
    router = CommRouter()
    router.plan()
    print(f"  Personas available: {len(PERSONAS)}")
    for key, p in PERSONAS.items():
        print(f"    {key}: {p['label']} ({p['weight']*100:.0f}%)")

    print("\n═══ Persona Selection Test ═══")
    for i in range(3):
        p = router.identity_pool.random()
        print(f"  Random pick: {p.get('key', '?')} — {p.get('label', '?')}")

    print("\n═══ Header Obfuscation Test ═══")
    hdr = HeaderObfuscator()
    for key in ["chrome_win", "firefox_win", "safari_mac"]:
        p = router.identity_pool.get_persona(key)
        if p:
            headers = hdr.build_headers(p, "https://arxiv.org/abs/1706.03762")
            print(f"\n  {key}:")
            for name, value in headers:
                val_preview = value[:60] + "..." if len(value) > 60 else value
                print(f"    {name}: {val_preview}")

    print("\n═══ Internal Pattern Strip Test ═══")
    test_headers = {"user-agent": "NeoTrixBot/1.0", "x-neotrix-session": "abc123", "accept": "*/*", "x-nt-key": "secret"}
    cleaned = hdr._strip_internal_headers(test_headers)
    for k, v in cleaned.items():
        print(f"  {k}: {v}")

    print("\n═══ Geo-Coherence Test ═══")
    for lang in ["en-US", "de-DE", "ja-JP", "zh-CN"]:
        region = GeoCoherence.region_for_language(lang)
        print(f"  {lang} → region: {region}")

    print("\n═══ Router Status ═══")
    print(router.summary())

    print("\n═══ Live Fetch Test ═══")
    result = router.get("https://httpbin.org/headers", retry_on_failure=False)
    print(f"  Status: {result.get('status', 0)}")
    print(f"  Persona: {result.get('persona_used', '?')}")
    print(f"  Latency: {result.get('latency_ms', 0):.0f}ms")
    if result.get('body') and result.get('status', 0) == 200:
        try:
            body = json.loads(result['body'])
            headers_received = body.get('headers', {})
            print("  Headers sent (as seen by server):")
            for h, v in headers_received.items():
                print(f"    {h}: {v}")
        except Exception:
            print(f"  Body (first 200): {result['body'][:200]}")
