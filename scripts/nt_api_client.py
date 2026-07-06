"""
nt_api_client — Unified API access pipeline for NeoTrix
========================================================
Pluggable transport backends + per-domain fallback strategies + capability probing.

Architecture:
    AccessPipeline
    ├── probe_capabilities() → capability_map (cache: ~/.neotrix/capability_cache.json)
    ├── Domain: arxiv.org
    │   ├── ArXivDirectAPI (primary) ✓ verified
    │   ├── ArXivHTMLScrape (fallback) ✓ verified
    │   └── ArXivViaProxy (if Tor available)
    ├── Domain: api.github.com
    │   ├── GitHubTokenAPI (if NEOTRIX_GITHUB_TOKEN set) → 5000/hr
    │   ├── GitHubNoTokenAPI → 60/hr
    │   └── GitHubCDN (raw.githubusercontent.com) → unlimited
    ├── Domain: en.wikipedia.org
    │   ├── WikipediaRESTAPI (unlimited)
    │   └── WikipediaMediaWikiAPI (unlimited)
    ├── Transport backends (probing, auto-fallback)
    │   ├── DirectConnection (always, UA rotation)
    │   ├── TorConnection (if available)
    │   └── ProxyConnection (from free pool)
    ├── RateLimitTracker (per-domain, sliding window)
    └── StrategySelector (capability → strategy routing)

Usage:
    pipeline = AccessPipeline()
    pipeline.probe()  # one-time capability probing
    
    # ArXiv
    meta = pipeline.fetch_arxiv('1706.03762')
    
    # GitHub  
    meta = pipeline.fetch_github('neotrix/neotrix')
    readme = pipeline.fetch_github_readme('neotrix/neotrix')
"""

import urllib.request, urllib.error, urllib.parse, json, time, random, socket, re, os
import xml.etree.ElementTree as ET
import html as html_module
import requests
from typing import Optional, Dict, List, Tuple, Set
from urllib.parse import urlparse

# ============================================================================
# User-Agent Rotator
# ============================================================================

UA_POOL = [
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36',
    'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
    'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:120.0) Gecko/20100101 Firefox/120.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:123.0) Gecko/20100101 Firefox/123.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:124.0) Gecko/20100101 Firefox/124.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:125.0) Gecko/20100101 Firefox/125.0',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/124.0',
    'Mozilla/5.0 (X11; Linux i686; rv:124.0) Gecko/20100101 Firefox/124.0',
    'Mozilla/5.0 (X11; Linux x86_64; rv:125.0) Gecko/20100101 Firefox/125.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3 Safari/605.1.15',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 Edg/123.0.0.0',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1',
    'Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.6367.82 Mobile Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 OPR/108.0.0.0',
    'curl/8.4.0', 'Wget/1.21.4',
    'Mozilla/5.0 (compatible; SemanticScholarBot/1.0; +http://semanticscholar.org/bot)',
    'Mozilla/5.0 (compatible; arXivBot/1.0; +https://arxiv.org/bot)',
]

class UARotator:
    def __init__(self):
        self._pool = UA_POOL.copy()
        random.shuffle(self._pool)
        self._idx = 0

    def get(self) -> str:
        ua = self._pool[self._idx]
        self._idx = (self._idx + 1) % len(self._pool)
        return ua

# ============================================================================
# Transport Backends (pluggable, auto-probed)
# ============================================================================

class TransportResult:
    def __init__(self, body: Optional[str] = None, status: int = 200,
                 headers: Optional[Dict] = None, error: str = ""):
        self.body = body
        self.status = status
        self.headers = headers or {}
        self.error = error

    @property
    def ok(self) -> bool:
        return self.body is not None and 200 <= self.status < 400


class DirectTransport:
    """Direct HTTP connection with UA rotation. Always available."""

    def __init__(self):
        self.ua = UARotator()

    def fetch(self, url: str, headers: Optional[Dict] = None,
              timeout: int = 30, max_retries: int = 3) -> TransportResult:
        req_headers = {'User-Agent': self.ua.get()}
        if headers:
            req_headers.update(headers)

        for attempt in range(max_retries):
            try:
                req_headers['User-Agent'] = self.ua.get()
                req = urllib.request.Request(url, headers=req_headers)
                with urllib.request.urlopen(req, timeout=timeout) as resp:
                    body = resp.read().decode('utf-8', errors='replace')
                    return TransportResult(body, resp.status, dict(resp.headers))
            except urllib.error.HTTPError as e:
                body = e.read().decode('utf-8', errors='replace')[:500]
                if e.code in (429, 503, 502) and attempt < max_retries - 1:
                    delay = 2.0 * (2 ** attempt) + random.uniform(0.5, 2)
                    time.sleep(delay)
                    continue
                return TransportResult(body=None, status=e.code, error=body[:200])
            except (urllib.error.URLError, socket.timeout) as e:
                if attempt < max_retries - 1:
                    time.sleep(2.0 * (2 ** attempt))
                    continue
                return TransportResult(body=None, status=0, error=str(e)[:200])
            except Exception as e:
                return TransportResult(body=None, status=0, error=str(e)[:200])

        return TransportResult(body=None, status=0, error="max retries")


