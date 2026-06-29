use super::message::AgentId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: u64,
    pub task_context: String,
    pub trajectory: String,
    pub score: f64,
    pub timestamp: Instant,
    pub hit_count: u64,
    pub domain: String,
    pub vsa: Option<crate::core::nt_core_hcube::VsaVector<4096>>,
}

#[derive(Debug, Clone)]
pub struct PoolWeights {
    pub exploitation_weight: f64,
    pub exploration_weight: f64,
    pub last_update: Instant,
    pub total_adjustments: u64,
}

impl PoolWeights {
    pub fn new() -> Self {
        Self {
            exploitation_weight: 0.6,
            exploration_weight: 0.4,
            last_update: Instant::now(),
            total_adjustments: 0,
        }
    }

    pub fn select_pool(&self) -> PoolType {
        let roll = fastrand::f64();
        if roll < self.exploitation_weight / (self.exploitation_weight + self.exploration_weight) {
            PoolType::Exploitation
        } else {
            PoolType::Exploration
        }
    }

    pub fn adjust(&mut self, exploitation_effective: bool, strength: f64) {
        let delta = strength * 0.05;
        if exploitation_effective {
            self.exploitation_weight = (self.exploitation_weight + delta).min(0.95);
            self.exploration_weight = (self.exploration_weight - delta).max(0.05);
        } else {
            self.exploration_weight = (self.exploration_weight + delta).min(0.95);
            self.exploitation_weight = (self.exploitation_weight - delta).max(0.05);
        }
        self.last_update = Instant::now();
        self.total_adjustments += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolType {
    Exploitation,
    Exploration,
}

#[derive(Debug, Clone)]
pub struct DecentralizedMemory {
    pub agent_id: AgentId,
    exploitation_pool: Vec<MemoryEntry>,
    exploration_pool: Vec<MemoryEntry>,
    weights: PoolWeights,
    max_pool_size: usize,
    next_id: u64,
    stats: DecentMemStats,
}

#[derive(Debug, Clone)]
pub struct DecentMemStats {
    pub exploitation_hits: u64,
    pub exploration_hits: u64,
    pub exploitation_misses: u64,
    pub exploration_misses: u64,
    pub total_stores: u64,
    pub total_retrievals: u64,
    pub weight_adjustments: u64,
    pub pool_switches: u64,
    pub avg_exploitation_score: f64,
    pub avg_exploration_score: f64,
}

#[derive(Debug, Clone)]
pub struct StageFeedback {
    pub stage: String,
    pub score: f64,
    pub exploitation_effective: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct RetrieveResult {
    pub entries: Vec<MemoryEntry>,
    pub pool_used: PoolType,
    pub confidence: f64,
}

impl DecentralizedMemory {
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            exploitation_pool: Vec::with_capacity(100),
            exploration_pool: Vec::with_capacity(100),
            weights: PoolWeights::new(),
            max_pool_size: 200,
            next_id: 1,
            stats: DecentMemStats {
                exploitation_hits: 0,
                exploration_hits: 0,
                exploitation_misses: 0,
                exploration_misses: 0,
                total_stores: 0,
                total_retrievals: 0,
                weight_adjustments: 0,
                pool_switches: 0,
                avg_exploitation_score: 0.0,
                avg_exploration_score: 0.0,
            },
        }
    }

    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.max_pool_size = size;
        self.exploitation_pool = Vec::with_capacity(size / 2);
        self.exploration_pool = Vec::with_capacity(size / 2);
        self
    }

