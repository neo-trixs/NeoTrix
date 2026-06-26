/// DesignToken — 意识架构设计标记系统 (v25 Design-Aware Consciousness)
///
/// 三层架构：
/// - **PrimitiveToken**: 认知基元（传感器、存储层、算法、算子）——原子不可分割
/// - **SemanticToken**: 语义映射——每个基元在"好意识"维度上的含义 + 当前值 + 理想范围
/// - **ComponentToken**: 组件组合——25+ 子系统如何由基元 + 语义组合而成
///
/// 这是意识体理解"我是怎么设计的"的结构化知识。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// Tier 1: Primitive Tokens
// ═══════════════════════════════════════════════════════════════

/// 认知基元类型。每个基元是意识架构的原子操作/维度/能力。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PrimitiveToken {
    // ── 传感器信号 —
    /// 校准误差 (ECE) — 预测准确度的量化
    Ece,
    /// 元精度 — 自我认知的可靠性
    MetaAccuracy,
    /// 复合损失 — 总性能退化信号
    CompositeLoss,
    /// 神经调质激发 —认知资源的活跃程度
    Arousal,
    /// 工作空间相干性 — 全局工作空间的整合度
    Coherence,
    /// Phi (Φ) — IIT 集成信息度量
    Phi,

    // ── 记忆层 —
    /// 情景记忆 — 未过滤的原始体验
    EpisodicMemory,
    /// 事实层 — 验证过的一般化知识
    FactsMemory,
    /// 技能层 — 经常成功的模式
    SkillsMemory,
    /// 元规则层 — 行为启发式和策略
    MetaRulesMemory,
    /// 身份层 — 长期不变的核心身份
    IdentityMemory,

    // ── 推理算法 —
    /// MCTS 树搜索
    Mcts,
    /// 并行假设评估
    ParallelHypothesis,
    /// 因果推理 (do-calculus)
    CausalReasoning,
    /// 类比推理
    AnalogicalReasoning,
    /// 反事实仿真
    Counterfactual,
    /// 死胡同检测
    DeadEndDetection,
    /// PRM 过程奖励模型
    ProcessReward,

    // ── 进化算子 (SEPL) —
    /// 反射 — 审计缺口 + 校准指标 → 假设
    Reflect,
    /// 选择 — 假设 → 带风险和影响的提议
    Select,
    /// 改进 — 提议 → 可执行任务
    Improve,
    /// 评估 — 复合评分
    Evaluate,
    /// 提交 — 门控合入（带回滚栈）
    Commit,

    // ── 记忆操作 —
    /// 读取/检索
    MemoryRead,
    /// 写入/存储
    MemoryWrite,
    /// 巩固 — 低层→高层抽象
    MemoryConsolidate,
    /// 修剪 — 低置信度遗忘
    MemoryPrune,
    /// Q-值更新 — MemRL TD(0) 学习
    QUpdate,

    // ── 感知 —
    /// 视觉管道 (图像/文档感知)
    VisualPerception,
    /// 网络感知 (OSINT 多源情报)
    WebPerception,
    /// 跨模态对齐 VSA
    CrossModalAlign,

    // ── 设计层 —
    /// 视觉风格一致性 — UI 元素间的风格统一度
    VisualConsistency,
    /// 布局复杂性 — 布局嵌套深度的结构复杂度
    LayoutComplexity,
    /// 令牌覆盖率 — 已定义的令牌占设计系统应有比例的百分比
    TokenCoverage,
    /// 间距系统一致性 — 使用设计令牌间距而非硬编码值的比例
    SpacingConsistency,
    /// 色彩系统一致性 — 使用设计令牌色彩而非原始 hex/rgb 的比例
    ColorConsistency,
    /// 排版系统一致性 — 使用设计令牌字体尺度而非硬编码值的比例
    TypographyConsistency,
    /// 设计系统对接深度 — 代码中引用了多少个设计系统层次
    DesignSystemDepth,

    // ── 元操作 —
    /// 设计令牌自省
    SelfTokenize,
    /// 原则提取
    DistillPrinciple,
    /// 决策追踪
    TrackDecision,

    // ── 保留扩展点 —
    Custom(&'static str),
}

