#!/usr/bin/env python3
"""nt_ascend_engine — 升维引擎：任务解决记录 → 逻辑+方法论记忆，自动入茧.

对齐库内正典：
  - `AutoCrystallizer` 反幻觉门：无证据结晶标记【未验证】+ 置信度封顶 0.4，
    不静默丢弃（mirror hallucination_bin 可审计精神）。
  - 68G 库 `evolution_records` 范式：pattern_type + before→after + effectiveness。
  - 落点：memory_type Solution（方法论）/ Causal（逻辑）/ Lesson（教训），
    经 digest --rich 灌 `cocoon-ascended-*`（domain=ascended）。

自动触发约定：任何 agent 解决任务后往 `datasets/tasks_done/<id>.json` 丢一条
记录（task/actions/result/reward/evidence/failures），本引擎 `--scan` 消费
未处理记录（`_processed.json` 幂等），生成并追加到 ascended.jsonl。
把 `--scan` 接 daemon tick 即成"解决即升维"闭环。

用法:
  python3 scripts/ops/nt_ascend_engine.py --scan
  python3 scripts/ops/nt_ascend_engine.py --selftest
"""
import argparse
import glob
import json
import os
import re
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from nt_foundation_laws import live_mid_set  # noqa: E402  # 活库 M-id 全集

TASK_DIR = os.path.join("datasets", "tasks_done")
PROCESSED = os.path.join(TASK_DIR, "_processed.json")
OUT_DIR = os.path.join("datasets", "hf_distilled")
ASCENDED = os.path.join(OUT_DIR, "ascended.jsonl")

MID_RE = re.compile(r"^M-\d{6}$")
_KNOWN = None


def known_mids():
    """活库 M-id 全集（进程内缓存；--scan 逐条消费，扫一次即可）。"""
    global _KNOWN
    if _KNOWN is None:
        _KNOWN = live_mid_set()
    return _KNOWN


def check_evidence(evidence, known=None):
    """形如 M-###### 的证据必须在活库存在，否则即为悬空边 → 中止。

    2026-09-28：本引擎把 task 记录的 evidence 原样写进 connections，从不
    自己造 id；但也**从不核对**——证据里若混进档案期号（旧 selftest 夹具里的
    477231 那一族），写出去就是永久悬空边，且逃过记忆级校验（每条记忆自身
    合法）。非 M- 形态的证据（URL/路径/自由文本）不校验，本就允许。
    """
    ids = [e for e in evidence if MID_RE.match(e)]
    if not ids:
        return
    pool = known if known is not None else known_mids()
    dead = sorted({i for i in ids if i not in pool})
    if dead:
        raise SystemExit(
            "ABORT: 证据引用了活库不存在的 M-id %s —— 写进 connections 即永久"
            "悬空边（该号段属档案期，活库只有 M-000001.. 的连续号）。请改引"
            "真实 id 或改为非 M- 形态的出处。" % "、".join(dead))


def load_processed():
    try:
        with open(PROCESSED, encoding="utf-8") as fh:
            return set(json.load(fh))
    except (OSError, ValueError):
        return set()


def save_processed(done):
    with open(PROCESSED, "w", encoding="utf-8") as fh:
        json.dump(sorted(done), fh, ensure_ascii=False, indent=1)


def gate(evidence, conf):
    """反幻觉门：无证据→【未验证】+ 0.4 封顶。"""
    if evidence:
        return "", max(0.0, min(1.0, conf))
    return "【未验证】", min(0.4, conf)


def ascend(task, known=None):
    tid = task.get("task_id", "?")
    t = task.get("task", "")
    acts = task.get("actions", [])
    res = task.get("result", "")
    reward = float(task.get("reward", 0.5))
    ev = [e for e in task.get("evidence", []) if isinstance(e, str)][:6]
    check_evidence(ev, known)
    out = []
    tag, conf = gate(ev, 0.55 + 0.3 * reward)
    out.append({
        "url": "ascend://%s/solution" % tid,
        "title": "%s方法论·%s" % (tag, t[:40]),
        "content": "%s【方法论·%s】问题：%s→做法：%s→结果：%s｜复用点：%s" % (
            tag, t[:30], t[:80], "→".join(acts)[:220], res[:120],
            task.get("reusable", "同构任务直套")),
        "domain": "ascended", "memory_type": "Solution",
        "confidence": round(conf, 2), "connections": ev})
    if acts and res:
        tag2, conf2 = gate(ev, 0.5 + 0.3 * reward)
        out.append({
            "url": "ascend://%s/causal" % tid,
            "title": "%s逻辑·%s" % (tag2, t[:40]),
            "content": "%s【逻辑·若则】若%s→则%s→故%s" % (
                tag2, acts[0][:80], res[:80],
                task.get("so", "此路径可复用")),
            "domain": "ascended", "memory_type": "Causal",
            "confidence": round(conf2, 2), "connections": ev})
    for f in task.get("failures", []):
        fev = [e for e in f.get("evidence", []) if isinstance(e, str)][:4] or ev
        check_evidence(fev, known)   # lesson 的 connections 同样要过闸
        tag3, conf3 = gate(fev, 0.7)
        out.append({
            "url": "ascend://%s/lesson" % tid,
            "title": "%s教训·%s" % (tag3, f.get("desc", "")[:40]),
            "content": "%s【教训·%s】因：%s→改：%s" % (
                tag3, f.get("desc", "")[:60], f.get("cause", "")[:100],
                f.get("fix", "")[:100]),
            "domain": "ascended", "memory_type": "Lesson",
            "confidence": round(conf3, 2), "connections": fev})
    return out


