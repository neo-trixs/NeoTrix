# 代码拓扑图（全域多维）

> 生成器 `scripts/ops/nt_topology.py`，索引 `scripts/ops/nt_mapgen.py`（1 秒重建）。
> **⛔ 本文件由代码生成，改它会被下次重建覆盖 —— 要改判据请改生成器。**

- **rs 文件** 2,863 · **代码行** 914,761 · **符号** 84,226
- 符号行号已全量核对：**84,226 个符号 100% 命中真实声明行**

## 维度 2 · 代码树分叉（⛔ 不可从目录名推断）

> ✅ **第二棵树已清空**（实测）：`neotrix-core/src/neotrix/` 仅剩 1 个 `.rs`（40 行，纯 re-export 面）。
> ✅ layer-map.json 的 consumer 路径全部存在（实测，无幽灵条目）。
>
> 上一版此处断言「8 个模块已全部显式登记 ⇒ 盲区已关」，**该断言已删除**：
> 本生成器无法验证「是否全部登记」，却把它写成结论。物理并成一棵目录的
> B 方案进度见 `docs/architecture/DIR-REMEDY-2026-09-28.md`。

| 树 | 文件 | 行数 | 占比 | |
|---|---:|---:|---:|---|
| 主分层树 (L0–L6) | 2480 | 787,046 | ███████████████████ | `layered` |
| 其他 | 1344 | 205,610 | █████ | `other` |
| 文档/会话 | 479 | 101,295 | ██ | `doc` |
| 独立 crate | 248 | 82,286 | ██ | `crate` |
| core 内、层外 (entry/bin/examples) | 153 | 43,702 | █ | `core-outside-layers` |
| 第二棵树（2026-09-30 B 方案后仅剩门面 re-export 面） | 1 | 40 |  | `second-tree` |

> ✅ **第二棵树已清空**：仅剩 `neotrix-core/src/neotrix/mod.rs`（40 行，纯 re-export 面，无实现）。
> 8 个模块全部回流至其声明层；另清掉两批死代码（`error_conversions.rs` 4 个无人触发的 `From` impl、`nt_core_capability_tree` 10 个零消费 re-export）。
> 搬迁过程记账 **19 条**层债（`layer-deps-baseline.txt`）。

### 2.1 L0–L6 分层明细（layered 树）

| 层 | 文件 | 行数 | 符号 | 占比 |
|---|---:|---:|---:|---|
| `l0_substrate` | 66 | 27,767 | 2,952 | `█` |
| `l1_action` | 550 | 174,359 | 16,312 | `████` |
| `l2_perception` | 360 | 97,310 | 10,114 | `██` |
| `l3_embodiment` | 246 | 69,918 | 7,079 | `██` |
| `l4_emotion` | 257 | 88,909 | 7,137 | `██` |
| `l5_cognition` | 750 | 250,234 | 23,816 | `██████` |
| `l6_meta` | 233 | 76,104 | 7,060 | `██` |

## 维度 1 · 物理目录树

```
neotrix-core/  (2617 文件, 828,353 行)
crates/  (233 文件, 81,506 行)
apps/  (12 文件, 4,862 行)
docs/  (1 文件, 40 行)
```

## 维度 3 · 符号密度 Top 30（定位入口）

> 「入口」= 含 `pub fn`/`pub struct` 的文件。改这些影响面最大。

