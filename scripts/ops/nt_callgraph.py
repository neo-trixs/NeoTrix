#!/usr/bin/env python3
"""nt_callgraph.py — 调用边图引擎：传递可达性 / 影响面（blast radius）.

为什么独立于 nt_calledges.py：抽取（compiler 委托，~30min 低频生成物）与
**查询**（秒级、反复调用）是两种负载。抽取器保持单文件可审计；本模块只读
`.project-map/edges-*.jsonl`，可被反复调用而不触发重抽取。

吸收来源与判定（2026-09-30 实测，非手推）：
  - `codebase-rag` 的 `impact_of` / `grafel` 的反向可达 → 需要**传递**闭包，
    而 nt_calledges 只有一跳子串扫描。本模块是它的直接解法（G4）。
  - `zvec-ai/zvec-grep` 的 `fresh` / `possibly_stale` 结果标记 → 边表是
    **时点快照**，任何边的「今天是否还成立」都不可知。本模块用
    `mtime(edges) < mtime(源文件)` 给出**诚实的下界标注**，不假装新鲜。

**关键限制（必须随输出一起给出，否则就是骗人）**：
  静态 call 边对 **async task / trait 动态派发 / 宏展开** 是盲区。
  实测：`drain_outbox_once` 只有 1 个直接调用者，且该调用者无上游 ⇒
  传递闭包在此终止。⇒ `--impact` 报的是**静态可达下界**，
  真实影响面 ⊇ 本结果。绝不可当「改这里只有这些受影响」用。

用法：
  # 影响面：谁（传递地）依赖 <target>。改 target 前跑这个。
  python3 scripts/ops/nt_callgraph.py --impact <path-substr> --db <edges.jsonl> \\
      [--depth N] [--files] [--stale]

  # 依赖面：<target> 传递地依赖谁（正向）。
  python3 scripts/ops/nt_callgraph.py --deps <path-substr> --db <edges.jsonl> [--depth N]

  # 图统计：深度分布 / 跨 crate 边 / 孤立节点。用来判断判据有效性。
  python3 scripts/ops/nt_callgraph.py --stats --db <edges.jsonl>

  # 不可达报告（不是门！见下）。零入边 + 全仓无调用点。
  python3 scripts/ops/nt_callgraph.py --unreachable --db <edges.jsonl> [--crate neotrix]

输出 TSV（`--files` 时按文件聚合，附最浅深度 = 最小爆炸半径）。

⛔ **不要给本模块的输出建门**（`CAPABILITY-GAP-2026-09-30.md` G7 裁决：
工具自述不可靠，门会立刻失去可信度）。本模块是**报告器**。

**实测得到的 5 类误报源**（2026-09-30，全部由本会话亲手踩出来并修掉，
`--unreachable` 已把它们编码为守卫；写新的 grep 式可达性判定前必读）：
  F1 注释行：`/// all_native_tools() 的来源` 含 `name(` 形态 → 被当调用点。
     本仓把禁词/签名当数据持有（`nt_meta/scanner.rs` 扫禁词），注释命中全是误报
     —— 即 R-SCAN-1b。
  F2 basename 碰撞：`plans.is_empty()` / `report.error_rate()` 与我方
     `is_empty` / `error_rate` 同名不同类型。子串搜索 100% 误报。
     ⇒ 这正是编译期边的价值：DefId 带 crate 哈希，结构上不可能混淆。
  F3 fn-as-value：`.map(block_to_text)` 是把函数当值传递，grep 找 `name(` 找不到，
     但 THIR 里有真边。
  F4 `pub use` 重导出：`mod.rs:87 use …::register_trade_full_cycle_capability`
     是重导出不是调用。**「导出 ≠ 接入」**（R-P79 同族）。
  F5 传递性死：private fn 只被另一个 dead fn 调用 ⇒ 连环断链，二者都零入边。
     此时「文本有调用点」但**生产不可达** —— 才是本工具要找的真死链。
  F6 **图自身的盲区（本轮实测发现，最要紧）**：`.map(f)` / `.filter(f)` 这类
     **fn-as-value** 传递**不产生指向 f 的 Call 边** —— THIR 里该调用点的
     DefId 归到匿名闭包，不归 f。证据：`doc_parse::blocks_to_text` /
     `items_to_text` / `cell_text_inner` 反向可达分别 11/1/17，而它们的
     被调方 `block_to_text` 反向可达 **0**，尽管 `block_to_text` 在
     `doc_parse.rs:204` 以 `.map(block_to_text)` 形态被真实使用。

⛔ **F6 的覆盖是「已穷举」吗？不是，而且实测证明做不到。**
`_fn_as_value_hits` 的判据经**五版**才收敛，每一版都是被手验推翻的：
  v1 只认迭代器适配器      → 漏 axum/clap 注册式（`get(h)`、`from_fn(h)`）
  v2 要求 fn 紧跟 `(`      → 漏 `from_fn_with_state(state, h)`（fn 是末位实参）
  v3 用 `[^)]*` 跨实参      → 漏 `state.clone()` 的 `)` 截断
  v4 一层括号配平          → 漏 rustfmt 拆行（`-U` 多行才解决）
  v5 补回调组合子 + 领域回调 → 3/3 手验正确
误报率轨迹：**100% → 67% → 0%**（对象从 36 条缩到 3 条）。
⇒ **推论**：任何「零调用 ⇒ 死」的判据在本仓都不可能一次做对，
每修一族就暴露新的一族。**残余风险是「未知族仍存在」，不是「零风险」。**
⇒ 这正是 `CAPABILITY-GAP-2026-09-30` G7「不要给死代码建门」的**实证**：
   本文件因此是**报告器**，suspect-dead 桶**不可直接行动**。
"""
import array
import json
import os
import re
import subprocess
import sys
from collections import deque

