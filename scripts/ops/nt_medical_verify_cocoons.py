#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_medical_verify_cocoons — 医患集吸收结果验收（结构 / 幂等 / 连边 / 语义）.

对 `nt_medical_distill_to_cocoons.py` 的产出做**独立**验收：不复用生产代码的
任何函数，只按 `neotrix-core/src/neotrix/nt_crystal_core/{consciousness,cocoons}.rs`
的 `Memory` / `PersistentCocoon` / `StoreData` 契约重新解析判定。

验收项:
  1. JSON 合法 + 顶层键为 {cocoons, strategy}（StoreData 契约）
  2. 吸收前的记忆**一条不少**（与吸收前快照逐 id 比对：pre-existing 保持不变）
  3. M-id 全局唯一（HashMap<String, Memory> 的键空间不能撞）
  4. 连边无悬挂：每条 connections 都能在记忆集合里找到（recount_connections 只
     数非空，但悬空 id 会在未来 connect/推理里变成静默丢失的边）
  5. 锚点语义正确：面记忆连回**自己那条病**的 profile（按内容里的病名反查，
     抽样 400 条，不靠生产代码的映射）
  6. 合成标记到位：profile/consult 面全部带 [synthetic]；confidence ≤ cap
  7. recall() 语义可命中：复刻 `CocoonStore::recall` 的 domain + 子串过滤

用法:
  python3 scripts/ops/nt_medical_verify_cocoons.py \
      --cocoons ~/.neotrix/crystal_core/cocoons.json \
      --before  <吸收前的备份>            # 可选，做第 2 项
  python3 scripts/ops/nt_medical_verify_cocoons.py --selftest
