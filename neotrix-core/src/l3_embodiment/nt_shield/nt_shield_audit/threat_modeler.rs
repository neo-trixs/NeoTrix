#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThreatCategory {
    Injection,
    Xss,
    Rce,
    PrivilegeEscalation,
    DataExfiltration,
    Dos,
    SupplyChain,
    Misconfiguration,
}

impl std::fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Injection => write!(f, "Injection"),
            Self::Xss => write!(f, "XSS"),
            Self::Rce => write!(f, "RCE"),
            Self::PrivilegeEscalation => write!(f, "Privesc"),
            Self::DataExfiltration => write!(f, "Exfil"),
            Self::Dos => write!(f, "DoS"),
            Self::SupplyChain => write!(f, "SupplyChain"),
            Self::Misconfiguration => write!(f, "Misconfig"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone)]
pub struct Threat {
    pub category: ThreatCategory,
    pub severity: Severity,
    pub description: String,
    pub mitigation: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ThreatModel {
    pub target: String,
    pub threats: Vec<Threat>,
    pub risk_score: f64,
}

pub struct ThreatModeler {
    rules: Vec<ThreatRule>,
}

#[derive(Debug, Clone)]
pub struct ThreatRule {
    category: ThreatCategory,
    severity: Severity,
    pattern: String,
    mitigation: String,
}

impl ThreatModeler {
    pub fn new() -> Self {
        let rules = vec![
            ThreatRule {
                category: ThreatCategory::Injection,
                severity: Severity::Critical,
                pattern: "execute(".into(),
                mitigation: "Use parameterized queries".into(),
            },
            ThreatRule {
                category: ThreatCategory::Rce,
                severity: Severity::Critical,
                pattern: "eval(".into(),
                mitigation: "Avoid eval, use safe parsing".into(),
            },
            ThreatRule {
                category: ThreatCategory::Rce,
                severity: Severity::Critical,
                pattern: "system(".into(),
                mitigation: "Validate and sanitize commands".into(),
            },
            ThreatRule {
                category: ThreatCategory::Xss,
                severity: Severity::High,
                pattern: "innerHTML".into(),
                mitigation: "Use textContent or sanitize".into(),
            },
            ThreatRule {
                category: ThreatCategory::DataExfiltration,
                severity: Severity::High,
                pattern: "send(".into(),
                mitigation: "Validate outbound data".into(),
            },
            ThreatRule {
                category: ThreatCategory::Misconfiguration,
                severity: Severity::Medium,
                pattern: "debug=true".into(),
                mitigation: "Disable debug in production".into(),
            },
            ThreatRule {
                category: ThreatCategory::PrivilegeEscalation,
                severity: Severity::High,
                pattern: "sudo".into(),
                mitigation: "Use least privilege".into(),
            },
        ];
        Self { rules }
    }

    pub fn analyze(&self, code: &str) -> ThreatModel {
        let mut threats = Vec::new();
        for rule in &self.rules {
            if code.contains(&rule.pattern) {
                threats.push(Threat {
                    category: rule.category.clone(),
                    severity: rule.severity.clone(),
                    description: format!("Found pattern: {}", rule.pattern),
                    mitigation: rule.mitigation.clone(),
                    references: Vec::new(),
                });
            }
        }
        let risk = threats
            .iter()
            .map(|t| match t.severity {
                Severity::Critical => 10.0,
                Severity::High => 7.0,
                Severity::Medium => 4.0,
                Severity::Low => 2.0,
                Severity::Info => 0.5,
            })
            .sum();
        ThreatModel {
            target: String::new(),
            threats,
            risk_score: risk,
        }
    }

    pub fn add_rule(&mut self, rule: ThreatRule) {
        self.rules.push(rule);
    }
    pub fn rules_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for ThreatModeler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_rce() {
        let m = ThreatModeler::new();
        let t = m.analyze("let result = eval(user_input);");
        assert!(t.threats.iter().any(|x| x.category == ThreatCategory::Rce));
        assert!(t.risk_score > 0.0);
    }

    #[test]
    fn test_analyze_clean() {
        let m = ThreatModeler::new();
        let t = m.analyze("let x = 5; println!(x);");
        assert!(t.threats.is_empty());
        assert_eq!(t.risk_score, 0.0);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
    }
}
