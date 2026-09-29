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
DETECTION_TARGETS = {
    ("scripts/check-api-surface.sh", "apps/neobot-desktop"):
        "存在性检测的目标 —— 它本该不存在，出现才说明桌面端回到了本仓",
    ("scripts/check-api-surface.sh", "src-tauri"):
        "同上（存在性检测目标）",
}

PROSE_PATHS = {
    "tests/mod.rs": "check-truth-surface.sh:198 的提示文字（告诉人该加 mod 声明），非扫描目标",
    "foo/bar.rs": "文档示例",
    "foo/bar/mod.rs": "文档示例",
    "bar/mod.rs": "文档示例",
    "thirdparty/.": "注释里提到已排除该目录",
    # 2026-09-28 补：sh 的 **echo 字符串**里也有像路径的散文
    # （首版只剥注释不剥字符串，`Axum/Tauri` / `nested/test` 因此漏网）。
    "Axum/Tauri": "check-api-surface.sh:26 的输出文案「export spec from Axum/Tauri surface」",
    "nested/test": "check-api-surface.sh:16 的输出文案「overcounts nested/test code」",
    "declares/uses": "check-truth-surface.sh 的输出文案「committed code declares/uses this file」",
    "crate/src/lib.rs": "check-forbid-coverage.sh 的 for 循环骨架 `for crate in ...; do check_root \"crates/$crate/src/lib.rs\"`",
    "function/class/const": "nt_mapgen.py 的输出文案",
    "interface/type/enum": "nt_mapgen.py 的输出文案",
    "struct/enum/trait/fn/mod/type": "nt_mapgen.py 的输出文案",
    # 2026-09-28 补：单段候选（≥4 字符无扩展名）带来的 9 条固定噪声 ——
    # 全是 lang/ext 映射的值与已知跳过目录，逐条登记而非放宽判据。
    ".next": "构建产物目录（next.js）",
    "__pycache__": "Python 构建产物，SKIP_DIRS 成员",
    "dist": "构建产物目录",
    "target": "cargo 构建产物，nt_mapgen.py SKIP_DIRS 成员（跳过用，非扫描根）",
    "node_modules": "依赖目录，SKIP_DIRS 成员",
    "html": "lang/ext 映射值（nt_mapgen.py LANGS）",
    "json": "lang/ext 映射值",
    "scss": "lang/ext 映射值",
    "toml": "lang/ext 映射值",
    "yaml": "lang/ext 映射值",
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
        else:
            # Python 侧：# 注释 + 三引号 docstring
            if st.startswith("#"):
                continue
            q = None
            if st[:3] in ('"""', "'''"):
                q = st[:3]
                # 单行 docstring：起止在同一行
                if len(st) <= 6 or not st.endswith(q):
                    in_heredoc = q
                continue
            if in_heredoc is not None:
                if in_heredoc in st:
                    in_heredoc = None
                continue
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
        if p in PROSE_PATHS:
            continue  # 输出文字里的路径，不是扫描目标
        # 只对**顶层是已知根**的路径做严格存在性检查。
        #
        # ⚠️ 2026-09-28 修正：首版在此处对不在 KNOWN_ROOTS 里的路径一律
        # `continue`，**结果连新增的死引用也一并滤掉** —— 注入
        # `("definitely-gone-dir", "x")` 到 nt_mapgen.py 的 ROOT_AREAS
        # 不会被报出，即「用它防死引用」这件事本身有盲区。
        out.add(p)
    # 2026-09-28 补第二处盲区：首版 PATH_RE **要求路径含 `/`**，于是
    # `("definitely-gone-dir", "x")` 这种**单段**扫描根连候选都进不来。
    # 扫描根完全可能是单段目录名（games / fuzz / ntos 就是）⇒ 必须覆盖。
    #
    # 但「所有二元组的首段」噪声极大（实测 24 条：lang/ext 映射、skip 名单
    # 里的 `yaml`/`yml` 等全是二元组）。故只取**长度≥4 且不含扩展名**
    # 的首段 —— 目录名通常长这样，而 `yaml`/`html`/`rs` 这类 2-4 字符的
    # 扩展名被排除。实测噪声 24 → 0，仍能抓到 `definitely-gone-dir`。
    for m in re.finditer(r'["\']([A-Za-z0-9_.-]{4,})["\']\s*,\s*["\'][^"\']*["\']', text):
        cand = m.group(1)
        if "." in cand and not cand.startswith("."):
            continue  # 带扩展名 ⇒ 文件名/lang key，不是目录根
        if cand in PROSE_PATHS:
            continue  # 已登记的散文/扩展名/构建目录
        out.add(cand)
    return out


def check(decl_path, text, is_sh):
    """返回 (现存, 硬缺失, 待人工确认)。

    **分级**是本工具能同时做到「不漏」与「不吵」的关键：
    - 顶层在 KNOWN_ROOTS 下 ⇒ 它**确实**是扫描面，不存在就是硬缺失
    - 不在已知根下但含分隔符（如 `foo/bar`）⇒ 疑似扫描面，报**待确认**
    - 不在已知根下且无分隔符（如 `definitely-gone-dir`）⇒ 仍可能是新增扫描
      根，同样报待确认。**绝不能静默丢弃** —— 2026-09-28 首版就是在这里
      静默 `continue`，导致注入的死引用完全不被报出（盲区）。

    代价：首版 20 个假警报的教训在此重演，故把「待确认」单列一档且不进
    退出码，避免靠调低灵敏度来换「看起来干净」。
    """
    ok, hard_missing, suspect = [], [], []
    for p in sorted(candidate_paths(text, is_sh)):
        full = os.path.join(REPO, p)
        if os.path.exists(full):
            ok.append(p)
            continue
        if (decl_path, p) in DETECTION_TARGETS:
            continue  # 存在性检测的目标，本就该不存在
        if p.split("/")[0] in KNOWN_ROOTS:
            hard_missing.append(p)
        else:
            suspect.append(p)
    return ok, hard_missing, suspect


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
        ok, missing, suspect = check(rel, text, rel.endswith((".sh", ".bash")))
        total_ok += len(ok)
        status = "OK" if not missing else f"{len(missing)} MISSING"
        print(f"scan-surface: {rel:42} 现存 {len(ok):>2} / 缺失 {len(missing):>2}"
              f" / 待确认 {len(suspect):>2}  [{status}]")
        for p in ok:
            print(f"    ✅ {p}")
        for p in suspect:
            print(f"    ? SUSPECT     {p}  （不在已知根下但缺失 —— 若是新增扫描根则须修）")
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
