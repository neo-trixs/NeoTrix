#!/usr/bin/env python3
"""NeoTrix Resource Pool: Direct KB injection (no API calls, no rate limits)."""
import sqlite3, json, time, os, hashlib
from nt_normalizer import normalize_text, normalize_lang, content_fingerprint, entity_resolve, normalize_metadata, compute_quality_score, validate_node_type

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())

def ndig(s): return hashlib.md5(s.encode()).hexdigest()[:20]

def insert_node(c, ntype, title, summary, url, domain, meta={}):
    ntype = validate_node_type(ntype)
    kid = f"nt-{ndig(url)}"
    fp = content_fingerprint(summary or url)
    existing = entity_resolve(c, title, url, fp)
    if existing: return existing
    title = normalize_text(title)
    summary = normalize_text(summary)
    meta = normalize_metadata(meta)
    meta["content_fp"] = fp
    meta["quality_score"] = round(compute_quality_score(ntype, len(summary), bool(summary), bool(url)), 3)
    meta_s = json.dumps(meta) if meta else "{}"
    c.execute("""INSERT OR IGNORE INTO nodes
        (id,node_type,title,summary,url,domain,language,confidence,importance,created_at,updated_at,metadata)
        VALUES (?,?,?,?,?,?,'en',1.0,0.7,?,?,?)""",
        (kid, ntype, title, summary, url, domain, NOW, NOW, meta_s))
    return kid

def insert_edge(c, src, tgt, rtype, weight=0.8, desc=""):
    eid = f"re-{ndig(f'{src}{tgt}')}"
    existing = c.execute("SELECT id FROM edges WHERE source_id=? AND target_id=? LIMIT 1", (src, tgt)).fetchone()
    if existing: return
    c.execute("INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) VALUES (?,?,?,?,?,?,?)",
              (eid, src, tgt, rtype, weight, desc, NOW))

