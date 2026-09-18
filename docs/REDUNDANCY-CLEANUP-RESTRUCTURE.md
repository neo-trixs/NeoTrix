# NeoTrix 冗余清理 + 架构重构设计方案

> **日期**: 2026-09-18
> **基于**: 500+ URL 深度吸收 + 代码库巡检 + 耦合分析
> **目标**: 消除聚焦冗余 + 修复扁平缺陷 + 纠正跨域错位

---

## 一、巡检发现汇总

### 1.1 冗余热点 (Redundancy Hotspots)

| # | 冗余类型 | 数量 | 严重度 | 根因 |
|---|---------|------|--------|------|
| R1 | TaskType 重复定义 | **9 处** | 🔴 Critical | 各层独立定义，无单一事实源 |
| R2 | TaskStatus 重复定义 | **13 处** | 🔴 Critical | 同上 |
| R3 | ModelRouter 重复实现 | **8+ 处** | 🔴 Critical | 3 代路由器未合并 |
| R4 | CapabilityRegistry 重复 | **4 处** | 🔴 Critical | l6_meta / l5_cognition / nt_file_ability / nt_core_capability_tree |
| R5 | SelfModel 重复 | **3 处** | 🟡 High | nt_core_meta / nt_core_self / nt_core_self_model |
| R6 | CapabilityVector 引用 | **20+ 处** | 🟡 High | 定义在 L5 但全层使用 |
| R7 | LlmRequest/LlmResponse | **15+ 处** | 🟡 High | 定义分散 |
| R8 | dead_code 抑制 | **100+ 处** | 🟡 High | 系统性死代码积累 |
| R9 | SelfTest 全层引用 | **30+ 处** | 🔴 Critical | L6 定义但 L0-L5 全部依赖 |
| R10 | NodeType/RelationType | **10+ 处** | 🟡 High | L6 定义但 L1/L2 使用 |

### 1.2 跨层耦合违规 (31+ breaches)

```
Source ↓  | L0 | L1 | L2 | L3 | L4 | L5 | L6
----------|----|----|----|----|----|----|----
L0  (sub) | ✓  |    |    |    |    |    | ⚠️ L0→L6
L1  (act) | ✓  | ✓  | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ L1→L2/L3/L4/L5/L6
L2  (per) | ✓  | ✓  | ✓  | ⚠️ |    | ⚠️ | ⚠️ L2→L3/L5/L6
L3  (emb) | ✓  | ✓  | ✓  | ✓  |    |    | ⚠️ L3→L6
L4  (emo) |    |    |    |    | ✓  |    | ⚠️ L4→L6
L5  (cog) | ✓  | ✓  | ✓  | ✓  | ✓  | ✓  | ⚠️ L5→L6
L6  (met) | ✓  | ✓  | ✓  | ✓  | ✓  | ✓  | ✓
```

### 1.3 循环依赖检测

| 循环 | 路径 | 根因 |
|------|------|------|
| **L0↔L6** | L0→L6 (cache→SelfTestRegistry) | SelfTest 定义位置错误 |
| **L1↔L2** | L1→L2 (bank→knowledge), L2→L1 (via facade) | 共享类型未下沉 |
| **L1↔L5** | L1→L5 (bank→kron, dispatcher→reasoning), L5→L1 (gate→llm_provider) | 接口未抽象 |
| **L2↔L5** | L2→L5 (e8→prm, knowledge→capability_types), L5→L2 (engine_core→e8) | CapabilityVector 位置错误 |

---

## 二、重构策略

### 2.1 类型下沉 (Type Downstreaming)

**原则**: 共享类型必须下沉到最低使用层

