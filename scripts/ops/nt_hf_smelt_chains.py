#!/usr/bin/env python3
"""nt_hf_smelt_chains — 2262 条茧记忆升维熔炼 → 高密度推理链记忆.

输入（项目内档案，不碰 346MB 茧）:
  datasets/hf_rows/crawl_queue.jsonl（2031 行）
  datasets/neotrixbrain/crawl_queue.jsonl（231 行）
  两者的行号→M-id 基线**只从各自旁的 `<archive>.mids.json` 读**（该文件
  由生成该 archive 的吸收步骤落盘）；缺失即报错退出。原 docstring 在此处
  写死「477231 起 / 479262 起」—— 那是档案期号段，活库
  M-000001..M-065651 里从不存在，照着续号必然产出悬空边，故不再宣传。
输出 datasets/hf_distilled/chains.jsonl（rich 行：memory_type/connections/confidence
直给，经 digest --rich 灌茧，domain=hf-distilled）。

两层链路：
  L1 归纳：簇内多前提 → 一条结论（connections=各集前3行 M-id），memory_type=Pattern
  L2 融合：跨簇结论互指 + 原始前提 → 更高阶结论，memory_type=Pattern（置信 0.65）
  （Rust 侧 ReasoningChain 只活内存，茧内链路以 connections 数组实现；
   融合引用 L1 结论 M-id，前向编号=当前全局max+1+行序，digest 同序分配）

  2026-09-28 修正：L2 原写 memory_type="CrossDomain"，那是 **ReasoningType**
  变体而非 MemoryType（consciousness.rs:67-82 vs :39-56），写进 cocoons.json
  后使 `CocoonStore::load()` 全有全无解析失败 → 整库静默变空，且
  `nt_train_export --ingest` 的 load→save 覆盖清空。已改 Pattern 并在
  emit() 加白名单闸（见 MEMORY_TYPES）。

用法:
  python3 scripts/ops/nt_hf_smelt_chains.py
  python3 scripts/ops/nt_hf_smelt_chains.py --selftest
"""
import argparse
import json
import os
import re
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from nt_hf_digest_to_cocoons import scan_max_mid, COCOONS  # noqa: E402

# MemoryType 合法变体（权威：`nt_crystal_core/consciousness.rs:39-56`）
MEMORY_TYPES = {"Fact", "Pattern", "Causal", "Contradiction",
                "Counterfactual", "Experience", "Lesson", "Solution"}

HF_ARCHIVE = os.path.join("datasets", "hf_rows", "crawl_queue.jsonl")
BRAIN_ARCHIVE = os.path.join("datasets", "neotrixbrain", "crawl_queue.jsonl")
OUT_DIR = os.path.join("datasets", "hf_distilled")
OUT = os.path.join(OUT_DIR, "chains.jsonl")

