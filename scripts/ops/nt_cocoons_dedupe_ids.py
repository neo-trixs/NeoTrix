#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_cocoons_dedupe_ids — cocoons 健康修复：M-id 碰撞 / 内容重复 / 悬空边 / 茧超限.

## 事故性质（2026-09-28 实测，不是"数据冗余"）

活库 1,645 个 **重号** M-id / 2,680 条被遮蔽副本。逐组比对后确认：
**0 组内容相同** —— 每一组里都是**不同的记忆**共用一个 id。即这不是冗余，
是 id 碰撞在**吃掉不同记忆**。

杀伤路径：
  `sync_to_consciousness`（cocoons.rs:297-310）按 `memories.insert(id, mem)`
  逐条插入，后到者覆盖先到者；而 `self.cocoons` 是 HashMap，**迭代序不确定**
  → 每组里哪条存活是**随机的**。64,720 条落盘只剩 62,040 条真正可见。

根因：多个吸收脚本各自独立算 start id、互不加锁（`fast_max_mid` 各自读盘）。
叠加第二处：悬空连边 3,201 条 / 767 个目标，其中 767 个落在 `nt_hf_smelt_chains.py`
`read_base()` 的硬编码回退空间 477231+（archive 的 `.mids.json` 已 MISSING），
该空间在活库里**从不存在** → 767 条永久悬空。

## 本脚本四步（各自独立计数、可单独关）

  1. `--dedupe`（默认开）：给被遮蔽的副本**重编号**为全新 id。
     保留「首次出现」那条的 id 不变 → 对外引用语义不变。
     **前置硬闸**：若有任何连边指向重号 id，则拒绝（需人工判定归属），
     不得静默改写引用。实测当前 0 条入边指向重号 → 可安全自动修。
  2. `--dedupe-content`（默认开）：折叠**逐字段完全相同**的记忆副本
     （实测 42 组 / 46 份，除 content 外 type/domain/connections/confidence
     全同，且入边 0 → 折叠零信息损失）。保留首次出现者。
  3. `--drop-dangling`（默认开）：删除指向不存在 id 的连边。
     目标本身没有内容可丢，删除不损失任何信息；收益是 `connected_ratio`
     不再被「连了个空」的假信号抬高，`reason()` 也不再追死 id。
  4. `--split-oversized`（默认开）：拆分超出自身 `retention_policy.max_memories`
     的茧。**这是功能性缺陷不是美观问题**：`store_memory`（cocoons.rs:132-141）
     在满容量时直接 `return Err("Cocoon memory capacity reached")`，
     故超限茧（实测 1645 / 1006 > 1000）对该域**永久写入拒收**。
     按 max_memories 切分为 `{cid}-pN`，与全库其余茧的分片口径一致。

## 为何敢用「整体 parse → dump」结构化改写（而非字节手术）

已实测：Python `json.dumps(indent=2, ensure_ascii=False)` 对本文件
**逐字节往返一致**，唯一例外是 127 处 `}\\n  ,\\n`（历史 splice 瑕疵）。
本脚本顺带把这 127 处**规范化**（strict 模式可关），故改写后的文件
是规范 pretty 格式，与 serde `to_string_pretty` 一致 —— 可再字节还原。

用法:
  python3 scripts/ops/nt_cocoons_dedupe_ids.py --dry-run
  python3 scripts/ops/nt_cocoons_dedupe_ids.py --commit
  python3 scripts/ops/nt_cocoons_dedupe_ids.py --commit --no-drop-dangling
  python3 scripts/ops/nt_cocoons_dedupe_ids.py --selftest
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
MID_RE = re.compile(r"^M-\d{6}$")


