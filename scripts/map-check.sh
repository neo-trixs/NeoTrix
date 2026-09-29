#!/bin/bash
# map:check — 前端 MAP 表三向对账（默认只报警不阻止；--strict 则漂移即非零退出）。
# 对账：frontend invoke("neobot_*") vs nt_commands.rs #[tauri::command] vs MAP.md 表格。
# 用法：scripts/map-check.sh [--strict]
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FE="$ROOT/apps/neobot-desktop/frontend/src"
CMD="$ROOT/apps/neobot-desktop/src/nt_commands.rs"
MAP="$ROOT/apps/neobot-desktop/frontend/MAP.md"
STRICT=0
[ "${1:-}" = "--strict" ] && STRICT=1

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# 1) 前端实际调用（带泛型与不带泛型两种写法都要抓）
grep -rhoE 'invoke(<[^>]*>)?\("neobot_[a-z_0-9]+"' "$FE" 2>/dev/null \
  | grep -oE '"neobot_[a-z_0-9]+"' | tr -d '"' | sort -u > "$tmp/fe.txt"
# listen/emit 的事件名不计（只对命令）
# 2) Rust 实际命令（含 nt_commands/ 子模块）
grep -B1 -E "pub (async )?fn neobot_" "$CMD" "$ROOT/apps/neobot-desktop/src/nt_commands/"*.rs 2>/dev/null | grep -oE "fn neobot_[a-z_0-9]+" \
  | sed 's/^fn //' | sort -u > "$tmp/rust.txt"
# 3) MAP 表格声明（分组行如 `neobot_convos/group/dm` 需展开，整行提 token）
grep -E "^\| neobot_" "$MAP" 2>/dev/null | grep -oE "neobot_[a-z_0-9]+" | sort -u > "$tmp/map.txt"

fail=0
echo "== map:check =="
echo "frontend: $(wc -l < "$tmp/fe.txt")  rust: $(wc -l < "$tmp/rust.txt")  map: $(wc -l < "$tmp/map.txt")"
echo "--- 代码有、MAP 无（MAP 漏记） ---"
comm -23 <(sort -u "$tmp/fe.txt" "$tmp/rust.txt") "$tmp/map.txt" | sed 's/^/  MISSING_MAP /' | tee "$tmp/m1" | head -20
[ -s "$tmp/m1" ] && fail=1
echo "--- MAP 有、代码无（MAP 过时/命令已删） ---"
comm -13 <(sort -u "$tmp/fe.txt" "$tmp/rust.txt") "$tmp/map.txt" | sed 's/^/  STALE_MAP /' | tee "$tmp/m2" | head -20
[ -s "$tmp/m2" ] && fail=1
echo "--- 前端调、Rust 无（断链，必修） ---"
comm -23 "$tmp/fe.txt" "$tmp/rust.txt" | sed 's/^/  BROKEN_IPC /' | tee "$tmp/m3" | head -20
if [ -s "$tmp/m3" ]; then fail=1; fi
if [ "$fail" -eq 0 ]; then echo "OK: 三向一致"; fi
echo "== map:check CLI (§7 vs bin/neobot.rs) =="
BIN="$ROOT/crates/neotrix-neobot/src/bin/neobot.rs"
# 用 awk 按 enum 块提取：EnumName -> variants
python3 - "$BIN" "$MAP" << 'PYEOF'
import re, sys
src = open(sys.argv[1], encoding='utf-8').read()
maptxt = open(sys.argv[2], encoding='utf-8').read()
# 只取 §7 之后，避免 IPC 表干扰（CLI 用空格分隔 `task claim` 形）
sec7 = maptxt.split('## 7.')[1] if '## 7.' in maptxt else ''
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
missing = sorted(e for e in expected if e not in sec7)
stale_note = '（仅核对命令存在，不核对 flags）'
if missing:
    print('MISSING_CLI_MAP:')
    for m in missing:
        print('  ', m)
    print(stale_note)
    sys.exit(1)
print(f'OK: CLI {len(expected)} 命令全在 §7 ' + stale_note)
PYEOF
cli_fail=$?
if [ "$STRICT" -eq 1 ]; then exit "$((fail | cli_fail))"; fi
exit 0