class TorTransport:
    """SOCKS5 via Tor. Probed on startup, used only if available."""

    def __init__(self, host='127.0.0.1', port=9050):
        self.host = host
        self.port = port
        self.available = self._probe()
        self._request_count = 0
        self._circuit_count = 0

    def _probe(self) -> bool:
        try:
            s = socket.socket()
            s.settimeout(2)
            s.connect((self.host, self.port))
            s.close()
            return True
        except Exception:
            return False

    def fetch(self, url: str, headers: Optional[Dict] = None,
              timeout: int = 30) -> TransportResult:
        if not self.available:
            return TransportResult(body=None, status=0, error="Tor unavailable")
        try:
            import socks
            s = socks.socksocket()
            s.set_proxy(socks.SOCKS5, self.host, self.port)
            s.settimeout(timeout)
            parsed = urlparse(url)
            port = 443 if parsed.scheme == 'https' else 80
            s.connect((parsed.hostname, port))

            if parsed.scheme == 'https':
                import ssl
                ctx = ssl.create_default_context()
                s = ctx.wrap_socket(s, server_hostname=parsed.hostname)

            path = parsed.path or '/'
            if parsed.query:
                path += '?' + parsed.query
            ua = UA_POOL[random.randint(0, len(UA_POOL)-1)]
            req = (f"GET {path} HTTP/1.1\r\n"
                   f"Host: {parsed.hostname}\r\n"
                   f"User-Agent: {ua}\r\n"
                   f"Accept: */*\r\n"
                   f"Connection: close\r\n\r\n")
            s.sendall(req.encode())
            resp = b''
            while True:
                chunk = s.recv(4096)
                if not chunk:
                    break
                resp += chunk

            header_end = resp.find(b'\r\n\r\n')
            if header_end == -1:
                return TransportResult(body=None, status=0, error="no headers")

            status_line = resp[:resp.find(b'\r\n')].decode(errors='replace')
            status_code = int(status_line.split(' ')[1]) if ' ' in status_line else 0
            body = resp[header_end+4:].decode('utf-8', errors='replace')

            self._request_count += 1
            return TransportResult(body, status_code, {}, "")

        except Exception as e:
            return TransportResult(body=None, status=0, error=str(e)[:200])