| # | 文件 | 行 | 符号 | pub 符号 | modpath |
|---:|---|---:|---:|---:|---|
| 1 | `neotrix-core/src/l0_substrate/nt_ecs.rs` | 1,709 | 267 | 133 | `neotrix::l0_substrate::nt_ecs` |
| 2 | `neotrix-core/src/l5_cognition/nt_core_gate/nt_provenance.rs` | 2,821 | 238 | 117 | `neotrix::l5_cognition::nt_core_gate::nt_provenance` |
| 3 | `crates/neotrix-neobot/src/nt_agent.rs` | 4,419 | 210 | 19 | `neotrix_neobot::nt_agent` |
| 4 | `neotrix-core/src/l5_cognition/nt_mind/nt_game/tests/mod.rs` | 1,485 | 181 | 0 | `neotrix::l5_cognition::nt_mind::nt_game::tests` |
| 5 | `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/mod.rs` | 484 | 173 | 99 | `neotrix::l4_emotion::nt_memory::nt_memory_kb` |
| 6 | `neotrix-core/src/agent.rs` | 1,078 | 163 | 83 | `neotrix::agent` |
| 7 | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_sandbox/mod.rs` | 1,186 | 160 | 100 | `neotrix::l3_embodiment::nt_shield::nt_shield_sandbox` |
| 8 | `neotrix-core/tests/nt_shield_integration.rs` | 1,900 | 155 | 0 | `neotrix::..::tests::nt_shield_integration` |
| 9 | `neotrix-core/src/l2_perception/nt_world/source/osint_bridge.rs` | 788 | 153 | 3 | `neotrix::l2_perception::nt_world::source::osint_bridge` |
| 10 | `neotrix-core/src/l2_perception/nt_world/osint/mod.rs` | 1,503 | 151 | 78 | `neotrix::l2_perception::nt_world::osint` |
| 11 | `crates/neotrix-neobot/src/nt_channel_dispatch.rs` | 2,258 | 148 | 18 | `neotrix_neobot::nt_channel_dispatch` |
| 12 | `neotrix-core/src/l1_action/nt_act/nt_act_trade/tests/test_extractors.rs` | 1,780 | 147 | 0 | `neotrix::l1_action::nt_act::nt_act_trade::tests::test_extractors` |
| 13 | `neotrix-core/src/l5_cognition/nt_core_ttc.rs` | 1,446 | 146 | 69 | `neotrix::l5_cognition::nt_core_ttc` |
| 14 | `neotrix-core/src/l2_perception/nt_world/nt_world_search.rs` | 1,391 | 144 | 49 | `neotrix::l2_perception::nt_world::nt_world_search` |
| 15 | `neotrix-core/src/l1_action/nt_act/nt_act_trade/tests/test_orchestration.rs` | 1,958 | 143 | 0 | `neotrix::l1_action::nt_act::nt_act_trade::tests::test_orchestration` |
| 16 | `neotrix-core/src/l4_emotion/nt_memory/nt_trade_product_spec.rs` | 1,245 | 142 | 25 | `neotrix::l4_emotion::nt_memory::nt_trade_product_spec` |
| 17 | `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/execution.rs` | 1,369 | 141 | 64 | `neotrix::l1_action::nt_io::nt_io_provider::gateway::execution` |
| 18 | `neotrix-core/src/l6_meta/nt_approval.rs` | 1,734 | 141 | 35 | `neotrix::l6_meta::nt_approval` |
| 19 | `neotrix-core/src/l1_action/nt_act/nt_act_trade/capability_registry.rs` | 1,233 | 140 | 39 | `neotrix::l1_action::nt_act::nt_act_trade::capability_registry` |
| 20 | `neotrix-core/src/l1_action/nt_act/nt_act_trade/trade_core.rs` | 931 | 137 | 46 | `neotrix::l1_action::nt_act::nt_act_trade::trade_core` |
| 21 | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/stage_contracts.rs` | 886 | 137 | 29 | `neotrix::l5_cognition::nt_mind::nt_mind::seal_core::self_iterating::stage_contracts` |
| 22 | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | 1,537 | 136 | 26 | `neotrix::l5_cognition::nt_mind::nt_mind_background_loop::run` |
| 23 | `neotrix-core/src/l0_substrate/nt_core_telemetry.rs` | 1,572 | 135 | 60 | `neotrix::l0_substrate::nt_core_telemetry` |
| 24 | `neotrix-core/src/l5_cognition/nt_mind/nt_game/render/components.rs` | 910 | 135 | 78 | `neotrix::l5_cognition::nt_mind::nt_game::render::components` |
| 25 | `crates/neotrix-neobot/src/nt_channel_telegram.rs` | 2,353 | 134 | 18 | `neotrix_neobot::nt_channel_telegram` |
| 26 | `neotrix-core/src/l0_substrate/nt_core_artifact_verdict.rs` | 1,562 | 134 | 66 | `neotrix::l0_substrate::nt_core_artifact_verdict` |
| 27 | `neotrix-core/src/l1_action/nt_file_ability/image_super_resolution.rs` | 1,672 | 134 | 66 | `neotrix::l1_action::nt_file_ability::image_super_resolution` |
| 28 | `neotrix-core/src/l0_substrate/nt_core_cross_layer.rs` | 1,036 | 133 | 75 | `neotrix::l0_substrate::nt_core_cross_layer` |
| 29 | `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/mod.rs` | 1,727 | 133 | 12 | `neotrix::l1_action::nt_io::nt_io_provider::gateway` |
| 30 | `crates/neotrix-gateway/src/gate.rs` | 1,314 | 127 | 56 | `neotrix_gateway::gate` |

## 维度 4 · 孤儿与异常

