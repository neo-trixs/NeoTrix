#!/usr/bin/env bash
# NeoTrix Batch Knowledge Absorption
# Absorbs GitHub topics, repos, and ArXiv papers into KB SQLite database.
# Usage: bash scripts/batch-absorb.sh [--kb-path <path>]
#
# Requires: curl, sqlite3, jq (optional for pretty output)
# GitHub API rate: 60 req/hr unauthenticated, 5000 req/hr with GITHUB_TOKEN

set -eo pipefail

# ── Config ──
KB_PATH="${KB_PATH:-$HOME/.neotrix/knowledge.db}"
GITHUB_TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
CURL_OPTS=(-sS)
[ -n "$GITHUB_TOKEN" ] && CURL_OPTS+=(-H "Authorization: Bearer $GITHUB_TOKEN")
UA="NeoTrix/0.19 (BatchAbsorber)"

NOW=$(date +%s)
OK=0
FAIL=0

# ── Helpers ──

log_ok()  { echo "  ✅ $1"; }
log_fail(){ echo "  ❌ $1"; }
log_info(){ echo "       $1"; }

sql() {
    sqlite3 "$KB_PATH" "$@"
}

ensure_schema() {
    sql "CREATE TABLE IF NOT EXISTS knowledge_nodes (
        id TEXT PRIMARY KEY,
        node_type TEXT NOT NULL,
        title TEXT NOT NULL,
        summary TEXT,
        content TEXT,
        url TEXT UNIQUE,
        domain TEXT,
        language TEXT DEFAULT 'en',
        confidence REAL DEFAULT 1.0,
        importance REAL DEFAULT 0.5,
        created_at INTEGER,
        updated_at INTEGER,
        access_count INTEGER DEFAULT 0,
        metadata TEXT
    );"
    sql "CREATE TABLE IF NOT EXISTS knowledge_edges (
        id TEXT PRIMARY KEY,
        source_id TEXT NOT NULL,
        target_id TEXT NOT NULL,
        relation_type TEXT NOT NULL,
        weight REAL DEFAULT 1.0,
        description TEXT,
        created_at INTEGER,
        metadata TEXT
    );"
    sql "CREATE TABLE IF NOT EXISTS ingest_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source_type TEXT,
        source_id TEXT,
        action TEXT,
        timestamp INTEGER,
        status TEXT,
        error TEXT
    );"
    sql "CREATE TABLE IF NOT EXISTS evo_records (
        id TEXT PRIMARY KEY,
        source_conversation_id TEXT,
        pattern_type TEXT,
        description TEXT,
        before_behavior TEXT,
        after_behavior TEXT,
        effectiveness_gain REAL,
        applied_to TEXT,
        verified INTEGER,
        timestamp INTEGER
    );"
    sql "CREATE TABLE IF NOT EXISTS kv_store (
        namespace TEXT NOT NULL,
        key TEXT NOT NULL,
        value TEXT,
        PRIMARY KEY (namespace, key)
    );"
    sql "CREATE TABLE IF NOT EXISTS crawl_queue (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        url TEXT UNIQUE NOT NULL,
        depth INTEGER DEFAULT 1,
        domain TEXT,
        priority INTEGER DEFAULT 0,
        discovered_at INTEGER,
        last_attempt INTEGER,
        error_count INTEGER DEFAULT 0
    );"
}

generate_uuid() {
    echo "nt-$(date +%s)-$$-$RANDOM"
}

