use crate::core::nt_core_experience::skill_acc::types::{
    InternalizedSkill, SkillComposition, SkillEvaluator, SkillMemory, SkillRefinement, SkillTrace,
    UCBConfig, VSASkill, VsaIndexEntry,
};
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use crate::core::nt_core_self::attention_head::AttentionDomain;
use std::collections::HashMap;

pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

pub struct SkillAccumulator {
    skills: Vec<VSASkill>,
    memories: HashMap<u64, SkillMemory>,
    compositions: Vec<SkillComposition>,
    evaluator: SkillEvaluator,
    refiner: SkillRefinement,
    next_id: u64,
    max_skills: usize,
    cycle: u64,
    internalized: Vec<InternalizedSkill>,
    sigmoid_gate_beta: f64,
    ucb_config: UCBConfig,
    visit_counts: HashMap<u64, u64>,
    vsa_index: Vec<VsaIndexEntry>,
    index_built_at_cycle: u64,
}

impl SkillAccumulator {
    pub fn new(max_skills: usize) -> Self {
        Self {
            skills: Vec::with_capacity(max_skills),
            memories: HashMap::new(),
            compositions: Vec::new(),
            evaluator: SkillEvaluator::new(0.5),
            refiner: SkillRefinement::new(5, 0.02),
            next_id: 1,
            max_skills,
            cycle: 0,
            internalized: Vec::new(),
            sigmoid_gate_beta: 8.0,
            ucb_config: UCBConfig::default(),
            visit_counts: HashMap::new(),
            vsa_index: Vec::new(),
            index_built_at_cycle: 0,
        }
    }

    pub fn with_evaluator(mut self, threshold: f64) -> Self {
        self.evaluator = SkillEvaluator::new(threshold);
        self
    }

    pub fn with_refiner(mut self, max_refinements: u32, strength: f64) -> Self {
        self.refiner = SkillRefinement::new(max_refinements, strength);
        self
    }

    pub fn with_sigmoid_beta(mut self, beta: f64) -> Self {
        self.sigmoid_gate_beta = beta;
        self
    }

    pub fn with_ucb(mut self, config: UCBConfig) -> Self {
        self.ucb_config = config;
        self
    }

