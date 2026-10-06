#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
nt_sdlc_metrics.py — SDLC 过程指标（吸收 Anthropic《AI-native SDLC playbook》
Stage 1-6「How to measure it」的**指标纪律**，用 git 可复算的量落地）。

================================ 判据 ================================
**吸收的是「每个环节都要有先行 + 滞后指标」这条纪律，不是原文的指标本身。**
原文给的是**口径**（如「PR 首次审查耗时」），不是实现。本文件只报**本仓真能
从 git 复算**的量；报不出来的**如实报 UNMEASURED 并写明为何报不出**。

⛔⛔ **禁止把测不出的报成 0。** 「0% 工件覆盖率」与「无法测」是两件事，
  把后者写成前者是本仓明令禁止的 `evals/VERIFICATION.md:8-9`
  （没跑 ≠ 通过）。故每个指标有三种取值之一：`实测值` / `UNMEASURED(原因)` /
  `无基线(新机制，样本 0)`。三者在输出里**视觉可区分**。

================================ 设计约束 ================================
1. **只读**：只跑 `git log/diff/name-only`，不写任何文件，不改 index。
2. **确定性**：同一 commit 图两次跑必同结果（⛔ 不用时间窗的「当前」概念，
   全部锚在 `HEAD` 上）。
3. **防空转**（抄 better-sidebar「a glob that silently matches nothing would
   make this contract vacuous」）：git 拿不到历史 ⇒ 报「无法测量」并 exit 非 0，
   **绝不**静默输出全 0。
4. **不是门**：它是 reporter，不判 pass/fail，因此**不登记进
   gate-registry.tsv**，也不进 CI 阻断（照 `nt_gate_coverage.py` 的判据 ①
   「advisory 就该是本地报告器，接进 CI 只会让恒红的门 = 没有门」）。
   发现路径靠 `.neotrix/task-index.json`（kind=ops）。

用法: python3 scripts/ops/nt_sdlc_metrics.py [--head N] [--json]
  --head N   只看最近 N 个 commit（默认 200；设 0 = 全历史 2622）
  --json     机器可读输出（供未来 dashboard / `check-*` 消费）

退出码: 0 = 指标已算出（含「无基线」这种诚实的零值）
         3 = **无法测量**（git 历史不可读）⇒ 这不是「指标为 0」
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict

REPO_MARK = "docs/plans/"

# 自检时指向临时夹具；正常运行为 None ⇒ 用本仓。
REPO_OVERRIDE = None


def git(*args: str) -> str:
    """Run git in the repo; empty string on any failure (never raises)."""
    cwd = REPO_OVERRIDE or REPO_OF(__file__)
    try:
        r = subprocess.run(
            ("git",) + args,
            capture_output=True, text=True, timeout=120,
            cwd=cwd,
        )
    except (OSError, subprocess.TimeoutExpired):
        return ""
    return r.stdout if r.returncode == 0 else ""


def REPO_OF(src: str) -> str:
    import os
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(src))))


# --------------------------------------------------------------------------
# 指标 1（先行）：工件覆盖率 —— 吸收的机制真的被用了吗？
# --------------------------------------------------------------------------
def metric_artifact_coverage(head_n: int) -> dict:
    """A product-code commit is 'covered' if a docs/plans/ artifact was
    created within the same commit or the 3 commits before it (in FULL
    history order, not in a code-only list).

    Why a window at all: one change legitimately spans a few commits, and the
    template's iron rule (docs/plans/_TEMPLATE.md §4) requires plan.md and the
    diff to move in the SAME commit only when the plan *changes*. Requiring the
    artifact in the same commit as every code commit would report a healthy
    adoption rate as near-zero, which is the same 'metric lies' failure mode.

    ⚠️ 早版把窗口建在「只含 product-code commit」的列表上 ⇒ **工件若单独成一个
    commit（docs-only），它压根不在该列表里，永远进不了窗口** ⇒ 指标恒 0。
    本仓恰恰就是这么提交的（工件与代码分开落）⇒ 该 bug 让指标在真实数据上
    **永远**报 0。是 --self-test 的合成夹具抓出来的，不是读代码看出来的。
    """
    rng = ["-n", str(head_n)] if head_n else []
    # 全历史提交顺序（位置即「前 N 个」的真实含义）
    full = [l for l in git("log", *rng, "--format=%H").split("\n") if l.strip()]
    if not full:
        return {"state": "UNMEASURED", "why": "git 历史读不到 commit 列表"}
    pos = {h: i for i, h in enumerate(full)}

    added = [l for l in git("log", *rng, "--diff-filter=A", "--name-only",
                            "--format=%H", "--", "docs/plans/").split("\n") if l]
    art_commits = {h for h in added if h in pos}

    code_all = [l for l in git("log", *rng, "--format=%H",
                               "--", "neotrix-core/src", "crates").split("\n") if l.strip()]
    # ⚠️ `-n N` 配 pathspec 时，git 数的是**匹配 path 的 N 个 commit**，
    #    而 `full` 是**全部 commit 的前 N 个** ⇒ 两者窗口不同，
    #    `code_all` 会含 `full` 里没有的 sha（实测 KeyError）。
    #    ⇒ 必须按 `pos` 裁剪到同一窗口，否则覆盖率的分母就不是这个窗口的。
    code = [h for h in code_all if h in pos]
    if not code:
        return {"state": "UNMEASURED", "why": "git 历史读不到 product-code commit"}

    covered = 0
    for h in code:
        i = pos[h]
        window = full[max(0, i - 3): i + 1]
        if any(w in art_commits for w in window):
            covered += 1
    n = len(code)
    return {
        "state": "OK",
        "covered": covered,
        "total": n,
        "pct": round(100.0 * covered / n, 1),
        "window": "same commit or ≤3 preceding commits (full history order)",
    }


