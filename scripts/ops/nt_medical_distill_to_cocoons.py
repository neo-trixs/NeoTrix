#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_medical_distill_to_cocoons — 医患对话疾病集 → 晶体核心记忆茧（结论直给）.

数据源（Apache-2.0，synthetic）：
  `nisten/opus5-5-doctor-patient-conversations-all-human-diseases`
  2194 条记录 = 2194 个疾病，每条 20 键结构化病历卡 + ChatML 问诊对话。

本脚本只做「蒸馏」，不做「倾倒」：75MB 原始对话被拆成**可路由、可连边、
高信号**的记忆面（profile / summary / drug / interaction / pitfall /
differential / related / literature / consult），而不是把 23KB 的对话
原文塞进记忆库。

与 `nt_hf_digest_to_cocoons.py`（crawl 行直灌）的关系：
  - 本脚本**复用**其已验证的 `cocoon_entry` / `splice` / `fast_max_mid` /
    备份轮转，不复制粘贴 → 茧条目格式永不漂移（R-P0-2 原子写同源）。
  - 区别在记忆**形态**：crawl 行是 `[url] title` 事实碎片；本脚本是
    `memory_type` 分型（Fact/Causal/Lesson/Pattern/Experience）+ 真实
    `connections` 跨病种连边。

「融入」而非「存入」的关键 —— 跨病种知识图：
  每条记录的 profile 记忆是**锚点**（anchor）。本记录的所有面记忆连回锚点；
  `related_diseases[].disease` 经精确 + 模糊（token Jaccard ≥ 0.55）解析到
  另一条记录的锚点，形成 2194 节点上的真实医学关系边（约 5051 条，
  44.8% 精确 + 2.0% 模糊；未命中者仍留文本记忆，只是不连边）。
  这直接喂 `CrystalConsciousness::recount_connections` → 进化相位
  （connected_ratio 抬升是 Transcend 的四个必要条件之一）。

**诚实性约束（重要）**：本数据集由 Claude Opus 5.5 **合成**，非临床ground truth。
故 confidence 上限 0.75（合成叙述 = 可信线索，非已验证事实），
profile/consult 面额外打 `[synthetic]` 标记，随内容一起传播；
provenance / sha256 / license 全量落在项目 manifest 里。
把合成医学文本按 0.9+ 置信度灌进知识核心 = 伪造权威，故不做。

用法（项目根执行）:
  python3 scripts/ops/nt_medical_distill_to_cocoons.py --dry-run
  python3 scripts/ops/nt_medical_distill_to_cocoons.py --commit
  python3 scripts/ops/nt_medical_distill_to_cocoons.py --commit --limit 300
  python3 scripts/ops/nt_medical_distill_to_cocoons.py --selftest
"""
import argparse
import json
import os
import re
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
# 复用已验证的茧条目/拼接/备份实现（格式单一事实源，禁复制）
import nt_hf_digest_to_cocoons as base  # noqa: E402

RAW = os.path.join("datasets", "hf_raw", "opus5-5diseaseconversations.jsonl")
MANIFEST = os.path.join("datasets", "hf_medical", "absorb_manifest.json")

DS_ID = "nisten/opus5-5-doctor-patient-conversations-all-human-diseases"
DS_URL = "https://huggingface.co/datasets/" + DS_ID
RAW_SHA256 = "f828c30cee7006a3cf5b88909f9b865688f98b11d4c886108b9b9a39e5402e0b"
LICENSE = "apache-2.0"
GENERATOR = "Claude Opus 5.5 (synthetic — NOT clinical ground truth)"

# 合成数据置信上限：合成叙述 ≠ 已验证临床事实
CONF_CAP = 0.75

# 记忆面 → (domain, memory_type, confidence, importance)
# domain 用 kebab-case，对齐 CrystalStore::recall(domain, query) 的路由键。
FACETS = {
    "profile":       ("medical-disease",  "Fact",       0.75, 0.80),
    "summary":       ("medical-disease",  "Pattern",    0.70, 0.90),
    "drug":          ("medical-pharma",   "Causal",     0.70, 0.60),
    "interaction":   ("medical-pharma",   "Causal",     0.70, 0.70),
    "food":          ("medical-pharma",   "Causal",     0.60, 0.50),
    "pitfall":       ("medical-clinical", "Lesson",     0.70, 0.85),
    "differential":  ("medical-clinical", "Pattern",    0.65, 0.70),
    "related":       ("medical-clinical", "Causal",     0.65, 0.65),
    "literature":    ("medical-literature", "Fact",     0.75, 0.55),
    "consult":       ("medical-consult",  "Experience", 0.55, 0.75),
}
DOMAIN_ORDER = ["medical-disease", "medical-pharma", "medical-clinical",
                "medical-literature", "medical-consult"]

WS_RE = re.compile(r"\s+")
STOP = {"syndrome", "disease", "disorder", "of", "the", "and", "with",
        "in", "a", "an", "to", "for"}
JACCARD_MIN = 0.55


def norm(s):
    return WS_RE.sub(" ", re.sub(r"[^a-z0-9]+", " ", (s or "").lower())).strip()


def toks(s):
    return {t for t in norm(s).split() if t not in STOP and len(t) > 2}


def flat(s, cap=0):
    """压平空白（源数据含 markdown 换行/## 标题），可选硬截断。"""
    t = WS_RE.sub(" ", (s or "")).strip()
    if cap and len(t) > cap:
        t = t[:cap].rsplit(" ", 1)[0] + " …"
    return t


