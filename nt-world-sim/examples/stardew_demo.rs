use nt_world_sim::core::{
    Component, UniversalWorld, UniversalSystem,
    ParallelScheduler, SystemDependency,
};
use nt_world_sim::mechanics::core_pet::{CorePetState, CorePetSystem};
use nt_world_sim::mechanics::core_hook::{CoreHookEvent, CoreHookManager};
use nt_world_sim::mechanics::core_theme::CoreThemeManager;
use nt_world_sim::codegen::parser::GameDefParser;
use nt_world_sim::codegen::generator::CodeGenerator;

// ---------------------------------------------------------------------------
// Local component / resource / event types
// ---------------------------------------------------------------------------

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

#[derive(Clone)]
struct NpcTag;
impl Component for NpcTag {}

// ---------------------------------------------------------------------------
// MovementSystem — integrates velocity into position
// ---------------------------------------------------------------------------

struct MovementSystem;

impl UniversalSystem for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }

    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        let entities: Vec<_> = world
            .query::<(Position, Velocity)>()
            .into_iter()
            .map(|(e, _)| e)
            .collect();

        for entity in entities {
            let vel = world.get_component::<Velocity>(entity).cloned().unwrap();
            if let Some(pos) = world.get_component_mut::<Position>(entity) {
                pos.x += vel.x * dt;
                pos.y += vel.y * dt;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main pipeline
// ---------------------------------------------------------------------------

fn main() {
    println!("=== NeoTrix nt-world-sim · stardew_demo ===\n");

    // 1. Create world
    let mut world = UniversalWorld::new();

    // 2. Spawn player with Position, Velocity, CorePetState
    let player = world.spawn();
    world.insert_component(
        player,
        Position { x: 0.0, y: 0.0 },
    );
    world.insert_component(
        player,
        Velocity { x: 1.5, y: 0.5 },
    );
    world.insert_component(player, CorePetState::new());
    println!("Spawned player (entity {:?})", player.id);

    // 3. Spawn 2 NPC entities
    let npc1 = world.spawn();
    world.insert_component(
        npc1,
        Position { x: 10.0, y: 20.0 },
    );
    world.insert_component(
        npc1,
        Velocity { x: -0.3, y: 0.1 },
    );
    world.insert_component(npc1, NpcTag);

    let npc2 = world.spawn();
    world.insert_component(
        npc2,
        Position { x: -5.0, y: 15.0 },
    );
    world.insert_component(
        npc2,
        Velocity {
            x: 0.2,
            y: -0.4,
        },
    );
    world.insert_component(npc2, NpcTag);
    println!("Spawned 2 NPCs (entity {:?}, {:?})", npc1.id, npc2.id);

    // 4. Load default theme
    CoreThemeManager::load_default_theme(&mut world);
    let anim_path = CoreThemeManager::get_animation_path(&world, "idle").unwrap_or_default();
    println!("Loaded default theme — idle animation: {}", anim_path);

    // 5. Create ParallelScheduler with MovementSystem
    let mut scheduler = ParallelScheduler::new();
    scheduler.add_system(
        Box::new(MovementSystem),
        SystemDependency::new(),
    );
    println!(
        "Scheduler ready — {} system(s), {} wave(s) after build\n",
        scheduler.system_count(),
        scheduler.wave_count()
    );

    // 6-8. Run 10 frames, hook events on frame 3 and 7
    let total_frames = 10;
    let dt = 0.016; // ~60 fps

    for frame in 1..=total_frames {
        // Hook events at specific frames
        if frame == 3 {
            println!(">>> Frame {}: sending SessionStart hook", frame);
            CoreHookManager::send_event(
                &mut world,
                CoreHookEvent::SessionStart {
                    agent: "stardew_demo".into(),
                    session_id: "demo-001".into(),
                },
            );
            let events = CoreHookManager::receive_events(&mut world);
            for ev in events {
                CoreHookManager::process_event(&mut world, ev);
            }
        }
        if frame == 7 {
            println!(">>> Frame {}: sending ToolEnd hook (success)", frame);
            CoreHookManager::send_event(
                &mut world,
                CoreHookEvent::ToolEnd {
                    tool: "bash".into(),
                    success: true,
                },
            );
            let events = CoreHookManager::receive_events(&mut world);
            for ev in events {
                CoreHookManager::process_event(&mut world, ev);
            }
        }

        // Run systems
        scheduler.run(&mut world, dt);

        // Run pet system
        CorePetSystem::update(&mut world, dt);

        // Print positions
        let all_entities = world.entities();
        print!("  Frame {:2}: ", frame);
        for entity in &all_entities {
            if let Some(pos) = world.get_component::<Position>(*entity) {
                print!(
                    "[{:?}]({:.2},{:.2}) ",
                    entity.id, pos.x, pos.y
                );
            }
        }
        println!();
    }

    // 9. Final pet state
    println!();
    println!("--- Final Pet State ---");
    let all_entities = world.entities();
    for entity in &all_entities {
        if let Some(pet) = world.get_component::<CorePetState>(*entity) {
            println!(
                "  Entity {:?}: state={:?}  timer={:.2}  idle_timer={:.2}",
                entity.id, pet.state, pet.state_timer, pet.idle_timer
            );
        }
    }

    // 10. Parse YAML and generate Bevy code
    println!();
    println!("--- Code Generation ---");
    let yaml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/stardew_valley.yaml");
    match GameDefParser::parse(&yaml_path) {
        Ok(def) => {
            println!(
                "Parsed '{}' v{} (target={})",
                def.name, def.version, def.engine.target
            );
            println!(
                "  entities: {:?}  systems: {:?}  resources: {:?}",
                def.entities.keys().collect::<Vec<_>>(),
                def.systems.keys().collect::<Vec<_>>(),
                def.resources.keys().collect::<Vec<_>>()
            );
            let gen = CodeGenerator::new(def);
            let code = gen.generate();
            println!();
            println!("Generated Bevy code ({} bytes):", code.len());
            println!("{}", code);
        }
        Err(e) => {
            eprintln!("Failed to parse YAML: {}", e);
        }
    }

    println!("\n=== stardew_demo complete ===");
}
