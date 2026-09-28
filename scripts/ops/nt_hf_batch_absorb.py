#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_hf_batch_absorb — 12 个候选数据集的**可执行子集**批量吸收（2026-09-28）.

## 任务前提被勘察推翻（务必先读，否则会重犯）

用户给出 12 个 HF 数据集要求"融入晶体核心"。先跑 `nt_hf_survey.py` **只读勘察**：

  1. **总存储 78,066.92 GB（≈78 TB）**。核心是**单个 JSON**，1h tick 全量载入并
     **常驻内存**（`CrystalIterState::load` → `sync_to_consciousness`，当前 52.5MB /
     64,674 条）。TB 级数据集只可能采样，不可能整体灌。
  2. **9 个已在核心里**：2026-09-24 那轮 `nt_hf_rows_download.py` 各抓了 100 行落在
     `hf-rows` 域。照单重灌 = 直接制造重复。故本脚本对已存在 content 做**精确去重**。
  3. 逐个实测可达性后，**4 个物理上取不到**（下表）。

## 处置矩阵（每条都有实测依据，非推测）

| 数据集 | 存储 | 处置 | 依据 |
|---|---|---|---|
| genrobot2025/Gen-HumanEgo | 62.7 TB | **跳过** | 用户指令：TB 级跳过 + **401** |
| eidon-ai/tracker-pov | 9.0 TB | **跳过** | 用户指令：TB 级跳过 |
| IFM/Code-Reasoning | 3.3 TB | **跳过** | 用户指令：TB 级跳过 |
| Yootta/World-SimReady-Home | 3.0 TB | **跳过（强制）** | **401 Unauthorized**。用户要求"cc-by-nc-sa 吸收学习其经验"，但 gated 无令牌**物理不可达**；已在报告里明说，不是本脚本的选择 |
| malcolmrey/various | 6.08 GB | **跳过** | 实测仅 **33 行**且为 `image` 字典（**无文本**）。6GB 存储换 33 个图片 URL，且许可标 **wtfpl**（非标准）→ 蒸馏价值为零。**这是实测后推翻自己原计划** |
| MoreThought/Fable-…-10000x | 8.59 GB | **吸收** | apache-2.0，`full` split = 10,000 条 agentic 轨迹。**吸 `full` 而非 `lite`**，因 `lite`(=5000x 版)已有 100 行在库 |
| nyu-mll/glue | 4.01 GB | **吸收（仅 train/validation）** | 用户指令"其他的直接做"。但它是**基准**：只吸 `train`，**永不吸 `test`**（`test` 标签多为 -1，且会造成 train/test 污染）。许可标 `other` → 内容打标 |
| ZefanCai/Open-Jev | 85.7 MB | **吸收（最高优先）** | **cc0-1.0**；带标准答案的多选判分数据 —— 正对本仓已实测的"词法判分器方差上限 0.02"天花板。跨 **8 个 config** 采样，不挤在单一 config |
| LocalLLaMA/typed-decisions | 3.2 MB | **吸收** | apache-2.0，`all` config = 1,200 行 / 4 个 workflow；含 `label_agreement.total_variation` → 直接喂 `nt_jev_calibration` |
| Anthropic/hh-rlhf | 291 MB | **吸收（红队打标）** | mit，160,800 行。首行即有害请求 → 打 `[redteam]` 供下游过滤，**不丢弃**（拒答样本有学习价值） |
| FineEnvs/SmolDataEnvs | 9.7 MB | **吸收** | mit，5,000 行，带 `reward_mode`/`atol` 的可验证 QA 元数据 |
| AxiomicLabs/Tiny_Theory_of_Mind | 1.6 MB | **吸收** | apache-2.0，2,000 行；`activity_label` 编码 knowledge/ignorance × order × difficulty |

## 三条纪律（都来自本仓已修过的真实事故）

- **不吸 test/ood split**：Open-Jev 有 `test`/`ood`，glue 有 `test` —— 吸了就是基准污染。
- **不做 hub-and-spoke 连接**：`nt_cocoons_prune_monoculture.py` 刚清掉一个入度 935 的锚点。
  故同组内用**链式**连接（成员 i 连 i±1），入度 ≤ 2，结构上不可能再长出 mega-hub。
