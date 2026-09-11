#[tokio::test]
async fn sim_200_ticks() {
    use neotrix_sim::world_sim::{WorldSim, WorldSimConfig};
    let config = WorldSimConfig::default();
    let mut sim = WorldSim::new(config);
    let initial = sim.agents.len();
    for _ in 0..200 {
        sim.tick().await;
    }
    let alive = sim.agents.iter().filter(|a| a.core.alive).count();
    println!("Initial: {}, Alive after 200 ticks: {}", initial, alive);
    assert!(alive > 0, "No agents survived 200 ticks!");
}
