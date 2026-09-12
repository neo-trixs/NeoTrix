use nt_world_sim::core::{
    UniversalWorld, UniversalEntity, Component, Resource, Event, ComponentTuple,
};
use nt_world_sim::core::scheduler::{ParallelScheduler, UniversalSystem, SystemDependency};
use nt_world_sim::mechanics::core_pet::{CorePetState, PetStateEnum, CorePetSystem};
use nt_world_sim::mechanics::core_hook::{CoreHookEvent, CoreHookManager};
use nt_world_sim::mechanics::core_theme::{CoreTheme, CoreThemeManager};

#[derive(Clone, Debug, PartialEq)]
struct Position { x: f32, y: f32 }
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct Velocity { x: f32, y: f32 }
impl Component for Velocity {}

#[derive(Clone, Debug)]
struct GameTime { delta: f32, elapsed: f32 }
impl Resource for GameTime {}

#[derive(Clone, Debug)]
struct HitEvent { entity_id: u64 }
impl Event for HitEvent {}

#[derive(Clone, Debug)]
struct DamageEvent { amount: f32 }
impl Event for DamageEvent {}

struct MovementSystem;
impl UniversalSystem for MovementSystem {
    fn name(&self) -> &str { "MovementSystem" }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        let entities: Vec<_> = world.entities();
        for entity in entities {
            let vel = world.get_component::<Velocity>(entity).cloned();
            if let (Some(vel), Some(pos)) = (vel, world.get_component_mut::<Position>(entity)) {
                pos.x += vel.x * dt;
                pos.y += vel.y * dt;
            }
        }
    }
}

struct VelocitySystem;
impl UniversalSystem for VelocitySystem {
    fn name(&self) -> &str { "VelocitySystem" }
    fn update(&mut self, world: &mut UniversalWorld, _dt: f32) {
        let entities: Vec<_> = world.entities();
        for entity in entities {
            if let Some(vel) = world.get_component_mut::<Velocity>(entity) {
                vel.x *= 0.99;
                vel.y *= 0.99;
            }
        }
    }
}

struct EventEmittingSystem;
impl UniversalSystem for EventEmittingSystem {
    fn name(&self) -> &str { "EventEmittingSystem" }
    fn update(&mut self, world: &mut UniversalWorld, _dt: f32) {
        let entities: Vec<_> = world.entities();
        for entity in entities {
            if world.get_component::<Position>(entity).is_some() {
                world.send_event(HitEvent { entity_id: entity.id.0 });
            }
        }
    }
}

struct ResourceTrackingSystem;
impl UniversalSystem for ResourceTrackingSystem {
    fn name(&self) -> &str { "ResourceTrackingSystem" }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        if let Some(time) = world.get_resource_mut::<GameTime>() {
            time.elapsed += dt;
        }
    }
}

// --- Test: Full ECS pipeline ---

#[test]
fn test_ecs_full_pipeline() {
    let mut world = UniversalWorld::new();
    world.insert_resource(GameTime { delta: 0.016, elapsed: 0.0 });

    let e1 = world.spawn();
    world.insert_component(e1, Position { x: 0.0, y: 0.0 });
    world.insert_component(e1, Velocity { x: 10.0, y: 5.0 });

    let e2 = world.spawn();
    world.insert_component(e2, Position { x: 100.0, y: 100.0 });
    world.insert_component(e2, Velocity { x: -1.0, y: 0.0 });

    let e3 = world.spawn();
    world.insert_component(e3, Position { x: 50.0, y: 50.0 });

    assert_eq!(world.entity_count(), 3);

    let moving = world.query::<(Position, Velocity)>();
    assert_eq!(moving.len(), 2);

    let all_pos = world.query::<(Position,)>();
    assert_eq!(all_pos.len(), 3);

    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.delta, 0.016);
}

// --- Test: Scheduler runs systems in order ---

#[test]
fn test_scheduler_runs_systems() {
    let mut world = UniversalWorld::new();
    let e = world.spawn();
    world.insert_component(e, Position { x: 0.0, y: 0.0 });
    world.insert_component(e, Velocity { x: 100.0, y: 0.0 });

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(MovementSystem), SystemDependency::new());
    sched.build_schedule();

    sched.run(&mut world, 0.1);

    let pos = world.get_component::<Position>(e).unwrap();
    assert!((pos.x - 10.0).abs() < 0.01);
    assert!((pos.y).abs() < 0.01);
}

