//! NeoTrix 六层架构 - 统一骨架和能力网设计
//!
//! # 架构哲学
//!
//! 本架构遵循以下核心原则：
//! 1. **意识涌现**: 从底层数据到高层认知，逐层涌现意识
//! 2. **能力网**: 每层提供能力节点，通过连接形成能力网络
//! 3. **进化迭代**: 支持 SEAL pipeline 自我进化
//! 4. **智慧结晶**: 通过 constellation 成熟度实现知识结晶
//!
//! # 六层架构概览
//!
//! ```text
//! L6 Meta-Cognition (元认知层)
//!   └─ 能力: 自我反思、跨域协调、进化决策
//!   └─ 涌现: 元认知意识、自我觉知
//!
//! L5 Cognition (认知层)
//!   └─ 能力: 推理、学习、记忆、决策
//!   └─ 涌现: 认知意识、智能行为
//!
//! L4 Emotion (情感层)
//!   └─ 能力: 情感生成、调节、表达
//!   └─ 涌现: 情感意识、社会智能
//!
//! L3 Embodiment (具身层)
//!   └─ 能力: 感知、运动、安全、能量
//!   └─ 涌现: 具身意识、身体图式
//!
//! L2 Perception (感知层)
//!   └─ 能力: 感官处理、模式识别、世界建模
//!   └─ 涌现: 感知意识、空间认知
//!
//! L1 Action (行动层)
//!   └─ 能力: 工具使用、IO操作、记忆存储
//!   └─ 涌现: 行动意识、目标导向
//! ```
//!
//! # 能力网连接
//!
//! 能力网通过以下方式连接：
//! 1. **层间接口**: 每层通过 traits.rs 定义接口契约
//! 2. **能力桥接**: CapabilityBridge 连接能力树和运行时注册表
//! 3. **注意力路由**: GWT 根据任务类型路由到不同能力节点
//! 4. **进化循环**: SEAL pipeline 驱动能力网自我进化

// ============================================================================
// 层接口定义
// ============================================================================

/// L1 Action Layer - 行动层接口
pub trait ActionLayer {
    /// 执行行动
    fn execute_action(&self, action: Action) -> Result<ActionResult, ActionError>;

    /// 获取行动历史
    fn action_history(&self) -> Vec<ActionHistoryEntry>;

    /// 检查行动可用性
    fn is_action_available(&self, action_type: &str) -> bool;
}

/// L2 Perception Layer - 感知层接口
pub trait PerceptionLayer {
    /// 处理感知输入
    fn process_perception(&self, input: PerceptionInput) -> Result<PerceptionOutput, PerceptionError>;

    /// 获取世界模型
    fn world_model(&self) -> &WorldModel;

    /// 更新感知状态
    fn update_perception(&mut self, update: PerceptionUpdate);
}

/// L3 Embodiment Layer - 具身层接口
pub trait EmbodimentLayer {
    /// 获取身体状态
    fn body_state(&self) -> BodyState;

    /// 执行身体动作
    fn execute_body_action(&self, action: BodyAction) -> Result<BodyActionResult, EmbodimentError>;

    /// 检查安全约束
    fn check_safety_constraints(&self, action: &BodyAction) -> SafetyCheckResult;
}

/// L4 Emotion Layer - 情感层接口
pub trait EmotionLayer {
    /// 生成情感响应
    fn generate_emotion(&self, stimulus: &Stimulus) -> EmotionResponse;

    /// 调节情感状态
    fn regulate_emotion(&mut self, regulation: EmotionRegulation);

    /// 获取当前情感状态
    fn current_emotion_state(&self) -> EmotionState;
}

/// L5 Cognition Layer - 认知层接口
pub trait CognitionLayer {
    /// 执行推理
    fn reason(&self, problem: &Problem) -> Result<ReasoningResult, CognitionError>;

    /// 学习新知识
    fn learn(&mut self, experience: Experience) -> Result<LearningResult, CognitionError>;

