#!/usr/bin/env python3
"""
NeoTrix Novel World Architecture Absorber — Qidian MCP 驱动

吸收:
  1. 起点中文网 14个排行榜 × 15个品类 → 爬取前20本书
  2. 每本书的详细信息 (书名/作者/分类/简介/字数/标签)
  3. 世界观架构分析 (设定类型/力量体系/修炼境界/世界层级)
  4. AI 辅助拆书 (可选, 需 ANTHROPIC_API_KEY)

注入 KB:
  - Book 节点 + 世界观分析元数据
  - Concept 节点: 世界观类型 / 力量体系 / 境界 / 作者 / 分类
  - related_to / about_topic / belongs_to / categorized / developed_by 边

运行:
  python3 scripts/novel-world-absorb.py [--cycles N] [--interval SEC] [--rankings N] [--daemon] [--once]
  python3 scripts/novel-world-absorb.py --once          # 单次快速测试
  python3 scripts/novel-world-absorb.py --rankings 2    # 只爬前2个排行榜
"""

import sys, os, json, time, sqlite3, hashlib, re, signal, traceback, random, asyncio

# ── Path setup: qidian-mcp-server ──
QIDIAN_MCP_DIR = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "qidian-mcp-server"))
VENV_PYTHON = os.path.join(QIDIAN_MCP_DIR, ".venv", "bin", "python3")

# Re-exec with venv to access playwright
if sys.executable != VENV_PYTHON and os.path.exists(VENV_PYTHON):
    os.execv(VENV_PYTHON, [VENV_PYTHON] + sys.argv)

sys.path.insert(0, QIDIAN_MCP_DIR)
from scraper import scrape_ranking, scrape_book_detail
from config import RANKINGS, GENRES, ranking_url, book_url

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
LOG_PATH = os.path.expanduser("~/.neotrix/novel-absorb-log.jsonl")
PID_PATH = os.path.expanduser("~/.neotrix/novel-absorb.pid")
STARTED_AT = int(time.time())
CYCLE_COUNT = 0
SHUTDOWN = False


# ── World Architecture Patterns ──
WORLD_PATTERNS = [
    ("Xianxia", ["修真", "修仙", "仙侠", "渡劫", "元婴", "金丹", "飞升", "灵根", "法宝", "功法", "道心", "天人", "仙", "凡"],
     "Immortal Ascension", "Qi Cultivation (修真)"),
    ("Xuanhuan", ["玄幻", "斗气", "魔法", "魔兽", "异界", "大陆", "位面", "神格", "领域", "斗帝", "神", "魔"],
     "Fantasy World", "Magic / Battle Qi (斗气魔法)"),
    ("Urban Supernatural", ["都市", "异能", "超能力", "现代", "校园", "娱乐圈", "重生", "保镖", "医生", "警察"],
     "Modern Earth", "Superpower (异能)"),
    ("Science Fiction", ["科幻", "星际", "机甲", "未来", "赛博", "人工智能", "基因", "宇宙", "时空", "星舰", "机器人"],
     "Interstellar / Futuristic", "Technology (科技)"),
    ("Historical", ["历史", "三国", "穿越", "古代", "王朝", "争霸", "帝王", "架空", "重生", "民国"],
     "Historical Earth", "Strategy / Martial Arts"),
    ("Game World", ["游戏", "电竞", "虚拟现实", "网游", "副本", "技能", "属性", "面板", "职业", "系统"],
     "Virtual World", "Game Mechanics (游戏系统)"),
    ("Wuxia", ["武侠", "江湖", "内力", "武功", "武林", "掌门", "宗师", "帮派", "侠客", "剑"],
     "Martial World", "Internal Energy (真气武学)"),
    ("Light Novel", ["轻小说", "二次元", "动漫", "同人", "综漫", "冒险", "异世界"],
     "Anime World", "Various / System"),
    ("Mystery / Horror", ["悬疑", "恐怖", "灵异", "惊悚", "盗墓", "鬼怪", "死亡游戏", "推理", "侦探"],
     "Dark Modern", "Supernatural / Curses"),
    ("Steampunk / Occult", ["蒸汽", "神秘学", "克苏鲁", "魔藥", "符文", "教会", "超凡", "非凡"],
     "Alternate History / Arcane", "Occult / Alchemy"),
]

