#!/bin/bash
# Supply-chain IOC sweep — R-P248 companion, SIM-37 implementation.
# Scans Cargo.lock for packages named in public 2026 Rust supply-chain
# advisories (Socket threat research: proc-macro1 loader, time-utility
# exfilfiltrators, finch typosquat chain). Behavioral scanning (Socket GA)
# remains a P3 bake item; this script is the zero-cost tripwire that runs now.
# IOC list reviewed: 2026-09-21. Update it when new advisories land.
# Usage: bash scripts/check-supply-iocs.sh [lockfile]  (default: Cargo.lock)
# Exit 0 = clean, 1 = HIT (treat host as suspect per Socket guidance).
# Note: bash-3.2-safe style. `bash -n` before commit.

LOCK="${1:-Cargo.lock}"
FAIL=0

# name|bad-version-or-empty(any version is bad when empty after colon handling below)
IOCS="proc-macro1 proc-macro-en aovine arone aronenao tinymember chrono_anchor dnp3times time_calibrator time_calibrators time-sync finch-rust sha-rust"
# pinned-malicious exact versions: name==version
PINNED_BAD="arrayref==0.3.10 internment==0.8.7 append-only-vec==0.1.9"

if [ ! -f "$LOCK" ]; then
  echo "SKIP: no $LOCK (nothing to sweep)"
  exit 0
fi

echo "=== NeoTrix supply-chain IOC sweep (SIM-37) ==="
for name in $IOCS; do
  if rg -q "^name = \"$name\"$" "$LOCK"; then
    echo "HIT: package '$name' present in $LOCK"
    FAIL=1
  fi
done

for spec in $PINNED_BAD; do
  name="${spec%%==*}"
  ver="${spec##*==}"
  if awk "/^\[\[package\]\]/{get=0} /^name = \"$name\"$/{get=1} get&&/^version = \"$ver\"$/{found=1; exit} END{exit !found}" "$LOCK"; then
    echo "HIT: pinned-malicious $name $ver in $LOCK"
    FAIL=1
  fi
done

if [ "$FAIL" -eq 0 ]; then
  echo "CLEAN: no known-malicious packages in $LOCK."
else
  echo "FAIL: see Socket remediation (rotate creds, hunt IOCs, rebuild clean)."
  exit 1
fi
