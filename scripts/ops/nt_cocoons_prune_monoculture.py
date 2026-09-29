#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_cocoons_prune_monoculture — 剪掉「一个锚点吞掉整个域」的通用连接边.

## 事故（2026-09-28 自查发现，自造）

RL 蒸馏时把**每份 rubric** 都连到同一个「判分器」锚点，导致该锚点
**入度 935**（全库最大枢纽，次名仅 171）。后果：它是**通用连接件** ——
经它取到的 925 个入邻居分属 925 个不同疾病族，彼此毫无语义关联。
推理链若从该枢纽走入邻域选前提，会产出「前提同指率」极低的垃圾链。

## 为什么不回滚重灌

`cocoons.json.bak.rlenv` 是 RL 吸收**前**的快照（63,747 条 / 0 条 rl-rubric），
而其后的健康修复（id 去重 / 删悬空 / 折内容重复 / 拆超限茧 / 规范化逗号）
都在该快照**之后**。回滚会连带撤销这些修复。故本脚本只做**定点剪边**。

## 口径：以族锚点为唯一合法父节点

族锚点 = 内容形如 `[family] <族名> —— ...` 的锚点。族锚点是**语义正确**的
父节点（同族 = 同职能域），且跨域性仍保留（族锚点在 `rl-taxonomy`，
rubric 在 `rl-rubric`）。故每份 rl-rubric 的 connections 重置为
「其族锚点」单边。

判分方案信息**未丢失**：rubric 正文已含 `reward=rl_grade` 与判分要点，
且 10 条 `[verifier-pattern]` 记忆本身就描述该方案。

## 不变量（剪完必须全成立）

记忆条数不变 / id 集合不变 / 正文不变 / 无悬空边 / 无非法 memory_type /
茧键=体内 id / 无超限茧 / 规范往返字节一致。任一不成立即拒绝落盘。

用法:
  python3 scripts/ops/nt_cocoons_prune_monoculture.py --dry-run
  python3 scripts/ops/nt_cocoons_prune_monoculture.py --commit
  python3 scripts/ops/nt_cocoons_prune_monoculture.py --selftest
