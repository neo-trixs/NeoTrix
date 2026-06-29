#![forbid(unsafe_code)]

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_experience::memory_consolidation::MemoryConsolidationPipeline;
use crate::core::nt_core_knowledge::entity_extractor::ExtractedFact;

/// Structured repository of abstract, reusable strategic principles (EvolveR, ICLR 2026).
///
/// Each principle distills a pattern from concrete facts into an actionable strategy.
/// Principles self-improve via success_rate tracking and periodic pruning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistilledPrinciple {
    pub id: u64,
    pub principle: String,
    pub source_facts: Vec<u64>,
    pub success_rate: f64,
    pub abstraction_level: u32,
    pub created_at: u64,
    pub invocation_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipleRepository {
    pub principles: Vec<DistilledPrinciple>,
    pub next_id: u64,
}

impl PrincipleRepository {
    pub fn new() -> Self {
        Self {
            principles: Vec::new(),
            next_id: 1,
        }
    }

    /// Distill strategic principles from extracted facts.
    /// Patterns: co-occurring subjects, repeated relation types, high-confidence clusters.
    pub fn distill_from_facts(&mut self, facts: &[ExtractedFact]) -> Vec<DistilledPrinciple> {
        if facts.is_empty() {
            return Vec::new();
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut distilled = Vec::new();

        // Group facts by subject to find repeated patterns
        let mut subject_groups: std::collections::HashMap<&str, Vec<&ExtractedFact>> =
            std::collections::HashMap::new();
        for f in facts {
            subject_groups
                .entry(f.triple.subject.as_str())
                .or_default()
                .push(f);
        }

        for (subject, group) in &subject_groups {
            if group.len() < 2 {
                continue;
            }

            let avg_conf: f64 =
                group.iter().map(|f| f.triple.confidence).sum::<f64>() / group.len() as f64;
            if avg_conf < 0.3 {
                continue;
            }

            // Collect relation types for this subject
            let rel_types: Vec<&str> = group.iter().map(|f| f.triple.relation.name()).collect();
            let unique_rels: std::collections::HashSet<&str> = rel_types.iter().copied().collect();

            let principle = if unique_rels.len() >= 2 {
                format!(
                    "When analyzing '{}', consider multiple relation types: {}",
                    subject,
                    unique_rels
                        .iter()
                        .map(|r| format!("'{}'", r))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else if let Some(rel) = unique_rels.iter().next() {
                format!(
                    "Subject '{}' is consistently associated with '{}' relations",
                    subject, rel
                )
            } else {
                continue;
            };

            let abstraction_level: u32 = if unique_rels.len() >= 3 {
                4
            } else if group.len() >= 5 {
                3
            } else {
                2
            };

            let principle_id = self.next_id;
            self.next_id += 1;

            let dp = DistilledPrinciple {
                id: principle_id,
                principle,
                source_facts: group.iter().map(|f| f.id).collect(),
                success_rate: avg_conf,
                abstraction_level,
                created_at: now,
                invocation_count: 0,
            };

            self.principles.push(dp.clone());
            distilled.push(dp);
        }

        distilled
    }

    /// Distill principles from consolidated memory stats.
    /// Uses memory state heuristics to infer strategic directions.
    pub fn distill_from_memory(
        &mut self,
        memory: &MemoryConsolidationPipeline,
    ) -> Vec<DistilledPrinciple> {
        let stats = memory.stats();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut distilled = Vec::new();

        // If working memory is under-utilized, principle: increase exploration
        let working_ratio = if stats.working_capacity > 0 {
            stats.working_count as f64 / stats.working_capacity as f64
        } else {
            0.0
        };

        if working_ratio < 0.2 && stats.cycle > 10 {
            let principle_id = self.next_id;
            self.next_id += 1;
            let dp = DistilledPrinciple {
                id: principle_id,
                principle: "Low working memory utilization indicates under-exploration — increase curiosity-driven observation".to_string(),
                source_facts: Vec::new(),
                success_rate: 0.5,
                abstraction_level: 3,
                created_at: now,
                invocation_count: 0,
            };
            self.principles.push(dp.clone());
            distilled.push(dp);
        }

        // If procedural memory is sparse relative to semantic, principle: promote patterns
        if stats.procedural_count < 3 && stats.semantic_count > 10 {
            let principle_id = self.next_id;
            self.next_id += 1;
            let dp = DistilledPrinciple {
                id: principle_id,
                principle: "Low procedural-to-semantic ratio suggests consolidating repeated patterns into procedures".to_string(),
                source_facts: Vec::new(),
                success_rate: 0.6,
                abstraction_level: 4,
                created_at: now,
                invocation_count: 0,
            };
            self.principles.push(dp.clone());
            distilled.push(dp);
        }

        distilled
    }

    /// Remove principles with success_rate below threshold, unless highly abstract (level >= 4).
    pub fn prune_principles(&mut self, min_success_rate: f64) {
        self.principles
            .retain(|p| p.success_rate >= min_success_rate || p.abstraction_level >= 4);
    }

    /// Find principles whose text contains any of the given keywords.
    pub fn find_matching_principles(&self, context: &str) -> Vec<&DistilledPrinciple> {
        let lower = context.to_lowercase();
        self.principles
            .iter()
            .filter(|p| {
                let principle_lower = p.principle.to_lowercase();
                // Split context into words; match if any word appears in the principle
                lower
                    .split_whitespace()
                    .any(|word| principle_lower.contains(word))
            })
            .collect()
    }
}

impl Default for PrincipleRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent0-inspired proposer-executor pair for co-evolutionary curriculum.
///
/// The proposer generates increasingly complex challenges;
/// the executor learns to solve them. Success rate drives
/// the pace of curriculum advancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoEvolutionPair {
    pub proposer_id: u64,
    pub executor_id: u64,
    pub success_rate: f64,
    pub completed_tasks: u64,
    pub last_proposal_complexity: f64,
    pub total_attempts: u64,
}

impl CoEvolutionPair {
    pub fn new(proposer_id: u64, executor_id: u64) -> Self {
        Self {
            proposer_id,
            executor_id,
            success_rate: 0.5,
            completed_tasks: 0,
            last_proposal_complexity: 0.3,
            total_attempts: 1,
        }
    }

    /// Propose the next challenge at an appropriate difficulty level.
    /// Complexity increases when success_rate is high, decreases when low.
    pub fn propose_next_challenge(&self, current_success_rate: f64) -> String {
        let complexity = if current_success_rate > 0.8 {
            (self.last_proposal_complexity + 0.15).min(1.0)
        } else if current_success_rate < 0.3 {
            (self.last_proposal_complexity - 0.1).max(0.1)
        } else {
            self.last_proposal_complexity
        };

        let difficulty_label = if complexity < 0.3 {
            "basic"
        } else if complexity < 0.6 {
            "intermediate"
        } else if complexity < 0.8 {
            "advanced"
        } else {
            "expert"
        };

        format!(
            "coevolve:proposer={}:executor={}:complexity={:.2}:level={}:tasks={}",
            self.proposer_id, self.executor_id, complexity, difficulty_label, self.completed_tasks
        )
    }

    /// Record task outcome and update pair statistics.
    pub fn record_task_outcome(&mut self, success: bool, complexity: f64) {
        self.total_attempts += 1;
        self.last_proposal_complexity = complexity;

        if success {
            self.completed_tasks += 1;
        }

        let window = self.total_attempts.min(20) as f64;
        let alpha = 2.0 / (window + 1.0);
        self.success_rate = if success {
            self.success_rate + alpha * (1.0 - self.success_rate)
        } else {
            self.success_rate - alpha * self.success_rate
        };
    }
}

/// Hermes-style post-turn review capturing what happened in a cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnReview {
    pub cycle: u64,
    pub tool_calls: usize,
    pub user_corrections: usize,
    pub errors_encountered: usize,
    pub discovered_pattern: Option<String>,
}

/// Bridges entity extraction + memory consolidation into the self-evolution loop.
///
/// Pipeline:
///   extract(text) → observe(facts) → memory.tick() → collect_evolution_feed() → record_result()
///
/// Upgrades (2026-06-14):
///   - EvolveR-style Offline Self-Distillation (ICLR 2026)
///   - Agent0-style Co-Evolutionary Curriculum
///   - Hermes-style Post-Turn Review
///   - Policy reinforcement loop closure
pub struct EvolutionBridge {
    pub cycle: u64,
    pub last_evolution_cycle: u64,
    pub evolution_interval: u64,
    pub min_facts_for_evolution: usize,

    // ── EvolveR: distilled strategic principles ──
    pub principles: PrincipleRepository,

    // ── Agent0: co-evolutionary pair ──
    pub co_evolution: Option<CoEvolutionPair>,

    // ── Hermes: post-turn review log ──
    pub turn_reviews: VecDeque<TurnReview>,
    pub max_reviews: usize,
}
