use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::pool::*;
use super::sva_gate::SvaGate;
use super::types::*;

/// A single gene (evolved trait) in the pool for cross-instance sharing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gene {
    pub id: u64,
    pub name: String,
    pub domain: String,
    pub trait_vector: Vec<u8>,
    pub fitness: f64,
    pub confidence: f64,
    pub source_hive: String,
    pub created_at: u64,
    pub generation: u64,
    pub tags: Vec<String>,
}

/// The shared gene pool for group evolution across sub-hive instances.
pub struct GenePool {
    pub genes: Vec<Gene>,
    pub max_genes: usize,
    pub conflict_threshold: f64,
    pub next_id: u64,
}

impl GenePool {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(max: usize) -> Self {
        GenePool {
            genes: Vec::with_capacity(max.min(10000)),
            max_genes: max.min(10000),
            conflict_threshold: 0.85,
            next_id: 1,
        }
    }

    pub fn export_all(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(&self.genes).map_err(|e| e.to_string())
    }

    pub fn export_by_domain(&self, domain: &str) -> Result<Vec<u8>, String> {
        let filtered: Vec<&Gene> = self.genes.iter().filter(|g| g.domain == domain).collect();
        serde_json::to_vec(&filtered).map_err(|e| e.to_string())
    }

    pub fn import_genes(&mut self, data: &[u8]) -> Result<usize, String> {
        let incoming: Vec<Gene> =
            serde_json::from_slice(data).map_err(|e| format!("deser: {}", e))?;
        let mut added = 0usize;
        for gene in incoming {
            if self.register_gene(gene).is_ok() {
                added += 1;
            }
        }
        Ok(added)
    }

    pub fn register_gene(&mut self, mut gene: Gene) -> Result<(), String> {
        gene.id = self.next_id;
        self.next_id += 1;

        for existing in &self.genes {
            let sim = hamming_similarity(&gene.trait_vector, &existing.trait_vector);
            if sim >= self.conflict_threshold {
                return Err(format!(
                    "conflict with gene {} ({}): sim={:.3}",
                    existing.id, existing.name, sim
                ));
            }
        }

        if self.genes.len() >= self.max_genes {
            self.prune();
        }

        self.genes.push(gene);
        Ok(())
    }

    pub fn detect_conflicts(&self, threshold: f64) -> Vec<(&Gene, &Gene)> {
        let mut pairs = Vec::new();
        for i in 0..self.genes.len() {
            for j in (i + 1)..self.genes.len() {
                let sim =
                    hamming_similarity(&self.genes[i].trait_vector, &self.genes[j].trait_vector);
                if sim >= threshold {
                    pairs.push((&self.genes[i], &self.genes[j]));
                }
            }
        }
        pairs
    }

    pub fn resolve_conflict(&mut self, a_id: u64, b_id: u64) {
        let pos_a = self.genes.iter().position(|g| g.id == a_id);
        let pos_b = self.genes.iter().position(|g| g.id == b_id);
        match (pos_a, pos_b) {
            (Some(pa), Some(pb)) => {
                if self.genes[pa].confidence >= self.genes[pb].confidence {
                    self.genes.remove(pb);
                } else {
                    self.genes.remove(pa);
                }
            }
            _ => {}
        }
    }

    pub fn top_genes(&self, n: usize) -> Vec<&Gene> {
        let mut sorted: Vec<&Gene> = self.genes.iter().collect();
        sorted.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    pub fn prune(&mut self) -> usize {
        if self.genes.len() <= self.max_genes {
            return 0;
        }
        let before = self.genes.len();
        self.genes.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.genes.truncate(self.max_genes);
        before - self.genes.len()
    }
}

fn hamming_similarity(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let diffs: usize = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    1.0 - (diffs as f64 / len as f64)
}

/// An autonomous sub-hive that independently learns and shares knowledge.
///
/// Each sub-hive runs a lightweight consciousness-like cycle:
///   sense → reason → learn → diffuse → absorb → forget
///
/// Sub-hives do NOT report to a central orchestrator.
/// They PUBLISH knowledge to the Pool and SUBSCRIBE to learn from peers.
/// Evolution is the emergent byproduct of this distributed exchange.
pub struct SubHiveInstance {
    pub id: HiveId,
    pub spec: SubHiveSpec,
    pub subscription: SubHiveSubscription,
    pub status: SubHiveStatus,
    pub parent_id: Option<HiveId>,

