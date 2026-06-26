use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;
use uuid::Uuid;

/// Global cost tracker instance (lazy singleton).
pub static COST_TRACKER: LazyLock<Mutex<CostTracker>> =
    LazyLock::new(|| Mutex::new(CostTracker::new()));

/// Tracks token usage and estimated cost per session.
pub struct CostTracker {
    /// Completed sessions (persisted in memory for this CLI run).
    sessions: Vec<CostSession>,
    /// Active (current) session token/cost accumulation.
    current: Option<CurrentSession>,
    /// Optional budget cap in USD.
    budget_limit: Option<f64>,
    /// Period over which the budget applies.
    budget_period: BudgetPeriod,
    /// Rich budget configuration (used by /budget command).
    pub budget: Mutex<BudgetConfig>,
    /// Hard total-spend limit in USD (from --max-budget-usd CLI arg).
    max_budget_usd: Option<f64>,
}

/// In-flight accumulator for the current session.
struct CurrentSession {
    model: String,
    provider: String,
    tokens_in: u64,
    tokens_out: u64,
    estimated_cost: f64,
    tool_calls: u64,
    started_at: DateTime<Utc>,
}

/// A completed (archived) cost session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSession {
    pub id: String,
    pub name: String,
    pub model: String,
    pub provider: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub estimated_cost: f64,
    pub tool_calls: u64,
    pub started_at: DateTime<Utc>,
    pub duration_secs: u64,
}

/// Summary snapshot used by the `/cost` command.
pub struct CostSummary {
    pub total_cost: f64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub session_count: usize,
    pub current_session_cost: f64,
    pub budget_remaining: Option<f64>,
    pub top_model: String,
}

/// Budget time window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
}

impl BudgetPeriod {
    fn label(self) -> &'static str {
        match self {
            BudgetPeriod::Daily => "daily",
            BudgetPeriod::Weekly => "weekly",
            BudgetPeriod::Monthly => "monthly",
        }
    }
}

/// What to do when a budget limit is hit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BudgetAction {
    Warn,
    Pause,
    Stop,
}

/// Budget configuration with multi-tier limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub max_session_cost: f64,
    pub max_daily_cost: f64,
    pub max_monthly_cost: f64,
    pub enabled: bool,
    pub action: BudgetAction,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_session_cost: 10.0,
            max_daily_cost: 50.0,
            max_monthly_cost: 200.0,
            enabled: false,
            action: BudgetAction::Warn,
        }
    }
}

/// Result of a budget check.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetStatus {
    Ok,
    Warning {
        message: String,
    },
    Exceeded {
        action: BudgetAction,
        message: String,
    },
}

impl CostTracker {
    /// Create a fresh tracker with no history.
    pub fn new() -> Self {
        let ct = Self {
            sessions: Vec::new(),
            current: None,
            budget_limit: None,
            budget_period: BudgetPeriod::Monthly,
            budget: Mutex::new(BudgetConfig::default()),
            max_budget_usd: None,
        };
        ct.load_budget_config();
        ct
    }

    /// Start tracking a new session — archives any active session first.
    pub fn start_session(&mut self, model: &str, provider: &str, name: &str) {
        self.finish_session();
        self.current = Some(CurrentSession {
            model: model.to_string(),
            provider: provider.to_string(),
            tokens_in: 0,
            tokens_out: 0,
            estimated_cost: 0.0,
            tool_calls: 0,
            started_at: Utc::now(),
        });
        self.sessions.push(CostSession {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            model: model.to_string(),
            provider: provider.to_string(),
            tokens_in: 0,
            tokens_out: 0,
            estimated_cost: 0.0,
            tool_calls: 0,
            started_at: Utc::now(),
            duration_secs: 0,
        });
    }

    /// Finish the active session (moves current → completed).
    pub fn finish_session(&mut self) {
        if let Some(cur) = self.current.take() {
            if let Some(last) = self.sessions.last_mut() {
                last.tokens_in = cur.tokens_in;
                last.tokens_out = cur.tokens_out;
                last.estimated_cost = cur.estimated_cost;
                last.tool_calls = cur.tool_calls;
                let elapsed = Utc::now() - cur.started_at;
                last.duration_secs = elapsed.num_seconds().max(0) as u64;
            }
        }
    }

