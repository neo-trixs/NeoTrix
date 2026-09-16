#!/usr/bin/env bash
# NeoTrix Code Quality Check Script
# Runs cargo clippy, parses warnings/suggestions, generates JSON report
set -euo pipefail

REPORT_DIR="${REPORT_DIR:-$(dirname "$0")/../.reports}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
REPORT_FILE="${REPORT_DIR}/quality_report.json"
EXIT_CODE=0

mkdir -p "$REPORT_DIR"

echo "=== NeoTrix Code Quality Check ==="
echo "Timestamp: ${TIMESTAMP}"
echo ""

# Run cargo clippy
CLIPPY_OUTPUT=$(cargo clippy -p neotrix --all-targets 2>&1) || true
CLIPPY_EXIT=$?

# Count warnings and suggestions
WARN_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^warning:" || true)
NOTE_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^note:" || true)
HELP_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^help:" || true)

# Extract warning details with file locations
WARNINGS_JSON="[]"
if [ "$WARN_COUNT" -gt 0 ]; then
    WARNINGS_JSON="["
    FIRST=true
    while IFS= read -r line; do
        LOC=$(echo "$line" | grep -oE '[a-zA-Z0-9_/.-]+\.[a-z]+:[0-9]+:[0-9]+' | head -1 || true)
        MSG=$(echo "$line" | sed 's/^warning: //' | sed 's/"/\\"/g' || true)
        if [ -n "$LOC" ] || [ -n "$MSG" ]; then
            [ "$FIRST" = true ] && FIRST=false || WARNINGS_JSON="${WARNINGS_JSON},"
            WARNINGS_JSON="${WARNINGS_JSON}{\"location\":\"${LOC:-unknown}\",\"message\":\"${MSG}\"}"
        fi
    done <<< "$(echo "$CLIPPY_OUTPUT" | grep "^warning:" || true)"
    WARNINGS_JSON="${WARNINGS_JSON}]"
fi

# Count warnings by lint category (e.g., clippy::style, clippy::perf)
LINT_CATEGORIES_JSON="{}"
if [ "$WARN_COUNT" -gt 0 ]; then
    LINT_CATEGORIES="{"
    FIRST=true
    # Extract lint names like "clippy::style", "unused_variables", etc.
    while IFS= read -r lint; do
        COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:.*\b${lint}\b" || true)
        if [ "$COUNT" -gt 0 ]; then
            [ "$FIRST" = true ] && FIRST=false || LINT_CATEGORIES="${LINT_CATEGORIES},"
            LINT_CATEGORIES="${LINT_CATEGORIES}\"${lint}\":${COUNT}"
        fi
    done <<< "$(echo "$CLIPPY_OUTPUT" | grep -oE 'clippy::[a-z_]+|unused_[a-z_]+|dead_code|unused_imports' | sort -u || true)"
    CLOSE_BRACE='}'
    LINT_CATEGORIES_JSON="${LINT_CATEGORIES}${CLOSE_BRACE}"
fi

# Determine exit code: warnings → 1
if [ "$WARN_COUNT" -gt 0 ]; then
    EXIT_CODE=1
fi

# Build JSON report
cat > "$REPORT_FILE" <<ENDJSON
{
  "script": "check_quality",
  "timestamp": "${TIMESTAMP}",
  "exit_code": ${EXIT_CODE},
  "clippy_exit": ${CLIPPY_EXIT},
  "warnings": ${WARN_COUNT},
  "notes": ${NOTE_COUNT},
  "helps": ${HELP_COUNT},
  "lint_categories": ${LINT_CATEGORIES_JSON},
  "warning_details": ${WARNINGS_JSON}
}
ENDJSON

echo ""
echo "=== Results ==="
echo "Warnings: ${WARN_COUNT}"
echo "Notes:    ${NOTE_COUNT}"
echo "Helps:    ${HELP_COUNT}"
echo "Report:   ${REPORT_FILE}"

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "Status:   PASS"
else
    echo "Status:   FAIL"
fi

exit $EXIT_CODE
