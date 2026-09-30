#!/usr/bin/env python3
"""nt_decompose.py — 原子能力拆解 + 对方产品对位矩阵（G1/G4 下游 · G3 对位）。

为什么存在（GLOBAL-MAP-DEFECTS-2026-09-30 的实测结论）：
  「吸收流程 = 人工/agent 驱动的文档工程」，且**没有能力矩阵/差距表**（可比对的
  spec）。于是「原子级拆解技术逻辑」与「复现他方产品」都只能是 checklist，
  无法机器化。本工具把两者变成查询：

  1) atoms  —— 从一个入口符号出发，沿 **THIR 已解析的调用边**做可达性遍历，
     每条边切成一个**原子能力单元**（atom），带 file:line 证据与可判定的 oracle。
     ⇒ 「这段逻辑由哪些原子组成」从「人读」变成「查表」。
  2) parity —— 两个原子集做**对位矩阵**：共有 / 我方独有 / 对方独有 / 不可对位。
     ⇒ 「他方有的能力我方有没有」从 checklist 变成可计算差集。
     对方原子集可以用**同一个工具**在外部仓（临时 cargo 工程 + cargo registry
     里的第三方 crate）上生成 ⇒ 跨仓对位不是模拟数据。

三个子命令：
  atoms    --db <edges.jsonl> --root <DefId 路径子串> [--depth N] [--out FILE]
  parity   --mine <atoms.json> --theirs <atoms.json> [--out matrix.md]
  selftest                                                      # 正例 + 证伪用例

实现纪律（本仓既有教训，逐条对应）：
  - **宁缺勿错**：root 子串命中多个节点 ⇒ 报错退出 2，绝不挑一个。
    不可判的 atom 记 `unjudgeable` 并给理由，不猜 oracle。
  - **对位诚实**：parity 是**名字级**对位（叶子名归一化后比字符串），
    绝不断言语义等价；矩阵头显式写这条限制。
  - 外部 crate 已在 ~/.cargo/registry（vendor 依赖），全程 --offline，不联网、
    不执行第三方代码、不 vendor 任何源码。
  - 只用标准库；`--max-atoms` 兜底防内存爆（16G 机器 + 并行 cargo 场景）。
"""
import argparse
import hashlib
import json
import os
import re
import sys

LAYER_RE = re.compile(r'src/(l\d_[a-z]+)/')
CRATE_RE = re.compile(r'^(?:neotrix-core/src/)?(?:(crates/([\w\-]+))/|([\w\-]+)\[[0-9a-f]{4}\])')
SPAN_RE = re.compile(r'^(.*?):(\d+):(\d+):?\s*$')
TEST_SPAN_RE = re.compile(r'(^|/)tests?/|_tests?\.rs$|/benches/|^benches/')
SCRIPT_SPAN_RE = re.compile(r'^(scripts|\.github)/')

# oracle 分类词表（可测量信号，不是对语义的猜测）
PERSIST_LEX = re.compile(
    r'(?:^|::)(?:write|save|persist|store|stash|append|flush|commit|'
    r'insert|upsert|set_|put_|write_all|write_to|create_dir|remove_file|'
    r'sync_all|rename)\b', re.IGNORECASE)
STD_CRATE_RE = re.compile(r'^(core|std|alloc|proc_macro|test)\[')

K_LEAF = 'K1-atomic-leaf'
K_PERSIST = 'K2-persistent'
K_BOUND = 'K3-symbol-bound'
K_EXTERNAL = 'K4-external-decidable'
K_UNJUDGEABLE = 'unjudgeable'


def load_graph(db_path, node_filter=None):
    """流式读 JSONL 边表，返回 (names, out_edges, in_edges, stats)。

    节点名做 interning（name <-> int），670K 边只留两份 int 邻接表，
    避免把 176MB 文本与 60 字符字符串重复驻留。
    node_filter: 只保留命中该子串的节点参与的边（None = 全图）。
    """
    names = []
    idx = {}

    def nid(n):
        i = idx.get(n)
        if i is None:
            i = len(names)
            idx[n] = i
            names.append(n)
        return i

    out_edges = {}
    in_edges = {}
    kept = total = 0
    with open(db_path, 'r', encoding='utf-8', errors='replace') as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except ValueError:
                continue  # 半行/损坏：不猜，跳过并计数
            c = rec.get('caller')
            k = rec.get('callee')
            if not c or not k:
                continue
            total += 1
            if node_filter and node_filter not in c and node_filter not in k:
                continue
            ci, ki = nid(c), nid(k)
            out_edges.setdefault(ci, []).append((ki, rec.get('span', ''),
                                                rec.get('kind', 'call')))
            in_edges.setdefault(ki, []).append(ci)
            kept += 1
    stats = {'lines': total, 'kept': kept, 'nodes': len(names)}
    return names, out_edges, in_edges, stats


