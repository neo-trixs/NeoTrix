#!/usr/bin/env python3
"""Phase 3: Distillation + Panorama for 12 trending AI repos.
Creates Insight nodes, cross-references, and KB panorama snapshot.
Usage: python3 scripts/absorb-trending-ai-distill.py"""
import sqlite3, json, time, os, hashlib
from collections import Counter

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())

def ndig(s): return hashlib.md5(s.encode()).hexdigest()[:20]
def retry(c, sql, p, n=3):
    for i in range(n):
        try: return c.execute(sql, p)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < n-1:
                time.sleep(0.5); continue
            raise

REPOS = [
    # (full_name, stars, lang, topics, category)
    ("topoteretes/cognee", 26901, "Python",
     ["ai-memory","knowledge-graph","agent","llm","rag","memory-platform"],
     "Agent-Memory"),
    ("DeusData/codebase-memory-mcp", 25871, "C",
     ["mcp","code-intelligence","knowledge-graph","code-search"],
     "MCP-Servers"),
    ("diegosouzapw/OmniRoute", 11113, "TypeScript",
     ["ai-gateway","llm","provider","free-llm","token-compression"],
     "AI-Gateway"),
    ("ogulcancelik/herdr", 11123, "Rust",
     ["agent","multiplexer","terminal","rust","cli"],
     "Rust-Agent-Tools"),
    ("msitarzewski/agency-agents", 126845, "Shell",
     ["ai-agency","agents","automation","skills","multi-agent"],
     "Agent-Frameworks"),
    ("stablyai/orca", 11888, "TypeScript",
     ["agent","ade","parallel-agents","desktop","mobile"],
     "Agent-Frameworks"),
    ("usestrix/strix", 35534, "Python",
     ["security","pentesting","ai-security","vulnerability"],
     "Security-Pentesting"),
    ("alibaba/page-agent", 22751, "TypeScript",
     ["gui-agent","browser-automation","web-ui","nlp","in-page"],
     "Browser-Automation"),
    ("browser-use/video-use", 14564, "Python",
     ["video-editing","agent","automation","multimedia"],
     "Multimedia-Agents"),
    ("callesthio/OpenMontage", 32825, "Python",
     ["video-production","agentic","pipelines","tools","skills"],
     "Multimedia-Agents"),
    ("openai/codex-plugin-cc", 23690, "JavaScript",
     ["codex","claude-code","mcp","integration","plugin"],
     "MCP-Servers"),
    ("JCodesMore/ai-website-cloner-template", 25423, "TypeScript",
     ["website-cloner","ai","template","automation"],
     "AI-Web-Tools"),
]

def insert_edge(c, s, t, r, w=0.8, d=""):
    eid = f"re-{ndig(f'{s}{t}')}"
    if c.execute("SELECT id FROM edges WHERE source_id=? AND target_id=? LIMIT 1", (s, t)).fetchone(): return
    retry(c, "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) VALUES (?,?,?,?,?,?,?)",
          (eid, s, t, r, w, d, NOW))

