//! neotrix-transit — 独立中转站守护进程
//!
//! 零意识依赖: 仅启动 TransitStation + NetworkMonitor + ConnectivityChecker.
//! 自动生成默认配置, pf divert-to 设置, launchd 自启动安装.
//!
//! Usage:
//!   cargo run --features stealth-net --bin neotrix-transit [--install] [--uninstall]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::signal;

use neotrix::core::nt_core_shutdown::ShutdownSignal;
use neotrix::core::nt_core_util;
use neotrix::neotrix::nt_shield_stealth_net::config::load as load_config;
use neotrix::neotrix::nt_shield_stealth_net::connectivity_checker::ConnectivityChecker;
use neotrix::neotrix::nt_shield_stealth_net::network_monitor::NetworkMonitor;
use neotrix::neotrix::nt_shield_stealth_net::proxy_control::DaemonMode;
use neotrix::neotrix::nt_shield_stealth_net::transit_station::{auto_start_transit, stop_transit, global_transit_station};
use neotrix::neotrix::nt_shield_stealth_net::circuit_isolation::{CircuitIsolationConfig, CircuitIsolationManager};

/// 默认配置内容（transit enabled）
const DEFAULT_CONFIG_TOML: &str = r##"# NeoTrix Transit Daemon — 自动生成
[proxy]
health_check_interval_secs = 30
min_nodes = 3
selection_strategy = "auto"

[rotation]
enabled = true
rotation_interval_secs = 60

[pool]
min_nodes = 3
health_check_interval_secs = 30
multi_hop_count = 3

[bandit]
enabled = true

[transit]
enabled = true
mode = "system_proxy"
listen_port = 11081
per_conn_ip_rotation = true
padding_enabled = true
timing_obfuscation_enabled = true

[ip_rotation]
enabled = false
"##;

fn config_path() -> PathBuf {
    let home = nt_core_util::home_dir().to_string_lossy().to_string();
    PathBuf::from(home).join(".neotrix").join("config.toml")
}

fn ensure_config() {
    let path = config_path();
    if path.exists() {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&path, DEFAULT_CONFIG_TOML) {
        Ok(_) => log::info!("[transit-daemon] created default config at {}", path.display()),
        Err(e) => log::error!("[transit-daemon] failed to create config: {}", e),
    }
}

fn install_launchd_plist() -> Result<(), String> {
        let home = nt_core_util::home_dir().to_string_lossy().to_string();
    let plist_content = format!(r##"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.neotrix.transit-daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}/neotrix-transit</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/.neotrix/transit-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>{}/.neotrix/transit-daemon.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>
</dict>
</plist>
"##, home, home, home);

    let plist_path = PathBuf::from(&home)
        .join("Library")
        .join("LaunchAgents")
        .join("com.neotrix.transit-daemon.plist");
    if let Some(parent) = plist_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&plist_path, plist_content.as_bytes())
        .map_err(|e| format!("write plist: {}", e))?;
    log::info!("[transit-daemon] plist installed at {}", plist_path.display());
    Ok(())
}

fn uninstall_launchd_plist() -> Result<(), String> {
        let home = nt_core_util::home_dir().to_string_lossy().to_string();
    let plist_path = PathBuf::from(&home)
        .join("Library")
        .join("LaunchAgents")
        .join("com.neotrix.transit-daemon.plist");
    if plist_path.exists() {
        std::fs::remove_file(&plist_path)
            .map_err(|e| format!("remove plist: {}", e))?;
        log::info!("[transit-daemon] plist removed from {}", plist_path.display());
    }
    Ok(())
}

