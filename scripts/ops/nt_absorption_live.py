#!/usr/bin/env python3
"""nt_absorption_live.py — 已吸收技术存活率检查（G5 的直接解法）.

背景：38+ 源、ROUND2–22、MAP-TASKS §1.1「已落地 35 项」——但没有任何机制回答
「吸收来的技术后来活下来了吗」。本工具把**文档本身当 baseline**（单一事实源，
不另建基线文件，避免基线漂移），逐条核对「已落地」断言今天是否成立。

判据（可证伪）：跑一次应当复现出已知量级 —— 若报「全部存活」，说明判据无效
而不是仓库健康（对照：今天已知 212 死文件 / 50,454 行）。

用法：
  python3 scripts/ops/nt_absorption_live.py [--strict] [--doc PATH]
      默认只读模式：报告每条存活/死亡/搬迁/不可判。
  python3 scripts/ops/nt_absorption_live.py --graph
      叠加**第二判据**（编译期调用图可达性）。这是本工具的 G5 修复：
      纯文本判据把「字符串出现过」当存活，而注释/文档/重导出/同名不同类型
      都会让字符串出现却生产不可达 —— 即 R-P79 的「导出 ≠ 接入」。
      两判据**不互相替代**：图对 shell 脚本与类型名静默（它们不是调用目标），
      故文本判据仍为覆盖面判据，图判据为**生产可达**判据。输出标注每条
      由哪个判据裁决。
  --strict：有死亡项则 exit 1（供 CI）。

实现纪律（本仓教训）：
  - 只用 `rg` **不带 `-E`**（本机 `rg -E` 任何模式静默返回 0，
    CAPABILITY-GAP 已记录）。用 `rg -F` 或纯字面模式。
  - 抽不出机器可判目标的行 → 记「不可判」，不猜（宁缺勿错）。
  - 测试数断言（"91 测试"）用 `#[test]` 计数近似，并在输出中标明是近似值。
  - 文本判据**不得**用于可达性判定：2026-09-30 实测 200 样本误报率 100%
    （注释/同名词/散文/prose 命中）。可达性只认编译期边，见 nt_callgraph.py
    的 F1–F6 误报源清单。
"""
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__ + '/..')))
REPO = '/Users/neo/Downloads/neotrix'
DOC_DEFAULT = os.path.join(
    REPO, 'docs/architecture/ABSORPTION-MAP-TASKS-2026-09-29.md')

# backtick token；测试数 "91 测试" / "6 单测"
RE_TOKEN = re.compile(r'`([^`]+)`')
RE_TESTS = re.compile(r'(\d+)\s*(单测|测试)')


def rg_count(pattern):
    """纯字面 rg（不用 -E）。返回命中文件数。"""
    try:
        out = subprocess.run(
            ['rg', '-l', '-F', '--', pattern, 'neotrix-core/src', 'crates'],
            capture_output=True, text=True, cwd=REPO, timeout=120)
    except (subprocess.SubprocessError, FileNotFoundError):
        return -1
    if out.returncode not in (0, 1):
        return -1
    return len([l for l in out.stdout.splitlines() if l.strip()])


def find_file(token):
    """路径 token 解析：原样 → neotrix-core/src/ 前缀 → basename 全仓搜。
    返回 (path|None, ambiguous)。多命中 = 文件存在但位置不明（alive + 注记），
    绝不因多命中判死（2026-09-30 实测：check-ci-refs.sh 有 scripts/ 与
    scripts/probes/ 两份，判死即误报）。"""
    cands = [token, os.path.join('neotrix-core/src', token)]
    # 连字符裸名（doc 常省略扩展名）：补常见扩展名再试
    if '/' not in token and '.' not in token:
        for ext in ('.sh', '.py', '.rs'):
            cands.append(token + ext)
    for c in cands:
        if os.path.isfile(os.path.join(REPO, c)):
            return c, False
    base = os.path.basename(token)
    try:
        out = subprocess.run(
            ['git', 'ls-files', f'**/{base}'],
            capture_output=True, text=True, cwd=REPO, timeout=60)
    except subprocess.SubprocessError:
        return None, False
    hits = [l for l in out.stdout.splitlines()
            if l.strip().endswith(('.rs', '.sh', '.py', '.md'))]
    if len(hits) == 1:
        return hits[0], False
    if len(hits) > 1:
        return hits[0], True
    return None, False


