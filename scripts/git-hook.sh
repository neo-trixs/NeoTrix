#!/bin/sh
# NeoTrix TODO 自动同步 Git Hook (pre-commit)
# 安装：ln -sf ../../scripts/git-hook.sh .git/hooks/pre-commit

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
[ -z "$PROJECT_ROOT" ] && exit 0

TODO_MD="$PROJECT_ROOT/TODO.md"
TODO_YML="$PROJECT_ROOT/TODO.yml"

# 检查 TODO.md 和 TODO.yml 是否同时被修改但不同步
if git diff --cached --name-only | grep -q "TODO.md\|TODO.yml"; then
    if [ -f "$TODO_MD" ] && [ -f "$TODO_YML" ]; then
        # 简单校验：TODO.md 的 [ ] 数量与 TODO.yml 的 pending 总量大致对齐
        todo_count=$(grep -c '\[ \]' "$TODO_MD" 2>/dev/null || echo 0)
        pending_count=$(grep -c 'status: pending' "$TODO_YML" 2>/dev/null || echo 0)
        if [ "$todo_count" -gt 0 ] && [ "$pending_count" -eq 0 ]; then
            echo "[GIT-HOOK] 警告：TODO.md 有 $todo_count 项待办，但 TODO.yml 无 pending 项"
            echo "[GIT-HOOK] 请先运行 scripts/sync_todos.py 同步后再提交"
            exit 1
        fi
    fi
fi

# 运行同步脚本（如果存在）
SYNC_SCRIPT="$PROJECT_ROOT/scripts/sync_todos.py"
if [ -f "$SYNC_SCRIPT" ]; then
    python3 "$SYNC_SCRIPT" --git-hook 2>/dev/null
fi

exit 0
