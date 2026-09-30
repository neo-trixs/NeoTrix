//! 失败引导诊断 — 从失败中学习修复模式
//!
//! 当记忆系统出现失败时，诊断根因并推荐修复方案。
//! 成功的修复被记录为 FailurePattern，供后续失败匹配。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆失败类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryFailureType {
    /// 编码失败: 输入 → 向量/表示
    EncodingFailure,
    /// 存储失败: 写入/持久化
    StorageFailure,
    /// 检索失败: 查询 → 结果
    RetrievalFailure,
    /// 管理失败: 缓存/淘汰/预算
    ManagementFailure,
}

impl MemoryFailureType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EncodingFailure => "encoding_failure",
            Self::StorageFailure => "storage_failure",
            Self::RetrievalFailure => "retrieval_failure",
            Self::ManagementFailure => "management_failure",
        }
    }

    pub fn all() -> &'static [MemoryFailureType] {
        &[
            Self::EncodingFailure,
            Self::StorageFailure,
            Self::RetrievalFailure,
            Self::ManagementFailure,
        ]
    }
}

/// 失败诊断结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    pub failure_type: MemoryFailureType,
    pub root_cause: String,
    pub affected_module: String,
    pub recommendation: String,
    pub confidence: f64,
    pub matched_pattern: Option<String>,
}

/// 失败模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub symptom: String,
    pub cause: String,
    pub fix: String,
    pub success_rate: f64,
    pub match_count: usize,
    pub failure_type: MemoryFailureType,
}

/// 失败引导诊断器
pub struct FailureGuidedDiagnosis {
    pub diagnosis_history: Vec<DiagnosisResult>,
    pub pattern_library: Vec<FailurePattern>,
}

impl FailureGuidedDiagnosis {
    pub fn new() -> Self {
        let mut pattern_library = Self::default_patterns();
        pattern_library.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Self {
            diagnosis_history: Vec::new(),
            pattern_library,
        }
    }

    /// 默认内置模式库
    fn default_patterns() -> Vec<FailurePattern> {
        vec![
            FailurePattern {
                symptom: "token count exceeds limit".into(),
                cause: "Input text too long for encoder context window".into(),
                fix: "Split input into chunks with overlap, encode separately, merge embeddings".into(),
                success_rate: 0.9,
                match_count: 0,
                failure_type: MemoryFailureType::EncodingFailure,
            },
            FailurePattern {
                symptom: "dimension mismatch in embedding".into(),
                cause: "Encoder output dimension does not match store index dimension".into(),
                fix: "Rebuild index with correct dimension, or use projection layer".into(),
                success_rate: 0.85,
                match_count: 0,
                failure_type: MemoryFailureType::EncodingFailure,
            },
            FailurePattern {
                symptom: "write timeout or disk full".into(),
                cause: "Storage backend cannot accept writes".into(),
                fix: "Check disk space, implement write-ahead log, or switch to in-memory store".into(),
                success_rate: 0.8,
                match_count: 0,
                failure_type: MemoryFailureType::StorageFailure,
            },
            FailurePattern {
                symptom: "index corruption detected".into(),
                cause: "Concurrent writes without proper locking".into(),
                fix: "Add write locks, implement index versioning, or use atomic writes".into(),
                success_rate: 0.75,
                match_count: 0,
                failure_type: MemoryFailureType::StorageFailure,
            },
            FailurePattern {
                symptom: "zero results returned".into(),
                cause: "Query encoding differs from stored encoding, or index empty".into(),
                fix: "Verify query preprocessing matches storage, check index population".into(),
                success_rate: 0.85,
                match_count: 0,
                failure_type: MemoryFailureType::RetrievalFailure,
            },
            FailurePattern {
                symptom: "low recall, missing relevant results".into(),
                cause: "Retriever type mismatch with query characteristics".into(),
                fix: "Switch retriever type or use hybrid approach with multiple signals".into(),
                success_rate: 0.7,
                match_count: 0,
                failure_type: MemoryFailureType::RetrievalFailure,
            },
            FailurePattern {
                symptom: "memory pressure, evictions too aggressive".into(),
                cause: "Eviction policy does not match access pattern".into(),
                fix: "Switch to Adaptive manager, or increase memory budget".into(),
                success_rate: 0.8,
                match_count: 0,
                failure_type: MemoryFailureType::ManagementFailure,
            },
            FailurePattern {
                symptom: "stale data served after update".into(),
                cause: "Cache not invalidated on write".into(),
                fix: "Implement write-through caching or cache invalidation on write".into(),
                success_rate: 0.85,
                match_count: 0,
                failure_type: MemoryFailureType::ManagementFailure,
            },
        ]
    }

