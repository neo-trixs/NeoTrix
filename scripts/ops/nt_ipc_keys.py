#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_ipc_keys.py — 前后端 IPC 键名静态核对器（2026-09-28 立项）.

## 它补的是哪个洞

`scripts/ops/nt_smoke.sh` 的 38 条 Rust 冒烟验的是**参数绑定层**（直接调 Rust
函数，键名由调用方自己写死）。真实前端走的是
`invoke("neobot_xxx", { someKey: value })` —— Tauri 把 JSON 对象的**键**映射到
Rust 函数的**形参名**（默认 `rename_all = "camelCase"`，即 `task_id` ↔ `taskId`）。

这条跨语言接缝的特征是：**键名对不上时编译期无感知、运行期静默失败**
（多余的键被直接忽略；该来却缺的 `Option<T>` 变 `None`；非 `Option` 缺键才报运行期错）。
`nt_smoke.sh` 头注释里登记的「已知缺口」就是这一条 —— 本脚本把它变成可随时跑的检查。

## 口径

- **前端**：`apps/neobot-desktop/frontend/src/**/*.ts` 里的 `invoke(<字面量>, {…})`。
  命令名不是字面量 / 第二个参数不是对象字面量 / 载荷里有展开或计算键
  → 记为「无法静态判定」，进人工清单，**不猜**。
- **Rust 侧**：`apps/neobot-desktop/src/nt_commands.rs` + `src/nt_commands/**/*.rs`
  里 `#[tauri::command]` 函数的形参名。
  `State<…>` / `AppHandle` / `Window` / `WebviewWindow` / `Webview` 是**托管参数**
  （Tauri 内部注入），不来自前端载荷，故不参与键名比对。
  `Channel<T>` 是**普通参数**（前端要传），参与比对。
- **注册表**：`src/main.rs` 的 `generate_handler!` 与声明集互相校验
  （漏注册 / 拼错 / 重复），与 `nt_commands.rs::registration_tests` 同口径、不重叠。

判为 **error**（退出码非 0）的是：键名不匹配、前端调了不存在的命令、声明/注册差集。
「可选形参没传」只报 warning（那是合法写法），「无前端调用点的命令」只报 info。

## 它**不**验什么（别拿「脚本绿了」当「IPC 没问题」的证据）

- **不验类型**：`{ taskId: 123 }` 打给 `task_id: String` 静态看不出，运行期才炸。
- **不验 `Channel` 载荷**：流式事件的对错要看真运行时。
- **不验命令名变量**：`invoke(cmd, …)`（`cmd` 由三元决定）判不了，进人工清单。
- **不验 `..` 展开 / 计算键的载荷**：只报多余键，不报缺键（会假绿，故单列出来）。
- **不验运行期注册**：只读 `generate_handler!` 源码文本，不启动 Tauri。

## 退出码

    0 = 无 error（可能有 warning / info）
    1 = 有 error
    2 = 环境不对（目录缺失）或解析器自检报警（report["counts"]["suspect"]）

## 为什么带 --self-test

静态扫描器最危险的失败模式是**假阴**（规则太松，漏报真错配）和
**假阳**（规则太紧，把正确代码判成错配）。两者都不会让脚本崩，只会安静地
给一份错误的绿。所以自测用内联小样本把两侧都喂进去，断言**确切的条数** ——
变异测试（删一条规则、改一个期望）必须立刻红。

「绿」的证据不止自测：本脚本在真仓库上的结论（97 命令 0 错配）另用一个
**纯正则的独立实现**（不复用本文件任何代码）交叉验证过，(命令,键) 组合集合
双向完全一致；又在真仓库副本上注入 8 类人为错配，全部被抓且给出精确行号。

纯标准库，零构建可跑（本机内存门 BLOCKED 时的唯一可用姿势）。

## 用法

    python3 scripts/ops/nt_ipc_keys.py                # 人读报告
    python3 scripts/ops/nt_ipc_keys.py --json         # 机器读
    python3 scripts/ops/nt_ipc_keys.py --self-test    # 内联样本自测（不碰仓库文件）
    python3 scripts/ops/nt_ipc_keys.py --root /path/to/repo
