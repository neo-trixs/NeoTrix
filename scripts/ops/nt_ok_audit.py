#!/usr/bin/env python3
"""清点生产代码里「丢弃 Result」的 `.ok()` / `let _ = ..`。

为什么需要这个工具（2026-10-06 实测）：
  `check-silent-failure` 只对**设门**的那几类报错；`.ok()` 是**报告项**，
  只打印总数，不给位置。而人工 grep 分诊连错两次：
    · `checkpoint.rs` 5 处、`edit_history.rs` 7 处初判为缺陷
      ⇒ 读代码发现**全在 `#[cfg(test)]` 里**（roundtrip 测试 / 临时文件清理）
    · 根因：grep 按**行内容**过滤（`grep -v test`），**不看模块归属**

⇒ 故本工具按**行区间**判定：先定位 `#[cfg(test)]` 起止，
   再用括号配平求出每个 item 的真实结束行，**逐行区间**排除。

用法:
    python3 scripts/ops/nt_ok_audit.py                 # 汇总 + 按文件分组
    python3 scripts/ops/nt_ok_audit.py --file <path>   # 单文件逐条
    python3 scripts/ops/nt_ok_audit.py --kind fail    # 只看 fs/写类（高风险）
"""
import os
import re
import sys
from collections import defaultdict

ROOTS = ["neotrix-core/src", "crates"]
OK_PAT = re.compile(r"\.ok\(\)\s*;?\s*$")
LET_PAT = re.compile(r"^\s*let\s+_[A-Za-z0-9_]*\s*(?::[^=]+)?=")
# 高风险：丢弃的是 IO / 持久化 / 进程控制类
RISKY = re.compile(
    r"\b(fs::|std::fs::|File::|OpenOptions|writeln?!\s*\(\s*(?!std))"
    r"|\.kill\(\)|\.wait\(\)|\.join\(\)|rename_conversation|persist|write"
)


def test_ranges(lines):
    """返回生产代码的行号集合（0-based），排除所有 #[cfg(test)] 块。"""
    prod = set(range(len(lines)))
    for i, line in enumerate(lines):
        if re.match(r"\s*#\[cfg\(test\)\]", line):
            # 从该行起做括号配平，直到 depth 归零
            depth = 0
            started = False
            j = i
            while j < len(lines):
                depth += lines[j].count("{") - lines[j].count("}")
                if "{" in lines[j]:
                    started = True
                prod.discard(j)
                if started and depth <= 0:
                    break
                j += 1
    return prod


# 整文件即测试模块（本仓存在这种形态，AGENTS.md 明确记录过）
TEST_FILE = re.compile(r"(^|/)(tests?\.rs|test_[^/]*\.rs|[^/]*_tests?\.rs)$")


def audit_file(path):
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        return []
    if TEST_FILE.search(path.replace(os.sep, "/")):
        return []
    prod = test_ranges(lines)
    hits = []
    for i in sorted(prod):
        s = lines[i]
        if not OK_PAT.search(s):
            continue
        stmt = s.strip()
        if not LET_PAT.match(s) and "(" not in stmt:
            continue
        kind = "risky" if RISKY.search(stmt) else "plain"
        hits.append((i + 1, kind, stmt[:88]))
    return hits


def main():
    args = sys.argv[1:]
    only_file = None
    kind = None
    if "--file" in args:
        only_file = args[args.index("--file") + 1]
    if "--kind" in args:
        kind = args[args.index("--kind") + 1]

    files = [only_file] if only_file else []
    if not files:
        for root in ROOTS:
            for dp, _, fns in os.walk(root):
                files.extend(
                    os.path.join(dp, f) for f in fns if f.endswith(".rs")
                )

    total = risky = 0
    by_file = defaultdict(list)
    for p in sorted(files):
        for line_no, k, stmt in audit_file(p):
            if kind and k != kind:
                continue
            total += 1
            risky += k == "risky"
            by_file[p].append((line_no, k, stmt))

    print("[ok-audit] 生产代码中丢弃 Result 的 .ok(): %d 处（其中 IO/写类 %d）"
          % (total, risky))
    for p in sorted(by_file, key=lambda x: -len(by_file[x])):
        rows = by_file[p]
        print("\n  %s（%d 处）" % (p, len(rows)))
        for line_no, k, stmt in rows:
            mark = "!" if k == "risky" else " "
            print("   %s %5d| %s" % (mark, line_no, stmt))


if __name__ == "__main__":
    main()