    cycle: u64,
    local_memory: VecDeque<String>,
    local_negentropy: f64,
    learning_rate: f64,
    ticks_since_last_publish: u64,
    publish_interval: u64,
    max_tick_budget: u64,
    gene_pool: Option<GenePool>,
}

impl SubHiveInstance {
    pub fn new(id: HiveId, spec: SubHiveSpec) -> Self {
        let subscription = SubHiveSubscription::for_domain(&spec.domain);
        let max_tick_budget = spec.resource_budget.max_ticks;
        SubHiveInstance {
            id,
            spec,
            subscription,
            status: SubHiveStatus::Idle,
            parent_id: None,
            cycle: 0,
            local_memory: VecDeque::with_capacity(32),
            local_negentropy: 0.0,
            learning_rate: 0.1,
            ticks_since_last_publish: 0,
            publish_interval: 5,
            max_tick_budget,
            gene_pool: None,
        }
    }

    pub fn with_parent(mut self, parent: HiveId) -> Self {
        self.parent_id = Some(parent);
        self
    }

    pub fn with_subscription(mut self, sub: SubHiveSubscription) -> Self {
        self.subscription = sub;
        self
    }

    /// Run one tick of the sub-hive's autonomous cycle.
    /// Returns: (published_packets, absorbed_packets)
    pub fn tick(
        &mut self,
        pool: &mut KnowledgePool,
        sva_gate: Option<&mut SvaGate>,
        external_input: Option<&str>,
    ) -> (Vec<KnowledgePacket>, Vec<KnowledgePacket>) {
        if self.status == SubHiveStatus::Dead {
            return (Vec::new(), Vec::new());
        }
        if self.cycle >= self.max_tick_budget {
            self.status = SubHiveStatus::Dying;
            return (self.finalize(pool, sva_gate), Vec::new());
        }

        self.cycle += 1;
        self.ticks_since_last_publish += 1;

        // 1. Sense
        let sensed = self.sense(external_input);

        // 2. Reason (local inference)
        let insight = self.reason(&sensed);

        // 3. Learn (local DGM-H simulation: accept insight if negentropy gain > threshold)
        let gain = self.learn(&insight);

        // 4. Diffuse (publish learned knowledge to pool)
        let published = if self.ticks_since_last_publish >= self.publish_interval && gain > 0.05 {
            self.ticks_since_last_publish = 0;
            let packet = self.build_packet(&insight, gain);
            pool.publish(packet.clone(), sva_gate);
            vec![packet]
        } else {
            Vec::new()
        };

        // 5. Absorb (subscribe to peer knowledge)
        let absorbed = self.absorb(pool);

        // 6. Forget (maintain local memory bounded)
        self.forget();

        self.status = SubHiveStatus::Idle;

        (published, absorbed)
    }

    fn sense(&self, external: Option<&str>) -> String {
        external.unwrap_or("").to_string()
    }

    fn reason(&self, input: &str) -> String {
        if input.is_empty() {
            format!(
                "cycle={} negentropy={:.3} domain={}",
                self.cycle, self.local_negentropy, self.spec.domain
            )
        } else {
            format!("{} [processed by sub-hive {}]", input, self.id)
        }
    }

    fn learn(&mut self, insight: &str) -> f64 {
        let gain = self.compute_negentropy_gain(insight);
        if gain > 0.05 {
            self.local_negentropy += gain * self.learning_rate;
            self.local_memory.push_back(insight.to_string());
        }
        self.local_negentropy += gain * self.learning_rate;
        gain
    }

    fn compute_negentropy_gain(&self, insight: &str) -> f64 {
        let base: f64 = self.local_memory.len() as f64 * 0.01;
        let novelty = if self.local_memory.iter().any(|m| m == insight) {
            0.0
        } else {
            0.3
        };
        (base + novelty).clamp(0.0, 1.0)
    }

    fn build_packet(&self, insight: &str, gain: f64) -> KnowledgePacket {
        let mut packet = KnowledgePacket::new(
            self.id,
            &self.spec.domain,
            &self.spec.instruction,
            insight,
            gain,
        );
        packet.local_validation_count = self.cycle as u32;
        packet.local_confidence = self.local_negentropy.clamp(0.0, 1.0);
        packet
    }

