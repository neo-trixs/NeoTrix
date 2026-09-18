# NeoTrix 完整迁移路线图 v3.0 — 最终优化版

> **日期**: 2026-09-18
> **基于**: 项目结构分析 + GitHub Copilot/Perplexity/Bun 迁移实践 + CTM-AI 架构 + MAGMA 记忆架构 + Cargo Workspace 最佳实践
> **目标**: 从 L0-L6 层级架构迁移到 Crystal-Radiant 辐射架构 (最优解)

---

## 一、项目现状诊断 (精确量化)

### 1.1 代码库统计

| 指标 | 数值 | 分类 |
|------|------|------|
| 总 .rs 文件 | **2047** | 1843 (L0-L6) + 204 (其他) |
| 总 .md 文件 | **2903** | 666 迭代批次 + 2237 其他 |
| Workspace Crates | **10** | 需要统一到 `crates/` 目录 |
| 根目录文件 | **~100+** | 过于杂乱，需要清理 |
| 循环依赖 | **31+** | 需要修复 |
| dead_code 注解 | **87** | 需要清理 |
| TaskType 重复 | **9** | 需要统一 |
| TaskStatus 重复 | **13** | 需要统一 |

### 1.2 模块迁移复杂度矩阵

| 模块 | 旧位置 | 文件数 | 迁移难度 | 阻塞点 | 推荐顺序 |
|------|--------|--------|----------|--------|---------|
| Crystal Core | nt_crystal_core | 28 | ⭐ | 无 | Phase 1 |
| NT-FEEL | l4_emotion | 11 | ⭐ | 无 | Phase 1 |
| NT-META | l6_meta (部分) | 90 | ⭐⭐ | 无 | Phase 1 |
| NT-SHIELD | l3_embodiment | 201 | ⭐⭐⭐ | SelfTest 依赖 | Phase 2 |
| NT-MEMORY | l1_action | 120 | ⭐⭐⭐ | L0/L2 类型依赖 | Phase 2 |
| NT-ACT | l1_action | 200 | ⭐⭐⭐ | L5 推理依赖 | Phase 2 |
| NT-WORLD | l2_perception | 289 | ⭐⭐⭐⭐ | L5 能力向量依赖 | Phase 3 |
| NT-MIND | l5_cognition | 400 | ⭐⭐⭐⭐⭐ | L6 自我模型依赖 | Phase 3 |
| NT-IO | l1_action | 300 | ⭐⭐⭐⭐ | L1/L5 接口依赖 | Phase 3 |

---

## 二、互联网最佳实践吸收 (最终版)

### 2.1 迁移策略 (来源: GitHub Copilot + Perplexity + Bun)

| 策略 | 来源 | 适用场景 | NeoTrix 应用 |
|------|------|---------|-------------|
| **Incremental Shadow** | GitHub Copilot | 大项目 | ✅ 主策略 |
| **Goal-Verifier-Workspace** | Perplexity | 每个任务 | ✅ 验证框架 |
| **Test Suite as Safety Net** | Bun | 迁移前准备 | ✅ Phase 0 |
| **Start from Leaves** | Mainmatter | 依赖分析 | ✅ Phase 1 |
| **PORTING.md Contract** | Bun | 迁移契约 | ✅ 每模块 |

### 2.2 记忆架构 (来源: MAGMA + Knowledge Graph Patterns)

