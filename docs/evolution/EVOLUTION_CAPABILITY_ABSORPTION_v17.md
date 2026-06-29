# NeoTrix 自进化架构 v17 — 外部知识吸收路线图

> 从 10+ GitHub 项目 + 7 篇前沿论文蒸馏
> 构建日期: 2026-06-23

## 核心发现：外部生态的 8 个可吸收能力

| # | 项目/论文 | 关键能力 | NeoTrix 当前状态 | 吸收优先级 |
|---|-----------|----------|-----------------|-----------|
| 1 | ProductNormaliser | 基于类别Schema的属性标准化+身份解析 | 17列标准格式已定义，但无类别感知的标准化引擎 | P0 |
| 2 | InTabular | AI驱动的CSV→Schema智能映射 | 手动编码精度高但效率低 | P0 |
| 3 | DGM-H (arXiv 2505.22954) | 开放式自改进 + agent存档 | SelfEvolutionMetaLayer存在但无存档机制 | P0 |
| 4 | APEX (arXiv 2606.15363) | 三轴协同进化（harness/原则/工作流） | 仅单轴进化（任务级别） | P1 |
| 5 | AGP/AGS (arXiv 2604.15034) | 总线架构 + SEPL进化循环 | ConsciousnessPipeline是固定管线 | P1 |
| 6 | csvnorm | DuckDB驱动的CSV验证 | 纯Python验证, 可集成DuckDB | P2 |
| 7 | canonical-product-schema | "Typed Structure + Flexible Payload" | 已采用类似模式（17列固定+自由字段） | P2 |
| 8 | AgentFactory | 三阶段生命周期（安装→自进化→部署） | 子agent能力积累未系统化 | P1 |

## 架构演进：三波规划

### Wave A — 外部知识吸收管道（~500行）

**问题**: 每次搜索GitHub/论文后发现的有价值能力，没有系统化地吸收到体系内。

**方案**: `ProjectAbsorptionPipeline`
- `scan_source()`: 从GitHub/arXiv抓取项目元数据
- `extract_methodology()`: 用LLM提取关键方法论到结构化格式
- `evaluate_relevance()`: 按8维评估（表征效率/推理深度/自我认知/...）
- `integrate_insight()`: 蒸馏到经验树 + 自动生成EvolutionTask
- `generate_implementation_plan()`: 如果P0则生成实现计划

```rust
pub struct ProjectAbsorption {
    pub source: String,          // GitHub URL / arXiv ID
    pub methodology: String,     // 关键方法论摘要
    pub relevance_scores: [f64; 8], // 意识体进化8维评分
    pub priority: Priority,       // P0/P1/P2
    pub absorbed: bool,          // 是否已吸收
    pub integration_task: Option<EvolutionTask>, // 集成任务
}
```

### Wave B — AI辅助Schema映射（~300行扩展已有脚本）

**问题**: 手动编码14家供应商的价格数据准确但低效（v3脚本硬编码）。

**方案**: 基于InTabular 4-step pipeline的 `SmartSchemaMapper`
1. `analyze_source()`: 分析CSV的列名和样本数据 → 推断业务语义
2. `map_to_schema()`: 将源列映射到17列标准格式
3. `transform_data()`: 执行确定性转换（口径标准化/价格四舍五入/类别提取）
4. `validate_quality()`: IQR异常检测 + 格式校验

### Wave C — 多轴自进化（已有组件接线）

**问题**: SelfEvolutionMetaLayer + EvolutionTaskSystem 存在但进化维度单一。

**方案**: 基于APEX 3轴 + AGP SEPL循环 + DGM-H存档
- **轴1-Harness**: SelfEvolutionMetaLayer tick() 已有 → 优化触发条件
- **轴2-原则**: EvolutionTaskSystem auto_discover_from_audit 已有 → 增加跨任务模式蒸馏
- **轴3-工作流**: SelfModifyGuard 已有 → 增加SEPL评估阶段

## 持续进化任务引擎

EvolutionTaskSystem (370行, 12测试) 已实现的基础：

### 当前能力
- 基于4指标自动发现任务（ECE>0.15 / loss>0.4 / meta_acc<0.7 / pending_modules>0）
- 10种任务类型
- 依赖感知的优先级排序
- 7态生命周期 (Discovered→Prioritized→Planned→InProgress→Completed/Blocked/Cancelled)

### 待增强
| 增强项 | 来源 | 优先级 |
|--------|------|--------|
| 外部项目吸收 → 自动任务创建 | AgentFactory | P0 |
| 跨任务模式蒸馏 → 新原则 | APEX | P1 |
| 存档+探索（不仅利用） | DGM-H | P1 |
| 执行结果反馈循环 → 自适应触发 | AGP SEPL | P2 |

## 经验树更新计划

### 新分支 XL — 外部知识吸收 (External Knowledge Absorption)

#### XL.1 搜索优先于设计 (conf 0.9, 更新)
- **规则**: 重大架构决策前，先搜索GitHub + arXiv。生态已经解决的问题不需要自己发明。
- **新证据**: 8个项目中，4个（ProductNormaliser/InTabular/DGM-H/APEX）直接映射到架构缺口
- **演化链**: v1(2026-06-13) → v2(2026-06-23) → v3(2026-06-23)

#### XL.2 Schema优先于手工编码 (conf 0.7, 新建)
- **规则**: 数据整理应采用类别感知的Schema映射（如ProductNormaliser），而非手工编写Python元组。
- **正确**: v3手工编码的409条数据精度高但无法扩展
- **方向**: 构建SmartSchemaMapper，自动推断列映射

#### XL.3 三轴优于单轴进化 (conf 0.6, 新建)
- **规则**: 自进化系统的效果与进化维度数量正相关。APEX的3轴协同优于Self-Harness的单轴。
- **正确**: NeoTrix已有SelfEvolutionMetaLayer（1轴）和EvolutionTaskSystem（1轴），需要增加原则和工作流进化

## 当前进化状态

```
当前:   架构版本 v0.17.0, 689条产品数据 14家供应商
回路:   5条反馈回路全部闭合 ✅
外部:   8个项目/论文吸收完成 ✅
缺口:   3个P0增强项待实现
经验树: 新分支XL已设计，待执行后蒸馏
任务:   EvolutionTaskSystem 自管理
```

## 下一步执行顺序

1. SmartSchemaMapper 构建（基于InTabular模式）
2. ProjectAbsorptionPipeline 构建（基于AgentFactory模式）
3. EvolutionTaskSystem 增强（APEX 3轴 + DGM-H存档）
4. 13家剩余供应商自动/半自动录入（利用Schema映射）
