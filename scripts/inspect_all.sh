#!/usr/bin/env bash
# NeoTrix Master Inspection Script
# Runs all check scripts in sequence, generates combined JSON report
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPORT_DIR="${REPORT_DIR:-${SCRIPT_DIR}/../.reports}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
COMBINED_REPORT="${REPORT_DIR}/inspection_report.json"
EXIT_CODE=0

mkdir -p "$REPORT_DIR"

echo "╔══════════════════════════════════════════════════════╗"
echo "║         NeoTrix Multi-Agent Inspection              ║"
echo "╚══════════════════════════════════════════════════════╝"
echo "Timestamp: ${TIMESTAMP}"
echo ""

# Track overall results
SCRIPTS=("check_compilation" "check_tests" "check_quality" "check_dependencies")
SCRIPT_LABELS=("Compilation" "Tests" "Code Quality" "Dependencies")
SCRIPT_FILES=("check_compilation.sh" "check_tests.sh" "check_quality.sh" "check_dependencies.sh")

RESULTS_JSON="{"
FIRST=true
OVERALL_PASS=true
TOTAL_ERRORS=0

for i in "${!SCRIPTS[@]}"; do
    SCRIPT_NAME="${SCRIPTS[$i]}"
    SCRIPT_LABEL="${SCRIPT_LABELS[$i]}"
    SCRIPT_FILE="${SCRIPT_FILES[$i]}"
    REPORT_FILE="${REPORT_DIR}/${SCRIPT_NAME}_report.json"
    SCRIPT_PATH="${SCRIPT_DIR}/${SCRIPT_FILE}"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Running: ${SCRIPT_LABEL} (${SCRIPT_FILE})"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if [ ! -x "$SCRIPT_PATH" ]; then
        echo "  ⚠ Script not executable, attempting chmod +x"
        chmod +x "$SCRIPT_PATH" 2>/dev/null || true
    fi

    # Run the script, capture exit code
    if bash "$SCRIPT_PATH" 2>&1; then
        SCRIPT_EXIT=0
        echo "  ✓ ${SCRIPT_LABEL}: PASS"
    else
        SCRIPT_EXIT=$?
        echo "  ✗ ${SCRIPT_LABEL}: FAIL (exit ${SCRIPT_EXIT})"
        OVERALL_PASS=false
    fi

    # Count errors from this script's report
    if [ -f "$REPORT_FILE" ]; then
        SCRIPT_ERRORS=$(grep -o '"errors": *[0-9]*' "$REPORT_FILE" | grep -oE '[0-9]+' || echo "0")
        SCRIPT_WARNINGS=$(grep -o '"warnings": *[0-9]*' "$REPORT_FILE" | grep -oE '[0-9]+' || echo "0")
        TOTAL_ERRORS=$((TOTAL_ERRORS + SCRIPT_ERRORS + SCRIPT_WARNINGS))
    fi

    # Add to combined JSON
    [ "$FIRST" = true ] && FIRST=false || RESULTS_JSON="${RESULTS_JSON},"
    RESULTS_JSON="${RESULTS_JSON}\"${SCRIPT_NAME}\":{\"exit_code\":${SCRIPT_EXIT},\"report\":\"${REPORT_FILE}\"}"

    echo ""
done

# Close combined JSON
if [ "$OVERALL_PASS" = true ]; then
    OVERALL_EXIT=0
    OVERALL_STATUS="PASS"
else
    OVERALL_EXIT=1
    OVERALL_STATUS="FAIL"
fi

RESULTS_JSON="${RESULTS_JSON}}"

# Generate final combined report
cat > "$COMBINED_REPORT" <<ENDJSON
{
  "script": "inspect_all",
  "timestamp": "${TIMESTAMP}",
  "overall_exit": ${OVERALL_EXIT},
  "overall_status": "${OVERALL_STATUS}",
  "total_issues": ${TOTAL_ERRORS},
  "scripts": ${RESULTS_JSON}
}
ENDJSON

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║               INSPECTION SUMMARY                   ║"
echo "╠══════════════════════════════════════════════════════╣"
echo "║ Compilation:  $(grep -o '"exit_code": *[0-9]*' "${REPORT_DIR}/compilation_report.json" 2>/dev/null | grep -oE '[0-9]' || echo "?") │"
echo "║ Tests:        $(grep -o '"exit_code": *[0-9]*' "${REPORT_DIR}/tests_report.json" 2>/dev/null | grep -oE '[0-9]' || echo "?") │"
echo "║ Quality:      $(grep -o '"exit_code": *[0-9]*' "${REPORT_DIR}/quality_report.json" 2>/dev/null | grep -oE '[0-9]' || echo "?") │"
echo "║ Dependencies: $(grep -o '"exit_code": *[0-9]*' "${REPORT_DIR}/dependencies_report.json" 2>/dev/null | grep -oE '[0-9]' || echo "?") │"
echo "╠══════════════════════════════════════════════════════╣"
echo "║ Overall:      ${OVERALL_STATUS}"
echo "║ Total Issues: ${TOTAL_ERRORS}"
echo "║ Report:       ${COMBINED_REPORT}"
echo "╚══════════════════════════════════════════════════════╝"

exit $OVERALL_EXIT