def sget(d, *ks, default=""):
    for k in ks:
        v = d.get(k)
        if isinstance(v, str) and v.strip():
            return v
    return default


def snum(d, *ks, default=""):
    """取标量字段（int/float/str 皆可）。`pubmed_refs[].year` 是 **int**，
    用 sget 只收 str 会静默丢年份（本轮实测 6377 条 PMID 年份全丢）→ 故单列。"""
    for k in ks:
        v = d.get(k)
        if isinstance(v, bool) or v is None:
            continue
        if isinstance(v, (int, float)):
            return str(v)
        if isinstance(v, str) and v.strip():
            return v
    return default


def lst(d, k):
    v = d.get(k)
    return v if isinstance(v, list) else []


# ---------------------------------------------------------------- pass A: 索引
def build_index(path):
    """流式扫全量：每条记录的记忆条数 + 病名/别名 → 记录序 的解析表。

    返回 (offsets, nrec, exact, tok2rec, key_toks)：
      offsets[i] = 第 i 条记录首个记忆的相对偏移（锚点相对 base 的位置）
    """
    offsets, exact, tok2rec, key_toks = [], {}, {}, {}
    nrec = 0
    with open(path, encoding="utf-8") as fh:
        for ln in fh:
            ln = ln.strip()
            if not ln:
                continue
            d = json.loads(ln)
            offsets.append(count_facets(d, 0))
            nrec += 1
            if nrec % 500 == 0:
                sys.stderr.write("\r  index pass: %d records" % nrec)
                sys.stderr.flush()
    sys.stderr.write("\r")
    # 精确索引：name + aliases + search
    with open(path, encoding="utf-8") as fh:
        for i, ln in enumerate(fh):
            ln = ln.strip()
            if not ln:
                continue
            d = json.loads(ln)
            for k in [sget(d, "name")] + [x for x in lst(d, "aliases") if isinstance(x, str)] \
                     + [x for x in lst(d, "search") if isinstance(x, str)]:
                n = norm(k)
                if n and n not in exact:
                    exact[n] = i
            ts = set()
            for k in [sget(d, "name")] + [x for x in lst(d, "aliases") if isinstance(x, str)]:
                ts |= toks(k)
            key_toks[i] = ts
            for t in ts:
                tok2rec.setdefault(t, set()).add(i)
    return offsets, nrec, exact, tok2rec, key_toks


def resolve(name, exact, tok2rec, key_toks, self_i=-1):
    """病名 → 记录序。精确优先，失败退模糊（token Jaccard ≥ 0.55）。"""
    n = norm(name)
    if not n:
        return None
    if n in exact:
        return exact[n]
    t = toks(name)
    if not t:
        return None
    cand = {}
    for x in t:
        for r in tok2rec.get(x, ()):  # noqa: B007
            cand[r] = cand.get(r, 0) + 1
    best, bs = None, 0.0
    for r, hit in cand.items():
        if r == self_i:
            continue
        kt = key_toks.get(r)
        if not kt:
            continue
        j = hit / float(len(t | kt))
        if j > bs:
            bs, best = j, r
    return best if bs >= JACCARD_MIN else None


