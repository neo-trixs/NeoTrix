#!/usr/bin/env python3
"""nt_fn_drift.py — 同名函数的「重复实现」与「语义漂移」审计。

为什么存在【实测动因】
  本会话用行为对位抓到并修掉一个真 bug：`entity_linking::linker::levenshtein`
  按 **UTF-8 字节**算编辑距离，而同仓另外两份（`semantic_entropy::levenshtein_distance`、
  `crawl/adaptive/helpers::levenshtein_distance`）按 **char**。⇒ 同类用途的实现
  副本之间**语义不一致**，且各自的单元测试**全是 ASCII**，谁也没发现。
  这不是孤例的坏运气，是一类**结构性缺陷：副本漂移**。
  本仓已有同类前科：`is_cjk` 四处副本（文档记为「八副本」）。
  ⇒ 缺的审计能力是：**「哪些函数有重复实现？副本之间语义一致吗？」**

候选来源：**编译器解析过的调用边**（`.project-map/edges-*.jsonl`），不是正则扫源码。
  DefId 带 crate 哈希 ⇒ 同名诱饵在结构上不可能混淆（`nt_calledges` 已实证）。

判定（每对同名函数，二选一 + 不可判）：
  IDENTICAL  归一化后函数体**逐字符相同** ⇒ 纯重复，可合并（不是 bug）
  DIFFERENT  函数体有实质差异 ⇒ **人工裁决**：可能是有意的（同名不同域），
            也可能是漂移（同名同用途不同语义）。本工具**不裁决**，只定位 + 给首差行
  UNRESOLVED 找不到定义 / 解析不出函数体 ⇒ **不猜**（宁缺勿错）

诚实边界（必须随输出一起看）：
  - 只能发现**同名**副本。⛔ **发现不了跨名漂移** —— 本会话那个字节/字符 bug
    恰恰是 `levenshtein` vs `levenshtein_distance` **不同名**，本工具抓不到。
    跨名漂移只能靠「同一能力配一份 vectors 做行为对位」发现（见 nt_parity_ref）。
  - `{impl#N}` 方法、测试函数已排除（同名 trait 方法是合法重复）。
  - 归一化会抹掉注释与字面量内容 ⇒ `// 中文注释` 差异**不算**差异（有意为之）。
  - **不是门**（不接 CI）：判据是文本相等，不足以当正确性判据（同 G7 裁决）。
    输出是给人/agent 分诊的**分诊单**。

用法：
  python3 scripts/ops/nt_fn_drift.py --db .project-map/edges-all.jsonl [--limit N]
  python3 scripts/ops/nt_fn_drift.py --db ... --only-different
  python3 scripts/ops/nt_fn_drift.py selftest
"""
import argparse
import json
import os
import re
import sys
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

IDENTICAL, DIFFERENT, UNRESOLVED = 'IDENTICAL', 'DIFFERENT', 'UNRESOLVED'


# ── 源码提取：字符串/注释感知的花括号配对 ────────────────────────────────────
def strip_noise(src):
    """去注释与字符串**内容**，保留结构。返回可归一化文本。

    为什么要字符串感知：naive 的花括号计数会被 `"}"` 或 `// {` 骗到，
    而本仓代码里字符串里带花括号是常态（format! 的模板）。
    """
    out = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == '/' and i + 1 < n and src[i + 1] == '/':
            j = src.find('\n', i)
            i = n if j < 0 else j
            continue
        if c == '/' and i + 1 < n and src[i + 1] == '*':
            j = src.find('*/', i + 2)
            i = n if j < 0 else j + 2
            out.append(' ')
            continue
        if c == '"':
            i += 1
            depth = 0
            while i < n:
                if src[i] == '\\':
                    i += 2
                    continue
                if src[i] == '"' and depth == 0:
                    i += 1
                    break
                i += 1
            out.append('""')  # 字符串一律归一为空串（内容差异不算语义差异）
            continue
        if c == "'":
            # char literal 或 lifetime：'a' / '\n' 才当字面量
            m = re.match(r"'(\\.|[^\\'])'", src[i:])
            if m:
                i += m.end()
                out.append("''")
                continue
            out.append(c)
            i += 1
            continue
        out.append(c)
        i += 1
    return ''.join(out)


