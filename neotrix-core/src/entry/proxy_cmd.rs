#[allow(unused_imports)]
use colored::Colorize;

#[cfg(feature = "stealth-net")]
pub async fn run_proxy_cmd(cmd_str: &str) {
    use neotrix::neotrix::stealth_net::local_proxy::TorManager;
    use neotrix::neotrix::stealth_net::proxy_control::{DaemonMode, ProxyClient};
    use neotrix::neotrix::proxy_daemon_wrapper;

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
                    println!("╭─ Proxy Daemon ─────────────────────╮");
                    println!("│ {}  {}", "Mode:".blue(), mode);
                    println!("│ {}  {}", "PID:".blue(), pid);
                    println!("│ {}  {}", "Port:".blue(), port);
                    println!("│ {}  {}s", "Uptime:".blue(), uptime);
                    println!("│ {}  {}", "Active:".blue(), active);
                    println!("│ {}  {}s", "Idle:".blue(), idle);
                    println!(
                        "│ {}  {}",
                        "Tor SOCKS5:".blue(),
                        if TorManager::socks5_reachable().await {
                            "✓ reachable".green()
                        } else {
                            "✗ unreachable".red()
                        }
                    );
                    println!("╰────────────────────────────────────╯");
                } else {
                    println!("{}", s);
                }
            }
            Err(_) => {
                println!("{}", "Daemon not reachable.".yellow());
                println!("Run `neotrix proxy start` to start the proxy daemon.");
            }
        },
        "mode" => {
            let mode_str = parts.get(1).copied();
            match mode_str {
                Some(m) => {
                    if let Some(mode) = DaemonMode::from_str(m) {
                        match client.set_mode(mode).await {
                            Ok(_) => println!("{} proxy mode → {}", "✓".green(), m),
                            Err(e) => eprintln!("{} {}", "✗".red(), e),
                        }
                    } else {
                        println!("{} 模式: {}. 可选: off, geo, stealth, tor", "未知".red(), m);
                    }
                }
                None => match client.status().await {
                    Ok(s) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                            println!("{} 模式: {}", "当前".blue(), v["mode"].as_str().unwrap_or("?"));
                        }
                    }
                    Err(e) => eprintln!("{} {}", "✗".red(), e),
                },
            }
        }
        "start" => {
            if client.is_reachable().await {
                println!("{} Proxy daemon already running.", "✓".green());
                return;
            }
            let daemon_path = match proxy_daemon_wrapper::resolve_daemon_path() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{} {}", "✗".red(), e);
                    return;
                }
            };
            let pid_file = "/tmp/neotrix-proxy-daemon.pid";
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
                            println!("{} Proxy daemon started.", "✓".green());
                            return;
                        }
                    }
                    eprintln!("{} Proxy daemon did not start in time.", "✗".red());
                }
                Err(e) => eprintln!("{} Failed to start proxy daemon: {}", "✗".red(), e),
            }
        }
        "stop" => match client.shutdown().await {
            Ok(_) => println!("{} Proxy daemon shutdown.", "✓".green()),
            Err(e) => {
                eprintln!("{} shutdown API failed: {}. Trying pkill...", "✗".red(), e);
                match std::process::Command::new("pkill").args(["-f", "neotrix-proxy-daemon"]).status() {
                    Ok(status) if status.success() => println!("{} Proxy daemon killed.", "✓".green()),
                    Ok(_) => eprintln!("{} No proxy daemon process found.", "✗".red()),
                    Err(e) => eprintln!("{} pkill failed: {}", "✗".red(), e),
                }
            }
        },
        "install" => {
            let plist = match proxy_daemon_wrapper::resolve_daemon_path() {
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
                eprintln!("{} plist not found at: {}", "✗".red(), plist.display());
                return;
            }
            match std::process::Command::new("launchctl").args(["load", &plist.to_string_lossy()]).status() {
                Ok(status) if status.success() => {
                    println!("{} launchd plist loaded.", "✓".green());
                }
                Ok(_) => {
                    eprintln!("{} launchctl load failed. Trying unload first...", "✗".red());
                    let _ = std::process::Command::new("launchctl").args(["unload", &plist.to_string_lossy()]).status();
                    match std::process::Command::new("launchctl").args(["load", &plist.to_string_lossy()]).status() {
                        Ok(status) if status.success() => println!("{} launchd plist loaded.", "✓".green()),
                        Ok(status) => eprintln!("{} launchctl load failed: {}", "✗".red(), status),
                        Err(e) => eprintln!("{} launchctl error: {}", "✗".red(), e),
                    }
                }
                Err(e) => eprintln!("{} launchctl error: {}", "✗".red(), e),
            }
        }
        _ => {
            println!("用法: neotrix proxy [status|mode [off|geo|stealth|tor]|start|stop|install]");
        }
    }
}

#[cfg(not(feature = "stealth-net"))]
pub async fn run_proxy_cmd(_cmd_str: &str) {
    println!("proxy 命令需要 --features stealth-net 编译");
}
