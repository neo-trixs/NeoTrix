use crate::core::{UniversalEntity, UniversalWorld, Component, EntityId, ArchetypeId};

pub trait ExternalEntity: Send + Sync + 'static {
    fn index(&self) -> u32;
    fn generation(&self) -> u32;
}

pub trait ExternalComponent: Send + Sync + Clone + 'static {}

pub trait ExternalPlugin: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn register(self, world: &mut UniversalWorld);
}

pub struct BevyAdapter;

impl BevyAdapter {
    pub fn from_external_entity<E: ExternalEntity>(entity: &E) -> UniversalEntity {
        UniversalEntity {
            id: EntityId(entity.index() as u64),
            generation: entity.generation(),
            archetype: ArchetypeId(0),
        }
    }

    pub fn to_external_id(entity: UniversalEntity) -> (u32, u32) {
        (entity.id.0 as u32, entity.generation)
    }

    pub fn register_plugin<P: ExternalPlugin>(world: &mut UniversalWorld, plugin: P) {
        plugin.register(world);
    }
}

pub struct NullPlugin;

impl ExternalPlugin for NullPlugin {
    fn name(&self) -> &str {
        "null"
    }

    fn register(self, _world: &mut UniversalWorld) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity {
        idx: u32,
        gen: u32,
    }

    impl ExternalEntity for TestEntity {
        fn index(&self) -> u32 {
            self.idx
        }

        fn generation(&self) -> u32 {
            self.gen
        }
    }

    #[test]
    fn test_from_external_entity() {
        let ext = TestEntity { idx: 42, gen: 3 };
        let ue = BevyAdapter::from_external_entity(&ext);
        assert_eq!(ue.id, EntityId(42));
        assert_eq!(ue.generation, 3);
    }

    #[test]
    fn test_to_external_id() {
        let ue = UniversalEntity {
            id: EntityId(7),
            generation: 2,
            archetype: ArchetypeId(0),
        };
        let (idx, gen) = BevyAdapter::to_external_id(ue);
        assert_eq!(idx, 7);
        assert_eq!(gen, 2);
    }

    #[test]
    fn test_register_plugin() {
        let mut world = UniversalWorld::new();
        BevyAdapter::register_plugin(&mut world, NullPlugin);
        assert_eq!(world.entity_count(), 0);
    }
}