def extract_fn(src, leaf):
    """取 `fn <leaf>(` 的函数体（归一化后）。返回 (body, start_line) 或 (None, 0)。

    只取**第一个**同签名定义：同文件内同名重载在 Rust 里不存在（除 trait impl，
    那些已被调用方过滤）。
    """
    clean = strip_noise(src)
    for m in re.finditer(r'\bfn\s+' + re.escape(leaf) + r'\s*(?:<[^>]*>)?\s*\(', clean):
        start = m.end()
        # 跳过参数表
        depth = 1
        i = start
        while i < len(clean) and depth > 0:
            if clean[i] == '(':
                depth += 1
            elif clean[i] == ')':
                depth -= 1
            i += 1
        # 返回类型 -> '{'
        while i < len(clean) and clean[i] != '{':
            if clean[i] == ';':
                i = -1
                break
            i += 1
        if i < 0 or i >= len(clean):
            continue
        # 花括号配对
        depth = 1
        j = i + 1
        while j < len(clean) and depth > 0:
            if clean[j] == '{':
                depth += 1
            elif clean[j] == '}':
                depth -= 1
            j += 1
        body = clean[i + 1:j - 1]
        line = clean[:m.start()].count('\n') + 1
        return normalize_code(body), line
    return None, 0


def normalize_code(body):
    """归一化到**足以比较语义**的粒度。

    v1 只做 `re.sub(r'\\s+', ' ')` ⇒ `x + 1` 与 `x+1` 判为不同。
    实测这是**误报方向**（偏严）：格式化工具（rustfmt 的空格风格、
    手写习惯）会制造纯空格差异，而那不是语义差异。
    ⇒ 现在把「空白 + 标点邻接空格」都归一，使 IDENTICAL 反映语义相等。

    ⚠️ 仍然**不做**的事：不删标识符、不重排语句、不做常量折叠。
    只归一排版 ⇒ 判据仍是「同形」，不是「等价」（工具定位是分诊单，非判决）。
    """
    s = re.sub(r'\s+', ' ', body).strip()
    # 去掉所有标识符/字面量**内部以外**的空格：让 `a + b` == `a+b` == `a  +  b`
    s = re.sub(r'\s*([^\w\s])\s*', r'\1', s)
    return s.strip()


def first_diff(a, b):
    n = min(len(a), len(b))
    for k in range(n):
        if a[k] != b[k]:
            return a[max(0, k - 30):k + 30], b[max(0, k - 30):k + 30]
    if len(a) != len(b):
        return a[n:n + 60] or '<shorter ends>', b[n:n + 60] or '<shorter ends>'
    return None, None


# ── 候选发现（走编译器的调用边，不用正则扫源码）────────────────────────────
SKIP = re.compile(r'^(test_|_test|bench_|main$)')


def discover(db_path):
    """leaf name -> {module: span}"""
    mods = defaultdict(dict)
    with open(db_path, encoding='utf-8', errors='replace') as fh:
        for line in fh:
            try:
                r = json.loads(line)
            except ValueError:
                continue
            c = r.get('caller') or ''
            if '{impl#' in c or '::tests' in c or '::benches' in c:
                continue
            parts = c.split('::')
            if len(parts) < 3:
                continue
            leaf = parts[-1]
            if not leaf or SKIP.match(leaf) or not re.match(r'^[a-z_][a-z0-9_]*$', leaf):
                continue
            mod = '::'.join(parts[1:-1])
            span = r.get('span') or ''
            file = span.split(':')[0]
            if file:
                mods[leaf].setdefault(mod, file)
    return {k: v for k, v in mods.items() if len(v) > 1}


def read(path):
    try:
        with open(os.path.join(REPO, path), encoding='utf-8', errors='replace') as fh:
            return fh.read()
    except OSError:
        return None


def compare_pair(leaf, mods):
    rows = []
    for mod, path in mods.items():
        src = read(path)
        if src is None:
            rows.append((mod, path, None, 0))
            continue
        body, line = extract_fn(src, leaf)
        rows.append((mod, path, body, line))
    if any(b is None for _, _, b, _ in rows):
        return UNRESOLVED, None, None, rows
    base = rows[0]
    for other in rows[1:]:
        if other[2] != base[2]:
            d_a, d_b = first_diff(base[2], other[2])
            return DIFFERENT, (base, other, d_a, d_b), None, rows
    return IDENTICAL, None, None, rows


