#!/usr/bin/env python3
"""nt_map_reconcile.py — 让全域 map 的每条「现状」断言**可执行**，从而不会腐烂。

为什么存在（本仓反复吃过的一类药）：
  地图/台账写着「现状【实测】」，几轮之后就全成陈旧值。本仓已实测的同类事故：
    · 3 处文档数字已错（frontmatter 32/59 非 41/58；SKILL.md 68 非 67；行号漂移）
    · `ABSORPTION-AGENT-ARCH-2026-09-30` 全篇按「无调用图」写，而调用图已落地
    · 单个文件 23 处 `/// Note: Real implementation needs` 而函数体完整
  ⇒ **陈旧记录比没有记录更危险**：下一个人会照着错的做。
  本工具给出机制而不是纪律：文档里的断言带一个**可执行谓词**，
  跑一次就知道它还成不成立。

判据纪律（与本仓既有裁决一致）：
  - 只支持**廉价、确定**的谓词：文件在/不在、字面量在/不在、测试名在不在、
    某命令是否 rc=0。**不支持**语义判断、相似度、"看起来对"。
  - 判不出来就写 UNKNOWN，**不猜**（宁缺勿错）。
  - 默认**只报告不失败**（`--strict` 才非零）：谓词本身可能写错，
    门建在错谓词上比没门更危险（同 G7 裁决）。
  - 0 断言 ⇒ 报错退出 2：一个「零断言却 PASS」的报告本身就是 vacuous green。

**谓词必须写在 ```assert 围栏块里**（不是行内代码）。
这不是洁癖，是实测教训：v1 用行内 `test:headed` 之类匹配，全仓扫出 **18 条
假阳性** —— 那是 `package.json` 的脚本名，不是断言。⇒ 谓词语法必须与散文
**结构上不可能混淆**。围栏块同时给文档一个明确的位置来声明
「这些是我的可核对声明」。

围栏块内每行：`谓词 [# 上下文说明]`（说明可省）。

谓词语法：
  file:<path>          路径存在且被 git 跟踪
  nfile:<path>         路径不存在
  lit:<文本>@<path>    字面量出现在该文件
  nlit:<文本>@<path>   字面量**不**出现在该文件
  test:<名字>@<path>   该测试名出现在该文件（防「测试被删但文档说通过」）
  cmd:<命令>           命令 rc=0（仅限亚秒级只读命令）

用法：
  python3 scripts/ops/nt_map_reconcile.py                    # 全仓扫描所有带断言的文档
  python3 scripts/ops/nt_map_reconcile.py --doc <path>       # 只查一份
  python3 scripts/ops/nt_map_reconcile.py --strict           # 有 VIOLATED 则 rc=1
  python3 scripts/ops/nt_map_reconcile.py --list-predicates  # 打印谓词语法
"""
import argparse
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# 只认 ```assert 围栏块 —— 见 docstring：行内匹配会造成假阳性（实测 18 条）。
FENCE_OPEN = '```assert'
FENCE_CLOSE = '```'

PRED_RE = re.compile(r'^(file|nfile|lit|nlit|test|cmd):(\S.*)$')


def git_tracked(path):
    try:
        p = subprocess.run(['git', 'ls-files', '--error-unmatch', '--', path],
                           cwd=REPO, capture_output=True, text=True,
                           timeout=60)
    except (subprocess.SubprocessError, FileNotFoundError):
        return None
    if p.returncode == 0:
        return True
    if p.returncode == 128 or 'not exist' in (p.stderr or ''):
        # git 对「不在索引里」返回 128；与「真不存在」不可区分 ⇒ 交给 file: 的
        # os.path.exists 二次判定，这里保守返回 False 但记为不确定由上层处理
        return False
    return None


def read_text(path):
    full = os.path.join(REPO, path)
    try:
        with open(full, encoding='utf-8', errors='replace') as fh:
            return fh.read()
    except OSError:
        return None


