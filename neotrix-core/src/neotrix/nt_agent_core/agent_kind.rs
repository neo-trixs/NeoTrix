#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentKind {
    Explorer,
    Worker,
    Planner,
}

impl AgentKind {
    pub fn label(&self) -> &'static str {
        match self {
            AgentKind::Explorer => "explorer",
            AgentKind::Worker => "worker",
            AgentKind::Planner => "planner",
        }
    }

    pub fn context_budget(&self) -> usize {
        match self {
            AgentKind::Explorer => 2048,
            AgentKind::Worker => 4096,
            AgentKind::Planner => 8192,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "explorer" => Some(AgentKind::Explorer),
            "worker" => Some(AgentKind::Worker),
            "planner" => Some(AgentKind::Planner),
            _ => None,
        }
    }
}
