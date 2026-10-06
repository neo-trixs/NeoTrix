#!/bin/bash
# 非空门证明：check-dead-config-flag
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# 注入形态：**零读点的 pub bool 字段**。
#   ⚠️ 首版注入写成**私有**字段（`foo: bool`）⇒ 门**不报** ⇒ 探针 FAIL。
#      原因：`_RE_FIELD_BOOL` 要求 `pub`（门只管「对外声称的配置面」，
#      私有字段不构成配置面）。⇒ 注入必须匹配门的**真实契约**，否则
#      探针会因选错形态而**假阴性** —— 那正是 check-orphan-dirs 探针
#      里记过的「选错落点得到 rc=0」同族坑。
#
# ⚠️ 断言必须**指名注入文件**：只判 rc 的话，门若因别处残留而红就会自证循环
#    （gate-registry.tsv 里 check-unwrap / check-silent-failure 的历史教训）。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

PROBE_REL="neotrix-core/src/l6_meta/nt_probe_deadflag.rs"
MARK="nt_probe_dead_flag_never_read"

cleanup() { rm -f "$PROBE_REL"; }
trap cleanup EXIT

if [ -e "$PROBE_REL" ]; then
  echo "probe: 落点已存在，先清理" >&2
  exit 2
fi

cat > "$PROBE_REL" <<EOF
// 探针注入（scripts/probes/check-dead-config-flag.sh，注入后自动清理）。
pub struct NtProbeDeadFlag {
    pub $MARK: bool,
}
EOF

out=$(bash scripts/check-dead-config-flag.sh --strict --types bool 2>&1)
if echo "$out" | grep -qF "$PROBE_REL"; then
  echo "PASS: 门指名报出 $PROBE_REL"
  exit 0
fi

echo "FAIL: 门未报出注入的 $PROBE_REL" >&2
echo "$out" | tail -5 >&2
exit 1
