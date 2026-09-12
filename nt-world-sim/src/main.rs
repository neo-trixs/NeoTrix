use nt_world_sim::core::{UniversalWorld, Component, Resource};
use nt_world_sim::core::scheduler::{ParallelScheduler, UniversalSystem, SystemDependency};
use nt_world_sim::mechanics::core_pet::{CorePetState, PetStateEnum, CorePetSystem};
use nt_world_sim::mechanics::core_hook::{CoreHookEvent, CoreHookManager};
use nt_world_sim::mechanics::core_theme::CoreThemeManager;

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

    // 1. Create world
    let mut world = UniversalWorld::new();
    world.insert_resource(GameTime {
        delta: 0.016,
        elapsed: 0.0,
    });

    // 2. Spawn entities
    let player = world.spawn();
    world.insert_component(player, Position { x: 0.0, y: 0.0 });
    world.insert_component(player, Velocity { x: 10.0, y: 5.0 });
    world.insert_component(
        player,
        CorePetState {
            state: PetStateEnum::Idle,
            state_timer: 0.0,
            idle_timer: 0.0,
        },
    );

    let npc = world.spawn();
    world.insert_component(npc, Position {
        x: 100.0,
        y: 100.0,
    });
    world.insert_component(npc, Velocity { x: -1.0, y: 0.0 });

    println!("[1] Spawned {} entities", world.entity_count());

    // 3. Load theme
    CoreThemeManager::load_default_theme(&mut world);
    println!("[2] Loaded default theme");

    // 4. Create scheduler
    let mut scheduler = ParallelScheduler::new();
    scheduler.add_system(Box::new(MovementSystem), SystemDependency::new());
    scheduler.build_schedule();
    println!(
        "[3] Scheduler built: {} systems in {} waves",
        scheduler.system_count(),
        scheduler.wave_count()
    );

    // 5. Simulate 5 frames
    println!("[4] Simulating 5 frames...");
    for frame in 0..5 {
        let dt = 0.016;

        // Update time resource
        if let Some(time) = world.get_resource_mut::<GameTime>() {
            time.elapsed += dt;
        }

        // Run movement system via scheduler
        scheduler.run(&mut world, dt);

        // Run pet system
        CorePetSystem::update(&mut world, dt);

        // Simulate hook event on frame 2
        if frame == 2 {
            CoreHookManager::process_event(
                &mut world,
                CoreHookEvent::SessionStart {
                    agent: "claude".to_string(),
                    session_id: "session-1".to_string(),
                },
            );
            println!("    Frame {}: Hook event sent (SessionStart)", frame);
        }

        // Print positions
        let pos = world.get_component::<Position>(player).unwrap();
        let pet = world.get_component::<CorePetState>(player).unwrap();
        println!(
            "    Frame {}: player=({:.1}, {:.1}) pet={:?}",
            frame, pos.x, pos.y, pet.state
        );
    }

    // 6. Final state
    println!();
    println!("[5] Final state:");
    let pos = world.get_component::<Position>(player).unwrap();
    println!("    Player position: ({:.1}, {:.1})", pos.x, pos.y);
    let pet = world.get_component::<CorePetState>(player).unwrap();
    println!("    Pet state: {:?}", pet.state);

    // 7. Code generation demo
    println!();
    println!("[6] Code generation demo:");
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