| 类别 | 数量 | 判据 | 含义 |
|---|---:|---|---|
| 无 modpath | 1 | 推不出 `crate::path` | 不在任何 Cargo crate 下（或路径异常）|
| 零符号 | 2 | items 为空 | 纯数据/宏/纯 impl 块，无可命名符号 |
| ⛔ 逃过层门 | 1 | `escapes-layer-gate` | **第二棵树，分层拓扑的盲区** |

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
| 全仓 | 5148 | 3529 | 230 |
| 测试目录内 | 1011 | 426 | 50 |
| **生产代码** | **4137** | **3103** | **180** |

**⛔ `AGENTS.md` / `RUST-STANDARDS.md` 明令生产代码禁这三者，但全仓无任何门或基线在度量** ⇒ 一次性历史债，存量裸奔，随时可能新增而无报警。

⇒ 建议建 `scripts/check-unwrap.sh` + 基线棘轮（只卡新增，不强求归零），与 `check-layer-deps` 的 8 条 known 同构。

### 5.4 重复类型 —— 同名 ≠ 同类型（L15 陷阱）

> ⛔ **判据是「筛子」不是「判据」** —— 2026-09-30 对抗性审计（子代理独立解析器交叉验证 + 28 个变异形态测试）得出：**353 组里 341 组（96.6%）归一化文本逐字节相同**，
> 但签名表示**看不见三类承重事实**，按它批量归并会**静默出错**：
>
> | 看不见 | 危害 | 已标记 |
> |---|---|---|
> | **派生 `Ord` 的声明序** | 两个 `Severity` 组 `derive(PartialOrd, Ord)` 且**声明序完全相反** ⇒ 合并后 `<`/`>`/`max()`/阈值**全部静默翻转**，零编译错误零测试失败 | `order_sensitive` |
> | **变体负载类型** | `X(u32)` 与 `X(f64)` 签名相同 | 需逐组读 |
> | **属性** | `#[default]` / `#[cfg]` / `#[repr]` 不可见（`RiskLevel` 5 份里两份 `#[default]` 不同） ⇒ `default()` 静默改变 | 需逐组读 |
> | **测试/函数内局部类型** | 归并做不到也不该做 | `suspect_local` |
>
> ⇒ **只有「无标记」的组才可批量归并**；enum 一律逐组读原文。
>
> 判据已修 **4 处**（每处都把数字抬高，说明历史每次都在**漏判**）：
> ① 只认无负载私有字段（`Position{f32}`≡`{f64}`）② 变体正则不认尾逗号（**所有无负载 enum 整体丢弃**）③ 签名顺序敏感 ④ **无 `{` 头部不终止，返回后面 80 行的代码**。
> ④ 是 ② 的根因：同一个缺失的守卫，在单行 enum 上表现为丢弃、在无 `{` 头上表现为**越界采集**。


| 口径 | 数量 | 含义 |
|---|---:|---|
| 重复的**类型名** | 1001 | 同名出现 ≥2 次的**名字**数 |
| 名义多余定义 | 1642 | 每名保留 1 份后余下的（**含异构**） |
| **结构完全相同**的真重复组 | **215** | 字段集合逐项相同 |
| **真正可归并的定义** | **242** | 只有这个数才叫「可归并」 |

> ⚠️ **本表数字是「候选」，不是「结论」** —— 三次判据缺陷已修（2026-09-30），每次都显著抬高数字，说明历史上每次都在**漏判**：
>
> 1. `fields()` 只认无负载私有字段 ⇒ 把 `Position{f32}` 与 `Position{f64}` 判同构
> 2. 变体正则不认尾逗号 ⇒ **所有无负载 enum 被静默丢弃**（`GoalPriority` 3 份，审计看见 0）
> 3. 签名顺序敏感 ⇒ 声明序相反的同枚举被判异构
>
> 三处修完：147 → **215** 组 / 174 → **242** 可归并。**下一个同类缺陷仍可能存在。**
>
> ⛔ **顺序敏感那条是双刃**：排序让「声明序不同」判同构了，但 enum 的**自定义 `Ord` 实现可能刻意不同于声明序**（如 `rank()`）—— 排序会把这种差异隐藏掉。
> ⇒ **每一组在归并前必须读 doc comment 判语义**，本表只负责缩小候选范围。
> 已验证的误报样例：`Position`（f32/f64，已排除）、`Output`（`Add`/`Sub`/`Mul` 的**强制**关联类型 `type Output = Self;`，不可合）、`ThreatLevel` 的 2 份组（带注释「mirrors anti_distillation for module independence」= 刻意重复）。

⇒ 1001 个同名里，**只有 242 个结构真同构**（占名义多余的 15%）。
其余是**合法的同名异构**（如 `TaskStatus` 出现 12 次却是 10 个不同枚举）—— 报原始名数会是对正确代码的误报，与 `unsafe` 字面量陷阱同一层次。

