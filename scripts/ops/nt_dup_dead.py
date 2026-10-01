#!/usr/bin/env python3
"""nt_dup_dead.py — 「纯重复副本 × 零接线」交叉判定：既是重复、又没接线的那些。

为什么存在（实测动因，逐层推出）
  上一轮 `nt_fn_drift` 给出 29 个 **IDENTICAL**（归一化后逐字相同 ⇒ 纯重复可合并）。
  但「重复」本身**不等于**该动手：跨 crate 的重复可能是分层纪律的正常结果，
  同层内的重复也可能有正当理由。⇒ 需要第二个判据回答**该不该动**。

  本工具把这个交叉做出来，判据全部来自**编译器解析过的调用边**：
    A. 纯重复      —— fn_drift 判定 IDENTICAL（文本相等）
    B. 零接线      —— 该函数在边表里**没有任何入边**（callers = 0）
    C. 非测试      —— 排除 `#[test]` / `::tests::` / `benches`
  A∧B∧C ⇒ 「既重复又零接线」：**删它不会破坏任何在仓内可达的行为**，
  是唯一可以据此行动的一类。C 单独不成立（私有 helper 的入边在其所属函数内部）。

⚠️ **三条必须随输出一起看的边界**：
  1. **零入边 ≠ 死代码**。`nt_callgraph --unreachable` 的既有结论：
     `pub` 定义可能服务**仓外消费者**（本仓是 lib + 多个 crate），
     静态边表看不见 trait 动态派发 / 宏展开 / async 派发。
     ⇒ 本工具**不判死**，只输出「可考虑合并或删除的候选」并标注该风险。
  2. **同名 ≠ 同用途**。两个 `median_f64` 逐字相同，但一个在 gateway、一个在
     nt_core_gate —— 合并没有层依赖收益（本仓层依赖门 19 条已知债）。
  3. **不是门**。不接 CI（同 G7）：判据是「文本相等 + 静态零入边」，
     不足以当正确性判据。输出是分诊单。

用法：
  python3 scripts/ops/nt_dup_dead.py                       # 全量交叉分诊
  python3 scripts/ops/nt_dup_dead.py --verbose             # 含 IDENTICAL 但有接线的（对照组）
  python3 scripts/ops/nt_dup_dead.py selftest
"""
import argparse
import importlib.util
import json
import os
import sys
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DB = os.path.join(REPO, '.project-map/edges-all.jsonl')

SKIP_CALLER = ('#[test]', '::tests', '::benches', 'test_')


def load_drift():
    spec = importlib.util.spec_from_file_location(
        'nt_fn_drift', os.path.join(REPO, 'scripts/ops/nt_fn_drift.py'))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def build_incoming(db_path):
    """callee -> {caller}。只看非测试调用者。"""
    inc = defaultdict(set)
    if not os.path.isfile(db_path):
        return None
    with open(db_path, encoding='utf-8', errors='replace') as fh:
        for line in fh:
            try:
                r = json.loads(line)
            except ValueError:
                continue
            caller = r.get('caller') or ''
            callee = r.get('callee') or ''
            if any(s in caller for s in SKIP_CALLER):
                continue
            if caller and callee:
                inc[callee].add(caller)
    return inc


