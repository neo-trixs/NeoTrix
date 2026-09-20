# NeoTrix 架构熔炼 — 核心路线任务清单

> 基于 10 个外部仓库技术逆向 + 全量架构审计 + 6 维度技术搜索
> 生成时间: 2026-09-20 | 审计范围: 2,383 files / 789K LOC

---

## 一、现状诊断

### 审计数据总览

| 维度 | 发现 | 严重度 |
|------|------|--------|
| 重复类型定义 | 40+ structs 重复 5-14 次 | HIGH |
| TODO/FIXME/STUB | 227 项 | HIGH |
| 孤儿模块声明 | 23 个 pub mod 指向不存在的文件 | MEDIUM |
| dead_code 抑制 | 90+ `#[allow(dead_code)]` | MEDIUM |
| 跨层违规 | L0→L5/L6 (CRITICAL), L1↔L2 (HIGH) | CRITICAL |
| 层级失衡 | L5 占 28.1% (最大), L4 仅 0.8% (最小) | MEDIUM |
| stub/no-op 函数 | 46 个空实现 | HIGH |
| facade 未完全采纳 | L2→L1 仅 15/26 文件走 l1_facade | MEDIUM |

### 跨层违规详情

| 违规方向 | 文件数 | 严重度 |
|----------|--------|--------|
| L0 → L5 (init.rs 引用 L7OrchestratorRegistry) | 1 | CRITICAL |
| L0 → L6 (agent 引用 UnifiedCapability) | 3 | CRITICAL |
| L1 → L3+ (IO 引用认知类型) | 15 | HIGH |
| L1 → L2 (直接绕过 facade) | 32 | MEDIUM |
| L2 → L3+ (感知引用治理) | 15 | HIGH |
| L2 → L1 (未走 facade) | 11 | MEDIUM |

### 重复类型 Top 10

| 类型 | 重复次数 | 应归位置 |
|------|----------|----------|
| TaskStatus (enum) | 14 | L0 / neotrix-types |
| SearchResult | 13 (7 struct + 6 alias) | L0 trait + 适配器 |
| MemoryEntry | 11 | L0 / neotrix-types |
| CircuitBreaker | 9 (7 struct + 2 alias) | L0 trait |
| GraphEdge | 8 | L0 / neotrix-types |
| CacheStats | 8 | L0 |
| AuditEntry | 8 | L0 |
| ValidationResult | 7 | L0 / neotrix-types |
| ScanReport | 7 | L0 |
| PerformanceMetrics | 7 | L0 |

---

## 二、外部仓库吸收矩阵

### P0 — 立即吸收 (核心架构级)

| 仓库 | 模式 | 吸收目标 | 核心价值 |
|------|------|----------|----------|
| **Agent-Reach** (83.5K★) | 能力层 + 多后端路由 | `nt_ability_net` | 统一外部服务路由架构 |
| **Jev MCP** (TypeSafe) | 类型化判断原语 | `nt_judgment` | 低成本概率判断层 (150-500ms) |
| **SoL-Pi** (NVIDIA) | 4 个效率机制 | `nt_harness` | 44-49% token 压缩 |
| **Memory Hierarchy** | 五层记忆级联 | `nt_memory` | 从扁平日志→分层记忆 |

### P1 — 本周吸收 (高价值)

| 仓库 | 模式 | 吸收目标 | 核心价值 |
|------|------|----------|----------|
| **Scrapling** (82.5K★) | 自适应选择器 + 分层抓取 | `nt_web_perception` | 自愈式网页采集 |
| **DeepTeam** (2.9K★) | 攻击-漏洞-护栏三元组 | `nt_governance` | 50+ 漏洞类型红队测试 |
| **Jev-cu** (318) | 观察-决策-执行循环 | `nt_agent` | 带策略门的 agent 执行 |
| **ARES** (259) | DAG 求解 + 范围防火墙 | `des/architect` | 目标导向任务规划 |
| **Hermes Agent** (5.7K★) | 分层架构 + 信任边界 | 全层参考 | 架构设计参考 |
| **Rust Agent 框架** | Tool trait + 4层上下文 | `nt_runtime` | Rust 原生执行模式 |

