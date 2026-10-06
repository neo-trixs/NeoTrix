#!/bin/bash
# 非空门证明：nt_claims_numbers（N-5）
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# 注入两处，覆盖本门的**两种**失败模式（缺一即门不完整）：
#   注入 #1  数字漂移：把 AGENTS.md 的「82 条索引」改成 9999，expected 仍是 82
#             ⇒ 门须红并指名 AGENTS.md + 报出文档值与期望值
#   注入 #2  断言空转：把一条 claim 的 pattern 改成永不命中的样子
#             ⇒ 门须红并报「pattern 一个都没命中」（防空转机制本身有判别力）
#
# ⚠️ 断言必须指名：只判 rc 会自证循环（门可能因别的 claim 变红）。
# ⚠️ 探针改的是**探针自己的备份副本**…不，它改的是真实注册表/文档，
#    故必须靠 trap 逐字节还原 —— 还原后与起始校验和比对（见下）。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

GATE="scripts/ops/nt_claims_numbers.py"
REG="scripts/claims-numbers.tsv"
DOC="AGENTS.md"

cp "$REG" "$REG.probe.bak"
cp "$DOC" "$DOC.probe.bak"
SUM_REG0=$(shasum -a 256 "$REG" | cut -d' ' -f1)
SUM_DOC0=$(shasum -a 256 "$DOC" | cut -d' ' -f1)

cleanup() {
  [ -f "$REG.probe.bak" ] && cp "$REG.probe.bak" "$REG"
  [ -f "$DOC.probe.bak" ] && cp "$DOC.probe.bak" "$DOC"
  rm -f "$REG.probe.bak" "$DOC.probe.bak"
  return 0
}
trap cleanup EXIT

# ── 注入 #1：数字漂移 ──────────────────────────────────────────────
# ⛔⛔ **不得硬编码那个数字**（2026-10-06 实测踩到）：初版写死「（82 条索引…）」，
#    结果真源一变（82→83）前置断言就失败 ⇒ 探针自己成了又一个陈旧数字。
#    ⇒ 先从文档里**读出**当前值，再 +1 注入漂移。
python3 - "$DOC" <<'PY'
import sys, pathlib, re
p = pathlib.Path(sys.argv[1])
s = p.read_text(encoding='utf-8')
m = re.search(r"（(\d+) 条索引", s)
assert m, "probe precondition failed: AGENTS.md 里找不到「N 条索引」"
cur = int(m.group(1))
p.write_text(s.replace(m.group(0), f"（{cur + 9177} 条索引"), encoding='utf-8')
print(f"  （注入漂移：{cur} → {cur + 9177}）")
PY

rc=0
out=$(python3 "$GATE" --strict 2>&1) || rc=$?
assert_gate_red "$GATE (注入 #1: 数字漂移)" "$rc"

if printf '%s' "$out" | grep -qF "$DOC" && printf '%s' "$out" | grep -qE '9[0-9]{3}'; then
  echo "  ✅ 门指名了文档并报出注入值 ⇒ 不是空门"
else
  echo "  ❌ 门红了但未指名文档/实测值" >&2; printf '%s\n' "$out" >&2; exit 1
fi
cp "$DOC.probe.bak" "$DOC"

# ── 注入 #2：断言空转（防空转机制自身的判别力）────────────────────
python3 - "$REG" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1])
lines = p.read_text(encoding='utf-8').split('\n')
for i, l in enumerate(lines):
    if l.startswith("AGENTS.md\t"):
        f = l.split('\t')
        # 改 doc_pattern（第 2 列），保留 src/source 不动
        f[1] = "ZZZ_NEVER_MATCHES_(\\d+)"
        lines[i] = '\t'.join(f)
        break
else:
    raise SystemExit("probe precondition failed: AGENTS.md claim not found")
p.write_text('\n'.join(lines), encoding='utf-8')
PY

rc=0
out2=$(python3 "$GATE" --strict 2>&1) || rc=$?
assert_gate_red "$GATE (注入 #2: 断言空转)" "$rc"

if printf '%s' "$out2" | grep -q '一个都没命中'; then
  echo "  ✅ 门抓到「pattern 命中不到」⇒ 防空转机制自己有判别力"
else
  echo "  ❌ 门红了但未报空转 ⇒ 防空转分支可能是死代码" >&2
  printf '%s\n' "$out2" >&2; exit 1
fi
cp "$REG.probe.bak" "$REG"

# ── 还原后必须逐字节一致（探针不污染工作树）──────────────────────
SUM_REG1=$(shasum -a 256 "$REG" | cut -d' ' -f1)
SUM_DOC1=$(shasum -a 256 "$DOC" | cut -d' ' -f1)
[ "$SUM_REG0" = "$SUM_REG1" ] || { echo "  ❌ $REG 还原后不一致" >&2; exit 1; }
[ "$SUM_DOC0" = "$SUM_DOC1" ] || { echo "  ❌ $DOC 还原后不一致" >&2; exit 1; }

# ── 还原后门必须转绿（可满足性）──────────────────────────────────
rc2=0
python3 "$GATE" --strict >/dev/null 2>&1 || rc2=$?
if [ "$rc2" -ne 0 ]; then
  echo "  ❌ 清理后门仍红（rc=${rc2}）⇒ 恒红的门等于没有门" >&2; exit 1
fi
echo "  ✅ 清理后门转绿 ⇒ 非空 + 可满足（两道判据都成立）"
echo "  ✅ 两个文件均逐字节还原"
echo "PROBE-OK: $GATE 两种失败模式都能抓到"