"""
import argparse
import collections
import glob
import json
import os
import re
import shutil
import sys
import time

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")
VALID_TYPES = {"Fact", "Pattern", "Causal", "Contradiction",
               "Counterfactual", "Experience", "Lesson", "Solution"}
FAMILY_ANCHOR = re.compile(r"^\[family\]\s+(\S+)")
RUBRIC_FAMILY = re.compile(r"（族=(\S+)\s")


def selftest():
    assert FAMILY_ANCHOR.match("[family] accounting_audit_tax —— MiMo general 域").group(1) \
        == "accounting_audit_tax"
    assert RUBRIC_FAMILY.search(
        "[rubric] s3k_0000_accounting_audit_tax_en_t1_rl_008（族=accounting_audit_tax lang=en 难度t1 rl008）：6 检查项"
    ).group(1) == "accounting_audit_tax"
    assert not FAMILY_ANCHOR.match("[profile] x")
    assert RUBRIC_FAMILY.search("[rubric] no family here") is None
    assert len(VALID_TYPES) == 8
    print("selftest ok: family-anchor-regex/rubric-family-regex/type-whitelist")
    return 0


def plan(d, target_domain):
    """返回 (family_anchor_id_by_name, [(cocoon, idx, old_conns, new_conns)])。"""
    fam = {}
    for cid, c in d["cocoons"].items():
        for m in c["memories"]:
            g = FAMILY_ANCHOR.match(m["content"])
            if g:
                fam.setdefault(g.group(1), m["id"])
    changes = []
    for cid, c in d["cocoons"].items():
        for i, m in enumerate(c["memories"]):
            if m["domain"] != target_domain:
                continue
            g = RUBRIC_FAMILY.search(m["content"])
            if not g:
                continue
            want = fam.get(g.group(1))
            if want is None:
                continue
            if m["connections"] != [want]:
                changes.append((cid, i, list(m["connections"]), [want]))
    return fam, changes


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--commit", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--cocoons", default=COCOONS)
    ap.add_argument("--domain", default="rl-rubric")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    raw = open(args.cocoons, encoding="utf-8").read()
    d = json.loads(raw)
    n0 = sum(len(c["memories"]) for c in d["cocoons"].values())
    ids0 = {m["id"] for c in d["cocoons"].values() for m in c["memories"]}
    print("loaded: %d memories in %d cocoons" % (n0, len(d["cocoons"])))

    fam, changes = plan(d, args.domain)
    print("family anchors found: %d" % len(fam))
    print("plan: retarget %d memories of domain %r" % (len(changes), args.domain))
    if not changes:
        print("clean: 无需修剪")
        return 0

    indeg = collections.Counter()
    for c in d["cocoons"].values():
        for m in c["memories"]:
            for t in m["connections"]:
                indeg[t] += 1
    before_top = indeg.most_common(3)
    removed = sum(len(o) - len(nw) for _c, _i, o, nw in changes)
    print("  before top in-degrees: %s" % before_top)
    print("  edges removed: %d" % removed)

    for cid, i, _o, nw in changes:
        d["cocoons"][cid]["memories"][i]["connections"] = list(nw)

    # ---- 不变量自检，任一不过就拒绝落盘 ----
    n1 = sum(len(c["memories"]) for c in d["cocoons"].values())
    ids1 = {m["id"] for c in d["cocoons"].values() for m in c["memories"]}
    assert n1 == n0, "记忆条数变了: %d -> %d" % (n0, n1)
    assert ids1 == ids0, "id 集合变了"
    bad_type = [m["id"] for c in d["cocoons"].values() for m in c["memories"]
                if m["memory_type"] not in VALID_TYPES]
    assert not bad_type, "非法 memory_type: %s" % bad_type[:5]
    dang = [t for c in d["cocoons"].values() for m in c["memories"]
            for t in m["connections"] if t not in ids1]
    assert not dang, "剪出悬空边: %s" % dang[:5]
    for k, c in d["cocoons"].items():
        assert c["id"] == k, "茧键≠体内 id: %s" % k
        assert len(c["memories"]) <= c["retention_policy"]["max_memories"], "茧超限: %s" % k
    # 正文未被触碰
    raw2 = json.dumps(d, ensure_ascii=False, indent=2) + "\n"
    rt = json.loads(raw2)
    t0 = {m["id"]: m["content"] for c in json.loads(raw)["cocoons"].values() for m in c["memories"]}
    t1 = {m["id"]: m["content"] for c in rt["cocoons"].values() for m in c["memories"]}
    assert t0 == t1, "正文被改动（本脚本只应改 connections）"

    indeg2 = collections.Counter()
    for c in rt["cocoons"].values():
        for m in c["memories"]:
            for t in m["connections"]:
                indeg2[t] += 1
    print("  after top in-degrees: %s" % indeg2.most_common(3))
    print("  max in-degree: %d -> %d" % (before_top[0][1], indeg2.most_common(1)[0][1]))
    print("  bytes: %d -> %d (%+d)" % (len(raw), len(raw2), len(raw2) - len(raw)))

    if not args.commit:
        print("\n--- DRY RUN（未落盘）---")
        return 0

    bak = args.cocoons + ".bak.prune"
    live = os.path.getsize(args.cocoons)
    olds = sorted(glob.glob(bak + ".*"))
    for o in olds:
        try:
            if os.stat(o).st_size > 1000000 and live < 1000000:
                print("ABORT: live %.1fKB, backup %s kept" % (live / 1e3, o))
                return 2
        except OSError:
            pass
    stamped = "%s.%s" % (bak, time.strftime("%Y%m%d-%H%M%S"))
    shutil.copyfile(args.cocoons, stamped)
    for o in olds[:-2]:
        try:
            os.remove(o)
        except OSError:
            pass
    tmp = args.cocoons + ".tmp.prune"
    with open(tmp, "w", encoding="utf-8") as fh:
        fh.write(raw2)
    os.replace(tmp, args.cocoons)
    print("backup: %s" % stamped)
    print("committed: %d 条重定向，剪除 %d 条通用连接边" % (len(changes), removed))
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
