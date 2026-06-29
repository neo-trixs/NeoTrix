//! ExperienceDistiller 扩展 — 支持 ReasoningTrace 蒸馏

use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_mind::distillation::{AntiPattern, ExperienceDistiller, StrategicPrinciple};
use crate::neotrix::nt_mind::reasoning_types::{ReasoningTrace, ReasoningType};

impl ExperienceDistiller {
    pub fn distill_traces(traces: &[ReasoningTrace]) -> Vec<StrategicPrinciple> {
        let mut grouped: std::collections::HashMap<ReasoningType, Vec<&ReasoningTrace>> =
            std::collections::HashMap::new();
        for t in traces {
            if t.success {
                grouped.entry(t.reasoning_type).or_default().push(t);
            }
        }

        let mut principles = Vec::new();
        for (rtype, group) in grouped {
            if group.len() < 2 {
                continue;
            }
            let avg_score: f64 =
                group.iter().map(|t| t.outcome_score).sum::<f64>() / group.len() as f64;
            let tt = match rtype {
                ReasoningType::TaskSolving => TaskType::Planning,
                ReasoningType::ErrorDebugging => TaskType::CodeReview,
                ReasoningType::KnowledgeQuery => TaskType::Research,
                _ => TaskType::General,
            };
            principles.push(StrategicPrinciple {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!(
                    "Success pattern for {:?}: {} successful traces, avg score {:.2}",
                    rtype,
                    group.len(),
                    avg_score
                ),
                task_type: tt,
                adjustment_pattern: std::collections::HashMap::new(),
                avg_reward: avg_score,
                application_count: group.len() as u32,
            });
        }
        principles
    }

    pub fn contrastive_reflect_traces(traces: &[ReasoningTrace]) -> Vec<AntiPattern> {
        let successes: Vec<&ReasoningTrace> = traces.iter().filter(|t| t.success).collect();
        let failures: Vec<&ReasoningTrace> = traces.iter().filter(|t| !t.success).collect();

        let mut anti = Vec::new();
        for f in &failures {
            let similar_success = successes
                .iter()
                .filter(|s| s.reasoning_type == f.reasoning_type)
                .max_by(|a, b| {
                    a.outcome_score
                        .partial_cmp(&b.outcome_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            if let Some(_s) = similar_success {
                let tt = match f.reasoning_type {
                    ReasoningType::TaskSolving => TaskType::Planning,
                    ReasoningType::ErrorDebugging => TaskType::CodeReview,
                    ReasoningType::KnowledgeQuery => TaskType::Research,
                    _ => TaskType::General,
                };
                anti.push(AntiPattern {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: format!(
                        "Failure pattern for {:?}: check approach",
                        f.reasoning_type
                    ),
                    task_type: tt,
                    harmful_pattern: std::collections::HashMap::new(),
                    failure_count: 1,
                });
            }
        }
        anti
    }
}
