#!/usr/bin/env bash
# NeoTrix Test Coverage Check Script
# Runs cargo test, parses results, generates JSON report
set -euo pipefail

REPORT_DIR="${REPORT_DIR:-$(dirname "$0")/../.reports}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
REPORT_FILE="${REPORT_DIR}/tests_report.json"
EXIT_CODE=0

mkdir -p "$REPORT_DIR"

echo "=== NeoTrix Test Coverage Check ==="
echo "Timestamp: ${TIMESTAMP}"
echo ""

# Run cargo test, capture output
TEST_OUTPUT=$(cargo test -p neotrix --lib 2>&1) || true
TEST_EXIT=$?

# Parse test results from cargo output
# cargo test prints lines like: "test module::test_name ... ok" / "FAILED" / "ignored"
PASSED=$(echo "$TEST_OUTPUT" | grep -cE '\.\.\. ok$' || true)
FAILED=$(echo "$TEST_OUTPUT" | grep -cE '\.\.\. FAILED$' || true)
IGNORED=$(echo "$TEST_OUTPUT" | grep -cE '\.\.\. ignored$' || true)
TOTAL=$((PASSED + FAILED + IGNORED))

# Extract failed test names
FAILED_TESTS_JSON="[]"
if [ "$FAILED" -gt 0 ]; then
    FAILED_TESTS_JSON="["
    FIRST=true
    while IFS= read -r line; do
        TEST_NAME=$(echo "$line" | sed 's/ *$//' | sed 's/^ *//' | sed 's/ \.\.\. FAILED$//' || true)
        if [ -n "$TEST_NAME" ]; then
            [ "$FIRST" = true ] && FIRST=false || FAILED_TESTS_JSON="${FAILED_TESTS_JSON},"
            FAILED_TESTS_JSON="${FAILED_TESTS_JSON}\"$(echo "$TEST_NAME" | sed 's/"/\\"/g')\""
        fi
    done <<< "$(echo "$TEST_OUTPUT" | grep '\.\.\. FAILED' || true)"
    FAILED_TESTS_JSON="${FAILED_TESTS_JSON}]"
fi

# Extract ignored test names
IGNORED_TESTS_JSON="[]"
if [ "$IGNORED" -gt 0 ]; then
    IGNORED_TESTS_JSON="["
    FIRST=true
    while IFS= read -r line; do
        TEST_NAME=$(echo "$line" | sed 's/ *$//' | sed 's/^ *//' | sed 's/ \.\.\. ignored$//' || true)
        if [ -n "$TEST_NAME" ]; then
            [ "$FIRST" = true ] && FIRST=false || IGNORED_TESTS_JSON="${IGNORED_TESTS_JSON},"
            IGNORED_TESTS_JSON="${IGNORED_TESTS_JSON}\"$(echo "$TEST_NAME" | sed 's/"/\\"/g')\""
        fi
    done <<< "$(echo "$TEST_OUTPUT" | grep '\.\.\. ignored' || true)"
    IGNORED_TESTS_JSON="${IGNORED_TESTS_JSON}]"
fi

# Extract test execution time if present
EXEC_TIME=$(echo "$TEST_OUTPUT" | grep -oE 'finished in [0-9]+\.[0-9]+s' | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
TOTAL_TIME=$(echo "$TEST_OUTPUT" | grep -oE 'test result: [a-z]+\. [0-9]+ passed' | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' || echo "$PASSED")

# Calculate coverage (passed / total)
if [ "$TOTAL" -gt 0 ]; then
    COVERAGE=$(awk "BEGIN {printf \"%.1f\", ($PASSED / $TOTAL) * 100}")
else
    COVERAGE="0.0"
fi

# Determine exit code
if [ "$FAILED" -gt 0 ] || [ "$TEST_EXIT" -ne 0 ]; then
    EXIT_CODE=1
fi

# Build JSON report
cat > "$REPORT_FILE" <<ENDJSON
{
  "script": "check_tests",
  "timestamp": "${TIMESTAMP}",
  "exit_code": ${EXIT_CODE},
  "cargo_exit": ${TEST_EXIT},
  "total": ${TOTAL},
  "passed": ${PASSED},
  "failed": ${FAILED},
  "ignored": ${IGNORED},
  "coverage_percent": ${COVERAGE},
  "execution_time_seconds": ${EXEC_TIME},
  "failed_tests": ${FAILED_TESTS_JSON},
  "ignored_tests": ${IGNORED_TESTS_JSON}
}
ENDJSON

echo ""
echo "=== Results ==="
echo "Total:    ${TOTAL}"
echo "Passed:   ${PASSED}"
echo "Failed:   ${FAILED}"
echo "Ignored:  ${IGNORED}"
echo "Coverage: ${COVERAGE}%"
echo "Time:     ${EXEC_TIME}s"
echo "Report:   ${REPORT_FILE}"

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "Status:   PASS"
else
    echo "Status:   FAIL"
fi

exit $EXIT_CODE
