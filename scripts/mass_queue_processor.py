#!/usr/bin/env python3
"""Queue processor: Wikipedia REST, ArXiv, GitHub APIs; skip dead domains; 16 concurrent workers."""
import sqlite3, json, hashlib, time, re, logging, sys
from datetime import datetime, timezone
from urllib.parse import urlparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from xml.etree import ElementTree

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
log = logging.getLogger("process_queue")

DB = "/Users/neo/.neotrix/knowledge.db"
BATCH = 200
MAX_WORKERS = 16

try:
    import httpx
    HTTPX = True
except ImportError:
    HTTPX = False

def http_get(url, timeout=20):
    if HTTPX:
        with httpx.Client(timeout=timeout, follow_redirects=True) as c:
            r = c.get(url, headers={"User-Agent": "NeoTrixBot/2.0"})
            r.raise_for_status()
            return r.content
    else:
        import urllib.request
        return urllib.request.urlopen(url, timeout=timeout).read()

def http_json(url, timeout=20):
    return json.loads(http_get(url, timeout))

def process_url(url, domain):
    try:
        path = urlparse(url).path

        if domain in ("books.google.co.jp", "books.google.com", "books.google.de", "books.google.fr"):
            return ("skipped", None)

        if domain.endswith("wikipedia.org"):
            title = path.split("/wiki/")[-1] if "/wiki/" in path else ""
            if title:
                from urllib.parse import unquote
                title = unquote(title.replace("_", " "))
                data = http_json(f"https://{domain}/api/rest_v1/page/summary/{__import__('urllib.parse').quote(title, safe='')}")
                extract = data.get("extract", "")[:3000]
                nid = hashlib.sha256(url.encode()).hexdigest()[:16]
                return ("completed", (nid, "wikipedia", data.get("title", title), extract, url, domain))
            return ("failed", None)

        if domain == "arxiv.org":
            aid = path.replace("/abs/", "").replace("/pdf/", "").split("v")[0].split("/")[-1]
            xml = http_get(f"http://export.arxiv.org/api/query?id_list={aid}")
            root = ElementTree.fromstring(xml)
            ns = {"a": "http://www.w3.org/2005/Atom"}
            entry = root.find("a:entry", ns)
            if entry is not None:
                t = entry.find("a:title", ns)
                s = entry.find("a:summary", ns)
                title = t.text.strip().replace("\n", " ") if t is not None else aid
                summary = s.text.strip()[:3000] if s is not None else ""
                nid = hashlib.sha256(url.encode()).hexdigest()[:16]
                return ("completed", (nid, "arxiv", title, summary, url, domain))
            return ("failed", None)

        if domain == "github.com":
            parts = [p for p in path.split("/") if p]
            if len(parts) >= 2:
                owner, repo = parts[0], parts[1]
                try:
                    data = http_json(f"https://api.github.com/repos/{owner}/{repo}")
                    desc = data.get("description") or ""
                    title = data.get("full_name", f"{owner}/{repo}")
                    nid = hashlib.sha256(url.encode()).hexdigest()[:16]
                    summary = f"{desc} | ⭐{data.get('stargazers_count',0)} | {data.get('language','')}"
                    return ("completed", (nid, "github", title, summary, url, domain))
                except:
                    pass
            return ("failed", None)

        if domain in ("ncbi.nlm.nih.gov", "pubmed.ncbi.nlm.nih.gov"):
            pmid_match = re.search(r'/pmc/articles/PMC(\d+)|/pubmed/(\d+)', path)
            if pmid_match:
                pmid = pmid_match.group(1) or pmid_match.group(2)
                try:
                    xml = http_get(f"https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=pubmed&id={pmid}&retmode=xml")
                    root = ElementTree.fromstring(xml)
                    title_el = root.find(".//Item[@Name='Title']")
                    title = title_el.text if title_el is not None else pmid
                    nid = hashlib.sha256(url.encode()).hexdigest()[:16]
                    return ("completed", (nid, "pubmed", title, "", url, domain))
                except:
                    pass
            return ("failed", None)

        if domain == "semanticscholar.org":
            parts = [p for p in path.split("/") if p]
            corpus = parts[-1] if parts else ""
            try:
                data = http_json(f"https://api.semanticscholar.org/graph/v1/paper/{corpus}?fields=title,abstract,year")
                title = data.get("title", corpus)
                abstract = data.get("abstract", "")[:3000]
                nid = hashlib.sha256(url.encode()).hexdigest()[:16]
                return ("completed", (nid, "semantic_scholar", title, abstract, url, domain))
            except:
                pass
            return ("failed", None)

        if domain == "doi.org":
            doi = path.lstrip("/")
            try:
                data = http_json(f"https://api.crossref.org/works/{doi}")
                msg = data.get("message", {})
                title = (msg.get("title") or [""])[0]
                abstract = (msg.get("abstract") or "")[:3000]
                nid = hashlib.sha256(url.encode()).hexdigest()[:16]
                return ("completed", (nid, "doi", title, abstract, url, domain))
            except:
                pass
            return ("failed", None)

        if domain == "dblp.uni-trier.de":
            pid = re.search(r'pid/([^/]+)', path)
            if pid:
                try:
                    xml = http_get(f"https://dblp.org/pid/{pid.group(1)}.xml")
                    root = ElementTree.fromstring(xml)
                    person = root.find(".//person")
                    name = person.get("name", pid.group(1)) if person is not None else pid.group(1)
                    nid = hashlib.sha256(url.encode()).hexdigest()[:16]
                    return ("completed", (nid, "dblp", name, "", url, domain))
                except:
                    pass
            return ("failed", None)

        c = http_get(url)
        text = c.decode("utf-8", errors="replace")
        title = ""
        m = re.search(r'<title[^>]*>(.*?)</title>', text, re.DOTALL | re.IGNORECASE)
        if m: title = m.group(1).strip()
        body = re.sub(r'<[^>]+>', ' ', text)
        body = re.sub(r'\s+', ' ', body).strip()[:2000]
        nid = hashlib.sha256(url.encode()).hexdigest()[:16]
        return ("completed", (nid, "web", title, body, url, domain))

    except Exception as e:
        return ("failed", str(e)[:200])

