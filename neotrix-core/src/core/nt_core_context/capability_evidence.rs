//! # CapabilityEvidence — 证据支持的模型参数声明
//!
//! 借鉴 Ouroboros capability_evidence.py:
//!   每个上下文窗口声明都有来源
//!   EvidenceSource::ProviderMetadata | LocalProbe | UserAck
//!   带 EvidenceStatus::Verified | Claimed | Unknown

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 证据来源
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceSource {
    ProviderMetadata, // 来自 LLM 提供商文档
    LocalProbe,       // 本地测试/测量
    UserAck,          // 用户确认
    CodeAnalysis,     // 代码分析
    Historical,       // 历史运行数据
}

// ============================================================
// 证据状态
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceStatus {
    Verified, // 已通过验证
    Claimed,  // 已声明但未验证
    Unknown,  // 未知
    Disputed, // 证据矛盾
}

// ============================================================
// 能力证据
// ============================================================

/// 单个能力声明及其证据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEvidence {
    pub capability: String, // 能力名称 (e.g. "max_context_tokens")
    pub value: String,      // 声明值 (e.g. "128000")
    pub source: EvidenceSource,
    pub status: EvidenceStatus,
    pub confidence: f64,         // [0, 1]
    pub evidence_detail: String, // 证据描述
    pub last_verified: u64,      // unix epoch ms
}

impl CapabilityEvidence {
    pub fn new(capability: &str, value: &str, source: EvidenceSource) -> Self {
        Self {
            capability: capability.to_string(),
            value: value.to_string(),
            source,
            status: EvidenceStatus::Claimed,
            confidence: 0.5,
            evidence_detail: String::new(),
            last_verified: now_ms(),
        }
    }

    /// 验证证据
    pub fn verify(&mut self, detail: &str) {
        self.status = EvidenceStatus::Verified;
        self.confidence = 0.95;
        self.evidence_detail = detail.to_string();
        self.last_verified = now_ms();
    }

    /// 标记为有争议
    pub fn dispute(&mut self, reason: &str) {
        self.status = EvidenceStatus::Disputed;
        self.confidence = 0.1;
        self.evidence_detail = reason.to_string();
    }
}

fn now_ms() -> u64 {
    crate::core::nt_core_time::unix_now_ms()
}

// ============================================================
// 证据注册表
// ============================================================

