#!/usr/bin/env python3
"""nt_dead_flag.py — 「声明了默认值，却从未被读取」的配置开关门。

## 要防的缺陷（A55 实测实例）

`EntityMappingConfig::enable_semantic_mapping`：
- 声明为 `pub enable_semantic_mapping: bool`（有默认值 `true`）
- 模块文档明写该能力是本模块核心
- **`cargo check` 通过、现有测试全绿、`deny(warnings)` 也不报**
  （因为字段出现在 `Default` 初始化里，被视为「已使用」）
- 但它在 `calculate_name_similarity` 中**从未被读取**
  ⇒ 该能力实际不存在，而 `test_name_similarity` 里那条
  `assert!(calculate_name_similarity("HashMap", "Dict") > 0.0)`
  **从写下来就不可能通过**

这是比编译错误更隐蔽的一类：**开关看起来是活的（默认值存在、warning 干净），
实际是死的。** 故需要独立门。

## 判据

1. 收集全部 `pub <name>: bool` 字段（配置开关候选）。
2. 剥离注释与字符串字面量（落实 R-SCAN-1b：裸 grep 命中不构成证据）。
3. 读点 = `<任意标识符>.<name>` 的出现。
   字段声明（`pub x: bool`）与字段初始化（`x: true`）**不带前导点**，
   故天然不会被计为读点 —— 这是本判据能成立的关键。
4. 零读点 ⇒ 上报为「疑似死开关」。

## 已知误报来源（进基线，不当缺陷）

- 只为序列化落盘、由外部工具读取的字段
- 被测试专用代码读取但生产路径未用的字段
- trait 关联常量（不在本判据范围）

## 用法

    python3 scripts/ops/nt_dead_flag.py            # advisory，列清单，exit 0
    python3 scripts/ops/nt_dead_flag.py --strict   # 对基线外的新增非零
    python3 scripts/ops/nt_dead_flag.py --audit    # 同时打印读点计数

退出码遵循门族约定：0 通过 / 1 有新增（--strict）/ 2 门自身无法判定。
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

SKIP_DIRS = {"target", ".git", "node_modules", ".worktrees", "models", "vendor"}
FIELD_RE = re.compile(r"^\s*pub\s+([a-z_][a-z0-9_]*)\s*:\s*bool\b")
CONFIG_HINT_RE = re.compile(r"\b(Config|Settings|Options|Params|Policy|Params)\b")


_RE_RAW = re.compile(r'r(#*)"(?:\\.|[^"\\])*"\1', re.S)
_RE_STR = re.compile(r'"(?:\\.|[^"\\])*"', re.S)
_RE_BLOCK = re.compile(r'/\*.*?\*/', re.S)
_RE_LINE = re.compile(r'//[^\n]*')
_RE_CHAR = re.compile(r"'(?:\\.|[^'\\])'")


def strip_comments_and_strings(src: str) -> str:
    """把注释与字符串字面量清空，保留行结构（供按行定位）。

    ⚠️ 这是**正则近似**而非完整词法分析：形如 `"//"` 或 `/* " */` 的
    极端嵌套可能误判。对门而言可接受——误判只会让某字段的读点数偏多
    （漏报），不会产生误报（误报才是棘轮门的成本）。
    落实 R-SCAN-1b：裸 grep 命中不构成证据，故必须先剥离非代码。
    """
    src = _RE_RAW.sub(_blank, src)
    src = _RE_STR.sub(_blank, src)
    src = _RE_CHAR.sub(_blank, src)
    src = _RE_BLOCK.sub(_blank_keep_nl, src)
    src = _RE_LINE.sub(lambda m: " " * len(m.group(0)), src)
    return src


def _blank(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def _blank_keep_nl(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def iter_rs_files(root: Path):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if fn.endswith(".rs"):
                yield Path(dirpath) / fn


_RE_FIELD_BOOL = re.compile(r"^\s*pub\s+([a-z_][a-z0-9_]*)\s*:\s*bool\b")
# 数值型配置字段（如 `*_interval_secs: u64`）—— 「子系统从未被调度」的直接判据
_RE_FIELD_NUM = re.compile(
    r"^\s*pub\s+([a-z_][a-z0-9_]*)\s*:\s*(?:u8|u16|u32|u64|usize|i32|i64|f32|f64)\b"
)
_CONFIG_STRUCT = re.compile(r"(?:Config|Settings|Options|Params|Policy)\b")
_RE_STRUCT = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)")
_RE_DOT_TOKEN = re.compile(r"\.\s*([a-z_][a-z0-9_]*)")


def scan_once(root: Path, want: str = "bool"):
    """单次扫描全仓：返回 (字段候选列表, 按 (文件,字段) 的读点计数, 字段名声明分布)。

    复杂度 O(总行数)。首版是 O(文件 × 字段)，实测超时 RC=124。

    ⚠️ 读点数必须**按声明所在文件**统计，不能按字段名全局聚合：
    AGENTS.md L15「同名 ≠ 同一符号」在本门内的复现 ——
    `enable_account_clustering` 在 `proxy_detection`（活，:290 真读）与
    `anti_distillation`（死）各有一份，全局聚合会把死的那个一并救活。
    实测此类「同名多声明」字段 183 个，其中 161 个因此被误救活。
    """
    fields = []
    read_counts: dict[tuple, int] = {}
    name_decls: dict[str, int] = {}
    for path in iter_rs_files(root):
        try:
            raw = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        code = strip_comments_and_strings(raw)
        lines = code.split("\n")
        struct_name = None
        derives_serde = False
        pending_serde = False
        for idx, line in enumerate(lines, start=1):
            # 属性行先暂存：derive 紧邻 struct 之前，必须由 struct 行消费。
            # ⚠️ 早期版本在 struct 行直接把 derives_serde 重置为 False，
            # 正好清掉紧邻其前的 #[derive(..., Serialize)] ⇒ 分类全部失真。
            if re.match(r"^\s*#\[", line):
                # `uniffi::Record` 与 serde 同族：字段由**Rust 树外**的
                # Swift/Kotlin 消费方读取 ⇒ 只写不读是合法的。
                # 门此前不认识这个边界 ⇒ ffi/types.rs 的 8 个字段全被误判
                # 进「待人工判定」（审计已独立确认它们是假阳性）。
                if ("Serialize" in line or "Deserialize" in line
                        or "uniffi::Record" in line):
                    pending_serde = True
                continue
            sm = _RE_STRUCT.match(line)
            if sm:
                struct_name = sm.group(1)
                derives_serde = pending_serde
                pending_serde = False
                continue
            fm = (
                _RE_FIELD_BOOL.match(line)
                if want == "bool"
                else _RE_FIELD_NUM.match(line)
            )
            if fm and (want == "bool" or (struct_name and _CONFIG_STRUCT.search(struct_name))):
                fields.append(
                    (fm.group(1), path, idx, struct_name, derives_serde)
                )
            for tok in _RE_DOT_TOKEN.findall(line):
                key = (str(path), tok)
                read_counts[key] = read_counts.get(key, 0) + 1
    for name, _, _, _, _ in fields:
        name_decls[name] = name_decls.get(name, 0) + 1
    return fields, read_counts, name_decls


def locate_read_sites(root: Path, field: str, limit: int = 3, only_file: Path | None = None):
    """零读点字段的定位复查（本应为空；有则说明索引口径需修正）。

    ⚠️ `only_file` 必须按**声明所在文件**限定：首版全仓复查，
    于是同名不同符号会互相救活 —— `enable_account_clustering` 在
    `proxy_detection` 的读点把 `anti_distillation` 的死声明救活，
    「二次复查」本身成了 D2 修复的漏斗（已实测复现）。
    """
    pat = re.compile(r"\.\s*" + re.escape(field) + r"\b")
    hits = []
    for path in iter_rs_files(root):
        if only_file is not None and path != only_file:
            continue
        try:
            code = strip_comments_and_strings(path.read_text(encoding="utf-8", errors="replace"))
        except OSError:
            continue
        for idx, line in enumerate(code.split("\n"), start=1):
            if pat.search(line):
                hits.append(f"{path}:{idx}")
                if len(hits) >= limit:
                    return hits
    return hits


def load_baseline(path: Path):
    if not path.exists():
        return set()
    names = set()
    for line in path.read_text(encoding="utf-8").split("\n"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        names.add(line.split("|")[0].strip())
    return names


def main() -> int:
    ap = argparse.ArgumentParser(description="检出「声明了默认值却从未被读取」的配置开关")
    ap.add_argument("--root", default=".")
    ap.add_argument("--strict", action="store_true", help="对基线外新增非零")
    ap.add_argument("--audit", action="store_true", help="打印读点计数与样例位置")
    ap.add_argument(
        "--types", choices=["bool", "numeric"], default="bool",
        help="bool=布尔开关（默认）；numeric=*Config 的数值字段（子系统是否被调度）",
    )
    args = ap.parse_args()

    root = Path(args.root).resolve()
    if not (root / "neotrix-core").exists() and not (root / "crates").exists():
        print(f"dead-flag: 无法在 {root} 下识别 NeoTrix 布局 ⇒ 门自身无法判定", file=sys.stderr)
        return 2

    baseline_path = root / "scripts" / "dead-flag-baseline.txt"
    baseline = load_baseline(baseline_path)

    fields, read_counts, name_decls = scan_once(root, args.types)

    def _reads(name, decl_path):
        own = read_counts.get((str(decl_path), name), 0)
        if name_decls.get(name, 0) == 1:
            return own or sum(v for (f, k), v in read_counts.items() if k == name)
        return own

    alive_names = {
        f[0] for f in fields if _reads(f[0], f[1]) > 0
    }

    dead = []
    for name, decl_path, line_no, struct_name, serde in fields:
        own = read_counts.get((str(decl_path), name), 0)
        if name_decls.get(name, 0) == 1:
            # 该字段名全仓唯一 => 允许跨文件读取，取全仓总和
            reads = own or sum(v for (f, k), v in read_counts.items() if k == name)
        else:
            # 同名多声明 => 只认本文件读点，否则同名不同符号会互相救活
            reads = own
        if reads == 0:
            # 二次复查：确认真的零读点（防止索引口径错误导致误报）
            if not locate_read_sites(root, name, limit=1, only_file=decl_path):
                dead.append((name, decl_path, line_no, struct_name, serde))
            else:
                continue
        elif args.audit:
            print(f"  活{name}: {reads} 读点")

    rel = lambda p: str(Path(p).resolve().relative_to(root))  # noqa: E731

    # ── ⭐ 本门已知的**四个盲区**（2026-10-07 实测逐个确认，记账在
    #    `scripts/dead-flag-baseline.txt`，⛔ 本轮**不改门**）：
    #
    #   ① **读者在 Rust 之外**（跨语言）
    #      字段经 serde 走 IPC → 由 TypeScript 界面读取 ⇒ Rust 侧**结构上**
    #      不可能有读点。例：`FileChange.content_omitted`（界面渲染「内容已略去」）。
    #      ⇒ 这类字段**必然**「零读点」，而它们往往是**活的**。
    #      ⛔ 处置：记 `cross-language-consumer`，**必须写明界面在哪一行读了它**；
    #         ⛔ 绝不可为过门而造一个 Rust 读点（那是为过门而造证据）。
    #
    #   ② **读者在兄弟模块里，且全仓同名多声明**
    #      门按设计「同名多声明 ⇒ 只认本文件读点」（防同名不同符号互相救活）。
    #      ⛔ 但当两处声明是**同一概念**时，这条规则产生**假阳性**。
    #      实证：`LedgerEntry.measured`（nt_store/mod.rs）的真读点是
    #      `nt_store/nt_store_ledger.rs:69` 的 `if entry.measured {…}`，
    #      它承载「未计量则费用强制为 0」这条永不猜测律；
    #      而同名声明 `TurnUsage.measured`（nt_reply_tag.rs:64）把门带进同名分支。
    #      ⇒ 根治需让门**按类型**解析，但实测读点多为 `entry.measured` 这类
    #         **小写变量名**，按类型也认不出 ⇒ 只记账，不改门（改门易造假阴性）。
    #
    #   ③ **「未接线规格」判据是关键词匹配**
    #      `SPEC_MARK_RE` 只认字段上方 6 行内的 `nt-unwired-spec` / `未接线规格`。
    #      ⇒ 它**分不出**「A 已废弃」与「B 没做」—— 只能由人裁决（见本段②的注释）。
    #
    #   ④ **字段总数会因他窗并发改动而漂**
    #      实测：同一份代码 30 分钟内 bool 字段数变了（`1603 → 1602`），
    #      而本窗**没碰** core ⇒ **账本必须带时间戳**（R-SCAN-3 的补充形态：
    #      门输出也会因为别人在改而过期，⛔ 不能照抄上一份快照）。

# ── 第四类：**未接线规格**（2026-10-07 新增）─────────────────
    # 「死配置」有两种**相反**成因：
    #   A. 已废弃        ⇒ 字段是残留，**应该删**
    #   B. 未接线规格    ⇒ 功能声明了但读者没写，字段是**意图声明**，**必须留**
    # ⛔ 门只能回答「有没有人读」，**答不了「是没做还是不做了」**（意图问题）。
    # ⇒ 若把 B 类持续报成「待人工判定」，后来者只能二选一，
    #    而**两个都是错的**（删 A 会销毁规格，留 A 会留垃圾）。
    #
    # 处置：用**代码内的标记**声明「这个字段是 B 类」——
    #   ⭐ 标记必须**长在字段旁边**（而非外部清单文件），
    #      否则清单会与代码漂移，变成第二份「关于代码的说法」。
    #
    # 标记语法：字段**上方 6 行内**出现
    #   `nt-unwired-spec:` 或 `nt-unwired-spec:`的中文变体 `未接线规格`
    SPEC_MARK_RE = re.compile(r"nt-unwired-spec|未接线规格")

    def _is_spec(d):
        try:
            lines = open(d[1], encoding="utf-8", errors="ignore").read().split("\n")
        except OSError:
            return False
        lo = max(0, d[2] - 7)          # d[2] 是 1-based 行号
        return any(SPEC_MARK_RE.search(x) for x in lines[lo:d[2]])

    spec = [d for d in dead if not d[4] and _is_spec(d)]
    spec_set = {(d[0], d[1], d[2]) for d in spec}

    new = [d for d in dead if d[0] not in baseline]
    # ── 第三类：配置噪声 ──
    # 异名近名孪生（`apple_silicon` vs `is_apple_silicon`、`allowed` vs `allow`、
    # `active` vs `is_active`）是历史命名重复：活的那份已接线，死的那份是**遗留字段**。
    # 它们的危害不是「缺能力」而是**污染清单**——待人工判定量因此从 674 降到 513。
    # ⚠️ 判据要求**异名**：同名不同结构体已由按声明文件计读点正确处理，
    #    若把同名也算「孪生」，会把那类正确情形误标成噪声（本轮实测踩过）。
    import difflib as _dl
    def _is_noise(nm):
        return any(x != nm for x in _dl.get_close_matches(nm, alive_names, n=3, cutoff=0.80))

    noise = [d for d in dead if not d[4] and _is_noise(d[0])]
    noise_set = {(d[0], d[1], d[2]) for d in noise}
    external = [d for d in dead if d[4]]
    behavioral = [d for d in dead if not d[4]]
    print(
        f"dead-flag[{args.types}] 字段 {len(fields)} 个；零读点 {len(dead)} 个"
        f"（外部消费者 serde/uniffi {len(external)}、配置噪声 {len(noise)}、"
        f"**未接线规格 {len(spec)}**、"
        f"待人工判定 {len(behavioral) - len(noise) - len(spec)}）；"
        f"基线已裁决 {len(baseline)}；新增 {len(new)}"
    )

    for name, decl_path, line_no, struct_name, serde in sorted(dead, key=lambda d: d[0]):
        tag = "已知" if name in baseline else "新增"
        hint = f" [{struct_name}]" if struct_name and CONFIG_HINT_RE.search(struct_name) else ""
        if serde:
            kind = "外部消费者(serde/uniffi)"
        elif (name, decl_path, line_no) in noise_set:
            kind = "配置噪声(异名近名)"
        elif (name, decl_path, line_no) in spec_set:
            kind = "**未接线规格**（功能声明了但未实现 ⇒⛔ 不要删）"
        else:
            kind = "**待人工判定**"
        print(f"      · {tag} {rel(decl_path)}:{line_no} `pub {name}`{hint}  ({kind})")

    if not dead:
        print("      · 无零读点布尔配置字段")

    if args.strict and new:
        print(f"⛔ 新增 {len(new)} 个疑似死开关（声明了默认值但从未被读取）", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())