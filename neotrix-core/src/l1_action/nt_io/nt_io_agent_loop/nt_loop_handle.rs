//! NT-IO AgentLoop handle — 句柄/控制（纯搬移，行为零变更）。
//!
//! 内容：`new` + `_with_*` 构建器 + `with_*` 链式配置 + 访问器
//!（`model`/`set_model`/`register_tool`/`tool_count`/`history_len`/
//! `reset_history`/`history`/`last_*`）。

use std::sync::Arc;

use super::nt_loop_types::{AgentLoop, ToolInvocation};
use crate::l1_action::nt_io::nt_io_multimodal_transform::MultimodalTransform;
use crate::l1_action::nt_io::nt_io_output_style::{GovernanceReport, OutputStyleId};
use crate::l1_action::nt_io::nt_io_provider::types::{LlmProvider, Message, Role, Usage};
use crate::l0_substrate::nt_core_traits::{NativeTool, PropagationGuardLike, SecretScanner};

impl AgentLoop {
    pub fn new(backend: Arc<dyn LlmProvider>, model: &str, system_prompt: &str) -> Self {
        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(Message::new(Role::System, system_prompt));
        }
        // 刻意**不**在这里调 `nt_capability_canary::reset()`：
        // 测试也调 `new()`，构造即归零会让并行测试互相清零对方的计数。
        // 归零是显式的会话生命周期事件，见 `begin_session_window()`。
        Self {
            backend,
            tools: Vec::new(),
            messages,
            model: model.to_string(),
            max_tool_rounds: 8,
            max_history: 64,
            context_token_budget: 24_000,
            max_tool_output_tokens: 3_000,
            last_usage: None,
            tool_log: Vec::new(),
            tool_hook: None,
            style: OutputStyleId::Plain,
            guard: None,
            multimodal: None,
            style_registry: None,
            secret_scanner: None,
            last_governance: None,
            canary_session: neotrix_neobot::nt_capability_canary::DEFAULT_SESSION.to_owned(),
        }
    }

    /// 会话窗口起点（金丝雀观察窗口归零），真会话路径专用。
    ///
    /// 2026-10-05 接通。此前 `nt_capability_canary::reset()` 零生产调用者，
    /// 于是 `window_ticks()` 恒 0，健康判据 `fired>0 || ticks<threshold`
    /// 的第二项恒真 —— 任何能力永远被判「健康」。照抄 plur `tools.ts:3594`：
    /// 不 reset 的话，一次信号就能让金丝雀在整个进程生命周期保持健康
    /// （它记的 #192 事故）。
    ///
    /// 2026-10-07 会话键化（`OPEN-DEFECTS` P1-5）：`session` 指定本循环
    /// 的窗口桶 ⇒ 只归零**本会话**的窗口，⛔ 不碰其他会话
    ///（旧实现是进程全局，两个会话互相 reset）。
    ///
    /// 不放在 `new()` 里的理由：测试也调 `new()`，构造即归零 ⇒ 并行测试互相
    /// 清零对方的计数 ⇒ 门与测试读到随机值。归零必须是显式的会话事件，
    /// 不能挂在「对象被构造过」这个事实上。
    ///
    /// 幂等：重复调只是把窗口再归零，无副作用。
    pub fn begin_session_window(&mut self, session: &str) {
        self.canary_session = session.to_owned();
        neotrix_neobot::nt_capability_canary::reset(session);
    }

    /// 装上工具调用钩子（B2 接线：loop→checkpoint 的反向通道）。
    pub fn with_tool_hook(mut self, hook: Arc<dyn Fn(&ToolInvocation) + Send + Sync>) -> Self {
        self.tool_hook = Some(hook);
        self
    }

    pub(crate) fn _with_multimodal_transform(mut self, stage: MultimodalTransform) -> Self {
        self.multimodal = Some(std::sync::Arc::new(stage));
        self
    }

    pub(crate) fn _with_output_style(mut self, style: OutputStyleId) -> Self {
        self.style = style;
        self
    }

    pub(crate) fn _with_secret_scanner(mut self, scanner: Box<dyn SecretScanner>) -> Self {
        self.secret_scanner = Some(scanner);
        self
    }

    pub(crate) fn _with_propagation_guard(mut self, guard: Box<dyn PropagationGuardLike>) -> Self {
        // 加固系统提示: 论文结论 — 一句话防线 → 近完全免疫。
        if guard.is_enabled() {
            if let Some(first) = self.messages.first_mut() {
                first.content = guard.harden_system_prompt(&first.content);
            }
        }
        self.guard = Some(guard);
        self
    }

    /// G27 最近一次最终输出的治理报告 (纯观测, 不影响循环行为)。
    pub fn last_governance(&self) -> Option<&GovernanceReport> {
        self.last_governance.as_ref()
    }

    pub fn with_tools(mut self, tools: Vec<Box<dyn NativeTool>>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_max_tool_rounds(mut self, max: usize) -> Self {
        self.max_tool_rounds = max;
        self
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// 设置上下文 token 预算 (估算口径, 与 neocodex ContextPipeline 一致)。
    /// 超预算时按 工具输出截断 → 最旧轮次驱逐 顺序收敛。
    pub fn with_context_token_budget(mut self, budget: usize) -> Self {
        self.context_token_budget = budget;
        self
    }

    /// 设置单条工具输出写入历史前的 token 截断上限 (0 = 不截断)。
    pub(crate) fn _with_tool_output_budget(mut self, max_tokens: usize) -> Self {
        self.max_tool_output_tokens = max_tokens;
        self
    }

    /// 按模型 context window 派生预算 (P1-B3, 入口模型感知):
    /// 上下文预算 = window × 0.8 (安全余量); 单条工具输出上限 ≥ 3k 且 ≤ window/8。
    /// 避免一律走默认 24k, 导致大窗口模型被过早驱逐 / 小窗口模型溢出。
    pub(crate) fn _with_context_window(mut self, window: usize) -> Self {
        let budget = ((window as f64) * 0.8).floor().max(1024.0) as usize;
        self.context_token_budget = budget;
        self.max_tool_output_tokens = self.max_tool_output_tokens.max(3_000).min(window / 8);
        self
    }

    /// 最近一次 LLM 调用的实际 token 用量 (prompt/completion/total)。
    pub fn last_usage(&self) -> Option<&Usage> {
        self.last_usage.as_ref()
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// 运行时切换模型（TUI `/model <name>`）。空串忽略（保持当前）。
    pub fn set_model(&mut self, model: &str) {
        if !model.trim().is_empty() {
            self.model = model.trim().to_string();
        }
    }

    /// 注册单个工具。
    pub fn register_tool(&mut self, tool: Box<dyn NativeTool>) {
        self.tools.push(tool);
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn history_len(&self) -> usize {
        self.messages.len()
    }

    /// P1-5 修复: 重置会话历史(保留 System 首条与已注册工具)。
    /// TUI 切换/清空会话时调用, 避免新会话沿用旧会话上下文造成语义串台。
    pub fn reset_history(&mut self, system_prompt: &str) {
        self.messages.clear();
        if !system_prompt.is_empty() {
            self.messages
                .push(Message::new(Role::System, system_prompt));
        }
    }

    /// 返回当前会话的消息历史（供持久化/检索）。
    pub fn history(&self) -> &[Message] {
        &self.messages
    }
}
