//! NT-WORLD NLP 统一能力接口实现

use std::sync::Arc;
use crate::core::nt_core_capability::*;

use super::nt_nlp_regex::InfoExtractor;
use super::nt_nlp_similarity::TextSimilarity;
use super::nt_nlp_detect::LanguageDetector;
use super::nt_nlp_keyword::KeywordExtractor;
use super::nt_nlp_sentiment::SentimentAnalyzer;
use super::nt_nlp_tokenizer::ChineseTokenizer;

/// NLP能力实现
pub struct NlpCapability {
    meta: CapabilityMeta,
    health: CapabilityHealth,
    extractor: InfoExtractor,
    sentiment: SentimentAnalyzer,
    tokenizer: ChineseTokenizer,
}

impl NlpCapability {
    /// 创建新的NLP能力
    pub fn new() -> Self {
        Self {
            meta: CapabilityMeta {
                id: "nt-world-nlp".into(),
                name: "NT-WORLD NLP".into(),
                layer: Layer::L2Perception,
                domain: Domain::NtWorld,
                version: "0.1.0".into(),
                description: "自然语言处理能力集合".into(),
                tags: vec!["nlp".into(), "chinese".into(), "text".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            },
            health: CapabilityHealth {
                state: CapabilityState::Ready,
                success_rate: 1.0,
                avg_latency_ms: 0.0,
                last_called: None,
                call_count: 0,
            },
            extractor: InfoExtractor::new().unwrap(),
            sentiment: SentimentAnalyzer::new(),
            tokenizer: ChineseTokenizer::new(),
        }
    }
}

impl UnifiedCapability for NlpCapability {
    fn meta(&self) -> CapabilityMeta {
        self.meta.clone()
    }

    fn health(&self) -> CapabilityHealth {
        self.health.clone()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        let nlp_input = match input {
            CapabilityInput::Nlp(nlp) => nlp,
            CapabilityInput::Text(text) => NlpInput {
                task: NlpTask::Tokenize,
                text,
                language: None,
            },
            _ => return Err(CapabilityError::UnsupportedInput("不支持的输入类型".into())),
        };

        let start = std::time::Instant::now();

        let result = match nlp_input.task {
            NlpTask::Tokenize => {
                let tokens = self.tokenizer.tokenize(&nlp_input.text);
                NlpResult::Tokens(tokens)
            }
            NlpTask::DetectLanguage => {
                let lang = LanguageDetector::detect(&nlp_input.text);
                NlpResult::Language(format!("{:?}", lang))
            }
            NlpTask::ExtractKeywords => {
                let keywords = KeywordExtractor::extract_by_tf(&nlp_input.text, 10);
                let result: Vec<KeywordResult> = keywords.into_iter()
                    .map(|k| KeywordResult { word: k.word, score: k.score })
                    .collect();
                NlpResult::Keywords(result)
            }
            NlpTask::SentimentAnalysis => {
                let score = self.sentiment.analyze(&nlp_input.text);
                NlpResult::Sentiment(SentimentResult {
                    polarity: format!("{:?}", score.polarity),
                    score: score.positive_score - score.negative_score,
                })
            }
            NlpTask::Similarity => {
                // 默认与空字符串比较
                let sim = TextSimilarity::cosine_similarity(&nlp_input.text, "");
                NlpResult::Similarity(sim)
            }
            NlpTask::ExtractInfo => {
                let emails = self.extractor.extract_emails(&nlp_input.text);
                let phones = self.extractor.extract_phones(&nlp_input.text);
                let urls = self.extractor.extract_urls(&nlp_input.text);

                let mut entities = Vec::new();
                for e in emails {
                    entities.push(EntityResult {
                        text: e.text,
                        entity_type: "email".into(),
                        start: e.start,
                        end: e.end,
                    });
                }
                for p in phones {
                    entities.push(EntityResult {
                        text: p.text,
                        entity_type: "phone".into(),
                        start: p.start,
                        end: p.end,
                    });
                }
                for u in urls {
                    entities.push(EntityResult {
                        text: u.text,
                        entity_type: "url".into(),
                        start: u.start,
                        end: u.end,
                    });
                }
                NlpResult::Entities(entities)
            }
            NlpTask::Classify => {
                // 简单分类: 根据语言
                let lang = LanguageDetector::detect(&nlp_input.text);
                NlpResult::Classification(ClassificationResult {
                    label: format!("{:?}", lang),
                    confidence: 0.8,
                })
            }
            NlpTask::Extract => {
                let emails = self.extractor.extract_emails(&nlp_input.text);
                let phones = self.extractor.extract_phones(&nlp_input.text);
                let urls = self.extractor.extract_urls(&nlp_input.text);
                let mut entities = Vec::new();
                for e in emails {
                    entities.push(EntityResult {
                        text: e.text,
                        entity_type: "email".into(),
                        start: e.start,
                        end: e.end,
                    });
                }
                for p in phones {
                    entities.push(EntityResult {
                        text: p.text,
                        entity_type: "phone".into(),
                        start: p.start,
                        end: p.end,
                    });
                }
                for u in urls {
                    entities.push(EntityResult {
                        text: u.text,
                        entity_type: "url".into(),
                        start: u.start,
                        end: u.end,
                    });
                }
                NlpResult::Entities(entities)
            }
        };

        let _elapsed = start.elapsed().as_millis() as u64;

        Ok(CapabilityOutput::Nlp(NlpOutput {
            task: nlp_input.task,
            result,
        }))
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Nlp(_) | CapabilityInput::Text(_))
    }
}

/// 创建NLP能力实例
pub fn create_nlp_capability() -> Arc<dyn UnifiedCapability> {
    Arc::new(NlpCapability::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nlp_capability_meta() {
        let cap = NlpCapability::new();
        let meta = cap.meta();
        assert_eq!(meta.id, "nt-world-nlp");
        assert_eq!(meta.layer, Layer::L2Perception);
        assert_eq!(meta.domain, Domain::NtWorld);
    }

    #[test]
    fn nlp_tokenize() {
        let cap = NlpCapability::new();
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "自然语言处理".into(),
            language: None,
        });

        let output = cap.execute(input).unwrap();
        match output {
            CapabilityOutput::Nlp(nlp) => {
                match nlp.result {
                    NlpResult::Tokens(tokens) => assert!(!tokens.is_empty()),
                    _ => panic!("Expected tokens"),
                }
            }
            _ => panic!("Expected NLP output"),
        }
    }

    #[test]
    fn nlp_sentiment() {
        let cap = NlpCapability::new();
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::SentimentAnalysis,
            text: "这个产品非常好".into(),
            language: None,
        });

        let output = cap.execute(input).unwrap();
        match output {
            CapabilityOutput::Nlp(nlp) => {
                match nlp.result {
                    NlpResult::Sentiment(s) => assert!(s.score > 0.0),
                    _ => panic!("Expected sentiment"),
                }
            }
            _ => panic!("Expected NLP output"),
        }
    }
}
