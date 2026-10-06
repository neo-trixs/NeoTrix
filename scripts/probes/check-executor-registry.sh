#!/bin/bash
# 非空门证明：check-executor-registry
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# 注入形态：**把无执行器的能力谎报为 Executable** —— 正是本轮 4 处
# 「上架了但不可用/是桩」的**记账层**形态。
#
# ⚠️ 断言必须**指名该 id**：只判 rc 的话，门若因别处残留而红就会自证循环。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

TARGET="neotrix-core/src/l1_action/nt_act/nt_act_trade/production_logistics.rs"
MARK='description: "生产物流",'

cleanup() { cp "$TARGET.bak" "$TARGET" 2>/dev/null; rm -f "$TARGET.bak"; }
trap cleanup EXIT

[ -f "$TARGET" ] || { echo "probe: 目标文件不存在" >&2; exit 2; }
cp "$TARGET" "$TARGET.bak"

python3 - "$TARGET" <<'PY'
import re, sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
# 在该 id 旁插入一个谎报为 Executable 的条目（manifest 里）
m = open("crates/nt-core-capability-tree/src/market.rs", encoding="utf-8").read()
assert 'NT-MIND::trade::trade_production_logistics' in m
bad = m.replace("executability: Executability::DeclaredOnly,\n    },\n    ManifestEntry {\n        id: \"NT-MIND::trade::trade_finance_compliance\"",
                "executability: Executability::Executable,\n    },\n    ManifestEntry {\n        id: \"NT-MIND::trade::trade_finance_compliance\"")
assert bad != m, "注入点未命中"
open("crates/nt-core-capability-tree/src/market.rs", "w", encoding="utf-8").write(bad)
open("/tmp/nt_probe_manifest.bak", "w", encoding="utf-8").write(m)
PY

cleanup() {
  cp /tmp/nt_probe_manifest.bak crates/nt-core-capability-tree/src/market.rs 2>/dev/null
  cp "$TARGET.bak" "$TARGET" 2>/dev/null
  rm -f "$TARGET.bak" /tmp/nt_probe_manifest.bak
}
trap cleanup EXIT

out=$(bash scripts/check-executor-registry.sh --strict 2>&1)
if echo "$out" | grep -qF "NT-MIND::trade::trade_production_logistics"; then
  echo "PASS: 门指名报出被谎报为 Executable 的能力"
  exit 0
fi
echo "FAIL: 门未报出谎报项" >&2
echo "$out" | tail -4 >&2
exit 1
