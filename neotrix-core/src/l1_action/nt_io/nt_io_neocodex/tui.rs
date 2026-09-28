// ── TUI Integration (bridge to existing TuiApp) ──

use std::sync::Arc;

use tokio::sync::Mutex;

use super::agent::NeoCodexAgent;
use super::goals::GoalState;
use super::provider::NeoCodexMode;

pub struct NeoCodexUI {
    pub agent: Arc<Mutex<NeoCodexAgent>>,
    pub mode: NeoCodexMode,
    pub status_text: String,
    pub streaming_text: String,
    pub input_buffer: String,
    pub message_log: Vec<(String, String)>,
    pub goal_active: bool,
    pub goal_id: String,
    pub goal_description: String,
    pub goal_state_label: String,
    pub goal_state_icon: String,
    pub goal_iterations: u32,
    pub goal_max_iterations: u32,
}

impl NeoCodexUI {
    pub fn new(session_id: &str) -> Self {
        Self {
            agent: Arc::new(Mutex::new(NeoCodexAgent::new(session_id))),
            mode: NeoCodexMode::Agent,
            status_text: "NeoCodex Ready".into(),
            streaming_text: String::new(),
            input_buffer: String::new(),
            message_log: Vec::new(),
            goal_active: false,
            goal_id: String::new(),
            goal_description: String::new(),
            goal_state_label: String::new(),
            goal_state_icon: String::new(),
            goal_iterations: 0,
            goal_max_iterations: 0,
        }
    }

    pub async fn send_message(&mut self, text: &str) {
        let mut agent = self.agent.lock().await;
        let response = agent.process(text).await;
        self.mode = agent.state.mode;
        self.message_log.push(("user".into(), text.to_string()));
        self.message_log.push(("assistant".into(), response));
        let report = agent.health_report();
        self.status_text = format!(
            "Turn {} | {} tools | {} tokens | ctx {:.0}% | {}",
            agent.state.turn_count,
            agent.state.tool_call_count,
            agent.state.tokens_used,
            report.context_usage * 100.0,
            agent.evolution.summary(),
        );
        if let Some(ref goal) = agent.goals.active {
            self.goal_active = true;
            self.goal_id = goal.id.clone();
            self.goal_description = goal.description.clone();
            self.goal_state_label = format!("{:?}", goal.state);
            self.goal_state_icon = match goal.state {
                GoalState::Active => "▶".into(),
                GoalState::Paused => "⏸".into(),
                GoalState::Completed => "✅".into(),
                GoalState::Blocked => "🚫".into(),
                GoalState::Cancelled => "❌".into(),
            };
            self.goal_iterations = goal.iterations as u32;
            self.goal_max_iterations = goal.max_iterations as u32;
        }
    }
}
