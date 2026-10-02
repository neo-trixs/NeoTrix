#!/usr/bin/env python3
"""自评函数丢弃入参门（2026-10-02）—— 拦「恒满分评判器」这一类缺陷。

# 为什么必须有

本会话实测到的**最严重缺陷**是 `eval_engine/llm_judge.rs::evaluate_response`：

```rust
let _ = (prompt, response);      // 入参整个丢弃
let normalized = 1.0;            // 恒 1.0
score: config.max_score,         // 恒给满分
```

它**任何输入都返回满分**。危害比「没有仪器」更大：恒返回最大值 ⇒
任何经此路径的「意识得分/涌现指数」都**自动为「已涌现」提供证据**。

# 为什么这个 bug 能活很久（**这才是门要拦的东西**）

它的 6 条单元测试**把伪造行为断言成了预期**：

```rust
// Equal scores → weighted average = max_score
assert!((result.total_score - 10.0).abs() < 0.01);
```

⇒ 「测试全绿」在这里**恰恰是缺陷被保护的证据**，不是质量证明。
⇒ 本门因此**不检查测试是否通过**，而检查**实现是否自称评判却丢弃入参** ——
因为「测试是绿的」无法区分「正确」与「把谎言钉成预期」。

# 判据（先定，再写实现）

**分两档**，依据是「它**能不能表达拒绝**」：

* **FAIL**：自评型函数（名字匹配 evaluate/judge/score/assess/rate/rank/grade/
  verify）丢弃形参，且返回类型**不是** `Option<…>`。
  ⇒ 它**没有拒绝的能力** ⇒ 只能返回一个「假装评判出来的」分数。
* **WARN（不失败）**：同上但返回类型**是** `Option<…>`。
  ⇒ 它**有能力**拒绝（`None`）；但仍需人确认它真的走了拒绝路径。
  这正是 `llm_judge::evaluate_response` 修复后的形态。

⇒ 判据的**原理**：危险的不是「参数没用」，而是「**无法表达拒绝**」。
一个返回 `Option` 的函数即使不读入参，也可以诚实地回答「我判不了」；
一个返回裸 `f32` 的函数不读入参，就**只能撒谎**。

⚠️ 这一分档是本门被自己的误报逼出来的：首版把「拒绝路径」的
`let _ = (prompt, response);` 也判成 FAIL —— 但那正是**修复后的正确形态**。
⇒ 门自己也必须能被自己的案例纠正。

⛔ 为什么不能只报「所有 `let _ = (…)`」：该写法是**合法的**未用警告压制手段，
本仓现存 4 处非测试用例（`sandbox_features.rs` 等）多数无害。
⇒ 只在「**自称要评判，却不看入参**」时报 —— 那才是语义依赖被丢掉。

⛔ 排除 `#[cfg(test)]` 模块内的代码：夹具里的 `let _ = (t, strength, st)`
是在构造测试数据，不是在评判。

⛔ 排除注释行：`// 并且用 `let _ = (prompt, response);` 把入参整个丢弃`
是**修复说明**，命中它会导致「修好了反而报错」。

退出码：0 = PASS，1 = FAIL。

# 反向的失效模式（写明以免被误用）
本门**只认显式 `let _ = (…)`**。若有人改成
`let _x = (a, b);` 或把形参改名 `_prompt` 来绕过，本门会漏。
⇒ 它是**兜底**，不是唯一防线；真正的防线是
`llm_judge` 里那条 36 组反向锁测试
（`test_no_input_can_ever_score_max`：任何输入都拿不到 max）。
"""
import re
import sys
from pathlib import Path

# 判据：自评型函数名
EVAL_NAME = re.compile(
    r'\b(?:evaluate|judge|score|assess|rate|rank|grade|verify)_?[a-z0-9_]*\b|'
    r'\b(?:evaluate|judge|score|assess|rate|rank|grade|verify)\b',
    re.IGNORECASE,
)
# 显式丢弃入参
DISCARD = re.compile(r'let\s+_\s*=\s*\(([^)]*)\)\s*;')
# 函数定义（Rust / Python 风格都覆盖）
FN_DEF = re.compile(r'^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)', re.MULTILINE)


