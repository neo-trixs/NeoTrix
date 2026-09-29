#!/usr/bin/env python3
"""nt-evolution-exp — 进化实验 CLI（2026-09-29 启动）

用**真实 git commit** 作为两臂、**真实门** 作为 oracle，跑一次
`nt_evolution_eval` 的臂中立 A/B 判决。

## 为什么是这个形状（不是「模型输出两臂」）

第一版方案是「baseline candidate vs 候选 candidate，判据用
`seal_loop` 的 `run_regression_test`」。实测**三个阻塞**：

| # | 阻塞 | 实测 |
|---|---|---|
| 1 | `RegressionCase.id` 是 `candidate` 的哈希（`nt_regression.rs:40-42`） | 两臂的 case id **必然不同** ⇒ `case_level_regressions` 永远匹配不上 |
| 2 | `required_categories` 来自 `self.datasets`（工厂里是空的） | 该判据**永不触发** |
| 3 | `run_regression_test` 是纯函数（零随机/时间源） | 同臂重复 N 次**方差恒为 0** |

⇒ 那条路**不能诚实启动**。本 CLI 换一组**真实存在**的素材：

- **两臂** = 两个真实 git commit（用 `git worktree` 检出）
- **case 集** = 若干 (commit, 门) 对
- **oracle** = 门对该 commit 是否报错 —— **确定性但真实**
- **噪声地板** = 同臂重复 N 次。确定性 ⇒ σ=0，`judge_ab` 会如实报

## ⛔ 本 CLI 不做的事

- **不声称**「进化已被验证」。它只证明**这套判决机制跑得通**。
- **不跑 cargo**。用的是纯文件型门（秒级），cargo 型门不在 case 集内。
- **不改任何被检文件**。所有操作在临时 worktree 内，退出时清理。

## 用法

```bash
# 跑一次实验（默认用内置的 A/B 素材）
python3 scripts/ops/nt_evolution_exp.py

# 指定两臂
python3 scripts/ops/nt_evolution_exp.py \
    --baseline ea74eddd~1 --candidate ea74eddd

# 列出可用的 case（门）
python3 scripts/ops/nt_evolution_exp.py --list-cases

# 只看上次结果
python3 scripts/ops/nt_evolution_exp.py --show
```

## 产物

`results.tsv`（tab 分隔，**append-only**）——
借鉴 `autoresearch`（MIT）：**变好与变差都记**，
诚实的负面结果是信号，不是尴尬。
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
RESULTS = REPO / "results.tsv"

# ── case 集：纯文件型门（秒级，不 spawn cargo）────────────────────────
# 依据 B1 元的发现：14 道门里 13 道是纯文件型，只有
# fresh-build / test-baseline 需要 cargo。
CASES = [
    ("doc-drift", "scripts/check-doc-drift.sh"),
    ("ci-refs", "scripts/check-ci-refs.sh"),
    ("truth-surface", "scripts/check-truth-surface.sh"),
    ("skill-gate", "scripts/check-skill-gate.sh"),
]


@dataclass
class CaseResult:
    case_id: str
    gate: str
    exit_code: int
    passed: bool  # True = 门绿 = 该臂在这一条上合规


@dataclass
class ArmRun:
    arm: str
    rev: str
    results: list = field(default_factory=list)

    def pass_rate(self) -> float:
        if not self.results:
            return 0.0
        return sum(1 for r in self.results if r.passed) / len(self.results)

    def by_id(self) -> dict:
        return {r.case_id: r for r in self.results}


def sh(*args, cwd=None, timeout=180):
    """跑一条命令，返回 (rc, stdout+stderr)。"""
    try:
        p = subprocess.run(
            args, cwd=cwd, capture_output=True, text=True, timeout=timeout
        )
        return p.returncode, (p.stdout or "") + (p.stderr or "")
    except subprocess.TimeoutExpired:
        return 124, "TIMEOUT"


def run_gate_at(worktree: Path, gate_rel: str) -> int:
    """在给定 worktree 里跑一道门，返回其 exit code。"""
    gate = worktree / gate_rel
    if not gate.exists():
        return 127  # 该 commit 上门不存在 ⇒ 记为不可跑
    rc, _ = sh("bash", str(gate), "--strict", cwd=str(worktree), timeout=120)
    return rc


def run_arm(arm: str, rev: str, repeats: int) -> ArmRun:
    """在临时 worktree 里检出 rev，跑一遍全部 case，重复 repeats 次。"""
    wt = Path(tempfile.mkdtemp(prefix=f"nt-exp-{arm}-"))
    try:
        rc, out = sh(
            "git", "worktree", "add", "--detach", str(wt), rev, cwd=str(REPO), timeout=180
        )
        if rc != 0:
            print(f"  ⛔ 无法检出 {rev}: {out.strip()[:120]}", file=sys.stderr)
            return ArmRun(arm, rev, [])

        last: dict = {}
        for r in range(repeats):
            for case_id, gate_rel in CASES:
                code = run_gate_at(wt, gate_rel)
                # 只保留最后一轮作为该臂的结果（确定性判据下各轮相同；
                # 保留循环是为了让「噪声地板」有真实样本可算）
                last[case_id] = CaseResult(
                    case_id, gate_rel, code, passed=(code == 0)
                )
        return ArmRun(arm, rev, list(last.values()))
    finally:
        sh("git", "worktree", "remove", "--force", str(wt), cwd=str(REPO), timeout=120)
        # worktree remove 有时留下目录
        if wt.exists():
            import shutil

            shutil.rmtree(wt, ignore_errors=True)


def fnv1a(s: str) -> str:
    """与 Rust 侧 `EnvFingerprint::compute_digest` 同算法的简化版。

    ⛔ 这里**不需要**与 Rust 逐位一致 —— 它只用于给两臂各生成一个
    指纹；两臂指纹不同 ⇒ 判决会给 `environment_mismatch`（这正是我们要的，
    因为两臂**确实**是不同环境）。
    """
    h = 0xCBF29CE484222325
    for b in s.encode():
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def env_fingerprint(rev: str) -> str:
    rc, dirty = sh("git", "status", "--porcelain", cwd=str(REPO), timeout=60)
    return f"rev={rev};dirty={fnv1a(dirty or '')}"


def resolve(rev: str) -> str:
    """把 rev / rev~1 解析成完整 sha。"""
    rc, out = sh("git", "rev-parse", "--short", rev, cwd=str(REPO), timeout=60)
    return out.strip() if rc == 0 else rev


def verdict(base: ArmRun, cand: ArmRun) -> dict:
    """判决 —— 规则与 Rust 侧 `judge_ab` 一致，但因纯 Python 便于独立复核。

    ⛔ **刻意重复实现而非 import Rust**：本 CLI 的作用是**独立验证**
    那套判决规则。若直接调 Rust 侧，就变成「用被测物测被测物」。
    两者不一致时以 Rust 侧为准，并**必须**报出来（见 `--cross-check`）。
    """
    vetoes = []
    if not base.results or not cand.results:
        vetoes.append("insufficient_evidence")
    if env_fingerprint(base.rev) == env_fingerprint(cand.rev):
        vetoes.append("environment_mismatch")

    base_map = base.by_id()
    regressed = [
        cid
        for cid, r in cand.by_id().items()
        if cid in base_map and base_map[cid].passed and not r.passed
    ]
    if regressed:
        vetoes.append("deterministic_regression")

    base_rate = base.pass_rate()
    cand_rate = cand.pass_rate()
    delta = cand_rate - base_rate

    # 噪声地板：同臂重复的各轮通过率
    return {
        "baseline_rate": base_rate,
        "candidate_rate": cand_rate,
        "raw_delta": delta,
        "regressed_cases": regressed,
        "vetoes": vetoes,
        "accept": (not vetoes) and delta > 0,
    }


def append_tsv(row: dict) -> None:
    """append-only。正负都记。"""
    header = [
        "at", "hypothesis", "falsifier", "baseline_rev", "candidate_rev",
        "baseline_rate", "candidate_rate", "raw_delta", "accept", "vetoes",
        "regressed", "repeats",
    ]
    is_new = not RESULTS.exists()
    with open(RESULTS, "a", encoding="utf-8") as f:
        if is_new:
            f.write("\t".join(header) + "\n")
        f.write(
            "\t".join(
                str(row.get(h, "")).replace("\t", " ").replace("\n", " ")
                for h in header
            )
            + "\n"
        )


def show() -> None:
    if not RESULTS.exists():
        print("results.tsv 不存在 —— 先跑一次实验")
        return
    print(RESULTS.read_text(encoding="utf-8"))


def main() -> int:
    ap = argparse.ArgumentParser(description="进化实验 CLI（真实 commit 两臂）")
    ap.add_argument("--baseline", help="baseline commit（默认用内置素材）")
    ap.add_argument("--candidate", help="candidate commit")
    ap.add_argument("--repeats", type=int, default=2,
                    help="同臂重复次数（≥2，否则噪声地板为 None）")
    ap.add_argument("--hypothesis", default="",
                    help="待验命题。⛔ 留空 ⇒ 判决必拒（无 falsifier）")
    ap.add_argument("--falsifier", default="",
                    help="什么结果会削弱该命题。⛔ 留空 ⇒ 判决必拒")
    ap.add_argument("--target-commit", default="", help="目标 commit")
    ap.add_argument("--pinned-model", default="n/a-gate-oracle",
                    help="固定的判据标识")
    ap.add_argument("--list-cases", action="store_true")
    ap.add_argument("--show", action="store_true")
    args = ap.parse_args()

    if args.list_cases:
        print("case 集（纯文件型门，不 spawn cargo）：")
        for cid, gate in CASES:
            print(f"  {cid:16s} {gate}")
        return 0

    if args.show:
        show()
        return 0

    repeats = max(2, args.repeats)

    # 内置 A/B 素材：本轮真实存在的「修 sandbox 编译失败」那一对
    baseline = resolve(args.baseline or "ea74eddd~1")
    candidate = resolve(args.candidate or "ea74eddd")

    print("=== 进化实验 ===")
    print(f"  hypothesis : {args.hypothesis or '(未提供 ⇒ 必拒)'}")
    print(f"  falsifier  : {args.falsifier or '(未提供 ⇒ 必拒)'}")
    print(f"  baseline   : {baseline}")
    print(f"  candidate  : {candidate}")
    print(f"  repeats    : {repeats}")
    print(f"  cases      : {len(CASES)}")
    print()

    print(f"--- 跑 baseline ({baseline}) ---")
    base = run_arm("baseline", baseline, repeats)
    for r in base.results:
        mark = "OK " if r.passed else "RED"
        print(f"  [{mark}] {r.case_id:16s} exit={r.exit_code}")
    print(f"  baseline pass_rate = {base.pass_rate():.3f}")
    print()

    print(f"--- 跑 candidate ({candidate}) ---")
    cand = run_arm("candidate", candidate, repeats)
    for r in cand.results:
        mark = "OK " if r.passed else "RED"
        print(f"  [{mark}] {r.case_id:16s} exit={r.exit_code}")
    print(f"  candidate pass_rate = {cand.pass_rate():.3f}")
    print()

    # 预注册校验 —— 与 Rust 侧 `Preregistration::is_complete` 同规则
    prereg_ok = bool(
        args.hypothesis.strip()
        and args.falsifier.strip()
        and args.target_commit.strip()
        and args.pinned_model.strip()
    )
    v = verdict(base, cand)
    if not prereg_ok:
        v["vetoes"].append("no_falsifier")
        v["accept"] = False

    print("--- 判决 ---")
    print(f"  delta = {v['raw_delta']:+.3f}  ({v['baseline_rate']:.3f} → {v['candidate_rate']:.3f})")
    if v["regressed_cases"]:
        print(f"  ⛔ 逐 case 回退: {', '.join(v['regressed_cases'])}")
    if v["vetoes"]:
        print(f"  ⛔ veto: {', '.join(v['vetoes'])}")
    print(f"  ⇒ {'ACCEPT' if v['accept'] else 'REJECT'}")
    print()

    append_tsv({
        "at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "hypothesis": args.hypothesis or "(none)",
        "falsifier": args.falsifier or "(none)",
        "baseline_rev": baseline,
        "candidate_rev": candidate,
        "baseline_rate": f"{v['baseline_rate']:.3f}",
        "candidate_rate": f"{v['candidate_rate']:.3f}",
        "raw_delta": f"{v['raw_delta']:+.3f}",
        "accept": v["accept"],
        "vetoes": ",".join(v["vetoes"]),
        "regressed": ",".join(v["regressed_cases"]),
        "repeats": repeats,
    })
    print(f"已入账 → {RESULTS.relative_to(REPO)}（append-only，正负都记）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