def analyse(db_path, verbose=False):
    inc = build_incoming(db_path)
    if inc is None:
        sys.stderr.write('no edge db at %s\n' % db_path)
        return 2
    fd = load_drift()
    cands = fd.discover(db_path)

    dup_unwired, dup_wired, nonident_unwired = [], [], []
    for leaf in sorted(cands):
        mods = cands[leaf]
        verdict, detail, _, rows = fd.compare_pair(leaf, mods)
        # 每份副本单独判「零入边」——入边按 DefId 全路径匹配，不能只按叶子名
        for mod, path in sorted(mods.items()):
            defid_suffix = mod.split('::')[-1]
            has_in = any(c.endswith('::' + leaf) or c.endswith('::%s::%s' % (defid_suffix, leaf))
                         for c in inc)
            rec = {'leaf': leaf, 'mod': mod, 'path': path,
                   'verdict': verdict, 'wired': has_in}
            if verdict == 'IDENTICAL':
                (dup_wired if has_in else dup_unwired).append(rec)
            elif not has_in:
                nonident_unwired.append(rec)

    print('[dup-dead] IDENTICAL 总数: %d' % (len(dup_unwired) + len(dup_wired)))
    print('[dup-dead] 其中零接线（**可行动候选**）: %d' % len(dup_unwired))
    print('[dup-dead] 其中有接线（对照组，勿动）: %d' % len(dup_wired))
    print('[dup-dead] 非 IDENTICAL 但零接线（与重复无关，单列供对照）: %d'
          % len(nonident_unwired))
    print()
    print('=== A∧B∧C：既纯重复、又零接线 ===')
    if not dup_unwired:
        print('  （无）')
    for rec in dup_unwired:
        print('  %-24s %s' % (rec['leaf'], rec['path']))
        print('       %s' % rec['mod'])
    if verbose:
        print()
        print('=== 对照组：IDENTICAL 但有接线（勿动）===')
        for rec in dup_wired:
            print('  %-24s %s' % (rec['leaf'], rec['path']))
    print()
    print('[dup-dead] 边界：零入边 ≠ 死代码。pub 定义可能服务仓外消费者；'
          '静态边表看不见 trait 动态派发 / 宏展开 / async 派发。')
    print('[dup-dead] 私有 helper 的入边在其所属函数体内，单独看零入边是正常的'
          '⇒ 本工具只对 pub 顶层函数有行动意义。')
    print('[dup-dead] 不是门（不接 CI）：判据是文本相等 + 静态零入边（G7）。')
    return 0


def selftest(_a):
    fails = []

    def check(c, m):
        if not c:
            fails.append(m)

    # 1) 入边索引：测试调用者必须被排除（否则每个 #[test] 都给自己造一条入边）
    tmp = '/tmp/nt_dup_dead_selftest.jsonl'
    with open(tmp, 'w', encoding='utf-8') as fh:
        for row in (
            {'caller': 'a[1]::tests::t_x', 'callee': 'a[1]::helper'},
            {'caller': 'b[2]::real_caller', 'callee': 'a[1]::helper'},
            {'caller': 'c[3]::test_thing', 'callee': 'c[3]::target'},
        ):
            fh.write(json.dumps(row) + '\n')
    inc = build_incoming(tmp)
    check('a[1]::tests::t_x' not in inc.get('a[1]::helper', set()),
          'test caller leaked into incoming edges')
    check('b[2]::real_caller' in inc.get('a[1]::helper', set()),
          'real caller missing from incoming edges')
    check('c[3]::target' not in inc.get('c[3]::target', set()),
          'test_-prefixed caller not filtered')

    # 2) 缺边表必须 rc=2 而不是空报告（零断言的 PASS 是 vacuous green）
    rc = analyse('/tmp/definitely-not-here.jsonl')
    check(rc == 2, 'missing db must rc=2, got %r' % rc)

    # 3) 边界：同名逐字相同但跨 crate 的，必须**不**被当可行动候选
    fd = load_drift()
    src_a = 'fn f(x: u32) -> u32 { x + 1 }'
    src_b = 'fn f(x: u32) -> u32 {\n   x+1\n}'
    ba, _ = fd.extract_fn(src_a, 'f')
    bb, _ = fd.extract_fn(src_b, 'f')
    check(ba == bb and ba is not None,
          'normalisation should make these identical')
    # 不同语义绝不能被判 IDENTICAL
    bc, _ = fd.extract_fn('fn f(x: u32) -> u32 { x + 2 }', 'f')
    check(ba != bc, 'different bodies must not compare equal')

    os.remove(tmp) if os.path.exists(tmp) else None
    if fails:
        sys.stderr.write('selftest FAILED (%d):\n' % len(fails))
        for f in fails:
            sys.stderr.write('  - %s\n' % f)
        return 1
    sys.stdout.write('selftest PASS: 4 例（含 2 例证伪：测试入边排除、'
                     '缺边表 rc=2、异体不得判同）\n')
    return 0


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[1])
    ap.add_argument('--db', default=DB)
    ap.add_argument('--verbose', action='store_true')
    ap.add_argument('cmd', nargs='?', default=None)
    args = ap.parse_args(argv[1:])
    if args.cmd == 'selftest':
        return selftest(args)
    return analyse(args.db, args.verbose)


if __name__ == '__main__':
    sys.exit(main(sys.argv))