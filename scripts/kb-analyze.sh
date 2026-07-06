#!/usr/bin/env bash
# NeoTrix KB Content Distiller + Panorama + Meta-Cognition
# Uses Python for all JSON-heavy analysis.
#
# Usage: bash scripts/kb-analyze.sh [--kb-path <path>]

set -eo pipefail

KB_PATH="${KB_PATH:-$HOME/.neotrix/knowledge.db}"
NOW=$(date +%s)
CYCLE_START=$NOW

ERROR_LOG=()
TOTAL_NODES=0

log_ok()     { echo "  ✅ $1"; }
log_fail()   { echo "  ❌ $1"; }
log_info()   { echo "       $1"; }
log_error()  { local m="$1"; echo "  ❌ ERROR: $m"; ERROR_LOG+=("$m"); }

sql() { sqlite3 "$KB_PATH" "$@"; }

generate_uuid() {
    python3 -c "import uuid; print('nt-' + uuid.uuid4().hex[:20])" 2>/dev/null || echo "nt-$$-$RANDOM-$NOW"
}

# ═══════════════════════════════════════════════
# DISTILLER — Content Analysis & Pattern Detection
# ═══════════════════════════════════════════════

distiller_analyze_repos() {
    echo ""
    echo "═══ Distiller: Repository Analysis ═══"

    sql "SELECT COALESCE(metadata,'{}') FROM knowledge_nodes WHERE node_type='Repository';" 2>/dev/null | \
    python3 -c "
import sys, json
from collections import Counter

langs = Counter()
topics = Counter()
total_stars = 0
total_repos = 0

for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try:
        d = json.loads(line)
    except:
        continue
    total_repos += 1
    stars = d.get('stars', 0)
    if isinstance(stars, (int, float)):
        total_stars += int(stars)
    lang = d.get('language') or 'unknown'
    langs[lang] += 1
    for t in d.get('topics', []):
        if t:
            topics[t] += 1

avg_stars = total_stars // total_repos if total_repos > 0 else 0
print(f'Repos: {total_repos} | Total stars: {total_stars} | Avg: {avg_stars}')
print()

print('Language distribution:')
for lang, cnt in langs.most_common():
    print(f'  {lang}: {cnt}')
print()

print('Topic distribution (top 20):')
for topic, cnt in topics.most_common(20):
    print(f'  {topic}: {cnt}')
print()

if total_repos > 0:
    top_lang = langs.most_common(1)[0]
    print(f'Dominant language: {top_lang[0]} ({top_lang[1]}/{total_repos} repos)')
    print(f'LANG_JSON: {json.dumps(dict(langs))}')
" 2>&1 || log_error "Repository analysis failed"
}

distiller_analyze_concepts() {
    echo ""
    echo "═══ Distiller: Concept Analysis ═══"

    sql "SELECT COALESCE(domain,'unknown'), LENGTH(COALESCE(content,'')) FROM knowledge_nodes WHERE node_type='Concept';" 2>/dev/null | \
    python3 -c "
import sys, json
from collections import Counter

domains = Counter()
total_content = 0
total = 0

for line in sys.stdin:
    line = line.strip()
    if not line: continue
    parts = line.split('|')
    if len(parts) >= 2:
        domain = parts[0]
        clen = int(parts[1]) if parts[1].isdigit() else 0
    else:
        domain = line
        clen = 0
    total += 1
    total_content += clen
    domains[domain] += 1

avg = total_content // total if total > 0 else 0
print(f'Concepts: {total} | Content: {total_content} chars | Avg: {avg} chars')
print()

print('Domain distribution:')
for d, cnt in domains.most_common():
    print(f'  {d}: {cnt}')
print(f'DOMAIN_JSON: {json.dumps(dict(domains))}')
" 2>&1 || log_error "Concept analysis failed"
}

distiller_cross_references() {
    echo ""
    echo "═══ Distiller: Cross-Reference Analysis ═══"

    local result
    result=$(sql "SELECT COALESCE(metadata,'{}') FROM knowledge_nodes WHERE node_type='Repository';" 2>/dev/null || echo "")

    echo "$result" | python3 -c "
import sys, json
from collections import Counter

topic_repos = Counter()

for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try:
        d = json.loads(line)
    except:
        continue
    for t in d.get('topics', []):
        if t:
            topic_repos[t] += 1

print('Topics with cross-references (3+ repos):')
cross = 0
for topic, cnt in topic_repos.most_common():
    if cnt >= 3:
        print(f'  {topic}: {cnt} repos')
        cross += 1

print(f'Total cross-reference topics: {cross}')

# Check which topics have KB concept nodes
repo_topics = set(t for t, c in topic_repos.items() if c >= 3)
print(f'TOPIC_NAMES: {json.dumps(list(repo_topics))}')
" 2>&1 || log_error "Cross-reference analysis failed"
}

