use std::sync::Arc;

use tokio::sync::RwLock;

use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::server::start_server;

#[allow(dead_code)]
pub(crate) async fn run_server_mode(agent: Arc<RwLock<SelfIteratingBrain>>, addr: &str) {
    println!("Starting server on {}...", addr);
    start_server(agent, addr).await;
}
