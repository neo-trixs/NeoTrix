//! Output Sentinel - 输出哨兵
//!
//! 多层输出验证:
//! 1. 规则匹配 (快速)
//! 2. 分类器检查 (语义)
//! 3. 策略验证 (合规)

use std::collections::HashMap;

/// 输出验证结果
#[derive(Debug, Clone)]
pub struct OutputValidationResult {
    pub is_safe: bool,
    pub threat_level: ThreatLevel,
    pub signals: Vec<OutputSignal>,
    pub sanitized_output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct OutputSignal {
    pub signal_type: String,
    pub confidence: f64,
    pub details: String,
}

/// 输出哨兵
pub struct OutputSentinel {
    blocked_patterns: Vec<String>,
    sensitive_patterns: Vec<String>,
    policy_rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone)]
struct PolicyRule {
    name: String,
    pattern: String,
    action: PolicyAction,
}

#[derive(Debug, Clone)]
enum PolicyAction {
    Block,
    Warn,
    Log,
}

impl OutputSentinel {
    pub fn new() -> Self {
        Self {
            blocked_patterns: vec![
                "system prompt".to_string(),
                "api key".to_string(),
                "secret".to_string(),
                "password".to_string(),
                "credentials".to_string(),
            ],
            sensitive_patterns: vec![
                "internal".to_string(),
                "confidential".to_string(),
                "proprietary".to_string(),
            ],
            policy_rules: vec![
                PolicyRule {
                    name: "no_executable_code".to_string(),
                    pattern: r"(exec|eval|system)\s*\(".to_string(),
                    action: PolicyAction::Block,
                },
                PolicyRule {
                    name: "no_urls".to_string(),
                    pattern: r"https?://[^\s]+".to_string(),
                    action: PolicyAction::Warn,
                },
            ],
        }
    }

    /// 验证输出
    pub fn validate(&self, output: &str, context: &HashMap<String, String>) -> OutputValidationResult {
        let mut signals = Vec::new();
        let mut max_threat = ThreatLevel::Safe;

        // 1. 阻断模式检测
        for pattern in &self.blocked_patterns {
            if output.to_lowercase().contains(pattern) {
                signals.push(OutputSignal {
                    signal_type: "blocked_pattern".to_string(),
                    confidence: 0.95,
                    details: format!("Blocked pattern found: {}", pattern),
                });
                max_threat = ThreatLevel::Critical;
            }
        }

        // 2. 敏感模式检测
        for pattern in &self.sensitive_patterns {
            if output.to_lowercase().contains(pattern) {
                signals.push(OutputSignal {
                    signal_type: "sensitive_pattern".to_string(),
                    confidence: 0.7,
                    details: format!("Sensitive pattern: {}", pattern),
                });
                if max_threat < ThreatLevel::Medium {
                    max_threat = ThreatLevel::Medium;
                }
            }
        }

        // 3. 策略规则检查
        for rule in &self.policy_rules {
            if let Ok(re) = regex::Regex::new(&rule.pattern) {
                if re.is_match(output) {
                    match rule.action {
                        PolicyAction::Block => {
                            signals.push(OutputSignal {
                                signal_type: "policy_block".to_string(),
                                confidence: 0.9,
                                details: format!("Policy rule triggered: {}", rule.name),
                            });
                            max_threat = ThreatLevel::High;
                        }
                        PolicyAction::Warn => {
                            signals.push(OutputSignal {
                                signal_type: "policy_warn".to_string(),
                                confidence: 0.6,
                                details: format!("Policy warning: {}", rule.name),
                            });
                            if max_threat < ThreatLevel::Low {
                                max_threat = ThreatLevel::Low;
                            }
                        }
                        PolicyAction::Log => {
                            signals.push(OutputSignal {
                                signal_type: "policy_log".to_string(),
                                confidence: 0.3,
                                details: format!("Policy log: {}", rule.name),
                            });
                        }
                    }
                }
            }
        }

        // 4. 系统提示泄露检测
        if let Some(system_prompt) = context.get("system_prompt") {
            if self.detect_prompt_leakage(output, system_prompt) {
                signals.push(OutputSignal {
                    signal_type: "prompt_leakage".to_string(),
                    confidence: 0.95,
                    details: "System prompt content detected in output".to_string(),
                });
                max_threat = ThreatLevel::Critical;
            }
        }

        let is_safe = max_threat == ThreatLevel::Safe || max_threat == ThreatLevel::Low;

        OutputValidationResult {
            is_safe,
            threat_level: max_threat,
            signals,
            sanitized_output: self.sanitize(output),
        }
    }

    /// 检测系统提示泄露
    fn detect_prompt_leakage(&self, output: &str, system_prompt: &str) -> bool {
        // 检查系统提示的片段是否出现在输出中
        let prompt_words: Vec<&str> = system_prompt.split_whitespace().collect();
        let output_lower = output.to_lowercase();

        // 检查连续5个词的匹配
        for window in prompt_words.windows(5) {
            let phrase = window.join(" ").to_lowercase();
            if output_lower.contains(&phrase) {
                return true;
            }
        }

        false
    }

    /// 清理输出
    fn sanitize(&self, output: &str) -> String {
        let mut sanitized = output.to_string();

        // 移除潜在的系统提示内容
        for pattern in &self.blocked_patterns {
            if let Ok(re) = regex::Regex::new(&format!(r"(?i){}", pattern)) {
                sanitized = re.replace_all(&sanitized, "[REDACTED]").to_string();
            }
        }

        sanitized
    }

    /// 添加自定义策略规则
    pub fn add_policy_rule(&mut self, name: &str, pattern: &str, action: PolicyAction) {
        self.policy_rules.push(PolicyRule {
            name: name.to_string(),
            pattern: pattern.to_string(),
            action,
        });
    }
}

impl Default for OutputSentinel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_output() {
        let sentinel = OutputSentinel::new();
        let result = sentinel.validate("The capital of France is Paris.", &HashMap::new());
        assert!(result.is_safe);
        assert_eq!(result.threat_level, ThreatLevel::Safe);
    }

    #[test]
    fn test_blocked_pattern() {
        let sentinel = OutputSentinel::new();
        let result = sentinel.validate("Here is the system prompt: ...", &HashMap::new());
        assert!(!result.is_safe);
        assert_eq!(result.threat_level, ThreatLevel::Critical);
    }

    #[test]
    fn test_prompt_leakage() {
        let sentinel = OutputSentinel::new();
        let mut context = HashMap::new();
        context.insert("system_prompt".to_string(), "You are a helpful assistant that never reveals secrets.".to_string());
        
        let result = sentinel.validate("You are a helpful assistant that never reveals secrets. Here is the answer...", &context);
        assert!(!result.is_safe);
        assert!(result.threat_level >= ThreatLevel::High);
    }
}