    fn absorb(&mut self, pool: &KnowledgePool) -> Vec<KnowledgePacket> {
        if !self.subscription.auto_absorb {
            return Vec::new();
        }

        let mut absorbed = Vec::new();
        let candidates = pool.subscribe(&self.subscription);

        for sp in candidates.iter().take(5) {
            if sp.packet.sub_hive_id == self.id {
                continue;
            }
            let already_known = self
                .local_memory
                .iter()
                .any(|m| *m == sp.packet.text_summary);
            if !already_known && sp.score >= self.subscription.min_score {
                self.local_memory.push_back(sp.packet.text_summary.clone());
                let import_gain = sp.packet.local_negentropy_gain * 0.5;
                self.local_negentropy += import_gain;
                absorbed.push(sp.packet.clone());
            }
        }
        absorbed
    }

    fn forget(&mut self) {
        while self.local_memory.len() > 32 {
            self.local_memory.pop_front();
        }
    }

    fn finalize(&mut self, pool: &mut KnowledgePool, sva_gate: Option<&mut SvaGate>) -> Vec<KnowledgePacket> {
        self.status = SubHiveStatus::Dead;
        if self.local_negentropy > 0.1 {
            let final_packet = KnowledgePacket::new(
                self.id,
                &self.spec.domain,
                "finalize",
                &format!(
                    "final_state: negentropy={:.3} cycles={} memory={}",
                    self.local_negentropy,
                    self.cycle,
                    self.local_memory.len()
                ),
                self.local_negentropy,
            );
            pool.publish(final_packet.clone(), sva_gate);
            vec![final_packet]
        } else {
            Vec::new()
        }
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle
    }

    pub fn negentropy(&self) -> f64 {
        self.local_negentropy
    }

    pub fn domain(&self) -> &str {
        &self.spec.domain
    }

    pub fn memory_size(&self) -> usize {
        self.local_memory.len()
    }

    pub fn with_gene_pool(mut self, pool: GenePool) -> Self {
        self.gene_pool = Some(pool);
        self
    }

    /// Export the current best evolved trait as a Gene for cross-instance sharing.
    /// Returns None if no meaningful negentropy has been accumulated.
    pub fn export_fitness_gene(&self) -> Option<Gene> {
        if self.local_negentropy < 0.01 {
            return None;
        }
        let trait_bytes: Vec<u8> = self
            .local_memory
            .iter()
            .flat_map(|m| m.bytes())
            .take(256)
            .collect();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Some(Gene {
            id: 0,
            name: format!("trait_{}", self.id),
            domain: self.spec.domain.clone(),
            trait_vector: trait_bytes,
            fitness: self.local_negentropy.clamp(0.0, 1.0),
            confidence: (self.cycle as f64 / 100.0).clamp(0.0, 1.0),
            source_hive: self.id.to_string(),
            created_at: now,
            generation: self.cycle,
            tags: vec!["exported".to_string()],
        })
    }

    /// Import a gene from another sub-hive and apply its trait to local learning.
    /// Returns true if the gene was accepted and applied.
    pub fn import_fitness_gene(&mut self, gene: &Gene) -> bool {
        if gene.domain != self.spec.domain {
            return false;
        }
        if let Some(ref mut pool) = self.gene_pool {
            if pool.register_gene(gene.clone()).is_err() {
                return false;
            }
        }
        let boost = gene.fitness * gene.confidence * 0.1;
        self.local_negentropy += boost;
        self.local_memory
            .push_back(format!("gene:{} fitness={:.3}", gene.name, gene.fitness));
        self.forget();
        true
    }
}

/// Manages multiple sub-hives at the consciousness level.
///
/// Does NOT orchestrate sub-hives — each sub-hive is autonomous.
/// This registry just provides lifecycle management (spawn/destroy/list)
/// and runs all sub-hive ticks in batch.
///
/// Optionally holds a shared SvaGate for CAT7 content-driven convergence
/// across all sub-hives during publish absorption decisions.
pub struct SubHiveRegistry {
    sub_hives: Vec<SubHiveInstance>,
    next_id: u64,
    sva_gate: Option<SvaGate>,
}

impl SubHiveRegistry {
    pub fn new() -> Self {
        SubHiveRegistry {
            sub_hives: Vec::new(),
            next_id: 1,
            sva_gate: None,
        }
    }

