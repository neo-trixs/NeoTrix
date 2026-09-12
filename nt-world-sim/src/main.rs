use nt_world_sim::core::{UniversalWorld, Component, Resource};
use nt_world_sim::core::scheduler::{ParallelScheduler, UniversalSystem, SystemDependency};

#[derive(Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

struct GameTime {
    #[allow(dead_code)]
    delta: f32,
    elapsed: f32,
}
impl Resource for GameTime {}

struct MovementSystem;

impl UniversalSystem for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }

    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        let entities: Vec<_> = world.entities();
        for entity in entities {
            let vel = world.get_component::<Velocity>(entity).cloned();
            if let Some(vel) = vel {
                if let Some(pos) = world.get_component_mut::<Position>(entity) {
                    pos.x += vel.x * dt;
                    pos.y += vel.y * dt;
                }
            }
        }
    }
}

fn main() {
    println!("=== NeoTrix Universal Game Engine ===");
    println!();

    let mut world = UniversalWorld::new();
    world.insert_resource(GameTime {
        delta: 0.016,
        elapsed: 0.0,
    });

    let player = world.spawn();
    world.insert_component(player, Position { x: 0.0, y: 0.0 });
    world.insert_component(player, Velocity { x: 10.0, y: 5.0 });

    let npc = world.spawn();
    world.insert_component(npc, Position {
        x: 100.0,
        y: 100.0,
    });
    world.insert_component(npc, Velocity { x: -1.0, y: 0.0 });

    println!("[1] Spawned {} entities", world.entity_count());

    let mut scheduler = ParallelScheduler::new();
    scheduler.add_system(Box::new(MovementSystem), SystemDependency::new());
    scheduler.build_schedule();
    println!(
        "[2] Scheduler built: {} systems in {} waves",
        scheduler.system_count(),
        scheduler.wave_count()
    );

    println!("[3] Simulating 5 frames...");
    for frame in 0..5 {
        let dt = 0.016;

        if let Some(time) = world.get_resource_mut::<GameTime>() {
            time.elapsed += dt;
        }

        scheduler.run(&mut world, dt);

        let pos = world.get_component::<Position>(player).unwrap();
        println!("    Frame {}: player=({:.1}, {:.1})", frame, pos.x, pos.y);
    }

    println!();
    println!("[4] Code generation demo:");
    let game_def = nt_world_sim::codegen::GameDefinition::default();
    let gen = nt_world_sim::codegen::CodeGenerator::new(game_def);
    let code = gen.generate();
    println!(
        "    Generated {} lines of Bevy code",
        code.lines().count()
    );

    println!();
    println!("=== Engine Demo Complete ===");
}
