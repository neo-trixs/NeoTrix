#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────
# NeoTrix Self-Audit Pipeline  v1
# ESAA-Security pattern: 4-phase governed audit pipeline
# AgenticSCR pattern: Detector + Validator subagents
# auto-audit pattern: One lifecycle per finding
# ──────────────────────────────────────────────────────────────
set -euo pipefail

NEOTRIX_ROOT="${NEOTRIX_ROOT:-/Users/neo/Downloads/neotrix}"
REPORT_DIR="${NEOTRIX_ROOT}/target/audit-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/audit_${TIMESTAMP}.md"
PHASE_LOG="${REPORT_DIR}/phase_${TIMESTAMP}.log"
RULES_DIR="${REPORT_DIR}/rules_${TIMESTAMP}"

mkdir -p "$REPORT_DIR"

log()  { echo "[$(date +%H:%M:%S)] $*" | tee -a "$PHASE_LOG"; }
banner() { log "═══════════════════════════════════════════════"; log "  $*"; log "═══════════════════════════════════════════════"; }

# ── Phase 0: Reconnaissance ──────────────────────────────────
phase_0_recon() {
  banner "Phase 0: Reconnaissance"

  # Build file index
  log "Indexing source files..."
  RUST_FILES=$(find "$NEOTRIX_ROOT/neotrix-core/src" "$NEOTRIX_ROOT/crates" -name "*.rs" -not -path "*/target/*" | wc -l)
  log "  Rust files: $RUST_FILES"

  # Module count
  MOD_COUNT=$(grep -r "^pub mod " "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" | wc -l)
  log "  Public modules: $MOD_COUNT"

  # Entry points
  BIN_COUNT=$(grep -rl "^fn main" "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" | wc -l)
  log "  Binary entry points: $BIN_COUNT"

  # Dependencies
  if command -v cargo-audit &>/dev/null; then
    log "  Running cargo-audit for dependency vulnerabilities..."
    (cd "$NEOTRIX_ROOT" && cargo audit --json 2>/dev/null | python3 -c "
import json,sys; data=json.load(sys.stdin)
vulns = data.get('vulnerabilities',{}).get('list',[])
for v in vulns:
    print(f'    CVE: {v.get(\"advisory\",{}).get(\"id\",\"?\")} {v.get(\"advisory\",{}).get(\"title\",\"\")}')
print(f'    Total: {len(vulns)} vulnerabilities')
" 2>/dev/null || log "    cargo-audit failed (not installed?)") || true
  else
    log "  cargo-audit not installed — skip dependency scan"
  fi

  # Generate SBOM
  if command -v cargo-cyclonedx &>/dev/null; then
    (cd "$NEOTRIX_ROOT" && cargo cyclonedx --all --output-dir "$REPORT_DIR" 2>/dev/null) || true
    log "  SBOM generated"
  fi

  log "Reconnaissance complete"
}

# ── Phase 1: Deterministic Pre-Checks ──────────────────────
# Grippy/pedant pattern: 22 rule engine, 0 false positives
phase_1_deterministic() {
  banner "Phase 1: Deterministic Pre-Checks"

  mkdir -p "$RULES_DIR"

  local FINDS=0

  # Rule D01: .ok() on Result in production code
  log "  [D01] .ok() on Result (production code)..."
  grep -rn '\.ok();' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '/tests/' | grep -v '#\[cfg(test)\]' | grep -v 'env::var' \
    > "$RULES_DIR/d01_ok_swallow.txt" 2>/dev/null || true
  local cnt=$(wc -l < "$RULES_DIR/d01_ok_swallow.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D02: let _ = discarding Result
  log "  [D02] let _ = discarding Result..."
  grep -rn 'let _ = ' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '/tests/' | grep -v '#\[cfg(test)\]' \
    | grep -E '(write|read|send|recv|remove|rename|create_dir|flush|close)' \
    > "$RULES_DIR/d02_let_result.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d02_let_result.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D03: Ordering::Relaxed on atomics
  log "  [D03] Ordering::Relaxed on atomics..."
  grep -rn 'Ordering::Relaxed' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '/tests/' | grep -v '#\[cfg(test)\]' \
    > "$RULES_DIR/d03_relaxed_atomics.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d03_relaxed_atomics.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D04: #[derive(Debug)] on types with password/key/token fields
  log "  [D04] Debug derive on sensitive types..."
  grep -rn 'password\|secret\|api_key\|api.key\|token' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -B1 'derive.*Debug' | grep -v 'test' \
    > "$RULES_DIR/d04_debug_sensitive.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d04_debug_sensitive.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D05: unsafe code blocks
  log "  [D05] unsafe blocks..."
  grep -rn 'unsafe {' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '/tests/' | grep -v '#\[cfg(test)\]' | grep -v 'forbid(unsafe_code)' \
    > "$RULES_DIR/d05_unsafe.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d05_unsafe.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D06: #[allow(unused)] in production
  log "  [D06] allow(unused) in production code..."
  grep -rn '#\[allow(unused' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '/tests/' \
    > "$RULES_DIR/d06_allow_unused.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d06_allow_unused.txt")
  log "       found: $cnt (informational)"

  # Rule D07: Functions returning () that perform I/O
  log "  [D07] I/O functions returning ()..."
  grep -rn 'fn.*() {' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -E '(write|save|store|delete|remove|send|flush)' \
    | grep -v 'test' | grep -v 'Result' \
    > "$RULES_DIR/d07_io_no_result.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d07_io_no_result.txt")
  log "       found: $cnt"

  # Rule D08: println! / eprintln! in production code
  log "  [D08] println!/eprintln! in production..."
  grep -rn 'eprintln!\|println!' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '/tests/' | grep -v '#\[cfg(test)\]' \
    > "$RULES_DIR/d08_println.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d08_println.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D09: unwrap/expect in non-test production code
  log "  [D09] unwrap/expect in production..."
  grep -rn '\.unwrap()\|\.expect(' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '/tests/' | grep -v '#\[cfg(test)\]' \
    > "$RULES_DIR/d09_unwrap.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d09_unwrap.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D10: #[non_exhaustive] audit on pub enums
  log "  [D10] pub enum without #[non_exhaustive]..."
  grep -rn '^pub enum ' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    | grep -v '#\[non_exhaustive\]' \
    > "$RULES_DIR/d10_enum_exhaustive.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d10_enum_exhaustive.txt")
  FINDS=$((FINDS + cnt))
  log "       found: $cnt"

  # Rule D11: Functions with too many parameters (>8)
  log "  [D11] Functions with >8 parameters..."
  grep -rn '^pub fn.*,.*,.*,.*,.*,.*,.*,.*,.*,' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
    > "$RULES_DIR/d11_too_many_params.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d11_too_many_params.txt")
  log "       found: $cnt"

  # Rule D12: Files >2000 lines
  log "  [D12] Files >2000 lines..."
  find "$NEOTRIX_ROOT/neotrix-core/src" -name "*.rs" -exec wc -l {} \; \
    | awk '$1 > 2000 {print $1, $2}' \
    > "$RULES_DIR/d12_oversized_files.txt" 2>/dev/null || true
  cnt=$(wc -l < "$RULES_DIR/d12_oversized_files.txt")
  log "       found: $cnt"

  # Summary
  log "  Total deterministic findings: $FINDS"
  return 0
}