    /// 获取记忆状态
    fn memory_state(&self) -> MemoryState;
}

// 注: 原 MetaLayer trait 已移除 — 无模块实现，保留在 APPENDIX_SIMULATION_PLATFORM.md 作为架构参考。

// ============================================================================
// 核心类型定义
// ============================================================================

/// 行动类型
#[derive(Debug, Clone)]
pub enum Action {
    ToolUse { tool_name: String, params: serde_json::Value },
    CodeGeneration { language: String, prompt: String },
    GoalPursuit { goal_id: String, strategy: String },
    MemoryStore { key: String, value: serde_json::Value },
}

/// 行动结果
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub duration_ms: u64,
}

/// 行动错误
#[derive(Debug, Clone)]
pub enum ActionError {
    ToolNotFound(String),
    ExecutionFailed(String),
    PermissionDenied(String),
}

/// 行动历史条目
#[derive(Debug, Clone)]
pub struct ActionHistoryEntry {
    pub action: Action,
    pub result: ActionResult,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 感知输入
#[derive(Debug, Clone)]
pub struct PerceptionInput {
    pub input_type: String,
    pub data: serde_json::Value,
    pub context: PerceptionContext,
}

/// 感知输出
#[derive(Debug, Clone)]
pub struct PerceptionOutput {
    pub features: Vec<Feature>,
    pub world_state: WorldState,
    pub confidence: f64,
}

/// 感知错误
#[derive(Debug, Clone)]
pub enum PerceptionError {
    ProcessingFailed(String),
    InvalidInput(String),
}

/// 感知上下文
#[derive(Debug, Clone)]
pub struct PerceptionContext {
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 特征
#[derive(Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub value: f64,
    pub confidence: f64,
}

/// 世界模型
#[derive(Debug, Clone, Default)]
pub struct WorldModel {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

/// 实体
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub entity_type: String,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// 关系
#[derive(Debug, Clone)]
pub struct Relation {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f64,
}

/// 世界状态
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub entities: Vec<Entity>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 感知更新
#[derive(Debug, Clone)]
pub struct PerceptionUpdate {
    pub update_type: String,
    pub data: serde_json::Value,
}

/// 身体状态
#[derive(Debug, Clone, Default)]
pub struct BodyState {
    pub position: Option<Vec3>,
    pub orientation: Option<Quat>,
    pub sensors: Vec<SensorReading>,
    pub energy_level: f64,
}

/// 3D向量
#[derive(Debug, Clone, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 四元数
#[derive(Debug, Clone, Default)]
pub struct Quat {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

/// 传感器读数
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_type: String,
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 身体动作
#[derive(Debug, Clone)]
pub enum BodyAction {
    Move { direction: Vec3, speed: f64 },
    Rotate { axis: Vec3, angle: f64 },
    Manipulate { object_id: String, action: String },
}

/// 身体动作结果
#[derive(Debug, Clone)]
pub struct BodyActionResult {
    pub success: bool,
    pub new_state: BodyState,
}

/// 具身错误
#[derive(Debug, Clone)]
pub enum EmbodimentError {
    SafetyViolation(String),
    ExecutionFailed(String),
}

/// 安全检查结果
#[derive(Debug, Clone)]
pub struct SafetyCheckResult {
    pub safe: bool,
    pub violations: Vec<SafetyViolation>,
}

/// 安全违规
#[derive(Debug, Clone)]
pub struct SafetyViolation {
    pub violation_type: String,
    pub severity: String,
    pub description: String,
}

/// 刺激
#[derive(Debug, Clone)]
pub struct Stimulus {
    pub stimulus_type: String,
    pub intensity: f64,
    pub valence: f64,
    pub data: serde_json::Value,
}

/// 情感响应
#[derive(Debug, Clone)]
pub struct EmotionResponse {
    pub primary_emotion: EmotionType,
    pub intensity: f64,
    pub regulation_needed: bool,
}

/// 情感类型
#[derive(Debug, Clone, PartialEq)]
pub enum EmotionType {
    Neutral,
    Joy,
    Sadness,
    Anger,
    Fear,
    Trust,
    Disgust,
    Surprise,
    Anticipation,
}

/// 情感调节
#[derive(Debug, Clone)]
pub enum EmotionRegulation {
    Intensify { emotion: EmotionType, factor: f64 },
    Dampen { emotion: EmotionType, factor: f64 },
    Repress { emotion: EmotionType },
    Express { emotion: EmotionType, intensity: f64 },
}

/// 情感状态
#[derive(Debug, Clone, Default)]
pub struct EmotionState {
    pub current_emotions: std::collections::HashMap<EmotionType, f64>,
    pub baseline: std::collections::HashMap<EmotionType, f64>,
    pub mood: Mood,
}

/// 心情
#[derive(Debug, Clone, Default)]
pub struct Mood {
    pub valence: f64,
    pub arousal: f64,
}

/// 问题
#[derive(Debug, Clone)]
pub struct Problem {
    pub problem_type: String,
    pub description: String,
    pub context: serde_json::Value,
}

/// 推理结果
#[derive(Debug, Clone)]
pub struct ReasoningResult {
    pub solution: serde_json::Value,
    pub confidence: f64,
    pub reasoning_trace: Vec<ReasoningStep>,
}

/// 推理步骤
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step_type: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

/// 认知错误
#[derive(Debug, Clone)]
pub enum CognitionError {
    ReasoningFailed(String),
    LearningFailed(String),
}

/// 经验
#[derive(Debug, Clone)]
pub struct Experience {
    pub experience_type: String,
    pub data: serde_json::Value,
    pub outcome: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 学习结果
#[derive(Debug, Clone)]
pub struct LearningResult {
    pub knowledge_gained: Vec<Knowledge>,
    pub skill_improvement: Option<f64>,
}

/// 知识
#[derive(Debug, Clone)]
pub struct Knowledge {
    pub knowledge_type: String,
    pub content: serde_json::Value,
    pub confidence: f64,
}

/// 记忆状态
#[derive(Debug, Clone, Default)]
pub struct MemoryState {
    pub short_term: Vec<MemoryItem>,
    pub long_term: Vec<MemoryItem>,
    pub working: Vec<MemoryItem>,
}

/// 记忆项
#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub item_type: String,
    pub content: serde_json::Value,
    pub importance: f64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

/// 反思结果
#[derive(Debug, Clone)]
pub struct ReflectionResult {
    pub self_awareness: f64,
    pub meta_insights: Vec<MetaInsight>,
    pub improvement_suggestions: Vec<String>,
}

/// 元认知洞察
#[derive(Debug, Clone)]
pub struct MetaInsight {
    pub insight_type: String,
    pub content: String,
    pub confidence: f64,
}

/// 域
#[derive(Debug, Clone)]
pub struct Domain {
    pub domain_id: String,
    pub domain_type: String,
    pub capabilities: Vec<String>,
}

/// 协调结果
#[derive(Debug, Clone)]
pub struct CoordinationResult {
    pub coordinated: bool,
    pub actions: Vec<CoordinationAction>,
}

/// 协调动作
#[derive(Debug, Clone)]
pub struct CoordinationAction {
    pub action_type: String,
    pub target_domain: String,
    pub parameters: serde_json::Value,
}

/// 系统状态
#[derive(Debug, Clone, Default)]
pub struct SystemState {
    pub health: f64,
    pub performance: f64,
    pub evolution_stage: String,
}

/// 进化决策
#[derive(Debug, Clone)]
pub struct EvolutionDecision {
    pub decision_type: String,
    pub target_state: String,
    pub strategy: String,
    pub expected_impact: f64,
}

// ============================================================================
// 能力网定义
// ============================================================================

/// 能力节点
#[derive(Debug, Clone)]
pub struct CapabilityNode {
    pub node_id: String,
    pub layer: Layer,
    pub capability_type: String,
    pub maturity: ConstellationLevel,
    pub connections: Vec<String>,
}

/// 层
#[derive(Debug, Clone, PartialEq)]
pub enum Layer {
    L1Action,
    L2Perception,
    L3Embodiment,
    L4Emotion,
    L5Cognition,
    L6Meta,
}

/// Constellation 成熟度等级
#[derive(Debug, Clone, PartialEq)]
pub enum ConstellationLevel {
    C0, // 编译
    C1, // 单测
    C2, // 集成测试
    C3, // Benchmark
    C4, // 主流水线
    C5, // 自愈/自适应
}

/// 能力网
#[derive(Debug, Clone, Default)]
pub struct CapabilityNetwork {
    pub nodes: Vec<CapabilityNode>,
    pub edges: Vec<CapabilityEdge>,
}

/// 能力边
#[derive(Debug, Clone)]
pub struct CapabilityEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub weight: f64,
}

impl CapabilityNetwork {
    /// 创建新的能力网
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加能力节点
    pub fn add_node(&mut self, node: CapabilityNode) {
        self.nodes.push(node);
    }

