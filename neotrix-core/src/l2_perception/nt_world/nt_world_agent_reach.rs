//! Agent-Reach — 智能体互联网访问 (C1)
//!
//! 吸收 github.com/Panniantong/Agent-Reach: 给 AI 智能体"看互联网的眼睛"，
//! 覆盖 Twitter/Reddit/YouTube/GitHub/Bilibili/小红书，单 CLI、零 API 费
//! (走公开页抓取而非付费 API)。提供统一多平台读取/搜索契约 stub。

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// 受支持平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _ReachPlatform {
    Twitter,
    Reddit,
    YouTube,
    GitHub,
    Bilibili,
    Xiaohongshu,
}

impl _ReachPlatform {
    pub fn all() -> &'static [_ReachPlatform] {
        &[
            _ReachPlatform::Twitter,
            _ReachPlatform::Reddit,
            _ReachPlatform::YouTube,
            _ReachPlatform::GitHub,
            _ReachPlatform::Bilibili,
            _ReachPlatform::Xiaohongshu,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            _ReachPlatform::Twitter => "twitter",
            _ReachPlatform::Reddit => "reddit",
            _ReachPlatform::YouTube => "youtube",
            _ReachPlatform::GitHub => "github",
            _ReachPlatform::Bilibili => "bilibili",
            _ReachPlatform::Xiaohongshu => "xiaohongshu",
        }
    }
}

/// 平台读取/搜索结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _ReachResult {
    pub platform: _ReachPlatform,
    pub target: String,
    pub accessible: bool,
    pub note: String,
}



/// SelfTest (T1)
pub struct _AgentReachSelfTest;

impl SelfTest for _AgentReachSelfTest {
    fn name(&self) -> &str {
        "nt_world_agent_reach"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if _ReachPlatform::all().len() != 6 {
            return Err(vec!["agent_reach: expected 6 platforms".into()]);
        }
        Ok(())
    }
}

/// 注册 Agent-Reach SelfTest
pub fn _register_agent_reach_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(_AgentReachSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_platforms() {
        assert_eq!(_ReachPlatform::all().len(), 6);
    }

    #[test]
    fn github_reachable_reports_not_wired() {
        let r = _PublicScrapeReach.reach(_ReachPlatform::GitHub, "neotrix");
        // C1 stub: not wired — returns accessible=false with explanation
        assert!(!r.accessible);
        assert!(r.note.contains("not wired"));
        assert_eq!(r.platform.as_str(), "github");
    }

    #[test]
    fn empty_target_inaccessible() {
        let r = _PublicScrapeReach.reach(_ReachPlatform::Reddit, "  ");
        assert!(!r.accessible);
    }
}
