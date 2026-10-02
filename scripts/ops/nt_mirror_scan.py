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



# ── 同名 impl 的方法集指纹（2026-09-30 新增）────────────────────────────
# 起因：`MIRROR-BANK §7` 记录的 ⛔ P1 跨域错位 —— L0 里藏着 L5 的
# `IITPhiCalculator` 陈旧分叉。原版本工具只比**同名模块**的函数重叠，
# 而该分叉藏在两个**不同模块名**的文件里（`nt_core_consciousness_types.rs`
# vs `nt_iit_phi.rs`）⇒ 按模块名匹配根本看不见它。
# ⇒ 改为直接扫「同名 `struct`/`impl`」，比较其**方法集**：
# 方法集不同 = 同一算法有两个已分叉的实现，是比「函数逐字重复」更硬的证据。

_STRUCT_RE = re.compile(r'(?:^|\n)\s*pub\s+struct\s+([A-Z][A-Za-z0-9_]*)')
_IMPL_RE = re.compile(r'\bimpl(?:<[^>]*>)?\s+([A-Z][A-Za-z0-9_]*)\b\s*(?:where[^\{]*)?\{')


def _brace_end(clean: str, open_at: int) -> int:
    depth = 0
    for j in range(open_at, len(clean)):
        c = clean[j]
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return j
    return -1


def impl_methods(fd, path, all_visibility=False):
    """该文件里每个 `impl <Type>` 的方法集合。返回 {Type: {method,...}}。

    ⚠️ 2026-09-30 修正（真实误报）：首版**只收 `pub fn`**，于是把
    「方法存在但可见性不同」误报成「缺方法」。
    实测踩中：`WalshMemoryIndex` 的 l0 版把 `wh_dot`/`wh_inverse`/
    `wh_transform` 设为 `pub fn`（12 个全 pub），而 types 与 l5 版
    把它们设为**私有** `fn`（13 个 = 9 pub + 4 私有）。
    ⇒ 首版报「types/l5 缺 3 个方法」是**错的**，它们存在，只是私有。
    默认只收 `pub fn`（比较**公开 API 面**）；`all_visibility=True` 时
    连私有一起收（比较**实现完整度**）。可见性差异另行报告，不混进「缺方法」。
    """
    src = fd.read(path)
    if src is None:
        return {}
    clean = fd.strip_noise(src)
    out = {}
    for m in _IMPL_RE.finditer(clean):
        b = clean.find('{', m.end() - 1)
        if b == -1:
            continue
        e = _brace_end(clean, b)
        if e == -1:
            continue
        body = clean[b:e]
        pat = (r'\n\s*(?:pub\s+)?fn\s+([a-z_][A-Za-z0-9_]*)'
               if all_visibility else r'\n\s*pub\s+fn\s+([a-z_][A-Za-z0-9_]*)')
        ms = set(re.findall(pat, body))
        if ms:
            out.setdefault(m.group(1), set()).update(ms)
    return out


def impl_visibility(fd, path):
    """该文件里每个 impl 方法的可见性。返回 {Type: {method: 'pub'|'priv'}}。"""
    src = fd.read(path)
    if src is None:
        return {}
    clean = fd.strip_noise(src)
    out = {}
    for m in _IMPL_RE.finditer(clean):
        b = clean.find('{', m.end() - 1)
        if b == -1:
            continue
        e = _brace_end(clean, b)
        if e == -1:
            continue
        body = clean[b:e]
        vis = {}
        for mm in re.finditer(r'\n\s*(pub\s+)?fn\s+([a-z_][A-Za-z0-9_]*)', body):
            vis[mm.group(2)] = 'pub' if mm.group(1) else 'priv'
        if vis:
            out.setdefault(m.group(1), {}).update(vis)
    return out


