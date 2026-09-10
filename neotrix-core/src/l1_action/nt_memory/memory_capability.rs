//! NT-MEMORY 领域能力实现
//!
//! 知识存储、检索、版本控制能力

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// 知识存储能力
pub struct KnowledgeStoreCapability;

impl UnifiedCapability for KnowledgeStoreCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-memory-store".into(),
            name: "知识存储".into(),
            description: "存储和检索知识条目".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMemory,
            layer: Layer::L1Action,
            tags: vec!["memory".into(), "store".into(), "knowledge".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 10.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let entry = KnowledgeEntry {
                    id: format!("kb_{}", chrono::Utc::now().timestamp()),
                    content: text,
                    metadata: KnowledgeMetadata {
                        source: "manual".into(),
                        confidence: 0.9,
                        tags: vec!["text".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    },
                    version: 1,
                };
                Ok(CapabilityOutput::Knowledge(entry))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 知识检索能力
pub struct KnowledgeRetrievalCapability;

impl UnifiedCapability for KnowledgeRetrievalCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-memory-retrieve".into(),
            name: "知识检索".into(),
            description: "基于语义相似度检索知识".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMemory,
            layer: Layer::L2Perception,
            tags: vec!["memory".into(), "retrieve".into(), "semantic".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 20.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Nlp(nlp) => {
                // 模拟检索结果
                let results = vec![
                    SearchResult {
                        id: "kb_1".into(),
                        content: format!("与\"{}\"相关的知识", nlp.text),
                        score: 0.85,
                        metadata: KnowledgeMetadata {
                            source: "search".into(),
                            confidence: 0.85,
                            tags: vec!["search".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        },
                    },
                ];
                Ok(CapabilityOutput::SearchResults(results))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要NLP输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Nlp(_))
    }
}

/// 知识版本管理能力
pub struct KnowledgeVersionCapability;

impl UnifiedCapability for KnowledgeVersionCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-memory-version".into(),
            name: "知识版本".into(),
            description: "知识条目版本控制".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMemory,
            layer: Layer::L1Action,
            tags: vec!["memory".into(), "version".into(), "control".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 5.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let version_history = VersionHistory {
                    entry_id: "kb_1".into(),
                    versions: vec![
                        Version {
                            number: 1,
                            content: text,
                            author: "system".into(),
                            timestamp: chrono::Utc::now(),
                            change_description: "初始版本".into(),
                        },
                    ],
                    current_version: 1,
                };
                Ok(CapabilityOutput::VersionHistory(version_history))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 创建NT-MEMORY能力
pub fn create_memory_capabilities() -> Vec<Arc<dyn UnifiedCapability>> {
    vec![
        Arc::new(KnowledgeStoreCapability),
        Arc::new(KnowledgeRetrievalCapability),
        Arc::new(KnowledgeVersionCapability),
    ]
}

/// 知识条目
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    pub id: String,
    pub content: String,
    pub metadata: KnowledgeMetadata,
    pub version: u32,
}

/// 知识元数据
#[derive(Debug, Clone)]
pub struct KnowledgeMetadata {
    pub source: String,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f64,
    pub metadata: KnowledgeMetadata,
}

/// 版本历史
#[derive(Debug, Clone)]
pub struct VersionHistory {
    pub entry_id: String,
    pub versions: Vec<Version>,
    pub current_version: u32,
}

/// 版本
#[derive(Debug, Clone)]
pub struct Version {
    pub number: u32,
    pub content: String,
    pub author: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub change_description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knowledge_store() {
        let cap = KnowledgeStoreCapability;
        let input = CapabilityInput::Text("测试知识存储".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn knowledge_retrieval() {
        let cap = KnowledgeRetrievalCapability;
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Search,
            text: "测试知识检索".into(),
            language: None,
        });
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn knowledge_version() {
        let cap = KnowledgeVersionCapability;
        let input = CapabilityInput::Text("测试版本管理".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }
}
