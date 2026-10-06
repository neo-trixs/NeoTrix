#!/bin/bash
# 非空门证明：check-orphan-dirs
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# 注入形态：**孤儿目录** —— 一个含 .rs、但父 mod.rs 从不声明它的目录。
# 这正是本门要防的那件事（代码从未编译、测试从未跑，而 CI 全绿）。
#
# 落点选 `neotrix-core/src/l6_meta/`：该层是 L6，门扫 neotrix-core 全部层，
# 且 l6_meta 下已有若干已裁决的已知孤儿目录 ⇒ 注入落在同一「形状」里，
# 证明门不是靠「目录名陌生」才报。
#
# ⚠️ 断言必须**指名注入目录**：只判 rc 的话，门若因别处残留而红就会自证循环
#    （gate-registry.tsv 里 check-unwrap / check-silent-failure 的历史教训）。
#
# ⛔ 不往 `scripts/` 落：`scripts/` 不在任何 cargo crate 里，门根本不扫它
#    ⇒ 注在那里会得到 rc=0 的**假阴性**，看起来「门是空的」其实是我选错落点。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

GATE="check-orphan-dirs.sh"
DIR="neotrix-core/src/l6_meta/nt_probe_orphan"
FILE="$DIR/nt_probe_orphan.rs"

cleanup() {
  # 只删本探针自己建的那一个目录；trap 保证断言失败也还原
  [ -f "$FILE" ] && rm -f "$FILE"
  [ -d "$DIR" ] && rmdir "$DIR" 2>/dev/null
  return 0
}
trap cleanup EXIT

if [ -e "$DIR" ]; then
  PROBE_FAIL "注入目标已存在（疑似上次未清理）: $DIR"
fi

mkdir -p "$DIR" || exit 2
printf '// probe injection: module never declared by any parent mod.rs\npub fn nt_probe_orphan_fn() {}\n' > "$FILE"

rc=0
out=$(bash "scripts/$GATE" --strict 2>&1) || rc=$?
assert_gate_red "$GATE (注入孤儿目录)" "$rc"

if printf '%s' "$out" | grep -qF "nt_probe_orphan"; then
  echo "  ✅ 门精确指向注入的孤儿目录（nt_probe_orphan）⇒ 非自证循环"
else
  echo "  ❌ 门红了但未指向注入目录 ⇒ 可能命中了别的残留，门不精确" >&2
  printf '%s\n' "$out" | tail -12 >&2
  exit 1
fi

# 还原后必须恢复绿色（可满足性：不能是恒红门）
# ⚠️ 必须**先显式 cleanup 再验**：EXIT trap 此刻还没跑，注入目录仍在树上
#    ⇒ 不先清就断言「清理后转绿」，等于拿未清理状态当清理后（实测 rc=1）。
cleanup
rc2=0
bash "scripts/$GATE" --strict >/dev/null 2>&1 || rc2=$?
if [ "$rc2" -ne 0 ]; then
  echo "  ❌ 清理后门仍红（rc=${rc2}）⇒ 该门是恒红的，恒红的门等于没有门" >&2
  exit 1
fi
echo "  ✅ 清理后门转绿 ⇒ 非空且可满足（两道判据都成立）"
echo "PROBE-OK: $GATE 孤儿目录注入 ⇒ 变红 + 指名 + 清理后转绿"