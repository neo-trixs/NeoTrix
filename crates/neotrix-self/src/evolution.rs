use serde::{Deserialize, Serialize};

use crate::identity::IdentityCore;

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

    pub fn propose_mutations(&self, identity: &IdentityCore, session_success_rate: f64) -> Vec<IdentityVersion> {
        let num_candidates = (self.config.max_versions / 2).max(2);
        let mut rng = rand::thread_rng();

        let mut candidates = Vec::with_capacity(num_candidates);
        for i in 0..num_candidates {
            let mut mutated_vsa = identity.self_vsa.clone();
            let mut mutation_log = Vec::new();

            let mutation_count = (mutated_vsa.len() as f64 * self.config.mutation_rate).round() as usize;
            for _ in 0..mutation_count {
                let idx = rand::Rng::gen_range(&mut rng, 0..mutated_vsa.len());
                mutated_vsa[idx] = rand::Rng::gen(&mut rng);
            }
            mutation_log.push(format!("mutated {} bits", mutation_count));

            let mut mutated_traits: Vec<Vec<u8>> = identity.personality_traits.clone();
            if rand::Rng::gen_bool(&mut rng, 0.2) {
                let new_trait: Vec<u8> = (0..4096).map(|_| rand::Rng::gen(&mut rng)).collect();
                if mutated_traits.len() < 32 {
                    mutated_traits.push(new_trait);
                    mutation_log.push("added personality trait".to_string());
                }
            }

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let fitness = session_success_rate + rand::Rng::gen::<f64>(&mut rng) * 0.2 - 0.1;

            candidates.push(IdentityVersion {
                version: self.current_version + 1 + i as u64,
                self_vsa: mutated_vsa,
                personality_traits: mutated_traits,
                core_values: identity.core_values.clone(),
                fitness: fitness.clamp(0.0, 1.0),
                parent_version: Some(self.current_version),
                created_at: now,
                mutation_log,
                selection_event: None,
            });
        }

        candidates
    }

    pub fn apply_evolution(&mut self, identity: &mut IdentityCore, session_success_rate: f64) -> u64 {
        let candidates = self.propose_mutations(identity, session_success_rate);
        self.candidate_pool.extend(candidates);

        let selection_count = (self.candidate_pool.len() as f64 * self.config.selection_strength).ceil() as usize;
        let selection_count = selection_count.max(1).min(self.candidate_pool.len());

        self.candidate_pool.sort_by(|a, b| {
            b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal)
        });

        let best = self.candidate_pool[0].clone();
        let selected: Vec<IdentityVersion> = self.candidate_pool.drain(..selection_count).collect();

        identity.self_vsa = best.self_vsa;
        identity.personality_traits = best.personality_traits;
        identity.core_values = best.core_values;

        let new_version = IdentityVersion {
            version: self.current_version + 1,
            self_vsa: identity.self_vsa.clone(),
            personality_traits: identity.personality_traits.clone(),
            core_values: identity.core_values.clone(),
            fitness: best.fitness,
            parent_version: Some(self.current_version),
            created_at: best.created_at,
            mutation_log: best.mutation_log,
            selection_event: Some(format!("selected from {} candidates", selected.len())),
        };

        self.versions.push(new_version);
        if self.versions.len() > self.config.max_versions {
            self.versions.remove(0);
        }
        self.current_version += 1;

        self.current_version
    }

    pub fn rollback_to(&mut self, identity: &mut IdentityCore, version: u64) -> bool {
        let target = match self.versions.iter().find(|v| v.version == version) {
            Some(v) => v.clone(),
            None => return false,
        };

        identity.self_vsa = target.self_vsa;
        identity.personality_traits = target.personality_traits;
        identity.core_values = target.core_values;
        true
    }

    pub fn report(&self) -> String {
        format!(
            "evolution:versions_{}_current_v{}_candidates_{}",
            self.versions.len(),
            self.current_version,
            self.candidate_pool.len(),
        )
    }
}