| 类型 | 当前位置 | 目标位置 | 理由 |
|------|---------|---------|------|
| `SelfTest` trait | L6 | L0 | 全层使用，应在基础层 |
| `SelfTestRegistry` | L6 | L0 | 同上 |
| `TaskType` | L2 (neotrix-types) | L0 (neotrix-types) | 已统一，确认位置 |
| `TaskStatus` | 13 处 | L0 (neotrix-types) | 合并为 1 处 |
| `CapabilityVector` | L5 | L0 | 全层使用 |
| `LlmRequest/LlmResponse` | L1/L2 | L0 | 全层使用 |
| `NodeType/RelationType` | L6 | L0 | L1/L2 使用 |
| `KnowledgeBase` | L5 | L0 | 全层使用 |

### 2.2 接口抽象 (Interface Abstraction)

**原则**: 高层通过 trait 定义需求，低层实现

```
L5 (Cognition)
  ↓ 定义 trait LlmProvider, KnowledgeStore, CapabilityQuery
L1 (Action)
  ↓ 实现 trait LlmProviderImpl, KnowledgeStoreImpl
L0 (Substrate)
  ↓ 提供基础 trait 定义
```

### 2.3 Facade 模式 (统一入口)

**原则**: 每层只能通过 facade 访问其他层

```
L1 Facade: nt_l1_facade.rs
  - LlmProvider trait (L2 实现)
  - KnowledgeQuery trait (L5 实现)
  - SelfTest trait (L0 实现)

L2 Facade: nt_l2_facade.rs (已有)
  - VectorStore trait (L0 实现)
  - WorldModel trait (L2 内部)
```

---

## 三、执行计划

### Phase 1: 类型下沉 (Sprint 0 增强) — 2 天

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T-R1 | L6 SelfTest → L0 | 移动 trait + registry | cargo check |
| T-R2 | TaskStatus 13→1 | 合并到 neotrix-types | cargo check |
| T-R3 | CapabilityVector → L0 | 移动到 substrate | cargo check |
| T-R4 | NodeType/RelationType → L0 | 移动到 substrate | cargo check |

### Phase 2: 接口抽象 (Sprint 1 增强) — 3 天

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T-R5 | L1 Facade | 定义向上访问 trait | cargo check |
| T-R6 | L5→L1 断开 | 通过 facade trait | cargo check |
| T-R7 | L1→L2 断开 | 通过 facade trait | cargo check |
| T-R8 | L5→L6 断开 | 移除 nt_meta re-export | cargo check |

### Phase 3: 冗余清理 (Sprint 4 增强) — 2 天

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T-R9 | CapabilityRegistry 4→1 | 统一到 L0 | cargo check |
| T-R10 | SelfModel 3→1 | 统一到 L0 | cargo check |
| T-R11 | dead_code 清理 50%+ | 删除确认无消费者的项 | cargo check |
| T-R12 | L0↔L6 循环断开 | SelfTest 移动后验证 | cargo check |

---

## 四、验证矩阵

| 验证项 | 命令 | 预期 |
|--------|------|------|
| 编译检查 | `cargo check -p neotrix --lib` | 0 新增 error |
| 类型单一 | `grep -r "enum TaskType" --include="*.rs" \| wc -l` | ≤2 |
| 状态单一 | `grep -r "enum TaskStatus" --include="*.rs" \| wc -l` | ≤2 |
| 路由器单一 | `grep -r "fn route\|fn select_model" --include="*.rs" \| wc -l` | 1 |
| 耦合违规 | `grep -r "use crate::l[1-6]" neotrix-core/src/l0_substrate/ \| wc -l` | 0 |
| dead_code | `grep -r "#\[allow(dead_code)\]" --include="*.rs" \| wc -l` | <50 |
| 循环依赖 | `cargo check` (无 "cycle" error) | 0 cycles |

---

## 五、风险评估

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 类型下沉导致大量 import 变更 | 高 | 中 | 渐进式，每次只移动一个类型 |
| 接口抽象引入泛型复杂度 | 中 | 中 | 优先使用 trait object 而非泛型 |
| facade 模式增加间接层 | 中 | 低 | 仅在必要处使用 |
| 重构期间编译失败 | 高 | 高 | 每步验证，git stash 可回滚 |
