//! 能力网核心类型定义
//! 
//! 定义6层架构的能力类型系统

use serde::{Deserialize, Serialize};
use std::fmt;

/// 6层架构层级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layer {
    /// L1: 行动层 - 资源获取、执行、IO
    L1Action,
    /// L2: 感知层 - 环境感知、感官处理
    L2Perception,
    /// L3: 具身层 - 物理实体、安全、情感具身
    L3Embodiment,
    /// L4: 情感层 - 情感标签、情感分析
    L4Emotion,
    /// L5: 认知层 - 推理、决策、学习
    L5Cognition,
    /// L6: 元认知层 - 监控、修复、进化
    L6MetaCognition,
}

impl Layer {
    /// 获取层级编号
    pub fn number(&self) -> u8 {
        match self {
            Layer::L1Action => 1,
            Layer::L2Perception => 2,
            Layer::L3Embodiment => 3,
            Layer::L4Emotion => 4,
            Layer::L5Cognition => 5,
            Layer::L6MetaCognition => 6,
        }
    }

    /// 获取层级名称
    pub fn name(&self) -> &'static str {
        match self {
            Layer::L1Action => "Action",
            Layer::L2Perception => "Perception",
            Layer::L3Embodiment => "Embodiment",
            Layer::L4Emotion => "Emotion",
            Layer::L5Cognition => "Cognition",
            Layer::L6MetaCognition => "Meta-Cognition",
        }
    }

    /// 获取层级中文名称
    pub fn name_cn(&self) -> &'static str {
        match self {
            Layer::L1Action => "行动层",
            Layer::L2Perception => "感知层",
            Layer::L3Embodiment => "具身层",
            Layer::L4Emotion => "情感层",
            Layer::L5Cognition => "认知层",
            Layer::L6MetaCognition => "元认知层",
        }
    }

    /// 所有层级
    pub fn all() -> &'static [Layer] {
        &[
            Layer::L1Action,
            Layer::L2Perception,
            Layer::L3Embodiment,
            Layer::L4Emotion,
            Layer::L5Cognition,
            Layer::L6MetaCognition,
        ]
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L{} {}", self.number(), self.name())
    }
}

/// 能力类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityKind {
    // L1 Action 能力
    Download,
    Upload,
    Execute,
    IO,
    
    // L2 Perception 能力
    Sense,
    Classify,
    Recognize,
    Perceive,
    
    // L3 Embodiment 能力
    Physical,
    Security,
    EmotionalEmbodiment,
    
    // L4 Emotion 能力
    EmotionLabel,
    EmotionAnalysis,
    EmotionExpression,
    
    // L5 Cognition 能力
    Reason,
    Decide,
    Learn,
    Plan,
    
    // L6 Meta-Cognition 能力
    Monitor,
    Repair,
    Evolve,
    MetaCognition,
}

impl CapabilityKind {
    /// 获取能力类型所属层级
    pub fn layer(&self) -> Layer {
        match self {
            CapabilityKind::Download | CapabilityKind::Upload | 
            CapabilityKind::Execute | CapabilityKind::IO => Layer::L1Action,
            
            CapabilityKind::Sense | CapabilityKind::Classify | 
            CapabilityKind::Recognize | CapabilityKind::Perceive => Layer::L2Perception,
            
            CapabilityKind::Physical | CapabilityKind::Security | 
            CapabilityKind::EmotionalEmbodiment => Layer::L3Embodiment,
            
            CapabilityKind::EmotionLabel | CapabilityKind::EmotionAnalysis | 
            CapabilityKind::EmotionExpression => Layer::L4Emotion,
            
            CapabilityKind::Reason | CapabilityKind::Decide | 
            CapabilityKind::Learn | CapabilityKind::Plan => Layer::L5Cognition,
            
            CapabilityKind::Monitor | CapabilityKind::Repair | 
            CapabilityKind::Evolve | CapabilityKind::MetaCognition => Layer::L6MetaCognition,
        }
    }
}

impl fmt::Display for CapabilityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            CapabilityKind::Download => "Download",
            CapabilityKind::Upload => "Upload",
            CapabilityKind::Execute => "Execute",
            CapabilityKind::IO => "IO",
            CapabilityKind::Sense => "Sense",
            CapabilityKind::Classify => "Classify",
            CapabilityKind::Recognize => "Recognize",
            CapabilityKind::Perceive => "Perceive",
            CapabilityKind::Physical => "Physical",
            CapabilityKind::Security => "Security",
            CapabilityKind::EmotionalEmbodiment => "EmotionalEmbodiment",
            CapabilityKind::EmotionLabel => "EmotionLabel",
            CapabilityKind::EmotionAnalysis => "EmotionAnalysis",
            CapabilityKind::EmotionExpression => "EmotionExpression",
            CapabilityKind::Reason => "Reason",
            CapabilityKind::Decide => "Decide",
            CapabilityKind::Learn => "Learn",
            CapabilityKind::Plan => "Plan",
            CapabilityKind::Monitor => "Monitor",
            CapabilityKind::Repair => "Repair",
            CapabilityKind::Evolve => "Evolve",
            CapabilityKind::MetaCognition => "MetaCognition",
        };
        write!(f, "{}", name)
    }
}

/// 能力成熟度级别 (C0-C6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MaturityLevel {
    /// C0: 编译通过
    C0,
    /// C1: 单元测试通过
    C1,
    /// C2: 集成测试通过
    C2,
    /// C3: 性能基准测试通过
    C3,
    /// C4: 集成到主流水线
    C4,
    /// C5: 自愈/自适应
    C5,
    /// C6: 涌现新能力
    C6,
}

