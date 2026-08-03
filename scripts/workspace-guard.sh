#!/bin/bash
# 工作区守卫 (进化点5: R-P53 并发 reset 盲区)
# 检测 git status 未预期清空 (staged 文件消失 / 工作区被还原) 时报警
# 部署: 加入 pre-commit hook 或定时任务

REPO_ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null)}"
[[ -z "$REPO_ROOT" ]] && { echo "Not in a git repo"; exit 1; }
cd "$REPO_ROOT"

LOG_FILE="${HOME}/.neotrix/workspace-guard-violations.log"
mkdir -p "$(dirname "$LOG_FILE")"

# 记录当前状态快照
snapshot_file="/tmp/neotrix-ws-guard-snapshot-$(date +%s).txt"
git status --short > "$snapshot_file"

# 如果有历史快照, 对比
prev_snapshot=$(ls -t /tmp/neotrix-ws-guard-snapshot-*.txt 2>/dev/null | head -2 | tail -1 || true)
if [[ -n "$prev_snapshot" && -f "$prev_snapshot" ]]; then
    # 检测: 原本有 staged 文件现在没了
    prev_staged=$(grep -c '^[A-Z]' "$prev_snapshot" 2>/dev/null || echo 0)
    curr_staged=$(grep -c '^[A-Z]' "$snapshot_file" 2>/dev/null || echo 0)
    if [[ $prev_staged -gt 0 && $curr_staged -eq 0 ]]; then
        echo "[$(date -Is)] ⚠️ STAGED FILES LOST (was $prev_staged, now 0) — possible git reset --hard" | tee -a "$LOG_FILE"
        echo "  Prev:" | tee -a "$LOG_FILE"
        cat "$prev_snapshot" | sed 's/^/    /' | tee -a "$LOG_FILE"
        echo "  Curr:" | tee -a "$LOG_FILE"
        cat "$snapshot_file" | sed 's/^/    /' | tee -a "$LOG_FILE"
    fi

    # 检测: 原本有 modified 文件现在回到 clean (可能被 checkout)
    prev_modified=$(grep -c '^ M' "$prev_snapshot" 2>/dev/null || true)
    curr_modified=$(grep -c '^ M' "$snapshot_file" 2>/dev/null || true)
    if [[ $prev_modified -gt 0 && $curr_modified -eq 0 ]]; then
        echo "[$(date -Is)] ⚠️ MODIFIED FILES REVERTED (was $prev_modified, now 0) — possible git checkout/reset" | tee -a "$LOG_FILE"
    fi

    # 检测: HEAD 移动但工作区未更新 (detached HEAD / reset --hard)
    prev_head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null | head -1)
    # Note: this runs after the fact; real-time guard would need a background daemon
fi

# 保留最近 10 个快照
ls -t /tmp/neotrix-ws-guard-snapshot-*.txt 2>/dev/null | tail -n +11 | xargs -r rm -f

exit 0