def report_dup_impls(fd, roots, limit):
    """跨层同名 struct/impl 且方法集不同 ⇒ 报告（不判删）。"""
    where = defaultdict(list)
    for root in roots:
        base = os.path.join(REPO, root) if not os.path.isabs(root) else root
        if not os.path.isdir(base):
            continue
        for dirpath, _dirs, files in os.walk(base):
            norm = dirpath.replace(os.sep, '/')
            if '/target' in norm or '/.worktrees' in norm:
                continue
            for f in sorted(files):
                if not f.endswith('.rs'):
                    continue
                p = os.path.relpath(os.path.join(dirpath, f), REPO)
                for ty, ms in impl_methods(fd, p, all_visibility=True).items():
                    where[ty].append((p, ms))
    rows = []
    for ty, sites in sorted(where.items()):
        if len(sites) < 2:
            continue
        layers = {layer_of(p) for p, _ in sites}
        if len(layers) < 2:
            continue  # 只关心**跨层**同名（同层同名多为正常的不同类型）
        sets = [ms for _p, ms in sites]
        if all(x == sets[0] for x in sets):
            continue  # 方法集完全一致 ⇒ 暂无分叉证据
        union = set().union(*sets)
        inter = set.intersection(*sets)
        rows.append({
            'ty': ty, 'sites': sites, 'layers': sorted(x for x in layers if x),
            'union': union, 'inter': inter,
            'jaccard': len(inter) / len(union) if union else 0.0,
        })
    # ── 判据（经两轮修正，最终版）────────────────────────────────────────
    # 首版：方法集「不相等」即报 ⇒ 135 组，其中 Jaccard 0.00 的 `Actor`/`Channel`
    #      只是不同域撞名，纯噪声。
    # 二版：0<Jaccard<1 ⇒ 仍 94 组，因为 J=0.08 实测只共享 1 个方法（`new`）。
    # 三版：交集>=3 且 J>=0.5 ⇒ **漏掉了已知真例 `IITPhiCalculator`**，
    #      原因是它有**三份**副本，用「所有站点共同交集」时
    #      交集被离群副本拉低到 3（J=0.38）⇒ **度量本身有缺陷**。
    # 最终：**以方法最多的站点为基准（候选真身），看其他站点是否缺方法**。
    #   「缺方法」才是分叉的语义签名 —— 同一算法，一个实现有、另一个没有。
    #   用 max 而非 all-intersection，离群副本不再拉低指标。
    # 四版：单靠「缺方法」又报出 114 组 —— 因为**同名不等于同源**
    # （`CostAwareRouter` l1 有 1 方法 vs l5 有 15，那是两个不同的东西撞名）。
    # ⇒ 还需第三个条件：**与基准高度相似**（pairwise Jaccard >= MIN_J）。
    # 「高度相似 + 缺方法」同时成立，才是冻结分叉；
    # 「不相似」⇒ 同名巧合；「相似且不缺」⇒ 尚未漂移。
    MIN_MISSING, MIN_J = 2, 0.4
    forks = []
    for r in rows:
        ref_i = max(range(len(r['sites'])), key=lambda i: len(r['sites'][i][1]))
        ref_ms = r['sites'][ref_i][1]
        sites2 = []
        worst = 0
        for i, (p, ms) in enumerate(r['sites']):
            if i == ref_i:
                continue
            miss = ref_ms - ms
            union = ref_ms | ms
            jac = len(ref_ms & ms) / len(union) if union else 0.0
            if len(miss) >= MIN_MISSING and jac >= MIN_J:
                sites2.append((p, ms, sorted(miss), jac))
                worst = max(worst, len(miss))
        if sites2:
            forks.append({
                'ty': r['ty'], 'layers': r['layers'], 'ref': r['sites'][ref_i],
                'forks': sites2, 'worst': worst,
                'union': r['union'], 'inter': r['inter'], 'jaccard': r['jaccard'],
            })
    coincidental = len(rows) - len(forks)
    print('[dup-impl] 跨层同名 struct/impl：%d 组同名，其中**疑似陈旧分叉 %d 组**'
          % (len(rows), len(forks)))
    print('[dup-impl] （判据 = 以方法最多的站点为基准，其他站点同时满足'
          '**缺 >= %d 个方法**且**pairwise Jaccard >= %.2f** —— '
          '「高度相似却缺方法」才是冻结分叉；不相似是同名巧合，不缺是尚未漂移。'
          '%d 组不满足，不报）' % (MIN_MISSING, MIN_J, coincidental))
    rows = forks
    rows.sort(key=lambda r: -r['worst'])
    for r in rows[:limit]:
        print('== %s   基准 %d 方法(%s)   最多缺 %d 个'
              % (r['ty'], len(r['ref'][1]), r['ref'][0].split('/')[-1], r['worst']))
        print('   %-13s %-46s %2d 方法  ← 基准'
              % (layer_of(r['ref'][0]), r['ref'][0], len(r['ref'][1])))
        for p, ms, miss, jac in r['forks']:
            print('   %-13s %-46s %2d 方法  J=%.2f  缺: %s'
                  % (layer_of(p), p, len(ms), jac, ', '.join(miss)))
            # 可见性差异单列，**不混进「缺方法」**（见 impl_methods 文档）
            pv = impl_visibility(fd, p).get(r['ty'], {})
            rv = impl_visibility(fd, r['ref'][0]).get(r['ty'], {})
            visdiff = sorted(m for m in (set(pv) & set(rv)) if pv[m] != rv[m])
            if visdiff:
                print('      %s可见性不同: %s'
                      % ('私有' if pv.get(visdiff[0]) == 'priv' else '公开',
                         ', '.join(visdiff)))
        print()
    rows = forks
    if rows:
        print('[dup-impl] 判读要点：')
        print('  · Jaccard 低 + 一侧明显缺方法 ⇒ 那侧是**陈旧冻结分叉**；')
        print('  · 成因通常是「低层无法依赖高层，于是手工复制一份」；')
        print('  · ⛔ 本工具**不判删**：真身该放哪、要不要迁移，属架构决策。')
        print('    实例与三条处置建议见 docs/architecture/MIRROR-BANK-2026-09-30.md §7')
    return len(rows)