```
NeoTrix 记忆架构 (对标 MAGMA 四图):

┌─────────────────────────────────────────────────────────────┐
│                    NT-MEMORY 辐射臂                          │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  L1: Working Memory (STM, 容量=1)                       │ │
│  │  - CrystalConsciousness.workspace                        │ │
│  │  - 当前活跃 chunk                                        │ │
│  └─────────────────────────────────────────────────────────┘ │
│                          ↓                                    │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  L2: Episodic Memory (Vector Store)                     │ │
│  │  - 事件序列 (时间绑定)                                   │ │
│  │  - 情境回忆 (语义相似)                                   │ │
│  │  - 检索方式: 近似最近邻 (ANN)                            │ │
│  └─────────────────────────────────────────────────────────┘ │
│                          ↓                                    │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  L3: Semantic Memory (SQL/Graph)                        │ │
│  │  - 实体-关系三元组 (Knowledge Graph)                     │ │
│  │  - 因果链 (Causal Graph)                                │ │
│  │  - 检索方式: 路径遍历 + 精确查询                        │ │
│  └─────────────────────────────────────────────────────────┘ │
│                          ↓                                    │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  L4: Procedural Memory (Skills)                         │ │
│  │  - 可复用技能 (CrystalRegistry)                         │ │
│  │  - 工作流模板                                            │ │
│  │  - 检索方式: 技能匹配                                   │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**关键设计决策**:
1. **固定 schema 结构迁移可靠** (KG-fixed accuracy change: +0.0004)
2. **压缩笔记模型耦合高** (NOTES accuracy shift: ±9.91-13.28 percentage points)
3. **图记忆适合关系密集领域** (因果链、组织知识、依赖追踪)
4. **分层记忆**: Working → Episodic (Vector) → Semantic (SQL/Graph) → Procedural (Skills)

### 2.3 Cargo Workspace 最佳实践 (2026)

| 实践 | 说明 | NeoTrix 应用 |
|------|------|-------------|
| **Shared Test Infrastructure** | test-utils crate | ✅ 创建 nt-test-utils |
| **cargo-nextest** | 更快的测试执行 | ✅ CI 集成 |
| **cargo-deny** | 许可证/安全策略 | ✅ 依赖审计 |
| **cargo-udeps** | 未使用依赖检测 | ✅ 清理 dead_code |
| **cargo-machete** | 自动移除未使用依赖 | ✅ 依赖清理 |
| **Feature Flags** | 可选功能门控 | ✅ 按辐射臂 |

### 2.4 CTM-AI 架构规范 (最终版)

```
CTM 7-tuple: <STM, LTM, Up-Tree, Down-Tree, Links, Input, Output>

NeoTrix Crystal-Radiant 映射:
├── STM (短期记忆)     → CrystalConsciousness.workspace (容量=1)
├── LTM (长期记忆)     → 8 个辐射臂的内部状态
├── Up-Tree (竞争)     → Chunk 竞争选择 (显著性 ≥ 阈值)
├── Down-Tree (广播)   → 全局广播到所有辐射臂
├── Links (链接)       → 辐射臂间无意识通信 (LinkGraph)
├── Input (输入)       → SensoryInput (多模态)
└── Output (输出)      → CrystalOutput (决策 + 动作)
```

**无中央执行器原则**:
- 没有 conductor，通过竞争选择
- 每个辐射臂独立响应
- 晶体决定哪个 chunk 进入 workspace
- 所有通信经过晶体，无臂间直接调用

---

## 三、迁移策略设计 (最终版)

### 3.1 四阶段增量迁移

```
Phase 0: 基础设施 (3 天)
├── 目录清理 + 归档 (716 文件)
├── 测试套件完善 (nt-test-utils)
├── 影子对比框架 (shadow-comparator)
├── PORTING.md 契约 (每个辐射臂)
└── 依赖审计 (cargo-deny + cargo-udeps)

Phase 1: 无阻塞模块 (5 天) — 从叶子开始
├── NT-FEEL (11 文件, 零依赖)
├── NT-META (90 文件, 零外部依赖)
├── Crystal Core 完善 (28 文件)
├── 影子对比验证
└── 回滚点: git tag migration-phase-1

Phase 2: 中等难度模块 (8 天) — 逐步扩展
├── NT-SHIELD (201 文件, 依赖 L6 SelfTest)
├── NT-MEMORY (120 文件, 依赖 L0/L2 类型)
├── NT-ACT (200 文件, 依赖 L5 推理)
├── 影子对比验证
└── 回滚点: git tag migration-phase-2

Phase 3: 高难度模块 (12 天) — 核心重构
├── NT-WORLD (289 文件, 依赖 L5 能力向量)
├── NT-MIND (400 文件, 依赖 L6 自我模型)
├── NT-IO (300 文件, 依赖 L1/L5 接口)
├── 影子对比验证
└── 回滚点: git tag migration-phase-3

Phase 4: CTM 通信 + 治理 (5 天) — 集成验证
├── CTM 10步循环完善
├── 辐射臂隔离验证 (无臂间直接调用)
├── Governance 层实现 (审计日志)
├── 全系统集成测试
└── 最终回滚点: git tag migration-complete
```

### 3.2 每个模块的 PORTING.md 契约 (模板)

```markdown
# PORTING.md — [模块名] 迁移契约

## 目标
将 [旧位置] 迁移到 [新位置]