REALM_KEYWORDS = [
    "炼气", "筑基", "金丹", "元婴", "化神", "合体", "渡劫", "大乘", "仙人",
    "斗者", "斗师", "斗王", "斗皇", "斗宗", "斗尊", "斗圣", "斗帝",
    "学徒", "战士", "师级", "王级", "皇级", "帝级", "神级",
    "凡人", "超凡", "圣者", "神话", "半神",
]


def _handle_sigterm(signum, frame):
    global SHUTDOWN; SHUTDOWN = True

signal.signal(signal.SIGTERM, _handle_sigterm)
signal.signal(signal.SIGINT, _handle_sigterm)


def log(level, phase, message, extra=None):
    entry = {"ts": int(time.time()), "level": level, "cycle": CYCLE_COUNT, "phase": phase,
             "message": message, "uptime_sec": int(time.time()) - STARTED_AT}
    if extra: entry["data"] = extra
    os.makedirs(os.path.dirname(LOG_PATH), exist_ok=True)
    with open(LOG_PATH, "a") as f:
        f.write(json.dumps(entry) + "\n"); f.flush()
    prefix = f"[{time.strftime('%H:%M:%S')}][C{CYCLE_COUNT}][{phase}]"
    icon = {"ERROR": " ❌", "WARN": " ⚠", "INFO": " ", "OK": " ✅"}.get(level, " ")
    print(f"  {prefix}{icon} {message}", flush=True)


_seq = 0
def record_defect(dtype, source, desc, severity=0.5):
    global _seq; _seq += 1
    log("WARN", "defect", f"{dtype}: {desc}")
    try:
        db = _get_db(); now = int(time.time())
        key = f"novel_{CYCLE_COUNT}_{now:x}_{_seq:04x}"
        val = json.dumps({"defect_type":dtype,"source":source,"description":desc,"severity":severity,"ts":now,"cycle":CYCLE_COUNT})
        _exec(db, "INSERT OR IGNORE INTO kv_store (namespace,key,value,updated_at) VALUES (?,?,?,?)",
              ("meta_cognition", key, val, now))
        db.commit()
    except: pass


_db = None
def _get_db():
    global _db
    if _db is None:
        _db = sqlite3.connect(KB_PATH, timeout=120)
        _db.execute("PRAGMA journal_mode=WAL")
        _db.execute("PRAGMA busy_timeout=60000")
        _db.execute("PRAGMA synchronous=NORMAL")
    return _db

def _exec(db, sql, params=None):
    for i in range(5):
        try: return db.execute(sql, params) if params else db.execute(sql)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < 4: time.sleep(1.0*(i+1))
            else: raise

def _f1(db, sql, p=None):
    try: c = _exec(db, sql, p); return c.fetchone()
    except: return None

def _nid(url): return "nt-"+hashlib.md5(url.encode()).hexdigest()[:20]
def _genid(): return "nt-"+hashlib.md5((str(time.time())+str(random.random())).encode()).hexdigest()[:20]

def _edge(db, src, tgt, rel, w=0.5, desc=None):
    if not src or not tgt or src == tgt: return False
    try:
        _exec(db, "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,created_at,description) VALUES (?,?,?,?,?,?,?)",
              (f"n_{src[:12]}_{tgt[:12]}_{rel[:6]}", src, tgt, rel, w, int(time.time()), desc))
        return True
    except: return False

def _store_book(db, nid, title, author, category, summary, url):
    now = int(time.time())
    meta = json.dumps({"author":author,"category":category,"source":"qidian"})
    try:
        _exec(db, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,metadata) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
              (nid, "Book", title, (summary or "")[:2000], (summary or "")[:2000], url, "qidian.com", "zh", 0.8, 0.6, now, now, meta))
        _exec(db, "INSERT OR IGNORE INTO nodes_fts(rowid,title,summary,content,domain) VALUES ((SELECT rowid FROM nodes WHERE id=?),?,?,?,?)",
              (nid, title, (summary or "")[:2000], (summary or "")[:2000], "qidian.com"))
        return True
    except: return False

