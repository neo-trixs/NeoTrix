#!/usr/bin/env python3
"""
nt_ci_wiring — CI 步骤接线的结构性守门。

# 为什么必须存在

2026-10-07 实测才发现的缺口：`desktop` job 里 9 道读 `neobot-ui/dist` 的门
**排在 `Build frontend` 之前** ⇒ 干净检出里 dist 不存在 ⇒ 那 9 道必红 ⇒
后续 `cargo test` 步骤**从未执行过**。
而同一份 ci.yml 里声称「9 道 UI 门**首次在 CI 真正生效**」。
⇒「写进配置」≠「真的能跑」。这是结构性缺陷，需要一道**独立于门本身**的
  结构检查来防止复发。

# 它查什么（三条，每条都可证伪）

  (O) 依赖排序：任何 run 里引用了 `dist` / `UI_DIST` / `frontendDist` 的步骤，
      必须排在某个运行 `vite build` / `pnpm run build` / `pnpm build` 的步骤**之后**。
      ⛔ 只查「读 dist 的前面是否有 build」—— 不查「build 是否成功」
      （那是 cargo/pnpm 的职责，不是本门）。

  (U) 不可达检测：一个步骤若是 `continue-on-error: true`，或其 `if:` 条件里
      出现永假形态（实测没见过，但留着），则视为**结构上可缺席** ⇒ 判红。
      （GitHub Actions 语义：continue-on-error 的步骤失败不红，等于把门删了。）

  (L) 本地↔CI 双向对账：`scripts/ops/` 下所有可执行的门脚本（.mjs / .sh / .py），
      必须至少**一个**出现在某个 workflow 的 run 里，**或**在
      `nt_gate_coverage.py` 的豁免表里有 entry。
      ⇒ 本窗口第 1 条修复抓的正是这个缺口（11 道门不在任何 workflow）。

# ⛔ 本门不做什么

不验证「门在真实 runner 上绿」—— 那是 CI 的事，本门无法触达。
本门只保证「**结构上**不可能跑不到、不可能在缺依赖时运行、不可能被无声跳过」。
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
WF = os.path.join(ROOT, ".github", "workflows")
OPS = os.path.join(ROOT, "scripts", "ops")
COVERAGE = os.path.join(ROOT, "scripts", "ops", "nt_gate_coverage.py")

# `run:` 里出现这些标记 ⇒ 该步骤**读取** dist 或与构建产物相关
_DIST_MARKERS = ("dist", "UI_DIST", "frontendDist", "neobot-ui/dist", "/dist/")
# `run:` 里出现这些标记 ⇒ 该步骤**产出** dist
_BUILD_MARKERS = ("vite build", "pnpm run build", "pnpm build", "tsc --noEmit")


def _yaml_steps() -> list[tuple[str, dict]]:
    """返回 [(workflow_name, step_dict)]，逐文件逐 step。"""
    out: list[tuple[str, dict]] = []
    if not os.path.isdir(WF):
        return out
    try:
        import yaml  # type: ignore
    except Exception:  # noqa: BLE001
        # 没有 pyyaml 时退化为纯文本行扫描
        for fn in sorted(os.listdir(WF)):
            if not fn.endswith((".yml", ".yaml")):
                continue
            with open(os.path.join(WF, fn), encoding="utf-8") as fh:
                content = fh.read()
            # 粗略抽 step name + run，够排序检查用
            names = re.findall(r"- name:\s*(.+)", content)
            runs = re.findall(r"\brun:\s*(.*?)(?=\n\s*- \w|\Z)", content, re.S)
            for n, r in zip(names, runs):
                out.append((fn, {"name": n.strip(), "run": r}))
        return out
    for fn in sorted(os.listdir(WF)):
        if not fn.endswith((".yml", ".yaml")):
            continue
        try:
            with open(os.path.join(WF, fn), encoding="utf-8") as fh:
                doc = yaml.safe_load(fh)
        except Exception:  # noqa: BLE001
            continue
        jobs = (doc or {}).get("jobs") or {}
        for job in jobs.values():
            for st in (job or {}).get("steps") or []:
                if isinstance(st, dict):
                    st = dict(st)
                    st["_wf"] = fn
                    out.append((fn, st))
    return out


def _read(s: dict) -> str:
    return str(s.get("run") or "")


def _script_reads_dist(run_text: str) -> bool:
    """解析 run 文本里引用的本地门脚本，读其源码判断是否**硬依赖** `neobot-ui/dist`。

    ⭐ 2026-10-07 收紧：只有显式「要求 dist 就位」才是硬依赖。
      · `requireFreshDist(` / `neobotDistFresh(` = 公用件，缺 dist 就 exit 1 ⇒ 硬依赖
      · 存在性判断 + process.exit/sys.exit ⇒ 硬依赖
    ⛔ 仅出现 "dist" 路径、`if os.path.isdir(dist)` 的**容忍性**读取
      （nt_neobot_ui_wiring 的产物纯净门「若有 dist」）、或把 dist 当**排除目录**
      的扫描（nt_feature_viability）—— 全部不算硬依赖，对它们判红是误报
      （本轮这两例都误报过）。
    """
    if "neobot-ui/dist" in run_text or "frontendDist" in run_text or "/dist/" in run_text:
        return True
    for m in re.finditer(r"scripts/(?:ops/)?[\w./-]+\.(?:mjs|sh|py)", run_text):
        rel = m.group(0)
        for cand in (rel, os.path.join(ROOT, rel)):
            if os.path.isfile(cand):
                try:
                    txt = open(cand, encoding="utf-8", errors="replace").read()
                except Exception:  # noqa: BLE001
                    break
                # ⭐ 唯一精确信号：16 道读 dist 的门统一走
                #   `scripts/ops/nt_dist_freshness.mjs` 的 `requireFreshDist(` /
                #   `neobotDistFresh(`，缺 dist 就 exit 1 ⇒ 硬依赖。
                #   ⛔ 刻意不匹配裸 "dist" 子串 / `isdir` / `exit 1` 的组合 ——
                #     那会把 `nt_neobot_ui_wiring`（容忍性读）、`nt_feature_viability`
                #     （只扫不读）、以及 `nt_security_wiring`（为别的原因 exit 1）
                #     全部误判成硬依赖（本轮这三类都误报过）。
                if "requireFreshDist(" in txt or "neobotDistFresh(" in txt:
                    return True
                break
    return False


def check_order(steps: list[tuple[str, dict]]) -> list[str]:
    """(O)：读 dist 的步骤前面必须出现过 build 步骤（同一 job 内按文本顺序近似）。"""
    bad, seen_build = [], False
    for fn, st in steps:
        r = _read(st)
        if any(m in r for m in _BUILD_MARKERS):
            seen_build = True
            continue  # build 步骤自己是产出，不计入「读」
        if _script_reads_dist(r) and not seen_build:
            bad.append(
                f"  ❌ {fn} /「{st.get('name','(无名)')[:40]}」会硬依赖 neobot-ui/dist，"
                f"但它前面没有任何 build 步骤 ⇒ 干净检出里 dist 不存在，必红。"
            )
    return bad


def check_unreachable(steps: list[tuple[str, dict]]) -> list[str]:
    """(U)：continue-on-error 的步骤 = 失败不红 = 结构上可缺席。"""
    bad = []
    for fn, st in steps:
        # ⛔ 只对**门**步骤查 —— 外部 action 的 dry-run（dummy key + 明确
        #   允许失败）是合法 smoke，对它判红是误报。判据面限定在
        #   run 里引用了 scripts/ops/ 或 scripts/check- 的步骤。
        r = _read(st)
        if "scripts/ops/" not in r and "scripts/check-" not in r and "nt_check_" not in r and "nt_gate" not in r:
            continue
        coe = st.get("continue-on-error")
        if coe is True or str(coe).lower() == "true":
            bad.append(f"  ❌ {fn} /「{st.get('name','(无名)')[:40]}」是门但设了 continue-on-error ⇒ 失败不红，等于删了这道门。")
        c = str(st.get("if") or "")
        if c and re.search(r"\bfalse\b", c) and not re.search(r"\btrue\b", c):
            bad.append(f"  ❌ {fn} /「{st.get('name','(无名)')[:40]}」if: 恒假形态 ⇒ 结构上不可达。")
    return bad


def check_coverage(steps: list[tuple[str, dict]]) -> list[str]:
    """(L)：scripts/ops 下的门脚本，必须在某 step 的 run 里被引用，或在豁免表里登记。"""
    if not os.path.isdir(OPS):
        return []
    referenced: set[str] = set()
    blob = "\n".join(_read(st) for _, st in steps)
    for fn in os.listdir(OPS):
        if fn.endswith((".mjs", ".sh", ".py")) and os.path.isfile(os.path.join(OPS, fn)):
            if fn in blob or fn.replace(".mjs", "").replace(".py", "").replace(".sh", "") in blob:
                referenced.add(fn)
    # 豁免表
    exempt: set[str] = set()
    if os.path.isfile(COVERAGE):
        try:
            txt = open(COVERAGE, encoding="utf-8", errors="replace").read()
            exempt = set(re.findall(r"'([^']+)':", txt))
        except Exception:  # noqa: BLE001
            pass
    bad = []
    for fn in sorted(os.listdir(OPS)):
        if not fn.endswith((".mjs", ".sh", ".py")):
            continue
        stem = fn.rsplit(".", 1)[0]
        if fn not in referenced and stem not in referenced:
            # 豁免表可能用不带扩展名的 stem
            if stem not in exempt and fn not in exempt and fn.replace(".mjs", "").replace(".py", "").replace(".sh", "") not in exempt:
                # 工具型（需传参才能跑）不算门 —— 但没法静态断定，先报告，不判红
                bad.append(f"  ⓘ  scripts/ops/{fn} 不在任何 workflow 的 run 里，豁免表也未登记 ⇒ 无 CI 覆盖")
    return bad if bad else []


def main() -> int:
    print("=== CI 步骤接线结构检查 ===\n")
    steps = _yaml_steps()
    if not steps:
        print("  ⛔ 没在 .github/workflows 读到任何 step ⇒ 判据无从执行 ⇒ 判红（不假装通过）")
        return 1

    b_order = check_order(steps)
    b_unreach = check_unreachable(steps)
    b_cover = check_coverage(steps)

    print("── (O) 依赖排序：读 dist 必须在 build 之后 ──")
    print("  ✅ 通过" if not b_order else "\n".join(b_order))
    print("\n── (U) 不可达/静默放行检测 ──")
    print("  ✅ 通过" if not b_unreach else "\n".join(b_unreach))
    print("\n── (L) 本地门 ↔ CI 双向对账（不判红，只报告覆盖缺口）──")
    print("  ✅ 每道 ops 门都在 CI 或豁免表里" if not b_cover else "\n".join(b_cover))

    hard_fail = len(b_order) + len(b_unreach)
    if hard_fail:
        print(f"\nFAIL: {hard_fail} 条结构性缺陷")
        return 1
    print("\nPASS: 结构上没有排序错误、没有静默放行的步骤。")
    return 0


if __name__ == "__main__":
    sys.exit(main())