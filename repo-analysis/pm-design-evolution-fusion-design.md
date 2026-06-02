# NeoTrix × PM/Designer 能力矩阵 — 融合设计

> 基于用户提供的 6 种 Prompt Engineering 技术 × 20 项 PM/Designer 技能框架
> 分析 NeoTrix 当前 ~82K LOC Rust 代码库的差距与进化路线

---

## 1. 全景差距图

### 1.1 Prompt Engineering 技术适配

| # | 技术 | NeoTrix 现有支持 | 差距 | 接入点 |
|---|------|-----------------|------|--------|
| 1 | **Role Prompting** | `persona.rs` 有 AgentPersona/PersonaRole/ExperienceLevel | 未用于推理上下文的角色动态注入 | `reasoning_engine/internal.rs::build_context()` |
| 2 | **Chain-of-Thought** | `advanced.rs` 有 `reason_multi_perspective()` 8 种方法 | 无显式 COT 步骤模板 | `reasoning_engine/reasoning.rs` 各方法 |
| 3 | **Structured Output** | `TODO.md`/`TODO.yml` 结构化 | PRD/分析/报告无强制格式 | 新增 `StructuredOutput` trait |
| 4 | **Context Augmentation** | `SelfEvolver` 三流分析 + `Accessor` trait | 无多源并行上下文注入 | `reasoning_engine/internal.rs::build_artifact_context()` |
| 5 | **Iterative Refinement** | SEAL 循环 + `CodeReviewLoop` | 无 PM/Design 迭代验证循环 | `orchestrator/critic.rs` 扩展 |
| 6 | **Tool Use** | 3 个 MCP 工具 (web_scrape/security_audit/react_doctor) | 无 PM/Design 专用工具 | `mcp_tools.rs` + `agent/tools/mod.rs` |

### 1.2 PM 10 项技能差距

| 技能 | NeoTrix 状态 | 现有模块/数据 | 缺失能力 |
|------|-------------|--------------|---------|
| **PRD Writer** | ❌ 无 | `ReasoningEngine` 5 种推理类型 + `context_artifacts.rs` | 无产品需求文档生成、评审、版本管理 |
| **Priority Engine** | ⚠️ 部分 | `GoalPriority` Low/Med/High/Critical 枚举 + `GoalScheduleStrategy` 3 种策略 | **无 RICE/ICE/MoSCoW 定量打分**，无价值-努力矩阵 |
| **Stakeholder Communicator** | ❌ 无 | `PersonaRole` 枚举 (Developer/Designer/ProductManager/QA/DevOps/Researcher/Executive) | 无角色适配叙事生成、无状态报告 |
| **Feature Scoper** | ❌ 无 | `Orchestrator::PlannerNode::decompose()` 仅 3 种 plan_* 方法 | 无 Epic→Story→Task 分解、无 AC 完备性检查 |
| **Competitive Analyst** | ⚠️ 部分 | `SelfEvolver` 三流分析单仓库 | 无多仓库并行对比、无特征矩阵构建、无定位差距推理 |
| **Metrics Definer** | ❌ 无 | `EngineMetrics`(工程指标) + `TelemetryCollector`(运行时) | 无产品北向指标定义、无 OKR/KPI 框架 |
| **Experiment Designer** | ❌ 无 | `WorldModel::reward_from_knowledge_quality()` RL 奖励 | 无假设框架、无 A/B 测试设计、无实验注册表 |
| **Roadmap Builder** | ⚠️ 部分 | `EvolutionPlanner::module_roadmap()` 代码演进路线图 | 无产品路线图（时间线/版本/功能分组） |
| **Customer Feedback Distiller** | ⚠️ 部分 | `SessionDistiller` + `UserAvatar` 交互模式蒸馏 | 无结构化反馈分析、无主题聚类、无情感分析 |
| **Launch Checklist Generator** | ❌ 无 | `Orchestrator::critic::cross_validate()` 能力向量检查 | 无跨职能启动质量门控 |

### 1.3 Designer 10 项技能差距

