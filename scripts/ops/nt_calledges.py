#!/usr/bin/env python3
"""nt_calledges.py — 编译器级调用边抽取（THIR 委托，不猜）.

原理（2026-09-30 实测确立）：
  RUSTC_BOOTSTRAP=1 cargo rustc --lib -- -Zunpretty=thir-tree
  输出的每个调用点自带编译器已解析的目标：
      Call { ty: FnDef(DefId(2:12569 ~ core[edd1]::fmt::{impl#12}::write_str), …)
      fn_span: crates/.../epistemic.rs:23:10: 23:15 (#28)
  包围的 Scope 带 HirId(DefId(0:433 ~ <caller owner>).N) —— 调用方直接可得。
  同名诱饵在结构上不可能混淆（DefId 带 crate 哈希）。

判据（可证伪）：抽出边数 / Call 总数。
  nt-core-capability-tree 实测 5892/6241 = 94%。漏掉的是闭包/泛型边界，
  按「宁缺勿错」接受（漏边无害，错边致命）。

用法：
  python3 scripts/ops/nt_calledges.py --crate <dir> [--out edges.jsonl]
      流式抽取单个 crate（不落中间 250MB 文本，直接管道进解析器）
  python3 scripts/ops/nt_calledges.py --callers <DefId-path-substr> --db edges.jsonl
  python3 scripts/ops/nt_calledges.py --callees <DefId-path-substr> --db edges.jsonl

输出 JSONL，每行：
  {"caller": "nt_core_capability_tree[0ae4]::registry::legacy_domain_of",
   "callee": "core[edd1]::fmt::{impl#12}::write_str",
   "span": "crates/nt-core-capability-tree/src/registry.rs:940:13: 940:16",
   "kind": "call|method"}

约束：
  - 只要 stable rustc + RUSTC_BOOTSTRAP=1（本仓 Homebrew 1.94 实测通过），
    不要 nightly / rustup / rust-analyzer / scip。
  - 全量 867K 行约 45GB 流式文本、~30min —— 定位是**低频生成物**
    （与 .project-map 同级，gitignored），不是 CI 门。
"""
import json
import re
import subprocess
import sys

CALL = re.compile(r'(?:^|[^A-Za-z_])(?:Method)?Call \{')
TY = re.compile(r'ty: FnDef\(DefId\(\d+:\d+ ~ ([^)]+)\)')
TY_METHOD = re.compile(r'MethodCall')
OWN = re.compile(r'HirId\(DefId\(\d+:\d+ ~ ([^)]+)\)')
SPAN = re.compile(r'fn_span: (\S+)')


def extract(stream):
    """流式解析 THIR 文本。返回 (edges, calls)；edges 为 (caller, callee, span, kind)。"""
    buf = []
    owner = None
    calls = 0
    edges = []
    pending = None  # (caller, kind, lines_seen)
    for line in stream:
        m = OWN.search(line)
        if m:
            owner = m.group(1)
        if CALL.search(line):
            calls += 1
            kind = 'method' if 'MethodCall' in line else 'call'
            pending = [owner, kind, 0, None]
            continue
        if pending is not None:
            pending[2] += 1
            t = TY.search(line)
            if t and pending[3] is None:
                pending[3] = t.group(1)
            s = SPAN.search(line)
            if s and pending[3] is not None:
                edges.append((pending[0], pending[3], s.group(1), pending[1]))
                pending = None
            elif pending[2] > 40:
                pending = None  # 40 行内无解析目标：按宁缺勿错丢弃
    return edges, calls


def run_crate(crate_dir, out_path):
    cmd = (['env', 'RUSTC_BOOTSTRAP=1', 'cargo', 'rustc', '--lib',
            '--', '-Zunpretty=thir-tree'])
    proc = subprocess.Popen(cmd, cwd=crate_dir, stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL, text=True,
                            errors='ignore', bufsize=1 << 20)
    edges, calls = extract(proc.stdout)
    proc.wait()
    if proc.returncode != 0:
        print(f"[calledges] rustc failed rc={proc.returncode} for {crate_dir}",
              file=sys.stderr)
        return 0, 0
    n = 0
    fh = open(out_path, 'w', encoding='utf-8') if out_path else sys.stdout
    for caller, callee, span, kind in edges:
        fh.write(json.dumps({'caller': caller, 'callee': callee,
                             'span': span, 'kind': kind},
                            ensure_ascii=False) + '\n')
        n += 1
    if out_path:
        fh.close()
    print(f"[calledges] {crate_dir}: {n}/{calls} edges "
          f"({100*n//max(1,calls)}%)", file=sys.stderr)
    return n, calls


def query(db_path, direction, substr):
    hits = []
    with open(db_path, encoding='utf-8') as fh:
        for line in fh:
            try:
                e = json.loads(line)
            except ValueError:
                continue
            key = e['caller'] if direction == 'callees' else e['callee']
            if substr in key:
                other = e['callee'] if direction == 'callees' else e['caller']
                hits.append((other, e['span'], e['kind']))
    seen = set()
    for other, span, kind in sorted(hits):
        if (other, span) not in seen:
            seen.add((other, span))
            print(f"{other}  @ {span}  [{kind}]")
    print(f"[calledges] {len(seen)} unique {direction} for '{substr}'",
          file=sys.stderr)


def main(argv):
    if '--crate' in argv:
        d = argv[argv.index('--crate') + 1]
        out = argv[argv.index('--out') + 1] if '--out' in argv else None
        run_crate(d, out)
    elif '--callers' in argv or '--callees' in argv:
        direction = 'callers' if '--callers' in argv else 'callees'
        flag = '--callers' if direction == 'callers' else '--callees'
        substr = argv[argv.index(flag) + 1]
        db = argv[argv.index('--db') + 1]
        query(db, direction, substr)
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
