#!/usr/bin/env bash
# NeoTrix Unified KB Analysis — reads from BOTH nodes + knowledge_nodes tables
set -eo pipefail

KB_PATH="${KB_PATH:-$HOME/.neotrix/knowledge.db}"
sql() { sqlite3 "$KB_PATH" "$@"; }

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     NeoTrix Unified KB Comprehensive Analysis               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── 1. OVERALL STATS ──
echo "═══ 1. Overall KB Stats ═══"
echo "  Nodes (Rust pipeline):   $(sql "SELECT COUNT(*) FROM nodes;")"
echo "  Nodes (Script pipeline): $(sql "SELECT COUNT(*) FROM knowledge_nodes;")"
echo "  Edges (Rust pipeline):   $(sql "SELECT COUNT(*) FROM edges;")"
echo "  Edges (Script pipeline): $(sql "SELECT COUNT(*) FROM knowledge_edges;")"
echo "  Embeddings:              $(sql "SELECT COUNT(*) FROM embeddings;")"
echo "  Crawl Queue (pending):   $(sql "SELECT COUNT(*) FROM crawl_queue WHERE status='pending';")"
echo ""

# ── 2. NODE TYPE DISTRIBUTION (both tables) ──
echo "═══ 2. Node Type Distribution ═══"
echo "--- Rust nodes table (main) ---"
sql "SELECT node_type, COUNT(*) as cnt FROM nodes GROUP BY node_type ORDER BY cnt DESC LIMIT 20;"
echo ""
echo "--- Script knowledge_nodes table ---"
sql "SELECT node_type, COUNT(*) as cnt FROM knowledge_nodes GROUP BY node_type ORDER BY cnt DESC LIMIT 20;"
echo ""

# ── 3. DOMAIN COVERAGE ──
echo "═══ 3. Top Domain Coverage (Rust nodes) ═══"
sql "SELECT domain, COUNT(*) as cnt FROM nodes WHERE domain != '' AND domain IS NOT NULL GROUP BY domain ORDER BY cnt DESC LIMIT 30;"
echo ""

# ── 4. GITHUB REPOS (most starred from script pipeline) ──
echo "═══ 4. Top Starred GitHub Repos (script pipeline) ═══"
sql "SELECT title, json_extract(metadata, '$.stars') as stars, json_extract(metadata, '$.language') as lang 
     FROM knowledge_nodes WHERE node_type='Repository' AND metadata IS NOT NULL 
     ORDER BY CAST(json_extract(metadata, '$.stars') AS INTEGER) DESC LIMIT 30;"
echo ""

# ── 5. Rust Pipeline GitHub Data (from nodes table) ──
echo "═══ 5. Rust Pipeline: GitHub domain top repos ═══"
sql "SELECT title, node_type, domain, summary FROM nodes WHERE domain LIKE '%github%' AND length(summary) > 50 ORDER BY importance DESC LIMIT 20;"
echo ""

# ── 6. CONCEPT COVERAGE (knowledge graph) ──
echo "═══ 6. Knowledge Graph Edge Types ═══"
echo "--- Rust edges ---"
sql "SELECT relation_type, COUNT(*) as cnt FROM edges GROUP BY relation_type ORDER BY cnt DESC LIMIT 15;"
echo ""
echo "--- Script knowledge_edges ---"
sql "SELECT relation_type, COUNT(*) as cnt FROM knowledge_edges GROUP BY relation_type ORDER BY cnt DESC LIMIT 15;"
echo ""

# ── 7. EVOLUTION RECORDS ──
echo "═══ 7. Evolution Records ═══"
sql "SELECT pattern_type, COUNT(*) as cnt FROM evolution_records GROUP BY pattern_type ORDER BY cnt DESC;" 2>/dev/null
echo ""

# ── 8. KV STORE STATE ──
echo "═══ 8. KV Store Namespaces ═══"
sql "SELECT namespace, COUNT(*) as cnt FROM kv_store GROUP BY namespace ORDER BY cnt DESC LIMIT 20;" 2>/dev/null
echo ""

# ── 9. KNOWLEDGE QUALITY METRICS ──
echo "═══ 9. Knowledge Quality ═══"
echo "  Nodes WITH summary:   $(sql "SELECT COUNT(*) FROM nodes WHERE summary IS NOT NULL AND length(summary) > 20;")"
echo "  Nodes WITH content:   $(sql "SELECT COUNT(*) FROM nodes WHERE content IS NOT NULL AND length(content) > 100;")"
echo "  Nodes WITH url:       $(sql "SELECT COUNT(*) FROM nodes WHERE url IS NOT NULL;")"
echo "  Unique domains:       $(sql "SELECT COUNT(DISTINCT domain) FROM nodes WHERE domain != '' AND domain IS NOT NULL;")"
echo "  Avg importance:       $(sql "SELECT AVG(importance) FROM nodes;")"
echo "  Avg confidence:       $(sql "SELECT AVG(confidence) FROM nodes;")"
echo ""

