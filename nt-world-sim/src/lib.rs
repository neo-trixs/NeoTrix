pub mod adapters;
pub mod builder;
pub mod codegen;
pub mod core;
pub mod engine;
pub mod game;
pub mod tauri_bridge;
pub mod ui;
pub mod world;


pub use engine::{
    Color, Vec2, Rect, Transform, Sprite, TileDef, TileMap, Camera, Renderer, CanvasRenderer,
    PhysicsEntity, BodyType, RigidBody, Collider, CollisionInfo, PhysicsWorld, SimplePhysicsWorld,
    KeyCode, MouseButton, GamepadAxis, GamepadButton, InputState, InputProvider, SimpleInputProvider,
};
pub use engine::scene::{SceneGraph, SceneNode};
pub use engine::event_bus::{TypedEventBus, GameEventHandler};
pub use engine::audio::{AudioManager, AudioBackend, AudioHandle, StubAudioBackend};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::{CodeGenerator, EntityDef, GameDefinition, SystemDef};

    #[test]
    fn test_physics() {
        let mut physics = SimplePhysicsWorld::new();
        let entity = crate::core::entity::EntityId(0);
        let body = RigidBody::dynamic();
        physics.add_body(entity, body);
        physics.step(0.016);
    }

    #[test]
    fn test_input() {
        let mut input = SimpleInputProvider::new();
        input.key_down(KeyCode::W);
        assert!(input.is_key_pressed(KeyCode::W));
        assert!(input.is_key_just_pressed(KeyCode::W));
    }

    #[test]
    fn test_game_def_default() {
        let def = GameDefinition::default();
        assert_eq!(def.name, "MyGame");
        assert_eq!(def.version, "0.1.0");
        assert_eq!(def.engine.target, "bevy");
    }

    #[test]
    fn test_code_generator_bevy() {
        let mut def = GameDefinition::default();
        def.entities.insert(
            "Player".to_string(),
            EntityDef {
                components: vec!["health".to_string(), "speed".to_string()],
                systems: vec!["movement".to_string()],
            },
        );
        def.systems.insert(
            "movement".to_string(),
            SystemDef {
                priority: 0,
                read: vec![],
                write: vec![],
            },
        );
        let gen = CodeGenerator::new(def);
        let code = gen.generate_bevy();
        assert!(code.contains("struct Player"));
        assert!(code.contains("health: f32"));
        assert!(code.contains("fn movement_system"));
    }

    #[test]
    fn test_code_generator_unity() {
        let mut def = GameDefinition::default();
        def.engine.target = "unity".to_string();
        def.entities.insert(
            "Enemy".to_string(),
            EntityDef {
                components: vec!["damage".to_string()],
                systems: vec![],
            },
        );
        let gen = CodeGenerator::new(def);
        let code = gen.generate_unity();
        assert!(code.contains("struct Enemy : IComponentData"));
        assert!(code.contains("public float damage"));
    }

    #[test]
    fn test_code_generator_godot() {
        let mut def = GameDefinition::default();
        def.engine.target = "godot".to_string();
        def.entities.insert(
            "NPC".to_string(),
            EntityDef {
                components: vec![],
                systems: vec![],
            },
        );
        let gen = CodeGenerator::new(def);
        let code = gen.generate_godot();
        assert!(code.contains("class_name NPC"));
        assert!(code.contains("func _ready()"));
    }

    #[test]
    fn test_json_roundtrip() {
        let mut def = GameDefinition::default();
        def.entities.insert(
            "Test".to_string(),
            EntityDef {
                components: vec!["x".to_string()],
                systems: vec![],
            },
        );
        let json = serde_json::to_string(&def).unwrap();
        let parsed: GameDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, def.name);
        assert!(parsed.entities.contains_key("Test"));
    }
}
