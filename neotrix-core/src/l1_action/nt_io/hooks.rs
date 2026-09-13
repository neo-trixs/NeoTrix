//! 三拍子 Hook 系统 — 基于 Grok Build 模式
//!
//! Session/Tool/Compact 三个层级的生命周期钩子。

/// Hook 事件类型
pub enum HookEvent {
    SessionStart {
        session_id: String,
    },
    SessionEnd {
        session_id: String,
    },
    PreToolUse {
        tool_name: String,
        input: String,
    },
    PostToolUse {
        tool_name: String,
        output: String,
        success: bool,
    },
    PreCompact {
        current_tokens: usize,
        target_tokens: usize,
    },
    PostCompact {
        before_tokens: usize,
        after_tokens: usize,
    },
    SubagentStart {
        agent_id: String,
    },
    SubagentStop {
        agent_id: String,
        success: bool,
    },
    PermissionDenied {
        tool_name: String,
        reason: String,
    },
}

/// Hook 决策
pub enum HookDecision {
    /// 允许继续
    Allow,
    /// 阻止执行
    Deny { reason: String },
    /// 修改输入
    Modify { modified_input: String },
}

/// Hook 处理函数类型
pub type HookHandler = Box<dyn Fn(&HookEvent) -> HookDecision + Send + Sync>;

/// 单个 Hook 注册
pub struct HookRegistration {
    pub name: String,
    pub event_type: String,
    pub handler: HookHandler,
    pub priority: u32,
    pub blocking: bool,
}

/// Hook 管理器 — 三拍子生命周期钩子
pub struct HookManager {
    hooks: Vec<HookRegistration>,
    hook_count: u32,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            hook_count: 0,
        }
    }

    /// 注册 hook
    pub fn register(&mut self, name: &str, event_type: &str, handler: HookHandler, blocking: bool) {
        self.hook_count += 1;
        self.hooks.push(HookRegistration {
            name: name.to_string(),
            event_type: event_type.to_string(),
            handler,
            priority: self.hook_count,
            blocking,
        });
        self.hooks.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 触发 hook 事件
    pub fn trigger(&self, event: &HookEvent) -> Vec<HookDecision> {
        let event_type = match event {
            HookEvent::SessionStart { .. } | HookEvent::SessionEnd { .. } => "session",
            HookEvent::PreToolUse { .. } | HookEvent::PostToolUse { .. } => "tool",
            HookEvent::PreCompact { .. } | HookEvent::PostCompact { .. } => "compact",
            HookEvent::SubagentStart { .. } | HookEvent::SubagentStop { .. } => "subagent",
            HookEvent::PermissionDenied { .. } => "permission",
        };

        self.hooks
            .iter()
            .filter(|h| h.event_type == event_type || h.event_type == "*")
            .map(|h| (h.handler)(event))
            .collect()
    }

    /// 检查是否有 hook 阻止了执行
    pub(crate) fn _should_block(&self, event: &HookEvent) -> Option<String> {
        for decision in self.trigger(event) {
            if let HookDecision::Deny { reason } = decision {
                return Some(reason);
            }
        }
        None
    }

    /// 获取 hook 统计
    pub fn stats(&self) -> (usize, usize) {
        let blocking = self.hooks.iter().filter(|h| h.blocking).count();
        (self.hooks.len(), blocking)
    }
}
