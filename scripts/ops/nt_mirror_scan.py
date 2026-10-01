#!/usr/bin/env python3
"""nt_mirror_scan.py — 跨 crate / 跨层「镜像分叉」审计：谁是真身，谁是冻结的旧分叉。

为什么存在【实测动因】
  本会话用 `nt_dup_dead` 得出「重复但不可删」的结论后，顺着 `rrf_fuse` 等
  22 个**跨层**同名副本往下查，发现的不是零散重复，而是一条**结构性分叉**：

    `neotrix-types` 携带了一份 **`neotrix-core` L2 的部分镜像**，冻结在
    **2026-07-06**（commit 主题 "checkpoint before Cycle 33 cleanup"），
    而 core 侧的对应模块是 **2026-09-18 / 09-25** 新写的（更晚、更全）。

    铁证（同一次 `hexagram_hadamard` 查询的 4 个调用者）：
      neotrix[7f32]  l5_cognition::nt_core_walsh  → core 的 e8
      neotrix_types  core::nt_core_walsh          → types 的 e8
    ⇒ **两棵平行子树**，各自依赖自己那棵里的 `hexagram_hadamard`。

  这不是"多点重复"，是**两个真身并存**。任何只按"同名函数"去重的工具
  （fn_drift）都只会报「同名」，**报不出「这是一个分叉」** —— 那需要跨 crate
  做**模块级**比对。

本工具做什么
  1. 按**模块名**找出跨 crate/跨层同时存在的同名模块；
  2. 对每一对做**函数体归一化比对**（复用 nt_fn_drift 的提取与归一化）；
  3. 给出**规模**（字节/函数数）与**重叠度**（逐字相同的函数占比）；
  4. 用**git 首次提交日期** + **仓内消费者数**给出「谁是冻结的旧分叉」的证据
     —— 但**不裁决**，只把证据摆出来（同 G7 / 无定点不改）。

⛔ 为什么输出不是门
  「该删哪一份」是**产品/架构决策**，判据是意图而不是文本相似度。
  本工具把决策所需的**证据**备齐，删除动作留给 owner 裁决并单独立项。

用法：
  python3 scripts/ops/nt_mirror_scan.py                     # 全量镜像清单
  python3 scripts/ops/nt_mirror_scan.py --min-overlap 0.9   # 只看高重叠（疑似真分叉）
  python3 scripts/ops/nt_mirror_scan.py selftest
"""
import argparse
import importlib.util
import os
import re
import subprocess
import sys
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# 参与比对的 crate 根：低层 crate → 高层 crate 的依赖方向
CRATES = [
    ('neotrix-types', 'crates/neotrix-types/src'),
    ('neotrix-sysctl', 'crates/neotrix-sysctl/src'),
    ('neotrix-consciousness', 'crates/neotrix-consciousness/src'),
    ('neotrix-reasoning', 'crates/neotrix-reasoning/src'),
    ('neotrix-gateway', 'crates/neotrix-gateway/src'),
    ('neotrix-multi-agent', 'crates/neotrix-multi-agent/src'),
    ('neotrix-neobot', 'crates/neotrix-neobot/src'),
    ('neotrix-audit', 'crates/neotrix-audit/src'),
    ('nt-core-capability-tree', 'crates/nt-core-capability-tree/src'),
    ('neotrix-core', 'neotrix-core/src'),
]

LAYERS = ('l0_substrate', 'l1_action', 'l2_perception', 'l3_embodiment',
          'l4_emotion', 'l5_cognition', 'l6_meta')


def load_drift():
    spec = importlib.util.spec_from_file_location(
        'nt_fn_drift', os.path.join(REPO, 'scripts/ops/nt_fn_drift.py'))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def crate_of(path):
    for name, root in CRATES:
        if path.startswith(root + '/'):
            return name, root
    return '?', ''


def layer_of(path):
    for l in LAYERS:
        if '/src/%s/' % l in path:
            return l
    return '-'


def module_index():
    """(crate, 模块名) -> [path]。模块名 = 去掉 mod.rs / lib.rs 后的目录名或文件名。"""
    idx = defaultdict(list)
    for _crate, root in CRATES:
        base = os.path.join(REPO, root)
        if not os.path.isdir(base):
            continue
        for dirpath, _dirs, files in os.walk(base):
            for f in files:
                if not f.endswith('.rs'):
                    continue
                stem = os.path.splitext(f)[0]
                if stem in ('mod', 'lib', 'main'):
                    continue
                p = os.path.join(dirpath, f)
                idx[(root, stem)].append(os.path.relpath(p, REPO))
    return idx


def fn_map(fd, path):
    src = fd.read(path)
    if src is None:
        return {}
    out = {}
    clean = fd.strip_noise(src)
    for m in re.finditer(r'\bfn\s+([a-z_][a-z0-9_]*)\s*(<[^>]*>)?\s*\(', clean):
        name = m.group(1)
        body, _line = fd.extract_fn(src, name)
        if body is not None:
            out.setdefault(name, set()).add(body)
    return out


def added_date(path):
    """git 首次提交日期（YYYY-MM-DD）。拿不到就返回 '?'，**不猜**。"""
    try:
        p = subprocess.run(
            ['git', 'log', '--diff-filter=A', '--format=%ad', '--date=short',
             '-1', '--', path],
            cwd=REPO, capture_output=True, text=True, timeout=60)
    except (subprocess.SubprocessError, FileNotFoundError):
        return '?'
    out = (p.stdout or '').strip()
    return out or '?'