def _store_concept(db, nid, title, summary, domain="novel_world"):
    now = int(time.time())
    try:
        _exec(db, "INSERT OR IGNORE INTO nodes (id,node_type,title,summary,content,domain,language,confidence,importance,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
              (nid, "Concept", title, summary, summary, domain, "zh", 0.9, 0.5, now, now))
        _exec(db, "INSERT OR IGNORE INTO nodes_fts(rowid,title,summary,content,domain) VALUES ((SELECT rowid FROM nodes WHERE id=?),?,?,?,?)",
              (nid, title, summary, summary, domain))
        return True
    except: return False

def _update_fts(db):
    try:
        _exec(db, "INSERT OR REPLACE INTO nodes_fts(rowid,title,summary,content,domain) SELECT rowid,title,COALESCE(summary,''),COALESCE(content,''),COALESCE(domain,'') FROM nodes WHERE updated_at>=?",
              (int(time.time())-300,))
    except: pass


def classify_setting(title, summary, genre, tags):
    """Classify novel's world architecture from all available signals."""
    text = f"{title} {summary} {genre} {' '.join(tags)}".lower()
    best_score, best = 0, None
    for name, keywords, tier, power in WORLD_PATTERNS:
        score = sum(3 for k in keywords if k.lower() in genre.lower())  # genre match = high weight
        score += sum(1 for k in keywords if k in text)
        if score > best_score:
            best_score, best = score, (name, tier, power)
    return best or ("General", "Unknown", "Mixed")


def extract_realms(text):
    return [kw for kw in REALM_KEYWORDS if kw in (text or "")]


# ── Qidian Async Scraper Integration ──

async def run_qidian_ranking(ranking_name, genre_name, detail_count=5):
    """Scrape one ranking page and enrich with book details."""
    result = await scrape_ranking(ranking_name, genre_name)
    books = result.get("books", [])
    log("INFO", "qidian", f"Ranking '{ranking_name}'/{genre_name}: {len(books)} books")

    # Enrich with book details for top N
    for book in books[:detail_count]:
        bid = book.get("book_id", "")
        if not bid: continue
        try:
            detail = await scrape_book_detail(bid)
            if detail:
                book["detail"] = detail
                book["synopsis"] = detail.get("synopsis", "")
                book["tags"] = detail.get("tags", [])
                book["word_count"] = detail.get("word_count", "")
                book["sub_genre"] = detail.get("sub_genre", "")
                book["status"] = detail.get("status", "")
                book["chapter_count"] = detail.get("chapter_count", 0)
        except Exception as e:
            log("WARN", "qidian", f"  Detail failed for {book.get('title','?')}: {e}")

    return result


