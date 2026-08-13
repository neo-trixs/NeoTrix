//! nt_io::web_data::planner — 摄取规划 (输入卫生)
//!
//! 节点: nt_io::web_data::planner (L0)
//! Provides: web_data_acquisition, fetch_planning, url_hygiene

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FetchKind {
    Html,
    Markdown,
    Api,
    File,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FetchPlan {
    pub url: String,
    pub kind: FetchKind,
    pub max_bytes: usize,
    pub tags: Vec<String>,
}

/// 摄取规划器 — 校验 URL 卫生, 拒绝协议攻击/内网/非 http(s)
#[derive(Debug, Clone, Default)]
pub struct AcquisitionPlanner;

impl AcquisitionPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(&self, url: &str, kind: FetchKind) -> Result<FetchPlan, NeoTrixError> {
        if url.len() > 2048 {
            return Err(NeoTrixError::InvalidInput("URL 过长".into()));
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(NeoTrixError::InvalidInput(format!(
                "仅支持 http(s), 收到: {}",
                url
            )));
        }
        // 内网/本地地址防护 (SSRF 轻量防线)
        for bad in [
            "127.0.0.1",
            "localhost",
            "10.",
            "192.168.",
            "169.254.",
            "0.0.0.0",
        ] {
            if url.contains(bad) {
                return Err(NeoTrixError::InvalidInput(format!(
                    "疑似内网/保留地址: {}",
                    bad
                )));
            }
        }
        Ok(FetchPlan {
            url: url.into(),
            kind,
            max_bytes: 512 * 1024,
            tags: vec![],
        })
    }

    /// 批量去重: 相同 URL 只保留首个计划
    pub fn dedup(&self, plans: &[FetchPlan]) -> Vec<FetchPlan> {
        let mut seen = std::collections::HashSet::new();
        plans
            .iter()
            .filter(|p| seen.insert(p.url.clone()))
            .cloned()
            .collect()
    }
}

impl CapabilityNode for AcquisitionPlanner {
    fn node_id(&self) -> &str {
        "nt_io::web_data::planner"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "web_data_acquisition".into(),
            "fetch_planning".into(),
            "url_hygiene".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec![]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Obsidian]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for AcquisitionPlanner {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let p = AcquisitionPlanner::new();
        let ok = p
            .plan("https://example.com/doc", FetchKind::Markdown)
            .map_err(|e| vec![e.to_string()])?;
        assert_eq!(ok.kind, FetchKind::Markdown);
        assert!(
            p.plan("ftp://example.com", FetchKind::Html).is_err(),
            "非 http 应拒绝"
        );
        assert!(
            p.plan("http://localhost/admin", FetchKind::Html).is_err(),
            "内网应拒绝"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_io_web_data_planner"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_plan() {
        let p = AcquisitionPlanner::new();
        let plan = p.plan("https://example.com/a", FetchKind::Html).unwrap();
        assert_eq!(plan.url, "https://example.com/a");
        assert_eq!(plan.max_bytes, 512 * 1024);
    }

    #[test]
    fn test_rejects_non_http() {
        let p = AcquisitionPlanner::new();
        assert!(p.plan("ftp://x.com", FetchKind::Html).is_err());
        assert!(p.plan("file:///etc/passwd", FetchKind::File).is_err());
    }

    #[test]
    fn test_rejects_ssrf_targets() {
        let p = AcquisitionPlanner::new();
        for bad in [
            "http://127.0.0.1:8080",
            "http://10.0.0.1/x",
            "http://192.168.1.1/x",
            "https://localhost/x",
        ] {
            assert!(p.plan(bad, FetchKind::Html).is_err(), "应拒绝: {bad}");
        }
    }

    #[test]
    fn test_dedup() {
        let p = AcquisitionPlanner::new();
        let a = p.plan("https://a.com/1", FetchKind::Html).unwrap();
        let b = p.plan("https://a.com/1", FetchKind::Markdown).unwrap();
        let c = p.plan("https://b.com/2", FetchKind::Html).unwrap();
        let uniq = p.dedup(&[a, b, c]);
        assert_eq!(uniq.len(), 2);
    }
}