## 验证标准
- [ ] 所有现有测试通过 (cargo test -p [crate])
- [ ] 影子模式输出一致 (shadow-comparator)
- [ ] 无循环依赖 (cargo tree --workspace)
- [ ] dead_code 注解减少 50%
- [ ] 编译时间不增加 50%

## 迁移步骤
1. 生成架构文档 (模块分析)
2. 创建新 crate 结构
3. 逐文件迁移 (保持功能一致)
4. 更新依赖路径
5. 编译验证 (cargo check)
6. 测试验证 (cargo test)
7. 影子对比测试
8. 代码审查

## 回滚计划
- 保留旧代码在 archive/l0-l6/
- git tag 标记每个里程碑
- 如果问题: git revert migration-phase-N

## 依赖关系
- 前置依赖: [列表]
- 后置依赖: [列表]
- 禁止依赖: [辐射臂间直接调用]
```

### 3.3 影子对比框架 (增强版)

```rust
/// 影子对比器 — 验证新旧架构输出一致性
pub struct ShadowComparator {
    old_arch: OldArchitecture,
    new_arch: CrystalRadiant,
    diff_log: Vec<DiffRecord>,
    config: ComparatorConfig,
}

/// 对比配置
pub struct ComparatorConfig {
    /// 对比模式: Exact (精确) / Fuzzy (模糊) / Semantic (语义)
    pub mode: CompareMode,
    /// 容差阈值 (Fuzzy 模式)
    pub tolerance: f64,
    /// 最大对比次数
    pub max_iterations: usize,
    /// 自动回滚阈值 (差异率超过此值自动回滚)
    pub auto_rollback_threshold: f64,
}

impl ShadowComparator {
    /// 对比单次请求
    pub fn compare(&mut self, input: &str) -> ComparisonResult {
        let old_output = self.old_arch.process(input);
        let new_output = self.new_arch.process(input);
        
        let diff = self.compute_diff(&old_output, &new_output);
        self.diff_log.push(diff.clone());
        
        // 检查是否需要自动回滚
        if self.diff_log.len() >= self.config.max_iterations {
            let divergence_rate = self.calculate_divergence_rate();
            if divergence_rate > self.config.auto_rollback_threshold {
                return ComparisonResult::AutoRollbackNeeded {
                    divergence_rate,
                    threshold: self.config.auto_rollback_threshold,
                };
            }
        }
        
        ComparisonResult {
            identical: diff.is_empty(),
            old_output,
            new_output,
            diff,
        }
    }
    
