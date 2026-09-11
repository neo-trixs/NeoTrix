use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum InteractionType {
    PredatorPrey,
    Mutualism,
    Competition,
}

pub struct Coevolution {
    pub species_fitness: HashMap<String, f32>,
    pub interactions: Vec<(String, String, InteractionType)>,
}

impl Coevolution {
    pub fn new() -> Self {
        Coevolution {
            species_fitness: HashMap::new(),
            interactions: Vec::new(),
        }
    }

    pub fn add_species(&mut self, name: &str) {
        self.species_fitness.insert(name.into(), 1.0);
    }

    pub fn add_interaction(&mut self, sp_a: &str, sp_b: &str, it: InteractionType) {
        self.interactions
            .push((sp_a.into(), sp_b.into(), it));
    }

    pub fn fitness_modifier(&self, sp_a: &str, sp_b: &str) -> f32 {
        self.interactions
            .iter()
            .find(|(a, b, _)| (a == sp_a && b == sp_b) || (a == sp_b && b == sp_a))
            .map(|(_, _, it)| match it {
                InteractionType::PredatorPrey => 0.8,
                InteractionType::Mutualism => 1.2,
                InteractionType::Competition => 0.9,
            })
            .unwrap_or(1.0)
    }

    pub fn tick(&mut self) {
        for (_, fitness) in &mut self.species_fitness {
            *fitness = (*fitness * 1.01).min(2.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_species_initializes_at_one() {
        let mut coevo = Coevolution::new();
        coevo.add_species("wolf");
        assert_eq!(coevo.species_fitness["wolf"], 1.0);
    }

    #[test]
    fn fitness_modifier_predator_prey() {
        let mut coevo = Coevolution::new();
        coevo.add_interaction("wolf", "deer", InteractionType::PredatorPrey);
        assert_eq!(coevo.fitness_modifier("wolf", "deer"), 0.8);
    }

    #[test]
    fn fitness_modifier_no_interaction() {
        let coevo = Coevolution::new();
        assert_eq!(coevo.fitness_modifier("a", "b"), 1.0);
    }

    #[test]
    fn tick_grows_fitness() {
        let mut coevo = Coevolution::new();
        coevo.add_species("test");
        coevo.tick();
        assert!(coevo.species_fitness["test"] > 1.0);
    }
}