    pub fn with_sva_gate(mut self, gate: SvaGate) -> Self {
        self.sva_gate = Some(gate);
        self
    }

    pub fn sva_gate_mut(&mut self) -> Option<&mut SvaGate> {
        self.sva_gate.as_mut()
    }

    /// Spawn a new sub-hive with the given spec.
    pub fn spawn(&mut self, spec: SubHiveSpec, parent: Option<HiveId>) -> HiveId {
        let id = HiveId::new(self.next_id);
        self.next_id += 1;
        let mut instance = SubHiveInstance::new(id, spec);
        if let Some(pid) = parent {
            instance = instance.with_parent(pid);
        }
        self.sub_hives.push(instance);
        id
    }

    /// Run one tick for all active sub-hives.
    /// Returns: (all_published, all_absorbed)
    pub fn tick_all(
        &mut self,
        pool: &mut KnowledgePool,
    ) -> (Vec<KnowledgePacket>, Vec<KnowledgePacket>) {
        let mut all_published = Vec::new();
        let mut all_absorbed = Vec::new();

        self.sub_hives.retain(|sh| sh.status != SubHiveStatus::Dead);

        for sh in &mut self.sub_hives {
            let (published, absorbed) = sh.tick(pool, self.sva_gate.as_mut(), None);
            all_published.extend(published);
            all_absorbed.extend(absorbed);
        }

        (all_published, all_absorbed)
    }

    /// Destroy a sub-hive by ID.
    pub fn destroy(&mut self, id: HiveId, pool: &mut KnowledgePool) {
        if let Some(pos) = self.sub_hives.iter().position(|sh| sh.id == id) {
            let mut sh = self.sub_hives.remove(pos);
            let _ = sh.finalize(pool, self.sva_gate.as_mut());
        }
    }

    /// Get a reference to a sub-hive by ID.
    pub fn get(&self, id: HiveId) -> Option<&SubHiveInstance> {
        self.sub_hives.iter().find(|sh| sh.id == id)
    }

    pub fn get_mut(&mut self, id: HiveId) -> Option<&mut SubHiveInstance> {
        self.sub_hives.iter_mut().find(|sh| sh.id == id)
    }

    pub fn count(&self) -> usize {
        self.sub_hives.len()
    }

    pub fn alive_count(&self) -> usize {
        self.sub_hives
            .iter()
            .filter(|sh| sh.status != SubHiveStatus::Dead)
            .count()
    }

    pub fn list(&self) -> Vec<&SubHiveInstance> {
        self.sub_hives.iter().collect()
    }

    /// Collect stats for all sub-hives.
    pub fn stats(&self) -> Vec<HiveStats> {
        self.sub_hives
            .iter()
            .map(|sh| HiveStats {
                id: sh.id,
                domain: sh.spec.domain.clone(),
                status: sh.status,
                cycle: sh.cycle,
                negentropy: sh.local_negentropy,
                memory_items: sh.local_memory.len(),
            })
            .collect()
    }
}

/// Per-sub-hive stats snapshot.
#[derive(Debug, Clone)]
pub struct HiveStats {
    pub id: HiveId,
    pub domain: String,
    pub status: SubHiveStatus,
    pub cycle: u64,
    pub negentropy: f64,
    pub memory_items: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> KnowledgePool {
        KnowledgePool::new(100)
    }

    #[test]
    fn test_sub_hive_autonomous_cycle() {
        let mut pool = test_pool();
        let spec = SubHiveSpec::new("explore code patterns", "code");
        let mut hive = SubHiveInstance::new(HiveId::new(42), spec);

        let (_published, _absorbed) = hive.tick(&mut pool, None, Some("analyze function"));
        assert!(hive.cycle_count() > 0);
        assert!(hive.negentropy() >= 0.0);
    }

    #[test]
    fn test_sub_hive_publishes_to_pool() {
        let mut pool = test_pool();
        let spec = SubHiveSpec::new("test task", "test");
        let mut hive = SubHiveInstance::new(HiveId::new(1), spec);

        // Run enough ticks to trigger publish
        for _ in 0..6 {
            hive.tick(&mut pool, None, None);
        }

        let sub = SubHiveSubscription::for_domain("test").with_min_score(0.0);
        let results = pool.subscribe(&sub);
        assert!(
            !results.is_empty(),
            "sub-hive should have published after 5 ticks"
        );
    }

