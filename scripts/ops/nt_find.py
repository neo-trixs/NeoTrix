#!/usr/bin/env python3
"""nt_find.py — 按**任务意图**查工具，取代「读 AGENTS.md 全文再人肉匹配」。

问题（2026-09-28 实测）：`scripts/` 的发现机制散落在 AGENTS.md（10 处引用）
与 RUST-STANDARDS.md（3 处）的**散落段落**里。agent 要回答「我要查死锁该跑
什么」必须先建立对两份文档的全文心智模型 —— 日常任务里这是纯开销。

解法：`.neotrix/task-index.json` 是**任务→工具的机器索引**（意图 slug + 触发词
+ 可执行命令 + 禁用条件），本命令按自然语言查它。

用法:
    python3 scripts/ops/nt_find.py 死锁
    python3 scripts/ops/nt_find.py worktree 磁盘
    python3 scripts/ops/nt_find.py --list
    python3 scripts/ops/nt_find.py --gate           # 只列门禁
    python3 scripts/ops/nt_find.py --explain 死锁   # 带 why/when_not 全文
    python3 scripts/ops/nt_find.py --audit          # 校验索引与磁盘一致

## 为什么每条都带 `when_not`
吸收 awesome-autoresearch 的领养前检查表（`check-skill-gate.sh:3-6`）时发现：
只写「什么时候用」而不写「什么时候**别**用」，agent 会把它用错。实测案例：
`check-naming.sh` 的 PASS **不代表合规**（规约 vs 现实差 1,612 个文件，2026-10-08 clean-HEAD 实测）；该门已改递减棘轮 ⇒ PASS 只意味着「没新增」，**不是**「存量已清」；
`nt_lock_audit.py` 报 12 条里 **2/3 是误报**。这两种误用都会造成真实损害。
"""
import argparse
import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
INDEX = os.path.join(REPO, ".neotrix", "task-index.json")

KIND_LABEL = {
    "gate": "门禁",
    "locate": "定位",
    "ops": "运维",
    "absorb": "吸收",
    "oneoff": "一次性",
}


def load():
    if not os.path.isfile(INDEX):
        sys.exit(f"nt_find: index not found: {INDEX}")
    with open(INDEX, encoding="utf-8") as f:
        return json.load(f)


def _tok(s):
    """粗分词：中文按 2-gram，英文/数字按 word。够用且无依赖。"""
    s = s.lower()
    out = set()
    for w in re.findall(r"[a-z0-9_./-]+", s):
        out.add(w)
    han = re.findall(r"[\u4e00-\u9fff]+", s)
    for run in han:
        if len(run) == 1:
            out.add(run)
        for i in range(len(run) - 1):
            out.add(run[i:i + 2])
    return out


def score(entry, words):
    """命中打分。

    分档：id 精确 > 触发词精确 > 触发词子串 > **id/触发词分词交集**。
    最后那档是 2026-09-28 补的 —— 实测「提交前要检查什么」「索引陈旧」
    两句都不含任何完整触发词，但拆成 2-gram 后能命中
    （提交/检查 → audit-all；索引/陈旧 → locate）。缺它则静默不命中，
    而**静默不命中比误命中更贵**：agent 会以为「没有工具能干这事」。
    """
    best = 0
    id_toks = _tok(entry["id"])
    query_toks = set()
    for q in entry.get("query", []):
        query_toks |= _tok(q)
    for w in words:
        w = w.lower()
        if w in entry["id"].lower():
            best = max(best, 10)
        for q in entry.get("query", []):
            ql = q.lower()
            if w == ql:
                best = max(best, 8)
            elif w in ql or ql in w:
                best = max(best, 5)
        # 分词交集 —— **按覆盖比例**而非绝对命中数计分。
        # 2026-09-28 实测反例：查询「提交前要检查什么」分词出
        # {提交,检查,前要,交前,查什,要检,什么}，audit-all 的 query 命中 3 个
        # （提交/检查/要检），而 worktree-gate 因「检查」子串命中 query 走了
        # 5 分档 —— 1 个子串压过 3 个分词，排序完全错位。
        # 比例口径让「命中更多且更集中」者胜出，与人的直觉一致。
        wt = _tok(w)
        if wt:
            best = max(best, int(12 * len(wt & id_toks) / len(wt)))
            if query_toks:
                best = max(best, int(10 * len(wt & query_toks) / len(wt)))
    return best


