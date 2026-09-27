#!/usr/bin/env python3
"""nt_lock_audit.py — 非重入 Mutex 自死锁静态扫描 (2026-09-27 六连死锁后立项)

病根家族: 同一函数内先 `let g = self.x.lock()` 持守卫, 后续又 `self.x.lock()`
二次加锁 → std::sync::Mutex 非重入 → 同线程永久阻塞 (线程挂死 + 级联阻塞 +
线程堆积)。已实证 6 例: deferred_loader / nt_action_facade / rise_reflector /
scan_build_status(嵌套 cargo) 等。

启发式: 按函数切块 + **块作用域感知** (记录持锁绑定所在 `{}` 深度), 收集形如
  let [mut] NAME = EXPR.lock()          → 持锁绑定
  drop(NAME)                            → 显式释放
  EXPR.lock()                           → 裸加锁
仅当 EXPR 的裸加锁**仍在同一块作用域内**且中途无 drop(NAME) 时才报告 —— 这样
才能区分真自死锁与"守卫已随块结束释放"的常见误报。

自检: `python3 scripts/ops/nt_lock_audit.py --selftest` 用内置正/负样本验证规则。

用法: python3 scripts/ops/nt_lock_audit.py [根目录默认 neotrix-core/src]
退出码: 0 = 无可疑; 1 = 有可疑 (供 CI/门禁)。
"""
import re
import sys
from pathlib import Path

FUNC_RE = re.compile(r"^(\s*)(pub(\(\w+\))?\s+)?(async\s+)?fn\s+\w+")
LET_LOCK_RE = re.compile(r"let\s+(mut\s+)?(\w+)\s*=\s*([A-Za-z0-9_\.\(\)\[\]\s]*?)\.lock\(\)")
BARE_LOCK_RE = re.compile(r"([A-Za-z0-9_\.]+)\.lock\(\)")
DROP_RE = re.compile(r"drop\(\s*(\w+)\s*\)")


def depth_delta(code: str) -> int:
    """行内花括号净深度变化 (粗略剔除字符串/字符字面量与行注释)。"""
    stripped = re.sub(r'"(\\.|[^"\\])*"', '""', code)
    stripped = re.sub(r"'(\\.|[^'\\])'", "''", stripped)
    return stripped.count("{") - stripped.count("}")


def audit_text(text: str, label: str = "<text>") -> list[str]:
    """按块身份栈扫描: 守卫随其所在 `{}` 关闭而失效。"""
    out = []
    lines = text.splitlines()
    fname, buf = None, []

    def scan(entries):
        stack = [0]
        counter = 1
        held: dict[str, tuple[str, int, int]] = {}  # expr -> (name, line, block_id)
        for off, raw in entries:
            code = raw.split("//")[0]
            lets = list(LET_LOCK_RE.finditer(code))
            let_lock_pos = {m.end() for m in lets}
            items = [(m.start(), "let", m) for m in lets]
            items += [
                (m.start(), "bare", m)
                for m in BARE_LOCK_RE.finditer(code)
                if m.end() not in let_lock_pos
            ]
            items += [(m.start(), "brace", m) for m in re.finditer(r"\{|\}", code)]
            items += [(m.start(), "drop", m) for m in DROP_RE.finditer(code)]
            items.sort(key=lambda t: t[0])
            for _pos, kind, m in items:
                if kind == "drop":  # 显式释放守卫
                    for expr, (n, _ln, _b) in list(held.items()):
                        if n == m.group(1):
                            held.pop(expr)
                    continue
                if kind == "brace":
                    if m.group() == "{":
                        counter += 1
                        stack.append(counter)
                    elif len(stack) > 1:
                        dead = stack.pop()
                        for expr, (_n, _ln, b) in list(held.items()):
                            if b == dead:
                                held.pop(expr)
                    continue
                if kind == "let":
                    expr = m.group(3).strip()
                    held.setdefault(expr, (m.group(2), off, stack[-1]))
                    continue
                expr = m.group(1)
                if expr in held:
                    name, ln, blk = held.pop(expr)
                    if blk == stack[-1]:  # 守卫所在块仍开着 → 真自死锁
                        out.append(
                            f"{label}:{ln}: {fname} — 持锁 `{name}` (行{ln}) 未释放即对 "
                            f"`{expr}` 二次 lock() → 自死锁 (行{off})"
                        )

    for i, line in enumerate(lines):
        if FUNC_RE.match(line):
            scan(buf)
            fname, buf = line.strip()[:70], []
            continue
        if fname is not None:
            buf.append((i + 1, line))
    scan(buf)
    return out


def audit_file(path: Path) -> list[str]:
    return audit_text(path.read_text(errors="ignore"), str(path))


SELFTEST = r'''
fn bad(p: &S) {
    let count = p.n.lock().unwrap();
    if *count >= 9 { let _ = 1; }
    *p.n.lock().unwrap() += 1;
}
fn good_scoped(p: &S) {
    { let mut c = p.n.lock().unwrap(); *c += 1; }
    let v = *p.n.lock().unwrap();
    let _ = v;
}
fn good_drop(p: &S) {
    let c = p.n.lock().unwrap();
    drop(c);
    *p.n.lock().unwrap() += 1;
}
fn good_other(p: &S, q: &S) {
    let a = p.n.lock().unwrap();
    let b = q.m.lock().unwrap();
    let _ = (a, b);
}
'''


def main() -> int:
    args = sys.argv[1:]
    if args and args[0] == "--selftest":
        found = audit_text(SELFTEST, "selftest")
        ok = len(found) == 1 and "bad" in found[0]
        for f in found:
            print(f)
        print(f"[lock-audit selftest] {'PASS' if ok else 'FAIL'} — 期望恰好 1 条 (bad)，实得 {len(found)}")
        return 0 if ok else 1
    root = Path(args[0] if args else "neotrix-core/src")
    findings: list[str] = []
    for rs in root.rglob("*.rs"):
        try:
            findings.extend(audit_file(rs))
        except Exception as exc:  # noqa: BLE001
            print(f"[warn] {rs}: {exc}", file=sys.stderr)
    for f in findings:
        print(f)
    print(f"\n[lock-audit] 扫描 {root} — 可疑 {len(findings)} 处")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
