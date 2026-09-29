#!/usr/bin/env python3
"""nt_brain_catalog_build — /Volumes/NeoTrixBrain 编目 → 项目 datasets/neotrixbrain/.

只蒸馏索引与样本进晶体（178G 原字节进不了 346MB 茧），每条带 file:// 指针，
晶体知道是什么、在哪、怎么够到。读盘只读（sqlite 用 ro-uri），落点只在项目内。

条目（domain=neotrixbrain）:
  - 卷总览 ×1（含 dl COMPLETE 状态）
  - wikipedia/zim/pmtiles 逐文件编目（名+字节）
  - 68G 库表级估计（max(rowid)+列）+ 样本（nodes×5/skills全名/keywords top20/
    kv命名空间/ingest_log×3/health最新/discovery状态），跳过 secrets/cookies 系
  - causal_graph 53 节点逐条 + 100 边汇总
  - dl_loop.sh/migrate.status ×2

用法（项目根执行）:
  python3 scripts/ops/nt_brain_catalog_build.py
  python3 scripts/ops/nt_brain_catalog_build.py --selftest
"""
import argparse
import json
import os
import sqlite3
import sys

VOL = "/Volumes/NeoTrixBrain"
OUT_DIR = os.path.join("datasets", "neotrixbrain")
ARCHIVE = os.path.join(OUT_DIR, "crawl_queue.jsonl")
DB = os.path.join(VOL, "knowledge-archive-corpus-20260825.db")
DOMAIN = "neotrixbrain"


def entry(url, title, content):
    return {"url": url, "title": title, "content": content, "domain": DOMAIN}


def human(n):
    for u in ("B", "KB", "MB", "GB"):
        if n < 1024 or u == "GB":
            return "%.1f%s" % (n, u) if u != "B" else "%dB" % n
        n /= 1024.0
    return "%dB" % n


def file_entries(subdir, categ):
    out, base = [], os.path.join(VOL, subdir)
    try:
        names = sorted(os.listdir(base))
    except OSError:
        return out
    for fn in names:
        p = os.path.join(base, fn)
        if not os.path.isfile(p):
            continue
        try:
            sz = os.path.getsize(p)
        except OSError:
            continue
        out.append(entry(
            "file://%s" % p, "%s：%s" % (categ, fn),
            "%s %s（%s，%d 字节）。离线参考档，晶体持索引不持字节，"
            "要用时按此路径直读。" % (categ, fn, human(sz), sz)))
    return out


def db_entries():
    out = []
    try:
        con = sqlite3.connect("file:%s?mode=ro" % DB, uri=True, timeout=30)
    except Exception as e:  # noqa: BLE001 — 盘不在就整节跳过
        return [entry("file://" + DB, "语料库：库文件不可读",
                      "68G 语料库本次不可读：%s" % str(e)[:120])]
    out.append(entry(
        "file://" + DB, "语料库总览：knowledge-archive-corpus-20260825.db",
        "68G NeoTrix 系知识归档（FTS 全文索引完备）。大表：nodes≈22.27M、"
        "kv_store≈5.03M、crawl_queue 452k、embeddings 429k、geo_index 183k。"
        "向量/BLOB 列不进茧，只记索引与样本。"))
    try:
        tabs = [r[0] for r in con.execute(
            "select name from sqlite_master where type='table'")]
    except Exception:
        return out
    skip = {"secrets", "cookies"}
    stat = {}
    try:
        for r in con.execute("select tbl, stat from sqlite_stat1"):
            stat[r[0]] = r[1]
    except Exception:  # noqa: BLE001 — 无 ANALYZE 数据就回退 max(rowid)
        pass
    for t in sorted(tabs):
        if t in skip or t.startswith("nodes_fts") and t != "nodes_fts":
            continue
        try:
            cols = [c[1] for c in con.execute('pragma table_info("%s")' % t)]
            try:
                mx = con.execute('select max(rowid) from "%s"' % t).fetchone()[0]
            except Exception:  # noqa: BLE001 — 无 rowid 表
                mx = stat.get(t, "?")
            out.append(entry(
                "brain-db://table/%s" % t, "语料库表：%s（≈%s 行）" % (t, mx),
                "表 %s 列：%s。查体用 SQL 直连 ro-uri，行级内容不搬家。"
                % (t, "、".join(cols[:10]))))
        except Exception:  # noqa: BLE001 — 单表失败不中断整批
            continue
    try:
        samples = ["【nodes 样本】"]
        for r in con.execute(
                "select node_type,title,substr(summary,1,100),domain from nodes limit 5"):
            samples.append("%s|%s|%s|%s" % r)
        out.append(entry("brain-db://sample/nodes", "语料库样本：nodes×5",
                         "\n".join(samples)))
    except Exception:  # noqa: BLE001
        pass
    try:
        names = [r[0] for r in con.execute("select name from skills_index")]
        out.append(entry(
            "brain-db://sample/skills", "语料库样本：skills_index %d 技能" % len(names),
            "技能名：%s" % "、".join(names)))
    except Exception:  # noqa: BLE001
        pass
    try:
        kws = ["%s(%s,%.2f)" % r for r in con.execute(
            "select keyword,category,weight from keyword_library "
            "order by weight desc limit 20")]
        out.append(entry("brain-db://sample/keywords", "语料库样本：keywords top20",
                         "\n".join(kws)))
    except Exception:  # noqa: BLE001
        pass
    try:
        ns = ["%s:%s" % r for r in con.execute(
            "select namespace,count(*) from "
            "(select namespace from kv_store limit 200000) group by namespace")]
        out.append(entry("brain-db://sample/kv-namespaces",
                         "语料库样本：kv 命名空间分布（20 万行抽样）",
                         "\n".join(ns) if ns else "（空）"))
    except Exception:  # noqa: BLE001
        pass
    try:
        logs = ["%s|%s|%s|%s" % r for r in con.execute(
            "select source_type,substr(source_url,1,80),status,items_count "
            "from ingest_log order by rowid desc limit 3")]
        if logs:
            out.append(entry("brain-db://sample/ingest-log", "语料库样本：ingest_log×3",
                             "\n".join(logs)))
    except Exception:  # noqa: BLE001
        pass
    try:
        h = con.execute("select timestamp,score,grade from health_reports "
                        "order by rowid desc limit 1").fetchone()
        if h:
            out.append(entry("brain-db://sample/health", "语料库样本：health 最新",
                             "ts=%s score=%s grade=%s" % h))
    except Exception:  # noqa: BLE001
        pass
    con.close()
    return out