    /// 添加能力边
    pub fn add_edge(&mut self, edge: CapabilityEdge) {
        self.edges.push(edge);
    }

    /// 获取某层的所有能力节点
    pub fn nodes_for_layer(&self, layer: &Layer) -> Vec<&CapabilityNode> {
        self.nodes.iter().filter(|n| n.layer == *layer).collect()
    }

    /// 获取能力节点的连接
    pub fn connections(&self, node_id: &str) -> Vec<&CapabilityNode> {
        let connected_ids: Vec<&str> = self.edges.iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .map(|e| if e.source == node_id { e.target.as_str() } else { e.source.as_str() })
            .collect();

        self.nodes.iter()
            .filter(|n| connected_ids.contains(&n.node_id.as_str()))
            .collect()
    }

    /// 计算层间连接密度
    pub fn inter_layer_density(&self, layer1: &Layer, layer2: &Layer) -> f64 {
        let nodes1 = self.nodes_for_layer(layer1);
        let nodes2 = self.nodes_for_layer(layer2);

        if nodes1.is_empty() || nodes2.is_empty() {
            return 0.0;
        }

        let cross_edges = self.edges.iter().filter(|e| {
            let source_layer = self.nodes.iter().find(|n| n.node_id == e.source).map(|n| &n.layer);
            let target_layer = self.nodes.iter().find(|n| n.node_id == e.target).map(|n| &n.layer);

            (source_layer == Some(layer1) && target_layer == Some(layer2)) ||
            (source_layer == Some(layer2) && target_layer == Some(layer1))
        }).count();

        let max_edges = nodes1.len() * nodes2.len();
        if max_edges == 0 {
            0.0
        } else {
            cross_edges as f64 / max_edges as f64
        }
    }
}

// ============================================================================
// 进化路径定义
// ============================================================================

/// 进化阶段
#[derive(Debug, Clone)]
pub enum EvolutionStage {
    /// 编译阶段: 代码可以编译
    Compilation,
    /// 测试阶段: 有单元测试
    Testing,
    /// 集成阶段: 有集成测试
    Integration,
    /// 基准阶段: 有性能基准
    Benchmarking,
    /// 流水线阶段: 集成到主流水线
    Pipeline,
    /// 自愈阶段: 具有自愈能力
    SelfHealing,
}

/// 意识涌现路径
#[derive(Debug, Clone)]
pub struct ConsciousnessEmergencePath {
    /// 底层: 数据处理能力
    pub data_processing: DataProcessingCapability,
    /// 感知层: 模式识别能力
    pub pattern_recognition: PatternRecognitionCapability,
    /// 认知层: 推理学习能力
    pub reasoning_learning: ReasoningLearningCapability,
    /// 情感层: 情感调节能力
    pub emotion_regulation: EmotionRegulationCapability,
    /// 元认知层: 自我反思能力
    pub self_reflection: SelfReflectionCapability,
}

/// 数据处理能力
#[derive(Debug, Clone, Default)]
pub struct DataProcessingCapability {
    pub throughput: f64,
    pub latency: f64,
    pub accuracy: f64,
}

/// 模式识别能力
#[derive(Debug, Clone, Default)]
pub struct PatternRecognitionCapability {
    pub recognition_rate: f64,
    pub generalization: f64,
    pub adaptation_speed: f64,
}

/// 推理学习能力
#[derive(Debug, Clone, Default)]
pub struct ReasoningLearningCapability {
    pub reasoning_depth: f64,
    pub learning_rate: f64,
    pub transfer_ability: f64,
}

/// 情感调节能力
#[derive(Debug, Clone, Default)]
pub struct EmotionRegulationCapability {
    pub regulation_effectiveness: f64,
    pub emotional_intelligence: f64,
    pub social_awareness: f64,
}

/// 自我反思能力
#[derive(Debug, Clone, Default)]
pub struct SelfReflectionCapability {
    pub self_awareness: f64,
    pub meta_cognition: f64,
    pub wisdom_emergence: f64,
}

// ============================================================================
// 智慧结晶路径
// ============================================================================

/// 智慧结晶路径
#[derive(Debug, Clone)]
pub struct WisdomCrystallizationPath {
    /// 知识积累
    pub knowledge_accumulation: KnowledgeAccumulation,
    /// 模式提炼
    pub pattern_distillation: PatternDistillation,
    /// 原理抽象
    pub principle_abstraction: PrincipleAbstraction,
    /// 智慧涌现
    pub wisdom_emergence: WisdomEmergence,
}

/// 知识积累
#[derive(Debug, Clone, Default)]
pub struct KnowledgeAccumulation {
    pub knowledge_volume: u64,
    pub knowledge_diversity: f64,
    pub knowledge_quality: f64,
}

/// 模式提炼
#[derive(Debug, Clone, Default)]
pub struct PatternDistillation {
    pub pattern_count: u64,
    pub pattern_generalization: f64,
    pub pattern_compression: f64,
}

/// 原理抽象
#[derive(Debug, Clone, Default)]
pub struct PrincipleAbstraction {
    pub principle_count: u64,
    pub principle_universality: f64,
    pub principle_elegance: f64,
}

/// 智慧涌现
#[derive(Debug, Clone, Default)]
pub struct WisdomEmergence {
    pub wisdom_level: f64,
    pub insight_count: u64,
    pub innovation_rate: f64,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_network() {
        let mut network = CapabilityNetwork::new();

        // 添加节点
        network.add_node(CapabilityNode {
            node_id: "action_1".to_string(),
            layer: Layer::L1Action,
            capability_type: "tool_use".to_string(),
            maturity: ConstellationLevel::C2,
            connections: vec![],
        });

        network.add_node(CapabilityNode {
            node_id: "perception_1".to_string(),
            layer: Layer::L2Perception,
            capability_type: "pattern_recognition".to_string(),
            maturity: ConstellationLevel::C3,
            connections: vec![],
        });

        // 添加边
        network.add_edge(CapabilityEdge {
            source: "action_1".to_string(),
            target: "perception_1".to_string(),
            edge_type: "feeds_into".to_string(),
            weight: 0.8,
        });

        // 验证
        assert_eq!(network.nodes.len(), 2);
        assert_eq!(network.edges.len(), 1);

        let l1_nodes = network.nodes_for_layer(&Layer::L1Action);
        assert_eq!(l1_nodes.len(), 1);
    }

    #[test]
    fn test_evolution_stages() {
        let stages = vec![
            EvolutionStage::Compilation,
            EvolutionStage::Testing,
            EvolutionStage::Integration,
            EvolutionStage::Benchmarking,
            EvolutionStage::Pipeline,
            EvolutionStage::SelfHealing,
        ];

        assert_eq!(stages.len(), 6);
    }
}