> ⚠️ 本表第一版把「名义多余 1642」误写成「可归并」并算出 160% —— **那正是本节警告的那个错误，我自己犯了一遍**。三个数已分列，逐个标明含义。

#### Top 12 真同构组（按可归并数）

| # | 类型 | kind | 字段数 | 份数 | 跨层分布 |
|---:|---|---|---:|---:|---|
| 1 | `CircuitState` | enum | 3 | 5 | l1_action, l5_cognition, l6_meta |
| 2 | `RiskLevel` | enum | 4 | 5 | l1_action, l3_embodiment |
| 3 | `Severity` | enum | 5 | 5 | l1_action, l3_embodiment |
| 4 | `StepStatus` | enum | 5 | 5 | l1_action, l5_cognition |
| 5 | `Severity` | enum | 4 | 4 | l3_embodiment |
| 6 | `ThreatLevel` | enum | 5 | 4 | l3_embodiment |
| 7 | `AwarenessReport` | struct | 6 | 3 | l0_substrate, l1_action, l5_cognition |
| 8 | `Bm25Document` | struct | 2 | 3 | l1_action, l4_emotion |
| 9 | `CapabilityGap` | struct | 5 | 3 | l0_substrate, l1_action, l5_cognition |
| 10 | `Cli` | struct | 1 | 3 | 跨 crate |
| 11 | `CrtTimeScale` | enum | 3 | 3 | l0_substrate, l5_cognition |
| 12 | `DecomposeSuggestion` | struct | 2 | 3 | l0_substrate, l1_action, l5_cognition |

**逐处位置**（`nt_locate --component <名>` 可直查）：

- `CircuitState` ×5
  - `neotrix-core/src/l1_action/nt_act/actions/core/nt_act_circuit_breaker.rs:47`
  - `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/resilience/nt_resilience_types.rs:8`
  - `neotrix-core/src/l5_cognition/nt_mind/nt_mind/evolution/goal_loop/types.rs:55`
  - `neotrix-core/src/l6_meta/healing/self_healing/circuit_breaker.rs:9`
  - `neotrix-core/src/l6_meta/nt_core_guardian/circuit_breaker.rs:29`
- `RiskLevel` ×5
  - `neotrix-core/src/agent.rs:462`
  - `neotrix-core/src/l1_action/nt_act/nt_act_trade/production_logistics.rs:314`
  - `neotrix-core/src/l1_action/nt_act/nt_act_trade/capabilities/risk_assessor.rs:47`
  - `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/mod.rs:65`
  - `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_approval/human_approval.rs:24`
- `Severity` ×5
  - `neotrix-core/src/l1_action/nt_infra_ai/inspection.rs:15`
  - `neotrix-core/src/l3_embodiment/nt_shield/vulnerability_pipeline.rs:24`
  - `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_audit/threat_modeler.rs:29`
  - `neotrix-core/src/l3_embodiment/nt_shield/scanners/container_scan/vulnerability.rs:5`
  - `neotrix-core/src/l3_embodiment/nt_shield/scanners/red_team/result.rs:6`
- `StepStatus` ×5
  - `neotrix-core/src/l1_action/nt_act/actions/orchestration/operator_runbook.rs:15`
  - `neotrix-core/src/l1_action/nt_act/actions/orchestration/provider_migration_router.rs:122`
  - `neotrix-core/src/l1_action/nt_act/nt_act_autonomy/per_agent.rs:24`
  - `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/self_evolver.rs:142`
  - `neotrix-core/src/l5_cognition/nt_core_plan/mod.rs:70`
- `Severity` ×4
  - `crates/neotrix-audit/src/nt_finding.rs:53`
  - `neotrix-core/src/l3_embodiment/nt_shield/compliance/requirement.rs:9`
  - `neotrix-core/src/l3_embodiment/nt_shield/safety/nt_safety_alignment.rs:78`
  - `neotrix-core/src/l3_embodiment/nt_shield/scanners/secret_scanner/finding.rs:12`
- `ThreatLevel` ×4
  - `neotrix-core/src/l3_embodiment/nt_shield/dual_evidence.rs:25`
  - `neotrix-core/src/l3_embodiment/nt_shield/defense/unified_defense.rs:33`
  - `neotrix-core/src/l3_embodiment/nt_shield/guard/input_gatekeeper.rs:19`
  - `neotrix-core/src/l3_embodiment/nt_shield/guard/output_sentinel.rs:20`

⚠️ **归并不是免费的**：`Severity` 散在 L1/L3 与两个 crate，合并会改公开 API 与跨层依赖方向 ⇒ 需逐组评估，不宜批量脚本化。