// --- Test: Scheduler with dependency ordering ---

#[test]
fn test_scheduler_dependency_ordering() {
    let mut world = UniversalWorld::new();
    let e = world.spawn();
    world.insert_component(e, Position { x: 0.0, y: 0.0 });
    world.insert_component(e, Velocity { x: 200.0, y: 100.0 });

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(MovementSystem), SystemDependency::new());
    sched.add_system(
        Box::new(VelocitySystem),
        SystemDependency::new().after(vec!["MovementSystem".into()]),
    );
    sched.build_schedule();

    assert!(sched.wave_count() >= 2);

    sched.run(&mut world, 0.1);

    let pos = world.get_component::<Position>(e).unwrap();
    assert!((pos.x - 20.0).abs() < 0.01);

    let vel = world.get_component::<Velocity>(e).unwrap();
    assert!((vel.x - 198.0).abs() < 0.1);
}

// --- Test: Scheduler runs multiple waves ---

#[test]
fn test_scheduler_multiple_waves() {
    let mut world = UniversalWorld::new();

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(MovementSystem), SystemDependency::new());
    sched.add_system(
        Box::new(VelocitySystem),
        SystemDependency::new().after(vec!["MovementSystem".into()]),
    );
    sched.add_system(
        Box::new(ResourceTrackingSystem),
        SystemDependency::new(),
    );
    sched.build_schedule();

    let e = world.spawn();
    world.insert_component(e, Position { x: 0.0, y: 0.0 });
    world.insert_component(e, Velocity { x: 10.0, y: 0.0 });
    world.insert_resource(GameTime { delta: 0.016, elapsed: 0.0 });

    sched.run(&mut world, 0.1);

    let pos = world.get_component::<Position>(e).unwrap();
    assert!((pos.x - 1.0).abs() < 0.01);

    let time = world.get_resource::<GameTime>().unwrap();
    assert!((time.elapsed - 0.1).abs() < 0.001);
}

// --- Test: Pet state transitions idle -> sleeping ---

#[test]
fn test_pet_idle_to_sleeping() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(
        pet,
        CorePetState {
            state: PetStateEnum::Idle,
            state_timer: 0.0,
            idle_timer: 0.0,
        },
    );

    // 3812 * 0.016 = 60.992 > 60.0 threshold
    for _ in 0..3812 {
        CorePetSystem::update(&mut world, 0.016);
    }

    let pet_state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(pet_state.state, PetStateEnum::Sleeping);
}

// --- Test: Pet state priority prevents downgrade ---

#[test]
fn test_pet_priority_no_downgrade() {
    let mut pet = CorePetState::new();
    pet.state = PetStateEnum::Error;
    assert!(!pet.transition(PetStateEnum::Idle));
    assert_eq!(pet.state, PetStateEnum::Error);

    assert!(pet.transition(PetStateEnum::Thinking { duration: 0.0 }));
    assert_eq!(pet.state, PetStateEnum::Thinking { duration: 0.0 });
}

// --- Test: Hook triggers pet thinking on session start ---

#[test]
fn test_hook_triggers_pet_thinking() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(
        pet,
        CorePetState {
            state: PetStateEnum::Idle,
            state_timer: 0.0,
            idle_timer: 0.0,
        },
    );

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::SessionStart {
            agent: "claude".into(),
            session_id: "s1".into(),
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert!(matches!(state.state, PetStateEnum::Thinking { .. }));
}

// --- Test: Hook tool start sets typing for write tools ---

#[test]
fn test_hook_tool_start_typing() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(pet, CorePetState::new());

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolStart {
            tool: "write".into(),
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Typing { progress: 0.0 });
}

// --- Test: Hook tool start sets thinking for read tools ---

#[test]
fn test_hook_tool_start_thinking() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(pet, CorePetState::new());

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolStart {
            tool: "grep".into(),
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert!(matches!(state.state, PetStateEnum::Thinking { .. }));
}

// --- Test: Hook tool end success -> happy ---

#[test]
fn test_hook_tool_end_success() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(pet, CorePetState::new());

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolEnd {
            tool: "bash".into(),
            success: true,
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Happy);
}

// --- Test: Hook tool end failure -> error ---

#[test]
fn test_hook_tool_end_failure() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(pet, CorePetState::new());

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolEnd {
            tool: "bash".into(),
            success: false,
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Error);
}

// --- Test: Hook session end -> idle ---

