# 重复类型归并 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 147 组结构完全相同的重复类型定义收敛为单一定义，减少 174 个冗余定义，同时不破坏跨层依赖方向与公开 API。

**Architecture:** 三档风险分层施工。T1（同 crate 同层，52 组）先做 —— 纯移动，风险最低，用来验证工具链与门禁。T2（跨层，95 组）需裁决层归属真源（`.neotrix/layer-map.json`）。T3（跨 crate，64 组）改公开 API，最后做。

**Tech Stack:** Rust 2021 · workspace 多 crate（`neotrix-core` + `crates/*`）· 门禁 `check-layer-deps.sh --strict` / `check-unwrap.sh --strict` / `check-fresh-build.sh --full`

---

## ⛔ 施工前必读：三个会让人栽跟头的实测事实

**1. 「同名」≠「可归并」。** 1,121 个类型名重复，但只有 **174 个结构真同构**。
`TaskStatus` 出现 12 次却是 **10 个不同枚举** —— 归并非同构的会**改语义**。

**2. 本仓把禁词当数据写。** 用 `grep` 找这些类型名会命中
`nt_meta/scanner.rs` / `nt_laws.rs` 的测试夹具与注释。
⇒ 一律用 `nt_locate --component <名>`，**不用 grep**（AGENTS.md R-SCAN-1b）。

**3. 报告里的行号会腐化。** 本文件行号 2026-09-29 有效。
**施工时先跑**：
```sh
python3 scripts/ops/nt_locate.py --component <类型名>
```

## 取数入口

```sh
python3 scripts/ops/nt_topology.py          # 维度 5.4，含 147 组逐处位置
```

---

# T1 · 同 crate 同层（52 组）· 低风险，先做

## Task 1: 建立基线与工具自检

**Files:**
- Read: `docs/architecture/CODE-TOPOLOGY.md`（维度 5.4）
- Create: `scripts/dup-types-baseline.txt`（归并进度台账）

**Step 1: 记录当前门禁基线**

Run:
```sh
cargo check -p neotrix --lib -j4 2>&1 | tail -2
bash scripts/check-layer-deps.sh --strict 2>&1 | tail -3
bash scripts/check-unwrap.sh --strict; echo "rc=$?"
```
Expected: `Finished` 0 error · layer-deps `0 new violation(s); 8 known` · unwrap `rc=0`

**Step 2: 记录真同构组总数**

Run:
```sh
python3 scripts/ops/nt_topology.py >/dev/null && \
  grep -A4 '真同构组' docs/architecture/CODE-TOPOLOGY.md | head -4
```
Expected: 147 组 / 174 可归并

**Step 3: 建台账**

台账格式（**列表不存计数**，同 `layer-deps-baseline.txt` 理由 ——
计数会让「归并一个又新增一个」永久隐身）：
```
<类型名>\t<层>\t<份数>\t<状态>
ThreatLevel\tl3_embodiment\t4\tpending
```

**Step 4: Commit**

```sh
git add scripts/dup-types-baseline.txt
git commit -m "chore(audit): 建重复类型归并台账（147 组基线）"
```

---

## Task 2: 归并 `ThreatLevel`（4x，L3 同层，最高 fan-in）

**为什么先做它**：T1 里 fan-in 最高（13 个文件引用）且**全在 `l3_embodiment/nt_shield/` 一个子树内** —— 是验证「移动 + 改引用 + 过门」全流程的最佳样本。

**Files:**
- Modify: `neotrix-core/src/l3_embodiment/nt_shield/dual_evidence.rs:25`
- Modify: `neotrix-core/src/l3_embodiment/nt_shield/defense/unified_defense.rs:33`
- Modify: `neotrix-core/src/l3_embodiment/nt_shield/guard/input_gatekeeper.rs:19`
- Modify: `neotrix-core/src/l3_embodiment/nt_shield/defense/anti_distillation/mod.rs:18`
- Modify: `neotrix-core/src/l3_embodiment/nt_shield/proxy_detection/mod.rs:26`
- Test: `neotrix-core/tests/`（或就近模块测试）

**Step 1: 定位全部 4 处定义（不要信本文件的行号）**

Run:
```sh
python3 scripts/ops/nt_locate.py --component ThreatLevel
```
Expected: 4 处 `pub enum ThreatLevel` + 各自的行号

**Step 2: 写失败测试**

在保留处（选 fan-in 最高的那处，暂定 `nt_shield/` 下的公共位置）加：
```rust
#[test]
fn threat_level_is_single_definition_across_shield() {
    // 归并后：任一 shield 子模块都能用同一个 ThreatLevel
    use crate::l3_embodiment::nt_shield::defense::unified_defense::ThreatLevel as A;
    use crate::l3_embodiment::nt_shield::guard::input_gatekeeper::ThreatLevel as B;
    // 若两处仍各自定义，以下断言无法编译 —— 这就是我们要的失败
    let _same: fn(A) -> B = |a| a;
}
```
Run: `cargo test -p neotrix --lib threat_level_is_single 2>&1 | tail -5`
Expected: **编译失败**（两处类型不同源）

**Step 3: 选定唯一保留点并移动定义**

保留 `unified_defense.rs:33` 那份（若它有更完整的 `Display`/转换实现，优先它）。
删除其余 3 处定义，改为 `use` 导入：
```rust
use super::super::defense::unified_defense::ThreatLevel;
```

**Step 4: 跑测试确认通过**

Run: `cargo test -p neotrix --lib threat_level_is_single 2>&1 | tail -3`
Expected: PASS

**Step 5: 跑门禁**