class ProxyPool:
    """Manages a pool of free HTTP/HTTPS proxies. Self-discovering and validating.

    Scrapes free proxy list sources, validates connectivity, and provides
    random proxy selection for ProxyTransport. Pure stdlib — no external deps.
    """

    SOURCES = [
        "https://free-proxy-list.net/",
        "https://www.sslproxies.org/",
        "https://api.proxyscrape.com/v2/?request=getproxies&protocol=http&timeout=10000&country=all",
    ]

    VALIDATE_URL = "http://httpbin.org/ip"
    CACHE_SECONDS = 300

    def __init__(self):
        self._pool: List[Dict] = []
        self._last_refresh = 0.0
        self._stats = {'total': 0, 'valid': 0, 'failed': 0, 'avg_latency_ms': 0.0}
        self._ua = UARotator()
        self._env_proxy = os.environ.get("NEOTRIX_PROXY_URL", "")
        if self._env_proxy:
            self._pool.append({
                'proxy': self._env_proxy,
                'latency': 0,
                'last_validated': time.time(),
                'valid': True,
                '_env': True,
            })
            self._stats['total'] = 1
            self._stats['valid'] = 1

    def discover(self, timeout: int = 10) -> List[str]:
        """Scrape free proxy list sources for raw 'ip:port' strings."""
        candidates: Set[str] = set()
        for url in self.SOURCES:
            try:
                req = urllib.request.Request(url, headers={'User-Agent': self._ua.get()})
                with urllib.request.urlopen(req, timeout=timeout) as resp:
                    html = resp.read().decode('utf-8', errors='replace')
                extracted = self._extract_proxies(html, url)
                candidates.update(extracted)
            except Exception:
                continue
        return list(candidates)

    def _extract_proxies(self, text: str, source_url: str = "") -> Set[str]:
        """Extract unique ip:port strings from HTML or plain text.

        Handles three formats:
        - Table rows: <tr><td>IP</td><td>PORT</td>...
        - Plain ip:port per line (proxyscrape API)
        - Inline ip:port anywhere in text
        """
        # Table format used by free-proxy-list.net & sslproxies.org
        table_matches = re.findall(
            r'<tr[^>]*>\s*<td[^>]*>(\d+\.\d+\.\d+\.\d+)</td>\s*<td[^>]*>(\d+)</td>',
            text, re.IGNORECASE
        )
        if table_matches:
            return {f"{ip}:{port}" for ip, port in table_matches}

        # Plain ip:port per line (proxyscrape API response)
        line_matches = re.findall(r'^(\d+\.\d+\.\d+\.\d+):(\d+)$', text, re.MULTILINE)
        if line_matches:
            return {f"{ip}:{port}" for ip, port in line_matches}

        # Fallback: inline ip:port anywhere in text
        inline_matches = re.findall(r'(?<!\d)(\d+\.\d+\.\d+\.\d+):(\d+)(?!\d)', text)
        return {f"{ip}:{port}" for ip, port in inline_matches}

    def validate(self, proxy: str, timeout: int = 5) -> Tuple[bool, float]:
        """Test a proxy by fetching a known endpoint. Returns (ok, latency_ms)."""
        start = time.time()
        try:
            handler = urllib.request.ProxyHandler({
                'http': f'http://{proxy}',
                'https': f'http://{proxy}',
            })
            auth_handler = urllib.request.ProxyBasicAuthHandler()
            opener = urllib.request.build_opener(handler, auth_handler)
            req = urllib.request.Request(
                self.VALIDATE_URL,
                headers={'User-Agent': self._ua.get()},
            )
            with opener.open(req, timeout=timeout) as resp:
                resp.read()
                latency = (time.time() - start) * 1000
                return True, latency
        except Exception:
            return False, 0.0

    def refresh(self, max_valid: int = 10) -> int:
        """Re-discover and re-validate proxies. Cached for CACHE_SECONDS.

        Samples a random subset of candidates to avoid spending minutes
        testing dead proxies. Stops early once max_valid are found.
        """
        now = time.time()
        if now - self._last_refresh < self.CACHE_SECONDS:
            return len([p for p in self._pool if p.get('valid')])

        self._last_refresh = now
        candidates = self.discover()
        self._stats['total'] = len(candidates)

        if not candidates:
            self._pool = []
            self._stats['valid'] = 0
            self._stats['failed'] = 0
            return 0

        random.shuffle(candidates)
        max_attempts = min(len(candidates), max_valid * 4 + 10)
        val_timeout = 3

        valid_pool = []
        latencies = []
        failures = 0
        for proxy in candidates[:max_attempts]:
            if len(valid_pool) >= max_valid:
                break
            ok, latency = self.validate(proxy, timeout=val_timeout)
            if ok:
                valid_pool.append({
                    'proxy': f'http://{proxy}',
                    'latency': latency,
                    'last_validated': now,
                    'valid': True,
                })
                latencies.append(latency)
            else:
                failures += 1

        self._pool = valid_pool
        if self._env_proxy:
            self._pool.insert(0, {
                'proxy': self._env_proxy,
                'latency': 0,
                'last_validated': now,
                'valid': True,
                '_env': True,
            })
        self._stats['valid'] = len([p for p in self._pool if p.get('valid')])
        self._stats['failed'] = failures
        self._stats['avg_latency_ms'] = round(sum(latencies) / len(latencies), 1) if latencies else 0.0
        return self._stats['valid']

    def get(self) -> Optional[str]:
        """Return a validated proxy URL.
        
        Always prefers NEOTRIX_PROXY_URL (env var) over free proxies.
        Falls back to random free proxy, or None if pool empty.
        """
        if not self._pool:
            return None
        # Prefer env proxy (always at index 0 if set)
        if self._env_proxy and len(self._pool) > 0:
            env_entry = self._pool[0]
            if env_entry.get('_env'):
                return env_entry['proxy']
        return random.choice(self._pool)['proxy']

    def stats(self) -> Dict:
        return dict(self._stats)