# ── 「低层复制高层算法」检测（2026-09-30 新增）─────────────────────────
# 起因：MIRROR-BANK §7 的 IIT Phi 案例。根因是**层规则**：
#   L0 全层实测 **0 处**引用 L2-L6 ⇒ L0 不能直接用 L5 的真身
#   ⇒ 只能手工复制一份到 L0（于是有了那份陈旧分叉）。
# ⇒ 这个缺陷的**成因**是架构约束，靠「找重复」发现不了，
#   必须显式检查「低层是否持有高层算法的副本」。
# 判据：某类型在 L0 有定义，且在更高层也有同名定义（方法集重叠 ⇒ 同源）。
# 与 dup-impl 的区别：那个只看「方法集差异」，这个专门盯**层级倒挂**。

_LAYER_ORDER = ['l0_substrate', 'l1_action', 'l2_perception', 'l3_embodiment',
                'l4_emotion', 'l5_cognition', 'l6_meta']


def report_inverted_hierarchy(fd, roots, limit):
    """L0（或低层）持有高层同名类型 ⇒ 层规则迫使的复制，报告成因。"""
    low, high = {}, {}
    for root in roots:
        base = os.path.join(REPO, root) if not os.path.isabs(root) else root
        if not os.path.isdir(base):
            continue
        for dirpath, _dirs, files in os.walk(base):
            norm = dirpath.replace(os.sep, '/')
            if '/target' in norm or '/.worktrees' in norm:
                continue
            for f in sorted(files):
                if not f.endswith('.rs'):
                    continue
                p = os.path.relpath(os.path.join(dirpath, f), REPO)
                ly = layer_of(p)
                if ly not in _LAYER_ORDER:
                    continue
                bucket = low if ly == 'l0_substrate' else high
                for ty, ms in impl_methods(fd, p, all_visibility=True).items():
                    bucket.setdefault(ty, []).append((ly, p, ms))
    rows = []
    for ty, lsites in low.items():
        hsites = high.get(ty, [])
        if not hsites:
            continue
        for ly, lp, lms in lsites:
            for hy, hp, hms in hsites:
                if hy == 'l0_substrate':
                    continue
                inter = len(lms & hms)
                if inter < 3:
                    continue  # 共享面过小 ⇒ 同名巧合，不是复制
                rows.append({
                    'ty': ty, 'low': (ly, lp, len(lms)),
                    'high': (hy, hp, len(hms)),
                    'inter': inter, 'jac': inter / len(lms | hms),
                })
    rows.sort(key=lambda r: -r['inter'])
    print('[inverted] L0 持有高层同名类型（层规则迫使的复制）：%d 组' % len(rows))
    for r in rows[:limit]:
        print('== %s   L0(%d 方法, %s) ↔ %s(%d 方法, %s)   共享 %d 方法 J=%.2f'
              % (r['ty'], r['low'][2], r['low'][1].split('/')[-1],
                 r['high'][0], r['high'][2], r['high'][1].split('/')[-1],
                 r['inter'], r['jac']))
    if rows:
        print('[inverted] 判读要点：')
        print('  · 成因是**架构约束**而非疏忽：实测 L0 全层 0 处引用 L2-L6，')
        print('    所以 L0 无法直接用高层真身，只能复制。⇒ 光「去重」修不了，')
        print('    必须先决定真身该落在哪一层（或下沉到契约层）。')
        print('  · ⛔ 本工具**不判删**：真身归属属架构决策。')
        print('    实例与三条处置建议见 docs/architecture/MIRROR-BANK-2026-09-30.md §7')
    return len(rows)