# --------------------------------------------------------------------------
# 指标 2（先行）：工件→diff 漂移 —— 计划说改的文件与实际改的是否一致
# --------------------------------------------------------------------------
def metric_plan_drift(head_n: int) -> dict:
    """For each artifact that lists files under '## 改动文件', compare against
    the files actually touched by the next commit(s).

    This is the machine-checkable half of _TEMPLATE.md §4's iron rule.
    Only artifacts that HAVE the section are counted — artifacts without it are
    counted separately, never silently folded into 'drift = 0'.
    """
    rng = ["-n", str(head_n)] if head_n else []
    listing = git("log", *rng, "--diff-filter=A", "--name-only",
                  "--format=COMMIT:%H", "--", "docs/plans/")
    blocks = [b for b in listing.split("COMMIT:") if b.strip()]
    analyzed = drift = no_section = 0
    offenders = []

    for b in blocks:
        lines = b.split("\n")
        sha = lines[0].strip()
        path = next((l.strip() for l in lines[1:] if l.strip().endswith(".md")), None)
        if not path or not sha:
            continue
        # ⛔ 排除模板自身：_TEMPLATE.md 里那些路径是**示例**，不是变更清单。
        #    不排除它，drift 分析会把模板的示例路径当成真计划（实测：analyzed=1
        #    的那件就是它）⇒ 指标用假数据报 0%，比不报更坏。
        if path.rsplit("/", 1)[-1] == "_TEMPLATE.md":
            continue
        body = git("show", f"{sha}:{path}")
        if "## 改动文件" not in body:
            no_section += 1
            continue
        sec = body.split("## 改动文件", 1)[1]
        sec = re.split(r"\n##\s", sec)[0]
        # ⚠️ 单个捕获组 + 可选反引号：group(1) 恒不为 None
        #   （早版用双 alternation 导致 group(1)/group(2) 只有一个非 None，
        #    在 .strip() 处炸 AttributeError —— 跑出来才发现，不是想出来的）
        planned = {m.group(1) for m in
                   re.finditer(r"`?([A-Za-z0-9_][\w./-]*\.[A-Za-z0-9]+)`?", sec)}
        planned = {p for p in planned
                   if "/" in p or p.endswith((".rs", ".ts", ".tsx", ".py", ".sh"))}
        if not planned:
            continue
        analyzed += 1
        # the commit that follows the artifact commit, same branch tip
        after = git("log", "-n", "1", "--format=%H", f"{sha}..HEAD", "--",
                    "neotrix-core/src", "crates", "scripts")
        sha_after = after.strip().split("\n")[0].strip() if after.strip() else ""
        if not sha_after:
            continue
        touched = {l.strip() for l in
                   git("show", "--name-only", "--format=", sha_after).split("\n") if l.strip()}
        extra = touched - planned
        if extra:
            drift += 1
            offenders.append({"artifact": path, "next": sha_after[:8],
                              "unlisted": sorted(extra)[:5]})

    if analyzed == 0:
        return {"state": "NO_BASELINE", "why":
                f"{no_section} 件工件无 `## 改动文件` 节（存量不追溯，见 artifact-chain-baseline）",
                "artifacts_seen": no_section}
    return {"state": "OK", "analyzed": analyzed, "drifted": drift,
            "pct": round(100.0 * drift / analyzed, 1), "offenders": offenders[:5]}


