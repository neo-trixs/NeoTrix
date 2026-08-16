//! Agentic 浏览器任务循环 (吸收自 browser-use + firecrawl)
//!
//! LLM 驱动的浏览器任务状态机: Interact → Agent → Interact → ... → Done。
//! 纯确定性模拟 — 无网络、无真实浏览器、无 tokio。

use crate::core::nt_core_self_test::SelfTest;

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct AgentActionResult {
    pub ok: bool,
    pub message: String,
}

impl AgentActionResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self { ok: true, message: message.into() }
    }
    pub fn err(message: impl Into<String>) -> Self {
        Self { ok: false, message: message.into() }
    }
}

/// 一个可注册的浏览器工具动作 (fn 指针即可)
#[derive(Debug, Clone, Copy)]
pub struct ToolAction {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(&str) -> AgentActionResult,
}

/// 工具注册表
#[derive(Debug, Clone, Default)]
pub struct ToolRegistry {
    actions: Vec<ToolAction>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, action: ToolAction) {
        self.actions.push(action);
    }

    pub fn execute(&self, name: &str, arg: &str) -> AgentActionResult {
        match self.actions.iter().find(|a| a.name == name) {
            Some(action) => (action.run)(arg),
            None => AgentActionResult::err(format!("tool '{name}' not registered")),
        }
    }

    pub fn list(&self) -> Vec<&'static str> {
        self.actions.iter().map(|a| a.name).collect()
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.actions.iter().any(|a| a.name == name)
    }
}

/// 默认工具集 (click/type/extract/scroll/wait)
pub fn default_tools() -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(ToolAction {
        name: "click",
        description: "click a selector",
        run: |sel| AgentActionResult::ok(format!("clicked {sel}")),
    });
    reg.register(ToolAction {
        name: "type",
        description: "type text into focused input",
        run: |text| AgentActionResult::ok(format!("typed {text}")),
    });
    reg.register(ToolAction {
        name: "extract",
        description: "extract text from a selector",
        run: |sel| AgentActionResult::ok(format!("extracted from {sel}")),
    });
    reg.register(ToolAction {
        name: "scroll",
        description: "scroll the page",
        run: |_| AgentActionResult::ok("scrolled"),
    });
    reg.register(ToolAction {
        name: "wait",
        description: "wait some ticks",
        run: |ticks| AgentActionResult::ok(format!("waited {ticks} ticks")),
    });
    reg
}

