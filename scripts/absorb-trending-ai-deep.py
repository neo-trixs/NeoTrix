#!/usr/bin/env python3
"""Phase 2: Deep-absorb README content for 12 trending AI repos.
Uses raw.githubusercontent.com CDN with 10s per-fetch timeout.
Usage: python3 scripts/absorb-trending-ai-deep.py"""
import sqlite3, json, time, os, hashlib, urllib.request, re

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())
UA = "NeoTrix/0.19-TrendingDeep"
RAW = "https://raw.githubusercontent.com"

REPOS = [
    "topoteretes/cognee", "DeusData/codebase-memory-mcp",
    "diegosouzapw/OmniRoute", "ogulcancelik/herdr",
    "msitarzewski/agency-agents", "stablyai/orca",
    "usestrix/strix", "alibaba/page-agent",
    "browser-use/video-use", "callesthio/OpenMontage",
    "openai/codex-plugin-cc", "JCodesMore/ai-website-cloner-template",
]

def fetch(url, timeout=8):
    try:
        r = urllib.request.Request(url, headers={"User-Agent": UA})
        return urllib.request.urlopen(r, timeout=timeout).read().decode("utf-8", errors="replace")
    except Exception: return ""

def strip_md(text):
    text = re.sub(r"```[\s\S]*?```", "", text)
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"(?m)^\s*[#*\-=]+\s*", "", text)
    text = re.sub(r"\*\*|__|~~|`", "", text)
    text = re.sub(r"\s+", " ", text).strip()
    return text

def main():
    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()
    total = 0
    ok = 0

    print("═══ Phase 2: Deep absorb README content ═══\n")
    for fn in REPOS:
        url = f"https://github.com/{fn}"
        row = c.execute("SELECT id, content FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone()
        if not row:
            print(f"  ⚠️  {fn}: no node in KB, skipping"); continue
        nid, cur = row[0], row[1] or ""
        if len(cur) > 500:
            print(f"  ⏭️  {fn}: already has content ({len(cur)} chars)"); ok += 1; continue

        owner, repo = fn.split("/", 1)
        readme = ""
        for p in [f"{RAW}/{fn}/master/README.md", f"{RAW}/{fn}/main/README.md"]:
            content = fetch(p)
            if content and len(content) > 50:
                readme = content; break

        if not readme:
            print(f"  ⚠️  {fn}: no README"); continue

        clean = strip_md(readme)
        summary = clean[:2000] if clean else ""
        c.execute("UPDATE nodes SET summary=?, content=?, updated_at=? WHERE id=?",
                  (summary, readme[:50000], NOW, nid))
        meta = c.execute("SELECT metadata FROM nodes WHERE id=?", (nid,)).fetchone()
        if meta and meta[0]:
            try:
                m = json.loads(meta[0])
                m["readme_absorbed"] = True
                m["readme_size"] = len(readme)
                m["content_fp"] = hashlib.sha256(readme.encode()).hexdigest()[:16]
                c.execute("UPDATE nodes SET metadata=? WHERE id=?", (json.dumps(m), nid))
            except Exception: pass
        conn.commit()
        print(f"  📖 {fn}: {len(readme)} chars")
        total += len(readme)
        ok += 1
        time.sleep(0.3)

    conn.close()
    print(f"\n✅ Deep absorb: {ok} repos, {total:,} total chars")

if __name__ == "__main__":
    main()
