#!/usr/bin/env bash
# NeoTrix Session-Batch Absorption Wrapper
# 封装 neotrix-experience absorb 的流程优化（根因修复自 cycle 290-386 挖掘）：
#   1. 锁检测: knowledge.db 被并发进程持写锁时等待而非立即 panic
#   2. 幂等确认: 吸收后 sqlite 查 session_id 确认落盘（不信命令输出）
#   3. 失败重试: absorb 失败时按指数退避重试（最优解: 错误→改参数→重试≤2次）
#   4. 汇总报告: 成功/跳过/失败 + KB 总量
#
# Usage: bash scripts/absorb-session-batch.sh <file...>  (支持 glob)
# Env:   ABSORB_RETRY (默认 2), ABSORB_LOCK_WAIT (默认 120s)

set -uo pipefail

KB="${KB:-$HOME/.neotrix/knowledge.db}"
RETRY_MAX="${ABSORB_RETRY:-2}"
LOCK_WAIT="${ABSORB_LOCK_WAIT:-120}"
OK=0; SKIP=0; FAIL=0
FAILED_FILES=()

log_info() { echo "  · $1"; }
log_ok()   { echo "  ✓ $1"; }
log_fail() { echo "  ✗ $1"; }

# 等待 KB 写锁释放: 有进程持有 db / db-wal 时等待
wait_lock() {
    local waited=0
    while [ $waited -lt "$LOCK_WAIT" ]; do
        if ! lsof "$KB" "$KB-wal" "$KB-shm" >/dev/null 2>&1; then
            return 0
        fi
        sleep 5
        waited=$((waited+5))
    done
    log_fail "KB 锁等待超时 (${LOCK_WAIT}s): $KB 仍被占用"
    return 1
}

# 校验文件: 存在 + 合法 JSON + entries 数组
validate_file() {
    local f="$1"
    [ -f "$f" ] || { log_fail "文件不存在: $f"; return 1; }
    python3 -c "
import json, sys
d = json.load(open('$f'))
if isinstance(d, dict) and 'entries' in d and len(d['entries']) > 0:
    sys.exit(0)
else:
    print('  ✗ 结构非法: 需 dict 且 entries 非空')
    sys.exit(1)
" 2>/dev/null || return 1
    return 0
}

# 幂等确认: 该 cycle 的分支已存在（value 是压缩 BLOB，不能 LIKE 匹配 session_id）
# 改用 key 前缀 branch_<cycle>_ 判断（cycle 是 session 文件内的唯一标识）
is_absorbed() {
    local cycle="$1"
    sqlite3 "$KB" "SELECT count(*) FROM kv_store WHERE key LIKE 'branch_${cycle}_%';" 2>/dev/null | grep -q '^[1-9]'
}

absorb_one() {
    local f="$1"
    local sid cycle
    sid=$(python3 -c "import json; print(json.load(open('$f')).get('session_id',''))" 2>/dev/null)
    cycle=$(python3 -c "import json; print(json.load(open('$f')).get('cycle',''))" 2>/dev/null)
    [ -z "$sid" ] && sid="(unknown)"
    [ -z "$cycle" ] && cycle="unknown"

    if is_absorbed "$cycle"; then
        log_ok "跳过(已吸收) $f [cycle=$cycle]"
        SKIP=$((SKIP+1))
        return 0
    fi

    local attempt=0
    while [ $attempt -le "$RETRY_MAX" ]; do
        wait_lock || break
        local out
        out=$(neotrix-experience absorb "$f" 2>&1)
        if [ $? -eq 0 ] && is_absorbed "$cycle"; then
            log_ok "吸收成功 $f [cycle=$cycle]"
            OK=$((OK+1))
            return 0
        fi
        attempt=$((attempt+1))
        if [ $attempt -le "$RETRY_MAX" ]; then
            log_info "重试 $attempt/$RETRY_MAX: $f (锁或失败)"
            sleep $((attempt*3))
        fi
    done
    log_fail "吸收失败 $f [cycle=$cycle]"
    FAIL=$((FAIL+1))
    FAILED_FILES+=("$f")
}

# ── Main ──
[ $# -eq 0 ] && { echo "用法: bash $0 <session-batch-*.json>"; exit 1; }

echo "╔══════════════════════════════════════╗"
echo "║  Session-Batch Absorb (锁检测+重试)  ║"
echo "╚══════════════════════════════════════╝"
echo "  KB: $KB"
echo "  文件: $# 个"
echo ""

for f in "$@"; do
    echo "── $(basename "$f")"
    if validate_file "$f"; then
        absorb_one "$f"
    else
        log_fail "校验失败: $f"
        FAIL=$((FAIL+1))
        FAILED_FILES+=("$f")
    fi
done

echo ""
echo "══════════════════════════════════════"
echo "  完成: $OK 成功, $SKIP 跳过(已吸收), $FAIL 失败"
if [ ${#FAILED_FILES[@]} -gt 0 ]; then
    echo "  失败文件:"
    for f in "${FAILED_FILES[@]}"; do echo "    $f"; done
fi
echo "  KB 总量: $(sqlite3 "$KB" "SELECT count(*) FROM kv_store WHERE key LIKE 'branch_%';" 2>/dev/null) entries"
exit $(( FAIL > 0 ? 1 : 0 ))