SEP = '::'


def _crate_of(node):
    """DefId 路径的 crate 段：`neotrix[7f32]::a::b` → `neotrix[7f32]`。"""
    return node.split(SEP, 1)[0]


class Graph:
    """双向邻接。字符串池 + int 索引：670K 边实测常驻 ~150MB 以内。"""

    def __init__(self):
        self.ids = {}
        self.nodes = []
        self.fwd = {}   # id -> array('i')  callees
        self.rev = {}   # id -> array('i')  callers
        self.nedges = 0
        self.bad = 0

    def _id(self, name):
        i = self.ids.get(name)
        if i is None:
            i = len(self.nodes)
            self.ids[name] = i
            self.nodes.append(name)
        return i

    def add(self, caller, callee):
        a, b = self._id(caller), self._id(callee)
        f = self.fwd.get(a)
        if f is None:
            f = self.fwd[a] = array.array('i')
        f.append(b)
        r = self.rev.get(b)
        if r is None:
            r = self.rev[b] = array.array('i')
        r.append(a)
        self.nedges += 1

    def load(self, path):
        with open(path, encoding='utf-8', errors='ignore') as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    e = json.loads(line)
                    c, k = e['caller'], e['callee']
                except (ValueError, KeyError, TypeError):
                    self.bad += 1          # 宁缺勿错：坏行丢弃，不猜边
                    continue
                self.add(c, k)
        return self

    def bfs(self, seeds, adj, max_depth):
        """从 seeds 出发的可达闭包。返回 {id: 最浅深度}（不含 seed 自身）。"""
        seen = {}
        q = deque()
        for s in seeds:
            if s not in seen:
                seen[s] = 0
                q.append(s)
        while q:
            cur = q.popleft()
            d = seen[cur]
            if max_depth and d >= max_depth:
                continue
            for nxt in adj.get(cur, ()):
                if nxt not in seen:
                    seen[nxt] = d + 1
                    q.append(nxt)
        return seen

    def seeds_matching(self, substr):
        return [i for n, i in self.ids.items() if substr in n]

    def spans_for(self, ids):
        """节点 → 源码 span。边表里 span 记在 caller 侧，故取该节点任一出现处。"""
        out = {}
        return out


def load_graph(db):
    if not os.path.isfile(db):
        sys.exit(f"[callgraph] 边表不存在：{db}\n"
                 f"  生成：python3 scripts/ops/nt_calledges.py --crate <dir> --out …")
    g = Graph().load(db)
    print(f"[callgraph] {db}: {g.nedges} edges, {len(g.nodes)} nodes, "
          f"{g.bad} bad lines dropped", file=sys.stderr)
    return g