"""

from __future__ import annotations

import argparse
import difflib
import json
import os
import re
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Sequence, Set, Tuple

# ─── 仓库布局（可被 --root 覆盖）────────────────────────────────────────────

REL_FRONTEND = os.path.join("apps", "neobot-desktop", "frontend", "src")
REL_CMDS = os.path.join("apps", "neobot-desktop", "src", "nt_commands")
REL_CMDS_MOD = os.path.join("apps", "neobot-desktop", "src", "nt_commands.rs")
REL_MAIN = os.path.join("apps", "neobot-desktop", "src", "main.rs")

# Tauri 内部注入、前端**不传**的形参类型（末段名）。
MANAGED_ARG_TYPES = frozenset(
    {"State", "AppHandle", "Window", "WebviewWindow", "Webview", "RunEvent", "Manager"}
)

IDENT_RE = re.compile(r"[A-Za-z_$][A-Za-z0-9_$]*")
# 上一个有效字符是这些时，后面的 `/` 是正则字面量而不是除号。
REGEX_PREV_CHARS = frozenset("(,=:[!&|?{};+-*%~^<>")
REGEX_PREV_WORDS = frozenset(
    """return typeof instanceof in of new delete void do else case yield await throw""".split()
)


# ══════════════════════════════════════════════════════════════════════════
# 1. TS 侧：把源码切成「代码 / 非代码」掩码，再在代码区里找 invoke(…)
# ══════════════════════════════════════════════════════════════════════════


def _skip_line_comment(ts: str, i: int) -> int:
    j = ts.find("\n", i)
    return len(ts) if j < 0 else j


def _skip_block_comment(ts: str, i: int) -> int:
    j = ts.find("*/", i + 2)
    return len(ts) if j < 0 else j + 2


def _skip_quoted(ts: str, i: int) -> int:
    """`"`/`'` 起的普通字符串（i 指向开引号）。返回闭引号之后的下标。"""
    q = ts[i]
    i += 1
    n = len(ts)
    while i < n:
        c = ts[i]
        if c == "\\":
            i += 2
            continue
        if c == q:
            return i + 1
        if c == "\n":  # 未闭合（TS 不允许裸换行）——保守停在行尾
            return i
        i += 1
    return n


def _skip_template(ts: str, i: int) -> int:
    """模板字面量（含 `${…}` 里的嵌套代码与其内层字符串/模板）。"""
    n = len(ts)
    i += 1  # 开反引号
    while i < n:
        c = ts[i]
        if c == "\\":
            i += 2
            continue
        if c == "`":
            return i + 1
        if c == "$" and i + 1 < n and ts[i + 1] == "{":
            i = _skip_braced_code(ts, i + 1)
            continue
        i += 1
    return n


def _skip_braced_code(ts: str, i: int) -> int:
    """`{` … 配平 `}`，期间跳过注释/字符串/模板/正则。"""
    n = len(ts)
    depth = 0
    while i < n:
        c = ts[i]
        if c == "/" and i + 1 < n and ts[i + 1] == "/":
            i = _skip_line_comment(ts, i)
            continue
        if c == "/" and i + 1 < n and ts[i + 1] == "*":
            i = _skip_block_comment(ts, i)
            continue
        if c in "\"'":
            i = _skip_quoted(ts, i)
            continue
        if c == "`":
            i = _skip_template(ts, i)
            continue
        if c == "/" and _prev_is_expr_start(ts, i, None):
            i = _skip_regex(ts, i)
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def _skip_regex(ts: str, i: int) -> int:
    """i 指向正则开头的 `/`。跳过字面量与尾随 flag。"""
    n = len(ts)
    i += 1
    in_class = False
    while i < n:
        c = ts[i]
        if c == "\\":
            i += 2
            continue
        if c == "\n":
            return i  # 不是正则，保守放弃
        if c == "[":
            in_class = True
        elif c == "]":
            in_class = False
        elif c == "/" and not in_class:
            i += 1
            while i < n and ts[i].isalpha():
                i += 1
            return i
        i += 1
    return n


def _skip_trivia(ts: str, mask: bytearray, i: int) -> int:
    """只跳空白与注释（**遇字符串就停**：引号键/命令名要靠 spans 取内容）。"""
    n = len(ts)
    while i < n:
        c = ts[i]
        if c in " \t\r\n":
            i += 1
        elif not mask[i] and ts.startswith("//", i):
            i = _skip_line_comment(ts, i)
        elif not mask[i] and ts.startswith("/*", i):
            i = _skip_block_comment(ts, i)
        else:
            break
    return i


def _trim_span(ts: str, mask: bytearray, lo: int, hi: int) -> Tuple[int, int]:
    """掐掉一段源码两端的空白与注释（**不碰字符串**）。"""
    a = _skip_trivia(ts, mask, lo)
    b = hi
    while b > a:
        c = ts[b - 1]
        if c in " \t\r\n":
            b -= 1
            continue
        if not mask[b - 1]:
            if b >= 2 and ts.startswith("*/", b - 2):
                k = ts.rfind("/*", 0, b - 2)
                b = k if k >= 0 else a
                continue
            if b >= 2 and ts.startswith("//", b - 2):
                k = ts.rfind("\n", 0, b - 2)
                b = (k + 1) if k >= 0 else a
                continue
        break
    return a, b


def _prev_is_expr_start(ts: str, i: int, mask: Optional[bytearray]) -> bool:
    j = i - 1
    if j < 0:
        return True
    if mask is not None:
        while j >= 0 and not mask[j]:
            j -= 1
        if j < 0:
            return True
    else:
        while j >= 0 and ts[j] in " \t\r\n":
            j -= 1
        if j < 0:
            return True
    ch = ts[j]
    if ch in REGEX_PREV_CHARS:
        return True
    if IDENT_RE.match(ch):
        k = j
        while k >= 0 and IDENT_RE.match(ts[k]):
            k -= 1
        word = ts[k + 1 : j + 1]
        if word in REGEX_PREV_WORDS:
            return True
    return False


def ts_mask(ts: str) -> bytearray:
    """1 = 该字符是代码（不在注释/字符串/模板/正则里）。"""
    n = len(ts)
    mask = bytearray(n)
    i = 0
    while i < n:
        c = ts[i]
        if c == "/" and i + 1 < n and ts[i + 1] == "/":
            i = _skip_line_comment(ts, i)
            continue
        if c == "/" and i + 1 < n and ts[i + 1] == "*":
            i = _skip_block_comment(ts, i)
            continue
        if c in "\"'":
            i = _skip_quoted(ts, i)
            continue
        if c == "`":
            i = _skip_template(ts, i)
            continue
        if c == "/" and _prev_is_expr_start(ts, i, mask):
            i = _skip_regex(ts, i)
            continue
        mask[i] = 1
        i += 1
    return mask


def _string_spans(ts: str, mask: bytearray) -> Dict[int, str]:
    """开引号下标 → 字符串内容（用于取 invoke 的命令名 / 对象字面量的引号键）。"""
    spans: Dict[int, str] = {}
    n = len(ts)
    i = 0
    while i < n:
        c = ts[i]
        if mask[i]:
            i += 1
            continue
        if c in "\"'":
            j = _skip_quoted(ts, i)
            raw = ts[i + 1 : max(i + 1, j - 1)]
            if raw.endswith('"') or raw.endswith("'"):
                raw = raw[:-1]
            spans[i] = raw
            i = j
            continue
        if c == "/" and i + 1 < n and ts[i + 1] in "/*":
            i = _skip_line_comment(ts, i) if ts[i + 1] == "/" else _skip_block_comment(ts, i)
            continue
        if c == "`":
            i = _skip_template(ts, i)
            continue
        if c == "/" and _prev_is_expr_start(ts, i, mask):
            i = _skip_regex(ts, i)
            continue
        i += 1
    return spans


def _match_paren(ts: str, mask: bytearray, open_at: int) -> int:
    """`(` 的配平 `}` —— 返回闭括号之后的下标（开括号须在代码区）。"""
    depth = 0
    i = open_at
    n = len(ts)
    while i < n:
        if mask[i]:
            c = ts[i]
            if c == "(":
                depth += 1
            elif c == ")":
                depth -= 1
                if depth == 0:
                    return i + 1
        i += 1
    return n


def _split_top_level(ts: str, mask: bytearray, lo: int, hi: int) -> List[Tuple[int, int]]:
    """把 `[lo, hi)` 按深度 0 的 `,` 切成若干段（返回各段下标区间）。"""
    out: List[Tuple[int, int]] = []
    depth = 0
    start = lo
    i = lo
    while i < hi:
        if mask[i]:
            c = ts[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
            elif c == "," and depth == 0:
                out.append((start, i))
                start = i + 1
        i += 1
    if ts[start:hi].strip() or not out:
        out.append((start, hi))
    return out


def _skip_generic_args(ts: str, mask: bytearray, i: int) -> int:
    """`invoke<T>` 里的 `<` … `>`（跳过字符串/模板/`=>`）。返回 `>` 之后下标。"""
    n = len(ts)
    depth = 0
    while i < n:
        c = ts[i]
        if c == "/" and i + 1 < n and ts[i + 1] == "/":
            i = _skip_line_comment(ts, i)
            continue
        if c == "/" and i + 1 < n and ts[i + 1] == "*":
            i = _skip_block_comment(ts, i)
            continue
        if c in "\"'":
            i = _skip_quoted(ts, i)
            continue
        if c == "`":
            i = _skip_template(ts, i)
            continue
        if c == "<":
            depth += 1
        elif c == ">":
            if ts[i - 1 : i] == "=":  # `=>` 不是闭尖括号
                i += 1
                continue
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


@dataclass
class InvokeCall:
    file: str
    line: int
    cmd: Optional[str]
    keys: List[str] = field(default_factory=list)
    dynamic: bool = False  # 载荷里有展开/计算键 → 键集不可知
    reason: str = ""  # 无法静态判定的原因


def _object_keys(
    ts: str, mask: bytearray, spans: Dict[int, str], lo: int, hi: int
) -> Tuple[List[str], bool, str]:
    """对象字面量 `[lo, hi)` 的顶层键。返回 (键, 是否动态, 说明)。"""
    keys: List[str] = []
    dynamic = False
    i = lo
    n = hi  # hi = 对象闭括号 `}` 的下标（不是内容上界）
    while i < n:
        if not mask[i]:
            # 引号键 / 模板键 / 注释 —— 都不能按「空白」处理掉。
            if ts.startswith("//", i) or ts.startswith("/*", i):
                i = _skip_line_comment(ts, i) if ts[i + 1] == "/" else _skip_block_comment(ts, i)
                continue
            if i in spans:
                keys.append(spans[i])
                i = _skip_quoted(ts, i)
            else:
                dynamic = True  # 模板/多行字符串键：键集不可知
                i = _skip_template(ts, i) if ts[i] == "`" else _skip_quoted(ts, i)
            continue
        c = ts[i]
        if c in " \t\r\n":
            i += 1
            continue
        if c == ",":
            i += 1
            continue
        if ts.startswith("...", i):
            dynamic = True
            i = _skip_value(ts, mask, spans, i + 3, n)
            continue
        if c == "[":
            dynamic = True  # 计算键
            i = _skip_value(ts, mask, spans, i, n)
            continue
        m = IDENT_RE.match(ts, i)
        if not m:
            dynamic = True
            i += 1
            continue
        word = m.group(0)
        i = _skip_trivia(ts, mask, m.end())
        if i < n and ts[i] == ":":
            keys.append(word)
            i = _skip_value(ts, mask, spans, i + 1, n)
        elif i < n and ts[i] == "(":  # 方法简写：不是载荷键
            i = _match_paren(ts, mask, i)
        else:  # 简写属性 `{ name }`
            keys.append(word)
        if i <= lo:  # 防御：不前进就跳一格
            i = lo + 1
    return keys, dynamic, ""


def _skip_value(ts: str, mask: bytearray, spans: Dict[int, str], lo: int, hi: int) -> int:
    """跳过一个属性值，返回其后的下标（深度 0 的 `,` 或对象闭括号处）。"""
    depth = 0
    i = lo
    while i < hi:
        if not mask[i]:
            i += 1
            continue
        c = ts[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                return i
            depth -= 1
        elif c == "," and depth == 0:
            return i
        i += 1
    return hi


#: 允许当作「IPC 调用点」的函数名。
#:
#: 早先只认裸 `invoke`。后来前端把 97 个调用点收进了 typed wrapper
#: （`invoke.ts` 的 `ntInvoke` / `ntInvokeStream`，把命令名与参数形状钉在类型里），
#: 本工具于是**一个调用点都认不出**，却照样打印「合计 error 0 ⇒ OK」——
#: 一次**静默失去全部覆盖**的回归：安全网还在跑，但已经什么都看不见了。
#:
#: 所以这里是显式白名单，且配套 `parse_health` 里有 fail-closed 兜底
#: （解析到 0 个调用点即报错退出）。白名单漏了新 wrapper 时会**响**，
#: 而不是安静地变瞎 —— 这正是那次事故缺的那一环。
INVOKE_FN_NAMES = ("invoke", "ntInvoke", "ntInvokeStream")


def _ident_bounds(ts: str, s: int, e: int) -> Tuple[int, int]:
    """把 `invoke` 匹配向两侧扩成完整标识符（`ntInvoke` → `ntInvoke` 整段）。"""
    i = s
    while i > 0 and (ts[i - 1].isalnum() or ts[i - 1] in "_$"):
        i -= 1
    j = e
    while j < len(ts) and (ts[j].isalnum() or ts[j] in "_$"):
        j += 1
    return i, j


def ts_invoke_calls(ts: str, rel_path: str) -> List[InvokeCall]:
    mask = ts_mask(ts)
    spans = _string_spans(ts, mask)
    calls: List[InvokeCall] = []
    # 大小写不敏感：`ntInvoke` 里是**大写 I**，用 `r"invoke"` 根本匹配不到 ——
    # 这才是「wrapper 化之后一个调用点都抓不到」的真正原因（白名单是对的，
    # 但白名单永远没被问过）。命中后由 INVOKE_FN_NAMES 精确过滤。
    for m in re.finditer(r"invoke", ts, re.IGNORECASE):
        s, e = m.span()
        if not all(mask[s:e]):
            continue  # 注释 / 字符串 / 正则里的 "invoke"
        name_s, name_e = _ident_bounds(ts, s, e)
        name = ts[name_s:name_e]
        if name not in INVOKE_FN_NAMES:
            continue  # `invokeX` / 别人的 `xInvoke` / 恰好含 invoke 的标识符
        s, e = name_s, name_e
        before = ts[s - 1] if s > 0 else ""
        if before == ".":
            continue  # 别人的 obj.invoke(...)
        i = _skip_trivia(ts, mask, e)
        if i < len(ts) and ts[i] == "<":  # invoke<T>(...) 的类型实参
            i = _skip_trivia(ts, mask, _skip_generic_args(ts, mask, i))
        if i >= len(ts) or ts[i] != "(":
            continue  # 只是提到这个词（import / 类型标注）
        close = _match_paren(ts, mask, i)
        args = _split_top_level(ts, mask, i + 1, close - 1)
        line = ts.count("\n", 0, s) + 1
        calls.append(_parse_invoke(ts, mask, spans, rel_path, line, args))
    return calls


def _parse_invoke(
    ts: str, mask: bytearray, spans: Dict[int, str], rel_path: str, line: int, args
) -> InvokeCall:
    def trim(a: int, b: int) -> Tuple[int, int]:
        return _trim_span(ts, mask, a, b)

    if not args:
        return InvokeCall(rel_path, line, None, reason="invoke() 没有参数")
    a0, b0 = trim(*args[0])
    if a0 < b0 and ts[a0] == "<":
        a0 = _skip_generic_args(ts, mask, a0)
        a0, b0 = trim(a0, args[0][1])
    if a0 >= b0:
        return InvokeCall(rel_path, line, None, reason="命令名参数为空/非字面量")
    if a0 in spans and spans[a0] and b0 == a0 + len(spans[a0]) + 2 and ts[b0 - 1] in "\"'":
        cmd: Optional[str] = spans[a0]
    else:
        return InvokeCall(
            rel_path, line, None, reason="命令名不是字符串字面量: %s" % ts[a0:b0][:40].replace("\n", " ")
        )
    if len(args) < 2:
        return InvokeCall(rel_path, line, cmd)
    a1, b1 = trim(*args[1])
    if a1 >= b1:
        return InvokeCall(rel_path, line, cmd, reason="第二个参数为空")
    if ts[a1] != "{":
        return InvokeCall(
            rel_path,
            line,
            cmd,
            reason="第二个参数不是对象字面量: %s" % ts[a1:b1][:40].replace("\n", " "),
        )
    obj_close = _match_brace(ts, mask, a1)
    keys, dynamic, _ = _object_keys(ts, mask, spans, a1 + 1, obj_close - 1)
    reason = ""
    if len(args) > 2:
        reason = "第三个及以后的实参: %s" % ts[args[2][0] : args[2][1]][:30].replace("\n", " ")
    return InvokeCall(rel_path, line, cmd, keys, dynamic, reason)


def _match_brace(ts: str, mask: bytearray, open_at: int) -> int:
    depth = 0
    i = open_at
    n = len(ts)
    while i < n:
        if mask[i]:
            c = ts[i]
            if c in "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    return i + 1
        i += 1
    return n


# ══════════════════════════════════════════════════════════════════════════
# 2. Rust 侧：`#[tauri::command]` 函数 + `generate_handler!` 表
# ══════════════════════════════════════════════════════════════════════════


def rs_mask(rs: str) -> bytearray:
    """Rust 版掩码：额外处理生命周期 `'a`（不是字符字面量）与 `r#"…"#` 原始字符串。"""
    n = len(rs)
    mask = bytearray(n)
    i = 0
    while i < n:
        c = rs[i]
        if c == "/" and i + 1 < n and rs[i + 1] == "/":
            j = rs.find("\n", i)
            i = n if j < 0 else j
            continue
        if c == "/" and i + 1 < n and rs[i + 1] == "*":
            depth = 1
            i += 2
            while i < n and depth:
                if rs.startswith("/*", i):
                    depth += 1
                    i += 2
                elif rs.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            continue
        if c == "r" and i + 1 < n and (rs[i + 1] == '"' or rs[i + 1] == "#"):
            k = i + 1
            hashes = 0
            while k < n and rs[k] == "#":
                hashes += 1
                k += 1
            if k < n and rs[k] == '"':
                close = '"' + "#" * hashes
                j = rs.find(close, k + 1)
                i = n if j < 0 else j + len(close)
                continue
        if c == '"':
            i += 1
            while i < n:
                if rs[i] == "\\":
                    i += 2
                    continue
                if rs[i] == '"':
                    i += 1
                    break
                i += 1
            continue
        if c == "'":
            # 字符字面量 vs 生命周期：`'\n'` / `'x'` 是字符；`'_` / `'a` 是生命周期。
            if i + 2 < n and rs[i + 1] == "\\":
                i += 3
                while i < n and rs[i] != "'":
                    i += 1
                i += 1
                continue
            if i + 2 < n and rs[i + 2] == "'":
                i += 3
                continue
            i += 1
            continue
        mask[i] = 1
        i += 1
    return mask


FN_RE = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")


@dataclass
class RustParam:
    name: str
    key: str  # 前端要用的键名（camelCase 化后）
    type_text: str
    managed: bool = False  # State / AppHandle / … Tauri 内部注入
    optional: bool = False


@dataclass
class RustCommand:
    name: str
    file: str
    line: int
    params: List[RustParam] = field(default_factory=list)
    odd: List[str] = field(default_factory=list)  # 解析不了的形参


def to_frontend_key(name: str, rename_all: str = "camelCase") -> str:
    """Rust 形参名 → 前端键名（Tauri 的 `rename_all`）。"""
    if rename_all == "snake_case":
        return name
    parts = name.split("_")
    head = parts[0]
    tail = [p[:1].upper() + p[1:] for p in parts[1:] if p]
    if rename_all == "kebab-case":
        return "-".join([head] + [p.lower() for p in parts[1:] if p])
    if rename_all == "SCREAMING_SNAKE_CASE":
        return name.upper()
    if rename_all == "PascalCase":
        return "".join([head[:1].upper() + head[1:]] + tail)
    if rename_all in ("none", "verbatim"):
        return name
    return head + "".join(tail)  # camelCase（Tauri 默认）


def _norm_type(t: str) -> str:
    """压掉类型里的空白与生命周期，便于比对。"""
    t = re.sub(r"'\w*", "", t)
    return re.sub(r"\s+", "", t)


def _split_params(rs: str, mask: bytearray, lo: int, hi: int) -> List[Tuple[int, int]]:
    out: List[Tuple[int, int]] = []
    depth = 0
    start = lo
    for i in range(lo, hi):
        if not mask[i]:
            continue
        c = rs[i]
        if c in "([{<":
            depth += 1
        elif c in ")]}>":
            if depth == 0:  # 闭尖括号 = 返回类型起点，参数表到此为止
                break
            depth -= 1
        elif c == "," and depth == 0:
            out.append((start, i))
            start = i + 1
    if rs[start:hi].strip():
        out.append((start, hi))
    return out


def parse_params(rs: str, mask: bytearray, lo: int, hi: int, rename_all: str) -> Tuple[List[RustParam], List[str]]:
    params: List[RustParam] = []
    odd: List[str] = []
    for a, b in _split_params(rs, mask, lo, hi):
        seg = rs[a:b].strip()
        while not seg and a < b:
            a += 1
            seg = rs[a:b].strip()
        if not seg or seg in ("&", "&&"):
            continue
        seg = re.sub(r"^pub(\([^)]*\))?\s+", "", seg)
        seg = re.sub(r"^mut\s+", "", seg)
        m = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", seg)
        if not m or ":" not in seg:
            odd.append(seg.replace("\n", " ")[:50])
            continue
        name = m.group(1)
        type_text = _norm_type(seg[m.end() :].lstrip(":"))
        last = type_text.split("::")[-1].split("<")[0].replace("&", "")
        managed = last in MANAGED_ARG_TYPES
        optional = type_text.startswith("Option<") or type_text.startswith("std::option::Option<")
        if name == "_":
            odd.append("匿名形参 %s" % type_text[:30])
            continue
        if name.startswith("_"):
            odd.append("下划线开头形参 %s" % name)
        params.append(RustParam(name, to_frontend_key(name, rename_all), type_text, managed, optional))
    return params, odd


def rs_commands(rs: str, rel_path: str) -> List[RustCommand]:
    mask = rs_mask(rs)
    out: List[RustCommand] = []
    for m in re.finditer(r"#\s*\[\s*tauri\s*::\s*command", rs):
        if not mask[m.start()]:
            continue  # 测试里 `trimmed == "#[tauri::command]"` 这种字符串不算
        # 属性本体（可能多层嵌套方括号）+ 其后的其他属性 / 文档注释
        i = _match_bracket(rs, mask, m.start())
        rename_all = "camelCase"
        attr_text = rs[m.start() : i]
        rm = re.search(r"rename_all\s*=\s*\"(\w+)\"", attr_text)
        if rm:
            rename_all = rm.group(1)
        j = _skip_trivia(rs, mask, i)
        while j < len(rs) and rs[j] == "#":
            j = _skip_trivia(rs, mask, _match_bracket(rs, mask, j))
        fm = FN_RE.match(rs, j)
        if not fm:
            continue
        popen = rs.index("(", fm.end())
        pclose = _match_paren(rs, mask, popen)
        params, odd = parse_params(rs, mask, popen + 1, pclose - 1, rename_all)
        out.append(
            RustCommand(
                name=fm.group(1),
                file=rel_path,
                line=rs.count("\n", 0, fm.start()) + 1,
                params=params,
                odd=odd,
            )
        )
    return out


def _match_bracket(rs: str, mask: bytearray, open_at: int) -> int:
    """`[` … 配平 `]`（跳过字符串/注释里的方括号）。"""
    depth = 0
    i = open_at
    while i < len(rs):
        if mask[i]:
            if rs[i] == "[":
                depth += 1
            elif rs[i] == "]":
                depth -= 1
                if depth == 0:
                    return i + 1
        i += 1
    return len(rs)


def _skip_trivia(rs: str, mask: bytearray, i: int) -> int:
    """跳空白与注释（Rust 版；`//!`/`///`/`/**` 都在此被吃掉；遇字符串停）。"""
    n = len(rs)
    while i < n:
        c = rs[i]
        if c in " \t\r\n":
            i += 1
        elif not mask[i] and rs.startswith("//", i):
            j = rs.find("\n", i)
            i = n if j < 0 else j + 1
        elif not mask[i] and rs.startswith("/*", i):
            j = rs.find("*/", i)
            i = n if j < 0 else j + 2
        else:
            break
    return i


def rs_handler(main_rs: str, rel_path: str) -> List[Tuple[str, str, int]]:
    """`generate_handler!` 里的 (module, command, line)。"""
    mask = rs_mask(main_rs)
    out: List[Tuple[str, str, int]] = []
    for m in re.finditer(r"generate_handler\s*!\s*\[", main_rs):
        if not mask[m.start()]:
            continue
        lo = main_rs.index("[", m.start())
        hi = _match_bracket(main_rs, mask, lo)
        for a, b in _split_params(main_rs, mask, lo + 1, hi - 1):
            a2, b2 = _trim_span(main_rs, mask, a, b)
            seg = main_rs[a2:b2]
            if not seg:
                continue
            parts = [p for p in seg.split("::") if p]
            out.append((parts[-2] if len(parts) >= 2 else "", parts[-1], main_rs.count("\n", 0, a2) + 1))
    return out


# ══════════════════════════════════════════════════════════════════════════
# 3. 比对
# ══════════════════════════════════════════════════════════════════════════


def _keynorm(k: str) -> str:
    return k.lower().replace("_", "").replace("-", "")


def _suggest(actual: str, expected: Sequence[str]) -> Tuple[str, str]:
    """把「疑似同一键」的实际情况归类：(kind, 建议键)。"""
    norm = {e: _keynorm(e) for e in expected}
    an = _keynorm(actual)
    for e, en in norm.items():
        if en == an:
            return ("casing" if e != actual else "same", e)
    close = difflib.get_close_matches(actual, list(expected), n=1, cutoff=0.72)
    if close:
        return ("typo", close[0])
    return ("extra", "")


def audit(
    root: str,
) -> Dict:
    fe_dir = os.path.join(root, REL_FRONTEND)
    cmd_dir = os.path.join(root, REL_CMDS)
    mod_rs = os.path.join(root, REL_CMDS_MOD)
    main_rs = os.path.join(root, REL_MAIN)
    missing_paths = [p for p in (fe_dir, cmd_dir, mod_rs, main_rs) if not os.path.exists(p)]
    if missing_paths:
        return {"ok": False, "env_error": "缺路径: " + ", ".join(missing_paths)}

    calls: List[InvokeCall] = []
    ts_files = sorted(
        os.path.join(fe_dir, f) for f in os.listdir(fe_dir) if f.endswith(".ts")
    )
    for dirpath, _dirnames, filenames in os.walk(fe_dir):
        for fn in sorted(filenames):
            if fn.endswith(".ts"):
                p = os.path.join(dirpath, fn)
                if p not in ts_files:
                    ts_files.append(p)
    for p in ts_files:
        with open(p, "r", encoding="utf-8") as fh:
            calls.extend(ts_invoke_calls(fh.read(), os.path.relpath(p, root)))
    calls.sort(key=lambda c: (c.file, c.line))

    commands: Dict[str, RustCommand] = {}
    dup_decl: List[str] = []
    rs_files = [mod_rs] + [
        os.path.join(cmd_dir, fn) for fn in sorted(os.listdir(cmd_dir)) if fn.endswith(".rs")
    ]
    for p in rs_files:
        if not os.path.isfile(p):
            continue
        with open(p, "r", encoding="utf-8") as fh:
            for c in rs_commands(fh.read(), os.path.relpath(p, root)):
                if c.name in commands:
                    dup_decl.append(c.name)
                commands[c.name] = c

    with open(main_rs, "r", encoding="utf-8") as fh:
        reg = rs_handler(fh.read(), os.path.relpath(main_rs, root))
    reg_names = [r[1] for r in reg]
    reg_set = set(reg_names)
    reg_dups = sorted({n for n in reg_names if reg_names.count(n) > 1})
    declared = set(commands)
    missing_reg = sorted(declared - reg_set)
    unknown_reg = sorted(reg_set - declared)

    errors: List[Dict] = []
    warnings: List[Dict] = []
    undecidable: List[Dict] = []
    dynamic_sites: List[Dict] = []
    used: Set[str] = set()

    for name in missing_reg:
        errors.append({"kind": "declared_not_registered", "command": name,
                       "file": commands[name].file, "line": commands[name].line,
                       "detail": "声明了但不在 generate_handler!，前端永远调不到"})
    for name in unknown_reg:
        errors.append({"kind": "registered_not_declared", "command": name,
                       "file": os.path.relpath(main_rs, root).replace(os.sep, "/"), "line": 0,
                       "detail": "注册表里没有对应的 #[tauri::command]（模块名/函数名拼错）"})
    for name in reg_dups:
        errors.append({"kind": "duplicate_registration", "command": name,
                       "file": os.path.relpath(main_rs, root).replace(os.sep, "/"), "line": 0,
                       "detail": "generate_handler! 重复注册"})
    for name in dup_decl:
        errors.append({"kind": "duplicate_declaration", "command": name,
                       "file": commands[name].file, "line": commands[name].line,
                       "detail": "同名 #[tauri::command] 声明了两次"})

    for call in calls:
        if call.cmd is None:
            undecidable.append({"file": call.file, "line": call.line, "reason": call.reason,
                                "keys": call.keys, "command": None})
            continue
        used.add(call.cmd)
        if call.reason:
            # 命令名认出来了但载荷认不出来 ⇒ 键集不可知，**不猜**，进人工清单。
            undecidable.append({"file": call.file, "line": call.line, "command": call.cmd,
                                "keys": call.keys, "reason": call.reason})
            continue
        cmd = commands.get(call.cmd)
        if cmd is None:
            errors.append({"kind": "unknown_command", "command": call.cmd,
                           "file": call.file, "line": call.line,
                           "detail": "前端调用的命令在 nt_commands 里没有声明"
                                     + ("（注册表里也没有）" if call.cmd not in reg_set else "")})
            continue
        expected = {p.key: p for p in cmd.params if not p.managed}
        extras: List[Dict] = []
        missing: List[Dict] = []
        if call.dynamic:
            dynamic_sites.append({"file": call.file, "line": call.line, "command": call.cmd,
                                  "keys": call.keys,
                                  "note": "载荷含展开/计算键 ⇒ 键集不完整：多余键照报，缺键不报"})
        for k in call.keys:
            if k in expected:
                continue
            kind, suggestion = _suggest(k, list(expected))
            extras.append({"key": k, "kind": kind, "suggest": suggestion})
        if not call.dynamic:
            for p in expected.values():
                if p.optional or p.key in call.keys:
                    continue
                if any(e["suggest"] == p.key for e in extras):
                    # 前端把 `title` 写成了 `titel`：同一条根因由 key_mismatch 报，
                    # 不再单开一条「缺键」（否则同一个错配被数两遍）。
                    continue
                missing.append({"key": p.key, "rust_name": p.name, "type": p.type_text,
                                "kind": "missing", "suggest": ""})
        for e in extras:
            errors.append({
                "kind": "key_mismatch" if e["suggest"] else "unknown_key",
                "command": call.cmd, "file": call.file, "line": call.line,
                "expected": e["suggest"] or "（无同名形参）",
                "actual": e["key"],
                "hint": {"casing": "大小写/下划线不符（serde camelCase）", "same": "同名",
                         "typo": "疑似拼写偏差", "extra": "Rust 侧没有这个形参 ⇒ 该键被静默丢弃"}.get(
                             e["kind"], ""),
                "detail": "Rust 形参 %s ⇒ 前端键 %s" % (
                    (expected[e["suggest"]].name + " ⇒ " + e["suggest"]) if e["suggest"] in expected else "—",
                    e["key"]),
            })
        for m in missing:
            errors.append({
                "kind": "key_mismatch_missing", "command": call.cmd,
                "file": call.file, "line": call.line,
                "expected": m["key"], "actual": "（未传）",
                "hint": "非 Option 形参缺失 ⇒ 运行期报 missing key",
                "detail": "Rust 形参 %s: %s 是必填" % (m["rust_name"], m["type"]),
            })
        for p in expected.values():
            if p.optional and p.key not in call.keys and not call.dynamic:
                warnings.append({"command": call.cmd, "file": call.file, "line": call.line,
                                 "key": p.key, "rust_name": p.name, "type": p.type_text,
                                 "detail": "Option 形参未传 ⇒ 该值运行期为 None（合法，但值得看一眼）"})

    no_frontend = sorted(declared - used)
    odd_params = [
        {"command": c.name, "file": c.file, "line": c.line, "params": c.odd}
        for c in sorted(commands.values(), key=lambda c: (c.file, c.line))
        if c.odd
    ]
    # 解析器健康：拿「源码里逐字出现的证据」当交叉下界，而不是拍一个固定门限
    # （固定门限在真仓库和小样本上会互相打架）。任一条破了 ⇒ 结论不可信。
    raw_attr = 0
    for p in rs_files:
        if not os.path.isfile(p):
            continue
        with open(p, "r", encoding="utf-8") as fh:
            text = fh.read()
        mask = rs_mask(text)
        raw_attr += sum(1 for m in re.finditer(r"#\s*\[\s*tauri\s*::\s*command", text) if mask[m.start()])
    raw_invoke = 0
    for p in ts_files:
        with open(p, "r", encoding="utf-8") as fh:
            text = fh.read()
        mask = ts_mask(text)
        raw_invoke += sum(1 for m in re.finditer(r"invoke", text, re.IGNORECASE) if mask[m.start()])
    parse_health = {
        "commands_declared": len(declared),
        "commands_registered": len(reg_names),
        "commands_used_frontend": len(used),
        "ts_files": len(ts_files),
        "invoke_calls": len(calls),
        "invoke_calls_parsed": sum(1 for c in calls if c.cmd),
        "rs_attr_mentions_in_code": raw_attr,
        "invoke_mentions_in_code": raw_invoke,
    }
    suspect: List[str] = []
    if len(declared) < raw_attr:
        suspect.append("代码区里有 %d 个 #[tauri::command] 却只解析出 %d 个声明" % (raw_attr, len(declared)))
    if not len(declared):
        suspect.append("一个命令都没抓到")
    if not len(reg_names):
        suspect.append("generate_handler! 没抓到任何注册项")
    if not len(calls):
        suspect.append("前端一个 invoke() 都没抓到")
    if parse_health["invoke_calls_parsed"] > raw_invoke:
        suspect.append("解析出的调用点多于源码里 invoke 的逐字出现次数（不可能，解析器有 bug）")
    if suspect:
        parse_health["suspect"] = "；".join(suspect)

    report = {
        "ok": not errors,
        "root": root,
        "counts": parse_health,
        "errors": errors,
        "warnings": warnings,
        "undecidable": undecidable,
        "dynamic_payload_sites": dynamic_sites,
        "no_frontend_call": no_frontend,
        "odd_params": odd_params,
        "registration": {
            "declared": len(declared),
            "registered": len(reg_names),
            "declared_not_registered": missing_reg,
            "registered_not_declared": unknown_reg,
            "duplicates": reg_dups,
        },
        "commands": {
            name: {
                "file": c.file,
                "line": c.line,
                "keys": {p.key: ("managed" if p.managed else ("optional" if p.optional else "required"))
                         for p in c.params},
            }
            for name, c in commands.items()
        },
        "frontend_keys": {c.cmd: sorted(c.keys) for c in calls if c.cmd},
    }
    return report


# ══════════════════════════════════════════════════════════════════════════
# 4. 报告
# ══════════════════════════════════════════════════════════════════════════


KEY_ERR_KINDS = frozenset({"key_mismatch", "key_mismatch_missing", "unknown_key"})


def _rule(title: str) -> str:
    return "\n── %s %s" % (title, "─" * max(4, 62 - len(title) * 2))


def render(rep: Dict) -> str:
    if "env_error" in rep:
        return "ABORT: %s" % rep["env_error"]
    out: List[str] = []
    c = rep["counts"]
    out.append("nt_ipc_keys — 前后端 IPC 键名核对")
    out.append(
        "  命令 声明 %d / 注册 %d / 前端用到 %d ；invoke 调用点 %d（可解析 %d），前端文件 %d"
        % (c["commands_declared"], c["commands_registered"], c["commands_used_frontend"],
           c["invoke_calls"], c["invoke_calls_parsed"], c["ts_files"])
    )
    if "suspect" in c:
        out.append("  ⚠ %s" % c["suspect"])

    out.append(_rule("① 键名不匹配（error）"))
    key_errs = [e for e in rep["errors"] if e["kind"] in KEY_ERR_KINDS]
    other = [e for e in rep["errors"] if e["kind"] not in KEY_ERR_KINDS]
    if not key_errs:
        out.append("  （无）")
    for e in key_errs:
        out.append(
            "  %s:%s  %s" % (e["file"], e["line"], e["command"])
        )
        out.append("      期望 %s  ←  实得 %s" % (e["expected"], e["actual"]))
        if e.get("hint"):
            out.append("      %s ｜ %s" % (e["hint"], e["detail"]))

    out.append(_rule("② 声明 / 注册 差集"))
    r = rep["registration"]
    out.append("  声明 %d，注册 %d" % (r["declared"], r["registered"]))
    for label, key in (("声明了却没注册（前端调不到）", "declared_not_registered"),
                       ("注册了却没声明（拼错）", "registered_not_declared"),
                       ("重复注册", "duplicates")):
        out.append("  %s: %s" % (label, ", ".join(r[key]) if r[key] else "无"))
    if other:
        out.append("  其他 error:")
        for e in other:
            loc = "%s:%s" % (e["file"], e["line"]) if e["line"] else e["file"]
            out.append("    [%s] %s  %s" % (e["kind"], loc, e["detail"]))

    out.append(_rule("③ 无法静态判定（人工清单）"))
    if not (rep["undecidable"] or rep["odd_params"] or rep["dynamic_payload_sites"]):
        out.append("  （无）")
    for u in rep["undecidable"]:
        keys = ("  已认出的键 %s" % u["keys"]) if u.get("keys") else ""
        out.append("  %s:%s  %s%s" % (u["file"], u["line"], u["reason"], keys))
    for d in rep["dynamic_payload_sites"]:
        out.append("  %s:%s  %s  %s（已认出的键 %s）" %
                   (d["file"], d["line"], d["command"], d["note"], d["keys"]))
    for o in rep["odd_params"]:
        out.append("  %s:%s  %s 形参解析不了: %s" % (o["file"], o["line"], o["command"], o["params"]))

    if rep["warnings"]:
        out.append(_rule("④ warning：可选形参未传（合法，但要扫一眼）"))
        for w in rep["warnings"]:
            out.append("  %s:%s  %s  未传 `%s`（%s: %s）" %
                       (w["file"], w["line"], w["command"], w["key"], w["rust_name"], w["type"]))

    out.append(_rule("⑤ info：声明了但前端无调用点（不是错）"))
    out.append("  %d 个: %s" % (len(rep["no_frontend_call"]),
                                ", ".join(rep["no_frontend_call"]) or "无"))

    out.append("")
    # **汇总行不许说谎。** 早先这行只看 `rep["ok"]`（= 没有键名错配），
    # 于是「一个调用点都没抓到」这种**解析器失明**状态会打印「⇒ OK」而退出码是 2
    # —— 退出码是对的（fail-closed），但**给人看的那行字在说反话**。
    # 只扫退出码的人以为绿了，只看输出的人被骗。这与本仓库反复修的
    # 「注释/回执说谎」是同一类缺陷，只是这次轮到了工具自己的输出。
    suspect = (rep.get("counts") or {}).get("suspect")
    if not rep["ok"]:
        verdict = "有错配"
    elif suspect:
        # 解析器自己都怀疑自己覆盖面时，**不能**报 OK。
        verdict = "解析器可疑（非 OK，见上 ⚠）"
    else:
        verdict = "OK"
    out.append("合计 error %d ／ warning %d ／ 无法判定 %d ⇒ %s" %
               (len(rep["errors"]), len(rep["warnings"]),
                len(rep["undecidable"]) + len(rep["odd_params"]) + len(rep["dynamic_payload_sites"]),
                verdict))
    return "\n".join(out)


# ══════════════════════════════════════════════════════════════════════════
# 5. 自测（内联样本，不依赖仓库文件）
# ══════════════════════════════════════════════════════════════════════════

TS_OK = """
import { invoke, Channel } from "@tauri-apps/api/core";
// 注释里的 invoke("neobot_ghost", { badKey: 1 }) 不该被算
const s = "invoke('neobot_str_lit', { x: 1 })";   // 字符串里的也不该
const re = /invoke\\("neobot_regex", { y: 1 }\\)/;  // 正则里的更不该
async function go(a: string) {
  const ch = new Channel<any>();
  await invoke("neobot_task_claim", { taskId: a, actorId: "me" });
  await invoke<string>("neobot_convo_rename", { id: "c1", title: "t" });
  await invoke(
    "neobot_run_stream",
    { actorName: "me", onEvent: ch },
  );
  await invoke("neobot_tasks");
  await invoke("neobot_opt", { x: "1" });                 // Option 未传 → warning
  await invoke("neobot_snake", { id: "c", task_id: "t" }); // rename_all=snake_case
  const payload = { dyn: 1 };
  await invoke("neobot_var_payload", payload);            // 无法判定
  await invoke(NAME, { a: 1 });                            // 无法判定
  await invoke("neobot_spread", { ...payload, id: 2 });    // 动态键
}
"""

TS_BAD = """
async function go() {
  await invoke("neobot_task_claim", { task_id: "t1", actorId: "me" });  // snake ≠ camel
  await invoke("neobot_rename", { id: "c", titel: "typo" });            // 拼错（顶掉 title）
  await invoke("neobot_extra", { id: "c", ghostKey: 1 });               // 多余键
  await invoke("neobot_ghost_cmd", { a: 1 });                           // 命令压根不存在
  await invoke("neobot_need_two", { a: 1 });                            // 缺一个必填键
}
"""

RS_OK = """
use std::collections::HashMap;
/// #[tauri::command]  ← 这行在注释里，不该被算成声明
#[tauri::command]
pub async fn neobot_task_claim(
    task_id: String,
    actor_id: String,
    gate: tauri::State<'_, DaemonMap>,
) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn neobot_convo_rename(id: String, title: String, app: tauri::AppHandle) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn neobot_run_stream(
    actor_name: String,
    on_event: tauri::ipc::Channel<StreamEvent>,
) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn neobot_tasks() -> Result<Vec<i64>, String> { Ok(vec![]) }

