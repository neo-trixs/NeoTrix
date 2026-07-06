#!/usr/bin/env bash
# fable-absorb.sh — 批量吸收 Claude Fable 5 reasoning traces 到 NeoTrix 意识核心
#
# 用法:
#   ./fable-absorb.sh                    # 默认: 20类 × 5条 = 100 traces
#   ./fable-absorb.sh 10                 # 每类10条
#   ./fable-absorb.sh 5 "debugging,code_generation"   # 仅指定类别
#   ./fable-absorb.sh --distill-only     # 仅蒸馏(跳过吸收)
#
# 前置条件: HF_API_TOKEN=<your_token>  (或 ~/.config/neotrix/config.toml 中配置)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="${PROJECT_DIR}/.cleanup/logs/fable_absorb_${TIMESTAMP}.log"
mkdir -p "$(dirname "${LOG_FILE}")"

# ── 配置 ──
NEOTRIX_BIN="${NEOTRIX_BIN:-${PROJECT_DIR}/target/release/neotrix}"
if [ ! -x "${NEOTRIX_BIN}" ]; then
    echo "⚠️  未找到 neotrix 二进制, 尝试 cargo run..."
    NEOTRIX_CMD="cargo run --manifest-path ${PROJECT_DIR}/Cargo.toml -p neotrix -- "
else
    NEOTRIX_CMD="${NEOTRIX_BIN}"
fi

# ── 参数 ──
PER_CATEGORY="${1:-5}"
if [ "$1" = "--distill-only" ]; then
    DISTILL_ONLY=true
    PER_CATEGORY=0
    shift
fi

CATEGORIES="${2:-debugging,code_generation,architecture_design,data_analysis,algorithm_design,system_design,security_audit,performance_optimization,refactoring,testing,api_design,database_design,networking,concurrency,mathematical_proof,scientific_computation,machine_learning,natural_language,cryptography,documentation}"

echo "══════════════════════════════════════════════════"
echo "  NeoTrix Fable 5 Traces Absorber"
echo "  Model: AliesTaha/fable-traces (Qwen3-4B)"
echo "  Traces: ~2.3M Claude Fable 5 reasoning traces"
echo "══════════════════════════════════════════════════"
echo ""
echo "  Start:     $(date)"
echo "  Categories: ${PER_CATEGORY}/cat × $(echo ${CATEGORIES} | tr ',' '\n' | wc -l | tr -d ' ') categories"
if [ -n "$DISTILL_ONLY" ]; then
    echo "  Mode:      Distill only (skip acquisition)"
else
    echo "  Total:     ~$(( PER_CATEGORY * $(echo ${CATEGORIES} | tr ',' '\n' | wc -l | tr -d ' ') )) traces"
fi
echo "  Log:       ${LOG_FILE}"
echo ""

# ── Phase 1: 吸收循环 ──
if [ -z "$DISTILL_ONLY" ] && [ "${PER_CATEGORY}" -gt 0 ]; then
    echo "▶ Phase 1: Fable trace acquisition cycle..."
    echo "   Categories: ${CATEGORIES}"
    echo "   Per cat:    ${PER_CATEGORY}"

    if ! ${NEOTRIX_CMD} /fable cycle "${CATEGORIES}" "${PER_CATEGORY}" 2>&1 | tee -a "${LOG_FILE}"; then
        echo "⚠️  Cycle reported issues (see log)"
    fi

    echo ""
    echo "✅ Phase 1 complete: $(date)"
    echo ""

    # 写入吸收记录
    cat >> "${LOG_FILE}" <<EOF

=== ABSORPTION CYCLE ${TIMESTAMP} ===
Categories: ${CATEGORIES}
Per category: ${PER_CATEGORY}
Status: completed
EOF
else
    echo "⏩ Phase 1: Skipped (distill-only mode)"
fi

# ── Phase 2: 蒸馏 + 模式检测 ──
echo "▶ Phase 2: Trace distillation & pattern detection..."
if ! ${NEOTRIX_CMD} /fable distill 2>&1 | tee -a "${LOG_FILE}"; then
    echo "⚠️  Distill reported issues (see log)"
fi
echo ""

# ── Phase 3: 全景更新 ──
echo "▶ Phase 3: Knowledge panorama update..."
if ! ${NEOTRIX_CMD} /explore panorama 2>&1 | tee -a "${LOG_FILE}"; then
    echo "⚠️  Panorama update reported issues (see log)"
fi
echo ""

# ── Phase 4: 状态报告 ──
echo "▶ Phase 4: Final status..."
${NEOTRIX_CMD} /fable status 2>&1 | tee -a "${LOG_FILE}"
echo ""

echo "══════════════════════════════════════════════════"
echo "  Fable Absorb Complete: $(date)"
echo "  Log: ${LOG_FILE}"
echo "══════════════════════════════════════════════════"
echo ""
echo "KB nodes:       /explore panorama"
echo "Traces status:  /fable status"
echo "Parse a trace:  /fable parse <text>"
echo "Quality check:  /fable quality <text>"
echo ""
echo "Set HF_API_TOKEN to query the Fable model API."