#[test]
fn test_hook_session_end() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    let mut pet_state = CorePetState::new();
    pet_state.state = PetStateEnum::Thinking { duration: 5.0 };
    world.insert_component(pet, pet_state);

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::SessionEnd {
            agent: "test".into(),
            session_id: "s1".into(),
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Idle);
    assert_eq!(state.idle_timer, 0.0);
}

// --- Test: Hook permission request -> notification ---

#[test]
fn test_hook_permission_request() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(pet, CorePetState::new());

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::PermissionRequest {
            tool: "bash".into(),
            request_id: "r1".into(),
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Notification);
}

// --- Test: Theme default loaded ---

#[test]
fn test_theme_default_loaded() {
    let mut world = UniversalWorld::new();
    CoreThemeManager::load_default_theme(&mut world);

    let path = CoreThemeManager::get_animation_path(&world, "idle").unwrap();
    assert_eq!(path, "idle.gif");

    let thinking = CoreThemeManager::get_animation_path(&world, "thinking").unwrap();
    assert_eq!(thinking, "thinking.gif");

    let missing = CoreThemeManager::get_animation_path(&world, "nonexistent");
    assert_eq!(missing, None);
}

// --- Test: Theme custom loading ---

#[test]
fn test_theme_custom_loading() {
    let mut world = UniversalWorld::new();
    CoreThemeManager::load_theme(
        &mut world,
        "Dark",
        vec![("idle", "dark_idle.gif"), ("sleeping", "dark_sleep.gif")],
    );

    let theme = world.get_resource::<CoreTheme>().unwrap();
    assert_eq!(theme.name, "Dark");
    assert_eq!(theme.states.len(), 2);

    let path = CoreThemeManager::get_animation_path(&world, "idle").unwrap();
    assert_eq!(path, "dark_idle.gif");
}

// --- Test: Theme replacement ---

#[test]
fn test_theme_replacement() {
    let mut world = UniversalWorld::new();
    CoreThemeManager::load_default_theme(&mut world);

    let custom = CoreTheme::new("Minimal")
        .with_state("idle", "min_idle.gif")
        .with_state("sleeping", "min_sleep.gif");
    CoreThemeManager::replace_theme(&mut world, custom);

    let theme = world.get_resource::<CoreTheme>().unwrap();
    assert_eq!(theme.name, "Minimal");
    assert_eq!(theme.states.len(), 2);
}

// --- Test: Event roundtrip with drain ---

#[test]
fn test_event_roundtrip_drain() {
    let mut world = UniversalWorld::new();
    world.send_event(HitEvent { entity_id: 42 });
    world.send_event(HitEvent { entity_id: 99 });
    world.send_event(DamageEvent { amount: 25.0 });

    let hits = world.receive_events::<HitEvent>();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].entity_id, 42);
    assert_eq!(hits[1].entity_id, 99);

    let hits2 = world.receive_events::<HitEvent>();
    assert_eq!(hits2.len(), 0);

    let dmg = world.receive_events::<DamageEvent>();
    assert_eq!(dmg.len(), 1);
    assert_eq!(dmg[0].amount, 25.0);
}

// --- Test: Entity spawn and despawn ---

#[test]
fn test_entity_spawn_despawn() {
    let mut world = UniversalWorld::new();
    let e1 = world.spawn();
    world.insert_component(e1, Position { x: 1.0, y: 1.0 });
    let e2 = world.spawn();
    world.insert_component(e2, Position { x: 2.0, y: 2.0 });

    assert_eq!(world.entity_count(), 2);

    world.despawn(e1);
    assert_eq!(world.entity_count(), 1);

    let remaining = world.query::<(Position,)>();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].1.x, 2.0);
}

// --- Test: Component removal ---

#[test]
fn test_component_removal() {
    let mut world = UniversalWorld::new();
    let e = world.spawn();
    world.insert_component(e, Position { x: 5.0, y: 5.0 });
    world.insert_component(e, Velocity { x: 1.0, y: 1.0 });

    assert!(world.has_component::<Velocity>(e));

    let removed = world.remove_component::<Velocity>(e).unwrap();
    assert_eq!(removed.x, 1.0);

    assert!(!world.has_component::<Velocity>(e));
    assert!(world.has_component::<Position>(e));

    let with_pos = world.query::<(Position,)>();
    assert_eq!(with_pos.len(), 1);

    let with_vel = world.query::<(Velocity,)>();
    assert_eq!(with_vel.len(), 0);
}

