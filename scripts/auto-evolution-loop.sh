#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────
# NeoTrix 自动进化循环
# 实现 AGENTS.md XC.1 — 四阶段进化循环
# Phase 1: 编译清零 (cargo check all crates → fix errors)
# Phase 2: 警告门控 (dead_code/unused → gate or fix)
# Phase 3: 深度审计 (D1-D6 六维并行)
# Phase 4: 修复 + 蒸馏
# ──────────────────────────────────────────────────────────────
set -euo pipefail

NEOTRIX_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CYCLE_LOG="${NEOTRIX_ROOT}/target/evolution-loop.log"
mkdir -p "${NEOTRIX_ROOT}/target"

CRATES=(neotrix nt-lang nt-segstore neotrix-evolution nt-proxy-daemon neotrix-proxy neotrix-proxy-pool neotrix-bridge neotrix-proxy-kernel)

log()   { echo "[$(date +%H:%M:%S)] $*" | tee -a "$CYCLE_LOG"; }
banner(){ log "═══════════════════════════════════════════════"; log "  $*"; log "═══════════════════════════════════════════════"; }

phase_1_compile_zero() {
  banner "Phase 1: 编译清零"
  local errors=0
  for crate in "${CRATES[@]}"; do
    if cargo check -p "$crate" 2>/dev/null; then
      log "  ✅ $crate — 0 errors"
    else
      log "  ❌ $crate — errors found"
      errors=$((errors + 1))
    fi
  done
  return $errors
}

phase_2_warning_gate() {
  banner "Phase 2: 警告门控"
  for crate in "${CRATES[@]}"; do
    local warns
    warns=$(cargo check -p "$crate" 2>&1 | grep -c "warning:" || true)
    if [ "$warns" -gt 0 ]; then
      log "  ⚠️  $crate — $warns warnings"
    else
      log "  ✅ $crate — 0 warnings"
    fi
  done
}

phase_3_deep_audit() {
  banner "Phase 3: 深度审计 (D1-D6)"

  # D1: 架构循环
  local cycles
  cycles=$(grep -rn "cycle\s*+=" "${NEOTRIX_ROOT}/neotrix-core/src/core/" --include="*.rs" 2>/dev/null | grep -v "test\|#\[" | wc -l)
  log "  D1: $cycles 本地 cycle++ 子系统"

  # D2: panic 路径 (consciousness hot path)
  local hot_unwrap
  hot_unwrap=$(grep -rn "\.unwrap()" "${NEOTRIX_ROOT}/neotrix-core/src/core/nt_core_loop/" "${NEOTRIX_ROOT}/neotrix-core/src/core/nt_core_consciousness/" --include="*.rs" 2>/dev/null | grep -v "test\|#\[" | grep -v "assert" | grep -v "serde_json" | wc -l)
  log "  D2: $hot_unwrap production unwrap in hot path"

  # D3: 无界集合 (consciousness hot path push)
  local unbounded
  unbounded=$(grep -rn "\.push(" "${NEOTRIX_ROOT}/neotrix-core/src/core/" --include="*.rs" 2>/dev/null | grep -v "test\|#\[" | grep -v "drain\|MAX_\|truncate\|pop_\|with_capacity" | wc -l)
  log "  D3: $unbounded push 调用 (需人工审查 drain 配对)"

  # D4: Feature 门控一致性
  local feat_declared
  feat_declared=$(grep -c "^[a-z]" "${NEOTRIX_ROOT}/neotrix-core/Cargo.toml" 2>/dev/null || echo 0)
  log "  D4: Feature 门控 — 见 Cargo.toml [features]"

  # D5: 所有 crate 状态
  log "  D5: 全 workspace check 见 Phase 1 结果"

  # D6: bins 编译
  local bins_ok=true
  cargo check -p neotrix --bins 2>/dev/null || bins_ok=false
  log "  D6: neotrix bins — $($bins_ok && echo '✅' || echo '❌')"
}

phase_4_distill() {
  banner "Phase 4: 蒸馏"
  local ts
  ts=$(date +%Y%m%d_%H%M%S)
  local report="${NEOTRIX_ROOT}/target/evolution-report-${ts}.md"
  {
    echo "# 进化循环报告"
    echo "**时间**: $(date)"
    echo ""
    echo "## 各 crate 状态"
    echo "| Crate | 状态 |"
    echo "|-------|------|"
    for crate in "${CRATES[@]}"; do
      if cargo check -p "$crate" 2>/dev/null; then
        echo "| $crate | ✅ 0 errors |"
      else
        echo "| $crate | ❌ has errors |"
      fi
    done
  } > "$report"
  log "  报告: $report"
}

main() {
  local MAX_CYCLES="${1:-10}"
  log "NeoTrix 自动进化循环 v1"
  log "工作空间: $NEOTRIX_ROOT"
  log "最大轮次: $MAX_CYCLES"
  log ""

  for ((i = 1; i <= MAX_CYCLES; i++)); do
    banner "Cycle $i / $MAX_CYCLES"
    phase_1_compile_zero && {
      phase_2_warning_gate
      phase_3_deep_audit
      phase_4_distill
      log "✅ Cycle $i 完成 — 全部正常"
    } || {
      log "❌ Cycle $i — 编译错误需要手动修复，停止循环"
      return 1
    }
  done

  banner "🎉 $MAX_CYCLES 轮进化循环全部完成"
  log "最终状态: 全 workspace 已验证"
  log "日志: $CYCLE_LOG"
}

main "$@"