    /// 诊断失败
    pub fn diagnose(
        &self,
        failure_type: MemoryFailureType,
        context: &str,
    ) -> DiagnosisResult {
        let context_lower = context.to_lowercase();

        // 先从模式库中匹配
        let matched = self
            .pattern_library
            .iter()
            .filter(|p| p.failure_type == failure_type)
            .filter(|p| {
                let symptom_lower = p.symptom.to_lowercase();
                context_lower.contains(&symptom_lower)
                    || symptom_lower
                        .split_whitespace()
                        .any(|w| context_lower.contains(w))
            })
            .max_by(|a, b| {
                a.success_rate
                    .partial_cmp(&b.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(pattern) = matched {
            let affected_module = match failure_type {
                MemoryFailureType::EncodingFailure => "encoder".into(),
                MemoryFailureType::StorageFailure => "store".into(),
                MemoryFailureType::RetrievalFailure => "retriever".into(),
                MemoryFailureType::ManagementFailure => "manager".into(),
            };

            DiagnosisResult {
                failure_type,
                root_cause: pattern.cause.clone(),
                affected_module,
                recommendation: pattern.fix.clone(),
                confidence: pattern.success_rate,
                matched_pattern: Some(pattern.symptom.clone()),
            }
        } else {
            // 无匹配模式，生成通用诊断
            self.generic_diagnosis(failure_type, context)
        }
    }

    /// 通用诊断 (无模式匹配时)
    fn generic_diagnosis(
        &self,
        failure_type: MemoryFailureType,
        context: &str,
    ) -> DiagnosisResult {
        let (affected_module, root_cause, recommendation) = match failure_type {
            MemoryFailureType::EncodingFailure => (
                "encoder".into(),
                format!("Unrecognized encoding failure: {}", context),
                "Check encoder input format and model loading. \
                 Try fallback encoder (Extractive or Keyword)."
                    .into(),
            ),
            MemoryFailureType::StorageFailure => (
                "store".into(),
                format!("Unrecognized storage failure: {}", context),
                "Check storage backend health. \
                 Try switching to a simpler store (KeyValue)."
                    .into(),
            ),
            MemoryFailureType::RetrievalFailure => (
                "retriever".into(),
                format!("Unrecognized retrieval failure: {}", context),
                "Check query preprocessing. \
                 Try a different retriever type (BM25 or CosineSimilarity)."
                    .into(),
            ),
            MemoryFailureType::ManagementFailure => (
                "manager".into(),
                format!("Unrecognized management failure: {}", context),
                "Check memory budget and eviction settings. \
                 Try Adaptive manager with increased budget."
                    .into(),
            ),
        };

        DiagnosisResult {
            failure_type,
            root_cause,
            affected_module,
            recommendation,
            confidence: 0.3, // 低置信度: 无模式匹配
            matched_pattern: None,
        }
    }

    /// 学习新模式
    pub fn learn_pattern(&mut self, pattern: FailurePattern) {
        // 检查是否已有相似模式
        let existing = self
            .pattern_library
            .iter_mut()
            .find(|p| {
                p.failure_type == pattern.failure_type
                    && similarity(&p.symptom, &pattern.symptom) > 0.6
            });

        if let Some(existing) = existing {
            // 更新已有模式: 指数移动平均 success_rate
            existing.success_rate =
                0.7 * existing.success_rate + 0.3 * pattern.success_rate;
            existing.match_count += 1;
            // 如果新模式的修复更优，合并
            if pattern.success_rate > existing.success_rate {
                existing.fix = pattern.fix;
            }
        } else {
            self.pattern_library.push(pattern);
        }

        // 按 success_rate 排序
        self.pattern_library.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// 获取推荐修复
    pub fn recommend_fix(&self, diagnosis: &DiagnosisResult) -> String {
        // 优先使用匹配的模式
        if let Some(symptom) = &diagnosis.matched_pattern {
            if let Some(pattern) = self
                .pattern_library
                .iter()
                .find(|p| &p.symptom == symptom)
            {
                return format!(
                    "[confidence: {:.0}%] {}",
                    pattern.success_rate * 100.0,
                    pattern.fix
                );
            }
        }

        // 回退到诊断结果的推荐
        format!(
            "[confidence: {:.0}%] {}",
            diagnosis.confidence * 100.0,
            diagnosis.recommendation
        )
    }

    /// 诊断统计
    pub fn stats(&self) -> DiagnosisStats {
        let total_diagnoses = self.diagnosis_history.len();
        let by_type: HashMap<String, usize> = self
            .diagnosis_history
            .iter()
            .map(|d| (d.failure_type.as_str().to_string(), 1))
            .fold(HashMap::new(), |mut acc, (k, v)| {
                *acc.entry(k).or_insert(0) += v;
                acc
            });

        let avg_confidence = if total_diagnoses > 0 {
            self.diagnosis_history
                .iter()
                .map(|d| d.confidence)
                .sum::<f64>()
                / total_diagnoses as f64
        } else {
            0.0
        };

        let matched_count = self
            .diagnosis_history
            .iter()
            .filter(|d| d.matched_pattern.is_some())
            .count();

        DiagnosisStats {
            total_diagnoses,
            by_type,
            avg_confidence,
            pattern_match_rate: if total_diagnoses > 0 {
                matched_count as f64 / total_diagnoses as f64
            } else {
                0.0
            },
            patterns_in_library: self.pattern_library.len(),
        }
    }

    /// 记录一次诊断
    pub fn record_diagnosis(&mut self, result: DiagnosisResult) {
        self.diagnosis_history.push(result);
    }
}

impl Default for FailureGuidedDiagnosis {
    fn default() -> Self {
        Self::new()
    }
}

/// 诊断统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisStats {
    pub total_diagnoses: usize,
    pub by_type: HashMap<String, usize>,
    pub avg_confidence: f64,
    pub pattern_match_rate: f64,
    pub patterns_in_library: usize,
}

/// 简单词符串相似度 (Jaccard)
fn similarity(a: &str, b: &str) -> f64 {
    let set_a: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let set_b: std::collections::HashSet<&str> = b.split_whitespace().collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}