def staleness(g, db, limit=8):
    """边表 vs 源文件 mtime。返回 (检查文件数, 过期文件数, 样本)。

    ⚠️ **实现陷阱（2026-09-30 亲手踩到并修）**：第一版从 DefId 路径段推文件
    （`l1_action::nt_action_facade::{impl#3}::store`），但那些段是**模块路径**，
    永远不含 `/` ⇒ 循环体一次都不执行 ⇒ 恒返回 `0 个过期`。
    **一个从不检查却报「0 过期」的工具，比没有工具更坏**（本仓「零命中不是零消费」
    纪律的同族：无声的安心 = 最贵的安心）。

    正确来源是边表自带的 `span` 字段（真实文件路径，如
    `crates/neotrix-audit/src/nt_baseline.rs:17:10:`），已由抽取器保证落盘。
    """
    db_mtime = os.path.getmtime(db)
    files = {}
    with open(db, encoding='utf-8', errors='ignore') as fh:
        for line in fh:
            try:
                sp = json.loads(line)['span']
            except (ValueError, KeyError, TypeError):
                continue
            p = sp.split(':', 1)[0].strip()          # `path:line:col: line:col (#id)`
            if p.endswith('.rs'):
                files[p] = True
    checked = stale = 0
    sample = []
    for p in files:
        if not os.path.isfile(p):
            continue
        checked += 1
        if os.path.getmtime(p) > db_mtime:
            stale += 1
            if len(sample) < limit:
                sample.append(p)
    return checked, stale, sample



def _source_file(node):
    """从 DefId 路径取源文件名：`…::l6_meta::nt_laws::{impl#3}::check` → `nt_laws.rs`。

    必须**跳过 `{impl#N}` / closure 段** —— 直接取 [-2] 会产出 `{impl#6}.rs`
    这种不存在的文件名（实测踩到过）。
    """
    segs = node.split(SEP)[1:]          # 去掉 crate 段
    for s in reversed(segs[:-1] if len(segs) > 1 else segs):
        if s.startswith('{impl#') or s.startswith('{closure') or s == 'closure':
            continue
        if s.startswith('{') or not s:
            continue
        return s if s.endswith('.rs') else s + '.rs'
    return None


def emit(g, reach, args, label):
    rows = []
    for nid, depth in reach.items():
        if depth == 0:
            continue                      # seed 自身
        rows.append((depth, g.nodes[nid]))
    if getattr(args, 'files', False):
        # 按文件聚合：取该文件内最浅深度 = 最小爆炸半径
        best = {}
        for depth, node in rows:
            f = _source_file(node)
            if f is None:
                continue
            if f not in best or depth < best[f]:
                best[f] = depth
        for f, d in sorted(best.items(), key=lambda kv: (kv[1], kv[0])):
            print(f"{d}\t{f}")
        print(f"[callgraph] {label}: {len(best)} files, {len(rows)} nodes",
              file=sys.stderr)
    else:
        for depth, node in sorted(rows):
            print(f"{depth}\t{node}")
        print(f"[callgraph] {label}: {len(rows)} nodes at depth ≤{args.depth or '∞'}",
              file=sys.stderr)


def cmd_impact(g, args):
    """反向：谁传递地依赖 target。改 target 前的影响面。"""
    seeds = g.seeds_matching(args.target)
    if not seeds:
        print(f"[callgraph] 边表中无节点匹配 {args.target!r}；"
              f"注意边表是时点快照，不代表符号不存在", file=sys.stderr)
        return 3
    print(f"[callgraph] {len(seeds)} seed(s) 匹配 {args.target!r}", file=sys.stderr)
    reach = g.bfs(seeds, g.rev, args.depth)
    emit(g, reach, args, 'impact(静态可达下界)')
    if args.stale:
        checked, s, sample = staleness(g, args.db)
        if checked == 0:
            print(f"[callgraph] ⚠ staleness **未检查任何文件**（边表无可解析 span）—— "
                  f"不得读作「新鲜」", file=sys.stderr)
        elif s:
            print(f"[callgraph] staleness: {s}/{checked} 个源文件新于边表 "
                  f"⇒ 本结果可能偏低", file=sys.stderr)
            for p in sample:
                print(f"    stale? {p}", file=sys.stderr)
        else:
            print(f"[callgraph] staleness: {checked} 个文件全部早于边表 ⇒ 快照新鲜",
                  file=sys.stderr)
    return 0