# ── 孤儿文件检测（2026-09-30 新增）───────────────────────────────────
# 起因：删掉 `l6_meta/nt_core_qtest.rs`（538 行，含 15 个测试）后发现 ——
# 那个文件**从未被编译**，因为 `l6_meta/mod.rs` 里根本没有 `pub mod nt_core_qtest`。
#
# ⚠️ 这是**另外两个工具都看不见的盲区**：
# `nt_dup_dead` / `nt_pub_dead` 都基于源码**文本**分析「有什么」，
# 而孤儿文件的问题是「**是否被编译**」—— 文本再丰富也不回答这个问题。
# ⇒ 只能沿 **mod 声明链**核实：从 crate 根（lib.rs/main.rs）出发，
#    递归跟随 `mod X;` / `pub mod X;` / `#[path=...] mod X;`，
#    走不到的文件就是孤儿。
#
# 边界（诚实声明）：
# · `include!` 宏引入的文件**不在本检查覆盖范围**（不是 `mod` 声明）；
# · `cfg_attr` / 宏生成的 `mod` 声明无法静态判定；
# · 跨 crate 的 `#[path]` 相对路径按声明所在文件解析。
# ⇒ 本检查**只报候选**，且只报「在磁盘上存在、但从 crate 根走不到」的文件。

_MOD_DECL = re.compile(
    r'(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?(?:'
    r'\#\s*\[\s*path\s*=\s*"([^"]+)"[^\]]*\]\s*'
    r'|mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:;|\{)'
    r')'
)


def reachable_mods(entry: str, fd=None):
    """从 crate 入口出发沿 mod 声明链走一遍，返回可达的 .rs 绝对路径集合。"""
    seen, queue, out = set(), [os.path.abspath(entry)], set()
    while queue:
        f = queue.pop()
        if f in seen:
            continue
        seen.add(f)
        out.add(f)
        if not f.endswith('.rs') or not os.path.isfile(f):
            continue
        try:
            with open(f, encoding='utf-8', errors='replace') as fh:
                clean = fd.strip_noise(fh.read())
        except OSError:
            continue
        base_dir = os.path.dirname(f)
        for m in _MOD_DECL.finditer(clean):
            path_attr, name = m.group(1), m.group(2)
            if path_attr:
                cand = os.path.normpath(os.path.join(base_dir, path_attr))
                if cand.endswith('.rs'):
                    queue.append(cand)
                continue
            if not name:
                continue
            # ── Rust 2018 模块解析（这里踩了两次坑，都已修）──────────────
            # 坑 1：两个候选都不存在时**没有 break**，路径沿用上一轮继续拼，
            #       产出 `l4_emotion/mod.rs/nt_memory.rs` 这种假路径。
            # 坑 2：**忽略了「同名 .rs 与同名目录并存」**。本仓真实存在
            #       `nt_memory_search.rs` + `nt_memory_search/` 并存；
            #       对**非 mod.rs** 文件内的 `mod bar;`，Rust 优先找
            #       **同名子目录**下的 `bar.rs`，其次才是同级 `bar.rs`。
            #       漏掉这条 ⇒ `nt_pure_fns.rs` 等被误判成孤儿
            #       （实测 343 个假孤儿，真孤儿只有 1 个）。
            #
            # 正确顺序：
            #   若当前文件是 `X/mod.rs`  ⇒ 候选 `X/name.rs`、`X/name/mod.rs`
            #   若当前文件是 `X.rs`        ⇒ 候选 `X/name.rs`（同名子目录优先）、
            #                                  `name.rs`（同级）
            # ⚠️ 坑 3（首版修正时引入）：`lib.rs`/`main.rs` 是**特例** ——
            #   它们没有同名目录，若按「去掉 .rs 得 x_stem」处理，
            #   `lib.rs` 会算出 `src/li/` 这个不存在的目录，
            #   候选全落空 ⇒ 可达数从 2187 掉到 **10**。
            stem = os.path.basename(f)
            if stem in ('mod.rs', 'lib.rs', 'main.rs'):
                mod_root = base_dir
                cands = [os.path.join(mod_root, name + '.rs'),
                         os.path.join(mod_root, name, 'mod.rs')]
            else:
                x_stem = stem[:-3]  # 去掉 .rs
                sibling_dir = os.path.join(base_dir, x_stem)
                cands = [os.path.join(sibling_dir, name + '.rs'),
                         os.path.join(sibling_dir, name, 'mod.rs'),
                         os.path.join(base_dir, name + '.rs')]
            found = False
            for cand in cands:
                if os.path.isfile(cand):
                    queue.append(cand)
                    found = True
                    break
            if not found:
                continue
    return out


