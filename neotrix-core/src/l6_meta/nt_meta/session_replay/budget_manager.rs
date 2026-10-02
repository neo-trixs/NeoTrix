#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};

/// Budget configuration for an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub max_tokens_per_session: Option<u64>,
    pub max_cost_per_session: Option<f64>,
    pub max_duration_ms: Option<u64>,
    pub alert_threshold: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_session: None,
            max_cost_per_session: None,
            max_duration_ms: None,
            alert_threshold: 0.8,
        }
    }
}

/// Current budget usage state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetUsage {
    pub tokens_used: u64,
    pub cost_usd: f64,
    pub duration_ms: u64,
}

/// Budget alert severity.
///
/// 2026-09-30 修正：原先 `derive(..., Eq)`，但变体 `Warning` 含 **`f64` 字段**
/// ⇒ `f64` **不满足** `Eq`（IEEE754 的 `NaN != NaN`，`Eq` 要求自反），
/// 导致 `E0277: the trait bound f64: std::cmp::Eq is not satisfied`。
/// ⇒ 去掉 `Eq`，保留 `PartialEq`（本类型未被用作 `HashSet`/`BTreeSet` 的键，
/// 已全模块核实，故 `Eq` 本就是多余的）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BudgetAlert {
    None,
    Warning { metric: String, usage_pct: f64 },
    Exceeded { metric: String, usage: String },
}

/// Tracks budget consumption and emits alerts/errors on exceedance.
pub struct BudgetTracker {
    config: BudgetConfig,
    usage: BudgetUsage,
    alerts: Vec<BudgetAlert>,
}

impl BudgetTracker {
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            usage: BudgetUsage::default(),
            alerts: Vec::new(),
        }
    }

    /// Record token consumption.
    pub fn record_tokens(&mut self, tokens: u64) {
        self.usage.tokens_used += tokens;
        self.check_thresholds();
    }

    /// Record cost consumption.
    pub fn record_cost(&mut self, cost: f64) {
        self.usage.cost_usd += cost;
        self.check_thresholds();
    }

    /// Record compute duration.
    pub fn record_duration(&mut self, duration_ms: u64) {
        self.usage.duration_ms += duration_ms;
        self.check_thresholds();
    }

    /// Set the alert threshold (0.0 - 1.0). Default is 0.8 (80%).
    pub fn set_alert_threshold(&mut self, threshold: f64) {
        self.config.alert_threshold = threshold.clamp(0.0, 1.0);
        self.check_thresholds();
    }

    /// Check if budget is still within limits.
    /// Returns Ok(remaining_budget_summary) or Err(BudgetExceeded).
    pub fn check_budget(&self) -> Result<BudgetStatus, BudgetExceeded> {
        if let Some(max_tokens) = self.config.max_tokens_per_session {
            if self.usage.tokens_used >= max_tokens {
                return Err(BudgetExceeded {
                    metric: "tokens".into(),
                    usage: format!("{}/{}", self.usage.tokens_used, max_tokens),
                });
            }
        }

        if let Some(max_cost) = self.config.max_cost_per_session {
            if self.usage.cost_usd >= max_cost {
                return Err(BudgetExceeded {
                    metric: "cost_usd".into(),
                    usage: format!("{:.6}/{:.6}", self.usage.cost_usd, max_cost),
                });
            }
        }

        if let Some(max_duration) = self.config.max_duration_ms {
            if self.usage.duration_ms >= max_duration {
                return Err(BudgetExceeded {
                    metric: "duration_ms".into(),
                    usage: format!("{}/{}", self.usage.duration_ms, max_duration),
                });
            }
        }

        Ok(BudgetStatus {
            remaining_tokens: self
                .config
                .max_tokens_per_session
                .map(|m| m.saturating_sub(self.usage.tokens_used)),
            remaining_cost: self
                .config
                .max_cost_per_session
                .map(|m| (m - self.usage.cost_usd).max(0.0)),
            remaining_duration_ms: self
                .config
                .max_duration_ms
                .map(|m| m.saturating_sub(self.usage.duration_ms)),
        })
    }

    /// Get current usage snapshot.
    pub fn usage(&self) -> &BudgetUsage {
        &self.usage
    }

    /// Get accumulated alerts.
    pub fn alerts(&self) -> &[BudgetAlert] {
        &self.alerts
    }

    /// Drain alerts, returning them and clearing.
    pub fn drain_alerts(&mut self) -> Vec<BudgetAlert> {
        std::mem::take(&mut self.alerts)
    }

    /// Check each metric against threshold and push alerts.
    fn check_thresholds(&mut self) {
        self.alerts.clear();

        if let Some(max_tokens) = self.config.max_tokens_per_session {
            let pct = self.usage.tokens_used as f64 / max_tokens as f64;
            if pct >= 1.0 {
                self.alerts.push(BudgetAlert::Exceeded {
                    metric: "tokens".into(),
                    usage: format!("{}/{}", self.usage.tokens_used, max_tokens),
                });
            } else if pct >= self.config.alert_threshold {
                self.alerts.push(BudgetAlert::Warning {
                    metric: "tokens".into(),
                    usage_pct: pct * 100.0,
                });
            }
        }

        if let Some(max_cost) = self.config.max_cost_per_session {
            let pct = self.usage.cost_usd / max_cost;
            if pct >= 1.0 {
                self.alerts.push(BudgetAlert::Exceeded {
                    metric: "cost_usd".into(),
                    usage: format!("{:.6}/{:.6}", self.usage.cost_usd, max_cost),
                });
            } else if pct >= self.config.alert_threshold {
                self.alerts.push(BudgetAlert::Warning {
                    metric: "cost_usd".into(),
                    usage_pct: pct * 100.0,
                });
            }
        }

        if let Some(max_duration) = self.config.max_duration_ms {
            let pct = self.usage.duration_ms as f64 / max_duration as f64;
            if pct >= 1.0 {
                self.alerts.push(BudgetAlert::Exceeded {
                    metric: "duration_ms".into(),
                    usage: format!("{}/{}", self.usage.duration_ms, max_duration),
                });
            } else if pct >= self.config.alert_threshold {
                self.alerts.push(BudgetAlert::Warning {
                    metric: "duration_ms".into(),
                    usage_pct: pct * 100.0,
                });
            }
        }
    }
}