    /// 生成对比报告
    pub fn report(&self) -> ComparisonReport {
        let total = self.diff_log.len();
        let identical = self.diff_log.iter().filter(|d| d.is_empty()).count();
        let divergence_rate = 1.0 - (identical as f64 / total as f64);
        
        ComparisonReport {
            total_requests: total,
            identical_outputs: identical,
            divergence_rate,
            details: self.diff_log.clone(),
            recommendation: if divergence_rate < 0.01 {
                "继续迁移".to_string()
            } else if divergence_rate < 0.05 {
                "修复差异后继续".to_string()
            } else {
                "回滚并分析原因".to_string()
            },
        }
    }
}
```

---

## 四、目录重构方案 (最终版)

### 4.1 新目录结构

```
neotrix/
├── Cargo.toml                    # Workspace root
├── Cargo.lock                    # 共享锁文件
├── .cargo/config.toml            # Cargo 配置
│
├── crates/
│   ├── nt-types/                 # 共享类型 (Clean: shared)
│   │   └── src/
│   │       ├── task.rs           # TaskType, TaskStatus (统一)
│   │       ├── capability.rs     # CapabilityVector
│   │       ├── knowledge.rs      # NodeType, RelationType
│   │       └── error.rs          # 共享错误类型
│   │
│   ├── nt-crystal/               # 晶体核心 (Clean: domain)
│   │   └── src/
│   │       ├── identity/         # L1 身份层
│   │       ├── knowledge/        # L2 知识层
│   │       ├── experience/       # L3 经验层
│   │       ├── evolution/        # L4 进化层
│   │       ├── ctm/              # CTM 通信引擎
│   │       │   ├── workspace.rs  # STM (容量=1)
│   │       │   ├── competition.rs # Up-Tree 竞争
│   │       │   ├── broadcast.rs  # Down-Tree 广播
│   │       │   └── links.rs      # 无意识链接
│   │       ├── cocoons/          # 持久记忆茧
│   │       └── consciousness.rs  # 统一意识
│   │
│   ├── nt-shield/                # 辐射臂: 安全 (Clean: infrastructure)
│   ├── nt-world/                 # 辐射臂: 世界感知
│   ├── nt-memory/                # 辐射臂: 知识存储
│   ├── nt-act/                   # 辐射臂: 工具调用
│   ├── nt-feel/                  # 辐射臂: 情感计算
│   ├── nt-meta/                  # 辐射臂: 元认知/自愈
│   ├── nt-mind/                  # 辐射臂: 推理/进化
│   ├── nt-io/                    # 辐射臂: 用户界面
│   │
│   └── nt-test-utils/            # 测试工具 (dev-dependencies)
│       └── src/
│           ├── shadow.rs         # 影子对比器
│           ├── fixtures.rs       # 测试数据
│           └── helpers.rs        # 辅助函数
│
├── apps/
│   ├── src-tauri/                # 桌面应用
│   └── neotrix-cli/              # CLI 工具
│
├── docs/
│   ├── architecture/             # 架构设计文档
│   ├── porting/                  # PORTING.md 契约
│   ├── guides/                   # 使用指南
│   ├── reference/                # API 参考
│   ├── iterations/               # 迭代记录 (归档)
│   └── adr/                      # 架构决策记录
│
├── archive/                      # 旧代码归档
│   ├── l0-l6/                    # 旧层级架构
│   └── iterations/               # 旧迭代批次
│
├── tests/                        # 集成测试
│   ├── shadow/                   # 影子对比测试
│   └── integration/              # 集成测试
│
├── tools/                        # 内部工具
│   └── migration/                # 迁移工具
│
├── benches/                      # 基准测试
│   └── performance/              # 性能基准
│
└── scripts/                      # 脚本
    ├── migrate.sh                # 迁移脚本
    └── verify.sh                 # 验证脚本
```

### 4.2 依赖流向 (Clean Architecture)

```
nt-types (shared)
    ↑
    └── 所有其他 crate

nt-crystal (domain)
    ↑
    ├── nt-shield (infrastructure)
    ├── nt-world (infrastructure)
    ├── nt-memory (infrastructure)
    ├── nt-act (infrastructure)
    ├── nt-feel (infrastructure)
    ├── nt-meta (infrastructure)
    ├── nt-mind (infrastructure)
    └── nt-io (presentation)

规则:
- nt-crystal 不依赖任何辐射臂
- 辐射臂只依赖 nt-crystal 和 nt-types
- 辐射臂之间不直接依赖 (通过 CTM 通信)
- nt-test-utils 只被 dev-dependencies 引用
```

### 4.3 Workspace Cargo.toml (最终版)

```toml
[workspace]
members = [
    "crates/nt-types",
    "crates/nt-crystal",
    "crates/nt-shield",
    "crates/nt-world",
    "crates/nt-memory",
    "crates/nt-act",
    "crates/nt-feel",
    "crates/nt-meta",
    "crates/nt-mind",
    "crates/nt-io",
    "apps/src-tauri",
    "apps/neotrix-cli",
]

[workspace.package]
version = "0.22.0"
edition = "2021"
rust-version = "1.81"
authors = ["NeoTrix Team"]
license = "MIT"
repository = "https://github.com/neo-trixs/NeoTrix"

[workspace.dependencies]
# 共享依赖
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
dirs = "6.0.0"
uuid = { version = "1", features = ["v4", "serde"] }

# 内部依赖
nt-types = { path = "crates/nt-types" }
nt-crystal = { path = "crates/nt-crystal" }
nt-test-utils = { path = "crates/nt-test-utils" }

[workspace.lints.clippy]
unwrap_used = "warn"
dead_code = "warn"

