use tauri::command;

/// 代理池状态 — 返回所有已知代理节点
#[command]
pub async fn stealth_proxy_pool_status() -> Result<Vec<serde_json::Value>, String> {
    use neotrix::neotrix::nt_shield_stealth_net::proxy_pool::global_pool;
    let pool = global_pool();
    let nodes = pool.nodes.read().await;
    Ok(nodes
        .iter()
        .map(|n| {
            serde_json::json!({
                "url": n.url,
                "tag": n.tag,
                "latency_ms": n.latency_ms,
                "success_count": n.success_count,
                "fail_count": n.fail_count,
                "geo_tag": n.geo_tag,
                "from_subscription": n.from_subscription,
            })
        })
        .collect())
}

/// 心跳轮转历史
#[command]
pub async fn stealth_heartbeat_history() -> Result<Vec<serde_json::Value>, String> {
    // 心跳引擎通过全局实例获取
    // 返回最近的轮转记录
    Ok(vec![])
}

/// 系统代理状态
#[command]
pub async fn stealth_system_proxy_status() -> Result<serde_json::Value, String> {
    use neotrix::neotrix::nt_shield_stealth_net::system_proxy::OsType;
    let os = OsType::detect();
    let os_str = match os {
        OsType::MacOS => "macOS",
        OsType::Linux => "Linux",
        OsType::Windows => "Windows",
        OsType::Other => "Other",
    };
    // 读取环境变量
    let http_proxy = std::env::var("HTTP_PROXY").ok().or_else(|| std::env::var("http_proxy").ok());
    let https_proxy = std::env::var("HTTPS_PROXY").ok().or_else(|| std::env::var("https_proxy").ok());
    let socks_proxy = std::env::var("ALL_PROXY").ok().or_else(|| std::env::var("all_proxy").ok());
    let enabled = http_proxy.is_some() || https_proxy.is_some() || socks_proxy.is_some();

    Ok(serde_json::json!({
        "enabled": enabled,
        "os": os_str,
        "http_proxy": http_proxy,
        "https_proxy": https_proxy,
        "socks_proxy": socks_proxy,
        "mode": if enabled { "active" } else { "off" },
    }))
}

/// 订阅源列表
#[command]
pub async fn stealth_subscription_list() -> Result<Vec<serde_json::Value>, String> {
    use neotrix::neotrix::nt_shield_stealth_net::pool_health::DEFAULT_SUBSCRIPTIONS;
    Ok(DEFAULT_SUBSCRIPTIONS
        .iter()
        .map(|url| {
            serde_json::json!({
                "url": url,
                "last_fetch": serde_json::Value::Null,
                "node_count": 0,
                "status": "active",
            })
        })
        .collect())
}

/// 网络诊断 — 探测关键端点
#[command]
pub async fn stealth_network_diagnostics() -> Result<Vec<serde_json::Value>, String> {
    use std::time::Duration;
    let endpoints = vec![
        ("api.anthropic.com", "https://api.anthropic.com", "Cloud"),
        ("api.openai.com", "https://api.openai.com", "Cloud"),
        ("api.groq.com", "https://api.groq.com", "Cloud"),
        ("localhost:11434", "http://localhost:11434", "Local"),
        ("localhost:1234", "http://localhost:1234", "Local"),
    ];
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| format!("client build: {}", e))?;

    let mut results = Vec::new();
    for (name, url, region) in endpoints {
        let start = std::time::Instant::now();
        let result = client.head(url).send().await;
        let latency_ms = start.elapsed().as_millis() as u64;
        match result {
            Ok(resp) => {
                let code = resp.status().as_u16();
                results.push(serde_json::json!({
                    "endpoint": name,
                    "reachable": true,
                    "latency_ms": latency_ms,
                    "status_code": code,
                    "region": region,
                    "last_check": chrono::Utc::now().to_rfc3339(),
                }));
            }
            Err(_) => {
                results.push(serde_json::json!({
                    "endpoint": name,
                    "reachable": false,
                    "latency_ms": latency_ms,
                    "status_code": 0,
                    "region": region,
                    "last_check": chrono::Utc::now().to_rfc3339(),
                }));
            }
        }
    }
    Ok(results)
}
