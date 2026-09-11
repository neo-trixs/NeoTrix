use neotrix_sim::world_sim::{WorldSim, WorldSimConfig};

#[tokio::test]
async fn simulation_100_ticks() {
    let config = WorldSimConfig::default();
    let mut sim = WorldSim::new(config);
    
    println!("=== NT-WORLD-SIM Simulation Test ===");
    println!("Initial agents: {}", sim.agents.len());
    println!("World size: {}x{}", sim.config.world_width, sim.config.world_height);
    
    // Run 100 ticks
    for i in 0..100 {
        sim.tick().await;
        if i % 10 == 0 {
            let alive = sim.agents.iter().filter(|a| a.core.alive).count();
            println!("Tick {}: {} agents alive", sim.tick, alive);
        }
    }
    
    let final_alive = sim.agents.iter().filter(|a| a.core.alive).count();
    println!("=== Final: {} agents alive after 100 ticks ===", final_alive);
    assert!(final_alive > 0, "No agents survived!");
    println!("=== Simulation Test PASSED ===");
}
