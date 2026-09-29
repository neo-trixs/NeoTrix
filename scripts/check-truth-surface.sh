#!/bin/bash
# Truth-surface gate — keeps "code that exists" from diverging from
# "code the compiler and test runner can actually see".
#
# Motivation (audit 2026-09-27): `cargo test` was fully green while
# 311 tests in 3 files under neotrix-core/src/l1_action/nt_act/nt_act_trade/tests/
# were never compiled, and 54 zero-byte .rs files were re-exported as
# if they were real public API. Green signals meant nothing because the
# offending code was invisible to the toolchain.
#
# Three classes of truth-drift, all structural (not style, so not clippy's job):
#   1) EMPTY     0-byte .rs file declared `pub mod` — compiles as an empty
#                namespace, so every downstream `use` succeeds and yields nothing.
#   2) UNDECLARED a sibling .rs in a tests/ dir with no `mod` declaration in
#                tests/mod.rs — written, reviewed, never executed.
#   3) TRACKED   build artifacts / runtime DB under version control.
#
# Baseline is a LIST (scripts/truth-surface-baseline.txt), not a count, so that
# deleting one offender and adding another cannot hide the new one. Ratchet down
# with --update-baseline once entries are genuinely resolved.
#
# Usage:
#   bash scripts/check-truth-surface.sh              # advisory, always exit 0
#   bash scripts/check-truth-surface.sh --strict     # exit 1 if any NEW offender
#   bash scripts/check-truth-surface.sh --update-baseline
#
# Note: bash-3.2-safe style (macOS system bash). `bash -n` before commit.

set -uo pipefail

BASELINE="scripts/truth-surface-baseline.txt"
STRICT=0
UPDATE=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --update-baseline) UPDATE=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

# Roots to scan. Excludes target/, .worktrees/ (stale full copies), thirdparty/.
#
# 2026-09-28 删两条死引用（`nt_scan_surface.py` 定位）：
#   - `apps`         → 已于本会话彻底移除（apps/neobot-desktop 归档后仅剩空壳，
#                      实测 0 跟踪文件、0 代码引用）
#   - `src-tauri/src`→ 桌面端随 5c02e738 归档（599 files），现由 crates/neotrix-neobot
# 两行此前靠下游的 `[ -d "$root" ] || continue` 静默兜住 ⇒ **不报错**，
# 但也**不告知少扫了哪两个面**。静默少扫与「扫过且干净」在报告上无法区分。
SCAN_ROOTS="neotrix-core/src crates"
TRACK_GLOBS='\.(rlib|so|dylib|a|o|wasm)$|^\.neotrix/.*\.(db|sqlite|sqlite3)$'

CUR=$(mktemp)
CUR_C=$(mktemp)
BASE_C=$(mktemp)
NEW=$(mktemp)
GONE=$(mktemp)
trap 'rm -f "$CUR" "$CUR_C" "$BASE_C" "$NEW" "$GONE"' EXIT

# ---------- 1) EMPTY: zero-byte .rs files ----------
for root in $SCAN_ROOTS; do
  [ -d "$root" ] || continue
  find "$root" -name '*.rs' -type f -size 0 2>/dev/null | sed 's|^\./||' | sort
done | sed 's/^/EMPTY /' >> "$CUR"

