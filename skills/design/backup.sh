#!/bin/bash
# preview-ui-v2.html 备份脚本
# 用法: bash design/backup.sh [备注]
TS=$(date +%Y%m%d_%H%M%S)
NOTE="${1:-auto}"
cp preview-ui-v2.html "design/backups/preview-ui-v2.html.bak.${TS}"
echo "[${TS}] backed up (${NOTE})" >> design/CHANGELOG.md
echo "Backup saved: design/backups/preview-ui-v2.html.bak.${TS}"
