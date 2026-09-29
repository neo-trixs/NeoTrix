#!/usr/bin/env python3
"""nt_feature_matrix.py — 测量「每个 feature 能否独立编译」，把腐烂变成可复现数字。

## 为什么需要它（R-FEAT-1）

2026-09-29 实测事故链：
1. `nt_shield_sandbox` 的 import 少一层模块 ⇒ `--features sandbox` 编译失败。
2. 该 feature 不在默认集里，`cargo check --lib` 编不到 ⇒ 缺陷长期存活。
3. 唯一的 `full-features` CI job 没有 `continue-on-error`，**CI 必红但没人读结果**。

修完 sandbox 后顺藤摸瓜，发现同类腐烂有 **4 个家族 / 20 个错误**
（`desktop` 10、`audio-decode` 6、`video-decode` 4、`self_model` 3），
其中 3 个 feature **在任何 workflow 里都没被编过**。

**本工具的价值：把「不知道烂了多久」变成「一个可复现的数字」。**
不修复任何东西 —— 修不修、删不删是产品决策，不是工具该做的。

## 成本意识（这是设计约束，不是优化）

实测单 feature 编译耗时 10s~3m12s（sandbox 最慢），
17 个 feature 串行不可接受。故：
- `--jobs N` 并发（默认 min(4, cpu)）
- 已测过的 feature 记入 `.cache`，默认**跳过**（`--recheck` 强制重测）
- `--list` 只读 Cargo.toml，**零编译**，用于「先看有哪些」

## 关键设计：区分「编译失败」与「本来就该失败」

- `--features <f>` **单独**编：某些 feature 设计上就依赖别的（如 `full` 是聚合）。
  本工具默认**跳过聚合 feature**（`full`），它们由 `--all-features` 覆盖。
- `crystal_game_disabled` / `wip` 等零门控 feature 单编必然「通过」但**无意义**，
  工具会把「声明了却零门控」单独标出（那才是死 feature 信号）。

## 用法
    # 零编译：列出 feature 与门控规模，标出死 feature
    python3 scripts/ops/nt_feature_matrix.py --list

    # 测单个（快速验证）
    python3 scripts/ops/nt_feature_matrix.py --check sandbox

    # 测全部真门控 feature（并发，缓存）
    python3 scripts/ops/nt_feature_matrix.py --check-all

    # 强制重测 + 测 all-features
    python3 scripts/ops/nt_feature_matrix.py --check-all --recheck --include-agg

退出码：0=全部通过；1=有失败（供 CI 提示）；2=用法/环境错。
"""

from __future__ import annotations

import argparse
import concurrent.futures as cf
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

# 聚合 feature：单独编无意义（它们是「把别人打进来」的全集）
AGGREGATE = {"full"}
# 明显是占位/开关的 feature，不是真能力
PLACEHOLDER = {"crystal_game_disabled", "wip", "default"}
CACHE = Path(".cache/nt_feature_matrix.json")
ERRSUB = re.compile(r"^error(\[[A-Z]\d+\])?:", re.M)


def read_features(toml: Path) -> dict[str, list[str]]:
    s = toml.read_text(encoding="utf-8")
    if "[features]" not in s:
        return {}
    body = s.split("[features]", 1)[1]
    m = re.search(r"\n\[(?!features)", body)
    if m:
        body = body[: m.start()]
    out: dict[str, list[str]] = {}
    for fm in re.finditer(r"^([a-z0-9_-]+)\s*=\s*\[([^\]]*)\]", body, re.M | re.S):
        out[fm.group(1)] = re.findall(r'"([^"]+)"', fm.group(2))
    return out


def gate_counts(src: Path) -> dict[str, int]:
    """统计每个 feature 门控了多少 .rs 文件（死 feature 检测依据）。"""
    import collections

    c: collections.Counter = collections.Counter()
    for f in src.rglob("*.rs"):
        t = f.read_text(encoding="utf-8", errors="replace")
        for m in re.finditer(r'#\[cfg(?:_attr)?[^\]]*?feature\s*=\s*"([^"]+)"', t):
            c[m.group(1)] += 1
    return dict(c)


def classify(declared: list[str], gates: dict[str, int]) -> list[dict]:
    rows = []
    for f in sorted(declared):
        n = gates.get(f, 0)
        if f in AGGREGATE:
            kind = "aggregate"
        elif f == "default":
            # `default` 是 cargo 的默认集语义，永远不会有 #[cfg(feature="default")]
            # ⇒ 拿「零门控」判它死是**误判**（2026-09-29 自测时发现）。
            kind = "default-set"
        elif f in PLACEHOLDER:
            kind = "placeholder"
        elif n == 0:
            kind = "dead"  # 真正声明了却零门控
        else:
            kind = "real"
        rows.append({"feature": f, "gated_sites": n, "kind": kind})
    return rows


def run_check(feat: str, root: Path, jobs: int, extra: str = "") -> dict:
    cmd = f"cargo check --features {feat} --lib -p neotrix {extra}".split()
    t0 = time.time()
    try:
        p = subprocess.run(cmd, cwd=root, capture_output=True, text=True, timeout=1800)
        out = p.stdout + p.stderr
        return {
            "feature": feat,
            "ok": p.returncode == 0,
            "errors": sorted(set(ERRSUB.findall(out) and re.findall(r"^error[^\n]*", out, re.M))),
            "secs": round(time.time() - t0, 1),
        }
    except subprocess.TimeoutExpired:
        return {"feature": feat, "ok": False, "errors": ["TIMEOUT"], "secs": 1800.0}