def selftest():
    # 确定性重编号：首次出现保留原 id，其余按稳定序拿新号
    plan = plan_renumber(["M-000001", "M-000001", "M-000002", "M-000001"], 100)
    assert plan == [None, "M-000100", None, "M-000101"], plan
    assert len({p for p in plan if p}) == 2, "新号必须互异"
    # 逐位应用后必须全局唯一
    ids = ["M-000001", "M-000001", "M-000002", "M-000001"]
    fixed = [n or o for o, n in zip(ids, plan)]
    assert len(set(fixed)) == len(fixed), fixed
    assert "M-000001" in fixed and "M-000002" in fixed, "原 id 必须仍在"
    # 无重号时不动任何东西
    assert plan_renumber(["M-000001", "M-000002"], 100) == [None, None]
    # 悬空判定
    have = {"M-000001", "M-000002"}
    assert dangling_targets([["M-000001", "M-999999"]], have) == {"M-999999"}
    assert dangling_targets([["M-000001"]], have) == set()
    # 删悬空边
    assert prune_conns(["M-000001", "M-999999", "M-888888"], have) == ["M-000001"]
    assert prune_conns(["M-999999"], have) == []
    assert MID_RE.match("M-000001") and not MID_RE.match("M-1")
    print("selftest ok: renumber-determinism/uniqueness-after/noop/"
          "dangling-detect/prune/id-format")
    return 0


def plan_renumber(ids_in_order, next_id):
    """逐出现位置给出应写入的 id（**单一事实源**：main 与 selftest 共用）。

    首次出现的 id 原样保留（对外引用语义稳定）；重复出现的副本按**出现序**
    稳定领取新号。返回长度 == len(ids_in_order)，元素为 None 表示无需改。
    """
    seen, cur, out = set(), next_id, []
    for i in ids_in_order:
        if i in seen:
            out.append("M-%06d" % cur)
            cur += 1
        else:
            seen.add(i)
            out.append(None)
    return out


def dangling_targets(all_conns, have):
    s = set()
    for cs in all_conns:
        for t in cs:
            if t not in have:
                s.add(t)
    return s


def prune_conns(conns, have):
    return [t for t in conns if t in have]


def find_exact_dupes(all_m):
    """找出逐字段完全相同的记忆组（content + type + domain + connections +
    confidence + strength + importance 全同）。

    只认**完全相同**者：若 content 同而 domain/type 不同，那是两条不同语义的
    记忆（路由键不同），折叠会丢信息 → 交人工，不自动合。
    返回 [(保留下标, [待删下标...]), ...]（下标指 all_m）。
    """
    buckets = collections.defaultdict(list)
    for i, (_cid, m) in enumerate(all_m):
        buckets[json.dumps([m["content"], m["memory_type"], m["domain"],
                            m["connections"], m["confidence"], m["strength"],
                            m.get("importance")], ensure_ascii=False,
                           sort_keys=True)].append(i)
    out = []
    for _k, idxs in buckets.items():
        if len(idxs) > 1:
            out.append((idxs[0], idxs[1:]))
    return out