def cmd_deps(g, args):
    seeds = g.seeds_matching(args.target)
    if not seeds:
        print(f"[callgraph] 边表中无节点匹配 {args.target!r}", file=sys.stderr)
        return 3
    reach = g.bfs(seeds, g.fwd, args.depth)
    emit(g, reach, args, 'deps')
    return 0


def cmd_stats(g, args):
    """深度分布 + 跨 crate 边 —— 用来判断「传递闭包是否真的有信息量」。"""
    xcrate = 0
    for c, callees in g.fwd.items():
        cn = _crate_of(g.nodes[c])
        for k in callees:
            if _crate_of(g.nodes[k]) != cn:
                xcrate += 1
    print(f"nodes\t{len(g.nodes)}")
    print(f"edges\t{g.nedges}")
    print(f"bad_lines_dropped\t{g.bad}")
    print(f"cross_crate_edges\t{xcrate}")
    # 深度分布：从「无入边」的节点（候选入口）正向 BFS
    indeg = {n: 0 for n in range(len(g.nodes))}
    for callees in g.fwd.values():
        for k in callees:
            indeg[k] += 1
    entries = [n for n, d in indeg.items() if d == 0]
    hist = {}
    for n in entries:
        for _, d in g.bfs([n], g.fwd, 12).items():
            hist[d] = hist.get(d, 0) + 1
    for d in sorted(hist):
        print(f"fwd_depth_{d}\t{hist[d]}")
    print(f"entry_nodes_no_inbound\t{len(entries)}")
    return 0


# ── 不可达报告 ──────────────────────────────────────────────────────────
# ⛔ 报告器，不是门（G7 裁决）。每条结论必须人工读代码证实/证伪后才能动（R-SCAN-1）。
#
# 本函数只信**编译期边**做判定，不用 grep 做可达性 —— 因为 grep 有 F1..F4 四类
# 结构性误报（见模块 docstring），实测 200 样本里 100% 误报率。
# 文本搜索在此**只用于标注**，不用于判定。

RE_COMMENT = re.compile(r'^\s*(///|//!|//|\*|/\*)')


def _is_natural_leaf(name):
    """天然无调用者的形态：trait 实现体、闭包、main、Drop/Clone 转发、
    转换器（from_/into_/as_）。不判死。"""
    tail = name.split(SEP)[-1]
    return ('{impl#' in name
            or 'closure' in tail
            or tail in ('main', 'drop', 'fmt', 'clone', 'eq', 'hash', 'default',
                        'clone_from')
            or tail.startswith(('from_', 'into_', 'as_', 'to_')))


def _is_static_like(name):
    """static / const：访问走 `.read()`/`.get()`，不产生以自身为目标的 Call 边。"""
    tail = name.split(SEP)[-1]
    return tail.isupper() or tail.startswith('GLOBAL_') or tail.endswith('_ID')


def _src_roots():
    return ('neotrix-core/src', 'crates', 'apps', 'evals', 'scripts')


def _cfg_test_lines(path):
    """`#[cfg(test)]` 覆盖的行号集合。**花括号配平**，不用行号启发式
    （本会话先用了 60 行回看窗口，实测把 test 调用误判为生产调用）。"""
    try:
        lines = open(path, encoding='utf-8', errors='ignore').read().splitlines()
    except OSError:
        return set()
    out = set()
    i = 0
    while i < len(lines):
        if 'cfg(test)' in lines[i]:
            depth = 0
            started = False
            j = i
            for j in range(i, min(i + 4000, len(lines))):
                for ch in lines[j]:
                    if ch == '{':
                        depth += 1
                        started = True
                    elif ch == '}':
                        depth -= 1
                out.add(j + 1)
                if started and depth <= 0:
                    break
            i = j
        i += 1
    return out


