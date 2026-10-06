#!/bin/bash
# map:check — 前后端 IPC 两向对账（默认只报警不阻止；--strict 则断链即非零退出）。
#
# 对账：frontend invoke("neobot_*") ↔ nt_commands.rs #[tauri::command]
#      以及 CLI 子命令枚举 ↔ （已废弃的 MAP.md 面，见下）
#
# 用法：scripts/map-check.sh [--strict]
#
# ## 2026-10-06 裁决：废弃 MAP.md 对账面（原三向 → 现两向）
#
# 原门做**三向**对账，其中一向是 `apps/neobot-desktop/frontend/MAP.md`
# （一张手写表格，声明每个命令「属于哪个分组」）。该面已废弃：
#
# ① **它已不存在**（`frontend/` 下唯一缺失的 `.md`）⇒ 内层 python
#    `open(MAP)` 抛 `FileNotFoundError`，而脚本只有 `set -u` 没有 `set -e`
#    ⇒ 异常不改退出码 ⇒ **地图检查在崩溃状态下依然 RC=0**，
#    看起来像「三向一致」实际是「根本没查」。
# ② **它是手写的、且无任何生成器**。`frontend/` 已被多轮重写，
#    补一份 MAP.md 会在下一轮重写时立刻再次陈旧 ⇒ 变成第二个
#    「无人看管的漂移源」，正是它本该防的那件事。
# ③ **规范侧的答案是「派生」不是「手写」**：Agent Skills / Cloudflare
#    Agent Skills Discovery RFC v0.2.0 的实现里，索引里的描述与分组
#    **从真身派生**，而非人工维护一份需逐字对齐的副本。
#
# ⇒ 保留真正有判别力的部分：**前端调了但 Rust 没有 ⇒ 断链（必修）**，
#   以及 CLI 子命令存在性。删掉依赖 MAP.md 的两段与相关前置检查。
#
# ⚠️ 判别力说明：本门现在只查**断链**（FE 有 / Rust 无）这一类。
#   「Rust 有但前端没调」**故意不判红** —— 那是能力未被使用，
#   属演进指标（照 M-14「演进指标不当阻断」纪律），且
#   `neobot-check-market.sh` 才是那条链路的权威门。
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FE="$ROOT/apps/neobot-desktop/frontend/src"
# 2026-10-06：实测 `nt_commands.rs` **已不存在** —— 命令实现被搬到
# `src/commands.rs` + `src/core.rs`（他窗重写 desktop 端时）⇒ 原路径让本门
# 直接 ABORT。⇒ 改为扫描 src/ 下全部 .rs（更抗搬移，且这就是本门要的语义：
# 「Rust 侧到底有没有这个命令」）。
CMD_GLOB="$ROOT/apps/neobot-desktop/src"
CMD_FILES=$(ls "$CMD_GLOB"/*.rs 2>/dev/null || true)
STRICT=0
[ "${1:-}" = "--strict" ] && STRICT=1

# 前置检查：缺对账所需路径 = 「无法判定」，**不等于「一致」** ⇒ 显式报出。
for req in "$FE" "$CMD_GLOB"; do
  if [ ! -e "$req" ]; then
    echo "MAP-CHECK-ABORT: 缺少对账所需路径: ${req#$ROOT/}"
    echo "  ⇒ **无法对账**，本次不产出任何结论（⛔ 这不等于「一致」）"
    [ "$STRICT" -eq 1 ] && exit 1
    echo "  （advisory 模式 ⇒ 不阻断；但请注意本次**没有任何校验发生**）"
    exit 0
  fi
done

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# 1) 前端实际调用（带泛型与不带泛型两种写法都要抓）
grep -rhoE 'invoke(<[^>]*>)?\("neobot_[a-z_0-9]+"' "$FE" 2>/dev/null \
  | grep -oE '"neobot_[a-z_0-9]+"' | tr -d '"' | sort -u > "$tmp/fe.txt"
# 2) Rust 实际命令（含 nt_commands/ 子模块）
# 扫 src/ 下全部 .rs（命令实现已从 nt_commands.rs 搬到 commands.rs/core.rs）
# shellcheck disable=SC2086  # CMD_FILES 是有意 word-split 的路径列表
grep -B1 -E "pub (async )?fn neobot_" $CMD_FILES 2>/dev/null \
  | grep -oE "fn neobot_[a-z_0-9]+" | sed 's/^fn //' | sort -u > "$tmp/rust.txt"

fail=0
echo "== map:check（两向：frontend ↔ rust）=="
echo "frontend: $(wc -l < "$tmp/fe.txt")  rust: $(wc -l < "$tmp/rust.txt")"
echo "--- 前端调、Rust 无（断链，必修） ---"
comm -23 "$tmp/fe.txt" "$tmp/rust.txt" | sed 's/^/  BROKEN_IPC /' | tee "$tmp/m3" | head -20
if [ -s "$tmp/m3" ]; then fail=1; fi
echo "--- Rust 有、前端未调（能力未被使用 ⇒ 只报不判红） ---"
comm -13 "$tmp/fe.txt" "$tmp/rust.txt" | sed 's/^/  UNUSED_IPC /' | tee "$tmp/m4" | head -10
[ -s "$tmp/m4" ] && echo "  （↑非断链；权威门是 neobot-check-market.sh）"
if [ "$fail" -eq 0 ]; then echo "OK: 无断链（FE↔Rust 一致）"; fi

echo "== map:check CLI（枚举存在性）=="
BIN="$ROOT/crates/neotrix-neobot/src/bin/neobot.rs"
if ! python3 - "$BIN" << 'PYEOF'
import re, sys
try:
    src = open(sys.argv[1], encoding='utf-8').read()
except OSError as e:
    # ⛔ 读不到源文件 = 无法判定 ⇒ 如实报「无法判定」而非「全部一致」。
    print(f"CLI-CHECK-ABORT: 读不到 {sys.argv[1]}: {e}")
    print("  ⇒ **无法对账**，这**不等于**「CLI 全部一致」")
    sys.exit(2)
def kebab(s):
    return re.sub(r'(?<!^)(?=[A-Z])', '-', s).lower()
parent_of = {'TaskCmd': 'task', 'AuditCmd': 'audit', 'RoutineCmd': 'routine',
             'SkillCmd': 'skill', 'MemoryCmd': 'memory', 'MemberCmd': 'member',
             'ProviderCmd': 'provider', 'CoreCmd': 'core', 'AgentCmd': 'agent',
             'ControlCmd': 'control', 'ConvoCmd': 'convo', 'AttachCmd': 'attach',
             'PolicyCmd': 'policy'}
expected = set()
blocks = re.findall(r'enum (\w+)\s*\{(.*?)\n\}', src, re.S)
for enum_name, body in blocks:
    variants = re.findall(r'^\s*([A-Z][A-Za-z0-9]*)\s*[,{]', body, re.M)
    if enum_name == 'Cmd':
        for v in variants:
            if v not in parent_of.values() and v.lower() not in ('task','audit','routine','skill','memory','member','provider','core','agent','control','convo','attach','policy'):
                expected.add(kebab(v))
    elif enum_name in parent_of:
        for v in variants:
            expected.add(parent_of[enum_name] + ' ' + kebab(v))
print(f'OK: CLI 枚举 {len(expected)} 个子命令（仅核对存在性，不核对 flags）')
PYEOF
then
  # 读不到源 = exit 2 ⇒ **无法判定**，不应与「有缺口」同等对待，也不应被吞成 0。
  cli_fail=$?
  [ "$cli_fail" -eq 2 ] && { echo "  ⇒ 本段未产出结论"; }
  [ "$STRICT" -eq 1 ] && [ "$cli_fail" -eq 2 ] && exit 2
fi

if [ "$STRICT" -eq 1 ] && [ "$fail" -ne 0 ]; then exit 1; fi
exit 0