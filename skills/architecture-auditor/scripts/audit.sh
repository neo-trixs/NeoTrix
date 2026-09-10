#!/bin/bash
# Architecture Auditor — 自动化架构审计脚本
# 用法: ./audit.sh [cycle_number]

set -e

CYCLE=${1:-$(date +%s | tail -c 7)}
REPORT_DIR="/Users/neo/Downloads/neotrix/docs/1-DESIGN"
SRC_DIR="/Users/neo/Downloads/neotrix/neotrix-core/src"

echo "=== Architecture Audit Cycle $CYCLE ==="
echo "Timestamp: $(date)"
echo ""

# Phase 1: 模块健康度
echo "## Phase 1: Module Health"
echo ""

echo "### nt_core_* modules"
for dir in $SRC_DIR/core/nt_core_*/; do
  name=$(basename "$dir")
  lines=$(cat "$dir"/*.rs 2>/dev/null | wc -l | tr -d ' ')
  todos=$(grep -c "TODO\|FIXME\|unimplemented!\|panic!" "$dir"/*.rs 2>/dev/null | tail -1 | awk -F: '{print $2}' || echo "0")
  echo "$name | lines=$lines | todos=${todos:-0}"
done

echo ""
echo "### nt_* subsystems"
for dir in $SRC_DIR/neotrix/nt_*/; do
  name=$(basename "$dir")
  lines=$(find "$dir" -name "*.rs" -exec cat {} + 2>/dev/null | wc -l | tr -d ' ')
  echo "$name | lines=$lines"
done

# Phase 2: 错误处理热点
echo ""
echo "## Phase 2: Error Handling Hotspots"
grep -rn "unwrap()\|expect(" $SRC_DIR --include="*.rs" | grep -v test | awk -F: '{print $1}' | sort | uniq -c | sort -rn | head -10

# Phase 3: EventBus 连接
echo ""
echo "## Phase 3: EventBus Coverage"
publishers=$(grep -rn "CoreEvent::" $SRC_DIR --include="*.rs" | grep -v test | awk -F: '{print $1}' | sort -u | wc -l | tr -d ' ')
subscribers=$(grep -rn "subscribe\|register_hook\|on_event" $SRC_DIR --include="*.rs" | grep -v test | awk -F: '{print $1}' | sort -u | wc -l | tr -d ' ')
echo "Publishers: $publishers"
echo "Subscribers: $subscribers"

# Phase 4: KB 覆盖
echo ""
echo "## Phase 4: KB Coverage"
kb_modules=$(grep -rn "kv_store\|kv_get\|kv_set\|knowledge.db" $SRC_DIR --include="*.rs" | grep -v test | awk -F: '{print $1}' | sort -u | wc -l | tr -d ' ')
echo "KB-using modules: $kb_modules"

# Phase 5: 空壳模块
echo ""
echo "## Phase 5: Thin Modules (mod.rs < 15 lines)"
thin_count=0
for f in $(find $SRC_DIR -name "mod.rs"); do
  lines=$(wc -l < "$f" | tr -d ' ')
  if [ "$lines" -lt 15 ]; then
    echo "THIN: $f ($lines lines)"
    thin_count=$((thin_count + 1))
  fi
done
echo "Total thin modules: $thin_count"

# Phase 6: Trait 实现
echo ""
echo "## Phase 6: Trait Implementations"
trait_impls=$(grep -rn "impl.*for" $SRC_DIR --include="*.rs" | grep -v test | wc -l | tr -d ' ')
echo "Trait implementations: $trait_impls"

# Phase 7: Async 覆盖
echo ""
echo "## Phase 7: Async Coverage"
async_fns=$(grep -rn "async fn" $SRC_DIR --include="*.rs" | grep -v test | wc -l | tr -d ' ')
total_fns=$(grep -rn "pub fn\|fn " $SRC_DIR --include="*.rs" | grep -v test | wc -l | tr -d ' ')
echo "Async functions: $async_fns / $total_fns"

# Phase 8: Unsafe 审计
echo ""
echo "## Phase 8: Unsafe Usage"
unsafe_count=$(grep -rn "unsafe" $SRC_DIR --include="*.rs" | grep -v test | wc -l | tr -d ' ')
echo "Unsafe blocks: $unsafe_count"

# Phase 9: TODO 趋势
echo ""
echo "## Phase 9: TODO/FIXME Count"
todo_count=$(grep -rn "TODO\|FIXME" $SRC_DIR --include="*.rs" | grep -v test | wc -l | tr -d ' ')
panic_count=$(grep -rn "panic!\|unimplemented!\|todo!" $SRC_DIR --include="*.rs" | grep -v test | wc -l | tr -d ' ')
echo "TODO/FIXME: $todo_count"
echo "panic!/unimplemented!/todo!: $panic_count"

# Phase 10: 依赖热度
echo ""
echo "## Phase 10: Top Imports"
grep -rn "^use " $SRC_DIR --include="*.rs" | grep -v test | awk '{print $3}' | sort | uniq -c | sort -rn | head -10

echo ""
echo "=== Audit Complete ==="
echo "Report cycle: $CYCLE"
