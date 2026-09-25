//! nt_politeness — 礼貌爬取/限流：站点限速 + robots 缓存 + 429 冷却 + SSRF/allowlist 策略门.
//! 从 `nt_io_browser_engine/engine.rs` 纯搬移, 行为零变更.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::BrowserEngine;
use super::super::error::BrowserError;
use super::super::fetch::{parse_robots_disallows, robots_denied};
use super::super::policy::{domain_allowed, ssrf_refused};

impl BrowserEngine {
    pub(crate) async fn polite_wait(&self, url: &str) -> Result<(), BrowserError> {
        let parsed = url::Url::parse(url)
            .map_err(|e| BrowserError::ActionFailed(format!("bad url: {e}")))?;
        let host = parsed.host_str().unwrap_or("").to_string();
        if host.is_empty() {
            return Ok(());
        }
        // 策略门：SSRF 常闭 + allowlist 默认拒绝（ single choke point：所有后端导航必经）
        if ssrf_refused(&host) {
            return Err(BrowserError::SsrfRefused(host));
        }
        if let Some(list) = self.config.allowed_domains.as_ref() {
            if !domain_allowed(&host, list) {
                return Err(BrowserError::DomainDenied(host));
            }
        }
        // 429 冷却：周期内直接拒收，不再打请求
        {
            let polite = self.polite.read().await;
            if let Some(until) = polite.cooldowns.get(&host) {
                if Instant::now() < *until {
                    let left = until
                        .duration_since(Instant::now())
                        .as_millis() as u64;
                    return Err(BrowserError::CoolingDown(left));
                }
            }
        }
        if self.config.respect_robots {
            let rules = self.robots_for(&parsed).await;
            if robots_denied(&rules, parsed.path()) {
                return Err(BrowserError::ActionFailed(format!(
                    "robots.txt disallows {url}"
                )));
            }
        }
        let wait = {
            let polite = self.polite.read().await;
            match polite.last_fetch.get(&host) {
                Some(last) => {
                    Duration::from_millis(self.config.min_interval_ms)
                        .saturating_sub(last.elapsed())
                }
                None => Duration::ZERO,
            }
        };
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
        Ok(())
    }

    pub(crate) async fn record_fetch(&self, final_url: &str) {
        if let Ok(parsed) = url::Url::parse(final_url) {
            if let Some(host) = parsed.host_str() {
                let mut polite = self.polite.write().await;
                polite.last_fetch.insert(host.to_string(), Instant::now());
            }
        }
    }

    /// 429 落盘：记域名冷却（调用方返回 RateLimited，不自动重试，由 Agent 决策）
    pub(crate) async fn record_rate_limited(&self, url: &str, retry_after_secs: u64) {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                let mut polite = self.polite.write().await;
                polite.cooldowns.insert(
                    host.to_string(),
                    Instant::now() + Duration::from_secs(retry_after_secs),
                );
            }
        }
    }

    pub(crate) async fn robots_for(&self, url: &url::Url) -> Vec<String> {
        let host = url.host_str().unwrap_or("").to_string();
        {
            let polite = self.polite.read().await;
            if let Some(entry) = polite.robots.get(&host) {
                if entry.fetched_at.elapsed() < ROBOTS_TTL {
                    return entry.rules.clone();
                }
            }
        }
        // 未命中：直抓 /robots.txt（不走礼貌递归，10s 上限，失败放行）
        let robots_url = format!("{}://{}/robots.txt", url.scheme(), host);
        let resp = self
            .client
            .get(&robots_url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .ok()
            .filter(|r| r.status().is_success());
        let mut rules = Vec::new();
        if let Some(r) = resp {
            if let Ok(body) = r.text().await {
                rules = parse_robots_disallows(&body);
            }
        }
        {
            let mut polite = self.polite.write().await;
            polite.robots.insert(
                host,
                RobotsEntry {
                    rules: rules.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }
        rules
    }
}

/// 礼貌爬取状态
#[derive(Debug, Default)]
pub(crate) struct Politeness {
    last_fetch: HashMap<String, Instant>,
    robots: HashMap<String, RobotsEntry>,
    /// 域名冷却截止（429 触发）
    cooldowns: HashMap<String, Instant>,
}

#[derive(Debug, Clone)]
pub(crate) struct RobotsEntry {
    rules: Vec<String>,
    fetched_at: Instant,
}

pub(crate) const ROBOTS_TTL: Duration = Duration::from_secs(600);
