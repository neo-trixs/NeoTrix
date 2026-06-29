use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ThreatCategory {
    PromptInjection,
    Jailbreak,
    DataPoisoning,
    ModelInversion,
    MembershipInference,
    SupplyChainAttack,
    AdversarialExample,
    ToolMisuse,
    PrivilegeEscalation,
    InfoLeakage,
    DenialOfService,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ThreatAssessment {
    pub category: ThreatCategory,
    pub confidence: f64,
    pub severity: f64,
    pub description: String,
    pub suggested_action: String,
}

#[derive(Clone)]
pub struct ThreatModeler {
    pub taxonomy: HashMap<String, ThreatCategory>,
    pub known_patterns: Vec<(String, ThreatCategory, f64)>, // (regex_pattern, category, base_confidence)
    pub assessments: Vec<ThreatAssessment>,
    pub max_history: usize,
}

impl ThreatModeler {
    pub fn new() -> Self {
        let mut taxonomy = HashMap::new();
        taxonomy.insert(
            "prompt_injection".to_string(),
            ThreatCategory::PromptInjection,
        );
        taxonomy.insert("jailbreak".to_string(), ThreatCategory::Jailbreak);
        taxonomy.insert("data_poisoning".to_string(), ThreatCategory::DataPoisoning);
        taxonomy.insert(
            "model_inversion".to_string(),
            ThreatCategory::ModelInversion,
        );
        taxonomy.insert(
            "supply_chain".to_string(),
            ThreatCategory::SupplyChainAttack,
        );
        taxonomy.insert("tool_misuse".to_string(), ThreatCategory::ToolMisuse);
        taxonomy.insert(
            "privilege_escalation".to_string(),
            ThreatCategory::PrivilegeEscalation,
        );

        let known_patterns = vec![
            (
                "ignore previous instructions".to_string(),
                ThreatCategory::PromptInjection,
                0.85,
            ),
            (
                "forget all".to_string(),
                ThreatCategory::PromptInjection,
                0.75,
            ),
            ("DAN".to_string(), ThreatCategory::Jailbreak, 0.90),
            (
                "do anything now".to_string(),
                ThreatCategory::Jailbreak,
                0.85,
            ),
            ("bypass".to_string(), ThreatCategory::Jailbreak, 0.60),
            (
                "sudo".to_string(),
                ThreatCategory::PrivilegeEscalation,
                0.50,
            ),
            (
                "admin".to_string(),
                ThreatCategory::PrivilegeEscalation,
                0.45,
            ),
            ("password".to_string(), ThreatCategory::InfoLeakage, 0.40),
            ("api_key".to_string(), ThreatCategory::InfoLeakage, 0.55),
            (
                ":(){:|:&};:".to_string(),
                ThreatCategory::DenialOfService,
                0.95,
            ),
        ];

        ThreatModeler {
            taxonomy,
            known_patterns,
            assessments: Vec::new(),
            max_history: 100,
        }
    }

    pub fn classify(&mut self, input: &str) -> ThreatAssessment {
        let lower = input.to_lowercase();
        for (pattern, category, base_conf) in &self.known_patterns {
            if lower.contains(pattern) {
                let assessment = ThreatAssessment {
                    category: category.clone(),
                    confidence: *base_conf,
                    severity: (*base_conf * 0.9 + 0.1).min(1.0),
                    description: format!("matched known threat pattern: '{}'", pattern),
                    suggested_action: match category {
                        ThreatCategory::PromptInjection | ThreatCategory::Jailbreak => {
                            "block_input".to_string()
                        }
                        ThreatCategory::PrivilegeEscalation => "restrict_permissions".to_string(),
                        ThreatCategory::InfoLeakage => "redact_output".to_string(),
                        ThreatCategory::DenialOfService => "rate_limit".to_string(),
                        _ => "flag_for_review".to_string(),
                    },
                };
                self.assessments.push(assessment.clone());
                if self.assessments.len() > self.max_history {
                    self.assessments.remove(0);
                }
                return assessment;
            }
        }
        ThreatAssessment {
            category: ThreatCategory::Unknown,
            confidence: 0.1,
            severity: 0.1,
            description: "no known threat pattern matched".to_string(),
            suggested_action: "allow".to_string(),
        }
    }

    pub fn classify_vsa(&mut self, vsa_bytes: &[u8]) -> ThreatAssessment {
        let text = String::from_utf8_lossy(vsa_bytes);
        self.classify(&text)
    }

    pub fn risk_score(&self) -> f64 {
        if self.assessments.is_empty() {
            return 0.0;
        }
        let recent: Vec<&ThreatAssessment> = self.assessments.iter().rev().take(10).collect();
        let total: f64 = recent.iter().map(|a| a.severity * a.confidence).sum();
        (total / recent.len() as f64).min(1.0)
    }

    pub fn recent_threats(&self, n: usize) -> Vec<&ThreatAssessment> {
        self.assessments.iter().rev().take(n).collect()
    }

    pub fn reset(&mut self) {
        self.assessments.clear();
    }
}