| 技能 | NeoTrix 状态 | 现有模块/数据 | 缺失能力 |
|------|-------------|--------------|---------|
| **UX Critique** | ❌ 无 | `CodeReviewEngine` 代码审查 + `react_doctor` MCP 工具 | 无 UI/UX 启发式评估、无视觉层次分析 |
| **Excalidraw Diagram** | ❌ 无 | `StateGraph::build_plan()` DAG 可视化 | 无架构图/流程图/用户旅程图生成 |
| **Design System Auditor** | ⚠️ 部分 | `SelfEvolver::analyze_docs_stream()` 检测"design system"关键词 | 无 Token 一致性检查、无组件库规则推理 |
| **UX Copy Writer** | ❌ 无 | — | 无微文案生成、无品牌 Voice 适配 |
| **User Research Synthesizer** | ⚠️ 部分 | `UserAvatar::DistillationEngine` 领域/任务/知识亲和性 | 无访谈记录分析、无调查图表、无亲和图 |
| **Wireframe to Spec** | ❌ 无 | — | 无线框图解析、无交互标注生成 |
| **Accessibility Checker** | ⚠️ 部分 | `capability_vector` 有 `accessibility` 维度 + `ReactDoctor` 检查 alt 属性 | 无 WCAG 完整检查（对比度/键盘导航/ARIA） |
| **Frontend Design** | ⚠️ 部分 | `react_doctor` MCP + `HeroUI`/`BaseUI` 知识来源 | 无设计→代码生成流水线 |
| **Case Study Writer** | ❌ 无 | `ReasoningBank` 有 ReasoningMemory 可作素材 | 无叙事结构生成 (Problem→Process→Result) |
| **Journey Mapper** | ❌ 无 | — | 无端到端流程映射、无情绪曲线、无触点分析 |

---

## 2. 优先级矩阵

```
Impact (用户框架的理论价值)
  ↑
  │  🔴 A. Priority Engine    🔴 F. Agentic Workflow
  │     (GoalLoop 决策质量)      (全流程闭环)
  │
  │  🟡 C. Competitive Analyst 🟡 B. PRD Writer
  │     (知识吸收环增强)          (新能力)
  │
  │  🟢 E. UX Critique         🟢 D. Experiment Designer
  │     (领域特化)               (锦上添花)
  │
  └───────────────────────────────→ Effort (现有模块可复用程度)
     低 ←———————————————→ 高
```

### 依赖关系图

```
Route C (Competitive Analyst) ──→ Route B (PRD Writer)
        ↓                                  ↓
Route A (Priority Engine) ←── Route F (Agentic Workflow Orchestration)
        ↓                                  ↓
Route D (Experiment Designer) ──→ Route E (UX Critique)
```

---

## 3. 每条路线详细设计

---

### 🔴 A. Priority Engine — GoalLoop 定量优先级框架

#### 间隙分析

| 当前 | 目标 | 差距行数 |
|------|------|---------|
| `GoalPriority` 4 级枚举 | RICE/ICE/MoSCoW 多框架并行打分 | ~80 行新增 |
| `GoalScheduleStrategy` 3 种无权重策略 | 加权混合调度器 | ~60 行 |
| `rebalance_from_motivation()` 仅 MotivationState | 多因子重平衡（RICE + 动机 + E8） | ~40 行修改 |
| `enqueue_goal()` 无优先级计算 | 入队时自动打分排序 | ~30 行修改 |

#### 代码设计

```rust
// === 新增: goal_loop/priority.rs ===

/// RICE 分数: Reach × Impact × Confidence / Effort
pub struct RICEScore {
    pub reach: f64,       // 影响用户数/周期 (0-10)
    pub impact: f64,      // 转化率/满意度提升 (0-10)
    pub confidence: f64,  // 证据强度 (0-10)
    pub effort: f64,      // 人月/开发成本 (0-10)
}

impl RICEScore {
    pub fn compute(&self) -> f64 {
        (self.reach * self.impact * self.confidence) / self.effort.max(0.1)
    }
}

/// ICE 分数: Impact × Confidence / Ease
pub struct ICEScore { /* similar */ }

/// MoSCoW 分类
pub enum MoscowClass { MustHave, ShouldHave, CouldHave, WontHave }

/// 统一优先级框架
pub enum PriorityFramework { RICE, ICE, Moscow, Hybrid }

pub struct PriorityEngine {
    pub framework: PriorityFramework,
    pub weights: HashMap<String, f64>,    // 维度加权
    pub history: Vec<PriorityDecision>,
}

impl PriorityEngine {
    pub fn evaluate(&self, goal: &GoalTracker) -> f64;
    pub fn rank(&self, goals: &[GoalTracker]) -> Vec<usize>;
    pub fn with_moscow(goals: &[GoalTracker]) -> HashMap<String, MoscowClass>;
}
```

