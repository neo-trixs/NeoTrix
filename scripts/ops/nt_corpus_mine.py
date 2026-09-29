#!/usr/bin/env python3
"""nt_corpus_mine — 22M 语料采矿管线（等卷状态，卷回即开工）.

链路：卷上 68G 库 FTS 概念探测 → 抽取式蒸馏（英文原样，不编造中文）
  → 标题级防撞 → 卦符归属 → mined-review 富行 → digest --rich 入茧。
认识论纪律（对齐反幻觉门）：矿出条目一律【待润色】+ 0.55 置信进
  `mined-review` 域，不直入 source-core；人工/agent 复核后转正。
  无证据（FTS 无命中）概念直接跳过，不写空条。

卷离线时：检测不到库文件即打印等卷并正常退出（exit 0），
  不报错不空转。卷回后按 `--limit/--offset` 切片采矿，
  状态记 `datasets/mining/_state.json`，支持 100+ 轮循环。

用法:
  python3 scripts/ops/nt_corpus_mine.py              # 卷在则采 50 条，否则等卷
  python3 scripts/ops/nt_corpus_mine.py --limit 100 --offset 50
  python3 scripts/ops/nt_corpus_mine.py --selftest   # 内存库全链路自检
"""
import argparse
import json
import os
import sqlite3
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from nt_symbol_brain import LIT  # noqa: E402  # 簇key→(卦名,断语)，序即M-id序
from nt_foundation_laws import (  # noqa: E402  # 卦符 M-id 活库反查（唯一来源）
    live_symbol_mids, live_mid_set)

VOL_DB = "/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db"
OUT_DIR = os.path.join("datasets", "mining")
OUT_JSONL = os.path.join(OUT_DIR, "mined_chains.jsonl")
STATE_JSON = os.path.join(OUT_DIR, "_state.json")
# 原 BASE_MID = 479512（裸数字，静默扫描器看不见）已删。
# 那是档案期号段，活库 M-000001..M-065651 里 M-479512 从不存在 → 每条矿出
# 记忆的 connections 都指向空气（悬空边逃过记忆级校验）。现在改为运行时
# 从活库 neuro-symbols 域按卦名反查真实 M-id（live_symbol_mids）。
# 反查不到 → SystemExit，不回退任何常量。
_LIT_TABLE = None


def lit_table():
    """卦名 → 活库 M-id（进程内缓存；每行都调，不能每次重扫 52MB）。"""
    global _LIT_TABLE
    if _LIT_TABLE is None:
        _LIT_TABLE = live_symbol_mids()
    return _LIT_TABLE

