use clap::Parser;
use nt_daemon::{DaemonConfig, DaemonServer};

#[derive(Parser, Debug)]
#[command(name = "nt-daemon", about = "NeoTrix Remote Agent Execution Daemon")]
struct Args {
    #[arg(long, default_value = "127.0.0.1:0")]
    listen: String,

    #[arg(long, env = "NT_DAEMON_TOKEN")]
    token: Option<String>,

    #[arg(long, default_value = "5")]
    max_concurrent: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = DaemonConfig {
        listen_addr: args.listen,
        daemon_token: args.token.unwrap_or_default(),
        max_concurrent: args.max_concurrent,
        ..DaemonConfig::default()
    };

    eprintln!("╔══════════════════════════════════════════╗");
    eprintln!("║      NeoTrix Remote Agent Daemon         ║");
    eprintln!("╚══════════════════════════════════════════╝");
    eprintln!(
        "[nt-daemon] max_concurrent={} token={}",
        config.max_concurrent,
        if config.daemon_token.is_empty() {
            "disabled"
        } else {
            "enabled"
        }
    );

    let server = DaemonServer::new(config);
    let state = server.state.clone();

    tokio::spawn(async move {
        server.run().await.ok();
    });

    tokio::signal::ctrl_c().await?;
    eprintln!("\n[nt-daemon] shutting down ({} active tasks)...", {
        state.active_tasks.lock().unwrap().len()
    });
    Ok(())
}