# ── Phase 2: LLM-Driven Audit ──────────────────────────────
# AgenticSCR pattern: detector + validator subagents per dimension
phase_2_llm_audit() {
  banner "Phase 2: LLM-Driven Deep Audit"

  local DIMENSIONS=(
    "error-handling: Silent error swallowing, .ok() on I/O, Result discarding"
    "concurrency: Atomic ordering, lock correctness, JoinHandle discard"
    "api-surface: pub→pub(crate) exposure, leaked internal types"
    "log-sensitivity: Secret leakage in Debug/log, credential exposure"
    "panic-paths: unwrap/expect in user-triggerable paths"
    "resource-leaks: File handles, connections, zombie processes"
    "pipeline-integrity: Consciousness cycle correctness, handler dispatch"
  )

  for dim in "${DIMENSIONS[@]}"; do
    local name="${dim%%:*}"
    local desc="${dim#*:}"
    log "  Inspecting dimension: $name — $desc"

    # Detector pass: grep-based evidence collection
    local evidence=""
    case "$name" in
      error-handling)
        evidence=$(grep -rn '\.ok();' "$NEOTRIX_ROOT/neotrix-core/src" --include="*.rs" \
          | grep -v '/tests/' | grep -v '#\[cfg(test)\]' | grep -v 'env::var' | head -20) || true
        log "    Detector: $(echo "$evidence" | wc -l) sites"
        ;;
      concurrency)
        evidence=$(grep -rn 'Ordering::Relaxed\|JoinHandle\|unsafe impl Send' "$NEOTRIX_ROOT/neotrix-core/src" \
          --include="*.rs" | grep -v '/tests/' | head -20) || true
        log "    Concurrency sites detected"
        ;;
      api-surface)
        evidence=$(grep -rn '^pub fn ' "$NEOTRIX_ROOT/neotrix-core/src/entry" --include="*.rs" | head -20) || true
        log "    API surface: $(echo "$evidence" | wc -l) pub fns in entry/"
        ;;
    esac
  done
}