### P2 — 下周吸收 (增强)

| 仓库 | 模式 | 吸收目标 | 核心价值 |
|------|------|----------|----------|
| **AntV Infographic** (6.7K★) | 声明式渲染 + 模板 | `nt_presentation` | 数据可视化引擎 |
| **Patchright Enhanced** | 反检测补丁策略 | 参考 | 浏览器反检测策略 |

---

## 三、核心路线任务清单

### Sprint 1: 地基修复 (Day 1-2) — 消除架构债务

#### T1.1 [CRITICAL] L0 上行依赖彻底清除
- **现状**: L0 引用 L5/L6 共 4 个文件
- **目标**: L0 零上行依赖
- **动作**:
  1. `Domain`, `Layer`, `UnifiedCapability` 从 L6 移至 `neotrix-types` (L0 可依赖)
  2. `nt_core_platform/init.rs` 移除 L7OrchestratorRegistry 引用
  3. `nt_core_platform/agent.rs` 移除 UnifiedCapability 引用
  4. `nt_core_cross_layer.rs` 的 L6 类型改从 neotrix-types 引入
- **验证**: `cargo check` 0 errors + 无 L0→L5/L6 import
- **预估**: 2h

#### T1.2 [HIGH] 核心类型统一 — 消除重复
- **现状**: 40+ 重复类型
- **目标**: 每类 1 个规范定义 + From 适配器
- **动作**:
  1. `neotrix-types/src/core/nt_core_knowledge/types.rs` 扩充:
     - `TaskStatus` 枚举 (合并 14 个版本)
     - `SearchResult` trait (合并 13 个版本)
     - `MemoryEntry` (合并 11 个版本)
     - `GraphEdge`, `GraphNode` (合并 8 个版本)
     - `CircuitBreaker` trait (合并 9 个版本)
  2. 各模块添加 `impl From<XxxLocal> for XxxCanonical`
  3. 逐步替换 import 路径
- **验证**: 无同名 struct 重复定义
- **预估**: 4h

#### T1.3 [HIGH] L1↔L2 循环依赖打破
- **现状**: L2→L1 仅 15/26 走 facade，L1→L2 有 32 直接引用
- **目标**: L2→L1 100% 走 facade；L1→L2 仅限 L0 类型
- **动作**:
  1. L2 剩余 11 文件改用 `l1_facade.rs`
  2. L1 中 `TaskType` 改从 neotrix-types 引入
  3. L1 中 LLM 类型创建 `nt_llm_types` trait 边界
- **验证**: `grep -rn "use crate::l2_" neotrix-core/src/l1_*/` 结果 ≤5
- **预估**: 3h

#### T1.4 [MEDIUM] 孤儿模块清理
- **现状**: 23 个 `pub mod` 指向不存在的文件
- **目标**: 0 孤儿声明
- **动作**: 逐一检查并移除或补全
- **验证**: 无编译警告
- **预估**: 1h

### Sprint 2: 能力吸收 (Day 3-5) — 外部模式融入

#### T2.1 [P0] nt_judgment — 类型化判断原语 (Jev MCP)
- **输入**: 10 个判断工具 (verify, screen, find, rerank, classify, decide, compare, extract, review, gate)
- **目标模块**: `l2_perception/nt_judgment/`
- **核心类型**:
  ```rust
  pub trait JudgmentPrimitive {
      fn judge(&self, input: &JudgmentInput) -> Result<JudgmentOutput>;
  }
  pub struct Verdict { pub result: Verified | Contradicted | Unsupported, pub confidence: f64 }
  pub struct JudgmentOutput { pub verdict: Verdict, pub reasoning: String, pub latency_ms: u64 }
  ```
- **消费方**: rev-officer (D4-D15), meta-coordinator, capability-net
- **预估**: 3h