/// Agent 决策的动作
#[derive(Debug, Clone, PartialEq)]
pub enum AgentAction {
    Click { selector: &'static str },
    Type { text: &'static str },
    Extract { selector: &'static str },
    Scroll,
    Wait { ticks: usize },
    Done,
}

/// 会话事件 (状态机转移产物)
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    Interact { tick: usize, action_taken: &'static str },
    Extracted { text: String },
    Done,
    MaxStepsReached,
}

/// 任务最终结果
#[derive(Debug, Clone, PartialEq)]
pub struct TaskOutcome {
    pub completed: bool,
    pub steps: usize,
    pub reason: &'static str,
}

/// Agent 会话状态机: Interact → Agent → Interact → ... → Done
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub ticks: usize,
    pub steps_taken: usize,
    pub max_steps: usize,
    pub extracted: Option<String>,
    pub terminated: bool,
}

impl AgentSession {
    pub fn new() -> Self {
        Self::with_max_steps(20)
    }

    pub fn with_max_steps(max_steps: usize) -> Self {
        Self {
            ticks: 0,
            steps_taken: 0,
            max_steps,
            extracted: None,
            terminated: false,
        }
    }

    /// 单步推进状态机
    pub fn step(&mut self, action: AgentAction, page_text: &str) -> SessionEvent {
        if self.terminated {
            return SessionEvent::Done;
        }
        self.steps_taken += 1;
        self.ticks += 1;
        match action {
            AgentAction::Done => {
                self.terminated = true;
                SessionEvent::Done
            }
            AgentAction::Wait { ticks } => {
                self.ticks += ticks;
                self.maybe_terminal("wait")
            }
            AgentAction::Extract { .. } => {
                let trimmed = page_text.trim();
                if !trimmed.is_empty() {
                    self.terminated = true;
                    self.extracted = Some(trimmed.to_string());
                    SessionEvent::Extracted { text: trimmed.to_string() }
                } else {
                    self.maybe_terminal("extract")
                }
            }
            AgentAction::Click { .. } | AgentAction::Type { .. } | AgentAction::Scroll => {
                self.maybe_terminal("interact")
            }
        }
    }

    fn maybe_terminal(&mut self, action_taken: &'static str) -> SessionEvent {
        if self.steps_taken >= self.max_steps {
            self.terminated = true;
            SessionEvent::MaxStepsReached
        } else {
            SessionEvent::Interact { tick: self.ticks, action_taken }
        }
    }

    /// 启发式选工具: 按任务关键词优先, 未注册的工具不会选中
    fn pick_action(&self, task: &str, page_text: &str, tools: &ToolRegistry) -> AgentAction {
        let task = task.to_lowercase();
        if task.contains("click") && tools.is_registered("click") {
            AgentAction::Click { selector: "main" }
        } else if task.contains("type") && tools.is_registered("type") {
            AgentAction::Type { text: "query" }
        } else if task.contains("scroll") && tools.is_registered("scroll") {
            AgentAction::Scroll
        } else if task.contains("wait") && tools.is_registered("wait") {
            AgentAction::Wait { ticks: 1 }
        } else if task.contains("extract") && tools.is_registered("extract") {
            AgentAction::Extract { selector: "main" }
        } else if !page_text.trim().is_empty() && self.extracted.is_none() {
            AgentAction::Extract { selector: "body" }
        } else {
            AgentAction::Done
        }
    }

    /// 确定性任务循环: 每步经 page_supplier 取页面文本, 启发式选工具并推进状态
    pub fn run_task(
        &mut self,
        task: &str,
        tools: &ToolRegistry,
        page_supplier: fn(&mut Self, &str) -> String,
    ) -> TaskOutcome {
        let mut page = String::new();
        loop {
            if self.steps_taken >= self.max_steps {
                return TaskOutcome {
                    completed: false,
                    steps: self.steps_taken,
                    reason: "max_steps_reached",
                };
            }
            page = page_supplier(self, &page);
            let action = self.pick_action(task, &page, tools);
            match self.step(action, &page) {
                SessionEvent::Extracted { .. } => {
                    return TaskOutcome { completed: true, steps: self.steps_taken, reason: "extracted" };
                }
                SessionEvent::Done => {
                    return TaskOutcome {
                        completed: self.extracted.is_some(),
                        steps: self.steps_taken,
                        reason: "done",
                    };
                }
                SessionEvent::MaxStepsReached => {
                    return TaskOutcome {
                        completed: false,
                        steps: self.steps_taken,
                        reason: "max_steps_reached",
                    };
                }
                SessionEvent::Interact { .. } => {}
            }
        }
    }
}

impl Default for AgentSession {
    fn default() -> Self {
        Self::new()
    }
}

/// SelfTest (T1): 验证工具注册 + 会话推进 + 任务循环
pub struct AgenticBrowseSelfTest;

impl SelfTest for AgenticBrowseSelfTest {
    fn name(&self) -> &str {
        "nt_world_browse_auto_agentic_browse"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let tools = default_tools();
        if !tools.is_registered("click") || !tools.is_registered("extract") {
            return Err(vec!["default tool registry incomplete".into()]);
        }
        if tools.execute("missing", "").ok {
            return Err(vec!["unregistered tool should error".into()]);
        }

        let mut session = AgentSession::new();
        let ev = session.step(AgentAction::Click { selector: "main" }, "<html/>");
        if !matches!(ev, SessionEvent::Interact { .. }) {
            return Err(vec!["interact step should yield Interact event".into()]);
        }

        let mut session = AgentSession::new();
        let outcome = session.run_task("extract the headline", &tools, |_, _| "Headline: NeoTrix".into());
        if !outcome.completed {
            return Err(vec![format!("task should complete, got {outcome:?}")]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_with_content(_s: &mut AgentSession, _prev: &str) -> String {
        "Headline: NeoTrix launches".into()
    }

    fn empty_page(_s: &mut AgentSession, _prev: &str) -> String {
        String::new()
    }

    #[test]
    fn test_tool_register_list() {
        let mut reg = ToolRegistry::new();
        reg.register(ToolAction {
            name: "click",
            description: "click",
            run: |_| AgentActionResult::ok("clicked"),
        });
        reg.register(ToolAction {
            name: "extract",
            description: "extract",
            run: |_| AgentActionResult::ok("extracted"),
        });
        assert_eq!(reg.list(), vec!["click", "extract"]);
    }

    #[test]
    fn test_tool_execute_registered() {
        let mut reg = ToolRegistry::new();
        reg.register(ToolAction {
            name: "type",
            description: "type",
            run: |arg| AgentActionResult::ok(format!("typed {arg}")),
        });
        let res = reg.execute("type", "hello");
        assert!(res.ok);
        assert_eq!(res.message, "typed hello");
    }

    #[test]
    fn test_tool_execute_unregistered_error() {
        let reg = ToolRegistry::new();
        let res = reg.execute("nope", "arg");
        assert!(!res.ok);
        assert!(res.message.contains("not registered"));
        assert!(!reg.is_registered("nope"));
    }

    #[test]
    fn test_tool_is_registered() {
        let tools = default_tools();
        for name in ["click", "type", "extract", "scroll", "wait"] {
            assert!(tools.is_registered(name));
        }
        assert!(!tools.is_registered("screenshot"));
    }

    #[test]
    fn test_session_step_click_transition() {
        let mut s = AgentSession::new();
        let ev = s.step(AgentAction::Click { selector: "#a" }, "<div/>");
        assert!(matches!(ev, SessionEvent::Interact { .. }));
        assert_eq!(s.steps_taken, 1);
        assert_eq!(s.ticks, 1);
        assert!(!s.terminated);
    }

    #[test]
    fn test_session_extract_completes() {
        let mut s = AgentSession::new();
        let ev = s.step(AgentAction::Extract { selector: "main" }, "  content here  ");
        assert!(matches!(ev, SessionEvent::Extracted { .. }));
        assert!(s.terminated);
        assert_eq!(s.extracted.as_deref(), Some("content here"));
    }

    #[test]
    fn test_session_max_steps_cap() {
        let mut s = AgentSession::with_max_steps(3);
        let ev1 = s.step(AgentAction::Scroll, "<html/>");
        let ev2 = s.step(AgentAction::Scroll, "<html/>");
        let ev3 = s.step(AgentAction::Scroll, "<html/>");
        assert!(matches!(ev1, SessionEvent::Interact { .. }));
        assert!(matches!(ev2, SessionEvent::Interact { .. }));
        assert_eq!(ev3, SessionEvent::MaxStepsReached);
        assert!(s.terminated);
        assert_eq!(s.steps_taken, 3);
    }

    #[test]
    fn test_session_done_terminal() {
        let mut s = AgentSession::new();
        let ev = s.step(AgentAction::Done, "<html/>");
        assert_eq!(ev, SessionEvent::Done);
        assert!(s.terminated);
        assert_eq!(s.step(AgentAction::Scroll, "<html/>"), SessionEvent::Done);
    }

    #[test]
    fn test_run_task_completion() {
        let tools = default_tools();
        let mut s = AgentSession::new();
        let outcome = s.run_task("extract the headline", &tools, page_with_content);
        assert!(outcome.completed);
        assert_eq!(outcome.reason, "extracted");
        assert!(outcome.steps >= 1);
    }

    #[test]
    fn test_run_task_max_steps_cap() {
        let tools = default_tools();
        let mut s = AgentSession::with_max_steps(2);
        let outcome = s.run_task("click the button", &tools, empty_page);
        assert!(!outcome.completed);
        assert_eq!(outcome.reason, "max_steps_reached");
        assert_eq!(outcome.steps, 2);
    }

    #[test]
    fn test_run_task_done_without_extraction() {
        let tools = ToolRegistry::new();
        let mut s = AgentSession::new();
        let outcome = s.run_task("just browse around", &tools, empty_page);
        assert!(!outcome.completed);
        assert_eq!(outcome.reason, "done");
    }

    #[test]
    fn test_selftest_agentic_browse_passes() {
        let t = AgenticBrowseSelfTest;
        assert_eq!(t.name(), "nt_world_browse_auto_agentic_browse");
        assert!(t.self_test().is_ok());
    }
}