# ---------------------------------------------------------------- 记忆构造
def count_facets(d, cap):
    """与 distill() 的产出条数严格一致（dry-run 与 commit 共用此口径）。"""
    c = cap
    n = 1                                    # profile（锚点，必出）
    if flat(sget(d, "executive_summary")):
        n += 1                                # summary 仅在有 executive_summary 时出
    n += min(len(lst(d, "related_drugs")), c) if c else len(lst(d, "related_drugs"))
    n += min(len(lst(d, "drug_interactions")), c) if c else len(lst(d, "drug_interactions"))
    n += min(len(lst(d, "food_interactions")), c) if c else len(lst(d, "food_interactions"))
    n += min(len(lst(d, "common_mistakes")), c) if c else len(lst(d, "common_mistakes"))
    n += min(len(lst(d, "differential_diagnosis")), c) if c else len(lst(d, "differential_diagnosis"))
    n += min(len(lst(d, "related_diseases")), c) if c else len(lst(d, "related_diseases"))
    n += 1 if lst(d, "pubmed_refs") else 0
    n += 1 if lst(d, "conversation") else 0
    return n


def distill(d, anchor, cap, exact, tok2rec, key_toks, self_i, consult_chars):
    """把一条疾病记录蒸馏成 (facet_kind, memory_dict) 列表。锚点 = profile。"""
    name = flat(sget(d, "name", "source_disease"), 120)
    out = []

    def mk(facet, content, conns=None, conf=None, imp=None):
        dom, mt, c0, i0 = FACETS[facet]
        cc = CONF_CAP if conf is None else min(conf, CONF_CAP)
        ii = i0 if imp is None else imp
        out.append((facet, {
            "id": None,                      # commit 时统一编号
            "content": content,
            "memory_type": mt,
            "domain": dom,
            "strength": 1.0,
            "confidence": cc,
            "importance": ii,
            "connections": conns or [],
            "created_at": 0,                 # commit 时统一盖时间戳
            "last_accessed": 0,
            "access_count": 0,
        }))

    # 1) profile —— 锚点。aliases 命中其他记录时反向连边（入边）。
    aliases = [flat(a, 60) for a in lst(d, "aliases") if isinstance(a, str)]
    systems = ", ".join(flat(s, 40) for s in lst(d, "body_systems") if isinstance(s, str))
    prof = ("[profile·synthetic] %s%s | aliases: %s | prevalence: %s | body systems: %s | %s"
            % (name,
               (" (ICD-10 %s)" % flat(sget(d, "icd10"), 24)) if sget(d, "icd10") else "",
               "; ".join(aliases[:8]) or "-",
               flat(sget(d, "prevalence"), 160) or "-",
               systems or "-",
               flat(sget(d, "description"), 900) or "-"))
    mk("profile", prof)
    # 锚点 M-id 由 commit 的前缀和预算好（= 本条 profile 的 id）；容错 int 入参
    anchor_id = anchor if isinstance(anchor, str) else "M-%06d" % anchor

    # 2) summary —— executive_summary 已是对话蒸馏，勿再压
    es = flat(sget(d, "executive_summary"), 1600)
    if es:
        mk("summary", "[summary] %s — %s" % (name, es), [anchor_id])

    # 3) 用药
    drugs = [x for x in lst(d, "related_drugs") if isinstance(x, dict)]
    if cap:
        drugs = drugs[:cap]
    drug_ids = {}
    for dr in drugs:
        ing = flat(sget(dr, "ingredient"), 80)
        if not ing:
            continue
        mid = "D:" + ing.lower()
        drug_ids[mid] = True
        mk("drug", "[drug] %s: %s [%s] brand=%s ATC=%s DIN=%s"
           % (name, ing, flat(sget(dr, "role"), 40) or "-",
              flat(sget(dr, "brand_name"), 40) or "-",
              flat(sget(dr, "atc_code"), 16) or "-",
              flat(sget(dr, "din"), 12) or "-"), [anchor_id])

    # 4) 药物相互作用 —— 连到本记录内对应 drug 面（面内连边）
    inter = [x for x in lst(d, "drug_interactions") if isinstance(x, dict)]
    if cap:
        inter = inter[:cap]
    for it in inter:
        a, b = sget(it, "drug_a"), sget(it, "drug_b")
        if not (a or b):
            continue
        conns = [anchor_id]
        mk("interaction", "[drug-interaction] %s: %s + %s — %s: %s"
           % (name, a or "?", b or "?", flat(sget(it, "severity"), 20) or "?",
              flat(sget(it, "description"), 500) or "-"), conns)

    # 5) 食物相互作用
    food = [x for x in lst(d, "food_interactions") if isinstance(x, dict)]
    if cap:
        food = food[:cap]
    for it in food:
        mk("food", "[food-interaction] %s: %s x %s — %s: %s (%s)"
           % (name, flat(sget(it, "drug"), 60) or "?", flat(sget(it, "food"), 60) or "?",
              flat(sget(it, "severity"), 20) or "?", flat(sget(it, "mechanism"), 200) or "-",
              flat(sget(it, "description"), 400) or "-"), [anchor_id])

    # 6) 常见错误 —— 临床最高价值面（Lesson）
    mis = [x for x in lst(d, "common_mistakes") if isinstance(x, dict)]
    if cap:
        mis = mis[:cap]
    for it in mis:
        m = flat(sget(it, "mistake"), 300)
        if not m:
            continue
        mk("pitfall", "[pitfall] %s: %s — %s"
           % (name, m, flat(sget(it, "explanation"), 600) or "-"), [anchor_id])

    # 7) 鉴别诊断 —— 条件能解析到别的记录就连跨病种边
    diff = [x for x in lst(d, "differential_diagnosis") if isinstance(x, dict)]
    if cap:
        diff = diff[:cap]
    for it in diff:
        cond = flat(sget(it, "condition"), 160)
        if not cond:
            continue
        conns = [anchor_id]
        t = resolve(cond, exact, tok2rec, key_toks, self_i)
        if t is not None:
            conns.append("A:%d" % t)          # commit 时换成该记录锚点 M-id
        mk("differential", "[differential] %s: %s — %s"
           % (name, cond, flat(sget(it, "differentiating_factors"), 600) or "-"), conns)

    # 8) 关联病种 —— 跨病种图的主干边
    rel = [x for x in lst(d, "related_diseases") if isinstance(x, dict)]
    if cap:
        rel = rel[:cap]
    for it in rel:
        dn = flat(sget(it, "disease"), 160)
        if not dn:
            continue
        conns = [anchor_id]
        t = resolve(dn, exact, tok2rec, key_toks, self_i)
        if t is not None:
            conns.append("A:%d" % t)
        mk("related", "[related] %s <-> %s: %s"
           % (name, dn, flat(sget(it, "relationship"), 400) or "-"), conns)

    # 9) 文献 —— 整卡一条（pmid 可检索）
    refs = [x for x in lst(d, "pubmed_refs") if isinstance(x, dict)]
    if refs:
        parts = []
        for r in refs[:24]:
            parts.append("PMID %s (%s) %s" % (flat(sget(r, "pmid"), 12),
                                             flat(snum(r, "year"), 6) or "?",
                                             flat(sget(r, "title"), 140)))
        mk("literature", "[literature] %s — %d refs: %s"
           % (name, len(refs), "; ".join(parts)), [anchor_id])

    # 10) 问诊对话 —— 只取首尾弧（首 3 轮 + 末条医生陈述），丢 system 样板
    conv = [m for m in lst(d, "conversation")
            if isinstance(m, dict) and m.get("role") != "system"]
    if conv:
        picks = conv[:6]
        if len(conv) > 7:
            picks = picks + [conv[-1]]
        turns = []
        for m in picks:
            r = "Patient" if m.get("role") == "user" else "Clinician"
            turns.append("[%s] %s" % (r, flat(m.get("content"), 320)))
        body = flat(" ".join(turns), consult_chars)
        mk("consult", "[consult·synthetic] %s — clinician: %s | patient: %s | "
                      "dialogue (%d turns total): %s"
           % (name, flat(sget(d, "clinician_persona"), 260) or "-",
              flat(sget(d, "patient_scenario"), 420) or "-",
              len(conv), body), [anchor_id])
    return out


