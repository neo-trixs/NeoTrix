#!/usr/bin/env python3
"""⭐⭐⭐ L1 分层裁决的**可证伪信号**门 —— 让「继续用方案乙」这个决策**有牙**。

## 背景（`docs/architecture/CRYSTAL-CORE-LAYERING-RULING-2026-10-03.md`）

裁决判定：`nt_crystal_core`（**20,849 行**的多智能体编排/仲裁/自愈）**留在 L5**；
那 16 处 L1→L5 违规的根因是 ⭐**「L1 的文件在承载认知编排职责」**。

裁决给出两条修法：
· ⭐ **甲**：把这些 shell（TUI / CLI / stdin-human）**移出 L1**
· ⭐ **乙**：承认它们是 L1 的 LLM 门面，经 **L1 自己的 facade** 转出（**已做两批**）

⇒ 裁决**推荐乙**，但 ⭐⭐ **明确写下乙有天花板**：

> 「当 facade 转出符号**逼近整个 `nt_crystal_core`** 时，facade 就成了
> **换名字的整包 re-export** ⇒ 那时必须回头认真考虑甲。」

## ⭐⭐ 本门就是那个天花板的**探测器**

⛔ **若不把阈值变成门**，「继续用乙」会变成一个**没有牙齿的默认**——
每次推进都「看起来没问题」，直到 facade 变成整包 re-export 才被发现。
⇒ ⭐ 本门让它**在越线的那一刻就红**。

## 两个判据（任一越线即 FAIL ⇒ 方案乙的前提被推翻）

| 判据 | 阈值 | 含义 |
|---|---|---|
| `JUDGE_MAX_FACADE_SYMBOLS` | **8** | facade 从 `nt_crystal_core` 转出的**符号数** |
| `JUDGE_MIN_BASELINE` | **10** | `scripts/layer-deps-baseline.txt` 的**基线条数** |

⭐⭐ 第二个判据是⭐**反向**的：基线**降到 10 以下**意味着「16 处违规几乎清完」，
⭐⭐ 而清完的**唯一**途径是 facade 吸收（或甲方案）⇒ ⭐ 那正是「facade 变成整包
re-export」的信号。⇒ ⭐ 两个判据从**两端**夹住同一个结论。

## ⭐ 口径纪律（本日三次踩坑，故写进文件头）
· ⭐ **必须处理嵌套花括号**：第一版正则把 `NtTaskFusionError}`
  与一个孤立的 `}` 当成符号 ⇒ 数出 8 而非 7 ⇒ **恰好等于阈值** ⇒ 会误报越线。
· ⭐ **只数 `pub use` 语句里的标识符**，⛔ 不数注释与文档字符串。
· ⭐ 基线只数**非注释、非空行**。

## 用法
    python3 scripts/ops/nt_crystal_core_judge.py            # 判定
    python3 scripts/ops/nt_crystal_core_judge.py --audit    # 只打印
    python3 scripts/ops/nt_crystal_core_judge.py --self-test # ⭐ 负向测试
"""
from __future__ import annotations

import os
import re
import shutil
import sys
import tempfile

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
FACADE = "neotrix-core/src/l1_action/nt_action_facade.rs"
BASELINE = "scripts/layer-deps-baseline.txt"

# ⭐⭐ 裁决 §3④ 写下的两个阈值（**改这里等于改裁决，须同步改文档**）
JUDGE_MAX_FACADE_SYMBOLS = 8
JUDGE_MIN_BASELINE = 10

_USE = re.compile(
    r"pub\s+use\s+crate::l5_cognition::nt_crystal_core::(\{[^;]*\}|[^;]*);"
)
_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def facade_symbols(text: str) -> list[str]:
    """⭐ 取出 facade 从 `nt_crystal_core` 转出的**符号名**。

    ⭐⭐ 必须**按花括号配平**取 body —— 第一版用 `\\{?([^;]*);` 在
    `pub use …::{A, B};` 这种多行写法下会把 `B}` 与孤立 `}` 当成符号
    （实测数出 8 而非 7，⭐ 恰好等于阈值 ⇒ 会误报越线）。
    """
    out: list[str] = []
    for m in _USE.finditer(text):
        body = m.group(1)
        if body.startswith("{"):
            inner = body[1:-1] if body.rstrip().endswith("}") else body[1:]
        else:
            inner = body
        # ⭐ 只取标识符：`self`/`as` 等在 use 里不是符号
        for tok in _IDENT.findall(inner):
            if tok in ("as", "self", "crate", "super", "pub", "use"):
                continue
            out.append(tok)
    return out


