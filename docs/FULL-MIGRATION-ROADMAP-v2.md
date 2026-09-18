# NeoTrix 完整迁移路线图 v2.0 — 深度优化版

> **日期**: 2026-09-18
> **基于**: 项目结构分析 + 互联网最新迁移实践 + CTM-AI 架构规范 + 代码库深度扫描
> **目标**: 从 L0-L6 层级架构迁移到 Crystal-Radiant 辐射架构
> **优化来源**: GitHub Copilot (800K lines) / Perplexity / Bun / CTM-AI / RustPrint

---

## 一、项目现状诊断

### 1.1 目录结构统计

| 指标 | 数值 | 问题 |
|------|------|------|
| 总 .rs 文件 | **2047** | 1843 (L0-L6) + 204 (其他) |
| 总 .md 文件 | **2903** | 666 迭代批次 + 2237 其他 |
| Workspace Crates | **10** | 需要统一到 `crates/` 目录 |
| 根目录文件 | **~100+** | 过于杂乱，需要清理 |
| 循环依赖 | **31+** | 需要修复 |
| dead_code 注解 | **87** | 需要清理 |

### 1.2 架构迁移复杂度

| 模块 | 旧位置 | 文件数 | 迁移难度 | 阻塞点 |
|------|--------|--------|----------|--------|
| Crystal Core | nt_crystal_core | 28 | ⭐ | 无 |
| NT-FEEL | l4_emotion | 11 | ⭐ | 无 |
| NT-META | l6_meta (部分) | 90 | ⭐⭐ | 无 |
| NT-SHIELD | l3_embodiment | 201 | ⭐⭐⭐ | SelfTest 依赖 |
| NT-MEMORY | l1_action | 120 | ⭐⭐⭐ | L0/L2 类型依赖 |
| NT-ACT | l1_action | 200 | ⭐⭐⭐ | L5 推理依赖 |
| NT-WORLD | l2_perception | 289 | ⭐⭐⭐⭐ | L5 能力向量依赖 |
| NT-MIND | l5_cognition | 400 | ⭐⭐⭐⭐⭐ | L6 自我模型依赖 |
| NT-IO | l1_action | 300 | ⭐⭐⭐⭐ | L1/L5 接口依赖 |

---

## 二、互联网最佳实践吸收

### 2.1 迁移策略选择 (来源: GitHub Copilot)

| 策略 | 优点 | 缺点 | 适用场景 |
|------|------|------|---------|
| **Big Bang** | 一次性切换，无并行维护 | 风险高，回滚困难 | 小项目 |
| **In-Place Atomic** | 组件级原子替换 | 需要 interop 层 | 中等项目 |
| **In-Place A/B** | 可热切换，风险最低 | 维护成本高 | 大项目 |
| **Incremental Shadow** | 风险可控，可验证 | 需要对比基础设施 | ✅ NeoTrix 推荐 |

**NeoTrix 策略**: **Incremental Shadow** — 增量影子迁移

```
阶段 1: 影子模式 (Shadow)
  旧架构继续运行
  新架构并行运行，仅读取
  对比输出差异

阶段 2: 渐进切换 (Gradual)
  逐模块从旧→新
  每次切换一个辐射臂
  保留回滚能力

阶段 3: 旧架构退役 (Sunset)
  所有模块迁移到新架构
  旧代码归档
  清理 interop 层
```

### 2.2 安全机制 (来源: Perplexity + Bun)

| 机制 | 说明 | NeoTrix 应用 |
|------|------|-------------|
| **Goal-Verifier-Workspace** | 目标+验证器+工作空间 | 每个迁移任务必须有验证标准 |
| **Shadow Workspace** | 影子空间测试 | 新旧架构并行运行 |
| **PORTING.md Contract** | 迁移契约 | 每个模块的迁移规范文档 |
| **Test Suite as Safety Net** | 测试套件安全网 | 迁移前必须有完整测试 |
| **Mechanical Porting First** | 先机械移植 | 先保持功能一致，再优化 |

### 2.3 CTM-AI 架构规范 (来源: CTM-AI 论文)

```
CTM 7-tuple: <STM, LTM, Up-Tree, Down-Tree, Links, Input, Output>

NeoTrix 映射:
├── STM (短期记忆)     → CrystalConsciousness.workspace
├── LTM (长期记忆)     → 8 个辐射臂的内部状态
├── Up-Tree (竞争)     → Chunk 竞争选择
├── Down-Tree (广播)   → 全局广播到所有辐射臂
├── Links (链接)       → 辐射臂间无意识通信
├── Input (输入)       → SensoryInput
└── Output (输出)      → CrystalOutput
```

**关键原则**:
1. **无中央执行器**: 没有 conductor，通过竞争选择
2. **有限容量 Workspace**: 容量=1，强制选择性注意
3. **全局广播**: 获胜 chunk 广播到所有处理器
4. **链接形成**: 处理器间形成无意识通信链接