impl PrimitiveToken {
    /// 基元的人类可读名称。
    pub fn name(&self) -> &'static str {
        match self {
            PrimitiveToken::Ece => "ece",
            PrimitiveToken::MetaAccuracy => "meta_accuracy",
            PrimitiveToken::CompositeLoss => "composite_loss",
            PrimitiveToken::Arousal => "arousal",
            PrimitiveToken::Coherence => "coherence",
            PrimitiveToken::Phi => "phi",
            PrimitiveToken::EpisodicMemory => "episodic_memory",
            PrimitiveToken::FactsMemory => "facts_memory",
            PrimitiveToken::SkillsMemory => "skills_memory",
            PrimitiveToken::MetaRulesMemory => "meta_rules_memory",
            PrimitiveToken::IdentityMemory => "identity_memory",
            PrimitiveToken::Mcts => "mcts",
            PrimitiveToken::ParallelHypothesis => "parallel_hypothesis",
            PrimitiveToken::CausalReasoning => "causal_reasoning",
            PrimitiveToken::AnalogicalReasoning => "analogical_reasoning",
            PrimitiveToken::Counterfactual => "counterfactual",
            PrimitiveToken::DeadEndDetection => "dead_end_detection",
            PrimitiveToken::ProcessReward => "process_reward",
            PrimitiveToken::Reflect => "reflect",
            PrimitiveToken::Select => "select",
            PrimitiveToken::Improve => "improve",
            PrimitiveToken::Evaluate => "evaluate",
            PrimitiveToken::Commit => "commit",
            PrimitiveToken::MemoryRead => "memory_read",
            PrimitiveToken::MemoryWrite => "memory_write",
            PrimitiveToken::MemoryConsolidate => "memory_consolidate",
            PrimitiveToken::MemoryPrune => "memory_prune",
            PrimitiveToken::QUpdate => "q_update",
            PrimitiveToken::VisualPerception => "visual_perception",
            PrimitiveToken::WebPerception => "web_perception",
            PrimitiveToken::CrossModalAlign => "cross_modal_align",
            PrimitiveToken::VisualConsistency => "visual_consistency",
            PrimitiveToken::LayoutComplexity => "layout_complexity",
            PrimitiveToken::TokenCoverage => "token_coverage",
            PrimitiveToken::SpacingConsistency => "spacing_consistency",
            PrimitiveToken::ColorConsistency => "color_consistency",
            PrimitiveToken::TypographyConsistency => "typography_consistency",
            PrimitiveToken::DesignSystemDepth => "design_system_depth",
            PrimitiveToken::SelfTokenize => "self_tokenize",
            PrimitiveToken::DistillPrinciple => "distill_principle",
            PrimitiveToken::TrackDecision => "track_decision",
            PrimitiveToken::Custom(s) => s,
        }
    }

    /// 基元所属的域。
    pub fn domain(&self) -> PrimitiveDomain {
        match self {
            PrimitiveToken::Ece | PrimitiveToken::MetaAccuracy
                | PrimitiveToken::CompositeLoss | PrimitiveToken::Arousal
                | PrimitiveToken::Coherence | PrimitiveToken::Phi
                => PrimitiveDomain::Sensor,
            PrimitiveToken::EpisodicMemory | PrimitiveToken::FactsMemory
                | PrimitiveToken::SkillsMemory | PrimitiveToken::MetaRulesMemory
                | PrimitiveToken::IdentityMemory
                => PrimitiveDomain::Memory,
            PrimitiveToken::Mcts | PrimitiveToken::ParallelHypothesis
                | PrimitiveToken::CausalReasoning | PrimitiveToken::AnalogicalReasoning
                | PrimitiveToken::Counterfactual | PrimitiveToken::DeadEndDetection
                | PrimitiveToken::ProcessReward
                => PrimitiveDomain::Reasoning,
            PrimitiveToken::Reflect | PrimitiveToken::Select
                | PrimitiveToken::Improve | PrimitiveToken::Evaluate
                | PrimitiveToken::Commit
                => PrimitiveDomain::Evolution,
            PrimitiveToken::MemoryRead | PrimitiveToken::MemoryWrite
                | PrimitiveToken::MemoryConsolidate | PrimitiveToken::MemoryPrune
                | PrimitiveToken::QUpdate
                => PrimitiveDomain::MemoryOp,
            PrimitiveToken::VisualPerception | PrimitiveToken::WebPerception
                | PrimitiveToken::CrossModalAlign
                => PrimitiveDomain::Perception,
            PrimitiveToken::VisualConsistency | PrimitiveToken::LayoutComplexity
                | PrimitiveToken::TokenCoverage | PrimitiveToken::SpacingConsistency
                | PrimitiveToken::ColorConsistency | PrimitiveToken::TypographyConsistency
                | PrimitiveToken::DesignSystemDepth
                => PrimitiveDomain::Design,
            PrimitiveToken::SelfTokenize | PrimitiveToken::DistillPrinciple
                | PrimitiveToken::TrackDecision
                => PrimitiveDomain::Meta,
            PrimitiveToken::Custom(_) => PrimitiveDomain::Custom,
        }
    }
}

