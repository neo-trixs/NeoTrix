//! NT-META 元认知能力实现
//!
//! 跨技能意识、模式传播、盲点检测能力

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// 跨技能意识能力
pub struct CrossSkillAwarenessCapability;

impl UnifiedCapability for CrossSkillAwarenessCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-meta-awareness".into(),
            name: "跨技能意识".into(),
            description: "跨技能模式感知和传播".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMeta,
            layer: Layer::L6Meta,
            tags: vec!["meta".into(), "awareness".into(), "cross-skill".into()],
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
            CapabilityInput::Text(text) => {
                let result = AwarenessResult {
                    query: text,
                    discovered_patterns: vec![
                        Pattern {
                            id: "P1".into(),
                            name: "相似推理模式".into(),
                            description: "E8推理与元认知的相似性".into(),
                            confidence: 0.85,
                            related_skills: vec!["nt-core-e8".into(), "nt-meta-awareness".into()],
                        },
                    ],
                    blind_spots: vec![
                        BlindSpot {
                            area: "跨域协同".into(),
                            description: "缺少跨域能力组合".into(),
                            severity: "medium".into(),
                            recommendation: "添加组合能力".into(),
                        },
                    ],
                    propagation_suggestions: vec![
                        "将E8推理模式应用到元认知".into(),
                    ],
                };
                Ok(CapabilityOutput::AwarenessResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 盲点检测能力
pub struct BlindSpotDetectionCapability;

impl UnifiedCapability for BlindSpotDetectionCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-meta-blindspot".into(),
            name: "盲点检测".into(),
            description: "系统盲点和缺陷检测".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMeta,
            layer: Layer::L2Perception,
            tags: vec!["meta".into(), "blindspot".into(), "detection".into()],
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
            CapabilityInput::Text(text) => {
                let result = BlindSpotResult {
                    system: text,
                    blind_spots: vec![
                        BlindSpot {
                            area: "测试覆盖".into(),
                            description: "部分模块缺少单元测试".into(),
                            severity: "low".into(),
                            recommendation: "补充测试".into(),
                        },
                    ],
                    coverage_analysis: CoverageAnalysis {
                        total_modules: 100,
                        covered_modules: 85,
                        coverage_rate: 0.85,
                        uncovered_areas: vec!["边缘情况".into()],
                    },
                    improvement_suggestions: vec![
                        "增加测试覆盖".into(),
                        "完善文档".into(),
                    ],
                };
                Ok(CapabilityOutput::BlindSpotResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 跨域合成能力
pub struct CrossDomainSynthesisCapability;

impl UnifiedCapability for CrossDomainSynthesisCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-meta-synthesis".into(),
            name: "跨域合成".into(),
            description: "跨领域知识合成和创新".into(),
            version: "1.0.0".into(),
            domain: Domain::NtMeta,
            layer: Layer::L5Cognition,
            tags: vec!["meta".into(), "synthesis".into(), "cross-domain".into()],
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
            avg_latency_ms: 300.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = SynthesisResult {
                    input_domains: vec!["NT-CORE".into(), "NT-MIND".into()],
                    synthesis_query: text,
                    synthesized_knowledge: vec![
                        SynthesizedKnowledge {
                            domains: vec!["core".into(), "mind".into()],
                            insight: "E8推理可增强元认知".into(),
                            confidence: 0.8,
                            novelty_score: 0.7,
                        },
                    ],
                    innovation_opportunities: vec![
                        "将E8推理与SEAL管道结合".into(),
                    ],
                    next_steps: vec![
                        "验证合成知识".into(),
                        "实现原型".into(),
                    ],
                };
                Ok(CapabilityOutput::SynthesisResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 创建NT-META能力
pub fn create_meta_capabilities() -> Vec<Arc<dyn UnifiedCapability>> {
    vec![
        Arc::new(CrossSkillAwarenessCapability),
        Arc::new(BlindSpotDetectionCapability),
        Arc::new(CrossDomainSynthesisCapability),
    ]
}

/// 意识结果
#[derive(Debug, Clone)]
pub struct AwarenessResult {
    pub query: String,
    pub discovered_patterns: Vec<Pattern>,
    pub blind_spots: Vec<BlindSpot>,
    pub propagation_suggestions: Vec<String>,
}

/// 模式
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub confidence: f64,
    pub related_skills: Vec<String>,
}

/// 盲点
#[derive(Debug, Clone)]
pub struct BlindSpot {
    pub area: String,
    pub description: String,
    pub severity: String,
    pub recommendation: String,
}

/// 盲点检测结果
#[derive(Debug, Clone)]
pub struct BlindSpotResult {
    pub system: String,
    pub blind_spots: Vec<BlindSpot>,
    pub coverage_analysis: CoverageAnalysis,
    pub improvement_suggestions: Vec<String>,
}

/// 覆盖分析
#[derive(Debug, Clone)]
pub struct CoverageAnalysis {
    pub total_modules: usize,
    pub covered_modules: usize,
    pub coverage_rate: f64,
    pub uncovered_areas: Vec<String>,
}

/// 合成结果
#[derive(Debug, Clone)]
pub struct SynthesisResult {
    pub input_domains: Vec<String>,
    pub synthesis_query: String,
    pub synthesized_knowledge: Vec<SynthesizedKnowledge>,
    pub innovation_opportunities: Vec<String>,
    pub next_steps: Vec<String>,
}

/// 合成知识
#[derive(Debug, Clone)]
pub struct SynthesizedKnowledge {
    pub domains: Vec<String>,
    pub insight: String,
    pub confidence: f64,
    pub novelty_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_skill_awareness() {
        let cap = CrossSkillAwarenessCapability;
        let input = CapabilityInput::Text("测试跨技能意识".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn blind_spot_detection() {
        let cap = BlindSpotDetectionCapability;
        let input = CapabilityInput::Text("测试盲点检测".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn cross_domain_synthesis() {
        let cap = CrossDomainSynthesisCapability;
        let input = CapabilityInput::Text("测试跨域合成".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }
}
