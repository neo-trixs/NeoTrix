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


# --- 间接重入 (跨函数) 扫描 -------------------------------------------------
# 2026-09-27: 原有扫描只看**单函数体内**的二次 lock(), 因此漏掉
# 「A 持锁 -> A 调 B -> B 再锁同一把」这一形态。实测漏掉了两处真实死锁:
#   l6_meta/runtime_monitor.rs monitor() 持 metrics 守卫后调
#     check_thresholds(), 后者内部 `self.metrics.lock()` -> 永久自死锁
#   (另一处同族见 handoff §1 #5 rise_reflector: 守卫活到函数尾)
# 这是调用图性质, 词法扫描看不到。此处用两趟近似:
#   1) 收集每个方法体内 lock 了哪些锁表达式
#   2) 若某方法在持有 self.X 守卫期间调用 self.method(), 而 method 体内
#      也 lock 了 self.X, 则报可疑
CALL_RE = re.compile(r"self\.(\w+)\s*\(")
SELF_LOCK_RE = re.compile(r"self\.(\w+)\.lock\s*\(")


def _method_bodies(text: str) -> dict[str, set[str]]:
    """方法名 -> 该方法体内 lock 过的 self.<field> 集合。"""
    bodies: dict[str, list[str]] = {}
    cur: str | None = None
    for line in text.splitlines():
        m = FUNC_RE.match(line)
        if m:
            cur = None
            mm = re.search(r"fn\s+(\w+)", line)
            if mm:
                cur = mm.group(1)
                bodies.setdefault(cur, [])
            continue
        if cur is not None:
            bodies[cur].append(line.split("//")[0])
    return {k: {g for ln in v for g in SELF_LOCK_RE.findall(ln)} for k, v in bodies.items()}


def _field_of(expr: str | None) -> str | None:
    """从 let 绑定的右侧推出被锁字段名。

    LET_LOCK_RE 已把 `.lock()` 吃掉, 所以 group(3) 形如 `self.process`;
    但若表达式里还含 `.lock()`(嵌套调用形态)则先整体匹配。
    """
    if not expr:
        return None
    m = SELF_LOCK_RE.search(expr)
    if m:
        return m.group(1)
    m = re.search(r"self\.(\w+)\s*$", expr.strip())
    return m.group(1) if m else None


def audit_indirect(text: str, label: str) -> list[str]:
    """跨函数(间接)重入扫描。

    2026-09-27 新增。原 audit_text 只看**单函数体内**的二次 lock(), 因此漏掉
    「A 持锁 -> A 调 self.B() -> B 再锁同一把」这一形态; 实测漏掉了
    l6_meta/runtime_monitor.rs monitor() -> check_thresholds() 的真实死锁
    (以及 handoff §1 #2/#3/#5 同族)。这是调用图性质, 词法扫描看不到。

    两趟近似:
      1) 收集每个方法体内 lock 过的 self.<field> 集合
      2) 逐行跟踪当前函数持有的守卫, 遇 self.method() 调用且该 method
         也 lock 了同一 field 则报可疑
    守卫生命周期: 随所在 `{}` 关闭而失效; `drop(var)` 显式释放 (不论深度);
    条件上下文 (`if let Ok(g) = ..lock() {`) 的守卫属于内层块。
    """
    locks_by_method = _method_bodies(text)
    out: list[str] = []
    fname: str | None = None
    depth = 0
    held: dict[str, int] = {}       # field -> 获取时的块深度
    var2field: dict[str, str] = {}  # let 变量名 -> field

    def release(dead_depth: int) -> None:
        for f in [f for f, d in held.items() if d >= dead_depth]:
            held.pop(f)

    for i, raw in enumerate(text.splitlines()):
        if FUNC_RE.match(raw):
            fname = raw.strip()[:70]
            depth, held, var2field = 0, {}, {}
            continue
        if fname is None:
            continue
        code = raw.split("//")[0]
        items: list[tuple[int, str, re.Match]] = []
        for m in LET_LOCK_RE.finditer(code):
            if _field_of(m.group(3)):
                items.append((m.start(), "letlock", m))
        let_end = {m.end() for _p, k, m in items if k == "letlock"}
        # 只保留条件上下文的裸 lock (if let/while let/match ... = self.x.lock())
        items += [(m.start(), "lock", m) for m in SELF_LOCK_RE.finditer(code)
                  if m.end() not in let_end
                  and re.search(r"\b(if|while|match)\b[^;]*$", code[: m.start()])]
        items += [(m.start(), "call", m) for m in CALL_RE.finditer(code)]
        items += [(m.start(), "drop", m) for m in DROP_RE.finditer(code)]
        items += [(m.start(), "brace", m) for m in re.finditer(r"\{|\}", code)]
        items.sort(key=lambda t: t[0])

        for _pos, kind, m in items:
            if kind == "brace":
                if m.group() == "{":
                    depth += 1
                else:
                    release(depth)
                    depth = max(0, depth - 1)
            elif kind == "drop":
                held.pop(var2field.get(m.group(1), m.group(1)), None)
            elif kind == "letlock":
                field = _field_of(m.group(3))
                if not field:
                    continue
                pre = code[: m.start()]
                in_cond = bool(re.search(r"\b(if|while|match)\b[^;]*$", pre))
                held.setdefault(field, depth + 1 if in_cond else depth)
                var2field[m.group(2)] = field
            elif kind == "lock":
                # 语句级临时守卫 (`*self.x.lock() = v;` / `self.x.lock();`)
                # 在分号即失效, **不得**记为持有, 否则满屏假阳性。
                # 只有条件上下文 (`if let Ok(g) = self.x.lock() {`) 才算绑定。
                pre = code[: m.start()]
                if re.search(r"\b(if|while|match)\b[^;]*$", pre):
                    held.setdefault(m.group(1), depth + 1)
            else:
                callee = m.group(1)
                for field in sorted(set(held) & locks_by_method.get(callee, set())):
                    out.append(
                        f"{label}:{i + 1}: {fname} — 持有 self.{field} 守卫期间调用 "
                        f"self.{callee}(), 后者内部再 lock self.{field} → 间接自死锁"
                    )
    return out


def audit_file(path: Path) -> list[str]:
    text = path.read_text(errors="ignore")
    return audit_text(text, str(path)) + audit_indirect(text, str(path))


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
fn bad_indirect(&self) {
    let g = self.metrics.lock().unwrap();
    *g = 1;
    self.check_thresholds();
}
fn check_thresholds(&self) {
    let m = self.metrics.lock().unwrap();
    let _ = m;
}
fn good_indirect_scoped(&self) {
    { let g = self.metrics.lock().unwrap(); *g = 1; }
    self.check_thresholds();
}
'''


def main() -> int:
    args = sys.argv[1:]
    if args and args[0] == "--selftest":
        found = audit_text(SELFTEST, "selftest") + audit_indirect(SELFTEST, "selftest")
        ok = len(found) == 2 and any("bad_indirect" in f for f in found) \
            and any("bad(" in f or "bad " in f or f.endswith("bad") for f in found) \
            and not any("good_" in f for f in found)
        for f in found:
            print(f)
        print(f"[lock-audit selftest] {'PASS' if ok else 'FAIL'} — 期望恰好 2 条 (bad + bad_indirect)，实得 {len(found)}")
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
