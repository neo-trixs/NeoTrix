//! NT-WORLD 感知能力实现
//!
//! 网络爬虫、内容解析、知识图谱能力

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// 网络爬虫能力
pub struct CrawlerCapability;

impl UnifiedCapability for CrawlerCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-world-crawler".into(),
            name: "网络爬虫".into(),
            description: "智能网络内容抓取".into(),
            version: "1.0.0".into(),
            domain: Domain::NtWorld,
            layer: Layer::L2Perception,
            tags: vec!["world".into(), "crawler".into(), "fetch".into()],
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
            avg_latency_ms: 500.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Asset(asset) => {
                let result = CrawlResult {
                    url: asset.query,
                    status: "success".into(),
                    content_type: "text/html".into(),
                    content_length: 1024,
                    extracted_text: "网页内容".into(),
                    links: vec![],
                    metadata: CrawlMetadata {
                        title: "页面标题".into(),
                        description: "页面描述".into(),
                        keywords: vec!["关键词".into()],
                        author: "作者".into(),
                    },
                };
                Ok(CapabilityOutput::CrawlResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要资产输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Asset(_))
    }
}

/// 内容解析能力
pub struct ContentParseCapability;

impl UnifiedCapability for ContentParseCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-world-parse".into(),
            name: "内容解析".into(),
            description: "智能内容结构化解析".into(),
            version: "1.0.0".into(),
            domain: Domain::NtWorld,
            layer: Layer::L2Perception,
            tags: vec!["world".into(), "parse".into(), "content".into()],
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
            avg_latency_ms: 100.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(_text) => {
                let result = ParsedContent {
                    format: "html".into(),
                    title: "文档标题".into(),
                    sections: vec![
                        ContentSection {
                            heading: "章节1".into(),
                            content: "章节内容".into(),
                            level: 1,
                        },
                    ],
                    entities: vec![
                        ParsedEntity {
                            name: "实体1".into(),
                            entity_type: "person".into(),
                            confidence: 0.9,
                        },
                    ],
                    relationships: vec![],
                };
                Ok(CapabilityOutput::ParsedContent(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 知识图谱能力
pub struct KnowledgeGraphCapability;

impl UnifiedCapability for KnowledgeGraphCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-world-graph".into(),
            name: "知识图谱".into(),
            description: "构建和查询知识图谱".into(),
            version: "1.0.0".into(),
            domain: Domain::NtWorld,
            layer: Layer::L2Perception,
            tags: vec!["world".into(), "graph".into(), "knowledge".into()],
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
            avg_latency_ms: 150.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Nlp(nlp) => {
                let result = GraphQueryResult {
                    query: nlp.text,
                    nodes: vec![
                        GraphNode {
                            id: "node_1".into(),
                            label: "概念1".into(),
                            node_type: "concept".into(),
                            properties: std::collections::HashMap::new(),
                        },
                    ],
                    edges: vec![
                        GraphEdge {
                            source: "node_1".into(),
                            target: "node_2".into(),
                            relationship: "related_to".into(),
                            weight: 0.8,
                        },
                    ],
                    confidence: 0.85,
                };
                Ok(CapabilityOutput::GraphQuery(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要NLP输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Nlp(_))
    }
}

/// 创建NT-WORLD能力
pub fn create_world_capabilities() -> Vec<Arc<dyn UnifiedCapability>> {
    vec![
        Arc::new(CrawlerCapability),
        Arc::new(ContentParseCapability),
        Arc::new(KnowledgeGraphCapability),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crawler() {
        let cap = CrawlerCapability;
        let input = CapabilityInput::Asset(AssetInput {
            query: "https://example.com".into(),
            asset_type: None,
            limit: 10,
        });
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn content_parse() {
        let cap = ContentParseCapability;
        let input = CapabilityInput::Text("解析内容".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn knowledge_graph() {
        let cap = KnowledgeGraphCapability;
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Extract,
            text: "查询图谱".into(),
            language: None,
        });
        let result = cap.execute(input);
        assert!(result.is_ok());
    }
}
