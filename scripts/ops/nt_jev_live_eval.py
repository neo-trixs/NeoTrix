#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_jev_live_eval — 用**已吸收的真实标答数据**实测词法判分器的判别力。

## 为什么要这个

RFC §v3.2 那个「词法判分器判别力天花板 ≈ 0.02」的结论，是在**模板**上量出来的
（`nt_verify_sim.py` 的受控探针）。那是合成数据，可能低估也可能高估。

本轮吸收了 431 条 `ZefanCai/Open-Jev` 记忆，**每条自带标答**（cc0-1.0）：
```
[jev-choice] {题干} → {标答} || 客户原话: {对话摘录} || 候选: {opt1} | {opt2} | ...
```
这构成一个**有标准答案、可量化的真实判别任务**：
给定对话，从若干候选里挑出正确的那一个。

于是可以给出三个此前拿不到的东西：
  1. 词法判分器在真实数据上的 **top-1 准确率 / MRR**（而非模板方差）；
  2. **随机基线**（候选数的倒数）—— 没有它，准确率高低都没有意义；
  3. 对 RFC §v4.2 那条断言的判决：「Open-Jev 的信息量在**对话**不在题干」
     是否真的成立 —— 靠对比 `仅对话` / `仅题干` / `对话+题干` 三种上下文。

## 纪律

- 复用核心的权威分词（**不重写**）—— 必须量的是核心真正在用的那个切分函数，
  重写一份就变成在量另一个东西。
  2026-09-28 起：`nt_verify_sim` 已随 2bbed32c 删除，改调 Rust 导出点
  `cargo build -p neotrix --bin nt_keywords`（内部即
  `CrystalConsciousness::keywords`，consciousness.rs:546）。
  **首次使用需先 build**（见下方 keywords() 的报错指引）。
- 只读活库，不写。
- 截断处理：`候选` 每项截到 44 字符而 `标答` 截到 90，故标答↔候选用**前缀**匹配
  对齐，不是等值匹配；对不齐的条目**丢弃并计数**（不静默）。

用法:
  python3 scripts/ops/nt_jev_live_eval.py --selftest
  python3 scripts/ops/nt_jev_live_eval.py
