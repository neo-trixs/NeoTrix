#!/bin/bash
# 非空门证明：check-doc-claims
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# ── 2026-10-06 重写：探针的「注入符号」从硬编码改为**运行时自选** ──
#
# ## 为什么重写（这是一次真实误判，不是洁癖）
#   旧版硬编码 `noise_ik`。2026-10-06 跑出 `PROBE-BROKEN: 门是空的（假门）`
#   ⇒ 元门把 check-doc-claims 记成「门有效性未获证」。
#   **但门是好的。** 读现场发现：`noise_ik` 在 2026-09-29 被接进生产
#   （`.../nt_shield_ztnet/protocol/mod.rs`），TODO.md:642 自己都记着
#   「该状态已于同日被改变」⇒ 注入的「零生产消费者」断言**变成真的**
#   ⇒ 门正确返回 0。
#   ⛔ 若照字面结论去「修门」，就是把 bug 引进**正确**的代码（R-SCAN-1）。
#
# ## 病根不是「符号选错」，是「符号会腐化」
#   任何硬编码的符号都会在某次接线后失效，于是探针周期性假报「假门」，
#   而人每次都会重新怀疑一遍门。⇒ **改为自选**：按门自己的判据
#   （唯一 basename + nt_ 前缀 + 有词边界引用）枚举候选，**逐个注入实测**，
#   取第一个真能让门变红且被门点名的符号。
#   ⛔ 若一个候选都试不红 ⇒ 那时门**确实**可疑了 ⇒ PROBE_FAIL 硬失败。
#      「找不到候选」与「门是假门」被刻意混成同一个失败，因为二者在
#      「本探针无法证明门可信」这件事上等价（宁缺勿错，本仓门纪律）。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

GATE="check-doc-claims.sh"
T="TODO.md"

[ -f "$T.probe.bak" ] || cp "$T" "$T.probe.bak"
# 起始校验和：用于末尾比对「还原是否逐字节一致」。
# ⛔⛔ 不能用 `git diff --quiet` 当判据 —— 那是问「相对 HEAD 有没有 diff」，
#    本轮实测：TODO.md 有本窗口自己的 3 处正当订正注释，探针立刻假失败 rc=1。
#    ⇒ 分不清「我的改动」与「探针残留」，正是本仓反复吃的那类错。
SUM0=$(shasum -a 256 "$T" | cut -d' ' -f1)
# ⛔ 必须用 cp 不用 mv：mv 会**消耗** .bak，第二次 restore 就失败
#    （实测第一轮选符号 + 第二轮正式断言共调 2 次 ⇒ 残留 2 行进 TODO.md）
restore() { cp "$T.probe.bak" "$T"; }
cleanup() { restore; rm -f "$T.probe.tmp" "$T.probe.bak"; }
trap cleanup EXIT

# ── 枚举候选符号 ────────────────────────────────────────────────────
# 判据与门一致：basename 唯一（同名文件会让门无法定位消费者 ⇒ 门会跳过）、
# nt_ 前缀且够长（避免 ring/text 这类子串污染）、有其他 .rs 的词边界引用。
#
# ⛔⛔ **不能用 `mapfile`**：macOS 自带 bash 是 **3.2**（实测 `mapfile: command
#    not found` + `set -u` 下 `CANDS: unbound variable` 连环炸）。
#    本仓 scripts/probes/_lib.sh 早就记着「bash 3.2 的已知怪癖」⇒ 同一坑。
#    改用 `while read` + `+=`，3.2 可用。
CAND_FILE=$(mktemp)
python3 - > "$CAND_FILE" <<'PY'
import subprocess, collections, os, re
files = [f for f in subprocess.run(["git","ls-files","*.rs"],
         capture_output=True, text=True).stdout.split("\n") if f.strip()]
base = collections.Counter(os.path.basename(f) for f in files)
out = []
for f in files:
    if base[os.path.basename(f)] != 1:      # 同名 ⇒ 门会跳过（非阻断）
        continue
    mod = os.path.splitext(os.path.basename(f))[0]
    if len(mod) < 10 or not re.match(r'^nt_[a-z0-9_]+$', mod):
        continue
    r = subprocess.run(["git","grep","-l","-w","-e",mod,"--","*.rs"],
                       capture_output=True, text=True).stdout.split("\n")
    if len([x for x in r if x.strip() and x.strip() != f]) >= 2:
        out.append(mod)
print("\n".join(out[:40]))
PY
CANDS=()
while IFS= read -r line; do
  [ -n "$line" ] && CANDS+=("$line")
done < "$CAND_FILE"
rm -f "$CAND_FILE"

if [ "${#CANDS[@]}" -eq 0 ]; then
  PROBE_FAIL "枚举不到候选符号 ⇒ 无法证明 $GATE 可注入（宁缺勿错）"
fi

# ── 逐个实测：取第一个真能让门变红、且被门点名的符号 ────────────────
CHOSEN=""; TRIED=0
for m in "${CANDS[@]}"; do
  TRIED=$((TRIED + 1))
  restore
  printf '\n%s\n' "| 99 | \`$m\` 零生产消费者 | 探针 | — |" >> "$T"
  rc=0
  out=$(bash "scripts/$GATE" 2>&1) || rc=$?
  if [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -qF "$m"; then
    CHOSEN="$m"; break
  fi
  [ "$TRIED" -ge 12 ] && break
done
restore

if [ -z "$CHOSEN" ]; then
  PROBE_FAIL "试了 $TRIED 个候选，$GATE 一个都没判红 ⇒ 该门确实无法证明非空"
fi
echo "  自选注入符号: ${CHOSEN}（试了 $TRIED 个候选）"

# ── 正式断言（在干净状态上重注入一次，rc 单独测）──────────────────
rc=0
printf '\n%s\n' "| 99 | \`$CHOSEN\` 零生产消费者 | 探针 | — |" >> "$T"
out=$(bash "scripts/$GATE" 2>&1) || rc=$?
assert_gate_red "$GATE (注入自选符号 $CHOSEN)" "$rc"

if printf '%s' "$out" | grep -qF "$CHOSEN"; then
  echo "  ✅ 门精确指向注入的符号（${CHOSEN}）⇒ 非自证循环"
else
  echo "  ❌ 门红了但未指向注入的符号 ⇒ 命中了别的残留，门不精确" >&2
  exit 1
fi
restore

# ── 字节安全：探针必须能把 TODO.md 还原到**逐字节**一致 ─────────────
#   旧版用 `grep -v > tmp` 只删注入行会改变字节（实测把 TODO.md 弄出 diff）。
SUM1=$(shasum -a 256 "$T" | cut -d' ' -f1)
if [ "$SUM0" != "$SUM1" ]; then
  echo "  ❌ 探针跑完 ${T} 与探针开始时不一致 ⇒ 还原不是字节安全的" >&2
  echo "     start=${SUM0}" >&2; echo "     now =${SUM1}" >&2
  exit 1
fi
echo "  ✅ ${T} 还原后与探针开始时逐字节一致（探针不污染工作树，也不误伤既有改动）"