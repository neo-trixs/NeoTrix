use crate::core::{UniversalEntity, EntityId, ArchetypeId};

/// Unreal Engine 5 adapter
/// Maps UE5 Actors, Components, and Gameplay Tags to universal types
pub struct UnrealAdapter;

impl UnrealAdapter {
    /// Convert UE5 Actor ID to UniversalEntity
    /// UE5 uses FName for actors, but we map to numeric ID
    pub fn from_unreal_actor(unique_id: u32, name: &str) -> UniversalEntity {
        let _ = name; // Reserved for future use
        UniversalEntity {
            id: EntityId(unique_id as u64),
            generation: 0,
            archetype: ArchetypeId(0),
        }
    }

    /// Convert UniversalEntity to UE5 format
    pub fn to_unreal_entity(entity: UniversalEntity) -> (u32, u32) {
        (entity.id.0 as u32, entity.generation)
    }

    /// Map UE5 Gameplay Tag to string path
    pub fn gameplay_tag_to_string(tag: &str) -> String {
        // UE5 tags use dot notation: "Ability.Skill.Fireball"
        tag.to_string()
    }

    /// Map string to UE5 Gameplay Tag format
    pub fn string_to_gameplay_tag(path: &str) -> String {
        path.to_string()
    }

    /// Convert UE5 ActorComponent type name to TypeId-compatible string
    pub fn component_type_name(component_class: &str) -> String {
        // UE5 components: UStaticMeshComponent, UPrimitiveComponent, etc.
        component_class.to_string()
    }

    /// Generate UE5 GAS (Gameplay Ability System) attribute set name
    pub fn attribute_set_name(entity_name: &str) -> String {
        format!("U{}AttributeSet", entity_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unreal_entity_conversion() {
        let entity = UnrealAdapter::from_unreal_actor(42, "MyActor");
        assert_eq!(entity.id, EntityId(42));
        let (id, gen) = UnrealAdapter::to_unreal_entity(entity);
        assert_eq!(id, 42);
        assert_eq!(gen, 0);
    }

    #[test]
    fn test_gameplay_tag() {
        let tag = UnrealAdapter::gameplay_tag_to_string("Ability.Skill.Fireball");
        assert_eq!(tag, "Ability.Skill.Fireball");
        let reversed = UnrealAdapter::string_to_gameplay_tag(&tag);
        assert_eq!(reversed, "Ability.Skill.Fireball");
    }

    #[test]
    fn test_attribute_set_name() {
        let name = UnrealAdapter::attribute_set_name("Player");
        assert_eq!(name, "UPlayerAttributeSet");
    }
}
