#!/bin/bash
# 非空门证明：check-cli-plugin-descriptors（TODO.md F3）
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# 注入形态：一份 command 拼错的 descriptor —— 正是 F3 描述的缺陷形态
# （「command 拼错 / 参数非法 / mode 写错，没有任何门会红」）。
#
# ⚠️ 注入落点是 **NEOTRIX_PLUGINS_DIR 指向的临时目录**。
#    ⛔ 绝不是 ~/.config/neotrix/plugins/ —— 那是用户的真 descriptor，
#    本探针**绝不允许**碰它（改坏了只有用户自己会发现）。
# ⚠️ 断言必须**指名该 descriptor 文件名**：只判 rc 的话，门若因别处残留
#    而红就会自证循环（2026-09-29 同族教训）。
# ⚠️ 断言前先钉**注入前是绿的**（可满足性）：否则「门恒红」会被
#    当成「注入成功」而通过 —— 那正是恒红门骗过元门的路径。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

GATE="scripts/check-cli-plugin-descriptors.sh"
BAD_NAME="nt_probe_bad_command.json"
OK_NAME="nt_probe_ok.json"

FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/nt-cli-plugin-probe.XXXXXX")" || exit 2
cleanup() { rm -rf "$FIXTURE"; }
trap cleanup EXIT
[ -d "$FIXTURE" ] || PROBE_FAIL "临时夹具目录未建成"

cat > "$FIXTURE/$OK_NAME" <<'JSON'
{"name":"nt-probe-ok","command":"/bin/echo","args":["hi"],"probe_args":["nt-probe-ok"],"mode":"interactive"}
JSON

# ── Q2 可满足性前置：只有合规 descriptor 时门必须是绿的 ──────────────
out=$(NEOTRIX_PLUGINS_DIR="$FIXTURE" bash "$GATE" --strict 2>&1)
rc=$?
if [ "$rc" -ne 0 ]; then
  echo "PROBE-BROKEN: 注入**前**门就红了（exit=$rc）⇒ 拿恒红状态当注入效果" >&2
  echo "$out" | tail -5 >&2
  exit 2
fi
echo "PROBE-OK: 注入前门是绿的（exit=0，可满足性成立）"

# ── 注入违规：command 拼错 ⇒ C8 探活必红 ────────────────────────────
cat > "$FIXTURE/$BAD_NAME" <<'JSON'
{"name":"nt-probe-bad","command":"definitely-not-a-real-cli-xyz","args":[],"probe_args":["--version"],"mode":"interactive"}
JSON

out=$(NEOTRIX_PLUGINS_DIR="$FIXTURE" bash "$GATE" --strict 2>&1)
rc=$?
assert_gate_red "check-cli-plugin-descriptors" "$rc"

if echo "$out" | grep -qF "$BAD_NAME"; then
  # ⚠️ 必须写 ${BAD_NAME}：中文全角括号紧跟变量名时，bash 会把 `$BAD_NAME（`
  #    当成**一个**变量名（_lib.sh:80-83 记录的真坑）⇒ unbound variable。
  echo "PASS: 门指名报出 command 拼错的 descriptor（${BAD_NAME}）"
  exit 0
fi
echo "FAIL: 门未指名被注入的 descriptor" >&2
echo "$out" | tail -5 >&2
exit 1