def strip_comment(line: str) -> str:
    """粗略去掉行注释 —— ⛔ 不做字符串字面量解析（Rust 的 `"//"` 会误伤）。
    仅用于**门自身的保守过滤**：宁可漏报，不可对注释报错。"""
    t = line.strip()
    if t.startswith('//') or t.startswith('/*') or t.startswith('*'):
        return ''
    return line


def body_of(lines, start_idx):
    """从 fn 定义行起，按花括号配平取函数体；解析不到则返回 None。"""
    depth = 0
    seen = False
    out = []
    for i in range(start_idx, len(lines)):
        ln = lines[i]
        out.append(ln)
        depth += ln.count('{') - ln.count('}')
        if '{' in ln:
            seen = True
        if seen and depth <= 0:
            return out, i
    return None, None


def scan_file(path: Path):
    findings = []
    try:
        src = path.read_text(encoding='utf-8', errors='replace')
    except OSError:
        return findings
    lines = src.split('\n')
    in_test = False
    test_depth = None
    depth = 0
    for idx, raw in enumerate(lines):
        # 追踪 #[cfg(test)] mod 区域（粗略：以 mod 行 + 花括号配平）
        if not in_test and re.match(r'\s*(?:#\[cfg\(test\)\]\s*)?mod\s+\w*test', raw):
            test_depth = depth
            in_test = True
        depth += raw.count('{') - raw.count('}')
        if in_test and depth <= (test_depth or 0):
            in_test = False

        line = strip_comment(raw)
        if not line:
            continue
        m = FN_DEF.match(line)
        if not m:
            continue
        name = m.group(1)
        # 返回类型：可能跨行（多行签名时 `-> T {` 在 fn 行之后最多 5 行处）
        ret = ''
        for j in range(idx, min(idx + 6, len(lines))):
            r = re.search(r'->\s*(.+?)\s*\{\s*$', strip_comment(lines[j]))
            if r:
                ret = r.group(1)
                break
        if not EVAL_NAME.search(name):
            continue
        body, _ = body_of(lines, idx)
        if body is None:
            continue
        # 函数体内的丢弃（逐行再过一遍注释过滤）
        for off, bl in enumerate(body):
            bl2 = strip_comment(bl)
            if not bl2:
                continue
            for dm in DISCARD.finditer(bl2):
                names = [n.strip() for n in dm.group(1).split(',') if n.strip()]
                if not names:
                    continue
                findings.append((path, idx + off + 1, name, names, ret))
    return findings


def main():
    roots = sys.argv[1:] or ['neotrix-core/src', 'crates']
    files = []
    for r in roots:
        p = Path(r)
        if p.is_file():
            files.append(p)
        elif p.is_dir():
            files += [f for f in p.rglob('*.rs') if 'target' not in f.parts]
    files.sort()

    findings = []
    for f in files:
        findings += scan_file(f)

    print('=== 自评函数丢弃入参门（拦「恒满分评判器」）===')
    print('扫描 %d 个 .rs' % len(files))
    if not findings:
        print('PASS: 未发现「自称评判却丢弃入参」的函数。')
        return 0
    hard = [f for f in findings if not f[4].lstrip().startswith('Option')]
    soft = [f for f in findings if f[4].lstrip().startswith('Option')]
    for path, line, name, names, _ in soft:
        print('  WARN %s:%d  `%s` 返回 `%s` ⇒ **有能力**拒绝评分（确认它真走了 None 路径）'
              % (path, line, name, 'Option<…>'))
    if not hard:
        print('\nPASS: 无「无法表达拒绝却丢弃入参」的自评函数（%d 处 Option 返回型已 WARN）。'
              % len(soft))
        return 0
    print('\nFAIL: %d 处自评函数丢弃入参且**无法拒绝**' % len(hard))
    for path, line, name, names, ret in hard:
        print('  · %s:%d  `%s` -> %s  丢弃入参 %s' % (path, line, name, ret or '?', '、'.join(names)))
    print('\n判读：自评函数（evaluate/judge/score/…）丢弃入参 ⇒ 它宣称依赖这些输入，')
    print('      实际不读。**恒定返回值比没有函数更危险**：若返回上限值，')
    print('      等于自动为「已通过/已涌现」提供证据。')
    print('      修法：真读入参，或**显式拒绝**（返回 Option::None）而非恒定分数。')
    return 1


if __name__ == '__main__':
    sys.exit(main())
