#!/bin/bash
# 清理 cargo 锁文件，解决 "Blocking waiting for file lock" 问题
# 用法: ./scripts/cargo-clean-locks.sh

set -e

echo "🔧 清理 cargo 锁文件..."

# 杀死残留进程
pkill -9 -f cargo 2>/dev/null || true
pkill -9 -f rustc 2>/dev/null || true

# 删除锁文件
rm -f target/.cargo-lock
rm -f target/.package-cache

echo "✅ 锁文件已清理"
echo "💡 提示: 如果问题持续，尝试: cargo clean && cargo check"