def selftest():
    # 夹具里的 evidence 用**活库真实 id**（M-065651 = hf-rows 末条灌茧记忆）。
    # 原写 M-477231：那是档案期号，活库从不存在，夹具本身就在演示「写悬空边」。
    # 注入 known 集合 → 自检不依赖活库，也顺带把「引擎不造 id」钉死。
    known = {"M-065651"}
    t = {"task_id": "t1", "task": "灌茧", "actions": ["备份", "续号", "splice"],
         "result": "2031/2031", "reward": 1.0, "evidence": ["M-065651"],
         "failures": [{"desc": "500", "cause": "后端坏", "fix": "改天重跑"}],
         "so": "大JSON增量改通用"}
    rs = ascend(t, known)
    assert len(rs) == 3, rs
    assert rs[0]["memory_type"] == "Solution" and rs[0]["confidence"] == 0.85
    assert rs[1]["memory_type"] == "Causal"
    assert rs[2]["memory_type"] == "Lesson" and rs[2]["confidence"] == 0.7
    t2 = dict(t, evidence=[])
    assert ascend(t2, known)[0]["confidence"] <= 0.4
    assert ascend(t2, known)[0]["title"].startswith("【未验证】")
    # ── 悬空边防护（2026-09-28）──────────────────────────────────────────
    # ① 引擎只透传 evidence，绝不自造 id：connections ⊆ 输入 evidence
    for r in rs:
        assert set(r["connections"]) <= set(t["evidence"]), r
    # ② 形如 M-###### 但活库没有的证据必须中止（含 failures 里的）
    #    探针写成 "M-%06d" % n 而非字面量：nt_graph_audit 的源码扫描只匹
    #    配 M-\d{6}，把「必须被拒绝的反例」也报成命中，告警就被自己人淹
    #    没，真问题反而看不见（这正是 R-SCAN-1 的反面教材）。
    for bad in ("M-%06d" % 477231, "M-%06d" % 479512,
                "M-%06d" % 479531, "M-%06d" % 479942):
        try:
            ascend(dict(t, evidence=[bad]), known)
        except SystemExit:
            pass
        else:
            raise AssertionError("悬空证据 %s 未被拦下" % bad)
        try:
            ascend(dict(t, failures=[{"desc": "d", "evidence": [bad]}]), known)
        except SystemExit:
            pass
        else:
            raise AssertionError("failures 里的悬空证据 %s 未被拦下" % bad)
    # ③ 非 M- 形态的证据（URL/路径）不校验，仍直通
    assert ascend(dict(t, evidence=["http://x/y"]), known)[0]["connections"] \
        == ["http://x/y"]
    assert check_evidence([], known) is None
    print("selftest ok: solution/causal/lesson + gate, 悬空 M-id 中止")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scan", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    if not args.scan:
        ap.print_help()
        return
    os.makedirs(TASK_DIR, exist_ok=True)
    done = load_processed()
    new_rs, fresh = [], []
    for f in sorted(glob.glob(os.path.join(TASK_DIR, "*.json"))):
        if os.path.basename(f).startswith("_"):
            continue
        try:
            with open(f, encoding="utf-8") as fh:
                task = json.load(fh)
        except (OSError, ValueError):
            continue
        tid = task.get("task_id", os.path.basename(f))
        if tid in done:
            continue
        new_rs += ascend(task)
        fresh.append(tid)
        done.add(tid)
    if new_rs:
        with open(ASCENDED, "a", encoding="utf-8") as fh:
            for r in new_rs:
                fh.write(json.dumps(r, ensure_ascii=False) + "\n")
    save_processed(done)
    print("tasks=%d ascended=%d total=%s" % (len(fresh), len(new_rs), ASCENDED))


if __name__ == "__main__":
    main()
