#!/bin/bash
# add-proprietary-spdx.sh — 发布前为专有模块添加 SPDX 标记 (幂等)
#
# 用途: 在打 release tag 前运行, 为 LICENSE-EXCEPTIONS.md 列出的专有模块
#       源码文件头部添加 SPDX-License-Identifier: LicenseRef-NeoTrix-Proprietary。
# 幂等: 已含 SPDX 标记的文件跳过; 可重复运行。
# 注意: 仅添加标记行, 不修改其他内容 — 可与工作区未提交改动共存。

set -e
MARK="// SPDX-License-Identifier: LicenseRef-NeoTrix-Proprietary"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

count=0
for f in \
  $(find neotrix-core/src/neotrix/l1_body_impl -path "*nt_shield*" -name "*.rs") \
  $(find neotrix-core/src/core/nt_core_hcube -name "*.rs") \
  $(find neotrix-core/src/neotrix/l8_autonomic_impl -path "*nt_mind*" -name "*.rs"); do
  if ! head -1 "$f" | grep -q "SPDX-License-Identifier"; then
    echo "$MARK" > "$f.tmp" && cat "$f" >> "$f.tmp" && mv "$f.tmp" "$f"
    count=$((count+1))
  fi
done

echo "Added SPDX header to $count files."
echo "Verify: git diff --stat | tail -1"