def _text_call_sites(base, root):
    """标注用：文本里的调用点，**已排除注释行（F1）与定义行**。
    返回 (生产点数, 测试点数)。判定仍以图为准，本函数只提供证据标签。"""
    prod = test = 0
    cache = {}
    try:
        r = subprocess.run(['rg', '-n', '--no-heading', '-F', '--', base] + list(root),
                           capture_output=True, text=True, timeout=60)
    except (subprocess.SubprocessError, FileNotFoundError):
        return -1, -1
    for line in r.stdout.splitlines():
        m = re.match(r'^(\.?/?[^:]+):(\d+):(.*)$', line)
        if not m:
            continue
        f, ln, body = m.group(1), int(m.group(2)), m.group(3)
        f = f[2:] if f.startswith('./') else f
        if RE_COMMENT.match(body) or re.search(r'\bfn\s+' + re.escape(base), body):
            continue                      # F1：注释 / 定义行不算
        if not re.search(re.escape(base) + r'\s*[\(\!\.]', body):
            continue                      # F3：fn-as-value（.map(f)）此处不匹配
        # 只在 .rs 里佐证：图是 Rust 调用图，工具/门文档（.md/.py/.json）引用
        # Rust 符号名属范畴错误。实测本模块自己的 docstring 就引用了
        # all_native_tools() 作 F1 范例，把自己算成了「生产调用点」。
        if not f.endswith('.rs'):
            continue
        if not os.path.isfile(f):
            continue
        if f not in cache:
            cache[f] = _cfg_test_lines(f)
        if ln in cache[f]:
            test += 1
        else:
            prod += 1
    return prod, test


MODULE_ROOTS = ('neotrix-core/src', 'crates', 'apps')


def _module_path(node):
    """DefId 路径 → 候选源文件相对路径（去掉 crate 段与末位函数名，跳过 impl/closure 段）。

    `neotrix[7f32]::l1_action::nt_file_ability::doc_parse::parse_batch`
      → `l1_action/nt_file_ability/doc_parse.rs`
    """
    segs = node.split(SEP)[1:]
    mods = []
    for s in segs[:-1]:
        if s.startswith('{impl#') or s.startswith('{closure') or s == 'closure':
            continue
        if s.startswith('{'):
            continue
        mods.append(s)
    return '/'.join(mods) + '.rs' if mods else None


def _definition_visibility(node):
    """该函数的可见性：`public` / `private` / `unknown`。

    这是把「308 条待人工裁决」变成「高置信死链 + 公共面待裁」的关键机械判据：
      - **private** + 零入边 + 全仓无调用点 ⇒ 外部不可能用（private 不出模块），
        宏注册的可能性另由 F4/F5 承担 ⇒ **高置信死链**。
      - **public** + 同上 ⇒ 属于公共 API 面，可能是给仓外消费者用的
        ⇒ **不能**据此判死，归入待裁。
    判不出来就返回 unknown（宁缺勿错，不猜）。
    """
    mp = _module_path(node)
    base = node.split(SEP)[-1]
    if not mp or not base:
        return 'unknown'
    rel = None
    for root in MODULE_ROOTS:
        cand = os.path.join(root, mp)
        if os.path.isfile(cand):
            rel = cand
            break
    if rel is None:
        return 'unknown'
    try:
        lines = open(rel, encoding='utf-8', errors='ignore').read().splitlines()
    except OSError:
        return 'unknown'
    # 只认定义行（排除 impl/closure 体里的同名局部定义无法区分，取首个 fn 定义）
    pat = re.compile(r'^(\s*)(pub(\([a-z()\s]+\))?\s+)?(async\s+)?(unsafe\s+)?fn\s+'
                     + re.escape(base) + r'\b')
    for line in lines:
        if RE_COMMENT.match(line):
            continue
        m = pat.match(line)
        if m:
            return 'public' if m.group(2) else 'private'
    return 'unknown'