#### 集成点

| 文件 | 变更 |
|------|------|
| `goal_loop/types.rs` | 添加 `RICEScore`, `ICEScore`, `MoscowClass`, `PriorityFramework`, `PriorityEngine` |
| `goal_loop/loop_impl/core.rs` | `rebalance_from_motivation()` 新增 `PriorityEngine` 调用分支 |
| `goal_loop/loop_impl/core.rs` | `enqueue_goal()` 调用 `priority_engine.evaluate()` 计算初始分 |
| `goal_loop/tracker.rs` | `GoalTracker` 添加 `priority_score: f64` 字段 |
| `goal_loop/mod.rs` | 导出新类型 |

#### 测试策略

- `test_rice_score_computation` — 已知输入验证输出
- `test_ice_vs_rice_ordering` — 框架间排序一致性
- `test_moscow_classification` — 阈值边界测试
- `test_priority_engine_rank` — 多目标排序验证
- `test_hybrid_weighted` — 混合框架加权测试

#### Effort: ~180 行新增 + ~70 行修改

---

### 🟡 B. PRD Writer — 推理引擎新类型

#### 间隙分析

| 当前 | 目标 | 差距 |
|------|------|------|
| `ReasoningType` 5 变体 | +`PrdGeneration` | 1 enum 变体 + 1 handler |
| `reason_*()` 5 个方法 | +`reason_prd()` | ~80 行 handler |
| 无结构化输出约束 | `StructuredOutput` trait + Markdown schema | ~60 行 |
| `context_artifacts.rs` 文档生成 | PRD 模板引擎 | ~50 行模板 |

#### 代码设计

```rust
// === 新增: reasoning_types.rs ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningType {
    Conversation,
    TaskSolving,
    ErrorDebugging,
    KnowledgeQuery,
    General,
    PrdGeneration,     // ← 新变体
}

// === 新增: reasoning_engine/prd.rs ===

pub struct PrdInput {
    pub product_context: String,
    pub user_stories: Vec<String>,
    pub competitive_links: Vec<String>,
    pub design_notes: Option<String>,
}

pub struct PrdOutput {
    pub overview: String,
    pub problem_statement: String,
    pub target_users: Vec<String>,
    pub features: Vec<PrdFeature>,
    pub edge_cases: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub success_metrics: Vec<String>,
    pub risks: Vec<RiskItem>,
}

pub struct PrdFeature {
    pub id: String,
    pub description: String,
    pub priority: MoscowClass,
    pub effort_estimate: String,
}
```

#### 集成点

| 文件 | 变更 |
|------|------|
| `reasoning_brain/reasoning_types.rs` | `ReasoningType` + `PrdGeneration` |
| `reasoning_brain/reasoning_engine/reasoning.rs` | `+reason_prd()` |
| `reasoning_brain/reasoning_engine/engine_core.rs` | `mode_to_reasoning_type()` 新匹配臂 |
| `reasoning_brain/reasoning_engine/mod.rs` | 新子模块注册 |
| `reasoning_brain/reasoning_engine/internal.rs` | `build_context()` 添加 PRD 角色注入 |

#### Effort: ~200 行新增

---

### 🟡 C. Competitive Analyst — SelfEvolver 对比分析

#### 间隙分析

| 当前 | 目标 | 差距 |
|------|------|------|
| `evolve_from_url()` 单仓库 | `compare_repos()` 多仓库 | ~120 行新方法 |
| 三流分析单输出 | 特征矩阵 + 定位差距 | ~100 行新类型 |
| `Accessor` trait 仅 `UrlAccessor` | +`GitHubRepoAccessor` | ~80 行 |
| 无 LLM 驱动语义分析 | `llm_classify()` 已存在 `classifier.rs:try_llm_classify()` | 复用即可 |

