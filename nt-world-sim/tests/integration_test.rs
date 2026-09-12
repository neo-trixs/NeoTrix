use nt_world_sim::core::{
    UniversalWorld, UniversalEntity, Component, Resource, Event, ComponentTuple,
};
use nt_world_sim::core::scheduler::{ParallelScheduler, UniversalSystem, SystemDependency};
use nt_world_sim::mechanics::core_pet::{CorePetState, PetStateEnum, CorePetSystem};
use nt_world_sim::mechanics::core_hook::{CoreHookEvent, CoreHookManager};
use nt_world_sim::mechanics::core_theme::{CoreTheme, CoreThemeManager};

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

#[derive(Clone, Debug)]
struct GameTime {
    delta: f32,
    elapsed: f32,
}

impl Resource for GameTime {}

#[derive(Clone, Debug)]
struct CollisionEvent {
    entity_a: u64,
    entity_b: u64,
}

impl Event for CollisionEvent {}

struct MovementSystem;

impl UniversalSystem for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }

    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        let _ = world.get_resource::<GameTime>();
        let entities: Vec<_> = world.entities();
        for entity in entities {
            let vel = world.get_component::<Velocity>(entity).cloned();
            if let (Some(vel), Some(pos)) = (
                vel,
                world.get_component_mut::<Position>(entity),
            ) {
                pos.x += vel.x * dt;
                pos.y += vel.y * dt;
            }
        }
    }
}

#[test]
fn test_full_ecs_pipeline() {
    let mut world = UniversalWorld::new();

    world.insert_resource(GameTime {
        delta: 0.016,
        elapsed: 0.0,
    });

    let player = world.spawn();
    world.insert_component(player, Position { x: 0.0, y: 0.0 });
    world.insert_component(player, Velocity { x: 10.0, y: 5.0 });

    let npc = world.spawn();
    world.insert_component(npc, Position { x: 100.0, y: 100.0 });
    world.insert_component(npc, Velocity { x: -1.0, y: 0.0 });

    let moving = world.query::<(Position, Velocity)>();
    assert_eq!(moving.len(), 2);

    let positioned = world.query::<(Position,)>();
    assert_eq!(positioned.len(), 2);

    let pos = world.get_component::<Position>(player).unwrap();
    assert_eq!(pos.x, 0.0);
}

#[test]
fn test_scheduler_execution() {
    let mut world = UniversalWorld::new();
    world.insert_resource(GameTime {
        delta: 0.016,
        elapsed: 0.0,
    });

    let e = world.spawn();
    world.insert_component(e, Position { x: 0.0, y: 0.0 });
    world.insert_component(e, Velocity { x: 10.0, y: 0.0 });

    let mut scheduler = ParallelScheduler::new();
    scheduler.add_system(
        Box::new(MovementSystem),
        SystemDependency::new().reads(vec![]).writes(vec![]),
    );

    scheduler.build_schedule();
    assert!(scheduler.wave_count() >= 1);

    scheduler.run(&mut world, 0.016);

    let pos = world.get_component::<Position>(e).unwrap();
    assert!((pos.x - 0.16).abs() < 0.01);
}

#[test]
fn test_pet_state_system() {
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

    // 3812 * 0.016 = 60.992 seconds > 60.0 threshold
    for _ in 0..3812 {
        CorePetSystem::update(&mut world, 0.016);
    }

    let pet_state = world.get_component::<CorePetState>(pet).unwrap();
    assert_eq!(pet_state.state, PetStateEnum::Sleeping);
}

#[test]
fn test_hook_event_processing() {
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
            agent: "claude".to_string(),
            session_id: "test".to_string(),
        },
    );

    let pet_state = world.get_component::<CorePetState>(pet).unwrap();
    assert!(matches!(pet_state.state, PetStateEnum::Thinking { .. }));
}

#[test]
fn test_theme_resource() {
    let mut world = UniversalWorld::new();
    CoreThemeManager::load_default_theme(&mut world);

    let path = CoreThemeManager::get_animation_path(&world, "idle").unwrap();
    assert_eq!(path, "idle.gif");
}

#[test]
fn test_event_send_receive() {
    let mut world = UniversalWorld::new();
    world.send_event(CollisionEvent {
        entity_a: 1,
        entity_b: 2,
    });

    let events = world.receive_events::<CollisionEvent>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity_a, 1);
}
