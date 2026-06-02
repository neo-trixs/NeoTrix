use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;
use neotrix::neotrix::stealth_net::local_proxy::{LocalProxy, TorManager};
use neotrix::neotrix::stealth_net::proxy_pool::global_pool;
use neotrix::neotrix::stealth_net::rules::RuleEngine;
use neotrix::neotrix::stealth_net::geo_proxy::RuleUpdater;
use neotrix::neotrix::stealth_net::tor_crawler::TorCrawler;
use neotrix::neotrix::stealth_net::self_iterating::FingerprintManager;
use neotrix::neotrix::stealth_net::proxy_control::{ProxyControl, DaemonMode};

#[tokio::main]
async fn main() {
    println!("[proxy-daemon] PID={}", std::process::id());

    let args: Vec<String> = std::env::args().collect();
    let initial_mode = args.iter()
        .find_map(|a| {
            if a.starts_with("--mode=") { DaemonMode::from_str(&a[7..]) }
            else { None }
        })
        .unwrap_or(DaemonMode::Geo);

    // --- 控制面 (Unix socket) ---
    let control = Arc::new(ProxyControl::new());
    control.set_mode(initial_mode).await;
    let ctrl = control.clone();
    tokio::spawn(async move {
        if let Err(e) = ctrl.start_control_server().await {
            eprintln!("[control] 退出: {}", e);
        }
    });

    // --- 规则引擎 ---
    let rule_engine = Arc::new(RwLock::new(RuleEngine::new()));

    // --- 代理池 ---
    let pool = global_pool();
    pool.load_subscriptions().await;
    if let Ok(sub_url) = std::env::var("NEOTRIX_PROXY_SUB_URL") {
        let url = sub_url.trim().to_string();
        if !url.is_empty() {
            pool.add_subscription(&url).await;
        }
    }
    let p = pool.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        p.start_health_loop(60).await;
    });

    // --- Tor ---
    let tor = Arc::new(TorManager::new());
    let t = tor.clone();
    tokio::spawn(async move { t.auto_install_and_start().await; });
    let t2 = tor.clone();
    tokio::spawn(async move { t2.start_health_monitor().await; });

    // --- Geo 规则更新 ---
    let updater = Arc::new(RuleUpdater::new(RuleUpdater::default_cache()));
    let uc = updater.clone();
    tokio::spawn(async move {
        uc.start_auto_update(6).await;
    });

    // --- Tor 深网爬虫 ---
    let nt_world_crawl = Arc::new(TorCrawler::new(
        "127.0.0.1:9050".to_string(),
        dirs::home_dir().unwrap_or_default().join(".neotrix"),
    ));
    let cr = nt_world_crawl.clone();
    tokio::spawn(async move { println!("[tor-nt_world_crawl] 启动"); cr.run().await; });

    // --- 指纹管理器 ---
    let fp_manager = Arc::new(std::sync::Mutex::new(FingerprintManager::new()));

    // --- LocalProxy HTTP CONNECT ---
    let proxy = Arc::new(
        LocalProxy::new()
            .with_rule_engine(rule_engine.clone())
            .with_fingerprint_manager(fp_manager.clone())
            .with_mode_controller(control.mode_ref()),
    );
    let p1 = proxy.clone();
    tokio::spawn(async move {
        if let Err(e) = p1.start().await {
            eprintln!("[proxy] CONNECT 退出: {}", e);
        }
    });

    println!("[proxy-daemon] ready, mode={}", initial_mode.as_str());

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        r.store(false, Ordering::SeqCst);
        println!("\n[proxy-daemon] shutting down...");
        std::process::exit(0);
    });

    let mut net_monitor = neotrix::neotrix::stealth_net::NetworkMonitor::default();

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_secs(15)).await;
        net_monitor.tick().await;

        let current_mode = control.current_mode().await;

        // 指纹旋转: 仅在 stealth 模式下
        if current_mode == DaemonMode::Stealth {
            if let Ok(mut guard) = fp_manager.lock() {
                guard.atomic_rotate();
            }
        }

        // 热重载
        global_pool().reload_subscriptions().await;

        // 空闲超时检测 (off 模式 + 无活动 5min)
        if control.should_shutdown_idle().await {
            println!("[proxy-daemon] idle timeout, exiting");
            break;
        }

        // 心跳日志
        let pool_count = global_pool().available_count().await;
        let fp_count = fp_manager.lock().map(|g| g.fingerprint_count()).unwrap_or(0);
        println!("[proxy-daemon] mode={} | 池: {} | fp: {} | tor: {}",
            current_mode.as_str(), pool_count, fp_count,
            if TorManager::socks5_reachable().await { "✓" } else { "✗" },
        );
    }
}