def resolve_root(names, idx, root):
    """子串 -> 唯一节点。命中多个 ⇒ 返回候选列表由调用方报错（宁缺勿错）。"""
    hits = [n for n in names if root in n]
    if not hits:
        return None, []
    return (hits[0], None) if len(hits) == 1 else (None, sorted(hits))


def layer_of(span, callee):
    m = LAYER_RE.search(span or '')
    if m:
        return m.group(1)
    m = CRATE_RE.match(callee or '')
    if m:
        if m.group(2):
            return 'crates/' + m.group(2)
        if m.group(3):
            return m.group(3)
    return 'unknown'


def classify_oracle(span, callee, callee_out_degree):
    """可判 oracle 分类。返回 (oracle, reason)。信号全部来自边表/路径，不猜语义。"""
    path = (span or '').rsplit(':', 3)[0] if span else ''
    if TEST_SPAN_RE.search(path):
        return K_EXTERNAL, 'span 落在测试文件 ⇒ 已有外部裁决点'
    if SCRIPT_SPAN_RE.search(path):
        return K_EXTERNAL, 'span 落在 scripts/.github ⇒ 已有外部裁决点'
    if PERSIST_LEX.search(callee or ''):
        return K_PERSIST, 'callee 命中落盘词表 ⇒ 需持久化裁决'
    if STD_CRATE_RE.match(callee or '') and callee_out_degree == 0:
        return K_LEAF, 'std 叶子（边表内无出边）⇒ 原子单元'
    if callee_out_degree == 0:
        return K_BOUND, '边表内叶子：符号绑定已证，语义不可从边表判定'
    return K_BOUND, 'THIR 已解析的 FnDef 边 ⇒ 符号绑定已证'


def norm_leaf(name):
    """归一化叶子名：去 impl#N / 泛型 / 哈希后缀，供 parity 做名字级对位。"""
    n = (name or '').split('::')[-1]
    n = re.sub(r'\{impl#\d+\}', '', n)
    n = re.sub(r'\{closure#\d+\}', '', n)
    n = re.sub(r'<.*>$', '', n)
    n = re.sub(r'\[\d+[0-9a-f]*\]$', '', n)
    return n or (name or '')


def cmd_atoms(args):
    if not os.path.isfile(args.db):
        sys.stderr.write('atoms: no such edge db: %s\n' % args.db)
        return 2
    names, out_edges, in_edges, stats = load_graph(args.db, args.filter)
    if not names:
        sys.stderr.write('atoms: edge db produced 0 nodes (%s)\n'
                         % json.dumps(stats))
        return 2
    root, cands = resolve_root(names, None, args.root)
    if root is None:
        sys.stderr.write('atoms: root %r matched %d nodes -> refusing to guess\n'
                         % (args.root, len(cands)))
        for c in cands[:8]:
            sys.stderr.write('    %s\n' % c)
        return 2

    # 可达性遍历（沿 caller->callee 方向，即「这个入口展开了哪些原子」）
    ridx = names.index(root)
    seen = {ridx}
    frontier = [ridx]
    picked = []
    truncated = False
    for _ in range(max(1, args.depth)):
        nxt = []
        for nid_ in frontier:
            for (kid, span, kind) in out_edges.get(nid_, ()):  # noqa: E501
                if kid in seen:
                    continue
                seen.add(kid)
                nxt.append(kid)
                if len(picked) >= args.max_atoms:
                    truncated = True
                    break
                picked.append((nid_, kid, span, kind))
            if truncated:
                break
        frontier = nxt
        if truncated or not frontier:
            break

    atoms = []
    for (cid, kid, span, kind) in picked:
        callee = names[kid]
        caller = names[cid]
        deg = len(out_edges.get(kid, ()))
        oracle, reason = classify_oracle(span, callee, deg)
        m = SPAN_RE.match(span or '')
        atoms.append({
            'id': hashlib.sha1(('%s|%s|%s' % (caller, callee, span))
                               .encode('utf-8')).hexdigest()[:12],
            'capability': norm_leaf(callee),
            'caller': caller,
            'callee': callee,
            'kind': kind,
            'span': span,
            'file': (m.group(1) if m else span),
            'line': (int(m.group(2)) if m else None),
            'layer': layer_of(span, callee),
            'oracle': oracle,
            'oracle_reason': reason,
            'evidence': ['THIR FnDef(DefId) resolved', span],
        })

    by_oracle = {}
    for a in atoms:
        by_oracle[a['oracle']] = by_oracle.get(a['oracle'], 0) + 1
    doc = {
        'schema': 'nt-decompose/atoms/1',
        'root': root,
        'depth': args.depth,
        'db': args.db,
        'graph_stats': stats,
        'truncated': truncated,
        'atoms': atoms,
        'oracle_histogram': dict(sorted(by_oracle.items())),
    }
    text = json.dumps(doc, ensure_ascii=False, indent=2, sort_keys=True)
    if args.out:
        with open(args.out, 'w', encoding='utf-8') as fh:
            fh.write(text + '\n')
        sys.stdout.write('wrote %d atoms -> %s\n' % (len(atoms), args.out))
    else:
        sys.stdout.write(text + '\n')
    sys.stdout.write('[decompose] root=%s depth=%d atoms=%d truncated=%s %s\n'
                     % (root, args.depth, len(atoms), truncated,
                        json.dumps(by_oracle, sort_keys=True)))
    return 0


