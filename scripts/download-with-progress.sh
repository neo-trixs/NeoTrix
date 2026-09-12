#!/bin/bash
# nt-download: 带实时进度条的下载工具
# 用法: nt-download <url> <output>

set -euo pipefail

URL="${1:?用法: nt-download <url> <output>}"
OUT="${2:?用法: nt-download <url> <output>}"

# 获取文件总大小
get_content_length() {
    curl -sI -L "$URL" 2>/dev/null | grep -i 'content-length' | tail -1 | tr -d '\r' | awk '{print $2}'
}

TOTAL=$(get_content_length)
if [ -z "$TOTAL" ] || [ "$TOTAL" = "0" ]; then
    echo "⚠️  无法获取文件大小，使用简单模式"
    curl -L -C - -o "$OUT" "$URL" --retry 10 --retry-delay 3 --progress-bar
    echo ""
    echo "✅ 下载完成: $(ls -lh "$OUT" | awk '{print $5}')"
    exit 0
fi

TOTAL_MB=$(echo "scale=1; $TOTAL / 1048576" | bc)

echo "📥 下载: $(basename "$OUT")"
echo "📦 大小: ${TOTAL_MB} MB"
echo "🔗 源:   $URL"
echo ""

# 后台启动下载
curl -L -C - -o "$OUT" "$URL" \
    --retry 20 --retry-delay 3 --retry-all-errors \
    --connect-timeout 60 --max-time 7200 \
    -s 2>/dev/null &
CURL_PID=$!

# 等待文件创建
sleep 1

# 进度监控循环
LAST_BYTES=0
LAST_TIME=$(date +%s)
BAR_WIDTH=40

while kill -0 $CURL_PID 2>/dev/null; do
    if [ -f "$OUT" ]; then
        CURRENT=$(stat -f%z "$OUT" 2>/dev/null || echo 0)
        NOW=$(date +%s)
        ELAPSED=$((NOW - LAST_TIME))
        
        if [ $ELAPSED -ge 1 ]; then
            SPEED=$(( (CURRENT - LAST_BYTES) / ELAPSED ))
            LAST_BYTES=$CURRENT
            LAST_TIME=$NOW
            
            if [ $SPEED -gt 0 ]; then
                REMAIN=$(( (TOTAL - CURRENT) / SPEED ))
                REM_MIN=$((REMAIN / 60))
                REM_SEC=$((REMAIN % 60))
            else
                REM_MIN=0
                REM_SEC=0
            fi
            
            # 百分比
            if [ "$TOTAL" -gt 0 ]; then
                PCT=$(( CURRENT * 100 / TOTAL ))
            else
                PCT=0
            fi
            
            # 进度条
            FILLED=$(( PCT * BAR_WIDTH / 100 ))
            EMPTY=$(( BAR_WIDTH - FILLED ))
            BAR=""
            for ((i=0; i<FILLED; i++)); do BAR="${BAR}█"; done
            for ((i=0; i<EMPTY; i++)); do BAR="${BAR}░"; done
            
            # 速度
            if [ $SPEED -gt 1048576 ]; then
                SPEED_STR="$(echo "scale=1; $SPEED / 1048576" | bc) MB/s"
            elif [ $SPEED -gt 1024 ]; then
                SPEED_STR="$(echo "scale=1; $SPEED / 1024" | bc) KB/s"
            else
                SPEED_STR="${SPEED} B/s"
            fi
            
            # 已下载大小
            DOWN_MB=$(echo "scale=1; $CURRENT / 1048576" | bc)
            
            # 清行并输出
            printf "\r\033[K  %3d%% %s %s/%s MB | %s | 剩余 %dm%02ds" \
                "$PCT" "$BAR" "$DOWN_MB" "$TOTAL_MB" "$SPEED_STR" "$REM_MIN" "$REM_SEC"
        fi
    fi
    sleep 0.5
done

# 等待进程结束
wait $CURL_PID 2>/dev/null
EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    FINAL_SIZE=$(ls -lh "$OUT" | awk '{print $5}')
    echo "✅ 下载完成: $FINAL_SIZE"
else
    echo "❌ 下载失败 (exit code: $EXIT_CODE)"
    # 断点续传提示
    if [ -f "$OUT" ]; then
        CURRENT=$(stat -f%z "$OUT" 2>/dev/null || echo 0)
        DOWN_MB=$(echo "scale=1; $CURRENT / 1048576" | bc)
        echo "💡 已下载 ${DOWN_MB} MB，可断点续传"
    fi
fi