def count_tests(path):
    """文件内 #[test] + #[tokio::test] 计数（近似值，输出中标明）。"""
    try:
        with open(os.path.join(REPO, path), encoding='utf-8',
                  errors='ignore') as fh:
            text = fh.read()
    except OSError:
        return -1
    return len(re.findall(r'#\[(?:tokio::)?test\]', text))


def check_token(token):
    """返回 (status, detail)。
    status ∈ alive/dead-or-moved/moved/unparseable/external。
    注意：调用方按 section 解读（gone-section 的 absent = 与拒绝结论一致）。"""
    t = token.strip()
    # 外部仓库名（org/repo，无扩展名）：不是本地路径，不可判
    # （2026-09-30 实测：NVlabs/kda、qybaihe/mu 被误判为 dead 路径）
    if re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', t) \
            and not t.endswith(('.rs', '.sh', '.py', '.md', '.toml', '.json')):
        return ('external', f'外部仓库名 {t}，非本地路径，不可判')
    if '/' in t or t.endswith(('.rs', '.sh', '.py', '.md', '.toml', '.json')):
        hit, ambig = find_file(t)
        if hit is None:
            return ('dead-or-moved',
                    f'路径 {t} 在当前树中找不到（可能被删或改名）')
        if ambig:
            return ('alive',
                    f'文件存在但多处同名（取首个 {hit}），位置不明')
        if hit != t and not t.startswith(('neotrix-core/', 'crates/',
                                          'scripts/', 'docs/', 'apps/')):
            return ('moved', f'原写 {t}，现位于 {hit}')
        return ('alive', f'文件存在：{hit}')
    # 纯标识符：查代码出现
    if re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', t):
        n = rg_count(t)
        if n < 0:
            return ('unparseable', 'rg 执行失败')
        if n == 0:
            return ('dead-or-moved', f'标识符 {t} 全仓代码 0 命中')
        return ('alive', f'标识符 {t} 在 {n} 个文件中出现')
    return ('unparseable', f'无法分类的 token：{t[:60]}')


def extract_section(text, start_marker, end_marker=None):
    i = text.find(start_marker)
    if i < 0:
        return ''
    j = text.find(end_marker, i) if end_marker else len(text)
    return text[i:j if j > 0 else len(text)]


