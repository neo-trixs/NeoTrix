//! # NeoTrix Core Protocol Traits
//!
//! 核心接口定义 + L0 共享类型，解耦各层之间的直接引用。
//! 所有实现都在各自层中，traits 本身在 core 层。

use crate::core::nt_core_bank::{ReasoningBankStats, ReasoningMemory};
use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_knowledge::{AbsorptionRecord, KnowledgeSource};
use serde::{Deserialize, Serialize};

pub use neotrix_types::core::nt_core_traits::{NativeTool, ToolDef, ToolOutput, ToolProvider};

pub use crate::core::nt_core_self_test::SelfTest;

/// Rune Socket — 每模块 5 色符文槽位配置 (数据/变换/缓存/错误恢复/监控)。
/// 定义在 `nt_core_capability_tree` (node.rs), 此处 re-export 供 core 层引用。
pub use nt_core_capability_tree::RuneSocket;

/// CapabilityNode — 能力树节点抽象 trait。
/// 实现者声明自身在能力树中的坐标 (provides/requires)、符文槽位与星座成熟度。
/// 注意: `nt_core_capability_tree::CapabilityNode` 是注册表数据 struct,
/// 此 trait 是能力节点实现方 (l8/l9/l10) 向 core 层声明节点契约的接口。
pub trait CapabilityNode: Send + Sync {
    /// 节点唯一标识 (域::模块::实例路径)
    fn node_id(&self) -> &str;
    /// 该节点提供的能力标识符列表
    fn provides(&self) -> Vec<String>;
    /// 该节点运行所需的上游能力标识符列表
    fn requires(&self) -> Vec<String>;
    /// 符文槽位配置 (模块级 rune socketing)
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        Vec::new()
    }
    /// 星座成熟度 (C0-C6, 返回 0-6)
    fn constellation_level(&self) -> u8 {
        0
    }
    /// 尝试晋升星座成熟度一级
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

/// L0 共享专型枚举：GWT 专家模块类型 / SEAL 阶段路由 / PRM 过程奖励评分。
/// 定义在 L0 以防止 L4→L5 反向依赖。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialistType {
    PatternMatcher,
    AnomalyDetector,
    KnowledgeRetriever,
    CodeAnalyzer,
    Planner,
    KnowledgeIntegrator,
    GoalPrioritizer,
    RiskAssessor,
    CreativityGenerator,
    ReflectionEngine,
    MetaCognitionAnalyst,
    AISecurity,
    ImageGenerator,
    EvidenceWeightedHypothesis,
    Orchestrator,
}

/// MemoryProvider — 记忆存储/检索抽象
pub trait MemoryProvider {
    fn store(&mut self, key: &str, value: &str) -> Result<String, String>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, String)>, String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
}

/// RichMemoryProvider — 针对 ReasoningBank 的完整记忆抽象
pub trait RichMemoryProvider: Send + Sync {
    fn store_memory(&mut self, memory: ReasoningMemory) -> bool;
    fn recall_similar(&self, query: &str, limit: usize) -> Vec<ReasoningMemory>;
    fn stats(&self) -> ReasoningBankStats;
}

/// AgentExecutor — Agent 执行抽象
pub trait AgentExecutor {
    type Output;
    fn execute(&mut self, task: &str) -> Result<Self::Output, String>;
    fn interrupt(&mut self) -> Result<(), String>;
    fn status(&self) -> String;
    fn capability(&self) -> &CapabilityVector;
    fn capability_mut(&mut self) -> &mut CapabilityVector;
}

/// SessionProvider — 会话管理抽象
pub trait SessionProvider {
    type Session;
    fn create_session(&mut self, id: &str, name: &str) -> Self::Session;
    fn switch_session(&mut self, id: &str) -> bool;
    fn active_session(&self) -> Option<&Self::Session>;
    fn list_sessions(&self) -> Vec<&Self::Session>;
}

/// KnowledgeProvider re-export (defined in knowledge.rs)
pub use super::nt_core_knowledge::KnowledgeProvider;

