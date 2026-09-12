pub mod adapters;
pub mod builder;
pub mod codegen;
pub mod core;
pub mod ecs;
pub mod engine;
pub mod mechanics;

pub use ecs::{Entity, World, System, SystemScheduler};
pub use ecs::world::Component;
pub use engine::{
    Color, Vec2, Rect, Transform, Sprite, TileDef, TileMap, Camera, Renderer, CanvasRenderer,
    BodyType, RigidBody, Collider, CollisionInfo, PhysicsWorld, SimplePhysicsWorld,
    KeyCode, MouseButton, GamepadAxis, GamepadButton, InputState, InputProvider, SimpleInputProvider,
    Event, EventHandler, EventBus, MoveEvent, CollisionEvent, InputEvent, GameEvent,
    PetState, PetStateComponent, PetAnimation, EyeTracking, PermissionBubble,
    PermissionAction, SessionInfo, SubagentInfo, ZzzParticle,
    PetStateSystem, EyeTrackingSystem, PermissionBubbleSystem, SessionSystem,
    HookEvent, HookConfig, HookManager, PermissionMode, PermissionRequest,
    PermissionBubbleLayout, PermissionHotkeys, hook_event_to_pet_state,
    ThemeConfig, AnimationDef, ShadowDef, ThemeManager, ThemeVariant,
    SpriteSheet, AnimationPlayer,
};
pub use engine::scene::{SceneGraph, SceneNode};
pub use engine::event_bus::{TypedEventBus, GameEventHandler};
pub use engine::audio::{AudioManager, AudioBackend, AudioHandle, StubAudioBackend};
pub use mechanics::{
    ConsciousnessEntity, TransformComponent, RenderComponent, AiComponent, MaslowNeeds,
    SocialRelationship, EconomyComponent,
    ConsciousnessSystem, MaslowSystem, AiSystem,
};

/// 游戏引擎
pub struct GameEngine {
    pub world: World,
    pub scheduler: SystemScheduler,
    pub renderer: Box<dyn Renderer>,
    pub physics: SimplePhysicsWorld,
    pub input: SimpleInputProvider,
    pub events: EventBus,
}

impl GameEngine {
    pub fn new(renderer: Box<dyn Renderer>) -> Self {
        Self {
            world: World::new(),
            scheduler: SystemScheduler::new(),
            renderer,
            physics: SimplePhysicsWorld::new(),
            input: SimpleInputProvider::new(),
            events: EventBus::new(),
        }
    }

    /// 更新游戏状态
    pub fn update(&mut self, dt: f32) {
        // 更新输入
        self.input.update();

        // 运行系统
        self.scheduler.run(&mut self.world, dt);

        // 步进物理
        self.physics.step(dt);

        // 处理事件
        self.events.process(&mut self.world);
    }

    /// 渲染游戏
    pub fn render(&mut self) {
        self.renderer.clear(Color::rgb(0.1, 0.1, 0.1));
        // TODO: 渲染所有实体
        self.renderer.present();
    }

    /// 创建实体
    pub fn spawn(&mut self) -> Entity {
        self.world.spawn()
    }

    /// 添加组件
    pub fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        self.world.insert_component(entity, component);
    }

    /// 获取组件
    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        self.world.get_component(entity)
    }

    /// 获取可变组件
    pub fn get_component_mut<T: Component + 'static>(
        &mut self,
        entity: Entity,
    ) -> Option<&mut T> {
        self.world.get_component_mut(entity)
    }

    /// 添加系统
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.scheduler.add_system(system);
    }
}

/// 快速创建星谷物语风格游戏
pub fn create_stardew_valley_game() -> GameEngine {
    // 创建渲染器
    let renderer = Box::new(CanvasRenderer::new(800.0, 600.0));
    let mut engine = GameEngine::new(renderer);

    // 添加系统
    engine.add_system(Box::new(MaslowSystem));
    engine.add_system(Box::new(AiSystem));
    engine.add_system(Box::new(ConsciousnessSystem));
    
    // 添加宠物状态系统
    engine.add_system(Box::new(PetStateSystem));
    engine.add_system(Box::new(EyeTrackingSystem));
    engine.add_system(Box::new(PermissionBubbleSystem));
    engine.add_system(Box::new(SessionSystem));

    // 创建玩家实体
    let player = engine.spawn();
    engine.add_component(
        player,
        ConsciousnessEntity::new("core"),
    );
    engine.add_component(
        player,
        TransformComponent::new(Vec2::new(300.0, 300.0)),
    );
    engine.add_component(
        player,
        RenderComponent::new("🧠", "#1565c0"),
    );
    engine.add_component(player, MaslowNeeds::new());
    engine.add_component(
        player,
        EconomyComponent::new(100),
    );

    // 创建其他意识实体
    let domains = [
        ("mind", "🔮", "#6a1b9a"),
        ("memory", "📚", "#2e7d32"),
        ("world", "🌍", "#e65100"),
        ("act", "⚔️", "#b71c1c"),
        ("io", "📡", "#00838f"),
        ("feel", "💖", "#ad1457"),
    ];

    for (i, (domain, icon, color)) in domains.iter().enumerate() {
        let entity = engine.spawn();
        engine.add_component(entity, ConsciousnessEntity::new(domain));
        engine.add_component(
            entity,
            TransformComponent::new(Vec2::new(
                300.0 + (i as f32 * 100.0).cos() * 100.0,
                300.0 + (i as f32 * 100.0).sin() * 100.0,
            )),
        );
        engine.add_component(entity, RenderComponent::new(icon, color));
        engine.add_component(entity, MaslowNeeds::new());
        engine.add_component(entity, AiComponent::new());
    }

    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::{CodeGenerator, EntityDef, GameDefinition, SystemDef};

    #[test]
    fn test_ecs() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert_component(entity, TransformComponent::new(Vec2::new(0.0, 0.0)));
        assert!(world.has_component::<TransformComponent>(entity));
    }

    #[test]
    fn test_physics() {
        let mut physics = SimplePhysicsWorld::new();
        let entity = Entity::new(0, 0);
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
    fn test_maslow() {
        let mut needs = MaslowNeeds::new();
        needs.satisfy("physiological", 0.1);
        assert!(needs.physiological < 0.2);
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
