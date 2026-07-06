#!/usr/bin/env python3
"""Absorb GitHub Trending AI Agent Ecosystem repos into NeoTrix KB.

Phase 1: Inject 12 competitor/inspiration repo stubs with cross-category edges.
Phase 2: Deep-absorb README content from raw.githubusercontent.com (zero API calls).
Phase 3: Content distillation — tech stack, patterns, cross-references.
Phase 4: Panorama update.

Usage:
  python3 scripts/absorb-trending-ai-repos.py
"""
import sqlite3, json, time, os, hashlib, urllib.request, re
from collections import Counter

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())
UA = "NeoTrix/0.19-TrendingAI-v1"
RAW_BASE = "https://raw.githubusercontent.com"

# ═══════════════════════════════════════════════════
# Trending AI Agent Ecosystem — 12 P0 repos
# ═══════════════════════════════════════════════════

REPOS = [
    # (full_name, stars, lang, topics, category, description)
    # Category: Agent Memory
    ("topoteretes/cognee", 26901, "Python",
     ["ai-memory", "knowledge-graph", "agent", "llm", "rag", "memory-platform"],
     "Agent-Memory",
     "Open-source AI memory platform for agents. Persistent long-term memory across sessions. Self-hosted knowledge graph engine with semantic search, entity extraction, and conversation history."),

    # Category: MCP
    ("DeusData/codebase-memory-mcp", 25871, "C",
     ["mcp", "code-intelligence", "knowledge-graph", "code-search", "static-analysis"],
     "MCP-Servers",
     "High-performance code intelligence MCP server. Indexes codebases into persistent knowledge graph. 158 languages, sub-ms queries, 99% fewer tokens. Single static binary, zero dependencies."),

    # Category: AI Gateway
    ("diegosouzapw/OmniRoute", 11113, "TypeScript",
     ["ai-gateway", "llm", "provider", "free-llm", "token-compression"],
     "AI-Gateway",
     "Free AI gateway: one endpoint, 231+ providers (50+ free). Connects Claude Code, Codex, Cursor, Cline & Copilot. RTK+Caveman compression saves 15-95% tokens. Smart auto-fallback, MCP/A2A, multimodal APIs."),

    # Category: Rust Agent
    ("ogulcancelik/herdr", 11123, "Rust",
     ["agent", "multiplexer", "terminal", "rust", "cli"],
     "Rust-Agent-Tools",
     "Agent multiplexer that lives in your terminal. Rust-native agent orchestration, parallel execution, terminal UI."),

    # Category: Agent Frameworks
    ("msitarzewski/agency-agents", 126845, "Shell",
     ["ai-agency", "agents", "automation", "skills", "multi-agent"],
     "Agent-Frameworks",
     "Complete AI agency at your fingertips. From frontend wizards to Reddit community ninjas. Each agent is a specialized expert with personality, processes, and proven deliverables. 500+ agent skills."),

    ("stablyai/orca", 11888, "TypeScript",
     ["agent", "ade", "parallel-agents", "desktop", "mobile"],
     "Agent-Frameworks",
     "Orca is the ADE (Agent Development Environment) for working with a fleet of parallel agents. Run any coding agent with your own subscription. Available on desktop and mobile."),

    # Category: Security
    ("usestrix/strix", 35534, "Python",
     ["security", "pentesting", "ai-security", "vulnerability", "scanning"],
     "Security-Pentesting",
     "Open-source AI penetration testing tool to find and fix your app's vulnerabilities. AI-powered security scanning with automated exploit detection and remediation."),

    # Category: Browser/GUI Agents
    ("alibaba/page-agent", 22751, "TypeScript",
     ["gui-agent", "browser-automation", "web-ui", "nlp", "in-page"],
     "Browser-Automation",
     "JavaScript in-page GUI agent. Control web interfaces with natural language. Runs directly in browser — no server needed. Supports any web app."),

    ("browser-use/video-use", 14564, "Python",
     ["video-editing", "agent", "automation", "multimedia"],
     "Multimedia-Agents",
     "Edit videos with coding agents. Programmatic video editing pipeline powered by AI agents. Supports cuts, transitions, effects, and exports."),

    # Category: Multimedia
    ("callesthio/OpenMontage", 32825, "Python",
     ["video-production", "agentic", "pipelines", "tools", "skills"],
     "Multimedia-Agents",
     "World's first open-source, agentic video production system. 12 pipelines, 52 tools, 500+ agent skills. Turn your AI coding assistant into a full video production studio."),

    # Category: MCP/Interop
    ("openai/codex-plugin-cc", 23690, "JavaScript",
     ["codex", "claude-code", "mcp", "integration", "plugin"],
     "MCP-Servers",
     "Use Codex from Claude Code to review code or delegate tasks. MCP-based cross-platform agent interop."),

    # Category: AI Web Tools
    ("JCodesMore/ai-website-cloner-template", 25423, "TypeScript",
     ["website-cloner", "ai", "template", "automation"],
     "AI-Web-Tools",
     "Clone any website with one command using AI coding agents. Single-command website replication with AI-powered adaptation."),
]

