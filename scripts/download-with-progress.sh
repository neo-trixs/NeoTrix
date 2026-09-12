#!/bin/bash
# nt-download: 带实时进度条的断点续传下载工具
# 用法: nt-download <url> <output>
# 特性: 断点续传 / 实时进度条 / 速度 / ETA / 信号清理

set -uo pipefail

URL="${1:?用法: nt-download <url> <output>}"
OUT="${2:?用法: nt-download <url> <output>}"
BAR_W=40

# ── 信号清理 ──────────────────────────────────
CURL_PID=""
cleanup() {
    if [ -n "$CURL_PID" ] && kill -0 "$CURL_PID" 2>/dev/null; then
        kill "$CURL_PID" 2>/dev/null
        wait "$CURL_PID" 2>/dev/null
    fi
    printf "\n"
}
trap cleanup INT TERM HUP EXIT

# ── 工具函数 ──────────────────────────────────
fmt_bytes() {
    local b=$1
    if   [ "$b" -ge 1073741824 ]; then printf "%.1f GB" "$(echo "scale=1; $b/1073741824" | bc -l 2>/dev/null || echo "0")"
    elif [ "$b" -ge 1048576 ];    then printf "%.1f MB" "$(echo "scale=1; $b/1048576" | bc -l 2>/dev/null || echo "0")"
    elif [ "$b" -ge 1024 ];       then printf "%.1f KB" "$(echo "scale=1; $b/1024" | bc -l 2>/dev/null || echo "0")"
    else printf "%d B" "$b"
    fi
}

# ── 获取远端大小 ──────────────────────────────
get_remote_size() {
    # 用 Range 请求探测总大小
    local size
    size=$(curl -sI -L -r 0-0 "$URL" 2>/dev/null \
        | tr -d '\r' \
        | awk -F'[: ]+' '/[Cc]ontent-[Rr]ange/{for(i=1;i<=NF;i++)if($i~/[0-9]/){gsub(/[^0-9]/,"",$i);if(length($i)>5){print $i;exit}}}' \
        | head -1)
    if [ -z "$size" ] || [ "$size" = "0" ]; then
        size=$(curl -sI -L "$URL" 2>/dev/null \
            | tr -d '\r' \
            | awk '/[Cc]ontent-[Ll]ength/{print $NF}' \
            | tail -1)
    fi
    echo "$size"
}

# ── 检查已有文件 ──────────────────────────────
EXISTED=0
if [ -f "$OUT" ]; then
    EXISTED=$(stat -f%z "$OUT" 2>/dev/null || echo 0)
fi

REMOTE=$(get_remote_size)

# ── 无大小信息 → 简单模式 ────────────────────
if [ -z "$REMOTE" ] || [ "$REMOTE" = "0" ]; then
    echo "⚠️  无法获取远端大小，简单模式下载"
    curl -L -C - -o "$OUT" "$URL" --progress-bar \
        --retry 10 --retry-delay 3 --retry-all-errors
    echo ""
    echo "✅ 完成: $(fmt_bytes "$(stat -f%z "$OUT" 2>/dev/null || echo 0)")"
    exit 0
fi

TOTAL=$REMOTE

echo "📥 $(basename "$OUT")"
echo "📦 总大小: $(fmt_bytes $TOTAL)"
[ "$EXISTED" -gt 0 ] && echo "♻️  已有: $(fmt_bytes $EXISTED) → 断点续传"
echo ""

# ── 后台启动 curl ─────────────────────────────
curl -L -C - -o "$OUT" "$URL" \
    --retry 20 --retry-delay 3 --retry-all-errors \
    --connect-timeout 60 --max-time 7200 \
    -s 2>/dev/null &
CURL_PID=$!

# 等待文件出现（最多 5 秒）
WAIT=0
while [ ! -f "$OUT" ] && [ "$WAIT" -lt 5 ]; do
    sleep 0.2
    WAIT=$((WAIT + 1))
done

# ── 进度条循环 ────────────────────────────────
LAST_SIZE=$EXISTED
LAST_TIME=$(date +%s%N 2>/dev/null || date +%s)
STABLE_COUNT=0

while kill -0 "$CURL_PID" 2>/dev/null; do
    CURRENT=$(stat -f%z "$OUT" 2>/dev/null || echo 0)
    NOW=$(date +%s%N 2>/dev/null || date +%s)

    # 百分比
    PCT=0
    [ "$TOTAL" -gt 0 ] && PCT=$(( CURRENT * 100 / TOTAL ))
    [ "$PCT" -gt 100 ] && PCT=100

    # 进度条
    FILLED=$(( PCT * BAR_W / 100 ))
    EMPTY=$(( BAR_W - FILLED ))
    BAR=""
    for ((i=0; i<FILLED; i++)); do BAR="${BAR}█"; done
    for ((i=0; i<EMPTY; i++)); do BAR="${BAR}░"; done

    # 速度 (字节/秒，基于 1 秒采样)
    ELAPSED=$(( NOW - LAST_TIME ))
    SPEED=0
    if [ "$ELAPSED" -gt 0 ]; then
        DELTA=$(( CURRENT - LAST_SIZE ))
        SPEED=$(( DELTA * 1000 / ELAPSED ))  # 毫秒→秒换算
    fi
    LAST_SIZE=$CURRENT
    LAST_TIME=$NOW

    # 稳定性检测 (连续 3 次无增长 = 可能卡住)
    if [ "$CURRENT" -eq "$LAST_SIZE" ] 2>/dev/null; then
        STABLE_COUNT=$((STABLE_COUNT + 1))
    else
        STABLE_COUNT=0
    fi

    # ETA
    ETA="..."
    if [ "$SPEED" -gt 0 ] && [ "$TOTAL" -gt "$CURRENT" ]; then
        REMAIN=$(( (TOTAL - CURRENT) / SPEED ))
        printf -v ETA "剩余 %dm%02ds" $((REMAIN/60)) $((REMAIN%60))
    fi

    # 输出
    printf "\r\033[K  %3d%% %s %s/%s | %s | %s" \
        "$PCT" "$BAR" "$(fmt_bytes $CURRENT)" "$(fmt_bytes $TOTAL)" \
        "$(fmt_bytes $SPEED)/s" "$ETA"

    sleep 1
done

# ── 收尾 ──────────────────────────────────────
wait "$CURL_PID" 2>/dev/null
EXIT_CODE=$?

# 清理 trap（不再需要 kill curl）
CURL_PID=""

echo ""
FINAL=$(stat -f%z "$OUT" 2>/dev/null || echo 0)

if [ $EXIT_CODE -eq 0 ] && [ "$FINAL" -ge "$TOTAL" ]; then
    echo "✅ 下载完成: $(fmt_bytes $FINAL)"
elif [ $EXIT_CODE -eq 0 ]; then
    echo "⚠️  大小不符: $(fmt_bytes $FINAL) / $(fmt_bytes $TOTAL)"
    echo "💡 重新运行可断点续传"
else
    echo "❌ 下载失败 (exit: $EXIT_CODE)"
    [ "$FINAL" -gt 0 ] && echo "💡 已下载 $(fmt_bytes $FINAL)，可断点续传"
fi
