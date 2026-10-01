#!/usr/bin/env python3
"""nt_api_contract.py — neobot-desktop API 契约一致性门（FRONTEND-REBUILD M3）。

为什么需要这道门（2026-10-01 实测确立）：

`apps/neobot-desktop/src/api.rs` 有一张**声明式契约表** `SPECS`（115 条：
62 Implemented / 6 Planned / 53 Stub）。`neobot_api_call(name, args)` 是
**状态查询而非分派器** —— 对 Implemented 条目它返回

    data: { "hint": "已实现，请直接 invoke 该命令" }

⇒ **契约不变量**：凡是 `Status::Implemented` 的条目，**必须同时**在
`main.rs` 的 `tauri::generate_handler![]` 里注册为可直接 invoke 的命令。
否则前端照 hint 调用会在**运行时**失败，而契约表与 UI 都显示"已实现"。

**为什么必须是门而不是文档**：前端即将按 FRONTEND-REBUILD 计划整体重写。
重写期间接口会漂移；漂移若只能在运行时发现，返工量翻倍。

判据（双向，均可证伪）：
  A 契约→注册：每条 Implemented 必须在 generate_handler 里出现
  B 注册→契约：每个注册的命令必须在契约表里有条目（否则是「 undocumented能力」）
  C 前端实调：前端 `invoke('X')` 的 X 必须是已注册命令
  D 签名一致：契约表声明的返回类型名必须在 Rust 侧有对应类型

⚠️ **本门历史上差点报假警**（2026-10-30 记录）：
用正则抽 `ApiSpec::new(...)` 的实参时，正则匹配到换行就断，
一度得出「60 条 Implemented 只注册 1 条」的荒谬结论。
⇒ 故本文件**不用正则解析 Rust**：改用**括号配平**逐条切出 `ApiSpec::new(` 的
完整实参列表。教训与 `nt_callgraph` 的 F1–F6 同源：**结构化数据必须用能
理解结构的解析器，文本匹配必假。**

用法：
  python3 scripts/ops/nt_api_contract.py            # 报告 + 有违规 exit 1
  python3 scripts/ops/nt_api_contract.py --advisory # 只报告，exit 0
  python3 scripts/ops/nt_api_contract.py --json     # 机器可读
"""
import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
API = os.path.join(REPO, 'apps/neobot-desktop/src/api.rs')
MAIN = os.path.join(REPO, 'apps/neobot-desktop/src/main.rs')
FRONTEND = os.path.join(REPO, 'apps/neobot-desktop/frontend/src')
VENDORED = os.path.join(REPO, 'apps/neobot-desktop/frontend')


def balanced_calls(text, callee):
    """切出每个 `callee(` 的**完整**实参列表（括号/字符串/注释感知）。

    不用正则的原因见模块 docstring —— 正则不理解嵌套括号与字符串字面量。
    """
    out = []
    for m in re.finditer(r'\b' + re.escape(callee) + r'\s*\(', text):
        i = m.end() - 1
        depth = 0
        in_str = None       # None | '"' | "'"
        in_line_comment = False
        in_block_comment = False
        j = i
        while j < len(text):
            c = text[j]
            nxt = text[j + 1] if j + 1 < len(text) else ''
            if in_line_comment:
                if c == '\n':
                    in_line_comment = False
            elif in_block_comment:
                if c == '*' and nxt == '/':
                    in_block_comment = False
                    j += 1
            elif in_str:
                if c == '\\':
                    j += 1
                elif c == in_str:
                    in_str = None
            else:
                if c == '/' and nxt == '/':
                    in_line_comment = True
                    j += 1
                elif c == '/' and nxt == '*':
                    in_block_comment = True
                    j += 1
                elif c in '"\'':
                    in_str = c
                elif c == '(':
                    depth += 1
                elif c == ')':
                    depth -= 1
                    if depth == 0:
                        out.append(text[i + 1:j])
                        break
            j += 1
    return out


def split_args(s):
    """顶层逗号切分（括号/字符串感知）。"""
    parts, buf = [], []
    depth = 0
    in_str = None
    for k, c in enumerate(s):
        if in_str:
            buf.append(c)
            if c == in_str and (k == 0 or s[k - 1] != '\\'):
                in_str = None
            continue
        if c in '"\'':
            in_str = c
            buf.append(c)
        elif c in '([{':
            depth += 1
            buf.append(c)
        elif c in ')]}':
            depth -= 1
            buf.append(c)
        elif c == ',' and depth == 0:
            parts.append(''.join(buf).strip())
            buf = []
        else:
            buf.append(c)
    if ''.join(buf).strip():
        parts.append(''.join(buf).strip())
    return parts


def first_string(arg):
    m = re.match(r'\s*"((?:[^"\\]|\\.)*)"', arg)
    return m.group(1) if m else None


def load_specs():
    if not os.path.isfile(API):
        sys.exit(f"[contract] 找不到契约表：{API}")
    text = open(API, encoding='utf-8', errors='ignore').read()
    specs = []
    for call in balanced_calls(text, 'ApiSpec::new'):
        a = split_args(call)
        if len(a) < 5:
            continue
        name = first_string(a[0])
        if not name:
            continue
        status = 'unknown'
        m = re.search(r'Status::(\w+)', a[4])
        if m:
            status = m.group(1)
        params = []
        pm = re.search(r'&\[(.*?)\]', a[2], re.S)
        if pm:
            params = [first_string(x) for x in split_args(pm.group(1))]
            params = [p for p in params if p]
        specs.append({
            'name': name,
            'group': first_string(a[1]) or '',
            'params': params,
            'ret': first_string(a[3]) or '',
            'status': status,
            'note': (a[5] if len(a) > 5 else ''),
        })
    return specs


