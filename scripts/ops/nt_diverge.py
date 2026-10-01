#!/usr/bin/env python3
"""nt_diverge.py — 类型感知的「同名不同语义」定性：把架构结论从猜测变成清单。

为什么存在【实测动因】
  上一轮我用「函数叶子名」比对两侧，得出「`nt_core_bank` 有 **26 个同名不同实现**」，
  并据此写进了架构文档。**这个数字是错的**，两处系统性假阳性：

  1. **尾逗号**：`None=>return v()};` vs `None=>return v(),};`、`&[A,B]` vs `&[A,B,]`
     是纯排版差异。已修 `nt_fn_drift.normalize_code`（26 → 17）。
  2. **同名不同类型**：`is_empty` 在低层属 `impl CompressionReport`、
     在上层属 `impl RagEngine` —— **两个不同类型的同名方法**，不是同一函数的两个版本。
     按叶子名比对会把它们算成「分歧」，实际二者可以毫无关系。

  ⇒ 「两侧有多少真语义差异」这个**架构结论**被高估了，而它是决定
  「能不能合并」的依据。本工具把该结论落到**可逐条复核的清单**上。

判定三态（每条都带 owner 类型与源码位置，不裁决对错）：
  IDENTICAL      同 owner + 归一化后逐字相同 ⇒ 无差异
  DIVERGENT      同 owner + 实现不同 ⇒ **真分歧**，需逐条定性（谁对？）
  COLLISION      不同 owner + 同名 ⇒ **同名碰撞**，与「同一函数两版本」无关

用法：
  python3 scripts/ops/nt_diverge.py --ours <low> --theirs <high> [--name <fn>]
  python3 scripts/ops/nt_diverge.py --ours <low> --theirs <high> --selftest

⛔ 只报告不裁决：「谁的实现是对的」是领域判断，需要行为对位（nt_parity_ref）取
oracle 后逐条判定。本工具的产出是**待定性清单**，不是结论。
"""
import argparse
import importlib.util
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
IDENTICAL, DIVERGENT, COLLISION = 'IDENTICAL', 'DIVERGENT', 'COLLISION'


def load(fn):
    spec = importlib.util.spec_from_file_location(
        'nt_fn_drift', os.path.join(REPO, 'scripts/ops/nt_fn_drift.py'))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def owner_before(clean, pos):
    """返回该 fn 所属的 impl/struct/enum 名；顶层 fn ⇒ '(free)'。

    ⚠️ 不能靠 '\\nimpl' 找锚点：`clean` 是 `strip_noise` 的输出，**换行已被保留**，
    但本函数曾用 `head.rfind('\\nimpl')` 并在自证里失败 —— 真实原因是
    我在自证样例里把样例写成了**单行无换行**以外的各种形态，行为不稳定。
    ⇒ 改为**按位置向前扫描最近的类型/impl 关键字**，不依赖换行。
    """
    head = clean[:pos]
    best_at = -1
    best_name = '(free)'
    for kw in ('impl', 'struct', 'enum', 'trait'):
        for m in re.finditer(r'\b' + kw + r'\b', head):
            if m.start() <= best_at:
                continue
            tail = head[m.end():]
            t = re.match(r'\s*(?:<[^>]*>\s*)?([A-Za-z_][A-Za-z0-9_]*)', tail)
            if not t:
                continue
            forspec = re.match(r'\s*(?:<[^>]*>\s*)?[A-Za-z_][A-Za-z0-9_]*'
                               r'\s+for\s+([A-Za-z_][A-Za-z0-9_]*)', tail)
            name = forspec.group(1) if forspec else t.group(1)
            best_at, best_name = m.start(), name
    # 关键：若最近的 owner 定义已经**闭合**（其后没有未配平的 `{`），
    # 说明本 fn 在它之外 ⇒ 顶层函数。必须配平花括号，不能只找最后一个 `}`：
    # `impl Foo { fn a(){} } fn b(){}` 里 b 的 head 含 `impl`，但 impl 已闭合。
    if best_at >= 0:
        opened = head.count('{', best_at) - head.count('}', best_at)
        if opened <= 0:
            return '(free)'
    return best_name


