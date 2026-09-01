# ADR-001: 域重组架构决策记录

## 状态

**提议** (Proposed)

## 日期

2026-08-31

## 背景

NeoTrix 项目当前存在三套并行的模块组织系统，导致架构混乱和维护困难。

### 当前状态

#### 系统 1: core/ 中的基础层 (10个层级)

```
neotrix-core/src/core/
├── l0_substrate/      # 基础层
├── l1_body/           # 身体层
├── l2_perception/     # 感知层
├── l3_memory/         # 记忆层
├── l4_cognition/      # 认知层
├── l5_consciousness/  # 意识层
├── l6_self/           # 自我层
├── l7_capability/     # 能力层
├── l8_autonomic/      # 自主层
└── l9_transcendent/   # 超越层
```

#### 系统 2: neotrix/ 中的实现层 (10个层级)

```
neotrix-core/src/neotrix/
├── l1_body_impl/      # 身体实现
├── l2_world_impl/     # 世界实现
├── l3_memory_impl/    # 记忆实现
├── l4_cognition_impl/ # 认知实现
├── l5_consciousness_impl/ # 意识实现
├── l6_self_impl/      # 自我实现
├── l7_capability_impl/ # 能力实现
├── l8_autonomic_impl/ # 自主实现
├── l9_transcendent_impl/ # 超越实现
└── l10_transcendent_impl/ # 超越实现
```

#### 系统 3: 外部模块 (NT-* 域)

```
neotrix-core/src/neotrix/
├── nt_act/            # NT-ACT 域
├── nt_core_capability_tree/ # 能力树
├── nt_file_ability/   # 文件能力
├── nt_harness/        # 工具
└── nt_shanhai_geo/    # 地理
```

### 问题分析

#### 问题 1: 三套系统没有清晰映射

| core/ 层级 | neotrix/ 层级 | NT-* 域 | 映射关系 |
|-----------|--------------|---------|---------|
| l0_substrate | (无) | (无) | 不明确 |
| l1_body | l1_body_impl | NT-PHYSICAL | 部分对应 |
| l2_perception | l2_world_impl | NT-WORLD | 部分对应 |
| l3_memory | l3_memory_impl | NT-MEMORY | 部分对应 |
| l4_cognition | l4_cognition_impl | NT-CORE | 部分对应 |
| l5_consciousness | l5_consciousness_impl | NT-CORE, NT-FEEL | 部分对应 |
| l6_self | l6_self_impl | NT-ACT (混乱) | 不对应 |
| l7_capability | l7_capability_impl | NT-IO, NT-GOVERNANCE | 不对应 |
| l8_autonomic | l8_autonomic_impl | NT-MIND, NT-IO | 部分对应 |
| l9_transcendent | l9_transcendent_impl | NT-REPAIR | 部分对应 |
| (无) | l10_transcendent_impl | NT-NEXUS | 不对应 |

#### 问题 2: 层级编号不一致

- core/ 从 l0 开始
- neotrix/ 从 l1 开始
- neotrix/ 有 l10, core/ 没有

#### 问题 3: 职责划分不清

- `core/l1_body` vs `neotrix/l1_body_impl` 有什么区别？
- `core/l3_memory` vs `neotrix/l3_memory_impl` 有什么区别？
- 没有文档说明这种分层的原因

#### 问题 4: 模块放置混乱

| 域 | 分散位置 | 数量 |
|----|---------|------|
| NT-ACT | l1_body_impl, l6_self_impl, nt_act/ | 3处 |
| NT-IO | l7_capability_impl, l8_autonomic_impl | 2处 |
| NT-CORE | l4_cognition_impl, l5_consciousness_impl, core/ | 3处 |
| NT-MIND | l8_autonomic_impl | 1处 |
| NT-MEMORY | l3_memory_impl | 1处 |
| NT-WORLD | l2_world_impl | 1处 |

## 决策

### 选项分析

#### 选项 A: 按域重组 (推荐)

**描述**: 删除所有 l*_impl 层级目录，每个 NT-* 域一个目录。

**优点**:
- 清晰的 1:1 映射
- 易于理解和维护
- 符合领域驱动设计原则

**缺点**:
- 需要移动大量文件 (100+)
- 需要更新大量 use 语句
- 风险较高

**实施步骤**:
1. 创建目标目录结构
2. 移动模块文件
3. 更新 mod.rs 声明
4. 更新 use 语句
5. 验证构建
6. 运行测试
7. 更新文档

#### 选项 B: 保持两层结构

**描述**: core/ = 基础类型/接口，neotrix/ = 实现，明确映射关系。

**优点**:
- 保持现有结构
- 只需要文档更新
- 风险较低

**缺点**:
- 仍然存在两套系统
- 映射关系可能不清晰
- 维护成本较高

#### 选项 C: 仅文档更新

**描述**: 不移动文件，只更新文档说明实际结构。

**优点**:
- 零风险
- 立即实施
- 无破坏性变更

**缺点**:
- 不解决根本问题
- 架构混乱持续
- 新开发者难以理解

### 决策结果

**选择: 选项 A (按域重组)**

**理由**:
1. 清晰的架构是长期维护的基础
2. 当前混乱已经影响开发效率
3. 分阶段实施可以降低风险

## 实施计划

### 阶段 1: 准备 (1-2天)

1. 创建详细的模块映射表
2. 备份当前代码
3. 创建目标目录结构

### 阶段 2: 核心域迁移 (3-5天)

1. NT-CORE 域 (最核心)
2. NT-MIND 域 (重要)
3. NT-MEMORY 域 (重要)

### 阶段 3: 功能域迁移 (3-5天)

1. NT-WORLD 域
2. NT-ACT 域
3. NT-IO 域

### 阶段 4: 支撑域迁移 (2-3天)

1. NT-SHIELD 域
2. NT-PHYSICAL 域
3. NT-FEEL 域

### 阶段 5: 治理域迁移 (1-2天)

1. NT-META 域
2. NT-REPAIR 域
3. NT-GOVERNANCE 域
4. NT-NEXUS 域

### 阶段 6: 清理 (1-2天)

1. 删除空目录
2. 更新所有文档
3. 运行完整测试

## 风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 构建失败 | 高 | 每次移动后验证构建 |
| 测试失败 | 高 | 运行完整测试套件 |
| 依赖断裂 | 中 | 使用 IDE 重构工具 |
| 性能回归 | 低 | 基准测试 |

## 验证

### 构建验证

```sh
cargo check -p neotrix --lib
cargo check -p neotrix --all-targets
```

### 测试验证

```sh
cargo test -p neotrix --lib
cargo test -p neotrix --integration
```

### 文档验证

- [ ] CONTEXT.md 更新
- [ ] AGENTS.md 更新
- [ ] 架构图更新
- [ ] ADR 文档完成

## 后续行动

1. **立即**: 创建详细的模块映射表
2. **本周**: 开始阶段 1 (准备)
3. **下周**: 开始阶段 2 (核心域迁移)
4. **本月**: 完成所有阶段

## 参考

- [领域驱动设计](https://martinfowler.com/bliki/BoundedContext.html)
- [架构决策记录](https://adr.github.io/)
- [NeoTrix 架构文档](./2026-08-31-unified-consciousness-embodiment-architecture.md)