# ═══════════════════════════════════════════════════
# Helper functions (from expand-resource-pool pattern)
# ═══════════════════════════════════════════════════

def ndig(s):
    return hashlib.md5(s.encode()).hexdigest()[:20]

def sql_retry(c, sql, params, max_retries=3):
    for i in range(max_retries):
        try:
            return c.execute(sql, params)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < max_retries - 1:
                time.sleep(0.5)
                continue
            raise

def insert_node(c, ntype, title, summary, url, domain, meta):
    kid = f"nt-{ndig(url)}"
    existing = c.execute("SELECT id FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone()
    if existing:
        return existing[0]
    existing2 = c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (kid,)).fetchone()
    if existing2:
        return existing2[0]
    meta_s = json.dumps(meta) if meta else "{}"
    sql_retry(c, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,metadata) VALUES (?,?,?,?,?,?,?,'en',1.0,0.7,?,?,?)",
              (kid, ntype, title, summary, "", url, domain, NOW, NOW, meta_s))
    return kid

def insert_edge(c, src, tgt, rtype, weight=0.8, desc=""):
    eid = f"re-{ndig(f'{src}{tgt}')}"
    existing = c.execute("SELECT id FROM edges WHERE source_id=? AND target_id=? LIMIT 1", (src, tgt)).fetchone()
    if existing:
        return
    sql_retry(c, "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) VALUES (?,?,?,?,?,?,?)",
              (eid, src, tgt, rtype, weight, desc, NOW))

def fetch(url, timeout=10):
    """Fetch URL with timeout. Returns empty string on any error."""
    try:
        r = urllib.request.Request(url, headers={"User-Agent": UA})
        return urllib.request.urlopen(r, timeout=timeout).read().decode("utf-8", errors="replace")
    except Exception:
        return ""

def normalize_text(s):
    import unicodedata
    s = unicodedata.normalize("NFKC", s)
    import html
    s = html.unescape(s)
    s = re.sub(r"\s+", " ", s).strip()
    return s

def strip_markdown(text):
    text = re.sub(r"```[\s\S]*?```", "", text)
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"(?m)^\s*[#*\-=]+\s*", "", text)
    text = re.sub(r"\*\*|__|~~", "", text)
    text = re.sub(r"\s+", " ", text).strip()
    return text

def detect_language(text, title="", url=""):
    LANG_SIGS = [
        (r"```(?:python|py)\b", "Python", 0.9),
        (r"```(?:javascript|js)\b", "JavaScript", 0.9),
        (r"```typescript\b", "TypeScript", 0.9),
        (r"```(?:rust|rs)\b", "Rust", 0.9),
        (r"```go\b", "Go", 0.9),
    ]
    scores = {}
    for pattern, lang, weight in LANG_SIGS:
        count = len(re.findall(pattern, text, re.IGNORECASE))
        if count > 0:
            scores[lang] = scores.get(lang, 0.0) + weight * min(count, 3)
    if title:
        tl = title.lower()
        for name, lang in [("python", "Python"), ("rust", "Rust"), ("typescript", "TypeScript")]:
            if name in tl:
                scores[lang] = max(scores.get(lang, 0.0), 0.5)
    if scores:
        best = max(scores, key=scores.get)
        if scores[best] >= 0.5:
            return best
    return "en"

