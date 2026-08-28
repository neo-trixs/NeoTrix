//! Pindo — Pindo 网站监测变更检测 (C2)
//!
//! 吸收 github.com/LunarXuan/Pindo: 针对 Pindo 站点的监测与变更检测。
//! 复用 worldmonitor 的 SiteChangeMonitor 模式 (hash 比对 + 索引)，
//! 以 Pindo 专属 trait 封装。C2 接线: 新增 `fetch_site` 真实 reqwest 同步
//! HTTP 调用，`check_target` 抓取站点内容并比对历史 hash 检测变更。
//! 监测端点 token 优先读 `PINDO_API_TOKEN` 环境变量，回退 `NeoTrixConfig.api_key`。

use crate::config::NeoTrixConfig;
use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// Pindo 监测目标
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PindoTarget {
    pub site: String,
    pub last_hash: String,
}

/// Pindo 监测变更检测契约 — 复用 monitor 模式
pub trait PindoMonitor {
    /// 计算站点内容 hash
    fn hash(&self, content: &str) -> String;
    /// 比对目标历史 hash，返回是否发生变更
    fn changed(&self, target: &PindoTarget, current: &str) -> bool;
    /// 真实抓取站点内容 (C2: reqwest 同步调用)
    fn fetch_site(&self, site: &str) -> String;
    /// 抓取站点并比对历史 hash，返回是否变更 (C2)
    fn check_target(&self, target: &PindoTarget) -> bool;
}

/// 默认实现 — 复用 worldmonitor 的 hash 思路
#[derive(Default)]
pub struct PindoWatcher;

fn pindo_hash(content: &str) -> String {
    let mut h: u64 = 1469598103934665603;
    for b in content.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    format!("{:x}", h)
}

impl PindoMonitor for PindoWatcher {
    fn hash(&self, content: &str) -> String {
        pindo_hash(content)
    }

    fn changed(&self, target: &PindoTarget, current: &str) -> bool {
        target.last_hash != self.hash(current)
    }

    fn fetch_site(&self, site: &str) -> String {
        let client = reqwest::blocking::Client::new();
        let mut builder = client.get(site);
        if let Some(token) = resolve_api_token() {
            builder = builder.bearer_auth(token);
        }
        match builder.send() {
            Ok(resp) => resp.text().unwrap_or_default(),
            Err(_) => String::new(),
        }
    }

    fn check_target(&self, target: &PindoTarget) -> bool {
        let content = self.fetch_site(&target.site);
        self.changed(target, &content)
    }
}

/// 解析监测 token: 优先 `PINDO_API_TOKEN` 环境变量，回退 `NeoTrixConfig.api_key`
fn resolve_api_token() -> Option<String> {
    if let Ok(k) = std::env::var("PINDO_API_TOKEN") {
        if !k.is_empty() {
            return Some(k);
        }
    }
    NeoTrixConfig::load().api_key
}

/// SelfTest (T1): Pindo 变更检测存在性 — 离线 (纯 hash 比对)
pub struct PindoSelfTest;

impl SelfTest for PindoSelfTest {
    fn name(&self) -> &str {
        "nt_world_pindo"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let w = PindoWatcher;
        let h = w.hash("baseline");
        let target = PindoTarget {
            site: "pindo".into(),
            last_hash: h.clone(),
        };
        if w.changed(&target, "baseline") {
            return Err(vec!["pindo: identical content should not change".into()]);
        }
        if !w.changed(&target, "changed") {
            return Err(vec!["pindo: differing content should change".into()]);
        }
        Ok(())
    }
}

/// 注册 Pindo SelfTest
pub fn register_pindo_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(PindoSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_stable() {
        let w = PindoWatcher;
        assert_eq!(w.hash("x"), w.hash("x"));
        assert_ne!(w.hash("x"), w.hash("y"));
    }

    #[test]
    fn test_unchanged_when_same() {
        let w = PindoWatcher;
        let h = w.hash("same");
        let target = PindoTarget { site: "s".into(), last_hash: h };
        assert!(!w.changed(&target, "same"));
    }

    #[test]
    fn test_changed_when_differs() {
        let w = PindoWatcher;
        let target = PindoTarget { site: "s".into(), last_hash: w.hash("a") };
        assert!(w.changed(&target, "b"));
        let t = PindoSelfTest;
        assert_eq!(t.name(), "nt_world_pindo");
        assert!(t.self_test().is_ok());
    }

    #[test]
    #[ignore = "requires network access"]
    fn test_fetch_site_real() {
        let w = PindoWatcher;
        let content = w.fetch_site("https://example.com");
        assert!(!content.is_empty());
    }
}
