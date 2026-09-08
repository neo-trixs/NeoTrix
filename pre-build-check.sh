#!/bin/bash
# pre-build-check.sh — 构建前检查 UI 修复是否在位
# 用法: bash pre-build-check.sh

set -e

echo "=== NeoTrix 构建前检查 ==="

# 检查 1: glass-side border-radius
echo -n "[1] glass-side border-radius: "
if grep -q "border-radius: 14px 0 0 14px" neocodex-frontend/src/styles/index.css; then
    echo "✓ 正确"
else
    echo "✗ 缺失! 需要修复"
    exit 1
fi

# 检查 2: glass-L1 border-radius
echo -n "[2] glass-L1 border-radius: "
if grep -q "border-radius: 0 14px 14px 0" neocodex-frontend/src/styles/index.css; then
    echo "✓ 正确"
else
    echo "✗ 缺失! 需要修复"
    exit 1
fi

# 检查 3: fadeIn keyframe
echo -n "[3] @keyframes fadeIn: "
if grep -q "@keyframes fadeIn" neocodex-frontend/src/styles/index.css; then
    echo "✓ 正确"
else
    echo "✗ 缺失! 需要修复"
    exit 1
fi

# 检查 4: Sidebar hover actions absolute
echo -n "[4] Sidebar hover actions absolute: "
if grep -q "absolute right-0 top-0 bottom-0" neocodex-frontend/src/components/Sidebar.tsx; then
    echo "✓ 正确"
else
    echo "✗ 缺失! 需要修复"
    exit 1
fi

# 检查 5: PTY snake_case
echo -n "[5] PTY session_id snake_case: "
if grep -q "session_id: sessionId" neocodex-frontend/src/api/pty.ts; then
    echo "✓ 正确"
else
    echo "✗ 缺失! 需要修复"
    exit 1
fi

# 检查 6: API adapter 存在
echo -n "[6] API adapter.ts: "
if [ -f neocodex-frontend/src/api/adapter.ts ]; then
    echo "✓ 存在"
else
    echo "✗ 缺失! 需要创建"
    exit 1
fi

# 检查 7: model-pool API 存在
echo -n "[7] API model-pool.ts: "
if [ -f neocodex-frontend/src/api/model-pool.ts ]; then
    echo "✓ 存在"
else
    echo "✗ 缺失! 需要创建"
    exit 1
fi

# 检查 8: proxy-pool API 存在
echo -n "[8] API proxy-pool.ts: "
if [ -f neocodex-frontend/src/api/proxy-pool.ts ]; then
    echo "✓ 存在"
else
    echo "✗ 缺失! 需要创建"
    exit 1
fi

# 检查 9: model_pool.rs 命令存在
echo -n "[9] Backend model_pool.rs: "
if [ -f src-tauri/src/commands/model_pool.rs ]; then
    echo "✓ 存在"
else
    echo "✗ 缺失! 需要创建"
    exit 1
fi

# 检查 10: proxy_pool.rs 命令存在
echo -n "[10] Backend proxy_pool.rs: "
if [ -f src-tauri/src/commands/proxy_pool.rs ]; then
    echo "✓ 存在"
else
    echo "✗ 缺失! 需要创建"
    exit 1
fi

echo ""
echo "=== 所有检查通过，可以构建 ==="
