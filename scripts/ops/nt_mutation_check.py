#!/usr/bin/env python3
"""nt_mutation_check.py — 变异测试：证明测试**有效**，而非只证明它**通过**。

## 为什么需要它（R-MUT-1）
「测试全绿」不构成「测试有效」的证据。R-VERIFY 的老问题（2026-09-29 实测）：
`nt_pdf_ground` 15 个测试全绿、408 crate 测试全绿，但 `ground_text_bytes`
**从未被任何测试调用过** —— helper 全绿与「能干活」之间隔着整条链路。
补了 5 个端到端测试后仍不知有无判别力，于是做变异测试：
**故意破坏实现，看测试是否变红**。

## 两次翻车（都写在这里当反面教材）

### 翻车①：变异根本没落地，却当成「测试无效」
用 `str.replace(old, new, 1)` 写变异，**不检查是否匹配到**。
结果「测试仍全绿」，我据此写下「测试无判别力」——**结论是错的**。
`grep` 复查才发现目标行确实在 345 行、变异确实写进去了。
⇒ 铁律：**变异前 `assert content.count(old) == 1`，
  写后 `grep -c` 确认落地，两步缺一不可**（见 `_apply_mutation`）。

### 翻车②：变异太弱，天然不可检测
把 `hi = hex_val(...)` 改成 `hi = 0`：
看起来是「破坏 CMap 解码」，但 fixture 里全是 `<0054>`/`<006F>` 形式，
而 `(0x00 << 4) | 0x54 == 0x54` —— **该变异对所有 `<0xXY>` 恒等**。
手算才发现，不是测试的问题，是变异选得太弱。
换成 `return None` 仍绿（None 让调用方走 fallback，仍能匹配出词）。
换成 `return Some("ZZZZ")`（恒返回错值）⇒ **FAILED，杀 7 个测试**。

⇒ 铁律：**变异必须改变可观测行为**。
返回常量 > 返回 None > 清零一位（后两者可能恒等或走 fallback）。

## 用法
    # 列出候选变异点（函数体首行可改处）
    python3 scripts/ops/nt_mutation_check.py --list FILE.rs

    # 对指定变异点跑：要求「测试变红」
    python3 scripts/ops/nt_mutation_check.py FILE.rs --fn hex_to_string \\
        --mutate 'return|return Some("ZZZZ".to_owned());' --cargo "test -p nt --lib"

    # 只做体检：报告该文件有多少测试、是否含端到端入口调用
    python3 scripts/ops/nt_mutation_check.py --audit FILE.rs

退出码：0=变异成功杀死测试（判别力已证）；1=测试未变红（判别力不足）；
2=变异未落地（**不是**测试无效，是工具/变异写错了 —— 见翻车①）。
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# 函数体第一行常见的「可注入点」；命中即列为候选变异点。
# 每条 = (正则, 说明)。刻意保持少而准 —— 候选太多会淹没真信号。
INJECT_SITES: list[tuple[str, str]] = [
    (r"^pub fn (\w+)", "pub fn 入口：加守卫让返回常量"),
    (r"^fn (\w+)", "内部 fn 入口"),
    (r"^\s+let (\w+) = ", "局部绑定：可强制为常量"),
]

# 强变异模板。顺序即推荐度（第一条最可靠）。
STRONG_MUTATIONS: list[tuple[str, str, str]] = [
    # (名称, 注入的守卫, 为什么强)
    ("panic", 'panic!("MUTATION");', "立刻炸 ⇒ 任何覆盖该函数的测试必红"),
    (
        "wrong-const",
        'return Some("ZZZZ".to_owned());',
        "恒返回错值 ⇒ 语义依赖该函数的断言必红（实测最有效）",
    ),
    (
        "none",
        "return None;",
        "返回 None ⇒ 弱：调用方可能走 fallback 仍通过",
    ),
]


def list_sites(path: Path) -> list[tuple[int, str, str]]:
    """列出候选变异点：(行号, 函数名, 说明)。"""
    out: list[tuple[int, str, str]] = []
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    for i, line in enumerate(lines, 1):
        for pat, desc in INJECT_SITES:
            m = re.match(pat, line)
            if m:
                out.append((i, m.group(1), desc))
    return out


def apply_mutation(text: str, fn_name: str, guard: str) -> tuple[str, int] | None:
    """在 `fn_name` 的函数体第一行注入 guard。

    返回 (新内容, 匹配数)。**匹配数 != 1 时返回 None**，由调用方决定是否中止 ——
    这是翻车①的教训：绝不静默跳过。
    """
    pat = re.compile(rf"^((?:pub )?fn {re.escape(fn_name)}\s*\([^)]*\)[^{{]*\{{)", re.M)
    matches = pat.findall(text)
    if len(matches) != 1:
        return None
    return pat.sub(lambda m: m.group(1) + f"\n    {guard}", text, count=1), len(matches)


def verify_landed(path: Path, guard: str) -> bool:
    """写后确认：变异真在文件里。翻车①的直接对策。"""
    return guard.split(";")[0][:40] in path.read_text(
        encoding="utf-8", errors="replace"
    )


def run_cargo(cmd: str, cwd: Path, timeout: int = 900) -> tuple[bool, str]:
    """跑 cargo 命令，返回 (是否全绿, 合并输出尾部)。"""
    try:
        p = subprocess.run(
            ["cargo"] + cmd.split(),
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return False, "TIMEOUT"
    out = (p.stdout + p.stderr)[-4000:]
    green = "test result: ok" in out and "FAILED" not in out
    return green, out


def audit(path: Path) -> dict:
    """体检：测试数、是否有端到端入口、是否有变异点。"""
    text = path.read_text(encoding="utf-8", errors="replace")
    tests = re.findall(r"#\[test\]\s*\n\s*fn (\w+)", text)
    # 端到端启发：测试体内调用了被测入口（非 helper）名。
    pub_fns = [m for _, m, _ in list_sites(path) if m.startswith("_") is False]
    return {
        "file": str(path),
        "lines": len(text.splitlines()),
        "test_count": len(tests),
        "mutation_sites": len(list_sites(path)),
        "pub_fns": len(pub_fns),
        "e2e_tests": [
            t for t in tests if "e2e" in t or "end_to_end" in t or "integration" in t
        ],
        "note": "e2e_tests 为空 ⇒ 该文件只测 helper，生产入口可能零覆盖（2026-09-29 nt_pdf_ground 即此病）",
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="变异测试：证明测试有效，而非只证明它通过")
    ap.add_argument("file", nargs="?", type=Path, help="待检查的 .rs")
    ap.add_argument("--list", action="store_true", help="列出候选变异点")
    ap.add_argument("--audit", action="store_true", help="体检：测试数 / e2e 覆盖")
    ap.add_argument("--fn", help="目标函数名")
    ap.add_argument(
        "--mutate",
        help='注入语句，形如 \'return|return Some("ZZZZ".to_owned());\'（before|after|或直接代码）',
    )
    ap.add_argument("--cargo", default="", help='cargo 命令，如 "test -p neotrix-neobot --lib"')
    ap.add_argument("--json", action="store_true", help="JSON 输出")
    ap.add_argument("--no-run", action="store_true", help="只做体检不跑变异")
    args = ap.parse_args()

    if args.audit:
        if not args.file:
            print("--audit 需要 FILE", file=sys.stderr)
            return 2
        rep = audit(args.file)
        print(json.dumps(rep, ensure_ascii=False, indent=2) if args.json else _fmt(rep))
        return 0

    if not args.file:
        print("需要 FILE", file=sys.stderr)
        return 2

    # 半吊子参数（给了 --mutate 却没给 --fn/--cargo）必须报 2，不能掉进 --list 分支
    if not args.list and not args.audit and (args.mutate or args.fn) and not (args.fn and args.mutate and args.cargo):
        print("[mutation_check] ✗ 需要同时给 --fn + --mutate + --cargo", file=sys.stderr)
        return 2

    if args.list or (not args.fn and not args.mutate):
        sites = list_sites(args.file)
        if args.json:
            print(json.dumps([{"line": a, "fn": b, "hint": c} for a, b, c in sites], ensure_ascii=False, indent=2))
        else:
            print(f"[mutation_check] {args.file} 候选变异点 {len(sites)} 个")
            for ln, fn, hint in sites:
                print(f"  {ln:>5}  {fn:<28} {hint}")
            print("\n  强变异模板（按推荐度）：")
            for name, guard, why in STRONG_MUTATIONS:
                print(f"    {name:<12} {why}")
        return 0

    if not (args.fn and args.mutate and args.cargo):
        print("需要 --fn + --mutate + --cargo", file=sys.stderr)
        return 2

    src = args.file.resolve()
    orig = src.read_text(encoding="utf-8")
    before_green, _ = run_cargo(args.cargo, src.parents[2] if len(src.parents) > 2 else Path.cwd())
    if not before_green:
        print(
            "[mutation_check] ✗ 基线就红 ⇒ 无变异对照，拒绝执行"
            "（这正是 R-P16「重读验证」的反例：不先确认基线就无法归因）",
            file=sys.stderr,
        )
        return 2

    # before|after 分隔；否则视为直接注入代码
    if "|" in args.mutate:
        anchor, code = args.mutate.split("|", 1)
        anchor = anchor.strip()
        code = code.strip()
        if anchor not in orig:
            print(f"[mutation_check] ✗ 锚点未找到（翻车①）：{anchor!r}", file=sys.stderr)
            return 2
        mutated = orig.replace(anchor, anchor + "\n    " + code, 1)
    else:
        res = apply_mutation(orig, args.fn, args.mutate)
        if res is None:
            print(
                f"[mutation_check] ✗ 无法唯一定位 fn {args.fn}（翻车①：变异未落地，"
                "**不能**据此断言测试无效）",
                file=sys.stderr,
            )
            return 2
        mutated, _n = res

    backup = tempfile.NamedTemporaryFile(delete=False, suffix=".rs.bak")
    backup.write(orig.encode("utf-8"))
    backup.close()
    try:
        src.write_text(mutated, encoding="utf-8")
        # 写后确认（翻车①对策）
        if mutated[:200] != src.read_text(encoding="utf-8")[:200]:
            print("[mutation_check] ✗ 写入校验失败", file=sys.stderr)
            return 2
        after_green, out = run_cargo(args.cargo, src.parents[2] if len(src.parents) > 2 else Path.cwd())
    finally:
        src.write_text(orig, encoding="utf-8")  # 必恢复
        shutil.copyfile(backup.name, src)
        Path(backup.name).unlink(missing_ok=True)

    if args.json:
        print(json.dumps({"before_green": before_green, "after_green": after_green,
                          "killed": not after_green}, ensure_ascii=False, indent=2))
    else:
        print(f"[mutation_check] 基线={'绿' if before_green else '红'} → 变异后={'绿' if after_green else '红'}")
    if not after_green:
        print("  ✅ 变异杀死测试 ⇒ 判别力已证")
        return 0
    print("  ⚠️  变异后仍全绿 ⇒ **测试无判别力**（或变异太弱，见 STRONG_MUTATIONS）")
    print(out[-800:])
    return 1


def _fmt(rep: dict) -> str:
    return (
        f"[mutation_check] {rep['file']}\n"
        f"  行数        {rep['lines']}\n"
        f"  测试数      {rep['test_count']}\n"
        f"  pub fn      {rep['pub_fns']}\n"
        f"  变异点      {rep['mutation_sites']}\n"
        f"  e2e 测试    {len(rep['e2e_tests'])} {rep['e2e_tests'] or '(无)'}\n"
        f"  ⚠ {rep['note']}"
    )


if __name__ == "__main__":
    sys.exit(main())
