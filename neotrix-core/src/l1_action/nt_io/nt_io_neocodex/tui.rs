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
    pub goal_display: crate::cli::tui::app::types::GoalDisplay,
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
            goal_display: crate::cli::tui::app::types::GoalDisplay::idle(),
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
            self.goal_display = crate::cli::tui::app::types::GoalDisplay {
                has_goal: true,
                id: goal.id.clone(),
                description: goal.description.clone(),
                state_label: format!("{:?}", goal.state),
                state_icon: match goal.state {
                    GoalState::Active => "▶".into(),
                    GoalState::Paused => "⏸".into(),
                    GoalState::Completed => "✅".into(),
                    GoalState::Blocked => "🚫".into(),
                    GoalState::Cancelled => "❌".into(),
                },
                iterations: goal.iterations,
                max_iterations: goal.max_iterations,
                ..crate::cli::tui::app::types::GoalDisplay::idle()
            };
        }
    }
}