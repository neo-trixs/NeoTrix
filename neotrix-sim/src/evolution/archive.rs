use crate::agents::SimAgent;

#[derive(Debug, Clone)]
pub struct EliteEntry {
    pub agent: SimAgent,
    pub fitness: f32,
    pub generation: u64,
    pub novelty: f32,
}

pub struct EliteArchive {
    pub entries: Vec<EliteEntry>,
    pub max_size: usize,
    pub min_fitness: f32,
}

impl EliteArchive {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
            min_fitness: 0.0,
        }
    }

    pub fn insert(&mut self, agent: &SimAgent, generation: u64, novelty: f32) -> bool {
        let fitness = agent.fitness() as f32;

        if fitness < self.min_fitness && self.entries.len() >= self.max_size {
            return false;
        }

        let entry = EliteEntry {
            agent: agent.clone(),
            fitness,
            generation,
            novelty,
        };

        self.entries.push(entry);
        self.entries
            .sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));

        if self.entries.len() > self.max_size {
            self.entries.truncate(self.max_size);
        }

        self.min_fitness = self.entries.last().map(|e| e.fitness).unwrap_or(0.0);
        true
    }

    pub fn get_best(&self) -> Option<&EliteEntry> {
        self.entries.first()
    }

    pub fn get_diverse_sample(&self, n: usize) -> Vec<&EliteEntry> {
        if self.entries.len() <= n {
            return self.entries.iter().collect();
        }

        let mut selected = Vec::new();
        let step = self.entries.len() / n;

        for i in (0..self.entries.len()).step_by(step) {
            if selected.len() >= n {
                break;
            }
            selected.push(&self.entries[i]);
        }

        selected
    }

    pub fn average_fitness(&self) -> f32 {
        if self.entries.is_empty() {
            return 0.0;
        }
        self.entries.iter().map(|e| e.fitness).sum::<f32>() / self.entries.len() as f32
    }

    pub fn generation_span(&self) -> u64 {
        if self.entries.is_empty() {
            return 0;
        }
        let max_gen = self.entries.iter().map(|e| e.generation).max().unwrap_or(0);
        let min_gen = self.entries.iter().map(|e| e.generation).min().unwrap_or(0);
        max_gen - min_gen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn test_archive_insert() {
        let mut archive = EliteArchive::new(10);
        let agent = SimAgent::new(0, Vec2::zero());
        assert!(archive.insert(&agent, 0, 0.5));
        assert_eq!(archive.entries.len(), 1);
    }

    #[test]
    fn test_archive_max_size() {
        let mut archive = EliteArchive::new(2);
        for i in 0..5 {
            let agent = SimAgent::new(i, Vec2::new(i as f32, i as f32));
            archive.insert(&agent, 0, 0.5);
        }
        assert_eq!(archive.entries.len(), 2);
    }

    #[test]
    fn test_archive_best() {
        let mut archive = EliteArchive::new(10);
        let mut a1 = SimAgent::new(0, Vec2::zero());
        a1.core.energy = 10.0;
        let mut a2 = SimAgent::new(1, Vec2::zero());
        a2.core.energy = 20.0;

        archive.insert(&a1, 0, 0.5);
        archive.insert(&a2, 0, 0.5);

        assert_eq!(archive.get_best().unwrap().fitness, archive.entries[0].fitness);
    }
}
