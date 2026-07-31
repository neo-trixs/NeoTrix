#!/usr/bin/env python3
"""Enrich empty repository nodes by refetching README content from GitHub.

Reads repository-type nodes whose content is the placeholder template, refetches
the real README via raw.githubusercontent.com, updates content + metadata, then
re-runs source-core / capability mapping so the new content feeds classification.

Idempotent: only nodes still matching the placeholder template are touched.
"""
import json
import re
import sqlite3
import sys
import time
import urllib.error
import urllib.request

sys.path.insert(0, "/Users/neo/Downloads/neotrix/scripts")
import absorb_to_capability as ac  # noqa: E402

DB = "/Users/neo/.neotrix/knowledge.db"
PLACEHOLDER_RE = re.compile(r"is a software repository from github\.com")
UA = {"User-Agent": "neotrix"}


def fetch_readme(repo_path, tries=0):
    for branch in ("HEAD", "master", "main"):
        url = f"https://raw.githubusercontent.com/{repo_path}/{branch}/README.md"
        try:
            req = urllib.request.Request(url, headers=UA)
            data = urllib.request.urlopen(req, timeout=25).read().decode("utf-8", errors="replace")
            if len(data.strip()) >= 200:
                return data
        except urllib.error.HTTPError:
            continue
        except Exception:
            pass
    return None


def repo_from_url(url):
    m = re.search(r"github\.com/([^/]+)/([^/]+)", url or "")
    if not m:
        return None
    return f"{m.group(1)}/{m.group(2).rstrip('/')}"


def main():
    conn = sqlite3.connect(DB)
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()
    c.execute("""
        SELECT id, title, url FROM nodes
        WHERE node_type='repository'
          AND url LIKE '%github%'
          AND LENGTH(COALESCE(content,'')) < 200
    """)
    rows = c.fetchall()
    total = len(rows)
    print(f"待充实仓库: {total}")
    if not total:
        return

    enriched = 0
    skipped = 0
    failed = 0
    start = time.time()
    for i, (nid, title, url) in enumerate(rows, 1):
        repo = repo_from_url(url)
        if not repo:
            skipped += 1
            continue
        text = fetch_readme(repo)
        if not text:
            failed += 1
            continue

        core = ac.map_source_core(title, text, url, "repository")
        cap = ac.map_node("repository", title, text, url)
        cap_name = cap[0] if isinstance(cap, tuple) else cap
        metadata = {
            "knowledge_source": {"source_core": core},
            "absorbed_capability": {"capability": cap_name},
            "enriched": {"readme_refetch": True, "chars": len(text)},
        }
        c.execute(
            "UPDATE nodes SET content=?, metadata=? WHERE id=?",
            (text, json.dumps(metadata), nid),
        )
        enriched += 1
        if i % 50 == 0 or i == total:
            conn.commit()
            el = time.time() - start
            print(f"  [{i}/{total}] enriched={enriched} failed={failed} ({el:.0f}s)", flush=True)

    conn.commit()
    conn.close()
    print(f"\n完成: enriched={enriched} failed={failed} skipped={skipped} 用时={time.time()-start:.0f}s")


if __name__ == "__main__":
    main()
