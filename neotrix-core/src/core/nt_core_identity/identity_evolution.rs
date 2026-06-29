use rand::Rng;
use serde::{Deserialize, Serialize};

use super::identity_core::IdentityCore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityEvolutionConfig {
    pub mutation_rate: f64,
    pub selection_strength: f64,
    pub max_versions: usize,
    pub elitism_ratio: f64,
}

impl Default for IdentityEvolutionConfig {
    fn default() -> Self {
        Self {
            mutation_rate: 0.05,
            selection_strength: 0.3,
            max_versions: 16,
            elitism_ratio: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityVersion {
    pub version: u64,
    pub self_vsa: Vec<u8>,
    pub personality_traits: Vec<Vec<u8>>,
    pub core_values: Vec<String>,
    pub fitness: f64,
    pub parent_version: Option<u64>,
    pub created_at: u64,
    pub mutation_log: Vec<String>,
    pub selection_event: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IdentityEvolution {
    config: IdentityEvolutionConfig,
    versions: Vec<IdentityVersion>,
    current_version: u64,
    candidate_pool: Vec<IdentityVersion>,
}

impl IdentityEvolution {
    pub fn new(config: IdentityEvolutionConfig) -> Self {
        let max_versions = config.max_versions;
        Self {
            config,
            versions: Vec::with_capacity(max_versions),
            current_version: 0,
            candidate_pool: Vec::new(),
        }
    }

    pub fn init_from(&mut self, identity: &IdentityCore) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let version = IdentityVersion {
            version: 0,
            self_vsa: identity.self_vsa.clone(),
            personality_traits: identity.personality_traits.clone(),
            core_values: identity.core_values.clone(),
            fitness: 1.0,
            parent_version: None,
            created_at: now,
            mutation_log: vec!["initial version".to_string()],
            selection_event: Some("identity initialized".to_string()),
        };
        self.versions.push(version);
        self.current_version = 0;
        0
    }

    pub fn propose_mutations(
        &self,
        identity: &IdentityCore,
        session_success_rate: f64,
    ) -> Vec<IdentityVersion> {
        let num_candidates = (self.config.max_versions / 2).max(2);
        let mut rng = rand::thread_rng();
        let effective_mutation_rate = if session_success_rate > 0.7 {
            self.config.mutation_rate * 0.5
        } else if session_success_rate < 0.3 {
            (self.config.mutation_rate * 3.0).min(0.5)
        } else {
            self.config.mutation_rate
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut candidates = Vec::with_capacity(num_candidates);

        for i in 0..num_candidates {
            let mut self_vsa = identity.self_vsa.clone();
            let mut personality_traits = identity.personality_traits.clone();
            let mut core_values = identity.core_values.clone();
            let mut mutation_log = Vec::new();

            let vsa_flips: usize = self_vsa
                .iter()
                .filter(|_| rng.gen::<f64>() < effective_mutation_rate)
                .count();
            for byte in self_vsa.iter_mut() {
                if rng.gen::<f64>() < effective_mutation_rate {
                    *byte ^= rng.gen::<u8>();
                }
            }
            if vsa_flips > 0 {
                mutation_log.push(format!("vsa_{}_bits_flipped", vsa_flips));
            }

            let mut trait_flips = 0;
            for trait_vsa in personality_traits.iter_mut() {
                for byte in trait_vsa.iter_mut() {
                    if rng.gen::<f64>() < effective_mutation_rate {
                        *byte ^= 1 << rng.gen_range(0..8);
                        trait_flips += 1;
                    }
                }
            }
            if trait_flips > 0 {
                mutation_log.push(format!("traits_{}_bits_flipped", trait_flips));
            }

            if core_values.len() >= 2 {
                let a = rng.gen_range(0..core_values.len());
                let b = rng.gen_range(0..core_values.len());
                if a != b {
                    core_values.swap(a, b);
                    mutation_log.push(format!("values_{}_{}_swapped", a, b));
                }
            }

            if mutation_log.is_empty() {
                mutation_log.push("no_mutations".to_string());
            }

            candidates.push(IdentityVersion {
                version: self.current_version + 1 + i as u64,
                self_vsa,
                personality_traits,
                core_values,
                fitness: 0.0,
                parent_version: Some(self.current_version),
                created_at: now,
                mutation_log,
                selection_event: None,
            });
        }

        candidates
    }

    pub fn compute_fitness(&self, identity: &IdentityCore, version: &IdentityVersion) -> f64 {
        let anchor_sim = hamming_similarity(&version.self_vsa, &identity.anchor_self_vsa);
        let coherence = identity.current_coherence();

        let diversity = if version.personality_traits.len() >= 2 {
            let mut total_dist = 0.0;
            let mut pairs = 0;
            for i in 0..version.personality_traits.len() {
                for j in (i + 1)..version.personality_traits.len() {
                    total_dist += 1.0
                        - hamming_similarity(
                            &version.personality_traits[i],
                            &version.personality_traits[j],
                        );
                    pairs += 1;
                }
            }
            if pairs > 0 {
                total_dist / pairs as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        0.4 * anchor_sim + 0.3 * coherence + 0.3 * diversity
    }

    pub fn select_elite(
        &self,
        mut candidates: Vec<IdentityVersion>,
        n: usize,
    ) -> Vec<IdentityVersion> {
        if candidates.len() <= n {
            candidates.sort_by(|a, b| {
                b.fitness
                    .partial_cmp(&a.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return candidates;
        }

        let mut rng = rand::thread_rng();
        let tournament_size = (self.config.selection_strength * 10.0).ceil() as usize;
        let tournament_size = tournament_size.max(2).min(candidates.len());
        let mut selected = Vec::with_capacity(n);

        for _ in 0..n {
            let mut best_idx = rng.gen_range(0..candidates.len());
            let mut best_fitness = candidates[best_idx].fitness;
            for _ in 1..tournament_size {
                let idx = rng.gen_range(0..candidates.len());
                if candidates[idx].fitness > best_fitness {
                    best_idx = idx;
                    best_fitness = candidates[idx].fitness;
                }
            }
            selected.push(candidates.remove(best_idx));
        }

        selected.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        selected
    }

    pub fn apply_evolution(&mut self, identity: &mut IdentityCore, session_success_rate: f64) {
        let candidates = self.propose_mutations(identity, session_success_rate);
        if candidates.is_empty() {
            return;
        }

        let scored: Vec<IdentityVersion> = candidates
            .into_iter()
            .map(|mut c| {
                c.fitness = self.compute_fitness(identity, &c);
                c
            })
            .collect();

        let elite_count = (scored.len() as f64 * self.config.elitism_ratio).ceil() as usize;
        let elite_count = elite_count.max(1);
        let elite = self.select_elite(scored, elite_count);

        if elite.is_empty() {
            return;
        }

        let best = &elite[0];
        identity.self_vsa = best.self_vsa.clone();
        identity.personality_traits = best.personality_traits.clone();
        identity.core_values = best.core_values.clone();
        identity.mark_dirty();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let selection_event = if session_success_rate > 0.7 {
            "conservative_reinforcement"
        } else if session_success_rate < 0.3 {
            "exploratory_adaptation"
        } else {
            "balanced_evolution"
        };

        let mut evolved = best.clone();
        evolved.selection_event = Some(selection_event.to_string());
        evolved.created_at = now;
        self.current_version = evolved.version;
        self.candidate_pool = elite;

        if self.versions.len() >= self.config.max_versions {
            self.versions.remove(0);
        }
        self.versions.push(evolved);
    }

    pub fn rollback_to(&mut self, identity: &mut IdentityCore, version_num: u64) -> bool {
        if let Some(pos) = self.versions.iter().position(|v| v.version == version_num) {
            let target = &self.versions[pos];
            identity.self_vsa = target.self_vsa.clone();
            identity.personality_traits = target.personality_traits.clone();
            identity.core_values = target.core_values.clone();
            identity.mark_dirty();
            self.versions.truncate(pos + 1);
            self.current_version = version_num;
            true
        } else {
            false
        }
    }

    pub fn version_history(&self) -> &[IdentityVersion] {
        &self.versions
    }

    pub fn latest_version(&self) -> Option<&IdentityVersion> {
        self.versions.last()
    }

    pub fn report(&self) -> String {
        let latest = self.latest_version();
        format!(
            "evolution:versions_{}_current_{}_best_fitness_{:.4}",
            self.versions.len(),
            self.current_version,
            latest.map(|v| v.fitness).unwrap_or(0.0),
        )
    }
}

fn hamming_similarity(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let same = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
    same as f64 / a.len() as f64
}
