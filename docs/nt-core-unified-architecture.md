# NeoTrix — 统一架构拓扑 + 多维度程序簇

## 📐 六层架构拓扑 (L1-L6)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ L6 META-COGNITION (元认知层)                                                │
│ ┌─────────────┐ ┌──────────────┐ ┌──────────────┐ ┌─────────────────────┐ │
│ │ nt_meta     │ │ nt_repair    │ │ nt_governance│ │ coordination/       │ │
│ │ 元认知协调   │ │ 自愈修复      │ │ 架构仲裁      │ │ quality_gate        │ │
│ │             │ │              │ │              │ │ template_tag        │ │
│ └──────┬──────┘ └──────┬───────┘ └──────┬───────┘ └──────────┬──────────┘ │
│        │               │                │                     │            │
├────────┼───────────────┼────────────────┼─────────────────────┼────────────┤
│ L5 COGNITION (认知层)                                                        │
│ ┌──────▼──────┐ ┌──────▼───────┐ ┌──────▼───────┐ ┌──────────▼──────────┐ │
│ │ nt_core     │ │ nt_mind      │ │ consciousness│ │ reasoning/          │ │
│ │ 核心推理     │ │ 自我进化      │ │ 意识核心      │ │ knowledge/          │ │
│ │             │ │              │ │ (46 modules) │ │ skill_tree/         │ │
│ └──────┬──────┘ └──────┬───────┘ └──────┬───────┘ └──────────┬──────────┘ │
│        │               │                │                     │            │
├────────┼───────────────┼────────────────┼─────────────────────┼────────────┤
│ L4 EMOTION (情感层)                                                          │
│ ┌──────▼──────┐                                                               │
│ │ nt_feel     │                                                               │
│ │ 情感引擎     │                                                               │
│ │ EmotionLabel│                                                               │
│ └──────┬──────┘                                                               │
│        │                                                                      │
├────────┼──────────────────────────────────────────────────────────────────────┤
│ L3 EMBODIMENT (具身层)                                                       │
│ ┌──────▼──────┐ ┌──────────────┐ ┌──────────────┐                           │
│ │ nt_physical │ │ nt_shield    │ │ nt_feel(emb) │                           │
│ │ 传感器/执行器│ │ 安全/隐私      │ │ 情感具身      │                           │
│ │ video/armor │ │ sandbox/vault│ │ emotion_emb  │                           │
│ └──────┬──────┘ └──────┬───────┘ └──────┬───────┘                           │
│        │               │                │                                    │
├────────┼───────────────┼────────────────┼────────────────────────────────────┤
│ L2 PERCEPTION (感知层)                                                        │
│ ┌──────▼──────┐ ┌──────▼───────┐                                              │
│ │ nt_world    │ │ nt_sense     │                                              │
│ │ 世界感知     │ │ 感官处理      │                                              │
│ │ crawl/source│ │ encoder/predict│                                            │
│ └──────┬──────┘ └──────┬───────┘                                              │
│        │               │                                                      │
├────────┼───────────────┼──────────────────────────────────────────────────────┤
│ L1 ACTION (行动层)                                                            │
│ ┌──────▼──────┐ ┌──────▼───────┐ ┌──────────────┐                           │
│ │ nt_act      │ │ nt_io        │ │ nt_memory    │                           │
│ │ 工具/动作     │ │ IO/接口       │ │ 记忆          │                           │
│ │ mcp/tools   │ │ llm/cli/web  │ │ kb/embedding │                           │
│ └─────────────┘ └──────────────┘ └──────────────┘                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🔗 依赖方向规则 (Dependency Direction)

```
L6 → L5 → L4 → L3 → L2 → L1  (允许: 向下依赖)
L1 → L2 → L3 → L4 → L5 → L6  (禁止: 向上依赖, 必须通过 trait 抽象)
```

### 当前违规清单

