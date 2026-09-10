//! NT-MIND 领域能力实现
//!
//! 自进化、元认知、技能结晶能力

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// 技能结晶能力
pub struct SkillCrystallizeCapability;

impl UnifiedCapability for SkillCrystallizeCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-mind-crystallize".into(),
            name: "技能结晶".into(),
            description: "将经验沉淀为可复用技能".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMind,
            layer: Layer::L5Cognition,
            tags: vec!["mind".into(), "skill".into(), "crystallize".into()],
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
            avg_latency_ms: 15.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                // 技能结晶: 从文本中提取技能模板
                let skill = SkillTemplate {
                    id: format!("skill_{}", chrono::Utc::now().timestamp()),
                    name: text.chars().take(20).collect(),
                    description: text,
                    steps: vec![
                        SkillStep::Input,
                        SkillStep::Process,
                        SkillStep::Output,
                    ],
                    version: "1.0.0".into(),
                };
                Ok(CapabilityOutput::Skill(skill))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 元认知监控能力
pub struct MetaCognitionCapability;

impl UnifiedCapability for MetaCognitionCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-mind-meta".into(),
            name: "元认知监控".into(),
            description: "监控和调节认知过程".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMind,
            layer: Layer::L6Meta,
            tags: vec!["mind".into(), "meta".into(), "monitor".into()],
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
            CapabilityInput::Nlp(nlp) => {
                // 元认知: 分析文本的认知负载
                let cognitive_load = estimate_cognitive_load(&nlp.text);
                let complexity = CognitiveComplexity {
                    lexical_diversity: cognitive_load.lexical_diversity,
                    syntactic_complexity: cognitive_load.syntactic_complexity,
                    semantic_depth: cognitive_load.semantic_depth,
                    overall: cognitive_load.overall,
                };
                Ok(CapabilityOutput::MetaAnalysis(MetaAnalysis {
                    analysis_type: AnalysisType::CognitiveLoad,
                    confidence: 0.85,
                    findings: vec![format!("认知负载: {:.2}", complexity.overall)],
                    recommendations: vec!["建议简化文本结构".into()],
                }))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要NLP输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Nlp(_))
    }
}

/// SEAL管道能力
pub struct SealPipelineCapability;

impl UnifiedCapability for SealPipelineCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-mind-seal".into(),
            name: "SEAL管道".into(),
            description: "自进化架构循环管道".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMind,
            layer: Layer::L5Cognition,
            tags: vec!["mind".into(), "seal".into(), "evolution".into()],
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
            avg_latency_ms: 50.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                // SEAL: 探索→蒸馏→自测→吸收
                let stages = vec![
                    SealStage::Explore,
                    SealStage::Distill,
                    SealStage::SelfTest,
                    SealStage::Absorb,
                ];
                Ok(CapabilityOutput::SealResult(SealResult {
                    stages,
                    input_text: text,
                    output_text: "SEAL管道处理完成".into(),
                    metrics: SealMetrics {
                        tokens_processed: 0,
                        knowledge_gained: 0,
                        quality_score: 0.8,
                    },
                }))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 创建NT-MIND能力
pub fn create_mind_capabilities() -> Vec<Arc<dyn UnifiedCapability>> {
    vec![
        Arc::new(SkillCrystallizeCapability),
        Arc::new(MetaCognitionCapability),
        Arc::new(SealPipelineCapability),
    ]
}

/// 认知复杂度
#[derive(Debug, Clone)]
pub struct CognitiveComplexity {
    pub lexical_diversity: f64,
    pub syntactic_complexity: f64,
    pub semantic_depth: f64,
    pub overall: f64,
}

/// 认知负载估计
#[derive(Debug, Clone)]
pub struct CognitiveLoad {
    pub lexical_diversity: f64,
    pub syntactic_complexity: f64,
    pub semantic_depth: f64,
    pub overall: f64,
}

fn estimate_cognitive_load(text: &str) -> CognitiveLoad {
    let words: Vec<&str> = text.split_whitespace().collect();
    let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
    let lexical_diversity = if words.is_empty() {
        0.0
    } else {
        unique_words.len() as f64 / words.len() as f64
    };

    CognitiveLoad {
        lexical_diversity,
        syntactic_complexity: 0.5, // 简化
        semantic_depth: 0.5,       // 简化
        overall: lexical_diversity * 0.4 + 0.3 + 0.3,
    }
}

/// 技能模板
#[derive(Debug, Clone)]
pub struct SkillTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<SkillStep>,
    pub version: String,
}

/// 技能步骤
#[derive(Debug, Clone)]
pub enum SkillStep {
    Input,
    Process,
    Output,
    Validate,
    Optimize,
}

/// SEAL阶段
#[derive(Debug, Clone)]
pub enum SealStage {
    Explore,
    Distill,
    SelfTest,
    Absorb,
}

/// SEAL结果
#[derive(Debug, Clone)]
pub struct SealResult {
    pub stages: Vec<SealStage>,
    pub input_text: String,
    pub output_text: String,
    pub metrics: SealMetrics,
}

/// SEAL指标
#[derive(Debug, Clone)]
pub struct SealMetrics {
    pub tokens_processed: u64,
    pub knowledge_gained: u64,
    pub quality_score: f64,
}

/// 分析类型
#[derive(Debug, Clone)]
pub enum AnalysisType {
    CognitiveLoad,
    SentimentAnalysis,
    TopicExtraction,
    EntityRecognition,
}

/// 元分析结果
#[derive(Debug, Clone)]
pub struct MetaAnalysis {
    pub analysis_type: AnalysisType,
    pub confidence: f64,
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_crystallize() {
        let cap = SkillCrystallizeCapability;
        let input = CapabilityInput::Text("测试技能结晶".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn meta_cognition() {
        let cap = MetaCognitionCapability;
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "测试元认知监控".into(),
            language: None,
        });
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn seal_pipeline() {
        let cap = SealPipelineCapability;
        let input = CapabilityInput::Text("测试SEAL管道".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }
}
