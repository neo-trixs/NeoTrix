use std::sync::Arc;

use tokio::sync::RwLock;

use neotrix::cli::tui::TuiApp;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

pub(crate) async fn run_tui(_agent: Arc<RwLock<SelfIteratingBrain>>, ephemeral: bool) {
    let mut app = TuiApp::new(ephemeral);
    app.run(()).await;
}