fn setup_sudoers() -> Result<(), String> {
    let sudoers_line = "ALL ALL=(ALL) NOPASSWD: /sbin/pfctl *\n";
    let sudoers_path = PathBuf::from("/etc/sudoers.d/neotrix-transit");
    // Only attempt if we have permissions (otherwise warn)
    match std::fs::write(&sudoers_path, sudoers_line) {
        Ok(_) => log::info!("[transit-daemon] sudoers installed at {}", sudoers_path.display()),
        Err(e) => log::warn!("[transit-daemon] sudoers install failed (need sudo?): {}", e),
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse args
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--install" => {
                ensure_config();
                install_launchd_plist()?;
                let _ = setup_sudoers();
                log::info!("[transit-daemon] installed. Run `launchctl load ~/Library/LaunchAgents/com.neotrix.transit-daemon.plist` to start.");
                return Ok(());
            }
            "--uninstall" => {
                uninstall_launchd_plist()?;
                stop_transit().await;
                log::info!("[transit-daemon] uninstalled.");
                return Ok(());
            }
            _ => {
                eprintln!("Usage: neotrix-transit [--install] [--uninstall]");
                std::process::exit(1);
            }
        }
    }

    // Init logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    log::info!("[transit-daemon] starting...");

    // Ensure config exists
    ensure_config();

    // Start transit station
    let cfg = load_config();
    auto_start_transit(&cfg).await.map_err(|e| {
        log::error!("[transit-daemon] failed to start transit: {}", e);
        e
    })?;

    // Wire circuit isolation
    let cm = Arc::new(CircuitIsolationManager::new(
        CircuitIsolationConfig::default(),
        None,
    ));
    {
        let ts = global_transit_station();
        ts.set_circuit_manager_arc(cm).await;
        log::info!("[transit-daemon] circuit isolation wired");
    }

    // Create shutdown signal
    let shutdown = ShutdownSignal::new();
    let shutdown_clone = shutdown.clone();

    // Handle Ctrl+C
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap_or_else(|e| {
            log::error!("ctrl+c handler failed: {e}");
        });
        log::info!("[transit-daemon] received Ctrl+C, shutting down...");
        shutdown_clone.trigger("ctrl+c");
    });

    // Main loop: network tick + transit tick + stats
    let network_interval = Duration::from_secs(30);
    let transit_interval = Duration::from_secs(60);

    // Connectivity checker (auto-refills pool when low)
    let mode = Arc::new(tokio::sync::RwLock::new(DaemonMode::Off));
    let checker = Arc::new(ConnectivityChecker::new(mode.clone()));
    let checker_clone = checker.clone();
    let shutdown_checker = shutdown.clone();
    tokio::spawn(async move {
        checker_clone.start_background(shutdown_checker).await;
    });

    // Main loop
    let mut network_tick = tokio::time::interval(network_interval);
    network_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut transit_tick = tokio::time::interval(transit_interval);
    transit_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = network_tick.tick() => {
                log::info!("[transit-daemon] network tick");
                let mut monitor = NetworkMonitor::default();
                monitor.tick().await;
                let snap = checker.snapshot().await;
                log::info!(
                    "[transit-daemon] connectivity: direct={}, proxy_h={}/{} mode={:?}",
                    snap.direct_reachable, snap.proxy_healthy_count,
                    snap.proxy_total_count, snap.active_mode,
                );
            }
            _ = transit_tick.tick() => {
                let ts = global_transit_station();
                if !ts.is_enabled() { continue; }
                let stats = ts.stats().await;
                if stats.active_connections > 10 || stats.conn_count % 5 == 0 {
                    ts.auto_assign_roles().await;
                    ts.adapt_rotation_to_bandit().await;
                }
                log::info!(
                    "[transit-daemon] transit: conn={}, active={}, bytes={}, mode={:?}",
                    stats.conn_count, stats.active_connections,
                    stats.total_bytes_relayed, stats.mode,
                );
            }
            _ = shutdown.wait_shutdown() => {
                log::info!("[transit-daemon] shutting down gracefully...");
                stop_transit().await;
                break;
            }
        }
    }

    log::info!("[transit-daemon] exited cleanly");
    Ok(())
}