class ProxyTransport:
    """HTTP/HTTPS transport via auto-discovered proxy pool.

    Discovers free proxies, validates them, and routes requests through them.
    Retries with different proxies on failure (up to 3), falls back to DirectTransport.
    """

    def __init__(self):
        self.pool = ProxyPool()
        self.direct = DirectTransport()
        self._ready = False

    def _ensure_ready(self):
        if not self._ready:
            self.pool.refresh(max_valid=10)
            self._ready = True

    def fetch(self, url: str, headers: Optional[Dict] = None,
              timeout: int = 30, max_proxy_retries: int = 3) -> TransportResult:
        """Fetch a URL through the proxy pool. Retries different proxies on failure."""
        self._ensure_ready()

        req_headers = {'User-Agent': self.direct.ua.get()}
        if headers:
            req_headers.update(headers)

        tried: Set[str] = set()
        for attempt in range(max_proxy_retries):
            proxy = self.pool.get()
            if not proxy or proxy in tried:
                break
            tried.add(proxy)

            try:
                result = self._fetch_via(proxy, url, req_headers, timeout)
                if result.ok:
                    return result
            except Exception:
                if attempt < max_proxy_retries - 1:
                    continue

        return self.direct.fetch(url, headers=headers, timeout=timeout, max_retries=2)

    def _fetch_via(self, proxy: str, url: str, headers: Dict,
                   timeout: int) -> TransportResult:
        """Execute a single request through a specific proxy."""
        handler = urllib.request.ProxyHandler({
            'http': proxy,
            'https': proxy,
        })
        auth_handler = urllib.request.ProxyBasicAuthHandler()
        opener = urllib.request.build_opener(handler, auth_handler)
        req = urllib.request.Request(url, headers=headers)

        with opener.open(req, timeout=timeout) as resp:
            body = resp.read().decode('utf-8', errors='replace')
            return TransportResult(body, resp.status, dict(resp.headers))


class WikipediaTransport:
    """Wikipedia access with strategy chain: REST API → MediaWiki API.

    REST API (primary) returns full JSON with extract, thumbnail, links.
    MediaWiki API (fallback) returns plain-text extract via action=query.
    Uses requests.Session() with descriptive User-Agent for Wikimedia
    API compliance and exponential backoff retry (max 3).
    """

    REST_BASE = "https://en.wikipedia.org/api/rest_v1/page/summary"
    MEDIAWIKI_BASE = "https://en.wikipedia.org/w/api.php"
    NEOTRIX_UA = "NeoTrix/1.0 (auto-absorption pipeline; neotrix@example.com)"

    def __init__(self):
        self.session = requests.Session()
        self.session.headers.update({'User-Agent': self.NEOTRIX_UA})

    def fetch_summary(self, title: str, timeout: int = 15) -> Optional[Dict]:
        result = self._fetch_rest(title, timeout)
        if result:
            result['_source'] = 'rest_api'
            return result

        result = self._fetch_mediawiki(title, timeout)
        if result:
            result['_source'] = 'mediawiki_api'
            return result

        return None

    def _fetch_with_retry(self, url: str, timeout: int, max_retries: int = 3) -> Optional[Dict]:
        headers = {'Accept': 'application/json'}
        for attempt in range(max_retries):
            try:
                resp = self.session.get(url, headers=headers, timeout=timeout)
                if resp.status_code == 200:
                    return resp.json()
                if resp.status_code in (429, 503, 502) and attempt < max_retries - 1:
                    delay = 2.0 * (2 ** attempt) + random.uniform(0.5, 2.0)
                    time.sleep(delay)
                    continue
                return None
            except (requests.RequestException, json.JSONDecodeError) as e:
                if attempt < max_retries - 1:
                    delay = 2.0 * (2 ** attempt) + random.uniform(0.5, 2.0)
                    time.sleep(delay)
                    continue
                return None
        return None

    def _fetch_rest(self, title: str, timeout: int) -> Optional[Dict]:
        url = f"{self.REST_BASE}/{urllib.parse.quote(title.replace(' ', '_'))}"
        data = self._fetch_with_retry(url, timeout)
        if data:
            return data
        return None

    def _fetch_mediawiki(self, title: str, timeout: int) -> Optional[Dict]:
        params = urllib.parse.urlencode({
            'action': 'query',
            'titles': title,
            'prop': 'extracts|categories|info',
            'exintro': 1,
            'explaintext': 1,
            'cllimit': 10,
            'format': 'json',
        })
        url = f"{self.MEDIAWIKI_BASE}?{params}"
        data = self._fetch_with_retry(url, timeout)
        if data:
            try:
                for page_id, page in data.get('query', {}).get('pages', {}).items():
                    if page_id != '-1':
                        cats = []
                        for c in page.get('categories', []):
                            cat_title = c.get('title', '')
                            if cat_title.startswith('Category:'):
                                cat_title = cat_title[9:]
                            cats.append(cat_title)
                        return {
                            'title': page.get('title', title),
                            'extract': page.get('extract', ''),
                            'description': '',
                            'page_id': int(page_id),
                            'url': f"https://en.wikipedia.org/wiki/{urllib.parse.quote(page.get('title', title).replace(' ', '_'))}",
                            'categories': cats,
                        }
            except (json.JSONDecodeError, KeyError):
                pass
        return None


# ============================================================================
# Strategy-based API clients
# ============================================================================