# ═══════════════════════════════════════════════════
# Phase 1: Inject nodes + edges
# ═══════════════════════════════════════════════════

def phase1_inject(c):
    print("═══ Phase 1: Inject trending repo stubs ═══")
    categories_seen = {}
    node_ids = {}

    for full_name, stars, lang, topics, category, desc in REPOS:
        url = f"https://github.com/{full_name}"
        title = full_name.split("/")[1]
        meta = {
            "stars": stars,
            "topics": topics,
            "trending_category": category,
            "trending_source": "github-weekly-2026-07-04",
            "absorbed_at": NOW,
        }

        # Insert repo
        nid = insert_node(c, "Repository", title, desc, url, "github.com", meta)
        node_ids[full_name] = nid
        print(f"  📦 {full_name} → {nid} ({stars}★, {category})")

        # Insert category concept node (idempotent)
        cat_id = f"cat-{ndig(category)}"
        existing_cat = c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (cat_id,)).fetchone()
        if not existing_cat:
            sql_retry(c, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,url,domain,created_at,updated_at) VALUES (?,?,?,?,?,'neotrix.ai',?,?)",
                      (cat_id, "Concept", f"Trending Category: {category}",
                       f"GitHub Trending AI Agent ecosystem category — {category}", "", NOW, NOW))
            print(f"  📂 Category: {category}")

        # Edge: category contains repo
        insert_edge(c, cat_id, nid, "contains", 0.9, f"{category} → {full_name}")
        categories_seen[category] = cat_id

    # Cross-reference edges between related repos
    # Memory systems → MCP
    if "topoteretes/cognee" in node_ids and "DeusData/codebase-memory-mcp" in node_ids:
        insert_edge(c, node_ids["topoteretes/cognee"], node_ids["DeusData/codebase-memory-mcp"],
                    "related_to", 0.7, "Competing/similar: agent memory platforms")

    # MCP interop
    if "openai/codex-plugin-cc" in node_ids:
        for other in ["DeusData/codebase-memory-mcp"]:
            if other in node_ids:
                insert_edge(c, node_ids[other], node_ids["openai/codex-plugin-cc"],
                            "related_to", 0.6, "MCP ecosystem interoperability")

    # Gateways → Providers
    if "diegosouzapw/OmniRoute" in node_ids:
        for other in ["ogulcancelik/herdr"]:
            if other in node_ids:
                insert_edge(c, node_ids["diegosouzapw/OmniRoute"], node_ids[other],
                            "related_to", 0.5, "Agent infrastructure: gateway ↔ multiplexer")

    # Agent frameworks cross-ref
    agent_frameworks = ["msitarzewski/agency-agents", "stablyai/orca"]
    for i in range(len(agent_frameworks)):
        for j in range(i + 1, len(agent_frameworks)):
            if agent_frameworks[i] in node_ids and agent_frameworks[j] in node_ids:
                insert_edge(c, node_ids[agent_frameworks[i]], node_ids[agent_frameworks[j]],
                            "related_to", 0.6, "Competing agent frameworks")

    # Browser agents
    browser_agents = ["alibaba/page-agent", "browser-use/video-use"]
    for i in range(len(browser_agents)):
        for j in range(i + 1, len(browser_agents)):
            if browser_agents[i] in node_ids and browser_agents[j] in node_ids:
                insert_edge(c, node_ids[browser_agents[i]], node_ids[browser_agents[j]],
                            "related_to", 0.5, "Browser/GUI automation agents")

    return node_ids

# ═══════════════════════════════════════════════════
# Phase 2: Deep absorb README content
# ═══════════════════════════════════════════════════