#### T2.2 [P0] 五层记忆级联 (Memory Hierarchy)
- **输入**: Sensory → Working → Short-term → Episodic → Long-term
- **目标模块**: `l1_action/nt_memory/cascade/`
- **核心类型**:
  ```rust
  pub trait MemoryTier {
      fn store(&mut self, entry: MemoryEntry) -> Result<()>;
      fn recall(&self, query: &str, limit: usize) -> Vec<MemoryEntry>;
      fn promote(&mut self, id: &str) -> Result<()>;
      fn decay(&mut self, now: Instant) -> usize; // returns items removed
  }
  pub struct MemoryCascade {
      sensory: SensoryBuffer,      // <1min, raw
      working: WorkingMemory,      // ≤7 items, GWT-gated
      short_term: ShortTermStore,  // 1-7d, verbatim
      episodic: EpisodicStore,     // 5-10x compressed summaries
      long_term: LongTermStore,    // distilled rules
  }
  ```
- **预估**: 4h

#### T2.3 [P0] nt_harness — SoL-Pi 效率机制
- **输入**: Action Fusion + ObservationPack + Evidence-Preserving Reducer + Online Context Compact
- **目标模块**: `l1_action/nt_harness/`
- **核心类型**:
  ```rust
  pub trait EfficiencyMechanism {
      fn name(&self) -> &str;
      fn should_activate(&self, ctx: &Context) -> bool;
      fn apply(&self, ctx: &mut Context) -> Result<()>;
  }
  pub struct ActionFusion;      // 合并 follow-up validation 与 write
  pub struct ObservationPack;   // 大输出→稳定句柄 + 分页召回
  pub struct EvidencePreservingReducer; // 日志压缩，保留引用链
  pub struct OnlineContextCompact;      // 子任务边界触发压缩
  ```
- **预估**: 3h

#### T2.4 [P1] nt_ability_net — 多后端路由 (Agent-Reach)
- **输入**: 平台→后端有序列表 + health probe + fallback
- **目标模块**: `l1_action/nt_ability_net/` (增强现有)
- **核心模式**: 平台模块化 + 后端优先级 + 运行时健康检查
- **预估**: 3h

#### T2.5 [P1] nt_governance — 红队测试 (DeepTeam)
- **输入**: 50+ 漏洞类型 + 20+ 攻击方法 + OWASP/NIST/MITRE 映射
- **目标**: `l6_meta/nt_governance/red_team/`
- **预估**: 2h

#### T2.6 [P1] nt_agent — 观察-决策-执行循环 (Jev-cu)
- **输入**: AX 解析 → 候选选择 → 决策 → 策略门 → 执行
- **目标**: `l5_cognition/nt_agent/observe_decide_execute/`
- **预估**: 2h

### Sprint 3: 冗余清理 (Day 6-7) — 瘦身 + 质量

#### T3.1 stub 函数清理
- **现状**: 46 个 no-op stub
- **动作**: 实现或移除，不留空壳
- **预估**: 2h

#### T3.2 dead_code 全局抑制移除
- **现状**: lib.rs:36 `#![allow(dead_code)]` 全局抑制
- **动作**: 移除全局 allow，逐个处理 90+ 注解
- **预估**: 2h

#### T3.3 层级体重再平衡
- **现状**: L5 (28.1%) >> L4 (0.8%)
- **动作**: 将 L5 中记忆/情感相关模块下沉至 L4
- **预估**: 3h

#### T3.4 facade 模式强制化
- **动作**: 添加 CI 检查，L2→L1 必须走 l1_facade
- **预估**: 1h

### Sprint 4: 集成验证 (Day 8-10) — 全量回归

#### T4.1 全量编译回归
- `cargo clean && cargo build` 0 errors, 0 warnings
- **预估**: 1h

#### T4.2 单元测试回归
- `cargo test -p neotrix --lib` 全部通过
- **预估**: 2h

#### T4.3 架构约束验证脚本
- 编写 `scripts/check_architecture.sh`:
  - 检查 L0 零上行依赖
  - 检查 L2→L1 facade 使用率 = 100%
  - 检查无同名类型重复
  - 检查孤儿模块 = 0
- **预估**: 2h