HF_CLUSTERS = [
    ("ind-reasoning", "Pattern", "归纳·推理轨迹簇",
     "2026广场主线=显式CoT：Fable150M/Spark234K/Code452M/kimi997×13/ZGCM/RL-L3；"
     "桥thinking直收，pilot增补3。前提6集。",
     ["MoreThought/Fable-5.1-Max-Reasoning-Filtered-5000x", "OpenDataArena/Spark-234K",
      "IFM/Code-Reasoning", "echel0nn1881/kimi-cyber-reasoning",
      "zgcagi/ZGCM-1-Data", "openbmb/UltraData-RL-2609"]),
    ("ind-code", "Pattern", "归纳·代码簇",
     "代码三源：UltraData-Code（L0-L4体系）/Code-Reasoning452M/Fable-coding-traces；"
     "parquet+apache-2.0，en+zh。直灌无碍。",
     ["openbmb/UltraData-Code", "IFM/Code-Reasoning",
      "MoreThought/Fable-5.1-Max-Reasoning-Filtered-5000x"]),
    ("ind-physical", "Pattern", "归纳·物理世界簇",
     "具身四源：Yootta-SimReady/eidon-POV+IMU/nvidia-AV/CAD-1000h；"
     "video/3d/timeseries超桥二元映射，目录先行+映射扩展。",
     ["Yootta/World-SimReady-Home", "eidon-ai/tracker-pov",
      "nvidia/PhysicalAI-Autonomous-Vehicles", "markov-ai/cad-1000-hours"]),
    ("ind-bench", "Pattern", "归纳·基准锚簇",
     "评估三锚：glue理解/gsm8k数学/tmmluplus繁中（+typed/Open-Jev）；"
     "基准不训只测，晶体自验证候选。",
     ["nyu-mll/glue", "openai/gsm8k", "ikala/tmmluplus",
      "LocalLLaMA/typed-decisions", "ZefanCai/Open-Jev"]),
    ("ind-webscale", "Pattern", "归纳·网级底料簇",
     "B级底料：fineweb18.5T/wikipedia全语言/arxiv3.1M/TxT360-1.84B；"
     "预训练底，不直灌全文，持索引。",
     ["HuggingFaceFW/fineweb", "wikimedia/wikipedia",
      "secemp9/arxiv-complete", "IFM/TxT360-v2"]),
    ("ind-zh", "Pattern", "归纳·中文位簇",
     "中文三件套：UltraData-SFT-Agent/Code/RL（apache-2.0，en+zh）+TMMLU+繁+ZGCM；"
     "中文指令闭环用地内数据。",
     ["openbmb/UltraData-SFT-Agent-2609", "openbmb/UltraData-Code",
      "openbmb/UltraData-RL-2609", "ikala/tmmluplus", "zgcagi/ZGCM-1-Data",
      "Harland/OmniVChat", "ZefanCai/Open-Jev"]),
    ("ind-format", "Pattern", "归纳·格式事实",
     "parquet主导≈15/30，json次≈7/30，csv/imagefolder长尾；"
     "rows接口取数无碍，imagefolder/video行需映射扩展。",
     ["ILSVRC/imagenet-1k", "malcolmrey/various",
      "DeepMostInnovations/saas-sales-conversations", "ikala/tmmluplus"]),
    ("ind-license", "Causal", "因果·许可门",
     "可用apache/mit/cc系；wtfpl×2+other×7+空×2用前必查tags；"
     "gated401七集待token。若跳过此门→污染可商用链路。",
     ["echel0nn1881/kimi-cyber-reasoning", "malcolmrey/various",
      "nyu-mll/glue", "Yootta/World-SimReady-Home"]),
    ("ind-heat", "Pattern", "归纳·热度背离",
     "likes≠downloads：gsm8k1.7k→1.23M/glue1.1k→874k；选数看downloads，"
     "选向看likes增速。trending页=增速快照非总量榜。",
     ["openai/gsm8k", "nyu-mll/glue", "HuggingFaceFW/fineweb"]),
]

BRAIN_CLUSTERS = [
    ("ind-zim", "Pattern", "归纳·离线ZIM版图",
     "61离线ZIM85G：TED17.9G/iFixit3.5G/Gutenberg/LibreTexts/医学/备灾全覆盖；"
     "断网可读，晶体持编目+file指针。",
     "zim/", 3),
    ("ind-pmtiles", "Pattern", "归纳·地图瓦片50州",
     "50州protomaps瓦片（加州1.1G最大）；地理底座，与geo_index183k互引。",
     "pmtiles/", 3),
    ("ind-corpus", "Pattern", "归纳·68G库三围",
     "22.27M nodes（FTS完备）/5.03M kv/429k emb/452k crawl/49技能/1394关键词；"
     "向量BLOB不进茧，SQL ro-uri直查。",
     "brain-db://table/", 3),
    ("ind-causal", "Pattern", "归纳·因果图53节点",
     "健康remedy图：53节点100边；小体量全量入茧，可直接做症状→调理推理。",
     "causal_graph.json#node", 3),
    ("ind-wiki", "Pattern", "归纳·维基ZIM收齐",
     "all_mini12.5G+top_nopic2.2G+top_mini331M，dl_status=COMPLETE；"
     "英文维基离线底定稿。",
     "wikipedia/", 3),
]