| 违规 | 源文件 | 目标 | 修复方案 |
|------|--------|------|----------|
| L5→L1 直接导入 | `nt_mind/mod.rs:196-248` | `nt_act_trade` 类型 | 提取到 `shared_types` crate |
| L1→L5 直接导入 | `pipeline_autofixer.rs:11` | `self_diagnose` | 通过 `EvolutionLoopProvider` trait |
| L1→L5 直接导入 | `challenge.rs` | `nt_mind_benchmark` | 通过 trait 抽象 |

---

## 📊 类型统一拓扑 (Type Unification)

### 当前问题: 重复类型系统

```
TradePhase (FT01-FT17)     ← full_cycle.rs
TradePhase26 (FT01-FT26)   ← orchestrator.rs  (重复!)
TradeContext               ← full_cycle.rs
OrchTradeContext           ← orchestrator.rs  (重复!)
BuyerProfile               ← full_cycle.rs
OrchBuyerProfile           ← orchestrator.rs  (重复!)
Contract                   ← full_cycle.rs
OrchContract               ← orchestrator.rs  (重复!)
```

### 统一方案: 类型合并

```rust
// unified_trade_types.rs — 单一事实源
pub enum TradePhase {
    /// 17步完整贸易周期 (FT01-FT17)
    FullCycle(FullCyclePhase),
    /// 26步扩展编排 (FT18-FT26)
    Extended(ExtendedPhase),
}

pub struct TradeContext {
    pub base: TradeContextBase,           // 共享基础字段
    pub orchestrator: Option<OrchestratorMeta>, // 编排器元数据 (可选)
}

pub struct BuyerProfile {
    pub base: BuyerProfileBase,
    pub segments: Vec<MarketSegment>,
}

pub struct Contract {
    pub parties: (String, String),        // 买方, 卖方
    pub items: Vec<ContractItem>,
    pub terms: ContractTerms,
    pub metadata: Option<ContractMetadata>, // 扩展元数据
}
```

---

## 🗂️ 模块拓扑 (Module Topology)

### 快速定位索引

| 功能域 | 模块路径 | 入口文件 | 关键类型 |
|--------|----------|----------|----------|
| **意识核心** | `neotrix/nt_consciousness_core/` | `mod.rs` | `IterationAgent`, `MemoryKernel`, `Constitution` |
| **贸易引擎** | `l1_action/nt_act/nt_act_trade/` | `mod.rs` | `TradePhase`, `TradeContext`, `Contract` |
| **代码生成** | `l1_action/nt_act/nt_act_code/` | `mod.rs` | `SelfCodeWriter`, `ActionPlan` |
| **记忆系统** | `l1_action/nt_memory/` | `mod.rs` | `KnowledgeBase`, `Embedding` |
| **LLM路由** | `l1_action/nt_io/` | `mod.rs` | `Provider`, `ModelRouter` |
| **世界感知** | `l2_perception/nt_world/` | `mod.rs` | `Crawler`, `Source` |
| **安全防护** | `l3_embodiment/nt_shield/` | `mod.rs` | `Sandbox`, `Vault`, `Guard` |
| **情感引擎** | `l4_emotion/nt_feel/` | `mod.rs` | `EmotionEngine`, `EmotionLabel` |
| **核心推理** | `l5_cognition/nt_core/` | `mod.rs` | `E8`, `GWT`, `HyperCube` |
| **自我进化** | `l5_cognition/nt_mind/` | `mod.rs` | `SEAL`, `SkillTree`, `Evolution` |
| **元认知** | `l6_meta/` | `mod.rs` | `ConsciousnessTree`, `Governance` |

### 模块深度索引