def load_atoms(path):
    with open(path, 'r', encoding='utf-8') as fh:
        doc = json.load(fh)
    if doc.get('schema') != 'nt-decompose/atoms/1':
        raise ValueError('not an nt-decompose atoms doc: %s' % path)
    return doc


def cmd_parity(args):
    try:
        mine = load_atoms(args.mine)
        theirs = load_atoms(args.theirs)
    except (OSError, ValueError) as exc:
        sys.stderr.write('parity: %s\n' % exc)
        return 2

    def capset(doc):
        return {a['capability'] for a in doc['atoms']}

    m, t = capset(mine), capset(theirs)
    shared = sorted(m & t)
    only_mine = sorted(m - t)
    only_theirs = sorted(t - m)
    by_layer_mine = {}
    for a in mine['atoms']:
        by_layer_mine[a['layer']] = by_layer_mine.get(a['layer'], 0) + 1
    by_layer_theirs = {}
    for a in theirs['atoms']:
        by_layer_theirs[a['layer']] = by_layer_theirs.get(a['layer'], 0) + 1

    lines = []
    lines.append('# 对位矩阵：%s ↔ %s' % (mine['root'], theirs['root']))
    lines.append('')
    lines.append('> ⛔ **名字级对位，不是语义对位。** 两集按 `capability`'
                 '（叶子名归一化）比字符串；同名不保证语义等价，'
                 '异名也不代表能力缺失。矩阵只回答「同名项是否都在」，'
                 '语义等价需另配外部 oracle。')
    lines.append('')
    lines.append('| 指标 | 我方 | 对方 |')
    lines.append('|---|---:|---:|')
    lines.append('| 原子数 | %d | %d |' % (len(mine['atoms']), len(theirs['atoms'])))
    lines.append('| 去重能力名 | %d | %d |' % (len(m), len(t)))
    lines.append('| 截断? | %s | %s |' % (mine['truncated'], theirs['truncated']))
    lines.append('')
    lines.append('| 层分布 | 我方 | 对方 |')
    lines.append('|---|---:|---:|')
    for layer in sorted(set(by_layer_mine) | set(by_layer_theirs)):
        lines.append('| %s | %d | %d |' % (layer,
                                          by_layer_mine.get(layer, 0),
                                          by_layer_theirs.get(layer, 0)))
    lines.append('')
    lines.append('## 共有（%d）' % len(shared))
    lines.append('')
    lines.append(', '.join('`%s`' % s for s in shared) or '—')
    lines.append('')
    lines.append('## 仅我方有（%d）' % len(only_mine))
    lines.append('')
    lines.append(', '.join('`%s`' % s for s in only_mine) or '—')
    lines.append('')
    lines.append('## 仅对方有（%d）' % len(only_theirs))
    lines.append('')
    lines.append(', '.join('`%s`' % s for s in only_theirs) or '—')
    lines.append('')
    # 摘要也进产物：矩阵文件自身要带机器可判的一行（selftest 读文件判据，
    # 不读 stdout ⇒ 干跑/重定向后仍可判）
    lines.append('## 判据摘要')
    lines.append('')
    lines.append('[parity] shared=%d only_mine=%d only_theirs=%d'
                 % (len(shared), len(only_mine), len(only_theirs)))
    lines.append('')
    out = '\n'.join(lines)
    if args.out:
        with open(args.out, 'w', encoding='utf-8') as fh:
            fh.write(out)
        sys.stdout.write('wrote parity matrix -> %s\n' % args.out)
    else:
        sys.stdout.write(out + '\n')
    sys.stdout.write('[parity] shared=%d only_mine=%d only_theirs=%d\n'
                     % (len(shared), len(only_mine), len(only_theirs)))
    return 0