# ── 10. KNOWLEDGE GAP ANALYSIS (NeoTrix 7-domain relevance) ──
echo "═══ 10. Knowledge Gap Analysis (7-Domain Relevance) ═══"
NT_CORE=$(sql "SELECT COUNT(*) FROM nodes WHERE title LIKE '%E8%' OR title LIKE '%hexagram%' OR title LIKE '%hypercube%' OR title LIKE '%VSA%' OR title LIKE '%GWT%' OR title LIKE '%Mamba%' OR title LIKE '%state space%' OR title LIKE '%resonance%' OR title LIKE '%process reward%' OR title LIKE '%PRM%' OR title LIKE '%SAE%' OR title LIKE '%policy gradient%' OR title LIKE '%reasoning%' OR title LIKE '%attention%' OR title LIKE '%consciousness%' OR title LIKE '%meta-cognition%' OR title LIKE '%world model%'")
NT_MIND=$(sql "SELECT COUNT(*) FROM nodes WHERE title LIKE '%self-improve%' OR title LIKE '%self-evolv%' OR title LIKE '%SEAL%' OR title LIKE '%skill%' OR title LIKE '%pipeline%' OR title LIKE '%auto%edit%' OR title LIKE '%validation%' OR title LIKE '%reinforcement learning%' OR title LIKE '%self model%'")
NT_MEMORY=$(sql "SELECT COUNT(*) FROM nodes WHERE title LIKE '%knowledge base%' OR title LIKE '%SQLite%' OR title LIKE '%embedding%' OR title LIKE '%FTS5%' OR title LIKE '%BM25%' OR title LIKE '%vector search%' OR title LIKE '%knowledge graph%' OR title LIKE '%memory%' OR title LIKE '%retrieval%' OR title LIKE '%RAG%'")
NT_WORLD=$(sql "SELECT COUNT(*) FROM nodes WHERE title LIKE '%browser%' OR title LIKE '%crawl%' OR title LIKE '%scrape%' OR title LIKE '%web search%' OR title LIKE '%world model%' OR title LIKE '%JEPA%' OR title LIKE '%perception%' OR title LIKE '%vision%' OR title LIKE '%sense%'")
NT_ACT=$(sql "SELECT COUNT(*) FROM nodes WHERE title LIKE '%crypto%' OR title LIKE '%wallet%' OR title LIKE '%DEX%' OR title LIKE '%earn%' OR title LIKE '%social%' OR title LIKE '%twitter%' OR title LIKE '%code%generat%' OR title LIKE '%goal%' OR title LIKE '%autonomy%' OR title LIKE '%voice%'")
NT_IO=$(sql "SELECT COUNT(*) FROM nodes WHERE title LIKE '%CLI%' OR title LIKE '%TUI%' OR title LIKE '%server%' OR title LIKE '%WebSocket%' OR title LIKE '%HTTP%' OR title LIKE '%desktop%' OR title LIKE '%Tauri%' OR title LIKE '%notification%'")
NT_SHIELD=$(sql "SELECT COUNT(*) FROM nodes WHERE title LIKE '%security%' OR title LIKE '%vault%' OR title LIKE '%permission%' OR title LIKE '%sandbox%' OR title LIKE '%rail%' OR title LIKE '%prompt%inject%' OR title LIKE '%guard%' OR title LIKE '%safe%'")

echo "  NT-CORE (推理核):    $NT_CORE nodes"
echo "  NT-MIND (自我进化):  $NT_MIND nodes"
echo "  NT-MEMORY (持久记忆):$NT_MEMORY nodes"
echo "  NT-WORLD (感知交互): $NT_WORLD nodes"
echo "  NT-ACT (行动工具):   $NT_ACT nodes"
echo "  NT-IO (人机界面):    $NT_IO nodes"
echo "  NT-SHIELD (安全防护):$NT_SHIELD nodes"
echo ""

# Score: aim for 1000+ per domain
echo "  Health assessment:"
for domain in "NT-CORE:$NT_CORE" "NT-MIND:$NT_MIND" "NT-MEMORY:$NT_MEMORY" "NT-WORLD:$NT_WORLD" "NT-ACT:$NT_ACT" "NT-IO:$NT_IO" "NT-SHIELD:$NT_SHIELD"; do
    name="${domain%%:*}"
    count="${domain##*:}"
    if [ "$count" -gt 500 ]; then echo "    ✅ $name: $count (healthy)"; 
    elif [ "$count" -gt 100 ]; then echo "    🟡 $name: $count (moderate)";
    else echo "    ❌ $name: $count (gap - needs absorption)"; fi
done
echo ""

# ── 11. SCRIPT-KB PANORAMA STATE ──
echo "═══ 11. Panorama State (from kv_store) ═══"
sql "SELECT key, substr(value, 1, 80) as preview FROM kv_store WHERE key LIKE 'panorama_%' OR key LIKE 'distiller_%' LIMIT 20;" 2>/dev/null
echo ""

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Analysis Complete                                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