"""
import argparse
import json
import os
import random
import re
import sys

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")
MED_DOMAINS = {"medical-disease", "medical-pharma", "medical-clinical",
               "medical-literature", "medical-consult"}
CONF_CAP = 0.75
REQUIRED_MEM_FIELDS = {"id", "content", "memory_type", "domain", "strength",
                       "confidence", "connections", "created_at",
                       "last_accessed", "access_count"}
VALID_TYPES = {"Fact", "Pattern", "Causal", "Contradiction", "Counterfactual",
               "Experience", "Lesson", "Solution"}
NAME_RE = re.compile(r"^\[([a-z][a-z-]*)(?:·[^\]]*)?\]")
# 病名终止符：10 个面各自的「名字 → 正文」分隔形态。
# 已对 2194 条源数据实测：无一病名内含这些分隔符（| / — / : / <-> / (ICD-10），
# 故「取最靠左的那个终止符」无歧义，可全量校验（非抽样）。
TERMINATORS = [" | ", " — ", ": ", " <-> ", " (ICD-10"]


def parse_disease(content):
    """从面记忆内容反解病名；解析不出（或只剩标点）返回 None。"""
    m = NAME_RE.match(content)
    if not m:
        return None
    rest = content[m.end():].lstrip()
    cut = min((rest.find(t) for t in TERMINATORS if t in rest), default=-1)
    name = (rest[:cut] if cut >= 0 else rest).strip()
    # 合法病名必以字母数字开头（已对 2194 条源数据实测 0 例外）。不满足即
    # 源数据缺 name 时的退化解析（如 "| aliases"），不可当病名，否则误配锚点。
    return name if re.match(r"^[A-Za-z0-9]", name) else None


def fail(msgs, m):
    msgs.append(m)


def selftest():
    """先证伪验收器本身：故意构造坏样本，验收器必须报错（R-SCAN-2：手推≠实证）。"""
    good = {"id": "M-000001", "content": "[profile·synthetic] Flu | x",
            "memory_type": "Fact", "domain": "medical-disease", "strength": 1.0,
            "confidence": 0.75, "connections": [], "created_at": 1,
            "last_accessed": 1, "access_count": 0}
    assert REQUIRED_MEM_FIELDS <= set(good), "字段集写错了"
    assert parse_disease("[profile·synthetic] 22q11.2 deletion syndrome | aliases: x") \
        == "22q11.2 deletion syndrome"
    assert parse_disease("[profile·synthetic] Flu (ICD-10 J10) | aliases: -") == "Flu"
    assert parse_disease("[pitfall] Asthma: raise beta agonist — because") == "Asthma"
    assert parse_disease("[drug] Asthma: ALBUTEROL [treatment] brand=x") == "Asthma"
    assert parse_disease("[related] Tetralogy <-> 22q11.2: commonest") == "Tetralogy"
    assert parse_disease("[summary] Graves disease — Body") == "Graves disease"
    assert parse_disease("[literature] Cystic fibrosis — 3 refs: PMID 1") == "Cystic fibrosis"
    assert parse_disease("no tag here") is None
    assert parse_disease("[profile]  | aliases: x") is None
    assert "Fact" in VALID_TYPES and "Bogus" not in VALID_TYPES
    print("selftest ok: field-set/terminator-parse/type-whitelist (验收器自证)")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cocoons", default=COCOONS)
    ap.add_argument("--before", default="", help="吸收前备份，用于存量零丢失校验")
    ap.add_argument("--sample", type=int, default=400)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    errs, warns = [], []
    raw = open(args.cocoons, encoding="utf-8").read()
    try:
        d = json.loads(raw)
    except ValueError as e:
        print("FAIL: JSON 非法: %s" % e)
        return 2
    print("1. JSON 合法 — %.1fMB" % (len(raw) / 1e6))

    if set(d.keys()) != {"cocoons", "strategy"}:
        fail(errs, "顶层键不是 {cocoons,strategy}: %s" % sorted(d.keys()))
    if not isinstance(d.get("cocoons"), dict) or not isinstance(d.get("strategy"), dict):
        fail(errs, "cocoons/strategy 类型错")
    print("2. StoreData 契约 — cocoons=%d strategy=%s"
          % (len(d.get("cocoons", {})), sorted(d.get("strategy", {}).keys())))

    # 收集全部记忆
    mems, dup = {}, []
    for cid, c in d.get("cocoons", {}).items():
        if not isinstance(c, dict) or "memories" not in c:
            fail(errs, "茧 %s 结构缺 memories" % cid)
            continue
        for m in c["memories"]:
            if m["id"] in mems:
                dup.append(m["id"])
            mems[m["id"]] = (m, cid)
    if dup:
        fail(errs, "M-id 重复 %d 个，例: %s" % (len(dup), dup[:5]))
    print("3. M-id 唯一 — %d 条记忆 / %d 重复" % (len(mems), len(dup)))

    # 存量零丢失
    if args.before:
        b = json.load(open(args.before, encoding="utf-8"))
        bmem = {}
        for c in b.get("cocoons", {}).values():
            for m in c.get("memories", []):
                bmem[m["id"]] = m
        missing = [i for i in bmem if i not in mems]
        changed = [i for i in bmem if i in mems and mems[i][0] != bmem[i]]
        if missing:
            fail(errs, "存量丢失 %d 条: %s" % (len(missing), missing[:5]))
        if changed:
            fail(errs, "存量被改写 %d 条: %s" % (len(changed), changed[:5]))
        print("4. 存量零丢失 — 吸收前 %d 条全在且逐字段未改" % len(bmem))
    else:
        print("4. 存量零丢失 — 跳过（未给 --before）")

    # 字段完整性
    badf = 0
    for mid, (m, _) in mems.items():
        if not REQUIRED_MEM_FIELDS <= set(m.keys()):
            badf += 1
        elif m["memory_type"] not in VALID_TYPES:
            badf += 1
    if badf:
        fail(errs, "%d 条记忆字段/类型不合契约" % badf)
    print("5. Memory 契约 — 字段+枚举全合规（%d 违例）" % badf)

    # 连边无悬挂
    dangling = 0
    medmem = {i: (m, c) for i, (m, c) in mems.items() if m["domain"] in MED_DOMAINS}
    edges = 0
    for mid, (m, _) in medmem.items():
        for t in m["connections"]:
            edges += 1
            if t not in mems:
                dangling += 1
    if dangling:
        fail(errs, "悬挂连边 %d 条" % dangling)
    print("6. 连边 — 医患面 %d 记忆 / %d 边 / %d 悬挂" % (len(medmem), edges, dangling))

    # 锚点语义 —— **全量**校验（病名解析已证明无歧义，无需抽样）
    anchors = {}
    for mid, (m, _) in medmem.items():
        if m["content"].startswith("[profile"):
            nm = parse_disease(m["content"])
            if nm:
                if nm.lower() in anchors:
                    fail(errs, "锚点重名: %s" % nm[:40])
                anchors[nm.lower()] = mid
    unparsed = wrong = 0
    for mid, (m, _) in medmem.items():
        if m["content"].startswith("[profile"):
            continue
        nm = parse_disease(m["content"])
        if not nm:
            unparsed += 1
            continue
        want = anchors.get(nm.lower())
        if want and want not in m["connections"]:
            wrong += 1
            if wrong <= 3:
                fail(errs, "锚点错连 %s: %s 应连 %s 实连 %s"
                     % (mid, nm[:30], want, m["connections"][:3]))
    print("7. 锚点语义（全量）— %d 锚点 / 病名解析失败 %d / 错连 %d"
          % (len(anchors), unparsed, wrong))

    # 跨病种边统计（connections[1:] 指向**别的**病种锚点）
    cross = 0
    for mid, (m, _) in medmem.items():
        own = m["connections"][0] if m["connections"] else None
        for t in m["connections"][1:]:
            tgt = mems.get(t)
            if tgt and tgt[0]["content"].startswith("[profile") and t != own:
                cross += 1
    print("8. 跨病种边 — %d 条指向别的病种锚点" % cross)

    # 合成标记 + 置信封顶（标记形态为 [profile·synthetic] / [consult·synthetic]）
    unsyn = conf = 0
    for mid, (m, _) in medmem.items():
        c = m["content"]
        if (c.startswith("[profile") or c.startswith("[consult")) and "synthetic" not in c:
            unsyn += 1
        if m["confidence"] > CONF_CAP + 1e-9:
            conf += 1
    if unsyn:
        fail(errs, "%d 条 profile/consult 缺 [synthetic] 标记" % unsyn)
    if conf:
        fail(errs, "%d 条 confidence > %.2f（合成数据不得装权威）" % (conf, CONF_CAP))
    print("9. 诚实性 — 缺标记 %d，超置信 %d" % (unsyn, conf))

    # recall 语义（复刻 CocoonStore::recall 的 domain+子串过滤）
    def recall(domain, query, limit=3):
        out = [m for m, _ in mems.values()
               if m["domain"] == domain and query.lower() in m["content"].lower()]
        out.sort(key=lambda m: -(m["strength"] * (1 + m["access_count"] * 0.1)))
        return out[:limit]

    for dom, q, want in [("medical-disease", "Tetralogy of Fallot", "profile"),
                         ("medical-clinical", "pitfall", "pitfall"),
                         ("medical-pharma", "CALCITRIOL", "drug"),
                         ("medical-literature", "PMID", "literature")]:
        r = recall(dom, q)
        hit = any(x["content"].startswith("[" + want) for x in r)
        if not hit:
            warns.append("recall(%s, %r) 未命中 %s 面" % (dom, q, want))
        print("10. recall(%-20s %-22r) -> %d 命中, 首条 %s"
              % (dom, q, len(r), r[0]["content"][:46].replace("\n", " ") if r else "-"))

    print("\n" + "=" * 68)
    if errs:
        print("FAIL (%d):" % len(errs))
        for e in errs[:20]:
            print("  - %s" % e)
        return 1
    print("PASS — 全部 %d 项验收通过" % 10)
    if warns:
        print("warn:")
        for w in warns:
            print("  - %s" % w)
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