# ═══════════════════════════════════════════════
# PANORAMA — Knowledge Landscape Report
# ═══════════════════════════════════════════════

panorama_build() {
    echo ""
    echo "═══ Panorama: Knowledge Landscape ═══"

    sql "
    SELECT 'KB Size: ' || COUNT(*) || ' nodes, ' || (SELECT COUNT(*) FROM knowledge_edges) || ' edges'
    FROM knowledge_nodes;
    " 2>/dev/null

    echo ""
    echo "Domain Coverage:"
    sql "SELECT printf('  %-30s %d', COALESCE(domain,'unknown'), COUNT(*))
         FROM knowledge_nodes GROUP BY domain ORDER BY COUNT(*) DESC;" 2>/dev/null

    echo ""
    sql "
    SELECT 'Content Freshness:';
    SELECT '  Created in last 24h:  ' || COUNT(*) || ' nodes' FROM knowledge_nodes WHERE created_at > $((NOW - 86400))
    UNION ALL
    SELECT '  Created in last 7d:   ' || COUNT(*) || ' nodes' FROM knowledge_nodes WHERE created_at > $((NOW - 604800))
    UNION ALL
    SELECT '  Created in last 30d:  ' || COUNT(*) || ' nodes' FROM knowledge_nodes WHERE created_at > $((NOW - 2592000));
    " 2>/dev/null

    echo ""
    echo "Edge Analysis:"
    sql "SELECT printf('  %-25s %d', relation_type, COUNT(*))
         FROM knowledge_edges GROUP BY relation_type ORDER BY COUNT(*) DESC;" 2>/dev/null

    echo ""
    echo "Knowledge Gap Analysis:"
    local ncount
    ncount=$(sql "SELECT COUNT(*) FROM knowledge_nodes;" 2>/dev/null || echo "0")
    local repo_count
    repo_count=$(sql "SELECT COUNT(*) FROM knowledge_nodes WHERE node_type='Repository';" 2>/dev/null || echo "0")
    local topic_count
    topic_count=$(sql "SELECT COUNT(*) FROM knowledge_nodes WHERE node_type='Concept' AND domain='github.com/topic';" 2>/dev/null || echo "0")
    local orphaned
    orphaned=$(sql "SELECT COUNT(*) FROM knowledge_nodes n WHERE n.node_type='Repository'
        AND NOT EXISTS (SELECT 1 FROM knowledge_edges e WHERE e.source_id=n.id OR e.target_id=n.id);" 2>/dev/null || echo "0")

    echo "  Total nodes: $ncount"
    echo "  Repository nodes: $repo_count"
    echo "  Topic concepts: $topic_count"
    [ "$topic_count" -gt 0 ] && echo "  Repos per topic: $((repo_count / topic_count)):1"
    echo "  Orphaned repos (no edges): $orphaned"
    [ "$orphaned" -gt "$((repo_count / 2))" ] && echo "  ⚠️  High orphan rate — consider adding edges"

    echo ""
    echo "Repo Quality Metrics (via Python):"
    local quality
    quality=$(sql "SELECT COALESCE(metadata,'{}') FROM knowledge_nodes WHERE node_type='Repository';" 2>/dev/null | \
    python3 -c "
import sys, json
high=mid=low=no=0
for line in sys.stdin:
    line=line.strip()
    if not line: continue
    try:
        d=json.loads(line)
        s=d.get('stars',0)
        if isinstance(s,(int,float)):
            if s>10000: high+=1
            elif s>=1000: mid+=1
            elif s>0: low+=1
            else: no+=1
        else: no+=1
    except: no+=1
print(f'High: {high} | Mid: {mid} | Low: {low} | Unknown: {no}')
" 2>&1 || echo "?")
    echo "  $quality"

    echo ""
    echo "Top 10 Repositories by Stars:"
    sql "SELECT COALESCE(metadata,'{}') FROM knowledge_nodes WHERE node_type='Repository';" 2>/dev/null | \
    python3 -c "
import sys, json
repos = []
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try:
        d = json.loads(line)
    except:
        continue
    # Need title - get it from the SQL query output
    print(line)
" 2>/dev/null || true
    # Simpler approach: use Python with joined data
    sql "SELECT title, COALESCE(metadata,'{}') FROM knowledge_nodes WHERE node_type='Repository';" 2>/dev/null | \
    python3 -c "
import sys, json
repos = []
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    parts = line.split('|', 1)
    if len(parts) < 2: continue
    title, meta = parts
    try:
        d = json.loads(meta)
        stars = d.get('stars', 0)
        if isinstance(stars, (int,float)) and stars > 0:
            lang = d.get('language', '')
            repos.append((title, int(stars), lang))
    except:
        pass

repos.sort(key=lambda x: -x[1])
for title, stars, lang in repos[:10]:
    print(f'  {title:<40} {stars}★ {lang}')
" 2>/dev/null

    # Save panorama snapshot
    local report_json="{\"nodes\":$ncount,\"repos\":$repo_count,\"topics\":$topic_count,\"edges\":$(sql "SELECT COUNT(*) FROM knowledge_edges;"),\"high_stars\":$high_stars,\"mid_stars\":$mid_stars,\"orphaned\":$orphaned,\"ts\":$NOW}"
    local report_esc; report_esc=$(echo "$report_json" | sed "s/'/''/g")
    sql "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at)
         VALUES ('panorama', 'latest', '$report_esc', $NOW);"
    sql "INSERT INTO kv_store (namespace, key, value, updated_at)
         VALUES ('panorama', 'snapshot_$NOW', '$report_esc', $NOW);"
    echo ""
    echo "  Panorama snapshot saved to kv_store"
}

# ═══════════════════════════════════════════════
# META-COGNITIVE ERROR ANALYSIS
# ═══════════════════════════════════════════════

meta_error_analysis() {
    echo ""
    echo "═══ Meta-Cognitive Error Analysis ═══"

    local failed_count
    failed_count=$(sql "SELECT COUNT(*) FROM crawl_queue WHERE status='failed';" 2>/dev/null || echo "0")
    local ingest_errors
    ingest_errors=$(sql "SELECT COUNT(*) FROM ingest_log WHERE status='error' OR status='empty';" 2>/dev/null || echo "0")
    local pending
    pending=$(sql "SELECT COUNT(*) FROM crawl_queue WHERE status='pending';" 2>/dev/null || echo "0")

    echo "  Failed crawl entries: $failed_count"
    echo "  Ingestion errors: $ingest_errors"
    echo "  Pending crawl entries: $pending"

    # Record error report
    local error_report="{\"failed_crawl\":$failed_count,\"ingest_errors\":$ingest_errors,\"pending\":$pending,\"ts\":$NOW}"
    local error_esc; error_esc=$(echo "$error_report" | sed "s/'/''/g")
    sql "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at)
         VALUES ('meta_cognition', 'error_analysis', '$error_esc', $NOW);"

    # Record RecurringError if high failure rate
    if [ "$failed_count" -gt 100 ] || [ "$ingest_errors" -gt 10 ]; then
        local uuid; uuid=$(generate_uuid)
        local desc="KB absorption: $failed_count failed crawl, $ingest_errors ingestion errors"
        sql "INSERT OR IGNORE INTO evolution_records (id, pattern_type, description, applied_to, effectiveness_gain, verified, timestamp)
             VALUES ('$uuid', 'RecurringError', '$desc', 'knowledge_base', -0.3, 1, $NOW);"
        echo "  ⚠️ Recorded RecurringError pattern (high failure rate)"
    fi

    echo ""
    echo "  Recommendations:"
    [ "$failed_count" -gt 0 ] && echo "    - Review $failed_count failed crawl entries"
    [ "$pending" -gt 0 ] && echo "    - $pending entries still pending in crawl queue"
    echo "    - Run embedding refresh to enable semantic search"
}

# ═══════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║     NeoTrix KB Content Analysis              ║"
echo "║     Distiller + Panorama + Meta-Cognition    ║"
echo "╚══════════════════════════════════════════════╝"
echo "  KB: $KB_PATH"
echo ""

TOTAL_NODES=$(sql "SELECT COUNT(*) FROM knowledge_nodes;" 2>/dev/null || echo "0")
CYCLE_UUID=$(generate_uuid)
sql "INSERT INTO kv_store (namespace, key, value, updated_at)
     VALUES ('analysis_cycle', '$CYCLE_UUID', '{\"phase\":\"start\",\"ts\":$NOW}', $NOW);"

distiller_analyze_repos
distiller_analyze_concepts
distiller_cross_references
panorama_build
meta_error_analysis

DURATION=$(( $(date +%s) - CYCLE_START ))
NOW_NODES=$(sql "SELECT COUNT(*) FROM knowledge_nodes;" 2>/dev/null || echo "0")

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║  Analysis Complete                           ║"
echo "╚══════════════════════════════════════════════╝"
echo "  Duration: ${DURATION}s"
echo "  KB growth: $((NOW_NODES - TOTAL_NODES)) nodes"
echo "  Final KB size: $NOW_NODES nodes"
echo ""

sql "INSERT INTO kv_store (namespace, key, value, updated_at)
     VALUES ('analysis_cycle', '$CYCLE_UUID', '{\"phase\":\"complete\",\"duration\":$DURATION,\"ts\":$NOW}', $NOW);"

echo "  Results in kv_store: distiller, panorama, meta_cognition, analysis_cycle"
echo ""