def main():
    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()

    # Build node_id map
    nids = {}
    for fn, _, _, _, _ in REPOS:
        url = f"https://github.com/{fn}"
        row = c.execute("SELECT id FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone()
        if row: nids[fn] = row[0]

    # ── Stats ──
    lang_c = Counter(); topic_c = Counter(); cat_c = Counter(); total_stars = 0
    for fn, stars, lang, topics, cat in REPOS:
        lang_c[lang] += 1; cat_c[cat] += 1; total_stars += stars
        for t in topics: topic_c[t] += 1

    print("═══ Phase 3: Content Distillation ═══")
    print(f"\n  {len(REPOS)} repos, {total_stars:,}★ total")
    print(f"  Languages: {dict(lang_c.most_common(6))}")
    print(f"  Topics: {dict(topic_c.most_common(8))}")
    print(f"  Categories: {dict(cat_c.most_common(9))}")

    # ── Insight nodes ──
    insights = [
        ("2026-07-04 Trending AI Agent Ecosystem Weekly Snapshot",
         f"GitHub Week 27, 2026: {len(REPOS)} trending AI repos ({total_stars:,}★ total). "
         f"Dominant themes: agent memory platforms (cognee, codebase-memory-mcp), "
         f"AI gateways (OmniRoute 231 providers), multi-agent frameworks (agency-agents 126K★, orca 12K★), "
         f"Rust agent tooling (herdr 11K★), browser agents (page-agent, video-use).",
         "trending-analysis",
         ["topoteretes/cognee","DeusData/codebase-memory-mcp","diegosouzapw/OmniRoute",
          "ogulcancelik/herdr","msitarzewski/agency-agents","stablyai/orca",
          "alibaba/page-agent","browser-use/video-use","callesthio/OpenMontage"]),

        ("Competitor Analysis: Agent Memory (cognee vs nt_memory_kb)",
         f"cognee (26,901★) is the most direct competitor to NeoTrix nt_memory_kb. "
         f"It provides persistent long-term agent memory with knowledge graph + semantic search. "
         f"codebase-memory-mcp (25,871★, +10,186★/wk) provides code-specific memory via MCP. "
         f"Both indexed at sub-ms with single static binary deployment.",
         "competitive-analysis",
         ["topoteretes/cognee","DeusData/codebase-memory-mcp"]),

        ("Competitor Analysis: AI Gateway (OmniRoute vs GatewayV2)",
         f"OmniRoute (11,113★, +3,631★/wk) directly competes with NeoTrix GatewayV2. "
         f"Key gaps: 231 providers (NeoTrix ~10), 50+ free, RTK+Caveman token compression (15-95%), "
         f"web+PWA client. NeoTrix needs provider count expansion and token compression.",
         "competitive-analysis",
         ["diegosouzapw/OmniRoute"]),

        ("Competitor Analysis: Agent Frameworks (agency-agents, orca vs nt_core_orch_agent)",
         f"agency-agents (126,845★, +10,483★/wk) dominates the multi-agent space with 500+ skills "
         f"and a complete agency model. orca (11,888★) focuses on parallel agent ADE with desktop+mobile. "
         f"NeoTrix nt_core_orch_agent is synchronous only — needs async background execution.",
         "competitive-analysis",
         ["msitarzewski/agency-agents","stablyai/orca"]),

        ("NeoTrix Evolution Opportunity: Rust Agent Multiplexer (herdr)",
         f"herdr (11,123★, Rust) proves there is demand for a Rust-native agent multiplexer. "
         f"NeoTrix's entire stack is Rust, but agent orchestration lacks a multiplexer pattern. "
         f"Potential: integrate herdr-style multiplexing into nt_core_orch_agent.",
         "architecture-insight",
         ["ogulcancelik/herdr"]),

        ("NeoTrix Evolution Opportunity: AI Security Testing (strix)",
         f"strix (35,534★, +7,567★/wk) is an open-source AI pentesting tool. "
         f"NeoTrix has nt_shield but no AI-powered security scanning. "
         f"Potential: add strix-style agent-driven vulnerability detection to nt_shield.",
         "architecture-insight",
         ["usestrix/strix"]),
    ]

    for title, summary, itype, related in insights:
        iid = f"insight-{ndig(title)}"
        if c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (iid,)).fetchone():
            print(f"  ⏭️  Insight exists: {title[:40]}...")
            continue
        retry(c, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,domain,created_at,updated_at,metadata) VALUES (?,?,?,?,?,'neotrix.ai',?,?,?)",
              (iid, "Insight", title, summary, summary, NOW, NOW,
               json.dumps({"insight_type": itype, "source": "trending-absorb-2026-07-04"})))

        # Link insight to related repos
        for fn in related:
            if fn in nids:
                insert_edge(c, iid, nids[fn], "references", 0.8, f"Insight about {fn}")
        print(f"  🧠 {itype}: {title[:50]}...")

    # ── Panorama snapshot ──
    total_nodes = c.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]
    total_edges = c.execute("SELECT COUNT(*) FROM edges").fetchone()[0]
    repos_count = c.execute("SELECT COUNT(*) FROM nodes WHERE node_type='Repository'").fetchone()[0]

    snapshot = json.dumps({
        "timestamp": NOW,
        "total_nodes": total_nodes,
        "total_edges": total_edges,
        "repositories": repos_count,
        "trending_repos_added": 12,
        "source": "trending-absorb-2026-07-04",
    })
    retry(c, "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
          ("panorama", "trending_20260704", snapshot, NOW))

    conn.commit()
    conn.close()

    print(f"\n═══ Phase 4: KB Panorama ═══")
    print(f"  Nodes: {total_nodes:,}")
    print(f"  Edges: {total_edges:,}")
    print(f"  Repos: {repos_count:,}")
    print(f"\n✅ Distill + Panorama complete: {len(insights)} insights, snapshot stored")

if __name__ == "__main__":
    main()