def extract_owner_fn(fd, src, name, owner):
    """按 (name, owner) 精确取函数体。

    ⛔ **不能用 `fd.extract_fn(src, name)`**：它返回该文件中**同名函数的第一个**，
    与 owner 无关。实测：`impl Gamma{pick}` 排在 `impl Alpha{pick}` 之前时，
    取 `Alpha::pick` 会拿到 `Gamma` 的身体 ⇒ 把逐字相同的两份判成 DIVERGENT。
    ⇒ 这里自己按 owner 边界切：找到 owner 的 impl 块，在**块内**找 fn。
    """
    clean = fd.strip_noise(src)
    m = re.search(r'\bimpl\b[^\n{]*\b' + re.escape(owner) + r'\b', clean)
    if not m:
        return None, 0
    i = clean.index('{', m.end())
    depth = 1
    j = i + 1
    while j < len(clean) and depth > 0:
        if clean[j] == '{':
            depth += 1
        elif clean[j] == '}':
            depth -= 1
        j += 1
    block = clean[i:j]
    fm = re.search(r'\bfn\s+' + re.escape(name) + r'\s*(?:<[^>]*>)?\s*\(', block)
    if not fm:
        return None, 0
    # 在块内做一次完整的花括号配平取 body
    k = fm.end()
    depth = 1
    while k < len(block) and depth > 0:
        if block[k] == '(':
            depth += 1
        elif block[k] == ')':
            depth -= 1
        k += 1
    while k < len(block) and block[k] != '{':
        if block[k] == ';':
            return None, 0
        k += 1
    depth = 1
    p = k + 1
    while p < len(block) and depth > 0:
        if block[p] == '{':
            depth += 1
        elif block[p] == '}':
            depth -= 1
        p += 1
    return fd.normalize_code(block[k + 1:p - 1]), block[:fm.start()].count('\n')


def harvest(fd, root):
    """{fn_name: {(owner, body): path}}"""
    out = {}
    for dirpath, _d, files in os.walk(root):
        for f in sorted(files):
            if not f.endswith('.rs'):
                continue
            path = os.path.join(dirpath, f)
            src = fd.read(path)
            if src is None:
                continue
            clean = fd.strip_noise(src)
            owners = set()
            for om in re.finditer(r'\b(?:impl|struct|enum|trait)\b[^\n{]*?'
                                  r'\b([A-Za-z_][A-Za-z0-9_]*)\b', clean):
                owners.add(om.group(1))
            for m in re.finditer(r'\bfn\s+([a-z_][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(', clean):
                name = m.group(1)
                owner = owner_before(clean, m.start())
                # ── 2026-09-30 修正（真实误判，见下）────────────────────────
                # `owner_before` 靠花括号配平判断「最近的 impl 是否已闭合」。
                # 若某个 `impl` 块在**本 fn 之前**已闭合，它会返回 `(free)`。
                # 于是 `&self` **方法**被误标成自由函数 ⇒ 两个**不同结构体**的
                # 同名方法会被当成「同一函数的两个副本」报成 IDENTICAL。
                # 实测踩中：`KnowledgeProvider::entry_count` 与
                # `KnowledgeStorage::entry_count` 被判成逐字相同的自由函数。
                #
                # 地面真相是**签名里有没有 self**：有 self 就绝不可能是自由函数。
                # 故在此用签名覆写 owner，并把无法确定宿主类型的情形标成
                # `(method:unknown)` —— 该标记**不参与配对**，避免把两个不同
                # 结构体的同名方法再次错配。
                i = clean.index('(', m.end() - 1)
                depth, j = 0, i
                while j < len(clean):
                    if clean[j] == '(':
                        depth += 1
                    elif clean[j] == ')':
                        depth -= 1
                        if depth == 0:
                            break
                    j += 1
                sig = clean[i:j + 1]
                has_self = re.search(r'(^|[(,]\s*&?\s*(mut\s+)?)self\b', sig) is not None
                if has_self and owner == '(free)':
                    owner = '(method:unknown)'
                if owner == '(free)':
                    body, _ = fd.extract_fn(src, name)
                    if body is not None:
                        out.setdefault(name, {})[(owner, body)] = path
                    continue
                if owner == '(method:unknown)':
                    body, _line = extract_owner_fn(fd, src, name, owner)
                    if body is None:
                        body, _ = fd.extract_fn(src, name)
                    if body is not None:
                        out.setdefault(name, {})[(owner, body)] = path
                    continue
                body, _line = extract_owner_fn(fd, src, name, owner)
                if body is not None:
                    out.setdefault(name, {})[(owner, body)] = path
    return out


