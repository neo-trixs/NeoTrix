#!/usr/bin/env python3
"""NeoTrix 50轮数据采集 — 智能轮换 + 限速避让"""

import json, time, urllib.request, urllib.parse, re, os, sys
from datetime import datetime

OUTDIR = "/tmp/neotrix-ingest-50r"
os.makedirs(OUTDIR, exist_ok=True)

SOURCES = {
    "hn":      {"url": "https://hacker-news.firebaseio.com/v0/topstories.json", "parser": "hn"},
    "arxiv":   {"url": "http://export.arxiv.org/api/query?search_query=cat:cs.AI&sortBy=submittedDate&start=0&max_results=10", "parser": "arxiv"},
    "github":  {"url": "https://api.github.com/search/repositories?q=stars:>1000+pushed:>2026-01-01&sort=stars&per_page=5", "parser": "github"},
    "scholar": {"url": "https://api.semanticscholar.org/graph/v1/paper/search?query=AI+reasoning&limit=5&fields=title,year", "parser": "scholar"},
}

# Diverse query pool for arXiv rotations
ARXIV_QUERIES = [
    "cat:cs.AI", "cat:cs.LG", "cat:cs.CL", "cat:cs.CV",
    "cat:cs.RO", "cat:cs.NE", "cat:cs.MA", "cat:cs.SE",
    "ti:neural+network", "ti:transformer", "ti:reinforcement+learning",
    "ti:knowledge+graph", "ti:reasoning", "ti:memory",
    "all:consciousness", "all:world+model", "all:agent",
    "all:emergence", "all:attention", "all:deep+learning",
    "all:robot", "all:planning", "all:decision+mak",
]

# Diverse query pool for Semantic Scholar
SCHOLAR_QUERIES = [
    "AI reasoning", "machine learning", "deep learning", "neural network",
    "reinforcement learning", "knowledge graph", "transformer", "attention mechanism",
    "world model", "consciousness", "emergence", "agent architecture",
    "memory systems", "planning algorithms", "decision theory",
    "natural language", "computer vision", "robotics",
    "causal inference", "Bayesian methods", "information theory",
    "cognitive architecture", "self-supervised learning", "meta-learning",
]

# Diverse query pool for GitHub
GITHUB_QUERIES = [
    "stars:>1000+pushed:>2026-01-01", "topic:machine-learning+stars:>500",
    "topic:deep-learning+pushed:>2026-01-01", "topic:rust+stars:>2000",
    "topic:python+stars:>5000", "topic:typescript+stars:>2000",
    "topic:go+stars:>1000", "topic:react+stars:>3000",
    "topic:database+stars:>1000", "topic:cli+stars:>500",
    "topic:transformer+stars:>100", "topic:agent+stars:>100",
    "topic:web+stars:>2000", "topic:ai+stars:>500",
    "topic:data+stars:>1000", "topic:graph+stars:>200",
    "pushed:>2026-05-01+stars:>100", "topic:compiler+stars:>300",
    "topic:game+stars:>1000", "topic:security+stars:>500",
]

TOTAL_ROUNDS = 50
TOTAL_START = time.time()

print("╔══════════════════════════════════════════════════╗")
print("║  NeoTrix 50轮外部数据采集循环 (Python)           ║")
print("╚══════════════════════════════════════════════════╝")

def fetch(url, timeout=10):
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "NeoTrix/0.18"})
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.read().decode("utf-8", errors="replace")
    except Exception as e:
        return None

def parse_hn(data):
    if not data: return 0
    try:
        ids = json.loads(data)[:15]
        # Just count from top stories list, don't fetch individual items
        return len(ids)
    except: return 0

def parse_arxiv(data):
    if not data: return 0
    titles = re.findall(r'<title[^>]*>([^<]+)', data)
    # Filter out the feed title
    titles = [t for t in titles if t.strip() and "ArXiv Query" not in t]
    return len(titles)

def parse_github(data):
    if not data: return 0
    try:
        d = json.loads(data)
        return len(d.get("items", []))
    except: return 0

def parse_scholar(data):
    if not data: return 0
    try:
        d = json.loads(data)
        return len(d.get("data", []))
    except: return 0