    pub fn store(
        &mut self,
        task_context: &str,
        trajectory: &str,
        domain: &str,
        score: f64,
        pool: PoolType,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let entry = MemoryEntry {
            id,
            task_context: task_context.to_string(),
            trajectory: trajectory.to_string(),
            score,
            timestamp: Instant::now(),
            hit_count: 0,
            domain: domain.to_string(),
            vsa: None,
        };
        match pool {
            PoolType::Exploitation => {
                if self.exploitation_pool.len() >= self.max_pool_size / 2 {
                    self.exploitation_pool.sort_by(|a, b| {
                        a.score
                            .partial_cmp(&b.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    self.exploitation_pool.remove(0);
                }
                self.exploitation_pool.push(entry);
                let total: f64 = self.exploitation_pool.iter().map(|e| e.score).sum();
                self.stats.avg_exploitation_score = total / self.exploitation_pool.len() as f64;
            }
            PoolType::Exploration => {
                if self.exploration_pool.len() >= self.max_pool_size / 2 {
                    self.exploration_pool.sort_by(|a, b| {
                        a.score
                            .partial_cmp(&b.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    self.exploration_pool.remove(0);
                }
                self.exploration_pool.push(entry);
                let total: f64 = self.exploration_pool.iter().map(|e| e.score).sum();
                self.stats.avg_exploration_score = total / self.exploration_pool.len() as f64;
            }
        }
        self.stats.total_stores += 1;
        id
    }

    pub fn retrieve(&mut self, task_context: &str, domain: &str, top_k: usize) -> RetrieveResult {
        self.stats.total_retrievals += 1;
        let pool_type = self.weights.select_pool();
        let pool = match pool_type {
            PoolType::Exploitation => &mut self.exploitation_pool,
            PoolType::Exploration => &mut self.exploration_pool,
        };
        let mut scored: Vec<(f64, usize)> = pool
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let ctx_sim = if e.task_context.contains(task_context)
                    || task_context.contains(&e.task_context)
                {
                    0.7
                } else {
                    0.0
                };
                let domain_sim = if e.domain == domain { 0.3 } else { 0.0 };
                let recency = ((Instant::now() - e.timestamp).as_secs_f64().max(1.0)).recip();
                let score = ctx_sim + domain_sim + recency * 0.1 + e.score * 0.3;
                (score, i)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let confidence = if scored.is_empty() {
            0.0
        } else {
            scored.iter().take(top_k).map(|(s, _)| s).sum::<f64>() / top_k as f64
        };
        let indices: Vec<usize> = scored.iter().take(top_k).map(|(_, i)| *i).collect();
        let mut entries = Vec::new();
        for &i in &indices {
            if let Some(e) = pool.get_mut(i) {
                e.hit_count += 1;
                entries.push(e.clone());
            }
        }
        match pool_type {
            PoolType::Exploitation => {
                if entries.is_empty() {
                    self.stats.exploitation_misses += 1;
                } else {
                    self.stats.exploitation_hits += 1;
                }
            }
            PoolType::Exploration => {
                if entries.is_empty() {
                    self.stats.exploration_misses += 1;
                } else {
                    self.stats.exploration_hits += 1;
                }
            }
        }
        RetrieveResult {
            entries,
            pool_used: pool_type,
            confidence: confidence.min(1.0),
        }
    }

    pub fn generate_exploration_candidate(
        &self,
        task_context: &str,
        domain: &str,
    ) -> Option<String> {
        if self.exploration_pool.is_empty() {
            return None;
        }
        let mut rng = fastrand::i32(0..self.exploration_pool.len() as i32) as usize;
        let mut attempts = 0;
        while attempts < 10 {
            let candidate = &self.exploration_pool[rng % self.exploration_pool.len()];
            if candidate.domain == domain && candidate.task_context != task_context {
                return Some(candidate.trajectory.clone());
            }
            rng = (rng + 1) % self.exploration_pool.len();
            attempts += 1;
        }
        self.exploration_pool.first().map(|e| e.trajectory.clone())
    }

    pub fn apply_stage_feedback(&mut self, feedback: &[StageFeedback]) {
        for fb in feedback {
            self.weights
                .adjust(fb.exploitation_effective, fb.confidence);
            self.stats.weight_adjustments += 1;
            if fb.exploitation_effective {
            } else {
            }
        }
    }

    pub fn record_pool_switch(&mut self) {
        self.stats.pool_switches += 1;
    }

    pub fn consume_trajectory(
        &mut self,
        task_context: &str,
        domain: &str,
        trajectory: &str,
        score: f64,
    ) {
        let pool_type = if score > self.stats.avg_exploitation_score.max(0.3) {
            PoolType::Exploitation
        } else {
            PoolType::Exploration
        };
        self.store(task_context, trajectory, domain, score, pool_type);
    }

    pub fn consolidate(&mut self) {
        self.exploitation_pool.retain(|e| {
            let age = Instant::now().duration_since(e.timestamp);
            age < Duration::from_secs(86400 * 7) || e.hit_count > 2
        });
        self.exploration_pool.retain(|e| {
            let age = Instant::now().duration_since(e.timestamp);
            age < Duration::from_secs(86400 * 3) || e.hit_count > 0
        });
    }

    pub fn stats(&self) -> &DecentMemStats {
        &self.stats
    }

    pub fn exploitation_pool_size(&self) -> usize {
        self.exploitation_pool.len()
    }

    pub fn exploration_pool_size(&self) -> usize {
        self.exploration_pool.len()
    }

    pub fn weights(&self) -> &PoolWeights {
        &self.weights
    }

    pub fn prune(&mut self, target_size: usize) {
        let per_pool = target_size / 2;
        if self.exploitation_pool.len() > per_pool {
            self.exploitation_pool.sort_by(|a, b| {
                let a_val = a.score * (a.hit_count as f64).max(0.5);
                let b_val = b.score * (b.hit_count as f64).max(0.5);
                a_val
                    .partial_cmp(&b_val)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.exploitation_pool.truncate(per_pool);
        }
        if self.exploration_pool.len() > per_pool {
            self.exploration_pool.sort_by(|a, b| {
                let a_val = a.score * (a.hit_count as f64).max(0.3);
                let b_val = b.score * (b.hit_count as f64).max(0.3);
                a_val
                    .partial_cmp(&b_val)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.exploration_pool.truncate(per_pool);
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecentralizedMemoryManager {
    agents: HashMap<AgentId, DecentralizedMemory>,
    max_agents: usize,
    global_stats: DecentMemStats,
}

impl DecentralizedMemoryManager {
    pub fn new(max_agents: usize) -> Self {
        Self {
            agents: HashMap::new(),
            max_agents,
            global_stats: DecentMemStats {
                exploitation_hits: 0,
                exploration_hits: 0,
                exploitation_misses: 0,
                exploration_misses: 0,
                total_stores: 0,
                total_retrievals: 0,
                weight_adjustments: 0,
                pool_switches: 0,
                avg_exploitation_score: 0.0,
                avg_exploration_score: 0.0,
            },
        }
    }

    pub fn register_agent(&mut self, agent_id: AgentId) {
        if self.agents.contains_key(&agent_id) {
            return;
        }
        if self.agents.len() >= self.max_agents {
            let oldest = self.agents.keys().next().cloned();
            if let Some(old) = oldest {
                self.agents.remove(&old);
            }
        }
        self.agents
            .insert(agent_id.clone(), DecentralizedMemory::new(agent_id));
    }

    pub fn get_mut(&mut self, agent_id: &AgentId) -> Option<&mut DecentralizedMemory> {
        self.agents.get_mut(agent_id)
    }

    pub fn get(&self, agent_id: &AgentId) -> Option<&DecentralizedMemory> {
        self.agents.get(agent_id)
    }

    pub fn all_stats(&self) -> &DecentMemStats {
        &self.global_stats
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn agent_ids(&self) -> Vec<AgentId> {
        self.agents.keys().cloned().collect()
    }

    pub fn store_collaborative(
        &mut self,
        agents: &[AgentId],
        task_context: &str,
        domain: &str,
        trajectory: &str,
        score: f64,
    ) {
        for agent_id in agents {
            if let Some(mem) = self.agents.get_mut(agent_id) {
                mem.consume_trajectory(task_context, domain, trajectory, score);
                self.global_stats.total_stores += 1;
            }
        }
    }

    pub fn consolidate_all(&mut self) {
        for mem in self.agents.values_mut() {
            mem.consolidate();
        }
    }

    pub fn prune_all(&mut self, target_size: usize) {
        for mem in self.agents.values_mut() {
            mem.prune(target_size);
        }
    }

    pub fn stats_report(&self) -> String {
        let s = &self.global_stats;
        format!(
            "DecentralizedMemory: {} agents | E-pool hits={} X-pool hits={} | \
             stores={} retrievals={} | weight_adj={} pool_switches={} | \
             avg_e_score={:.3} avg_x_score={:.3}",
            self.agents.len(),
            s.exploitation_hits,
            s.exploration_hits,
            s.total_stores,
            s.total_retrievals,
            s.weight_adjustments,
            s.pool_switches,
            s.avg_exploitation_score,
            s.avg_exploration_score,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_agent_id(label: &str) -> AgentId {
        AgentId::new(label, "1.0")
    }

    #[test]
    fn test_store_and_retrieve_exploitation() {
        let mut mem = DecentralizedMemory::new(test_agent_id("alpha"));
        mem.store(
            "train classifier",
            "used adam optimizer",
            "ml",
            0.85,
            PoolType::Exploitation,
        );
        mem.store(
            "train classifier",
            "used sgd momentum",
            "ml",
            0.72,
            PoolType::Exploitation,
        );
        let result = mem.retrieve("train classifier", "ml", 2);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.pool_used, PoolType::Exploitation);
    }

    #[test]
    fn test_store_and_retrieve_exploration() {
        let mut mem = DecentralizedMemory::new(test_agent_id("beta"));
        mem.store(
            "unseen task",
            "novel approach",
            "research",
            0.45,
            PoolType::Exploration,
        );
        mem.store(
            "different task",
            "alternative method",
            "research",
            0.50,
            PoolType::Exploration,
        );
        let result = mem.retrieve("unseen task", "research", 1);
        assert!(result.confidence >= 0.0);
    }

    #[test]
    fn test_pool_weights_initial() {
        let w = PoolWeights::new();
        assert!((w.exploitation_weight - 0.6).abs() < 1e-6);
        assert!((w.exploration_weight - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_pool_weights_adjust() {
        let mut w = PoolWeights::new();
        w.adjust(true, 1.0);
        assert!(w.exploitation_weight > 0.6);
        assert!(w.exploration_weight < 0.4);
        assert_eq!(w.total_adjustments, 1);
    }

    #[test]
    fn test_stage_feedback() {
        let mut mem = DecentralizedMemory::new(test_agent_id("gamma"));
        let feedback = vec![
            StageFeedback {
                stage: "decompose".into(),
                score: 0.8,
                exploitation_effective: true,
                confidence: 0.9,
            },
            StageFeedback {
                stage: "reason".into(),
                score: 0.3,
                exploitation_effective: false,
                confidence: 0.7,
            },
        ];
        mem.apply_stage_feedback(&feedback);
        assert_eq!(mem.stats.weight_adjustments, 2);
    }

    #[test]
    fn test_consume_trajectory() {
        let mut mem = DecentralizedMemory::new(test_agent_id("delta"));
        mem.consume_trajectory("solve math", "math", "used chain-of-thought", 0.92);
        mem.consume_trajectory("write code", "code", "used test-driven", 0.45);
        assert_eq!(mem.exploitation_pool_size(), 1);
        assert_eq!(mem.exploration_pool_size(), 1);
    }

    #[test]
    fn test_generate_exploration_candidate() {
        let mut mem = DecentralizedMemory::new(test_agent_id("epsilon"));
        mem.store("task_a", "method_a", "domain_x", 0.5, PoolType::Exploration);
        mem.store("task_b", "method_b", "domain_x", 0.6, PoolType::Exploration);
        let candidate = mem.generate_exploration_candidate("task_c", "domain_x");
        assert!(candidate.is_some());
    }

    #[test]
    fn test_consolidation() {
        let mut mem = DecentralizedMemory::new(test_agent_id("zeta"));
        mem.store(
            "old task",
            "old method",
            "legacy",
            0.3,
            PoolType::Exploration,
        );
        mem.consolidate();
        assert_eq!(mem.exploration_pool_size(), 0);
    }

    #[test]
    fn test_manager_register_and_store() {
        let mut mgr = DecentralizedMemoryManager::new(10);
        let a1 = test_agent_id("agent_a");
        let a2 = test_agent_id("agent_b");
        mgr.register_agent(a1.clone());
        mgr.register_agent(a2.clone());
        assert_eq!(mgr.agent_count(), 2);
        mgr.store_collaborative(
            &[a1.clone(), a2.clone()],
            "collaborative task",
            "teamwork",
            "shared approach",
            0.75,
        );
        assert!(mgr.get(&a1).is_some());
        assert!(mgr.get(&a2).is_some());
    }

    #[test]
    fn test_manager_prune_all() {
        let mut mgr = DecentralizedMemoryManager::new(5);
        let a = test_agent_id("prune_test");
        mgr.register_agent(a.clone());
        if let Some(mem) = mgr.get_mut(&a) {
            for i in 0..20 {
                mem.store(
                    &format!("task_{}", i),
                    &format!("method_{}", i),
                    "test",
                    0.5,
                    PoolType::Exploitation,
                );
                mem.store(
                    &format!("task_{}", i),
                    &format!("method_{}", i),
                    "test",
                    0.5,
                    PoolType::Exploration,
                );
            }
        }
        mgr.prune_all(20);
        if let Some(mem) = mgr.get(&a) {
            assert!(mem.exploitation_pool_size() <= 10);
            assert!(mem.exploration_pool_size() <= 10);
        }
    }

    #[test]
    fn test_retrieve_no_match() {
        let mut mem = DecentralizedMemory::new(test_agent_id("no_match"));
        let result = mem.retrieve("something totally new", "unknown", 3);
        assert!(result.entries.is_empty());
    }

    #[test]
    fn test_manager_max_agents() {
        let mut mgr = DecentralizedMemoryManager::new(3);
        for i in 0..5 {
            mgr.register_agent(AgentId::new(&format!("agent_{}", i), "1.0"));
        }
        assert_eq!(mgr.agent_count(), 3);
    }

    #[test]
    fn test_pool_select_bias() {
        let w = PoolWeights::new();
        let mut exploit_count = 0;
        let total = 1000;
        for _ in 0..total {
            if w.select_pool() == PoolType::Exploitation {
                exploit_count += 1;
            }
        }
        let ratio = exploit_count as f64 / total as f64;
        assert!(ratio > 0.45 && ratio < 0.75);
    }

    #[test]
    fn test_stats_report_format() {
        let mgr = DecentralizedMemoryManager::new(5);
        let report = mgr.stats_report();
        assert!(report.contains("DecentralizedMemory"));
        assert!(report.contains("agents"));
    }
}
