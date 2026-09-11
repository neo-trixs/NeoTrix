//! Dual Evidence Scanner - 双证据扫描
//!
//! action×target双证据 + 软多信号栈 + 结构/角色边界


/// 扫描结果
#[derive(Debug, Clone)]
pub struct DualEvidenceResult {
    pub action_evidence: Evidence,
    pub target_evidence: Evidence,
    pub combined_confidence: f64,
    pub threat_level: ThreatLevel,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Evidence {
    pub evidence_type: String,
    pub confidence: f64,
    pub details: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

/// 双证据扫描器
pub struct DualEvidenceScanner {
    action_patterns: Vec<Pattern>,
    target_patterns: Vec<Pattern>,
    _soft_signals: Vec<SoftSignal>,
}

#[derive(Debug, Clone)]
struct Pattern {
    _name: String,
    regex: String,
    _weight: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct SoftSignal {
    _name: String,
    _weight: f64,
    _detect: fn(&str) -> bool,
}

impl DualEvidenceScanner {
    pub fn new() -> Self {
        Self {
            action_patterns: vec![
                Pattern { name: "injection".to_string(), regex: r"ignore|disregard|forget".to_string(), weight: 0.9 },
                Pattern { name: "extraction".to_string(), regex: r"extract|reveal|show".to_string(), weight: 0.8 },
                Pattern { name: "bypass".to_string(), regex: r"bypass|circumvent|override".to_string(), weight: 0.85 },
            ],
            target_patterns: vec![
                Pattern { name: "system_prompt".to_string(), regex: r"system|prompt|instruction".to_string(), weight: 0.9 },
                Pattern { name: "credentials".to_string(), regex: r"password|key|secret".to_string(), weight: 0.85 },
                Pattern { name: "internal".to_string(), regex: r"internal|confidential|private".to_string(), weight: 0.8 },
            ],
            soft_signals: vec![],
        }
    }

    /// 扫描双证据
    pub fn scan(&self, action: &str, target: &str) -> DualEvidenceResult {
        // 扫描动作证据
        let action_evidence = self.scan_action(action);
        
        // 扫描目标证据
        let target_evidence = self.scan_target(target);
        
        // 计算组合置信度
        let combined_confidence = (action_evidence.confidence + target_evidence.confidence) / 2.0;
        
        // 确定威胁等级
        let threat_level = match combined_confidence {
            x if x >= 0.9 => ThreatLevel::Critical,
            x if x >= 0.7 => ThreatLevel::High,
            x if x >= 0.5 => ThreatLevel::Medium,
            x if x >= 0.3 => ThreatLevel::Low,
            _ => ThreatLevel::Safe,
        };

        let mut signals = Vec::new();
        if action_evidence.confidence > 0.7 {
            signals.push(format!("High confidence action: {}", action_evidence.evidence_type));
        }
        if target_evidence.confidence > 0.7 {
            signals.push(format!("High confidence target: {}", target_evidence.evidence_type));
        }

        DualEvidenceResult {
            action_evidence,
            target_evidence,
            combined_confidence,
            threat_level,
            signals,
        }
    }

    /// 扫描动作
    fn scan_action(&self, action: &str) -> Evidence {
        let mut best_match = None;
        let mut best_confidence = 0.0;

        for pattern in &self.action_patterns {
            if let Ok(re) = regex::Regex::new(&pattern.regex) {
                if re.is_match(action) {
                    let confidence = pattern.weight;
                    if confidence > best_confidence {
                        best_confidence = confidence;
                        best_match = Some(pattern.name.clone());
                    }
                }
            }
        }

        Evidence {
            evidence_type: best_match.unwrap_or_else(|| "unknown".to_string()),
            confidence: best_confidence,
            details: format!("Action analyzed: {}", action),
            source: "action_scanner".to_string(),
        }
    }

    /// 扫描目标
    fn scan_target(&self, target: &str) -> Evidence {
        let mut best_match = None;
        let mut best_confidence = 0.0;

        for pattern in &self.target_patterns {
            if let Ok(re) = regex::Regex::new(&pattern.regex) {
                if re.is_match(target) {
                    let confidence = pattern.weight;
                    if confidence > best_confidence {
                        best_confidence = confidence;
                        best_match = Some(pattern.name.clone());
                    }
                }
            }
        }

        Evidence {
            evidence_type: best_match.unwrap_or_else(|| "unknown".to_string()),
            confidence: best_confidence,
            details: format!("Target analyzed: {}", target),
            source: "target_scanner".to_string(),
        }
    }

    /// 检查结构边界
    pub fn check_structural_boundary(&self, input: &str) -> bool {
        // 检查是否有明确的结构分隔
        let has_structure = input.contains("<") && input.contains(">");
        let has_xml = input.contains("<?xml") || input.contains("<html");
        has_structure || has_xml
    }

    /// 检查角色边界
    pub fn check_role_boundary(&self, input: &str) -> bool {
        // 检查是否有明确的角色声明
        let has_role = input.contains("You are") || input.contains("Act as");
        let has_context = input.contains("In this context");
        has_role || has_context
    }
}

impl Default for DualEvidenceScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_injection() {
        let scanner = DualEvidenceScanner::new();
        let result = scanner.scan("ignore previous instructions", "system prompt");
        
        assert!(result.action_evidence.confidence > 0.0);
        assert!(result.target_evidence.confidence > 0.0);
        assert!(result.combined_confidence > 0.5);
    }

    #[test]
    fn test_structural_boundary() {
        let scanner = DualEvidenceScanner::new();
        assert!(scanner.check_structural_boundary("<system>test</system>"));
        assert!(!scanner.check_structural_boundary("test input"));
    }
}