# --------------------------------------------------------------------------
# 指标 3（滞后）：返工代理 —— 同一文件在窗口内被反复触碰
# --------------------------------------------------------------------------
def metric_rework(head_n: int) -> dict:
    """Files touched by >= 3 distinct commits inside the window.
    A proxy only — it cannot distinguish 'iterative design' from 'churn'.
    Stated as a proxy in the output so nobody reads it as a DORA metric.
    """
    rng = ["-n", str(head_n)] if head_n else []
    log = git("log", *rng, "--name-only", "--format=C:%H",
              "--", "neotrix-core/src", "crates")
    counts = defaultdict(set)
    cur = None
    for line in log.split("\n"):
        line = line.strip()
        if line.startswith("C:"):
            cur = line[2:]
        elif line and cur and (line.endswith((".rs", ".ts", ".tsx"))):
            counts[line].add(cur)
    if not counts:
        return {"state": "UNMEASURED", "why": "git 历史读不到文件级改动"}
    hot = {f: len(c) for f, c in counts.items() if len(c) >= 3}
    hot = dict(sorted(hot.items(), key=lambda kv: -kv[1])[:5])
    return {"state": "OK", "files_tracked": len(counts), "hot_count": len(hot),
            "pct": round(100.0 * len(hot) / len(counts), 1), "top": hot,
            "caveat": "代理指标：分不清「迭代设计」与「真返工」。不是 DORA。"}


# --------------------------------------------------------------------------
# REVIEW.md §6 声明为「未接线」的三项 —— 如实标注为何测不出
# --------------------------------------------------------------------------
UNMEASURABLE = [
    ("PR 首次审查耗时", "本仓 agent 走本地工作树 + `git commit --only`，"
                        "不用 GitHub PR 流程 ⇒ 无 first-review 事件可测。"),
    ("审查发现被门重复报出的比例", "需要结构化的审查发现台账（每条发现落盘）。"
                                  "目前 REVIEW.md 的发现只存在于对话里，无落盘格式 ⇒ 无数据源。"),
    ("合并前拦下 vs 逃逸到生产的缺陷", "需要事故台账（生产缺陷的登记入口）。本仓无生产环境 ⇒ 分子分母都没有。"),
]


# --------------------------------------------------------------------------
# 自检：防空转（⛔ 否则这就是一个「永远输出 0」的假指标）
# --------------------------------------------------------------------------
SELF_TEST_FIXTURE = """\
## 问题
probe

## 目标
probe

## 影响面
probe

## 约束
probe

## 未决问题
probe

## 改动文件
`neotrix-core/src/probe_a.rs`

## 施工顺序
1. probe

## 风险
probe

## 证据
probe
"""


