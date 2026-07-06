use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetAction {
    Warn,
    Pause,
    Stop,
}

#[derive(Debug, Clone)]
pub struct BudgetConfig {
    pub enabled: bool,
    pub max_session_cost: f64,
    pub max_daily_cost: f64,
    pub max_monthly_cost: f64,
    pub action: BudgetAction,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_session_cost: 0.50,
            max_daily_cost: 2.00,
            max_monthly_cost: 20.00,
            action: BudgetAction::Warn,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: String,
    pub name: String,
    pub model: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub estimated_cost: f64,
    pub tool_calls: u64,
}

#[derive(Debug, Clone)]
pub struct AgentCostAccount {
    pub agent_id: String,
    pub agent_name: String,
    pub total_cost: f64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub session_count: u64,
    pub budget_limit: Option<f64>,
    pub tool_calls: u64,
    pub last_active: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentCostSummary {
    pub agent_id: String,
    pub agent_name: String,
    pub total_cost: f64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub session_count: u64,
    pub budget_limit: Option<f64>,
    pub budget_remaining: Option<f64>,
    pub tool_calls: u64,
    pub last_active: u64,
}

#[derive(Debug, Clone)]
pub struct CostSummary {
    pub total_cost: f64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub session_count: u64,
    pub current_session_cost: f64,
    pub top_model: String,
    pub budget_remaining: Option<f64>,
    pub agent_count: usize,
}

pub struct CostTracker {
    pub budget: Mutex<BudgetConfig>,
    total_cost: f64,
    total_tokens_in: u64,
    total_tokens_out: u64,
    sessions: Vec<SessionRecord>,
    current_session_cost: f64,
    top_model: String,
    sessions_since_reset: u64,
    agent_accounts: HashMap<String, AgentCostAccount>,
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            budget: Mutex::new(BudgetConfig::default()),
            total_cost: 0.0,
            total_tokens_in: 0,
            total_tokens_out: 0,
            sessions: Vec::new(),
            current_session_cost: 0.0,
            top_model: String::from("unknown"),
            sessions_since_reset: 0,
            agent_accounts: HashMap::new(),
        }
    }

    pub fn summary(&self) -> CostSummary {
        let budget = self.budget.lock().unwrap_or_else(|e| e.into_inner());
        let remaining = if budget.enabled {
            Some(budget.max_monthly_cost - self.total_cost)
        } else {
            None
        };
        CostSummary {
            total_cost: self.total_cost,
            total_tokens_in: self.total_tokens_in,
            total_tokens_out: self.total_tokens_out,
            session_count: self.sessions_since_reset + 1,
            current_session_cost: self.current_session_cost,
            top_model: self.top_model.clone(),
            budget_remaining: remaining,
            agent_count: self.agent_accounts.len(),
        }
    }

    pub fn sessions(&self) -> Vec<SessionRecord> {
        self.sessions.clone()
    }

    pub fn set_budget(&mut self, amount: f64, _period: BudgetPeriod) {
        if let Ok(mut budget) = self.budget.lock() {
            budget.max_monthly_cost = amount;
            budget.enabled = true;
        }
    }

    pub fn set_max_budget_usd(&mut self, limit: f64) {
        if let Ok(mut budget) = self.budget.lock() {
            budget.max_monthly_cost = limit;
            budget.enabled = true;
        }
    }

    pub fn reset(&mut self) {
        self.total_cost = 0.0;
        self.total_tokens_in = 0;
        self.total_tokens_out = 0;
        self.sessions.clear();
        self.current_session_cost = 0.0;
        self.sessions_since_reset = 0;
        self.agent_accounts.clear();
    }

    pub fn budget_status(&self) -> String {
        let budget = self.budget.lock().unwrap_or_else(|e| e.into_inner());
        format!(
            "💰 预算状态\n  开启: {}\n  会话上限: ${:.2}\n  日上限: ${:.2}\n  月上限: ${:.2}\n  当前消耗: ${:.4}\n  超限动作: {:?}",
            if budget.enabled { "是" } else { "否" },
            budget.max_session_cost,
            budget.max_daily_cost,
            budget.max_monthly_cost,
            self.total_cost,
            budget.action,
        )
    }

    pub fn save_budget_config(&self) {
        if let Ok(budget) = self.budget.lock() {
            let config_path = dirs::home_dir()
                .map(|h| h.join(".neotrix").join("budget.json"))
                .unwrap_or_else(|| std::path::PathBuf::from("budget.json"));
            if let Some(parent) = config_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) { log::warn!("[cost] create dir: {}", e); }
            }
            let content = format!(
                "enabled={}\nmax_session_cost={}\nmax_daily_cost={}\nmax_monthly_cost={}\naction={:?}\n",
                budget.enabled,
                budget.max_session_cost,
                budget.max_daily_cost,
                budget.max_monthly_cost,
                budget.action,
            );
            if let Err(e) = std::fs::write(&config_path, &content) { log::warn!("[cost] write budget: {}", e); }
        }
    }

    pub fn reset_budget_period(&self, period: BudgetPeriod) {
        if let Ok(mut budget) = self.budget.lock() {
            match period {
                BudgetPeriod::Daily => {
                    budget.max_daily_cost = BudgetConfig::default().max_daily_cost;
                }
                BudgetPeriod::Weekly => {
                    budget.max_daily_cost = BudgetConfig::default().max_daily_cost;
                }
                BudgetPeriod::Monthly => {
                    budget.max_monthly_cost = BudgetConfig::default().max_monthly_cost;
                }
            }
            budget.enabled = true;
        }
    }

    pub fn register_agent(&mut self, id: &str, name: &str, budget_limit: Option<f64>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.agent_accounts.insert(
            id.to_string(),
            AgentCostAccount {
                agent_id: id.to_string(),
                agent_name: name.to_string(),
                total_cost: 0.0,
                total_tokens_in: 0,
                total_tokens_out: 0,
                session_count: 0,
                budget_limit,
                tool_calls: 0,
                last_active: now,
            },
        );
    }

    pub fn record_agent_cost(
        &mut self,
        id: &str,
        cost: f64,
        tokens_in: u64,
        tokens_out: u64,
        tool_calls: u64,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if let Some(account) = self.agent_accounts.get_mut(id) {
            account.total_cost += cost;
            account.total_tokens_in += tokens_in;
            account.total_tokens_out += tokens_out;
            account.session_count += 1;
            account.tool_calls += tool_calls;
            account.last_active = now;
        }
    }

    pub fn get_agent_account(&self, id: &str) -> Option<AgentCostAccount> {
        self.agent_accounts.get(id).cloned()
    }

    pub fn list_agent_accounts(&self) -> Vec<AgentCostAccount> {
        let mut accounts: Vec<_> = self.agent_accounts.values().cloned().collect();
        accounts.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        accounts
    }

    pub fn agent_budget_status(&self, id: &str) -> String {
        match self.agent_accounts.get(id) {
            Some(account) => {
                if let Some(limit) = account.budget_limit {
                    let remaining = limit - account.total_cost;
                    if remaining <= 0.0 {
                        format!(
                            "{} ({}) 预算耗尽: ${:.4}/{} (超出 ${:.4})",
                            account.agent_name,
                            account.agent_id,
                            account.total_cost,
                            limit,
                            remaining.abs(),
                        )
                    } else {
                        format!(
                            "{} ({}) 预算剩余: ${:.4}/{}",
                            account.agent_name,
                            account.agent_id,
                            remaining,
                            limit,
                        )
                    }
                } else {
                    format!(
                        "{} ({}) 总消耗: ${:.4} (无预算限制)",
                        account.agent_name, account.agent_id, account.total_cost,
                    )
                }
            }
            None => format!("Agent '{}' 未注册", id),
        }
    }

    pub fn remove_agent(&mut self, id: &str) {
        self.agent_accounts.remove(id);
    }

    pub fn status_line(&self) -> String {
        let agent_count = self.agent_accounts.len();
        format!(
            "${:.4}/{}tok/{}agents",
            self.total_cost,
            self.total_tokens_in + self.total_tokens_out,
            agent_count,
        )
    }
}