```
neotrix-core/src/
├── l1_action/                          # L1 行动层
│   ├── traits.rs                       # ActionLayer trait
│   ├── nt_act/                         # 工具/动作
│   │   ├── nt_act_trade/               # 贸易引擎
│   │   │   ├── mod.rs                  # 入口 + re-exports
│   │   │   ├── trade_core.rs           # 核心算法 (StateMachine, CostCalculator)
│   │   │   ├── full_cycle.rs           # 17步完整周期
│   │   │   ├── orchestrator.rs         # 26步扩展编排
│   │   │   ├── finance_compliance.rs   # 金融合规
│   │   │   └── production_logistics.rs # 生产物流
│   │   ├── nt_act_code/                # 代码生成
│   │   │   ├── code_writer.rs          # 核心写入器
│   │   │   ├── template_registry.rs    # 模板注册
│   │   │   └── safe_applier.rs         # 安全应用
│   │   └── ...
│   ├── nt_io/                          # IO/接口
│   │   ├── nt_io_provider/             # LLM提供者
│   │   └── ...
│   └── nt_memory/                      # 记忆
│       ├── nt_memory_kb/               # 知识库
│       └── ...
├── l2_perception/                      # L2 感知层
│   ├── traits.rs                       # PerceptionLayer trait
│   ├── nt_world/                       # 世界感知
│   │   ├── crawl/                      # 爬虫
│   │   ├── source/                     # 数据源
│   │   └── sense/                      # 感官处理
│   └── nt_sense/                       # 感官
├── l3_embodiment/                      # L3 具身层
│   ├── traits.rs                       # EmbodimentLayer trait
│   ├── nt_physical/                    # 物理
│   ├── nt_shield/                      # 安全
│   └── nt_feel/                        # 情感具身
├── l4_emotion/                         # L4 情感层
│   ├── traits.rs                       # EmotionLayer trait
│   └── nt_feel/                        # 情感引擎
├── l5_cognition/                       # L5 认知层
│   ├── traits.rs                       # CognitionLayer trait
│   ├── nt_core/                        # 核心推理
│   │   ├── core_capability.rs          # 能力注册
│   │   ├── nt_core_model_router.rs     # 模型路由
│   │   ├── reasoning/                  # 推理引擎
│   │   ├── knowledge/                  # 知识管理
│   │   └── ...
│   └── nt_mind/                        # 自我进化
│       ├── mod.rs                      # 入口 + re-exports
│       ├── evolution/                  # 进化循环
│       ├── seal_core/                  # SEAL核心
│       ├── skill_tree/                 # 技能树
│       └── ...
├── l6_meta/                            # L6 元认知层
│   ├── traits.rs                       # MetaLayer trait
│   ├── coordination/                   # 协调
│   ├── evolution/                      # 进化
│   ├── healing/                        # 自愈
│   └── memory/                         # 记忆管理
├── neotrix/                            # Neotrix根模块
│   ├── mod.rs                          # 入口
│   ├── nt_consciousness_core/          # 意识核心 (46 modules)
│   │   ├── mod.rs                      # 模块声明 + re-exports
│   │   ├── memory_kernel.rs            # 双记忆机制
│   │   ├── cas_store.rs                # CAS存储
│   │   ├── bitemporal_graph.rs         # 双时态图
│   │   ├── constitution.rs             # 宪法门控
│   │   ├── skill_crystallizer.rs       # 技能结晶
│   │   └── ... (46 files total)
│   ├── ffi/                            # FFI接口
│   └── ...
└── core/                               # 核心模块
    ├── nt_core_capability/             # 能力系统
    └── ...
```

---

## 🔧 冗余清理清单 (Redundancy Cleanup)

### 1. 重复类型合并

| 重复类型A | 重复类型B | 合并方案 | 优先级 |
|-----------|-----------|----------|--------|
| `TradePhase` | `TradePhase26` | 合并为 `TradePhase { FullCycle, Extended }` | P0 |
| `TradeContext` | `OrchTradeContext` | 合并为 `TradeContext { base, orchestrator? }` | P0 |
| `BuyerProfile` | `OrchBuyerProfile` | 合并为 `BuyerProfile { base, segments }` | P0 |
| `Contract` | `OrchContract` | 合并为 `Contract { parties, items, terms, metadata? }` | P0 |
| `CapabilityError` (L1) | `CapabilityError` (L5) | 统一到 `shared_types` crate | P0 |
| `VoiceConfig` (L4) | `VoiceConfig` (vtuber) | 统一到 L4 traits | P1 |

