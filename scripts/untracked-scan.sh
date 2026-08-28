#!/usr/bin/env bash
# 全量编译前 untracked 扫描 (Task #6): 列出未跟踪的 .rs 文件。
# 防止类似 nt_mind_rsi_exam.rs / nt_core_capability_tree 的未纳管/破损 .rs
# 在缓存失效时阻断整个 neotrix 构建 (R-P16 持久化验证前置)。
# 兼容 bash/zsh (不依赖 mapfile)。
set -u
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

files="$(git ls-files --others --exclude-standard -- '*.rs')"

if [ -z "$files" ]; then
    echo "[untracked-scan] 无未跟踪 .rs 文件。可安全全量构建。"
    exit 0
fi

count="$(printf '%s\n' "$files" | grep -c .)"
echo "[untracked-scan] 发现 ${count} 个未跟踪 .rs 文件 (可能导致构建/测试失败):"
printf '%s\n' "$files"
echo "[untracked-scan] 建议: git add 纳管或确认其为临时文件后继续构建。"
exit 1
