# 代码拓扑图（全域多维）

> 生成器 `scripts/ops/nt_topology.py`，索引 `scripts/ops/nt_mapgen.py`（1 秒重建）。
> **⛔ 本文件由代码生成，改它会被下次重建覆盖 —— 要改判据请改生成器。**

- **rs 文件** 2,855 · **代码行** 880,304 · **符号** 82,737
- 符号行号已全量核对：**82,737 个符号 100% 命中真实声明行**

## 维度 2 · 代码树分叉（⛔ 不可从目录名推断）

| 树 | 文件 | 行数 | 占比 | |
|---|---:|---:|---:|---|
| 主分层树 (L0–L6) | 2362 | 722,833 | ██████████████████ | `layered` |
| 文档/会话 | 370 | 77,109 | ██ | `doc` |
| 独立 crate | 243 | 75,386 | ██ | `crate` |
| ⚠️ 第二棵树 (逃过 check-layer-deps.sh) | 130 | 44,908 | █ | `second-tree` |
| core 内、层外 (entry/bin/examples) | 148 | 40,102 | █ | `core-outside-layers` |
| 其他 | 131 | 27,568 | █ | `other` |

> ⛔ **第二棵树 = 130 文件 / 44,908 行**，不参与 L0–L6，**逃过 `check-layer-deps.sh`**。
> 任何「目录 → 层」的自动推导都会漏掉它，故本图显式分叉。
> 依据：`docs/architecture/DIR-REMEDY-2026-09-28.md` §2.5。

### 2.1 L0–L6 分层明细（layered 树）

| 层 | 文件 | 行数 | 符号 | 占比 |
|---|---:|---:|---:|---|
| `l0_substrate` | 48 | 20,261 | 2,341 | `█` |
| `l1_action` | 505 | 156,792 | 15,007 | `████` |
| `l2_perception` | 360 | 92,295 | 9,784 | `██` |
| `l3_embodiment` | 247 | 68,114 | 7,045 | `██` |
| `l4_emotion` | 255 | 86,945 | 6,977 | `██` |
| `l5_cognition` | 698 | 223,720 | 21,883 | `██████` |
| `l6_meta` | 234 | 72,732 | 6,903 | `██` |

## 维度 1 · 物理目录树

```
neotrix-core/  (2625 文件, 805,617 行)
crates/  (229 文件, 74,647 行)
docs/  (1 文件, 40 行)
```

## 维度 3 · 符号密度 Top 30（定位入口）

> 「入口」= 含 `pub fn`/`pub struct` 的文件。改这些影响面最大。