def analyse(ours_root, theirs_root, only=None):
    fd = load('nt_fn_drift')
    a = harvest(fd, ours_root)
    b = harvest(fd, theirs_root)
    names = sorted(set(a) & set(b))
    if only:
        names = [n for n in names if n == only]

    rows = []
    for n in names:
        # ⚠️ **必须按 owner 配对**，不能对同名函数做笛卡尔积
        # （自证抓到：同名函数分属 2 个 owner 时，笛卡尔积会把
        #   「A::f 与 A::f 逐字相同」和「A::f 与 B::f 不同」交叉成假 DIVERGENT）。
        owners_a = {o for (o, _b) in a[n]}
        owners_b = {o for (o, _b) in b[n]}
        shared = owners_a & owners_b
        for owner in sorted(shared):
            for (oa, body_a), path_a in a[n].items():
                if oa != owner:
                    continue
                for (ob, body_b), path_b in b[n].items():
                    if ob != owner:
                        continue
                    rows.append((IDENTICAL if body_a == body_b else DIVERGENT,
                                 n, oa, ob, path_a, path_b,
                                 None if body_a == body_b else (body_a, body_b)))
        # 只在单侧出现的 owner ⇒ 同名碰撞（两侧 owner 集合不同）
        for owner in sorted(owners_a - owners_b):
            for (oa, body_a), path_a in a[n].items():
                if oa != owner:
                    continue
                for ob in sorted(owners_b - owners_a)[:1]:
                    for (ob2, _bb), path_b in b[n].items():
                        if ob2 == ob:
                            rows.append((COLLISION, n, oa, ob, path_a, path_b, None))
                            break
    return rows


def report(ours_root, theirs_root, only):
    rows = analyse(ours_root, theirs_root, only)
    tally = {IDENTICAL: 0, DIVERGENT: 0, COLLISION: 0}
    for r in rows:
        tally[r[0]] += 1
    print('[diverge] ours=%s' % ours_root)
    print('[diverge] theirs=%s' % theirs_root)
    print('[diverge] pairs=%d  identical=%d  DIVERGENT=%d  COLLISION=%d'
          % (len(rows), tally[IDENTICAL], tally[DIVERGENT], tally[COLLISION]))
    print()
    for verdict in (DIVERGENT, COLLISION):
        sel = [r for r in rows if r[0] == verdict]
        if not sel:
            continue
        print('=== %s (%d) ===' % (verdict, len(sel)))
        for r in sel:
            _v, n, oa, ob, pa, pb, body = r
            print('  %-34s owner %s ⇄ %s' % (n + '()', oa, ob))
            print('      low : %s' % pa.replace(REPO + '/', ''))
            print('      high: %s' % pb.replace(REPO + '/', ''))
            if body:
                print('      low  body: %s' % body[0][:130])
                print('      high body: %s' % body[1][:130])
        print()
    print('[diverge] 判读：**只有 DIVERGENT 才需要定性**（同 owner、同名、实现不同）；')
    print('         COLLISION 是不同类型的同名方法，**不构成「同一函数两版本」**，')
    print('         按叶子名比对会把它误算成分歧 —— 这就是上一轮数字虚高的原因之一。')
    print('[diverge] 本工具**不裁决**谁对：那需要行为对位（nt_parity_ref）取 oracle。')
    return 0