impl MaturityLevel {
    /// 获取成熟度编号
    pub fn number(&self) -> u8 {
        match self {
            MaturityLevel::C0 => 0,
            MaturityLevel::C1 => 1,
            MaturityLevel::C2 => 2,
            MaturityLevel::C3 => 3,
            MaturityLevel::C4 => 4,
            MaturityLevel::C5 => 5,
            MaturityLevel::C6 => 6,
        }
    }
}

impl fmt::Display for MaturityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{}", self.number())
    }
}

/// 能力向量 - 表示能力的多维特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityVector {
    /// 能力强度 (0.0-1.0)
    pub strength: f64,
    /// 能力效率 (0.0-1.0)
    pub efficiency: f64,
    /// 能力可靠性 (0.0-1.0)
    pub reliability: f64,
    /// 能力适应性 (0.0-1.0)
    pub adaptability: f64,
    /// 能力创造性 (0.0-1.0)
    pub creativity: f64,
}

impl CapabilityVector {
    /// 创建新的能力向量
    pub fn new() -> Self {
        Self {
            strength: 0.5,
            efficiency: 0.5,
            reliability: 0.5,
            adaptability: 0.5,
            creativity: 0.5,
        }
    }

    /// 计算综合能力分数
    pub fn overall_score(&self) -> f64 {
        (self.strength + self.efficiency + self.reliability + 
         self.adaptability + self.creativity) / 5.0
    }
}

impl Default for CapabilityVector {
    fn default() -> Self {
        Self::new()
    }
}

/// 能力成本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCost {
    /// CPU 使用量 (毫秒)
    pub cpu_ms: u64,
    /// 内存使用量 (字节)
    pub memory_bytes: u64,
    /// 网络使用量 (字节)
    pub network_bytes: u64,
    /// Token 使用量
    pub tokens: u64,
}

impl CapabilityCost {
    /// 创建零成本
    pub fn zero() -> Self {
        Self {
            cpu_ms: 0,
            memory_bytes: 0,
            network_bytes: 0,
            tokens: 0,
        }
    }
}

/// 能力统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStats {
    /// 调用次数
    pub call_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 平均执行时间 (毫秒)
    pub avg_execution_time_ms: f64,
    /// 最后使用时间戳
    pub last_used_at: Option<i64>,
}

impl CapabilityStats {
    /// 创建空统计
    pub fn new() -> Self {
        Self {
            call_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_execution_time_ms: 0.0,
            last_used_at: None,
        }
    }

    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.call_count as f64
        }
    }
}

impl Default for CapabilityStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 智慧类型 - 从能力涌现而来
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Wisdom {
    /// 行动智慧 - 从行动能力涌现
    Action {
        capability_id: String,
        insight: String,
        strength: f64,
    },
    /// 感知智慧 - 从感知能力涌现
    Perception {
        capability_id: String,
        pattern: String,
        confidence: f64,
    },
    /// 情感智慧 - 从情感能力涌现
    Emotion {
        capability_id: String,
        emotion: String,
        intensity: f64,
    },
    /// 认知智慧 - 从认知能力涌现
    Cognition {
        capability_id: String,
        reasoning: String,
        depth: u32,
    },
    /// 元认知智慧 - 从元认知能力涌现
    MetaCognition {
        capability_id: String,
        reflection: String,
        insight: String,
    },
}

impl Wisdom {
    /// 获取智慧所属层级
    pub fn layer(&self) -> Layer {
        match self {
            Wisdom::Action { .. } => Layer::L1Action,
            Wisdom::Perception { .. } => Layer::L2Perception,
            Wisdom::Emotion { .. } => Layer::L4Emotion,
            Wisdom::Cognition { .. } => Layer::L5Cognition,
            Wisdom::MetaCognition { .. } => Layer::L6MetaCognition,
        }
    }

    /// 获取智慧强度
    pub fn strength(&self) -> f64 {
        match self {
            Wisdom::Action { strength, .. } => *strength,
            Wisdom::Perception { confidence, .. } => *confidence,
            Wisdom::Emotion { intensity, .. } => *intensity,
            Wisdom::Cognition { depth, .. } => *depth as f64 / 10.0,
            Wisdom::MetaCognition { .. } => 0.8,
        }
    }
}

/// 意识状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    /// 当前激活的层级
    pub active_layers: Vec<Layer>,
    /// 智慧池容量
    pub wisdom_pool_size: usize,
    /// 意识树健康度
    pub consciousness_tree_health: f64,
    /// SEAL 管线状态
    pub seal_pipeline_status: String,
}

impl ConsciousnessState {
    /// 创建初始状态
    pub fn initial() -> Self {
        Self {
            active_layers: vec![],
            wisdom_pool_size: 0,
            consciousness_tree_health: 1.0,
            seal_pipeline_status: "Idle".to_string(),
        }
    }
}

/// 插件事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// 配置变更
    ConfigChanged,
    /// 会话开始
    SessionStarted,
    /// 会话结束
    SessionEnded,
    /// 任务接收
    TaskReceived(String),
    /// 任务完成
    TaskCompleted(String),
    /// 能力注册
    CapabilityRegistered(String),
    /// 能力注销
    CapabilityUnregistered(String),
    /// 智慧产生
    WisdomGenerated(Wisdom),
}

/// 插件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    /// 未加载
    Unloaded,
    /// 已加载
    Loaded,
    /// 运行中
    Running,
    /// 错误
    Error(String),
    /// 已禁用
    Disabled,
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件作者
    pub author: String,
    /// 插件描述
    pub description: String,
    /// 插件标签
    pub tags: Vec<String>,
    /// 插件状态
    pub status: PluginStatus,
}

impl PluginInfo {
    /// 创建新的插件信息
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            author: String::new(),
            description: String::new(),
            tags: vec![],
            status: PluginStatus::Unloaded,
        }
    }
}
