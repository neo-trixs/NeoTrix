use nt_world_sim::core::{UniversalWorld, Component, Resource};
use nt_world_sim::core::scheduler::{ParallelScheduler, UniversalSystem, SystemDependency};

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
