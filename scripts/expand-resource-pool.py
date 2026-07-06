#!/usr/bin/env python3
"""NeoTrix Resource Pool v2: Expand with Security, DevOps, Mobile, Web, Data, Gaming, Blockchain repos.
Phase 1: Inject node stubs. Phase 2: Deep absorb READMEs from raw.githubusercontent.com."""
import sqlite3, json, time, os, hashlib, urllib.request, html
from nt_normalizer import (
    normalize_text, strip_markdown, content_fingerprint,
    normalize_lang,
)

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())
UA = "NeoTrix/0.19-ResourcePool-v2"
RAW_BASE = "https://raw.githubusercontent.com"

def ndig(s): return hashlib.md5(s.encode()).hexdigest()[:20]


def sql_retry(c, sql, params, max_retries=3):
    for i in range(max_retries):
        try:
            return c.execute(sql, params)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < max_retries - 1:
                time.sleep(0.5)
                continue
            raise

def insert_node(c, ntype, title, summary, url, domain, meta={}):
    kid = f"nt-{ndig(url)}"
    existing = c.execute("SELECT id FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone()
    if existing: return existing[0]
    # Also check by ID (handles cross-source dedup)
    existing2 = c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (kid,)).fetchone()
    if existing2: return existing2[0]
    title = normalize_text(title)
    summary = normalize_text(summary)
    meta_s = json.dumps(meta) if meta else "{}"
    sql_retry(c, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,metadata) VALUES (?,?,?,?,?,?,?,'en',1.0,0.7,?,?,?)",
              (kid, ntype, title, summary, "", url, domain, NOW, NOW, meta_s))
    return kid

def insert_edge(c, src, tgt, rtype, weight=0.8, desc=""):
    eid = f"re-{ndig(f'{src}{tgt}')}"
    existing = c.execute("SELECT id FROM edges WHERE source_id=? AND target_id=? LIMIT 1", (src, tgt)).fetchone()
    if existing: return
    sql_retry(c, "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) VALUES (?,?,?,?,?,?,?)",
              (eid, src, tgt, rtype, weight, desc, NOW))

def fetch(url, timeout=15):
    try:
        r = urllib.request.Request(url, headers={"User-Agent": UA})
        return urllib.request.urlopen(r, timeout=timeout).read().decode("utf-8", errors="replace")
    except Exception: return ""

# ════════════════════════════════════════════
# NEW REPOS — 7 additional categories
# ════════════════════════════════════════════

