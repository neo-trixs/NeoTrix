#!/usr/bin/env python3
"""nt_docclaims.py — 「注释里的可检验断言」vs「代码事实」矛盾检测器 (2026-09-28)

立项背景: 2026-09-28 收口时靠人工阅读发现并修掉 7 处「注释在说谎」—— 注释描述的
行为与代码实际行为不符 (nt_types.rs TurnStatus / nt_channel_serve.rs slice_sleep /
nt_store BotRow / nt_cmd_channels.rs / nt_channel.rs OutboundMessage.edit_of /
nt_workspace.rs MAX_SEARCH_HITS / nt_git.rs check_rel)。这类缺陷比没注释更坏:
读代码的人会信它、并据此做出错误设计 (照着不存在的 cancelled 态去实现 /stop)。

本工具**不是**再去找新的 (那是审计的活, 已做过)。本工具的目标是: 让这一类错误
以后能被**机械发现**, 而不是每次靠人读注释碰运气。

设计的第一原则是**误报率**, 理由: 这类工具唯一的死因是满屏正常代码, 把真信号淹掉
(见 AGENTS.md §R-SCAN-1/2/3 —— 上一轮静态扫描器 12 条告警 2/3 是误报, 且陈旧门记录
比没有门更危险)。因此本工具**宁可漏, 不可滥报**, 判据全部要求「确证矛盾」才给 error。

检测器 (4 类断言 + 1 类豁免):

  D1 `enum_variants`  枚举/变体断言 —— 注释里列出的一组名字, 与代码里真实 enum 变体
                       **逐名比对**, 多一个或少一个都报 error。
                       判定依据: 注释必须 (a) 挂在 `enum` 声明的 doc 位置上,
                       (b) 给出 ≥2 个 identifier 形态的名字, (c) 与真实变体**至少
                       重叠 1 个** —— (c) 是关键精度闸: 重叠 0 说明那串名字压根不是
                       在说这个 enum (实测 corpus 里 "clean run / broken run"、
                       "self-preference / family-bias" 这类会命中宽正则, 靠 (c) 滤掉),
                       此时只在「同时带计数词且计数也对不上」时降级报 warn。
  D2 `numeric_bound`  数字断言 —— 注释里的数字 (2000 / 200 / 1 MiB / 最坏 200ms)
                       与**同位置的 const 字面量**核对一致性。
                       只有两种形态升级到 error: (a) doc 块直接挂在
                       `const NAME: T = <数字字面量>` 上; (b) 注释用反引号点名了某个
                       同文件 const。其余一律 `info 无法静态判定` (不猜)。
  D3 `call_site`      调用点断言 —— 注释自称某符号「已接线 / 被调用 / 生效」→
                       全仓统计**真实调用点数量**, 为 0 则报 warn。
                       永远只到 warn: 跨 crate / trait 实现 / 宏展开都能骗过符号计数,
                       本工具不假装能判。永远不到 error。
  D4 `absolute`       能力断言 —— 注释含「恒为 / 永远 / 一律 / 绝不 / 全部 / 所有 /
                       总是 / 必然」等绝对量词 → info, 提示需人工复核。
                       纯提示, **不判错** (这类无法可靠自动化)。
  X  `declared_gap`   已承认的缺口 → **豁免**。
                       两个机制: (1) 全局剥离 `「…」` 引用段 —— 本项目实测约定是
                       「用 「」 引起来的话 = 在讨论一句话, 不是在断言」, 4/4 的历史
                       错误注释都用 「」 引着错的那句 (nt_types.rs:9 / nt_workspace.rs:42
                       / nt_git.rs:19 / nt_cmd_channels.rs:308), 故引用段一律不作断言;
                       (2) 行级否定/纠正标记 (早先 / 曾经 / 不能写成 / 别当成 / 半接 /
                       缺口 / 尚未 / 不生效 / 存了但 / 待办 / 得改代码 …)。

输出分级: error (确证矛盾) / warn (可疑, 需人看) / info (绝对量词 + 无法判定) /
skip (已声明缺口或非断言语境, 仅计数不打印)。

自测: `--self-test` 用**内联小样本**(不读仓库) 覆盖每个检测器, 并且做**变异注入**:
逐个把一侧改坏(改变体名 / 改常量值 / 删调用点) 必须翻红, 改回必须变绿。
另有一组**反误报断言**: nt_workspace / nt_git 那种「已如实登记的缺口」样本必须
**零 error** —— 误报率是这类工具唯一的死因, 必须被测试钉住。

用法:
    python3 scripts/ops/nt_docclaims.py [路径...]          # 默认扫 neotrix-core/src crates/*/src
    python3 scripts/ops/nt_docclaims.py --json
    python3 scripts/ops/nt_docclaims.py --self-test
退出码: 0 = 无 error; 1 = 有 error (供 CI/门禁); 2 = 用法/IO 错误。
只读审计器 —— 本脚本**绝不写入**任何被检查的源文件。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Iterable, Iterator, Sequence

VERSION = 1
DEFAULT_ROOTS = ["neotrix-core/src"] + [str(p) for p in Path("crates").glob("*/src")]

# 扫描时永远跳过的目录 (worktrees / 产物 / 依赖 —— 里面是**别人窗口的**代码副本,
# 扫了会产生"同一处注释报三次"的三倍噪声)。
SKIP_DIRS = {
    ".git", "target", "node_modules", ".worktrees", "dist", "build",
    "__pycache__", ".venv", "venv", ".next", ".pytest_cache",
}

SOURCE_SUFFIXES = {".rs", ".ts", ".tsx"}

# ---------------------------------------------------------------------------
# 分级
# ---------------------------------------------------------------------------
LEVEL_ERROR = "error"
LEVEL_WARN = "warn"
LEVEL_INFO = "info"
LEVEL_SKIP = "skip"
LEVELS = (LEVEL_ERROR, LEVEL_WARN, LEVEL_INFO, LEVEL_SKIP)


@dataclass
class Finding:
    level: str
    rule: str
    path: str
    line: int
    doc: str
    message: str
    claim: dict = field(default_factory=dict)

    def render(self) -> str:
        head = f"{self.level.upper():5s} [{self.rule}] {self.path}:{self.line}"
        body = f"       注释: {self.doc}"
        fact = f"       事实: {self.message}"
        return "\n".join([head, body, fact])


# ---------------------------------------------------------------------------
# 引用段剥离 / 否定语境
# ---------------------------------------------------------------------------
# 本项目约定: 「」/`""` 引起来的是「被讨论的那句话」, 不是断言。
# 4/4 已确认的历史错误注释都用 「」 引着错的那句, 故引用段一律不作断言。
QUOTE_SPAN_RE = re.compile(r"[「」『』“”][^「」『』“”\n]*[「」『』“”]|“[^”\n]*”")

# 行级否定 / 纠正 / 已承认缺口标记。命中即豁免该行的断言。
# 取宽是有意的: 过宽的豁免只损失漏报, 过宽的判错会制造噪声淹没真信号。
NEGATION_MARKERS = (
    # 引用旧错
    "早先", "曾经", "历史上", "误写", "写错", "以前这行", "之前这行", "原来写",
    # 显式禁止某种说法
    "不能写成", "别写成", "别当成", "别在这里写", "不要当成", "不要写成",
    "别当", "误以为", "会让人以为",
    # 半接 / 缺口
    "半接", "缺口", "尚未", "未接线", "还没接", "没接", "没有实现", "没实现",
    "不生效", "存了但", "待办", "得改代码", "不是改注释", "未接",
    "不查", "不检查", "够不着", "没有任何一条", "产不出", "停不了",
)

# 绝对量词 (D4) —— 只提示, 不判错。
ABSOLUTE_QUANTIFIERS = (
    "恒为", "永远", "一律", "绝不", "全部", "所有", "总是", "必然", "必定",
    "任何时候", "百分百", "万无一失", "永远不",
)

# 「声称已接线」动词 (D3)
WIRING_VERBS = (
    "已接线", "接线", "被调用", "调用了", "已生效", "生效了", "接上了",
    "已实现", "实现了", "会调用", "走的是", "被使用", "已挂载", "挂上了",
)

# 数字的「界」语境词 (D2) —— 只有落在这个语境里的数字才是「可检验的界」。
# 分两类, 这个区分是精度关键:
#   * ANYWHERE: 出现在数字前后 8 字符内即可
#   * POSITIONAL: 必须**紧邻**数字 (中间只允许空白/×)
# 为什么「前」「内」只能当 POSITIONAL: 「前」会撞「前提/前缀/提前」,
# 「内」会撞「内存/内容/境内」—— 实测这两条单独贡献了整批误报
# (「cap=5 时**近一半前提是凑数的**」被当成「前 5 个」的界断言)。
BOUND_CONTEXT_ANYWHERE = (
    "上限", "最多", "不超过", "不超", "最坏", "至少", "至多", "截断", "封顶", "限制", "以内",
)
BOUND_CONTEXT_POSITIONAL = ("前", "每", "上", "下", "超")
BOUND_POSITIONAL_AFTER = ("内", "封顶", "上限", "为止")


def strip_quotes(text: str) -> tuple[str, list[str]]:
    """剥离「…」引用段, 返回 (剩余文本, 被剥离的引用段列表)。"""
    quoted: list[str] = []
    kept: list[str] = []
    idx = 0
    for m in QUOTE_SPAN_RE.finditer(text):
        kept.append(text[idx:m.start()])
        quoted.append(m.group(0))
        idx = m.end()
    kept.append(text[idx:])
    return "".join(kept), quoted


def negation_reason(text: str) -> str | None:
    for marker in NEGATION_MARKERS:
        if marker in text:
            return marker
    return None


# ---------------------------------------------------------------------------
# 文档注释块 → 挂载的代码项
# ---------------------------------------------------------------------------
DOC_RE = re.compile(r"^\s*(///|//!)(.*)$")
BLOCK_RE = re.compile(r"^\s*/\*")
ITEM_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:async\s+|unsafe\s+|const\s+|extern\s+\"[^\"]+\"\s+)*"
    r"(fn|enum|struct|const|static|type|trait|impl|mod|macro_rules!|union)\b"
)
ENUM_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)"
)
CONST_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*"
    r"(?::\s*([A-Za-z_][A-Za-z0-9_:<>, ]*?))?\s*=\s*(.+?)\s*;"
)
FN_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?"
    r"fn\s+([A-Za-z_][A-Za-z0-9_]*)"
)


@dataclass
class DocBlock:
    """一段连续文档注释 + 它所挂载的代码项 (可能为 None, 如变体/字段内注释)。"""
    path: str
    lines: list[tuple[int, str]]      # (1-based lineno, 已去 /// 前缀的正文)
    item_lineno: int | None
    item_text: str

    def joined(self) -> str:
        return " ".join(t for _, t in self.lines)

    def cleaned(self) -> tuple[str, list[tuple[int, str]]]:
        """逐行剥离引用段。返回 (拼接文本, [(lineno, 保留文本)])。"""
        out: list[tuple[int, str]] = []
        for lineno, text in self.lines:
            kept, _quoted = strip_quotes(text)
            out.append((lineno, kept))
        return " ".join(t for _, t in out), out


def scan_doc_blocks(text: str, path: str) -> Iterator[DocBlock]:
    """线性扫描, 把每段连续文档注释关联到它下面第一个真正的代码项。

    跳过 `#[...]` 属性 (含跨行) 与空行; 若下一个非属性行不是项声明 (例如落在
    struct 字段 / enum 变体上), 则 item=None —— D1/D2a 不适用。
    """
    lines = text.splitlines()
    i = 0
    n = len(lines)
    while i < n:
        m = DOC_RE.match(lines[i])
        if not m:
            # 非文档注释的行注释: 归零, 不并入块
            i += 1
            continue
        block: list[tuple[int, str]] = []
        while i < n:
            dm = DOC_RE.match(lines[i])
            if not dm:
                break
            block.append((i + 1, dm.group(2).strip()))
            i += 1
        # 向前跳过属性与空行 (支持 #[cfg(...)] 跨行, 用方括号深度跟踪)
        j = i
        depth = 0
        while j < n:
            stripped = lines[j].strip()
            if stripped == "" and depth == 0:
                j += 1
                continue
            if stripped.startswith("#") or stripped.startswith("!["):
                depth += stripped.count("[") - stripped.count("]")
                j += 1
                continue
            break
        if j < n and ITEM_RE.match(lines[j]):
            yield DocBlock(path, block, j + 1, lines[j])
        else:
            yield DocBlock(path, block, None, "")


# ---------------------------------------------------------------------------
# 代码事实抽取
# ---------------------------------------------------------------------------
def strip_code_noise(text: str) -> str:
    """去掉行注释与字符串字面量, 只留结构 token。"""
    out: list[str] = []
    for line in text.splitlines():
        code = re.sub(r'"(?:\\.|[^"\\])*"', '""', line)
        code = re.sub(r"//.*$", "", code)
        code = re.sub(r"'(?:\\.|[^'\\])'", "''", code)
        out.append(code)
    return "\n".join(out)


def strip_block_comments(text: str) -> str:
    return re.sub(r"/\*.*?\*/", " ", text, flags=re.S)


def extract_enum_body(text: str, name: str) -> str | None:
    """取出 `enum <name> { ... }` 的花括号体 (括号深度跟踪, 容忍嵌套与属性)。"""
    code = strip_block_comments(strip_code_noise(text))
    m = re.search(r"\benum\s+" + re.escape(name) + r"\b", code)
    if not m:
        return None
    start = code.find("{", m.end())
    if start < 0:
        return None
    depth = 0
    for idx in range(start, len(code)):
        ch = code[idx]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return code[start + 1:idx]
    return code[start + 1:]


IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def extract_variants(body: str) -> list[str]:
    """取 enum 体的一级变体名 (跳过属性与文档注释; 兼容元组/结构体/判别式变体)。"""
    if not body:
        return []
    lines: list[str] = []
    for raw in body.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith("//") or line.startswith("///"):
            continue
        lines.append(line)
    joined = " ".join(lines)
    variants: list[str] = []
    depth = 0
    token = ""
    i = 0
    while i < len(joined):
        ch = joined[i]
        if ch in "{([<":
            depth += 1
        elif ch in "})]>":
            depth -= 1
        if ch == "," and depth == 0:
            name = first_variant_name(token)
            if name:
                variants.append(name)
            token = ""
        else:
            token += ch
        i += 1
    name = first_variant_name(token)
    if name:
        variants.append(name)
    return variants


def first_variant_name(token: str) -> str | None:
    t = token.strip()
    if not t:
        return None
    t = re.sub(r"^#\s*", "", t)
    m = re.match(r"([A-Z][A-Za-z0-9_]*)\b", t)
    if m and m.group(1) not in {"Self", "crate", "super", "self"}:
        return m.group(1)
    # 元组变体 `Variant(..)` 已被上面的 \b 命中; 兜底: 全小写的 type alias
    m = re.match(r"([a-z_][A-Za-z0-9_]*)\b", t)
    return m.group(1) if m else None


def to_snake(name: str) -> str:
    s = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)
    return s.replace("-", "_").lower()


def norm(name: str) -> str:
    return to_snake(name.replace("`", "").strip())


CN_DIGITS = {
    "一": 1, "两": 2, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6,
    "七": 7, "八": 8, "九": 9, "十": 10,
}


def parse_count(word: str) -> int | None:
    word = word.strip()
    if word.isdigit():
        return int(word)
    if word in CN_DIGITS:
        return CN_DIGITS[word]
    if len(word) == 2 and word[0] == "十":
        return 10 + CN_DIGITS.get(word[1], 0)
    if len(word) == 2 and word[1] == "十":
        return CN_DIGITS.get(word[0], 0) * 10
    if len(word) == 3 and word[1] == "十":
        a, b = CN_DIGITS.get(word[0], 0), CN_DIGITS.get(word[2], 0)
        if word[0] in CN_DIGITS and word[2] in CN_DIGITS:
            return a * 10 + b
    return None


COUNT_WORD_RE = re.compile(
    r"([一二两三四五六七八九十]+|\d+)\s*(?:个)?\s*(?:态|种|状态|档位|类|项|级别|变体)"
)
# 注释里「identifier/identifier/...」形态的名字表
SLASHLIST_RE = re.compile(
    r"(?<![A-Za-z0-9_])((?:[a-z][a-z0-9_]*)(?:\s*/\s*[a-z][a-z0-9_]*)+)(?![A-Za-z0-9_])"
)
BACKTICKED_RE = re.compile(r"`([A-Za-z_][A-Za-z0-9_]*)`")
# 定义式断言: 注释里直接写 `SYM = 5`。这是**自足**的可检验断言 ——
# 符号被点名、数值被断言, 不依赖附近有没有「上限/最多」这类界语境词。
# (「单轮前提数上界（默认 `DEFAULT_PER_ROUND_CAP` = 5）」里的「上界」在 20 字符外,
#  靠邻近词窗根本够不着 —— 但这条断言本身比邻近词窗强得多。)
DEFINITIONAL_EQ_RE = re.compile(
    r"`([A-Za-z_][A-Za-z0-9_]*)`\s*=\s*(\d[\d_]*)"
)
NUMBER_RE = re.compile(r"(\d[\d_]*)(\s*)([A-Za-z]*|毫秒|秒|条|次|个|层|字节)?")

# 名字表的候选 span: 括号内容, 或冒号之后到句末。
# 精度要点: 不再用「正则扫到哪算哪」, 而是**取整段 span 再按 `/` 切**,
# 并要求**每一段都是裸 identifier** —— 这样 "clean run / broken run"、
# "core/recall/archive tiers"、"Axon 的 will/may/review 分组" 会被整段否掉
# (它们不是名字清单, 是散文), 而 "（五态：done/continue/…）" 保留。
SPAN_PAREN_RE = re.compile(r"[（(]([^（()）]*)[)）]")
SPAN_COLON_RE = re.compile(r"[：:]\s*([^：:。；;，,\n]{2,120})")
# span 开头可能带的计数前缀: 「五态：」「3 种:」「five states:」
COUNT_PREFIX_RE = re.compile(
    r"^\s*(?:[一二两三四五六七八九十]+|\d+)\s*(?:个)?\s*(?:态|种|状态|档位|类|项|级别|变体)?\s*[：:]?\s*"
)
BARE_IDENT_RE = re.compile(r"^[a-z][a-z0-9_]*$")

# 数字的「单位种类」—— 不同种类之间**不可直接比较** (1 KiB 与 1024 个条目不是一个量纲)
BYTE_UNITS = {"B", "KB", "MB", "GB", "TB", "KiB", "MiB", "GiB", "字节"}
TIME_UNITS = {"ms", "毫秒", "s", "秒", "min", "分钟", "h", "小时"}
COUNT_UNITS = {"", "条", "次", "个", "层", "项", "轮", "档", "级", "倍"}

# 表达式 / 轶事语境: 落在这里的数字不是「对这个常量的界」的断言
EXPR_MARKERS = ("×", "*", "+", "=", "倍")
HEDGE_MARKERS = ("约", "大约", "接近", "实测", "曾", "已", "历史", "~", "≈", ">>", "P90", "P99")
# 参与算式的运算符: 数字两侧出现这些 → 它属于一个算式, 不是独立的界断言
ARITH_CHARS = ("×", "*", "+", "倍")
# 裸 CamelCase 类型名 (未加反引号时也要能认出「主语是别的类型」)
CAMEL_TYPE_RE = re.compile(r"\b([A-Z][a-z]+(?:[A-Z][a-z0-9]+)+)\b")


def unit_kind(unit: str) -> str:
    if unit in BYTE_UNITS:
        return "bytes"
    if unit in TIME_UNITS:
        return "time"
    return "count"


def num_is_standalone(text: str, m: re.Match) -> bool:
    """数字必须**独立**成一个 token。

    否掉: `E8` 的 8、`D-6` 的 6、`缺陷 #5` 的 5、`P0-6` 的 6、`#[test]`。
    这些是标识符/工单号的一部分, 不是可检验的界。
    """
    start = m.start(1)
    before = text[start - 1] if start > 0 else ""
    if before and (before.isalnum() or before in "_#"):
        return False
    # `-` 只有在前一个字符也是字母数字时才是「工单号/版本号」的一部分
    # (`D-6` / `P0-6` / `v2-3` 的 6、6、3), 否则是负号或范围连接号。
    if before == "-" and start >= 2 and text[start - 2].isalnum():
        return False
    after_m = m.end(1)
    after = text[after_m] if after_m < len(text) else ""
    if after and (after.isalnum() or after == "_"):
        return False
    return True


def is_expr_or_hedge(text: str, m: re.Match) -> str | None:
    """数字是否落在表达式或轶事语境里。返回原因 (可读) 或 None。

    算术链判定: 若该数字与**相邻数字**之间隔着 `× * + 倍`，则它属于一个
    算式 (`2 × 重试上限 3` = 5), 不是对某个常量的独立界断言。
    """
    start, end = m.start(1), m.end(1)
    before = text[max(0, start - 12):start]
    after = text[end:end + 12]
    for mk in EXPR_MARKERS:
        if mk not in (before[-3:] + after[:3]):
            continue
        # `SYM = N` 是**定义式**断言 (`DEFAULT_PER_ROUND_CAP` = 5) ——
        # 这是最可检验的一种claim, 绝不能当算式放过去。
        # 只有当 `=` 左边不是符号时才当算式 (`2 × 3 = 6`)。
        if mk == "=":
            lhs = before[:before.rfind("=")] if "=" in before else before
            if BACKTICKED_RE.search(lhs) or re.search(r"[A-Z_][A-Z0-9_]{2,}", lhs) \
               or CAMEL_TYPE_RE.search(lhs):
                continue
        return f"表达式 ({mk})"
    for mk in ARITH_CHARS:
        if mk in text[start:end + 1] and mk != "=":
            return f"算式 ({mk})"
    # 与相邻数字之间的算术链
    all_nums = [(mm.start(1), mm.end(1)) for mm in NUMBER_RE.finditer(text)]
    for i, (s, e) in enumerate(all_nums):
        if s != start:
            continue
        for j in (i - 1, i + 1):
            if 0 <= j < len(all_nums):
                gap = text[e:all_nums[j][0]] if j > i else text[all_nums[j][1]:s]
                if any(a in gap for a in ARITH_CHARS):
                    return "算式链 (与相邻数字由 ×/+ 相连)"
    for mk in HEDGE_MARKERS:
        if mk in before:
            return f"轶事/约数 ({mk})"
    return None


def candidate_lists(text: str) -> list[list[str]]:
    """从一行注释里抽出**全部是裸 identifier** 的 `/` 分隔名字表。

    精度闸在 `all(BARE_IDENT_RE)` 上: 只要有一段带空格/中文/尾随修饰
    ("clean run"、"archive tiers"、"分组"), 整段就不是名字清单 → 不作断言。
    """
    out: list[list[str]] = []
    spans = [m.group(1) for m in SPAN_PAREN_RE.finditer(text)]
    spans += [m.group(1) for m in SPAN_COLON_RE.finditer(text)]
    for span in spans:
        s = COUNT_PREFIX_RE.sub("", span.strip())
        if "/" not in s:
            continue
        parts = [p.strip().strip("`") for p in s.split("/")]
        parts = [p for p in parts if p]
        if len(parts) < 2:
            continue
        if not all(BARE_IDENT_RE.match(p) for p in parts):
            continue
        if len(set(parts)) != len(parts):
            continue
        out.append(parts)
    return out


# ---------------------------------------------------------------------------
# 符号索引 (供 D3)
# ---------------------------------------------------------------------------
class SymbolIndex:
    """全仓 `fn` 定义名 → 定义位置。用于 D3 的「声称已接线但零调用点」。"""

    def __init__(self) -> None:
        self.defs: dict[str, list[tuple[str, int]]] = {}
        self.call_counts: dict[str, int] = {}
        self._built = False

    def add_def(self, name: str, path: str, line: int) -> None:
        self.defs.setdefault(name, []).append((path, line))

    def is_defined(self, name: str) -> bool:
        return name in self.defs

    def build_counts(self, files: dict[str, str]) -> None:
        for name in self.defs:
            self.call_counts[name] = 0
        for path, text in files.items():
            code = strip_block_comments(strip_code_noise(text))
            for lineno, line in enumerate(code.splitlines(), 1):
                for m in re.finditer(r"(?<![A-Za-z0-9_])([A-Za-z_][A-Za-z0-9_]*)\s*(?:::|\()", line):
                    nm = m.group(1)
                    if nm not in self.call_counts:
                        continue
                    if FN_RE.match(line) and re.search(r"\bfn\s+" + re.escape(nm) + r"\b", line):
                        continue  # 定义行自身, 不算调用点
                    self.call_counts[nm] += 1
        self._built = True

    def call_count(self, name: str) -> int:
        return self.call_counts.get(name, 0)


# ---------------------------------------------------------------------------
# 分析器
# ---------------------------------------------------------------------------
@dataclass
class Stats:
    files: int = 0
    comment_lines: int = 0
    doc_blocks: int = 0
    findings: list[Finding] = field(default_factory=list)
    per_rule: dict[str, dict[str, int]] = field(default_factory=dict)

    def add(self, f: Finding) -> None:
        self.findings.append(f)
        bucket = self.per_rule.setdefault(f.rule, {lvl: 0 for lvl in LEVELS})
        bucket[f.level] = bucket.get(f.level, 0) + 1

    def counts(self) -> dict[str, int]:
        out = {lvl: 0 for lvl in LEVELS}
        for f in self.findings:
            out[f.level] += 1
        return out


class Analyzer:
    def __init__(self, index: SymbolIndex | None = None, check_call_sites: bool = True) -> None:
        self.index = index
        self.check_call_sites = check_call_sites and index is not None

    # -- D1 ---------------------------------------------------------------
    def check_enum_claim(self, blk: DocBlock, cleaned: list[tuple[int, str]], stats: Stats) -> None:
        if blk.item_lineno is None:
            return
        m = ENUM_RE.match(blk.item_text)
        if not m:
            return
        enum_name = m.group(1)
        variants = self._variants_of(blk, enum_name)
        if not variants:
            return
        # 两种归一: snake (NeedsClarification→needs_clarification) 与去下划线
        # (XHigh→xhigh)。注释写 xhigh 而代码写 XHigh 不是「注释说谎」, 是大小写风格。
        code_snake = {to_snake(v): v for v in variants}
        code_flat = {to_snake(v).replace("_", ""): v for v in variants}

        for lineno, text in cleaned:
            if negation_reason(text):
                continue
            for claimed_raw in candidate_lists(text):
                claimed_snake = {to_snake(c) for c in claimed_raw}
                claimed_flat = {c.replace("_", "") for c in claimed_raw}
                overlap = claimed_snake & set(code_snake)
                missing = sorted(claimed_snake - set(code_snake))        # 注释有、代码无
                absent = sorted(set(code_snake) - claimed_snake)          # 代码有、注释无
                if claimed_flat == {c.replace("_", "") for c in code_snake}:
                    missing = absent = []                              # 仅下划线差异 → 无事
                if not missing and not absent:
                    continue
                if not overlap:
                    stats.add(Finding(
                        LEVEL_SKIP, "enum_variants", blk.path, lineno, text.strip(),
                        f"名字表 {'/'.join(claimed_raw)} 与 `enum {enum_name}` 零重叠 "
                        f"(实际 {', '.join(sorted(code_snake.values()))}) —— "
                        f"不是在说这个 enum 的变体, 不作断言",
                        {"enum": enum_name, "claimed": claimed_raw},
                    ))
                    continue
                stats.add(Finding(
                    LEVEL_ERROR, "enum_variants", blk.path, lineno, text.strip(),
                    f"注释列的变体与 `enum {enum_name}` 实际变体不一致: "
                    f"多出 {missing or '无'}, 漏列 {absent or '无'} "
                    f"(实际 {', '.join(sorted(code_snake.values()))})",
                    {"enum": enum_name, "claimed": claimed_raw,
                     "code_variants": sorted(code_snake.values()),
                     "missing_in_code": missing, "absent_from_comment": absent},
                ))
            cw = self._count_word(text, enum_name)
            if cw is not None and cw != len(code_snake):
                already = any(
                    f.rule == "enum_variants" and f.level == LEVEL_ERROR and f.line == lineno
                    for f in stats.findings
                )
                if not already:
                    stats.add(Finding(
                        LEVEL_ERROR, "enum_variants", blk.path, lineno, text.strip(),
                        f"注释称 {cw} 个变体, `enum {enum_name}` 实际 {len(code_snake)} 个 "
                        f"({', '.join(sorted(code_snake.values()))})",
                        {"enum": enum_name, "claimed_count": cw,
                         "code_variants": sorted(code_snake.values())},
                    ))

    def _variants_of(self, blk: DocBlock, enum_name: str) -> list[str]:
        text = getattr(blk, "_src", None)
        if text:
            body = extract_enum_body(text, enum_name)
            return extract_variants(body or "")
        return []

    def _count_word(self, text: str, enum_name: str) -> int | None:
        """抽「N 态/种」这类计数。两个精度闸:

        1. N 必须**独立成 token** —— 否则 `E8 状态` 的 8 会被当成「8 个状态」。
        2. **主语必须是本 enum** —— 计数附近若点名了别的类型
           (`SpecialistType 共 15 种`), 那句话说的不是本 enum → 不作断言。
        """
        for m in COUNT_WORD_RE.finditer(text):
            start = m.start(1)
            before = text[start - 1] if start > 0 else ""
            # 注意用 m.end() (整个匹配之后) 而不是 m.end(1) (计数数字之后) ——
            # 后者紧跟着量词「态/种」, 会被误判成「数字后面粘着字母数字」。
            after = text[m.end()] if m.end() < len(text) else ""
            if before and (before.isalnum() or before in "_#"):
                continue
            if after and (after.isalnum() or after == "_"):
                continue
            if before == "-" and start >= 2 and text[start - 2].isalpha():
                continue
            n = parse_count(m.group(1))
            if n is None:
                continue
            # 闸 2: 主语。计数句若点名了别的类型, 说的就不是本 enum。
            others = [o for o in re.findall(r"`([A-Z][A-Za-z0-9_]*)`", text) if o != enum_name]
            others += [o for o in re.findall(r"\b([A-Z][A-Z0-9_]{2,})\b", text)
                       if o != enum_name.upper()]
            others += [o for o in CAMEL_TYPE_RE.findall(text) if o != enum_name]
            if others:
                continue
            return n
        return None


    # -- D2 ---------------------------------------------------------------
    def check_numeric_claim(
        self, blk: DocBlock, cleaned: list[tuple[int, str]], file_text: str, stats: Stats
    ) -> None:
        consts = {m.group(1): parse_int_literal(m.group(3))
                  for m in (CONST_RE.match(l) for l in file_text.splitlines()) if m}
        consts = {k: v for k, v in consts.items() if v is not None}

        # D2a: doc 块直接挂在 const 上 → 与字面量核对
        if blk.item_lineno is not None:
            cm = CONST_RE.match(blk.item_text)
            if cm:
                const_name, value = cm.group(1), cm.group(3)
                lit = parse_int_literal(value)
                if lit is not None:
                    self._compare_block(
                        blk, cleaned, const_name, lit, stats, block_level=True
                    )
                    return
        # D2a': 注释用反引号点名了同文件某个 const → 核对
        for lineno, text in cleaned:
            if negation_reason(text):
                continue
            for sym in BACKTICKED_RE.findall(text):
                if sym in consts:
                    self._compare_block(
                        blk, [(lineno, text)], sym, consts[sym], stats, block_level=False
                    )
        # D2b: 界语境的数字但定不到常量 → info 无法静态判定
        for lineno, text in cleaned:
            if negation_reason(text):
                continue
            if any(b in BACKTICKED_RE.findall(text) for b in consts):
                continue
            for num, unit, ctx in bound_numbers(text):
                why = is_expr_or_hedge(text, _number_match(text, num))
                stats.add(Finding(
                    LEVEL_INFO, "numeric_bound_unresolved", blk.path, lineno, text.strip(),
                    (f"界语境的数字 {num}{unit} ({ctx}) 落在{why}, 或定不到对应 const"
                     if why else
                     f"界语境的数字 {num}{unit} ({ctx}) 在同文件找不到对应 const/字面量, "
                     f"无法静态判定 (需人工核对)"),
                    {"claimed": num + unit, "context": ctx},
                ))

    def _compare_block(
        self, blk: DocBlock, cleaned: list[tuple[int, str]], const_name: str,
        lit: int, stats: Stats, block_level: bool,
    ) -> None:
        """把 doc 块的界数字与常量核对。三个精度闸 (按顺序):

        1. **块级 any-match**: 整个 doc 块里只要**有一个**界数字等于常量值,
           就算这块与常量同步 —— 块里其它数字是散文/历史/推演, 不是对常量的断言。
           (例: `nt_premise_selector` 的常量旁注记着「由 5 降为 3」并引用 cap=5 的
           实测数据 —— 整块自洽, 块内任何单行都不该被单独挑出来报错。)
        2. **量纲闸**: 注释数字带 KiB/MB/ms/秒 而常量是裸 `usize` 计数 →
           不可比 → info (1 KiB 与 1024 个条目不是一个量纲)。
        3. **表达式/轶事闸**: `2 × 重试上限 3`、`实测最大约 4MB`、`曾达 935` →
           info。
        """
        unit_of_const = "count"          # Rust 里 const 的量纲要靠类型+名字猜, 这里保守取 count
        seen: set[int] = set()
        for _lineno, text in cleaned:
            for num, unit, ctx in bound_numbers(text):
                n = normalize_int(num)
                if n in seen:
                    continue
                seen.add(n)
        # 闸 1
        for lineno, text in cleaned:
            if negation_reason(text):
                continue
            for num, unit, ctx in bound_numbers(text):
                n = normalize_int(num)
                mm = _number_match(text, num)
                if unit and unit_kind(unit) != unit_of_const and unit not in COUNT_UNITS:
                    stats.add(Finding(
                        LEVEL_INFO, "numeric_bound_unit", blk.path, lineno, text.strip(),
                        f"注释数字 {num}{unit} 带单位 ({unit_kind(unit)}), 而 `{const_name}` "
                        f"= {lit} 是裸计数 —— 量纲不可比, 需人工核对 (不判错)",
                        {"const": const_name, "claimed": num + unit, "code_value": lit},
                    ))
                    continue
                why = is_expr_or_hedge(text, mm)
                if why:
                    stats.add(Finding(
                        LEVEL_INFO, "numeric_bound_expr", blk.path, lineno, text.strip(),
                        f"数字 {num}{unit} 落在{why} —— 是推演/轶事, 不是对 `{const_name}` "
                        f"(= {lit}) 的界断言 (不判错)",
                        {"const": const_name, "claimed": num + unit, "code_value": lit},
                    ))
                    continue
                if n == lit:
                    continue      # 该数字与常量一致
                # 块级 any-match: 整块里有任一数字 == lit → 整块自洽
                if block_level and lit in seen:
                    continue
                stats.add(Finding(
                    LEVEL_ERROR, "numeric_bound", blk.path, lineno, text.strip(),
                    f"注释说 {num}{unit} (界语境: {ctx}), 但 `{const_name}` 字面量是 {lit}",
                    {"const": const_name, "claimed": num + unit,
                     "code_value": lit, "context": ctx},
                ))

    # -- D3 ---------------------------------------------------------------
    def check_call_site_claim(
        self, blk: DocBlock, cleaned: list[tuple[int, str]], stats: Stats
    ) -> None:
        if not self.check_call_sites or self.index is None:
            return
        for lineno, text in cleaned:
            if negation_reason(text):
                continue
            if not any(v in text for v in WIRING_VERBS):
                continue
            for sym in BACKTICKED_RE.findall(text):
                if not self.index.is_defined(sym):
                    continue
                n = self.index.call_count(sym)
                if n == 0:
                    stats.add(Finding(
                        LEVEL_WARN, "call_site", blk.path, lineno, text.strip(),
                        f"注释声称 `{sym}` 已接线/被调用, 但全仓真实调用点 = 0 "
                        f"(跨 crate/宏/反射可能骗过本统计, 需人工确认)",
                        {"symbol": sym, "call_sites": 0},
                    ))

    # -- D4 ---------------------------------------------------------------
    def check_absolute(self, blk: DocBlock, cleaned: list[tuple[int, str]], stats: Stats) -> None:
        for lineno, text in cleaned:
            if negation_reason(text):
                continue
            hit = next((q for q in ABSOLUTE_QUANTIFIERS if q in text), None)
            if hit:
                stats.add(Finding(
                    LEVEL_INFO, "absolute_qualifier", blk.path, lineno, text.strip(),
                    f"绝对量词「{hit}」—— 能力断言, 无法可靠自动化, 仅提示人工复核 (不判错)",
                    {"quantifier": hit},
                ))

    # -- 汇总 -------------------------------------------------------------
    def analyze_block(self, blk: DocBlock, file_text: str, stats: Stats) -> None:
        stats.doc_blocks += 1
        stats.comment_lines += len(blk.lines)
        joined_clean, cleaned = blk.cleaned()
        quoted_total = sum(len(strip_quotes(t)[1]) for _, t in blk.lines)
        if negation_reason(joined_clean):
            for lineno, text in cleaned:
                if not text.strip():
                    continue
                stats.add(Finding(
                    LEVEL_SKIP, "declared_gap", blk.path, lineno, text.strip(),
                    f"已声明的缺口/纠正语境 (标记: {negation_reason(joined_clean)}) —— 豁免",
                    {},
                ))
            return
        if quoted_total:
            stats.add(Finding(
                LEVEL_SKIP, "quoted_claim", blk.path, blk.lines[0][0], blk.joined()[:120],
                f"{quoted_total} 处 「…」 引用段 —— 本项目约定: 引起来的是在讨论的句子, 不作断言",
                {},
            ))
        self.check_enum_claim(blk, cleaned, stats)
        self.check_numeric_claim(blk, cleaned, file_text, stats)
        self.check_call_site_claim(blk, cleaned, stats)
        self.check_absolute(blk, cleaned, stats)


# ---------------------------------------------------------------------------
# 数字工具
# ---------------------------------------------------------------------------
def parse_int_literal(value: str) -> int | None:
    v = value.strip()
    m = re.match(r"^(\d[\d_]*)", v)
    if not m:
        return None
    return int(m.group(1).replace("_", ""))


def normalize_int(num: str) -> int:
    return int(num.replace("_", ""))


def _number_match(text: str, num: str) -> re.Match:
    """找回该行里第一个等于 `num` 的数字 match (供 is_expr_or_hedge 定位上下文)。"""
    for m in NUMBER_RE.finditer(text):
        if m.group(1) == num:
            return m
    return re.match(r"\S", num)


def bound_numbers(text: str) -> list[tuple[str, str, str]]:
    """取处在「界语境」里的数字: 返回 (数字, 单位, 命中的语境词)。

    两道精度闸:
    1. 数字必须**独立成 token** (`num_is_standalone`) —— 否掉 `E8 状态` 的 8、
       `D-6` 的 6、`缺陷 #5` 的 5: 它们是标识符/工单号的一部分。
    2. 数字必须紧邻一个界语境词 (前后 8 字符内) 或带一个**量纲单位** ——
       否则「三个字段」这种普通计数也会被当成可检验的界。
    """
    out: list[tuple[str, str, str]] = []
    definitional: dict[str, str] = {}
    for dm in DEFINITIONAL_EQ_RE.finditer(text):
        definitional[dm.group(2)] = f"定义式 `{dm.group(1)}` = {dm.group(2)}"
    for m in NUMBER_RE.finditer(text):
        if not num_is_standalone(text, m):
            continue
        num, unit = m.group(1), (m.group(3) or "")
        if num in definitional:
            out.append((num, unit, definitional[num]))
            continue
        start, end = m.start(1), m.end(1)
        before = text[max(0, start - 8):start]
        after = text[end:end + 8]
        ctx = next((c for c in BOUND_CONTEXT_ANYWHERE if c in before or c in after), None)
        if ctx is None:
            # POSITIONAL: 语境词必须紧邻数字 (中间只允许空白 / ×)
            pre = re.sub(r"^[\s×*]*", "", before)
            post = re.sub(r"^[\s×*]*", "", after)
            hit = next((c for c in BOUND_CONTEXT_POSITIONAL if pre.endswith(c)), None)
            if hit is None:
                hit = next((c for c in BOUND_POSITIONAL_AFTER
                            if c in re.match(r"^[\s×*]*[^\s]{0,4}", after).group(0)
                            and after.lstrip(" ×*").startswith(c)), None)
            if hit is not None:
                ctx = f"紧邻「{hit}」"
        dim = next((u for u in ("ms", "毫秒", "MiB", "MB", "KiB", "GB", "B", "秒")
                    if unit == u), None)
        if ctx is None and dim is None:
            continue
        out.append((num, unit, ctx or f"单位 {dim}"))
    return out


# ---------------------------------------------------------------------------
# 文件遍历
# ---------------------------------------------------------------------------
def iter_source_files(roots: Sequence[Path]) -> Iterator[Path]:
    seen: set[Path] = set()
    for root in roots:
        root = Path(root)
        if root.is_file():
            if root not in seen:
                seen.add(root)
                yield root
            continue
        if not root.exists():
            continue
        for p in sorted(root.rglob("*")):
            if p.suffix not in SOURCE_SUFFIXES:
                continue
            if any(part in SKIP_DIRS for part in p.parts):
                continue
            if p in seen:
                continue
            seen.add(p)
            yield p


def build_index(roots: Sequence[Path], files: dict[str, str]) -> SymbolIndex:
    idx = SymbolIndex()
    for path, text in files.items():
        for lineno, line in enumerate(text.splitlines(), 1):
            m = FN_RE.match(line)
            if m:
                idx.add_def(m.group(1), path, lineno)
    idx.build_counts(files)
    return idx


def analyze_texts(
    files: dict[str, str],
    index: SymbolIndex | None,
    check_call_sites: bool = True,
) -> Stats:
    stats = Stats(files=len(files))
    an = Analyzer(index, check_call_sites)
    for path, text in files.items():
        blocks = list(scan_doc_blocks(text, path))
        for blk in blocks:
            setattr(blk, "_src", text)  # 供 D1 取 enum 体
        for blk in blocks:
            an.analyze_block(blk, text, stats)
    return stats


# ---------------------------------------------------------------------------
# 报告
# ---------------------------------------------------------------------------
def report_text(stats: Stats, roots: Sequence[str], max_print: int) -> str:
    lines: list[str] = []
    counts = stats.counts()
    lines.append("=" * 78)
    lines.append("nt_docclaims — 注释断言 vs 代码事实")
    lines.append("=" * 78)
    lines.append(f"扫描: {stats.files} 文件 / {stats.doc_blocks} 文档块 / "
                 f"{stats.comment_lines} 注释行")
    lines.append(f"根: {', '.join(roots)}")
    lines.append("")
    lines.append(f"error={counts[LEVEL_ERROR]}  warn={counts[LEVEL_WARN]}  "
                 f"info={counts[LEVEL_INFO]}  skip={counts[LEVEL_SKIP]}")
    lines.append("")
    for rule, bucket in sorted(stats.per_rule.items()):
        row = "  ".join(f"{lvl}={bucket.get(lvl, 0)}" for lvl in LEVELS)
        lines.append(f"  [{rule}] {row}")
    lines.append("")

    for level in (LEVEL_ERROR, LEVEL_WARN, LEVEL_INFO, LEVEL_SKIP):
        group = [f for f in stats.findings if f.level == level]
        if not group:
            continue
        lines.append("-" * 78)
        lines.append(f"### {level.upper()} ({len(group)})")
        lines.append("-" * 78)
        if level == LEVEL_SKIP:
            lines.append(f"(skip 仅计数不逐条打印, 共 {len(group)} 条 —— "
                         f"可用 --show-skip 查看)")
            lines.append("")
            continue
        shown = group[:max_print]
        for f in shown:
            lines.append(f.render())
            lines.append("")
        if len(group) > len(shown):
            lines.append(f"... 另有 {len(group) - len(shown)} 条 {level} 未打印 "
                         f"(调 --max-print 查看更多)")
            lines.append("")
    return "\n".join(lines)


# ===========================================================================
# 自测: 内联样本 + 变异注入 + 反误报断言
# ===========================================================================
SAMPLE_ENUM_OK = '''\
/// 回合终态协议（五态：done/continue/needs_clarification/blocked/waiting）.
#[derive(Debug, Clone, Copy)]
pub enum TurnStatus {
    Done,
    Continue,
    NeedsClarification,
    Blocked,
    Waiting,
}
'''

SAMPLE_ENUM_LYING = '''\
/// 回合终态协议（五态：done/continue/needs_clarification/failed/cancelled）.
#[derive(Debug, Clone, Copy)]
pub enum TurnStatus {
    Done,
    Continue,
    NeedsClarification,
    Blocked,
    Waiting,
}
'''

SAMPLE_CONST_OK = '''\
/// 全局搜索命中数上限（硬上限 200）.
pub const MAX_SEARCH_HITS: usize = 200;
'''

SAMPLE_CONST_LYING = '''\
/// 全局搜索命中数上限（硬上限 5000）.
pub const MAX_SEARCH_HITS: usize = 200;
'''

# 第 6 例: 已如实登记的缺口 —— 必须零 error
SAMPLE_DECLARED_GAP_CAP = '''\
/// 全局搜索总访问目录项上限（防大仓空转）。
///
/// 不能写成「总访问上限」：那会让人以为撞到它之后搜索会停下来。
pub const MAX_SEARCH_VISITS: usize = 20_000;
'''

# 第 7 例: 已如实登记的缺口 —— 必须零 error
SAMPLE_DECLARED_GAP_SYMLINK = '''
//! 所有 pathspec 只过 `nt_workspace::check_rel`（**纯词法**判定）。
//! 注意 pathspec 与侧边栏那套**不同源**：侧边栏的 `read_text` 走
//! `nt_workspace::resolve_within`，本模块够不着它（`resolve_within` 是
//! `nt_workspace` 私有的）。不能写成「都过 `jail_join`」：那会让人以为 git
//! 这条路也带软链接防护，而补这道门得改代码，不是改注释能了事的。
pub fn stage() {}
'''

# 第 1 例的修正版: 注释引着旧错的那句 —— 必须零 error
SAMPLE_QUOTED_HISTORY = '''\
/// 回合终态协议（五态：done/continue/needs_clarification/blocked/waiting）.
///
/// **没有 `failed`，更没有 `cancelled`** —— 早先这行注释写着
/// 「done/continue/waiting/failed/cancelled」，两个都不存在。
#[derive(Debug, Clone)]
pub enum TurnStatus {
    Done,
    Continue,
    NeedsClarification,
    Blocked,
    Waiting,
}
'''

SAMPLE_ABSOLUTE = '''\
/// 适配器一律发新消息，绝不做 `editMessage`。
pub fn send() {}
'''

SAMPLE_CALLSITE_OK = '''\
/// `drain_outbox_once` 已被接线，跑轮每轮调用一次。
pub fn drain_outbox_once() {}
fn caller() { drain_outbox_once(); }
'''

SAMPLE_CALLSITE_ZERO = '''\
/// `ghost_helper` 已接线，每轮调用一次。
pub fn ghost_helper() {}
fn unrelated() { let _ = 1; }
'''

# --- 反误报样本: 以下每条在真实仓库里都被第一版误报过, 必须被钉死为「不报 error」--

# 散文里的 "clean run / broken run" —— 不是变体清单 (第一版误报)
SAMPLE_PROSE_LIST = '''\
/// 黄金轨迹标签 — 来自真实日志 (clean run / broken run)。
pub enum TrajectoryLabel {
    Clean,
    Broken,
}
'''

# 计数词主语是别的类型 —— 说的不是本 enum (第一版误报)
SAMPLE_COUNT_OTHER_TYPE = '''\
/// 注 (D6 对齐): SpecialistType 共 15 种, 而 resonance 的数组是 14 个。
pub enum CognitiveType {
    Linguistic,
    Logical,
    Knowledge,
    Social,
}
'''

# 数字是标识符的一部分: `E8 状态` / `D-6` / `缺陷 #5` (第一版误报)
SAMPLE_IDENT_NUMBER = '''\
/// 文件能力操作 — 每类操作驱动一次 E8 状态转移 (Ext-6, 缺陷 #5)。
pub const CAP: usize = 6;
'''

# 量纲不可比: 1 KiB vs 1024 个条目 —— 代码是对的 (第一版误报)
SAMPLE_UNIT_MISMATCH = '''\
/// 最小可报告大小 (1 KiB)
pub const MIN_SIZE: usize = 1024;
'''

# 表达式/轶事: 2 × 3 = 5, 与 const 一致 (第一版误报)
SAMPLE_EXPR = '''\
/// 候选上限 2 × 重试上限 3
pub const MAX_NARRATION_CALLS: usize = 5;
'''

# 块内自洽: 常量旁注记着「由 5 降为 3」并引用 cap=5 的实测 —— 整块自洽 (第一版误报)
SAMPLE_BLOCK_CONSISTENT = '''\
/// 单轮前提数上界。
///
/// 2026-09-28 由 5 降为 **3**。即 cap=5 时**近一半前提是凑数的**。
pub const DEFAULT_PER_ROUND_CAP: usize = 3;
'''

# 工单号 `P0-6` 的 6 —— 第一版把它当成了界断言 (第二版也漏了, 因为 `0` 不是字母)
SAMPLE_TICKET_NUM = '''\
/// P0-6 注入预算封顶: build_context 检索上下文拼进 prompt 前的 token 估算上限。
pub const MAX_KB_INJECTION_TOKENS: usize = 512;
'''

# 定义式断言 `SYM = N` 必须仍然能翻红 (第二版把它误当算式吞掉 → 真实漏报回归)
SAMPLE_DEFINITIONAL_EQ = '''\
/// 单轮前提数上界（默认 `DEFAULT_PER_ROUND_CAP` = 5）。传 0 → 选不出前提。
pub fn with_per_round_cap(k: usize) {}
pub const DEFAULT_PER_ROUND_CAP: usize = 3;
'''


def _count(text: str, level: str, rule: str | None = None) -> int:
    stats = analyze_texts({"<selftest>": text}, None, check_call_sites=False)
    return sum(
        1 for f in stats.findings
        if f.level == level and (rule is None or f.rule == rule)
    )


def _count_callsite(text: str, level: str) -> int:
    files = {"<selftest>": text}
    idx = build_index([], files)
    stats = analyze_texts(files, idx, check_call_sites=True)
    return sum(1 for f in stats.findings if f.level == level and f.rule == "call_site")


def selftest() -> int:
    results: list[tuple[str, bool, str]] = []

    def case(name: str, ok: bool, detail: str = "") -> None:
        results.append((name, ok, detail))

    # --- D1 枚举断言: 绿 / 红 ---
    case("D1 绿 (注释与变体一致)", _count(SAMPLE_ENUM_OK, LEVEL_ERROR) == 0)
    case("D1 红 (注释多列 failed/cancelled)",
         _count(SAMPLE_ENUM_LYING, LEVEL_ERROR, "enum_variants") == 1)

    # --- D1 反误报: 引用段不是断言 ---
    case("D1 反误报 (「」引着的旧错不算断言)",
         _count(SAMPLE_QUOTED_HISTORY, LEVEL_ERROR) == 0)

    # --- D2 数字断言: 绿 / 红 ---
    case("D2 绿 (注释 200 == const 200)", _count(SAMPLE_CONST_OK, LEVEL_ERROR) == 0)
    case("D2 红 (注释 5000 != const 200)",
         _count(SAMPLE_CONST_LYING, LEVEL_ERROR, "numeric_bound") == 1)

    # --- 已承认缺口必须零 error (第 6、7 例) ---
    case("X 绿 (第6例 已登记缺口 不报 error)",
         _count(SAMPLE_DECLARED_GAP_CAP, LEVEL_ERROR) == 0)
    case("X 绿 (第7例 已登记缺口 不报 error)",
         _count(SAMPLE_DECLARED_GAP_SYMLINK, LEVEL_ERROR) == 0)
    case("X 绿 (第6/7例 被豁免为 skip)",
         _count(SAMPLE_DECLARED_GAP_CAP, LEVEL_SKIP, "declared_gap") > 0
         and _count(SAMPLE_DECLARED_GAP_SYMLINK, LEVEL_SKIP, "declared_gap") > 0)

    # --- D4 绝对量词: 只 info, 绝不 error ---
    case("D4 绝对量词 → info 不判错",
         _count(SAMPLE_ABSOLUTE, LEVEL_INFO, "absolute_qualifier") >= 1
         and _count(SAMPLE_ABSOLUTE, LEVEL_ERROR) == 0)

    # --- D3 调用点: 绿 / 红 ---
    case("D3 绿 (有真实调用点)", _count_callsite(SAMPLE_CALLSITE_OK, LEVEL_WARN) == 0)
    case("D3 红 (声称已接线但零调用点)",
         _count_callsite(SAMPLE_CALLSITE_ZERO, LEVEL_WARN, ) == 1)

    # --- 反误报钉死: 这 6 类在真实仓库里都被第一版误报过, 必须零 error ---
    no_fp = [
        ("散文 slash 短语 (clean run / broken run)", SAMPLE_PROSE_LIST),
        ("计数词主语是别的类型 (SpecialistType 共 15 种)", SAMPLE_COUNT_OTHER_TYPE),
        ("数字是标识符一部分 (E8 状态 / D-6 / 缺陷 #5)", SAMPLE_IDENT_NUMBER),
        ("量纲不可比 (1 KiB vs 1024 条目)", SAMPLE_UNIT_MISMATCH),
        ("表达式 2 × 3 = 5 (与 const 自洽)", SAMPLE_EXPR),
        ("块内自洽 (由 5 降为 3 + 引用 cap=5 实测)", SAMPLE_BLOCK_CONSISTENT),
        ("工单号 P0-6 的 6 (数字是标识符一部分)", SAMPLE_TICKET_NUM),
    ]
    for name, src in no_fp:
        got = _count(src, LEVEL_ERROR)
        case(f"反误报 {name} — 零 error", got == 0, f"got {got}")

    # --- 真实漏报回归: 定义式 `SYM = N` 必须仍然翻红 ---
    got_def = _count(SAMPLE_DEFINITIONAL_EQ, LEVEL_ERROR)
    case("回归 定义式断言 `SYM = 5` vs const 3 → 必须报 error",
         got_def == 1, f"got {got_def}")

    # -------------------------------------------------------------------
    # 变异注入: 改坏一侧必须翻红, 改回必须变绿
    # -------------------------------------------------------------------
    def mutate(src: str, old: str, new: str) -> str:
        if old not in src:
            raise AssertionError(f"变异锚点不存在: {old!r}")
        return src.replace(old, new, 1)

    mutations = [
        ("MUT-D1 改坏注释侧 (删掉注释里的 blocked)",
         SAMPLE_ENUM_OK, _count(SAMPLE_ENUM_OK, LEVEL_ERROR), 0,
         lambda s: mutate(s, "needs_clarification/blocked/waiting",
                          "needs_clarification/waiting"), 1),
        ("MUT-D1 改坏代码侧 (把 Blocked 变体改名)",
         SAMPLE_ENUM_OK, _count(SAMPLE_ENUM_OK, LEVEL_ERROR), 0,
         lambda s: mutate(s, "    Blocked,", "    Stuck,"), 1),
        ("MUT-D1 改坏计数词 (五态 → 七态)",
         SAMPLE_ENUM_OK, _count(SAMPLE_ENUM_OK, LEVEL_ERROR), 0,
         lambda s: mutate(s, "五态", "七态"), 1),
        ("MUT-D2 改坏注释侧 (200 → 4000)",
         SAMPLE_CONST_OK, _count(SAMPLE_CONST_OK, LEVEL_ERROR), 0,
         lambda s: mutate(s, "上限 200", "上限 4000"), 1),
        ("MUT-D2 改坏代码侧 (const 200 → 50)",
         SAMPLE_CONST_OK, _count(SAMPLE_CONST_OK, LEVEL_ERROR), 0,
         lambda s: mutate(s, "= 200;", "= 50;"), 1),
        ("MUT-D3 改坏代码侧 (删掉唯一调用点)",
         SAMPLE_CALLSITE_OK, _count_callsite(SAMPLE_CALLSITE_OK, LEVEL_WARN), 0,
         lambda s: mutate(s, "fn caller() { drain_outbox_once(); }", "fn caller() {}"), 1),
    ]
    for name, src, base, base_expect, fn, mutated_expect in mutations:
        got_base = base
        case(f"{name} — 原样本为绿", got_base == base_expect, f"got {got_base}")
        mutated = fn(src)
        got_mut = (_count_callsite(mutated, LEVEL_WARN)
                   if name.startswith("MUT-D3")
                   else _count(mutated, LEVEL_ERROR))
        case(f"{name} — 变异后翻红", got_mut == mutated_expect, f"got {got_mut}")
        case(f"{name} — 改回复绿", got_base == base_expect, f"got {got_base}")

    passed = sum(1 for _, ok, _ in results if ok)
    total = len(results)
    for name, ok, detail in results:
        mark = "PASS" if ok else "FAIL"
        extra = f"  ({detail})" if detail and not ok else ""
        print(f"[{mark}] {name}{extra}")
    print()
    print(f"[selftest] {passed}/{total} 通过")
    return 0 if passed == total else 1


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------
def main(argv: Sequence[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        prog="nt_docclaims.py",
        description="注释里的可检验断言 vs 代码事实 —— 矛盾检测器 (只读)",
    )
    ap.add_argument("paths", nargs="*", help="要扫描的目录/文件 (默认 neotrix-core/src crates/*/src)")
    ap.add_argument("--json", action="store_true", help="输出 JSON")
    ap.add_argument("--self-test", action="store_true", help="跑内联样本自测 (不读仓库)")
    ap.add_argument("--level", choices=LEVELS, action="append",
                    help="只显示该级别 (可重复)")
    ap.add_argument("--rule", action="append", help="只显示该检测器 (可重复)")
    ap.add_argument("--max-print", type=int, default=60, help="每级最多打印多少条 (默认 60)")
    ap.add_argument("--show-skip", action="store_true", help="逐条打印 skip")
    ap.add_argument("--no-call-sites", action="store_true", help="跳过 D3 (不建符号索引, 更快)")
    args = ap.parse_args(argv)

    if args.self_test:
        return selftest()

    roots = args.paths or [str(p) for p in
                           [Path("neotrix-core/src")] + list(Path("crates").glob("*/src"))]
    files: dict[str, str] = {}
    read_errors: list[str] = []
    for p in iter_source_files([Path(r) for r in roots]):
        try:
            files[str(p)] = p.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            read_errors.append(f"{p}: {exc}")
    if not files:
        print("[nt_docclaims] 没有扫到任何源文件 (检查路径参数)", file=sys.stderr)
        for e in read_errors:
            print(f"  {e}", file=sys.stderr)
        return 2

    index = None if args.no_call_sites else build_index([Path(r) for r in roots], files)
    stats = analyze_texts(files, index, check_call_sites=not args.no_call_sites)

    if args.level or args.rule:
        stats.findings = [
            f for f in stats.findings
            if (not args.level or f.level in args.level)
            and (not args.rule or f.rule in args.rule)
        ]

    counts = stats.counts()
    if args.json:
        payload = {
            "tool": "nt_docclaims",
            "version": VERSION,
            "roots": list(roots),
            "files_scanned": stats.files,
            "doc_blocks": stats.doc_blocks,
            "comment_lines": stats.comment_lines,
            "counts": counts,
            "per_rule": stats.per_rule,
            "read_errors": read_errors,
            "findings": [asdict(f) for f in stats.findings],
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
    else:
        print(report_text(stats, list(roots), args.max_print))
        if args.show_skip:
            for f in stats.findings:
                if f.level == LEVEL_SKIP:
                    print(f.render())
                    print("")

    # 退出码: 有 error → 1; 无 error → 0 (info/skip 不阻塞, 否则工具会被自己淹死)
    return 1 if counts[LEVEL_ERROR] else 0


if __name__ == "__main__":
    sys.exit(main())
