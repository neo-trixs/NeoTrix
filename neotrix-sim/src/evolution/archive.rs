pub struct AgentSnapshot {
    pub id: u32,
    pub fitness: f32,
    pub generation: u64,
}

pub struct EliteArchive {
    pub elites: Vec<AgentSnapshot>,
    pub max_size: usize,
}

impl EliteArchive {
    pub fn new(max_size: usize) -> Self {
        EliteArchive {
            elites: Vec::new(),
            max_size,
        }
    }

    pub fn update(&mut self, agent: AgentSnapshot) {
        self.elites.push(agent);
        self.elites.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.elites.truncate(self.max_size);
    }

    pub fn get_best(&self) -> Option<&AgentSnapshot> {
        self.elites.first()
    }

    pub fn diversity(&self) -> f32 {
        if self.elites.len() < 2 {
            return 0.0;
        }
        let mean: f32 =
            self.elites.iter().map(|e| e.fitness).sum::<f32>() / self.elites.len() as f32;
        self.elites
            .iter()
            .map(|e| (e.fitness - mean).powi(2))
            .sum::<f32>()
            .sqrt()
            / self.elites.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_capacity() {
        let archive = EliteArchive::new(5);
        assert_eq!(archive.max_size, 5);
        assert!(archive.elites.is_empty());
    }

    #[test]
    fn update_truncates_to_max() {
        let mut archive = EliteArchive::new(3);
        for i in 0..5 {
            archive.update(AgentSnapshot {
                id: i,
                fitness: i as f32 * 0.1,
                generation: 1,
            });
        }
        assert_eq!(archive.elites.len(), 3);
    }

    #[test]
    fn get_best_returns_highest_fitness() {
        let mut archive = EliteArchive::new(5);
        archive.update(AgentSnapshot { id: 1, fitness: 0.3, generation: 1 });
        archive.update(AgentSnapshot { id: 2, fitness: 0.9, generation: 1 });
        assert_eq!(archive.get_best().unwrap().id, 2);
    }

    #[test]
    fn diversity_empty_is_zero() {
        let archive = EliteArchive::new(5);
        assert_eq!(archive.diversity(), 0.0);
    }

    #[test]
    fn diversity_single_is_zero() {
        let mut archive = EliteArchive::new(5);
        archive.update(AgentSnapshot { id: 1, fitness: 0.5, generation: 1 });
        assert_eq!(archive.diversity(), 0.0);
    }
}