# ---------- 2) UNDECLARED: a sibling .rs with no `mod` in its own mod.rs ----------
#
# 2026-09-29 扩域（见 EVOLUTION-ROADMAP-CODE-NODES-2026-09-29.md N-2）。
# 原范围是 `-path '*/tests/*'`，即**只有字面叫 tests 的目录**。这正是本脚本动机注释
# (:4-9) 描述的那个病的复发点：那次修好了 nt_act_trade/tests/，但同类问题在普通模块
# 目录里从来没被覆盖过 —— 实测 **212 个 .rs 从不被任何 target 编译**，含：
#   - neotrix-core/src/l1_action/nt_act/tool_registry.rs                (770 行)
#   - neotrix-core/src/l6_meta/nt_meta/eval_engine/                   (641 行，全仓唯一
#     的 dataset / experiment / llm-judge 抽象)
#   - neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/context_budget.rs
#   - neotrix-core/src/l3_embodiment/nt_shield/defense/**              (21 个)
# 现在扩到**所有含 mod.rs 的目录**（实测 381 个）。
#
# ⛔ 判读纪律：「未被编译」≠「功能缺失」。多数是已归档的旧引擎
# (nt_consciousness_core/archive/、ring_defense/ 明显是)。
# 另有 `hybrid_retrieval/` 是**声明被注释掉**（nt_memory/mod.rs:54，注「内部编译错误
# 待修复」）—— 那不是"忘了声明"，是"声明了但编译不过"，处置方式不同。
# **先分类再处置**；盲删会打断活路径（DIR-REMEDY §2.5「导出 ≠ 调用」已错过 3 次）。
for mod_rs in $(find $SCAN_ROOTS -type f -name 'mod.rs' 2>/dev/null); do
  dir=$(dirname "$mod_rs")
  for sib in "$dir"/*.rs; do
    [ -f "$sib" ] || continue
    base=$(basename "$sib")
    [ "$base" = "mod.rs" ] && continue
    stem="${base%.rs}"
    if grep -qE "(^|[^A-Za-z0-9_])mod[[:space:]]+$stem[[:space:]]*[;{]" "$mod_rs" 2>/dev/null; then
      continue
    fi
    # #[path = "..."] attributed module pointing at this file
    if grep -q "#\[path" "$mod_rs" 2>/dev/null && \
       grep -q "\"$base\"" "$mod_rs" 2>/dev/null; then
      continue
    fi
    echo "UNDECLARED $sib"
  done
done | sort -u >> "$CUR"

# ---------- 2b) UNREACHABLE: 磁盘上有，但从任何 crate root 传递不可达 ----------
#
# 与 2 的区别：2 只看「同目录 mod.rs 有没有声明它」；本类做**从 crate root 出发的
# 传递可达性**，因此能穿透多层（a/mod.rs -> a/b/mod.rs -> b/c.rs），
# 并且能识别「crate root 自己就没人声明」的情况。
#
# 为什么不用 dep-info（target/debug/deps/*.d）当 oracle：**dep-info 只覆盖单个 target**。
# 实测 `target/debug/deps/neotrix.d` 列 2225 个 neotrix-core 源，而磁盘有 2550 ——
# 差额里绝大部分是 bin / integration-test target 的文件，用 dep-info 判会大面积误报。
# 传递可达性是 target 无关的。
#
# 识别三种模块形式（缺一个就误报）：
#   1) mod NAME;   /  mod NAME {   -> 同级 NAME.rs | NAME/mod.rs，或 父级/同名子目录
#   2) #[path="..."] mod NAME;      -> 任意路径
#   3) include!("...")             -> 文本包含
python3 - "$CUR" <<'PY' 2>/dev/null
import os, re, sys
out = open(sys.argv[1], 'a')

def crate_roots():
    roots = []
    for d in sorted(os.listdir('crates')) if os.path.isdir('crates') else []:
        src = os.path.join('crates', d, 'src')
        if os.path.isfile(os.path.join('crates', d, 'Cargo.toml')) and os.path.isdir(src):
            roots.append(src)
    # 只列**真实存在**的面。桌面端原在 `src-tauri/src`，已随 5c02e738 归档
    # （599 files），现由 `crates/neotrix-neobot` 承担 —— 上面那个 crates 循环
    # 已经覆盖它。⛔ 不要在这里写回 `src-tauri/src`：路径不存在会让本函数
    # 静默少扫一个面，而「少扫」与「扫过且干净」在输出上无法区分
    # （这正是 nt_scan_surface.py 报 KNOWN-GONE 的同一类病）。
    for src in ('neotrix-core/src',):
        if os.path.isdir(src):
            roots.append(src)
    return roots

def target_roots(src):
    r = []
    for c in (os.path.join(src, 'lib.rs'), os.path.join(src, 'main.rs')):
        if os.path.isfile(c):
            r.append(c)
    bd = os.path.join(src, 'bin')
    if os.path.isdir(bd):
        for f in sorted(os.listdir(bd)):
            p = os.path.join(bd, f)
            if f.endswith('.rs') and os.path.isfile(p):
                r.append(p)
            elif os.path.isdir(p) and os.path.isfile(os.path.join(p, 'main.rs')):
                r.append(os.path.join(p, 'main.rs'))
    return r

def mods_of(f):
    try:
        with open(f, encoding='utf-8', errors='ignore') as fh:
            src = fh.read()
    except Exception:
        return []
    d = os.path.dirname(f)
    stem = os.path.basename(f)[:-3]
    res = []
    for m in re.finditer(r'^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z0-9_]+)\s*[;{]', src, re.M):
        n = m.group(1)
        for c in (f'{d}/{n}.rs', f'{d}/{n}/mod.rs',
                  f'{d}/{stem}/{n}.rs', f'{d}/{stem}/{n}/mod.rs'):
            if os.path.exists(c):
                res.append(c)
                break
    # 2026-09-29 修：`[^\n]*?` 只允许 `#[path = "..."]` 与 `mod NAME` **同行**。
    # 实际代码常写成：
    #     #[path = "tests/x.rs"]      ← 换行
    #     mod x;
    # 那种写法会让本正则整条不匹配 ⇒ 真实存在的文件被判为 UNREACHABLE
    # （假阳性）。实测踩中：nt_evolution_eval_tests.rs。
    #
    # 改成 [\s\S]{0,200}? 并**限制窗口**，避免跨越到下一个 mod 声明。
    for m in re.finditer(r'#\[path\s*=\s*"([^"]+)"\][\s\S]{0,200}?\bmod\s+([A-Za-z0-9_]+)', src):
        t = m.group(1)
        for c in (f'{d}/{t}', f'{stem}/{t}', t):
            if os.path.exists(c):
                res.append(os.path.normpath(c))
                break
    for m in re.finditer(r'include!\s*\(\s*"([^"]+)"', src):
        t = m.group(1)
        for c in (f'{d}/{t}', f'{stem}/{t}'):
            if os.path.exists(c):
                res.append(os.path.normpath(c))
                break
    return res

reachable = set()
for src in crate_roots():
    seen, frontier = set(), target_roots(src)
    while frontier:
        f = frontier.pop()
        if f in seen or not os.path.exists(f):
            continue
        seen.add(f)
        reachable.add(f)
        frontier.extend(mods_of(f))

on_disk = set()
for src in crate_roots():
    for dp, dn, fn in os.walk(src):
        for f in fn:
            if f.endswith('.rs'):
                on_disk.add(os.path.normpath(os.path.join(dp, f)))

for u in sorted(on_disk - reachable):
    out.write(f'UNREACHABLE {u}\n')
out.close()
PY

# ---------- 3) TRACKED: build artifacts / runtime DB in git ----------
git ls-files 2>/dev/null | grep -E "$TRACK_GLOBS" | sed 's/^/TRACKED /' >> "$CUR"

# ---------- 4) UNCOMMITTED_DEP: a `mod X;` target that is not in git ----------
# This is the check that would have caught the 2026-09-27 broken HEAD: three
# committed files did `use crate::l6_meta::nt_approval::...` while the only
# `mod nt_approval;` lived in an uncommitted mod.rs, so a fresh clone could
# not build.
#
# Reads the WORKING TREE on purpose, not `git show HEAD:` — the whole point is
# to fire *before* the breaking commit lands, when worktree != HEAD. In CI the
# two are identical, so behaviour is unchanged there.
#
# Module resolution honours BOTH layouts:
#   sibling : foo.rs  declaring `mod bar;`  ->  bar.rs | bar/mod.rs
#   nested  : foo.rs  declaring `mod bar;`  ->  foo/bar.rs | foo/bar/mod.rs
python3 - "$CUR" <<'PY' 2>/dev/null
import subprocess, sys, re, os
out = open(sys.argv[1], 'a')
tracked = set(subprocess.run(['git', 'ls-files'], capture_output=True,
                             text=True).stdout.split())
roots = sorted(f for f in tracked if f.endswith('.rs') and os.path.exists(f))
def mods_of(f):
    try:
        with open(f, encoding='utf-8', errors='ignore') as fh:
            src = fh.read()
    except Exception:
        return []
    d, stem = os.path.dirname(f), os.path.basename(f)[:-3]
    res = []
    for m in re.finditer(r'^\s*(?:pub\s+)?mod\s+([a-z_0-9]+)\s*;', src, re.M):
        n = m.group(1)
        # resolve on the FILESYSTEM, then ask git whether it is committed.
        # Resolving against `tracked` here would be a no-op: a never-committed
        # file could never be discovered in the first place.
        for c in (f'{d}/{n}.rs', f'{d}/{n}/mod.rs',
                  f'{d}/{stem}/{n}.rs', f'{d}/{stem}/{n}/mod.rs'):
            if os.path.exists(c):
                res.append(c)
                break
    return res
# transitive closure: committed code can reach an uncommitted file through
# another uncommitted file, so walk until fixpoint
seen, frontier, need = set(roots), list(roots), set()
while frontier:
    nxt = []
    for f in frontier:
        for c in mods_of(f):
            if c not in tracked:
                need.add(c)
            if c not in seen:
                seen.add(c)
                nxt.append(c)
    frontier = nxt
for c in sorted(need):
    out.write(f'UNCOMMITTED_DEP {c}\n')
out.close()
PY

sort -u "$CUR" -o "$CUR"

# ---------- diff against baseline ----------
# Baseline lines starting with '#' are comments (rationale), stripped before diff
# so the file can self-document WHY a known offender is tolerated.
CUR_C=$(mktemp)
BASE_C=$(mktemp)
grep -v '^#' "$CUR" 2>/dev/null | sort -u > "$CUR_C" || : > "$CUR_C"
if [ -f "$BASELINE" ]; then
  grep -v '^#' "$BASELINE" | sort -u > "$BASE_C" || : > "$BASE_C"
  comm -23 "$CUR_C" "$BASE_C" > "$NEW"    # in tree, not in baseline => NEW
  comm -13 "$CUR_C" "$BASE_C" > "$GONE"   # in baseline, gone from tree => RESOLVED
else
  cp "$CUR_C" "$NEW"
  : > "$GONE"
fi

N_EMPTY=$(grep -c '^EMPTY ' "$NEW" || true)
N_UNDECL=$(grep -c '^UNDECLARED ' "$NEW" || true)
N_UNREACH=$(grep -c '^UNREACHABLE ' "$NEW" || true)
N_TRACK=$(grep -c '^TRACKED ' "$NEW" || true)
N_DEP=$(grep -c '^UNCOMMITTED_DEP ' "$NEW" || true)
N_GONE=$(grep -c . "$GONE" || true)
N_BASE=$(grep -vc '^#' "$BASELINE" 2>/dev/null || true)

echo "=== NeoTrix truth-surface gate ==="
echo "baseline entries: $N_BASE   resolved since baseline: $N_GONE"
echo "NEW offenders  -> EMPTY:$N_EMPTY  UNDECLARED:$N_UNDECL  UNREACHABLE:$N_UNREACH  TRACKED:$N_TRACK  UNCOMMITTED_DEP:$N_DEP"

if [ "$N_GONE" -gt 0 ]; then
  echo "--- resolved (drop from baseline via --update-baseline) ---"
  cat "$GONE"
fi

  # 2026-09-28 修：条件漏了 N_DEP ⇒ UNCOMMITTED_DEP offender **只计数、从不出现在
  # 清单里**，而下面的 FAIL 文案却写「see the list above」—— 读者无从行动。
  # 那一类恰恰是最可交付性的一类（新 clone 编不过），必须列出来。
  if [ "$N_EMPTY" -gt 0 ] || [ "$N_UNDECL" -gt 0 ] || [ "$N_UNREACH" -gt 0 ] || \
     [ "$N_TRACK" -gt 0 ] || [ "$N_DEP" -gt 0 ]; then
    echo "--- NEW offenders (regression, not in baseline) ---"
    cat "$NEW"
  fi

if [ "$N_UNREACH" -gt 0 ]; then
  echo
  echo "--- UNREACHABLE: 判读纪律（⛔ 不要盲删）---"
  echo "  「从不被编译」≠「功能缺失」。多数是**已归档的旧引擎**。处置前先分类："
  echo "    (a) 已归档/被取代  -> 删，或整目录归档"
  echo "    (b) 忘了加 mod     -> 确认无外部消费者后补声明（补之前先问：它能编译吗）"
  echo "    (c) 声明被注释掉   -> 那是「编译不过」，不是「忘了声明」，处置方式不同"
  echo "  盲删会打断活路径：DIR-REMEDY §2.5 记「导出 ≠ 调用」已错过 3 次。"
fi


if [ "$UPDATE" -eq 1 ]; then
  # preserve any leading comment block, replace the entry list
  if [ -f "$BASELINE" ]; then
    grep '^#' "$BASELINE" > "$BASELINE.tmp" || : > "$BASELINE.tmp"
  else
    : > "$BASELINE.tmp"
  fi
  cat "$BASELINE.tmp" "$CUR_C" > "$BASELINE"
  rm -f "$BASELINE.tmp"
  echo "baseline updated: $BASELINE now has $(grep -vc '^#' "$BASELINE") entries"
  exit 0
fi

if [ "$STRICT" -eq 1 ] && [ -s "$NEW" ]; then
  echo "FAIL(strict): $((N_EMPTY + N_UNDECL + N_UNREACH + N_TRACK + N_DEP)) new truth-drift offenders."
  echo "  EMPTY          -> delete the file, or implement it (a 0-byte 'pub mod' is a lie)"
  echo "  UNDECLARED     -> add 'mod <name>;' to the sibling mod.rs, or delete the file"
  echo "  UNREACHABLE    -> on disk but not reachable from any crate root (never compiled)."
  echo "                    Classify before acting: archived / forgot-mod / commented-out."
  echo "  TRACKED        -> git rm --cached <path> (and fix the matching .gitignore rule)"
  echo "  UNCOMMITTED_DEP-> committed code declares/uses this file but it is NOT in git;"
  echo "                    a fresh clone cannot build. git add it (see the list above)."
  exit 1
fi

echo "DONE(advisory)."