# ---------------------------------------------------------------- 主流程
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--raw", default=RAW)
    ap.add_argument("--commit", action="store_true", help="真落盘（默认/--dry-run 只投影）")
    ap.add_argument("--dry-run", action="store_true",
                    help="显式干跑（默认行为，留作与兄弟脚本一致的开关）")
    ap.add_argument("--limit", type=int, default=0, help="只取前 N 条记录（0=全量）")
    ap.add_argument("--cap", type=int, default=0, help="每个面每条记录最多取 N 项（0=不限）")
    ap.add_argument("--consult-chars", type=int, default=1400)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    if not os.path.exists(args.raw):
        print("ABORT: raw not found: %s" % args.raw)
        return 2

    t0 = time.time()
    print("raw: %s (%.1fMB)" % (args.raw, os.path.getsize(args.raw) / 1e6))
    print("pass A: building name index (2 stream scans) …")
    offsets, nrec, exact, tok2rec, key_toks = build_index(args.raw)
    if args.limit:
        offsets = offsets[:args.limit]
        nrec = len(offsets)
    total = sum(offsets)
    print("records=%d  distilled memories=%d  (%.1f/record, %.1fs)"
          % (nrec, total, total / max(nrec, 1), time.time() - t0))

    mx, n = base.fast_max_mid(base.COCOONS)
    print("cocoons: %.1fMB  max M-id: M-%06d (%s)"
          % (os.path.getsize(base.COCOONS) / 1e6, mx,
             "cache hit" if n < 0 else "rescanned"))

    # 锚点 M-id 表：记录序 → 锚点编号（offsets 前缀和）
    anchor_of, run = {}, mx
    for i, c in enumerate(offsets):
        anchor_of[i] = "M-%06d" % (run + 1)
        run += c

    # pass B：流式生成 → 按 domain 分片 → 边写边落临时片段
    now = int(time.time())
    next_id, i, per_dom = mx + 1, 0, {d: [] for d in DOMAIN_ORDER}
    new_ids = []
    frag_path = args.raw + ".frag.tmp"
    handles, frag_bytes, wrote = {}, 0, []
    cross = 0
    with open(args.raw, encoding="utf-8") as fh, \
            open(frag_path, "w", encoding="utf-8") as spill:
        def flush(domain, mems, part):
            if not mems:
                return
            cid = "cocoon-%s-%d%s" % (domain, now, "" if part == 0 else "-p%d" % part)
            blk = base.cocoon_entry(cid, mems, now)
            spill.write(blk + ",\n")
            return cid

        part = {d: 0 for d in DOMAIN_ORDER}
        for ln in fh:
            ln = ln.strip()
            if not ln:
                continue
            if args.limit and i >= args.limit:
                break
            d = json.loads(ln)
            out = distill(d, anchor_of[i], args.cap, exact, tok2rec, key_toks, i,
                          args.consult_chars)
            for _facet, m in out:
                m["id"] = "M-%06d" % next_id
                new_ids.append(m["id"])
                next_id += 1
                m["created_at"] = now
                m["last_accessed"] = now
                conns = []
                for c in m["connections"]:
                    if c.startswith("A:"):
                        cross += 1
                        conns.append(anchor_of[int(c[2:])])
                    else:
                        conns.append(c)
                m["connections"] = conns
                dom = m["domain"]
                per_dom[dom].append(m)
                if len(per_dom[dom]) >= base.SHARD:
                    cid = flush(dom, per_dom[dom], part[dom])
                    if cid:
                        wrote.append((cid, len(per_dom[dom])))
                    part[dom] += 1
                    per_dom[dom] = []
            i += 1
            if i % 500 == 0:
                sys.stderr.write("\r  distill pass: %d/%d records" % (i, nrec))
                sys.stderr.flush()
        sys.stderr.write("\r")
        for dom in DOMAIN_ORDER:
            cid = flush(dom, per_dom[dom], part[dom])
            if cid:
                wrote.append((cid, len(per_dom[dom])))
            per_dom[dom] = []
    frag = os.path.getsize(frag_path)
    print("new cocoons: %d  memories=%d  cross-disease edges=%d  frag=%.1fMB"
          % (len(wrote), total, cross, frag / 1e6))
    for cid, cnt in wrote:
        print("   %-44s %5d" % (cid, cnt))

    manifest = {
        "source": DS_ID, "url": DS_URL, "license": LICENSE,
        "generator": GENERATOR, "raw_sha256": RAW_SHA256,
        "raw_bytes": os.path.getsize(args.raw), "records": nrec,
        "cap_per_facet": args.cap, "limit": args.limit,
        "memories": total, "cross_disease_edges": cross,
        "new_cocoons": [c for c, _ in wrote],
        "base_max_mid": mx, "mids_from": mx + 1, "mids_to": next_id - 1,
        "cocoons_bytes_before": os.path.getsize(base.COCOONS),
        "fragment_bytes": frag, "confidence_cap": CONF_CAP,
        "note": "synthetic Opus-5.5 data; confidence capped at %.2f; "
                "profile/consult tagged [synthetic]" % CONF_CAP,
    }

    if not args.commit:
        os.remove(frag_path)
        print("\n--- DRY RUN (no disk change) ---")
        print("projected cocoons.json: %.1fMB -> ~%.1fMB"
              % (manifest["cocoons_bytes_before"] / 1e6,
                 (manifest["cocoons_bytes_before"] + frag) / 1e6))
        print("re-run with --commit to splice.")
        return 0

    # 备份（R-P0-2 同款轮转，拒绝 wiped-live 提交）
    bak = base.COCOONS + ".bak.medical"
    live = os.path.getsize(base.COCOONS)
    import glob
    import shutil
    import time as _t
    # 写盘前硬闸：新 id 不得与盘上重号（2026-09-28 事故：id 碰撞静默遮蔽不同记忆）
    base.guard_id_collision(open(base.COCOONS, "rb").read(), new_ids)
    olds = sorted(glob.glob(bak + ".*"))
    for old in olds:
        try:
            if os.stat(old).st_size > 1000000 and live < 1000000:
                print("ABORT: live %.1fKB, backup %s kept" % (live / 1e3, old))
                os.remove(frag_path)
                return 2
        except OSError:
            pass
    stamped = "%s.%s" % (bak, _t.strftime("%Y%m%d-%H%M%S"))
    shutil.copyfile(base.COCOONS, stamped)
    for old in olds[:-2]:
        try:
            os.remove(old)
        except OSError:
            pass
    shutil.copyfile(base.COCOONS, bak)
    print("backup: %s (+stamped %s)" % (bak, stamped))

    with open(frag_path, encoding="utf-8") as fh:
        entries = fh.read().rstrip()
    if entries.endswith(","):
        entries = entries[:-1]
    new_data = base.splice(base.COCOONS, entries)
    tmp = base.COCOONS + ".tmp.medical"
    with open(tmp, "wb") as fh:
        fh.write(new_data)
    os.replace(tmp, base.COCOONS)
    os.remove(frag_path)
    manifest["cocoons_bytes_after"] = os.path.getsize(base.COCOONS)
    os.makedirs(os.path.dirname(MANIFEST), exist_ok=True)
    with open(MANIFEST + ".tmp", "w", encoding="utf-8") as fh:
        json.dump(manifest, fh, ensure_ascii=False, indent=1)
    os.replace(MANIFEST + ".tmp", MANIFEST)
    # 刷新 base 的 mid 缓存，避免下次 fast_max_mid 走错
    try:
        st = os.stat(base.COCOONS)
        os.makedirs(os.path.dirname(base.MID_STATE), exist_ok=True)
        with open(base.MID_STATE, "w", encoding="utf-8") as fh:
            json.dump({"max_mid": next_id - 1, "mtime": int(st.st_mtime),
                       "size": st.st_size}, fh)
    except OSError:
        pass
    print("committed: cocoons.json %.1fMB -> %.1fMB  (%.1fs)"
          % (manifest["cocoons_bytes_before"] / 1e6,
             manifest["cocoons_bytes_after"] / 1e6, time.time() - t0))
    print("manifest: %s" % MANIFEST)
    return 0


