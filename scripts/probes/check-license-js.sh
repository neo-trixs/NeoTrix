#!/bin/bash
# 非空门证明：check-license-js
# 契约：注入一个已知违规（AGPL-3.0 假包）→ 跑门 → 断言门变红 → 清理。
#
# ⭐ 为什么选 AGPL 做注入物：它是本项目**真实遇到过的**许可阻断项
#   （volcengine/OpenViking，AGPL-3.0，且曾被 absorption-cache 回退静默隐藏）。
#   注入一个「假想的高危」不如注入一个「真发生过的」。
set -uo pipefail                       # ⛔ 不用 -e（断言要自己控制流）
cd "$(dirname "$0")/../.." || exit 2  # ⛔ 必须先 cd 到仓库根
. scripts/probes/_lib.sh

P="apps/neobot-desktop/neobot-ui/node_modules/.probe-license-js"

cleanup() { rm -rf "$P"; }
trap cleanup EXIT                       # ⛔ 必须有，且必须在注入之前

# ⛔ 探针目录**必须落在 node_modules 内**才能被门枚举到；
#    门对「node_modules 不存在」是 FAIL（判据 E），所以不能把它挪走。
inject_agpl_probe_pkg() {
  mkdir -p "$1" || return 1
  printf '{"name":"probe-agpl-pkg","version":"0.0.0","license":"AGPL-3.0"}' > "$1/package.json"
}

inject_agpl_probe_pkg "$P" || { echo "PROBE-FAIL: 注入失败" >&2; exit 2; }
rc=0
bash scripts/check-license-js.sh --strict >/dev/null 2>&1 || rc=$?
assert_gate_red "check-license-js" "$rc"
