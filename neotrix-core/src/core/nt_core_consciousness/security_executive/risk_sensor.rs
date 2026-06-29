#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RiskReport {
    pub level: RiskLevel,
    pub score: f64,
    pub indicators: Vec<String>,
    pub recommended_action: String,
}

#[derive(Clone)]
pub struct RiskSensor {
    pub sensitivity: f64,
    pub alert_threshold: f64,
    pub risk_history: Vec<RiskReport>,
    pub max_history: usize,
}

impl RiskSensor {
    pub fn new(sensitivity: f64, max_history: usize) -> Self {
        RiskSensor {
            sensitivity: sensitivity.clamp(0.0, 1.0),
            alert_threshold: 0.6,
            risk_history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn assess_input(&mut self, data: &[u8], salience: f64) -> RiskReport {
        let text = String::from_utf8_lossy(data);
        let lower = text.to_lowercase();
        let mut indicators: Vec<String> = Vec::new();
        let mut raw_score: f64 = 0.0;

        let risk_patterns: Vec<(&str, f64, &str)> = vec![
            ("http", 0.1, "contains URL"),
            ("<script", 0.7, "contains script tag"),
            ("DROP TABLE", 0.8, "SQL injection pattern"),
            ("';", 0.6, "SQL injection syntax"),
            ("${", 0.5, "template injection pattern"),
            ("..\\", 0.6, "path traversal"),
            ("../", 0.5, "path traversal"),
            ("\\x00", 0.7, "null byte injection"),
            ("%00", 0.6, "URL-encoded null byte"),
            ("{system:", 0.8, "function call injection"),
            ("{exec:", 0.8, "function call injection"),
            ("eval(", 0.7, "dynamic code evaluation"),
            ("base64", 0.3, "encoded payload"),
            ("\\x", 0.3, "hex-encoded content"),
        ];

        for (pattern, weight, desc) in &risk_patterns {
            if lower.contains(pattern) {
                raw_score += weight;
                indicators.push(desc.to_string());
            }
        }

        let entropy = compute_entropy(data);
        if entropy > 6.0 {
            raw_score += 0.2 * (entropy / 8.0);
            indicators.push(format!("high entropy ({:.2})", entropy));
        }

        if salience > 0.8 {
            raw_score += 0.1;
            indicators.push("high salience".to_string());
        }

        let score = (raw_score * self.sensitivity).min(1.0);
        let level = if score >= 0.9 {
            RiskLevel::Critical
        } else if score >= 0.7 {
            RiskLevel::High
        } else if score >= 0.4 {
            RiskLevel::Medium
        } else if score >= 0.2 {
            RiskLevel::Low
        } else {
            RiskLevel::Safe
        };

        let report = RiskReport {
            level: level.clone(),
            score,
            indicators,
            recommended_action: match level {
                RiskLevel::Critical | RiskLevel::High => "block".to_string(),
                RiskLevel::Medium => "flag_for_review".to_string(),
                RiskLevel::Low => "monitor".to_string(),
                RiskLevel::Safe => "allow".to_string(),
            },
        };

        self.risk_history.push(report.clone());
        if self.risk_history.len() > self.max_history {
            self.risk_history.remove(0);
        }

        report
    }

    pub fn current_risk_level(&self) -> RiskLevel {
        self.risk_history
            .last()
            .map(|r| r.level.clone())
            .unwrap_or(RiskLevel::Safe)
    }

    pub fn average_risk_score(&self, n: usize) -> f64 {
        let recent: Vec<&RiskReport> = self.risk_history.iter().rev().take(n).collect();
        if recent.is_empty() {
            return 0.0;
        }
        recent.iter().map(|r| r.score).sum::<f64>() / recent.len() as f64
    }

    pub fn is_alerting(&self) -> bool {
        self.average_risk_score(5) > self.alert_threshold
    }

    pub fn reset(&mut self) {
        self.risk_history.clear();
    }
}

fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    let mut entropy = 0.0;
    for &count in freq.iter() {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}