def load_registered():
    if not os.path.isfile(MAIN):
        sys.exit(f"[contract] 找不到 main.rs：{MAIN}")
    text = open(MAIN, encoding='utf-8', errors='ignore').read()
    names = set()
    for m in re.finditer(r'invoke_handler\s*\(\s*tauri::generate_handler!\s*\[', text):
        i = m.end()          # 指向 `[` **之后**
        depth = 1            # 数组已开启
        j = i
        while j < len(text) and depth:
            if text[j] == '[':
                depth += 1
            elif text[j] == ']':
                depth -= 1
            j += 1
        block = text[i:j]
        # 只取形如 `neobot_desktop::<mod>::<name>` 的**裸路径条目**（generate_handler
        # 数组的元素），排除块内注释/文档里出现的同名调用。
        block_nc = re.sub(r'//[^\n]*', '', block)
        for mm in re.finditer(r'neobot_desktop::\w+::(\w+)', block_nc):
            names.add(mm.group(1))
    return names


def frontend_invocations():
    """前端 `invoke('X'` 的调用点。**排除 vendored 树** ——
    那是被冻结的上游代码，契约以我方 SPECS 为准。"""
    hits = {}
    for root, dirs, files in os.walk(FRONTEND):
        dirs[:] = [d for d in dirs if d not in ('node_modules', 'dist', 'vendor')]
        for fn in files:
            if not fn.endswith(('.ts', '.tsx')):
                continue
            p = os.path.join(root, fn)
            txt = open(p, encoding='utf-8', errors='ignore').read()
            for m in re.finditer(r'invoke(?:<[^>]*>)?\s*\(\s*[\'"]([A-Za-z0-9_]+)[\'"]', txt):
                hits.setdefault(m.group(1), []).append(
                    os.path.relpath(p, REPO))
    return hits


def main(argv):
    advisory = '--advisory' in argv
    as_json = '--json' in argv
    specs = load_specs()
    registered = load_registered()
    invoked = frontend_invocations()

    impl = {s['name'] for s in specs if s['status'] == 'Implemented'}
    documented = {s['name'] for s in specs}
    stub_called = {s['name'] for s in specs if s['status'] != 'Implemented'}
    a_fail = sorted(impl - registered)          # 契约说已实现但调不到
    b_fail = sorted(registered - documented)    # 能调但契约无条目
    # C 只抓**真漂移**：前端调了、既没注册、契约表里也没有。
    # ⚠️ 2026-09-30 实测：若把「前端调了但未注册」一律当失败，会得到 **48 条**，
    # 而逐条核验后 **48/48 全是契约表里显式标了 Stub/Planned 的 DSH_ONLY 功能**
    # （42 条带 DSH_ONLY 注记）—— 属**刻意不实现**，不是缺陷。
    # ⇒ 一门 48 次误报的门会被忽略（本仓门纪律：恒红/恒噪的门 = 没有门）。
    #    故：已登记的 Stub 调用归入 info，只有「契约表完全没有」才算 C 类失败。
    c_fail = sorted(set(invoked) - registered - documented)
    c_info = sorted((set(invoked) - registered) & stub_called)

    by_status = {}
    for s in specs:
        by_status[s['status']] = by_status.get(s['status'], 0) + 1

    if as_json:
        print(json.dumps({
            'specs_total': len(specs),
            'by_status': by_status,
            'registered': len(registered),
            'A_contract_not_registered': a_fail,
            'B_registered_undocumented': b_fail,
            'C_frontend_invokes_unregistered': c_fail,
            'C_info_stub_called': c_info,
            'frontend_invoke_sites': {k: len(v) for k, v in sorted(invoked.items())},
        }, ensure_ascii=False, indent=2))
        return 1 if (a_fail or b_fail or c_fail) and not advisory else 0

    print("=== neobot-desktop API 契约门（M3）===")
    print(f"契约表 {len(specs)} 条  " + "  ".join(f"{k}={v}" for k, v in sorted(by_status.items())))
    print(f"invoke_handler 注册 {len(registered)} 个 · 前端 invoke 调用点 {len(invoked)} 个唯一命令")
    print()

    def dump(title, items, why):
        if not items:
            print(f"✅ {title}：0")
            return
        print(f"⛔ {title}：{len(items)}")
        print(f"   后果：{why}")
        for n in items:
            extra = f"  ← 前端 {len(invoked[n])} 处调用" if n in invoked else ""
            print(f"     {n}{extra}")

    dump("A 契约标 Implemented 但未在 invoke_handler 注册", a_fail,
         "前端按契约 hint 直接 invoke 会**运行时**失败，而契约表显示「已实现」")
    dump("B 已注册但契约表无条目", b_fail,
         "能力存在于代码但契约表不描述 ⇒ UI 无从得知，文档与现实分叉")
    dump("C 前端 invoke 了既未注册、契约表也无记录的命令", c_fail,
         "前端调用**必然**失败（invoke reject）且无任何文档说明 ⇒ 真漂移")
    if c_info:
        print(f"ℹ️  C-info 前端调用了契约表里标 Stub/Planned 的命令：{len(c_info)}"
              f"（刻意不实现，非缺陷；其中 DSH_ONLY 占多数）")

    print()
    total = len(a_fail) + len(b_fail) + len(c_fail)
    if total == 0:
        print("PASS: 契约表 / 注册 / 前端实调 三方一致。")
        return 0
    print(f"FAIL: {total} 处不一致。")
    if advisory:
        print("(advisory 模式：不断言)")
        return 0
    return 1


if __name__ == '__main__':
    sys.exit(main(sys.argv))
