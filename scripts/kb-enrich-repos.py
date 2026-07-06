#!/usr/bin/env python3
"""Enrich NeoTrix KB Repository nodes with metadata from GitHub API.

Queries KB for Repository nodes with missing metadata (no stars/language/topics),
fetches metadata via GitHub API through AccessPipeline, and updates KB.

Phase 1: Fetch metadata from GitHub API
Phase 2: Report results to KB kv_store

Usage:
    python3 scripts/kb-enrich-repos.py                  # enrich up to 50 repos
    python3 scripts/kb-enrich-repos.py --limit 100      # enrich up to 100
    python3 scripts/kb-enrich-repos.py --check          # dry-run, no writes
"""
import sqlite3, json, time, os, re, argparse
from nt_api_client import AccessPipeline
from nt_normalizer import normalize_lang

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
MAX_DEFAULT = 50
REQUEST_DELAY = 2.0

GITHUB_URL_RE = re.compile(r'^https?://github\.com/([^/]+)/([^/#?]+)')

def sql_retry(c, sql, params, max_retries=3):
    for i in range(max_retries):
        try:
            return c.execute(sql, params)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < max_retries - 1:
                time.sleep(0.5)
                continue
            raise

def get_repos_needing_enrichment(c, limit):
    """Query Repository nodes missing metadata (stars)."""
    rows = c.execute("""
        SELECT id, url, title
        FROM nodes
        WHERE node_type = 'Repository'
          AND url IS NOT NULL
          AND url LIKE 'https://github.com/%'
          AND (
               metadata IS NULL
            OR metadata = ''
            OR metadata = '{}'
            OR json_extract(metadata, '$.stars') IS NULL
          )
        ORDER BY updated_at ASC
        LIMIT ?
    """, (limit,)).fetchall()
    return rows

def extract_owner_repo(url):
    m = GITHUB_URL_RE.match(url)
    if not m:
        return None, None
    owner = m.group(1).lower()
    repo = m.group(2).lower()
    if not owner or not repo:
        return None, None
    return owner, repo

def update_node_metadata(c, node_id, meta, language):
    """Update a KB node with fetched metadata."""
    meta_json = json.dumps(meta)
    importance = min(1.0, meta.get("stars", 0) / 100000.0)
    now = int(time.time())

    sql_retry(c,
        "UPDATE nodes SET metadata=?, importance=COALESCE(NULLIF(?, importance), importance), language=COALESCE(NULLIF(?, language), language), updated_at=? WHERE id=?",
        (meta_json, importance, language, now, node_id)
    )

def store_report(c, report):
    """Store enrichment report to KB kv_store."""
    now = int(time.time())
    sql_retry(c,
        "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
        ("meta_cognition", "repo_enrichment_report", json.dumps(report), now)
    )

def main():
    parser = argparse.ArgumentParser(description="Enrich KB Repository nodes with GitHub metadata")
    parser.add_argument("--limit", type=int, default=MAX_DEFAULT,
                        help=f"Max repos to enrich (default: {MAX_DEFAULT})")
    parser.add_argument("--check", action="store_true",
                        help="Dry-run mode: scan and report without writing")
    args = parser.parse_args()

    limit = min(args.limit, 200)

    print(f"═══ KB Repository Enrichment ═══")
    print(f"  KB: {KB}")
    print(f"  Limit: {limit} repos")
    if args.check:
        print(f"  Mode: DRY-RUN (no writes)")
    print()

    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA busy_timeout=30000")
    conn.execute("PRAGMA journal_mode=WAL")
    c = conn.cursor()

    repos = get_repos_needing_enrichment(c, limit)
    if not repos:
        print("  ✓ No repositories need enrichment.")
        conn.close()
        return

    total = len(repos)
    print(f"  Found {total} repositories needing metadata enrichment.\n")

    pipeline = AccessPipeline(cache_capability=True)
    pipeline.probe()

    enriched = 0
    failed = 0
    skipped = 0
    total_stars = 0

    for i, (node_id, url, title) in enumerate(repos, 1):
        owner, repo = extract_owner_repo(url)
        if not owner or not repo:
            skipped += 1
            print(f"  [{i}/{total}] ⏭️  {title}: could not parse owner/repo from {url}")
            continue

        print(f"  [{i}/{total}] {owner}/{repo}...", end=" ", flush=True)

        if args.check:
            print(f"🟡 [DRY-RUN] would fetch")
            continue

        meta = pipeline.fetch_github(owner, repo)
        if meta is None:
            failed += 1
            print(f"❌ failed")
            time.sleep(REQUEST_DELAY)
            continue

        stars = meta.get("stars", 0)
        language = meta.get("language") or ""
        topics = meta.get("topics") or []
        total_stars += stars

        if language:
            language = normalize_lang(language)

        metadata = {
            "stars": stars,
            "language": language,
            "topics": topics,
            "owner": owner,
            "forks": meta.get("forks", 0),
            "license": meta.get("license", ""),
            "default_branch": meta.get("default_branch", "main"),
            "updated_at": meta.get("updated_at", ""),
            "enriched_at": int(time.time()),
        }

        update_node_metadata(c, node_id, metadata, language)
        enriched += 1
        print(f"✅ {stars}★ {language or '?'} ({len(topics)} topics)")

        conn.commit()
        time.sleep(REQUEST_DELAY)

    conn.commit()

    report = {
        "total_found": total,
        "enriched": enriched,
        "failed": failed,
        "skipped": skipped,
        "total_stars_added": total_stars,
        "timestamp": int(time.time()),
        "limit": limit,
        "dry_run": args.check,
    }

    if not args.check:
        store_report(c, report)
        conn.commit()

    print()
    print("═══ Enrichment Report ═══")
    print(f"  Total found:   {total}")
    print(f"  Enriched:      {enriched}")
    print(f"  Failed:        {failed}")
    print(f"  Skipped:       {skipped}")
    print(f"  Stars added:   {total_stars:,}")
    if not args.check:
        print(f"  Report stored: kv_store meta_cognition:repo_enrichment_report")

    conn.close()

if __name__ == "__main__":
    main()
