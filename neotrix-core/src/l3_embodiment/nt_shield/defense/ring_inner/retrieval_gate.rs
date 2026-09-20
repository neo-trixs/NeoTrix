//! Retrieval Gate - 检索门控
//!
//! 知识访问分级：按信任等级过滤检索结果
//! 设计原则：防止低信任输入获取高敏感知识

use super::super::ring_core::TrustLevel;

/// 检索门控结果
#[derive(Debug, Clone)]
pub struct GateResult {
    pub allowed: bool,
    pub filtered_count: usize,
    pub total_count: usize,
    pub trust_required: TrustLevel,
    pub signals: Vec<String>,
}

/// 检索门控器
pub struct RetrievalGate {
    /// 知识敏感度分级
    sensitivity_map: Vec<(String, TrustLevel)>,
}

impl RetrievalGate {
    pub fn new() -> Self {
        Self {
            sensitivity_map: vec![
                // 低敏感：任何信任等级可访问
                ("public".to_string(), TrustLevel::Unknown),
                ("documentation".to_string(), TrustLevel::Unknown),
                // 中敏感：至少 Medium 信任
                ("internal".to_string(), TrustLevel::Medium),
                ("technical".to_string(), TrustLevel::Medium),
                // 高敏感：至少 High 信任
                ("confidential".to_string(), TrustLevel::High),
                ("secret".to_string(), TrustLevel::High),
                // 极高敏感：需要 Trusted
                ("credentials".to_string(), TrustLevel::Trusted),
                ("tokens".to_string(), TrustLevel::Trusted),
            ],
        }
    }

    /// 过滤检索结果
    pub fn filter(&self, results: &[(String, String)], user_trust: TrustLevel) -> GateResult {
        let mut filtered = Vec::new();
        let mut signals = Vec::new();

        for (content, namespace) in results {
            let required_trust = self.get_required_trust(namespace);

            if user_trust >= required_trust {
                filtered.push(content.clone());
            } else {
                signals.push(format!(
                    "Filtered '{}' (requires {:?}, user has {:?})",
                    namespace, required_trust, user_trust
                ));
            }
        }

        GateResult {
            allowed: filtered.len() == results.len(),
            filtered_count: results.len() - filtered.len(),
            total_count: results.len(),
            trust_required: TrustLevel::Medium,
            signals,
        }
    }

    /// 获取命名空间要求的信任等级
    fn get_required_trust(&self, namespace: &str) -> TrustLevel {
        self.sensitivity_map
            .iter()
            .find(|(name, _)| namespace.contains(name.as_str()))
            .map(|(_, level)| *level)
            .unwrap_or(TrustLevel::Medium) // 默认要求 Medium
    }
}

impl Default for RetrievalGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_public() {
        let gate = RetrievalGate::new();
        let results = vec![("content".to_string(), "public".to_string())];
        let result = gate.filter(&results, TrustLevel::Low);
        assert!(result.allowed);
    }

    #[test]
    fn test_gate_confidential() {
        let gate = RetrievalGate::new();
        let results = vec![("secret".to_string(), "confidential".to_string())];
        let result = gate.filter(&results, TrustLevel::Low);
        assert!(!result.allowed);
        assert_eq!(result.filtered_count, 1);
    }

    #[test]
    fn test_gate_trusted() {
        let gate = RetrievalGate::new();
        let results = vec![("token".to_string(), "credentials".to_string())];
        let result = gate.filter(&results, TrustLevel::Trusted);
        assert!(result.allowed);
    }
}