def report_orphan_files(roots, limit, fd=None):
    """磁盘上存在、但从 crate 根沿 mod 链走不到的 .rs 文件。"""
    entries = []
    for root in roots:
        base = os.path.join(REPO, root) if not os.path.isabs(root) else root
        if not os.path.isdir(base):
            continue
        for cand in (os.path.join(base, 'lib.rs'), os.path.join(base, 'main.rs')):
            if os.path.isfile(cand):
                entries.append(cand)
        # ⚠️ 坑 4：`src/bin/*.rs` 与 `src/bin/<name>/main.rs` 由 **cargo 自动发现**，
        #    **不经 `mod` 声明**。漏掉这条 ⇒ 每个 bin 都被误报成孤儿
        #    （实测 `src/bin` 下 20 个文件全是假孤儿）。
        binroot = os.path.join(base, 'bin')
        if os.path.isdir(binroot):
            for n in sorted(os.listdir(binroot)):
                cand = os.path.join(binroot, n)
                if os.path.isfile(cand) and n.endswith('.rs'):
                    entries.append(cand)
                elif os.path.isdir(cand) and os.path.isfile(os.path.join(cand, 'main.rs')):
                    entries.append(os.path.join(cand, 'main.rs'))
    reach = set()
    for e in entries:
        reach |= reachable_mods(e, fd)
    on_disk = set()
    for root in roots:
        base = os.path.join(REPO, root) if not os.path.isabs(root) else root
        if not os.path.isdir(base):
            continue
        for dirpath, _dirs, files in os.walk(base):
            norm = dirpath.replace(os.sep, '/')
            if '/target' in norm or '/.worktrees' in norm:
                continue
            for f in files:
                if f.endswith('.rs'):
                    on_disk.add(os.path.abspath(os.path.join(dirpath, f)))
    orphans = sorted(on_disk - reach)
    print('[orphan] 磁盘上的 .rs: %d，从 crate 根沿 mod 链 + cargo 自动发现的 bin 可达: %d，'
          '**孤儿 %d**' % (len(on_disk), len(on_disk) - len(orphans), len(orphans)))
    rows = []
    for o in orphans:
        try:
            n = len(re.findall(r'#\[test\]', open(o, encoding='utf-8',
                                                 errors='replace').read()))
        except OSError:
            n = 0
        rows.append((o, n))
    rows.sort(key=lambda r: -r[1])
    shown = rows[:limit]
    for o, n in shown:
        print('   %-88s 测试 %d' % (os.path.relpath(o, REPO), n))
    # ⛔ 计数与展示必须一致：否则读的人看到「孤儿 167」却只看到 21 行，
    # 会把 21 当成全部 —— 这就是「声称 vs 实际」的失真。
    # 本轮实测：默认 limit 下报告 167、实显 21，差 146 条看不见。
    # 而 `compressor.rs`(7 测试) / `nt_crypto_util.rs`(3 测试) 恰好在被截断的那段里
    # ⇒ 不提高 limit 就等于「工具没发现」。
    if len(shown) < len(rows):
        print('   ⛔ **仅显示前 %d 条，共 %d 个孤儿**（另 %d 条未显示）。'
              % (len(shown), len(rows), len(rows) - len(shown)))
        print('      看全部请加 `--limit %d`（或更大）；**上面那个「孤儿 %d」才是总数**，'
              % (len(rows), len(orphans)))
        print('      本行数字才是你实际看到的 —— 别把显示条数当总数。')
    if rows:
        print('[orphan] 判读要点：')
        print('  · 孤儿文件**不参与编译**，其测试也从不运行 ⇒ 它们的历史绿灯是假的；')
        print('  · ⛔ **不代表可以删**：可能被 `include!` 引入，或被 feature 门控的')
        print('    `mod` 声明引用（静态不可判定）⇒ 逐条核实后再动。')
        print('  · 实例：neotrix-core/src/l6_meta/nt_core_qtest.rs（538 行 / 15 测试）')
    return len(orphans)


