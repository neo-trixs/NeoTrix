#!/bin/bash
# 非空门证明：check-truth-surface
# 契约：注入一个已知违规 → 跑门 → 断言门变红 → 清理。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

T="neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/zz_probe_undeclared.rs"

cleanup() {
  rm -f "neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/zz_probe_undeclared.rs"
  [ -f "neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/zz_probe_undeclared.rs.probe.bak" ] && mv "neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/zz_probe_undeclared.rs.probe.bak" "neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/zz_probe_undeclared.rs"
  rm -f ".github/workflows/ci.yml.probe.bak"
}
trap cleanup EXIT

inject_undeclared_rs "$T"
rc=0
bash scripts/check-truth-surface.sh --strict >/dev/null 2>&1 || rc=$?
assert_gate_red "check-truth-surface" "$rc"
