//! Search report — aggregate statistics and JSON export for social account searches.

use serde::{Deserialize, Serialize};

use super::social_search::{AccountStatus, PlatformResult};

/// Complete report from a social username search run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReport {
    /// The username that was searched.
    pub username: String,
    /// Total number of platforms that were checked.
    pub platforms_checked: usize,
    /// Number of platforms where the account was found.
    pub found_count: usize,
    /// Number of platforms that returned NotFound.
    pub not_found_count: usize,
    /// Number of platforms that returned an error.
    pub error_count: usize,
    /// Number of platforms that returned RateLimited.
    pub rate_limited_count: usize,
    /// Average response time across all platforms (ms).
    pub avg_response_time_ms: f64,
    /// Risk score: ratio of found platforms to total checked, weighted.
    /// Higher = more exposed identity. Range: 0.0–1.0.
    pub risk_score: f64,
    /// Per-platform results.
    pub results: Vec<PlatformResult>,
}

impl SearchReport {
    /// Build a report from raw results.
    pub fn build(username: &str, results: Vec<PlatformResult>, risk_weight: f64) -> Self {
        let platforms_checked = results.len();
        let found_count = results.iter().filter(|r| r.status == AccountStatus::Found).count();
        let not_found_count = results.iter().filter(|r| r.status == AccountStatus::NotFound).count();
        let error_count = results.iter().filter(|r| r.status == AccountStatus::Error).count();
        let rate_limited_count = results.iter().filter(|r| r.status == AccountStatus::RateLimited).count();

        let total_time: u64 = results.iter().map(|r| r.response_time_ms).sum();
        let avg_response_time_ms = if platforms_checked > 0 {
            total_time as f64 / platforms_checked as f64
        } else {
            0.0
        };

        let risk_score = if platforms_checked > 0 {
            (found_count as f64 / platforms_checked as f64 * risk_weight).min(1.0)
        } else {
            0.0
        };

        Self {
            username: username.to_string(),
            platforms_checked,
            found_count,
            not_found_count,
            error_count,
            rate_limited_count,
            avg_response_time_ms,
            risk_score,
            results,
        }
    }

    /// Serialize the report to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize the report to compact JSON.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Return only the results where the account was found.
    pub fn found_results(&self) -> Vec<&PlatformResult> {
        self.results
            .iter()
            .filter(|r| r.status == AccountStatus::Found)
            .collect()
    }

    /// Return only the results where the account was not found.
    pub fn not_found_results(&self) -> Vec<&PlatformResult> {
        self.results
            .iter()
            .filter(|r| r.status == AccountStatus::NotFound)
            .collect()
    }

    /// Risk level as a human-readable label.
    pub fn risk_level(&self) -> &'static str {
        match self.risk_score {
            s if s >= 0.7 => "HIGH",
            s if s >= 0.3 => "MEDIUM",
            s if s > 0.0 => "LOW",
            _ => "NONE",
        }
    }
}

impl std::fmt::Display for SearchReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══ Social Search Report ═══════════════════════")?;
        writeln!(f, "  Username:        {}", self.username)?;
        writeln!(f, "  Platforms:       {}", self.platforms_checked)?;
        writeln!(f, "  Found:           {}", self.found_count)?;
        writeln!(f, "  Not Found:       {}", self.not_found_count)?;
        writeln!(f, "  Errors:          {}", self.error_count)?;
        writeln!(f, "  Rate Limited:    {}", self.rate_limited_count)?;
        writeln!(f, "  Avg Response:    {:.0}ms", self.avg_response_time_ms)?;
        writeln!(f, "  Risk Score:      {:.2} ({})", self.risk_score, self.risk_level())?;
        writeln!(f, "─────────────────────────────────────────────────")?;
        for r in &self.results {
            let icon = match r.status {
                AccountStatus::Found => "✓",
                AccountStatus::NotFound => "✗",
                AccountStatus::Error => "!",
                AccountStatus::RateLimited => "~",
            };
            writeln!(f, "  {} {:<16} {} ({}ms)", icon, r.platform, r.url, r.response_time_ms)?;
        }
        writeln!(f, "═════════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(platform: &str, status: AccountStatus) -> PlatformResult {
        PlatformResult {
            platform: platform.into(),
            url: format!("https://example.com/{}", platform.to_lowercase()),
            status,
            response_time_ms: 100,
        }
    }

    #[test]
    fn build_report_counts() {
        let results = vec![
            make_result("GitHub", AccountStatus::Found),
            make_result("Twitter", AccountStatus::NotFound),
            make_result("Reddit", AccountStatus::Error),
            make_result("LinkedIn", AccountStatus::RateLimited),
        ];
        let report = SearchReport::build("testuser", results, 1.0);
        assert_eq!(report.platforms_checked, 4);
        assert_eq!(report.found_count, 1);
        assert_eq!(report.not_found_count, 1);
        assert_eq!(report.error_count, 1);
        assert_eq!(report.rate_limited_count, 1);
    }

    #[test]
    fn risk_score_calculation() {
        let results = vec![
            make_result("GitHub", AccountStatus::Found),
            make_result("Twitter", AccountStatus::Found),
            make_result("Reddit", AccountStatus::NotFound),
        ];
        let report = SearchReport::build("testuser", results, 1.0);
        assert!((report.risk_score - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn risk_score_capped_at_one() {
        let results = vec![make_result("GitHub", AccountStatus::Found)];
        let report = SearchReport::build("testuser", results, 10.0);
        assert!(report.risk_score <= 1.0);
    }

    #[test]
    fn risk_level_labels() {
        let r = SearchReport {
            username: "x".into(),
            platforms_checked: 10,
            found_count: 0,
            not_found_count: 10,
            error_count: 0,
            rate_limited_count: 0,
            avg_response_time_ms: 0.0,
            risk_score: 0.0,
            results: vec![],
        };
        assert_eq!(r.risk_level(), "NONE");

        let r = SearchReport { risk_score: 0.1, ..r };
        assert_eq!(r.risk_level(), "LOW");

        let r = SearchReport { risk_score: 0.5, ..r };
        assert_eq!(r.risk_level(), "MEDIUM");

        let r = SearchReport { risk_score: 0.8, ..r };
        assert_eq!(r.risk_level(), "HIGH");
    }

    #[test]
    fn found_results_filter() {
        let results = vec![
            make_result("GitHub", AccountStatus::Found),
            make_result("Twitter", AccountStatus::NotFound),
        ];
        let report = SearchReport::build("testuser", results, 1.0);
        assert_eq!(report.found_results().len(), 1);
        assert_eq!(report.not_found_results().len(), 1);
    }

    #[test]
    fn json_export() {
        let report = SearchReport::build("testuser", vec![], 1.0);
        let json = report.to_json().unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("platforms_checked"));
    }

    #[test]
    fn compact_json_export() {
        let report = SearchReport::build("testuser", vec![], 1.0);
        let json = report.to_json_compact().unwrap();
        assert!(json.contains("testuser"));
    }

    #[test]
    fn display_format() {
        let results = vec![make_result("GitHub", AccountStatus::Found)];
        let report = SearchReport::build("testuser", results, 1.0);
        let s = format!("{report}");
        assert!(s.contains("Social Search Report"));
        assert!(s.contains("testuser"));
    }

    #[test]
    fn empty_report() {
        let report = SearchReport::build("nobody", vec![], 1.0);
        assert_eq!(report.platforms_checked, 0);
        assert_eq!(report.risk_score, 0.0);
        assert_eq!(report.risk_level(), "NONE");
    }
}
