//! Agent-Reach — 智能体互联网访问 (C1)
//!
//! 吸收 github.com/Panniantong/Agent-Reach: 给 AI 智能体"看互联网的眼睛"，
//! 覆盖 Twitter/Reddit/YouTube/GitHub/Bilibili/小红书，单 CLI、零 API 费
//! (走公开页抓取而非付费 API)。提供统一多平台读取/搜索契约 stub。

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// 受支持平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReachPlatform {
    Twitter,
    Reddit,
    YouTube,
    GitHub,
    Bilibili,
    Xiaohongshu,
}

impl ReachPlatform {
    pub fn all() -> &'static [ReachPlatform] {
        &[
            ReachPlatform::Twitter,
            ReachPlatform::Reddit,
            ReachPlatform::YouTube,
            ReachPlatform::GitHub,
            ReachPlatform::Bilibili,
            ReachPlatform::Xiaohongshu,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ReachPlatform::Twitter => "twitter",
            ReachPlatform::Reddit => "reddit",
            ReachPlatform::YouTube => "youtube",
            ReachPlatform::GitHub => "github",
            ReachPlatform::Bilibili => "bilibili",
            ReachPlatform::Xiaohongshu => "xiaohongshu",
        }
    }
}

/// 平台读取/搜索结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachResult {
    pub platform: ReachPlatform,
    pub target: String,
    pub accessible: bool,
    pub note: String,
}

/// 零 API 费互联网访问契约 (Agent-Reach 抽象)
pub trait InternetReach: Send + Sync {
    fn reach(&self, platform: ReachPlatform, target: &str) -> ReachResult;
}

/// 公开页抓取访问器 (零 API 费: 拒绝付费 endpoint)
pub struct PublicScrapeReach;

impl InternetReach for PublicScrapeReach {
    fn reach(&self, platform: ReachPlatform, target: &str) -> ReachResult {
        if target.trim().is_empty() {
            return ReachResult {
                platform,
                target: target.into(),
                accessible: false,
                note: "empty target".into(),
            };
        }
        ReachResult {
            platform,
            target: target.into(),
            accessible: true,
            note: format!("scrape public page (zero api fee): {}", platform.as_str()),
        }
    }
}

/// SelfTest (T1)
pub struct AgentReachSelfTest;

impl SelfTest for AgentReachSelfTest {
    fn name(&self) -> &str {
        "nt_world_agent_reach"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let r = PublicScrapeReach;
        let res = r.reach(ReachPlatform::GitHub, "rust-lang/rust");
        if !res.accessible {
            return Err(vec!["agent_reach: github target should be reachable".into()]);
        }
        let empty = r.reach(ReachPlatform::Twitter, "");
        if empty.accessible {
            return Err(vec!["agent_reach: empty target must be inaccessible".into()]);
        }
        if ReachPlatform::all().len() != 6 {
            return Err(vec!["agent_reach: expected 6 platforms".into()]);
        }
        Ok(())
    }
}

/// 注册 Agent-Reach SelfTest
pub fn register_agent_reach_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(AgentReachSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_platforms() {
        assert_eq!(ReachPlatform::all().len(), 6);
    }

    #[test]
    fn github_reachable() {
        let r = PublicScrapeReach.reach(ReachPlatform::GitHub, "neotrix");
        assert!(r.accessible);
        assert_eq!(r.platform.as_str(), "github");
    }

    #[test]
    fn empty_target_inaccessible() {
        let r = PublicScrapeReach.reach(ReachPlatform::Reddit, "  ");
        assert!(!r.accessible);
    }
}