def _fn_as_value_hits(base, root):
    r"""F3/F6 探测：base 是否以 **fn 值** 形态被传递（原始字符串：内含 `\(`）。

    ⚠️ 2026-09-30 **手验推翻**了本函数的第一版覆盖范围。第一版只认
    `map|filter|for_each|and_then|…` 这类迭代器适配器，于是把三条**活代码**
    误判进 dead-private：
        `middleware::from_fn_with_state(state, auth_middleware)`
        `.route("/health",  get(health_handler))`
        `.route("/v1/models", get(models_handler))`
    ⇒ axum/actix/warp 的**注册式 API**（handler / middleware / service / route）
    与 clap 的 `command()` 同属「按值注册」，静态调用边一律看不见。
    对一个既做 HTTP 又做 CLI 的 agent 框架，这类接线占比很大。

    由此得到本工具最重要的结论：
      **`--unreachable` 无法产出「高置信死链」桶。** 任何这样的桶都会被
      注册式 API 污染（实测抽样 3/3 全错）。故桶名从 `dead-private` 改为
      `suspect-dead`，并在命中注册式 API 时另标 `registered` ——
      **宁可疑似活的误报（只多花点人工），也不可疑似死的误报（会删活代码）。**

    ⚠️ 第三版才真正成立。前两版都被**实测**推翻，不是想出来的：
      v1 只认迭代器适配器 ⇒ 漏掉 axum/clap 注册式 API
         （`get(health_handler)` / `from_fn(base)`）
      v2 改成「实参里任意位置出现 base」，用 `[^)]*` ⇒ 仍漏
         `from_fn_with_state(state.clone(), auth_middleware)`
         因为 `state.clone()` 里的 `)` 截断了 `[^)]*`。
      ⇒ 且**我原先写的「漏判属安全方向」是错的**：漏判的后果是
         一个**活的** middleware 被放进 suspect-dead，即**假死**判据 ——
         那恰恰是危险方向，不是安全方向。**判据的方向性不能靠推测，
         必须看漏判的后果落在哪一边。**
      v3 用「一层括号配平」`(?:[^()]|\([^()]*\))*` 跨过 `state.clone()`。
         更深嵌套（`wrap(make(bar(a)), base)`）仍会漏 ⇒ 所以 suspect-dead
         桶**永远不可直接行动**，必须读代码。
    """
    pat = re.compile(r'\b(?:map|filter|for_each|and_then|or_else|unwrap_or_else|'
                     r'ok_or_else|unwrap_or|inspect|map_err'
                     # 回调式组合子（fn 作为值传入）—— v5 补：实测
                     # `query_map(params![id], row_to_fact)`（rusqlite）与
                     # `map_or(false, is_index_token)` 都是活的，但前四版都没覆盖
                     r'|map_or|map_or_else|unwrap_or_else|fold|find_map|flat_map'
                     r'|any|all|position|max_by|min_by|sort_by_key|reduce'
                     # 注册式 API（web / CLI / DI / actor）
                     r'|from_fn|from_fn_with_state|route|nest|nest_service|merge'
                     r'|get|post|put|delete|patch|head|options|any|method|handler'
                     r'|layer|wrap_fn|service|mount|command|subcommand|table|group'
                     # 领域回调（DB / 序列化 / 任务）：无法穷举，见下方结论
                     r'|query_map|query_and_then|try_fold|spawn|dispatch'
                     r')\s*\((?:[^()]|\([^()]*\))*' + re.escape(base) + r'\b')

    try:
        # `-U`（multiline）是必需的，不是优化：rustfmt 会把多实参注册拆行，
        # 实测 `server.rs:195` 是
        #     .layer(middleware::from_fn_with_state(
        #         state.clone(),
        #         rate_limit_middleware,
        #     ))
        # fn 在**另一行** ⇒ 逐行 rg 匹配不到 ⇒ 又被误放进 suspect-dead。
        # 这是本函数第三次被实测推翻（v1 适配器 / v2 位置 / v3 括号配平 / v4 跨行）。
        r = subprocess.run(['rg', '-U', '-n', '--no-heading', '--glob', '*.rs',
                            '-e', pat.pattern, '--'] + list(root),
                           capture_output=True, text=True, timeout=60)
    except (subprocess.SubprocessError, FileNotFoundError):
        return -1
    if r.returncode not in (0, 1):
        return -1
    return len([l for l in r.stdout.splitlines() if l.strip()])