### 2.4 文档驱动迁移 (来源: RustPrint)

```
5阶段管线:
1. 源代码文档生成 (DocGen)
2. Crate 级规划和翻译
3. 工作空间级合成
4. 文档驱动需求精化 (K轮)
5. 执行感知修复 (L轮)
```

**NeoTrix 应用**:
- 每个辐射臂迁移前生成架构文档
- 文档作为迁移蓝图
- 对比源和目标文档检测缺失功能
- 测试驱动修复行为错误

---

## 三、迁移策略设计

### 3.1 四阶段迁移

```
Phase 0: 基础设施 (3 天)
├── 目录清理 + 归档
├── 测试套件完善
├── 影子对比框架
└── PORTING.md 契约

Phase 1: 无阻塞模块 (5 天)
├── NT-FEEL (11 文件)
├── NT-META (90 文件)
├── Crystal Core 完善 (28 文件)
└── 验证: 影子模式对比

Phase 2: 中等难度模块 (8 天)
├── NT-SHIELD (201 文件)
├── NT-MEMORY (120 文件)
├── NT-ACT (200 文件)
└── 验证: 功能回归测试

Phase 3: 高难度模块 (12 天)
├── NT-WORLD (289 文件)
├── NT-MIND (400 文件)
├── NT-IO (300 文件)
└── 验证: 端到端测试

Phase 4: CTM 通信 + 治理 (5 天)
├── CTM 10步循环完善
├── 辐射臂隔离验证
├── Governance 层实现
└── 验证: 全系统集成测试
```

### 3.2 每个模块的 PORTING.md 契约

```markdown
# PORTING.md — NT-SHIELD 迁移契约

## 目标
将 l3_embodiment/nt_shield/ 迁移到 crates/nt-shield/

## 验证标准
- [ ] 所有现有测试通过
- [ ] 影子模式输出一致
- [ ] 无循环依赖
- [ ] dead_code 注解减少 50%

## 迁移步骤
1. 生成架构文档 (shieldscope)
2. 创建 crates/nt-shield/ 结构
3. 逐文件迁移
4. 更新依赖路径
5. 编译验证
6. 影子对比测试

## 回滚计划
- 保留旧代码在 archive/l0-l6/
- git tag 标记每个里程碑
```

### 3.3 影子对比框架

```rust
/// 影子对比器 — 验证新旧架构输出一致性
pub struct ShadowComparator {
    old_arch: OldArchitecture,
    new_arch: CrystalRadiant,
    diff_log: Vec<DiffRecord>,
}

impl ShadowComparator {
    /// 对比单次请求
    pub fn compare(&mut self, input: &str) -> ComparisonResult {
        let old_output = self.old_arch.process(input);
        let new_output = self.new_arch.process(input);
        
        let diff = self.compute_diff(&old_output, &new_output);
        self.diff_log.push(diff.clone());
        
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
        
        ComparisonReport {
            total_requests: total,
            identical_outputs: identical,
            divergence_rate: 1.0 - (identical as f64 / total as f64),
            details: self.diff_log.clone(),
        }
    }
}
```

---

## 四、目录重构方案

### 4.1 新目录结构 (对标 Clean Architecture + CTM-AI)

```
neotrix/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── nt-types/                 # 共享类型 (Clean: shared)
│   │   └── src/
│   │       ├── task.rs           # TaskType, TaskStatus (统一)
│   │       ├── capability.rs     # CapabilityVector
│   │       └── knowledge.rs      # NodeType, RelationType
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
│   └── nt-io/                    # 辐射臂: 用户界面
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
└── tools/                        # 内部工具
    └── migration/                # 迁移工具
```

### 4.2 依赖流向 (Clean Architecture)

```
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

nt-types (shared)
    ↑
    └── 所有其他 crate

规则:
- nt-crystal 不依赖任何辐射臂
- 辐射臂只依赖 nt-crystal 和 nt-types
- 辐射臂之间不直接依赖 (通过 CTM 通信)
```

---

## 五、实施路线图 (详细)

### Phase 0: 基础设施 (3 天)

| 任务 | 说明 | 依赖 | 验证 |
|------|------|------|------|
| T0.1 | 归档 666 个 iteration_batch 文件 | 无 | find 命令验证 |
| T0.2 | 归档根目录散乱 .md 文件 | 无 | find 命令验证 |
| T0.3 | 创建 docs/porting/ 目录 | 无 | 目录存在 |
| T0.4 | 为每个辐射臂编写 PORTING.md | T0.3 | 文档完整 |
| T0.5 | 完善现有测试套件 | 无 | cargo test 通过 |
| T0.6 | 创建影子对比框架 | 无 | 框架可运行 |
| T0.7 | 创建 archive/l0-l6/ 目录 | 无 | 目录存在 |