# Store a concept node
store_concept() {
    local title="$1"
    local summary="$2"
    local url="$3"
    local domain="$4"
    local extra_meta="${5:-}"
    local id
    id=$(generate_uuid)

    # Check for existing by URL
    if [ -n "$url" ]; then
        local existing
        existing=$(sql "SELECT id FROM knowledge_nodes WHERE url = '$url' LIMIT 1")
        if [ -n "$existing" ]; then
            echo "$existing"
            return 0
        fi
    fi

    local meta="{}"
    [ -n "$extra_meta" ] && meta="$extra_meta"

    # Escape single quotes in string values
    local title_esc
    title_esc=$(echo "$title" | sed "s/'/''/g")
    local summary_esc
    summary_esc=$(echo "$summary" | sed "s/'/''/g")
    local url_esc
    url_esc=$(echo "$url" | sed "s/'/''/g")
    local domain_esc
    domain_esc=$(echo "$domain" | sed "s/'/''/g")
    local meta_esc
    meta_esc=$(echo "$meta" | sed "s/'/''/g")

    sql "INSERT OR IGNORE INTO knowledge_nodes (id, node_type, title, summary, url, domain, language, confidence, importance, created_at, updated_at, metadata)
         VALUES ('$id', 'Concept', '$title_esc', '$summary_esc', '$url_esc', '$domain_esc', 'en', 1.0, 0.5, $NOW, $NOW, '$meta_esc');"
    echo "$id"
}

# Store a Repository node
store_repo() {
    local full_name="$1"
    local description="$2"
    local url="$3"
    local stars="$4"
    local language="$5"
    local topics="$6"
    local id
    id=$(generate_uuid)

    local existing
    existing=$(sql "SELECT id FROM knowledge_nodes WHERE url = '$url' LIMIT 1")
    if [ -n "$existing" ]; then
        echo "$existing"
        return 0
    fi

    local fn_esc; fn_esc=$(echo "$full_name" | sed "s/'/''/g")
    local desc_esc; desc_esc=$(echo "$description" | sed "s/'/''/g")
    local url_esc; url_esc=$(echo "$url" | sed "s/'/''/g")
    local topics_esc; topics_esc=$(echo "$topics" | sed "s/'/''/g")

    local meta="'{\"stars\":$stars,\"language\":\"$language\",\"topics\":$topics}'"
    sql "INSERT OR IGNORE INTO knowledge_nodes (id, node_type, title, summary, url, domain, language, confidence, importance, created_at, updated_at, metadata)
         VALUES ('$id', 'Repository', '$fn_esc', '$desc_esc', '$url_esc', 'github.com', 'en', 1.0, 0.5, $NOW, $NOW, $meta);"
    echo "$id"
}

store_edge() {
    local src="$1" tgt="$2" rtype="$3" weight="${4:-0.5}" desc="${5:-}"
    local id
    id=$(generate_uuid)

    local desc_esc
    desc_esc=$(echo "$desc" | sed "s/'/''/g")
    sql "INSERT OR IGNORE INTO knowledge_edges (id, source_id, target_id, relation_type, weight, description, created_at)
         VALUES ('$id', '$src', '$tgt', '$rtype', $weight, '$desc_esc', $NOW);"
}

log_ingest() {
    local stype="$1" surl="$2" status="$3" err="${4:-}"
    local id
    id=$(generate_uuid)
    local surl_esc
    surl_esc=$(echo "$surl" | sed "s/'/''/g")
    local err_esc
    err_esc=$(echo "$err" | sed "s/'/''/g")
    sql "INSERT INTO ingest_log (id, source_type, source_url, status, items_count, started_at, completed_at, error)
         VALUES ('$id', '$stype', '$surl_esc', '$status', 0, $NOW, $NOW, '$err_esc');"
}

# ── GitHub API ──

github_api() {
    local path="$1"
    curl "${CURL_OPTS[@]}" -H "User-Agent: $UA" "https://api.github.com/$path" 2>/dev/null || echo '{}'
}

github_raw() {
    local url="$1"
    curl "${CURL_OPTS[@]}" -H "User-Agent: $UA" "$url" 2>/dev/null || echo ""
}

search_repos_by_topic() {
    local topic="$1"
    local q
    q=$(echo "topic:$topic stars:>100" | sed 's/ /%20/g; s/:/%3A/g; s/>/%3E/g')
    github_api "search/repositories?q=$q&sort=stars&order=desc&per_page=10"
}

get_repo() {
    local owner="$1" repo="$2"
    github_api "repos/$owner/$repo"
}

# ── Absorption Functions ──

