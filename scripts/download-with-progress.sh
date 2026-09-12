#!/bin/bash
# nt-download: 带实时进度条的断点续传下载工具
# 用法: nt-download <url> <output>

set -uo pipefail

URL="${1:?用法: nt-download <url> <output>}"
OUT="${2:?用法: nt-download <url> <output>}"

# 获取远端文件大小
get_remote_size() {
    curl -sI -L "$URL" 2>/dev/null | tr -d '\r' | awk '/[Cc]ontent-[Ll]ength/{print $2}' | tail -1
}

# 格式化字节
fmt_bytes() {
    local b=$1
    if [ "$b" -ge 1073741824 ]; then
        echo "$(echo "scale=1; $b/1073741824" | bc) GB"
    elif [ "$b" -ge 1048576 ]; then
        echo "$(echo "scale=1; $b/1048576" | bc) MB"
    elif [ "$b" -ge 1024 ]; then
        echo "$(echo "scale=1; $b/1024" | bc) KB"
    else
        echo "${b} B"
    fi
}

# 格式化速度
fmt_speed() {
    local bps=$1
    if [ "$bps" -ge 1048576 ]; then
        echo "$(echo "scale=1; $bps/1048576" | bc) MB/s"
    elif [ "$bps" -ge 1024 ]; then
        echo "$(echo "scale=1; $bps/1024" | bc) KB/s"
    else
        echo "${bps} B/s"
    fi
}

# 已有大小
EXISTED=0
if [ -f "$OUT" ]; then
    EXISTED=$(stat -f%z "$OUT" 2>/dev/null || echo 0)
fi

REMOTE=$(get_remote_size)
if [ -z "$REMOTE" ] || [ "$REMOTE" = "0" ]; then
    echo "⚠️  无法获取远端大小，简单模式下载"
    curl -L -C - -o "$OUT" "$URL" --progress-bar --retry 10 --retry-delay 3
    echo ""
    echo "✅ 完成: $(fmt_bytes "$(stat -f%z "$OUT" 2>/dev/null || echo 0)")"
    exit 0
fi

TOTAL=$REMOTE
DOWNLOADED=$EXISTED

echo "📥 $(basename "$OUT")"
echo "📦 总大小: $(fmt_bytes $TOTAL)"
[ "$EXISTED" -gt 0 ] && echo "♻️  已有: $(fmt_bytes $EXISTED) (断点续传)"
echo ""

# 后台启动 curl (断点续传)
curl -L -C - -o "$OUT" "$URL" \
    --retry 20 --retry-delay 3 --retry-all-errors \
    --connect-timeout 60 --max-time 7200 \
    -s 2>/dev/null &
CURL_PID=$!

# 等文件出现
sleep 0.5

BAR_W=40
LAST_SPEED=0

while kill -0 $CURL_PID 2>/dev/null; do
    CURRENT=$(stat -f%z "$OUT" 2>/dev/null || echo 0)
    NOW=$(date +%s)

    # 百分比
    if [ "$TOTAL" -gt 0 ]; then
        PCT=$(( CURRENT * 100 / TOTAL ))
    else
        PCT=0
    fi
    [ "$PCT" -gt 100 ] && PCT=100

    # 进度条
    FILLED=$(( PCT * BAR_W / 100 ))
    EMPTY=$(( BAR_W - FILLED ))
    BAR=""
    for ((i=0; i<FILLED; i++)); do BAR="${BAR}█"; done
    for ((i=0; i<EMPTY; i++)); do BAR="${BAR}░"; done

    # 速度 (用文件差值估算)
    DELTA=$(( CURRENT - DOWNLOADED ))
    DOWNLOADED=$CURRENT
    [ "$DELTA" -gt 0 ] && LAST_SPEED=$DELTA

    # 剩余时间
    if [ "$LAST_SPEED" -gt 0 ] && [ "$TOTAL" -gt "$CURRENT" ]; then
        REMAIN=$(( (TOTAL - CURRENT) / LAST_SPEED ))
        REM_MIN=$((REMAIN / 60))
        REM_SEC=$((REMAIN % 60))
        ETA="剩余 ${REM_MIN}m$(printf '%02d' $REM_SEC)s"
    else
        ETA="计算中..."
    fi

    printf "\r\033[K  %3d%% %s %s/%s | %s | %s" \
        "$PCT" "$BAR" "$(fmt_bytes $CURRENT)" "$(fmt_bytes $TOTAL)" \
        "$(fmt_speed $LAST_SPEED)" "$ETA"

    sleep 1
done

wait $CURL_PID 2>/dev/null
EXIT_CODE=$?

echo ""
FINAL=$(stat -f%z "$OUT" 2>/dev/null || echo 0)
if [ $EXIT_CODE -eq 0 ] && [ "$FINAL" -ge "$TOTAL" ]; then
    echo "✅ 下载完成: $(fmt_bytes $FINAL)"
elif [ $EXIT_CODE -eq 0 ]; then
    echo "⚠️  下载结束但大小不符: $(fmt_bytes $FINAL) / $(fmt_bytes $TOTAL)"
    echo "💡 可重新运行续传"
else
    echo "❌ 下载失败 (exit: $EXIT_CODE)"
    [ "$FINAL" -gt 0 ] && echo "💡 已下载 $(fmt_bytes $FINAL)，可断点续传"
fi