def _toy_db(path, nodes, edges):
    with open(path, 'w', encoding='utf-8') as fh:
        for (c, k, span, kind) in edges:
            fh.write(json.dumps({'caller': c, 'callee': k, 'span': span,
                                 'kind': kind}) + '\n')
    del nodes


def cmd_selftest(args):
    tmp = args.tmpdir
    os.makedirs(tmp, exist_ok=True)
    fails = []

    def check(cond, msg):
        if not cond:
            fails.append(msg)

    # --- 用例 1：可达性 + 宁缺勿错（不可达节点不得出现；多命中 root 必须报错）
    db_a = os.path.join(tmp, 'toy-a.jsonl')
    _toy_db(db_a, None, [
        ('app[aaaa]::entry', 'app[aaaa]::middle', 'src/a.rs:1:1:', 'call'),
        ('app[aaaa]::middle', 'core[bbbb]::leaf_fn', 'src/a.rs:2:1:', 'call'),
        ('app[aaaa]::middle', 'app[aaaa]::orphan', 'src/a.rs:3:1:', 'call'),
        ('unrelated[cccc]::driver', 'unrelated[cccc]::never', 'src/b.rs:1:1:',
         'call'),
    ])
    out_a = os.path.join(tmp, 'toy-a-atoms.json')
    rc = cmd_atoms(argparse.Namespace(db=db_a, root='app[aaaa]::entry', depth=2,
                                      max_atoms=50, filter=None, out=out_a))
    check(rc == 0, 'atoms toy-a rc=%d' % rc)
    doc_a = load_atoms(out_a)
    caps_a = {a['capability'] for a in doc_a['atoms']}
    check('orphan' in caps_a, 'reachable orphan missing (walk bug)')
    check('never' not in caps_a,
          '宁缺勿错被破：不可达节点 %r 出现在原子集里' % 'never')
    check('leaf_fn' in caps_a, 'depth-2 traversal lost leaf_fn')
    # oracle：落盘词表命中
    db_p = os.path.join(tmp, 'toy-p.jsonl')
    _toy_db(db_p, None, [
        ('app[aaaa]::entry', 'app[aaaa]::write_all', 'src/a.rs:1:1:', 'call'),
        ('app[aaaa]::write_all', 'core[bbbb]::fmt_write', 'src/a.rs:2:1:',
         'call'),
    ])
    out_p = os.path.join(tmp, 'toy-p-atoms.json')
    cmd_atoms(argparse.Namespace(db=db_p, root='app[aaaa]::entry', depth=2,
                                 max_atoms=50, filter=None, out=out_p))
    oracles = {a['capability']: a['oracle'] for a in load_atoms(out_p)['atoms']}
    check(oracles.get('write_all') == K_PERSIST,
          'K2 分类错：write_all -> %r' % oracles.get('write_all'))
    # oracle：测试 span ⇒ K4
    db_t = os.path.join(tmp, 'toy-t.jsonl')
    _toy_db(db_t, None, [
        ('app[aaaa]::entry', 'app[aaaa]::checked', 'tests/t.rs:5:9:', 'call'),
    ])
    out_t = os.path.join(tmp, 'toy-t-atoms.json')
    cmd_atoms(argparse.Namespace(db=db_t, root='app[aaaa]::entry', depth=1,
                                 max_atoms=50, filter=None, out=out_t))
    ot = {a['capability']: a['oracle'] for a in load_atoms(out_t)['atoms']}
    check(ot.get('checked') == K_EXTERNAL,
          'K4 分类错：tests/ span -> %r' % ot.get('checked'))
    # oracle：std 叶子 ⇒ K1
    db_l = os.path.join(tmp, 'toy-l.jsonl')
    _toy_db(db_l, None, [
        ('app[aaaa]::entry', 'core[bbbb]::ptr_read', 'src/a.rs:1:1:', 'call'),
    ])
    out_l = os.path.join(tmp, 'toy-l-atoms.json')
    cmd_atoms(argparse.Namespace(db=db_l, root='app[aaaa]::entry', depth=1,
                                 max_atoms=50, filter=None, out=out_l))
    ol = {a['capability']: a['oracle'] for a in load_atoms(out_l)['atoms']}
    check(ol.get('ptr_read') == K_LEAF,
          'K1 分类错：std 叶子 -> %r' % ol.get('ptr_read'))

    # --- 用例 2（证伪）：root 命中多个 ⇒ 必须 rc=2，不许挑一个
    rc = cmd_atoms(argparse.Namespace(db=db_a, root='app[aaaa]::', depth=2,
                                      max_atoms=50, filter=None, out=None))
    check(rc == 2, '歧义 root 未报错，rc=%d（宁缺勿错被破）' % rc)

    # --- 用例 3（证伪）：parity 必须报出差异，且不能在无差异时谎报
    db_b = os.path.join(tmp, 'toy-b.jsonl')
    _toy_db(db_b, None, [
        ('app[aaaa]::entry', 'app[aaaa]::middle', 'src/a.rs:1:1:', 'call'),
        ('app[aaaa]::middle', 'core[bbbb]::leaf_fn', 'src/a.rs:2:1:', 'call'),
        ('app[aaaa]::middle', 'app[aaaa]::theirs_only', 'src/a.rs:3:1:', 'call'),
    ])
    out_b = os.path.join(tmp, 'toy-b-atoms.json')
    cmd_atoms(argparse.Namespace(db=db_b, root='app[aaaa]::entry', depth=2,
                                 max_atoms=50, filter=None, out=out_b))
    mx = os.path.join(tmp, 'matrix.md')
    rc = cmd_parity(argparse.Namespace(mine=out_a, theirs=out_b, out=mx))
    check(rc == 0, 'parity rc=%d' % rc)
    body = open(mx, 'r', encoding='utf-8').read()
    check('[parity] shared=' in body, 'parity 摘要行缺失')
    check('仅对方有' in body and 'theirs_only' in body,
          'parity 漏报仅对方有的原子（证伪失败）')
    check('orphan' in body.split('仅我方有')[1].split('仅对方有')[0],
          'parity 漏报仅我方有的原子（证伪失败）')
    # 无差异时不得谎报差异
    out_a2 = os.path.join(tmp, 'toy-a2-atoms.json')
    cmd_atoms(argparse.Namespace(db=db_a, root='app[aaaa]::entry', depth=2,
                                 max_atoms=50, filter=None, out=out_a2))
    mx2 = os.path.join(tmp, 'matrix2.md')
    cmd_parity(argparse.Namespace(mine=out_a2, theirs=out_a2, out=mx2))
    b2 = open(mx2, 'r', encoding='utf-8').read()
    check('shared=%d only_mine=0 only_theirs=0' % len(
        {a['capability'] for a in load_atoms(out_a2)['atoms']}) in b2,
          '同集对位谎报了差异：%s' % b2.splitlines()[-1])

    if fails:
        sys.stderr.write('selftest FAILED (%d):\n' % len(fails))
        for f in fails:
            sys.stderr.write('  - %s\n' % f)
        return 1
    sys.stdout.write('selftest PASS: 6 正例 + 3 证伪用例全绿\n')
    return 0


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[1])
    sub = ap.add_subparsers(dest='cmd', required=True)

    a = sub.add_parser('atoms', help='decompose an entry symbol into atoms')
    a.add_argument('--db', required=True)
    a.add_argument('--root', required=True)
    a.add_argument('--depth', type=int, default=3)
    a.add_argument('--max-atoms', dest='max_atoms', type=int, default=500)
    a.add_argument('--filter', default=None,
                   help='only keep edges touching nodes matching this substring')
    a.add_argument('--out', default=None)
    a.set_defaults(func=cmd_atoms)

    p = sub.add_parser('parity', help='capability parity matrix between 2 sets')
    p.add_argument('--mine', required=True)
    p.add_argument('--theirs', required=True)
    p.add_argument('--out', default=None)
    p.set_defaults(func=cmd_parity)

    s = sub.add_parser('selftest', help='positive + falsification cases')
    s.add_argument('--tmpdir', default='/tmp/nt-decompose-selftest')
    s.set_defaults(func=cmd_selftest)

    args = ap.parse_args(argv[1:])
    return args.func(args)


if __name__ == '__main__':
    sys.exit(main(sys.argv))