### 2. 缓存系统合并

| 缓存实现 | 位置 | 合并方案 |
|----------|------|----------|
| `nt_act_cache` | L1 Action | 统一到 `CacheService` trait |
| `prompt_cache` | L5 Cognition | 同上 |
| `video_prompt_cache` | L5 Cognition | 同上 |
| `spatial_cache` | L1 Memory | 同上 |
| `media_cache` | Unified Archive | 同上 |

### 3. 事件系统合并

| 事件实现 | 位置 | 合并方案 |
|----------|------|----------|
| `MetaEvent` | L6 Meta | 统一 EventBus |
| `SecurityEvent` | L3 Shield | 同上 |
| `AbsorptionEvent` | L5 Mind | 同上 |
| `HookEvent` | L5 Mind | 同上 |
| 10+ 其他 | 各层 | 同上 |

### 4. 错误类型合并

| 错误类型 | 位置 | 合并方案 |
|----------|------|----------|
| `CapabilityError` | L1 + L5 | 统一到 `nt_core_error` |
| `AgentError` | L5 多处 | 同上 |
| 70+ 模块错误 | 各模块 | 分层: `thiserror`(库) + `anyhow`(应用) |

---

## 📏 扁平化清单 (Flattening)

### 深层嵌套修复

| 当前路径 | 问题 | 修复方案 |
|----------|------|----------|
| `l4_emotion/nt_feel/nt_feel/nt_feel/` | 三层嵌套 | 扁平化为 `l4_emotion/nt_feel/` |
| `l3_embodiment/nt_shield/nt_shield/` | 双层嵌套 | 扁平化为 `l3_embodiment/nt_shield/` |
| `l1_action/nt_memory/nt_memory_kb/ntx/` | 深层KB | 扁平化为 `l1_action/nt_memory/kb/` |

---

## 🎯 跨域错位修复 (Cross-Domain Misalignment)

### 依赖方向修复

| 违规 | 当前 | 修复方案 |
|------|------|----------|
| L5→L1 直接导入 | `nt_mind` 导入 `nt_act_trade` 类型 | 提取到 `shared_types` crate |
| L1→L5 直接导入 | `pipeline_autofixer` 导入 `self_diagnose` | 通过 `EvolutionLoopProvider` trait |
| L1→L5 直接导入 | `challenge.rs` 导入 `nt_mind_benchmark` | 通过 trait 抽象 |

### 类型边界修复

| 泄漏 | 当前 | 修复方案 |
|------|------|----------|
| L1 类型重导出到 L5 | `nt_mind/mod.rs` 重导出 `TradePhase26` 等 | 删除重导出, 使用 `shared_types` |
| 测试类型污染生产代码 | `finance_compliance.rs` 测试使用 `ContractParties` | 修复测试使用正确类型 |

---

## 📈 实施优先级

```
Week 1: 类型统一 (P0)
├── 合并 TradePhase/TradePhase26
├── 合并 TradeContext/OrchTradeContext
├── 合并 BuyerProfile/OrchBuyerProfile
├── 合并 Contract/OrchContract
└── 统一 CapabilityError

Week 2: 缓存/配置合并 (P1)
├── 创建 CacheService trait
├── 统一 CacheConfig
└── 合并缓存实现

Week 3: 事件系统合并 (P1)
├── 创建统一 EventBus
├── 迁移 MetaEvent/SecurityEvent
└── 统一事件发布/订阅

Week 4: 架构对齐 (P2)
├── 修复依赖方向违规
├── 扁平化深层嵌套
├── 清理类型边界
└── 更新文档
```

---

**文档版本**: v1.0
**创建时间**: 2026-09-09
**状态**: 可执行