# （FTS 查询，域提示）—— 跨学科种子队列，可按 offset 切片循环
CONCEPTS = [
    ("Bernoulli principle", "physics"), ("Ohm law", "physics"),
    ("Schrodinger cat thought", "physics"), ("double slit experiment", "physics"),
    ("Carnot cycle", "physics"), ("Doppler effect", "physics"),
    ("saponification", "chemistry"), ("esterification", "chemistry"),
    ("Avogadro number", "chemistry"), ("mole concept", "chemistry"),
    ("mitosis phases", "biology"), ("meiosis", "biology"),
    ("reflex arc", "biology"), ("synapse", "biology"),
    ("Euler formula polyhedron", "math"), ("Pythagorean theorem", "math"),
    ("normal distribution", "math"), ("p value statistics", "math"),
    ("monsoon", "geo"), ("el nino", "geo"),
    ("karst topography", "geo"), ("loess plateau", "geo"),
    ("lunar eclipse", "astro"), ("solar eclipse", "astro"),
    ("Milky Way galaxy", "astro"), ("Andromeda galaxy", "astro"),
    ("blood pressure", "medicine"), ("blood sugar diabetes", "medicine"),
    ("antibiotic resistance", "medicine"), ("placebo effect", "medicine"),
    ("cognitive dissonance", "psych"), ("learned helplessness", "psych"),
    ("attachment theory Bowlby", "psych"), ("flow state Csikszentmihalyi", "psych"),
    ("inflation CPI", "econ"), ("unemployment rate", "econ"),
    ("comparative advantage Ricardo", "econ"), ("invisible hand Smith", "econ"),
    ("Turing machine", "computing"), ("von Neumann architecture", "computing"),
    ("relational database Codd", "computing"), ("TCP handshake", "computing"),
    ("Zipf law", "language"), ("Chomsky universal grammar", "language"),
    ("Saussure signifier", "language"), ("Sapir Whorf", "language"),
    ("butterfly effect Lorenz", "systems"), ("tragedy of the commons", "systems"),
    ("prisoner dilemma", "systems"), ("Nash equilibrium", "systems"),
    ("cogito Descartes", "philosophy"), ("allegory of the cave", "philosophy"),
    ("categorical imperative Kant", "philosophy"), ("veil of ignorance Rawls", "philosophy"),
    ("yin yang wuxing", "xuan"), ("bagua trigrams", "xuan"),
    ("heavenly stems branches", "xuan"), ("solar terms 24", "xuan"),
    ("meridian acupuncture points", "medicine"), ("pulse diagnosis", "medicine"),
    ("herbal compendium Bencao", "medicine"),     ("yin yang balance", "medicine"),
    ("Analects Confucius ren", "confucian"), ("doctrine of the mean", "confucian"),
    ("Mencius nature good", "confucian"), ("Xunzi nature evil", "confucian"),
    ("Tao Te Ching Laozi", "daoist"), ("Zhuangzi butterfly dream", "daoist"),
    ("Four Noble Truths", "buddhist"), ("Eightfold Path", "buddhist"),
    ("Heart Sutra emptiness", "buddhist"), ("Zen koan", "buddhist"),
    ("Art of War Sunzi", "military"), ("Thirty-Six Stratagems", "military"),
    ("Qimin Yaoshu agriculture", "agriculture"), ("crop rotation", "agriculture"),
    ("Oracle bone script", "language"), ("Seal script Shuowen", "language"),
    ("papermaking Cai Lun", "tech"), ("movable type Bi Sheng", "tech"),
    ("gunpowder", "tech"), ("compass Sinan", "tech"),
    ("abacus", "tech"), ("Nine Chapters mathematics", "math"),
    ("Pythagoras", "math"), ("Euclid elements", "math"),
    ("calculus Newton Leibniz", "math"), ("probability Kolmogorov", "math"),
    ("Big Bang", "astro"), ("black hole event horizon", "astro"),
    ("plate tectonics", "geo"), ("water cycle", "geo"),
    ("photosynthesis", "biology"), ("mitosis", "biology"),
    ("natural selection Darwin", "biology"), ("DNA double helix", "biology"),
    ("periodic table Mendeleev", "chemistry"), ("chemical bond", "chemistry"),
    ("redox", "chemistry"), ("pH scale", "chemistry"),
    ("Newton laws", "physics"), ("thermodynamics entropy", "physics"),
    ("relativity Einstein", "physics"), ("quantum uncertainty", "physics"),
]

# 域→卦（归属；无命中回退按序轮转 64 卦）
DOMAIN_HEX = {"physics": "震", "chemistry": "鼎", "biology": "颐",
              "math": "乾", "geo": "坤", "astro": "离", "medicine": "鼎",
              "psych": "蒙", "econ": "姤", "computing": "巽",
              "language": "兑", "systems": "恒", "philosophy": "乾",
              "xuan": "泰", "confucian": "泰", "daoist": "坤",
              "buddhist": "蒙", "military": "师", "agriculture": "坤",
              "tech": "鼎"}


def lit_mid(hexname, table=None):
    """卦名 → 活库 M-id；该卦未点亮/无对应记忆则 ""（不编 id）。

    语义与旧实现一致（旧实现也是查不到就返回 ""，让 connections 少一条），
    差别只在 id 从活库反查而非 BASE_MID+j 推算。DOMAIN_HEX 里的「颐」「姤」
    等 2 个卦本就不在 20 点亮符内，故 "" 是合法结果，不是失败。
    但**整表反查不到**是失败（活库被清空/换库）→ SystemExit，绝不静默
    退化成"全部无连接"，那是另一种形式的静默腐化。
    """
    if hexname not in {hx for _, (hx, _) in LIT.items()}:
        return ""
    return (table if table is not None else lit_table()).get(hexname, "")


TRI8 = ("乾", "坤", "震", "巽", "坎", "离", "艮", "兑")


def probe(con, query, limit=3):
    try:
        return con.execute(
            "select rowid,title,substr(summary,1,200),domain "
            "from nodes_fts where nodes_fts match ? order by rank limit ?",
            (query, limit)).fetchall()
    except Exception:  # noqa: BLE001 — 单概念失败跳过，不中断整批
        return []


def distill(rowid, title, summary, domain, hexa):
    title = (title or "").strip().replace("ZimEntry: ", "")[:60]
    summary = (summary or "").strip()[:300]
    if not title:
        return None
    return {"url": "mined://rowid%d" % rowid,
            "title": "【待润色·%s】%s" % (domain, title),
            "content": "【待润色·%s】%s｜库源rowid=%d（英文原样，中文待润色转正）"
                       % (domain, summary, rowid),
            "domain": "mined-review", "memory_type": "Pattern",
            "confidence": 0.55,
            "connections": [m for m in [lit_mid(hexa)] if m]}


