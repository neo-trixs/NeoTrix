#!/bin/bash
# API surface inventory — NTS-D10 advisory implementation.
# Counts Tauri commands + Axum routes, checks for an OpenAPI spec.
# Without a spec there is nothing to diff (oasdiff) or fuzz (schemathesis):
# spec-first is step zero of the bake plan.
# Baseline (2026-09-21): 19 tauri commands, 267 .route( hits, 0 OpenAPI spec.
# Usage: bash scripts/check-api-surface.sh
# Note: bash-3.2-safe style. `bash -n` before commit.

echo "=== NeoTrix API surface inventory (NTS-D10) ==="

TAURI=$(rg -c '#\[tauri::command\]' --glob '*.rs' src-tauri/src neotrix-core/src 2>/dev/null | awk -F: '{s+=$2} END {print s+0}')
echo "Tauri commands: $TAURI"

ROUTES=$(rg -c '\.route\(' --glob '*.rs' neotrix-core/src src-tauri/src 2>/dev/null | awk -F: '{s+=$2} END {print s+0}')
echo "Axum .route( hits: $ROUTES (overcounts nested/test code; treat as ceiling)"

SPEC=$(rg --files . 2>/dev/null | rg -v "^./target/|/.git/" | rg -i "openapi|swagger" | head -n 3)
if [ -z "$SPEC" ]; then
  echo "OpenAPI spec: MISSING (bake plan step 0: export spec from Axum/Tauri surface)"
else
  echo "OpenAPI spec:"
  echo "$SPEC"
fi

echo "Bake plan: export spec -> oasdiff breaking gate (fail-on ERR) -> schemathesis 4 checks."
echo "DONE(advisory). Baseline: 19 tauri / 267 routes-ceiling / 0 spec."