# ── 被注释掉的 mod 声明检测（2026-09-30 新增）────────────────────────────
# 起因（本轮最贵的一课）：`nt_memory/mod.rs` 里一行注释
# 「以下模块已声明但内部编译错误待修复」下挂 4 个模块，导致
# **4 个模块 / 约 4,300 行 / 129 个从不运行的测试**静默消失。
# 其中 `hybrid_retrieval` 实测 **0 编译错误** —— 纯属**笼统注释的误伤**。
#
# ⚠️ 为什么 `--orphan` 抓不到：它报「文件不可达」，但**不解释原因**。
# 而「被注释掉的 mod 声明」正是最常见的**可修复**成因 ——
# 它是一个**明确的、单行的、可撤销的开关**，比「忘记声明」好处理得多。
#
# 判据：`// pub mod X;` / `//mod X;` 形式，且对应文件在磁盘上存在。
#   ⇒ 若文件不存在，说明是纯注释残留（无害），不报。
# ⛔ 不判删：注释掉可能是有意的（等 ABI 决策 / 等修 bug），
#    本工具只**把候选连同「开启后是否编译通过」的事实**呈现出来。

_COMMENTED_MOD = re.compile(
    r'(?:^|\n)[ \t]*//[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?mod[ \t]+([A-Za-z_][A-Za-z0-9_]*)[ \t]*;'
)