#[tauri::command]
pub fn neobot_opt(x: String, note: Option<String>) -> Result<(), String> { Ok(()) }

#[tauri::command(rename_all = "snake_case")]
pub fn neobot_snake(id: String, task_id: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn neobot_var_payload(dyn: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn neobot_spread(id: i64) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn neobot_unused(x: String) -> Result<(), String> { Ok(()) }
"""

RS_BAD = """
#[tauri::command]
pub fn neobot_rename(id: String, title: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn neobot_extra(id: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn neobot_need_two(a: String, b: String) -> Result<(), String> { Ok(()) }
"""

MAIN_OK = """
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            nt_cmd_sys::neobot_task_claim,
            nt_cmd_convo::neobot_convo_rename,
            nt_cmd_run::neobot_run_stream,
            nt_cmd_tasks::neobot_tasks,
            nt_cmd_x::neobot_opt,
            nt_cmd_x::neobot_snake,
            nt_cmd_x::neobot_var_payload,
            nt_cmd_x::neobot_spread,
            nt_cmd_x::neobot_unused,
        ])
}
"""


def _write_fake(root: str) -> None:
    fe = os.path.join(root, REL_FRONTEND)
    cd = os.path.join(root, REL_CMDS)
    os.makedirs(fe, exist_ok=True)
    os.makedirs(cd, exist_ok=True)
    with open(os.path.join(fe, "a.ts"), "w", encoding="utf-8") as fh:
        fh.write(TS_OK + TS_BAD)
    with open(os.path.join(cd, "nt_cmd_sys.rs"), "w", encoding="utf-8") as fh:
        fh.write(RS_OK)
    with open(os.path.join(cd, "nt_cmd_extra.rs"), "w", encoding="utf-8") as fh:
        fh.write(RS_BAD)
    with open(os.path.join(root, REL_CMDS_MOD), "w", encoding="utf-8") as fh:
        fh.write("pub mod nt_cmd_sys;\npub mod nt_cmd_extra;\n")
    with open(os.path.join(root, REL_MAIN), "w", encoding="utf-8") as fh:
        fh.write(MAIN_OK)


def selftest() -> int:
    import tempfile

    checks: List[Tuple[str, bool, str]] = []

    def check(name: str, got, want) -> None:
        checks.append((name, got == want, "got=%r want=%r" % (got, want)))

    def by_kind(rep: Dict, kind: str) -> List[Dict]:
        return [e for e in rep["errors"] if e["kind"] == kind]

    # ── TS 解析层（不落盘）──────────────────────────────────────────────
    calls = ts_invoke_calls(TS_OK + TS_BAD, "a.ts")
    parsed = [c for c in calls if c.cmd]
    cmds = {c.cmd for c in parsed}
    check("TS: 可判定的 invoke 数", len(parsed), 13)
    check("TS: 注释里的 invoke 不算数", "neobot_ghost" in cmds, False)
    check("TS: 字符串字面量里的不算", "neobot_str_lit" in cmds, False)
    check("TS: 正则里的不算", "neobot_regex" in cmds, False)
    check("TS: 跨行调用能抓", "neobot_run_stream" in cmds, True)
    check("TS: 无参调用能抓", "neobot_tasks" in cmds, True)
    # 「无法判定」= 命令名认不出（cmd is None）**或**命令名认出了但载荷认不出（reason 非空）
    und = [c for c in calls if c.cmd is None or c.reason]
    check("TS: 2 处无法判定（变量载荷 / 非字面量命令名）", len(und), 2)
    check("TS: 变量载荷进人工清单（cmd 仍认得出，但键集不可知）",
          [(c.cmd, "不是对象字面量" in c.reason) for c in und], [("neobot_var_payload", True), (None, False)])
    check("TS: 非字面量命令名进人工清单", any("不是字符串字面量" in c.reason for c in und), True)
    spread = [c for c in parsed if c.cmd == "neobot_spread"][0]
    check("TS: 展开键标 dynamic", (spread.keys, spread.dynamic), (["id"], True))
    check("TS: 键名与顺序稳定（取最后一条同命令调用）",
          [c for c in parsed if c.cmd == "neobot_task_claim"][-1].keys, ["task_id", "actorId"])
    check("TS: 引号键认得",
          ts_invoke_calls('invoke("x", { "a-b": 1 });', "q.ts")[0].keys, ["a-b"])
    check("TS: 简写属性当键",
          ts_invoke_calls("invoke(\"x\", { a, b: 1 });", "q.ts")[0].keys, ["a", "b"])
    check("TS: 嵌套对象/数组不误切",
          ts_invoke_calls('invoke("x", { a: { b: [1, 2] }, c: 3 });', "q.ts")[0].keys, ["a", "c"])

    # ── Rust 解析层 ─────────────────────────────────────────────────────
    rcs = rs_commands(RS_OK, "nt_cmd_sys.rs")
    rmap = {c.name: c for c in rcs}
    check("RS: 抓到 9 个命令", len(rcs), 9)
    check("RS: 注释里的 #[tauri::command] 不算数", len(rcs) == len(rmap), True)
    check("RS: 形参名抓对",
          [p.name for p in rmap["neobot_task_claim"].params], ["task_id", "actor_id", "gate"])
    check("RS: camelCase 键",
          [p.key for p in rmap["neobot_task_claim"].params], ["taskId", "actorId", "gate"])
    check("RS: State 是托管参数", rmap["neobot_task_claim"].params[2].managed, True)
    check("RS: AppHandle 是托管参数", rmap["neobot_convo_rename"].params[2].managed, True)
    check("RS: Channel 不是托管参数", rmap["neobot_run_stream"].params[1].managed, False)
    check("RS: Channel 键 = onEvent", rmap["neobot_run_stream"].params[1].key, "onEvent")
    check("RS: 多行签名解析", len(rmap["neobot_run_stream"].params), 2)
    check("RS: Option → optional", rmap["neobot_opt"].params[1].optional, True)
    check("RS: rename_all=snake_case 不转驼峰",
          [p.key for p in rmap["neobot_snake"].params], ["id", "task_id"])
    check("RS: 生命周期不误判成字符字面量", len(rmap["neobot_task_claim"].params), 3)
    check("RS: to_frontend_key 驼峰", to_frontend_key("src_path"), "srcPath")
    check("RS: to_frontend_key 单段", to_frontend_key("id"), "id")
    check("RS: to_frontend_key 全驼峰保持", to_frontend_key("actorID"), "actorID")
    check("RS: to_frontend_key kebab", to_frontend_key("task_id", "kebab-case"), "task-id")

    reg = rs_handler(MAIN_OK, "main.rs")
    check("MAIN: 注册 9 条", len(reg), 9)
    check("MAIN: 模块名 + 行号抓对", reg[0], ("nt_cmd_sys", "neobot_task_claim", 5))

    # ── 端到端（落盘到临时目录，不碰真仓库）─────────────────────────────
    with tempfile.TemporaryDirectory() as tmp:
        _write_fake(tmp)
        rep = audit(tmp)
        check("E2E: 声明 12 个", rep["registration"]["declared"], 12)
        check("E2E: 注册 9 个", rep["registration"]["registered"], 9)
        check("E2E: 漏注册 = 未注册文件里那 3 个",
              sorted(rep["registration"]["declared_not_registered"]),
              ["neobot_extra", "neobot_need_two", "neobot_rename"])
        check("E2E: 注册了却没声明 = 空", rep["registration"]["registered_not_declared"], [])
        check("E2E: 重复注册 = 空", rep["registration"]["duplicates"], [])
        check("E2E: 键名不匹配 3 处（casing 1 + typo 1 + 缺必填 1）",
              len(by_kind(rep, "key_mismatch")) + len(by_kind(rep, "key_mismatch_missing")), 3)
        check("E2E: casing 配对成一条（不重复计缺键）",
              sorted((e["expected"], e["actual"]) for e in by_kind(rep, "key_mismatch")),
              [("taskId", "task_id"), ("title", "titel")])
        check("E2E: 拼错那条点名 title",
              [e["actual"] for e in by_kind(rep, "key_mismatch") if e["expected"] == "title"],
              ["titel"])
        check("E2E: 真正缺的必填键被点名",
              [e["expected"] for e in by_kind(rep, "key_mismatch_missing")], ["b"])
        check("E2E: 命令不存在 1 条",
              [e["command"] for e in by_kind(rep, "unknown_command")], ["neobot_ghost_cmd"])
        check("E2E: Option 未传只报 warning 不报 error",
              [(w["command"], w["key"]) for w in rep["warnings"]], [("neobot_opt", "note")])
        check("E2E: 动态载荷（...展开）不产生缺键误报",
              [e["command"] for e in by_kind(rep, "key_mismatch_missing")], ["neobot_need_two"])
        check("E2E: 载荷不可知的调用点不算缺键",
              [e["command"] for e in by_kind(rep, "unknown_key")], ["neobot_extra"])
        check("E2E: 交叉下界：声明数 == 代码区里的 #[tauri::command] 数",
              rep["counts"]["commands_declared"], rep["counts"]["rs_attr_mentions_in_code"])
        check("E2E: 交叉下界：解析出的调用点不超过 invoke 逐字出现数",
              rep["counts"]["invoke_calls_parsed"] <= rep["counts"]["invoke_mentions_in_code"], True)
        check("E2E: 无法判定 2 条", len(rep["undecidable"]), 2)
        check("E2E: 动态载荷进人工清单（只报多余键、不报缺键）",
              [(d["command"], d["keys"]) for d in rep["dynamic_payload_sites"]],
              [("neobot_spread", ["id"])])
        check("E2E: 无前端调用点只报 info", sorted(rep["no_frontend_call"]), ["neobot_unused"])
        check("E2E: 解析器健康自检无告警", "suspect" in rep["counts"], False)
        check("E2E: 假阳性总闸 —— error 恰好 8 条（键名 3 + 多余 1 + 未知命令 1 + 漏注册 3）",
              len(rep["errors"]), 8)
        check("E2E: 每条 error 都能定位到行",
              all(e["line"] > 0 for e in by_kind(rep, "key_mismatch") + by_kind(rep, "unknown_key")),
              True)

    # ── 变异测试：改坏一侧必须变红（证明断言不是空转）────────────────────
    def mutated(path_rel: str, old: str, new: str) -> Dict:
        with tempfile.TemporaryDirectory() as tmp:
            _write_fake(tmp)
            p = os.path.join(tmp, path_rel)
            src = open(p, encoding="utf-8").read()
            assert old in src, "变异锚点没找到: %s" % old
            with open(p, "w", encoding="utf-8") as fh:
                fh.write(src.replace(old, new))
            return audit(tmp)

    rep_m = mutated(os.path.join(REL_FRONTEND, "a.ts"),
                    '{ task_id: "t1", actorId: "me" }', '{ taskId: "t1", actorId: "me" }')
    check("变异: 前端键改对 ⇒ 键名 error 3→2",
          len(by_kind(rep_m, "key_mismatch")) + len(by_kind(rep_m, "key_mismatch_missing")), 2)
    check("变异: 改对后整体仍非 0（别的错还在）", rep_m["ok"], False)

    rep_m2 = mutated(os.path.join(REL_CMDS, "nt_cmd_extra.rs"),
                     "pub fn neobot_extra", "pub fn neobot_extra_renamed")
    check("变异: Rust 侧改名 ⇒ unknown_command 冒出来",
          [e["command"] for e in by_kind(rep_m2, "unknown_command")],
          ["neobot_extra", "neobot_ghost_cmd"])

    rep_m3 = mutated(os.path.join(REL_MAIN), "nt_cmd_x::neobot_opt,", "")
    check("变异: 从注册表删一行 ⇒ 漏注册 +1",
              sorted(rep_m3["registration"]["declared_not_registered"]),
              ["neobot_extra", "neobot_need_two", "neobot_opt", "neobot_rename"])
    check("变异: 删注册后仍能算出总数", rep_m3["registration"]["registered"], 8)

    rep_m4 = mutated(os.path.join(REL_MAIN), "nt_cmd_x::neobot_opt,",
                     "nt_cmd_x::neobot_opt,\n            nt_cmd_x::neobot_opt,")
    check("变异: 注册表重复一行 ⇒ duplicate_registration",
          [e["command"] for e in by_kind(rep_m4, "duplicate_registration")], ["neobot_opt"])

    rep_m5 = mutated(os.path.join(REL_FRONTEND, "a.ts"),
                     '{ id: "c", titel: "typo" }', '{ id: "c", title: "typo" }')
    check("变异: typo 改对 ⇒ key_mismatch 只剩 casing 那条",
          len(by_kind(rep_m5, "key_mismatch")), 1)

    bad = [n for n, ok, _ in checks if not ok]
    for name, ok, detail in checks:
        print("  [%s] %s%s" % ("ok" if ok else "FAIL", name, "" if ok else "  \u2190 " + detail))
    print("[ipc-keys selftest] %s — %d/%d 通过" % ("PASS" if not bad else "FAIL",
                                                   len(checks) - len(bad), len(checks)))
    return 0 if not bad else 1


def main() -> int:
    ap = argparse.ArgumentParser(description="前后端 IPC 键名静态核对器")
    ap.add_argument("--root", default=os.path.dirname(os.path.dirname(os.path.dirname(
        os.path.abspath(__file__)))), help="仓库根目录（默认按本脚本位置推）")
    ap.add_argument("--json", action="store_true", help="机器可读输出")
    ap.add_argument("--self-test", "--selftest", dest="self_test", action="store_true",
                    help="内联样本自测（不读仓库文件）")
    args = ap.parse_args()
    if args.self_test:
        return selftest()
    try:
        rep = audit(args.root)
    except Exception as exc:  # noqa: BLE001
        print("ABORT: 解析异常: %s: %s" % (type(exc).__name__, exc), file=sys.stderr)
        return 2
    if "env_error" in rep:
        print("ABORT: %s" % rep["env_error"], file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(rep, ensure_ascii=False, indent=1, default=str))
    else:
        print(render(rep))
    if "suspect" in rep["counts"]:
        return 2
    return 0 if rep["ok"] else 1


if __name__ == "__main__":
    sys.exit(main() or 0)