// --- Test: Resource mutation ---

#[test]
fn test_resource_mutation() {
    let mut world = UniversalWorld::new();
    world.insert_resource(GameTime { delta: 0.016, elapsed: 0.0 });

    {
        let time = world.get_resource_mut::<GameTime>().unwrap();
        time.elapsed += 1.0;
    }

    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.elapsed, 1.0);
}

// --- Test: Event emission from system ---

#[test]
fn test_event_emission_from_system() {
    let mut world = UniversalWorld::new();
    let e = world.spawn();
    world.insert_component(e, Position { x: 0.0, y: 0.0 });

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(EventEmittingSystem), SystemDependency::new());
    sched.build_schedule();
    sched.run(&mut world, 0.016);

    let events = world.receive_events::<HitEvent>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity_id, e.id.0);
}

// --- Test: Scheduler clears ---

#[test]
fn test_scheduler_clear() {
    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(MovementSystem), SystemDependency::new());
    sched.add_system(Box::new(VelocitySystem), SystemDependency::new());
    sched.build_schedule();

    assert_eq!(sched.system_count(), 2);
    assert!(sched.wave_count() >= 1);

    sched.clear();
    assert_eq!(sched.system_count(), 0);
}

// --- Test: Pet Happy -> Idle timeout ---

#[test]
fn test_pet_happy_timeout() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(
        pet,
        CorePetState {
            state: PetStateEnum::Happy,
            state_timer: 2.5,
            idle_timer: 0.0,
        },
    );

    CorePetSystem::update(&mut world, 1.0);

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Idle);
}

// --- Test: Pet typing progress increments ---

#[test]
fn test_pet_typing_progress() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(
        pet,
        CorePetState {
            state: PetStateEnum::Typing { progress: 0.0 },
            state_timer: 0.0,
            idle_timer: 0.0,
        },
    );

    CorePetSystem::update(&mut world, 1.0);

    let state = world.get_component::<CorePetState>(pet).unwrap();
    match &state.state {
        PetStateEnum::Typing { progress } => {
            assert!(*progress > 0.0);
            assert!(*progress <= 1.0);
        }
        _ => panic!("Expected Typing state"),
    }
}

// --- Test: Full integration hook -> pet -> theme ---

#[test]
fn test_full_hook_pet_theme_integration() {
    let mut world = UniversalWorld::new();

    // Load theme
    CoreThemeManager::load_default_theme(&mut world);

    // Spawn pet
    let pet = world.spawn();
    world.insert_component(pet, CorePetState::new());

    // Session starts -> pet goes to Thinking
    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::SessionStart {
            agent: "claude".into(),
            session_id: "test-session".into(),
        },
    );
    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert!(matches!(state.state, PetStateEnum::Thinking { .. }));

    // Tool "write" starts -> pet goes to Typing
    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolStart {
            tool: "write".into(),
        },
    );
    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Typing { progress: 0.0 });

    // Tool ends success -> pet goes to Happy
    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolEnd {
            tool: "write".into(),
            success: true,
        },
    );
    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Happy);

    // Theme should still work
    let path = CoreThemeManager::get_animation_path(&world, "happy").unwrap();
    assert_eq!(path, "happy.gif");
}

// --- Test: Multi-entity pet hook propagation ---

#[test]
fn test_multi_entity_hook_propagation() {
    let mut world = UniversalWorld::new();

    let pet1 = world.spawn();
    world.insert_component(pet1, CorePetState::new());

    let pet2 = world.spawn();
    world.insert_component(pet2, CorePetState::new());

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::SessionStart {
            agent: "claude".into(),
            session_id: "s1".into(),
        },
    );

    let s1 = world.get_component::<CorePetState>(pet1).unwrap();
    assert!(matches!(s1.state, PetStateEnum::Thinking { .. }));

    let s2 = world.get_component::<CorePetState>(pet2).unwrap();
    assert!(matches!(s2.state, PetStateEnum::Thinking { .. }));
}

// --- Test: Empty world operations ---

#[test]
fn test_empty_world() {
    let mut world = UniversalWorld::new();
    assert_eq!(world.entity_count(), 0);

    let all = world.query::<(Position,)>();
    assert_eq!(all.len(), 0);

    let events = world.receive_events::<HitEvent>();
    assert_eq!(events.len(), 0);

    assert!(world.get_resource::<GameTime>().is_none());
}