class ArxivClient:
    """Multi-strategy ArXiv paper fetcher.
    Chain: Direct API → HTML fallback → (Tor if available).
    """

    ARXIV_API = "http://export.arxiv.org/api/query?id_list={}&max_results=1"
    ARXIV_NS = {"a": "http://www.w3.org/2005/Atom"}

    def __init__(self, tor: Optional[TorTransport] = None):
        self.direct = DirectTransport()
        self.tor = tor
        self._last_fetch = 0.0
        self.min_delay = 3.0  # ArXiv rate limit: 1 req / 3s

    def _rate_limit(self):
        elapsed = time.time() - self._last_fetch
        if elapsed < self.min_delay:
            time.sleep(self.min_delay - elapsed)
        self._last_fetch = time.time()

    def fetch(self, arxiv_id: str) -> Optional[Dict]:
        self._rate_limit()

        # Strategy 1: Direct API
        meta = self._fetch_api(arxiv_id)
        if meta:
            return meta

        # Strategy 2: HTML fallback
        print(f"    API fail, HTML fallback...", end=" ", flush=True)
        meta = self._fetch_html(arxiv_id)
        if meta:
            return meta

        # Strategy 3: Via Tor (if available)
        if self.tor and self.tor.available:
            print(f"    Tor proxy...", end=" ", flush=True)
            meta = self._fetch_api_via(arxiv_id, self.tor)
            if meta:
                return meta

        return None

    def _fetch_api(self, arxiv_id: str) -> Optional[Dict]:
        url = self.ARXIV_API.format(arxiv_id)
        result = self.direct.fetch(url, headers={'Accept': 'application/atom+xml'}, timeout=45, max_retries=3)
        if result.ok and result.body:
            return self._parse_atom(result.body, arxiv_id)
        return None

    def _fetch_api_via(self, arxiv_id: str, transport) -> Optional[Dict]:
        url = self.ARXIV_API.format(arxiv_id)
        result = transport.fetch(url, {'Accept': 'application/atom+xml'}, timeout=45)
        if result.ok and result.body:
            return self._parse_atom(result.body, arxiv_id)
        return None

    def _fetch_html(self, arxiv_id: str) -> Optional[Dict]:
        url = f"https://arxiv.org/abs/{arxiv_id}"
        result = self.direct.fetch(url, timeout=30, max_retries=2)
        if not result.ok or not result.body:
            return None
        return self._parse_html(result.body, arxiv_id)

    def _parse_atom(self, xml: str, arxiv_id: str) -> Optional[Dict]:
        try:
            root = ET.fromstring(xml)
            entry = root.find("a:entry", self.ARXIV_NS)
            if entry is None:
                return None
            title_el = entry.find("a:title", self.ARXIV_NS)
            summary_el = entry.find("a:summary", self.ARXIV_NS)
            published_el = entry.find("a:published", self.ARXIV_NS)
            authors = [a.find("a:name", self.ARXIV_NS).text
                       for a in entry.findall("a:author", self.ARXIV_NS)
                       if a.find("a:name", self.ARXIV_NS) is not None]
            categories = [c.get("term", "")
                          for c in entry.findall("a:category", self.ARXIV_NS)]
            doi_el = entry.find("arxiv:doi", {"arxiv": "http://arxiv.org/schemas/atom"})

            return {
                "arxiv_id": arxiv_id,
                "title": (title_el.text or "").strip().replace("\n", " ") if title_el is not None else "",
                "abstract": (summary_el.text or "").strip() if summary_el is not None else "",
                "authors": authors,
                "categories": categories,
                "published": (published_el.text or "")[:10] if published_el is not None else "",
                "doi": (doi_el.text or "").strip() if doi_el is not None else "",
            }
        except ET.ParseError:
            return None

    def _parse_html(self, html: str, arxiv_id: str) -> Optional[Dict]:
        abs_m = re.search(r'<blockquote class="abstract[^"]*"[^>]*>\s*<span[^>]*>(.*?)</span>\s*</blockquote>', html, re.DOTALL)
        abstract = ""
        if abs_m:
            abstract = re.sub(r'<[^>]+>', '', abs_m.group(1))
            abstract = html_module.unescape(abstract).strip()

        title_m = re.search(r'<h1 class="title mathjax"[^>]*>\s*(.*?)\s*</h1>', html, re.DOTALL)
        title = ""
        if title_m:
            title = re.sub(r'<[^>]+>', '', title_m.group(1))
            title = re.sub(r'^Title:\s*', '', title, flags=re.IGNORECASE).strip()
            title = html_module.unescape(title)

        auth_m = re.search(r'<div class="authors"[^>]*>(.*?)</div>', html, re.DOTALL)
        authors = []
        if auth_m:
            author_links = re.findall(r'<a[^>]*>(.*?)</a>', auth_m.group(1))
            authors = [html_module.unescape(re.sub(r'<[^>]+>', '', a)).strip() for a in author_links]

        cat_m = re.search(r'<span class="primary-subject"[^>]*>(.*?)</span>', html, re.DOTALL)
        categories = []
        if cat_m:
            cat_text = html_module.unescape(re.sub(r'<[^>]+>', '', cat_m.group(1))).strip()
            categories.append(cat_text)

        return {
            "arxiv_id": arxiv_id,
            "title": title,
            "abstract": abstract,
            "authors": authors,
            "categories": categories,
            "published": "",
            "doi": "",
            "_source": "html_fallback",
        }