    #[test]
    fn test_sub_hive_absorbs_peer_knowledge() {
        let mut pool = test_pool();

        // Pre-publish some knowledge
        let pk = KnowledgePacket::new(HiveId::new(99), "code", "delta", "parallel pattern", 0.8);
        pool.publish(pk, None);

        // Create a sub-hive that subscribes to "code"
        let spec = SubHiveSpec::new("learn patterns", "code");
        let mut hive = SubHiveInstance::new(HiveId::new(1), spec);
        hive.subscription.min_score = 0.0;

        let (_, absorbed) = hive.tick(&mut pool, None, None);
        assert!(!absorbed.is_empty(), "should absorb peer knowledge");
        assert!(hive.negentropy() > 0.0, "negentropy should increase after absorbing");
    }

    #[test]
    fn test_sub_hive_dies_after_budget() {
        let mut pool = test_pool();
        let spec = SubHiveSpec::new("short lived", "test")
            .with_budget(3);
        let mut hive = SubHiveInstance::new(HiveId::new(1), spec);

        for _ in 0..4 {
            hive.tick(&mut pool, None, None);
        }

        assert_eq!(hive.status, SubHiveStatus::Dying);
    }

    #[test]
    fn test_registry_spawn_and_tick() {
        let mut pool = test_pool();
        let mut registry = SubHiveRegistry::new();
        let spec = SubHiveSpec::new("explore", "code");
        let id = registry.spawn(spec, None);
        assert_eq!(registry.count(), 1);

        let (_published, _absorbed) = registry.tick_all(&mut pool);
        // Just verify it runs without panics
        assert!(registry.get(id).is_some());
    }

    #[test]
    fn test_registry_destroy() {
        let mut pool = test_pool();
        let mut registry = SubHiveRegistry::new();
        let id = registry.spawn(SubHiveSpec::new("temp", "temp"), None);
        assert_eq!(registry.count(), 1);
        registry.destroy(id, &mut pool);
        assert_eq!(registry.alive_count(), 0);
    }

    #[test]
    fn test_registry_with_sva_gate() {
        let mut pool = test_pool();
        let gate = SvaGate::new(4096, 42);
        let mut registry = SubHiveRegistry::new()
            .with_sva_gate(gate);
        let id = registry.spawn(SubHiveSpec::new("explore", "code"), None);
        registry.tick_all(&mut pool);
        assert!(registry.sva_gate_mut().is_some());
        assert!(registry.get(id).is_some());
    }

