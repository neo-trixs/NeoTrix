use nt_world_sim::core::{UniversalWorld, Component, Resource};
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

struct VelocityDampingSystem;
impl UniversalSystem for VelocityDampingSystem {
    fn name(&self) -> &str { "VelocityDampingSystem" }
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

struct ResourceTrackingSystem;
impl UniversalSystem for ResourceTrackingSystem {
    fn name(&self) -> &str { "ResourceTrackingSystem" }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        if let Some(time) = world.get_resource_mut::<GameTime>() {
            time.elapsed += dt;
        }
    }
}

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

#[test]
fn test_scheduler_dependency_ordering() {
    let mut world = UniversalWorld::new();
    let e = world.spawn();
    world.insert_component(e, Position { x: 0.0, y: 0.0 });
    world.insert_component(e, Velocity { x: 200.0, y: 100.0 });

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(MovementSystem), SystemDependency::new());
    sched.add_system(
        Box::new(VelocityDampingSystem),
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

#[test]
fn test_scheduler_multiple_waves() {
    let mut world = UniversalWorld::new();

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(MovementSystem), SystemDependency::new());
    sched.add_system(
        Box::new(VelocityDampingSystem),
        SystemDependency::new().after(vec!["MovementSystem".into()]),
    );
    sched.add_system(Box::new(ResourceTrackingSystem), SystemDependency::new());
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

    for _ in 0..3812 {
        CorePetSystem::update(&mut world, 0.016);
    }

    let pet_state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(pet_state.state, PetStateEnum::Sleeping);
}

#[test]
fn test_pet_priority_no_downgrade() {
    let mut pet = CorePetState::new();
    assert_eq!(pet.state, PetStateEnum::Idle);

    pet.state = PetStateEnum::Error;
    assert!(!pet.transition(PetStateEnum::Idle));
    assert_eq!(pet.state, PetStateEnum::Error);

    assert!(!pet.transition(PetStateEnum::Thinking { duration: 0.0 }));
    assert_eq!(pet.state, PetStateEnum::Error);

    assert!(pet.transition(PetStateEnum::Error));
    assert_eq!(pet.state, PetStateEnum::Error);
}

#[test]
fn test_pet_priority_can_upgrade() {
    let mut pet = CorePetState::new();
    assert!(pet.transition(PetStateEnum::Carrying));
    assert!(pet.transition(PetStateEnum::Happy));
    assert!(pet.transition(PetStateEnum::Building));
    assert!(pet.transition(PetStateEnum::Typing { progress: 0.0 }));
    assert!(pet.transition(PetStateEnum::Thinking { duration: 0.0 }));
    assert!(pet.transition(PetStateEnum::Error));
}

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

#[test]
fn test_hook_permission_response() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    let mut pet_state = CorePetState::new();
    pet_state.state = PetStateEnum::Notification;
    world.insert_component(pet, pet_state);

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::PermissionResponse {
            request_id: "r1".into(),
            approved: true,
        },
    );

    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Idle);
}

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

#[test]
fn test_theme_all_states() {
    let mut world = UniversalWorld::new();
    CoreThemeManager::load_default_theme(&mut world);

    let states = [
        "idle", "thinking", "typing", "building", "groove",
        "juggling", "error", "happy", "notification",
        "sweeping", "carrying", "sleeping",
    ];
    for state in &states {
        let path = CoreThemeManager::get_animation_path(&world, state);
        assert!(path.is_some(), "Missing state: {}", state);
    }
}

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

#[test]
fn test_scheduler_clear() {
    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(MovementSystem), SystemDependency::new());
    sched.add_system(Box::new(VelocityDampingSystem), SystemDependency::new());
    sched.build_schedule();

    assert_eq!(sched.system_count(), 2);
    assert!(sched.wave_count() >= 1);

    sched.clear();
    assert_eq!(sched.system_count(), 0);
}

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