class GitHubClient:
    """Multi-strategy GitHub repository fetcher.
    Chain: Token API → no-token API → raw CDN → HTML scrape.
    """

    def __init__(self):
        self.direct = DirectTransport()
        self.token = os.environ.get("NEOTRIX_GITHUB_TOKEN", "")
        self._remaining = 60 if not self.token else 5000
        self._reset_time = 0

    def fetch_repo(self, owner: str, repo: str) -> Optional[Dict]:
        result = None

        # Strategy 1: GitHub API (token or unauthenticated)
        if self._remaining > 0:
            url = f"https://api.github.com/repos/{owner}/{repo}"
            headers = {'Accept': 'application/vnd.github.v3+json'}
            if self.token:
                headers['Authorization'] = f'token {self.token}'

            api_result = self.direct.fetch(url, headers=headers, timeout=15)
            if api_result.ok and api_result.body:
                try:
                    data = json.loads(api_result.body)
                    self._remaining = int(api_result.headers.get('X-RateLimit-Remaining', self._remaining - 1))
                    self._reset_time = int(api_result.headers.get('X-RateLimit-Reset', 0))
                    return {
                        'stars': data.get('stargazers_count', 0),
                        'language': data.get('language', 'unknown'),
                        'topics': data.get('topics', []),
                        'description': (data.get('description') or '')[:500],
                        'default_branch': data.get('default_branch', 'main'),
                        'license': (data.get('license') or {}).get('spdx_id', ''),
                        'forks': data.get('forks_count', 0),
                        'updated_at': data.get('updated_at', ''),
                    }
                except (json.JSONDecodeError, KeyError):
                    pass

            if 'rate limit' in (api_result.error or '').lower():
                self._remaining = 0
        else:
            print(f"  GitHub API rate limit exhausted, falling back to HTML scrape")

        # Strategy 2: HTML scrape (no rate limit)
        html_result = self.fetch_repo_html(owner, repo)
        if html_result:
            print(f"  HTML scrape succeeded for {owner}/{repo}")
            return html_result

        return None

    def fetch_repo_html(self, owner: str, repo: str) -> Optional[Dict]:
        """Fetch repo metadata from public GitHub HTML page (no API rate limit)."""
        url = f"https://github.com/{owner}/{repo}"
        result = self.direct.fetch(url, headers={'Accept': 'text/html'}, timeout=20, max_retries=2)
        if not result.ok or not result.body:
            return None
        return self._parse_html(result.body, owner, repo)

    def _parse_html(self, html: str, owner: str, repo: str) -> Optional[Dict]:
        import html as html_mod
        stars = 0
        forks = 0
        language = "unknown"
        topics = []
        description = ""

        # Description from meta tag (fallback: og:description)
        desc_m = re.search(r'<meta\s+name="description"[^>]*content="([^"]*)"', html)
        if desc_m:
            description = html_mod.unescape(desc_m.group(1)).strip()[:500]

        # Stars from repo-stars-counter-star title attribute (new GH UI)
        stars_m = re.search(r'repo-stars-counter-star[^>]*title="([\d,]+)"', html)
        if stars_m:
            stars = int(stars_m.group(1).replace(',', ''))
        if not stars:
            # Old GH UI
            stars_m = re.search(r'class="Counter js-social-count[^"]*"[^>]*>([\d.]+[kKmM]?)<', html)
            if stars_m:
                raw = stars_m.group(1)
                if 'k' in raw.lower():
                    stars = int(float(raw.lower().replace('k', '')) * 1000)
                elif 'm' in raw.lower():
                    stars = int(float(raw.lower().replace('m', '')) * 1000000)
                else:
                    stars = int(raw)

        # Forks from repo-network-counter title attribute (new GH UI)
        forks_m = re.search(r'repo-network-counter[^>]*title="([\d,]+)"', html)
        if forks_m:
            forks = int(forks_m.group(1).replace(',', ''))
        if not forks:
            # Alternative: find Counter after "Fork" text
            forks_area = re.search(r'Fork\s*<span[^>]*>([\d.]+[kKmM]?)<', html)
            if forks_area:
                raw = forks_area.group(1)
                if 'k' in raw.lower():
                    forks = int(float(raw.lower().replace('k', '')) * 1000)
                elif 'm' in raw.lower():
                    forks = int(float(raw.lower().replace('m', '')) * 1000000)
                else:
                    forks = int(raw)

        # Language from metadata or JSON initial state
        lang_m = re.search(r'repository_language_for_display[^>]*content="([^"]+)"', html)
        if lang_m:
            language = lang_m.group(1).strip()
        if not language or language == "unknown":
            # Try JSON initial state
            lang_m = re.search(r'"language"\s*:\s*"([^"]+)"', html)
            if lang_m:
                language = lang_m.group(1).strip()
        if not language or language == "unknown":
            # Try page title: "GitHub - owner/repo: description with language"
            title_m = re.search(r'<title>(?:GitHub\s*-\s*)?[^:]+:\s*([^·(]+)', html)
            if title_m and 'programming language' not in title_m.group(1).lower():
                candidate = title_m.group(1).strip()
                if len(candidate) < 60:
                    language = candidate

        # Topics from repo topics section
        topic_m = re.findall(r'"(?:topic|TopicLabel)"[^>]*>\s*([^<]+?)\s*<', html)
        if not topic_m:
            topic_m = re.findall(r'topic-tag[^"]*"[^>]*>\s*([^<]+?)\s*<', html)

        # Topics from about-links (new GH UI uses a-tag list)
        if not topic_m:
            topic_section = re.search(r'about-links[^>]*>(.*?)(?:</ul>|</div>)', html, re.DOTALL)
            if topic_section:
                topic_m = re.findall(r'>\s*([^<]{2,40}?)\s*<', topic_section.group(1))
                topic_m = [t for t in topic_m if t.strip() and '/' not in t and len(t.strip()) > 1]

        topics = list(dict.fromkeys(html_mod.unescape(t.strip()) for t in topic_m if t.strip()))

        if not stars and not description and not language and not forks:
            return None  # Failed to parse anything useful

        return {
            'stars': stars,
            'language': language or "unknown",
            'topics': topics,
            'description': description,
            'forks': forks,
            'default_branch': 'main',
            'license': '',
            'updated_at': '',
            '_source': 'html_scrape',
        }

    def fetch_readme(self, owner: str, repo: str, branch: str = 'main') -> Optional[str]:
        """Fetch README from raw.githubusercontent.com (no rate limit)."""
        urls = [
            f'https://raw.githubusercontent.com/{owner}/{repo}/{branch}/README.md',
            f'https://raw.githubusercontent.com/{owner}/{repo}/{branch}/README.rst',
            f'https://raw.githubusercontent.com/{owner}/{repo}/{branch}/README',
            f'https://raw.githubusercontent.com/{owner}/{repo}/master/README.md',
        ]
        for url in urls:
            result = self.direct.fetch(url, timeout=15)
            if result.ok and result.body and len(result.body) > 50:
                return result.body
        return None

    def rate_limit_status(self) -> str:
        if self._remaining == 0:
            return f"EXHAUSTED (reset {time.ctime(self._reset_time)})"
        return f"{self._remaining} remaining"


