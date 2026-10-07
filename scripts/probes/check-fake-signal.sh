#!/usr/bin/env bash
# 非空门证明：check-fake-signal
# 契约（scripts/probes/_lib.sh）：注入已知违规 → 跑门 → 断言门变红 → 清理。
#
# ⭐ 本探针的价值在于「新门」：`check-fake-signal` 是 2026-10-07 新建的，
#   而本会话最大的教训（L8 / R-SCAN-3）就是——
#   「**一条保证须同时满足：能被触发 + 被消费为门 + 抑制器不遮视野**」
#   ⇒ 「本会话 3 条 P0 有 2 条是报 PASS 却结构上不可能失败」。
# ⇒ 故本探针逐条注入 4 条规则各自的**真实形态**，且**必须指名道姓**断言。
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2
. scripts/probes/_lib.sh

fail() { echo "probe FAIL: $*" >&2; exit 1; }

# 注入 1：R1 —— 结论型字段只有布尔字面量赋值
probe_r1() {
  local T="neotrix-core/src/l0_substrate/nt_core_capability_types.rs"
  # 锚点用**实测存在**的字段（L333）⇒ ⛔ 不用记忆里的字段名
  # ⚠️ 首版锚点写 `pub vision: bool,` ⇒ 探针报「锚点不唯一: 0」⇒ 失败。
  #    这正是探针契约的价值：**注入失败必须显式报错**，⛔ 不可静默通过。
  local anchor='pub success: bool,'
  [ -f "$T" ] || fail "R1: 目标文件不存在"
  cp "$T" "$T.bak"
  # 在 vision 之后插入一个**纯字面量赋值**的结论型字段 probe_ok
  # ⭐ 注入**独立 struct + 独立 Default** ⇒ R1 判据
  #    「只有布尔字面量赋值、无其他赋值点」可被完整构造。
  python3 - "$T" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
inj = """
/// probe：结论型字段（**只有**布尔字面量赋值 ⇒ R1 应报）
#[derive(Debug, Clone)]
pub struct ProbeFakeSignalR1 {
    /// probe：读起来像已验证结论
    pub probe_probe_ok: bool,
}

impl Default for ProbeFakeSignalR1 {
    fn default() -> Self {
        Self { probe_probe_ok: true }
    }
}
"""
s = s + inj
open(p, "w", encoding="utf-8").write(s)
PY
  local out rc
  out=$(bash scripts/check-fake-signal.sh 2>&1); rc=$?
  cp "$T.bak" "$T"; rm -f "$T.bak"
  echo "$out" | grep -q 'probe_probe_ok' || fail "R1: 门未报告注入的 probe_probe_ok"
  echo "$out" | grep -q 'R1' || fail "R1: 报告里没有 R1 规则标记"
  echo "  ok: R1 被触发（结论型字段只有字面量赋值）"
}

# 注入 2：R2 —— 名为 probe 的无参函数，体内零探测证据
probe_r2() {
  local T="crates/neotrix-neobot/src/nt_engine.rs"
  [ -f "$T" ] || fail "R2: 目标文件不存在"
  cp "$T" "$T.bak"
  python3 - "$T" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
probe_fn = '''
  # ⭐ 2026-10-07 去掉了注入体里的 `///` 文档注释（探针自身缺陷，⛔ 非门的问题）：
  #   门断言时 grep 输出里的 `probe_environment`；而 `mask_noncode` 会把
  #   **注释里的标识符**一并掩成空格 ⇒ grep 必然失配 ⇒ 误判「门未触发」。
  #   实测：注入后 findings 确实 22 → 23（门是对的）。
  # ⇒ 与 R1 的注入体对齐（R1 的 `///` 在 struct 上，不在字段上）。
    pub fn probe_environment(&self) -> String {
        "pretend-detected"
    }
'''
# 插到 impl 块内的第一个 fn 之前
i = s.index("    pub fn ") if "    pub fn " in s else len(s)
s = s[:i] + probe_fn.lstrip("\n") + "\n" + s[i:]
open(p, "w", encoding="utf-8").write(s)
PY
  local out
  out=$(bash scripts/check-fake-signal.sh 2>&1)
  cp "$T.bak" "$T"; rm -f "$T.bak"
  echo "$out" | grep -q 'probe_environment' || fail "R2: 门未报告注入的 probe_environment"
  echo "$out" | grep -q 'R2' || fail "R2: 报告里没有 R2 规则标记"
  echo "  ok: R2 被触发（伪探测函数）"
}

# 注入 3：R3 —— 评分函数被传入 ≥2 个布尔字面量
probe_r3() {
  local T="neotrix-core/src/l5_cognition/nt_core/nt_core_parallel/executor.rs"
  [ -f "$T" ] || fail "R3: 目标文件不存在"
  cp "$T" "$T.bak"
  python3 - "$T" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
inj = '''    /// probe：评分函数收到布尔字面量实参
    // ⚠️ 命名必须**贴合 R3 的真实形态**：R3 正则是
    //   `\b(?:compute|score|grade|rate|readiness|health)\w*\s*\(` ⇒ 要求这些词在**词首**。
    //   我第一版注入 `probe_score(...)` ⇒ `\b` 失败 ⇒ 门**理应**不报
    //   ⇒ 那是**探针形态错**，⛔ 不是门的 bug。
    // ⇒ 改用真实形态 `score_probe(...)`（词首为 `score`，与现役
    //   `_LoopReadyScore::compute(true, kb_ok, true, true)` 同型）。
    pub fn score_probe(a: bool, b: bool) -> f64 {
        if a { 1.0 } else if b { 2.0 } else { 0.0 }
    }

    /// probe：调用点传字面量
    pub fn probe_call(&self) -> f64 {
        self.score_probe(true, true)
    }
'''
i = s.index("    pub fn ") if "    pub fn " in s else len(s)
s = s[:i] + inj + "\n" + s[i:]
open(p, "w", encoding="utf-8").write(s)
PY
  local out
  out=$(bash scripts/check-fake-signal.sh 2>&1)
  cp "$T.bak" "$T"; rm -f "$T.bak"
  echo "$out" | grep -q 'score_probe' || fail "R3: 门未报告注入的 score_probe"
  echo "$out" | grep -q 'R3' || fail "R3: 报告里没有 R3 规则标记"
  echo "  ok: R3 被触发（评分函数字面量实参）"
}