def self_test() -> int:
    """Build a throwaway git repo with KNOWN answers, then assert the metrics
    report those numbers.

    Why this exists: a reporter that can only ever print 0 is indistinguishable
    from a broken one. The repo has been bitten by exactly that ('a glob that
    silently matches nothing would make this contract vacuous'). Two ground
    truths are asserted:
      A. coverage must go ABOVE 0 when an artifact commit is paired with a
         product-code commit inside the window.
      B. drift must fire when the following commit touches a file the
         artifact never listed.
    """
    import os
    import shutil
    import tempfile

    tmp = tempfile.mkdtemp(prefix="ntx-sdlc-selftest-")
    try:
        def g(*a: str) -> str:
            r = subprocess.run(("git",) + a, cwd=tmp, capture_output=True, text=True)
            return r.stdout.strip()

        g("init", "-q")
        g("config", "user.email", "probe@example.invalid")
        g("config", "user.name", "probe")
        os.makedirs(f"{tmp}/docs/plans", exist_ok=True)
        os.makedirs(f"{tmp}/neotrix-core/src", exist_ok=True)

        def commit(msg: str) -> None:
            g("add", "-A")
            g("commit", "-q", "-m", msg)

        # c1: product code only
        open(f"{tmp}/neotrix-core/src/probe_a.rs", "w").write("// a\n")
        commit("c1 code only")
        # c2: an artifact that lists only probe_a.rs
        open(f"{tmp}/docs/plans/2026-01-01-probe.md", "w").write(SELF_TEST_FIXTURE)
        commit("c2 add artifact")
        # c3: code commit that ALSO touches an unlisted file  => drift
        open(f"{tmp}/neotrix-core/src/probe_b.rs", "w").write("// b\n")
        commit("c3 code again")

        global REPO_OVERRIDE
        REPO_OVERRIDE = tmp
        cov = metric_artifact_coverage(0)
        drift = metric_plan_drift(0)

        fails = []
        if cov.get("state") != "OK":
            fails.append(f"coverage 状态异常: {cov}")
        elif cov.get("pct", 0) <= 0.0:
            fails.append(f"coverage 应 >0（工件与代码同窗）却报 {cov.get('pct')}")

        if drift.get("state") != "OK":
            fails.append(f"drift 状态异常: {drift}")
        elif drift.get("drifted", 0) <= 0:
            fails.append(f"drift 应 >0（下一 commit 多改了未列文件）却报 {drift}")
        else:
            off = drift["offenders"][0]
            if not any("probe_b" in u for o in drift["offenders"] for u in o["unlisted"]):
                fails.append(f"drift 指名了未列文件但不是 probe_b.rs: {off}")

        if fails:
            print("SELF-TEST FAIL:", file=sys.stderr)
            for f in fails:
                print("  -", f, file=sys.stderr)
            return 1
        print(f"SELF-TEST OK — 合成夹具上 coverage={cov['pct']}% "
              f"drift={drift['drifted']}/{drift['analyzed']} "
              f"(unlisted={drift['offenders'][0]['unlisted']}) "
              f"⇒ 两个指标都有判别力，不是恒 0 的假指标")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--head", type=int, default=200,
                    help="commits to look at (0 = all). default 200")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--self-test", action="store_true",
                    help="合成夹具上证明两个指标都有判别力（防空转）")
    a = ap.parse_args()

    if a.self_test:
        return self_test()

    if not git("rev-parse", "HEAD").strip():
        print("sdlc-metrics: 无法读取 git 历史 ⇒ 指标无法计算", file=sys.stderr)
        print("⛔ 这**不是**「指标为 0」。门设计见文件头「防空转」条。", file=sys.stderr)
        return 3

    cov = metric_artifact_coverage(a.head)
    drift = metric_plan_drift(a.head)
    rw = metric_rework(a.head)

    if a.json:
        print(json.dumps({"coverage": cov, "drift": drift, "rework": rw,
                          "unmeasurable": UNMEASURABLE},
                         ensure_ascii=False, indent=2))
        return 0

    print("=== SDLC 过程指标（git 可复算 · 锚在 HEAD）===")
    print(f"窗口: 最近 {a.head or '全部'} 个 commit\n")

    print("[先行 1] 工件覆盖率（吸收机制真的被用了吗）")
    if cov["state"] == "OK":
        print(f"  {cov['pct']}%  ({cov['covered']}/{cov['total']} 个 product-code commit "
              f"在 ±{cov['window']})")
        if cov["pct"] == 0.0:
            print("  ⚠️ 0% —— 机制接上了但还没被使用。这不是失败，是「刚接上」。")
    else:
        print(f"  UNMEASURED —— {cov['why']}")

    print("\n[先行 2] 工件→diff 漂移（_TEMPLATE.md §4 的机器判据）")
    if drift["state"] == "OK":
        print(f"  {drift['drifted']}/{drift['analyzed']} 件工件的下一个 commit 越过了「改动文件」清单")
        for o in drift["offenders"]:
            print(f"    - {o['artifact']} → {o['next']} 多改了 {o['unlisted']}")
    else:
        print(f"  {drift['state']} —— {drift['why']}")

    print("\n[滞后 3] 返工代理（同文件被 ≥3 个 commit 触碰）")
    if rw["state"] == "OK":
        print(f"  {rw['hot_count']}/{rw['files_tracked']} 个源文件 ({rw['pct']}%)")
        for f, n in list(rw["top"].items())[:3]:
            print(f"    - {f}: {n} commits")
        print(f"  ⚠️ {rw['caveat']}")
    else:
        print(f"  UNMEASURED —— {rw['why']}")

    print("\n[如实标注] REVIEW.md §6 声明为「未接线」的三项 —— 至今仍不可测：")
    for name, why in UNMEASURABLE:
        print(f"  · {name}：{why}")

    print("\n⛔ 本文件是**报告器**，不判 pass/fail ⇒ 不进 gate-registry、不进 CI 阻断。")
    print("   口径提醒：工件覆盖率刚接上时必然≈0，**别把它当成熟度汇报**。")
    return 0


if __name__ == "__main__":
    sys.exit(main())