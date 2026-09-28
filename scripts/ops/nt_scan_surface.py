#!/usr/bin/env python3
"""nt_scan_surface.py — 扫描面守门：让「我少扫了一个面」变成 loud 失败。

## 问题（2026-09-28 实测，四处同型）
`check-api-surface.sh:12` 硬编码 `src-tauri/src`（已随 5c02e738 删除）：

    TAURI=$(rg -c "#tauri-command" ... src-tauri/src neotrix-core/src 2>/dev/null | awk ...)

`rg` 对不存在路径 **exit 2** 并把错误写 stderr，但 `2>/dev/null` 吞掉它，
`awk` 仍照常输出 `0` ⇒ 脚本打印 **`Tauri commands: 0`** ——
一个**看起来正常的假数字**。用它当证据会得出「本仓没有 Tauri 命令面」
的错误结论，而真实原因是「我压根没扫那个面」。

同型还有 3 处：
  - `check-truth-surface.sh:43` `SCAN_ROOTS` 含 `src-tauri/src` 与已删的 `apps/`
    （靠 `[ -d "$root" ] || continue` 兜住 ⇒ 静默少扫，不报）
  - `nt_mapgen.py:48-50` 三条 `src-tauri/*` 根
  - `check-api-surface.sh:15` 同样的 `.route(` 双路径

前两处已被前人用不同方式修过（删引用 + 注释 / `[ -d ] || continue` 兜底），
但**都不报「少扫了」**。本脚本给出统一解法。

## 关键区分
    路径不存在  ≠  该面为空
前者是**本脚本的缺陷**，后者才是**被扫描对象的事实**。
把两者混为一谈，就会产出一个「0 命中」的数字，让读者以为「扫过了，确实没有」。

## 用法
    python3 scripts/ops/nt_scan_surface.py --decl scripts/check-api-surface.sh
    python3 scripts/ops/nt_scan_surface.py --decl scripts/check-truth-surface.sh
    python3 scripts/ops/nt_scan_surface.py --decl scripts/ops/nt_mapgen.py
    python3 scripts/ops/nt_scan_surface.py --self     # 扫自己

退出码：0 = 全部声明面都存在；1 = 有面已消失（须修脚本或明确记为已知豁免）。
"""
import argparse
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# 已知的"面已消失"豁免。键是声明文件，值是 (路径, 该面为何消失, 该面现在归谁)
KNOWN_GONE = {
    ("scripts/check-api-surface.sh", "src-tauri/src"): (
        "桌面端已随 5c02e738 归档（599 files），现由 crates/neotrix-neobot 承担",
    ),
    ("scripts/check-api-surface.sh", "src-tauri"): (
        "同上",
    ),
    ("scripts/check-truth-surface.sh", "src-tauri/src"): (
        "桌面端已随 5c02e738 归档",
    ),
    ("scripts/check-truth-surface.sh", "src-tauri"): (
        "桌面端已随 5c02e738 归档",
    ),
    ("scripts/check-truth-surface.sh", "apps"): (
        "apps/ 已于 2026-09-28 彻底移除（apps/neobot-desktop 归档后仅剩空壳）",
    ),
    ("scripts/ops/nt_mapgen.py", "src-tauri/src"): (
        "桌面端已随 5c02e738 归档",
    ),
    ("scripts/ops/nt_mapgen.py", "src-tauri/frontend"): (
        "桌面端已随 5c02e738 归档",
    ),
    ("scripts/ops/nt_mapgen.py", "src-tauri/frontend/src"): (
        "桌面端已随 5c02e738 归档",
    ),
}

# 「长得像路径但是给人看的说明文字」，逐条给出理由。
# 判据标准：**它在输出/报错里出现，不参与实际遍历**。
PROSE_PATHS = {
    "tests/mod.rs": "check-truth-surface.sh:198 的提示文字（告诉人该加 mod 声明），非扫描目标",
    "foo/bar.rs": "文档示例",
    "foo/bar/mod.rs": "文档示例",
    "bar/mod.rs": "文档示例",
    "thirdparty/.": "注释里提到已排除该目录",
}

# 路径字面量：允许 . / - / 字母数字 / 斜杠；排除明显的非路径串
PATH_RE = re.compile(r"""(?<![\w/])([A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)+)""")