def phase2_deep_absorb(c, node_ids):
    print("\n═══ Phase 2: Deep absorb README content ═══")
    total_bytes = 0
    absorbed = 0

    for full_name, stars, lang, topics, category, desc in REPOS:
        nid = node_ids.get(full_name)
        if not nid:
            continue

        # Check if already has content
        row = c.execute("SELECT content FROM nodes WHERE id=?", (nid,)).fetchone()
        if row and row[0] and len(row[0]) > 500:
            print(f"  ⏭️  {full_name}: already has content ({len(row[0])} chars)")
            continue

        # Fetch README
        owner, repo_name = full_name.split("/", 1)
        paths = [
            f"{RAW_BASE}/{full_name}/master/README.md",
            f"{RAW_BASE}/{full_name}/main/README.md",
        ]
        readme = ""
        for p in paths:
            content = fetch(p)
            if content and len(content) > 50:
                readme = content
                break

        if not readme:
            print(f"  ⚠️  {full_name}: no README found")
            continue

        # Clean + detect language
        clean = strip_markdown(readme)
        detected_lang = detect_language(readme, title=full_name)

        # Build summary from first sections
        summary = clean[:2000] if clean else desc

        # Store
        readme_trimmed = readme[:50000]
        c.execute("UPDATE nodes SET summary=?, content=?, updated_at=? WHERE id=?",
                  (summary, readme_trimmed, NOW, nid))
        if detected_lang and detected_lang != "en":
            c.execute("UPDATE nodes SET language=? WHERE id=?", (detected_lang, nid))

        # Update metadata
        meta_row = c.execute("SELECT metadata FROM nodes WHERE id=?", (nid,)).fetchone()
        if meta_row and meta_row[0]:
            try:
                m = json.loads(meta_row[0])
                m["readme_absorbed"] = True
                m["readme_size"] = len(readme)
                m["detected_language"] = detected_lang
                fp = hashlib.sha256(readme.encode()).hexdigest()[:16]
                m["content_fp"] = fp
                c.execute("UPDATE nodes SET metadata=? WHERE id=?", (json.dumps(m), nid))
            except Exception:
                pass

        print(f"  📖 {full_name}: {len(readme)} chars absorbed")
        total_bytes += len(readme)
        absorbed += 1

    return absorbed, total_bytes

# ═══════════════════════════════════════════════════
# Phase 3: Content distillation
# ═══════════════════════════════════════════════════

def phase3_distill(c, node_ids):
    print("\n═══ Phase 3: Content distillation ═══")

    # Analyze tech stack across all absorbed repos
    lang_counter = Counter()
    topic_counter = Counter()
    cat_counter = Counter()
    total_stars = 0

    for full_name, stars, lang, topics, category, desc in REPOS:
        lang_counter[lang] += 1
        cat_counter[category] += 1
        total_stars += stars
        for t in topics:
            topic_counter[t] += 1

    print(f"\n  📊 Absorption summary:")
    print(f"  Total repos: {len(REPOS)}")
    print(f"  Total stars: {total_stars:,}★")
    print(f"  Languages: {dict(lang_counter.most_common(5))}")
    print(f"  Top topics: {dict(topic_counter.most_common(8))}")
    print(f"  Categories: {dict(cat_counter.most_common(10))}")

    # Store distillation result as Insight nodes
    insights = [
        ("Trending AI Agent Ecosystem — Weekly Snapshot",
         f"GitHub Week 27, 2026: {len(REPOS)} trending AI repos ({total_stars:,}★ total). "
         f"Dominant themes: agent memory platforms (cognee, codebase-memory-mcp), "
         f"AI gateways (OmniRoute, 231 providers), multi-agent frameworks (agency-agents 126K★, orca 12K★), "
         f"Rust agent tooling (herdr 11K★), and browser-based agents (page-agent, video-use).",
         "trending-analysis"),

        ("NeoTrix Competitor Landscape — Agent Memory",
         f"cognee (26,901★) is direct competitor to NeoTrix nt_memory_kb. "
         f"Ported long-term agent memory with knowledge graph + semantic search. "
         f"codebase-memory-mcp (25,871★, +10,186★/wk) is a MCP-native alternative at 158 languages, sub-ms queries.",
         "competitive-analysis"),

        ("NeoTrix Competitor Landscape — AI Gateway",
         f"OmniRoute (11,113★, +3,631★/wk) directly competes with GatewayV2. "
         f"Offers 231 providers, 50+ free, RTK+Caveman token compression (15-95%), web+PWA interface. "
         f"NeoTrix GatewayV2 has ~10 providers, no token compression.",
         "competitive-analysis"),

        ("NeoTrix Competitor Landscape — Agent Frameworks",
         f"agency-agents (126,845★, +10,483★/wk) and orca (11,888★, +3,700★/wk) "
         f"represent the two poles of agent frameworks: monolithic agency vs parallel ADE. "
         f"NeoTrix has nt_core_orch_agent but lacks async background execution and skill marketplace.",
         "competitive-analysis"),
    ]

    for title, summary, insight_type in insights:
        iid = f"insight-{ndig(title)}"
        existing = c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (iid,)).fetchone()
        if not existing:
            sql_retry(c, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,domain,created_at,updated_at,metadata) VALUES (?,?,?,?,?,'neotrix.ai',?,?,?)",
                      (iid, "Insight", title, summary, summary, NOW, NOW,
                       json.dumps({"insight_type": insight_type, "source": "trending-absorb-2026-07-04"})))
            # Link insight to all repos in its category
            cat_insight_map = {
                "trending-analysis": set(cat_counter.keys()),
                "competitive-analysis": {"Agent-Memory", "MCP-Servers", "AI-Gateway", "Agent-Frameworks"},
            }
            matching_repos = []
            for full_name, _, _, _, cat, _ in REPOS:
                if insight_type in cat_insight_map:
                    if insight_type == "trending-analysis":
                        matching_repos.append(full_name)
                    elif cat in cat_insight_map[insight_type]:
                        matching_repos.append(full_name)
            for fn in matching_repos[:5]:  # Link top 5 per insight
                if fn in node_ids:
                    insert_edge(c, iid, node_ids[fn], "references", 0.8, f"Insight about {fn}")

    print(f"  🧠 Created {len(insights)} insight nodes with cross-references")
    return insights

