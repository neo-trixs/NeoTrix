#!/usr/bin/env bash
# file-edit-safety.sh — Pre-edit backup + post-edit integrity check
# Usage: source file-edit-safety.sh; protect_file "path/to/file.rs"
# Or:   ./file-edit-safety.sh check "path/to/file.rs" <expected_min_lines>
set -euo pipefail

BACKUP_DIR="${NEOTRIX_SAFETY_BACKUP_DIR:-/tmp/neotrix-edit-backups}"
mkdir -p "$BACKUP_DIR"

protect_file() {
    local f="$1"
    if [ ! -f "$f" ]; then return 0; fi
    local hash; hash=$(md5 -q "$f" 2>/dev/null || sha256sum "$f" | cut -d' ' -f1)
    local lines; lines=$(wc -l < "$f")
    local backup="$BACKUP_DIR/$(echo "$f" | tr '/' '_')"
    cp "$f" "$backup"
    echo "[safety] backed up $f ($lines lines, hash=$hash)" >&2
}

verify_file() {
    local f="$1"
    local min_lines="${2:-10}"
    if [ ! -f "$f" ]; then
        echo "[safety] ERROR: file $f no longer exists!" >&2
        return 1
    fi
    local lines; lines=$(wc -l < "$f")
    if [ "$lines" -lt "$min_lines" ]; then
        local backup="$BACKUP_DIR/$(echo "$f" | tr '/' '_')"
        echo "[safety] WARNING: $f truncated from expected >=$min_lines to $lines lines!" >&2
        if [ -f "$backup" ]; then
            local b_lines; b_lines=$(wc -l < "$backup")
            echo "[safety] backup has $b_lines lines" >&2
            echo "[safety] to restore: cp $backup $f" >&2
        fi
        return 1
    fi
    echo "[safety] OK: $f has $lines lines (min=$min_lines)" >&2
    return 0
}

case "${1:-}" in
    check)
        verify_file "${2:?}" "${3:-10}"
        ;;
    backup)
        protect_file "${2:?}"
        ;;
    *)
        echo "Usage: $0 {check|backup} <file> [min_lines]" >&2
        exit 1
        ;;
esac