    pub fn accumulate(
        &mut self,
        name: &str,
        trigger_str: &str,
        action_str: &str,
        outcome_str: &str,
        domain: AttentionDomain,
        success: bool,
        source_heuristic_ids: Vec<u64>,
    ) -> Option<u64> {
        self.cycle += 1;
        let trigger = QuantizedVSA::seeded_random(self.stable_hash(trigger_str), 4096);
        let action = QuantizedVSA::seeded_random(self.stable_hash(action_str), 4096);
        let outcome = QuantizedVSA::seeded_random(self.stable_hash(outcome_str), 4096);

        if let Some(existing) = self.find_matching_skill(&trigger, 0.75).first() {
            let idx = existing.1;
            self.skills[idx].use_count += 1;
            self.skills[idx].success_rate = (self.skills[idx].success_rate
                * (self.skills[idx].use_count as f64 - 1.0)
                + if success { 1.0 } else { 0.0 })
                / self.skills[idx].use_count as f64;
            self.skills[idx].last_used_at = self.cycle;
            self.skills[idx].utility = self.compute_utility(idx);
            self.record_skill_trace(self.skills[idx].id, success, None, self.cycle);
            return Some(self.skills[idx].id);
        }

        if self.skills.len() >= self.max_skills {
            self.prune(0.15);
        }
        if self.skills.len() >= self.max_skills {
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;
        let skill = VSASkill {
            id,
            name: name.to_string(),
            trigger,
            action,
            outcome,
            domain,
            success_rate: if success { 0.7 } else { 0.3 },
            use_count: 1,
            utility: 0.5,
            created_at: self.cycle,
            last_used_at: self.cycle,
            source_heuristic_ids,
            version: 1,
            refinement_count: 0,
        };
        self.memories.insert(id, SkillMemory::new(id, 100));
        self.skills.push(skill);
        self.record_skill_trace(id, success, None, self.cycle);
        Some(id)
    }

    pub fn create_skill_on_demand(
        &mut self,
        name: &str,
        description: &str,
        domain: AttentionDomain,
    ) -> Option<u64> {
        let trigger_str = format!("demand:{}", name);
        let action_str = format!("action:{}", description);
        let outcome_str = format!("outcome:{}", name);
        self.accumulate(
            name,
            &trigger_str,
            &action_str,
            &outcome_str,
            domain,
            true,
            vec![],
        )
    }

    fn record_skill_trace(
        &mut self,
        skill_id: u64,
        success: bool,
        error_category: Option<String>,
        cycle: u64,
    ) {
        let context_hash = self.stable_hash(&format!("ctx_{}", cycle));
        if let Some(mem) = self.memories.get_mut(&skill_id) {
            mem.record_trace(SkillTrace {
                context_hash,
                success,
                duration_ms: 0,
                error_category,
                cycle,
            });
        }
    }

    pub fn evaluate_skills(&mut self) -> Vec<(u64, bool)> {
        let mut results = Vec::new();
        let skill_ids: Vec<u64> = self.skills.iter().map(|s| s.id).collect();
        for &sid in &skill_ids {
            let skill = match self.skills.iter().find(|s| s.id == sid) {
                Some(s) => s,
                None => {
                    log::error!(
                        "[skill_acc] skill_id {} not found in skills (abnormal)",
                        sid
                    );
                    continue;
                }
            };
            let mem = self
                .memories
                .get(&sid)
                .cloned()
                .unwrap_or_else(|| SkillMemory::new(sid, 100));
            let passed = self.evaluator.evaluate(skill, &mem);
            results.push((sid, passed));
            if !passed {
                if let Some(skill_mut) = self.skills.iter_mut().find(|s| s.id == sid) {
                    self.refiner.refine(skill_mut, &mem);
                }
            }
        }
        results
    }

    pub fn find_matching(&mut self, trigger_vec: &[u8], top_k: usize) -> Vec<&VSASkill> {
        self.find_matching_skill(trigger_vec, 0.0)
            .into_iter()
            .take(top_k)
            .map(|(_, i)| &self.skills[i])
            .collect()
    }

    fn find_matching_skill(&mut self, trigger_vec: &[u8], min_sim: f64) -> Vec<(f64, usize)> {
        if self.ucb_config.use_ucb {
            let use_index = !self.vsa_index.is_empty()
                && self.cycle - self.index_built_at_cycle <= 100
                && self.skills.len() > 10;
            let results = if use_index {
                let n_centroids = self
                    .vsa_index
                    .iter()
                    .map(|e| e.centroid_id)
                    .max()
                    .unwrap_or(0)
                    + 1;
                let n_probe = if n_centroids > 3 { 3 } else { n_centroids };
                self.search_index(trigger_vec, n_probe, self.skills.len())
            } else {
                if self.skills.len() > 10
                    && (self.vsa_index.is_empty() || self.cycle - self.index_built_at_cycle > 100)
                {
                    let n_centroids = (self.skills.len() / 10).max(3).min(20);
                    self.build_index(n_centroids);
                }
                let mut scored: Vec<(f64, usize)> = self
                    .skills
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let sim = QuantizedVSA::similarity(trigger_vec, &s.trigger);
                        let score = self.ucb_score(s, sim, min_sim);
                        (score, i)
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                scored
            };
            for &(_, idx) in &results {
                *self.visit_counts.entry(self.skills[idx].id).or_insert(0) += 1;
            }
            return results;
        }
        let mut scored: Vec<(f64, usize)> = self
            .skills
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let sim = QuantizedVSA::similarity(trigger_vec, &s.trigger);
                let gate = sigmoid(self.sigmoid_gate_beta * (sim - min_sim));
                let recency =
                    1.0 + 0.1 * (-(self.cycle as f64 - s.last_used_at as f64) / 50.0).exp();
                let score = gate * s.utility * recency;
                (score, i)
            })
            .filter(|(s, _)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    fn ucb_score(&self, skill: &VSASkill, sim: f64, min_sim: f64) -> f64 {
        let gate = sigmoid(self.sigmoid_gate_beta * (sim - min_sim));
        let recency = 1.0 + 0.1 * (-(self.cycle as f64 - skill.last_used_at as f64) / 50.0).exp();
        let exploitation = gate * skill.utility * recency;
        let total_visits: u64 = self.visit_counts.values().sum();
        let n_visits = self.visit_counts.get(&skill.id).copied().unwrap_or(0);
        let total = if total_visits == 0 { 1 } else { total_visits };
        let ln_total = (total as f64).ln();
        let exploration = if n_visits == 0 {
            self.ucb_config.exploration_constant * (ln_total + 1.0).sqrt()
        } else {
            self.ucb_config.exploration_constant * (ln_total / n_visits as f64).sqrt()
        };
        exploitation + exploration
    }

    pub fn build_index(&mut self, n_centroids: usize) {
        if self.skills.is_empty() || n_centroids == 0 {
            return;
        }
        let n = n_centroids.min(self.skills.len());
        let mut centroids: Vec<usize> = Vec::with_capacity(n);
        let first = self
            .skills
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.utility
                    .partial_cmp(&b.utility)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0);
        centroids.push(first);
        while centroids.len() < n {
            let dists: Vec<f64> = self
                .skills
                .iter()
                .enumerate()
                .map(|(_i, s)| {
                    let min_dist = centroids
                        .iter()
                        .map(|&c| {
                            1.0 - QuantizedVSA::similarity(&s.trigger, &self.skills[c].trigger)
                        })
                        .fold(f64::MAX, f64::min);
                    min_dist * min_dist
                })
                .collect();
            let sum: f64 = dists.iter().sum();
            if sum <= 0.0 {
                break;
            }
            let r = rand::random::<f64>() * sum;
            let mut cumulative = 0.0;
            let mut picked = centroids.len();
            for (i, d) in dists.iter().enumerate() {
                if centroids.contains(&i) {
                    continue;
                }
                cumulative += d;
                if cumulative >= r || i == self.skills.len() - 1 {
                    picked = i;
                    break;
                }
            }
            if picked < self.skills.len() && !centroids.contains(&picked) {
                centroids.push(picked);
            } else {
                break;
            }
        }
        self.vsa_index.clear();
        for skill in &self.skills {
            let centroid_id = centroids
                .iter()
                .enumerate()
                .max_by(|(_, &c1), (_, &c2)| {
                    let sim1 = QuantizedVSA::similarity(&skill.trigger, &self.skills[c1].trigger);
                    let sim2 = QuantizedVSA::similarity(&skill.trigger, &self.skills[c2].trigger);
                    sim1.partial_cmp(&sim2).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(id, _)| id)
                .unwrap_or(0);
            self.vsa_index.push(VsaIndexEntry {
                skill_id: skill.id,
                centroid_id,
                vector: skill.trigger.clone(),
            });
        }
        self.index_built_at_cycle = self.cycle;
    }

    pub fn search_index(&self, query: &[u8], n_probe: usize, top_k: usize) -> Vec<(f64, usize)> {
        if self.vsa_index.is_empty() || self.skills.is_empty() {
            return Vec::new();
        }
        let n_centroids = self
            .vsa_index
            .iter()
            .map(|e| e.centroid_id)
            .max()
            .unwrap_or(0)
            + 1;
        let probe = n_probe.min(n_centroids);
        let mut centroid_sims: Vec<(f64, usize)> = (0..n_centroids)
            .map(|cid| {
                let rep = self
                    .vsa_index
                    .iter()
                    .find(|e| e.centroid_id == cid)
                    .map(|e| e.vector.as_slice())
                    .unwrap_or_else(|| {
                        log::warn!("[skill_acc] centroid {} not found, using zero", cid);
                        &[]
                    })
                    .to_vec();
                let rep = if rep.is_empty() {
                    vec![0u8; VSA_DIM]
                } else {
                    rep
                };
                let rep = &rep;
                let sim = QuantizedVSA::similarity(query, rep);
                (sim, cid)
            })
            .collect();
        centroid_sims.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let top_centroids: Vec<usize> = centroid_sims
            .into_iter()
            .take(probe)
            .map(|(_, c)| c)
            .collect();
        let mut candidate_ids: Vec<u64> = self
            .vsa_index
            .iter()
            .filter(|e| top_centroids.contains(&e.centroid_id))
            .map(|e| e.skill_id)
            .collect();
        candidate_ids.sort();
        candidate_ids.dedup();
        let mut results: Vec<(f64, usize)> = candidate_ids
            .iter()
            .filter_map(|&sid| {
                self.skills.iter().position(|s| s.id == sid).map(|idx| {
                    let sim = QuantizedVSA::similarity(query, &self.skills[idx].trigger);
                    let score = self.ucb_score(&self.skills[idx], sim, 0.0);
                    (score, idx)
                })
            })
            .collect();
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    pub fn find_by_domain(&self, domain: &AttentionDomain) -> Vec<&VSASkill> {
        let mut result: Vec<&VSASkill> =
            self.skills.iter().filter(|s| s.domain == *domain).collect();
        result.sort_by(|a, b| {
            b.utility
                .partial_cmp(&a.utility)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }

    fn compute_utility(&self, idx: usize) -> f64 {
        let s = &self.skills[idx];
        let breadth = self
            .skills
            .iter()
            .filter(|other| {
                other.id != s.id && QuantizedVSA::similarity(&s.trigger, &other.trigger) > 0.6
            })
            .count() as f64
            / self.skills.len().max(1) as f64;
        let mem_bonus = self
            .memories
            .get(&s.id)
            .map(|m| m.aggregated_confidence * 0.2)
            .unwrap_or(0.0);
        s.success_rate * (1.0 + breadth) * (s.use_count as f64).sqrt()
            / (1.0 + self.skills.len() as f64).sqrt()
            + mem_bonus
    }

    pub fn recompute_utilities(&mut self) {
        for i in 0..self.skills.len() {
            self.skills[i].utility = self.compute_utility(i);
        }
    }

    pub fn prune(&mut self, min_utility: f64) -> usize {
        let before = self.skills.len();
        let to_remove: Vec<u64> = self
            .skills
            .iter()
            .filter(|s| {
                !(s.use_count > 0 && (s.utility >= min_utility || self.cycle - s.created_at < 20))
            })
            .map(|s| s.id)
            .collect();
        for sid in &to_remove {
            self.memories.remove(sid);
        }
        self.skills.retain(|s| !to_remove.contains(&s.id));
        self.recompute_utilities();
        self.skills.sort_by(|a, b| {
            b.utility
                .partial_cmp(&a.utility)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.skills.truncate(self.max_skills);
        before.saturating_sub(self.skills.len())
    }

    pub fn compose(&mut self, name: &str, skill_ids: &[u64]) -> Option<SkillComposition> {
        let available: std::collections::HashSet<u64> = self.skills.iter().map(|s| s.id).collect();
        let all_exist = skill_ids.iter().all(|id| available.contains(id));
        if !all_exist || skill_ids.len() < 2 {
            return None;
        }

        let selected: Vec<&VSASkill> = self
            .skills
            .iter()
            .filter(|s| skill_ids.contains(&s.id))
            .collect();

        let trigger = QuantizedVSA::bundle(
            &selected
                .iter()
                .map(|s| s.trigger.as_slice())
                .collect::<Vec<&[u8]>>(),
        );
        let eff = selected.iter().map(|s| s.success_rate).sum::<f64>() / selected.len() as f64;

        let id = self.next_id;
        self.next_id += 1;
        let comp = SkillComposition {
            id,
            name: name.to_string(),
            skill_ids: skill_ids.to_vec(),
            trigger,
            effectiveness: eff,
            use_count: 0,
        };
        self.compositions.push(comp.clone());
        Some(comp)
    }

    pub fn internalize_from_skill_ids(
        &mut self,
        skill_ids: &[u64],
        abstraction_level: u32,
    ) -> Option<InternalizedSkill> {
        let skill_set: std::collections::HashSet<u64> = self.skills.iter().map(|s| s.id).collect();
        let all_exist = skill_ids.iter().all(|id| skill_set.contains(id));
        if !all_exist || skill_ids.is_empty() {
            return None;
        }

        let selected: Vec<&VSASkill> = self
            .skills
            .iter()
            .filter(|s| skill_ids.contains(&s.id))
            .collect();

        let mut vectors: Vec<&[u8]> = Vec::with_capacity(selected.len() * 3);
        for s in &selected {
            vectors.push(&s.trigger);
            vectors.push(&s.action);
            vectors.push(&s.outcome);
        }
        let core_vector = QuantizedVSA::bundle(&vectors);

        let mean_utility: f64 =
            selected.iter().map(|s| s.utility).sum::<f64>() / selected.len() as f64;
        if mean_utility < 0.15 {
            return None;
        }

        let name = if selected.len() == 1 {
            format!("internalized:{}", selected[0].name)
        } else {
            let names: Vec<&str> = selected.iter().map(|s| s.name.as_str()).collect();
            format!("internalized:{}", names.join("+"))
        };

        let id = self.next_id;
        self.next_id += 1;
        let internalized = InternalizedSkill {
            id,
            name,
            core_vector,
            source_skill_ids: skill_ids.to_vec(),
            abstraction_level,
            confidence: mean_utility,
            created_at_cycle: self.cycle,
        };
        self.internalized.push(internalized.clone());
        Some(internalized)
    }

    pub fn internalize_top_skills(&mut self, top_k: usize) -> Vec<InternalizedSkill> {
        let best: Vec<u64> = self
            .best_skills(self.skills.len())
            .into_iter()
            .take(top_k)
            .filter(|s| {
                !self
                    .internalized
                    .iter()
                    .any(|is| is.source_skill_ids.contains(&s.id))
            })
            .map(|s| s.id)
            .collect();
        if best.is_empty() {
            return Vec::new();
        }
        let abstraction = (best.len().min(3) as u32).max(1);
        self.internalize_from_skill_ids(&best, abstraction)
            .into_iter()
            .collect()
    }

    pub fn internalized_skills(&self) -> &[InternalizedSkill] {
        &self.internalized
    }

    pub fn internalization_count(&self) -> usize {
        self.internalized.len()
    }

    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    pub fn composition_count(&self) -> usize {
        self.compositions.len()
    }

    pub fn best_skills(&self, top_k: usize) -> Vec<&VSASkill> {
        let mut sorted: Vec<&VSASkill> = self.skills.iter().collect();
        sorted.sort_by(|a, b| {
            b.utility
                .partial_cmp(&a.utility)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(top_k);
        sorted
    }

    pub fn skill_memory(&self, skill_id: u64) -> Option<&SkillMemory> {
        self.memories.get(&skill_id)
    }

    pub fn evaluator_stats(&self) -> (u64, u64, f64) {
        (
            self.evaluator.eval_count,
            self.evaluator.pass_count,
            self.evaluator.pass_rate(),
        )
    }

    pub fn total_refinements(&self) -> u64 {
        self.refiner.total_refinements
    }

    fn stable_hash(&self, s: &str) -> u64 {
        let mut h: u64 = 0xa1b2c3d4e5f67890u64;
        for b in s.bytes() {
            h = h.wrapping_mul(0x9e3779b97f4a7c15u64);
            h ^= b as u64;
            h = h.rotate_left(17);
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn make_acc() -> SkillAccumulator {
        SkillAccumulator::new(100)
    }

    #[test]
    fn test_accumulate_creates_skill() {
        let mut acc = make_acc();
        let id = acc.accumulate(
            "test_skill",
            "trigger",
            "action",
            "outcome",
            AttentionDomain::Code,
            true,
            vec![],
        );
        assert!(id.is_some());
        assert_eq!(acc.skill_count(), 1);
    }

    #[test]
    fn test_accumulate_updates_existing() {
        let mut acc = make_acc();
        let id1 = acc
            .accumulate(
                "skill1",
                "same_trigger",
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        let id2 = acc
            .accumulate(
                "skill2",
                "same_trigger",
                "act",
                "out",
                AttentionDomain::Code,
                false,
                vec![],
            )
            .unwrap();
        assert_eq!(id1, id2);
        assert_eq!(acc.skill_count(), 1);
        let s = &acc.skills[0];
        assert_eq!(s.use_count, 2);
        assert!(s.success_rate > 0.0 && s.success_rate < 1.0);
    }

    #[test]
    fn test_accumulate_increments_use_count() {
        let mut acc = make_acc();
        for _ in 0..3 {
            acc.accumulate(
                "s",
                "trig",
                "act",
                "out",
                AttentionDomain::Semantic,
                true,
                vec![],
            );
        }
        assert_eq!(acc.skills[0].use_count, 3);
    }

    #[test]
    fn test_find_by_domain_filters() {
        let mut acc = make_acc();
        acc.accumulate("s1", "t1", "a1", "o1", AttentionDomain::Code, true, vec![]);
        acc.accumulate(
            "s2",
            "t2",
            "a2",
            "o2",
            AttentionDomain::Semantic,
            true,
            vec![],
        );
        assert_eq!(acc.find_by_domain(&AttentionDomain::Code).len(), 1);
        assert_eq!(acc.find_by_domain(&AttentionDomain::Semantic).len(), 1);
    }

    #[test]
    fn test_find_by_domain_orders_by_utility() {
        let mut acc = make_acc();
        acc.accumulate("s1", "t1", "a1", "o1", AttentionDomain::Code, true, vec![]);
        acc.accumulate("s2", "t2", "a2", "o2", AttentionDomain::Code, true, vec![]);
        acc.recompute_utilities();
        let results = acc.find_by_domain(&AttentionDomain::Code);
        if results.len() >= 2 {
            assert!(results[0].utility >= results[1].utility);
        }
    }

    #[test]
    fn test_compose_creates_composition() {
        let mut acc = make_acc();
        let id1 = acc
            .accumulate(
                "s1",
                "trig1",
                "act1",
                "out1",
                AttentionDomain::Planning,
                true,
                vec![],
            )
            .unwrap();
        let id2 = acc
            .accumulate(
                "s2",
                "trig2",
                "act2",
                "out2",
                AttentionDomain::Planning,
                true,
                vec![],
            )
            .unwrap();
        let comp = acc.compose("combo", &[id1, id2]);
        assert!(comp.is_some());
        assert_eq!(acc.composition_count(), 1);
    }

    #[test]
    fn test_compose_fails_with_one_skill() {
        let mut acc = make_acc();
        let id = acc
            .accumulate("s1", "t1", "a1", "o1", AttentionDomain::Code, true, vec![])
            .unwrap();
        assert!(acc.compose("single", &[id]).is_none());
    }

    #[test]
    fn test_compose_fails_with_missing_skill() {
        let mut acc = make_acc();
        assert!(acc.compose("missing", &[42, 99]).is_none());
    }

    #[test]
    fn test_prune_removes_low_utility() {
        let mut acc = make_acc();
        acc.accumulate(
            "keep",
            "t1",
            "a1",
            "o1",
            AttentionDomain::Code,
            true,
            vec![],
        );
        acc.accumulate(
            "prune_me",
            "t2",
            "a2",
            "o2",
            AttentionDomain::Code,
            false,
            vec![],
        );
        acc.cycle = 100;
        acc.skills[1].utility = 0.01;
        let pruned = acc.prune(0.5);
        assert!(pruned > 0 || acc.skill_count() <= 1);
    }

    #[test]
    fn test_best_skills_returns_sorted() {
        let mut acc = make_acc();
        acc.accumulate(
            "high",
            "t1",
            "a1",
            "o1",
            AttentionDomain::Code,
            true,
            vec![],
        );
        acc.accumulate(
            "low",
            "t2",
            "a2",
            "o2",
            AttentionDomain::Code,
            false,
            vec![],
        );
        acc.recompute_utilities();
        let best = acc.best_skills(10);
        assert_eq!(best.len(), 2);
        assert!(best[0].utility >= best[1].utility);
    }

    #[test]
    fn test_skill_memory_tracks_traces() {
        let mut acc = make_acc();
        let id = acc
            .accumulate("s", "t", "a", "o", AttentionDomain::Code, true, vec![])
            .unwrap();
        acc.accumulate("s", "t", "a", "o", AttentionDomain::Code, false, vec![])
            .unwrap();
        let mem = acc.skill_memory(id).unwrap();
        assert_eq!(mem.traces.len(), 2);
        assert!(mem.aggregated_confidence > 0.0);
    }

    #[test]
    fn test_evaluate_skills_no_crash() {
        let mut acc = make_acc();
        acc.accumulate("s1", "t1", "a1", "o1", AttentionDomain::Code, true, vec![]);
        acc.accumulate("s2", "t2", "a2", "o2", AttentionDomain::Code, false, vec![]);
        let results = acc.evaluate_skills();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_skill_memory_aggregates_confidence() {
        let mut acc = make_acc();
        let id = acc
            .accumulate("s", "t", "a", "o", AttentionDomain::Code, true, vec![])
            .unwrap();
        for _ in 0..10 {
            acc.accumulate("s", "t", "a", "o", AttentionDomain::Code, false, vec![]);
            acc.cycle += 1;
        }
        let mem = acc.skill_memory(id).unwrap();
        assert!(
            mem.aggregated_confidence < 0.5,
            "after 10 fails confidence should be low"
        );
    }

    #[test]
    fn test_internalize_from_valid_skills() {
        let mut acc = make_acc();
        let id1 = acc
            .accumulate(
                "s1",
                "trig1",
                "act1",
                "out1",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        let id2 = acc
            .accumulate(
                "s2",
                "trig2",
                "act2",
                "out2",
                AttentionDomain::Semantic,
                true,
                vec![],
            )
            .unwrap();
        let internalized = acc.internalize_from_skill_ids(&[id1, id2], 1);
        assert!(internalized.is_some());
        let is = internalized.unwrap();
        assert_eq!(is.source_skill_ids.len(), 2);
        assert!(!is.core_vector.is_empty());
        assert_eq!(is.abstraction_level, 1);
        assert_eq!(acc.internalization_count(), 1);
    }

    #[test]
    fn test_internalize_from_missing_skill_returns_none() {
        let mut acc = make_acc();
        acc.accumulate("exists", "t", "a", "o", AttentionDomain::Code, true, vec![]);
        let result = acc.internalize_from_skill_ids(&[42, 99], 1);
        assert!(result.is_none());
        assert_eq!(acc.internalization_count(), 0);
    }

    #[test]
    fn test_internalize_top_skills_selects_best() {
        let mut acc = make_acc();
        let id_a = acc
            .accumulate(
                "best",
                "t1",
                "a1",
                "o1",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        acc.accumulate(
            "mid",
            "t2",
            "a2",
            "o2",
            AttentionDomain::Code,
            false,
            vec![],
        );
        acc.accumulate(
            "worst",
            "t3",
            "a3",
            "o3",
            AttentionDomain::Code,
            false,
            vec![],
        );
        acc.recompute_utilities();
        acc.skills.iter_mut().for_each(|s| {
            if s.id == id_a {
                s.utility = 0.9;
            }
        });
        let results = acc.internalize_top_skills(1);
        assert_eq!(results.len(), 1);
        assert!(results[0].source_skill_ids.contains(&id_a));
    }

    #[test]
    fn test_internalization_creates_core_vector() {
        let mut acc = make_acc();
        let id = acc
            .accumulate("s", "t", "a", "o", AttentionDomain::Code, true, vec![])
            .unwrap();
        let internalized = acc.internalize_from_skill_ids(&[id], 1).unwrap();
        assert_eq!(internalized.core_vector.len(), VSA_DIM);
    }

    #[test]
    fn test_sigmoid_returns_continuous() {
        let high = sigmoid(8.0 * (0.9 - 0.5));
        let low = sigmoid(8.0 * (0.1 - 0.5));
        assert!(high > 0.95, "high sim should gate near 1.0, got {}", high);
        assert!(low < 0.05, "low sim should gate near 0.0, got {}", low);
        assert!(high > low);
    }

    #[test]
    fn test_sigmoid_with_beta_zero() {
        let _acc = SkillAccumulator::new(100).with_sigmoid_beta(0.0);
        let g1 = sigmoid(0.0 * (0.9 - 0.5));
        let g2 = sigmoid(0.0 * (0.1 - 0.5));
        assert!((g1 - 0.5).abs() < 1e-6);
        assert!((g2 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_recent_skill_gets_boost() {
        let mut acc = make_acc();
        acc.sigmoid_gate_beta = 8.0;
        let id_old = acc
            .accumulate(
                "old",
                "trigger_x",
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        acc.cycle = 200;
        let id_recent = acc
            .accumulate(
                "recent",
                "trigger_x",
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        assert_eq!(id_old, id_recent, "same trigger should match");
        let trigger = acc.skills[0].trigger.clone();
        let matches = acc.find_matching_skill(&trigger, 0.5);
        assert!(!matches.is_empty(), "should find matches");
        let (score, _) = matches[0];
        assert!(score > 0.0);
    }

    #[test]
    fn test_combined_internalization_and_gating() {
        let mut acc = make_acc();
        let id = acc
            .accumulate(
                "core",
                "trig",
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        acc.recompute_utilities();
        acc.skills.iter_mut().for_each(|s| {
            if s.id == id {
                s.utility = 0.8;
            }
        });
        let internalized = acc.internalize_from_skill_ids(&[id], 1);
        assert!(internalized.is_some());
        let trigger = acc.skills[0].trigger.clone();
        let matches = acc.find_matching_skill(&trigger, 0.5);
        assert!(!matches.is_empty());
        let (score, _) = matches[0];
        assert!(score > 0.0);
    }

    #[test]
    fn test_sigmoid_matching_accuracy() {
        let mut acc = make_acc();
        let id_high = acc
            .accumulate(
                "high_sim",
                "unique_pattern_a",
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        let _id_low = acc
            .accumulate(
                "low_sim",
                "different_pattern_b",
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            )
            .unwrap();
        acc.recompute_utilities();
        let high_trigger = acc
            .skills
            .iter()
            .find(|s| s.id == id_high)
            .unwrap()
            .trigger
            .clone();
        let matches = acc.find_matching_skill(&high_trigger, 0.5);
        assert!(!matches.is_empty());
        let top_idx = matches[0].1;
        assert_eq!(
            acc.skills[top_idx].id, id_high,
            "high-sim skill should rank first"
        );
    }

    fn make_acc_ucb() -> SkillAccumulator {
        SkillAccumulator::new(100).with_ucb(UCBConfig {
            exploration_constant: 0.5,
            use_ucb: true,
        })
    }

    #[test]
    fn test_ucb_exploration_bonus() {
        let mut acc = make_acc_ucb();
        acc.accumulate(
            "a",
            "trig_a",
            "act",
            "out",
            AttentionDomain::Code,
            true,
            vec![],
        );
        acc.accumulate(
            "b",
            "trig_b",
            "act",
            "out",
            AttentionDomain::Code,
            true,
            vec![],
        );
        acc.visit_counts.insert(acc.skills[0].id, 5);
        acc.visit_counts.insert(acc.skills[1].id, 0);
        let sim = QuantizedVSA::similarity(&acc.skills[0].trigger, &acc.skills[1].trigger);
        let score_visited = acc.ucb_score(&acc.skills[0], sim, 0.0);
        let score_unvisited = acc.ucb_score(&acc.skills[1], sim, 0.0);
        assert!(
            score_unvisited > score_visited,
            "unvisited skill ({}) should score higher than visited ({})",
            score_unvisited,
            score_visited
        );
    }

    #[test]
    fn test_ucb_exploitation_dominates_after_many_visits() {
        let mut acc = make_acc_ucb();
        acc.accumulate(
            "a",
            "trig",
            "act",
            "out",
            AttentionDomain::Code,
            true,
            vec![],
        );
        let sid = acc.skills[0].id;
        acc.visit_counts.insert(sid, 100);
        let sim = QuantizedVSA::similarity(&acc.skills[0].trigger, &acc.skills[0].trigger);
        let score = acc.ucb_score(&acc.skills[0], sim, 0.0);
        let total: u64 = acc.visit_counts.values().sum();
        let n = *acc.visit_counts.get(&sid).unwrap_or(&0);
        let ln_total = (if total == 0 { 1 } else { total } as f64).ln();
        let exploration = if n == 0 {
            0.5
        } else {
            0.5 * (ln_total / n as f64).sqrt()
        };
        assert!(
            score > exploration * 5.0,
            "exploitation should dominate exploration: score={}, exploration={}",
            score,
            exploration
        );
    }

    #[test]
    fn test_build_index_creates_centroids() {
        let mut acc = make_acc_ucb();
        for i in 0..15 {
            acc.accumulate(
                &format!("s{}", i),
                &format!("trigger_{}", i),
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            );
        }
        assert_eq!(acc.vsa_index.len(), 0);
        acc.build_index(3);
        assert_eq!(acc.vsa_index.len(), 15);
        let cids: std::collections::HashSet<usize> =
            acc.vsa_index.iter().map(|e| e.centroid_id).collect();
        assert!(
            cids.len() <= 3 && !cids.is_empty(),
            "should have <= 3 centroids, got {}",
            cids.len()
        );
        assert_eq!(acc.index_built_at_cycle, acc.cycle);
    }

    #[test]
    fn test_search_index_returns_results() {
        let mut acc = make_acc_ucb();
        for i in 0..12 {
            acc.accumulate(
                &format!("s{}", i),
                &format!("pat_{}", i),
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            );
        }
        acc.build_index(3);
        let query = acc.skills[0].trigger.clone();
        let results = acc.search_index(&query, 2, 5);
        assert!(!results.is_empty(), "search should return results");
        assert!(results.len() <= 5, "should respect top_k");
        for (score, _) in &results {
            assert!(*score > 0.0, "scores should be positive");
        }
    }

    #[test]
    fn test_ucb_config_defaults() {
        let config = UCBConfig::default();
        assert!(!config.use_ucb);
        assert!((config.exploration_constant - 0.5).abs() < 1e-10);
        let acc = SkillAccumulator::new(100);
        assert!(!acc.ucb_config.use_ucb);
    }

    #[test]
    fn test_visit_count_increments_on_match() {
        let mut acc = make_acc_ucb();
        acc.accumulate(
            "a",
            "trig_a",
            "act",
            "out",
            AttentionDomain::Code,
            true,
            vec![],
        );
        acc.accumulate(
            "b",
            "trig_b",
            "act",
            "out",
            AttentionDomain::Code,
            true,
            vec![],
        );
        assert!(acc.visit_counts.is_empty(), "no visits before match");
        let query = acc.skills[0].trigger.clone();
        let matches = acc.find_matching_skill(&query, 0.0);
        assert!(!matches.is_empty());
        assert!(
            !acc.visit_counts.is_empty(),
            "visit_counts should be non-empty after matching"
        );
    }

    #[test]
    fn test_ucb_finds_high_utility_skills() {
        let mut acc = make_acc_ucb();
        acc.accumulate(
            "low",
            "trig_x",
            "act",
            "out",
            AttentionDomain::Code,
            true,
            vec![],
        );
        acc.accumulate(
            "high",
            "trig_x",
            "act",
            "out",
            AttentionDomain::Code,
            true,
            vec![],
        );
        acc.skills.iter_mut().for_each(|s| {
            if s.name == "high" {
                s.utility = 0.9;
            } else {
                s.utility = 0.1;
            }
        });
        let query = acc.skills[0].trigger.clone();
        let matches = acc.find_matching_skill(&query, 0.0);
        assert!(!matches.is_empty());
        assert_eq!(
            acc.skills[matches[0].1].name, "high",
            "UCB should rank high-utility skill first"
        );
    }

    #[test]
    fn test_index_rebuild_after_mutation() {
        let mut acc = make_acc_ucb();
        for i in 0..12 {
            acc.accumulate(
                &format!("s{}", i),
                &format!("pat_{}", i),
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            );
        }
        acc.build_index(3);
        let first_built = acc.index_built_at_cycle;
        for i in 0..5 {
            acc.accumulate(
                &format!("new{}", i),
                &format!("new_pat_{}", i),
                "act",
                "out",
                AttentionDomain::Code,
                true,
                vec![],
            );
        }
        acc.cycle += 200;
        let query = acc.skills[0].trigger.clone();
        let _ = acc.find_matching_skill(&query, 0.0);
        assert!(
            acc.index_built_at_cycle > first_built,
            "index should have been rebuilt after 200 cycles past build"
        );
    }
}