NEW_REPOS = {
    # 🛡️ Security & Privacy
    "OWASP/CheatSheetSeries":            (28000, "Markdown", ["security", "owasp", "cheatsheet"]),
    "drduh/macOS-Security-and-Privacy-Guide": (21000, "Markdown", ["macos", "security", "privacy"]),
    "swisskyrepo/PayloadsAllTheThings":  (62000, "Python", ["payloads", "pentesting", "security"]),
    "cure53/HTTPLeaks":                  (2600, "HTML", ["http", "leaks", "security"]),
    "veeral-patel/how-to-secure-anything": (9000, "Markdown", ["security", "guide"]),
    "mikewest/https-string":             (2500, "Markdown", ["https", "security", "guidelines"]),
    "OWASP/wstg":                        (5500, "Markdown", ["testing", "web-security", "owasp"]),

    # ⚡ DevOps & Infrastructure
    "kubernetes/kubernetes":             (112000, "Go", ["kubernetes", "container", "orchestrator"]),
    "ansible/ansible":                   (63000, "Python", ["ansible", "automation", "devops"]),
    "docker/compose":                    (34000, "Go", ["docker", "compose", "container"]),
    "prometheus/prometheus":             (56000, "Go", ["monitoring", "metrics", "time-series"]),
    "grafana/grafana":                   (66000, "TypeScript", ["dashboard", "monitoring", "analytics"]),
    "hashicorp/terraform":              (43000, "Go", ["terraform", "iac", "infrastructure"]),
    "nvm-sh/nvm":                        (81000, "Shell", ["nodejs", "version-manager"]),

    # 📱 Mobile Development
    "flutter/flutter":                   (166000, "Dart", ["flutter", "mobile", "ui-toolkit"]),
    "facebook/react-native":             (119000, "JavaScript", ["react-native", "mobile", "cross-platform"]),
    "nicklausw/awesome-flutter":         (52000, "Markdown", ["flutter", "awesome-list"]),
    "JStumpp/awesome-android":           (11000, "Markdown", ["android", "awesome-list"]),
    "vsouza/awesome-ios":                (47000, "Markdown", ["ios", "awesome-list"]),
    "dotnet/maui":                       (22000, "C#", ["maui", "dotnet", "cross-platform"]),

    # 🌐 Web Development
    "facebook/react":                    (232000, "JavaScript", ["react", "ui", "frontend"]),
    "vuejs/vue":                         (208000, "TypeScript", ["vue", "frontend", "framework"]),
    "angular/angular":                   (96000, "TypeScript", ["angular", "frontend", "framework"]),
    "vercel/next.js":                    (128000, "JavaScript", ["nextjs", "react", "framework"]),
    "expressjs/express":                 (65000, "JavaScript", ["express", "nodejs", "server"]),
    "django/django":                     (80000, "Python", ["django", "web-framework", "python"]),
    "tailwindlabs/tailwindcss":          (84000, "TypeScript", ["tailwind", "css", "framework"]),

    # 🗄️ Data & Databases
    "redis/redis":                       (67000, "C", ["redis", "cache", "database"]),
    "elastic/elasticsearch":             (70000, "Java", ["elasticsearch", "search", "analytics"]),
    "numpy/numpy":                       (28000, "Python", ["numpy", "numerical", "computing"]),
    "pandas-dev/pandas":                 (44000, "Python", ["pandas", "data-analysis", "python"]),
    "apache/spark":                      (40000, "Scala", ["spark", "big-data", "distributed"]),
    "postgres/postgres":                 (16000, "C", ["postgresql", "database", "rdbms"]),
    "sqlite/sqlite":                     (6000, "C", ["sqlite", "database", "embedded"]),
    "mongodb/mongo":                     (26000, "C++", ["mongodb", "database", "nosql"]),

    # 🎮 Game Development
    "godotengine/godot":                 (91000, "C++", ["godot", "game-engine", "2d", "3d"]),
    "leereilly/games":                   (19000, "Markdown", ["games", "awesome-list", "gamedev"]),
    "Kavex/awesome-gamedev":             (12000, "Markdown", ["gamedev", "resources", "awesome-list"]),
    "HackerPoet/MarbleMarcher":          (5200, "C++", ["shader", "gamedev", "graphics"]),

    # ⛓️ Blockchain & Web3
    "bitcoin/bitcoin":                   (80000, "C++", ["bitcoin", "cryptocurrency", "blockchain"]),
    "ethereum/go-ethereum":              (48000, "Go", ["ethereum", "blockchain", "smart-contracts"]),
    "OpenZeppelin/openzeppelin-contracts": (25000, "Solidity", ["solidity", "smart-contracts", "security"]),
    "ethereum/solidity":                 (23000, "C++", ["solidity", "language", "smart-contracts"]),
    "hyperledger/fabric":                (16000, "Go", ["hyperledger", "blockchain", "enterprise"]),

    # 🖥️ OS, Languages & Compilers
    "torvalds/linux":                    (181000, "C", ["linux", "kernel", "operating-system"]),
    "python/cpython":                    (63000, "Python", ["python", "language", "interpreter"]),
    "golang/go":                         (124000, "Go", ["go", "language", "concurrent"]),
    "rust-lang/rust":                    (98000, "Rust", ["rust", "language", "systems"]),
    "microsoft/typescript":              (101000, "TypeScript", ["typescript", "language", "compiler"]),
    "apple/swift":                       (67000, "C++", ["swift", "language", "compiler"]),
}