def main():
    conn = sqlite3.connect(DB)
    total = conn.execute("SELECT COUNT(*) FROM crawl_queue WHERE status='pending'").fetchone()[0]
    log.info(f"Pending: {total}")

    processed = 0
    while processed < total:
        rows = conn.execute(
            "SELECT id, url, COALESCE(domain,'') FROM crawl_queue WHERE status='pending' ORDER BY priority DESC, discovered_at ASC LIMIT ?",
            (BATCH,)
        ).fetchall()
        if not rows: break

        now = int(time.time())

        with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
            fut_map = {}
            for (uid, url, domain) in rows:
                conn.execute("UPDATE crawl_queue SET status='processing', last_attempt=? WHERE id=?", (now, uid))
                fut = pool.submit(process_url, url, domain)
                fut_map[fut] = (uid, url)

            for fut in as_completed(fut_map):
                uid, url = fut_map[fut]
                status, payload = fut.result()
                if status == "completed" and payload:
                    nid, ntype, title, summary, purl, domain = payload
                    conn.execute(
                        "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, domain, created_at, updated_at) VALUES (?,?,?,?,?,?,?,?)",
                        (nid, ntype, title[:500], summary[:5000], purl, domain, now, now)
                    )
                conn.execute("UPDATE crawl_queue SET status=?, retry_count=retry_count+1, last_attempt=? WHERE id=?",
                             (status, now, uid))
                processed += 1

        conn.commit()
        remaining = conn.execute("SELECT COUNT(*) FROM crawl_queue WHERE status='pending'").fetchone()[0]
        log.info(f"Batch done: {processed}/{total} processed, {remaining} remaining")

    conn.close()
    log.info(f"Done: {processed} URLs processed")

if __name__ == "__main__":
    main()