def cmd_list(data, args):
    tasks = data["tasks"]
    if args.gate:
        tasks = [t for t in tasks if t["kind"] == "gate"]
    print(f"nt_find: {len(tasks)} tasks in {os.path.relpath(INDEX, REPO)}\n")
    for t in tasks:
        ro = "只读" if t.get("read_only") else "有副作用"
        print(f"  {t['id']:<22} [{KIND_LABEL.get(t['kind'], t['kind']):<4}] {ro}")
        print(f"      {t['tool']}")
    return 0


def cmd_search(data, args):
    words = args.words
    hits = sorted(
        ((score(t, words), t) for t in data["tasks"] if score(t, words) > 0),
        key=lambda x: -x[0],
    )
    if not hits:
        print(f"nt_find: 无匹配：{' '.join(words)}")
        print(f"  试 --list 看全部，或在 {os.path.relpath(INDEX, REPO)} 加一条")
        return 1

    explain = args.explain
    print(f"nt_find: {len(hits)} hit(s) for {' '.join(words)}\n")
    for sc, t in hits:
        ro = "只读" if t.get("read_only") else "⚠ 有副作用"
        print(f"── {t['id']}  [{KIND_LABEL.get(t['kind'], t['kind'])}] {ro}")
        print(f"   跑: {t['tool']}")
        if explain:
            print(f"   触发词: {', '.join(t.get('query', []))}")
            if t.get("spec"):
                print(f"   规范: {t['spec']}")
            if t.get("when_not"):
                print(f"   ⛔ 何时别用: {t['when_not']}")
        print()
    if not explain:
        top = hits[0][1]
        print(f"最快：{top['tool']}")
        if top.get("when_not"):
            print(f"⛔ {top['when_not']}")
        print(f"（--explain 看全部命中与禁用条件）")
    return 0


def cmd_audit(data, args):
    """校验索引与磁盘一致 —— 索引本身也会漂移（这是它自己的门）。"""
    problems = []
    for t in data["tasks"]:
        tool = t["tool"]
        # 抽命令里的脚本路径（第一个看起来像路径的 token）
        # 只认 `scripts/...` 开头的 token，且切掉行尾标点（`;` `&&` `|`）。
        # 2026-09-28 实测踩坑：naive 的 `"/" in p` 判据把 `nt_mem_gate.sh;`
        # 整段（含分号）当成路径，--audit 误报「工具不存在」——自门先错。
        path = None
        for p in tool.split():
            p = p.strip(";&|")
            if p.startswith("scripts/"):
                path = p
                break
        if path is None and t["tool"].startswith("make "):
            # make target：查 Makefile 是否真的声明了它
            tgt = t["tool"].split()[1]
            mk = os.path.join(REPO, "Makefile")
            if os.path.isfile(mk):
                with open(mk, encoding="utf-8", errors="replace") as fh:
                    body = chr(10) + fh.read()
                if (chr(10) + tgt + ":") not in body:
                    problems.append(f"{t['id']}: Makefile 无 target '{tgt}'")
            continue
        if path and not os.path.exists(os.path.join(REPO, path)):
            problems.append(f"{t['id']}: 工具不存在 {path}")
    n = len(data["tasks"])
    if problems:
        print(f"nt_find --audit: {n} tasks, {len(problems)} problem(s)")
        for p in problems:
            print("  " + p)
        return 1
    print(f"nt_find --audit: {n} tasks, 全部工具路径存在 ✅")
    return 0


def main():
    ap = argparse.ArgumentParser(
        description="按任务意图查工具（替代读 AGENTS.md 全文人肉匹配）",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("words", nargs="*", help="自然语言任务描述，如「死锁」「worktree 磁盘」")
    ap.add_argument("--list", action="store_true", help="列出全部任务")
    ap.add_argument("--gate", action="store_true", help="配合 --list：只列门禁")
    ap.add_argument("--explain", action="store_true", help="显示全部命中 + 禁用条件")
    ap.add_argument("--audit", action="store_true", help="校验索引与磁盘一致")
    args = ap.parse_args()

    data = load()
    if args.audit:
        return cmd_audit(data, args)
    if args.list:
        return cmd_list(data, args)
    if not args.words:
        ap.print_help()
        return 1
    return cmd_search(data, args)


if __name__ == "__main__":
    sys.exit(main())