def graph_verdicts(tokens, db):
    """第二判据：编译期调用图可达性。返回 {token: (verdict, detail)}。

    verdict ∈ 'reachable'（有传递调用者 ⇒ 生产可达）
            'no-callers'（是调用目标但零入边 ⇒ 生产不可达，R-P79 的「导出≠接入」）
            'not-call-target'（图里没有该节点 —— shell 脚本/类型名，不是调用目标）
    ⛔ 不判死：'no-callers' 只标「生产不可达」，因为 F6 盲区（fn-as-value）
    与 pub API 外部消费都可能让它仍然有用。裁决必须人工读代码（R-SCAN-1）。
    """
    if not os.path.isfile(db):
        return None
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import nt_callgraph as cg                          # noqa: E402 运行时导入
    g = cg.Graph().load(db)
    out = {}
    for tok in tokens:
        # 文档里常写成 `fn_name()`；调用目标判定只看标识符本体
        sym = tok[:-2] if tok.endswith('()') else tok
        # shell 脚本 / 类型名 / 文档名不是调用目标，直接判 not-call-target
        if (tok.endswith(('.sh', '.md', '.json', '.toml', '.yml', '.yaml'))
                or not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', sym)):
            out[tok] = ('not-call-target', '非 Rust 函数名，图判据不适用')
            continue
        seeds = g.seeds_matching(sym)
        if not seeds:
            out[tok] = ('not-call-target', '边表无此节点（时点快照或非调用目标）')
            continue
        reach = g.bfs(seeds, g.rev, 0)
        for s in seeds:
            reach.pop(s, None)
        if reach:
            out[tok] = ('reachable', f'传递调用者 {len(reach)} 个 ⇒ 生产可达')
        else:
            out[tok] = ('no-callers', f'{len(seeds)} 个节点零入边 ⇒ 生产不可达')
    return out


def main(argv):
    strict = '--strict' in argv
    use_graph = '--graph' in argv
    db = os.path.join(REPO, '.project-map/edges-all.jsonl')
    if '--db' in argv:
        db = argv[argv.index('--db') + 1]
    doc = DOC_DEFAULT
    if '--doc' in argv:
        doc = argv[argv.index('--doc') + 1]
    with open(doc, encoding='utf-8') as fh:
        text = fh.read()
    landed = extract_section(text, '### 1.1', '### 1.2')
    gone = extract_section(text, '### 1.3', '## 2.')
    rows = []
    for sec, expect in ((landed, 'alive'), (gone, 'gone')):
        for m in RE_TOKEN.finditer(sec):
            rows.append((m.group(1), expect))
    # 去重保序
    seen, uniq = set(), []
    for tok, exp in rows:
        if tok not in seen:
            seen.add(tok)
            uniq.append((tok, exp))
    gv = graph_verdicts([t for t, _ in uniq], db) if use_graph else None
    if use_graph:
        if gv is None:
            print(f"[absorb-live] 边表不存在，跳过图判据：{db}")
        else:
            print(f"[absorb-live] 图判据已启用（第二判据，不替代文本判据）")
    else:
        # T4：默认模式的判据只能证明「字符串出现过」。本文件 docstring 自述
        # 「若报全部存活说明判据无效」—— 而默认模式历史上正会报全部存活。
        # ⇒ 不加 --graph 时必须把这句话打在输出上，否则 PASS 会被当成健康证明。
        print("[absorb-live] ⚠ 未加 --graph：本判据**只能证明文本存在**，"
              "不能证明生产可达（注释/重导出/同名词都会让字符串出现却不可达）。")
        print("[absorb-live]   本工具 PASS **不是**健康证明；要判接线层请加 --graph。")
    print(f"[absorb-live] doc={os.path.basename(doc)} tokens={len(uniq)}")
    dead, alive, moved, unp, ext, review = [], [], [], [], [], []
    unwired = []
    for tok, expect in uniq:
        status, detail = check_token(tok)
        tag = f"{expect}/{status}"
        if gv and tok in gv:
            gv_status, gv_detail = gv[tok]
            tag += f" |graph:{gv_status}"
            detail += f" 〔图：{gv_detail}〕"
            if expect == 'alive' and status == 'alive' and gv_status == 'no-callers':
                unwired.append((tok, gv_detail))
        print(f"  [{tag}] {tok[:64]} — {detail[:80]}")
        if status == 'external':
            ext.append(tok)
        elif expect == 'gone':
            # 拒绝区语义反转：absent = 与拒绝结论一致（好事）；
            # present = 名字还在某处，需人工看是证伪记录还是复活。
            if status in ('dead-or-moved',):
                alive.append(tok)  # 计入一致项
            elif status in ('alive', 'moved'):
                review.append((tok, detail))
            else:
                unp.append(tok)
        elif status in ('dead-or-moved',):
            dead.append((tok, expect, detail))
        elif status == 'moved':
            moved.append((tok, detail))
        elif status == 'alive':
            alive.append(tok)
        else:
            unp.append(tok)
    print(f"[absorb-live] alive-or-consistent={len(alive)} moved={len(moved)} "
          f"dead={len(dead)} needs-review={len(review)} "
          f"external={len(ext)} unparseable={len(unp)}")
    if unwired:
        print(f"[absorb-live] ⚠ 文本存活但**生产不可达**（R-P79「导出≠接入」）："
              f"{len(unwired)} 条")
        for tok, d in unwired[:15]:
            print(f"    ⊘ {tok[:60]} — {d[:70]}")
    if review:
        print("[absorb-live] 需人工复核（拒绝区名字仍在代码中出现）：")
        for tok, d in review[:15]:
            print(f"    ? {tok[:60]} — {d[:70]}")
    if dead:
        print("[absorb-live] 死亡项（期望存活但找不到）：")
        for tok, exp, d in dead[:20]:
            print(f"    ⛔ [{exp}] {tok[:60]} — {d[:70]}")
    if strict and dead:
        print(f"[absorb-live] FAIL: {len(dead)} 项已落地断言死亡")
        return 1
    print("[absorb-live] PASS" if not dead else "[absorb-live] 报告模式：有死亡项（未加 --strict，不断言）")
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
