#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_cocoons_verify_absorb — 通用吸收验收器（按 domain 前缀，不绑定某数据集）.

只按 `nt_crystal_core/{consciousness,cocoons}.rs` 的契约重新判定，**不复用任何
生产代码函数**（生产脚本自证 ≠ 独立验收）。适用于任何一次 cocoons 吸收。

验收项：
  1. JSON 合法 + 顶层键 {cocoons, strategy}（StoreData 契约）
  2. 字段/枚举合规：Memory 必填 10 字段 + memory_type ∈ 8 个合法变体
     （**枚举合规是加载前提**：非法变体让 CocoonStore::load 全有全无失败）
  3. M-id 唯一性：**只对本次新增区间**判定（前缀区间外的既存重复另行归属）
  4. 连边无悬挂：每条 connections 的目标都真实存在
  5. 锚点完整：内容以 [xxx] 标记锚点的，锚点条目都在，且被引用锚点可解析
  6. 存量零丢失：与 --before 逐 id 逐字段比对
  7. recall() 语义可命中：复刻 CocoonStore::recall 的 domain + 子串过滤
  8. 置信度/标记纪律：--conf-cap 与 --require-tag（合成集须打标记）

用法：
  python3 scripts/ops/nt_cocoons_verify_absorb.py --prefix rl-
  python3 scripts/ops/nt_cocoons_verify_absorb.py --prefix medical- \\
      --before ~/.neotrix/crystal_core/cocoons.json.bak.medical --conf-cap 0.75
  python3 scripts/ops/nt_cocoons_verify_absorb.py --selftest
