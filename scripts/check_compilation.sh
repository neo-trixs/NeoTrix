#!/usr/bin/env bash
# NeoTrix Compilation Check Script
# Runs cargo check, parses errors/warnings, generates JSON report
set -euo pipefail

REPORT_DIR="${REPORT_DIR:-$(dirname "$0")/../.reports}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
REPORT_FILE="${REPORT_DIR}/compilation_report.json"
EXIT_CODE=0

mkdir -p "$REPORT_DIR"

echo "=== NeoTrix Compilation Check ==="
echo "Timestamp: ${TIMESTAMP}"
echo ""

# Run cargo check, capture both stdout and stderr
CHECK_OUTPUT=$(cargo check --all-targets -p neotrix 2>&1) || true
CHECK_EXIT=$?

# Count errors and warnings
ERROR_COUNT=$(echo "$CHECK_OUTPUT" | grep -c "^error" || true)
WARN_COUNT=$(echo "$CHECK_OUTPUT" | grep -c "^warning" || true)

# Extract error locations (file:line:col patterns after "error")
ERRORS_JSON="[]"
if [ "$ERROR_COUNT" -gt 0 ]; then
    ERRORS_JSON="["
    FIRST=true
    while IFS= read -r line; do
        LOC=$(echo "$line" | grep -oE '[a-zA-Z0-9_/.-]+\.[a-z]+:[0-9]+:[0-9]+' | head -1 || true)
        MSG=$(echo "$line" | sed 's/^error\[:.*\]: //' | sed 's/"/\\"/g' || true)
        if [ -n "$LOC" ]; then
            [ "$FIRST" = true ] && FIRST=false || ERRORS_JSON="${ERRORS_JSON},"
            ERRORS_JSON="${ERRORS_JSON}{\"location\":\"${LOC}\",\"message\":\"${MSG}\"}"
        fi
    done <<< "$(echo "$CHECK_OUTPUT" | grep "^error" || true)"
    ERRORS_JSON="${ERRORS_JSON}]"
fi

# Extract warning locations
WARNINGS_JSON="[]"
if [ "$WARN_COUNT" -gt 0 ]; then
    WARNINGS_JSON="["
    FIRST=true
    while IFS= read -r line; do
        LOC=$(echo "$line" | grep -oE '[a-zA-Z0-9_/.-]+\.[a-z]+:[0-9]+:[0-9]+' | head -1 || true)
        MSG=$(echo "$line" | sed 's/^warning\[:.*\]: //' | sed 's/"/\\"/g' || true)
        if [ -n "$LOC" ]; then
            [ "$FIRST" = true ] && FIRST=false || WARNINGS_JSON="${WARNINGS_JSON},"
            WARNINGS_JSON="${WARNINGS_JSON}{\"location\":\"${LOC}\",\"message\":\"${MSG}\"}"
        fi
    done <<< "$(echo "$CHECK_OUTPUT" | grep "^warning" || true)"
    WARNINGS_JSON="${WARNINGS_JSON}]"
fi

# Determine exit code: errors → 1, warnings-only → 0
if [ "$ERROR_COUNT" -gt 0 ]; then
    EXIT_CODE=1
fi

# Build JSON report
cat > "$REPORT_FILE" <<ENDJSON
{
  "script": "check_compilation",
  "timestamp": "${TIMESTAMP}",
  "exit_code": ${EXIT_CODE},
  "cargo_exit": ${CHECK_EXIT},
  "errors": ${ERROR_COUNT},
  "warnings": ${WARN_COUNT},
  "error_details": ${ERRORS_JSON},
  "warning_details": ${WARNINGS_JSON}
}
ENDJSON

echo ""
echo "=== Results ==="
echo "Errors:   ${ERROR_COUNT}"
echo "Warnings: ${WARN_COUNT}"
echo "Report:   ${REPORT_FILE}"

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "Status:   PASS"
else
    echo "Status:   FAIL"
fi

exit $EXIT_CODE