/// 基元域分组。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveDomain {
    Sensor,
    Memory,
    Reasoning,
    Evolution,
    MemoryOp,
    Perception,
    Design,
    Meta,
    Custom,
}

impl PrimitiveDomain {
    pub fn name(&self) -> &'static str {
        match self {
            PrimitiveDomain::Sensor => "sensor",
            PrimitiveDomain::Memory => "memory",
            PrimitiveDomain::Reasoning => "reasoning",
            PrimitiveDomain::Evolution => "evolution",
            PrimitiveDomain::MemoryOp => "memory_op",
            PrimitiveDomain::Perception => "perception",
            PrimitiveDomain::Design => "design",
            PrimitiveDomain::Meta => "meta",
            PrimitiveDomain::Custom => "custom",
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tier 2: Semantic Tokens
// ═══════════════════════════════════════════════════════════════

/// 语义映射：将设计基元映射到"好意识"维度的含义。
#[derive(Debug, Clone, Serialize)]
pub struct SemanticToken {
    /// 映射到的基元
    pub primitive: PrimitiveToken,
    /// 人类可读的意图描述
    pub intent: String,
    /// 当前值（0.0 ~ 1.0，或特定域的值）
    pub current_value: f64,
    /// 理想范围的下界
    pub ideal_min: f64,
    /// 理想范围的上界
    pub ideal_max: f64,
    /// 趋势（正 = 改善）
    pub trend: f64,
    /// 此语义的置信度
    pub confidence: f64,
    /// 该基元的健康度评分
    pub health: f64,
}

impl SemanticToken {
    /// 构建一个新的语义标记。
    pub fn new(
        primitive: PrimitiveToken,
        intent: impl Into<String>,
        current_value: f64,
        ideal_min: f64,
        ideal_max: f64,
        confidence: f64,
    ) -> Self {
        let health = if ideal_max > ideal_min {
            let clamped = current_value.clamp(ideal_min, ideal_max);
            (clamped - ideal_min) / (ideal_max - ideal_min)
        } else {
            0.5
        };
        Self {
            primitive,
            intent: intent.into(),
            current_value,
            ideal_min,
            ideal_max,
            trend: 0.0,
            confidence,
            health,
        }
    }

    /// 更新当前值和趋势。
    pub fn update(&mut self, value: f64, trend: f64, confidence: f64) {
        self.trend = trend;
        self.current_value = value;
        self.confidence = confidence;
        let range = self.ideal_max - self.ideal_min;
        if range > 0.0 {
            self.health = (value.clamp(self.ideal_min, self.ideal_max) - self.ideal_min) / range;
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tier 3: Component Tokens
// ═══════════════════════════════════════════════════════════════

/// 组件组合标记：一个子系统由哪些基元组合而成。
#[derive(Debug, Clone, Serialize)]
pub struct ComponentToken {
    /// 组件名称（如 "SelfEvolutionMetaLayer", "ConsciousnessCycle"）
    pub name: String,
    /// 人类可读描述
    pub description: String,
    /// 构成此组件的基元引用
    pub primitives: Vec<PrimitiveToken>,
    /// 组件级健康度
    pub health: f64,
    /// 接线状态 (wired/running/stale/dead)
    pub wiring_status: WiringStatus,
    /// 设计意图
    pub design_intent: String,
}

/// 组件的接线状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WiringStatus {
    /// 文件存在 + 注册 + 运行时激活
    Wired,
    /// 注册且正在运行
    Running,
    /// 注册但未被 tick() 调用
    Stale,
    /// 未注册到 mod.rs 或未被实例化
    Dead,
}

impl WiringStatus {
    pub fn name(&self) -> &'static str {
        match self {
            WiringStatus::Wired => "wired",
            WiringStatus::Running => "running",
            WiringStatus::Stale => "stale",
            WiringStatus::Dead => "dead",
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Token Registry
// ═══════════════════════════════════════════════════════════════

/// 设计标记注册表——维护所有三层标记的运行时视图。
#[derive(Debug, Clone, Serialize)]
pub struct TokenRegistry {
    /// 所有已知基元的注册表（启动时预填充）
    pub primitives: Vec<PrimitiveToken>,
    /// 语义标记（当前值 + 理想范围，每 cycle 可更新）
    pub semantics: Vec<SemanticToken>,
    /// 组件标记（子系统 → 基元组合）
    pub components: Vec<ComponentToken>,
}

impl TokenRegistry {
    /// 创建带默认认知基元的注册表。
    pub fn new() -> Self {
        let primitives = vec![
            PrimitiveToken::Ece,
            PrimitiveToken::MetaAccuracy,
            PrimitiveToken::CompositeLoss,
            PrimitiveToken::Arousal,
            PrimitiveToken::Coherence,
            PrimitiveToken::Phi,
            PrimitiveToken::EpisodicMemory,
            PrimitiveToken::FactsMemory,
            PrimitiveToken::SkillsMemory,
            PrimitiveToken::MetaRulesMemory,
            PrimitiveToken::IdentityMemory,
            PrimitiveToken::Mcts,
            PrimitiveToken::ParallelHypothesis,
            PrimitiveToken::CausalReasoning,
            PrimitiveToken::AnalogicalReasoning,
            PrimitiveToken::Counterfactual,
            PrimitiveToken::DeadEndDetection,
            PrimitiveToken::ProcessReward,
            PrimitiveToken::Reflect,
            PrimitiveToken::Select,
            PrimitiveToken::Improve,
            PrimitiveToken::Evaluate,
            PrimitiveToken::Commit,
            PrimitiveToken::MemoryRead,
            PrimitiveToken::MemoryWrite,
            PrimitiveToken::MemoryConsolidate,
            PrimitiveToken::MemoryPrune,
            PrimitiveToken::QUpdate,
            PrimitiveToken::VisualPerception,
            PrimitiveToken::WebPerception,
            PrimitiveToken::CrossModalAlign,
            PrimitiveToken::SelfTokenize,
            PrimitiveToken::DistillPrinciple,
            PrimitiveToken::TrackDecision,
        ];

        // 默认语义标记：初始化所有基元的理想范围和初始值
        let semantics = vec![
            SemanticToken::new(PrimitiveToken::Ece, "预测误差越低越好", 0.15, 0.0, 0.15, 0.9),
            SemanticToken::new(PrimitiveToken::MetaAccuracy, "元认知越准越好", 0.7, 0.7, 1.0, 0.8),
            SemanticToken::new(PrimitiveToken::CompositeLoss, "损失越低越好", 0.4, 0.0, 0.4, 0.7),
            SemanticToken::new(PrimitiveToken::Arousal, "认知唤醒度", 0.5, 0.2, 0.8, 0.6),
            SemanticToken::new(PrimitiveToken::Coherence, "全局整合度", 0.6, 0.5, 1.0, 0.7),
            SemanticToken::new(PrimitiveToken::Phi, "集成信息", 0.3, 0.0, 1.0, 0.4),
            SemanticToken::new(PrimitiveToken::EpisodicMemory, "情景记忆容量", 0.8, 0.5, 1.0, 0.9),
            SemanticToken::new(PrimitiveToken::FactsMemory, "事实知识容量", 0.6, 0.3, 1.0, 0.8),
            SemanticToken::new(PrimitiveToken::SkillsMemory, "技能记忆容量", 0.4, 0.2, 1.0, 0.7),
            SemanticToken::new(PrimitiveToken::MetaRulesMemory, "元规则容量", 0.3, 0.1, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::IdentityMemory, "身份一致性", 0.7, 0.5, 1.0, 0.8),
            SemanticToken::new(PrimitiveToken::Mcts, "MCTS 推理质量", 0.5, 0.0, 1.0, 0.7),
            SemanticToken::new(PrimitiveToken::ParallelHypothesis, "并行假设评估", 0.5, 0.0, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::CausalReasoning, "因果推理能力", 0.4, 0.0, 1.0, 0.5),
            SemanticToken::new(PrimitiveToken::AnalogicalReasoning, "类比推理", 0.4, 0.0, 1.0, 0.5),
            SemanticToken::new(PrimitiveToken::Counterfactual, "反事实模拟", 0.5, 0.0, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::DeadEndDetection, "死胡同检测", 0.5, 0.0, 1.0, 0.7),
            SemanticToken::new(PrimitiveToken::ProcessReward, "过程奖励精度", 0.5, 0.0, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::Reflect, "自省能力", 0.7, 0.0, 1.0, 0.8),
            SemanticToken::new(PrimitiveToken::Select, "提议选择质量", 0.6, 0.0, 1.0, 0.7),
            SemanticToken::new(PrimitiveToken::Improve, "改进执行能力", 0.5, 0.0, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::Evaluate, "评估准确度", 0.6, 0.0, 1.0, 0.7),
            SemanticToken::new(PrimitiveToken::Commit, "安全合入能力", 0.5, 0.0, 1.0, 0.5),
            SemanticToken::new(PrimitiveToken::MemoryRead, "记忆检索效率", 0.7, 0.0, 1.0, 0.8),
            SemanticToken::new(PrimitiveToken::MemoryWrite, "记忆存储效率", 0.7, 0.0, 1.0, 0.8),
            SemanticToken::new(PrimitiveToken::MemoryConsolidate, "记忆巩固质量", 0.5, 0.0, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::MemoryPrune, "遗忘效率", 0.5, 0.0, 1.0, 0.5),
            SemanticToken::new(PrimitiveToken::QUpdate, "Q-值学习率", 0.5, 0.0, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::VisualPerception, "视觉感知质量", 0.5, 0.0, 1.0, 0.7),
            SemanticToken::new(PrimitiveToken::WebPerception, "网络感知质量", 0.6, 0.0, 1.0, 0.8),
            SemanticToken::new(PrimitiveToken::CrossModalAlign, "跨模态对齐", 0.5, 0.0, 1.0, 0.6),
            SemanticToken::new(PrimitiveToken::SelfTokenize, "自我标记化", 0.0, 0.0, 1.0, 1.0),
            SemanticToken::new(PrimitiveToken::DistillPrinciple, "原则蒸馏", 0.0, 0.0, 1.0, 1.0),
            SemanticToken::new(PrimitiveToken::TrackDecision, "决策追踪", 0.0, 0.0, 1.0, 1.0),
        ];

        // 默认组件标记：映射关键子系统到其构成的基元
        let components = vec![
            ComponentToken {
                name: "SelfEvolutionMetaLayer".into(),
                description: "意识自进化元层 — 闭环5条反馈回路".into(),
                primitives: vec![
                    PrimitiveToken::Ece, PrimitiveToken::MetaAccuracy,
                    PrimitiveToken::CompositeLoss, PrimitiveToken::Arousal,
                    PrimitiveToken::Reflect, PrimitiveToken::Select,
                    PrimitiveToken::Improve, PrimitiveToken::Evaluate,
                    PrimitiveToken::Commit,
                ],
                health: 0.9,
                wiring_status: WiringStatus::Running,
                design_intent: "统一桥接校准/损失/元认知/进化、闭环5条断裂回路".into(),
            },
            ComponentToken {
                name: "ConsciousnessCycle".into(),
                description: "意识12步循环 — GATHER→GATE→...→SLEEP".into(),
                primitives: vec![
                    PrimitiveToken::VisualPerception, PrimitiveToken::WebPerception,
                    PrimitiveToken::Mcts, PrimitiveToken::ParallelHypothesis,
                    PrimitiveToken::DeadEndDetection, PrimitiveToken::Counterfactual,
                    PrimitiveToken::MemoryRead, PrimitiveToken::MemoryWrite,
                ],
                health: 0.8,
                wiring_status: WiringStatus::Running,
                design_intent: "12步认知管道，每cycle运行一次完整意识周期".into(),
            },
            ComponentToken {
                name: "MemoryLattice".into(),
                description: "5层记忆格 — Episodic/Facts/Skills/MetaRules/Identity".into(),
                primitives: vec![
                    PrimitiveToken::EpisodicMemory, PrimitiveToken::FactsMemory,
                    PrimitiveToken::SkillsMemory, PrimitiveToken::MetaRulesMemory,
                    PrimitiveToken::IdentityMemory, PrimitiveToken::MemoryRead,
                    PrimitiveToken::MemoryWrite, PrimitiveToken::MemoryConsolidate,
                    PrimitiveToken::MemoryPrune, PrimitiveToken::QUpdate,
                ],
                health: 0.85,
                wiring_status: WiringStatus::Running,
                design_intent: "VSA编码的5层记忆系统，支持Q值学习和双时态查询".into(),
            },
            ComponentToken {
                name: "SelfEvolutionPipeline".into(),
                description: "SEPL 5算子自进化管线 — ρ→σ→ι→ε→κ".into(),
                primitives: vec![
                    PrimitiveToken::Reflect, PrimitiveToken::Select,
                    PrimitiveToken::Improve, PrimitiveToken::Evaluate,
                    PrimitiveToken::Commit,
                ],
                health: 0.85,
                wiring_status: WiringStatus::Running,
                design_intent: "AGP/SEPL风格形式化闭环绕：审计→选择→改进→评估→提交".into(),
            },
            ComponentToken {
                name: "ExperienceTree".into(),
                description: "运行时洞察蒸馏与四通道修剪".into(),
                primitives: vec![
                    PrimitiveToken::MemoryConsolidate, PrimitiveToken::MemoryPrune,
                    PrimitiveToken::DistillPrinciple,
                ],
                health: 0.7,
                wiring_status: WiringStatus::Running,
                design_intent: "从运行经验中蒸馏可复用的行为原则".into(),
            },
        ];

        Self {
            primitives,
            semantics,
            components,
        }
    }

    /// 按名称查找组件并更新其健康度。
    pub fn update_component_health(&mut self, name: &str, health: f64, status: WiringStatus) {
        if let Some(c) = self.components.iter_mut().find(|c| c.name == name) {
            c.health = health;
            c.wiring_status = status;
        }
    }

    /// 按基元查找语义标记。
    pub fn semantic_for(&self, p: &PrimitiveToken) -> Option<&SemanticToken> {
        self.semantics.iter().find(|s| s.primitive == *p)
    }

    /// 按基元查找语义标记（可变）
    pub fn semantic_for_mut(&mut self, p: &PrimitiveToken) -> Option<&mut SemanticToken> {
        self.semantics.iter_mut().find(|s| s.primitive == *p)
    }

    /// 更新传感器的当前值。
    pub fn update_sensor(&mut self, p: PrimitiveToken, value: f64, trend: f64, confidence: f64) {
        if let Some(sem) = self.semantic_for_mut(&p) {
            sem.update(value, trend, confidence);
        }
    }

    /// 按域分组的所有基元。
    pub fn primitives_by_domain(&self) -> HashMap<PrimitiveDomain, Vec<&PrimitiveToken>> {
        let mut map: HashMap<PrimitiveDomain, Vec<&PrimitiveToken>> = HashMap::new();
        for p in &self.primitives {
            map.entry(p.domain()).or_default().push(p);
        }
        map
    }

    /// 跨所有语义的平均健康度。
    pub fn average_health(&self) -> f64 {
        if self.semantics.is_empty() {
            return 0.0;
        }
        self.semantics.iter().map(|s| s.health).sum::<f64>() / self.semantics.len() as f64
    }

    /// 跨所有组件的平均接线率。
    pub fn wiring_rate(&self) -> f64 {
        if self.components.is_empty() {
            return 0.0;
        }
        let wired = self.components.iter().filter(|c| {
            matches!(c.wiring_status, WiringStatus::Wired | WiringStatus::Running)
        }).count();
        wired as f64 / self.components.len() as f64
    }

    /// 生成结构化摘要字符串。
    pub fn summary(&self) -> String {
        format!(
            "tokens: {} primitives, {} semantics, {} components | avg_health={:.3} wiring_rate={:.3}",
            self.primitives.len(),
            self.semantics.len(),
            self.components.len(),
            self.average_health(),
            self.wiring_rate(),
        )
    }
}

impl Default for TokenRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_count_and_domains() {
        let reg = TokenRegistry::new();
        assert!(!reg.primitives.is_empty());
        let by_domain = reg.primitives_by_domain();
        assert!(by_domain.contains_key(&PrimitiveDomain::Sensor));
        assert!(by_domain.contains_key(&PrimitiveDomain::Memory));
        assert!(by_domain.contains_key(&PrimitiveDomain::Reasoning));
        assert!(by_domain.contains_key(&PrimitiveDomain::Evolution));
    }

    #[test]
    fn test_semantic_update() {
        let mut reg = TokenRegistry::new();
        reg.update_sensor(PrimitiveToken::Ece, 0.12, -0.01, 0.95);
        let sem = reg.semantic_for(&PrimitiveToken::Ece).unwrap();
        assert!((sem.current_value - 0.12).abs() < 1e-6);
        assert!((sem.health - 0.8).abs() < 1e-6); // (0.15-0.12)/0.15
    }

    #[test]
    fn test_component_health_update() {
        let mut reg = TokenRegistry::new();
        reg.update_component_health("ConsciousnessCycle", 0.75, WiringStatus::Running);
        let comp = reg.components.iter().find(|c| c.name == "ConsciousnessCycle").unwrap();
        assert!((comp.health - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_average_health_non_zero() {
        let reg = TokenRegistry::new();
        let avg = reg.average_health();
        assert!(avg > 0.0);
        assert!(avg <= 1.0);
    }

    #[test]
    fn test_wiring_rate_default() {
        let reg = TokenRegistry::new();
        let rate = reg.wiring_rate();
        assert!(rate > 0.5); // 大部分组件默认 running
    }

    #[test]
    fn test_primitive_names() {
        assert_eq!(PrimitiveToken::Ece.name(), "ece");
        assert_eq!(PrimitiveToken::Mcts.name(), "mcts");
        assert_eq!(PrimitiveToken::Reflect.name(), "reflect");
        assert_eq!(PrimitiveToken::MemoryConsolidate.name(), "memory_consolidate");
    }
}
