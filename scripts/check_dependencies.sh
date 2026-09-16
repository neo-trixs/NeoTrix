#!/usr/bin/env bash
# NeoTrix Dependency Audit Check Script
# Runs cargo audit (if available), checks for vulnerabilities, generates JSON report
set -euo pipefail

REPORT_DIR="${REPORT_DIR:-$(dirname "$0")/../.reports}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
REPORT_FILE="${REPORT_DIR}/dependencies_report.json"
EXIT_CODE=0
AUDIT_AVAILABLE=true

mkdir -p "$REPORT_DIR"

echo "=== NeoTrix Dependency Audit Check ==="
echo "Timestamp: ${TIMESTAMP}"
echo ""

# Check if cargo-audit is installed
if ! command -v cargo-audit &>/dev/null && ! cargo audit --version &>/dev/null 2>&1; then
    echo "WARNING: cargo-audit not installed. Attempting install..."
    if cargo install cargo-audit 2>/dev/null; then
        echo "cargo-audit installed successfully."
    else
        echo "ERROR: Failed to install cargo-audit. Skipping audit."
        AUDIT_AVAILABLE=false
    fi
fi

AUDIT_OUTPUT=""
AUDIT_EXIT=0
VULN_COUNT=0
UNAUDITED_COUNT=0
VULNS_JSON="[]"

if [ "$AUDIT_AVAILABLE" = true ]; then
    # Run cargo audit
    AUDIT_OUTPUT=$(cargo audit 2>&1) || true
    AUDIT_EXIT=$?

    # Parse vulnerability count
    VULN_COUNT=$(echo "$AUDIT_OUTPUT" | grep -c "vulnerability" || true)
    # Also check for the specific "Vulnerabilities found" line
    if echo "$AUDIT_OUTPUT" | grep -q "Vulnerabilities found"; then
        VULN_COUNT=$(echo "$AUDIT_OUTPUT" | grep "Vulnerabilities found" | grep -oE '[0-9]+' | head -1 || echo "0")
    fi

    # Parse advisory details
    VULNS_JSON="["
    FIRST=true
    # cargo audit outputs advisories with lines like "Crate: ...", "Version: ...", "Title: ..."
    IN_ADVISORY=false
    ADVISORY_CRATE=""
    ADVISORY_VERSION=""
    ADVISORY_TITLE=""
    ADVISORY_URL=""
    ADVISORY_RUSTSEC=""

    while IFS= read -r line; do
        if echo "$line" | grep -q "^Crate:"; then
            # Save previous advisory if any
            if [ -n "$ADVISORY_CRATE" ]; then
                [ "$FIRST" = true ] && FIRST=false || VULNS_JSON="${VULNS_JSON},"
                VULNS_JSON="${VULNS_JSON}{\"crate\":\"${ADVISORY_CRATE}\",\"version\":\"${ADVISORY_VERSION}\",\"title\":\"${ADVISORY_TITLE}\",\"url\":\"${ADVISORY_URL}\",\"advisory\":\"${ADVISORY_RUSTSEC}\"}"
            fi
            ADVISORY_CRATE=$(echo "$line" | sed 's/^Crate: //' | sed 's/"/\\"/g')
            ADVISORY_VERSION=""
            ADVISORY_TITLE=""
            ADVISORY_URL=""
            ADVISORY_RUSTSEC=""
            IN_ADVISORY=true
        elif echo "$line" | grep -q "^Version:"; then
            ADVISORY_VERSION=$(echo "$line" | sed 's/^Version: //' | sed 's/"/\\"/g')
        elif echo "$line" | grep -q "^Title:"; then
            ADVISORY_TITLE=$(echo "$line" | sed 's/^Title: //' | sed 's/"/\\"/g')
        elif echo "$line" | grep -q "^URL:"; then
            ADVISORY_URL=$(echo "$line" | sed 's/^URL: //' | sed 's/"/\\"/g')
        elif echo "$line" | grep -q "^RustSec:"; then
            ADVISORY_RUSTSEC=$(echo "$line" | sed 's/^RustSec: //' | sed 's/"/\\"/g')
        elif echo "$line" | grep -q "^Informational"; then
            IN_ADVISORY=true
        elif echo "$line" | grep -q "^Unaudited"; then
            UNAUDITED_COUNT=$(echo "$line" | grep -oE '[0-9]+' | head -1 || echo "0")
        fi
    done <<< "$AUDIT_OUTPUT"

    # Save last advisory
    if [ -n "$ADVISORY_CRATE" ]; then
        [ "$FIRST" = true ] && FIRST=false || VULNS_JSON="${VULNS_JSON},"
        VULNS_JSON="${VULNS_JSON}{\"crate\":\"${ADVISORY_CRATE}\",\"version\":\"${ADVISORY_VERSION}\",\"title\":\"${ADVISORY_TITLE}\",\"url\":\"${ADVISORY_URL}\",\"advisory\":\"${ADVISORY_RUSTSEC}\"}"
    fi
    VULNS_JSON="${VULNS_JSON}]"

    # Extract total dependencies count
    TOTAL_DEPS=$(echo "$AUDIT_OUTPUT" | grep -oE '[0-9]+ crates audited' | grep -oE '[0-9]+' || echo "0")

    # cargo audit exits 1 if vulnerabilities found
    if [ "$VULN_COUNT" -gt 0 ] || [ "$AUDIT_EXIT" -ne 0 ]; then
        EXIT_CODE=1
    fi
else
    TOTAL_DEPS="0"
fi

# Build JSON report
cat > "$REPORT_FILE" <<ENDJSON
{
  "script": "check_dependencies",
  "timestamp": "${TIMESTAMP}",
  "exit_code": ${EXIT_CODE},
  "audit_available": ${AUDIT_AVAILABLE},
  "audit_exit": ${AUDIT_EXIT},
  "vulnerabilities": ${VULN_COUNT},
  "unaudited": ${UNAUDITED_COUNT},
  "total_dependencies": ${TOTAL_DEPS},
  "vulnerability_details": ${VULNS_JSON}
}
ENDJSON

echo ""
echo "=== Results ==="
echo "Vulnerabilities: ${VULN_COUNT}"
echo "Unaudited:       ${UNAUDITED_COUNT}"
echo "Total Deps:      ${TOTAL_DEPS}"
echo "Report:          ${REPORT_FILE}"

if [ "$AUDIT_AVAILABLE" = false ]; then
    echo "Note: cargo-audit was not available, audit was skipped"
fi

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "Status:    PASS"
else
    echo "Status:    FAIL"
fi

exit $EXIT_CODE