| # | 文件 | 行 | 符号 | pub 符号 | modpath |
|---:|---|---:|---:|---:|---|
| 1 | `neotrix-core/src/l0_substrate/nt_ecs.rs` | 1,362 | 251 | 132 | `neotrix::l0_substrate::nt_ecs` |
| 2 | `neotrix-core/src/l5_cognition/nt_mind/nt_game/tests/mod.rs` | 1,485 | 181 | 0 | `neotrix::l5_cognition::nt_mind::nt_game::tests` |
| 3 | `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/mod.rs` | 463 | 171 | 97 | `neotrix::l4_emotion::nt_memory::nt_memory_kb` |
| 4 | `neotrix-core/src/agent.rs` | 1,048 | 163 | 83 | `neotrix::agent` |
| 5 | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_sandbox/mod.rs` | 1,186 | 160 | 100 | `neotrix::l3_embodiment::nt_shield::nt_shield_sandbox` |
| 6 | `neotrix-core/tests/nt_shield_integration.rs` | 1,900 | 155 | 0 | `neotrix::..::tests::nt_shield_integration` |
| 7 | `neotrix-core/src/l2_perception/nt_world/source/osint_bridge.rs` | 788 | 153 | 3 | `neotrix::l2_perception::nt_world::source::osint_bridge` |
| 8 | `crates/neotrix-neobot/src/nt_agent.rs` | 2,897 | 152 | 11 | `neotrix_neobot::nt_agent` |
| 9 | `neotrix-core/src/l1_action/nt_act/nt_act_trade/tests/test_extractors.rs` | 1,780 | 147 | 0 | `neotrix::l1_action::nt_act::nt_act_trade::tests::test_extractors` |
| 10 | `crates/neotrix-neobot/src/nt_channel_dispatch.rs` | 2,123 | 144 | 18 | `neotrix_neobot::nt_channel_dispatch` |
| 11 | `neotrix-core/src/l2_perception/nt_world/nt_world_search.rs` | 1,391 | 144 | 49 | `neotrix::l2_perception::nt_world::nt_world_search` |
| 12 | `neotrix-core/src/l2_perception/nt_world/osint/mod.rs` | 1,409 | 144 | 78 | `neotrix::l2_perception::nt_world::osint` |
| 13 | `neotrix-core/src/l1_action/nt_act/nt_act_trade/tests/test_orchestration.rs` | 1,958 | 143 | 0 | `neotrix::l1_action::nt_act::nt_act_trade::tests::test_orchestration` |
| 14 | `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/execution.rs` | 1,369 | 141 | 64 | `neotrix::l1_action::nt_io::nt_io_provider::gateway::execution` |
| 15 | `neotrix-core/src/l4_emotion/nt_memory/nt_trade_product_spec.rs` | 1,242 | 141 | 25 | `neotrix::l4_emotion::nt_memory::nt_trade_product_spec` |
| 16 | `neotrix-core/src/l5_cognition/nt_core_ttc.rs` | 1,285 | 139 | 67 | `neotrix::l5_cognition::nt_core_ttc` |
| 17 | `neotrix-core/src/l1_action/nt_act/nt_act_trade/trade_core.rs` | 931 | 137 | 46 | `neotrix::l1_action::nt_act::nt_act_trade::trade_core` |
| 18 | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/stage_contracts.rs` | 886 | 137 | 29 | `neotrix::l5_cognition::nt_mind::nt_mind::seal_core::self_iterating::stage_contracts` |
| 19 | `neotrix-core/src/l0_substrate/nt_core_telemetry.rs` | 1,572 | 135 | 60 | `neotrix::l0_substrate::nt_core_telemetry` |
| 20 | `neotrix-core/src/l5_cognition/nt_mind/nt_game/render/components.rs` | 910 | 135 | 78 | `neotrix::l5_cognition::nt_mind::nt_game::render::components` |
| 21 | `crates/neotrix-neobot/src/nt_channel_telegram.rs` | 2,353 | 134 | 18 | `neotrix_neobot::nt_channel_telegram` |
| 22 | `neotrix-core/src/neotrix/nt_file_ability/image_super_resolution.rs` | 1,672 | 134 | 66 | `neotrix::neotrix::nt_file_ability::image_super_resolution` |
| 23 | `neotrix-core/src/l0_substrate/nt_core_cross_layer.rs` | 1,036 | 133 | 75 | `neotrix::l0_substrate::nt_core_cross_layer` |
| 24 | `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/mod.rs` | 1,685 | 132 | 12 | `neotrix::l1_action::nt_io::nt_io_provider::gateway` |
| 25 | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | 1,345 | 131 | 26 | `neotrix::l5_cognition::nt_mind::nt_mind_background_loop::run` |
| 26 | `crates/neotrix-gateway/src/gate.rs` | 1,314 | 127 | 56 | `neotrix_gateway::gate` |
| 27 | `neotrix-core/src/l5_cognition/nt_mind/nt_game/render/input.rs` | 548 | 126 | 107 | `neotrix::l5_cognition::nt_mind::nt_game::render::input` |
| 28 | `neotrix-core/src/l0_substrate/nt_core_hex.rs` | 1,530 | 125 | 68 | `neotrix::l0_substrate::nt_core_hex` |
| 29 | `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/nt_memory_sweep_20260815.rs` | 1,074 | 125 | 76 | `neotrix::l4_emotion::nt_memory::nt_memory_kb::nt_memory_sweep_20260815` |
| 30 | `neotrix-core/tests/phase_integration_tests.rs` | 793 | 121 | 0 | `neotrix::..::tests::phase_integration_tests` |

## 维度 4 · 孤儿与异常

| 类别 | 数量 | 判据 | 含义 |
|---|---:|---|---|
| 无 modpath | 1 | 推不出 `crate::path` | 不在任何 Cargo crate 下（或路径异常）|
| 零符号 | 2 | items 为空 | 纯数据/宏/纯 impl 块，无可命名符号 |
| ⛔ 逃过层门 | 129 | `escapes-layer-gate` | **第二棵树，分层拓扑的盲区** |

**无 modpath 的 rs 文件（Top 15）** —— 这些是「精准定位」的死角：

- `docs/standards/templates/FITNESS-FN-TEMPLATE.rs` (40 行)