FUSIONS = [
    ("fuse-pillars", "Pattern", "融合·三层底料论",
     "晶体数据三支柱：网级底料（ind-webscale）+推理轨迹（ind-reasoning）+"
     "离线参考（ind-zim/ind-corpus）。缺任一→偏科。",
     ["ind-webscale", "ind-reasoning", "ind-zim", "ind-corpus"]),
    ("fuse-zh", "Pattern", "融合·中文闭环",
     "中文指令闭环=UltraData三件套（ind-zh）+繁中锚（ind-bench）+"
     "离线中文可查（ind-wiki）；三源互备，不另起炉灶。",
     ["ind-zh", "ind-bench", "ind-wiki"]),
    ("fuse-embodied", "Pattern", "融合·具身缺口",
     "具身数据（ind-physical）超文本桥+E2E视频行（tracker统计行已入）;"
     "地图瓦片（ind-pmtiles）+geo库=空间侧拼图。行级映射是下一刀。",
     ["ind-physical", "ind-pmtiles", "ind-corpus"]),
    ("fuse-gate", "Causal", "融合·门控总则",
     "许可门（ind-license）×全量不可穷尽（106万集分页+端侧5行限）："
     "先查tags再入桥，先种子再长尾。若反之→返工。",
     ["ind-license", "ind-heat", "ind-format"]),
    ("fuse-eval", "Pattern", "融合·评估闭环",
     "三锚（ind-bench）+健康分（corpus health_reports）+技能49（ind-corpus）="
     "晶体自验证最小闭环：测→评→技能迭代。",
     ["ind-bench", "ind-corpus", "ind-causal"]),
]


def load_archive(path):
    with open(path, encoding="utf-8") as fh:
        return [json.loads(ln) for ln in fh if ln.strip()]


def read_base(archive):
    """取 archive 记录的 M-id 基线。

    2026-09-28 修正：原实现 `.mids.json` 缺失时**静默回退硬编码 477231/479262**。
    那个高位空间在活库里从不存在（活库用的是 M-000001.. 连续低空间），
    于是产出的 767 条 connections **永久悬空** —— 且因 `.mids.json` 已不在，
    回退成了唯一路径，等于每次跑都必坏。

    改为：`.mids.json` 缺失时**不再编造 id**，而是报错退出。
    编号必须来自活库真实 max（`scan_max_mid(COCOONS)`），不能由脚本臆造。
    """
    try:
        with open(archive + ".mids.json", encoding="utf-8") as fh:
            return int(json.load(fh)["base"])
    except (OSError, ValueError, KeyError) as e:
        raise SystemExit(
            "ABORT: %s.mids.json 缺失/不可解析（%s）—— 该文件记录了此 archive 的 "
            "M-id 基线。缺失时**不得回退硬编码 id**：历史硬编码 477231 造成 767 条"
            "永久悬空连边。请先重跑生成该 archive 的吸收步骤以重建 .mids.json。"
            % (archive, type(e).__name__))


def ds_lines(items, ds_id, n=3):
    """某数据集前 n 行的行号（→M-id 换算基）。"""
    out = []
    for i, it in enumerate(items):
        if "hf-dataset://%s#" % ds_id in it.get("url", ""):
            out.append(i)
            if len(out) >= n:
                break
    return out


def prefix_lines(items, prefix, n=3):
    out = []
    for i, it in enumerate(items):
        if prefix in it.get("url", "") or prefix in it.get("title", ""):
            out.append(i)
            if len(out) >= n:
                break
    return out


def _selftest_signature_drift():
    """导入方签名漂移闸：2026-09-28 我把 read_base 的 fallback 参数删掉，
    却没检查 3 个导入方 → nt_symbol_brain.build() / nt_source_core.build()
    抛 TypeError，而它们的 --selftest 不走 build()，**测不出来**。
    故 read_base 的 owner 必须自己守：AST 扫全部调用点，参数数不等于 1 即失败。
    """
    import ast
    import glob
    here = os.path.dirname(os.path.abspath(__file__))
    bad = []
    for f in sorted(glob.glob(os.path.join(here, "*.py"))):
        try:
            tree = ast.parse(open(f, encoding="utf-8").read())
        except (OSError, SyntaxError) as e:
            bad.append("%s: 无法解析 %s" % (os.path.basename(f), e))
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Call) and getattr(node.func, "id", None) == "read_base":
                if len(node.args) != 1:
                    bad.append("%s:%d read_base 收到 %d 个参数（本版本只接受 1 个）"
                               % (os.path.basename(f), node.lineno, len(node.args)))
    assert not bad, "read_base 签名漂移:\n  " + "\n  ".join(bad)
    return True