def baseline_count() -> int:
    p = os.path.join(REPO, BASELINE)
    if not os.path.exists(p):
        return -1
    with open(p, encoding="utf-8") as fh:
        return sum(
            1 for ln in fh
            if ln.strip() and not ln.lstrip().startswith("#")
        )


def verdict() -> tuple[list[str], list[str]]:
    fp = os.path.join(REPO, FACADE)
    with open(fp, encoding="utf-8") as fh:
        syms = facade_symbols(fh.read())
    base = baseline_count()
    ok: list[str] = []
    bad: list[str] = []
    if len(syms) <= JUDGE_MAX_FACADE_SYMBOLS:
        ok.append(f"facade 转出符号 {len(syms)} ≤ 阈值 {JUDGE_MAX_FACADE_SYMBOLS}")
    else:
        bad.append(
            f"facade 从 nt_crystal_core 转出 **{len(syms)}** 个符号"
            f"（阈值 {JUDGE_MAX_FACADE_SYMBOLS}）⇒ {syms}\n"
            f"       ⇒ ⭐⭐ 方案乙的前提「L1 只是门面的消费者」**已被推翻**\n"
            f"       ⇒ facade 正在变成「换名字的整包 re-export」\n"
            f"       ⇒ **停止按乙推进**，改走甲（把 shell 移出 L1）。\n"
            f"       ⓘ 裁决原文：docs/architecture/CRYSTAL-CORE-LAYERING-RULING-2026-10-03.md §3④"
        )
    if base >= JUDGE_MIN_BASELINE:
        ok.append(f"layer-deps 基线 {base} ≥ 下限 {JUDGE_MIN_BASELINE}")
    elif base >= 0:
        bad.append(
            f"layer-deps 基线已降到 **{base}**（下限 {JUDGE_MIN_BASELINE}）\n"
            f"       ⇒ ⭐⭐ 16 处违规几乎清完，而唯一途径是 facade 吸收\n"
            f"       ⇒ ⭐ 这正是「facade 变成整包 re-export」的另一端信号\n"
            f"       ⇒ **停止按乙推进**，改走甲。"
        )
    else:
        bad.append(f"基线文件缺失: {BASELINE}")
    return ok, bad


def main() -> int:
    argv = sys.argv[1:]
    ok, bad = verdict()
    if "--audit" in argv:
        for m in ok:
            print("✅", m)
        for m in bad:
            print("⛔", m)
        return 0
    for m in ok:
        print("✅", m)
    for m in bad:
        print("⛔", m)
    print()
    if bad:
        print(f"FAIL: {len(bad)} 项 —— 方案乙的前提已被推翻，须改走甲", file=sys.stderr)
        return 1
    print("PASS: 方案乙的前提仍成立（facade 未膨胀、基线未耗尽）")
    return 0


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        # ⭐⭐ 变异必须**让判据越线**，而不是改阈值。
        # ⭐ 教训（同型三次）：变异方向必须与被测性质相关。
        before_ok, before_bad = verdict()
        target = os.path.join(REPO, FACADE)
        with open(target, encoding="utf-8") as fh:
            original = fh.read()
        tmpdir = tempfile.mkdtemp()
        try:
            # 注入 3 个符号 ⇒ 7 + 3 = 10 > 8 ⇒ 必须翻红
            inject = (
                "\npub use crate::l5_cognition::nt_crystal_core::{\n"
                "    JUDGE_PROBE_A, JUDGE_PROBE_B, JUDGE_PROBE_C,\n};\n"
            )
            with open(target, "w", encoding="utf-8") as fh:
                fh.write(original + inject)
            after_ok, after_bad = verdict()
            if not after_bad:
                print(f"SELFTEST FAIL: 注入后仍全绿（before_bad={before_bad}）")
                sys.exit(1)
            tripped = [b for b in after_bad if "facade 从 nt_crystal_core" in b]
            if not tripped:
                print("SELFTEST FAIL: 未触发 facade 越线判据（变异与被测性质不相关）")
                sys.exit(1)
            print(f"SELFTEST OK: 注入 3 符号后判据翻红 ⇒ {tripped[0].splitlines()[0]}")
        finally:
            with open(target, "w", encoding="utf-8") as fh:
                fh.write(original)
            shutil.rmtree(tmpdir, ignore_errors=True)
        rest_ok, rest_bad = verdict()
        if rest_bad != before_bad:
            print(f"SELFTEST FAIL: 还原后不一致（污染工作树）before={before_bad} after={rest_bad}")
            sys.exit(1)
        print("SELFTEST OK: 还原后与注入前一致")
        sys.exit(0)
    sys.exit(main())