/// Remaining budget after check_budget succeeds.
#[derive(Debug, Clone)]
pub struct BudgetStatus {
    pub remaining_tokens: Option<u64>,
    pub remaining_cost: Option<f64>,
    pub remaining_duration_ms: Option<u64>,
}

/// Error type when budget is exceeded.
#[derive(Debug, Clone)]
pub struct BudgetExceeded {
    pub metric: String,
    pub usage: String,
}

impl std::fmt::Display for BudgetExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Budget exceeded for {}: {}", self.metric, self.usage)
    }
}

impl std::error::Error for BudgetExceeded {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_budget_ok() {
        let config = BudgetConfig {
            max_tokens_per_session: Some(10_000),
            max_cost_per_session: Some(1.0),
            max_duration_ms: Some(60_000),
            alert_threshold: 0.8,
        };
        let mut tracker = BudgetTracker::new(config);
        tracker.record_tokens(5_000);
        tracker.record_cost(0.3);
        let status = tracker.check_budget().unwrap();
        assert_eq!(status.remaining_tokens, Some(5_000));
        assert!(status.remaining_cost.unwrap() > 0.0);
    }

    #[test]
    fn test_check_budget_exceeded() {
        let config = BudgetConfig {
            max_tokens_per_session: Some(100),
            max_cost_per_session: None,
            max_duration_ms: None,
            alert_threshold: 0.8,
        };
        let mut tracker = BudgetTracker::new(config);
        tracker.record_tokens(150);
        let err = tracker.check_budget().unwrap_err();
        assert_eq!(err.metric, "tokens");
    }

    #[test]
    fn test_alert_threshold() {
        let config = BudgetConfig {
            max_tokens_per_session: Some(1000),
            max_cost_per_session: None,
            max_duration_ms: None,
            alert_threshold: 0.8,
        };
        let mut tracker = BudgetTracker::new(config);
        tracker.record_tokens(850);
        let alerts = tracker.alerts();
        assert!(!alerts.is_empty());
        assert!(matches!(alerts[0], BudgetAlert::Warning { .. }));
    }

    #[test]
    fn test_set_alert_threshold() {
        let config = BudgetConfig {
            max_tokens_per_session: Some(1000),
            max_cost_per_session: None,
            max_duration_ms: None,
            alert_threshold: 0.8,
        };
        let mut tracker = BudgetTracker::new(config);
        tracker.record_tokens(700);
        assert!(tracker.alerts().is_empty());
        tracker.set_alert_threshold(0.5);
        assert!(!tracker.alerts().is_empty());
    }

    #[test]
    fn test_drain_alerts() {
        let config = BudgetConfig {
            max_tokens_per_session: Some(100),
            max_cost_per_session: None,
            max_duration_ms: None,
            alert_threshold: 0.5,
        };
        let mut tracker = BudgetTracker::new(config);
        tracker.record_tokens(80);
        let alerts = tracker.drain_alerts();
        assert!(!alerts.is_empty());
        assert!(tracker.alerts().is_empty());
    }

    #[test]
    fn test_display_error() {
        let err = BudgetExceeded {
            metric: "tokens".into(),
            usage: "150/100".into(),
        };
        assert!(err.to_string().contains("tokens"));
    }
}