REPOS = {
    # 🎵 Music / Audio
    "mikeroyal/Awesome-Music-Production":    (1270, "Python", ["music-production", "audio", "guide"]),
    "ad-si/awesome-music-production":        (870,  "Markdown", ["music", "production", "awesome-list"]),
    "noteflakes/awesome-music":              (2000, "Markdown", ["music", "programming", "resources"]),
    "ciconia/awesome-music":                 (1900, "Markdown", ["music", "awesome-list"]),
    "yt-dlp/yt-dlp":                         (100000, "Python", ["youtube", "downloader", "video", "audio"]),
    "FFmpeg/FFmpeg":                         (48000, "C", ["video", "audio", "codec", "multimedia"]),
    "mpv-player/mpv":                        (29000, "C", ["video-player", "media", "audio"]),
    "HandBrake/HandBrake":                   (18000, "C", ["video", "transcoder", "multimedia"]),

    # 📚 Books / Learning
    "EbookFoundation/free-programming-books":(350000, "Markdown", ["books", "programming", "free", "education"]),
    "getify/You-Dont-Know-JS":               (185000, "Markdown", ["javascript", "book", "learning"]),
    "jwasham/coding-interview-university":   (310000, "Markdown", ["study-plan", "computer-science", "interview"]),
    "ossu/computer-science":                 (175000, "Markdown", ["computer-science", "curriculum", "free"]),
    "krahets/hello-algo":                    (128000, "Java", ["algorithms", "data-structures", "animation"]),
    "kamranahmedse/developer-roadmap":       (305000, "Markdown", ["roadmap", "developer", "guide"]),
    "practical-tutorials/project-based-learning": (210000, "Markdown", ["tutorials", "project-based", "learning"]),

    # 🔬 Research & Science
    "papers-we-love/papers-we-love":         (91000, "Markdown", ["papers", "computer-science", "research"]),
    "rasbt/LLMs-from-scratch":               (100000, "Jupyter", ["llm", "deep-learning", "tutorial"]),
    "microsoft/generative-ai-for-beginners": (113000, "Jupyter", ["generative-ai", "course", "microsoft"]),
    "mlpack/mlpack":                         (5200, "C++", ["machine-learning", "library", "fast"]),
    "academic/awesome-datascience":          (26000, "Markdown", ["data-science", "awesome-list"]),
    "ujjwalkarn/Machine-Learning-Tutorials": (16000, "Markdown", ["machine-learning", "tutorials"]),
    "ZuzooVn/machine-learning-for-software-engineers": (28000, "Markdown", ["ml", "study-plan"]),

    # 🎨 Images / Graphics / Design
    "danistefanovic/build-your-own-x":       (330000, "Markdown", ["tutorial", "build", "diy"]),
    "bradtraversy/design-resources-for-developers": (60000, "Markdown", ["design", "resources", "ui"]),
    "terkelg/awesome-creative-coding":       (14000, "Markdown", ["creative-coding", "generative-art"]),
    "ohansfavour/awesome-photography":       (1200, "Markdown", ["photography", "resources"]),
    "thedaviddias/Resources-Front-End-Beginner": (42000, "Markdown", ["frontend", "resources"]),
    "dypsilon/frontend-dev-bookmarks":       (45000, "Markdown", ["frontend", "bookmarks"]),

    # 🎬 Video & Multimedia
    "videolan/vlc":                          (15000, "C", ["media-player", "video", "streaming"]),
    "anibali/awesome-video":                 (2800, "Markdown", ["video", "technology", "awesome-list"]),

    # 🧠 Knowledge Management
    "siyuan-note/siyuan":                    (30000, "TypeScript", ["knowledge-management", "note-taking"]),
    "logseq/logseq":                         (35000, "Clojure", ["knowledge-base", "privacy", "note-taking"]),
    "antoineAa/awesome-personal-knowledge-management": (1200, "Markdown", ["pkm", "knowledge-management"]),
    "brettkromkamp/awesome-knowledge-management":      (800, "Markdown", ["knowledge-management", "awesome-list"]),

    # 🤖 AI Tools & Agents
    "Significant-Gravitas/AutoGPT":          (185000, "Python", ["ai-agent", "autonomous", "gpt"]),
    "langflow-ai/langflow":                  (151000, "Python", ["ai", "workflow", "visual"]),
    "langgenius/dify":                       (148000, "TypeScript", ["llm", "platform", "ai-app"]),
    "n8n-io/n8n":                            (130000, "TypeScript", ["workflow", "automation"]),
    "firecrawl/firecrawl":                   (144000, "TypeScript", ["web-scraping", "ai", "crawler"]),
    "sherlock-project/sherlock":             (86000, "Python", ["osint", "search", "social-media"]),

    # 🛡️ Security & Privacy
    "sindresorhus/awesome":                  (350000, "Markdown", ["awesome-list", "curated", "resources"]),
    "awesome-selfhosted/awesome-selfhosted": (220000, "Markdown", ["self-hosted", "privacy", "software"]),
    "ripienaar/free-for-dev":                (92000, "Markdown", ["free", "developer-tools", "tiers"]),
    "davemachado/public-api-lists":          (12000, "Markdown", ["api", "public", "resources"]),
}

CATEGORIES = {
    "🎵 Music & Audio":      ["mikeroyal/Awesome-Music-Production", "ad-si/awesome-music-production", "noteflakes/awesome-music", "ciconia/awesome-music", "yt-dlp/yt-dlp", "FFmpeg/FFmpeg", "mpv-player/mpv", "HandBrake/HandBrake"],
    "📚 Books & Learning":    ["EbookFoundation/free-programming-books", "getify/You-Dont-Know-JS", "jwasham/coding-interview-university", "ossu/computer-science", "krahets/hello-algo", "kamranahmedse/developer-roadmap", "practical-tutorials/project-based-learning"],
    "🔬 Research & Science":  ["papers-we-love/papers-we-love", "rasbt/LLMs-from-scratch", "microsoft/generative-ai-for-beginners", "mlpack/mlpack", "academic/awesome-datascience", "ujjwalkarn/Machine-Learning-Tutorials", "ZuzooVn/machine-learning-for-software-engineers"],
    "🎨 Design & Graphics":   ["danistefanovic/build-your-own-x", "bradtraversy/design-resources-for-developers", "terkelg/awesome-creative-coding", "ohansfavour/awesome-photography", "thedaviddias/Resources-Front-End-Beginner", "dypsilon/frontend-dev-bookmarks"],
    "🎬 Video & Multimedia":  ["videolan/vlc", "anibali/awesome-video"],
    "🧠 Knowledge Management": ["siyuan-note/siyuan", "logseq/logseq", "antoineAa/awesome-personal-knowledge-management", "brettkromkamp/awesome-knowledge-management"],
    "🤖 AI Agents & Tools":   ["Significant-Gravitas/AutoGPT", "langflow-ai/langflow", "langgenius/dify", "n8n-io/n8n", "firecrawl/firecrawl", "sherlock-project/sherlock"],
    "📦 Resource Aggregators":["sindresorhus/awesome", "awesome-selfhosted/awesome-selfhosted", "ripienaar/free-for-dev", "davemachado/public-api-lists"],
}