```sh
cargo check -p neotrix --lib -j4 2>&1 | tail -2
bash scripts/check-layer-deps.sh --strict 2>&1 | tail -2
bash scripts/check-unwrap.sh --strict; echo "rc=$?"
```
Expected: `Finished` · `0 new` · `rc=0`

**Step 6: 更新台账 + Commit**

```sh
sed -i '' 's/^ThreatLevel\tl3_embodiment\t4\tpending/ThreatLevel\tl3_embodiment\t4\tdone/' scripts/dup-types-baseline.txt
git add neotrix-core/src/l3_embodiment/ scripts/dup-types-baseline.txt
git commit -m "refactor(l3): 归并 ThreatLevel 4→1（147 组真同构 T1 首批）"
```

**Step 7: 重新生成索引**

```sh
python3 scripts/ops/nt_mapgen.py && python3 scripts/ops/nt_topology.py
git add docs/architecture/CODE-TOPOLOGY.md
git commit -m "docs(map): 归并后刷新拓扑"
```
Expected: 5.4 的「可归并」从 174 降到 173

---

## Task 3: T1 剩余 51 组（批量，每组一个 commit）

**按 fan-in 升序处理**（先做低 fan-in 的，冲突面小）。完整清单：
`docs/architecture/CODE-TOPOLOGY.md` 维度 5.4 的 Top 12 表 + 逐处位置。

每组重复 Task 2 的 6 步。**每组一个独立 commit** —— 理由：`check-layer-deps` 的
基线是列表式的，批量提交会让「哪组引入了新违规」无法定位。

**T1 完成判据**：
- 台账中 52 组全部 `done`
- `check-layer-deps --strict` 仍 `0 new`
- `cargo test -p neotrix --lib` 无回归

---

# T2 · 跨层（95 组）· 需裁决层归属

**⛔ 开工前必读** `docs/architecture/DIR-REMEDY-2026-09-28.md` §2.5：
`neotrix-core/src/neotrix/`（130 文件 / 44,908 行）是**第二棵树**，
不参与 L0–L6 且**逃过 `check-layer-deps.sh`**。
⇒ 跨层归并若涉及 `neotrix/` 子树，**层门不会报**，必须手工核对
`.neotrix/layer-map.json` 的 `role` 字段。

## Task 4: 裁决「保留在哪一层」

**Step 1: 列出 T2 全部组与跨层情况**

```sh
python3 - <<'PY'
import json,re
G=json.load(open('/tmp/groups.json'))   # 由 nt_topology.py 维度 5.4 产出
for g in G:
    if len(g['layers'])>1 and not any(l.startswith('crate:') for l in g['layers']):
        print(f"{len(g['sites'])}x {g['name']:<24} {' → '.join(g['layers'])}")
PY
```

**Step 2: 对每组裁决保留层**

判据（按优先级）：
1. **被上层（更高层号）依赖的那层不能持有定义** —— 依赖只能向上
2. 定义应留在**语义最贴近**该类型的层（例：`ThreatLevel` 是盾牌概念 ⇒ L3）
3. 若跨层是因为「各层都自己造了个轮子」，保留点应选**最底层**
   （否则上层定义 ⇒ 下层要反向依赖）

**Step 3: 把裁决写进台账，加一列 `keep_at`**

**Step 4: 逐组施工，同 Task 2 的 6 步**

**T2 完成判据**：95 组全部 `done` 且 `check-layer-deps --strict` **仍 0 new**。
若出现 new violation，说明裁决违反了单向依赖 ⇒ **停下重新裁决该组**，不要改基线。

---

# T3 · 跨 crate（64 组）· 改公开 API，最后做

## Task 5: 逐组评估「值得合吗」

**⛔ 关键判断：跨 crate 同名类型**往往**不该合**。例如 `Severity` 既是
`neotrix-audit` 的 `nt_finding.rs:53` 又是 `neotrix-core` 的
`compliance/requirement.rs:9` —— 审计 crate 的发现等级与 core 的合规要求等级
**语义不同**，合并会污染下游。

**Step 1: 对每组回答三个问题**
1. 语义**真的**相同吗？（读 doc comment，不是只看字段名）
2. 依赖方向合法吗？（被依赖方不能依赖依赖方）
3. 合并后公开 API 变化会不会破坏外部消费者？

**Step 2: 三问有任一为否 ⇒ 标记 `keep-separate` 并写明理由，不合**

**预期结论**：T3 的 64 组里，**真正该合的可能不到 1/3**。
**「不合并」是合法产出** —— 本计划的目标是消除**真冗余**，不是消灭所有同名。

**Step 3: 对判定该合的组，同 Task 2 的 6 步**

---

# 全程必过的门（每个 Task 结束前）

```sh
cargo check -p neotrix --lib -j4        # 禁默认 -j10（16G 机 OOM，见 NTS-D01）
cargo check -p neotrix-types -j4         # T3 涉及
bash scripts/check-layer-deps.sh --strict
bash scripts/check-unwrap.sh --strict
bash scripts/check-fresh-build.sh --full # 干净检出仍能构建
python3 scripts/ops/nt_mapgen.py && python3 scripts/ops/nt_topology.py
```

# 不要做的事

- ⛔ 勿**批量脚本化**改 147 组 —— 每组要读 doc comment 判语义，脚本判不了
- ⛔ 勿把「同名」直接当「可合」—— 1,121 同名里只有 174 真同构
- ⛔ 勿为让门变绿而改 `layer-deps-baseline.txt` —— 那是掩盖不是修复
- ⛔ 勿在共享工作树原地做（他窗在改 Rust）⇒ 用独立 worktree
- ⛔ 勿信本文件与 `CODE-TOPOLOGY.md` 里的行号 ⇒ 用 `nt_locate` 现查
