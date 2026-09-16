#[derive(Debug, Clone)]
pub struct SkillVersion {
    pub name: String,
    pub version: u32,
    pub fitness: f64,
}

pub struct MetaSkillEvolve {
    skills: Vec<SkillVersion>,
    mutation_rate: f64,
}

impl MetaSkillEvolve {
    pub fn new(mutation_rate: f64) -> Self {
        Self {
            skills: Vec::new(),
            mutation_rate,
        }
    }

    pub fn register(&mut self, name: &str) {
        self.skills.push(SkillVersion {
            name: name.to_string(),
            version: 0,
            fitness: 0.5,
        });
    }

    pub fn evolve(&mut self, name: &str) -> bool {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.version += 1;
            s.fitness =
                (s.fitness + (rand::random::<f64>() - 0.5) * self.mutation_rate).clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    pub fn best(&self) -> Option<&SkillVersion> {
        self.skills.iter().max_by(|a, b| {
            a.fitness
                .partial_cmp(&b.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }
}

impl Default for MetaSkillEvolve {
    fn default() -> Self {
        Self::new(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolve() {
        let mut e = MetaSkillEvolve::new(0.1);
        e.register("rust");
        e.evolve("rust");
        assert_eq!(e.skills[0].version, 1);
    }

    #[test]
    fn test_best() {
        let mut e = MetaSkillEvolve::new(0.1);
        e.register("a");
        e.register("b");
        assert!(e.best().is_some());
    }
}