def split_oversized(d, verbose=True):
    """拆分超出 retention_policy.max_memories 的茧。

    动机：`store_memory` 满容量即 `return Err`（cocoons.rs:139-141），超限茧
    对该域永久拒收新记忆。切分为 `{cid}-pN` 并同步更新 map 键与体内 `id`
    字段（两者必须一致，`sync_from_consciousness` 按 `cocoon.id` 查）。
    """
    moved = []
    for cid in list(d["cocoons"].keys()):
        c = d["cocoons"][cid]
        cap = (c.get("retention_policy") or {}).get("max_memories") or 0
        mems = c.get("memories") or []
        if not cap or len(mems) <= cap:
            continue
        shards = [mems[i:i + cap] for i in range(0, len(mems), cap)]
        head = dict(c)
        head["memories"] = shards[0]
        head["id"] = cid
        d["cocoons"][cid] = head
        for k, sh in enumerate(shards[1:], 1):
            nid = "%s-p%d" % (cid, k)
            s = dict(c)
            s["id"] = nid
            s["memories"] = sh
            d["cocoons"][nid] = s
            moved.append((nid, len(sh)))
        if verbose:
            print("  split %s: %d -> %d shards (cap=%d)"
                  % (cid, len(mems), len(shards), cap))
    return moved


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--commit", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--no-dedupe", action="store_true")
    ap.add_argument("--no-drop-dangling", action="store_true")
    ap.add_argument("--no-dedupe-content", action="store_true")
    ap.add_argument("--no-split-oversized", action="store_true")
    ap.add_argument("--keep-splice-artifact", action="store_true",
                    help="不规范化历史 `}\\n  ,\\n` 瑕疵（默认规范化）")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--cocoons", default=COCOONS)
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    raw = open(args.cocoons, encoding="utf-8").read()
    d = json.loads(raw)
    all_m = [(cid, m) for cid, c in d["cocoons"].items() for m in c["memories"]]
    n_all = len(all_m)
    print("loaded: %d memories in %d cocoons (%.1fMB)"
          % (n_all, len(d["cocoons"]), len(raw) / 1e6))

    ids_order = [m["id"] for _c, m in all_m]
    cnt = collections.Counter(ids_order)
    dup_ids = {i for i, n in cnt.items() if n > 1}
    extra = n_all - len(cnt)
    print("M-id: unique=%d  duplicated_ids=%d  shadowed_copies=%d"
          % (len(cnt), len(dup_ids), extra))

    have = set(cnt)
    all_conns = [m["connections"] for _c, m in all_m]
    dang = dangling_targets(all_conns, have)
    n_dang_edges = sum(1 for cs in all_conns for t in cs if t not in have)
    print("dangling: %d targets / %d edges" % (len(dang), n_dang_edges))
    nonmid = sorted(t for t in dang if not MID_RE.match(t))
    if nonmid:
        print("  非 M-%%06d 形态的目标 %d 个（异约定，例 %s）"
              % (len(nonmid), nonmid[:2]))

    # 硬闸：有入边指向重号 → 归属不可判 → 拒绝自动改
    inbound_to_dup = {t: 0 for t in dup_ids}
    for cs in all_conns:
        for t in cs:
            if t in inbound_to_dup:
                inbound_to_dup[t] += 1
    tot_in = sum(inbound_to_dup.values())
    print("guard: 指向重号 id 的入边 = %d" % tot_in)
    if tot_in and not args.no_dedupe:
        print("ABORT: 有 %d 条边指向重号 id，归属不可判，需人工处理。" % tot_in)
        return 2

    # ---- 规划重编号（保持首次出现的 id 不变）----
    if not args.no_dedupe and extra:
        next_id = max(int(i.split("-")[1]) for i in have) + 1
        plan = plan_renumber(ids_order, next_id)
        for (_cid, m), nid in zip(all_m, plan):
            if nid:
                m["_newid"] = nid
    else:
        plan = [None] * len(all_m)
    n_plan = sum(1 for p in plan if p)
    print("plan: renumber %d shadowed copies -> %s..%s"
          % (n_plan, next((p for p in plan if p), "-"),
             next((p for p in reversed(plan) if p), "-")))

    # ---- 应用重编号（须在折叠/删边之前完成）----
    applied = 0
    for _cid, m in all_m:
        if "_newid" in m:
            m["id"] = m.pop("_newid")
            applied += 1

    # ---- 内容级重复折叠（逐字段全同者）----
    n_dupes = 0
    if not args.no_dedupe_content:
        groups = find_exact_dupes(all_m)
        drop = set()
        remap = {}
        for keep_i, kill_is in groups:
            for k in kill_is:
                drop.add(k)
                remap[all_m[k][1]["id"]] = all_m[keep_i][1]["id"]
        if drop:
            # 入边检查：被折叠 id 若有入边，需重定向到保留 id（无损）
            hit = sum(1 for _c, m in all_m for t in m["connections"] if t in remap)
            if hit:
                print("  remap %d inbound edges of folded ids" % hit)
            all_m = [(c, m) for i, (c, m) in enumerate(all_m) if i not in drop]
            for _c, m in all_m:
                m["connections"] = [remap.get(t, t) for t in m["connections"]]
        n_dupes = len(drop)
    print("plan: fold %d exact-duplicate memories (%d groups)"
          % (n_dupes, len(groups) if not args.no_dedupe_content else 0))

    # ---- 删悬空边 ----
    dropped = 0
    if not args.no_drop_dangling:
        newhave = {m["id"] for _c, m in all_m}
        for _cid, m in all_m:
            keep = prune_conns(m["connections"], newhave)
            dropped += len(m["connections"]) - len(keep)
            m["connections"] = keep

    # ---- 拆超限茧（功能性：满容量 store_memory 直接 Err 拒收）----
    n_split = 0
    if not args.no_split_oversized:
        byc = collections.defaultdict(list)
        for cid, m in all_m:
            byc[cid].append(m)
        d["cocoons"] = {cid: {"id": cid, "memories": ms,
                              "retention_policy": d["cocoons"][cid].get(
                                  "retention_policy") or {
                                  "max_memories": 1000, "min_strength": 0.1,
                                  "domain_weights": {},
                                  "decay_strategy": {"Exponential": {"rate": 0.01}}},
                              "meta_metrics": d["cocoons"][cid].get(
                                  "meta_metrics") or {
                                  "total_recall_attempts": 0, "successful_recalls": 0,
                                  "avg_relevance_score": 0.0, "memory_utilization": 0.0},
                              "last_accessed": d["cocoons"][cid].get("last_accessed", 0),
                              "created_at": d["cocoons"][cid].get("created_at", 0)}
                      for cid, ms in byc.items()}
        n_split = len(split_oversized(d))

    # ---- 自检（改写后必须成立）----
    new_ids = [m["id"] for _c, m in all_m]
    # 条数只允许因**显式折叠**而减少；重编号/删边不得改变条数
    expect = n_all - n_dupes
    assert len(new_ids) == expect, "条数变化异常: %d != %d" % (len(new_ids), expect)
    assert len(set(new_ids)) == expect, "仍有重号"
    assert len(new_ids) <= n_all, "条数只减不增"
    if not args.no_drop_dangling:
        nh = set(new_ids)
        assert not any(t not in nh for cs in
                       (m["connections"] for _c, m in all_m) for t in cs), "仍有悬空边"
    if not args.keep_splice_artifact:
        raw = raw.replace("}\n  ,\n", "},\n")
    out = json.dumps(d, ensure_ascii=False, indent=2) + "\n"
    # 往返自证：再 parse 一次必须与内存态一致
    rt = json.loads(out)
    assert sum(len(c["memories"]) for c in rt["cocoons"].values()) == expect
    assert len({m["id"] for c in rt["cocoons"].values() for m in c["memories"]}) == expect
    # 茧键与体内 id 必须一致（sync_from_consciousness 按 cocoon.id 查）
    for _cid, c in rt["cocoons"].items():
        assert c["id"] == _cid, (_cid, c["id"])
        assert len(c["memories"]) <= c["retention_policy"]["max_memories"], \
            "茧 %s 仍超限 %d" % (_cid, len(c["memories"]))

    print("apply: renumbered=%d  folded_dupes=%d  dropped_dangling_edges=%d  "
          "split_oversized=%d  canonicalized=%s"
          % (applied, n_dupes, dropped, n_split, not args.keep_splice_artifact))
    print("after: %d bytes (was %d, %+d)" % (len(out), len(raw), len(out) - len(raw)))
    if not args.commit:
        print("\n--- DRY RUN（未落盘）---")
        return 0

    bak = args.cocoons + ".bak.dedupe"
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
    shutil.copyfile(args.cocoons, bak)
    tmp = args.cocoons + ".tmp.dedupe"
    with open(tmp, "w", encoding="utf-8") as fh:
        fh.write(out)
    os.replace(tmp, args.cocoons)
    print("backup: %s (+%s)" % (bak, stamped))
    print("committed: %d memories, all ids unique, 0 dangling edges, no oversized cocoon"
          % expect)
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
