#!/bin/bash
# 非空门证明：check-arch-rules
# 契约：注入一个已知违规 → 跑门 → 断言门变红 → 清理。
#
# ⛔ 探针必须自身不硬编码任何"当前应绿"的事实 —— 它只断言"注入后变红"。
# ⛔ 清理用 trap，保证即使断言失败也不留脏状态。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

R=".neotrix/arch-rules.tsv"
BAK_R="$R.probe.bak"
BAK_B=".neotrix/arch-rules-baseline.txt"
BAK_BB="$BAK_B.probe.bak"

# ⛔ 2026-10-07 修正一个真 bug：初版 cleanup 是
#      `[ -f "$BAK_R" ] && mv "$BAK_R" "$R"; ... rm -f "$R" "$BAK_R" ...`
#    ⇒ mv 已把备份**移回**原位，随后的 `rm -f "$R"` 又把它删了
#    ⇒ 探针跑完，**规则表从仓库里消失**（实测：registry 随即报"恒红"，
#    根因就是规则源不存在）。这违反探针契约的反面 ——
#    不是"留脏状态"，而是**销毁了干净状态**。
# ⇒ 正确做法：备份文件一律 cp（不 mv），cleanup 只 rm 备份，永不碰原文件。
cleanup() {
  [ -f "$BAK_R" ] && cp "$BAK_R" "$R" && rm -f "$BAK_R"
  [ -f "$BAK_BB" ] && cp "$BAK_BB" "$BAK_B" && rm -f "$BAK_BB"
}
trap cleanup EXIT

[ -f "$R" ] || PROBE_FAIL "规则表不存在: $R"
[ -f "$BAK_B" ] || PROBE_FAIL "基线不存在: $BAK_B"

# ── 注入①：一条 falsifiable_by 为空的规则 ⇒ 违反①A 第一条准入 ──
cp "$R" "$BAK_R"
printf 'R_PROBE\t\tneotrix-core/src/l6_meta/coordination/nt_arch_rules.rs\n' >> "$R"

rc=0
bash scripts/check-arch-rules.sh >/dev/null 2>&1 || rc=$?
assert_gate_red "check-arch-rules(空 falsifiable_by)" "$rc"

# 还原后再验基线腐化也能被抓（注入②）
cp "$BAK_R" "$R"
cp "$BAK_B" "$BAK_BB"
printf 'R_DOES_NOT_EXIST\n' >> "$BAK_B"

rc=0
bash scripts/check-arch-rules.sh >/dev/null 2>&1 || rc=$?
assert_gate_red "check-arch-rules(基线腐化)" "$rc"

# ── 注入③：第三份层名清单 ⇒ 违反 R2（裁定 ②B 的"层名只定义一次"）──
# ⚠️ 必须注入到**代码**里（不是注释）—— 判据刻意跳过 `//` 行，
#    因为 nt_review_runner.rs 的文档注释本来就合法提到多个层名。
RUNNER="neotrix-core/src/l6_meta/nt_core_self_review/nt_review_runner.rs"
BAK_RUNNER="/tmp/.nt_probe_runner.$$.bak"
[ -f "$RUNNER" ] || PROBE_FAIL "目标不存在: $RUNNER"
cp "$RUNNER" "$BAK_RUNNER"
cleanup_runner() { [ -f "$BAK_RUNNER" ] && cp "$BAK_RUNNER" "$RUNNER" && rm -f "$BAK_RUNNER"; }
trap 'cleanup; cleanup_runner' EXIT

python3 - "$RUNNER" <<'PYINJ'
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
anchor = "    fn detect_import_layer(&self, line: &str) -> ArchLayer {"
inj = anchor + '\n        let _probe_third_list = ["l0_substrate", "l1_action", "l2_perception"];'
if anchor in s:
    open(p, 'w', encoding='utf-8').write(s.replace(anchor, inj, 1))
else:
    sys.stderr.write("PROBE-BROKEN: 注入锚点不存在\n"); sys.exit(2)
PYINJ
[ $? -eq 0 ] || PROBE_FAIL "注入③失败（锚点缺失）"

rc=0
bash scripts/check-arch-rules.sh >/dev/null 2>&1 || rc=$?
assert_gate_red "check-arch-rules(第三份层名清单)" "$rc"

cleanup_runner

# 还原后门应恢复绿（证明探针不是"永远红"）
cp "$BAK_BB" "$BAK_B"
rc=0
bash scripts/check-arch-rules.sh >/dev/null 2>&1 || rc=$?
if [ "$rc" -ne 0 ]; then
  echo "PROBE-BROKEN: 清理后门仍红 rc=$rc ⇒ 注入破坏了仓库或门本身有假红" >&2
  exit 2
fi
echo "PROBE-OK: 清理后门恢复绿（探针可复现且非破坏性）"
exit 0
