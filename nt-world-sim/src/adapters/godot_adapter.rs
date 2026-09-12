use crate::core::{UniversalEntity, EntityId, ArchetypeId, Event};

pub struct GodotAdapter;

impl GodotAdapter {
    pub fn from_godot_node(instance_id: u64) -> UniversalEntity {
        UniversalEntity {
            id: EntityId(instance_id),
            generation: 0,
            archetype: ArchetypeId(0),
        }
    }

    pub fn to_godot_node(entity: UniversalEntity) -> u64 {
        entity.id.0
    }

    pub fn signal_to_event(signal_name: &str, args: &[String]) -> Box<dyn Event> {
        Box::new(GodotSignalEvent {
            name: signal_name.to_string(),
            args: args.to_vec(),
        })
    }
}

#[allow(dead_code)]
struct GodotSignalEvent {
    name: String,
    args: Vec<String>,
}

impl Event for GodotSignalEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_godot_node() {
        let entity = GodotAdapter::from_godot_node(12345);
        assert_eq!(entity.id, EntityId(12345));
        assert_eq!(entity.generation, 0);
    }

    #[test]
    fn test_to_godot_node() {
        let entity = UniversalEntity {
            id: EntityId(99),
            generation: 1,
            archetype: ArchetypeId(0),
        };
        let id = GodotAdapter::to_godot_node(entity);
        assert_eq!(id, 99);
    }

    #[test]
    fn test_roundtrip() {
        let original = 42u64;
        let entity = GodotAdapter::from_godot_node(original);
        let result = GodotAdapter::to_godot_node(entity);
        assert_eq!(result, original);
    }

    #[test]
    fn test_signal_to_event() {
        let args = vec!["hello".to_string(), "world".to_string()];
        let event = GodotAdapter::signal_to_event("on_hit", &args);
        assert_eq!(event.type_id(), std::any::TypeId::of::<GodotSignalEvent>());
    }
}
