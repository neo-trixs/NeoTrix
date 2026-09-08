#!/usr/bin/env bash
# NeoTrix Crawl Queue Absorber
# Processes crawl_queue entries: Wikipedia, ArXiv, GitHub repos, and generic URLs.
# Records meta-cognitive error information to KB kv_store and evolution_records.
#
# Usage: bash scripts/crawl-queue-absorb.sh [--kb-path <path>] [--limit <N>]
#
# Requires: curl, sqlite3, python3

set -eo pipefail

# ── Config ──
KB_PATH="${KB_PATH:-$HOME/.neotrix/knowledge.db}"
GITHUB_TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
CURL_OPTS=(-sS --connect-timeout 15 --max-time 30)
[ -n "$GITHUB_TOKEN" ] && CURL_OPTS+=(-H "Authorization: Bearer $GITHUB_TOKEN")
UA="NeoTrix/0.19 (CrawlQueueAbsorber)"
LIMIT="${1:-999999}"

NOW=$(date +%s)
OK=0
FAIL=0
CYCLE_START=$NOW

# ── Error tracking ──
ERROR_LOG=()

log_ok()     { echo "  ✅ $1"; }
log_fail()   { echo "  ❌ $1"; }
log_info()   { echo "       $1"; }
log_error()  { local m="$1"; echo "  ❌ ERROR: $m"; ERROR_LOG+=("$m"); }

sql() { sqlite3 "$KB_PATH" "$@"; }

generate_uuid() {
    python3 -c "import uuid; print('nt-' + uuid.uuid4().hex[:20])" 2>/dev/null || echo "nt-$$-$RANDOM-$NOW"
}

# ── Meta-cognitive error recording ──

record_error() {
    local source="$1" error_msg="$2" severity="${3:-0.5}"
    local uuid; uuid=$(generate_uuid)
    local msg_esc; msg_esc=$(echo "$error_msg" | sed "s/'/''/g")
    # Store in kv_store under meta_cognition namespace
    sql "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at)
         VALUES ('meta_cognition', 'last_error', '{\"source\":\"$source\",\"msg\":\"$msg_esc\",\"severity\":$severity,\"ts\":$NOW}', $NOW);"
    # Append to error history
    sql "INSERT INTO kv_store (namespace, key, value, updated_at)
         VALUES ('meta_cognition', 'error_$uuid', '{\"source\":\"$source\",\"msg\":\"$msg_esc\",\"severity\":$severity,\"ts\":$NOW}', $NOW);"
}

record_cycle_meta() {
    local phase="$1" status="$2" details="$3"
    local uuid; uuid=$(generate_uuid)
    local d_esc; d_esc=$(echo "$details" | sed "s/'/''/g")
    sql "INSERT INTO kv_store (namespace, key, value, updated_at)
         VALUES ('absorption_cycle', '$uuid', '{\"phase\":\"$phase\",\"status\":\"$status\",\"details\":\"$d_esc\",\"ts\":$NOW}', $NOW);"
}

record_error_pattern() {
    local pattern="$1" description="$2" domain="$3"
    local uuid; uuid=$(generate_uuid)
    local desc_esc; desc_esc=$(echo "$description" | sed "s/'/''/g")
    # Insert into evolution_records as a RecurringError pattern
    sql "INSERT OR IGNORE INTO evolution_records (id, pattern_type, description, applied_to, effectiveness_gain, verified, timestamp)
         VALUES ('$uuid', 'RecurringError', '$desc_esc', '$domain', -0.1, 0, $NOW);"
}

# ── Wikipedia Absorption ──

