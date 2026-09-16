use super::component::ComponentStore;
use super::entity::EntityId;

pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn update(&self, entities: &[EntityId], components: &mut ComponentStore, dt: f64);
    fn priority(&self) -> i32 {
        0
    }
}

pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
        self.systems.sort_by_key(|s| s.priority());
    }

    pub fn run(&self, entities: &[EntityId], components: &mut ComponentStore, dt: f64) {
        for system in &self.systems {
            system.update(entities, components, dt);
        }
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    pub fn system_names(&self) -> Vec<&str> {
        self.systems.iter().map(|s| s.name()).collect()
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}