def selftest():
    assert len(HF_CLUSTERS) == 9 and len(BRAIN_CLUSTERS) == 5 and len(FUSIONS) == 5
    items = [{"url": "hf-dataset://o/n#row0"}, {"url": "hf-dataset://o/n#row1"},
             {"url": "hf-dataset://x/y#row0"}]
    assert ds_lines(items, "o/n", 3) == [0, 1]
    assert prefix_lines(items, "x/y", 3) == [2]
    # ── 悬空边防护（2026-09-28）──────────────────────────────────────────
    # ① read_base 必须 fail-loudly：缺 .mids.json 即 SystemExit，绝不编 id
    #    （原实现静默回退硬编码 477231/479262，产出 767 条永久悬空连边）。
    with tempfile.TemporaryDirectory() as td:
        missing = os.path.join(td, "crawl_queue.jsonl")
        try:
            read_base(missing)
        except SystemExit:
            pass
        else:
            raise AssertionError("缺 .mids.json 未中止（会退回硬编码基址）")
        broken = os.path.join(td, "b.jsonl")
        with open(broken + ".mids.json", "w", encoding="utf-8") as fh:
            fh.write("{not json")
        try:
            read_base(broken)
        except SystemExit:
            pass
        else:
            raise AssertionError("坏 .mids.json 未中止")
        good = os.path.join(td, "g.jsonl")
        with open(good + ".mids.json", "w", encoding="utf-8") as fh:
            json.dump({"base": 2300}, fh)
        assert read_base(good) == 2300
    # ② 静态闸：本文件源码里不得再出现档案期号段的 M-id 字面量
    #    （scan_max_mid 那条 `M-%06d` 是格式化模板，不匹配）。
    src = open(os.path.abspath(__file__), encoding="utf-8").read()
    stale = sorted(set(re.findall(r"M-(?:47|48)\d{4}", src)))
    assert not stale, "源码又出现档案期 M-id: %s" % stale
    # ③ 融合前提缺失必须中止（原来静默少连边）
    keys = {c[0] for c in HF_CLUSTERS} | {c[0] for c in BRAIN_CLUSTERS}
    for _k, _mt, _t, _c, refs in FUSIONS:
        assert set(refs) <= keys, "FUSIONS 引用了不存在的簇 %s" % (
            set(refs) - keys)
    _selftest_signature_drift()
    assert len(HF_CLUSTERS) == 9 and len(BRAIN_CLUSTERS) == 5 and len(FUSIONS) == 5
    items = [{"url": "hf-dataset://o/n#row0"}, {"url": "hf-dataset://o/n#row1"},
             {"url": "hf-dataset://x/y#row0"}]
    assert ds_lines(items, "o/n", 3) == [0, 1]
    assert prefix_lines(items, "x/y", 3) == [2]
    # ── 悬空边防护（2026-09-28）──────────────────────────────────────────
    # ① read_base 必须 fail-loudly：缺 .mids.json 即 SystemExit，绝不编 id
    #    （原实现静默回退硬编码 477231/479262，产出 767 条永久悬空连边）。
    with tempfile.TemporaryDirectory() as td:
        missing = os.path.join(td, "crawl_queue.jsonl")
        try:
            read_base(missing)
        except SystemExit:
            pass
        else:
            raise AssertionError("缺 .mids.json 未中止（会退回硬编码基址）")
        broken = os.path.join(td, "b.jsonl")
        with open(broken + ".mids.json", "w", encoding="utf-8") as fh:
            fh.write("{not json")
        try:
            read_base(broken)
        except SystemExit:
            pass
        else:
            raise AssertionError("坏 .mids.json 未中止")
        good = os.path.join(td, "g.jsonl")
        with open(good + ".mids.json", "w", encoding="utf-8") as fh:
            json.dump({"base": 2300}, fh)
        assert read_base(good) == 2300
    # ② 静态闸：本文件源码里不得再出现档案期号段的 M-id 字面量
    #    （scan_max_mid 那条 `M-%06d` 是格式化模板，不匹配）。
    src = open(os.path.abspath(__file__), encoding="utf-8").read()
    stale = sorted(set(re.findall(r"M-(?:47|48)\d{4}", src)))
    assert not stale, "源码又出现档案期 M-id: %s" % stale
    # ③ 融合前提缺失必须中止（原来静默少连边）
    keys = {c[0] for c in HF_CLUSTERS} | {c[0] for c in BRAIN_CLUSTERS}
    for _k, _mt, _t, _c, refs in FUSIONS:
        assert set(refs) <= keys, "FUSIONS 引用了不存在的簇 %s" % (
            set(refs) - keys)
    print("selftest ok: 9+5+5 chains, premise index, 基线只从 .mids.json 读 + 签名漂移闸")


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
    print("global max M-id baked start: M-%06d" % start)

    rich, mids = [], {}
    def emit(key, mtype, title, content, conns, conf):
        # 2026-09-28 地雷防护：memory_type 必须是 MemoryType 的合法变体。
        # 曾把 ReasoningType 的 "CrossDomain" 当 MemoryType 写进 cocoons.json，
        # 而 `CocoonStore::load()` 对 from_str 全有全无（cocoons.rs:85-91）→
        # 整库静默变空，且 `nt_train_export --ingest` 的 load→save 会覆盖清空。
        # 跨域融合结论的合法记忆类型是 Pattern（= consciousness.rs:42「从推理产生」）。
        if mtype not in MEMORY_TYPES:
            raise SystemExit(
                "ABORT: 非法 memory_type %r（key=%s）—— MemoryType 只有 %s。"
                "推理类型名（Deductive/Inductive/Abductive/Analogical/CrossDomain）"
                "不是记忆类型。" % (mtype, key, sorted(MEMORY_TYPES)))
        mid = "M-%06d" % (start + len(rich))
        mids[key] = mid
        rich.append({"url": "hf-chain://%s" % key, "title": title,
                     "content": content, "domain": "hf-distilled",
                     "memory_type": mtype, "confidence": conf,
                     "connections": conns})
    HF_BASE = read_base(HF_ARCHIVE)
    for key, mtype, title, content, dsids in HF_CLUSTERS:
        conns = []
        for ds in dsids:
            conns += ["M-%06d" % (HF_BASE + i) for i in ds_lines(hf, ds)]
        emit(key, mtype, "【%s】" % title, "【%s】%s" % (title, content),
             conns[:20], 0.7)
    BRAIN_BASE = read_base(BRAIN_ARCHIVE)
    for key, mtype, title, content, prefix, n in BRAIN_CLUSTERS:
        conns = ["M-%06d" % (BRAIN_BASE + i) for i in prefix_lines(brain, prefix, n)]
        emit(key, mtype, "【%s】" % title, "【%s】%s" % (title, content),
             conns, 0.7)
    for key, mtype, title, content, refs in FUSIONS:
        # 2026-09-28：原为 `[mids[r] for r in refs if r in mids]`，缺前提就
        # 静默少一条连边 —— 融合结论照样写出去，链路缺口无声无息。改为中止：
        # 前提缺失是簇表与 FUSIONS 不一致，属代码错，不是数据稀疏。
        missing = [r for r in refs if r not in mids]
        if missing:
            raise SystemExit(
                "ABORT: 融合 %s 引用了未产出的簇 %s —— 前提缺失时静默少连边会"
                "让链路缺口不可见（历史悬空边就是这么烂在暗处的）。"
                % (key, missing))
        conns = [mids[r] for r in refs]
        # 融合结论跨簇推导、间接支撑更多，置信度低于 L1 直归纳（0.7 → 0.65）。
        # 旧口径按 mtype=="CrossDomain" 分支，现类型已改 Pattern，改为按层判定。
        emit(key, mtype, "【%s】" % title, "【%s】%s" % (title, content),
             conns, 0.65)
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        for r in rich:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")
    print("chains=%d (L1=%d L2=%d) start=M-%06d end=M-%06d" % (
        len(rich), len(HF_CLUSTERS) + len(BRAIN_CLUSTERS), len(FUSIONS),
        start, start + len(rich) - 1))


if __name__ == "__main__":
    main()