NEW_CATEGORIES = OrderedDict([
    ("🛡️ Security & Privacy",    ["OWASP/CheatSheetSeries", "drduh/macOS-Security-and-Privacy-Guide", "swisskyrepo/PayloadsAllTheThings", "veeral-patel/how-to-secure-anything", "mikewest/https-string", "OWASP/wstg"]),
    ("⚡ DevOps & Infrastructure", ["kubernetes/kubernetes", "ansible/ansible", "docker/compose", "prometheus/prometheus", "grafana/grafana", "hashicorp/terraform", "nvm-sh/nvm"]),
    ("📱 Mobile Development",     ["flutter/flutter", "facebook/react-native", "nicklausw/awesome-flutter", "JStumpp/awesome-android", "vsouza/awesome-ios", "dotnet/maui"]),
    ("🌐 Web Development",        ["facebook/react", "vuejs/vue", "angular/angular", "vercel/next.js", "expressjs/express", "django/django", "tailwindlabs/tailwindcss"]),
    ("🗄️ Data & Databases",      ["redis/redis", "elastic/elasticsearch", "numpy/numpy", "pandas-dev/pandas", "apache/spark", "postgres/postgres", "sqlite/sqlite", "mongodb/mongo"]),
    ("🎮 Game Development",       ["godotengine/godot", "leereilly/games", "Kavex/awesome-gamedev", "HackerPoet/MarbleMarcher"]),
    ("⛓️ Blockchain & Web3",      ["bitcoin/bitcoin", "ethereum/go-ethereum", "OpenZeppelin/openzeppelin-contracts", "ethereum/solidity", "hyperledger/fabric"]),
    ("🖥️ Languages & OS",         ["torvalds/linux", "python/cpython", "golang/go", "rust-lang/rust", "microsoft/typescript", "apple/swift"]),
])