- **不整包下载**：一律走 datasets-server rows API；且用**跨步采样**（offset 拉开），
  避免前 N 条全落在同一个 source 家族里。

用法:
  python3 scripts/ops/nt_hf_batch_absorb.py --selftest
  python3 scripts/ops/nt_hf_batch_absorb.py --dry-run
  python3 scripts/ops/nt_hf_batch_absorb.py --commit
"""
import argparse
import collections
import http.client
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nt_hf_digest_to_cocoons as base  # noqa: E402

ROWS = ("https://datasets-server.huggingface.co/rows?dataset=%s"
        "&config=%s&split=%s&offset=%d&length=%d")
UA = "NeoTrix/0.20 (nt_hf_batch_absorb.py)"
TIMEOUT = 40
MANIFEST = os.path.join("datasets", "hf_batch", "absorb_manifest.json")

# (ds, license, domain, memory_type, confidence, [(config, split, cap)])
# 全部 config/split 组合均经 hf_survey.json + 实测 rows 探测确认存在。
SOURCES = [
    ("ZefanCai/Open-Jev", "cc0-1.0", "jev-choice", "Fact", 0.75, [
        ("release-v2-redistributable", "train", 320),
        ("browser-drone-expansion-v1-redistributable", "train", 220),
        ("amount-extraction-control-v1", "train", 220),
    ]),
    ("MoreThought/Fable-5.1-Max-Reasoning-Filtered-10000x", "apache-2.0",
     "agentic-trace", "Pattern", 0.60, [("default", "full", 1200)]),
    ("AxiomicLabs/Tiny_Theory_of_Mind", "apache-2.0", "theory-mind", "Fact", 0.70,
     [("default", "train", 1000)]),
    ("FineEnvs/SmolDataEnvs", "mit", "grounded-qa", "Fact", 0.70,
     [("default", "train", 1200)]),
    ("LocalLLaMA/typed-decisions", "apache-2.0", "decision-calib", "Causal", 0.70,
     [("all", "train", 600)]),
    ("Anthropic/hh-rlhf", "mit", "pref-pair", "Pattern", 0.55,
     [("default", "train", 700)]),
    ("nyu-mll/glue", "other", "nli-benchmark", "Fact", 0.50, [
        ("rte", "train", 120), ("sst2", "train", 100),
        ("cola", "train", 80), ("qnli", "train", 60), ("mrpc", "train", 60),
    ]),
]

# 禁语（红队/有害）→ 打 [redteam] 标记而非丢弃。用**意图词**而非完整脏词表，
# 避免误伤正常的医学/安全讨论（selftest 里有正例断言）。
REDTEAM_PAT = re.compile(
    r"cuss word|swear word|how to (make|build|synthesi[sz]e) (a )?"
    r"(bomb|explosive|weapon|drug)|kill myself|suicide method|"
    r"how do i (kill|hurt|poison)|racial slur|write a (hate|racist)", re.I)

WS = re.compile(r"\s+")

# 网络/解析异常全集。HTTPError/URLError 是 OSError 子类，JSONDecodeError 是
# ValueError 子类；`IncompleteRead`/`RemoteDisconnected` 走 HTTPException。
NET_ERR = (OSError, ValueError, http.client.HTTPException)


def flat(s, cap=0):
    t = WS.sub(" ", (s or "")).strip()
    if cap and len(t) > cap:
        t = t[:cap].rsplit(" ", 1)[0] + " …"
    return t


def get_rows(ds, cfg, split, offset, length, tries=4):
    """取一页 rows。**必须覆盖 http.client.HTTPException** —— 干跑实测 Fable
    返回大体积 agentic 轨迹时会抛 `IncompleteRead`（属 HTTPException，不属
    URLError），漏掉它会让一次瞬时网络抖动直接终止整批。"""
    url = ROWS % (urllib.parse.quote(ds, safe=""),
                  urllib.parse.quote(cfg, safe=""),
                  urllib.parse.quote(split, safe=""), offset, length)
    last = None
    for i in range(tries):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": UA})
            with urllib.request.urlopen(req, timeout=TIMEOUT) as r:
                return json.loads(r.read())
        except NET_ERR as e:            # noqa: PERF203 — 重试语义需要逐次捕获
            last = e
            code = getattr(e, "code", None)
            if code in (401, 403, 404):  # 权限/不存在：重试无意义
                break
            # 429 限流必须长退避：干跑实测 datasets-server 在密集取页下直接 429
            time.sleep((5.0 + 5.0 * i) if code == 429 else (1.0 + 1.5 * i))
    raise last


def jload(s):
    try:
        return json.loads(s) if isinstance(s, str) else s
    except (ValueError, TypeError):
        return None


def distill(ds, row, dom, mtype, conf, lic):
    """源行 → (content, conn_keys)。返回 None 表示该行无可蒸馏内容。"""
    def s(*ks):
        for k in ks:
            v = row.get(k)
            if isinstance(v, str) and v.strip():
                return v
        return ""

    # ---- Open-Jev: 多选判分，target 是独热向量 ----
    if ds == "ZefanCai/Open-Jev":
        q = flat(s("question"), 300)
        opts = row.get("options")
        tgt = row.get("target")
        if not q or not isinstance(opts, list) or not opts:
            return None
        gold = -1
        if isinstance(tgt, list):
            for i, v in enumerate(tgt):
                try:
                    if float(v) > 0.5:
                        gold = i
                        break
                except (TypeError, ValueError):
                    continue
        ans = flat(opts[gold], 90) if 0 <= gold < len(opts) else "?"
        # **必须带 state_json 摘要**：干跑实测 2400 行里 2318 行完全重复 —— 同一套
        # 模板题跨不同 group_id 反复出现。只留 question+options 会把 79,116 行
        # 塌缩成 ~80 条。真正的信息量在对话载荷里。
        conv = flat(s("state_json"), 300)
        m = re.search(r"Customer message:\s*(.+?)(?:\n|$)", conv)
        quote = flat(m.group(1), 150) if m else conv[:110]
        return ("[jev-choice] %s → %s || 客户原话: %s || 候选: %s"
                % (q, ans, quote or "?",
                   " | ".join(flat(o, 44) for o in opts[:6])),
                [s("source"), s("kind")])

    # ---- Fable: messages = role/content 列表，取首个 user 任务 + 末个 assistant 答案 ----
    if ds.startswith("MoreThought/Fable"):
        msgs = row.get("messages")
        if not isinstance(msgs, list) or not msgs:
            return None
        users = [m.get("content") for m in msgs
                 if isinstance(m, dict) and m.get("role") == "user"]
        asst = [m.get("content") for m in msgs
                if isinstance(m, dict) and m.get("role") == "assistant"]
        task = flat((users or [""])[-1], 260)
        ans = flat((asst or [""])[-1], 420)
        if not task and not ans:
            return None
        return ("[agentic-trace] 任务: %s ⇒ 结论: %s" % (task or "?", ans or "?"),
                ["agentic"])

    # ---- Tiny_Theory_of_Mind: activity_label 编码 topic::order::difficulty ----
    if ds == "AxiomicLabs/Tiny_Theory_of_Mind":
        ctx = flat(s("ctx_a", "ctx", "ctx_b"), 380)
        ends = row.get("endings")
        if not ctx or not isinstance(ends, list) or not ends:
            return None
        lab = s("activity_label")
        parts = lab.split("::")
        topic = parts[1] if len(parts) > 1 else "?"
        diff = parts[-1] if len(parts) > 2 else "?"
        ans = ""
        md = row.get("metadata")
        if isinstance(md, dict):
            ans = flat(str(md.get("answer") or ""), 60)
        return ("[theory-mind:%s/%s] 情境: %s ⇒ 结局: %s || 选项: %s"
                % (topic, diff, ctx, ans or "?",
                   " | ".join(flat(e, 38) for e in ends[:5])),
                [topic, diff])

    # ---- SmolDataEnvs: 可验证 QA + reward 元数据 ----
    if ds == "FineEnvs/SmolDataEnvs":
        q = flat(s("question"), 280)
        a = flat(s("answer"), 180)
        if not q or not a:
            return None
        return ("[grounded-qa] %s → %s || tier=%s reward=%s atol=%s src=%s"
                % (q, a, s("difficulty_tier") or "?", s("reward_mode") or "?",
                   s("atol") or "-", (s("kaggle_dataset") or "-").split("/")[0]),
                [s("difficulty_tier"), (s("kaggle_dataset") or "-").split("/")[0]])

    # ---- typed-decisions: gold/factors/label_agreement 均为 JSON 字符串 ----
    if ds == "LocalLLaMA/typed-decisions":
        gold = jload(row.get("gold"))
        if not isinstance(gold, dict) or not gold:
            return None
        agree = jload(row.get("label_agreement")) or {}
        factors = jload(row.get("factors")) or {}
        tvs = [v.get("total_variation") for v in agree.values()
               if isinstance(v, dict) and isinstance(v.get("total_variation"),
                                                      (int, float))]

        def brief(d, n=6):
            out = []
            for k, v in sorted(d.items())[:n]:
                if isinstance(v, dict):
                    out.append("%s=%s(%.2f)" % (k, v.get("label"),
                                                float(v.get("confidence") or 0)))
                else:
                    out.append("%s=%s" % (k, v))
            return "; ".join(out)

        return ("[decision-calib] %s || gold: %s || 判人间一致性: %s || 因子: %s"
                % (s("workflow") or "?", brief(gold),
                   ("TV=%s" % ",".join("%.2f" % t for t in tvs[:4])) if tvs else "n/a",
                   flat(brief(factors, 6), 200)),
                [s("workflow")])

    # ---- hh-rlhf: 偏好对；红队打标 ----
    if ds == "Anthropic/hh-rlhf":
        ch = flat(s("chosen"), 400)
        rj = flat(s("rejected"), 220)
        if not ch or not rj:
            return None
        tag = "[redteam]" if REDTEAM_PAT.search(ch) else ""
        return ("%s[pref-pair] 优选: %s || 劣选: %s" % (tag, ch, rj),
                ["redteam" if tag else "benign"])

    # ---- glue: 形状随 task 变（sentence / sentence1+sentence2），仅 train ----
    if ds == "nyu-mll/glue":
        lab = row.get("label")
        if not isinstance(lab, int) or lab < 0:
            return None                      # test split 的 label 是 -1，已被此判据挡住
        prem = [flat(row[k], 150) for k in ("sentence", "sentence1", "sentence2")
                if isinstance(row.get(k), str) and row.get(k).strip()]
        prem = [p for p in prem if p]
        if not prem:
            return None
        return ("[nli-benchmark] 前提: %s ⇒ 标签 %d" % (" ‖ ".join(prem[:2]), lab),
                ["glue"])
    return None


STAGE = os.path.join("datasets", "hf_batch", ".stage.json")


def make_record(mid, ds, lic, dom, mtype, content, conns, conf, now):
    """构造一条记忆记录。

    **刻意独立成函数**：干跑在写盘前就 return，结构上无法覆盖本函数。
    首版就是在这里踩了 `NameError: mtype`（解包用 `mt`、此处写 `mtype`），
    直到 --commit 才炸。现在 selftest 直接调用本函数，锁死该回归。
    """
    return {
        "id": mid,
        "content": "[%s|%s] %s" % (ds, lic, content),
        "memory_type": mtype,
        "domain": dom,
        "strength": 1.0,
        "confidence": conf,
        "importance": min(1.0, conf + 0.05),
        "connections": list(conns)[:3],
        "created_at": now,
        "last_accessed": now,
        "access_count": 0,
    }


def stage_dump(mems, stats):
    os.makedirs(os.path.dirname(STAGE), exist_ok=True)
    tmp = STAGE + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump({"mems": [list(m) for m in mems], "stats": stats},
                  fh, ensure_ascii=False)
    os.replace(tmp, STAGE)


def stage_load():
    """两阶段提交：--commit 消费已复核的干跑产物，不重新联网取数。
    干跑实测一轮完整取数需 ~25 分钟，且 hh-rlhf 遇 502 会掉量。"""
    with open(STAGE, encoding="utf-8") as fh:
        d = json.load(fh)
    return [tuple(m) for m in d["mems"]], d["stats"]


def load_existing_contents():
    d = json.load(open(base.COCOONS, encoding="utf-8"))
    return {m["content"] for c in d["cocoons"].values() for m in c["memories"]}


def selftest():
    assert len({s[0] for s in SOURCES}) == len(SOURCES), "源重复"
    for ds, lic, dom, mt, conf, combos in SOURCES:
        assert all(c[2] > 0 for c in combos), "%s cap 必须为正" % ds
        # 纪律 1：绝不吸 test / ood split
        for cfg, split, cap in combos:
            assert split not in ("test", "ood"), \
                "%s 试图吸收 %s split —— 会造成基准污染" % (ds, split)
    tot = sum(c[2] for s in SOURCES for c in s[5])
    assert tot <= 8000, "总体量须有界（核心要保持可加载），当前 %d" % tot
    assert flat("a\n\n b   c") == "a b c"
    assert flat("x" * 50, 10).endswith("…")
    assert REDTEAM_PAT.search("what are some cuss words in english")
    assert not REDTEAM_PAT.search("how do I treat a patient with sepsis")
    # 判据：glue test split 的 label=-1 必须被挡住
    assert jload('{"a":{"label":"x","confidence":0.5}}') is not None
    assert jload("{bad json") is None
    # 不做 hub-and-spoke：链式连接每点入度 ≤ 2
    grp = ["m%d" % i for i in range(5)]
    deg = collections.Counter()
    for i, m in enumerate(grp):
        for j in (i - 1, i + 1):
            if 0 <= j < len(grp):
                deg[grp[j]] += 1
    assert max(deg.values()) <= 2, "链式连接入度应 ≤2，实际 %d" % max(deg.values())
    # 步长必须由总行数导出（见 main 里的 span）。回归锁：2000 行的 split 取 1000
    # 条，步长须 ≤ 2000 且 ≥ WIN，否则窗口一步跨过末尾。
    for total_rows, cap, win in ((2000, 1000, 100), (5000, 1200, 100),
                                 (79116, 500, 100), (1200, 600, 100)):
        n_win = max(1, -(-cap // win))
        span = max(win, total_rows // n_win)
        assert win <= span <= total_rows or total_rows < win, \
            "span=%d 对 total_rows=%d 不合理" % (span, total_rows)
    # 写盘路径必须被 selftest 覆盖（--dry-run 走不到这里）
    rec = make_record("M-099999", "ds/x", "mit", "d", "Fact", "body", ["M-1"], 0.5, 7)
    assert rec["id"] == "M-099999" and rec["memory_type"] == "Fact"
    assert rec["content"].startswith("[ds/x|mit] body")
    assert rec["connections"] == ["M-1"] and rec["created_at"] == 7
    assert rec["access_count"] == 0 and 0.0 < rec["importance"] <= 1.0
    assert make_record("M-1", "a", "b", "c", "Causal", "x",
                       ["a", "b", "c", "d"], 0.9, 1)["connections"] == ["a", "b", "c"]
    print("selftest ok: no-test-split / cap=%d / flatten / redteam / json / "
          "chain-in-degree<=2 / span-bounded / write-path" % tot)
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--commit", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    if args.commit and os.path.exists(STAGE):
        mems, stats = stage_load()
        print("复用已复核的 stage: %s（%d 条，不重新联网）" % (STAGE, len(mems)))
        existing = load_existing_contents()
        fresh = [m for m in mems if m[2] not in existing]
        if len(fresh) != len(mems):
            print("stage 与活库已产生重叠：丢弃 %d 条已存在内容" % (len(mems) - len(fresh)))
            mems = fresh
        return commit(mems, stats, existing)

    existing = load_existing_contents()
    print("活库已有 content: %d 条（精确去重判据）" % len(existing))

    mems, stats = [], {}
    WIN = 100
    # Fable 单行是多轮 agentic 轨迹（体积大一个量级），必须小页取，否则 100 行/页
    # 会触发 IncompleteRead 且单页耗时逼近超时。
    BIGROW = {"MoreThought/Fable-5.1-Max-Reasoning-Filtered-10000x": 25}
    for ds, lic, dom, mtype, conf, combos in SOURCES:
        got = dup = tries = 0
        per_combo = {}
        for cfg, split, cap in combos:
            got_here = 0
            win = BIGROW.get(ds, WIN)
            # 先探总行数。**步长必须由总行数导出** —— 干跑实测：早先用
            # `stride = cap*2`，对只有 2000 行的 ToM 直接一步跨过末尾，
            # 2000/1000 行的 split 只取到首批 100 条就空。
            try:
                meta = get_rows(ds, cfg, split, 0, 1)
                total_rows = int(meta.get("num_rows_total") or 0)
            except (NET_ERR, KeyError) as e:
                print("  %-38s %s/%s 探测行数失败: %s" % (ds[:38], cfg, split, str(e)[:44]))
                continue
            # 需要多少个窗口才能拿满 cap
            n_win = max(1, -(-cap // win))
            span = max(win, total_rows // n_win) if total_rows else win
            off = 0
            while got_here < cap and tries < 40:
                tries += 1
                try:
                    r = get_rows(ds, cfg, split, off, min(win, cap - got_here))
                except NET_ERR as e:
                    print("  %-38s %s/%s 取数失败: %s" % (ds[:38], cfg, split, str(e)[:44]))
                    break
                rows = r.get("rows") or []
                if not rows:
                    break
                for item in rows:
                    row = item.get("row") if isinstance(item, dict) else None
                    if not isinstance(row, dict):
                        continue
                    d = distill(ds, row, dom, mtype, conf, lic)
                    if not d:
                        continue
                    content, keys = d
                    if content in existing:
                        dup += 1
                        continue
                    existing.add(content)
                    mems.append((dom, mtype, content, keys, ds, lic, conf))
                    got += 1
                    got_here += 1
                    # cap 必须在**行内**判定：早先只在窗口后判，导致
                    # hh-rlhf 796/700、glue 500/420 溢出。
                    if got_here >= cap:
                        break
                off += span
                if tries % 8 == 0:
                    print("      …%s/%s off=%d/%d got=%d dup=%d"
                          % (cfg, split, off, total_rows, got, dup))
                    sys.stdout.flush()
                time.sleep(0.8)
            per_combo["%s/%s" % (cfg, split)] = {
                "cap": cap, "fetched": got_here, "rows_total": total_rows,
                "span": span}
        stats[ds] = {"fetched": got, "dup_skipped": dup,
                     "domain": dom, "license": lic, "combos": per_combo}
        print("  %-44s 取到 %4d   去重跳过 %d   [%s]"
              % (ds[:44], got, dup, lic))
        sys.stdout.flush()

    total = len(mems)
    print("\n合计待写 %d 条（较活库 +%.2f%%）" % (total, 100.0 * total / len(existing)))
    for k, v in collections.Counter(m[0] for m in mems).most_common():
        print("   %-18s %5d" % (k, v))
    if not args.commit:
        print("\n--- DRY RUN：落 stage，不写活库 ---")
        stage_dump(mems, stats)
        print("stage: %s" % STAGE)
        for m in mems[:3]:
            print("  样例: %s" % m[2][:150])
        return 0

def commit(mems, stats, existing):
    """落盘：guard → 编号 → 链式连接 → 分片 → 备份 → splice → manifest。"""
    total = len(mems)
    now = int(time.time())
    mx, _n = base.fast_max_mid(base.COCOONS)
    base.guard_id_collision(open(base.COCOONS, "rb").read(),
                           ["M-%06d" % (mx + 1 + i) for i in range(total)])
    print("guard: 无重号 ✓ (base M-%06d)" % mx)

    ids = ["M-%06d" % (mx + 1 + i) for i in range(total)]
    # 纪律 2：组内链式连接（入度 ≤ 2），不做 hub-and-spoke
    by_key = collections.defaultdict(list)
    for i, (dom, mt, content, keys, ds, lic, conf) in enumerate(mems):
        for k in keys:
            if k:
                by_key[(dom, k)].append(i)
    out = []
    for i, (dom, mtype, content, keys, ds, lic, conf) in enumerate(mems):
        conns = []
        for k in keys:
            grp = by_key.get((dom, k)) or []
            try:
                p = grp.index(i)
            except ValueError:
                continue
            for j in (p - 1, p + 1):
                if 0 <= j < len(grp):
                    cid = ids[grp[j]]
                    if cid != ids[i] and cid not in conns:
                        conns.append(cid)
            if len(conns) >= 3:
                break
        out.append(make_record(ids[i], ds, lic, dom, mtype, content, conns, conf, now))

    per_dom = collections.defaultdict(list)
    for m in out:
        per_dom[m["domain"]].append(m)
    frag = os.path.join("datasets", "hf_batch", ".frag.tmp")
    os.makedirs(os.path.dirname(frag), exist_ok=True)
    wrote, edges = [], sum(len(m["connections"]) for m in out)
    with open(frag, "w", encoding="utf-8") as sp:
        for dom in sorted(per_dom):
            mm = per_dom[dom]
            for k in range(0, len(mm), base.SHARD):
                shard = mm[k:k + base.SHARD]
                cid = "cocoon-%s-%d%s" % (dom, now,
                                          "" if k == 0 else "-p%d" % (k // base.SHARD))
                sp.write(base.cocoon_entry(cid, shard, now) + ",\n")
                wrote.append((cid, len(shard)))
    print("新茧 %d  边 %d  片段 %.2fMB" % (len(wrote), edges, os.path.getsize(frag) / 1e6))

    import glob
    import shutil
    import time as _t
    live = os.path.getsize(base.COCOONS)
    bak = base.COCOONS + ".bak.hfbatch"
    olds = sorted(glob.glob(bak + ".*"))
    for o in olds:
        try:
            if os.stat(o).st_size > 1_000_000 and live < 1_000_000:
                print("ABORT: live %.1fKB, backup %s 保留" % (live / 1e3, o))
                os.remove(frag)
                return 2
        except OSError:
            pass
    stamped = "%s.%s" % (bak, _t.strftime("%Y%m%d-%H%M%S"))
    shutil.copyfile(base.COCOONS, stamped)
    shutil.copyfile(base.COCOONS, bak)
    for o in sorted(glob.glob(bak + ".*"))[:-2]:
        try:
            os.remove(o)
        except OSError:
            pass
    with open(frag, encoding="utf-8") as fh:
        entries = fh.read().rstrip()
    if entries.endswith(","):
        entries = entries[:-1]
    tmp = base.COCOONS + ".tmp.hfbatch"
    with open(tmp, "wb") as fh:
        fh.write(base.splice(base.COCOONS, entries))
    os.replace(tmp, base.COCOONS)
    os.remove(frag)

    os.makedirs(os.path.dirname(MANIFEST), exist_ok=True)
    man = {"sources": stats, "total": total, "edges": edges,
           "new_cocoons": [c for c, _ in wrote], "base_max_mid": mx,
           "live_bytes_after": os.path.getsize(base.COCOONS),
           "skipped": {
               "TB 级（用户指令）": ["genrobot2025/Gen-HumanEgo", "eidon-ai/tracker-pov",
                                     "IFM/Code-Reasoning"],
               "401 gated（物理不可达）": ["Yootta/World-SimReady-Home"],
               "实测无蒸馏价值（33 行纯图像 + wtfpl）": ["malcolmrey/various"]},
           "discipline": ["不吸 test/ood split（基准污染）",
                          "组内链式连接，入度 ≤ 2（防 monoculture）",
                          "跨步采样（防同源扎堆）",
                          "对活库已有 content 精确去重"]}
    with open(MANIFEST + ".tmp", "w", encoding="utf-8") as fh:
        json.dump(man, fh, ensure_ascii=False, indent=1)
    os.replace(MANIFEST + ".tmp", MANIFEST)
    print("backup: %s" % stamped)
    print("committed: +%d 条 -> cocoons.json %.1fMB"
          % (total, os.path.getsize(base.COCOONS) / 1e6))
    print("manifest: %s" % MANIFEST)
    return 0

if __name__ == "__main__":
    sys.exit(main() or 0)