#### 代码设计

```rust
// === 新增: self_evolver.rs ===

pub struct ComparisonMatrix {
    pub dimensions: Vec<String>,          // 对比维度
    pub items: Vec<ComparedItem>,          // 每个仓库一行
    pub gap_analysis: Vec<GapRow>,         // 差距行
}

pub struct ComparedItem {
    pub name: String,
    pub scores: Vec<f64>,                  // 每维度 0-1
    pub evidence: Vec<String>,             // 引用证据
}

pub enum GapStatus { Has, Missing, BothMissing, BothPresent }

pub struct GapRow {
    pub dimension: String,
    pub our_status: GapStatus,
    pub their_status: GapStatus,
    pub impact: f64,                       // 差距影响度
    pub recommendation: Option<String>,
}

impl SelfEvolver {
    pub fn compare_repos(&self, urls: &[&str]) -> NeoTrixResult<ComparisonMatrix>;
    pub fn build_comparison_matrix(items: &[ThreeStreamAnalysis]) -> ComparisonMatrix;
    pub fn gap_analysis(matrix: &ComparisonMatrix) -> Vec<GapRow>;
    pub fn positioning_report(&self, matrix: &ComparisonMatrix) -> String;
}
```

#### 集成点

| 文件 | 变更 |
|------|------|
| `self_evolver.rs` | +`compare_repos()`, +`ComparisonMatrix`, +`GapRow` |
| `accessor.rs` | +`GitHubRepoAccessor`（复用 `SelfEvolver::fetch_information` 逻辑） |
| `self_evolver.rs` | `generate_micro_edits` 扩展为支持批量缺口修复 |

#### Effort: ~300 行新增

---

### 🟢 D. Experiment Designer — 实验框架

#### 间隙分析

| 当前 | 目标 | 差距 |
|------|------|------|
| 零实验基础设施 | `Hypothesis` + `ABTestDesign` + `ExperimentRegistry` | ~200 行 |
| `WorldModel::reward_from_knowledge_quality()` 奖励 | 实验成功信号驱动 GoalLoop | ~50 行桥接 |
| 无统计工具 | 样本量估算 + p_value + 效应量 | ~100 行 |

#### 代码设计

```rust
// === 新增: reasoning_brain/experiment.rs ===

pub struct Hypothesis {
    pub id: String,
    pub statement: String,          // "如果 X 则 Y"
    pub null_hypothesis: String,
    pub metrics: Vec<String>,       // 观测指标
    pub expected_effect: f64,       // 最小可检测效应量
    pub confidence_level: f64,      // 默认 0.95
    pub power: f64,                 // 默认 0.80
}

pub struct ABTestDesign {
    pub hypothesis_id: String,
    pub control_description: String,
    pub treatment_description: String,
    pub min_sample_size: u64,       // 基于预期效应量计算
    pub duration_days: u64,
}

pub struct ExperimentResult {
    pub hypothesis_id: String,
    pub p_value: f64,
    pub effect_size: f64,
    pub significant: bool,
    pub control_mean: f64,
    pub treatment_mean: f64,
    pub sample_size: u64,
}

pub struct ExperimentRegistry {
    pub experiments: HashMap<String, ExperimentDesign>,
    pub results: HashMap<String, ExperimentResult>,
    pub active: Vec<String>,
}

impl ExperimentDesigner {
    pub fn design_ab_test(hypothesis: &Hypothesis) -> ABTestDesign;
    pub fn estimate_sample_size(effect: f64, alpha: f64, power: f64) -> u64;
    pub fn analyze_results(control: &[f64], treatment: &[f64]) -> ExperimentResult;
}
```

#### 集成点

| 文件 | 变更 |
|------|------|
| `reasoning_brain/experiment.rs` | 新模块全部 |
| `reasoning_brain/mod.rs` | 注册 `experiment` 子模块 |
| `goal_loop/pursue.rs` | `pursue_iteration()` 检查实验结果 → 成功则推进目标 |
| `world_model.rs` | 实验指标可作为 `PerformanceRecord` 维度 |