### Phase 1: 无阻塞模块 (5 天)

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T1.1 | NT-FEEL 迁移 | 11 | Phase 0 | cargo test |
| T1.2 | NT-META 迁移 | 90 | Phase 0 | cargo test |
| T1.3 | Crystal Core 完善 | 28 | 无 | cargo test |
| T1.4 | 影子对比测试 | - | T1.1-T1.3 | 对比报告 |
| T1.5 | 修复发现的差异 | - | T1.4 | 0 差异 |

### Phase 2: 中等难度模块 (8 天)

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T2.1 | NT-SHIELD 迁移 | 201 | Phase 1 | cargo test |
| T2.2 | NT-MEMORY 迁移 | 120 | Phase 1 | cargo test |
| T2.3 | NT-ACT 迁移 | 200 | Phase 1 | cargo test |
| T2.4 | 影子对比测试 | - | T2.1-T2.3 | 对比报告 |
| T2.5 | 修复发现的差异 | - | T2.4 | 0 差异 |

### Phase 3: 高难度模块 (12 天)

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T3.1 | NT-WORLD 迁移 | 289 | Phase 2 | cargo test |
| T3.2 | NT-MIND 迁移 | 400 | Phase 2 | cargo test |
| T3.3 | NT-IO 迁移 | 300 | Phase 2 | cargo test |
| T3.4 | 影子对比测试 | - | T3.1-T3.3 | 对比报告 |
| T3.5 | 修复发现的差异 | - | T3.4 | 0 差异 |

### Phase 4: CTM 通信 + 治理 (5 天)

| 任务 | 说明 | 文件数 | 依赖 | 验证 |
|------|------|--------|------|------|
| T4.1 | CTM 10步循环完善 | 50 | Phase 3 | cargo test |
| T4.2 | 辐射臂隔离验证 | 20 | T4.1 | 无臂间直接调用 |
| T4.3 | Governance 层实现 | 30 | T4.1 | 审计日志 |
| T4.4 | 全系统集成测试 | - | T4.1-T4.3 | E2E 测试 |

---

## 六、风险控制 (增强版)

### 6.1 风险矩阵

| 风险 | 概率 | 影响 | 缓解 | 检测 |
|------|------|------|------|------|
| 编译失败 | 高 | 高 | 每步 cargo check | 编译错误 |
| 功能退化 | 中 | 高 | 影子对比测试 | 输出差异 |
| 循环依赖 | 中 | 中 | 依赖图分析 | cargo tree |
| 性能下降 | 低 | 中 | 基准测试 | 性能回归 |
| 数据丢失 | 低 | 高 | git 备份 | git log |

### 6.2 回滚策略

```
每个 Phase 完成后:
1. git tag migration-phase-N
2. 保留旧代码在 archive/l0-l6/
3. 如果发现问题:
   git revert migration-phase-N
4. 分析原因，修复后重新迁移
```

### 6.3 验证命令 (增强)

```bash
# 目录清理验证
find . -name "iteration_batch_*.md" | wc -l  # 目标: 0
find . -maxdepth 1 -name "*.md" | wc -l      # 目标: <10

# Workspace 验证
cargo check --workspace                       # 目标: 0 error
cargo test --workspace                        # 目标: pass
cargo clippy --workspace                      # 目标: 0 warning

# 依赖分析
cargo tree --workspace --duplicates           # 目标: 0 duplicates
cargo tree --workspace --no-dedupe | grep -c "loop"  # 目标: 0

# 代码质量
grep -r "enum TaskType" --include="*.rs" | wc -l  # 目标: 1
grep -r "enum TaskStatus" --include="*.rs" | wc -l # 目标: 1
grep -r "#\[allow(dead_code)\]" --include="*.rs" | wc -l  # 目标: <30

# 影子对比
cargo test --test shadow_comparison            # 目标: 0 differences
```

---

## 七、总工期估算 (优化后)

| Phase | 内容 | 工期 | 文件数 | 里程碑 |
|-------|------|------|--------|--------|
| Phase 0 | 基础设施 | 3 天 | ~716 | 影子框架就绪 |
| Phase 1 | 无阻塞模块 | 5 天 | ~129 | FEEL/META 迁移完成 |
| Phase 2 | 中等难度模块 | 8 天 | ~521 | SHIELD/MEMORY/ACT 迁移完成 |
| Phase 3 | 高难度模块 | 12 天 | ~989 | WORLD/MIND/IO 迁移完成 |
| Phase 4 | CTM 通信 + 治理 | 5 天 | ~100 | 全系统集成完成 |
| **总计** | | **33 天** | **~2455** | |

---

## 八、成功标准

### 8.1 功能标准

| 标准 | 目标 | 验证方法 |
|------|------|---------|
| 编译通过 | 0 error | cargo check --workspace |
| 测试通过 | 100% | cargo test --workspace |
| 影子对比 | 0 差异 | shadow_comparison 测试 |
| 循环依赖 | 0 | cargo tree 分析 |
| dead_code | <30 | grep 统计 |

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