    /// Record token usage for the active session (or start one if none).
    pub fn record_usage(&mut self, model: &str, provider: &str, tokens_in: u64, tokens_out: u64) {
        let tokens_in_f = tokens_in as f64;
        let tokens_out_f = tokens_out as f64;
        let (price_in, price_out) = Self::model_price_per_1k(model);
        let cost = (tokens_in_f / 1000.0) * price_in + (tokens_out_f / 1000.0) * price_out;

        if self.current.is_none() {
            self.start_session(model, provider, "auto");
        }

        if let Some(cur) = &mut self.current {
            cur.tokens_in = cur.tokens_in.saturating_add(tokens_in);
            cur.tokens_out = cur.tokens_out.saturating_add(tokens_out);
            cur.estimated_cost += cost;
        }
    }

    /// Record a tool call in the active session.
    pub fn record_tool_call(&mut self) {
        if let Some(cur) = &mut self.current {
            cur.tool_calls += 1;
        }
    }

    /// Hardcoded pricing per model (USD per 1K tokens).
    pub fn model_price_per_1k(model: &str) -> (f64, f64) {
        match model {
            "gpt-4" => (0.03, 0.06),
            "gpt-4o" => (0.01, 0.03),
            "gpt-4o-mini" => (0.0015, 0.006),
            "gpt-3.5-turbo" => (0.0015, 0.002),
            "claude-3-opus" => (0.015, 0.075),
            "claude-3-sonnet" => (0.003, 0.015),
            "claude-3-haiku" => (0.00025, 0.00125),
            "claude-3.5-sonnet" => (0.003, 0.015),
            "deepseek-chat" => (0.0005, 0.002),
            "deepseek-reasoner" => (0.0005, 0.002),
            "gemini-pro" => (0.001, 0.002),
            "gemini-1.5-pro" => (0.0035, 0.0105),
            _ => (0.01, 0.03),
        }
    }

    /// Sum of all completed session costs + current in-flight cost.
    pub fn total_cost(&self) -> f64 {
        let completed: f64 = self.sessions.iter().map(|s| s.estimated_cost).sum();
        let current = self
            .current
            .as_ref()
            .map(|c| c.estimated_cost)
            .unwrap_or(0.0);
        completed + current
    }

    /// Aggregate token counts.
    pub fn total_tokens(&self) -> (u64, u64) {
        let mut tin = 0u64;
        let mut tout = 0u64;
        for s in &self.sessions {
            tin = tin.saturating_add(s.tokens_in);
            tout = tout.saturating_add(s.tokens_out);
        }
        if let Some(cur) = &self.current {
            tin = tin.saturating_add(cur.tokens_in);
            tout = tout.saturating_add(cur.tokens_out);
        }
        (tin, tout)
    }

    /// Number of completed sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Access completed sessions (for `/cost detail`).
    pub fn sessions(&self) -> &[CostSession] {
        &self.sessions
    }

    /// Cost of the current in-flight session.
    pub fn current_session_cost(&self) -> f64 {
        self.current
            .as_ref()
            .map(|c| c.estimated_cost)
            .unwrap_or(0.0)
    }

    /// Model name of the current session (or "—").
    pub fn current_model(&self) -> String {
        self.current
            .as_ref()
            .map(|c| c.model.clone())
            .unwrap_or_else(|| "—".to_string())
    }

    /// Provider name of the current session.
    pub fn current_provider(&self) -> String {
        self.current
            .as_ref()
            .map(|c| c.provider.clone())
            .unwrap_or_else(|| "—".to_string())
    }

    /// Budget remaining (None if no budget set).
    pub fn budget_remaining(&self) -> Option<f64> {
        self.budget_limit
            .map(|limit| (limit - self.total_cost()).max(0.0))
    }

