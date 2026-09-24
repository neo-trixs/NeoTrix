#!/bin/bash
# forbid(unsafe_code) 覆盖门 — 所有 workspace 成员根必须声明。
# 用法: bash scripts/check-forbid-coverage.sh
# Exit 0 = 全覆盖, Exit 1 = 有缺失 (打印缺失文件)。
# sysctl 例外: FFI 隔离 crate, 允许 allow(unsafe_code) + reason。
set -uo pipefail

FAIL=0
check_root() {
  local file="$1"
  if [ ! -f "$file" ]; then
    echo "MISSING-FILE: $file"
    FAIL=1
    return
  fi
  # 未入库的新文件是在途 WIP, 跳过 (入库后自动纳入)。
  if ! git ls-files --error-unmatch "$file" >/dev/null 2>&1; then
    echo "SKIP-UNTRACKED: $file"
    return
  fi
  if ! grep -qE '#!\[forbid\(unsafe_code\)\]|#!\[allow\(unsafe_code' "$file"; then
    echo "NO-FORBID: $file"
    FAIL=1
  fi
}

check_root "neotrix-core/src/lib.rs"
check_root "src-tauri/src/lib.rs"
for crate in neotrix-types neotrix-sysctl neotrix-consciousness neotrix-reasoning \
    neotrix-gateway neotrix-multi-agent neotrix-abilities neotrix-game \
    neotrix-neobot neotrix-audit; do
  check_root "crates/$crate/src/lib.rs"
done

if [ "$FAIL" -ne 0 ]; then
  echo "forbid coverage FAILED — 补 #![forbid(unsafe_code)] (sysctl 例外需 allow+reason)"
  exit 1
fi
echo "forbid coverage OK"