## 精准定位怎么用

```sh
# 文件 → 符号 + 行号 (v2 schema 新能力)
python3 scripts/ops/nt_locate.py --source-file nt_channel_dispatch.rs \
        --component on_inbound

# 索引新鲜度自检 (missing/extra 差集)
python3 scripts/ops/nt_locate.py --audit

# 索引损坏或过期时的降级: 绕开索引用 grep
python3 scripts/ops/nt_locate.py --index=off --component <名字>
```

索引是**派生产物**（`.project-map/` 已 gitignore），1 秒重建：

```sh
python3 scripts/ops/nt_mapgen.py     # → .project-map/codemap.json
python3 scripts/ops/nt_topology.py   # → 本文件
```

## 维度 5 · 全域代码审计（实测，非引用）

> 全部实测。扫描器告警先读现场证实/证伪再定性（R-SCAN-1）。

### 5.1 R-P1 零 unsafe

| 形态 | 数量 | 判定 |
|---|---:|---|
| `unsafe fn` | 0 | 真代码 |
| `unsafe impl` | 0 | 真代码 |
| `unsafe {}` / `unsafe(...)` 块 | 5 | 真代码 |
| `unsafe trait` | 0 | 逐个读现场判定 |

原始 grep `unsafe` 得 **206** 处，但逐处读现场后：全部是 `forbid` 声明、注释、或**字符串字面量** —— 本仓自带禁词扫描器（`nt_meta/scanner.rs`）、AST 检索器（`nt_act_code/ast_searcher.rs`）与代码生成器（`code_writer.rs`），它们把禁词当**数据**持有以便 grep。

⇒ **真实 unsafe = 5**，逐处证据：

| 位置 | 形态 | 判定 |
|---|---|---|
| `crates/neotrix-sysctl/src/lib.rs:24` | `unsafe { libc::getpid() }` | FFI，**正当** |
| `…/lib.rs:28,33,85,119` | `unsafe { libc::sysctl(…) }` | FFI + 裸指针解引用，**正当** |

5 处**全部集中在一个 crate**（`neotrix-sysctl`，macOS `sysctl` 进程枚举），属 FFI 必需，非任意内存操作。

⚠️ **但 `neotrix-core/src/lib.rs` 声明了 `#![forbid(unsafe_code)]`，而 `neotrix-sysctl` 同样声明 `forbid` 却含 5 处 unsafe** ⇒ `forbid` 声明与实际代码**不一致**，属**声明失效**，需裁决：要么该 crate 移除 `forbid` 并显式豁免 FFI，要么改用安全封装。

⇒ 结论：**R-P1 在 `neotrix-core` 内零违反**；`neotrix-sysctl` 的 5 处是 FFI 正当需求，但 `forbid` 声明是假的。

### 5.2 unwrap / expect / panic —— ⛔ 存量巨大且无门在管

| 位置 | `.unwrap()` | `.expect()` | `panic!` |
|---|---:|---:|---:|
| 全仓 | 5121 | 2908 | 173 |
| 测试目录内 | 1011 | 391 | 44 |
| **生产代码** | **4110** | **2517** | **129** |

**⛔ `AGENTS.md` / `RUST-STANDARDS.md` 明令生产代码禁这三者，但全仓无任何门或基线在度量** ⇒ 一次性历史债，存量裸奔，随时可能新增而无报警。

⇒ 建议建 `scripts/check-unwrap.sh` + 基线棘轮（只卡新增，不强求归零），与 `check-layer-deps` 的 8 条 known 同构。

### 5.4 重复类型 —— 同名 ≠ 同类型（L15 陷阱）

| 口径 | 数量 | 含义 |
|---|---:|---|
| 重复的**类型名** | 1121 | 同名出现 ≥2 次的**名字**数 |
| 名义多余定义 | 1797 | 每名保留 1 份后余下的（**含异构**） |
| **结构完全相同**的真重复组 | **147** | 字段集合逐项相同 |
| **真正可归并的定义** | **174** | 只有这个数才叫「可归并」 |

⇒ 1121 个同名里，**只有 174 个结构真同构**（占名义多余的 10%）。
其余是**合法的同名异构**（如 `TaskStatus` 出现 12 次却是 10 个不同枚举）—— 报原始名数会是对正确代码的误报，与 `unsafe` 字面量陷阱同一层次。

