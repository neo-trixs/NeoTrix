#!/usr/bin/env python3
"""orphan-dir gate — 检测「目录里有 .rs 但父 mod.rs 从不引用它」。

# 为什么需要这道门（2026-10-05 立，实测依据）

本仓 `neotrix-core/src/l4_emotion/nt_memory/` 下曾有 **5 个目录、2663 行、
若干测试从未被编译**：`nt_memory/mod.rs` 里零 `mod` 声明指向它们。
挂载后立刻暴露两个真缺陷：

1. `tier_recall.rs` 同秒写入 ⇒ `max_age == 0` ⇒ 相关度全 **NaN**，
   排序经 `partial_cmp`→`None`→`Equal` 静默退化为插入序；
2. `tier_archival.rs` 的 `insert` 对低于阈值者返回 `Ok(())`
   —— **宣称写成功而实际没写**，无任何可观测信号。

而当时 `--lib` 有 **13041 条测试全绿**。

## ⭐ 本门要防的正是这件事

**「测试全绿」不等于「代码被编译过」。**
一个从未被 `mod` 声明引入的目录，可以带着任意多测试安静地躺着。

⛔ 现有门为何都没覆盖：
· `check-layer-deps.sh` —— 管跨层引用，不管模块是否挂载；
· `check-naming.sh` —— 管文件名，不管可达性；
· `check-truth-surface.sh` / `nt_scan_surface.py` —— 管扫描器自身的面，
  **不扫「源码是否在编译树里」**。

## 语义：报告式 + 基线棘轮（默认不拦）

存量 25 个孤儿目录（2026-10-05 实测，合计 >19000 行）。若默认 blocking，
这门会**恒红** —— 而恒红的门等于没有门（它训练人忽略红色）。
故：

* 默认 `exit 0` + 列清单（advisory）；
* `--strict` 才非零；
* 判据是**新增**：与基线 `scripts/orphan-dir-baseline.txt` 求差集，
  只对**新增**报错。既有债记账不阻断 —— 与 `check-layout.sh` 同策略。

## 判据怎么算（避免误报）

对每个含 `mod.rs` 的目录：
1. 收集其**子目录**中至少含一个 `.rs` 的；
2. 若子目录名在父 `mod.rs` 里出现（`mod X;` / `use X::` / 任何 `\bX\b`）
   ⇒ 已挂载，跳过；
3. 否则记为孤儿。

⚠️ 已知的三类**误报来源**，本门按「宁可多列、不可漏列」处理：
· 有意的死代码归档目录；
· 通过 `#[path = "..."]` 挂载的非典型目录（本门不解析 `#[path]`）；
· 未来可能引入的代码生成目录。
⇒ 清单是**待裁决输入**，不是判决。裁决结论写回基线文件。

## 用法

```sh
python3 scripts/ops/nt_orphan_dir.py            # advisory，列清单
python3 scripts/ops/nt_orphan_dir.py --strict   # 有新增孤儿则非零
python3 scripts/ops/nt_orphan_dir.py --write-baseline  # 重建基线
```

## 失败长什么样

新增孤儿目录时 `scripts/check-orphan-dirs.sh --strict` 报红，
提示「该目录从未被编译，其测试也不会跑」——
⇒ 修法二选一：① 挂载它（并修随之暴露的编译错误）
② 明确归档并在基线登记，**不要让它继续静默躺着**。
"""

from __future__ import annotations

import argparse
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC_ROOTS = ("neotrix-core/src", "crates")
BASELINE = os.path.join(ROOT, "scripts", "orphan-dir-baseline.txt")

MOD_DECL = re.compile(r"^\s*(?:pub\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;", re.M)
PATH_ATTR = re.compile(r'#\[\s*path\s*=\s*"([^"]+)"')


def _read(path: str) -> str:
    try:
        with open(path, encoding="utf-8", errors="ignore") as fh:
            return fh.read()
    except OSError:
        return ""