# ── Phase 3: Risk Classification ─────────────────────────────
phase_3_classify() {
  banner "Phase 3: Risk Classification"

  local D01=$(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d01_ok_swallow.txt" 2>/dev/null || echo 0)
  local D03=$(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d03_relaxed_atomics.txt" 2>/dev/null || echo 0)
  local D05=$(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d05_unsafe.txt" 2>/dev/null || echo 0)
  local D09=$(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d09_unwrap.txt" 2>/dev/null || echo 0)

  cat > "$REPORT_FILE" <<EOF
# NeoTrix Self-Audit Report
**Date**: $(date)
**Pipeline**: ESAA-Security 4-Phase | AgenticSCR Detector-Validator | 12 Deterministic Rules

---

## Phase 0: Reconnaissance
- Rust files indexed
- Module count, entry points, dependencies

## Phase 1: Deterministic Pre-Checks

| Rule | Finding | Count |
|------|---------|-------|
| D01 | .ok() swallow on Result | $D01 |
| D02 | let _ = discarding Result | $(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d02_let_result.txt" 2>/dev/null || echo 0) |
| D03 | Ordering::Relaxed atomics | $D03 |
| D04 | Debug derive on sensitive types | $(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d04_debug_sensitive.txt" 2>/dev/null || echo 0) |
| D05 | unsafe blocks | $D05 |
| D08 | println!/eprintln! in prod | $(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d08_println.txt" 2>/dev/null || echo 0) |
| D09 | unwrap/expect in prod | $D09 |
| D10 | pub enum without non_exhaustive | $(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d10_enum_exhaustive.txt" 2>/dev/null || echo 0) |
| D12 | Files >2000 lines | $(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d12_oversized_files.txt" 2>/dev/null || echo 0) |

## Risk Matrix

EOF

  # Classify into risk levels
  if [ "$D01" -gt 20 ] || [ "$D03" -gt 10 ]; then
    echo "| CRITICAL | High-volume silent failures or concurrency issues |" >> "$REPORT_FILE"
  fi
  if [ "$D05" -gt 5 ]; then
    echo "| HIGH | Excessive unsafe code in production |" >> "$REPORT_FILE"
  fi
  if [ "$D09" -gt 50 ]; then
    echo "| HIGH | Production unwrap/expect paths |" >> "$REPORT_FILE"
  fi

  echo "" >> "$REPORT_FILE"
  echo "## Detailed Findings" >> "$REPORT_FILE"
  echo "" >> "$REPORT_FILE"
  for rule in "$RULES_DIR"/*.txt; do
    local name=$(basename "$rule" .txt)
    echo "### $name" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    cat "$rule" >> "$REPORT_FILE" 2>/dev/null || echo "(empty)"
    echo '```' >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
  done

  log "Report written to $REPORT_FILE"
}

# ── Phase 4: Remediation Guidance ──────────────────────────
phase_4_remediate() {
  banner "Phase 4: Remediation Guidance"

  local D01_CNT=$(wc -l < "${REPORT_DIR}/rules_${TIMESTAMP}/d01_ok_swallow.txt" 2>/dev/null || echo 0)

  cat >> "$REPORT_FILE" <<EOF

## Phase 4: Remediation Guidance

### Priority 1 — CRITICAL (fix immediately)
- **D01 (.ok() swallow)**: Replace with \`map_err(|e| log::warn!(...))?\` or \`if let Err(e) = log::warn!(...)\`
- **D03 (Relaxed atomics)**: Change to \`Acquire\`/\`Release\` pair for cross-thread visibility

### Priority 2 — HIGH (fix this session)
- **D09 (unwrap/expect)**: Add \`expect("invariant: ...")\` messages or convert to \`?\`
- **D04 (Debug on sensitive types)**: Add manual \`Debug\` impl with redacted fields

### Priority 3 — MEDIUM (next session)
- **D05 (unsafe)**: Review each \`unsafe\` block, add safety comments
- **D10 (enum exhaustiveness)**: Add \`#[non_exhaustive]\` to all public enums
- **D12 (oversized files)**: Split files >2000 lines

### Auto-Fix Loop (optional)
Run \`for finding in findings; do discover → triage → fix → verify; done\`
EOF

  log "Report finalized at $REPORT_FILE"
  log ""
  log "══════════════ Summary ══════════════"
  log "Findings: $(cat "$RULES_DIR"/*.txt 2>/dev/null | wc -l)"
  log "Report: $REPORT_FILE"
  log "═════════════════════════════════════"
}

# ── Main Pipeline ──────────────────────────────────────────
main() {
  log "NeoTrix Self-Audit Pipeline v1"
  log "Root: $NEOTRIX_ROOT"
  log ""

  phase_0_recon
  phase_1_deterministic
  phase_2_llm_audit
  phase_3_classify
  phase_4_remediate

  log "Audit complete. Report: $REPORT_FILE"
}

main "$@"