"""
import argparse
import collections
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# 2026-09-28 修断链依赖：本脚本原先 `from nt_verify_sim import keywords`，
# 而该模块已随 2bbed32c 删除 ⇒ **脚本当前完全跑不起来**（ImportError）。
#
# 修法（遵守本文件「不重写」纪律）：改为调 Rust 侧新导出的 `nt_keywords` bin，
# 它直接调用晶体核心的权威实现 `CrystalConsciousness::keywords`
# （l5_cognition/nt_crystal_core/consciousness.rs:546）。**不在 Python 里重写分词** ——
# 重写会立刻产生第二套口径，而「第二份真源会漂」是本仓反复治的病。
#
# 代价：首次调用要 cargo build（可能数分钟）。用 --selftest 预热。
_NT_KEYWORDS_BIN = None


def _keywords_bin():
    """定位 nt_keywords 可执行文件；找不到时给出可执行的修复指引。"""
    global _NT_KEYWORDS_BIN
    if _NT_KEYWORDS_BIN is not None:
        return _NT_KEYWORDS_BIN
    for cand in (
        os.path.expanduser("~/Downloads/neotrix/target/debug/nt_keywords"),
        os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
            os.path.abspath(__file__)))), "target/debug/nt_keywords"),
    ):
        if os.path.isfile(cand) and os.access(cand, os.X_OK):
            _NT_KEYWORDS_BIN = cand
            return _NT_KEYWORDS_BIN
    sys.exit(
        "jev-eval: 找不到 nt_keywords 可执行文件。\n"
        "  修法: cargo build -p neotrix --bin nt_keywords\n"
        "  （它是晶体核心权威分词的导出点，勿改为在 Python 里重写分词）"
    )


def keywords(text):
    """调 Rust 权威分词。批量喂（一行一段文本）避免逐条起进程。"""
    import subprocess
    proc = subprocess.run(
        [_keywords_bin()], input=text + "\n",
        capture_output=True, text=True, timeout=60,
    )
    if proc.returncode != 0:
        sys.exit(f"jev-eval: nt_keywords 失败: {proc.stderr.strip()}")
    return json.loads(proc.stdout.strip() or "[]")

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")
DOMAIN = "jev-choice"

# content 形状：[src|lic] [jev-choice] Q → A || 客户原话: Q || 候选: o1 | o2 | ...
RE_JEV = re.compile(
    r"\[jev-choice\]\s*(?P<q>.+?)\s*→\s*(?P<a>.+?)\s*\|\|\s*客户原话:\s*(?P<u>.+?)"
    r"\s*\|\|\s*候选:\s*(?P<opts>.+)\s*$", re.S)


def parse(content):
    """返回 (题干, 标答, 对话摘录, [候选…])；形状不符则 None。"""
    m = RE_JEV.search(content or "")
    if not m:
        return None
    opts = [o.strip() for o in m.group("opts").split("|") if o.strip()]
    if len(opts) < 2:
        return None
    # 三个字段都过 _norm：真实对话里含**字面** `\\n`（蒸馏时 flat() 把换行转义成
    # 两字符），而 keywords() 的切分符不含反斜杠 ⇒ 不清理会把两侧词粘成一个 token。
    return (_norm(m.group("q")), _norm(m.group("a")),
            _norm(m.group("u")), opts)


def _norm(s):
    """去掉截断标记与转义换行。

    实测：`候选` 每项截到 44 字符并追加 `…`，而 `标答` 截到 90 字符。
    直接前缀匹配会被那个 `…` 打断（"account: Login, … …" 不以对方开头），
    所以比对前必须先剥掉截断标记。
    """
    return (s or "").replace("\\n", " ").replace("\n", " ").strip().rstrip("…").strip()


def align_gold(ans, opts):
    """标答↔候选对齐：截断长度不同（90 vs 44），故用前缀匹配而非等值。"""
    a = _norm(ans)
    hits = [i for i, o in enumerate(opts) if a and (a == _norm(o)
            or a.startswith(_norm(o)) or _norm(o).startswith(a))]
    return hits[0] if len(hits) == 1 else None


def lex_score(ctx_kw, cand):
    """词法判别：候选与上下文的关键词交集大小（越大越像「这段话在说它」）。

    这是**故意保持朴素**的 —— 目的是量出「纯词法路线」的上限，
    好让 judge 模型的收益有可比基线。刻意不用 embedding / 模型。
    """
    ck = set(keywords(cand))
    if not ck:
        return 0.0
    return len(ck & ctx_kw) / float(len(ck))


def rank(ctx_text, opts):
    ck = set(keywords(ctx_text or ""))
    sc = [lex_score(ck, o) for o in opts]
    best = max(sc) if sc else 0.0
    # 全为 0（毫无词面交集）时退化为均匀分布 → 期望命中 1/n，计入随机基线
    return sc, best


def evaluate(items, ctx_key):
    """ctx_key: 'u'=对话 / 'q'=题干 / 'qu'=对话+题干 / ''=无上下文(对照)

    **零交集的处理是本函数的关键**：若所有候选得分并列 0（词法完全无信号），
    预测本质上是**不确定的**。此时若按「同分保序取第一个」去算，等于白送
    index 0 一个正确 —— 而金标常在 index 0（数据集按标答排序的痕迹），
    就会得到虚高的准确率。故此处**按随机基线计入**：acc 记 1/n，
    MRR 记 H_n/n（均匀随机下的期望倒排），并单独统计 degenerate 条数。
    """
    hit = 0
    tot = 0
    rr = 0.0
    rand = 0.0
    degenerate = 0
    for _q, _a, u, opts, gold in items:
        ctx = {"u": u, "q": _q, "qu": u + " " + _q, "": ""}[ctx_key]
        sc, best = rank(ctx, opts)
        n = len(opts)
        rand += 1.0 / n
        tot += 1
        if best <= 0.0:
            # 无判别信号 → 按机会水平计入，不给 index 0 免费分
            degenerate += 1
            hit += 1.0 / n
            rr += sum(1.0 / p for p in range(1, n + 1)) / n
            continue
        order = sorted(range(n), key=lambda i: (-sc[i], i))  # 稳定：同分保序
        if order[0] == gold:
            hit += 1.0
        for pos, i in enumerate(order, 1):
            if i == gold:
                rr += 1.0 / pos
                break
    if not tot:
        return None
    return {"n": tot, "acc": hit / tot, "mrr": rr / tot,
            "rand": rand / tot, "degenerate": degenerate}


def selftest():
    # Open-Jev 真实语料是**英文**（见活库样本），故夹具照抄英文，
    # 否则会在测一个根本不存在于数据里的形态。
    c = ("[ZefanCai/Open-Jev|cc0-1.0] [jev-choice] Which category is the urgent "
         "request? → account: Login, permissions || 客户原话: I need access "
         "restored after changing the email address used to sign in || "
         "候选: account: Login, permissions, profile, security | "
         "billing: Charges, invoices, refunds | bug_report: Something is broken")
    q, a, u, opts = parse(c)
    assert q.startswith("Which category"), q
    assert a == "account: Login, permissions", a
    assert "access restored" in u, u
    assert len(opts) == 3, opts
    assert align_gold(a, opts) == 0
    # 前缀对齐 + 剥截断标记（真实候选带尾随 …）
    assert align_gold("account: Login, permissions, profile, security",
                      ["account: Login, permissions, profile, …",
                       "bug_report: …"]) == 0
    # 字面 \n 不应破坏解析
    _q2, _a2, u2, _o2 = parse(c.replace("restored", "restored\\nnow"))
    assert "\\n" not in u2 and "now" in u2, u2
    # 歧义须返回 None 而非瞎猜
    assert align_gold("支付", ["支付", "支付"]) is None
    # 无上下文 => 全 0 => 计入随机基线
    sc, best = rank("", opts)
    assert best == 0.0 and all(v == 0.0 for v in sc)
    # 有词面信号的情形（金标放 index 1，**不是 0** —— 否则对照组能白捡）
    c2 = ("[ZefanCai/Open-Jev|cc0-1.0] [jev-choice] Which category? → "
          "billing: Charges, invoices, refunds || 客户原话: I need help with "
          "invoices and refunds on my charges || 候选: "
          "account: Login, permissions, profile, security | "
          "billing: Charges, invoices, refunds | bug_report: Something is broken")
    q2, a2, u2, opts2 = parse(c2)
    assert align_gold(a2, opts2) == 1, (a2, opts2)
    r = evaluate([(q2, a2, u2, opts2, 1)], "u")
    assert r["acc"] == 1.0 and r["degenerate"] == 0, r
    # 无上下文 -> **按机会水平**记 1/n，不得因「同分取第一个」白送 index 0
    r0 = evaluate([(q2, a2, u2, opts2, 1)], "")
    assert r0["degenerate"] == 1, r0
    assert abs(r0["acc"] - 1.0 / 3.0) < 1e-9, r0
    # **无词面信号**的情形（真实数据常态）：对话说 access restored，
    # 标答是 account: Login —— 靠理解而非词面 ⇒ 判分器必然退化。
    r_no = evaluate([(q, a, u, opts, 0)], "u")
    assert r_no["degenerate"] == 1 and r_no["acc"] == 1.0 / 3.0, r_no
    # **CJK 失明**（实测发现，锁死）：keywords() 按空白/标点切分，
    # 中文整句只成 1 个 token ⇒ 候选与上下文永不相交 ⇒ 词法判分对中文近乎无效。
    assert len(keywords("我的支付一直失败收不到验证码")) == 1, keywords("我的支付一直失败")
    r_cjk = evaluate([("问题", "答案", "我的支付一直失败", ["支付网关", "物流延迟"],
                       0)], "u")
    assert r_cjk["degenerate"] == 1 and r_cjk["acc"] == 0.5, r_cjk
    # **无词干还原**（实测发现，锁死）：invoice/invoices、refund/refunds
    # 在集合交集里不相等 ⇒ 纯复数差就让判分器归零。
    assert not ({"invoice"} & {"invoices"}), "若将来加了词干还原，此断言需更新"
    print("selftest ok: parse / prefix-align / 歧义返回 None / 零交集按机会水平 / "
          "非同分不侥幸 / CJK 失明 / 无词干 —— 均已锁定")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    d = json.load(open(COCOONS, encoding="utf-8"))
    items, bad = [], 0
    for c in d["cocoons"].values():
        for m in c["memories"]:
            if m.get("domain") != DOMAIN:
                continue
            p = parse(m["content"])
            if not p:
                bad += 1
                continue
            q, a, u, opts = p
            g = align_gold(a, opts)
            if g is None:
                bad += 1
                continue
            items.append((q, a, u, opts, g))
    if args.limit:
        items = items[:args.limit]

    print("=== 真实标答判别任务（domain=%s）===" % DOMAIN)
    print("可用条目 %d  丢弃(形状/对齐失败) %d  候选数分布 %s"
          % (len(items), bad,
             dict(sorted(collections.Counter(len(i[3]) for i in items).items()))))
    if not items:
        print("无可用条目")
        return 1

    print("\n%-22s %7s %7s %7s %9s" % ("上下文", "top-1", "MRR", "随机", "零交集"))
    print("-" * 58)
    res = {}
    for key, label in (("u", "仅对话(state_json)"), ("q", "仅题干"),
                       ("qu", "对话+题干"), ("", "无上下文(对照)")):
        r = evaluate(items, key)
        res[key] = r
        print("%-22s %6.1f%% %6.1f%% %6.1f%% %9d"
              % (label, 100 * r["acc"], 100 * r["mrr"], 100 * r["rand"],
                 r["degenerate"]))

    lift = res["u"]["acc"] - res["q"]["acc"]
    zero_rate = res["u"]["degenerate"] / float(res["u"]["n"])
    print("\n判决（注意区分两件**不同**的事）：")
    print("  A. 区分性/去重：§v4.2 的「补上对话后 79,116 行不再塌缩成 ~80 条」")
    print("     —— 由**去重实测**（2400 行中 2318 行自重复）单独成立，与本表无关。")
    print("  B. 词法判别力：对话是否比题干**更能被判分器用起来** ——")
    print("     差 = %+.1f 个百分点。" % (100 * lift))
    if lift > 0.02:
        print("     → 对话确实带**判别**信号，§v4.2 的读法可以保留。")
    else:
        print("     → 差异在噪声量级。**不可**由本表推出「对话无用」——")
        print("       A 与 B 是两回事：B 只说明**纯词法**用不上对话，")
        print("       不说明对话没有信息量（理解式判别/judge 或可用得上）。")
    print("\n  真正吃紧的是绝对水平：")
    print("    最好组合（对话+题干）top-1 = %.1f%%，随机基线 = %.1f%%，"
          "净增益仅 **%+.1f 个百分点**。"
          % (100 * res["qu"]["acc"], 100 * res["qu"]["rand"],
             100 * (res["qu"]["acc"] - res["qu"]["rand"])))
    print("    且 **%.0f%%** 的条目与上下文**零词面交集**（判分器根本无法起手）。"
          % (100 * zero_rate))
    print("    ⇒ 这是判 judge 必要性的**硬基线**：不是「分数不够好」，")
    print("      而是「这个任务上纯词法几乎没有信息可用」。")
    if res["qu"]["acc"] - res["qu"]["rand"] < 0.10:
        print("    ⇒ 结论：词法路线在此任务上**无实用价值**，强支持改用 judge 模型。")
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