def check_one(pred, doc_path):
    """返回 (status, evidence)。status ∈ HOLDS / VIOLATED / UNKNOWN。"""
    kind, arg = pred
    if kind == 'file':
        target = arg.strip()
        full = os.path.join(REPO, target)
        exists = os.path.exists(full)
        tracked = git_tracked(target)
        if exists and tracked:
            return 'HOLDS', 'exists + tracked'
        if exists and tracked is False:
            return 'HOLDS', 'exists (untracked — 若是生成物请改用 nfile 语义）'
        if not exists:
            return 'VIOLATED', 'path does not exist'
        return 'VIOLATED', 'exists but not in git index'
    if kind == 'nfile':
        target = arg.strip()
        if os.path.exists(os.path.join(REPO, target)):
            return 'VIOLATED', 'path still exists'
        return 'HOLDS', 'absent'
    if kind in ('lit', 'nlit', 'test'):
        body, _, target = arg.rpartition('@')
        needle = body.strip()
        target = target.strip()
        if not needle or not target:
            return 'UNKNOWN', 'malformed: need <text>@<path>'
        text = read_text(target)
        if text is None:
            return 'UNKNOWN', 'cannot read %s' % target
        present = needle in text
        if kind == 'lit':
            return ('HOLDS', 'found') if present else ('VIOLATED', 'literal absent')
        if kind == 'nlit':
            return ('VIOLATED', 'literal present') if present else ('HOLDS', 'absent')
        # test: 名字存在（顺带确认是 #[test] 上下文，避免只出现在注释里）
        if not present:
            return 'VIOLATED', 'test name not found'
        for m in re.finditer(re.escape(needle), text):
            start = max(0, m.start() - 200)
            window = text[start:m.start()]
            if '#[test]' in window or '#[tokio::test]' in window:
                return 'HOLDS', 'found in a #[test] block'
        return 'HOLDS', 'found (not in a #[test] block — 弱证据)'
    if kind == 'cmd':
        try:
            p = subprocess.run(arg, shell=True, cwd=REPO, capture_output=True,
                               text=True, timeout=120)
        except subprocess.SubprocessError as exc:
            return 'UNKNOWN', 'cmd failed to run: %s' % exc
        if p.returncode == 0:
            return 'HOLDS', 'rc=0'
        return 'VIOLATED', 'rc=%d' % p.returncode
    return 'UNKNOWN', 'unhandled predicate kind %r' % kind


def scan_doc(doc_path):
    """只解析 ```assert 围栏块。行内代码一律不看（见 docstring 的假阳性教训）。"""
    text = read_text(doc_path)
    if text is None:
        return None, []
    rows = []
    in_fence = False
    for lineno, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not in_fence:
            if stripped.startswith(FENCE_OPEN):
                in_fence = True
            continue
        if stripped.startswith(FENCE_CLOSE):
            in_fence = False
            continue
        if not stripped or stripped.startswith('#'):
            continue
        body, _, note = stripped.partition('#')
        pred = body.strip()
        m = PRED_RE.match(pred)
        if not m:
            rows.append({
                'doc': doc_path, 'line': lineno, 'pred': pred,
                'kind': '?unparsed', 'arg': pred,
                'context': note.strip() or '(unparsable predicate)',
            })
            continue
        rows.append({
            'doc': doc_path, 'line': lineno,
            'pred': '%s:%s' % (m.group(1), m.group(2).strip()),
            'kind': m.group(1), 'arg': m.group(2).strip(),
            'context': note.strip(),
        })
    return text, rows


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[1])
    ap.add_argument('--doc', action='append', default=None)
    ap.add_argument('--strict', action='store_true',
                    help='exit 1 if any assertion is VIOLATED')
    ap.add_argument('--list-predicates', action='store_true')
    ap.add_argument('--quiet-holds', action='store_true',
                    help='only print VIOLATED/UNKNOWN')
    args = ap.parse_args(argv[1:])

    if args.list_predicates:
        # 从 docstring 里取以两个空格 + 已知前缀开头的行（谓词表），避免
        # 靠字符串切分去猜段落边界（v1 就是切错了段落）。
        for line in __doc__.splitlines():
            if line.startswith('  ') and PRED_RE.match(line.strip()):
                print(line.rstrip())
        print('')
        print('形式：写在 ```assert 围栏块里，每行 `谓词 [# 说明]`')
        return 0

    docs = args.doc
    if not docs:
        r = subprocess.run(['git', 'ls-files', '*.md'], cwd=REPO,
                           capture_output=True, text=True, timeout=120)
        docs = [d for d in r.stdout.splitlines() if d.strip()]

    total = holds = violated = unknown = 0
    bad = []
    for doc in docs:
        text, rows = scan_doc(doc)
        if text is None:
            sys.stderr.write('cannot read %s\n' % doc)
            continue
        for row in rows:
            if row['kind'] == '?unparsed':
                status, evidence = 'UNKNOWN', 'unparsable predicate'
            else:
                status, evidence = check_one((row['kind'], row['arg']), doc)
            total += 1
            if status == 'HOLDS':
                holds += 1
                if not args.quiet_holds:
                    print('  ok   %s:%d  %s  (%s)'
                          % (doc, row['line'], row['pred'], evidence))
            elif status == 'VIOLATED':
                violated += 1
                bad.append((doc, row['line'], row['pred'], evidence, row['context']))
                print('  FAIL %s:%d  %s  (%s)'
                      % (doc, row['line'], row['pred'], evidence))
                print('       ↳ %s' % row['context'])
            else:
                unknown += 1
                print('  ??   %s:%d  %s  (%s)'
                      % (doc, row['line'], row['pred'], evidence))
                print('       ↳ %s' % row['context'])

    print()
    print('[map-reconcile] assertions=%d holds=%d violated=%d unknown=%d'
          % (total, holds, violated, unknown))
    if total == 0:
        sys.stderr.write(
            'no machine-checkable assertions found — a report with 0 '
            'assertions is a vacuous green.\n'
            'Add predicates (see --list-predicates) to the map\'s 现状 rows.\n')
        return 2
    if violated:
        print('[map-reconcile] %d claim(s) in the map are now FALSE — the map '
              'is stale, which is worse than absent.' % violated)
    if args.strict and violated:
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
