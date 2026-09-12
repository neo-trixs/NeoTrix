use std::any::TypeId;
use crate::core::{UniversalEntity, EntityId, ArchetypeId, Component};

pub struct UnityAdapter;

impl UnityAdapter {
    pub fn from_unity_entity(index: i32, generation: i32) -> UniversalEntity {
        UniversalEntity {
            id: EntityId(index as u64),
            generation: generation as u32,
            archetype: ArchetypeId(0),
        }
    }

    pub fn to_unity_entity(entity: UniversalEntity) -> (i32, i32) {
        (entity.id.0 as i32, entity.generation as i32)
    }

    pub fn map_component<T: Component>(component: T) -> Box<dyn Component> {
        Box::new(component)
    }

    pub fn to_unity_query(required: &[TypeId], excluded: &[TypeId]) -> String {
        format!("UnityQuery({:?} - {:?})", required, excluded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_unity_entity() {
        let entity = UnityAdapter::from_unity_entity(5, 3);
        assert_eq!(entity.id, EntityId(5));
        assert_eq!(entity.generation, 3);
    }

    #[test]
    fn test_to_unity_entity() {
        let entity = UniversalEntity {
            id: EntityId(42),
            generation: 7,
            archetype: ArchetypeId(0),
        };
        let (index, gen) = UnityAdapter::to_unity_entity(entity);
        assert_eq!(index, 42);
        assert_eq!(gen, 7);
    }

    #[test]
    fn test_roundtrip() {
        let original = (10u32 as i32, 5u32 as i32);
        let entity = UnityAdapter::from_unity_entity(original.0, original.1);
        let result = UnityAdapter::to_unity_entity(entity);
        assert_eq!(result, original);
    }

    #[test]
    fn test_to_unity_query() {
        let required = vec![TypeId::of::<i32>(), TypeId::of::<String>()];
        let excluded = vec![TypeId::of::<f64>()];
        let query = UnityAdapter::to_unity_query(&required, &excluded);
        assert!(query.starts_with("UnityQuery("));
        assert!(query.contains("-"));
    }
}