def deep_absorb(c, full_name):
    """Fetch README from raw.githubusercontent.com, store in existing node."""
    url = f"https://github.com/{full_name}"
    node = c.execute("SELECT id, content FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone()
    if not node: return None, 0
    nid, curr_content = node[0], (node[1] or "")

    if len(curr_content) > 500:
        return nid, 0

    owner, repo = full_name.split("/", 1)
    readme_raw = ""
    for branch in ("master", "main"):
        for ext in ("README.md", "README.rst", "readme.md"):
            r = fetch(f"{RAW_BASE}/{full_name}/{branch}/{ext}")
            if r and len(r) > 50:
                readme_raw = r; break
        if readme_raw: break
    if not readme_raw:
        for branch in ("master", "main"):
            r = fetch(f"{RAW_BASE}/{full_name}/{branch}/readme.md")
            if r and len(r) > 50: readme_raw = r; break
        if not readme_raw: return nid, 0

    # Normalize: Unicode NFKC + HTML decode
    readme = normalize_text(html.unescape(readme_raw))

    # Strip markdown for clean summary
    clean_text = strip_markdown(readme)
    lines = clean_text.split("\n")
    summary_parts = []
    current_sec = "overview"
    for line in lines:
        if line.startswith("## "):
            current_sec = line[3:].strip().lower()
        elif line.startswith("# "):
            continue
        elif current_sec in ("overview", "features", "about", "description", "introduction", "getting started", "quick start", "what is this"):
            if len(summary_parts) < 5:
                txt = line.strip()
                if txt and not txt.startswith("!"): summary_parts.append(txt)

    deep_summary = " ".join(summary_parts)[:2000] if summary_parts else readme[:500].strip()

    # Content fingerprint for cross-source dedup
    fp = content_fingerprint(readme)
    readme_content = readme[:50000]

    sql_retry(c, "UPDATE nodes SET summary=?, content=?, updated_at=? WHERE id=?",
              (deep_summary, readme_content, NOW, nid))

    meta = c.execute("SELECT metadata FROM nodes WHERE id=?", (nid,)).fetchone()
    if meta and meta[0]:
        try:
            m = json.loads(meta[0])
            m["readme_size"] = len(readme)
            m["content_fp"] = fp
            sql_retry(c, "UPDATE nodes SET metadata=? WHERE id=?", (json.dumps(m), nid))
        except Exception: pass

    return nid, len(readme)

def main():
    conn = sqlite3.connect(KB, timeout=60, check_same_thread=False)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    conn.execute("PRAGMA wal_autocheckpoint=1000")
    c = conn.cursor()

    global NEW_CATEGORIES

    # Phase 1: Inject all new repo nodes
    print("═══ Resource Pool v2: Phase 1 — Inserting Repos ═══")
    repo_ids = {}
    for cat_name, cat_repos in NEW_CATEGORIES.items():
        for full_name in cat_repos:
            if full_name in NEW_REPOS:
                stars, lang, topics = NEW_REPOS[full_name]
            else:
                continue
            url = f"https://github.com/{full_name}"
            norm_l = normalize_lang(lang)
            meta = {"stars": stars, "language": norm_l, "topics": topics, "source": "resource-pool-v2"}
            rid = insert_node(c, "Repository", full_name,
                f"GitHub: {full_name}, {stars}★, {norm_l}. {', '.join(topics)}",
                url, "github.com", meta)
            repo_ids[full_name] = rid
            print(f"  ✅ {full_name} ({stars}★, {lang})")
    conn.commit()

    # Phase 2: Category nodes + edges
    print(f"\n═══ Resource Pool v2: Phase 2 — Categories ═══")
    root_row = c.execute("SELECT id FROM nodes WHERE metadata LIKE '%resource-hub%' LIMIT 1").fetchone()
    root_id = root_row[0] if root_row else None
    cat_ids = {}
    for cat_name, cat_repos in NEW_CATEGORIES.items():
        nid = insert_node(c, "Resource", f"Resource Pool: {cat_name}",
            f"Curated {cat_name.lower()} resources. {len(cat_repos)} high-quality repos.",
            f"neotrix://resource-pool/{cat_name.lower().replace(' & ','-').replace(' ','-').replace('🛡️','security').replace('⚡','devops').replace('📱','mobile').replace('🌐','web').replace('🗄️','data').replace('🎮','gaming').replace('⛓️','blockchain').replace('🖥️','languages')}",
            "neotrix.local", {"category": cat_name, "type": "resource-category", "count": len(cat_repos)})
        cat_ids[cat_name] = nid
        print(f"  📁 {cat_name} ({len(cat_repos)} repos)")
        for r in cat_repos:
            if r in repo_ids and repo_ids[r]:
                insert_edge(c, nid, repo_ids[r], "contains", 0.9, f"{cat_name} → {r}")
        # Link to root hub
        if root_id: insert_edge(c, root_id, nid, "contains", 1.0)
        conn.commit()

    # Phase 3: Cross-link all 16 categories
    print(f"\n═══ Resource Pool v2: Phase 3 — Cross-linking ═══")
    # Fetch all category nodes
    raw_cats = c.execute("SELECT id, title FROM nodes WHERE node_type='Resource' AND title LIKE 'Resource Pool:%'").fetchall()
    all_cats = [(r[0], r[1]) for r in raw_cats] if raw_cats else [(v, cat_name) for cat_name, v in cat_ids.items()]
    for i in range(len(all_cats)):
        for j in range(i+1, len(all_cats)):
            insert_edge(c, all_cats[i][0], all_cats[j][0], "related_to", 0.5,
                f"Cross-domain: {all_cats[i][1]} ↔ {all_cats[j][1]}")
    conn.commit()

    n_new = len(repo_ids)
    print(f"\n✅ Phase 1-3 complete: {n_new} new repos + {len(NEW_CATEGORIES)} new categories")

    # Phase 4: Deep absorb READMEs
    print(f"\n═══ Resource Pool v2: Phase 4 — Deep Absorb READMEs ═══")
    total_bytes = 0
    absorbed = 0
    skipped = 0
    for full_name in repo_ids:
        nid, size = deep_absorb(c, full_name)
        if nid:
            if size > 0:
                absorbed += 1; total_bytes += size
                print(f"  📖 {full_name}: {size:,} chars")
            else:
                skipped += 1
        conn.commit()
        time.sleep(0.3)

    # Final stats
    all_pool = c.execute("SELECT COUNT(*) FROM nodes WHERE metadata LIKE '%resource-pool%'").fetchone()[0]
    total_readme = c.execute("""SELECT COALESCE(SUM(length(content)), 0) FROM nodes 
        WHERE metadata LIKE '%resource-pool%' AND content != '' AND length(content) > 500""").fetchone()[0]
    total_nodes = c.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]

    conn.close()
    print(f"\n{'═' * 60}")
    print(f"  Resource Pool v2 complete!")
    print(f"  New repos this round: {n_new}")
    print(f"  Total resource pool:  {all_pool}")
    print(f"  README content added: {total_readme:,} chars")
    print(f"  Absorbed: {absorbed}, No README: {skipped}")
    print(f"  KB total nodes: {total_nodes:,}")
    print(f"  Categories: {len(NEW_CATEGORIES)} new + 8 existing = {8 + len(NEW_CATEGORIES)} total")
    print(f"{'═' * 60}")

if __name__ == "__main__":
    from collections import OrderedDict
    main()