# 非扫描面的判据（这些字符串**长得像路径但不是扫描目标**）。
# 2026-09-28 首版把注释里的 `foo/bar.rs`、`nested/test`、
# `function/class/const` 全当成扫描面，20 个 MISSING 里大半是假警报 ——
# **守门工具自己噪声过大时，真实信号会被淹没**，那等于没守。
# 故按「是否有已知顶层目录」过滤：真实扫描面必落在仓库已知根之下。
KNOWN_ROOTS = (
    "neotrix-core", "crates", "apps", "scripts", "skills", "docs", "config",
    "sessions", "models", "src-tauri", "thirdparty", "assets", "target",
    ".neotrix", ".github", ".githooks", "os/notes", "pkg", "os/skills",
    "neotrix", ".project-map", "tests", "benches", "examples",
)


def _strip_comments_and_docs(text, is_sh):
    """去掉注释行与文档字符串，让候选只来自**可执行代码**。

    这是本工具能否可用的关键：门若连自己都在报噪声，人就会开始忽略它。
    """
    lines = []
    in_heredoc = None
    for ln in text.split("\n"):
        st = ln.strip()
        if in_heredoc is not None:
            if in_heredoc in st:
                in_heredoc = None
            continue
        if is_sh:
            if st.startswith("<<"):
                in_heredoc = st[2:].strip().strip("'\"") or "EOF"
                continue
            if st.startswith("#"):
                continue
            # 行尾注释（本仓脚本无字符串内 # 场景，够用）
            ln = re.sub(r"(?<![\"'])#(?![!]).*$", "", ln)
        lines.append(ln)
    return "\n".join(lines)


def candidate_paths(text, is_sh):
    """抽出**代码里**的路径字面量（注释已剥离）。"""
    out = set()
    for m in PATH_RE.finditer(_strip_comments_and_docs(text, is_sh)):
        p = m.group(1)
        if p.startswith(("http", "~")) or "://" in p:
            continue
        if p.split("/")[0] not in KNOWN_ROOTS:
            continue  # 长得像路径但不在已知根下 ⇒ 文档文字，不是扫描面
        if p in PROSE_PATHS:
            continue  # 输出文字里的路径，不是扫描目标
        out.add(p)
    return out


def check(decl_path, text, is_sh):
    missing, ok = [], []
    for p in sorted(candidate_paths(text, is_sh)):
        full = os.path.join(REPO, p)
        if os.path.exists(full):
            ok.append(p)
        else:
            missing.append(p)
    return ok, missing


def main():
    ap = argparse.ArgumentParser(
        description="扫描面守门：路径不存在 ≠ 该面为空（loud 失败而非静默 0）"
    )
    ap.add_argument("--decl", action="append", help="声明扫描面的文件（可多次）")
    ap.add_argument("--self", action="store_true", help="只查本脚本")
    args = ap.parse_args()

    if args.self:
        targets = [os.path.relpath(os.path.abspath(__file__), REPO)]
    elif args.decl:
        targets = args.decl
    else:
        targets = [
            "scripts/check-api-surface.sh",
            "scripts/check-truth-surface.sh",
            "scripts/check-forbid-coverage.sh",
            "scripts/ops/nt_mapgen.py",
        ]

    total_missing = 0
    total_ok = 0
    for rel in targets:
        full = os.path.join(REPO, rel)
        if not os.path.isfile(full):
            print(f"scan-surface: 声明文件不存在 {rel}")
            total_missing += 1
            continue
        with open(full, encoding="utf-8", errors="replace") as f:
            text = f.read()
        ok, missing = check(rel, text, rel.endswith((".sh", ".bash")))
        total_ok += len(ok)
        status = "OK" if not missing else f"{len(missing)} MISSING"
        print(f"scan-surface: {rel:42} 现存 {len(ok):>2} / 缺失 {len(missing):>2}  [{status}]")
        for p in ok:
            print(f"    ✅ {p}")
        for p in missing:
            note = KNOWN_GONE.get((rel, p))
            if note:
                print(f"    ⚠️  KNOWN-GONE  {p}")
                print(f"        原因: {note[0]}")
                print(f"        ⇒ 脚本里这行已是死引用，应删除或改指新位置（静默跳过 ≠ 正确）")
            else:
                print(f"    ❌ MISSING    {p}  （未知消失原因 —— 须人工确认）")
        total_missing += len(missing)

    hr = "-" * 66
    print(hr)
    if total_missing:
        print(f"scan-surface: 现存 {total_ok} 面，缺失 {total_missing} 面")
        print("  ⇒ 缺失面会让扫描器输出**看似正常的 0**（路径不存在 ≠ 该面为空）。")
        print("    修法二选一：① 删掉死引用并在注释里记因由（仿 check-forbid-coverage.sh:28）")
        print("             ② 改指新位置；若确实还要扫，脚本须在面缺失时**显式报错**。")
        return 1
    print(f"scan-surface: 全部 {total_ok} 个扫描面存在 ✅")
    return 0


if __name__ == "__main__":
    sys.exit(main())