parsers = {
    "hn": parse_hn, "arxiv": parse_arxiv,
    "github": parse_github, "scholar": parse_scholar,
}

total_recs = 0
counts = {"hn": 0, "arxiv": 0, "github": 0, "scholar": 0}
round_times = []

for round in range(1, TOTAL_ROUNDS + 1):
    ROUND_START = time.time()
    print(f"━━━ Round {round}/{TOTAL_ROUNDS} ━━━")

    # Smart source selection: rotate through pairs with diverse queries
    pair = (round - 1) % 8
    if pair < 2:
        sources = ["hn", "arxiv"]
    elif pair < 4:
        sources = ["github", "scholar"]
    elif pair < 6:
        sources = ["arxiv", "github"]
    else:
        sources = ["hn", "scholar"]

    # Every 7th round, use only 1 source with higher volume
    if round % 7 == 0:
        sources = ["arxiv"]

    q_idx = (round - 1) % len(ARXIV_QUERIES)
    s_idx = (round - 1) % len(SCHOLAR_QUERIES)
    g_idx = (round - 1) % len(GITHUB_QUERIES)

    round_recs = 0

    for src in sources:
        S = time.time()
        cfg = SOURCES[src].copy()

        # Apply query rotation
        if src == "arxiv":
            cfg["url"] = f"http://export.arxiv.org/api/query?search_query={ARXIV_QUERIES[q_idx]}&sortBy=submittedDate&start=0&max_results=10"
        elif src == "scholar":
            cfg["url"] = f"https://api.semanticscholar.org/graph/v1/paper/search?query={urllib.parse.quote(SCHOLAR_QUERIES[s_idx])}&limit=5&fields=title,year"
        elif src == "github":
            cfg["url"] = f"https://api.github.com/search/repositories?q={GITHUB_QUERIES[g_idx]}&sort=stars&per_page=5"

        data = fetch(cfg["url"])
        cnt = parsers[src](data)
        elapsed = time.time() - S

        if cnt > 0:
            counts[src] += cnt
            round_recs += cnt
            print(f"  📡 {src:8s}: {cnt} records [{elapsed:.0f}s] (query: {q_idx if src=='arxiv' else s_idx if src=='scholar' else g_idx if src=='github' else 0})")
        else:
            print(f"  📡 {src:8s}: 0 [{elapsed:.0f}s] (empty/error)")

        # Brief pause between sources
        time.sleep(0.5)

    total_recs += round_recs
    rt = time.time() - ROUND_START
    round_times.append(rt)
    total_elapsed = time.time() - TOTAL_START

    print(f"  📊 +{round_recs} records | round={rt:.0f}s ∑={total_elapsed:.0f}s")
    print()

    # Adaptive backoff: if we got 0 from all sources, rest longer
    if round_recs == 0:
        time.sleep(3)
    else:
        time.sleep(0.5)

total_time = time.time() - TOTAL_START
fastest = min(round_times)
slowest = max(round_times)
avg = sum(round_times) / len(round_times)

print("╔══════════════════════════════════════════════════════════════╗")
print("║  50 轮采集完成                                              ║")
print("╚══════════════════════════════════════════════════════════════╝")
print()
print(f"  📊 总计:")
print(f"     Records collected: {total_recs}")
print(f"     Total time:        {total_time:.0f}s")
print()
print(f"  📡 Per source:")
for src in ["hn", "arxiv", "github", "scholar"]:
    print(f"     {src:8s}: {counts[src]}")
print()
print(f"  ⏱  Round times:")
print(f"     Fastest: {fastest:.0f}s")
print(f"     Slowest: {slowest:.0f}s")
print(f"     Average: {avg:.0f}s")
print()

summary = {
    "total_records": total_recs,
    "total_seconds": round(total_time),
    "source_hn": counts["hn"],
    "source_arxiv": counts["arxiv"],
    "source_github": counts["github"],
    "source_scholar": counts["scholar"],
    "fastest_round_s": round(fastest),
    "slowest_round_s": round(slowest),
    "avg_round_s": round(avg),
}
with open(os.path.join(OUTDIR, "results.json"), "w") as f:
    json.dump(summary, f, indent=2)

print(f"  💾 {OUTDIR}/results.json")
print("  ✅ Done!")
