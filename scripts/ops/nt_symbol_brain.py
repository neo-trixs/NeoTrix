#!/usr/bin/env python3
"""nt_symbol_brain — 上古六爻符号脑：64 卦全图神经场 + 高维符号吸收入茧.

设计（对齐库内正典）：
  - 符号层：**64 卦全图**（文王卦序→卦符 U+4DC0 起、6-bit 上→下阳=1，
    二进制逐宫核验覆盖 0-63 全值域），口径与 `nt_core_e8::Hexagram`
    （upper<<3|lower）及 `nt_core_hex::ReasoningHexagram`
    （错卦=取反/综卦=倒序/共振=6-汉明）同构，可直连 E8 状态机。
  - 20 点亮（19 簇+脑中枢，有数据）+ 44 休眠（待后继数据唤醒）；
    休眠位仅持共振边，不进茧（茧只收有数据语义）。
  - 神经层：簇结论 M-id 为胞体，前提 connections 为树突；突触权重：
    簇内前提 1.0、汉明<=2 的 Hebb 边 0.5、融合跨簇 1.0、中枢星型 1.0；
    发放=共振×权重，阈值>=2.0 点火，3 跳演示。
  - 入茧：20 符记忆 + 1 全图总纲（64 码一览，Causal），经 digest --rich
    灌 `cocoon-neuro-symbols`（domain=neuro-symbols）。

产出（项目内）:
  datasets/hf_distilled/symbols.json      64 符号表（单一事实源）
  datasets/hf_distilled/brain.json        64 节点神经图 + 发放报告
  datasets/hf_distilled/symbol_chains.jsonl  rich 行（digest --rich 用）

用法:
  python3 scripts/ops/nt_symbol_brain.py
  python3 scripts/ops/nt_symbol_brain.py --selftest
"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from nt_hf_digest_to_cocoons import scan_max_mid, COCOONS  # noqa: E402
from nt_hf_smelt_chains import (  # noqa: E402
    HF_CLUSTERS, BRAIN_CLUSTERS, FUSIONS, HF_ARCHIVE, BRAIN_ARCHIVE,
    load_archive, ds_lines, prefix_lines, read_base)

OUT_DIR = os.path.join("datasets", "hf_distilled")
SYMBOLS_JSON = os.path.join(OUT_DIR, "symbols.json")
BRAIN_JSON = os.path.join(OUT_DIR, "brain.json")
CHAINS_JSONL = os.path.join(OUT_DIR, "symbol_chains.jsonl")

# 全表：（卦名, 文王卦序, 6-bit, 上卦, 下卦）— 按八宫，逐值核验 0-63 全覆盖
HEX64 = [
    ("乾", 1, 0b111111, "乾", "乾"), ("姤", 44, 0b111110, "乾", "巽"),
    ("遁", 33, 0b111100, "乾", "艮"), ("否", 12, 0b111000, "乾", "坤"),
    ("观", 20, 0b110000, "巽", "坤"), ("剥", 23, 0b100000, "艮", "坤"),
    ("晋", 35, 0b101000, "离", "坤"), ("大有", 14, 0b101111, "离", "乾"),
    ("坤", 2, 0b000000, "坤", "坤"), ("复", 24, 0b000001, "坤", "震"),
    ("临", 19, 0b000011, "坤", "兑"), ("泰", 11, 0b000111, "坤", "乾"),
    ("大壮", 34, 0b001111, "震", "乾"), ("夬", 43, 0b011111, "兑", "乾"),
    ("需", 5, 0b010111, "坎", "乾"), ("比", 8, 0b010000, "坎", "坤"),
    ("震", 51, 0b001001, "震", "震"), ("豫", 16, 0b001000, "震", "坤"),
    ("解", 40, 0b001010, "震", "坎"), ("恒", 32, 0b001110, "震", "巽"),
    ("升", 46, 0b000110, "坤", "巽"), ("井", 48, 0b010110, "坎", "巽"),
    ("大过", 28, 0b011110, "兑", "巽"), ("随", 17, 0b011001, "兑", "震"),
    ("巽", 57, 0b110110, "巽", "巽"), ("小畜", 9, 0b110111, "巽", "乾"),
    ("家人", 37, 0b110101, "巽", "离"), ("益", 42, 0b110001, "巽", "震"),
    ("无妄", 25, 0b111001, "乾", "震"), ("噬嗑", 21, 0b101001, "离", "震"),
    ("颐", 27, 0b100001, "艮", "震"), ("蛊", 18, 0b100110, "艮", "巽"),
    ("坎", 29, 0b010010, "坎", "坎"), ("节", 60, 0b010011, "坎", "兑"),
    ("屯", 3, 0b010001, "坎", "震"), ("既济", 63, 0b010101, "坎", "离"),
    ("革", 49, 0b011101, "兑", "离"), ("丰", 55, 0b001101, "震", "离"),
    ("明夷", 36, 0b000101, "坤", "离"), ("师", 7, 0b000010, "坤", "坎"),
    ("离", 30, 0b101101, "离", "离"), ("旅", 56, 0b101100, "离", "艮"),
    ("鼎", 50, 0b101110, "离", "巽"), ("未济", 64, 0b101010, "离", "坎"),
    ("蒙", 4, 0b100010, "艮", "坎"), ("涣", 59, 0b110010, "巽", "坎"),
    ("讼", 6, 0b111010, "乾", "坎"), ("同人", 13, 0b111101, "乾", "离"),
    ("艮", 52, 0b100100, "艮", "艮"), ("贲", 22, 0b100101, "艮", "离"),
    ("大畜", 26, 0b100111, "艮", "乾"), ("损", 41, 0b100011, "艮", "兑"),
    ("睽", 38, 0b101011, "离", "兑"), ("履", 10, 0b111011, "乾", "兑"),
    ("中孚", 61, 0b110011, "巽", "兑"), ("渐", 53, 0b110100, "巽", "艮"),
    ("兑", 58, 0b011011, "兑", "兑"), ("困", 47, 0b011010, "兑", "坎"),
    ("萃", 45, 0b011000, "兑", "坤"), ("咸", 31, 0b011100, "兑", "艮"),
    ("蹇", 39, 0b010100, "坎", "艮"), ("谦", 15, 0b000100, "坤", "艮"),
    ("小过", 62, 0b001100, "震", "艮"), ("归妹", 54, 0b001011, "震", "兑"),
]

# 点亮位：簇key → （卦名，断语）
LIT = {
    "ind-reasoning": ("乾", "天行健·推理不息"),
    "ind-code": ("鼎", "炉火炼码·成器"),
    "ind-physical": ("震", "雷动·具身行"),
    "ind-bench": ("履", "履虎尾·践标准"),
    "ind-webscale": ("坤", "地载万物·底料"),
    "ind-zh": ("同人", "同人于野·中文共同体"),
    "ind-format": ("节", "节制·格式为界"),
    "ind-license": ("讼", "有孚窒惕·许可门"),
    "ind-heat": ("离", "明两作·热度为光"),
    "ind-zim": ("大畜", "大有蓄积·离线仓"),
    "ind-pmtiles": ("观", "盥而不荐·观览地图"),
    "ind-corpus": ("井", "改邑不改井·68G深井"),
    "ind-causal": ("益", "损上益下·因果流注"),
    "ind-wiki": ("泰", "小往大来·维基通泰"),
    "fuse-pillars": ("既济", "初吉终乱·三支柱须守成"),
    "fuse-zh": ("家人", "利女贞·中文一家"),
    "fuse-embodied": ("无妄", "其匪正有眚·具身防妄动"),
    "fuse-gate": ("蹇", "利西南·门控险中求"),
    "fuse-eval": ("中孚", "豚鱼吉·评估之信"),
    "brain": ("未济", "小狐汔济·脑永未完成"),
}

# 休眠位断语（待后继数据唤醒）
DORMANT_MOTTO = {
    "屯": "勿用有攸往", "蒙": "童蒙求我", "需": "光亨贞吉", "师": "丈人吉",
    "比": "元永贞", "小畜": "密云不雨", "否": "不利君子贞", "大有": "元亨",
    "谦": "君子有终", "豫": "建侯行师", "随": "无咎", "蛊": "利涉大川",
    "临": "八月有凶", "噬嗑": "利用狱", "贲": "小利有攸往", "剥": "不利有攸往",
    "复": "出入无疾", "颐": "自求口实", "大过": "栋桡", "坎": "维心亨",
    "咸": "取女吉", "恒": "无咎利贞", "遁": "小利贞", "大壮": "利贞",
    "晋": "昼日三接", "明夷": "利艰贞", "睽": "小事吉", "解": "来复吉",
    "损": "元吉无咎", "夬": "孚号有厉", "姤": "勿用取女", "萃": "王假有庙",
    "升": "用见大人", "困": "大人吉", "革": "己日乃孚", "渐": "女归吉",
    "归妹": "征凶", "丰": "勿忧", "旅": "旅贞吉", "巽": "利见大人",
    "兑": "利贞", "涣": "利涉大川", "小过": "可小事", "艮": "不获其身",
}

TRIGRAM_UNICODE = {"乾": "☰", "兑": "☱", "离": "☲", "震": "☳",
                   "巽": "☴", "坎": "☵", "艮": "☶", "坤": "☷"}


def hexagram_char(wen):
    return chr(0x4DC0 + wen - 1)


def resonance(b1, b2):
    return 6 - bin(b1 ^ b2).count("1")


def selftest():
    assert len(HEX64) == 64
    assert {b for _, _, b, _, _ in HEX64} == set(range(64)), "全值域覆盖"
    assert len({n for n, _, _, _, _ in HEX64}) == 64
    assert hexagram_char(1) == "䷀" and hexagram_char(64) == "䷿"
    assert hexagram_char(63) == "䷾" and hexagram_char(30) == "䷝"
    assert resonance(0b111111, 0b111111) == 6
    assert resonance(0b111111, 0b000000) == 0
    lit_names = {v[0] for v in LIT.values()}
    assert len(lit_names) == 20, lit_names
    hex_names = {n for n, _, _, _, _ in HEX64}
    assert lit_names <= hex_names
    assert set(DORMANT_MOTTO) == hex_names - lit_names, "休眠44"
    print("selftest ok: 64全表值域+unicode+20点亮+44休眠+共振")


def build():
    hf = load_archive(HF_ARCHIVE)
    brain_arch = load_archive(BRAIN_ARCHIVE)
    mx, _ = scan_max_mid(COCOONS)
    start = mx + 1
    hf_base = read_base(HF_ARCHIVE)
    brain_base = read_base(BRAIN_ARCHIVE)
    chain_base = read_base(os.path.join(OUT_DIR, "chains.jsonl"))

    clu_conns, clu_mid = {}, {}
    order = [c[0] for c in HF_CLUSTERS] + [c[0] for c in BRAIN_CLUSTERS] \
        + [f[0] for f in FUSIONS]
    for idx, key in enumerate(order):
        clu_mid[key] = "M-%06d" % (chain_base + idx)
    for key, _, _, _, dsids in HF_CLUSTERS:
        conns = []
        for ds in dsids:
            conns += ["M-%06d" % (hf_base + i) for i in ds_lines(hf, ds)]
        clu_conns[key] = conns[:20]
    for key, _, _, _, prefix, n in BRAIN_CLUSTERS:
        clu_conns[key] = ["M-%06d" % (brain_base + i)
                          for i in prefix_lines(brain_arch, prefix, n)]
    fuse_refs = {f[0]: f[4] for f in FUSIONS}
    for key, refs in fuse_refs.items():
        clu_conns[key] = [clu_mid[r] for r in refs if r in clu_mid]

    hex_by_name = {n: (w, b, u, lo) for n, w, b, u, lo in HEX64}
    lit_by_hex = {v[0]: k for k, v in LIT.items()}
    neurons, rich = [], []
    for j, (ckey, (hname, motto)) in enumerate(LIT.items()):
        wen, bits, upper, lower = hex_by_name[hname]
        mid = "M-%06d" % (start + j)
        members = clu_conns.get(ckey, []) if ckey != "brain" else []
        syn = [{"to": m, "w": 1.0, "kind": "premise"} for m in members]
        for oname, (ow, ob, _, _) in hex_by_name.items():
            if oname != hname and bin(bits ^ ob).count("1") <= 2:
                syn.append({"to": oname, "w": 0.5, "kind": "hebb"})
        if ckey in fuse_refs:
            for r in fuse_refs[ckey]:
                syn.append({"to": lit_by_hex_inv(r, lit_by_hex, hex_by_name),
                            "w": 1.0, "kind": "fusion"})
        neurons.append({"key": ckey, "mid": mid, "hex": hname,
                        "symbol": hexagram_char(wen), "wen": wen,
                        "bits": format(bits, "06b"),
                        "tri": TRIGRAM_UNICODE[upper] + TRIGRAM_UNICODE[lower],
                        "motto": motto, "status": "lit",
                        "members": members, "synapses": syn,
                        "activation": round(min(1.0, 0.3 + 0.05 * len(members)), 3)
                        if ckey != "brain" else 1.0})
        conn_mids = list(members[:12])
        if ckey in fuse_refs:
            conn_mids += [clu_mid[r] for r in fuse_refs[ckey] if r in clu_mid]
        mtype = "Causal" if ckey.startswith("fuse-") or ckey == "ind-license" \
            else "Pattern"
        rich.append({
            "url": "hex://%s" % ckey,
            "title": "%s%s·%s" % (hexagram_char(wen), hname, motto),
            "content": "%s%s%s%s %s %s上%s下｜%s｜簇%s" % (
                hexagram_char(wen), hname, TRIGRAM_UNICODE[upper],
                TRIGRAM_UNICODE[lower], format(bits, "06b"), upper, lower,
                motto, clu_mid.get(ckey, "中枢")),
            "domain": "neuro-symbols", "memory_type": mtype,
            "confidence": 0.78 if ckey == "brain" else 0.72,
            "connections": conn_mids})
    # 休眠 44：仅共振边，不进茧
    for hname in sorted(hex_by_name, key=lambda n: hex_by_name[n][1]):
        if hname in lit_by_hex:
            continue
        wen, bits, upper, lower = hex_by_name[hname]
        syn = [{"to": on, "w": 0.5, "kind": "hebb"}
               for on, (ow, ob, _, _) in hex_by_name.items()
               if on != hname and bin(bits ^ ob).count("1") <= 2]
        neurons.append({"key": "dormant-%s" % hname, "mid": None,
                        "hex": hname, "symbol": hexagram_char(wen),
                        "wen": wen, "bits": format(bits, "06b"),
                        "tri": TRIGRAM_UNICODE[upper] + TRIGRAM_UNICODE[lower],
                        "motto": DORMANT_MOTTO[hname], "status": "dormant",
                        "members": [], "synapses": syn, "activation": 0.0})
    # brain 中枢连 19 符
    neurons[19]["synapses"] += [{"to": n["hex"], "w": 1.0, "kind": "hub"}
                                for n in neurons[:19]]
    lit_mids = [n["mid"] for n in neurons[:20]]
    # 全图总纲记忆：64 码一览
    roster = " ".join("%s%s" % (n, format(hex_by_name[n][1], "06b"))
                      for n, _, _, _, _ in HEX64)
    rich.append({"url": "hex://atlas",
                 "title": "64卦全图总纲·20点亮44休眠",
                 "content": "64卦全图（名+6bit上→下）：%s。点亮20（有数据簇）休眠44待唤醒。"
                            % roster,
                 "domain": "neuro-symbols", "memory_type": "Causal",
                 "confidence": 0.74,
                 "connections": lit_mids})
    brain_others = [n["mid"] for n in neurons[:19]]
    rich[19]["connections"] = brain_others  # brain 主符指 19 符（不自指）

    # 发放演示：乾为种，64 场 3 跳
    bmap = {n["hex"]: n for n in neurons}
    key_of = {}
    for n in neurons:
        key_of[n["key"]] = n["hex"]
        key_of[n["hex"]] = n["hex"]
    fired, frontier, seen = [], ["乾"], set()
    bbits = {n["hex"]: int(n["bits"], 2) for n in neurons}
    for _ in range(3):
        nxt = []
        for hx in frontier:
            if hx in seen:
                continue
            seen.add(hx)
            fired.append(hx)
            for s in bmap[hx]["synapses"]:
                tgt = s["to"]
                thex = key_of.get(tgt, tgt)
                if thex in bmap and thex not in seen:
                    r = resonance(bbits[hx], bbits[thex])
                    if r * s["w"] >= 2.0 and thex not in nxt:
                        nxt.append(thex)
        frontier = nxt
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(SYMBOLS_JSON, "w", encoding="utf-8") as fh:
        json.dump([{"hex": n, "symbol": hexagram_char(w),
                    "wen": w, "bits": format(b, "06b"),
                    "upper": u, "lower": lo,
                    "status": "lit" if n in lit_by_hex else "dormant",
                    "cluster": lit_by_hex.get(n),
                    "motto": dict(LIT.values()).get(n, DORMANT_MOTTO.get(n))}
                   for n, w, b, u, lo in HEX64],
                  fh, ensure_ascii=False, indent=1)
    with open(BRAIN_JSON, "w", encoding="utf-8") as fh:
        json.dump({"neurons": neurons,
                   "firing_demo": {"seed": "乾", "hops": 3, "fired": fired,
                                   "fired_count": len(fired)}},
                  fh, ensure_ascii=False, indent=1)
    with open(CHAINS_JSONL, "w", encoding="utf-8") as fh:
        for r in rich:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")
    print("symbols=64 lit=20 dormant=44 firing_n=%d" % len(fired))
    print("firing=%s" % fired[:24])
    print("rich=%d start=M-%06d end=M-%06d" % (
        len(rich), start, start + len(rich) - 1))
    return start


def lit_by_hex_inv(r, lit_by_hex, hex_by_name):
    for hx, ck in lit_by_hex.items():
        if ck == r:
            return hx
    return r


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    build()


if __name__ == "__main__":
    main()