def report_commented_mods(fd, roots, limit):
    """被注释掉的 mod 声明，且对应文件确实存在。"""
    global _CM_ROWS
    rows = []
    for root in roots:
        base = os.path.join(REPO, root) if os.path.isabs(root) else root
        if not os.path.isdir(base):
            continue
        for dirpath, _dirs, files in os.walk(base):
            norm = dirpath.replace(os.sep, '/')
            if '/target' in norm or '/.worktrees' in norm:
                continue
            for f in sorted(files):
                # ⚠️ 这里**不能**跳过 `mod.rs`：被注释掉的 mod 声明
                # **恰恰绝大多数就写在 mod.rs 里**（Rust 的模块声明
                # 惯例位置）。我第一版写了 `f == 'mod.rs': continue`
                # —— 结果 0 命中，且原因极隐蔽：正则明明能匹配、
                # 文件明明存在，只是被这个过滤条件挡在门外。
                # ⇒ 教训与本工具早期 bug 同源：**在错误的地方过滤**。
                if not f.endswith('.rs'):
                    continue
                p = os.path.relpath(os.path.join(dirpath, f), REPO)
                try:
                    # ⚠️ 这里必须读**原文**、不能用 `strip_noise`：
                    # 本检查要找的**正是注释里的文本**，而 `strip_noise`
                    # 的职责就是把注释剥掉 ⇒ 用它会得到 0 命中（我第一版
                    # 就踩了这个坑：判据「看起来对」但恒为假）。
                    # 代价是字符串字面量里的 `// pub mod X;` 会假阳性，
                    # 故下面额外要求该行**去掉 `//` 后就是一个纯 mod 声明**。
                    with open(os.path.join(REPO, p), encoding='utf-8', errors='replace') as fh:
                        raw = fh.read()
                except OSError:
                    continue
                for m in _COMMENTED_MOD.finditer(raw):
                    name = m.group(1)
                    cands = [os.path.join(dirpath, name + '.rs'),
                             os.path.join(dirpath, name, 'mod.rs')]
                    tgt = next((c for c in cands if os.path.isfile(c)), None)
                    if not tgt:
                        continue  # 文件不存在 ⇒ 纯注释残留，无害
                    nfiles = 1
                    nlines = 0
                    ntests = 0
                    if os.path.basename(tgt) == 'mod.rs':
                        sub = os.path.dirname(tgt)
                        if os.path.isdir(sub):
                            fs2 = [os.path.join(sub, x) for x in os.listdir(sub)
                                   if x.endswith('.rs')]
                            nfiles += len(fs2)
                            for x in fs2:
                                try:
                                    txt = open(x, encoding='utf-8',
                                               errors='replace').read()
                                except OSError:
                                    continue
                                nlines += txt.count('\n')
                                ntests += len(re.findall(r'#\[test\]', txt))
                    try:
                        txt = open(tgt, encoding='utf-8', errors='replace').read()
                        nlines += txt.count('\n')
                        ntests += len(re.findall(r'#\[test\]', txt))
                    except OSError:
                        pass
                    rows.append({
                        'decl': p, 'line': raw.count('\n', 0, m.start()) + 1,
                        'name': name, 'nfiles': nfiles, 'nlines': nlines,
                        'ntests': ntests,
                    })
    rows.sort(key=lambda r: -r['ntests'])
    globals()['_CM_ROWS'] = rows
    print('[commented-mod] 「注释掉的 mod 声明」且文件存在：%d 处' % len(rows))
    for r in rows[:limit]:
        print('   %s:%d  // pub mod %s;   → %d 文件 / 约 %d 行 / **%d 个从不运行的测试**'
              % (r['decl'], r['line'], r['name'], r['nfiles'], r['nlines'], r['ntests']))
    if rows:
        print('[commented-mod] 判读要点：')
        print('  · 这是 `--orphan` 里**最可修复的一类成因**：它是一个明确的开关，')
        print('    而「忘记声明」往往无从判断意图。')
        print('  · ⚠️ 注释掉可能**是有意的**（等 ABI 决策 / 等修 bug / 暂时的），')
        print('    ⛔ 本工具**不判删**，只要求逐条核实「开启后是否编译通过」。')
        print('  · 实例：`nt_memory/mod.rs` 曾用**一行笼统注释**关掉 4 个模块，')
        print('    其中 `hybrid_retrieval` 实测 0 错误（纯误伤）。')
    return len(rows)

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
    print()
    report_dup_impls(fd, [root for _c, root in CRATES], args.limit)
    print()
    report_inverted_hierarchy(fd, [root for _c, root in CRATES], args.limit)
    print()
    report_orphan_files([root for _c, root in CRATES], args.limit, fd)
    print()
    report_commented_mods(fd, [root for _c, root in CRATES], args.limit)
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

    # 3b) dup-impl 判据回归（2026-09-30）：三次判据修正留下的坑，全部固化成用例。
    #  · v1 只判「方法集不相等」 ⇒ 同名巧合全报（实测 135 组）
    #  · v2 判 0<J<1 ⇒ 只共享 1 个方法也报（94 组）
    #  · v3 判 inter>=3 且 J>=0.5 ⇒ **漏掉已知真例 IITPhiCalculator**
    #    （它有三份副本，all-intersection 被离群副本拉低到 J=0.38）
    #  · v4 最终：以方法最多者为基准 + 「缺方法」+ pairwise J>=0.4
    # 用 %s 放文件名：**不能用 str.format**，Rust 源码里满是 `{}`，
    # format 会把它们当占位符 → ValueError（写这个自证时当场踩到）。
    src_a = ("pub struct %s {}\nimpl %s {\n"
             "  pub fn a(&self){}\n  pub fn b(&self){}\n"
             "  pub fn c(&self){}\n  pub fn d(&self){}\n}\n")
    src_stale = ("pub struct %s {}\nimpl %s {\n"
                 "  pub fn a(&self){}\n  pub fn b(&self){}\n}\n")
    src_coincide = ("pub struct %s {}\nimpl %s {\n"
                    "  pub fn zzz(&self){}\n}\n")
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        def _w(fn, txt):
            fp = os.path.join(td, fn)
            with open(fp, 'w', encoding='utf-8') as fh:
                fh.write(txt % ('T', 'T'))
            return fp
        pa = _w('ref.rs', src_a)
        ps = _w('stale.rs', src_stale)
        pc = _w('coincide.rs', src_coincide)
        m_ref = impl_methods(fd, pa).get('T', set())
        m_stale = impl_methods(fd, ps).get('T', set())
        m_co = impl_methods(fd, pc).get('T', set())
        check(len(m_ref) == 4 and len(m_stale) == 2 and len(m_co) == 1,
              'impl_methods 抽取不对: %r %r %r' % (m_ref, m_stale, m_co))
        # 陈旧分叉：缺 2 个方法且 J = 2/4 = 0.5 ⇒ 必须判为分叉
        miss = m_ref - m_stale
        jac = len(m_ref & m_stale) / len(m_ref | m_stale)
        check(len(miss) == 2 and jac >= 0.4,
              'stale fork must satisfy criterion: miss=%r jac=%.2f' % (miss, jac))
        # 同名巧合：J 极低 ⇒ 必须**不**判为分叉
        jac_co = len(m_ref & m_co) / len(m_ref | m_co)
        check(jac_co < 0.4,
              'coincidental same-name must NOT satisfy criterion: jac=%.2f' % jac_co)

    # 3c) 可见性回归（2026-09-30 真实误报）：首版只收 `pub fn`，
    # 把「方法存在但私有」误报成「缺方法」。实测踩中 `WalshMemoryIndex`：
    # l0 全 12 个 pub，types/l5 各 9 pub + 4 私有 ⇒ 首版报「缺 3 个」是错的。
    src_pub = "pub struct %s {}\nimpl %s {\n  pub fn a(&self){}\n  pub fn b(&self){}\n}\n"
    src_priv = "pub struct %s {}\nimpl %s {\n  pub fn a(&self){}\n  fn b(&self){}\n}\n"
    with tempfile.TemporaryDirectory() as td:
        def _w2(fn, txt):
            fp = os.path.join(td, fn)
            with open(fp, 'w', encoding='utf-8') as fh:
                fh.write(txt % ('T', 'T'))
            return fp
        pp = _w2('pub.rs', src_pub)
        pr = _w2('priv.rs', src_priv)
        pub_only = impl_methods(fd, pp).get('T', set())
        full = impl_methods(fd, pr, all_visibility=True).get('T', set())
        pub_only2 = impl_methods(fd, pr).get('T', set())
        vis = impl_visibility(fd, pr).get('T', {})
        check(len(pub_only) == 2, 'pub-only 抽取错: %r' % pub_only)
        check(len(full) == 2, 'all_visibility 抽取错: %r' % full)
        check(pub_only2 == {'a'},
              '私有方法不得进 pub-only 集合: %r' % pub_only2)
        check(vis.get('b') == 'priv' and vis.get('a') == 'pub',
              '可见性标注错: %r' % vis)
        # 关键回归：默认口径下两者 pub 集不同，但**全量口径下方法集相同**
        # ⇒ 「缺方法」不成立，只能报「可见性不同」
        check(len(full) == len(pub_only) and pub_only2 != pub_only,
              '可见性场景未被区分')

    # 3d) commented-mod 判据回归（2026-09-30）：
    # ① 本检查必须读**原文**（要找的就是注释里的文本），用 strip_noise 会恒为 0 命中；
    # ② 绝**不能**跳过 mod.rs（被注释的声明恰恰都写在 mod.rs 里）——
    #    这两条我都先写错过一轮，症状都是「正则能匹配但工具报 0」。
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        sub = os.path.join(td, 'demo')
        os.makedirs(os.path.join(sub, 'widget', 'deep'))
        # 注释掉的声明写在 mod.rs 里（真实形态）
        with open(os.path.join(sub, 'mod.rs'), 'w', encoding='utf-8') as fh:
            fh.write('pub mod live_thing;\n'
                     '// pub mod dead_thing;\n'
                     '//mod other_dead;\n')
        with open(os.path.join(sub, 'live_thing.rs'), 'w', encoding='utf-8') as fh:
            fh.write('pub fn ok() {}\n')
        os.makedirs(os.path.join(sub, 'dead_thing'))
        with open(os.path.join(sub, 'dead_thing', 'mod.rs'), 'w', encoding='utf-8') as fh:
            fh.write('pub fn d() {}\n#[test]\nfn t() {}\n')
        os.makedirs(os.path.join(sub, 'other_dead'))
        with open(os.path.join(sub, 'other_dead', 'mod.rs'), 'w', encoding='utf-8') as fh:
            fh.write('pub fn o() {}\n')
        # 不存在的模块（纯注释残留）不应报
        with open(os.path.join(sub, 'extra.rs'), 'w', encoding='utf-8') as fh:
            fh.write('// pub mod never_existed;\n')
        captured = []
        _orig_print = print
        import builtins
        builtins.print = lambda *a, **k: captured.append(' '.join(str(x) for x in a))
        try:
            n = report_commented_mods(fd, [td], 20)
        finally:
            builtins.print = _orig_print
        names = sorted(r['name'] for r in _CM_ROWS)
        check(n == 2, '应报 2 处（dead_thing / other_dead），实得 %d: %r' % (n, names))
        check('never_existed' not in names,
              '文件不存在的纯注释残留不应报: %r' % names)
        check('live_thing' not in names, '未注释的模块不应被报')

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