[workspace.metadatacargo-deny]
advisories = { vulnerability = "deny", unmaintained = "warn" }
licenses = { allow = ["MIT", "Apache-2.0"] }
```

---

## 五、实施路线图 (最终版)

### Phase 0: 基础设施 (3 天)

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T0.1 | 归档 666 个 iteration_batch 文件 | 666 | 无 | find 命令验证 |
| T0.2 | 归档根目录散乱 .md 文件 | ~50 | 无 | find 命令验证 |
| T0.3 | 创建 nt-test-utils crate | 5 | 无 | cargo check |
| T0.4 | 实现 ShadowComparator | 3 | T0.3 | cargo test |
| T0.5 | 为每个辐射臂编写 PORTING.md | 9 | 无 | 文档完整 |
| T0.6 | 完善现有测试套件 | 0 | 无 | cargo test 通过 |
| T0.7 | 依赖审计 (cargo-deny) | 0 | 无 | 0 漏洞 |
| T0.8 | 未使用依赖检测 (cargo-udeps) | 0 | 无 | 清理报告 |

### Phase 1: 无阻塞模块 (5 天) — 从叶子开始

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T1.1 | NT-FEEL 迁移 | 11 | Phase 0 | cargo test |
| T1.2 | NT-META 迁移 | 90 | Phase 0 | cargo test |
| T1.3 | Crystal Core 完善 | 28 | 无 | cargo test |
| T1.4 | 影子对比测试 | - | T1.1-T1.3 | 0 差异 |
| T1.5 | 修复发现的差异 | - | T1.4 | 0 差异 |
| T1.6 | 回滚点创建 | - | T1.5 | git tag |

### Phase 2: 中等难度模块 (8 天) — 逐步扩展

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T2.1 | NT-SHIELD 迁移 | 201 | Phase 1 | cargo test |
| T2.2 | NT-MEMORY 迁移 | 120 | Phase 1 | cargo test |
| T2.3 | NT-ACT 迁移 | 200 | Phase 1 | cargo test |
| T2.4 | 影子对比测试 | - | T2.1-T2.3 | 0 差异 |
| T2.5 | 修复发现的差异 | - | T2.4 | 0 差异 |
| T2.6 | 回滚点创建 | - | T2.5 | git tag |

### Phase 3: 高难度模块 (12 天) — 核心重构

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T3.1 | NT-WORLD 迁移 | 289 | Phase 2 | cargo test |
| T3.2 | NT-MIND 迁移 | 400 | Phase 2 | cargo test |
| T3.3 | NT-IO 迁移 | 300 | Phase 2 | cargo test |
| T3.4 | 影子对比测试 | - | T3.1-T3.3 | 0 差异 |
| T3.5 | 修复发现的差异 | - | T3.4 | 0 差异 |
| T3.6 | 回滚点创建 | - | T3.5 | git tag |

### Phase 4: CTM 通信 + 治理 (5 天) — 集成验证

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T4.1 | CTM 10步循环完善 | 50 | Phase 3 | cargo test |
| T4.2 | 辐射臂隔离验证 | 20 | T4.1 | 0 臂间直接调用 |
| T4.3 | Governance 层实现 | 30 | T4.1 | 审计日志 |
| T4.4 | 全系统集成测试 | - | T4.1-T4.3 | E2E 测试 |
| T4.5 | 性能基准测试 | - | T4.4 | 不退化 10% |

---

## 六、风险控制 (最终版)

### 6.1 风险矩阵

| 风险 | 概率 | 影响 | 缓解 | 检测 | 回滚 |
|------|------|------|------|------|------|
| 编译失败 | 高 | 高 | 每步 cargo check | 编译错误 | git revert |
| 功能退化 | 中 | 高 | 影子对比测试 | 输出差异 | git revert |
| 循环依赖 | 中 | 中 | 依赖图分析 | cargo tree | 移除依赖 |
| 性能下降 | 低 | 中 | 基准测试 | 性能回归 | git revert |
| 数据丢失 | 低 | 高 | git 备份 | git log | git restore |
| 迁移超时 | 中 | 中 | 缓冲时间 | 进度跟踪 | 调整范围 |

### 6.2 自动回滚机制

```rust
/// 自动回滚检查器
pub struct RollbackChecker {
    threshold: f64,  // 差异率阈值
    history: Vec<MigrationCheckpoint>,
}

impl RollbackChecker {
    /// 检查是否需要回滚
    pub fn should_rollback(&self, report: &ComparisonReport) -> RollbackDecision {
        if report.divergence_rate > self.threshold {
            RollbackDecision::Rollback {
                reason: format!(
                    "差异率 {:.2%} 超过阈值 {:.2%}",
                    report.divergence_rate, self.threshold
                ),
                checkpoint: self.history.last().cloned(),
            }
        } else {
            RollbackDecision::Continue
        }
    }
}
```

### 6.3 验证命令 (最终)

```bash
# 目录清理验证
find . -name "iteration_batch_*.md" | wc -l  # 目标: 0
find . -maxdepth 1 -name "*.md" | wc -l      # 目标: <10