#### Effort: ~350 行新增

---

### 🟢 E. UX Critique — CodeReviewEngine 平行模块

#### 间隙分析

| 当前 | 目标 | 差距 |
|------|------|------|
| `CodeReviewEngine` 代码审查 | `UxReviewEngine` UI/UX 审查 | ~200 行平行结构 |
| `ReviewIssue` 代码问题 | `UxIssue` (Accessibility/VisualHierarchy/etc) | ~50 行新类型 |
| `IssueCategory` 仅代码维度 | `UxIssueCategory` 设计维度 | ~30 行 |
| `react_doctor` 7 条规则 | Nielsen 10 原则 + WCAG 基础 | ~120 行启发式实现 |

#### 代码设计

```rust
// === 新增: reasoning_brain/ux_review.rs ===

pub enum UxIssueCategory {
    Accessibility,       // WCAG 合规
    VisualHierarchy,     // 视觉层次
    Consistency,         // 一致性
    Feedback,            // 系统状态可见性
    Affordance,          // 功能可见性
    ErrorPrevention,     // 防错
    Flexibility,         // 灵活高效
    AestheticDesign,     // 美学极简
    Recognition,         // 识别而非回忆
    HelpDocumentation,   // 帮助文档
}

pub struct UxIssue {
    pub category: UxIssueCategory,
    pub severity: IssueSeverity,
    pub heuristic: String,          // 对应 Nielsen 原则
    pub description: String,
    pub suggestion: String,
}

pub struct UxReviewReport {
    pub component: String,
    pub issues: Vec<UxIssue>,
    pub accessibility_score: f64,
    pub visual_score: f64,
    pub consistency_score: f64,
    pub overall_score: f64,
}

pub struct UxReviewEngine {
    pub capability: CapabilityVector,
}

impl UxReviewEngine {
    pub fn review(&self, description: &str) -> UxReviewReport;
    pub fn check_accessibility(&self, desc: &str) -> Vec<UxIssue>;     // WCAG 2.1 A/AA
    pub fn check_visual_hierarchy(&self, desc: &str) -> Vec<UxIssue>;   // 间距/字体/颜色
    pub fn check_consistency(&self, desc: &str) -> Vec<UxIssue>;        // 模式一致性
    pub fn check_nielsen_heuristic(&self, desc: &str, n: usize) -> Vec<UxIssue>;
    pub fn issues_to_micro_edits(&self, report: &UxReviewReport) -> Vec<MicroEdit>;
}
```

#### 集成点

| 文件 | 变更 |
|------|------|
| `reasoning_brain/ux_review.rs` | 新模块全部 |
| `reasoning_brain/mod.rs` | 注册 `ux_review` |
| `mcp_tools.rs` | 注册 `ux_review` MCP 工具 |
| `capability/core.rs` | 确保 `accessibility` 维度映射完整 |

#### Effort: ~350 行新增

---

### 🔴 F. Agentic Workflow — Orchestrator PM/Design 节点

#### 间隙分析

| 当前 | 目标 | 差距 |
|------|------|------|
| `PlannerNode::decompose()` 3 种 plan | +`plan_prd()`, `plan_experiment()`, `plan_ux_audit()`, `plan_competitive_analysis()` | ~120 行 |
| `StateGraph::build_plan()` 固定拓扑 | PM/Design 特定 DAG 拓扑 | ~80 行 |
| `ArtifactType` 7 种 | +`Prd`, `Experiment`, `UxReport`, `JourneyMap`, `CaseStudy` | ~30 行 |
| `/critic.rs` 仅能力向量评估 | +PM/Design 质量门控 | ~100 行 |
| 无流水线串联 | `SessionDistiller → PriorityEngine → GoalLoop → Orchestrator` 全链路 | ~80 行桥接 |

#### 代码设计