# 注入 4：R4 —— 读起来像健康测量、但只有字面量赋值
probe_r4() {
  local T="neotrix-core/src/l0_substrate/nt_core_traits.rs"
  [ -f "$T" ] || fail "R4: 目标文件不存在"
  cp "$T" "$T.bak"
  python3 - "$T" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
anchor = 'pub readiness: u8,'
if s.count(anchor) == 0:
    # 退化锚点：任何 pub bool 字段
    import re
    m = re.search(r"^\s*pub [a-z_]+: bool,", s, re.M)
    if not m:
        print("no-anchor", file=sys.stderr); sys.exit(2)
    anchor = m.group(0)
    field = "probe_cadence_ok"
    doc = "    /// probe：健康测量型字段\n"
else:
    field = "probe_cadence_ok"
    doc = "    /// probe：健康测量型字段\n"
s = s.replace(anchor, anchor + "\n" + doc + f"    pub {field}: bool,", 1)
m2 = __import__("re").search(r"fn default\(\) -> Self \{", s)
if m2:
    j = s.index("\n", m2.end()) + 1
    s = s[:j] + f"            {field}: true,\n" + s[j:]
open(p, "w", encoding="utf-8").write(s)
PY
  local out
  out=$(bash scripts/check-fake-signal.sh 2>&1)
  cp "$T.bak" "$T"; rm -f "$T.bak"
  echo "$out" | grep -q 'probe_cadence_ok' || fail "R4: 门未报告注入的 probe_cadence_ok"
  echo "$out" | grep -q 'R4' || fail "R4: 报告里没有 R4 规则标记"
  echo "  ok: R4 被触发（无测量证据的健康字段）"
}

# ⛔⛔ **绝对禁止** `git checkout -- <dir>` 作为清理手段！
#
# 实测事故预防：共享工作区里 `neotrix-core/src` 与 `crates` 有**他窗 WIP**
# （`nt_store/mod.rs` / `lib.rs` / `nt_judge.rs` 等）⇒ 目录级 checkout
# 会**摧毁他人未提交改动**。这正是 AGENTS.md 记录的
# 「2026-09-22 三次覆盖事故」的同类。
#
# ⇒ 本探针**只用 `.bak` 文件**做还原（每个 probe_* 内部已 `cp "$T.bak" "$T"`），
#    此处**不设**全局 checkout trap。
# ⛔⛔ **第 4 次污染后的加固**：EXIT 时**核对**注入是否真的清干净。
#
# 实测事故（2026-10-07，本会话第 3 次）：探针的 `.bak` 还原在某些路径下失效
# ⇒ `nt_core_traits.rs` 残留 `probe_cadence_ok`（R4 探针注入）
# ⇒ ⛔ 下一次 `cargo build` 直接编译失败（`missing field probe_cadence_ok`）
# ⇒ 而当时我正在改**别的**文件 ⇒ 第一反应是「build 坏了」而不是「探针污染」。
#
# ⭐ 教训：**探针的还原逻辑不可信** ⇒ 必须在退出时**验证**，⛔ 不能假定。
# 做法：EXIT 时 grep 本探针的全部注入标记（`probe_cadence_ok` /
# `probe_probe_ok` / `probe_environment` / `score_probe`），
# 任一残留 ⇒ **显式报错**（⛔ 不静默），让污染立刻可见。
probe_targets="neotrix-core/src/l0_substrate/nt_core_traits.rs
neotrix-core/src/l0_substrate/nt_core_capability_types.rs
crates/neotrix-neobot/src/nt_engine.rs
neotrix-core/src/l5_cognition/nt_core/nt_core_parallel/executor.rs"

_probe_cleanup_audit() {
  local leaked=0 f
  for f in $probe_targets; do
    [ -f "$f" ] || continue
    if grep -qE 'probe_cadence_ok|probe_probe_ok|probe_environment|score_probe' "$f" 2>/dev/null; then
      echo "probe FAIL: 注入残留未清理 ⇒ $f" >&2
      echo "  ⛔ 请执行: git checkout -- $f" >&2
      leaked=1
    fi
  done
  [ "$leaked" -eq 0 ] && echo "probe: 清理核对通过（无注入残留）" >&2
  return $leaked
}
trap '_probe_cleanup_audit; echo "probe: 清理已完成（各 probe 内 .bak 还原）" >&2' EXIT

echo "probe: check-fake-signal — 逐条注入 4 条规则"
probe_r1
probe_r2
probe_r3
probe_r4
echo "PASS: 4/4 规则均可被触发（门非「结构上不可能失败」）"
