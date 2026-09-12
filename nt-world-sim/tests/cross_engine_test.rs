#[cfg(test)]
mod tests {
    use nt_world_sim::core::{
        UniversalWorld, Component, Resource, Event,
        ParallelScheduler, SystemDependency, UniversalSystem,
    };
    use nt_world_sim::adapters::bevy_adapter::{BevyAdapter, ExternalEntity};
    use nt_world_sim::adapters::unity_adapter::UnityAdapter;
    use nt_world_sim::codegen::{
        GameDefinition, EntityDef, SystemDef, EngineConfig,
        CodeGenerator, GameDefParser,
    };
    use std::collections::HashMap;
    use std::path::Path;

    // ── Shared test components ──────────────────────────────────────────

    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Clone, Debug, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Clone, Debug, PartialEq)]
    struct Health {
        value: f32,
    }
    impl Component for Health {}

    #[derive(Clone, Debug, PartialEq)]
    struct Damage {
        amount: f32,
    }
    impl Component for Damage {}

    struct Gravity {
        force: f32,
    }
    impl Resource for Gravity {}

    struct CollisionEvent {
        entity_a: u64,
        entity_b: u64,
    }
    impl Event for CollisionEvent {}

    // ── Test 1: ECS lifecycle ───────────────────────────────────────────

    #[test]
    fn test_ecs_lifecycle() {
        let mut world = UniversalWorld::new();

        // Spawn entities
        let player = world.spawn();
        let enemy = world.spawn();
        assert_eq!(world.entity_count(), 2);

        // Insert components
        world.insert_component(player, Position { x: 10.0, y: 20.0 });
        world.insert_component(player, Velocity { x: 1.0, y: 0.5 });
        world.insert_component(player, Health { value: 100.0 });

        world.insert_component(enemy, Position { x: 50.0, y: 50.0 });
        world.insert_component(enemy, Damage { amount: 25.0 });

        // Query single component
        let positions = world.query::<(Position,)>();
        assert_eq!(positions.len(), 2);

        // Query two components — only player matches
        let movers = world.query::<(Position, Velocity)>();
        assert_eq!(movers.len(), 1);
        let (_e, (pos, vel)) = &movers[0];
        assert_eq!(*pos, Position { x: 10.0, y: 20.0 });
        assert_eq!(*vel, Velocity { x: 1.0, y: 0.5 });

        // Get component individually
        let hp = world.get_component::<Health>(player).unwrap();
        assert_eq!(hp.value, 100.0);

        // Mutate component
        {
            let hp_mut = world.get_component_mut::<Health>(player).unwrap();
            hp_mut.value -= 30.0;
        }
        assert_eq!(world.get_component::<Health>(player).unwrap().value, 70.0);

        // Remove component
        let removed = world.remove_component::<Velocity>(player);
        assert!(removed.is_some());
        assert!(!world.has_component::<Velocity>(player));
        assert!(world.has_component::<Position>(player));

        // Resources
        world.insert_resource(Gravity { force: 9.81 });
        let g = world.get_resource::<Gravity>().unwrap();
        assert!((g.force - 9.81).abs() < f32::EPSILON);

        // Events
        world.send_event(CollisionEvent { entity_a: 0, entity_b: 1 });
        let events = world.receive_events::<CollisionEvent>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].entity_a, 0);

        // Despawn
        assert!(world.despawn(enemy));
        assert_eq!(world.entity_count(), 1);
        assert!(!world.despawn(enemy)); // double despawn returns false

        // Entity list only returns living entities
        let remaining = world.entities();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id.0, player.id.0);
    }

    // ── Test 2: Scheduler parallel waves ────────────────────────────────

    struct PhysicsSystem;
    impl UniversalSystem for PhysicsSystem {
        fn name(&self) -> &str { "physics" }
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {}
    }

    struct AiSystem;
    impl UniversalSystem for AiSystem {
        fn name(&self) -> &str { "ai" }
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {}
    }

    struct RenderSystem;
    impl UniversalSystem for RenderSystem {
        fn name(&self) -> &str { "render" }
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {}
    }

    #[test]
    fn test_scheduler_parallel_waves() {
        let mut sched = ParallelScheduler::new();

        // physics has no deps → wave 0
        sched.add_system(
            Box::new(PhysicsSystem),
            SystemDependency::new(),
        );

        // ai depends on physics → wave 1
        sched.add_system(
            Box::new(AiSystem),
            SystemDependency::new().after(vec!["physics".into()]),
        );

        // render depends on ai → wave 2
        sched.add_system(
            Box::new(RenderSystem),
            SystemDependency::new().after(vec!["ai".into()]),
        );

        sched.build_schedule();

        assert_eq!(sched.system_count(), 3);
        assert!(
            sched.wave_count() >= 2,
            "Expected at least 2 waves, got {}",
            sched.wave_count()
        );

        // Run should execute without panicking
        let mut world = UniversalWorld::new();
        sched.run(&mut world, 0.016);
    }

    // ── Test 3: Bevy adapter roundtrip ──────────────────────────────────

    struct FakeBevyEntity {
        idx: u32,
        gen: u32,
    }
    impl ExternalEntity for FakeBevyEntity {
        fn index(&self) -> u32 { self.idx }
        fn generation(&self) -> u32 { self.gen }
    }

    #[test]
    fn test_bevy_adapter_roundtrip() {
        // Bevy → Universal
        let bevy_entity = FakeBevyEntity { idx: 42, gen: 7 };
        let universal = BevyAdapter::from_external_entity(&bevy_entity);
        assert_eq!(universal.id.0, 42);
        assert_eq!(universal.generation, 7);

        // Universal → Bevy (id, generation)
        let (idx, gen) = BevyAdapter::to_external_id(universal);
        assert_eq!(idx, 42);
        assert_eq!(gen, 7);

        // Full roundtrip with world round-trip
        let mut world = UniversalWorld::new();
        let e = world.spawn();
        world.insert_component(e, Position { x: 3.0, y: 4.0 });

        let (out_idx, out_gen) = BevyAdapter::to_external_id(e);
        let back = BevyAdapter::from_external_entity(&FakeBevyEntity {
            idx: out_idx,
            gen: out_gen,
        });

        // Entity ID preserved through conversion
        assert_eq!(back.id, e.id);
        assert_eq!(back.generation, e.generation);
    }

    // ── Test 4: Unity adapter roundtrip ─────────────────────────────────

    #[test]
    fn test_unity_adapter_roundtrip() {
        // Unity → Universal
        let entity = UnityAdapter::from_unity_entity(10, 5);
        assert_eq!(entity.id.0, 10);
        assert_eq!(entity.generation, 5);

        // Universal → Unity
        let (index, gen) = UnityAdapter::to_unity_entity(entity);
        assert_eq!(index, 10);
        assert_eq!(gen, 5);

        // Full roundtrip
        let original = (99i32, 12i32);
        let converted = UnityAdapter::from_unity_entity(original.0, original.1);
        let result = UnityAdapter::to_unity_entity(converted);
        assert_eq!(result, original);

        // Query string generation
        let required = vec![std::any::TypeId::of::<i32>()];
        let excluded = vec![std::any::TypeId::of::<f64>()];
        let query = UnityAdapter::to_unity_query(&required, &excluded);
        assert!(query.starts_with("UnityQuery("));
        assert!(query.contains("-"));
    }

    // ── Test 5: YAML game definition parse ──────────────────────────────

    #[test]
    fn test_game_def_parse_yaml() {
        let yaml = r#"
name: CrossEngineTest
version: "2.0.0"
engine:
  target: bevy
  features:
    - 2d
    - physics
entities:
  Player:
    components:
      - health
      - speed
      - position
    systems:
      - movement
      - input
  Enemy:
    components:
      - damage
      - position
    systems:
      - ai_chase
systems:
  movement:
    priority: 10
    read:
      - position
    write:
      - velocity
  ai_chase:
    priority: 5
    read:
      - player_position
    write:
      - enemy_velocity
resources:
  GameTime:
    fields:
      delta: f32
      elapsed: f64
"#;
        let path = Path::new("/tmp/nt_world_sim_cross_engine_test.yaml");
        std::fs::write(path, yaml).expect("Failed to write YAML");

        let def = GameDefParser::parse_yaml(path).unwrap();

        // Verify top-level fields
        assert_eq!(def.name, "CrossEngineTest");
        assert_eq!(def.version, "2.0.0");
        assert_eq!(def.engine.target, "bevy");
        assert_eq!(def.engine.features, vec!["2d", "physics"]);

        // Verify entities
        assert_eq!(def.entities.len(), 2);
        let player = def.entities.get("Player").unwrap();
        assert_eq!(player.components, vec!["health", "speed", "position"]);
        assert_eq!(player.systems, vec!["movement", "input"]);

        let enemy = def.entities.get("Enemy").unwrap();
        assert_eq!(enemy.components, vec!["damage", "position"]);

        // Verify systems
        assert_eq!(def.systems.len(), 2);
        let movement = def.systems.get("movement").unwrap();
        assert_eq!(movement.priority, 10);
        assert_eq!(movement.read, vec!["position"]);
        assert_eq!(movement.write, vec!["velocity"]);

        // Verify resources
        assert_eq!(def.resources.len(), 1);
        let gt = def.resources.get("GameTime").unwrap();
        assert_eq!(gt.fields.get("delta").unwrap(), "f32");
        assert_eq!(gt.fields.get("elapsed").unwrap(), "f64");

        let _ = std::fs::remove_file(path);
    }

    // ── Test 6: Code generator all targets ──────────────────────────────

    fn build_test_game_def() -> GameDefinition {
        let mut entities = HashMap::new();
        entities.insert(
            "Player".to_string(),
            EntityDef {
                components: vec!["health".to_string(), "speed".to_string()],
                systems: vec!["movement".to_string()],
            },
        );

        let mut systems = HashMap::new();
        systems.insert(
            "movement".to_string(),
            SystemDef {
                priority: 10,
                read: vec!["position".to_string()],
                write: vec!["velocity".to_string()],
            },
        );

        GameDefinition {
            name: "AllTargetTest".to_string(),
            version: "1.0.0".to_string(),
            engine: EngineConfig {
                target: "bevy".to_string(),
                features: vec!["2d".to_string()],
            },
            entities,
            systems,
            resources: HashMap::new(),
        }
    }

    #[test]
    fn test_code_generator_all_targets() {
        let mut def = build_test_game_def();

        // Bevy
        def.engine.target = "bevy".to_string();
        let gen = CodeGenerator::new(def.clone());
        let bevy_code = gen.generate_bevy();
        assert!(bevy_code.contains("struct Player"), "Bevy code missing 'struct Player'");
        assert!(bevy_code.contains("health: f32"), "Bevy code missing 'health: f32'");
        assert!(bevy_code.contains("fn movement_system"), "Bevy code missing 'fn movement_system'");
        assert!(bevy_code.contains("use bevy::prelude"), "Bevy code missing import");

        // Unity
        def.engine.target = "unity".to_string();
        let gen = CodeGenerator::new(def.clone());
        let unity_code = gen.generate_unity();
        assert!(unity_code.contains("struct Player : IComponentData"), "Unity code missing IComponentData");
        assert!(unity_code.contains("public float health"), "Unity code missing 'public float health'");
        assert!(unity_code.contains("using Unity.Entities"), "Unity code missing using statement");

        // Godot
        def.engine.target = "godot".to_string();
        let gen = CodeGenerator::new(def.clone());
        let godot_code = gen.generate_godot();
        assert!(godot_code.contains("class_name Player"), "Godot code missing 'class_name Player'");
        assert!(godot_code.contains("func _ready()"), "Godot code missing '_ready()'");
        assert!(godot_code.contains("extends Node"), "Godot code missing 'extends Node'");

        // Also verify generate() dispatches correctly
        def.engine.target = "bevy".to_string();
        let gen = CodeGenerator::new(def);
        let dispatched = gen.generate();
        assert!(dispatched.contains("struct Player"));
    }
}
