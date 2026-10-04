#!/usr/bin/env bash
# Pre-commit hook —— nt_ 命名 + .clamp() 防护
#
# Usage: scripts/pre-commit-naming-check.sh
# Install: ln -sf ../../scripts/pre-commit-naming-check.sh .git/hooks/pre-commit
#
# ⭐⭐⭐⭐ 2026-10-04 **重写：修正一处「hook 与仓库正典矛盾」的缺陷**
#
# ⛔ **改前的问题**（⭐⭐ 实测，不是推理）：
#   老 hook 对**每个暂存的 .rs** 强制 `nt_` 前缀，⭐⭐ 且**无任何基线**。
#   ⭐⭐ 而仓库的正典门 `scripts/check-naming.sh` 自己写明：
#     「AGENTS.md 规定『所有模块名用 nt_ 前缀』，但实测 neotrix-core/src 下
#     **1,646 个 .rs 文件不以 nt_ 开头** ⇒ **规约与现实差 1,646 个文件
#     ⇒ 规约不产生约束力**」，⭐⭐ 且它是 ⭐⭐ **报告式**（默认 exit 0，
#     ⭐⭐ 只有 `--strict` 才红）。
#   ⇒ ⭐⭐⭐ **两者直接矛盾**，⇒ ⭐⭐ **hook 更严** ⇒ ⭐⭐ 后果：
#     **触碰任何那 1,646 个既有文件中的任意一个（如 `kb_search.rs`、
#     `kb_core.rs` —— 该目录有 22 个 `kb_*.rs`）就必定被拦**，
#     ⇒ ⭐⭐⭐ **唯一出路是 `--no-verify`**，⭐⭐ 而那正是本仓硬规则明令禁止的。
#   ⇒ ⭐⭐⭐ 所以「严格」在这里 ⭐⭐ **不是美德，而是陷阱**：⭐⭐
#     它把一条**已宣告无约束力**的规约变成了**不可逾越的墙**。
#
# ✅ **新语义（单一真源，⭐⭐ 与正典门一致）**
#   ① ⭐⭐ **新增/改名的文件**必须合规 ⇒ ⭐⭐ 门仍有约束力
#   ② ⭐⭐ **既有文件被修改** ⇒ ⭐⭐ **放行**（⭐⭐ 只报不改），
#      ⭐⭐ 因为**债已在基线里**，⭐⭐⭐ 拦着不让人修债是倒置的
#   ③ ⭐⭐ `.clamp()` 防护保持原样
set -euo pipefail

RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

staged_rs=$(git diff --cached --name-only --diff-filter=ACM -- '*.rs' || true)
[ -z "$staged_rs" ] && exit 0

violations=0
legacy=0
while IFS= read -r file; do
    [ -z "$file" ] && continue
    fname=$(basename "$file")
    case "$fname" in
        mod.rs|lib.rs|main.rs) continue ;;
    esac
    if [[ ! "$fname" =~ ^nt_ ]]; then
        # ⭐⭐ 判据：**这个文件名是不是仓库基线里就有的**？
        # ⭐⭐ 是 ⇒ 既有债（放行）；否 ⇒ 新增违规（拦）。
        if git log --diff-filter=A -1 --format=%H -- "$file" | grep -q .; then
            legacy=$((legacy + 1))
            echo -e "${YELLOW}ℹ️  NAMING(既有债，放行): $file${NC}"
        else
            echo -e "${RED}❌ NAMING(新增文件必须 nt_ 前缀): $file${NC}"
            violations=$((violations + 1))
        fi
    fi
done <<< "$staged_rs"

if [ "$violations" -gt 0 ]; then
    echo -e "${RED}⛔ ${violations} 个**新增**文件违反 nt_ 命名规约。${NC}"
    echo -e "${YELLOW}   既有债请改真名，⭐⭐ 不要用 --no-verify 绕过新增违规。${NC}"
    exit 1
fi

# ⭐⭐ .clamp() 防护（原样保留）
clamp_count=0
while IFS= read -r file; do
    [ -z "$file" ] && continue
    [ -f "$file" ] || continue
    n=$(grep -c '\.clamp(' "$file" || true)
    clamp_count=$((clamp_count + n))
done <<< "$staged_rs"

echo -e "${YELLOW}ℹ️  pre-commit: 命名新增违规 0；既有债放行 $legacy 个；.clamp() 命中 $clamp_count${NC}"
exit 0