#### T4.4 多 agent 自动巡检
- rev-officer D1-D51 维度巡检
- 代码质量 gate (clippy + deny(warnings))
- 架构合规 gate (上面的脚本)
- **预估**: 1h

---

## 四、融合架构设计

### 目标架构 (Sprint 4 之后)

```
L6 Meta ─────────────────────────────────────
  nt_governance (red team, compliance)
  nt_meta (dream replay, evolution)
  consciousness (orchestrator)
  ┌─────────────────────────────────────────┐
  │ nt_core_cross_layer.rs (L6→L0 re-exports)│
  └─────────────────────────────────────────┘

L5 Cognition ───────────────────────────────
  nt_mind (cost ladder, evidence gating)
  nt_core_gwt (decision layer)
  nt_agent (observe-decide-execute)
  rev-officer, methodology-researcher
  ┌─────────────────────────────────────────┐
  │ 只依赖 L4 (emotion) + L3 (embodiment)  │
  └─────────────────────────────────────────┘

L4 Emotion ─────────────────────────────────
  emotion_engine
  ┌─────────────────────────────────────────┐
  │ 只依赖 L3 (embodiment)                  │
  │ 新增: memory cascade 从 L5 下沉至此     │
  └─────────────────────────────────────────┘

L3 Embodiment ──────────────────────────────
  shield, compliance, physical sim
  ┌─────────────────────────────────────────┐
  │ 只依赖 L2 (perception)                  │
  └─────────────────────────────────────────┘

L2 Perception ──────────────────────────────
  nt_world (OSINT, crawl, knowledge)
  nt_judgment (P0 新增, Jev MCP 模式)
  nt_web_perception (P1 新增, Scrapling)
  ┌─────────────────────────────────────────┐
  │ 100% L1→L2 走 l1_facade               │
  └─────────────────────────────────────────┘

L1 Action ──────────────────────────────────
  nt_memory (五层级联, P0 新增)
  nt_harness (SoL-Pi 效率, P0 新增)
  nt_ability_net (多后端路由, P0 增强)
  nt_command, nt_io, nt_media
  ┌─────────────────────────────────────────┐
  │ 只依赖 L0 (substrate)                   │
  └─────────────────────────────────────────┘

L0 Substrate ───────────────────────────────
  nt_core_types (统一核心类型, P0 扩充)
  nt_core_traits (基础 trait)
  nt_core_error (统一错误)
  nt_core_cross_layer (bridge)
  ┌─────────────────────────────────────────┐
  │ 零上行依赖 — 唯一的基础层              │
  └─────────────────────────────────────────┘
```

### 关键设计决策

1. **判断原语 (nt_judgment)**: Jev MCP 的 10 个工具抽象为 `JudgmentPrimitive` trait，消费方通过 trait object 调用，不直接依赖具体模型实现
2. **记忆级联**: 五层自动晋升/衰减，`MemoryCascade` 作为 L1 核心组件，L2-L6 通过 `l1_facade` 访问
3. **效率机制**: SoL-Pi 4 个机制实现 `EfficiencyMechanism` trait，按需启用
4. **多后端路由**: Agent-Reach 的 channel 模式变为 `nt_ability_net` 的平台→后端有序列表 + 运行时 health probe

---

## 五、成功标准

| 指标 | 当前 | Sprint 4 目标 |
|------|------|---------------|
| L0 上行依赖 | 4 files | **0** |
| 重复类型 | 40+ | **≤5** |
| L2→L1 facade 使用率 | 57% (15/26) | **100%** |
| 孤儿模块 | 23 | **0** |
| stub 函数 | 46 | **0** |
| TODO/FIXME | 227 | **≤50** |
| 编译错误 | 0 | **0** |
| 编译警告 | ~30 | **0** |
| 新吸收模块 | 0 | **6** (judgment, memory cascade, harness, ability_net, red_team, observe_decide_execute) |
| 外部模式融合 | 0 | **6** (Jev, SoL-Pi, Memory, Agent-Reach, DeepTeam, Jev-cu) |