absorb_topic() {
    local topic="$1"
    log_info "Searching GitHub topic: $topic"

    local data
    data=$(search_repos_by_topic "$topic")
    local total_count
    total_count=$(echo "$data" | grep -o '"total_count":[0-9]*' | cut -d: -f2 || echo "0")

    if [ "$total_count" = "0" ]; then
        log_info "No repos found for topic '$topic'"
        log_ingest "github_topic" "https://github.com/topics/$topic" "empty" "no repos found"
        return 0
    fi

    # Store topic concept
    local topic_id
    topic_id=$(store_concept "$topic" "GitHub topic: $topic ($total_count repos)" "https://github.com/topics/$topic" "github.com/topic" "{\"source\":\"topic\",\"total_count\":$total_count}")
    log_info "Topic node: $topic_id"

    # Extract top repo names using Python JSON parser
    local names_str
    names_str=$(echo "$data" | python3 -c "
import sys, json
d = json.load(sys.stdin)
items = d.get('items', [])
names = [item['full_name'] for item in items[:5] if 'full_name' in item]
for n in names:
    print(n)
" 2>/dev/null || true)

    local names=()
    while IFS= read -r name; do
        [ -z "$name" ] && continue
        names+=("$name")
    done <<< "$names_str"

    log_info "Found ${#names[@]} repos (of $total_count)"

    # Absorb top repo fully
    if [ "${#names[@]}" -gt 0 ]; then
        local first="${names[0]}"
        absorb_repo "$first"
        local repo_id
        repo_id=$(sql "SELECT id FROM knowledge_nodes WHERE url = 'https://github.com/$first' ORDER BY created_at DESC LIMIT 1")
        if [ -n "$repo_id" ]; then
            store_edge "$repo_id" "$topic_id" "Related" 0.5 "GitHub topic: $topic"
            log_info "Linked repo $first → topic"
        fi
    fi

    # Add remaining repos to crawl queue
    for ((j=1; j<${#names[@]}; j++)); do
        local url="https://github.com/${names[j]}"
        local url_esc
        url_esc=$(echo "$url" | sed "s/'/''/g")
        sql "INSERT OR IGNORE INTO crawl_queue (url, depth, domain, priority, discovered_at)
             VALUES ('$url_esc', 1, 'github.com', 1, $NOW);"
    done

    log_ingest "github_topic" "https://github.com/topics/$topic" "ok" ""
    OK=$((OK+1))
}

absorb_repo() {
    local full_name="$1"
    local owner repo data
    owner=$(echo "$full_name" | cut -d/ -f1)
    repo=$(echo "$full_name" | cut -d/ -f2)

    log_info "Fetching repo: $full_name"
    data=$(get_repo "$owner" "$repo") || true

    local description stars language topics
    description=$(echo "$data" | grep -o '"description":"[^"]*"' | cut -d'"' -f4 2>/dev/null || echo "")
    stars=$(echo "$data" | grep -o '"stargazers_count":[0-9]*' | cut -d: -f2 2>/dev/null || echo "0")
    language=$(echo "$data" | grep -o '"language":"[^"]*"' | cut -d'"' -f4 2>/dev/null || echo "unknown")
    topics=$(echo "$data" | grep -o '"topics":\[[^\]]*\]' 2>/dev/null || echo "[]")

    # Check if response was valid
    local msg
    msg=$(echo "$data" | grep -o '"message":"[^"]*"' 2>/dev/null || true)
    if [ -n "$msg" ]; then
        log_info "API error: $msg"
        log_ingest "github_repo" "https://github.com/$full_name" "error" "$msg"
        FAIL=$((FAIL+1))
        return 1
    fi

    local repo_url="https://github.com/$full_name"
    local repo_id
    repo_id=$(store_repo "$full_name" "$description" "$repo_url" "$stars" "$language" "$topics")

    if [ -z "$repo_id" ]; then
        log_ingest "github_repo" "https://github.com/$full_name" "error" "failed to store"
        FAIL=$((FAIL+1))
        return 1
    fi

    log_info "Stored $full_name (${stars}★, ${language})"

    # Extract topics from JSON and link to topic concepts
    local topic_names
    topic_names=$(echo "$topics" | grep -o '"[a-z0-9_-]*"' | tr -d '"' 2>/dev/null || true)
    for t in $topic_names; do
        local tid
        tid=$(sql "SELECT id FROM knowledge_nodes WHERE title = '$t' AND node_type = 'Concept' AND domain = 'github.com/topic' LIMIT 1")
        if [ -n "$tid" ]; then
            store_edge "$repo_id" "$tid" "Related" 0.5 "GitHub topic: $t"
        fi
    done

    log_ingest "github_repo" "https://github.com/$full_name" "ok" ""
    OK=$((OK+1))
}

absorb_arxiv() {
    local paper_id="$1"
    local url="https://arxiv.org/abs/$paper_id"

    log_info "Fetching ArXiv: $paper_id"

    local data
    data=$(github_raw "https://export.arxiv.org/api/query?id_list=$paper_id" 2>/dev/null || echo "")

    if echo "$data" | grep -qi "error\|not found\|<entry>" >/dev/null 2>&1; then
        # Fall back to just storing the metadata
        local id
        id=$(store_concept "ArXiv $paper_id" "Paper from arxiv: $paper_id" "$url" "arxiv.org" "{\"paper_id\":\"$paper_id\"}")
        log_info "Stored arXiv stub: $id"
    else
        local title
        title=$(echo "$data" | grep -o '<title>[^<]*</title>' | head -1 | sed 's/<[^>]*>//g' | sed 's/^ *//')
        [ -z "$title" ] && title="ArXiv $paper_id"
        local id
        id=$(store_concept "$title" "Paper from arxiv: $paper_id" "$url" "arxiv.org" "{\"paper_id\":\"$paper_id\"}")
        log_info "Stored arXiv paper: $title"
    fi

    log_ingest "arxiv" "https://arxiv.org/abs/$paper_id" "ok" ""
    OK=$((OK+1))
}

# ── Main ──

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║     NeoTrix Batch Knowledge Absorption       ║"
echo "║     (SQLite direct, no Rust compilation)     ║"
echo "╚══════════════════════════════════════════════╝"
echo "  KB: $KB_PATH"
echo "  Token: $([ -n "$GITHUB_TOKEN" ] && echo "✅ set" || echo "❌ not set (60 req/hr limit)")"
echo ""

ensure_schema

# Official binary/paper sources
echo "═══ Individual Repos ═══"
for url in \
    "https://github.com/hanxiao/omni-macos" \
    "https://github.com/jianshuo/ccglass" \
    "https://github.com/jerrywu001/cc-sessions-viewer" \
    "https://github.com/KunAgent/Kun" \
    "https://github.com/Unclecheng-li/VulnClaw" \
    "https://github.com/ZHZisZZ/dllm" \
    "https://github.com/tripleyak/SkillForge"
do
    name=$(echo "$url" | sed 's|https://github.com/||' | sed 's|/$||')
    echo "  [$((OK+FAIL+1))/$((28+7))] Repo: $name ... "
    absorb_repo "$name"
done

echo ""
echo "═══ ArXiv Papers ═══"
echo "  [$((OK+FAIL+1))/$((28+7))] ArXiv: 2605.24517 ... "
absorb_arxiv "2605.24517"

echo ""
echo "═══ GitHub Topic Pages ═══"
TOPICS=(
    openai apple-neural-engine apple-intelligence anthropic
    decision-making ai-memory hooks graph-visualization tree-sitter
    cybersecurity book open-source claude-code automation claude-ai
    claude-skills design-systems design-tools ui-generator generative-ai
    oss self-correction markdown pdf
)

count=1
total_topics=${#TOPICS[@]}
for topic in "${TOPICS[@]}"; do
    echo "  [$((OK+FAIL+1))/$((total_topics+8))] Topic: $topic ... "
    absorb_topic "$topic"
    count=$((count+1))
    # Throttle to avoid rate limiting
    sleep 1
done

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║     Absorption Complete                       ║"
echo "╚══════════════════════════════════════════════╝"
echo "  Total: $OK OK, $FAIL Failed"
echo "  KB: $KB_PATH"
echo "  Run 'sqlite3 \"$KB_PATH\" \"SELECT COUNT(*) FROM knowledge_nodes\"' to check node count."