def main() -> int:
    ap = argparse.ArgumentParser(description="测量每个 feature 能否独立编译（不修复任何东西）")
    ap.add_argument("--list", action="store_true", help="零编译：列 feature + 门控规模 + 死 feature")
    ap.add_argument("--check", metavar="FEAT", help="测单个 feature")
    ap.add_argument("--check-all", action="store_true", help="测全部真门控 feature（并发）")
    ap.add_argument("--include-agg", action="store_true", help="也测聚合 feature（full）与 --all-features")
    ap.add_argument("--recheck", action="store_true", help="忽略缓存，强制重测")
    ap.add_argument("--jobs", type=int, default=0, help="并发数（默认 min(4,cpu)）")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    root = Path.cwd()
    toml = root / "neotrix-core" / "Cargo.toml"
    src = root / "neotrix-core" / "src"
    if not toml.exists():
        print("[feature-matrix] 未在仓库根运行（或 neotrix-core 不存在）", file=sys.stderr)
        return 2

    declared = read_features(toml)
    gates = gate_counts(src)
    rows = classify(list(declared), gates)

    # ---- --list：零编译 ----
    if args.list:
        if args.json:
            print(json.dumps(rows, ensure_ascii=False, indent=2))
        else:
            print(f"[feature-matrix] 声明 {len(rows)} 个 feature，源码内真门控 {len([r for r in rows if r['kind']=='real'])} 个\n")
            print(f"  {'feature':<24}{'门控处':>6}  种类")
            for r in rows:
                mark = {
                    "aggregate": "聚合(单独编无意义)",
                    "dead": "❌ 死 feature(零门控)",
                    "placeholder": "占位开关(非能力)",
                    "default-set": "默认集(cargo 语义)",
                }.get(r["kind"], "真门控")
                print(f"  {r['feature']:<24}{r['gated_sites']:>6}  {mark}")
            print("\n  ⚠ 本工具**不修复**任何东西 —— 修不修、删不删是产品决策。")
            print("    本工具只把「不知道烂了多久」变成「一个可复现的数字」。")
        return 0

    jobs = args.jobs or min(4, os.cpu_count() or 2)
    cache: dict = {}
    if CACHE.exists() and not args.recheck:
        try:
            cache = json.loads(CACHE.read_text(encoding="utf-8")).get("results", {})
        except Exception:
            cache = {}

    # ---- 单个 ----
    if args.check:
        if args.check not in declared:
            print(f"[feature-matrix] 未知 feature: {args.check}", file=sys.stderr)
            return 2
        r = run_check(args.check, root, jobs)
        print(f"  {args.check}: {'✅ 通过' if r['ok'] else '❌ 失败'} ({r['secs']}s)")
        for e in r["errors"][:6]:
            print(f"    {e}")
        return 0 if r["ok"] else 1

    # ---- 全部 ----
    targets = [r["feature"] for r in rows if r["kind"] == "real"]
    if args.include_agg:
        targets += sorted(AGGREGATE)
    todo = [t for t in targets if args.recheck or t not in cache]

    if not todo:
        print(f"[feature-matrix] 全部 {len(targets)} 个 feature 已在缓存中；--recheck 可强制重测")

    results = dict(cache)
    if todo:
        print(f"[feature-matrix] 并发 {jobs} 路编译 {len(todo)}/{len(targets)} 个 feature…", file=sys.stderr)
        with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
            futs = {ex.submit(run_check, f, root, jobs): f for f in todo}
            for fut in cf.as_completed(futs):
                r = fut.result()
                results[r["feature"]] = r
                mark = "✅" if r["ok"] else "❌"
                print(f"  {mark} {r['feature']:<24}{r['secs']:>7}s", file=sys.stderr)

    CACHE.parent.mkdir(parents=True, exist_ok=True)
    CACHE.write_text(
        json.dumps({"ts": int(time.time()), "results": results}, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    failed = [r for r in results.values() if not r["ok"]]
    if args.json:
        print(json.dumps({"results": results, "failed": len(failed)}, ensure_ascii=False, indent=2))
        return 1 if failed else 0

    print(f"\n[feature-matrix] 结果（{len(results)} 个已测，{len(failed)} 个失败）\n")
    for f in targets:
        r = results.get(f)
        if not r:
            continue
        mark = "✅" if r["ok"] else "❌"
        print(f"  {mark} {f:<24}{r['secs']:>7}s")
        for e in r["errors"][:3]:
            print(f"       {e[:100]}")
    if failed:
        print(f"\n  ❌ {len(failed)} 个 feature 编译失败 —— **本工具不修复**，请按产品决策处理：")
        print(f"     ① 有无真实使用方？无 ⇒ 删（比修划算）")
        print(f"     ② 有 ⇒ 跟依赖升版还是代码改？（如 ed25519_dalek / symphonia API 漂移）")
        print(f"     ③ 修完再考虑加 CI 门 —— 现在加会立刻红")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