def load_state():
    try:
        with open(STATE_JSON, encoding="utf-8") as fh:
            return json.load(fh)
    except (OSError, ValueError):
        return {"offset": 0}


def save_state(offset):
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(STATE_JSON, "w", encoding="utf-8") as fh:
        json.dump({"offset": offset}, fh)


def mine(db_path, limit, offset):
    con = sqlite3.connect("file:%s?mode=ro" % db_path, uri=True, timeout=30)
    batch = CONCEPTS[offset:offset + limit]
    out, skipped = [], 0
    hexa_keys = list(DOMAIN_HEX.values())
    for i, (q, domain) in enumerate(batch):
        rows = probe(con, q)
        if not rows:
            skipped += 1
            continue
        rid, title, summary, _d = rows[0]
        hexa = DOMAIN_HEX.get(domain, TRI8[i % 8])
        r = distill(rid, title, summary, domain, hexa)
        if r:
            out.append(r)
        else:
            skipped += 1
    con.close()
    return out, skipped


def selftest():
    con = sqlite3.connect(":memory:")
    con.execute("CREATE VIRTUAL TABLE nodes_fts USING fts5(title,summary,domain)")
    con.executemany("INSERT INTO nodes_fts VALUES (?,?,?)", [
        ("Bernoulli principle", "velocity up pressure down", "physics"),
        ("Tao Te Ching", "dao that can be told", "daoist"),
        ("No such thing here xyz", "", ""),
    ])
    rows = probe(con, "Bernoulli principle")
    assert rows and rows[0][1].startswith("Bernoulli"), rows
    assert probe(con, "qqqzzz-no-hit") == []
    r = distill(1, "ZimEntry: Bernoulli principle", "velocity up", "physics", "震")
    assert r["domain"] == "mined-review" and r["confidence"] == 0.55
    assert r["connections"] and "待润色" in r["title"]
    assert distill(2, "", "x", "physics", "震") is None
    assert lit_mid("乾").startswith("M-") and lit_mid("不存在") == ""
    # ── 悬空边防护（2026-09-28）──────────────────────────────────────────
    # 旧实现 lit_mid() = "M-%06d" % (479512 + j)：BASE_MID 是**裸数字**，
    # 仓库静态扫描只匹配 M-\d{6}，看不见它，而 M-479512 在活库从不存在 →
    # 每条矿出记忆的 connections 都悬空。改为活库反查后钉死三条：
    #  ① 脚本准备写出去的每个 id 都必须在活库里；
    #  ② 未点亮卦返回 ""（不编 id），且注入表时行为一致；
    #  ③ 库里一个卦符记忆都没有 → 必须 SystemExit，不得静默退化为空连接。
    tbl = lit_table()
    known = live_mid_set()
    live_hex = {hx for _, (hx, _) in LIT.items()}
    assert set(tbl) == live_hex, "反查表与 LIT 点亮卦不符: %s" % (
        set(tbl) ^ live_hex)
    for hx, mid in tbl.items():
        assert mid in known, "卦符 %s 反查到 %s，活库无此 id" % (hx, mid)
    for dom, hx in DOMAIN_HEX.items():
        got = lit_mid(hx)
        assert got == "" or got in known, (dom, hx, got)
    assert lit_mid("乾", {"乾": "M-000001"}) == "M-000001"
    assert lit_mid("乾", {}) == ""            # 反查不到也不编 id
    assert lit_mid("姤") == ""                # 姤未点亮：合法空
    try:
        live_symbol_mids("/nonexistent/cocoons.json")
    except SystemExit:
        pass
    else:
        raise AssertionError("活库缺失未中止（会退回硬编码基址）")
    print("selftest ok: probe/distill/hex/gate, 卦符 M-id 活库反查")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=50)
    ap.add_argument("--offset", type=int, default=-1)
    ap.add_argument("--db", default=VOL_DB)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    if not os.path.exists(args.db):
        print("等卷：%s 不在，管线休眠（offset 保持）。卷回即开工："
              "--limit N --offset M。" % args.db)
        return
    offset = args.offset if args.offset >= 0 else load_state()["offset"]
    out, skipped = mine(args.db, args.limit, offset)
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(OUT_JSONL, "a", encoding="utf-8") as fh:
        for r in out:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")
    save_state(offset + args.limit)
    print("mined=%d skipped=%d offset=%d→%d out=%s" % (
        len(out), skipped, offset, offset + args.limit, OUT_JSONL))


if __name__ == "__main__":
    main()
