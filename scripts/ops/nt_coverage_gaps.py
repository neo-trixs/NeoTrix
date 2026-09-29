#!/usr/bin/env python3
"""nt_coverage_gaps.py — 找出「测试从未触碰」的生产入口。

## 为什么需要它（R-COV-1）
2026-09-29 实测事故：`nt_pdf_ground.rs` 有 15 个测试、crate 408 测试全绿，
但真正干活的 `ground_text_bytes` / `ground_text` **从未被任何测试调用过**。
「测试全绿」与「入口被验证」之间没有必然联系 ——
helper 测得再细，入口崩了也不会红。

这不是孤例：同轮测量 neobot 有 **53 个 pub fn 零测试覆盖**（上界，见下）。

## 关键设计：只查 `pub fn`，且必须跨文件
- **只查 pub fn**：私有 helper 零覆盖是正常的（间接被覆盖），
  只有 `pub fn` 才是「生产入口」——外部/跨模块能调到的那个。
- **必须跨文件统计**：入口常在 A 文件、被 B 文件的测试调用。
  只看同文件会误报成百上千条。工具因此扫全 crate 的**全部 `#[cfg(test)]` 块**。

## 误报（诚实声明）
- 通过 trait / `Dyn*` / 宏 / 重新导出调用的入口算不到 ⇒ 假阴性。
- 在别的 crate 的测试里调用（集成测试）算不到 ⇒ 假阴性。
- 测试只调用了名字但没断言行为 ⇒ 算「覆盖」但其实无效
  —— **这一类要靠 `nt_mutation_check.py` 才能识别**。
⇒ 本工具是**筛选器**不是**证明**：它告诉你「哪里可能有洞」，
  不告诉你「哪里没问题」。

## 用法
    python3 scripts/ops/nt_coverage_gaps.py crates/neotrix-neobot/src
    python3 scripts/ops/nt_coverage_gaps.py <dir> --min-pub 2
    python3 scripts/ops/nt_coverage_gaps.py <dir> --json
退出码：0=无零覆盖入口；1=有（供 CI 提示，非硬门）。
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

FUNC_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)\s*[\(<]", re.M)
PUB_RE = re.compile(r"^\s*pub(?:\([^)]*\))?\s+(?:async\s+)?fn\s+(\w+)\s*[\(<]", re.M)
TEST_BLOCK_RE = re.compile(r"#\[cfg\(test\)\]")


def split_prod_test(text: str) -> tuple[str, str]:
    """切出生产段与测试段。`#[cfg(test)] mod tests` 之后全算测试。"""
    m = TEST_BLOCK_RE.search(text)
    if not m:
        return text, ""
    return text[: m.start()], text[m.start() :]


def collect(crate_root: Path) -> dict:
    """返回 {file: {"pub_fns": [...], "uncalled": [...]}}。

    测试调用集是**全 crate 汇总**（含跨文件），避免同文件视角的误报。
    """
    files = sorted(p for p in crate_root.rglob("*.rs") if "tests/" not in str(p))
    texts = {p: p.read_text(encoding="utf-8", errors="replace") for p in files}

    # 全 crate 的测试段文本拼接 ⇒ 跨文件调用也能命中
    test_blob = "\n".join(split_prod_test(t)[1] for t in texts.values())
    # 调用点判定：名字后紧跟 (  （排除定义行本身：定义在 prod 段，天然不在 blob）
    called = {m.group(1) for m in re.finditer(r"\b(\w+)\s*\(", test_blob)}

    # ── 死代码判定：一次性建索引，O(库大小)，不做「每名重扫全库」──
    # 2026-09-29 性能实测：朴素写法（对每个零覆盖名遍历全部文件）
    # 在 neotrix-core/src（798K 行）上 >400s 未完成 —— 那等于不可用。
    # 改为：先把候选名编成一个 alternation 正则，**一次**扫全库建表。
    all_pub: set[str] = set()
    for text in texts.values():
        all_pub.update(PUB_RE.findall(split_prod_test(text)[0]))
    candidates = all_pub - called

    # name -> [该名字所有出现处的行文本]（定义行/re-export 行除外）
    call_sites: dict[str, list[str]] = {n: [] for n in candidates}
    if candidates:
        big = re.compile(
            r"\b(" + "|".join(sorted(map(re.escape, candidates))) + r")\s*\("
        )
        for qt in texts.values():
            for m in big.finditer(qt):
                ls = qt.rfind("\n", 0, m.start()) + 1
                le = qt.find("\n", m.start())
                call_sites[m.group(1)].append(qt[ls : le if le != -1 else len(qt)])

    def is_dead(name: str) -> bool:
        """零**真实**调用者：排除定义行与 `pub use` 再导出行。"""
        for line in call_sites.get(name, ()):
            if re.match(
                rf"\s*pub(?:\([^)]*\))?\s+(?:async\s+)?fn\s+{re.escape(name)}\b", line
            ):
                continue  # 定义行
            if re.search(r"\bpub\s+use\b", line):
                continue  # re-export 行
            return False
        return True

    out: dict[str, dict] = {}
    for p, text in texts.items():
        prod, _ = split_prod_test(text)
        pubs = PUB_RE.findall(prod)
        if not pubs:
            continue
        uncalled = sorted({n for n in pubs if n not in called})
        dead = sorted(n for n in uncalled if is_dead(n))
        out[str(p.relative_to(crate_root))] = {
            "pub_count": len(pubs),
            "uncalled": uncalled,
            "dead": dead,
        }
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="找出测试从未触碰的生产入口（筛选器，非证明）")
    ap.add_argument("root", type=Path, help="crate 的 src 目录")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--min-pub", type=int, default=1, help="只报 pub fn 数 >= N 的文件")
    args = ap.parse_args()

    if not args.root.is_dir():
        print(f"[coverage-gaps] 目录不存在: {args.root}", file=sys.stderr)
        return 2

    data = collect(args.root)
    rows = []
    total_pub = total_gap = 0
    for fname, info in data.items():
        total_pub += info["pub_count"]
        total_gap += len(info["uncalled"])
        if info["uncalled"] and info["pub_count"] >= args.min_pub:
            rows.append((len(info["uncalled"]), fname, info))

    if args.json:
        print(json.dumps(
            {"root": str(args.root), "total_pub": total_pub, "total_uncalled": total_gap,
             "files": [{"file": f, **i} for _, f, i in sorted(rows, reverse=True)]},
            ensure_ascii=False, indent=2))
        return 1 if total_gap else 0

    rows.sort(reverse=True)
    print(f"[coverage-gaps] {args.root}")
    print(f"  生产 pub fn 合计 {total_pub}，测试从未调用 {total_gap}（占 {total_gap*100//max(total_pub,1)}%）")
    dead_all = [(f, n) for f, i in data.items() for n in i.get("dead", [])]
    live = total_gap - len(dead_all)
    if rows:
        print(f"\n  {'零覆盖':>6}  {'pub':>4}  file")
        for gap, fname, info in rows:
            print(f"  {gap:>6}  {info['pub_count']:>4}  {fname}")
            print(f"         └─ {', '.join(info['uncalled'][:6])}"
                  + (" …" if len(info["uncalled"]) > 6 else ""))
    print(f"\n  其中**零真实调用者**（死代码候选，删比补测试划算）：{len(dead_all)}")
    print(f"  **生产在用但测试没碰**（真洞，应补测试）：{live}")
    for f, n in dead_all[:10]:
        print(f"    ☠ {f}::{n}")
    if len(dead_all) > 10:
        print(f"    …共 {len(dead_all)} 个")
    print("\n  ⚠ 本工具是**筛选器**：能指出「哪里可能有洞」，不能证明「哪里没问题」。")
    print("    假阴性来源：trait/Dyn/宏/重导出调用、跨 crate 集成测试、")
    print("    「调用了但没断言」（后者需 nt_mutation_check.py 识别）。")
    return 1 if total_gap else 0


if __name__ == "__main__":
    sys.exit(main())
