#!/bin/bash
# 非空门证明：check-doc-drift
#
# ⚠️ 注入文件名**必须**匹配门自己的扫描规则 `-g 'nt_*.rs'`。
# 第一版注入 `zz_probe_undocumented.rs` ⇒ 被 glob 过滤 ⇒ 门返回 0
# ⇒ 元门报 `PROBE-BROKEN ... 仍返回 0 ⇒ 门是空的（假门）`。
# 实测是**探针坏了，不是门坏了** —— 这正是元门该报的东西，
# 只是需要人读出「报的是探针还是门」。见 _lib.sh 的契约注释。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

T="neotrix-core/src/l5_cognition/nt_probe_undocumented.rs"

cleanup() {
  rm -f "$T"
  [ -f "$T.probe.bak" ] && mv "$T.probe.bak" "$T"
}
trap cleanup EXIT

# 注入：内容无 //! 头，且**文件名匹配 nt_*.rs**
printf '// probe injection without module doc\n' > "$T"

rc=0
bash scripts/check-doc-drift.sh --strict >/dev/null 2>&1 || rc=$?
assert_gate_red "check-doc-drift" "$rc"