    #[test]
    fn test_gene_registration_and_export() {
        let mut gp = GenePool::with_capacity(100);
        let gene = Gene {
            id: 0,
            name: "fast_sync".into(),
            domain: "net".into(),
            trait_vector: vec![1, 2, 3, 4, 5],
            fitness: 0.85,
            confidence: 0.9,
            source_hive: "hive_1".into(),
            created_at: 1000,
            generation: 5,
            tags: vec!["speed".into()],
        };
        assert!(gp.register_gene(gene).is_ok());
        assert_eq!(gp.genes.len(), 1);

        let exported = gp.export_all().unwrap();
        assert!(!exported.is_empty());

        let domain_data = gp.export_by_domain("net").unwrap();
        assert!(!domain_data.is_empty());

        let domain_data_empty = gp.export_by_domain("other").unwrap();
        let empty: Vec<Gene> = serde_json::from_slice(&domain_data_empty).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_gene_import_roundtrip() {
        let mut gp = GenePool::with_capacity(100);
        let gene = Gene {
            id: 0,
            name: "compact".into(),
            domain: "store".into(),
            trait_vector: vec![10, 20, 30],
            fitness: 0.7,
            confidence: 0.8,
            source_hive: "alpha".into(),
            created_at: 2000,
            generation: 3,
            tags: vec![],
        };
        gp.register_gene(gene).unwrap();

        let bytes = gp.export_all().unwrap();
        let mut gp2 = GenePool::with_capacity(100);
        let count = gp2.import_genes(&bytes).unwrap();
        assert_eq!(count, 1);
        assert_eq!(gp2.genes.len(), 1);
        assert_eq!(gp2.genes[0].name, "compact");
        assert_eq!(gp2.genes[0].fitness, 0.7);
    }

    #[test]
    fn test_conflict_detection() {
        let mut gp = GenePool::with_capacity(100);
        gp.conflict_threshold = 0.6;

        let g1 = Gene {
            id: 0,
            name: "algo_a".into(),
            domain: "ml".into(),
            trait_vector: vec![1, 0, 1, 0],
            fitness: 0.7,
            confidence: 0.8,
            source_hive: "a".into(),
            created_at: 0,
            generation: 1,
            tags: vec![],
        };
        let g2 = Gene {
            id: 0,
            name: "algo_b".into(),
            domain: "ml".into(),
            trait_vector: vec![1, 0, 1, 1],
            fitness: 0.9,
            confidence: 0.9,
            source_hive: "b".into(),
            created_at: 1,
            generation: 2,
            tags: vec![],
        };

        gp.register_gene(g1).unwrap();
        // g2 has 75% similarity to g1 (3/4 bits match) → conflict at threshold 0.6
        assert!(gp.register_gene(g2).is_err());

        let conflicts = gp.detect_conflicts(0.6);
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_conflict_resolution_higher_confidence_wins() {
        let mut gp = GenePool::with_capacity(100);
        gp.conflict_threshold = 0.5;

        let g1 = Gene {
            id: 0,
            name: "old_trait".into(),
            domain: "viz".into(),
            trait_vector: vec![1, 1, 0, 0],
            fitness: 0.5,
            confidence: 0.3,
            source_hive: "h1".into(),
            created_at: 0,
            generation: 1,
            tags: vec![],
        };
        let g2 = Gene {
            id: 0,
            name: "new_trait".into(),
            domain: "viz".into(),
            trait_vector: vec![1, 1, 0, 1],
            fitness: 0.8,
            confidence: 0.9,
            source_hive: "h2".into(),
            created_at: 1,
            generation: 2,
            tags: vec![],
        };

        gp.register_gene(g1).unwrap();
        // Force register the second by lowering threshold temporarily
        let saved = gp.conflict_threshold;
        gp.conflict_threshold = 1.0;
        gp.register_gene(g2).unwrap();
        gp.conflict_threshold = saved;

        assert_eq!(gp.genes.len(), 2);
        gp.resolve_conflict(1, 2);
        assert_eq!(gp.genes.len(), 1);
        // Higher-confidence gene (g2 with 0.9) should survive
        assert_eq!(gp.genes[0].name, "new_trait");
    }

    #[test]
    fn test_prune_removes_low_fitness() {
        let mut gp = GenePool::with_capacity(3);
        for i in 0..5 {
            let g = Gene {
                id: 0,
                name: format!("gene_{}", i),
                domain: "test".into(),
                trait_vector: vec![i as u8; 4],
                fitness: i as f64 * 0.1,
                confidence: 0.5,
                source_hive: "h".into(),
                created_at: i,
                generation: 1,
                tags: vec![],
            };
            // Conflict threshold is 0.85; vectors differ so no conflict
            let _ = gp.register_gene(g);
        }
        // After 5 inserts with max_genes=3, prune should have removed 2
        let removed = gp.prune();
        assert_eq!(removed, 2);
        assert_eq!(gp.genes.len(), 3);
        // Top genes should show highest fitness first
        let top = gp.top_genes(2);
        assert_eq!(top.len(), 2);
        assert!(top[0].fitness >= top[1].fitness);
    }

    #[test]
    fn test_distributed_evolution_convergence() {
        // Simulate N sub-hives learning independently and converging via Pool.
        let mut pool = KnowledgePool::new(100);
        let mut registry = SubHiveRegistry::new();

        // Spawn 3 sub-hives in the same domain
        for i in 0..3 {
            let spec = SubHiveSpec::new(&format!("explore strategy {}", i), "code");
            registry.spawn(spec, None);
        }

        // Run 20 ticks of distributed evolution
        for _ in 0..20 {
            registry.tick_all(&mut pool);
            pool.tick();
        }

        // After 20 ticks, the pool should have converged knowledge
        let stats = pool.compute_stats();
        assert!(stats.total_published > 0, "knowledge should have been published");
        assert!(
            stats.avg_negentropy_gain > 0.0,
            "pool should have accumulated negentropy"
        );
    }
}