# Workspace 验证
cargo check --workspace                       # 目标: 0 error
cargo test --workspace                        # 目标: pass
cargo clippy --workspace -- -D warnings       # 目标: 0 warning

# 依赖分析
cargo tree --workspace --duplicates           # 目标: 0 duplicates
cargo deny check                             # 目标: 0 advisories
cargo udeps                                  # 目标: 0 unused

# 代码质量
grep -r "enum TaskType" --include="*.rs" | wc -l  # 目标: 1
grep -r "enum TaskStatus" --include="*.rs" | wc -l # 目标: 1
grep -r "#\[allow(dead_code)\]" --include="*.rs" | wc -l  # 目标: <30

# 影子对比
cargo test --test shadow_comparison            # 目标: 0 differences

# 性能基准
cargo bench --workspace                        # 目标: 不退化 10%
```

---

## 七、总工期估算 (最终版)

| Phase | 内容 | 工期 | 文件数 | 里程碑 |
|-------|------|------|--------|--------|
| Phase 0 | 基础设施 | 3 天 | ~716 | 影子框架就绪 |
| Phase 1 | 无阻塞模块 | 5 天 | ~129 | FEEL/META 迁移完成 |
| Phase 2 | 中等难度模块 | 8 天 | ~521 | SHIELD/MEMORY/ACT 迁移完成 |
| Phase 3 | 高难度模块 | 12 天 | ~989 | WORLD/MIND/IO 迁移完成 |
| Phase 4 | CTM 通信 + 治理 | 5 天 | ~100 | 全系统集成完成 |
| **总计** | | **33 天** | **~2455** | |

---

## 八、成功标准 (最终版)

### 8.1 功能标准

| 标准 | 目标 | 验证方法 |
|------|------|---------|
| 编译通过 | 0 error | cargo check --workspace |
| 测试通过 | 100% | cargo test --workspace |
| 影子对比 | 0 差异 | shadow_comparison 测试 |
| 循环依赖 | 0 | cargo tree 分析 |
| dead_code | <30 | cargo udeps |

### 8.2 架构标准

| 标准 | 目标 | 验证方法 |
|------|------|---------|
| 辐射臂隔离 | 无臂间直接调用 | 依赖图分析 |
| CTM 通信 | 所有通信经过晶体 | 代码审查 |
| 类型统一 | TaskType/TaskStatus 各 1 个 | grep 统计 |
| 目录规范 | 符合 Clean Architecture | 代码审查 |

### 8.3 性能标准

| 标准 | 目标 | 验证方法 |
|------|------|---------|
| 编译时间 | 不增加 50% | cargo build 时间对比 |
| 运行时性能 | 不退化 10% | 基准测试 |
| 内存使用 | 不增加 20% | 内存分析 |

### 8.4 记忆架构标准 (对标 MAGMA)

| 标准 | 目标 | 验证方法 |
|------|------|---------|
| 四层记忆 | Working/Episodic/Semantic/Procedural | 代码审查 |
| 图记忆支持 | 因果链 + 实体关系 | 功能测试 |
| 检索路由 | 按查询类型路由到正确后端 | 单元测试 |
| 迁移可靠性 | 固定 schema 迁移精度 >99% | 迁移测试 |

---

## 九、产出文件清单

| 文件 | 内容 | 状态 |
|------|------|------|
| `docs/FULL-MIGRATION-ROADMAP-v3.md` | 完整迁移路线图 v3.0 (最终优化版) | ✅ 本文件 |
| `docs/FULL-MIGRATION-ROADMAP-v2.md` | 完整迁移路线图 v2.0 | ✅ 已完成 |
| `docs/FULL-MIGRATION-ROADMAP.md` | 完整迁移路线图 v1.0 | ✅ 已完成 |
| `docs/MIGRATION-ANALYSIS.md` | 旧→新迁移分析 | ✅ 已完成 |
| `docs/IMPLEMENTATION-PLAN-MINIMAL.md` | 精简实施方案 | ✅ 已完成 |
| `docs/REDUNDANCY-CLEANUP-RESTRUCTURE.md` | 冗余清理设计 | ✅ 已完成 |