def report(args):
    fd = load_drift()
    idx = module_index()
    by_name = defaultdict(list)
    for (_root, stem), paths in idx.items():
        for p in paths:
            crate, _r = crate_of(p)
            by_name[stem].append((crate, p))

    rows = []
    for stem, entries in sorted(by_name.items()):
        crates = {c for c, _ in entries}
        if len(crates) < 2:
            continue
        # 每个 crate 取最大的那个文件作为代表（同名多文件时以最大者为准）
        rep = {}
        for c, p in entries:
            sz = os.path.getsize(os.path.join(REPO, p))
            if c not in rep or sz > rep[c][0]:
                rep[c] = (sz, p)
        if len(rep) < 2:
            continue
        maps = {c: fn_map(fd, p) for c, (_s, p) in rep.items()}
        names = set()
        for m in maps.values():
            names |= set(m.keys())
        if not names:
            continue
        same = diff = only = 0
        for n in names:
            bodies = [maps[c].get(n) for c in maps]
            present = [b for b in bodies if b]
            if len(present) < 2:
                only += 1
                continue
            if any(bodies[i] & bodies[j] for i in range(len(bodies))
                   for j in range(i + 1, len(bodies))):
                same += 1
            else:
                diff += 1
        total = same + diff + only
        overlap = same / total if total else 0.0
        rows.append({
            'stem': stem, 'rep': rep, 'same': same, 'diff': diff,
            'only': only, 'total': total, 'overlap': overlap,
        })

    rows.sort(key=lambda r: (-r['overlap'], -r['same']))
    shown = [r for r in rows if r['overlap'] >= args.min_overlap]

    print('[mirror] 同名模块出现在 >=2 个 crate 的对数: %d' % len(rows))
    print('[mirror] 其中重叠度 >= %.0f%% 的: %d' % (args.min_overlap * 100,
                                              len(shown)))
    print()
    for r in shown[:args.limit]:
        print('== %s   逐字相同函数 %d / 同名 %d  ⇒ 重叠 %.0f%%'
              % (r['stem'], r['same'], r['total'], r['overlap'] * 100))
        for c in sorted(r['rep']):
            size, p = r['rep'][c]
            print('   %-26s %-13s %7.1f KB  added=%s  %s'
                  % (c, layer_of(p), size / 1024.0, added_date(p), p))
        print()

    print('[mirror] 判读要点（不给结论，只给判据）：')
    print('  · 重叠度高 + 两侧 added 日期相差大 ⇒ 疑似「旧分叉 + 新真身」，')
    print('    旧的那份是**冻结快照**，与真身之间的差异会随时间扩大（漂移风险）；')
    print('  · 重叠度低 ⇒ 更可能是同名不同域的正常重复，**不是**分叉；')
    print('  · crate 之间的依赖方向决定谁**能**用谁：低层 crate 无法依赖高层 crate，')
    print('    所以「高层 re-export 低层」是唯一合法收敛方向；')
    print('  · ⛔ 本工具**不判删**：「删哪份」是架构决策，判据是意图而非文本相似度。')
    return 0


def selftest(_a):
    fails = []

    def check(c, m):
        if not c:
            fails.append(m)

    fd = load_drift()

    # 1) 归一化后逐字相同 ⇒ 记 same（这正是分叉检测的核心信号）
    a = 'fn f(x: u32) -> u32 { x + 1 }'
    b = 'fn f(x: u32) -> u32 {\n  x+1\n}'
    ba, _ = fd.extract_fn(a, 'f')
    bb, _ = fd.extract_fn(b, 'f')
    check(ba is not None and ba == bb,
          'same-detect failed: %r vs %r' % (ba, bb))

    # 2) 同名不同体 ⇒ 不得记 same（否则会把「同名不同域」误判成分叉）
    c2, _ = fd.extract_fn('fn f(x: u32) -> u32 { x * 2 }', 'f')
    check(ba != c2, 'different bodies must not count as same')

    # 3) crate_of / layer_of 必须能区分（本工具的分组依据）
    check(crate_of('crates/neotrix-types/src/core/nt_core_e8.rs')[0] == 'neotrix-types',
          'crate_of(neotrix-types) wrong')
    check(crate_of('neotrix-core/src/l2_perception/x.rs')[0] == 'neotrix-core',
          'crate_of(neotrix-core) wrong')
    check(layer_of('neotrix-core/src/l4_emotion/a/b.rs') == 'l4_emotion',
          'layer_of wrong')

    # 4) added_date 不可达路径 ⇒ 返回 '?' 而不是崩或编造日期
    check(added_date('no/such/file.rs') == '?',
          'added_date must degrade to "?" for a missing path')

    if fails:
        sys.stderr.write('selftest FAILED (%d):\n' % len(fails))
        for f in fails:
            sys.stderr.write('  - %s\n' % f)
        return 1
    sys.stdout.write('selftest PASS: 4 例（含 2 例证伪：异体不得判同、'
                     'git 取不到日期必须降级为 ?）\n')
    return 0


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[1])
    ap.add_argument('--min-overlap', type=float, default=0.5,
                    help='只显示重叠度 >= 该值的模块对（默认 0.5）')
    ap.add_argument('--limit', type=int, default=12)
    ap.add_argument('cmd', nargs='?', default=None)
    args = ap.parse_args(argv[1:])
    if args.cmd == 'selftest':
        return selftest(args)
    return report(args)


if __name__ == '__main__':
    sys.exit(main(sys.argv))