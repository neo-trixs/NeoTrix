#!/bin/bash
# Supply-chain IOC sweep — R-P248 companion, SIM-37 implementation.
# Scans Cargo.lock for packages named in public 2026 Rust supply-chain
# advisories (Socket threat research: proc-macro1 loader, time-utility
# exfilfiltrators, finch typosquat chain). Behavioral scanning (Socket GA)
# remains a P3 bake item; this script is the zero-cost tripwire that runs now.
# IOC list reviewed: 2026-09-21. Update it when new advisories land.
# M2 (SIM-52): +2 advisory sweeps (never FAIL, WARN only):
#   A. foreign-code loaders (KaLM/R11: checkpoint .py, trust_remote_code, pickle/torch loads).
#   B. license hygiene (multica/R12: BSL/FSL/PolyForm/Commons-Clause/AGPL + lock stanzas w/o license).
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
  echo "advisory sweeps below still run (informational)."
fi

# --- M2 advisory A: foreign-code loaders (WARN only, never FAIL) ---
echo "--- advisory A: foreign-code loader patterns ---"
A_HITS=$(rg -l --no-messages -g '*.py' -g '*.rs' -g '!docs/*' -g '!sessions/*' -g '!target/*' -e 'trust_remote_code' -e 'torch\.load\(' -e 'pickle\.loads?\(' -e 'marshal\.loads?\(' . 2>/dev/null | head -20)
if [ -z "$A_HITS" ]; then
  echo "advisory-clean A: no loader patterns."
else
  echo "$A_HITS"
  echo "advisory-review A: confirm each hit loads only trusted artifacts."
fi

# --- M2 advisory B: license hygiene (WARN only, never FAIL) ---
echo "--- advisory B: license hygiene ---"
B_HITS=$(rg -l --no-messages -g '!docs/*' -g '!sessions/*' -g '!target/*' -g '!Cargo.lock' -e 'Multica License' -e 'BSL-1\.1' -e 'FSL-' -e 'PolyForm' -e 'Commons Clause' . 2>/dev/null | head -20)
if [ -z "$B_HITS" ]; then
  echo "advisory-clean B1: no custom-license markers in code."
else
  echo "$B_HITS"
  echo "advisory-review B1: borrowing patterns is fine, vendoring code is not."
fi
if [ -f "$LOCK" ]; then
  B_NOLIC=$(awk '/^\[\[package\]\]/{if(n && !lic)print n; n=""; lic=0} /^name = /{n=$3} /^license = /{lic=1} END{if(n && !lic)print n}' "$LOCK" | head -20)
  if [ -z "$B_NOLIC" ]; then
    echo "advisory-clean B2: every lock stanza carries a license field."
  else
    echo "$B_NOLIC"
    echo "advisory-review B2: packages above lack license metadata; check before enforce."
  fi
fi

if [ "$FAIL" -eq 0 ]; then
  exit 0
else
  exit 1
fi
