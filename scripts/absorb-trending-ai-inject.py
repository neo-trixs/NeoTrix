#!/usr/bin/env python3
"""Phase 1: Inject 12 trending AI repos as KB stubs + cross-edges + category concepts.
Usage: python3 scripts/absorb-trending-ai-inject.py"""
import sqlite3, json, time, os, hashlib

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
    ("topoteretes/cognee", 26901, "Python",
     ["ai-memory","knowledge-graph","agent","llm","rag","memory-platform"],
     "Agent-Memory",
     "Open-source AI memory platform for agents. Persistent long-term memory across sessions. Self-hosted knowledge graph engine with semantic search."),
    ("DeusData/codebase-memory-mcp", 25871, "C",
     ["mcp","code-intelligence","knowledge-graph","code-search"],
     "MCP-Servers",
     "High-performance code intelligence MCP server. Indexes codebases into knowledge graph. 158 languages, sub-ms queries, 99% fewer tokens."),
    ("diegosouzapw/OmniRoute", 11113, "TypeScript",
     ["ai-gateway","llm","provider","free-llm","token-compression"],
     "AI-Gateway",
     "Free AI gateway: 231+ providers (50+ free). RTK+Caveman compression saves 15-95% tokens. Smart auto-fallback, MCP/A2A, multimodal APIs."),
    ("ogulcancelik/herdr", 11123, "Rust",
     ["agent","multiplexer","terminal","rust","cli"],
     "Rust-Agent-Tools",
     "Agent multiplexer that lives in your terminal. Rust-native agent orchestration, parallel execution, terminal UI."),
    ("msitarzewski/agency-agents", 126845, "Shell",
     ["ai-agency","agents","automation","skills","multi-agent"],
     "Agent-Frameworks",
     "Complete AI agency. 500+ specialized agent skills. Frontend wizards to Reddit community ninjas."),
    ("stablyai/orca", 11888, "TypeScript",
     ["agent","ade","parallel-agents","desktop","mobile"],
     "Agent-Frameworks",
     "Agent Development Environment for parallel agents. Run any coding agent with your subscription. Desktop + mobile."),
    ("usestrix/strix", 35534, "Python",
     ["security","pentesting","ai-security","vulnerability"],
     "Security-Pentesting",
     "Open-source AI penetration testing tool. Automated exploit detection and remediation."),
    ("alibaba/page-agent", 22751, "TypeScript",
     ["gui-agent","browser-automation","web-ui","nlp","in-page"],
     "Browser-Automation",
     "JavaScript in-page GUI agent. Control web interfaces with natural language. Runs in-browser, no server needed."),
    ("browser-use/video-use", 14564, "Python",
     ["video-editing","agent","automation","multimedia"],
     "Multimedia-Agents",
     "Edit videos with coding agents. Programmatic video editing pipeline powered by AI agents."),
    ("callesthio/OpenMontage", 32825, "Python",
     ["video-production","agentic","pipelines","tools","skills"],
     "Multimedia-Agents",
     "First open-source agentic video production system. 12 pipelines, 52 tools, 500+ agent skills."),
    ("openai/codex-plugin-cc", 23690, "JavaScript",
     ["codex","claude-code","mcp","integration","plugin"],
     "MCP-Servers",
     "Use Codex from Claude Code. MCP-based cross-platform agent interop."),
    ("JCodesMore/ai-website-cloner-template", 25423, "TypeScript",
     ["website-cloner","ai","template","automation"],
     "AI-Web-Tools",
     "Clone any website with one command using AI coding agents."),
]

def insert_node(c, ntype, title, summary, url, domain, meta):
    kid = f"nt-{ndig(url)}"
    if c.execute("SELECT id FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone(): return
    if c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (kid,)).fetchone(): return
    retry(c, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,metadata) VALUES (?,?,?,?,?,?,?,'en',1.0,0.7,?,?,?)",
          (kid, ntype, title, summary, "", url, domain, NOW, NOW, json.dumps(meta)))

def insert_edge(c, s, t, r, w=0.8, d=""):
    eid = f"re-{ndig(f'{s}{t}')}"
    if c.execute("SELECT id FROM edges WHERE source_id=? AND target_id=? LIMIT 1", (s, t)).fetchone(): return
    retry(c, "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) VALUES (?,?,?,?,?,?,?)",
          (eid, s, t, r, w, d, NOW))

def main():
    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()
    nids = {}

    print("═══ Phase 1: Inject 12 trending repo stubs ═══\n")
    cats = {}
    for fn, stars, lang, topics, cat, desc in REPOS:
        url = f"https://github.com/{fn}"
        meta = {"stars":stars,"topics":topics,"trending_category":cat,
                "trending_source":"github-weekly-2026-07-04","absorbed_at":NOW}
        nid = insert_node(c, "Repository", fn.split("/")[1], desc, url, "github.com", meta)
        if nid is None:
            nid = c.execute("SELECT id FROM nodes WHERE url=? LIMIT 1", (url,)).fetchone()[0]
        nids[fn] = nid
        print(f"  ✅ {fn} → {nid} ({stars}★, {cat})")
        if cat not in cats:
            cid = f"cat-{ndig(cat)}"
            if not c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (cid,)).fetchone():
                retry(c, "INSERT OR IGNORE INTO nodes (id,node_type,title,url,domain,created_at,updated_at) VALUES (?,?,'none','','neotrix.ai',?,?)",
                      (cid, "Concept", NOW, NOW))
                c.execute("UPDATE nodes SET title=?,summary=? WHERE id=?",
                          (f"Trending: {cat}", f"GitHub Trending AI ecosystem — {cat}", cid))
            cats[cat] = cid

    # Category → repo edges
    for fn, _, _, _, cat, _ in REPOS:
        insert_edge(c, cats[cat], nids[fn], "contains", 0.9)

    # Cross-reference edges
    mr = nids
    if "topoteretes/cognee" in mr and "DeusData/codebase-memory-mcp" in mr:
        insert_edge(c, mr["topoteretes/cognee"], mr["DeusData/codebase-memory-mcp"], "related_to", 0.7)
    if "openai/codex-plugin-cc" in mr:
        for o in ["DeusData/codebase-memory-mcp"]:
            if o in mr: insert_edge(c, mr[o], mr["openai/codex-plugin-cc"], "related_to", 0.6)
    af = ["msitarzewski/agency-agents","stablyai/orca"]
    for i in range(len(af)):
        for j in range(i+1,len(af)):
            if af[i] in mr and af[j] in mr:
                insert_edge(c, mr[af[i]], mr[af[j]], "related_to", 0.6)
    ba = ["alibaba/page-agent","browser-use/video-use"]
    for i in range(len(ba)):
        for j in range(i+1,len(ba)):
            if ba[i] in mr and ba[j] in mr:
                insert_edge(c, mr[ba[i]], mr[ba[j]], "related_to", 0.5)

    conn.commit()
    conn.close()
    print(f"\n✅ Phase 1 complete: {len(REPOS)} repos + {len(cats)} categories injected")

if __name__ == "__main__":
    main()