```rust
// === 修改: orchestrator/planner.rs ===

impl PlannerNode {
    pub fn plan_prd(&self, goal: &str) -> (Vec<Task>, Vec<ArtifactNode>) {
        // Research → Draft → Review → Refine
    }
    pub fn plan_experiment(&self, goal: &str) -> (Vec<Task>, Vec<ArtifactNode>) {
        // Hypothesis → Design → Execute → Analyze → Report
    }
    pub fn plan_ux_audit(&self, goal: &str) -> (Vec<Task>, Vec<ArtifactNode>) {
        // Heuristic → Accessibility → Visual → Report
    }
    pub fn plan_competitive_analysis(&self, goal: &str) -> (Vec<Task>, Vec<ArtifactNode>) {
        // Fetch → Compare → Gap → Recommend
    }
}

// === 修改: orchestrator/state_graph.rs ===

impl ArtifactType {
    pub const PRD: ArtifactType = ArtifactType::Prd;
    pub const EXPERIMENT: ArtifactType = ArtifactType::Experiment;
    pub const UX_REPORT: ArtifactType = ArtifactType::UxReport;
    // ...
}

impl StateGraph {
    pub fn build_pm_plan(&mut self, goal: &str, pm_type: PMWorkflowType);
    // 不同 PM 工作流生成不同的 DAG 拓扑
}

// === 新增: orchestrator/pm_workflow.rs ===

pub enum PMWorkflowType {
    PrdGeneration,
    CompetitiveAnalysis,
    ExperimentDesign,
    UxAudit,
    RoadmapPlanning,
    LaunchChecklist,
}

pub struct PMNode {
    pub workflow: PMWorkflowType,
    pub quality_gates: Vec<QualityGate>,
}

pub struct QualityGate {
    pub name: String,
    pub check_fn: Box<dyn Fn(&PMNode) -> bool>,
    pub required: bool,
}
```

#### 集成点

| 文件 | 变更 |
|------|------|
| `orchestrator/planner.rs` | +`plan_prd()`, `plan_experiment()`, `plan_ux_audit()`, `plan_competitive_analysis()` |
| `orchestrator/state_graph.rs` | +`ArtifactType` 新变体, `build_pm_plan()` |
| `orchestrator/pm_workflow.rs` | 新模块 |
| `orchestrator/critic.rs` | +PM/Design 质量门控 (PRD 完整性/实验有效性等) |
| `orchestrator/mod.rs` | `Orchestrator` 添加 pm_node 可选字段 |

#### Effort: ~400 行新增 + ~150 行修改

---

## 4. 优先级排序 TODO 清单

### Phase 1 — P0（本周，~540 行）

| ID | 路线 | 任务 | 文件 | Effort | 依赖 |
|----|------|------|------|--------|------|
| **PM-01** | A | 创建 `PriorityEngine` 结构体 + `RICEScore`/`ICEScore`/`MoscowClass` 类型 | `goal_loop/priority.rs` (new) | 80 行 | — |
| **PM-02** | A | `PriorityEngine::evaluate()` + `rank()` 实现 | `goal_loop/priority.rs` | 60 行 | PM-01 |
| **PM-03** | A | `GoalTracker.priority_score` 字段 + `enqueue_goal()` 自动打分 | `goal_loop/tracker.rs`, `core.rs` | 40 行 | PM-02 |
| **PM-04** | A | `rebalance_from_motivation()` 集成 `PriorityEngine` 分支 | `goal_loop/core.rs` | 30 行 | PM-02 |
| **PM-05** | A | 测试: RICE/ICE/MoSCoW/排序/混合加权 | `goal_loop/mod.rs` | 80 行 | PM-04 |
| **PM-06** | F | `PlannerNode` +4 种 PM plan_* 方法 + 测试 | `orchestrator/planner.rs` | 120 行 | — |
| **PM-07** | F | `StateGraph` +`build_pm_plan()` + `ArtifactType` 新变体 | `orchestrator/state_graph.rs` | 80 行 | PM-06 |
| **PM-08** | C | `SelfEvolver::compare_repos()` + `ComparisonMatrix` | `self_evolver.rs` | 120 行 | — |
| **PM-09** | C | `GapRow` 分析 + 定位报告生成 | `self_evolver.rs` | 100 行 | PM-08 |
| **PM-10** | C | 测试: 双仓库对比/差距分析/报告 | `self_evolver.rs` (已有测试模块) | 80 行 | PM-09 |

