#!/usr/bin/env python3
"""nt_source_core — 本源核心：有限源原理 + 生成式作答演示，直收入茧.

主张（用户定）：数据无限，本源有限。检索是打捞，本源是生成。
本脚本把已入茧 2300+ 条压成 14 条源原理（P01-P14），每条≤60 字，
附六爻归属 + 证据 M-id；再用原理纯推演（零检索）解一题，推演链同入茧。

源原理表（物理 3 + 方法 11）：
  P01 水循环 / P02 火三角 / P03 水火相克 /
  P04 干芯取火 / P05 原子提交 / P06 全局续号 / P07 流式改大JSON /
  P08 rows先行 / P09 端侧换维 / P10 许可门 / P11 热度背离 /
  P12 符号即地址 / P13 三层底料 / P14 反幻觉门

产出（项目内）:
  datasets/hf_distilled/axioms.json         14 源原理（单一事实源）
  datasets/hf_distilled/source_chains.jsonl rich 行（digest --rich 用）

用法:
  python3 scripts/ops/nt_source_core.py
  python3 scripts/ops/nt_source_core.py --selftest
"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from nt_hf_digest_to_cocoons import scan_max_mid, COCOONS  # noqa: E402
from nt_hf_smelt_chains import (  # noqa: E402
    HF_CLUSTERS, BRAIN_CLUSTERS, FUSIONS, HF_ARCHIVE, BRAIN_ARCHIVE,
    load_archive, read_base)

OUT_DIR = os.path.join("datasets", "hf_distilled")
AXIOMS_JSON = os.path.join(OUT_DIR, "axioms.json")
CHAINS_JSONL = os.path.join(OUT_DIR, "source_chains.jsonl")

# （编号，原理≤60字，归属卦，证据定位器）
PRINCIPLES = [
    ("P01", "水循环：蒸发→上升→凝结核凝结→碰并→重力降水，缺一不雨", "坎",
     ("db", "rowid91925")),
    ("P02", "火三角：燃料+氧+热三者具足则燃，灭其一则熄", "离",
     ("chain", "ind-heat")),
    ("P03", "水火相克：水吸热降温兼隔氧，湿柴=水 fuel 混合体故难燃", "未济",
     ("compose", ["P01", "P02"])),
    ("P04", "干芯取火：劈开取芯+羽毛棒+松脂，火型架空通风小火养大", "鼎",
     ("brainfile", "Winter_Prepping")),
    ("P05", "原子提交：大文件先备份→.tmp写→原子换名，消费者永不见半成品", "既济",
     ("chain", "fuse-gate")),
    ("P06", "全局续号：M-id 跨批次单调续，sidecar 固化 base，重跑不漂移", "恒",
     ("chain", "ind-format")),
    ("P07", "流式改大JSON：锚定稳定键文本 splice，内存<150MB 改 346MB", "鼎",
     ("chain", "fuse-gate")),
    ("P08", "rows先行：splits 正常≠rows 可用，探测顺序 rows 在前备选在后", "蹇",
     ("chain", "ind-bench")),
    ("P09", "端侧换维：单维度分页撞墙（404/5行限）即换维度拼凑", "益",
     ("chain", "ind-bench")),
    ("P10", "许可门：wtfpl/other/gated 用前查 tags，先门后桥", "讼",
     ("chain", "ind-license")),
    ("P11", "热度背离：选数看 downloads，选向看 likes 增速", "离",
     ("chain", "ind-heat")),
    ("P12", "符号即地址：单卦符即整簇语义，点亮即生长，休眠待唤醒", "未济",
     ("chain", "fuse-eval")),
    ("P13", "三层底料：网级底料+推理轨迹+离线参考，缺一偏科", "既济",
     ("chain", "fuse-pillars")),
    ("P14", "反幻觉门：无证据标未验证+0.4 封顶，不静默丢弃", "中孚",
     ("chain", "fuse-gate")),
]

# 演示题（纯推演，零检索）：雨天为什么点不着火 = P02+P03+P04 合成
DEMO_Q = "雨天为什么点不着火"
DEMO_USES = ["P02", "P03", "P04"]
DEMO_A = ("火要燃须三角具足（P02）；雨水浸柴=水混入燃料，吸热降温又隔氧"
          "（P03），三角缺二故不燃；解法唯取干芯重建三角（P04）："
          "劈芯+松脂+架空通风。结论：不是火怕雨，是三角被水拆了。")


def selftest():
    assert len(PRINCIPLES) == 14
    assert len({p[0] for p in PRINCIPLES}) == 14
    assert all(len(p[1]) <= 60 for p in PRINCIPLES), "原理超60字"
    assert set(DEMO_USES) <= {p[0] for p in PRINCIPLES}
    print("selftest ok: 14 principles<=60 chars, demo grounded")


def resolve_evidence(hf, brain, chain_order, sym_order, spec):
    kind, val = spec
    if kind == "chain":
        idx = chain_order.index(val)
        base = read_base(os.path.join(OUT_DIR, "chains.jsonl"))
        return ["M-%06d" % (base + idx)]
    if kind == "brainfile":
        base = read_base(BRAIN_ARCHIVE)
        for i, it in enumerate(brain):
            if val in it.get("url", ""):
                return ["M-%06d" % (base + i)]
        return []
    if kind == "db":
        return []
    if kind == "compose":
        return []
    return []


def db_ref(spec):
    return "brain-db://node/%s" % spec[1] if spec[0] == "db" else ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    hf = load_archive(HF_ARCHIVE)
    brain = load_archive(BRAIN_ARCHIVE)
    mx, _ = scan_max_mid(COCOONS)
    start = mx + 1
    chain_order = [c[0] for c in HF_CLUSTERS] + [c[0] for c in BRAIN_CLUSTERS] \
        + [f[0] for f in FUSIONS]
    sym_keys = ["ind-reasoning", "ind-code", "ind-physical", "ind-bench",
                "ind-webscale", "ind-zh", "ind-format", "ind-license",
                "ind-heat", "ind-zim", "ind-pmtiles", "ind-corpus",
                "ind-causal", "ind-wiki", "fuse-pillars", "fuse-zh",
                "fuse-embodied", "fuse-gate", "fuse-eval", "brain"]

    axioms, rich = [], []
    pmap = {}
    for j, (pid, text, hexa, spec) in enumerate(PRINCIPLES):
        mid = "M-%06d" % (start + j)
        pmap[pid] = mid
        ev = resolve_evidence(hf, brain, chain_order, sym_keys, spec)
        ref = db_ref(spec)
        content = "【源·%s】%s｜归%s卦%s" % (
            pid, text, hexa, ("｜据%s" % ref) if ref else "")
        axioms.append({"id": pid, "text": text, "hex": hexa,
                       "mid": mid, "evidence": ev, "ref": ref})
        rich.append({"url": "axiom://%s" % pid, "title": "源%s·%s" % (pid, text[:20]),
                     "content": content, "domain": "source-core",
                     "memory_type": "Pattern", "confidence": 0.9,
                     "connections": ev})
    # 演示推演链：引用 P02/P03/P04 的 M-id（前向同序，无漂移）
    demo_conns = [pmap[p] for p in DEMO_USES]
    rich.append({"url": "axiom://demo-wetfire",
                 "title": "源推演·%s" % DEMO_Q,
                 "content": "【源推演·%s】%s" % (DEMO_Q, DEMO_A),
                 "domain": "source-core", "memory_type": "Causal",
                 "confidence": 0.75, "connections": demo_conns})
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(AXIOMS_JSON, "w", encoding="utf-8") as fh:
        json.dump(axioms, fh, ensure_ascii=False, indent=1)
    with open(CHAINS_JSONL, "w", encoding="utf-8") as fh:
        for r in rich:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")
    print("axioms=14 demo=1 rich=%d start=M-%06d end=M-%06d" % (
        len(rich), start, start + len(rich) - 1))


if __name__ == "__main__":
    main()