absorb_wikipedia() {
    local url="$1"
    # Extract article title from URL
    local title
    title=$(echo "$url" | sed 's|https://en.wikipedia.org/wiki/||; s|https://en.wikipedia.org/||; s|_| |g; s|/| |g; s|%20| |g' | xargs)

    # Check if already exists
    local existing
    existing=$(sql "SELECT id FROM nodes WHERE url = '$url' LIMIT 1" 2>/dev/null || echo "")
    if [ -n "$existing" ]; then
        log_info "Already exists: $title ($url)"
        return 0
    fi

    log_info "Fetching Wikipedia: $title"

    # Fetch via REST API
    local api_url="https://en.wikipedia.org/api/rest_v1/page/summary/$(echo "$title" | sed 's/ /_/g' | python3 -c "import sys,urllib.parse; print(urllib.parse.quote(sys.stdin.read().strip()))" 2>/dev/null)"
    local data
    data=$(curl "${CURL_OPTS[@]}" -H "User-Agent: $UA" "$api_url" 2>/dev/null || echo '{}')

    local extract
    extract=$(echo "$data" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    extract = d.get('extract', '')
    if not extract:
        extract = d.get('description', '')
    print(extract[:2000].replace(chr(10), ' ').replace(\"'\", \"''\") if extract else '')
except:
    print('')
" 2>/dev/null || echo "")

    local page_title
    page_title=$(echo "$data" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    t = d.get('title', '') or d.get('displaytitle', '')
    print(t.replace(\"'\", \"''\") if t else '')
except:
    print('')
" 2>/dev/null || echo "")

    [ -z "$page_title" ] && page_title="$title"

    # Determine domain based on URL
    local domain="en.wikipedia.org"

    if [ -z "$extract" ]; then
        log_info "No extract for $page_title, storing as stub"
        local id; id=$(generate_uuid)
        sql "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, domain, language, confidence, importance, created_at, updated_at)
             VALUES ('$id', 'concept', '$page_title', 'Wikipedia: $page_title', '$url', '$domain', 'en', 0.7, 0.5, $NOW, $NOW);"
        log_info "Stored stub: $page_title"
    else
        local id; id=$(generate_uuid)
        local summary_esc
        summary_esc=$(echo "$extract" | sed "s/'/''/g" | head -c 1000)
        sql "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at)
             VALUES ('$id', 'concept', '$page_title', '$summary_esc', '$extract', '$url', '$domain', 'en', 0.8, 0.6, $NOW, $NOW);"
        log_info "Stored Wikipedia: $page_title (${#extract} chars)"
    fi

    # Mark crawl queue as completed
    sql "UPDATE crawl_queue SET status='completed', last_attempt=$NOW, retry_count=COALESCE(retry_count,0)+1 WHERE url='$url';"
    OK=$((OK+1))
}

# ── ArXiv Absorption ──

absorb_arxiv() {
    local url="$1"
    local paper_id
    paper_id=$(echo "$url" | sed 's|https://arxiv.org/abs/||; s|https://export.arxiv.org/api/query?id_list=||; s|&.*||')

    # Check if already exists
    local existing
    existing=$(sql "SELECT id FROM nodes WHERE url LIKE '%arxiv.org/abs/$paper_id%' LIMIT 1" 2>/dev/null || echo "")
    if [ -n "$existing" ]; then
        log_info "ArXiv $paper_id already exists"
        sql "UPDATE crawl_queue SET status='completed', last_attempt=$NOW WHERE url='$url';"
        return 0
    fi

    log_info "Fetching ArXiv: $paper_id"

    local data
    data=$(curl "${CURL_OPTS[@]}" -H "User-Agent: $UA" "https://export.arxiv.org/api/query?id_list=$paper_id&max_results=1" 2>/dev/null || echo "")

    if echo "$data" | grep -qi "error\|not found\|<entry>" >/dev/null 2>&1; then
        local id; id=$(generate_uuid)
        sql "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, domain, language, confidence, importance, created_at, updated_at)
             VALUES ('$id', 'concept', 'ArXiv $paper_id', 'Paper from arxiv: $paper_id', 'https://arxiv.org/abs/$paper_id', 'arxiv.org', 'en', 0.7, 0.5, $NOW, $NOW);"
        log_info "Stored arXiv stub: $paper_id"
    else
        local title
        title=$(echo "$data" | grep -o '<title>[^<]*</title>' | head -1 | sed 's/<[^>]*>//g' | sed 's/^ *//' | sed "s/'/''/g" | head -c 200)
        [ -z "$title" ] && title="ArXiv $paper_id"
        local summary
        summary=$(echo "$data" | grep -o '<summary>[^<]*</summary>' | head -1 | sed 's/<[^>]*>//g' | sed "s/'/''/g" | head -c 1000)
        local id; id=$(generate_uuid)
        sql "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, domain, language, confidence, importance, created_at, updated_at)
             VALUES ('$id', 'concept', '$title', '$summary', 'https://arxiv.org/abs/$paper_id', 'arxiv.org', 'en', 0.8, 0.6, $NOW, $NOW);"
        log_info "Stored ArXiv: $title"
    fi

    sql "UPDATE crawl_queue SET status='completed', last_attempt=$NOW WHERE url='$url';"
    OK=$((OK+1))
}

# ── GitHub Repo Absorption ──

absorb_github_repo() {
    local url="$1"
    local full_name
    full_name=$(echo "$url" | sed 's|https://github.com/||; s|/$||')

    # Check if already exists
    local existing
    existing=$(sql "SELECT id FROM nodes WHERE url = 'https://github.com/$full_name' LIMIT 1" 2>/dev/null || echo "")
    if [ -n "$existing" ]; then
        log_info "Repo $full_name already exists"
        sql "UPDATE crawl_queue SET status='completed', last_attempt=$NOW WHERE url='https://github.com/$full_name';"
        return 0
    fi

    local owner repo data
    owner=$(echo "$full_name" | cut -d/ -f1)
    repo=$(echo "$full_name" | cut -d/ -f2)

    log_info "Fetching repo: $full_name"
    data=$(curl "${CURL_OPTS[@]}" -H "User-Agent: $UA" "https://api.github.com/repos/$owner/$repo" 2>/dev/null || echo '{}')

    # Check for API error
    local msg
    msg=$(echo "$data" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('message',''))" 2>/dev/null || echo "")
    if [ -n "$msg" ]; then
        log_fail "API error for $full_name: $msg"
        record_error "github_api" "Repo $full_name: $msg" 0.7
        sql "UPDATE crawl_queue SET status='failed', last_attempt=$NOW, error_message='$msg', retry_count=COALESCE(retry_count,0)+1 WHERE url='$url';"
        FAIL=$((FAIL+1))
        return 1
    fi

    local description stars language topics
    description=$(echo "$data" | python3 -c "import sys,json; d=json.load(sys.stdin); print((d.get('description') or '').replace(\"'\",\"''\")[:500])" 2>/dev/null || echo "")
    stars=$(echo "$data" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('stargazers_count',0))" 2>/dev/null || echo "0")
    language=$(echo "$data" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('language') or 'unknown')" 2>/dev/null || echo "unknown")
    topics=$(echo "$data" | python3 -c "
import sys,json; d=json.load(sys.stdin); t=d.get('topics',[]); print(json.dumps(t))
" 2>/dev/null || echo "[]")

    local id; id=$(generate_uuid)
    local meta="{\"stars\":$stars,\"language\":\"$language\",\"topics\":$topics}"
    local meta_esc; meta_esc=$(echo "$meta" | sed "s/'/''/g")

    sql "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, domain, language, confidence, importance, created_at, updated_at, metadata)
         VALUES ('$id', 'repository', '$full_name', '$description', 'https://github.com/$full_name', 'github.com', 'en', 1.0, 0.5, $NOW, $NOW, '$meta_esc');"

    log_info "Stored $full_name ($stars★, $language)"
    sql "UPDATE crawl_queue SET status='completed', last_attempt=$NOW WHERE url='$url';"
    OK=$((OK+1))
}

# ── Crawl Queue Processing Loop ──

process_queue() {
    local processed=0
    local max_per_type="$1"
    [ -z "$max_per_type" ] && max_per_type=50

    record_cycle_meta "start" "ok" "Crawl queue absorption cycle started at $(date)"

    # Phase 1: Wikipedia entries
    echo ""
    echo "═══ Phase 1: Wikipedia Absorption ═══"
    local wiki_urls
    wiki_urls=$(sql "SELECT url FROM crawl_queue WHERE status='pending' AND domain LIKE '%wikipedia.org%' LIMIT $max_per_type;" 2>/dev/null || echo "")
    local wiki_count=0
    while IFS= read -r url; do
        [ -z "$url" ] && continue
        echo "  [$((OK+FAIL+1))] Wiki: $(echo $url | sed 's|.*/||' | sed 's/_/ /g')"
        absorb_wikipedia "$url" || true
        wiki_count=$((wiki_count+1))
        processed=$((processed+1))
        # Throttle to avoid being rate-limited
        sleep 1
    done <<< "$wiki_urls"
    echo "  ✅ Wikipedia: $wiki_count processed"

    # Phase 2: ArXiv entries
    echo ""
    echo "═══ Phase 2: ArXiv Absorption ═══"
    local arxiv_urls
    arxiv_urls=$(sql "SELECT url FROM crawl_queue WHERE status='pending' AND (domain='arxiv.org' OR domain='export.arxiv.org') LIMIT $max_per_type;" 2>/dev/null || echo "")
    local arxiv_count=0
    while IFS= read -r url; do
        [ -z "$url" ] && continue
        echo "  [$((OK+FAIL+1))] ArXiv: $(echo $url | sed 's|.*/||')"
        absorb_arxiv "$url" || true
        arxiv_count=$((arxiv_count+1))
        processed=$((processed+1))
        sleep 1
    done <<< "$arxiv_urls"
    echo "  ✅ ArXiv: $arxiv_count processed"

    # Phase 3: GitHub repos
    echo ""
    echo "═══ Phase 3: GitHub Repo Absorption ═══"
    local repo_urls
    repo_urls=$(sql "SELECT url FROM crawl_queue WHERE status='pending' AND domain='github.com' LIMIT $max_per_type;" 2>/dev/null || echo "")
    local repo_count=0
    while IFS= read -r url; do
        [ -z "$url" ] && continue
        echo "  [$((OK+FAIL+1))] Repo: $(echo $url | sed 's|https://github.com/||')"
        absorb_github_repo "$url" || true
        repo_count=$((repo_count+1))
        processed=$((processed+1))
        sleep 1  # Rate limiting: 1 req/sec max
    done <<< "$repo_urls"
    echo "  ✅ GitHub repos: $repo_count processed"

    # Phase 4: Other domain entries (api.github.com, meta.wikimedia, etc.)
    echo ""
    echo "═══ Phase 4: Other Domains ═══"
    local other_urls
    other_urls=$(sql "SELECT url FROM crawl_queue WHERE status='pending' AND domain NOT LIKE '%wikipedia%' AND domain != 'arxiv.org' AND domain != 'export.arxiv.org' AND domain != 'github.com' LIMIT $max_per_type;" 2>/dev/null || echo "")
    local other_count=0
    while IFS= read -r url; do
        [ -z "$url" ] && continue
        local domain
        domain=$(echo "$url" | sed 's|https://||; s|http://||; s|/.*||')
        echo "  [$((OK+FAIL+1))] $domain: $(echo $url | head -c 80)"
        # Try to fetch and store as generic Concept
        local existing
        existing=$(sql "SELECT id FROM nodes WHERE url = '$url' LIMIT 1" 2>/dev/null || echo "")
        if [ -n "$existing" ]; then
            log_info "Already exists"
            sql "UPDATE crawl_queue SET status='completed', last_attempt=$NOW WHERE url='$url';"
            OK=$((OK+1))
        else
            local data
            data=$(curl "${CURL_OPTS[@]}" -H "User-Agent: $UA" "$url" 2>/dev/null || echo "")
            local title="$url"
            local extract
            extract=$(echo "$data" | python3 -c "
import sys
try:
    txt = sys.stdin.read()
    # Extract <title> tag
    import re
    m = re.search(r'<title[^>]*>(.*?)</title>', txt, re.IGNORECASE)
    t = m.group(1) if m else ''
    # Get first 500 chars of body text
    body = re.sub(r'<[^>]+>', ' ', txt)[:500]
    body = ' '.join(body.split())[:500]
    print((t + ': ' + body).replace(\"'\", \"''\")[:1000])
except:
    print('')
" 2>/dev/null || echo "")
            [ -z "$extract" ] && extract="URL: $url"
            local id; id=$(generate_uuid)
            local domain_clean=$(echo "$domain" | sed "s/'/''/g")
            local extract_esc=$(echo "$extract" | sed "s/'/''/g" | head -c 500)
            sql "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, domain, confidence, importance, created_at, updated_at)
                 VALUES ('$id', 'concept', '$url', '$extract_esc', '$url', '$domain_clean', 0.5, 0.3, $NOW, $NOW);"
            log_info "Stored: $url"
            sql "UPDATE crawl_queue SET status='completed', last_attempt=$NOW WHERE url='$url';"
            OK=$((OK+1))
        fi
        other_count=$((other_count+1))
        processed=$((processed+1))
        sleep 0.5
    done <<< "$other_urls"
    echo "  ✅ Other domains: $other_count processed"
}

# ── Meta-cognitive Summary ──

generate_meta_report() {
    local duration=$(( $(date +%s) - CYCLE_START ))
    local node_count
    node_count=$(sql "SELECT COUNT(*) FROM nodes;" 2>/dev/null || echo "?")
    local edge_count
    edge_count=$(sql "SELECT COUNT(*) FROM edges;" 2>/dev/null || echo "?")
    local pending
    pending=$(sql "SELECT COUNT(*) FROM crawl_queue WHERE status='pending';" 2>/dev/null || echo "?")
    local failed
    failed=$(sql "SELECT COUNT(*) FROM crawl_queue WHERE status='failed';" 2>/dev/null || echo "?")

    local error_count=${#ERROR_LOG[@]}

    echo ""
    echo "╔══════════════════════════════════════════════╗"
    echo "║  Cycle Meta-Cognitive Report                 ║"
    echo "╚══════════════════════════════════════════════╝"
    echo "  Duration: ${duration}s"
    echo "  Nodes: $node_count | Edges: $edge_count"
    echo "  Crawl queue: $OK ok, $FAIL fail, $pending pending, $failed total failed"
    echo "  Errors this cycle: $error_count"
    if [ "$error_count" -gt 0 ]; then
        echo "  Error details:"
        local idx=0
        for err in "${ERROR_LOG[@]}"; do
            echo "    [$idx] $err"
            idx=$((idx+1))
        done
    fi

    # Estimate rate limit remaining
    local gh_remaining
    gh_remaining=$(curl -s -H "User-Agent: $UA" "https://api.github.com/rate_limit" 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('resources',{}).get('core',{}).get('remaining','?'))" 2>/dev/null || echo "?")

    echo ""
    echo "  GitHub API remaining: $gh_remaining/60"
    echo ""

    # Write comprehensive meta-cognitive summary
    local report_json
    report_json="{\"duration_secs\":$duration,\"ok\":$OK,\"fail\":$FAIL,\"errors\":$error_count,\"nodes\":\"$node_count\",\"edges\":\"$edge_count\",\"pending_queue\":$pending,\"failed_queue\":$failed,\"gh_api_remaining\":\"$gh_remaining\",\"cycle_start\":$CYCLE_START}"

    # Record to kv_store
    local report_esc
    report_esc=$(echo "$report_json" | sed "s/'/''/g")
    sql "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at)
         VALUES ('meta_cognition', 'cycle_report_$CYCLE_START', '$report_esc', $NOW);"

    # If we had errors, record them as evolution records
    if [ "$error_count" -gt 0 ] || [ "$FAIL" -gt 0 ]; then
        local error_detail="Crawl queue absorption cycle: $FAIL failed items, $error_count errors logged"
        record_error_pattern "KB_ABSORPTION_ERROR" "$error_detail" "knowledge_base"
        log_info "Recorded RecurringError pattern for KB absorption errors"
    fi

    # Record KB growth rate
    local yesterday_count
    yesterday_count=$(sql "SELECT COUNT(*) FROM nodes WHERE created_at < $((NOW - 3600))" 2>/dev/null || echo "0")
    local growth=$((node_count - yesterday_count))
    record_cycle_meta "kb_growth" "ok" "KB grew by $growth nodes in last hour"

    # Generate recommendations
    echo "  Recommendations:"
    if [ "$pending" -gt 0 ]; then
        echo "    - Continue absorbing $pending pending crawl queue entries"
    fi
    if [ "$FAIL" -gt 0 ] && [ "$FAIL" -gt "$OK" ]; then
        echo "    - ⚠️ Failure rate > 50% — consider checking network/API status"
    fi
    if [ "$error_count" -gt 5 ]; then
        echo "    - ⚠️ High error count ($error_count) — meta-cognitive review recommended"
    fi
    echo "    - Run content distiller to extract patterns from new KB nodes"
    echo "    - Run panorama generator for updated knowledge landscape"
}

# ── Main ──

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║     NeoTrix Crawl Queue Absorber             ║"
echo "║     Self-Evolving Knowledge Pipeline         ║"
echo "╚══════════════════════════════════════════════╝"
echo "  KB: $KB_PATH"
echo "  Token: $([ -n "$GITHUB_TOKEN" ] && echo "✅ set" || echo "❌ not set (60 req/hr limit)")"
echo "  Limit: $LIMIT items per domain"

process_queue "$LIMIT"

echo ""
echo "═══ Final Summary ═══"
echo "  Processed: $OK OK, $FAIL Failed"
echo ""

generate_meta_report

record_cycle_meta "complete" "$([ $FAIL -eq 0 ] && echo 'ok' || echo 'partial')" "Cycle completed: $OK ok, $FAIL fail"

echo "  Done. Run the same script again to process more crawl queue entries."
echo ""