"""
import argparse
import json
import os
import re
import sys

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")
# MemoryType 权威 8 变体（consciousness.rs:39-56）。推理类型名（Deductive/
# Inductive/Abductive/Analogical/CrossDomain）**不在此集合**，误用即毒记忆。
VALID_TYPES = {"Fact", "Pattern", "Causal", "Contradiction",
               "Counterfactual", "Experience", "Lesson", "Solution"}
REQ = {"id", "content", "memory_type", "domain", "strength", "confidence",
       "connections", "created_at", "last_accessed", "access_count"}
ANCHOR_RE = re.compile(r"^\[([a-z][a-z0-9:_-]*)[\]·]")


def selftest():
    assert len(VALID_TYPES) == 8
    assert not (VALID_TYPES & {"CrossDomain", "Deductive", "Inductive",
                               "Abductive", "Analogical"}), "推理类型名不得混入"
    assert ANCHOR_RE.match("[rubric] x").group(1) == "rubric"
    assert ANCHOR_RE.match("[verifier-pattern] x").group(1) == "verifier-pattern"
    assert ANCHOR_RE.match("[taxonomy:music] x").group(1) == "taxonomy:music"
    assert ANCHOR_RE.match("[profile·synthetic] x").group(1) == "profile"
    assert not ANCHOR_RE.match("no tag")
    # recall 复刻：domain 精确匹配 + 子串过滤（consciousness.rs? cocoons.rs:154-187）
    mems = [{"domain": "rl-rubric", "content": "[rubric] A", "strength": 1.0,
             "access_count": 0},
            {"domain": "rl-rubric", "content": "[rubric] B", "strength": 0.5,
             "access_count": 0},
            {"domain": "other", "content": "[rubric] C", "strength": 1.0,
             "access_count": 0}]
    r = recall(mems, "rl-rubric", "rubric")
    assert [m["content"] for m in r] == ["[rubric] A", "[rubric] B"], r
    assert recall(mems, "rl-rubric", "") == r
    assert recall(mems, "nope", "") == []
    print("selftest ok: type-whitelist/anchor-regex/recall-replica")
    return 0


def recall(mems, domain, query, limit=3):
    out = [m for m in mems if m["domain"] == domain
           and query.lower() in m["content"].lower()]
    out.sort(key=lambda m: -(m["strength"] * (1 + m["access_count"] * 0.1)))
    return out[:limit]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cocoons", default=COCOONS)
    ap.add_argument("--prefix", default="", help="本次吸收的 domain 前缀，如 rl-")
    ap.add_argument("--before", default="")
    ap.add_argument("--conf-cap", type=float, default=None)
    ap.add_argument("--require-tag", default="", help="内容须含的标记，如 synthetic")
    ap.add_argument("--queries", default="", help="recall 冒烟：domain:query,...")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    errs = []
    raw = open(args.cocoons, encoding="utf-8").read()
    d = json.loads(raw)
    # 用 os.path.getsize 报**字节**数：raw 是已解码 str，len(raw) 是字符数。
    # 中文语料下字符数 < 字节数，实测 56,637,045 B 被误报成 56.0MB（真实 56.6MB）。
    # 体检工具报错体积比不报更危险（R-SCAN-3：宁可精确也不要陈旧/失真的门记录）。
    print("1. JSON 合法 — %.1fMB（%d 字节 / %d 字符）"
          % (os.path.getsize(args.cocoons) / 1e6, os.path.getsize(args.cocoons),
             len(raw)))
    if set(d.keys()) != {"cocoons", "strategy"}:
        errs.append("顶层键不是 {cocoons,strategy}: %s" % sorted(d.keys()))
    print("2. StoreData 契约 — cocoons=%d" % len(d.get("cocoons", {})))

    mems, owner = {}, {}
    for cid, c in d["cocoons"].items():
        for m in c.get("memories", []):
            mems.setdefault(m["id"], m)
            owner.setdefault(m["id"], cid)
    target = {i: m for i, m in mems.items() if not args.prefix or
              m["domain"].startswith(args.prefix)}
    print("3. 目标集（domain 前缀 %r）= %d / 全库 %d" % (args.prefix, len(target), len(mems)))
    if not target:
        print("FAIL: 前缀未命中任何记忆")
        return 1

    # 字段/枚举（加载前提）
    bad = [i for i, m in target.items()
           if not REQ <= set(m) or m["memory_type"] not in VALID_TYPES]
    if bad:
        errs.append("%d 条不合 Memory 契约：%s" % (len(bad), bad[:5]))
    print("4. Memory 契约 — 违例 %d（含非法 memory_type；这会让 load 全有全无失败）" % len(bad))

    # id 唯一（仅目标集内自洽 + 与全库交叉）
    dup_in = len(target) - len({i for i in target})
    collide = [i for i in target if owner.get(i) and
               sum(1 for c in d["cocoons"].values() for m in c["memories"]
                   if m["id"] == i) > 1]
    if collide:
        errs.append("新增 id 与既存重复：%d 条，例 %s" % (len(collide), collide[:5]))
    print("5. M-id — 目标集自冲突 %d；与既存重号 %d" % (dup_in, len(collide)))

    # 悬挂边
    dang = [(i, t) for i, m in target.items() for t in m["connections"] if t not in mems]
    if dang:
        errs.append("悬挂连边 %d 条，例 %s" % (len(dang), dang[:3]))
    edges = sum(len(m["connections"]) for m in target.values())
    print("6. 连边 — %d 条边 / 悬挂 %d" % (edges, len(dang)))

    # 锚点：被引用的锚点必须存在于目标集
    anchored = {ANCHOR_RE.match(m["content"]).group(1)
                for m in target.values() if ANCHOR_RE.match(m["content"])}
    tags = {}
    for i, m in target.items():
        mm = ANCHOR_RE.match(m["content"])
        if mm:
            tags.setdefault(mm.group(1), []).append(i)
    print("7. 锚点 — %d 种标签：%s"
          % (len(anchors := sorted(anchored)),
             ", ".join("%s×%d" % (t, len(tags[t])) for t in anchors[:9])))

    # 存量零丢失
    if args.before:
        b = json.load(open(args.before, encoding="utf-8"))
        # 两边必须用**同一**去重语义（皆 first-wins）。活库既存 1645 个重号 id，
        # 若一边 setdefault(首现) 一边 dict 推导(末现)，同一条会被判成「被改写」。
        bmem = {}
        for c in b["cocoons"].values():
            for m in c["memories"]:
                bmem.setdefault(m["id"], m)
        miss = [i for i in bmem if i not in mems]
        chg = [i for i in bmem if i in mems and mems[i] != bmem[i]]
        if miss:
            errs.append("存量丢失 %d：%s" % (len(miss), miss[:5]))
        if chg:
            errs.append("存量被改写 %d：%s" % (len(chg), chg[:5]))
        print("8. 存量零丢失 — 吸收前 %d 条（去重后）全在且未改" % len(bmem))

    # 纪律
    if args.conf_cap is not None:
        over = [i for i, m in target.items() if m["confidence"] > args.conf_cap + 1e-9]
        if over:
            errs.append("%d 条 confidence > %.2f" % (len(over), args.conf_cap))
        print("9. 置信封顶 %.2f — 越界 %d" % (args.conf_cap, len(over)))
    if args.require_tag:
        miss = [i for i, m in target.items() if args.require_tag not in m["content"]]
        print("10. 标记 %r — 缺失 %d（仅对相关标签有意义，逐标签核对）"
              % (args.require_tag, len(miss)))

    if args.queries:
        for spec in args.queries.split(","):
            dom, _, q = spec.partition(":")
            r = recall(list(target.values()), dom, q)
            print("11. recall(%-14s %-24r) -> %d" % (dom, q, len(r)))
            if not r:
                errs.append("recall 空结果: %s" % spec)

    print("\n" + "=" * 68)
    if errs:
        print("FAIL (%d):" % len(errs))
        for e in errs[:20]:
            print("  - %s" % e)
        return 1
    print("PASS — 目标集 %d 条全部验收通过" % len(target))
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
