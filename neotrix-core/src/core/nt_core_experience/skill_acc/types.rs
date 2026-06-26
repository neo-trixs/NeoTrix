use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_self::attention_head::AttentionDomain;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct InternalizedSkill {
    pub id: u64,
    pub name: String,
    pub core_vector: Vec<u8>,
    pub source_skill_ids: Vec<u64>,
    pub abstraction_level: u32,
    pub confidence: f64,
    pub created_at_cycle: u64,
}

#[derive(Debug, Clone)]
pub struct VSASkill {
    pub id: u64,
    pub name: String,
    pub trigger: Vec<u8>,
    pub action: Vec<u8>,
    pub outcome: Vec<u8>,
    pub domain: AttentionDomain,
    pub success_rate: f64,
    pub use_count: u64,
    pub utility: f64,
    pub created_at: u64,
    pub last_used_at: u64,
    pub source_heuristic_ids: Vec<u64>,
    pub version: u32,
    pub refinement_count: u32,
}

#[derive(Debug, Clone)]
pub struct UCBConfig {
    pub exploration_constant: f64,
    pub use_ucb: bool,
}

impl Default for UCBConfig {
    fn default() -> Self {
        Self {
            exploration_constant: 0.5,
            use_ucb: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VsaIndexEntry {
    pub skill_id: u64,
    pub centroid_id: usize,
    pub vector: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SkillComposition {
    pub id: u64,
    pub name: String,
    pub skill_ids: Vec<u64>,
    pub trigger: Vec<u8>,
    pub effectiveness: f64,
    pub use_count: u64,
}

#[derive(Debug, Clone)]
pub struct SkillTrace {
    pub context_hash: u64,
    pub success: bool,
    pub duration_ms: u64,
    pub error_category: Option<String>,
    pub cycle: u64,
}

#[derive(Debug, Clone)]
pub struct SkillMemory {
    pub skill_id: u64,
    pub traces: VecDeque<SkillTrace>,
    pub aggregated_confidence: f64,
    pub derived_patterns: Vec<String>,
    max_traces: usize,
}

impl SkillMemory {
    pub fn new(skill_id: u64, max_traces: usize) -> Self {
        Self {
            skill_id,
            traces: VecDeque::with_capacity(max_traces),
            aggregated_confidence: 0.5,
            derived_patterns: Vec::new(),
            max_traces,
        }
    }

    pub fn record_trace(&mut self, trace: SkillTrace) {
        if self.traces.len() >= self.max_traces {
            self.traces.pop_front();
        }
        self.traces.push_back(trace);
        let recent: Vec<&SkillTrace> = self.traces.iter().rev().take(10).collect();
        let success_count = recent.iter().filter(|t| t.success).count();
        self.aggregated_confidence = success_count as f64 / recent.len().max(1) as f64;
        let failures: Vec<&&SkillTrace> = recent.iter().filter(|t| !t.success).collect();
        if failures.len() >= 3 {
            let cats: Vec<&str> = failures
                .iter()
                .filter_map(|t| t.error_category.as_deref())
                .collect();
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for c in &cats {
                *counts.entry(c).or_insert(0) += 1;
            }
            if let Some((cat, _)) = counts.iter().max_by_key(|e| e.1) {
                let pattern = format!("frequent_failure:{}", cat);
                if !self.derived_patterns.contains(&pattern) {
                    self.derived_patterns.push(pattern);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillEvaluator {
    pub eval_count: u64,
    pub pass_count: u64,
    pass_threshold: f64,
}

impl SkillEvaluator {
    pub fn new(pass_threshold: f64) -> Self {
        Self {
            eval_count: 0,
            pass_count: 0,
            pass_threshold,
        }
    }

    pub fn evaluate(&mut self, skill: &VSASkill, memory: &SkillMemory) -> bool {
        self.eval_count += 1;
        let recall_match = QuantizedVSA::similarity(&skill.trigger, &skill.outcome);
        let trace_confidence = memory.aggregated_confidence;
        let score = 0.3 * recall_match + 0.7 * trace_confidence;
        if score >= self.pass_threshold {
            self.pass_count += 1;
            true
        } else {
            false
        }
    }

    pub fn pass_rate(&self) -> f64 {
        if self.eval_count == 0 {
            return 0.0;
        }
        self.pass_count as f64 / self.eval_count as f64
    }
}

#[derive(Debug, Clone)]
pub struct SkillRefinement {
    pub total_refinements: u64,
    max_refinements: u32,
    perturbation_strength: f64,
}

impl SkillRefinement {
    pub fn new(max_refinements: u32, perturbation_strength: f64) -> Self {
        Self {
            total_refinements: 0,
            max_refinements,
            perturbation_strength,
        }
    }

    pub fn refine(&mut self, skill: &mut VSASkill, _memory: &SkillMemory) -> bool {
        if skill.refinement_count >= self.max_refinements {
            return false;
        }
        skill.refinement_count += 1;
        self.total_refinements += 1;
        skill.trigger = self.perturb(&skill.trigger);
        skill.action = self.perturb(&skill.action);
        skill.version += 1;
        true
    }

    fn perturb(&self, vec: &[u8]) -> Vec<u8> {
        let flip_bits = (vec.len() as f64 * self.perturbation_strength).round() as usize;
        let mut result = vec.to_vec();
        let step = vec.len() / flip_bits.max(1);
        for i in 0..flip_bits.min(vec.len()) {
            let idx = (i * step) % vec.len();
            result[idx] ^= 0x01;
        }
        result
    }
}

pub struct SkillFilter {
    pub min_utility: f64,
    pub min_success_rate: f64,
    pub domains: Option<Vec<AttentionDomain>>,
}

impl Default for SkillFilter {
    fn default() -> Self {
        Self {
            min_utility: 0.0,
            min_success_rate: 0.0,
            domains: None,
        }
    }
}