def _declared_names(src: str) -> set[str]:
    return set(MOD_DECL.findall(src))


def find_orphans(root: str) -> list[tuple[str, int]]:
    """返回 [(相对路径, .rs 文件数)]，已按路径排序。"""
    out: list[tuple[str, int]] = []
    for rel_root in SRC_ROOTS:
        abs_root = os.path.join(root, rel_root)
        if not os.path.isdir(abs_root):
            continue
        for dirpath, _dirnames, filenames in os.walk(abs_root):
            if "mod.rs" not in filenames:
                continue
            mod_path = os.path.join(dirpath, "mod.rs")
            src = _read(mod_path)
            declared = _declared_names(src)
            # `#[path="..."]` 挂载：把被引用文件的目录名也算已挂载
            for target in PATH_ATTR.findall(src):
                tgt_abs = os.path.normpath(os.path.join(dirpath, target))
                declared.add(os.path.basename(tgt_abs))
            try:
                children = sorted(os.listdir(dirpath))
            except OSError:
                continue
            for child in children:
                cpath = os.path.join(dirpath, child)
                if not os.path.isdir(cpath) or child in ("tests", "benches", "examples"):
                    continue
                n_rs = sum(1 for f in os.listdir(cpath) if f.endswith(".rs"))
                if n_rs == 0:
                    continue
                if child in declared:
                    continue
                # 父 mod.rs 任意位置提到该名（use/注释引用）也视为有意
                if re.search(r"\b" + re.escape(child) + r"\b", src):
                    continue
                out.append((os.path.relpath(cpath, root).replace(os.sep, "/"), n_rs))
    return sorted(set(out))


def load_baseline() -> set[str]:
    if not os.path.exists(BASELINE):
        return set()
    return {ln.strip() for ln in _read(BASELINE).splitlines() if ln.strip() and not ln.startswith("#")}


def main() -> int:
    ap = argparse.ArgumentParser(description="检测未挂载的源码目录")
    ap.add_argument("--strict", action="store_true", help="有新增孤儿则非零")
    ap.add_argument("--write-baseline", action="store_true", help="重建基线后退出")
    ap.add_argument("--root", default=ROOT)
    args = ap.parse_args()

    orphans = find_orphans(args.root)
    if args.write_baseline:
        lines = [
            "# orphan-dir baseline — 已裁决为「有意不挂载」的目录",
            "# 由 `python3 scripts/ops/nt_orphan_dir.py --write-baseline` 生成。",
            "# 每行一个相对路径（相对仓库根）。改这个文件 = 显式声明「它可以继续不编译」。",
            "",
        ] + [p for p, _ in orphans]
        with open(BASELINE, "w", encoding="utf-8") as fh:
            fh.write("\n".join(lines) + "\n")
        print(f"baseline 写入 {len(orphans)} 条 → {os.path.relpath(BASELINE, args.root)}")
        return 0

    known = load_baseline()
    new = [(p, n) for p, n in orphans if p not in known]
    total_rs = sum(n for _, n in orphans)

    print(f"orphan-dir: 孤儿目录 {len(orphans)} 个 / {total_rs} 个 .rs；"
          f"基线已裁决 {len(known)}；新增 {len(new)}")
    for p, n in new:
        print(f"    ⚠️  NEW  {p}  ({n} 个 .rs)  ← 从未编译，其测试也不会跑")
    if not new:
        for p, n in orphans[:3]:
            print(f"    ·  已知 {p}  ({n} 个 .rs)")
        if len(orphans) > 3:
            print(f"    ·  … 另有 {len(orphans) - 3} 个已在基线")

    if new and args.strict:
        print("\n修法二选一：")
        print("  ① 挂载它（在父 mod.rs 加 `pub mod X;`），并修随之暴露的编译错误")
        print("  ② 明确归档并写入 scripts/orphan-dir-baseline.txt")
        print("  ⛔ 不要让它继续静默躺着 —— 绿测试给未编译的代码背书，比红测试更危险。")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())