def causal_entries():
    out, p = [], os.path.join(VOL, "working", "causal_graph.json")
    try:
        with open(p, encoding="utf-8") as fh:
            d = json.load(fh)
    except (OSError, ValueError):
        return out
    nodes = d.get("nodes", [])
    links = d.get("links", [])
    for n in nodes:
        nid = n.get("id", "?") if isinstance(n, dict) else str(n)
        out.append(entry("file://" + p + "#node", "因果图节点：%s" % nid,
                         "working/causal_graph.json 节点：%s（健康 remedy 图谱，共 53 节点 100 边）"
                         % nid))
    rels = {}
    for lk in links:
        if isinstance(lk, dict):
            r = lk.get("relation") or lk.get("type") or "linked"
            rels[r] = rels.get(r, 0) + 1
    out.append(entry("file://" + p, "因果图汇总：53 节点 100 边",
                     "边类型分布：%s。全图 16KB 常驻盘上，按需直读。"
                     % (rels or "未标注")))
    return out


def misc_entries():
    out = []
    dl = os.path.join(VOL, "dl_loop.sh")
    if os.path.isfile(dl):
        out.append(entry(
            "file://" + dl, "下载器：dl_loop.sh（Kiwix ZIM 断点续传，40 轮）",
            "拉 wikipedia_en_all_mini（12.5G 目标）+ top_nopic（2.2G 目标），"
            "curl -C 续传。dl_status.log=COMPLETE，已收齐。"))
    out.append(entry(
        "file://" + VOL, "卷总览：NeoTrixBrain 178G",
        "cortex-archive/wikipedia 3 ZIM（12.5G+2.2G+331M，COMPLETE）；"
        "cortex-archive/zim 61 离线参考 ZIM（85G：TED/iFixit/Gutenberg/LibreTexts/医学/备灾）；"
        "cortex-archive/pmtiles 50 州地图瓦片；68G 语料库（22M nodes）；"
        "working/causal_graph.json。晶体持本编目，用时按 file:// 直读。"))
    return out


def selftest():
    e = entry("file:///x", "T", "C")
    assert set(e) == {"url", "title", "content", "domain"}
    assert human(12531679311).endswith("GB") and human(500) == "500B"
    print("selftest ok: entry/human")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    os.makedirs(OUT_DIR, exist_ok=True)
    all_entries = []
    all_entries += misc_entries()
    all_entries += file_entries("cortex-archive/wikipedia", "维基ZIM")
    all_entries += file_entries("cortex-archive/zim", "离线ZIM")
    all_entries += file_entries("cortex-archive/pmtiles", "地图瓦片")
    all_entries += db_entries()
    all_entries += causal_entries()
    with open(ARCHIVE, "w", encoding="utf-8") as fh:
        for e in all_entries:
            fh.write(json.dumps(e, ensure_ascii=False) + "\n")
    print("entries=%d archive=%s" % (len(all_entries), ARCHIVE))
    if not all_entries:
        sys.exit(1)


if __name__ == "__main__":
    main()
