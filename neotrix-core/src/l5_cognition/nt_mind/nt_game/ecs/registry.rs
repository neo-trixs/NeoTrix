use super::component::ComponentStore;
use super::entity::EntityManager;
use super::system::SystemScheduler;

pub struct World {
    pub entities: EntityManager,
    pub components: ComponentStore,
    pub systems: SystemScheduler,
    pub time: f64,
    pub delta_time: f64,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            components: ComponentStore::new(),
            systems: SystemScheduler::new(),
            time: 0.0,
            delta_time: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.delta_time = dt;
        self.time += dt;
        let alive = self.entities.alive_ids();
        self.systems.run(&alive, &mut self.components, dt);
    }

    pub fn spawn_entity(&mut self) -> super::entity::EntityId {
        self.entities.create()
    }

    pub fn destroy_entity(&mut self, id: super::entity::EntityId) {
        self.entities.destroy(id);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct Position {
        x: f64,
        y: f64,
    }

    impl super::super::component::Component for Position {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_world_creation() {
        let w = World::new();
        assert_eq!(w.entities.count(), 0);
    }

    #[test]
    fn test_spawn_and_component() {
        let mut w = World::new();
        let e = w.spawn_entity();
        w.components.insert(e, Position { x: 1.0, y: 2.0 });
        assert_eq!(w.components.get::<Position>(e).unwrap().x, 1.0);
    }
}
