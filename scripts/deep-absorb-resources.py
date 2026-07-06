#!/usr/bin/env python3
"""Deep absorb resource pool repos: README, file tree, content from raw.githubusercontent.com.
No API rate limits — uses raw content CDN."""
import sqlite3, json, time, os, re, urllib.request
from nt_normalizer import strip_markdown, content_fingerprint, extract_key_sections, detect_language

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())

UA = "NeoTrix/0.19-ResourceDeepAbsorb"
RAW_BASE = "https://raw.githubusercontent.com"

def sql(): return sqlite3.connect(KB)

def fetch(url, timeout=15):
    try:
        r = urllib.request.Request(url, headers={"User-Agent": UA})
        return urllib.request.urlopen(r, timeout=timeout).read().decode("utf-8", errors="replace")
    except Exception: return ""

def estimate_stars(readme_text):
    """Extract star count from README badges."""
    m = re.search(r"(?:stars|★|stargazers)[^\d]*(\d[\d,.]*[kKmMbB]?)", readme_text, re.I)
    if m: return m.group(1)
    return ""

def deep_absorb_repo(c, full_name):
    """Deep absorb a repo's README content into the KB."""
    url = f"https://github.com/{full_name}"
    owner, repo = full_name.split("/", 1)

    # Check if node exists in either table
    existing = c.execute("SELECT id, summary, content FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone()
    if not existing:
        print(f"  ⚠️  {full_name}: no stub found, skipping")
        return None, 0

    node_id = existing[0]
    curr_summary = existing[1] or ""
    curr_content = existing[2] or ""

    # Skip if already has deep content (>500 chars)
    if len(curr_content) > 500:
        print(f"  ⏭️  {full_name}: already has content ({len(curr_content)} chars)")
        return node_id, 0

    # Phase 1: Fetch README.md
    paths_to_try = [
        f"{RAW_BASE}/{full_name}/master/README.md",
        f"{RAW_BASE}/{full_name}/main/README.md",
        f"{RAW_BASE}/{full_name}/master/README.rst",
        f"{RAW_BASE}/{full_name}/main/README.rst",
    ]

    readme = ""
    readme_path = ""
    for p in paths_to_try:
        content = fetch(p)
        if content and len(content) > 50:
            readme = content
            readme_path = p
            break

    if not readme:
        print(f"  ⚠️  {full_name}: no README found")
        return node_id, 0

    # Normalize: Unicode NFKC + HTML decode + strip markdown
    readme_clean = strip_markdown(readme)

    # Phase 2: Extract sections (from clean text)
    sections = extract_key_sections(readme_clean)
    lang = detect_language(readme, title=full_name, url=url)

    # Phase 3: Build deep summary (from clean text)
    summary_parts = []
    for sec_name, sec_content in sections.items():
        if sec_content:
            summary_parts.append(f"[{sec_name}] {sec_content[:300]}")

    deep_summary = "\n\n".join(summary_parts[:5])  # Top 5 sections
    if not deep_summary:
        deep_summary = readme_clean[:1000].strip()

    # Phase 4: Store content
    # Truncate README to first 50000 chars for content field
    readme_content = readme[:50000]

    # Update nodes table (with retry for lock)
    for _ in range(3):
        try:
            c.execute("UPDATE nodes SET summary=?, content=?, updated_at=? WHERE id=?",
                      (deep_summary[:2000], readme_content, NOW, node_id))
            break
        except sqlite3.OperationalError:
            conn.commit()
            time.sleep(0.5)
    # Update node with detected language + absorption stats + content fingerprint
    if lang and lang != "en":
        c.execute("UPDATE nodes SET language=? WHERE id=?", (lang, node_id))
    meta = c.execute("SELECT metadata FROM nodes WHERE id=?", (node_id,)).fetchone()
    if meta and meta[0]:
        try:
            m = json.loads(meta[0])
            m["readme_absorbed"] = True
            m["readme_size"] = len(readme)
            m["detected_language"] = lang
            fp = content_fingerprint(readme)
            if fp:
                m["content_fp"] = fp
            c.execute("UPDATE nodes SET metadata=? WHERE id=?", (json.dumps(m), node_id))
        except Exception: pass

    print(f"  📖 {full_name}: {len(readme)} chars README absorbed")
    return node_id, len(readme)

REPOS_TO_ABSORB = [
    # 🎵 Music & Audio
    "yt-dlp/yt-dlp", "FFmpeg/FFmpeg", "mpv-player/mpv", "HandBrake/HandBrake",
    # 📚 Books & Learning
    "EbookFoundation/free-programming-books", "getify/You-Dont-Know-JS",
    "jwasham/coding-interview-university", "ossu/computer-science",
    "krahets/hello-algo", "kamranahmedse/developer-roadmap",
    # 🔬 Research & Science
    "papers-we-love/papers-we-love", "rasbt/LLMs-from-scratch",
    "microsoft/generative-ai-for-beginners",
    # 🎨 Design
    "danistefanovic/build-your-own-x", "bradtraversy/design-resources-for-developers",
    # 🧠 KM
    "siyuan-note/siyuan", "logseq/logseq",
    # 🤖 AI
    "Significant-Gravitas/AutoGPT", "langgenius/dify", "n8n-io/n8n",
    "firecrawl/firecrawl",
    # 📦 Aggregators
    "sindresorhus/awesome", "public-apis/public-apis",
    "awesome-selfhosted/awesome-selfhosted", "ripienaar/free-for-dev",
]

def main():
    conn = sqlite3.connect(KB)
    c = conn.cursor()
    total_bytes = 0
    absorbed = 0
    skipped = 0

    print(f"═══ Deep Absorb Resource Pool Repos ═══")
    print(f"Target: {len(REPOS_TO_ABSORB)} repos\n")

    for full_name in REPOS_TO_ABSORB:
        nid, size = deep_absorb_repo(c, full_name)
        if nid:
            if size > 0:
                absorbed += 1
                total_bytes += size
            else:
                skipped += 1
        conn.commit()
        time.sleep(0.5)  # Polite delay for raw.githubusercontent.com

    conn.close()
    print(f"\n✅ Deep absorb complete!")
    print(f"   Absorbed: {absorbed} repos, {total_bytes:,} total bytes of README content")
    print(f"   Skipped (no README): {skipped}")
    print(f"   KB nodes now have real content from curated resource collections.")

if __name__ == "__main__":
    main()