def ingest_batch(db, results, cycle_label=""):
    """Inject scraped ranking results into KB."""
    total_books = 0
    total_edges = 0
    seen = set()

    for result in results:
        ranking_name = result.get("ranking", "?")
        genre_name = result.get("genre", "?")
        books = result.get("books", [])

        for book in books:
            try:
                title = book.get("title", "").strip()
                if not title or title in seen: continue
                seen.add(title)

                author = book.get("author", "")
                genre = book.get("genre", "") or genre_name
                bid = book.get("book_id", "")
                url_full = book.get("book_url", "") or f"https://www.qidian.com/book/{bid}/"
                rank = book.get("rank", 0)

                # Detail-enriched fields
                detail = book.get("detail", {})
                synopsis = book.get("synopsis", "") or detail.get("synopsis", "")
                tags = book.get("tags", []) or detail.get("tags", [])
                sub_genre = book.get("sub_genre", "") or detail.get("sub_genre", "")
                word_count = book.get("word_count", "") or detail.get("word_count", "")
                status = book.get("status", "") or detail.get("status", "")
                chapter_count = book.get("chapter_count", 0) or detail.get("chapter_count", 0)

                # Full category string
                full_cat = f"{genre} {sub_genre}".strip()

                # World architecture analysis
                stype, stier, spower = classify_setting(title, synopsis, full_cat, tags)
                realms = extract_realms(f"{title} {synopsis} {full_cat}")

                nid = _nid(url_full)

                # Build content metadata
                content_parts = [
                    f"来源: 起点中文网({ranking_name}) | 排名: #{rank}",
                    f"作者: {author}",
                    f"分类: {full_cat}",
                    f"字数: {word_count} | 章节: {chapter_count} | 状态: {status}",
                    f"世界观: {stype} | 层级: {stier} | 力量: {spower}",
                ]
                if realms:
                    content_parts.append(f"境界: {', '.join(realms)}")
                content = "\n".join(content_parts)

                if not _store_book(db, nid, title, author, full_cat, synopsis or content, url_full):
                    continue
                total_books += 1

                # World setting concept
                sid = _nid(f"ws_{stype}")
                _store_concept(db, sid, f"世界观: {stype}", f"小说世界观: {stype}。层级: {stier}。力量: {spower}", "novel_world_arch")
                if _edge(db, nid, sid, "about_topic", 0.85, f"世界观类型: {stype}"): total_edges += 1

                # Power system
                if spower and "Unknown" not in spower and "Mixed" not in spower:
                    pid = _nid(f"ps_{spower}")
                    _store_concept(db, pid, f"力量体系: {spower}", f"{title} 力量体系: {spower}", "novel_power")
                    if _edge(db, nid, pid, "related_to", 0.75, f"力量体系: {spower}"): total_edges += 1

                # Realms
                for r in realms:
                    rid = _nid(f"realm_{r}")
                    _store_concept(db, rid, f"境界: {r}", f"修炼境界: {r}", "novel_realm")
                    if _edge(db, nid, rid, "belongs_to", 0.65, f"包含境界: {r}"): total_edges += 1

                # Author
                if author:
                    aid = _nid(f"author_{author}")
                    _store_concept(db, aid, f"作者: {author}", f"起点中文网作者: {author}", "novel_author")
                    if _edge(db, aid, nid, "developed_by", 0.5, f"作者: {author}"): total_edges += 1

                # Genre category
                if genre:
                    gid = _nid(f"genre_{genre}")
                    _store_concept(db, gid, f"分类: {genre}", f"小说品类: {genre}", "novel_genre")
                    if _edge(db, nid, gid, "categorized", 0.7, f"品类: {genre}"): total_edges += 1

                # Ranking source
                if ranking_name:
                    rid = _nid(f"ranking_{ranking_name}")
                    _store_concept(db, rid, f"榜单: {ranking_name}", f"起点中文网排行榜: {ranking_name}", "novel_ranking")
                    if _edge(db, nid, rid, "categorized", 0.4, f"来源榜单: {ranking_name}"): total_edges += 1

            except Exception as e:
                log("WARN", "ingest", f"Error: {book.get('title','?')}: {e}")
                record_defect("INGEST_ERROR", "ingest", str(e), 0.3)

        db.commit()

    db.commit()
    _update_fts(db)
    db.commit()
    return total_books, total_edges


# ── Main Cycle ──

async def async_cycle(db, ranking_limit=5, detail_limit=3):
    """Run one async absorption cycle."""
    log("INFO", "async", "Starting async Qidian scan...")
    all_results = []

    # Rankings to scan: pick highest-value ones
    priority_rankings = ["月票榜", "畅销榜", "阅读指数榜", "推荐榜", "收藏榜", "书友榜", "更新榜", "签约作者新书榜", "新人签约新书榜", "新人作者新书榜"]

    tasks = []
    for rname in priority_rankings[:ranking_limit]:
        rname_cn = rname
        # Scan '全部' genre for broad coverage, plus 玄幻 and 都市 for focused world-building data
        for gname in ["全部", "玄幻", "都市", "科幻", "仙侠", "历史", "轻小说"][:3]:
            task = run_qidian_ranking(rname_cn, gname, detail_limit)
            tasks.append((rname_cn, gname, task))

    # Run in parallel with throttling (3 at a time)
    from asyncio import Semaphore
    sem = Semaphore(3)

    async def bounded(rname, gname, task):
        async with sem:
            try:
                return await task
            except Exception as e:
                log("ERROR", "qidian", f"Failed {rname}/{gname}: {e}")
                record_defect("QIDIAN_FAIL", "qidian", str(e), 0.5)
                return None

    coros = [bounded(r, g, t) for r, g, t in tasks]
    results = await asyncio.gather(*coros, return_exceptions=True)

    for r in results:
        if r and not isinstance(r, BaseException):
            all_results.append(r)

    log("INFO", "async", f"Scanned {len(all_results)} ranking pages")
    return all_results