# ============================================================================
# Capability Probe
# ============================================================================

def probe_capabilities(tor_host='127.0.0.1', tor_port=9050, cache_path=None) -> Dict:
    """Probe all transport backends and API availability. Results cached."""
    if cache_path and os.path.exists(cache_path):
        age = time.time() - os.path.getmtime(cache_path)
        if age < 3600:  # 1h cache
            try:
                with open(cache_path) as f:
                    return json.load(f)
            except Exception:
                pass

    cap = {'timestamp': time.time()}

    # Transport backends
    tor = TorTransport(tor_host, tor_port)
    cap['tor'] = tor.available

    # ArXiv API
    arxiv = ArxivClient()
    test = arxiv.fetch('1706.03762')
    cap['arxiv_api'] = test is not None
    cap['arxiv_html'] = test is not None  # falls through to html if api fails

    # GitHub API
    gh = GitHubClient()
    test = gh.fetch_repo('neotrix', 'neotrix')
    cap['github_api'] = test is not None
    cap['github_token'] = bool(gh.token)
    cap['github_remaining'] = gh._remaining

    # Wikipedia
    wiki = WikipediaTransport()
    test = wiki.fetch_summary('Artificial intelligence')
    cap['wikipedia'] = test is not None

    # External IP
    direct = DirectTransport()
    result = direct.fetch('https://httpbin.org/ip',
                          headers={'Accept': 'application/json'}, timeout=10)
    if result.ok and result.body:
        try:
            cap['external_ip'] = json.loads(result.body).get('origin', 'unknown')
        except Exception:
            cap['external_ip'] = 'unknown'
    else:
        cap['external_ip'] = 'unknown'

    # Proxy pool (quick probe: 3 candidates, 3s per-proxy timeout)
    proxy = ProxyTransport()
    discovered = proxy.pool.discover(timeout=5)
    if discovered:
        random.shuffle(discovered)
        valid = 0
        for p in discovered[:6]:
            ok, _ = proxy.pool.validate(p, timeout=3)
            if ok:
                valid += 1
                proxy.pool._pool.append({'proxy': f'http://{p}', 'latency': 0, 'last_validated': time.time()})
        cap['proxy_available'] = valid > 0
        cap['proxy_count'] = valid
    else:
        cap['proxy_available'] = False
        cap['proxy_count'] = 0

    if cache_path:
        os.makedirs(os.path.dirname(cache_path), exist_ok=True)
        with open(cache_path, 'w') as f:
            json.dump(cap, f)

    return cap