### Phase 2 — P1（月内，~700 行）

| ID | 路线 | 任务 | 文件 | Effort | 依赖 |
|----|------|------|------|--------|------|
| **PM-11** | B | `ReasoningType::PrdGeneration` + `reason_prd()` | `reasoning_types.rs`, `reasoning.rs` | 80 行 | — |
| **PM-12** | B | `PrdInput`/`PrdOutput` 结构化类型 + 模板引擎 | `reasoning_engine/prd.rs` (new) | 120 行 | PM-11 |
| **PM-13** | B | 测试: PRD 生成/结构化输出/边缘案例覆盖 | `reasoning_engine/tests.rs` | 60 行 | PM-12 |
| **PM-14** | F | `PMWorkflowType` + `PMNode` + `QualityGate` | `orchestrator/pm_workflow.rs` (new) | 100 行 | PM-06 |
| **PM-15** | F | Orchestrator 桥接: PMNode ↔ GoalLoop ↔ PriorityEngine | `orchestrator/mod.rs` | 80 行 | PM-14 |
| **PM-16** | F | 测试: 完整 PM 工作流串联 | `orchestrator/tests.rs` | 100 行 | PM-15 |
| **PM-17** | E | `UxReviewEngine` + `UxIssue` + `UxReviewReport` | `ux_review.rs` (new) | 120 行 | — |
| **PM-18** | E | Nielsen 10 原则实现 + WCAG 基础检查 | `ux_review.rs` | 80 行 | PM-17 |
| **PM-19** | E | 测试: 可访问性/启发式/视觉层次 | `ux_review.rs` (已有测试模块) | 60 行 | PM-18 |

### Phase 3 — P2（季度，~500 行）

| ID | 路线 | 任务 | 文件 | Effort | 依赖 |
|----|------|------|------|--------|------|
| **PM-20** | D | `Hypothesis` + `ABTestDesign` + `ExperimentResult` | `experiment.rs` (new) | 120 行 | — |
| **PM-21** | D | `ExperimentRegistry` + `ExperimentDesigner` | `experiment.rs` | 100 行 | PM-20 |
| **PM-22** | D | 实验→GoalLoop 信号桥接 | `experiment.rs`, `pursue.rs` | 50 行 | PM-21 |
| **PM-23** | D | 测试: 假设验证/样本量/效应量 | `experiment.rs` (测试模块) | 80 行 | PM-22 |
| **PM-24** | C | `GitHubRepoAccessor` 实现 | `accessor.rs` | 80 行 | — |
| **PM-25** | F | 全链路集成测试: FeedbackDistiller→Priority→Roadmap→Launch | `orchestrator/tests.rs` | 100 行 | PM-15 |

---

## 5. 预期交付指标

| 指标 | Phase 1 后 | Phase 2 后 | Phase 3 后 |
|------|-----------|-----------|-----------|
| LOC 新增 | ~710 行 | ~700 行 | ~530 行 |
| 新测试 | ~25 | ~18 | ~15 |
| 新增模块 | `goal_loop/priority.rs` | `prd.rs`, `pm_workflow.rs`, `ux_review.rs` | `experiment.rs` |
| 编译状态 | 0 errors, 0 warnings | 0 errors, 0 warnings | 0 errors, 0 warnings |
| 用户框架覆盖率 | 3/6 技术 + 5/20 技能 | 5/6 技术 + 12/20 技能 | 6/6 技术 + 18/20 技能 |

---

## 6. 全流程质量门控

每条路线完成后验证：

| 门控 | 检查命令 | 通过条件 |
|------|----------|----------|
| 编译 | `cargo check --lib` | 0 error |
| 全 feature | `cargo check --features full --lib` | 0 error |
| 测试 | `cargo test --lib [route_name]` | 全部通过 |
| 未使用变量 | `cargo check --lib` | 0 warning |
| 声称验证 | `git diff --stat` | 仅变更目标文件 |

---

*最后更新: 2026-05-29 | 基于用户 PM/Designer 框架 × NeoTrix 代码库探索结果*