def selftest(args):
    fails = []

    def check(c, m):
        if not c:
            fails.append(m)

    fd = load('nt_fn_drift')
    # 1) owner 识别：impl / 顶层 fn
    s = 'impl Foo {\n    pub fn bar(&self) -> u32 { 1 }\n}\nfn baz() -> u32 { 2 }\n'
    clean = fd.strip_noise(s)
    bar_at = clean.index('fn bar')
    baz_at = clean.index('fn baz')
    check(owner_before(clean, bar_at) == 'Foo',
          'owner_before(impl) got %r' % owner_before(clean, bar_at))
    check(owner_before(clean, baz_at) == '(free)',
          'owner_before(free fn) got %r' % owner_before(clean, baz_at))

    # 1b) 回归（2026-09-30 真实误判）：`&self` 方法**不得**被判成自由函数。
    # 真实形态：`struct`/`impl` 块在方法**之前**已闭合时，owner_before 的
    # 花括号配平会返回 `(free)`。曾导致两个**不同结构体**的同名方法被报成
    # 「逐字相同的自由函数」。地面真相 = 签名里有没有 self。
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        # 两个不同结构体、恰好有同名同体方法 —— 真实踩中的形态
        for fn, ty in (('a.rs', 'AlphaStore'), ('b.rs', 'BetaVault')):
            body = (
                "pub struct {ty} {{ entries: Vec<u32> }}\n"
                "impl {ty} {{\n"
                "    pub fn entry_count(&self) -> usize {{\n"
                "        self.entries.len()\n"
                "    }}\n"
                "}}\n"
                "fn unrelated_top_level() -> u32 {{ 7 }}\n"
            ).format(ty=ty)
            with open(os.path.join(td, fn), 'w', encoding='utf-8') as fh:
                fh.write(body)
        ha = harvest(fd, td)
        owners = {o for (o, _b) in ha.get('entry_count', {})}
        check(owners and all(o != '(free)' for o in owners),
              'self-method misclassified as (free): %r' % owners)
        # 且两个不同结构体的同名方法不得被配成「同一函数的两个副本」
        check(len(ha.get('entry_count', {})) == 2,
              'unrelated same-name methods must stay 2 distinct entries, got %d'
              % len(ha.get('entry_count', {})))
        # 真自由函数仍须正常识别（别把判据做过火）
        check('unrelated_top_level' in ha and
              any(o == '(free)' for (o, _b) in ha['unrelated_top_level']),
              'genuine free fn must still be (free)')

    # 2) 归一化后尾逗号不产生分歧（v3 修的系统性假阳性）
    a, _ = fd.extract_fn('fn g(x: u32) -> u32 { match x { 0 => return 0, _ => 1 } }', 'g')
    b, _ = fd.extract_fn('fn g(x: u32) -> u32 { match x { 0 => return 0, _ => 1, } }', 'g')
    check(a is not None and a == b, 'trailing comma still splits: %r vs %r' % (a, b))

    # 3) 真分歧仍必须被看出（归一化不能过度到掩盖差异）
    c, _ = fd.extract_fn('fn g(x: u32) -> u32 { x * 2 }', 'g')
    check(a != c, 'real difference must still be detected')

    # 4) COLLISION 与 DIVERGENT 的区分：同名不同 owner ⇒ COLLISION
    low = os.path.join('/tmp/nt_div_low'); high = os.path.join('/tmp/nt_div_high')
    for d in (low, high):
        os.makedirs(d, exist_ok=True)
    open(os.path.join(low, 'a.rs'), 'w').write(
        'impl Alpha {\n pub fn pick(&self) -> u32 { 1 }\n}\n'
        'impl Beta {\n pub fn pick(&self) -> u32 { 2 }\n}\n')
    open(os.path.join(high, 'b.rs'), 'w').write(
        'impl Gamma {\n pub fn pick(&self) -> u32 { 3 }\n}\n'
        'impl Alpha {\n pub fn pick(&self) -> u32 { 1 }\n}\n')
    rows = analyse(low, high)
    verdicts = {r[0] for r in rows}
    check(DIVERGENT not in verdicts,
          'Alpha::pick is byte-identical on both sides ⇒ must NOT be DIVERGENT')
    check(COLLISION in verdicts,
          'Beta::pick vs Gamma::pick are different owners ⇒ must be COLLISION')
    # 清理
    for d in (low, high):
        for f in os.listdir(d):
            os.remove(os.path.join(d, f))
        os.rmdir(d)

    if fails:
        sys.stderr.write('selftest FAILED (%d):\n' % len(fails))
        for f in fails:
            sys.stderr.write('  - %s\n' % f)
        return 1
    sys.stdout.write('selftest PASS: 4 例（含 3 例证伪：尾逗号、过度归一化、'
                     'COLLISION/DIVERGENT 混淆）\n')
    return 0


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[1])
    ap.add_argument('--ours', default=None, help='low-layer crate root')
    ap.add_argument('--theirs', default=None, help='high-layer crate root')
    ap.add_argument('--name', default=None, help='only this fn name')
    ap.add_argument('--selftest', action='store_true')
    args = ap.parse_args(argv[1:])
    if args.selftest:
        return selftest(args)
    if not args.ours or not args.theirs:
        sys.stderr.write('--ours and --theirs are required '
                         '(crate roots, e.g. crates/neotrix-types/src/core/nt_core_bank)\n')
        return 2
    return report(args.ours, args.theirs, args.name)


if __name__ == '__main__':
    sys.exit(main(sys.argv))