#[test]
fn test_pet_thinking_increments_duration() {
    let mut world = UniversalWorld::new();
    let pet = world.spawn();
    world.insert_component(
        pet,
        CorePetState {
            state: PetStateEnum::Thinking { duration: 0.0 },
            state_timer: 0.0,
            idle_timer: 0.0,
        },
    );

    CorePetSystem::update(&mut world, 1.5);

    let state = world.get_component::<CorePetState>(pet).unwrap();
    match &state.state {
        PetStateEnum::Thinking { duration } => {
            assert!((*duration - 1.5).abs() < 0.01);
        }
        _ => panic!("Expected Thinking state"),
    }
}

#[test]
fn test_full_hook_pet_theme_integration() {
    let mut world = UniversalWorld::new();

    CoreThemeManager::load_default_theme(&mut world);

    let pet = world.spawn();
    world.insert_component(pet, CorePetState::new());

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::SessionStart {
            agent: "claude".into(),
            session_id: "test-session".into(),
        },
    );
    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert!(matches!(state.state, PetStateEnum::Thinking { .. }));

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolStart {
            tool: "write".into(),
        },
    );
    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Typing { progress: 0.0 });

    CoreHookManager::process_event(
        &mut world,
        CoreHookEvent::ToolEnd {
            tool: "write".into(),
            success: true,
        },
    );
    let state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(state.state, PetStateEnum::Happy);

    let path = CoreThemeManager::get_animation_path(&world, "happy").unwrap();
    assert_eq!(path, "happy.gif");
}

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

#[test]
fn test_empty_world() {
    let world = UniversalWorld::new();
    assert_eq!(world.entity_count(), 0);

    let all = world.query::<(Position,)>();
    assert_eq!(all.len(), 0);

    assert!(world.get_resource::<GameTime>().is_none());
}

#[test]
fn test_scheduler_disabled_system_skipped() {
    struct DisabledSys;
    impl UniversalSystem for DisabledSys {
        fn name(&self) -> &str { "DisabledSys" }
        fn enabled(&self) -> bool { false }
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {
            panic!("Should not be called");
        }
    }

    let mut world = UniversalWorld::new();
    let e = world.spawn();
    world.insert_component(e, Position { x: 0.0, y: 0.0 });
    world.insert_component(e, Velocity { x: 100.0, y: 0.0 });

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(DisabledSys), SystemDependency::new());
    sched.build_schedule();
    sched.run(&mut world, 1.0);

    let pos = world.get_component::<Position>(e).unwrap();
    assert_eq!(pos.x, 0.0);
}

#[test]
fn test_scheduler_system_priority() {
    struct HighPrioritySys;
    impl UniversalSystem for HighPrioritySys {
        fn name(&self) -> &str { "HighPrioritySys" }
        fn priority(&self) -> i32 { 100 }
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {}
    }

    let mut sched = ParallelScheduler::new();
    sched.add_system(Box::new(HighPrioritySys), SystemDependency::new());
    sched.build_schedule();
    assert_eq!(sched.system_count(), 1);
}

#[test]
fn test_pet_all_state_priorities() {
    let priorities: Vec<(PetStateEnum, i32)> = vec![
        (PetStateEnum::Sleeping, 0),
        (PetStateEnum::Idle, 10),
        (PetStateEnum::Carrying, 25),
        (PetStateEnum::Sweeping, 30),
        (PetStateEnum::Happy, 40),
        (PetStateEnum::Groove, 50),
        (PetStateEnum::Juggling, 50),
        (PetStateEnum::Building, 60),
        (PetStateEnum::Typing { progress: 0.0 }, 70),
        (PetStateEnum::Thinking { duration: 0.0 }, 80),
        (PetStateEnum::Notification, 90),
        (PetStateEnum::Error, 100),
    ];

    for (state, expected_priority) in priorities {
        let pet = CorePetState {
            state,
            state_timer: 0.0,
            idle_timer: 0.0,
        };
        assert_eq!(pet.priority(), expected_priority);
    }
}
