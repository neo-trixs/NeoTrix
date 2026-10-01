#!/usr/bin/env bash
# nt_audit_bootstrap.sh — 一条命令让**全套审计工具**在干净检出上可跑。
#
# 为什么存在【实测动因，本会话最重要的一条结构性发现】
#   本会话新增了 6 个审计工具，它们**全部**依赖 `.project-map/edges-*.jsonl`：
#     nt_decompose · nt_parity_ref · nt_fn_drift · nt_dup_dead ·
#     nt_callgraph · nt_calledges
#   而 `.project-map/` 是 **gitignored**（.gitignore:293），且
#   **Makefile 里没有任何目标生成它** ⇒ **干净检出上这套审计能力等于零**。
#   换句话说：我交付的审计能力只有我自己这台机器能用，别人 clone 下来全是死工具。
#   这不是"低频生成物"的小事，而是**能力不可分发**。
#
# 本脚本的取舍（诚实写明，不要误读为"一键全量"）：
#   --scope quick  默认：抽取 workspace member **逐个**跑，落 per-crate 边表。
#                    实测单 crate ~13s（nt-core-capability-tree 6241 边 100%），
#                    11 个 member 顺序跑约 2-5 分钟（视机器）。
#                    工具链支持 per-crate db：查单一 crate 的问题用这个就够。
#   --scope full   全量合并成 edges-all.jsonl。实测 **~30min / 168MB**。
#                    只有需要**跨 crate 全局**分诊（fn_drift 的 815 对里大量是
#                    跨 crate 的）时才值得。⛔ 不要在 CI 里跑。
#   --merge-only   只把已有 per-crate 边表合并，不重新抽取。
#
# 用法：
#   bash scripts/ops/nt_audit_bootstrap.sh              # quick scope
#   bash scripts/ops/nt_audit_bootstrap.sh --scope full
#   bash scripts/ops/nt_audit_bootstrap.sh --merge-only
#   bash scripts/ops/nt_audit_bootstrap.sh --list       # 列出 member 与将产出的文件
set -uo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO" || exit 2
OUT_DIR=".project-map"
EDGE_DIR="$OUT_DIR"
SCOPE="quick"

while [ $# -gt 0 ]; do
  case "$1" in
    --scope) SCOPE="$2"; shift 2 ;;
    --scope=*) SCOPE="${1#*=}"; shift ;;
    --merge-only) MERGE_ONLY=1; shift ;;
    --list) LIST_ONLY=1; shift ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

members() {
  # workspace member 的 crate 目录（按 Cargo.toml 的 members 解析，不硬编码）。
  # ⚠️ 必须**从脚本自身位置**推导 REPO，不能依赖调用时的 cwd ——
  # 沙箱里 `cd /tmp/x && bash .../nt_audit_bootstrap.sh` 时 cwd 不是仓库根，
  # 用 cwd 会解析出 0 个 member 并**静默**报「0 成功 / 0 失败」。
  # 这条是我实测踩到的：v1 用 os.getcwd()，在沙箱副本里产出 0 个 member。
  python3 - "$REPO" <<'PY'
import os, re, sys
root = os.path.abspath(sys.argv[1])
p = os.path.join(root, 'Cargo.toml')
if not os.path.isfile(p):
    sys.stderr.write('cannot find %s\n' % p)
    sys.exit(2)
txt = open(p, encoding='utf-8').read()
m = re.search(r'^\[workspace\]\s*(.*?)(?=^\[|\Z)', txt, re.S | re.M)
if not m:
    sys.stderr.write('no [workspace] section in %s\n' % p)
    sys.exit(2)
mm = re.search(r'members\s*=\s*\[(.*?)\]', m.group(1), re.S)
if not mm:
    sys.stderr.write('no workspace.members in %s\n' % p)
    sys.exit(2)
found = 0
for entry in mm.group(1).split(','):
    e = entry.strip().strip('"\'')
    if not e or e.startswith('#'):
        continue
    d = e if os.path.isabs(e) else os.path.join(root, e)
    if os.path.isdir(d) and os.path.isfile(os.path.join(d, 'Cargo.toml')):
        print(os.path.relpath(d, root))
        found += 1
if not found:
    sys.stderr.write('workspace.members listed but none resolve to a crate dir\n')
    sys.exit(2)
PY
}