def run_async_cycle(db, ranking_limit=5, detail_limit=3):
    """Synchronous wrapper for async cycle."""
    results = asyncio.run(async_cycle(db, ranking_limit, detail_limit))
    log("INFO", "main", f"Absorbing {sum(len(r.get('books',[])) for r in results)} books into KB...")
    books, edges = ingest_batch(db, results)
    return books, edges


def snapshot(db):
    info = {}
    for row in _exec(db, "SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type ORDER BY COUNT(*) DESC").fetchall():
        info[f"n_{row[0]}"] = row[1]
    info["total"] = _f1(db, "SELECT COUNT(*) FROM nodes")[0]
    info["edges"] = _f1(db, "SELECT COUNT(*) FROM edges")[0]
    info["empty"] = _f1(db, "SELECT COUNT(*) FROM nodes WHERE content IS NULL OR content=''")[0]
    info["books"] = _f1(db, "SELECT COUNT(*) FROM nodes WHERE node_type='Book'")[0]
    log("OK", "snap", f"KB: {info['total']} nodes, {info['edges']} edges, {info['books']} books, {info['empty']} empty")
    _exec(db, "INSERT OR REPLACE INTO kv_store (namespace,key,value,updated_at) VALUES (?,?,?,?)",
          ("meta_cognition", f"novel_snap_{CYCLE_COUNT}", json.dumps(info), int(time.time())))
    db.commit()
    return info


def run_cycle(db, ranking_limit=5, detail_limit=3):
    global CYCLE_COUNT
    CYCLE_COUNT += 1
    log("INFO", "main", f"Starting Cycle {CYCLE_COUNT}")
    books, edges = run_async_cycle(db, ranking_limit, detail_limit)
    snapshot(db)
    log("OK", "main", f"Cycle {CYCLE_COUNT}: {books} books, {edges} edges")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="NeoTrix Novel World Architecture Absorber (Qidian)")
    parser.add_argument("--cycles", type=int, default=None, help="Number of cycles")
    parser.add_argument("--interval", type=int, default=600, help="Seconds between cycles")
    parser.add_argument("--rankings", type=int, default=5, help="Rankings per cycle")
    parser.add_argument("--detail", type=int, default=3, help="Books per ranking for detailed enrich")
    parser.add_argument("--once", action="store_true", help="Single cycle and exit")
    parser.add_argument("--daemon", action="store_true", help="Fork to background")
    args = parser.parse_args()

    if args.once: args.cycles = 1

    if args.daemon:
        pid = os.fork()
        if pid > 0:
            with open(PID_PATH, "w") as f: f.write(str(pid))
            print(f"Daemon PID {pid}")
            sys.exit(0)

    with open(PID_PATH, "w") as f: f.write(str(os.getpid()))

    print(f"NeoTrix Novel World Architecture Absorber")
    print(f"  KB:      {KB_PATH}")
    print(f"  Qidian:  {QIDIAN_MCP_DIR}")
    print(f"  Rankings: {args.rankings}/cycle, Details: {args.detail}/book")

    db = _get_db()
    goal = args.cycles or float('inf')
    cycle = 0
    start = time.time()

    while cycle < goal and not SHUTDOWN:
        try:
            run_cycle(db, args.rankings, args.detail)
        except Exception as e:
            log("ERROR", "main", f"Cycle failed: {e}\n{traceback.format_exc()}")
            record_defect("MAIN_FAIL", "main", str(e), 0.9)
        cycle += 1
        log("INFO", "main", f"Runtime: {(time.time()-start)/3600:.1f}h")
        if not SHUTDOWN and cycle < goal:
            for _ in range(args.interval // 5):
                if SHUTDOWN: break
                time.sleep(5)

    try: snapshot(db)
    except: pass
    db.close()
    if os.path.exists(PID_PATH): os.remove(PID_PATH)


if __name__ == "__main__":
    main()
