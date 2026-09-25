//! NT-IO AgentLoop types — 循环状态与类型（纯搬移，行为零变更）。
//!
//! 内容：`COMPACTION_*` 阈值常量 + `ToolInvocation` + `AgentLoop` 结构体定义。
//! 字段 `pub(crate)` 化以供 `nt_loop_core/step/handle` 同 crate 访问；
//! 对外 `pub` API（`AgentLoop` / `ToolInvocation` / `tool_log`）保持不变。

use std::sync::Arc;

use crate::l1_action::nt_io::nt_io_multimodal_transform::MultimodalTransform;
use crate::l1_action::nt_io::nt_io_output_style::{GovernanceReport, OutputStyleId, OutputStyleRegistry};
use crate::l1_action::nt_io::nt_io_provider::types::{LlmProvider, Message, Usage};
use crate::l0_substrate::nt_core_traits::{NativeTool, PropagationGuardLike, SecretScanner};

/// P1-B2 双相 compaction 阈值: 超过预算 90% 触发 LLM 摘要压缩 (OpenCode 40K 双相模式, E10)。
pub(crate) const COMPACTION_THRESHOLD_RATIO: f64 = 0.9;
/// 至少这么多条可压缩消息才值得一次 LLM 摘要调用 (否则驱逐即可, 省一次调用)。
pub(crate) const COMPACTION_MIN_MESSAGES: usize = 8;
/// 摘要输出预算。
pub(crate) const COMPACTION_SUMMARY_MAX_TOKENS: u32 = 1024;

/// 一次工具执行的记录（供调用方观测/审计）。
#[derive(Debug, Clone)]
pub struct ToolInvocation {
    pub name: String,
    pub arguments: String,
    pub success: bool,
    pub output: String,
}

/// AgentLoop — 系统主体对话循环。
pub struct AgentLoop {
    pub(crate) backend: Arc<dyn LlmProvider>,
    pub(crate) tools: Vec<Box<dyn NativeTool>>,
    /// 会话消息历史（含 System 首条）。
    pub(crate) messages: Vec<Message>,
    pub(crate) model: String,
    pub(crate) max_tool_rounds: usize,
    pub(crate) max_history: usize,
    /// 上下文 token 预算 (估算, chars/token 口径) — 超预算时工具输出截断 + 最旧轮次驱逐。
    pub(crate) context_token_budget: usize,
    /// 单条工具输出写入历史前的 token 上限 (0 = 不截断)。
    pub(crate) max_tool_output_tokens: usize,
    /// 最近一次 LLM 调用的 usage (观测杠杆: 每次 turn 可见实际 token 消耗)。
    pub(crate) last_usage: Option<Usage>,
    /// 本会话已执行的工具调用记录。
    pub tool_log: Vec<ToolInvocation>,
    /// 输出样式 (NT-IO output_style 骨架接线)。默认 Plain 原样透传。
    pub(crate) style: OutputStyleId,
    /// 心智病毒传播防护 (NT-SHIELD propagation_guard 骨架接线)。
    pub(crate) guard: Option<Box<dyn PropagationGuardLike>>,
    /// 多模态预处理 (NT-IO multimodal_transform 骨架接线)。
    pub(crate) multimodal: Option<std::sync::Arc<MultimodalTransform>>,
    /// 样式注册表 (单实例惰性共享)。
    pub(crate) style_registry: Option<std::sync::Arc<OutputStyleRegistry>>,
    /// 密钥/PII 扫描器 (NT-SHIELD redaction 抽象, R-P42 强化现有节点)。
    pub(crate) secret_scanner: Option<Box<dyn SecretScanner>>,
    /// G27 最近一次最终输出的治理报告 (观测杠杆: 每次 emit_final 可见纪律合规)。
    pub(crate) last_governance: Option<GovernanceReport>,
}