if [ -n "${LIST_ONLY:-}" ]; then
  echo "workspace members（将逐个抽取）:"
  members | sed 's/^/  /'
  echo
  echo "产物: $EDGE_DIR/edges-<crate-name>.jsonl"
  exit 0
fi

if [ ! -f scripts/ops/nt_calledges.py ]; then
  echo "missing scripts/ops/nt_calledges.py — cannot bootstrap" >&2
  exit 2
fi

mkdir -p "$EDGE_DIR"

if [ "${MERGE_ONLY:-}" = "1" ]; then
  echo "[bootstrap] 合并已有 per-crate 边表 -> $EDGE_DIR/edges-all.jsonl"
  : > "$EDGE_DIR/edges-all.jsonl.part"
  n=0
  for f in "$EDGE_DIR"/edges-*.jsonl; do
    [ -e "$f" ] || continue
    [ "$(basename "$f")" = "edges-all.jsonl" ] && continue
    cat "$f" >> "$EDGE_DIR/edges-all.jsonl.part"
    n=$((n + 1))
  done
  mv "$EDGE_DIR/edges-all.jsonl.part" "$EDGE_DIR/edges-all.jsonl"
  echo "[bootstrap] merged $n per-crate db(s); total lines: $(grep -c '' "$EDGE_DIR/edges-all.jsonl")"
  exit 0
fi

if [ "$SCOPE" = "full" ]; then
  echo "[bootstrap] scope=full —— 实测约 30min / 168MB。⛔ 不要在 CI 跑。"
  echo "[bootstrap] 开始抽取全部 workspace member 并合并…"
  : > "$EDGE_DIR/edges-all.jsonl.part"
  i=0
  members | while read -r d; do
    i=$((i + 1))
    name="$(basename "$d")"
    echo "[bootstrap] ($i) $name"
    python3 scripts/ops/nt_calledges.py --crate "$d" --out "$EDGE_DIR/edges-$name.jsonl"
    [ -f "$EDGE_DIR/edges-$name.jsonl" ] && cat "$EDGE_DIR/edges-$name.jsonl" >> "$EDGE_DIR/edges-all.jsonl.part"
  done
  mv "$EDGE_DIR/edges-all.jsonl.part" "$EDGE_DIR/edges-all.jsonl"
  echo "[bootstrap] done: $(grep -c '' "$EDGE_DIR/edges-all.jsonl") edges"
  exit 0
fi

# quick scope：逐 crate 抽取，不合并（跨 crate 分诊需要 --scope full 或 --merge-only）
echo "[bootstrap] scope=quick —— 逐 crate 抽取（单 crate 实测 ~13s）"
echo "[bootstrap] 需要**跨 crate** 分诊时再跑 --scope full 或 --merge-only"
ok=0; fail=0
while read -r d; do
  name="$(basename "$d")"
  printf '  %-34s ' "$name"
  if python3 scripts/ops/nt_calledges.py --crate "$d" --out "$EDGE_DIR/edges-$name.jsonl" 2>"$EDGE_DIR/.last-err"; then
    lines=$(grep -c '' "$EDGE_DIR/edges-$name.jsonl" 2>/dev/null || echo 0)
    echo "ok ($lines edges)"
    ok=$((ok + 1))
  else
    echo "FAIL — $(tail -1 "$EDGE_DIR/.last-err")"
    fail=$((fail + 1))
  fi
done < <(members)
rm -f "$EDGE_DIR/.last-err"

echo
echo "[bootstrap] 完成：$ok 成功 / $fail 失败"
echo "[bootstrap] 现在可跑："
echo "  python3 scripts/ops/nt_fn_drift.py --db $EDGE_DIR/edges-<crate>.jsonl --only-different"
echo "  python3 scripts/ops/nt_dup_dead.py --db $EDGE_DIR/edges-<crate>.jsonl"
echo "  python3 scripts/ops/nt_decompose.py atoms --db $EDGE_DIR/edges-<crate>.jsonl --root <sym>"
echo "⛔ 跨 crate 全局分诊需要 edges-all.jsonl：--scope full（或已有 per-crate 时 --merge-only）"
[ "$fail" -gt 0 ] && exit 1
exit 0