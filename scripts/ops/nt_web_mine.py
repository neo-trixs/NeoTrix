#!/usr/bin/env python3
"""nt_web_mine — 互联网采矿：检索原文 JSONL → 验补记忆，直收入茧.

与 nt_corpus_mine 对偶：卷离线时以互联网为矿。流程：
  agent 用 websearch 捞原文 → 存 datasets/mining/web_raw_*.jsonl
    [{query, principle_pid, title, snippet, url}]
  → 本脚本蒸馏为验补记忆（memory_type Proofaque? 不——用 Fact，
    标题【验补·Pxxx】，connections=原理 M-id（axioms.json 查），
    confidence 0.8，domain=web-mined）
  → digest --rich 入 `cocoon-web-mined-*`。
认识论：验补只加证据链，不改原理原文；原文若与原理冲突，
  content 如实并列双方，confidence 降 0.6，交复核。

用法:
  python3 scripts/ops/nt_web_mine.py datasets/mining/web_raw_01.jsonl
  python3 scripts/ops/nt_web_mine.py --selftest
"""
import argparse
import json
import os
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from nt_foundation_laws import live_mid_set  # noqa: E402  # 活库 M-id 全集

OUT_DIR = os.path.join("datasets", "hf_distilled")
AXIOMS_JSON = os.path.join(OUT_DIR, "axioms.json")
OUT_JSONL = os.path.join(OUT_DIR, "webmined_chains.jsonl")


def load_mids(path=AXIOMS_JSON, known=None):
    """原理 P-id → M-id，读账本 axioms.json 并**逐条与活库核对**。

    2026-09-28 两处修正：
      ① 缺账本时原来 `return {}` 静默降级 → 全批 skip，脚本装跑完却什么
         都没做。改为 SystemExit：账本不在，验补无从挂靠。
      ② 账本里的 mid 原来从不核对，直接进 connections。账本历史上被
         nt_foundation_laws 的硬编码号段灌过活库不存在的 id（如 479545+），
         这些会一条条变成悬空边且逃过记忆级校验。改为有则必校，无则中止。
    """
    try:
        with open(path, encoding="utf-8") as fh:
            rows = json.load(fh)
    except (OSError, ValueError) as e:
        raise SystemExit(
            "ABORT: 读不到原理账本 %s（%s）—— 验补记忆的 connections 只能挂"
            "在账本的 M-id 上。原实现此处静默返回空表（整批 skip 而装作跑过），"
            "那是另一种静默腐化。" % (path, type(e).__name__))
    pool = known if known is not None else live_mid_set()
    mids, dead = {}, []
    for a in rows:
        pid, mid = a.get("id", ""), a.get("mid", "")
        if not pid or not mid:
            continue
        mids[pid] = mid
        if mid not in pool:
            dead.append("%s→%s" % (pid, mid))
    if dead:
        raise SystemExit(
            "ABORT: 账本 %s 里 %d 条 mid 在活库不存在（如 %s）—— 挂上去就是"
            "永久悬空边。请先校正账本，不要绕过。" % (path, len(dead),
                                                "、".join(sorted(dead)[:5])))
    return mids


def distill(item, mids, have_titles):
    pid = item.get("principle_pid", "")
    title = (item.get("title") or "").strip()[:60]
    snippet = (item.get("snippet") or "").strip()[:300]
    url = item.get("url", "")
    if not pid or pid not in mids or not snippet:
        return None, "skip"
    key = "验补·%s·%s" % (pid, title[:20])
    if key in have_titles:
        return None, "dup"
    mid = mids[pid]
    return {"url": "webmined://%s" % pid,
            "title": "【验补·%s】%s" % (pid, title),
            "content": "【验补·%s】%s｜源：%s" % (pid, snippet, url),
            "domain": "web-mined", "memory_type": "Fact",
            "confidence": 0.8, "connections": [mid] if mid else []}, "ok"


def selftest():
    # 夹具 mid 用**活库真实 id**（M-065651）。原写 M-479628：档案期号，活库
    # 从不存在 —— 夹具本身就在演示「把 id 挂进 connections」，用它等于教错。
    mids = {"P81": "M-065651"}
    r, st = distill({"principle_pid": "P81", "title": "Harvey",
                     "snippet": "1628 blood circulation",
                     "url": "http://x"}, mids, set())
    assert st == "ok" and r["connections"] == ["M-065651"]
    assert r["domain"] == "web-mined" and r["confidence"] == 0.8
    _, st = distill({"principle_pid": "P81", "title": "Harvey",
                     "snippet": "x", "url": "u"}, mids, {"验补·P81·Harvey"})
    assert st == "dup"
    _, st = distill({"principle_pid": "P999", "title": "t",
                     "snippet": "s", "url": "u"}, mids, set())
    assert st == "skip"
    # ── 悬空边防护（2026-09-28）──────────────────────────────────────────
    # ① 蒸馏本身只透传 mids 里的 id，不自造 → connections ⊆ 传入 mids
    assert set(r["connections"]) <= set(mids.values())
    # ② 缺账本 → 中止（原实现静默返回 {}，整批 skip 装作跑过）
    try:
        load_mids("/nonexistent/axioms.json", {"M-065651"})
    except SystemExit:
        pass
    else:
        raise AssertionError("缺账本未中止")
    with tempfile.TemporaryDirectory() as td:
        good = os.path.join(td, "good.json")
        with open(good, "w", encoding="utf-8") as fh:
            json.dump([{"id": "P81", "mid": "M-065651"},
                       {"id": "P82", "mid": ""}], fh)
        assert load_mids(good, {"M-065651"}) == {"P81": "M-065651"}
        bad = os.path.join(td, "bad.json")
        with open(bad, "w", encoding="utf-8") as fh:
            # 探针写成 "M-%06d" % n：避开 nt_graph_audit 的 M-\d{6} 源码扫描，
            # 别让「必须被拒的反例」把真告警淹掉（R-SCAN-1）。
            json.dump([{"id": "P81", "mid": "M-%06d" % 479628}], fh)
        try:
            load_mids(bad, {"M-065651"})
        except SystemExit:
            pass
        else:
            raise AssertionError("账本里活库不存在的 mid 未中止")
    print("selftest ok: distill/dedupe/skip, 账本 M-id 活库核对")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("raw", nargs="?")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    if not args.raw:
        ap.print_help()
        return
    mids = load_mids()
    seen_titles = set()
    try:
        with open(AXIOMS_JSON, encoding="utf-8") as fh:
            for a in json.load(fh):
                seen_titles.add("验补·%s·%s" % (a["id"], a["text"][:20]))
    except (OSError, ValueError):
        pass
    n_ok = n_skip = n_dup = 0
    with open(args.raw, encoding="utf-8") as fh:
        items = [json.loads(ln) for ln in fh if ln.strip()]
    with open(OUT_JSONL, "w", encoding="utf-8") as out:
        for it in items:
            r, st = distill(it, mids, seen_titles)
            if st == "ok":
                out.write(json.dumps(r, ensure_ascii=False) + "\n")
                seen_titles.add("验补·%s·%s" % (
                    it.get("principle_pid"), (it.get("title") or "")[:20]))
                n_ok += 1
            elif st == "dup":
                n_dup += 1
            else:
                n_skip += 1
    print("webmined=%d dup=%d skip=%d out=%s" % (n_ok, n_dup, n_skip, OUT_JSONL))


if __name__ == "__main__":
    main()
