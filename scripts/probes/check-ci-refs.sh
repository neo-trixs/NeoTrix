#!/bin/bash
# 非空门证明：check-ci-refs
# 契约：注入一个已知违规 → 跑门 → 断言门变红 → 清理。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

WF=".github/workflows/ci.yml"

cleanup() {
  rm -f ".github/workflows/ci.yml"
  [ -f ".github/workflows/ci.yml.probe.bak" ] && mv ".github/workflows/ci.yml.probe.bak" ".github/workflows/ci.yml"
  rm -f ".github/workflows/ci.yml.probe.bak"
}
trap cleanup EXIT

inject_phantom_ci_job "$WF" "neocodex-frontend"
rc=0
bash scripts/check-ci-refs.sh --strict >/dev/null 2>&1 || rc=$?
assert_gate_red "check-ci-refs" "$rc"
