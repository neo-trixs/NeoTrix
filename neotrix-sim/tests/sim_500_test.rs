#[tokio::test]
async fn sim_500_ticks() {
    use neotrix_sim::world_sim::{WorldSim, WorldSimConfig};
    let config = WorldSimConfig::default();
    let mut sim = WorldSim::new(config);
    let initial = sim.agents.len();
    let mut deaths = 0;
    for tick in 0..500 {
        let before = sim.agents.iter().filter(|a| a.core.alive).count();
        sim.tick().await;
        let after = sim.agents.iter().filter(|a| a.core.alive).count();
        deaths += before.saturating_sub(after);
        if tick % 100 == 0 {
            println!("Tick {}: {} alive, {} total deaths", tick, after, deaths);
        }
    }
    let alive = sim.agents.iter().filter(|a| a.core.alive).count();
    println!("Final: {} alive, {} total deaths, {} tick", alive, deaths, sim.tick);
    assert!(alive > 0, "No agents survived 500 ticks!");
}