def classify_drift(rows):
    """把「同名不同语义」按**已实证过的缺陷形状**自动分类。

    为什么要有形状而不是让人眼看 160 个名字：本会话已经**两次**踩到同一类缺陷
      形状 A「错误处理漂移」：同一操作 11 份宽容 / 2 份 panic（now_ts）
      形状 B「单位漂移」：char 距离 ÷ byte 长度（char_similarity）、字节版距离
    ⇒ 这两类不是巧合，是**可复发**的形状。凡命中形状的排在报告最前面，
    其余仍列出但降级（多半是「同名不同域」的合法重复，如 osint::investigate）。

    返回 (shape_name, why) 或 (None, None)。只报告**形状**，不裁决对错。
    """
    bodies = [(r[0], r[2] or '') for r in rows]
    panic_re = re.compile(r'\.(?:expect|unwrap)\s*\(|panic!\s*\(|unreachable!\s*\(')
    tolerant_re = re.compile(
        r'\.unwrap_or(?:_default|_else)?\s*\(|\.map\(\s*\|'
        r'|\.ok\(\)|\.and_then\(|\.unwrap_or\*')
    len_re = re.compile(r'\.len\(\)')
    chars_re = re.compile(r'\.chars\(\)\.count\(\)')

    panicky = [m for m, b in bodies if panic_re.search(b)]
    tolerant = [m for m, b in bodies if tolerant_re.search(b)]
    if panicky and tolerant:
        return ('ERR-DIVERGENCE',
                '一侧 panic（%s）另一侧宽容（%s）⇒ 同一操作两种失败语义'
                % (', '.join(panicky[:2]), ', '.join(tolerant[:2])))

    bytelen = [m for m, b in bodies if len_re.search(b)]
    charlen = [m for m, b in bodies if chars_re.search(b)]
    if bytelen and charlen:
        return ('UNIT-DIVERGENCE',
                '一侧用字节 .len()（%s）另一侧用 .chars().count()（%s）'
                '⇒ 单位混用，比值类计算会偏'
                % (', '.join(bytelen[:2]), ', '.join(charlen[:2])))
    return None, None


