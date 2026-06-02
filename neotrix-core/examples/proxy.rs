//! NeoTrix 独立代理启动器 — 仅启动网络代理模块
//!
//! 不依赖全量 NeoTrix 构建，绕过有问题的 binary 模块
//! 使用方式: cargo run --example proxy

use std::sync::Arc;
use std::time::Duration;

use neotrix::neotrix::nt_shield_stealth_net::config::load as cfg;

#[tokio::main]
async fn main() {
    let c = cfg();
    println!("╭──────────────────────────────────────────────╮");
    println!("│  NeoTrix Smart Proxy                         │");
    println!("│  纯 Rust 智能路由 · 无需外部依赖               │");
    println!("╰──────────────────────────────────────────────╯");
    println!("  📄 配置: {}", neotrix::neotrix::nt_shield_stealth_net::config::config_file_path());

    // 1. 启动 Tor/Arti（后台安装）
    let tor = Arc::new(neotrix::neotrix::nt_shield_stealth_net::local_proxy::TorManager::new());
    let tor_bg = tor.clone();
    tokio::spawn(async move {
        tor_bg.auto_install_and_start().await;
    });

    // 2. 设置系统代理 (macOS networksetup)
    let sys_proxy = Arc::new(neotrix::neotrix::nt_shield_stealth_net::system_proxy::SystemProxyManager::new());
    match sys_proxy.enable().await {
        Ok(_) => println!("  ✅ 系统代理已设置 → HTTP :{}", c.proxy.local_port),
        Err(e) => eprintln!("  ⚠ 系统代理: {e}"),
    }
    // SIGTERM/SIGINT → 恢复系统代理
    sys_proxy.clone().install_shutdown_handler().await;

    // 3. 启动本地 HTTP CONNECT 代理
    let mut engine = neotrix::neotrix::nt_shield_stealth_net::rules::RuleEngine::new();
    engine.load_rules(neotrix::neotrix::nt_shield_stealth_net::rules::china_bypass_rules());
    let proxy = neotrix::neotrix::nt_shield_stealth_net::local_proxy::LocalProxy::new()
        .with_rule_engine(Arc::new(engine));
    let proxy = Arc::new(proxy);
    let p = proxy.clone();
    tokio::spawn(async move {
        let _ = p.start().await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 4. 代理 IP 池 — 启动健康检测 + 默认订阅
    let pool = neotrix::neotrix::nt_shield_stealth_net::proxy_pool::global_pool();
    // 用户可在此处添加订阅地址
    // pool.fetch_subscription("https://your-sub-url").await.ok();
    let pool_hc = pool.clone();
    tokio::spawn(async move {
        pool_hc.start_health_loop(60).await; // 每 60s 检测一次
    });

    // 5. 预热 Tor 电路（后台提前构建，减少首次请求延迟）
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await; // 等待 arti 启动
        for _ in 0..3 {
            if neotrix::neotrix::nt_shield_stealth_net::local_proxy::TorManager::socks5_reachable().await {
                // 通过 Google 预热 Tor 电路
                let _ = neotrix::neotrix::nt_shield_stealth_net::local_proxy::tor_connect("www.google.com", 443).await;
                break;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // 5. 状态摘要
    println!();
    println!("╭──────────────────────────────────────────────╮");
    println!("│  ✅ 本地代理 : {}                  │", proxy.proxy_url());
    println!("│                                                │");
    println!("│  浏览器访问 http://127.0.0.1:11080 查看状态     │");
    println!("│  直连已就绪 · Tor/Arti 后台安装中...           │");
    println!("╰──────────────────────────────────────────────╯");
    println!();

    // 保持运行
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
