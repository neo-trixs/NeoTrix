#!/usr/bin/env bash
# Provenance checker — verify external build inputs against provenance/external-inputs.json.
# Absorbed: grok-bot-0.18-reconstructed "pinned build input" pattern.
#
# Usage:
#   bash scripts/provenance_check.sh              # structural + local-cache checks
#   bash scripts/provenance_check.sh --url-check  # additionally probe URLs (network)
#
# Exit codes: 0 = all pass, 1 = any FAIL.
set -euo pipefail

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'
log()   { echo -e "${GREEN}==>${NC} ${BOLD}$1${NC}"; }
warn()  { echo -e "${YELLOW}==>${NC} $1"; }
error() { echo -e "${RED}==>${NC} $1" >&2; }

URL_CHECK=0
[ "${1:-}" = "--url-check" ] && URL_CHECK=1

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/provenance/external-inputs.json"

command -v python3 >/dev/null || error "python3 required"
[ -f "$MANIFEST" ] || error "manifest missing: $MANIFEST"

# sha256 helper: Linux (sha256sum) or macOS (shasum -a 256)
sha256_of() {
    if command -v sha256sum >/dev/null; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        return 1
    fi
}

log "Provenance check: $MANIFEST"

PASS=0
FAIL=0

# Emit one entry per input as TSV: name|kind|version|sha256|purpose
while IFS=$'\t' read -r name kind version sha purpose; do
    status="PASS"
    detail=""

    # 1. Structural: required fields non-empty
    if [ -z "$name" ] || [ -z "$kind" ] || [ -z "$version" ] || [ -z "$sha" ]; then
        status="FAIL"
        detail="missing required field"
    fi

    # 2. Hash verification when a real hash is anchored
    if [ "$status" = "PASS" ]; then
        case "$sha" in
            unpinned:*)
                detail="honest-unpinned (${sha#unpinned:})"
                ;;
            *)
                # Real hash: verify if a locally cached artifact exists at
                # provenance/cache/<name-safe> else mark deferred (no fetch in CI gate).
                cache="$ROOT/provenance/cache/$(echo "$name" | tr '/' '_')"
                if [ -f "$cache" ]; then
                    actual="$(sha256_of "$cache" || true)"
                    if [ "$actual" != "$sha" ]; then
                        status="FAIL"
                        detail="hash mismatch expected=$sha got=${actual:-none}"
                    else
                        detail="local cache verified"
                    fi
                else
                    detail="real hash anchored; cache absent (deferred to fetch-time verify)"
                fi
                ;;
        esac
    fi

    # 3. Optional live reachability probe
    if [ "$status" = "PASS" ] && [ "$URL_CHECK" = "1" ] && [ "$kind" = "github-action" ]; then
        owner="${name%%/*}"
        repo="${name#*/}"
        code="$(curl -s -o /dev/null -w '%{http_code}' "https://github.com/$owner/$repo" || echo net-err)"
        case "$code" in
            200) detail="$detail; url 200" ;;
            *) status="FAIL"; detail="$detail; url probe got $code" ;;
        esac
    fi

    if [ "$status" = "PASS" ]; then
        PASS=$((PASS + 1))
        echo "  PASS  $name@$version — $detail"
    else
        FAIL=$((FAIL + 1))
        warn "  FAIL  $name@$version — $detail"
    fi
done < <(python3 -c '
import json, sys
data = json.load(open(sys.argv[1]))
for i in data["inputs"]:
    row = [i.get("name",""), i.get("kind",""), i.get("version",""), i.get("sha256",""), i.get("purpose","")]
    print("\t".join(row))
' "$MANIFEST")

TOTAL=$((PASS + FAIL))
if [ "$TOTAL" -eq 0 ]; then
    error "no inputs parsed from manifest — schema drift?"
fi

log "Provenance: $PASS passed, $FAIL failed, $TOTAL total"
[ "$FAIL" -eq 0 ]
