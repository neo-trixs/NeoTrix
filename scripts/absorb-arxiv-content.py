#!/usr/bin/env python3
"""
Absorb-arxiv-content.py — fill empty Paper/article node content from ArXiv API.
Single-paper sequential fetches with UA rotation + exponential backoff + HTML fallback.

Usage:
  python3 absorb-arxiv-content.py                          # absorb all empty nodes
  python3 absorb-arxiv-content.py --limit 10                # absorb first 10 only
  python3 absorb-arxiv-content.py --id 1706.03762           # absorb a specific paper
  python3 absorb-arxiv-content.py --check                   # check only, no writes
"""

import sqlite3
import re
import time
import json
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from nt_api_client import AccessPipeline

DB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
DELAY = 3.5  # seconds between requests (ArXiv rate limit: ~1 req/3s)

_pipeline = None
def _get_pipeline():
    global _pipeline
    if _pipeline is None:
        _pipeline = AccessPipeline(cache_capability=False)
    return _pipeline


def extract_arxiv_id(url):
    """Extract arxiv ID from various URL formats."""
    if not url:
        return None
    # Pattern: abs/XXXX.XXXXX or abs/astro-ph/XXXXXXX
    m = re.search(r'arxiv\.org/(?:abs|pdf)/([a-z\-\.]+/\d{7}|\d{4}\.\d{4,5})(?:v\d+)?', url)
    if m:
        return m.group(1)
    return None


def build_content(meta):
    """Build KB content string from metadata."""
    parts = []
    if meta.get("abstract"):
        parts.append(f"Abstract: {meta['abstract']}")
    if meta.get("authors"):
        parts.append(f"Authors: {', '.join(meta['authors'])}")
    if meta.get("categories"):
        parts.append(f"Categories: {', '.join(meta['categories'])}")
    if meta.get("published"):
        parts.append(f"Published: {meta['published']}")
    if meta.get("doi"):
        parts.append(f"DOI: {meta['doi']}")
    if meta.get("comment"):
        parts.append(f"Comment: {meta['comment']}")
    if meta.get("journal_ref"):
        parts.append(f"Journal: {meta['journal_ref']}")
    return "\n\n".join(parts)


def update_node(db, node_id, meta, dry_run=False):
    """Update node with fetched content."""
    title = meta.get("title", "").strip()
    abstract = meta.get("abstract", "").strip()
    content = build_content(meta)
    authors = meta.get("authors", [])
    categories = meta.get("categories", [])

    # Build metadata JSON
    metadata = json.dumps({
        "arxiv_id": meta.get("arxiv_id", ""),
        "authors": authors,
        "categories": categories,
        "published": meta.get("published", ""),
        "doi": meta.get("doi", ""),
        "comment": meta.get("comment", ""),
        "journal_ref": meta.get("journal_ref", ""),
    })

    summary = f"[arXiv {meta['arxiv_id']}] {', '.join(categories[:3])}" if categories else f"[arXiv {meta['arxiv_id']}]"
    if authors:
        summary = f"{authors[0]}{' et al.' if len(authors) > 1 else ''} \u2014 {summary}"

    if dry_run:
        print(f"    Would update: title={title[:60]}, abstract_len={len(abstract)}, authors={len(authors)}")
        return True

    try:
        db.execute(
            """UPDATE nodes SET
                title = COALESCE(NULLIF(?, ''), title),
                summary = COALESCE(NULLIF(?, ''), summary),
                content = COALESCE(NULLIF(?, ''), content),
                metadata = ?,
                updated_at = strftime('%s', 'now')
            WHERE id = ?""",
            (title, summary, content, metadata, node_id)
        )
        return True
    except Exception as e:
        print(f"    DB error: {e}")
        return False


def get_empty_nodes(db, limit=None, node_types=None):
    """Get nodes needing content fill. Prioritizes paper/article types."""
    conditions = ["(content IS NULL OR content = '')", "url LIKE '%arxiv.org%'"]
    type_filter = ""
    params = []

    if node_types:
        placeholders = ",".join("?" for _ in node_types)
        type_filter = f"AND node_type IN ({placeholders})"
        params = list(node_types)

    query = f"""SELECT id, node_type, title, url FROM nodes
                WHERE {' AND '.join(conditions)} {type_filter}
                ORDER BY
                    CASE node_type
                        WHEN 'Paper' THEN 0
                        WHEN 'paper' THEN 1
                        WHEN 'article' THEN 2
                        WHEN 'Concept' THEN 3
                        WHEN 'Resource' THEN 4
                        ELSE 5
                    END,
                    created_at"""

    c = db.execute(query, params)
    results = c.fetchall()

    if limit:
        results = results[:limit]

    return results


def process_node(db, node_id, node_type, title, url, dry_run=False):
    """Process a single node: fetch arxiv content, update KB."""
    arxiv_id = extract_arxiv_id(url)
    if not arxiv_id:
        print(f"  SKIP {node_id[:12]}: Cannot extract arxiv ID from: {url}")
        return False

    print(f"  Fetching arXiv {arxiv_id}...", end=" ", flush=True)
    meta = _get_pipeline().fetch_arxiv(arxiv_id)

    if meta is None:
        print("FAILED")
        return False

    print(f"OK ({meta.get('title', '?')[:50]}...)")

    success = update_node(db, node_id, meta, dry_run=dry_run)
    if success:
        db.commit()
    return success


def main():
    dry_run = "--check" in sys.argv
    limit = None

    for arg in sys.argv[1:]:
        if arg.startswith("--limit="):
            limit = int(arg.split("=")[1])
        elif arg == "--check":
            dry_run = True

    db = sqlite3.connect(DB_PATH)
    db.execute("PRAGMA busy_timeout=30000")
    db.execute("PRAGMA journal_mode=WAL")

    # Get all empty arxiv nodes
    nodes = get_empty_nodes(db, limit=limit)

    print(f"Found {len(nodes)} empty nodes with arxiv URLs")
    if dry_run:
        print("DRY RUN MODE (--check): no writes")
    print()

    source_count = {}
    success_count = 0
    fail_count = 0

    for node_id, node_type, title, url in nodes:
        source_count[node_type] = source_count.get(node_type, 0) + 1
        print(f"[{node_type}] {node_id[:12]}: {title[:60] or '?'}")

        ok = process_node(db, node_id, node_type, title, url, dry_run=dry_run)
        if ok:
            success_count += 1
        else:
            fail_count += 1

        time.sleep(DELAY)

    print(f"\n{'='*60}")
    print(f"Results: {success_count} filled, {fail_count} failed, {len(nodes)} total")
    print(f"By source type: {json.dumps(source_count)}")

    if dry_run:
        print("(dry run \u2014 no data written)")

    db.close()


if __name__ == "__main__":
    main()