pub static COST_TRACKER: LazyLock<Mutex<CostTracker>> = LazyLock::new(|| {
    Mutex::new(CostTracker::new())
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_record_agent_cost() {
        let mut ct = CostTracker::new();
        ct.register_agent("agent-1", "Alpha", None);

        let acc = ct.get_agent_account("agent-1").unwrap();
        assert_eq!(acc.agent_id, "agent-1");
        assert_eq!(acc.agent_name, "Alpha");
        assert_eq!(acc.total_cost, 0.0);
        assert!(acc.budget_limit.is_none());

        ct.record_agent_cost("agent-1", 0.15, 100, 50, 2);
        let acc = ct.get_agent_account("agent-1").unwrap();
        assert!((acc.total_cost - 0.15).abs() < 1e-9);
        assert_eq!(acc.total_tokens_in, 100);
        assert_eq!(acc.total_tokens_out, 50);
        assert_eq!(acc.session_count, 1);
        assert_eq!(acc.tool_calls, 2);

        ct.record_agent_cost("agent-1", 0.10, 60, 30, 1);
        let acc = ct.get_agent_account("agent-1").unwrap();
        assert!((acc.total_cost - 0.25).abs() < 1e-9);
        assert_eq!(acc.total_tokens_in, 160);
        assert_eq!(acc.total_tokens_out, 80);
        assert_eq!(acc.session_count, 2);
        assert_eq!(acc.tool_calls, 3);
    }

    #[test]
    fn test_agent_budget_enforcement() {
        let mut ct = CostTracker::new();
        ct.register_agent("agent-b", "Beta", Some(1.00));

        let acc = ct.get_agent_account("agent-b").unwrap();
        assert_eq!(acc.budget_limit, Some(1.00));

        // Spend within budget
        ct.record_agent_cost("agent-b", 0.50, 200, 100, 2);
        let status = ct.agent_budget_status("agent-b");
        assert!(status.contains("剩余"));

        // Exceed budget
        ct.record_agent_cost("agent-b", 0.60, 200, 100, 2);
        let status = ct.agent_budget_status("agent-b");
        assert!(status.contains("耗尽"));
    }

    #[test]
    fn test_list_agents() {
        let mut ct = CostTracker::new();
        assert!(ct.list_agent_accounts().is_empty());

        ct.register_agent("a1", "Agent1", None);
        ct.register_agent("a2", "Agent2", Some(5.0));

        let accounts = ct.list_agent_accounts();
        assert_eq!(accounts.len(), 2);

        let ids: Vec<String> = accounts.iter().map(|a| a.agent_id.clone()).collect();
        assert!(ids.contains(&"a1".to_string()));
        assert!(ids.contains(&"a2".to_string()));
    }

    #[test]
    fn test_agent_remove() {
        let mut ct = CostTracker::new();
        ct.register_agent("agent-r", "ToRemove", None);
        assert!(ct.get_agent_account("agent-r").is_some());

        ct.remove_agent("agent-r");
        assert!(ct.get_agent_account("agent-r").is_none());
        assert_eq!(ct.list_agent_accounts().len(), 0);
    }

    #[test]
    fn test_multiple_agents_independent_accounting() {
        let mut ct = CostTracker::new();
        ct.register_agent("agent-x", "X-Ray", None);
        ct.register_agent("agent-y", "Yankee", Some(10.0));

        ct.record_agent_cost("agent-x", 1.0, 500, 200, 5);
        ct.record_agent_cost("agent-y", 2.0, 1000, 400, 10);

        let x = ct.get_agent_account("agent-x").unwrap();
        let y = ct.get_agent_account("agent-y").unwrap();

        assert!((x.total_cost - 1.0).abs() < 1e-9);
        assert_eq!(x.total_tokens_in, 500);
        assert_eq!(x.tool_calls, 5);

        assert!((y.total_cost - 2.0).abs() < 1e-9);
        assert_eq!(y.total_tokens_in, 1000);
        assert_eq!(y.tool_calls, 10);

        // Verify summary includes agent count
        let summary = ct.summary();
        assert_eq!(summary.agent_count, 2);
    }

    #[test]
    fn test_unregistered_agent_recording_is_noop() {
        let mut ct = CostTracker::new();
        // Should not panic
        ct.record_agent_cost("nonexistent", 0.5, 100, 50, 2);
        assert!(ct.get_agent_account("nonexistent").is_none());
    }
}