def cmd_unreachable(g, args):
    """零入边节点分类报告。分类只依据「是否 static/闭包/trait impl/转换器」，
    可达性判定完全交给图 —— 因为 grep 的可达性判定实测 100% 误报。"""
    scope = args.scope
    nodes = [n for n in g.nodes
             if (scope is None or n.startswith(scope))
             and g.fwd.get(g.ids[n])          # 有出边 ⇒ 至少定义过一个函数
             and not g.rev.get(g.ids[n])]     # 零入边
    root = _src_roots()
    cats = {'test-only': [], 'no-callsite': [], 'textual-prod': [],
            'registered': [], 'unwired-public': [], 'suspect-dead': [],
            'static-like': [], 'natural-leaf': []}
    for n in sorted(nodes):
        base = n.split(SEP)[-1]
        if _is_natural_leaf(n):
            cats['natural-leaf'].append((n, 'trait impl/closure/main/Drop/转换器'))
        elif _is_static_like(base):
            cats['static-like'].append((n, 'static/const：访问不产生自身 Call 边'))
        else:
            p, t = _text_call_sites(base, root)
            if p < 0:
                cats['no-callsite'].append((n, 'rg 执行失败，不可判'))
            elif p > 0:
                # 图说不可达但文本有生产调用点 → 图的已知盲区，需人工裁决
                cats['textual-prod'].append((n, f'文本有 {p} 处生产调用点但图零入边'))
            elif t > 0:
                cats['test-only'].append((n, f'仅 {t} 处 #[cfg(test)] 调用（--lib 不含测试）'))
            else:
                f3 = _fn_as_value_hits(base, root)
                vis = _definition_visibility(n)
                if f3 > 0:
                    # 注册式 / 适配器按值接线 ⇒ 图与 grep 都查不到，但**它是活的**
                    cats['registered'].append(
                        (n, f'{f3} 处按值接线（.map(f) 或 web/CLI/DI 注册 API）'
                            f'—— 静态边看不见，**不可判死**'))
                elif vis == 'private':
                    cats['suspect-dead'].append(
                        (n, 'private 定义 + 零入边 + 全仓无引用 ⇒ 疑似死链，'
                            '**仍须读代码**：宏注册 / 生成代码 / 外部 trait 接线都可能看不见'))
                elif vis == 'public':
                    cats['unwired-public'].append(
                        (n, 'pub 定义但仓内零调用 ⇒ 公共 API 面，可能是仓外消费者；'
                            '**不可据此判死**'))
                else:
                    cats['no-callsite'].append(
                        (n, '全仓无调用点，但可见性判不出 ⇒ 待人工'))
    for k in ('textual-prod', 'test-only', 'registered', 'unwired-public',
              'suspect-dead', 'no-callsite', 'static-like', 'natural-leaf'):
        print(f"# {k}\t{len(cats[k])}")
    print(f"# TOTAL_zero_inbound\t{len(nodes)}")
    for k in ('registered', 'suspect-dead', 'unwired-public', 'no-callsite',
              'textual-prod', 'f3-fn-as-value', 'test-only'):
        for n, why in cats[k]:
            print(f"{k}\t{n}\t{why}")
    print(f"[callgraph] 报告器非门。**没有任何桶可以直接删**："
          f"registered {len(cats['registered'])} = 按值接线（活的），"
          f"suspect-dead {len(cats['suspect-dead'])} = 疑似但未证，"
          f"unwired-public {len(cats['unwired-public'])} = 公共面。"
          f"逐条读代码证实/证伪后才可动手（R-SCAN-1）。", file=sys.stderr)
    return 0


def main(argv):
    if '--db' not in argv:
        print(__doc__)
        return 2
    args = type('A', (), {})()
    args.db = argv[argv.index('--db') + 1]
    args.depth = int(argv[argv.index('--depth') + 1]) if '--depth' in argv else 0
    args.files = '--files' in argv
    args.stale = '--stale' in argv
    args.scope = argv[argv.index('--crate') + 1] if '--crate' in argv else None
    g = load_graph(args.db)
    if '--stats' in argv:
        return cmd_stats(g, args)
    if '--unreachable' in argv:
        return cmd_unreachable(g, args)
    if '--impact' in argv:
        args.target = argv[argv.index('--impact') + 1]
        return cmd_impact(g, args)
    if '--deps' in argv:
        args.target = argv[argv.index('--deps') + 1]
        return cmd_deps(g, args)
    print(__doc__)
    return 2


if __name__ == '__main__':
    sys.exit(main(sys.argv))