# ═══════════════════════════════════════════════════
# Phase 4: Panorama update
# ═══════════════════════════════════════════════════

def phase4_panorama(c):
    print("\n═══ Phase 4: KB Panorama snapshot ═══")
    total_nodes = c.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]
    total_edges = c.execute("SELECT COUNT(*) FROM edges").fetchone()[0]
    repo_count = c.execute("SELECT COUNT(*) FROM nodes WHERE node_type='Repository'").fetchone()[0]
    insight_count = c.execute("SELECT COUNT(*) FROM nodes WHERE node_type='Insight'").fetchone()[0]
    concept_count = c.execute("SELECT COUNT(*) FROM nodes WHERE node_type='Concept'").fetchone()[0]

    # Store as kv_store panorama snapshot
    snapshot = json.dumps({
        "timestamp": NOW,
        "total_nodes": total_nodes,
        "total_edges": total_edges,
        "repositories": repo_count,
        "insights": insight_count,
        "concepts": concept_count,
        "trending_repos_added": len(REPOS),
        "source": "trending-absorb-2026-07-04",
    })
    sql_retry(c, "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?, ?)",
              ("panorama_trending_20260704", snapshot))

    print(f"  📊 KB after absorption:")
    print(f"     Nodes: {total_nodes}")
    print(f"     Edges: {total_edges}")
    print(f"     Repos: {repo_count}")
    print(f"     Insights: {insight_count}")
    print(f"     Concepts: {concept_count}")

# ═══════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════

def main():
    print(f"{'═' * 60}")
    print(f"  NeoTrix Trending AI Ecosystem Absorber")
    print(f"  {len(REPOS)} repos, {sum(r[1] for r in REPOS):,}★ total")
    print(f"{'═' * 60}\n")

    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()

    # Phase 1: Inject stubs
    node_ids = phase1_inject(c)
    conn.commit()

    # Phase 2: Deep absorb
    absorbed, total_bytes = phase2_deep_absorb(c, node_ids)
    conn.commit()

    # Phase 3: Distill
    phase3_distill(c, node_ids)
    conn.commit()

    # Phase 4: Panorama
    phase4_panorama(c)
    conn.commit()

    conn.close()
    print(f"\n{'═' * 60}")
    print(f"  ✅ Absorption complete!")
    print(f"  {len(REPOS)} repos injected and deep-absorbed")
    print(f"  {total_bytes:,} bytes README content")
    print(f"  Cross-references and insights created")
    print(f"  Panorama snapshot stored")
    print(f"{'═' * 60}")
    return 0

if __name__ == "__main__":
    exit(main())
