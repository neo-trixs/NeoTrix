#!/usr/bin/env bash
set -euo pipefail

# NeoTrix KB Guard — backup / verify / auto-restore
#
# 设计要点 (cycle 207 事故教训):
#   - 备份存独立目录 ~/Library/Application Support/NeoTrix/backups/
#     绝不放在 ~/.neotrix/ 内 (该目录可能被并发 session 整体删除)
#   - 用 sqlite3 .backup API 做 WAL 安全一致快照, 不用 cp 主库
#     (WAL 模式下 cp 只拷主库文件会丢未 checkpoint 的 wal 数据)
#   - guard 模式检测 db 缺失/损坏时自动从最近备份恢复

KB_DB="${NEOTRIX_KB:-$HOME/.neotrix/knowledge.db}"
BACKUP_DIR="${NEOTRIX_BACKUP_DIR:-$HOME/Library/Application Support/NeoTrix/backups}"
KEEP_N="${KEEP_N:-10}"
LOCK_FILE="${TMPDIR:-/tmp}/neotrix-kb-guard.lock"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }

acquire_lock() {
  if ! mkdir "$LOCK_FILE" 2>/dev/null; then
    log "skip: another guard instance running ($LOCK_FILE)"
    exit 0
  fi
  trap 'rmdir "$LOCK_FILE" 2>/dev/null || true' EXIT
}

db_healthy() {
  [[ -f "$KB_DB" ]] || return 1
  local out
  out=$(sqlite3 "$KB_DB" "PRAGMA integrity_check;" 2>/dev/null) || return 1
  [[ "$out" == "ok" ]]
}

find_latest_backup() {
  ls -1t "$BACKUP_DIR"/knowledge-*.db 2>/dev/null | head -1
}

backup() {
  mkdir -p "$BACKUP_DIR"
  local stamp backup_path
  stamp=$(date '+%Y%m%d-%H%M%S')
  backup_path="$BACKUP_DIR/knowledge-$stamp.db"

  # WAL 安全快照 (sqlite 在线备份 API), 不锁库、不产生中间态
  sqlite3 "$KB_DB" ".backup '$backup_path'" 2>/dev/null
  if ! db_healthy_from "$backup_path"; then
    rm -f "$backup_path"
    log "backup FAILED: integrity of snapshot not ok, discarded"
    return 1
  fi
  # 校验打开的连接会生成 shm/wal 残留, 快照本身已自洽, 清理之
  rm -f "$backup_path-shm" "$backup_path-wal"

  # 轮转: 只保留最近 KEEP_N 份 (连带清理 .backup 产生的 shm/wal 残留)
  local count
  count=$(ls -1 "$BACKUP_DIR"/knowledge-*.db 2>/dev/null | wc -l | tr -d ' ')
  while [[ "$count" -gt "$KEEP_N" ]]; do
    local oldest
    oldest=$(ls -1t "$BACKUP_DIR"/knowledge-*.db 2>/dev/null | tail -1)
    rm -f "$oldest" "$oldest-shm" "$oldest-wal"
    count=$((count - 1))
  done

  log "backup OK: $backup_path ($(du -h "$backup_path" | cut -f1))"
}

db_healthy_from() {
  local f="$1"
  [[ -s "$f" ]] || return 1
  local out
  out=$(sqlite3 "$f" "PRAGMA integrity_check;" 2>/dev/null) || return 1
  [[ "$out" == "ok" ]]
}

restore_latest() {
  local latest
  latest=$(find_latest_backup)
  [[ -n "$latest" ]] || { log "restore FAILED: no backup found in $BACKUP_DIR"; return 1; }
  mkdir -p "$(dirname "$KB_DB")"
  cp "$latest" "$KB_DB"
  # 清掉可能残留的 WAL/SHM (库已在备份时 checkpoint)
  rm -f "$KB_DB-wal" "$KB_DB-shm"
  log "restored $KB_DB from $latest"
}

guard() {
  if db_healthy; then
    log "guard: db healthy, no action"
    return 0
  fi
  log "guard: db missing or corrupted -> attempting auto-restore"
  if restore_latest && db_healthy; then
    log "guard: auto-restore SUCCESS"
  else
    log "guard: auto-restore FAILED (no usable backup)"
    return 1
  fi
}

case "${1:-guard}" in
  backup)  acquire_lock; backup ;;
  guard)   acquire_lock; guard ;;
  restore) acquire_lock; restore_latest ;;
  check)   if db_healthy; then echo "healthy"; exit 0; else echo "unhealthy"; exit 1; fi ;;
  *) echo "Usage: $0 [guard|backup|restore|check]" >&2; exit 1 ;;
esac
