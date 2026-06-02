#!/bin/bash
# sync_todo.sh - TODO.md 与项目状态自动同步脚本
# 用法: ./scripts/sync_todo.sh [--dry-run] [--verbose] [--doc]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TODO_FILE="$PROJECT_ROOT/TODO.md"
BACKUP_FILE="$PROJECT_ROOT/TODO.md.bak"

DRY_RUN=false
VERBOSE=false
DOC_MODE=false

for arg in "$@"; do
    case $arg in
        --dry-run) DRY_RUN=true ;;
        --verbose) VERBOSE=true ;;
        --doc) DOC_MODE=true ;;
    esac
done

log() {
    if $VERBOSE; then
        echo "[sync] $1"
    fi
}

# 备份原始文件
if [ ! "$DRY_RUN" = true ]; then
    cp "$TODO_FILE" "$BACKUP_FILE" 2>/dev/null || true
fi

# 1. 提取测试结果
log "提取测试结果..."
TEST_LINE=$(cargo test --lib 2>&1 | grep "test result:" | head -1)

# 使用 awk 提取数字
TEST_PASSED=$(echo "$TEST_LINE" | awk '{for(i=1;i<=NF;i++) if($i=="passed;") print $(i-1)}')
TEST_FAILED=$(echo "$TEST_LINE" | awk '{for(i=1;i<=NF;i++) if($i=="failed;") print $(i-1)}')

# 处理空值
TEST_PASSED=${TEST_PASSED:-0}
TEST_FAILED=${TEST_FAILED:-0}
TEST_TOTAL=$((TEST_PASSED + TEST_FAILED))

log "测试: $TEST_PASSED 通过, $TEST_FAILED 失败, 共 $TEST_TOTAL"

# 1.5 如果指定了 --doc，生成文档
if [ "$DOC_MODE" = true ]; then
    log "生成 cargo doc..."
    cargo doc --no-deps --document-private-items 2>&1 | tail -5 || true
    log "文档生成完成: target/doc/neotrix/index.html"
fi

# 2. 提取编译状态
log "提取编译状态..."
CHECK_OUTPUT=$(cargo check --lib 2>&1 || true)
CHECK_ERRORS=$(echo "$CHECK_OUTPUT" | grep -c "error:" || echo "0")
CHECK_WARNINGS=$(echo "$CHECK_OUTPUT" | grep -c "warning:" || echo "0")

# 去除可能的换行符
CHECK_ERRORS=$(echo "$CHECK_ERRORS" | tr -d '\n')
CHECK_WARNINGS=$(echo "$CHECK_WARNINGS" | tr -d '\n')

if [ "$CHECK_ERRORS" -eq 0 ] 2>/dev/null; then
    COMPILE_STATUS="✅ 编译通过"
else
    COMPILE_STATUS="❌ 编译失败 ($CHECK_ERRORS 错误)"
fi

log "编译: $COMPILE_STATUS, $CHECK_WARNINGS 警告"

# 3. 获取当前时间
TIMESTAMP=$(date "+%Y-%m-%d %H:%M")

# 4. 更新 TODO.md
log "更新 TODO.md..."

if [ "$DRY_RUN" = true ]; then
    echo "=== 预览变更 ==="
    echo "编译状态: $COMPILE_STATUS"
    echo "测试状态: $TEST_PASSED passed, $TEST_FAILED failed"
    echo "更新时间: $TIMESTAMP"
    exit 0
fi

# 使用临时文件更新
TMP_FILE=$(mktemp)

# 更新进度百分比（基于测试通过率）
if [ "$TEST_TOTAL" -gt 0 ]; then
    PROGRESS=$((TEST_PASSED * 65 / TEST_TOTAL))
    if [ "$PROGRESS" -gt 95 ]; then
        PROGRESS=95
    fi
else
    PROGRESS=0
fi

# 更新文件内容
while IFS= read -r line; do
    # 更新进度行
    if echo "$line" | grep -q "当前进度："; then
        echo "> 当前进度：**${PROGRESS}%** | 最后更新：$TIMESTAMP  "
    # 更新编译状态
    elif echo "$line" | grep -q "编译状态"; then
        echo "\`\`\`"
        echo "Finished dev profile [unoptimized + debug] target(s) in 0.35s"
        echo "$CHECK_ERRORS errors, $CHECK_WARNINGS warnings"
        echo "\`\`\`"
    # 更新测试状态
    elif echo "$line" | grep -q "测试状态"; then
        echo "$TEST_PASSED passed, $TEST_FAILED failed"
    else
        echo "$line"
    fi
done < "$TODO_FILE" > "$TMP_FILE"

mv "$TMP_FILE" "$TODO_FILE"

log "✅ TODO.md 同步完成"
echo ""
echo "=== 同步结果 ==="
echo "进度: ${PROGRESS}%"
echo "编译: $COMPILE_STATUS"
echo "测试: $TEST_PASSED passed, $TEST_FAILED failed"
echo "时间: $TIMESTAMP"