/// KnowledgeSink — 知识写入抽象 (L2 感知层对 L1 知识层的写入接口)
///
/// L2 模块通过此 trait 写入知识，而非直接依赖 L1 KnowledgeBase 具体类型。
/// 实现者: L1 KnowledgeBase (via blanket or manual impl).
pub trait KnowledgeSink: Send + Sync {
    /// 插入或获取节点，返回节点 ID
    fn sink_node(
        &self,
        title: &str,
        node_type: crate::core::nt_core_kb_types::NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String>;

    /// 插入或更新边
    fn sink_edge(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: crate::core::nt_core_kb_types::RelationType,
        weight: f64,
        description: Option<&str>,
    ) -> Result<(), String>;

    /// 插入带元数据的边
    fn sink_edge_with_metadata(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: crate::core::nt_core_kb_types::RelationType,
        weight: f64,
        description: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), String>;

    /// 检查边是否存在
    fn edge_exists(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: crate::core::nt_core_kb_types::RelationType,
    ) -> Result<bool, String>;

    /// 更新节点元数据
    fn update_node_metadata(
        &self,
        node_id: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String>;
}

/// SealResult — SEAL 自迭代循环的结果
#[derive(Debug, Clone)]
pub struct SealResult {
    pub score_before: f64,
    pub score_after: f64,
    pub delta: f64,
    pub iterations: usize,
}

/// BrainProvider — 推理大脑抽象
pub trait BrainProvider: Send + Sync {
    fn capability_vector(&self) -> CapabilityVector;
    fn absorb_knowledge(&mut self, source: KnowledgeSource) -> AbsorptionRecord;
    fn run_seal_iteration(&mut self) -> SealResult;
    fn total_absorb_count(&self) -> u64 {
        0
    }
    fn absorb(&mut self, source: KnowledgeSource) {
        self.absorb_knowledge(source);
    }
    fn register_knowledge_source(&mut self, _name: &str, _vector: CapabilityVector) {}
    fn absorb_from_custom(&mut self, _name: &str) -> bool {
        false
    }
}

/// SkillRunner — L8 SkillEngine 的 L1 抽象接口，避免 L1→L8 直接依赖
pub trait SkillRunner: Send + Sync {
    fn run_skill(&mut self, name: &str) -> Result<String, String>;
    fn has_skill(&self, name: &str) -> bool;
}

/// EngineProvider — 推理引擎抽象
pub trait EngineProvider: Send + Sync {
    fn reason(&mut self, prompt: &str) -> Result<String, String>;
}

/// SecurityAudit — 安全审计抽象 (L5 认知层对 L3 具身层的安全检查接口)
///
/// L5 模块通过此 trait 触发安全扫描，而非直接依赖 L3 具体类型。
pub trait SecurityAudit: Send + Sync {
    /// 执行浏览器安全扫描，返回扫描结果摘要
    fn scan_browser_security(&self, url: &str) -> Result<String, String>;
    /// 执行推理轨迹防护扫描
    fn scan_reasoning_trace(&self, text: &str, context: &str) -> Result<ReasoningTraceReport, String>;
}

/// 推理轨迹防护扫描报告
#[derive(Debug, Clone, Default)]
pub struct ReasoningTraceReport {
    pub session_binding_missing: usize,
    pub pii_findings: Vec<String>,
    pub injection_findings: Vec<String>,
    pub divergence_suspected: bool,
}

/// SecurityCheckRegistry — 安全检查注册表抽象
pub trait SecurityCheckRegistry: Send + Sync {
    /// 注册一个安全检查项
    fn register_check(&mut self, name: &str);
    /// 运行所有已注册的安全检查，返回通过/失败
    fn run_all_checks(&self) -> Result<Vec<String>, Vec<String>>;
}

// ════════════════════════════════════════════════════════════════
// L3 Shield 抽象 — 避免 L1→L3 直接依赖
// ════════════════════════════════════════════════════════════════

/// SecretRiskLevel — 脱敏风险等级 (与 l3_embodiment::nt_shield::redaction::RiskLevel 同构)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretRiskLevel {
    Safe,
    Suspicious,
    Dangerous,
}

/// SecretScanner — 密钥/PII 扫描抽象
/// L1 模块通过此 trait 执行凭据泄漏检测，而非直接依赖 L3 Redactor。
pub trait SecretScanner: Send + Sync {
    /// 分析文本风险等级 + 命中的规则描述
    fn analyze(&self, text: &str) -> (SecretRiskLevel, Vec<String>);
    /// 是否安全 (无可脱敏内容)
    fn is_safe(&self, text: &str) -> bool;
}

/// PropagationGuardLike — 心智病毒传播防护抽象
/// L1 模块通过此 trait 执行传播防护，而非直接依赖 L3 PropagationGuard。
pub trait PropagationGuardLike: Send + Sync {
    /// 是否启用
    fn is_enabled(&self) -> bool;
    /// 加固系统提示 (一句话防线)
    fn harden_system_prompt(&self, prompt: &str) -> String;
}

/// AbsorbVerdict — 吸收毒化扫描结果
#[derive(Debug)]
pub struct AbsorbVerdict {
    pub blocked: bool,
    pub reasons: Vec<String>,
}

impl AbsorbVerdict {
    pub fn is_blocked(&self) -> bool {
        self.blocked
    }
}

/// AbsorbTextScanner — 吸收文本毒化扫描抽象
/// L1 模块通过此 trait 执行吸收前毒化扫描，而非直接依赖 L3 self_poison。
pub trait AbsorbTextScanner: Send + Sync {
    /// 扫描待吸收文本 (标题/摘要/正文), 返回判定结果
    fn scan(&self, title: &str, summary: &Option<String>, content: &Option<String>) -> AbsorbVerdict;
}

/// ReceiptEmitter — 可验证回放收据生成抽象
/// L1 模块通过此 trait 生成收据，而非直接依赖 L3 AgentReceipt。
pub trait ReceiptEmitter: Send + Sync {
    /// 为一次运行/吸收动作生成签名收据, 返回签名字符串
    fn emit_receipt(&self, run_id: &str, input: &str, output: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_def_construction() {
        let t = ToolDef {
            name: "test_tool".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        };
        assert_eq!(t.name, "test_tool");
        assert_eq!(t.description, "A test tool");
    }

    #[test]
    fn test_tool_output_success() {
        let o = ToolOutput {
            success: true,
            content: "done".into(),
        };
        assert!(o.success);
        assert_eq!(o.content, "done");
    }

    #[test]
    fn test_tool_output_failure() {
        let o = ToolOutput {
            success: false,
            content: "error".into(),
        };
        assert!(!o.success);
    }

    #[test]
    fn test_seal_result() {
        let r = SealResult {
            score_before: 0.5,
            score_after: 0.8,
            delta: 0.3,
            iterations: 5,
        };
        assert!((r.delta - 0.3).abs() < 1e-10);
        assert_eq!(r.iterations, 5);
    }

    #[test]
    fn test_seal_result_zero_delta() {
        let r = SealResult {
            score_before: 1.0,
            score_after: 1.0,
            delta: 0.0,
            iterations: 0,
        };
        assert!((r.delta).abs() < 1e-10);
    }

    #[test]
    fn test_tool_def_display_trait() {
        let t = ToolDef {
            name: "calc".into(),
            description: "Calculator".into(),
            input_schema: serde_json::json!({}),
        };
        let _debug = format!("{:?}", t.input_schema);
        assert!(serde_json::to_string(&t.input_schema).is_ok());
    }

    #[test]
    fn test_memory_provider_trait_object_safe() {
        fn _take_memory_provider(_: &dyn MemoryProvider) {}
        let _ = _take_memory_provider;
    }

    #[test]
    fn test_agent_executor_trait_object_safe() {
        fn _take_executor(_: &dyn AgentExecutor<Output = String>) {}
        let _ = _take_executor;
    }

    #[test]
    fn test_engine_provider_trait_object_safe() {
        fn _take_engine(_: &dyn EngineProvider) {}
        let _ = _take_engine;
    }
}