# ============================================================================
# Unified Access Pipeline
# ============================================================================

class AccessPipeline:
    """Unified API access pipeline with automatic capability probing and
    per-domain fallback strategies."""

    def __init__(self, tor_host='127.0.0.1', tor_port=9050, cache_capability=True):
        self.cache_path = os.path.expanduser("~/.neotrix/capability_cache.json") if cache_capability else None
        self.tor = TorTransport(tor_host, tor_port)
        self.proxy = ProxyTransport()
        self.arxiv = ArxivClient(self.tor)
        self.github = GitHubClient()
        self.wikipedia = WikipediaTransport()
        self.capability = {}

    def probe(self) -> Dict:
        """Probe all capabilities. Call once at startup."""
        self.capability = probe_capabilities(
            cache_path=self.cache_path,
            tor_host=self.tor.host,
            tor_port=self.tor.port,
        )
        return self.capability

    def fetch_arxiv(self, arxiv_id: str) -> Optional[Dict]:
        return self.arxiv.fetch(arxiv_id)

    def fetch_github(self, owner: str, repo: str) -> Optional[Dict]:
        return self.github.fetch_repo(owner, repo)

    def fetch_github_readme(self, owner: str, repo: str) -> Optional[str]:
        return self.github.fetch_readme(owner, repo)

    def fetch_wikipedia(self, title: str) -> Optional[Dict]:
        return self.wikipedia.fetch_summary(title)

    def fetch_via_proxy(self, url: str, timeout: int = 30) -> TransportResult:
        """Fetch an arbitrary URL through the proxy pool."""
        return self.proxy.fetch(url, timeout=timeout)

    def summary(self) -> str:
        lines = [f"External IP: {self.capability.get('external_ip', '?')}"]
        lines.append(f"Tor: {'✅' if self.capability.get('tor') else '❌'}")
        proxy_ok = self.capability.get('proxy_available', False)
        proxy_cnt = self.capability.get('proxy_count', 0)
        lines.append(f"Proxy Pool: {'✅' if proxy_ok else '❌'} ({proxy_cnt} valid)")
        lines.append(f"ArXiv API: {'✅' if self.capability.get('arxiv_api') else '❌'}")
        lines.append(f"GitHub API: {self.github.rate_limit_status()}")
        lines.append(f"Wikipedia: {'✅' if self.capability.get('wikipedia') else '❌'}")
        return "\n".join(lines)


# ============================================================================
# Self-test
# ============================================================================

if __name__ == '__main__':
    pipeline = AccessPipeline(cache_capability=False)
    cap = pipeline.probe()

    print("═══ Capability Probe ═══")
    print(pipeline.summary())
    print()

    if cap.get('arxiv_api'):
        print("═══ ArXiv API Test ═══")
        meta = pipeline.fetch_arxiv('1706.03762')
        if meta:
            print(f"  Title: {meta['title'][:80]}")
            print(f"  Authors: {', '.join(meta['authors'][:3])}")
            print(f"  Categories: {', '.join(meta['categories'][:3])}")
            print(f"  Abstract: {meta['abstract'][:120]}...")

    if cap.get('github_api'):
        print("\n═══ GitHub API Test ═══")
        repo = pipeline.fetch_github('neotrix', 'neotrix')
        if repo:
            print(f"  Stars: {repo['stars']}")
            print(f"  Language: {repo['language']}")
            print(f"  Topics: {repo['topics'][:3]}")

    if cap.get('proxy_available'):
        print("\n═══ Proxy Pool Status ═══")
        ps = pipeline.proxy.pool.stats()
        print(f"  Proxies discovered: {ps['total']}")
        print(f"  Validated working:  {ps['valid']}")
        print(f"  Failed validation:  {ps['failed']}")
        print(f"  Avg latency:        {ps['avg_latency_ms']}ms")