    /// Set a spending budget.
    pub fn set_budget(&mut self, limit: f64, period: BudgetPeriod) {
        self.budget_limit = Some(limit);
        self.budget_period = period;
    }

    /// Current budget limit (if any).
    pub fn budget_limit(&self) -> Option<f64> {
        self.budget_limit
    }

    /// Current budget period label.
    pub fn budget_period_label(&self) -> &'static str {
        self.budget_period.label()
    }

    /// Set the hard total-spend limit in USD (from --max-budget-usd).
    pub fn set_max_budget_usd(&mut self, limit: f64) {
        self.max_budget_usd = Some(limit);
    }

    /// Get the hard total-spend limit.
    pub fn max_budget_usd(&self) -> Option<f64> {
        self.max_budget_usd
    }

    /// Check if total cost exceeds the `--max-budget-usd` hard limit.
    /// Returns `Some(warning)` if near/exceeding, `None` if OK or no limit set.
    pub fn check_max_budget_hard_limit(&self) -> Option<String> {
        let limit = self.max_budget_usd?;
        let total = self.total_cost();
        if total >= limit {
            Some(format!(
                "🛑 BUDGET EXCEEDED: Total cost ${:.4} exceeds --max-budget-usd ${:.2}. \
                 Set a higher limit or remove --max-budget-usd to continue.",
                total, limit
            ))
        } else if total >= limit * 0.9 {
            Some(format!(
                "⚠️ BUDGET WARNING: Total cost ${:.4} is at {:.0}% of --max-budget-usd ${:.2}",
                total,
                (total / limit * 100.0),
                limit
            ))
        } else {
            None
        }
    }

    /// Produce a summary snapshot.
    pub fn summary(&self) -> CostSummary {
        let (tin, tout) = self.total_tokens();
        let top = self
            .sessions
            .iter()
            .max_by(|a, b| {
                a.estimated_cost
                    .partial_cmp(&b.estimated_cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.model.clone())
            .or_else(|| self.current.as_ref().map(|c| c.model.clone()))
            .unwrap_or_else(|| "—".to_string());
        CostSummary {
            total_cost: self.total_cost(),
            total_tokens_in: tin,
            total_tokens_out: tout,
            session_count: self.sessions.len(),
            current_session_cost: self.current_session_cost(),
            budget_remaining: self.budget_remaining(),
            top_model: top,
        }
    }

    /// Formatted one-line status (for TUI status bar).
    pub fn status_line(&self) -> String {
        let (tin, tout) = self.total_tokens();
        let cost = self.total_cost();
        let model = self.current_model();
        let parts = format!(
            "Cost: ${:.4} | Tokens: {} in/{} out | Model: {}",
            cost, tin, tout, model
        );
        if let Some(remaining) = self.budget_remaining() {
            format!(
                "{} | Budget: ${:.2} remaining ({})",
                parts,
                remaining,
                self.budget_period_label()
            )
        } else {
            parts
        }
    }

    // ── Budget config persistence ──

    fn budget_config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".neotrix").join("budget.json")
    }

    fn load_budget_config(&self) {
        let path = Self::budget_config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<BudgetConfig>(&data) {
                *self.budget.lock().unwrap_or_else(|e| e.into_inner()) = cfg;
            }
        }
    }

    pub fn save_budget_config(&self) {
        let path = Self::budget_config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) =
            serde_json::to_string_pretty(&*self.budget.lock().unwrap_or_else(|e| e.into_inner()))
        {
            let tmp = path.with_extension("tmp");
            if std::fs::write(&tmp, &data).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    // ── New budget methods (used by /budget) ──

    /// Set the full budget configuration (persisted to disk).
    pub fn set_budget_config(&self, config: BudgetConfig) {
        *self.budget.lock().unwrap_or_else(|e| e.into_inner()) = config;
        self.save_budget_config();
    }

    /// Check whether the current spend exceeds any budget threshold.
    pub fn check_budget(&self, session_cost: f64) -> BudgetStatus {
        let cfg = self.budget.lock().unwrap_or_else(|e| e.into_inner());
        if !cfg.enabled {
            return BudgetStatus::Ok;
        }
        let max_session = cfg.max_session_cost;
        let max_daily = cfg.max_daily_cost;
        let max_monthly = cfg.max_monthly_cost;
        let action = cfg.action.clone();
        drop(cfg);

        // Session-level check
        if session_cost > max_session {
            return BudgetStatus::Exceeded {
                action: action.clone(),
                message: format!(
                    "Session cost ${:.2} exceeds limit ${:.2}",
                    session_cost, max_session
                ),
            };
        }

        let now = Utc::now();
        let today = now.format("%Y-%m-%d").to_string();
        let this_month = now.format("%Y-%m").to_string();

        let mut daily_cost: f64 = 0.0;
        let mut monthly_cost: f64 = 0.0;
        for s in &self.sessions {
            if s.started_at.format("%Y-%m-%d").to_string() == today {
                daily_cost += s.estimated_cost;
            }
            if s.started_at.format("%Y-%m").to_string() == this_month {
                monthly_cost += s.estimated_cost;
            }
        }
        if let Some(ref cur) = self.current {
            if cur.started_at.format("%Y-%m-%d").to_string() == today {
                daily_cost += cur.estimated_cost;
            }
            if cur.started_at.format("%Y-%m").to_string() == this_month {
                monthly_cost += cur.estimated_cost;
            }
        }

        // Daily-level check
        if daily_cost > max_daily {
            return BudgetStatus::Exceeded {
                action: action.clone(),
                message: format!(
                    "Daily cost ${:.2} exceeds limit ${:.2}",
                    daily_cost, max_daily
                ),
            };
        }

        // Monthly-level check
        if monthly_cost > max_monthly {
            return BudgetStatus::Exceeded {
                action: action.clone(),
                message: format!(
                    "Monthly cost ${:.2} exceeds limit ${:.2}",
                    monthly_cost, max_monthly
                ),
            };
        }

        // Warning thresholds (at 80%)
        let pct = |current, limit| -> f64 {
            if limit > 0.0 {
                current / limit
            } else {
                0.0
            }
        };
        let session_pct = pct(session_cost, max_session);
        let daily_pct = pct(daily_cost, max_daily);
        let monthly_pct = pct(monthly_cost, max_monthly);

        let mut warnings = Vec::new();
        if session_pct >= 0.8 {
            warnings.push(format!(
                "Session at {:.0}% of ${:.2} limit",
                session_pct * 100.0,
                max_session
            ));
        }
        if daily_pct >= 0.8 {
            warnings.push(format!(
                "Daily at {:.0}% of ${:.2} limit",
                daily_pct * 100.0,
                max_daily
            ));
        }
        if monthly_pct >= 0.8 {
            warnings.push(format!(
                "Monthly at {:.0}% of ${:.2} limit",
                monthly_pct * 100.0,
                max_monthly
            ));
        }

        if warnings.is_empty() {
            BudgetStatus::Ok
        } else {
            BudgetStatus::Warning {
                message: warnings.join("; "),
            }
        }
    }

    /// Convenience method used before LLM calls. Returns `None` if OK,
    /// `Some(warning)` if near/exceeding budget.
    /// Checks both the hard `--max-budget-usd` limit and the rich budget config.
    pub fn check_budget_and_warn(&self, session_cost: f64) -> Option<String> {
        // Hard limit check takes priority
        if let Some(msg) = self.check_max_budget_hard_limit() {
            return Some(msg);
        }
        match self.check_budget(session_cost) {
            BudgetStatus::Ok => None,
            BudgetStatus::Warning { message } => Some(format!("⚠️ {}", message)),
            BudgetStatus::Exceeded { action, message } => match action {
                BudgetAction::Stop => Some(format!(
                    "🛑 BUDGET EXCEEDED: {}. Session will stop.",
                    message
                )),
                BudgetAction::Pause => Some(format!(
                    "⏸️ BUDGET WARNING: {}. Processing paused.",
                    message
                )),
                BudgetAction::Warn => Some(format!("⚠️ BUDGET WARNING: {}", message)),
            },
        }
    }

    /// Reset billing period tracking (clears all cost data).
    pub fn reset_budget_period(&self, _period: BudgetPeriod) {
        *self.budget.lock().unwrap_or_else(|e| e.into_inner()) = BudgetConfig::default();
        self.save_budget_config();
    }

    /// Formatted budget status string (for /budget status).
    pub fn budget_status(&self) -> String {
        let cfg = self.budget.lock().unwrap_or_else(|e| e.into_inner());
        if !cfg.enabled {
            return "Budget Status: DISABLED\n  Use `/budget enable` to activate budget limits."
                .to_string();
        }

        let session_cost = self.current_session_cost();
        let now = Utc::now();
        let today = now.format("%Y-%m-%d").to_string();
        let this_month = now.format("%Y-%m").to_string();

        let mut daily_cost: f64 = 0.0;
        let mut monthly_cost: f64 = 0.0;
        for s in &self.sessions {
            if s.started_at.format("%Y-%m-%d").to_string() == today {
                daily_cost += s.estimated_cost;
            }
            if s.started_at.format("%Y-%m").to_string() == this_month {
                monthly_cost += s.estimated_cost;
            }
        }
        if let Some(ref cur) = self.current {
            if cur.started_at.format("%Y-%m-%d").to_string() == today {
                daily_cost += cur.estimated_cost;
            }
            if cur.started_at.format("%Y-%m").to_string() == this_month {
                monthly_cost += cur.estimated_cost;
            }
        }

        let action_str = match cfg.action {
            BudgetAction::Warn => "Warn",
            BudgetAction::Pause => "Pause",
            BudgetAction::Stop => "Stop",
        };

        let session_pct = if cfg.max_session_cost > 0.0 {
            (session_cost / cfg.max_session_cost * 100.0).min(100.0)
        } else {
            0.0
        };
        let daily_pct = if cfg.max_daily_cost > 0.0 {
            (daily_cost / cfg.max_daily_cost * 100.0).min(100.0)
        } else {
            0.0
        };
        let monthly_pct = if cfg.max_monthly_cost > 0.0 {
            (monthly_cost / cfg.max_monthly_cost * 100.0).min(100.0)
        } else {
            0.0
        };

        format!(
            "Budget Status: ENABLED\n  Session limit:  ${:.2} (current: ${:.2} — {:.0}%)\n  Daily limit:    ${:.2} (current: ${:.2} — {:.0}%)\n  Monthly limit:  ${:.2} (current: ${:.2} — {:.0}%)\n  Action on exceed: {}",
            cfg.max_session_cost, session_cost, session_pct,
            cfg.max_daily_cost, daily_cost, daily_pct,
            cfg.max_monthly_cost, monthly_cost, monthly_pct,
            action_str,
        )
    }

    /// Reset all tracking data.
    pub fn reset(&mut self) {
        self.sessions.clear();
        self.current = None;
        self.budget_limit = None;
        self.budget_period = BudgetPeriod::Monthly;
        self.max_budget_usd = None;
        *self.budget.lock().unwrap_or_else(|e| e.into_inner()) = BudgetConfig::default();
        self.save_budget_config();
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add #[serial] to any new tests that use global singletons
    #[test]
    fn test_new_tracker_empty() {
        let t = CostTracker::new();
        assert_eq!(t.total_cost(), 0.0);
        assert_eq!(t.total_tokens(), (0, 0));
        assert_eq!(t.session_count(), 0);
        assert!(t.budget_remaining().is_none());
    }

    #[test]
    fn test_record_usage_without_session() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4o-mini", "openai", 1000, 500);
        assert!(t.total_cost() > 0.0);
        assert_eq!(t.total_tokens(), (1000, 500));
        assert_eq!(t.session_count(), 1);
    }

    #[test]
    fn test_record_usage_multiple_calls() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4o", "openai", 2000, 1000);
        t.record_usage("gpt-4o", "openai", 500, 300);
        assert_eq!(t.total_tokens(), (2500, 1300));
    }

    #[test]
    fn test_session_lifecycle() {
        let mut t = CostTracker::new();
        t.start_session("claude-3-sonnet", "anthropic", "test-session");
        t.record_usage("claude-3-sonnet", "anthropic", 100, 50);
        t.finish_session();
        assert_eq!(t.session_count(), 1);
        let s = &t.sessions()[0];
        assert_eq!(s.name, "test-session");
        assert_eq!(s.tokens_in, 100);
        assert_eq!(s.tokens_out, 50);
    }

    #[test]
    fn test_budget_setting() {
        let mut t = CostTracker::new();
        t.set_budget(10.0, BudgetPeriod::Daily);
        assert!((t.budget_remaining().unwrap() - 10.0).abs() < 0.001);
        assert_eq!(t.budget_period_label(), "daily");
        t.record_usage("gpt-4o-mini", "openai", 1000, 500);
        assert!(t.budget_remaining().unwrap() < 10.0);
    }

    #[test]
    fn test_summary() {
        let mut t = CostTracker::new();
        t.record_usage("deepseek-chat", "deepseek", 2000, 1000);
        let s = t.summary();
        assert_eq!(s.session_count, 1);
        assert_eq!(s.total_tokens_in, 2000);
        assert_eq!(s.top_model, "deepseek-chat");
    }

    #[test]
    fn test_reset() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4", "openai", 100, 50);
        assert_eq!(t.session_count(), 1);
        t.reset();
        assert_eq!(t.session_count(), 0);
        assert_eq!(t.total_cost(), 0.0);
    }

    #[test]
    fn test_model_pricing_known() {
        let (input, output) = CostTracker::model_price_per_1k("gpt-4o");
        assert!((input - 0.01).abs() < 0.0001);
        assert!((output - 0.03).abs() < 0.0001);
    }

    #[test]
    fn test_model_pricing_unknown_fallback() {
        let (input, output) = CostTracker::model_price_per_1k("unknown-model");
        assert!((input - 0.01).abs() < 0.0001);
        assert!((output - 0.03).abs() < 0.0001);
    }

    #[test]
    fn test_current_session_cost() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4o-mini", "openai", 1000, 500);
        assert!(t.current_session_cost() > 0.0);
        t.finish_session();
        assert_eq!(t.current_session_cost(), 0.0);
    }

    #[test]
    fn test_status_line_format() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4o-mini", "openai", 1000, 500);
        let line = t.status_line();
        assert!(line.contains("Cost: $"));
        assert!(line.contains("Tokens:"));
    }

    #[test]
    fn test_tool_call_tracking() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4", "openai", 100, 50);
        t.record_tool_call();
        t.record_tool_call();
        t.finish_session();
        assert_eq!(t.sessions()[0].tool_calls, 2);
    }

    #[test]
    fn test_budget_period_labels() {
        assert_eq!(BudgetPeriod::Daily.label(), "daily");
        assert_eq!(BudgetPeriod::Weekly.label(), "weekly");
        assert_eq!(BudgetPeriod::Monthly.label(), "monthly");
    }

    #[test]
    fn test_current_model_and_provider() {
        let mut t = CostTracker::new();
        t.record_usage("claude-3-haiku", "anthropic", 100, 50);
        assert_eq!(t.current_model(), "claude-3-haiku");
        assert_eq!(t.current_provider(), "anthropic");
        t.finish_session();
        assert_eq!(t.current_model(), "—");
    }

    // ── New budget system tests ──

    #[test]
    fn test_budget_config_default_disabled() {
        let cfg = BudgetConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.action, BudgetAction::Warn);
        assert!((cfg.max_session_cost - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_check_budget_disabled_returns_ok() {
        let t = CostTracker::new();
        *t.budget.lock().unwrap_or_else(|e| e.into_inner()) = BudgetConfig::default();
        assert_eq!(t.check_budget(999.0), BudgetStatus::Ok);
    }

    #[test]
    fn test_check_budget_session_exceeded() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4o", "openai", 1000, 500);
        t.set_budget_config(BudgetConfig {
            max_session_cost: 0.001,
            enabled: true,
            ..Default::default()
        });
        let status = t.check_budget(t.current_session_cost());
        assert!(matches!(
            status,
            BudgetStatus::Exceeded {
                action: BudgetAction::Warn,
                ..
            }
        ));
    }

    #[test]
    fn test_check_budget_and_warn_ok() {
        let t = CostTracker::new();
        assert!(t.check_budget_and_warn(0.0).is_none());
    }

    #[test]
    fn test_check_budget_and_warn_exceeded() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4o", "openai", 10000, 5000);
        t.set_budget_config(BudgetConfig {
            max_session_cost: 0.001,
            enabled: true,
            action: BudgetAction::Stop,
            ..Default::default()
        });
        let msg = t.check_budget_and_warn(t.current_session_cost());
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("BUDGET EXCEEDED"));
    }

    #[test]
    fn test_budget_status_disabled() {
        let t = CostTracker::new();
        let s = t.budget_status();
        assert!(s.contains("DISABLED"));
    }

    #[test]
    fn test_budget_status_enabled() {
        let mut t = CostTracker::new();
        t.record_usage("gpt-4o-mini", "openai", 1000, 500);
        t.set_budget_config(BudgetConfig {
            enabled: true,
            ..Default::default()
        });
        let s = t.budget_status();
        assert!(s.contains("ENABLED"));
        assert!(s.contains("Session limit"));
    }

    #[test]
    fn test_budget_action_default() {
        assert_eq!(BudgetAction::Warn, BudgetAction::Warn);
        assert_ne!(BudgetAction::Warn, BudgetAction::Stop);
        assert_ne!(BudgetAction::Warn, BudgetAction::Pause);
    }

    // ── Max budget USD tests ──

    #[test]
    fn test_max_budget_usd_default_none() {
        let t = CostTracker::new();
        assert!(t.max_budget_usd().is_none());
    }

    #[test]
    fn test_max_budget_usd_setter() {
        let mut t = CostTracker::new();
        t.set_max_budget_usd(5.0);
        assert_eq!(t.max_budget_usd(), Some(5.0));
    }

    #[test]
    fn test_check_max_budget_hard_limit_not_exceeded() {
        let mut t = CostTracker::new();
        t.set_max_budget_usd(100.0);
        t.record_usage("gpt-4o-mini", "openai", 1000, 500);
        assert!(t.check_max_budget_hard_limit().is_none());
    }

    #[test]
    fn test_check_max_budget_hard_limit_exceeded() {
        let mut t = CostTracker::new();
        t.set_max_budget_usd(0.001);
        t.record_usage("gpt-4o", "openai", 10000, 5000);
        let msg = t.check_max_budget_hard_limit();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("BUDGET EXCEEDED"));
    }

    #[test]
    fn test_check_max_budget_hard_limit_warning() {
        let mut t = CostTracker::new();
        t.set_max_budget_usd(10.0);
        t.record_usage("gpt-4o-mini", "openai", 2000000, 1000000);
        let msg = t.check_max_budget_hard_limit();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("BUDGET WARNING"));
    }

    #[test]
    fn test_check_budget_and_warn_with_max_budget_exceeded() {
        let mut t = CostTracker::new();
        t.set_max_budget_usd(0.001);
        t.record_usage("gpt-4o", "openai", 10000, 5000);
        let msg = t.check_budget_and_warn(t.current_session_cost());
        assert!(msg.is_some());
        let m = msg.unwrap();
        assert!(m.contains("BUDGET EXCEEDED"));
        assert!(m.contains("max-budget-usd"));
    }

    #[test]
    fn test_max_budget_reset() {
        let mut t = CostTracker::new();
        t.set_max_budget_usd(5.0);
        assert!(t.max_budget_usd().is_some());
        t.reset();
        assert!(t.max_budget_usd().is_none());
    }

    #[test]
    fn test_max_budget_no_limit_returns_none() {
        let t = CostTracker::new();
        assert!(t.check_max_budget_hard_limit().is_none());
    }
}
