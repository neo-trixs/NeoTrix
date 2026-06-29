#[allow(unused_imports)]
use colored::Colorize;


/// Resolve proxy daemon binary path (inlined from removed nt_io_proxy module)
fn resolve_daemon_path_inline() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("NEOTRIX_PROXY_DAEMON_PATH") {
        let p = std::path::PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("neotrix-proxy-daemon");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("neotrix-proxy-daemon");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    Err("neotrix-proxy-daemon not found in PATH, NEOTRIX_PROXY_DAEMON_PATH, or next to current executable".to_string())
}

#[cfg(feature = "stealth-net")]
pub async fn run_proxy_cmd(cmd_str: &str) {
    use neotrix::neotrix::nt_shield_stealth_net::local_proxy::TorManager;
    use neotrix::neotrix::nt_shield_stealth_net::proxy_control::{DaemonMode, ProxyClient};

    let client = ProxyClient::new();
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let sub = parts.first().copied().unwrap_or("status");

    match sub {
        "status" => match client.status().await {
            Ok(s) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                    let mode = v["mode"].as_str().unwrap_or("?");
                    let pid = v["pid"].as_i64().unwrap_or(0);
                    let port = v["port"].as_i64().unwrap_or(0);
                    let uptime = v["uptime_secs"].as_i64().unwrap_or(0);
                    let active = v["active_count"].as_i64().unwrap_or(0);
                    let idle = v["idle_secs"].as_i64().unwrap_or(0);
                    log::info!("╭─ Proxy Daemon ─────────────────────╮");
                    log::info!("│ {}  {}", "Mode:".blue(), mode);
                    log::info!("│ {}  {}", "PID:".blue(), pid);
                    log::info!("│ {}  {}", "Port:".blue(), port);
                    log::info!("│ {}  {}s", "Uptime:".blue(), uptime);
                    log::info!("│ {}  {}", "Active:".blue(), active);
                    log::info!("│ {}  {}s", "Idle:".blue(), idle);
                    log::info!(
                        "│ {}  {}",
                        "Tor SOCKS5:".blue(),
                        if TorManager::socks5_reachable().await {
                            "✓ reachable".green()
                        } else {
                            "✗ unreachable".red()
                        }
                    );
                    log::info!("╰────────────────────────────────────╯");
                } else {
                    log::info!("{}", s);
                }
            }
            Err(_) => {
                log::info!("{}", "Daemon not reachable.".yellow());
                log::info!("Run `neotrix proxy start` to start the proxy daemon.");
            }
        },
        "mode" => {
            let mode_str = parts.get(1).copied();
            match mode_str {
                Some(m) => {
                    if let Some(mode) = DaemonMode::from_str(m) {
                        match client.set_mode(mode).await {
                            Ok(_) => log::info!("{} proxy mode → {}", "✓".green(), m),
                            Err(e) => log::error!("{} {}", "✗".red(), e),
                        }
                    } else {
                        log::info!("{} 模式: {}. 可选: off, geo, stealth, tor", "未知".red(), m);
                    }
                }
                None => match client.status().await {
                    Ok(s) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                            log::info!(
                                "{} 模式: {}",
                                "当前".blue(),
                                v["mode"].as_str().unwrap_or("?")
                            );
                        }
                    }
                    Err(e) => log::error!("{} {}", "✗".red(), e),
                },
            }
        }
        "start" => {
            if client.is_reachable().await {
                log::info!("{} Proxy daemon already running.", "✓".green());
                return;
            }
            let daemon_path = match resolve_daemon_path_inline() {
                Ok(p) => p,
                Err(e) => {
                    log::error!("{} {}", "✗".red(), e);
                    return;
                }
            };
            let home = nt_core_util::home_dir().to_string_lossy().to_string();
            let pid_file = format!("{}/.neotrix/neotrix-proxy-daemon.pid", home);
            match std::process::Command::new(&daemon_path)
                .arg("--pid-file")
                .arg(pid_file)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_) => {
                    for _ in 0..50 {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        if client.is_reachable().await {
                            log::info!("{} Proxy daemon started.", "✓".green());
                            return;
                        }
                    }
                    log::error!("{} Proxy daemon did not start in time.", "✗".red());
                }
                Err(e) => log::error!("{} Failed to start proxy daemon: {}", "✗".red(), e),
            }
        }
        "stop" => match client.shutdown().await {
            Ok(_) => log::info!("{} Proxy daemon shutdown.", "✓".green()),
            Err(e) => {
                log::error!("{} shutdown API failed: {}. Trying pkill...", "✗".red(), e);
                match std::process::Command::new("pkill")
                    .args(["-f", "neotrix-proxy-daemon"])
                    .status()
                {
                    Ok(status) if status.success() => {
                        log::info!("{} Proxy daemon killed.", "✓".green())
                    }
                    Ok(_) => log::error!("{} No proxy daemon process found.", "✗".red()),
                    Err(e) => log::error!("{} pkill failed: {}", "✗".red(), e),
                }
            }
        },
        "install" => {
            let plist = match resolve_daemon_path_inline() {
                Ok(p) => {
                    let parent = p.parent().unwrap_or(std::path::Path::new("/usr/local/bin"));
                    parent.join("com.neotrix.proxy-daemon.plist")
                }
                Err(_) => {
                    let home = dirs::home_dir().unwrap_or_default();
                    home.join(".config/neotrix/com.neotrix.proxy-daemon.plist")
                }
            };
            if !plist.exists() {
                log::error!("{} plist not found at: {}", "✗".red(), plist.display());
                return;
            }
            match std::process::Command::new("launchctl")
                .args(["load", &plist.to_string_lossy()])
                .status()
            {
                Ok(status) if status.success() => {
                    log::info!("{} launchd plist loaded.", "✓".green());
                }
                Ok(_) => {
                    log::warn!(
                        "{} launchctl load failed. Trying unload first...",
                        "✗".red()
                    );
                    let _ = std::process::Command::new("launchctl")
                        .args(["unload", &plist.to_string_lossy()])
                        .status();
                    match std::process::Command::new("launchctl")
                        .args(["load", &plist.to_string_lossy()])
                        .status()
                    {
                        Ok(status) if status.success() => {
                            log::info!("{} launchd plist loaded.", "✓".green())
                        }
                        Ok(status) => {
                            log::error!("{} launchctl load failed: {}", "✗".red(), status)
                        }
                        Err(e) => log::error!("{} launchctl error: {}", "✗".red(), e),
                    }
                }
                Err(e) => log::error!("{} launchctl error: {}", "✗".red(), e),
            }
        }
        _ => {
            log::info!(
                "用法: neotrix proxy [status|mode [off|geo|stealth|tor]|start|stop|install]"
            );
        }
    }
}

#[cfg(not(feature = "stealth-net"))]
pub async fn run_proxy_cmd(_cmd_str: &str) {
    log::info!("proxy 命令需要 --features stealth-net 编译");
}