def selftest():
    d = {
        "name": "22q11.2 deletion syndrome", "icd10": "Q93.81",
        "aliases": ["DiGeorge syndrome", "VCFS"], "search": ["digeorge"],
        "prevalence": "~1 in 2,000–4,000", "body_systems": ["cardiovascular"],
        "description": "microdeletion\ndisorder.", "executive_summary": "## Title\n\nBody.",
        "related_drugs": [{"ingredient": "CALCITRIOL", "role": "treatment",
                           "brand_name": "Rocaltrol", "atc_code": "A11AA05", "din": "02237528"}],
        "drug_interactions": [{"drug_a": "CALCITRIOL", "drug_b": "THIAZIDE",
                               "severity": "moderate", "description": "↑ Ca reabsorption"}],
        "food_interactions": [{"drug": "WARFARIN", "food": "kale", "severity": "moderate",
                               "mechanism": "vit K", "description": "↓ INR"}],
        "common_mistakes": [{"mistake": "Raising calcitriol", "explanation": "PTH low"}],
        "differential_diagnosis": [{"condition": "Hypomagnesemia",
                                    "differentiating_factors": "Mg < 1.2"}],
        "related_diseases": [{"disease": "Tetralogy of Fallot", "relationship": "commonest"}],
        "pubmed_refs": [{"pmid": "24577245", "title": "Psychiatric disorders", "year": 2014}],        "conversation": [{"role": "system", "content": "sys boilerplate"},
                         {"role": "user", "content": "I have chest pain"},
                         {"role": "assistant", "content": "Consider ECG"},
                         {"role": "user", "content": "It started yesterday"},
                         {"role": "assistant", "content": "Final: QT risk"}],
        "clinician_persona": "Dr. X, internist", "patient_scenario": "A 26-year-old woman",
    }
    # 索引：自己解析到自己应被 self_i 挡掉
    exact, tok2rec, key_toks = {}, {}, {}
    exact[norm("Tetralogy of Fallot")] = 99
    key_toks[99] = toks("Tetralogy of Fallot")
    for t in key_toks[99]:
        tok2rec.setdefault(t, set()).add(99)
    out = distill(d, "M-000001", 0, exact, tok2rec, key_toks, 0, 1400)
    kinds = [f for f, _ in out]
    by = {f: m for f, m in out}
    # 形态
    assert kinds[0] == "profile", kinds
    assert by["profile"]["id"] is None and by["profile"]["connections"] == []
    assert by["summary"]["connections"] == ["M-000001"]
    assert by["profile"]["domain"] == "medical-disease"
    assert by["pitfall"]["memory_type"] == "Lesson"
    assert by["interaction"]["memory_type"] == "Causal"
    # 内容：换行压平 + 合成标记 + 截断省略号
    assert "microdeletion disorder" in by["profile"]["content"]
    assert "[profile·synthetic]" in by["profile"]["content"]
    assert "[consult·synthetic]" in by["consult"]["content"]
    assert "sys boilerplate" not in by["consult"]["content"], "system 样板必须丢弃"
    assert "dialogue (4 turns total)" in by["consult"]["content"]
    assert "## Title Body." in by["summary"]["content"]
    # 跨病种边：Tetralogy of Fallot → 记录 99
    assert by["related"]["connections"] == ["M-000001", "A:99"], by["related"]["connections"]
    assert by["differential"]["connections"] == ["M-000001"], "Hypomagnesemia 不在索引"
    # 置信封顶
    for _f, m in out:
        assert m["confidence"] <= CONF_CAP, (m["confidence"], m["content"][:40])
    # 条数口径：count_facets 与 distill 产出一致（dry-run/commit 同源，禁漂移）
    assert count_facets(d, 0) == len(out), (count_facets(d, 0), len(out))
    assert count_facets(d, 2) == len(distill(d, 1, 2, exact, tok2rec, key_toks, 0, 1400))
    # cap 生效
    capped = distill(d, 1, 1, exact, tok2rec, key_toks, 0, 1400)
    assert sum(1 for f, _ in capped if f == "pitfall") == 1
    # 幂等：无重复 id 分配（id 由 commit 统一盖，此处只验 kind 序列稳定）
    assert [f for f, _ in distill(d, 1, 0, exact, tok2rec, key_toks, 0, 1400)] == kinds
    # int 型标量必须取到（pubmed_refs[].year 是 int；用 sget 会静默丢年份）
    assert snum({"year": 2014}, "year") == "2014"
    assert snum({"year": "2014"}, "year") == "2014"
    assert snum({"year": None}, "year") == ""
    assert snum({"year": True}, "year") == "", "bool 不算年份"
    assert snum({}, "year") == ""
    assert sget({"year": 2014}, "year") == "", "sget 只收 str（对照）"
    assert "PMID 24577245 (2014)" in by["literature"]["content"], by["literature"]["content"]
    assert "(?)" not in by["literature"]["content"]
    # 病名解析：精确 + 模糊
    assert resolve("  Tetralogy   of Fallot ", exact, tok2rec, key_toks) == 99
    assert resolve("Tetralogy of Fallot (TOF)", exact, tok2rec, key_toks) == 99
    assert resolve("Vitamin D deficiency", exact, tok2rec, key_toks) is None
    assert resolve("", exact, tok2rec, key_toks) is None
    print("selftest ok: facets/domains/types/flatten/[synthetic]/system-drop/"
          "cross-edge/conf-cap/count-parity/cap/int-year/resolve")
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
