#!/bin/bash
# NeoTrix 50轮外部数据采集循环 — macOS兼容版 v3

OUTDIR="/tmp/neotrix-ingest-50r"
mkdir -p "$OUTDIR"
RESULTS="$OUTDIR/results.json"

TOTAL_ROUNDS=50
TOTAL_START=$(date +%s)
echo "╔══════════════════════════════════════════════════╗"
echo "║  NeoTrix 50轮外部数据采集循环                     ║"
echo "╚══════════════════════════════════════════════════╝"

total_records=0
hn_all=0; arxiv_all=0; github_all=0; scholar_all=0

round_times_file=$(mktemp)

python3_parse_arxiv() {
    python3 -c "
import sys, re
try:
    data = sys.stdin.read()
    titles = re.findall(r'<title>([^<]+)', data)
    # Skip the feed title (first one if it's the feed)
    if titles and len(titles) > 1:
        titles = [t for t in titles if t != 'ArXiv Query: search_query=cat:cs.AI&sortBy=submittedDate&start=0&max_results=10']
    print(len(titles))
except:
    print(0)
" 2>/dev/null || echo 0
}

for round in $(seq 1 $TOTAL_ROUNDS); do
    ROUND_START=$(date +%s)
    echo "━━━ Round $round/$TOTAL_ROUNDS ━━━"

    pair=$(( (round - 1) / 2 % 4 ))
    case $pair in
        0) s1="hn"; s2="arxiv" ;;
        1) s1="github"; s2="scholar" ;;
        2) s1="arxiv"; s2="github" ;;
        3) s1="hn"; s2="scholar" ;;
    esac

    round_recs=0

    for src in "$s1" "$s2"; do
        [ -z "$src" ] && continue
        SRC_S=$(date +%s)

        case "$src" in
            hn)
                ids=$(curl -s "https://hacker-news.firebaseio.com/v0/topstories.json" 2>/dev/null | tr ',' '\n' | grep -oE '[0-9]+' | head -10)
                cnt=0
                for id in $ids; do
                    title=$(curl -s "https://hacker-news.firebaseio.com/v0/item/$id.json" 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('title','') or '')" 2>/dev/null)
                    [ -n "$title" ] && cnt=$((cnt + 1))
                done
                elapsed=$(( $(date +%s) - SRC_S ))
                echo "  📡 HN: $cnt stories [${elapsed}s]"
                hn_all=$((hn_all + cnt))
                round_recs=$((round_recs + cnt))
                ;;
            arxiv)
                resp=$(curl -s "http://export.arxiv.org/api/query?search_query=cat:cs.AI&sortBy=submittedDate&start=0&max_results=10" 2>/dev/null)
                cnt=$(echo "$resp" | python3_parse_arxiv)
                elapsed=$(( $(date +%s) - SRC_S ))
                echo "  📡 arXiv: $cnt papers [${elapsed}s]"
                arxiv_all=$((arxiv_all + cnt))
                round_recs=$((round_recs + cnt))
                ;;
            github)
                resp=$(curl -s "https://api.github.com/search/repositories?q=stars:>1000+pushed:>2026-01-01&sort=stars&per_page=5" 2>/dev/null)
                cnt=$(echo "$resp" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('items',[])))" 2>/dev/null || echo 0)
                elapsed=$(( $(date +%s) - SRC_S ))
                echo "  📡 GitHub: $cnt repos [${elapsed}s]"
                github_all=$((github_all + cnt))
                round_recs=$((round_recs + cnt))
                ;;
            scholar)
                resp=$(curl -s "https://api.semanticscholar.org/graph/v1/paper/search?query=AI+reasoning&limit=5&fields=title,year" 2>/dev/null)
                cnt=$(echo "$resp" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('data',[])))" 2>/dev/null || echo 0)
                elapsed=$(( $(date +%s) - SRC_S ))
                echo "  📡 Scholar: $cnt papers [${elapsed}s]"
                scholar_all=$((scholar_all + cnt))
                round_recs=$((round_recs + cnt))
                ;;
        esac
    done

    total_records=$((total_records + round_recs))
    rtime=$(( $(date +%s) - ROUND_START ))
    echo "$rtime" >> "$round_times_file"

    total_elapsed=$(( $(date +%s) - TOTAL_START ))
    echo "  📊 +${round_recs} records | round=${rtime}s ∑=${total_elapsed}s"
    echo ""
    sleep 0.3
done

TOTAL_SECONDS=$(( $(date +%s) - TOTAL_START ))

# Compute stats
sort -n "$round_times_file" > /tmp/st_$$.txt
fastest=$(head -1 /tmp/st_$$.txt)
slowest=$(tail -1 /tmp/st_$$.txt)
total_rt=0; n=0
while IFS= read -r t; do
    total_rt=$((total_rt + t))
    n=$((n + 1))
done < /tmp/st_$$.txt
avg=$(( n > 0 ? total_rt / n : 0 ))
rm -f "$round_times_file" /tmp/st_$$.txt

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  50 轮采集完成                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "  📊 总计:"
echo "     Records collected: $total_records"
echo "     Total real time:   ${TOTAL_SECONDS}s"
echo ""
echo "  📡 Per source:"
echo "     HN:      $hn_all"
echo "     arXiv:   $arxiv_all"
echo "     GitHub:  $github_all"
echo "     Scholar: $scholar_all"
echo ""
echo "  ⏱ Round times:"
echo "     Fastest: ${fastest}s"
echo "     Slowest: ${slowest}s"
echo "     Average: ${avg}s"
echo ""

python3 << EOF
import json
summary = {
    "total_records": $total_records,
    "total_seconds": $TOTAL_SECONDS,
    "source_hn": $hn_all,
    "source_arxiv": $arxiv_all,
    "source_github": $github_all,
    "source_scholar": $scholar_all,
    "fastest_round_s": $fastest,
    "slowest_round_s": $slowest,
    "avg_round_s": $avg,
}
with open("$RESULTS", "w") as f:
    json.dump(summary, f, indent=2)
print("  💾 Results: $RESULTS")
print("  ✅ Done!")
EOF