def report(args):
    db = args.db or os.path.join(REPO, '.project-map/edges-all.jsonl')
    if not os.path.isfile(db):
        sys.stderr.write('no edge db at %s — generate it with '
                         'nt_calledges.py --crate <dir> --out <f> '
                         '(gitignored low-frequency artifact)\n' % db)
        return 2
    cands = discover(db)
    verdicts = {IDENTICAL: [], DIFFERENT: [], UNRESOLVED: []}
    for leaf in sorted(cands, key=lambda s: -len(cands[s])):
        v, detail, _, rows = compare_pair(leaf, cands[leaf])
        shape = why = None
        if v == DIFFERENT:
            shape, why = classify_drift(rows)
        verdicts[v].append({'leaf': leaf, 'detail': detail, 'rows': rows,
                            'shape': shape, 'why': why})

    shaped = [i for i in verdicts[DIFFERENT] if i['shape']]
    unshaped = [i for i in verdicts[DIFFERENT] if not i['shape']]

    if args.only_different:
        shown = shaped + unshaped + verdicts[UNRESOLVED]
    else:
        shown = shaped + unshaped + verdicts[IDENTICAL] + verdicts[UNRESOLVED]

    print('[fn-drift] duplicate leaf names: %d | pairs: %d'
          % (len(cands), sum(len(v) * (len(v) - 1) // 2 for v in cands.values())))
    for v in (DIFFERENT, UNRESOLVED, IDENTICAL):
        print('  %-11s %d name(s)' % (v.lower(), len(verdicts[v])))
    print('  %-11s %d name(s)  ← 命中已实证的缺陷形状，优先分诊'
          % ('shaped', len(shaped)))

    print()
    for item in shown[:args.limit] if args.limit else shown:
        leaf = item['leaf']
        tag = ('  [' + item['shape'] + ']') if item['shape'] else ''
        if item['detail'] is None and item['rows'] and item['rows'][0][2] is None:
            print('  ?? %s  (%d modules, 定义解析不出：%s)'
                  % (leaf, len(item['rows']), item['rows'][0][1]))
            continue
        if item['detail'] is None:
            print('  == %s  (%d modules, 归一化后逐字相同 ⇒ 纯重复可合并)'
                  % (leaf, len(item['rows'])))
            for mod, path, _, line in item['rows']:
                print('       %s:%d  %s' % (path, line, mod))
            continue
        (mod_a, path_a, body_a, line_a), (mod_b, path_b, body_b, line_b), d_a, d_b = item['detail']
        print('  != %s  (%d modules, 语义不同 ⇒ 需人工裁决)%s'
              % (leaf, len(item['rows']), tag))
        if item['why']:
            print('       ⚠ %s' % item['why'])
        print('       A %s:%d  %s' % (path_a, line_a, mod_a))
        print('         …%s…' % d_a)
        print('       B %s:%d  %s' % (path_b, line_b, mod_b))
        print('         …%s…' % d_b)
        for mod, path, _, line in item['rows']:
            if path not in (path_a, path_b):
                print('       · %s:%d  %s' % (path, line, mod))

    print()
    print('[fn-drift] 边界：只能发现**同名**副本。跨名漂移（本会话的字节/char '
          'levenshtein）抓不到，只能靠 nt_parity_ref 的行为对位。')
    print('[fn-drift] 本工具不是门：判据是文本相等，不足以当正确性判据。')
    return 0


def selftest(_args):
    fails = []

    def check(cond, msg):
        if not cond:
            fails.append(msg)

    # 1) 字符串里带花括号，naive 计数会被骗（证伪用例）
    tricky = '''
fn probe(x: usize) -> usize {
    let s = "}}}  {{{";
    if x > 0 { return s.len() + x; }
    x
}
'''
    body, line = extract_fn(tricky, 'probe')
    check(body is not None, 'tricky: no body extracted')
    check(body is not None and body.rstrip().endswith('x'),
          'tricky: body truncated by a brace inside a string: %r' % (body,))
    check(line > 0, 'tricky: bad line number')

    # 2) 注释里的花括号同上
    commented = '''
fn probe2(x: usize) -> usize {
    // }}} not a real close
    /* {{{ also not */
    x
}
'''
    b2, _ = extract_fn(commented, 'probe2')
    check(b2 is not None and 'not a real close' not in b2,
          'commented: comment not stripped / body wrong: %r' % (b2,))

    # 3) 同名函数体相同 ⇒ IDENTICAL（纯重复）
    a = 'fn calc(a: u32, b: u32) -> u32 { a + b }'
    b = 'fn calc(a: u32, b: u32) -> u32 {\n    a  +  b\n}'
    ba, _ = extract_fn(a, 'calc')
    bb, _ = extract_fn(b, 'calc')
    check(ba is not None and ba == bb,
          'whitespace-normalised identical bodies not equal: %r vs %r' % (ba, bb))

    # 4) 注释差异不算差异（有意为之：注释不是语义）
    c1 = 'fn calc2(a: u32) -> u32 { /* 中文说明 */ a }'
    c2 = 'fn calc2(a: u32) -> u32 { a }'
    d1, _ = extract_fn(c1, 'calc2')
    d2, _ = extract_fn(c2, 'calc2')
    check(d1 == d2, 'comment difference must not count as semantic difference')

    # 5) 真实差异必须被看出 + 首差定位
    e1 = 'fn calc3(a: u32) -> u32 { a + 1 }'
    e2 = 'fn calc3(a: u32) -> u32 { a + 2 }'
    f1, _ = extract_fn(e1, 'calc3')
    f2, _ = extract_fn(e2, 'calc3')
    check(f1 != f2, 'real difference not detected')
    d_a, d_b = first_diff(f1, f2)
    check(d_a is not None and ('1' in d_a or '2' in d_a),
          'first_diff did not localise: %r / %r' % (d_a, d_b))

    # 6) lifetime 不是 char literal（否则 'a 会被吃掉，函数体解析全乱）
    lt = "fn probe4<'a>(x: &'a str) -> &'a str { x }"
    b6, _ = extract_fn(lt, 'probe4')
    check(b6 is not None and 'x' in b6, 'lifetime handled as char literal: %r' % (b6,))

    # 7) 只有声明没有定义 ⇒ 抽不出（不猜）
    decl_only = 'pub fn declared(a: u32) -> u32;'
    b7, _ = extract_fn(decl_only, 'declared')
    check(b7 is None, 'declaration-only must not yield a body: %r' % (b7,))

    if fails:
        sys.stderr.write('selftest FAILED (%d):\n' % len(fails))
        for f in fails:
            sys.stderr.write('  - %s\n' % f)
        return 1
    sys.stdout.write('selftest PASS: 7 例（含 3 例证伪：字符串/注释花括号、'
                     'lifetime、声明-only）\n')
    return 0


def selftest_units(_args):
    """对**同名函数族的单位一致性**给出可复跑判据。

    为什么需要它（实测动因）：fn-drift 只会说「这 7 个名字命中了
    UNIT-DIVERGENCE 形状」，但**不裁决哪个对**。要裁决就必须能对同一族副本
    逐个分类「按字节计数 / 按字符计数 / 混合」，否则分诊只能靠人眼读 7×N 份源码。
    ⇒ 本子命令给出该分类，并让调用方对**具体某一份**取判定。

    判据（全部可机械核对，不含语义猜测）：
      BYTE   体内有 `.len()` 且无 `.chars()`
      CHAR   体内有 `.chars()`
      MIXED  两者都有（如 `max(chars/4, words)` 里 chars 其实是字节）
      NONE   两者都没有（无法归类 ⇒ 不猜）

    ⚠️ **已知不可靠处**：`.len()` 也可能是「集合长度」而非字符串字节数。
    因此 BYTE 判定只作**候选**，必须读那一行确认（R-SCAN-1b）。
    这与本工具整体定位一致：输出是分诊单，不是判决。
    """
    db = _args.db or os.path.join(REPO, '.project-map/edges-all.jsonl')
    if not os.path.isfile(db):
        sys.stderr.write('no edge db at %s\n' % db)
        return 2
    cands = discover(db)
    wanted = _args.units or []
    if not wanted:
        sys.stderr.write('--units is required, e.g. --units estimate_tokens '
                         '--units truncate\n(list candidates with --list-units)\n')
        return 2

    rc = 0
    for leaf in wanted:
        mods = cands.get(leaf)
        if not mods:
            sys.stderr.write('%-22s no duplicate copies found (nothing to '
                             'compare — that is a fact, not a pass)\n' % leaf)
            rc = 2
            continue
        tally = defaultdict(list)
        for mod, path in sorted(mods.items()):
            src = read(path)
            body, line = extract_fn(src, leaf) if src else (None, 0)
            if body is None:
                tally['UNPARSED'].append('%s:%d  %s' % (path, line, mod))
                continue
            byt = '.len()' in body
            chr_ = '.chars()' in body
            kind = 'MIXED' if (byt and chr_) else ('CHAR' if chr_ else
                                                   ('BYTE' if byt else 'NONE'))
            tally[kind].append('%s:%d  %s' % (path, line, mod))
        print('== %s  (%d copies)' % (leaf, len(mods)))
        for kind in ('BYTE', 'CHAR', 'MIXED', 'NONE', 'UNPARSED'):
            if not tally[kind]:
                continue
            print('   %-9s %d' % (kind, len(tally[kind])))
            for loc in tally[kind]:
                print('        %s' % loc)
        if 'BYTE' in tally and 'CHAR' in tally:
            print('   ⚠ 同一族同时存在 BYTE 与 CHAR ⇒ 判定输入相同时结果不同。'
                  'BYTE 侧逐个读源码确认后再改（本工具只给候选）。')
    return rc


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[1])
    ap.add_argument('--db', default=None)
    ap.add_argument('--limit', type=int, default=40)
    ap.add_argument('--only-different', action='store_true')
    ap.add_argument('--units', action='append', default=[],
                    help='classify every copy of this function name by counting unit')
    ap.add_argument('--list-units', action='store_true',
                    help='list candidate duplicate names worth --units triage')
    ap.add_argument('cmd_positional', nargs='?', default=None)
    args = ap.parse_args(argv[1:])
    if args.cmd_positional == 'selftest':
        return selftest(args)
    if args.units:
        return selftest_units(args)
    if args.list_units:
        db = args.db or os.path.join(REPO, '.project-map/edges-all.jsonl')
        if not os.path.isfile(db):
            sys.stderr.write('no edge db at %s\n' % db)
            return 2
        cands = discover(db)
        rows = []
        for leaf, mods in cands.items():
            v, _, _, r = compare_pair(leaf, mods)
            if v != DIFFERENT:
                continue
            shape, _why = classify_drift(r)
            if shape == 'UNIT-DIVERGENCE':
                rows.append((len(mods), leaf))
        for n, leaf in sorted(rows, reverse=True):
            print('%-24s %d copies' % (leaf, n))
        return 0
    return report(args)


if __name__ == '__main__':
    sys.exit(main(sys.argv))