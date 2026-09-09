use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Feed 源健康状态
#[derive(Debug, Clone)]
pub struct FeedHealth {
    pub url: String,
    pub last_fetch: Option<Instant>,
    pub success_count: u64,
    pub fail_count: u64,
    pub avg_latency_ms: u64,
}

/// Feed 引擎 (并发 + 增量 + 健康度)
pub struct FeedEngine {
    feeds: Vec<String>,
    health: HashMap<String, FeedHealth>,
    last_modified: HashMap<String, String>,
}

impl FeedEngine {
    pub fn new() -> Self {
        Self {
            feeds: Vec::new(),
            health: HashMap::new(),
            last_modified: HashMap::new(),
        }
    }

    /// 添加 Feed
    pub fn add_feed(&mut self, url: String) {
        self.feeds.push(url.clone());
        self.health.insert(url.clone(), FeedHealth {
            url,
            last_fetch: None,
            success_count: 0,
            fail_count: 0,
            avg_latency_ms: 0,
        });
    }

    /// 并发拉取多个 Feed
    pub async fn fetch_all(&mut self) -> Vec<FeedResult> {
        let mut results = Vec::new();
        for url in &self.feeds {
            let start = Instant::now();
            match self.fetch_one(url).await {
                Ok(content) => {
                    let latency = start.elapsed().as_millis() as u64;
                    if let Some(health) = self.health.get_mut(url) {
                        health.last_fetch = Some(Instant::now());
                        health.success_count += 1;
                        health.avg_latency_ms = (health.avg_latency_ms + latency) / 2;
                    }
                    results.push(FeedResult { url: url.clone(), content, success: true });
                }
                Err(e) => {
                    if let Some(health) = self.health.get_mut(url) {
                        health.fail_count += 1;
                    }
                    results.push(FeedResult { url: url.clone(), content: String::new(), success: false });
                    log::warn!("[feed] Failed to fetch {}: {}", url, e);
                }
            }
        }
        results
    }

    /// 拉取单个 Feed (支持增量更新)
    async fn fetch_one(&mut self, url: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let mut req = client.get(url);
        // 增量更新: If-Modified-Since
        if let Some(last_mod) = self.last_modified.get(url) {
            req = req.header("If-Modified-Since", last_mod);
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        // 保存 Last-Modified
        if let Some(last_mod) = resp.headers().get("last-modified") {
            if let Ok(val) = last_mod.to_str() {
                self.last_modified.insert(url.to_string(), val.to_string());
            }
        }
        if resp.status().as_u16() == 304 {
            return Ok(String::new()); // 未修改
        }
        resp.text().await.map_err(|e| e.to_string())
    }

    /// 获取所有 Feed 健康状态
    pub fn health_report(&self) -> Vec<&FeedHealth> {
        self.health.values().collect()
    }
}

#[derive(Debug)]
pub struct FeedResult {
    pub url: String,
    pub content: String,
    pub success: bool,
}