/// 能力证据注册表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistry {
    pub entries: HashMap<String, Vec<CapabilityEvidence>>,
    pub max_entries_per_cap: usize,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries_per_cap: 5,
        }
    }

    /// 注册一个能力声明
    pub fn register(
        &mut self,
        capability: &str,
        value: &str,
        source: EvidenceSource,
    ) -> &mut CapabilityEvidence {
        let evidence = CapabilityEvidence::new(capability, value, source);
        self.entries
            .entry(capability.to_string())
            .or_default()
            .push(evidence);

        // 限制条目数
        if let Some(entries) = self.entries.get(capability) {
            if entries.len() > self.max_entries_per_cap {
                // 保留最近的
                let mut trimmed = entries.clone();
                trimmed.sort_by(|a, b| b.last_verified.cmp(&a.last_verified));
                trimmed.truncate(self.max_entries_per_cap);
                self.entries.insert(capability.to_string(), trimmed);
            }
        }

        self.entries
            .get_mut(capability)
            .and_then(|e| e.last_mut())
            .expect("capability entry was just created or retrieved")
    }

    /// 获取指定能力的最可信证据
    pub fn best_evidence(&self, capability: &str) -> Option<&CapabilityEvidence> {
        self.entries.get(capability).and_then(|entries| {
            entries
                .iter()
                .filter(|e| e.status == EvidenceStatus::Verified)
                .max_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .or_else(|| {
                    entries.iter().max_by(|a, b| {
                        a.confidence
                            .partial_cmp(&b.confidence)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                })
        })
    }

    /// 获取能力的聚合置信度
    pub fn consensus_value(&self, capability: &str) -> Option<(String, f64)> {
        let entries = self.entries.get(capability)?;
        if entries.is_empty() {
            return None;
        }
        // 取已验证 + 最高置信度的值
        let best = entries
            .iter()
            .filter(|e| e.status != EvidenceStatus::Disputed)
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })?;
        Some((best.value.clone(), best.confidence))
    }

    /// 反驳一个能力声明
    pub fn dispute(&mut self, capability: &str, reason: &str) -> bool {
        if let Some(entries) = self.entries.get_mut(capability) {
            for e in entries.iter_mut() {
                e.dispute(reason);
            }
            true
        } else {
            false
        }
    }

    /// 统计报告
    pub fn report(&self) -> CapabilityReport {
        let mut report = CapabilityReport::default();
        for (_cap, entries) in &self.entries {
            report.total_claims += entries.len();
            for e in entries {
                match e.status {
                    EvidenceStatus::Verified => report.verified += 1,
                    EvidenceStatus::Claimed => report.claimed += 1,
                    EvidenceStatus::Unknown => report.unknown += 1,
                    EvidenceStatus::Disputed => report.disputed += 1,
                }
            }
        }
        report
    }

    /// 注册 LLM 模型参数声明
    pub fn register_provider_model(
        &mut self,
        provider: &str,
        model: &str,
        max_tokens: usize,
        source: EvidenceSource,
    ) {
        self.register(
            &format!("{}/{}/max_tokens", provider, model),
            &max_tokens.to_string(),
            source.clone(),
        );
        self.register(
            &format!("{}/{}/provider", provider, model),
            provider,
            source,
        );
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityReport {
    pub total_claims: usize,
    pub verified: usize,
    pub claimed: usize,
    pub unknown: usize,
    pub disputed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_verify() {
        let mut reg = CapabilityRegistry::new();
        let ev = reg.register(
            "max_context_tokens",
            "128000",
            EvidenceSource::ProviderMetadata,
        );
        assert_eq!(ev.status, EvidenceStatus::Claimed);
        assert_eq!(ev.confidence, 0.5);

        if let Some(ev) = reg
            .entries
            .get_mut("max_context_tokens")
            .and_then(|e| e.last_mut())
        {
            ev.verify("Per OpenAI docs: gpt-4-turbo context = 128K tokens");
        }

        let best = reg.best_evidence("max_context_tokens").unwrap();
        assert_eq!(best.status, EvidenceStatus::Verified);
        assert_eq!(best.confidence, 0.95);
    }

    #[test]
    fn test_consensus_value() {
        let mut reg = CapabilityRegistry::new();
        let ev = reg.register(
            "max_context_tokens",
            "128000",
            EvidenceSource::ProviderMetadata,
        );
        ev.verify("OpenAI docs");

        let (value, confidence) = reg.consensus_value("max_context_tokens").unwrap();
        assert_eq!(value, "128000");
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_dispute() {
        let mut reg = CapabilityRegistry::new();
        reg.register(
            "max_context_tokens",
            "128000",
            EvidenceSource::ProviderMetadata,
        );
        reg.dispute("max_context_tokens", "Contradicts local test results");
        let best = reg.best_evidence("max_context_tokens").unwrap();
        assert!(best.confidence < 0.2);
    }

    #[test]
    fn test_report() {
        let mut reg = CapabilityRegistry::new();
        let ev = reg.register("cap1", "v1", EvidenceSource::ProviderMetadata);
        ev.verify("docs");
        reg.register("cap2", "v2", EvidenceSource::LocalProbe);
        let report = reg.report();
        assert_eq!(report.total_claims, 2);
        assert_eq!(report.verified, 1);
        assert_eq!(report.claimed, 1);
    }

    #[test]
    fn test_max_entries() {
        let mut reg = CapabilityRegistry::new();
        reg.max_entries_per_cap = 3;
        for i in 0..10 {
            reg.register("test_cap", &format!("v{}", i), EvidenceSource::LocalProbe);
        }
        assert_eq!(reg.entries.get("test_cap").unwrap().len(), 3);
    }

    #[test]
    fn test_register_provider_model() {
        let mut reg = CapabilityRegistry::new();
        reg.register_provider_model("openai", "gpt-4", 128000, EvidenceSource::ProviderMetadata);
        assert!(reg.entries.contains_key("openai/gpt-4/max_tokens"));
        assert!(reg.entries.contains_key("openai/gpt-4/provider"));
    }

    #[test]
    fn test_nonexistent_capability() {
        let reg = CapabilityRegistry::new();
        assert!(reg.best_evidence("nonexistent").is_none());
        assert!(reg.consensus_value("nonexistent").is_none());
    }
}