def main():
    conn = sqlite3.connect(KB)
    c = conn.cursor()
    total = 0

    # Phase 1: Insert all repo nodes
    print("═══ Resource Pool: Phase 1 — Inserting Repos ═══")
    repo_ids = {}
    for full_name, (stars, lang, topics) in REPOS.items():
        url = f"https://github.com/{full_name}"
        norm_l = normalize_lang(lang)
        meta = {"stars": stars, "language": norm_l, "topics": topics, "source": "resource-pool-v1"}
        rid = insert_node(c, "Repository", full_name,
            f"GitHub repository {full_name}: {stars}★, {norm_l}. Topics: {', '.join(topics)}",
            url, "github.com", meta)
        repo_ids[full_name] = rid
        if rid: total += 1; print(f"  ✅ {full_name} ({stars}★, {norm_l})")

    conn.commit()
    print(f"\n═══ Resource Pool: Phase 2 — Building Categories ═══")

    # Phase 2: Create category nodes and link repos
    cat_ids = {}
    for cat_name, repos in CATEGORIES.items():
        nid = insert_node(c, "Resource", f"Resource Pool: {cat_name}",
            f"Curated collection of {cat_name.lower()} resources. Contains {len(repos)} repositories.",
            f"neotrix://resource-pool/{cat_name.lower().replace(' & ','-').replace(' ','-')}",
            "neotrix.local", {"category": cat_name, "type": "resource-category", "count": len(repos)})
        cat_ids[cat_name] = nid
        print(f"  📁 {cat_name}")
        for r in repos:
            if r in repo_ids and repo_ids[r]:
                insert_edge(c, nid, repo_ids[r], "contains", 0.9, f"{cat_name} → {r}")
        conn.commit()

    print(f"\n═══ Resource Pool: Phase 3 — Cross-linking ═══")
    # Phase 3: Cross-link categories
    cats = list(CATEGORIES.keys())
    for i in range(len(cats)):
        for j in range(i+1, len(cats)):
            if cat_ids.get(cats[i]) and cat_ids.get(cats[j]):
                insert_edge(c, cat_ids[cats[i]], cat_ids[cats[j]], "related_to", 0.5,
                    f"Cross-domain: {cats[i]} ↔ {cats[j]}")
    conn.commit()

    # Phase 4: Root resource hub
    root = insert_node(c, "Resource", "NeoTrix Resource Hub",
        "Central resource pool: curated collections of music, books, research, design, video, knowledge management, and AI tools from GitHub.",
        "neotrix://resource-pool", "neotrix.local",
        {"type": "resource-hub", "version": "1.0", "categories": len(CATEGORIES), "repos": len(REPOS)})
    for nid in cat_ids.values():
        if nid: insert_edge(c, root, nid, "contains", 1.0)
    conn.commit()

    conn.close()
    print(f"\n✅ Resource pool complete! {total} new repos + {len(CATEGORIES)} categories inserted.")
    print(f"   KB now has resources across: Music, Books, Research, Design, Video, KM, AI, Aggregators.")

if __name__ == "__main__":
    main()