> ⚠️ 本表第一版把「名义多余 1797」误写成「可归并」并算出 160% —— **那正是本节警告的那个错误，我自己犯了一遍**。三个数已分列，逐个标明含义。

#### Top 12 真同构组（按可归并数）

| # | 类型 | kind | 字段数 | 份数 | 跨层分布 |
|---:|---|---|---:|---:|---|
| 1 | `CircuitState` | enum | 3 | 6 | l1_action, l3_embodiment, l5_cognition, l6_meta |
| 2 | `RiskLevel` | enum | 4 | 5 | l1_action, l3_embodiment |
| 3 | `Severity` | enum | 5 | 5 | l1_action, l3_embodiment |
| 4 | `Severity` | enum | 4 | 4 | l3_embodiment |
| 5 | `StepStatus` | enum | 5 | 4 | l1_action, l5_cognition |
| 6 | `ThreatLevel` | enum | 5 | 4 | l3_embodiment |
| 7 | `Cli` | struct | 1 | 3 | 跨 crate |
| 8 | `CrtTimeScale` | enum | 3 | 3 | l0_substrate, l5_cognition |
| 9 | `DepKind` | enum | 3 | 3 | l6_meta |
| 10 | `EventKind` | enum | 8 | 3 | l6_meta |
| 11 | `GapSeverity` | enum | 4 | 3 | l0_substrate, l1_action, l5_cognition |
| 12 | `GoalPriority` | enum | 4 | 3 | l5_cognition |

**逐处位置**（`nt_locate --component <名>` 可直查）：

- `CircuitState` ×6
  - `neotrix-core/src/l1_action/nt_act/actions/core/nt_act_circuit_breaker.rs:47`
  - `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/resilience/nt_resilience_types.rs:8`
  - `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/mod.rs:23`
  - `neotrix-core/src/l5_cognition/nt_mind/nt_mind/evolution/goal_loop/types.rs:91`
  - `neotrix-core/src/l6_meta/healing/self_healing/circuit_breaker.rs:9`
  - `neotrix-core/src/l6_meta/nt_core_guardian/circuit_breaker.rs:20`
- `RiskLevel` ×5
  - `neotrix-core/src/agent.rs:450`
  - `neotrix-core/src/l1_action/nt_act/nt_act_trade/production_logistics.rs:308`
  - `neotrix-core/src/l1_action/nt_act/nt_act_trade/capabilities/risk_assessor.rs:47`
  - `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/mod.rs:59`
  - `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_approval/human_approval.rs:24`
- `Severity` ×5
  - `neotrix-core/src/l1_action/nt_infra_ai/inspection.rs:15`
  - `neotrix-core/src/l3_embodiment/nt_shield/vulnerability_pipeline.rs:24`
  - `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_audit/threat_modeler.rs:29`
  - `neotrix-core/src/l3_embodiment/nt_shield/scanners/container_scan/vulnerability.rs:5`
  - `neotrix-core/src/l3_embodiment/nt_shield/scanners/red_team/result.rs:6`
- `Severity` ×4
  - `crates/neotrix-audit/src/nt_finding.rs:53`
  - `neotrix-core/src/l3_embodiment/nt_shield/compliance/requirement.rs:9`
  - `neotrix-core/src/l3_embodiment/nt_shield/safety/nt_safety_alignment.rs:78`
  - `neotrix-core/src/l3_embodiment/nt_shield/scanners/secret_scanner/finding.rs:12`
- `StepStatus` ×4
  - `neotrix-core/src/l1_action/nt_act/actions/orchestration/operator_runbook.rs:15`
  - `neotrix-core/src/l1_action/nt_act/actions/orchestration/provider_migration_router.rs:122`
  - `neotrix-core/src/l1_action/nt_act/nt_act_autonomy/per_agent.rs:24`
  - `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/self_evolver.rs:142`
- `ThreatLevel` ×4
  - `neotrix-core/src/l3_embodiment/nt_shield/dual_evidence.rs:25`
  - `neotrix-core/src/l3_embodiment/nt_shield/defense/unified_defense.rs:33`
  - `neotrix-core/src/l3_embodiment/nt_shield/guard/input_gatekeeper.rs:19`
  - `neotrix-core/src/l3_embodiment/nt_shield/guard/output_sentinel.rs:20`

⚠️ **归并不是免费的**：`Severity` 散在 L1/L3 与两个 crate，合并会改公开 API 与跨层依赖方向 ⇒ 需逐组评估，不宜批量脚本化。
