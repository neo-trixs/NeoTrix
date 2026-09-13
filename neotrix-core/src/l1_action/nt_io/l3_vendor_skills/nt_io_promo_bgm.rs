//! BGM 推广技能节点 (NT-IO)
//!
//! 吸收源: github.com/whyubel1eve/promo-bgm-skill
//! 成熟度: C1 (unit-tested stub, 无外部平台集成)
//!
//! 核心能力: 将音乐曲目元数据转换为跨平台推广文案与标签组合, 提升曝光分发效率。

use crate::core::nt_core_self_test::SelfTest;

/// BGM 推广器 trait — 把曲目信息映射为推广文案与标签集。
pub trait BgmPromoter: Send + Sync {
    /// 生成一条推广文案。
    fn compose_promo(&self, track: &str, mood: &str) -> String;
    /// 为曲目生成 hashtag 列表 (不含 # 前缀)。
    fn derive_tags(&self, track: &str, mood: &str) -> Vec<String>;
}

/// 默认实现: 模板化文案 + 基于 mood/track 的标签派生。
#[derive(Default)]
pub struct BgmPromoEngine;

impl BgmPromoter for BgmPromoEngine {
    fn compose_promo(&self, track: &str, mood: &str) -> String {
        format!("Now playing `{}` — a {} loop, perfect for focus & chill.", track, mood)
    }

    fn derive_tags(&self, track: &str, mood: &str) -> Vec<String> {
        let mut tags = vec!["bgm".to_string(), "music".to_string(), mood.to_lowercase()];
        tags.push(track.to_lowercase().replace(' ', "_"));
        tags
    }
}

/// T1 SelfTest: 验证推广引擎存在并能产出文案与标签。
#[derive(Default)]
pub struct PromoBgmSelfTest;

impl SelfTest for PromoBgmSelfTest {
    fn name(&self) -> &str {
        "nt_io_promo_bgm"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let e = BgmPromoEngine;
        let promo = e.compose_promo("Midnight", "lofi");
        let tags = e.derive_tags("Midnight", "lofi");
        if promo.contains("Midnight") && tags.iter().any(|t| t == "lofi") {
            Ok(())
        } else {
            Err(vec!["nt_io_promo_bgm: promo/tags missing expected content".into()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_promo_mentions_track() {
        let e = BgmPromoEngine;
        let p = e.compose_promo("Aurora", "ambient");
        assert!(p.contains("Aurora"));
        assert!(p.contains("ambient"));
    }

    #[test]
    fn test_derive_tags_includes_mood_and_track() {
        let e = BgmPromoEngine;
        let tags = e.derive_tags("Deep Sea", "calm");
        assert!(tags.contains(&"calm".to_string()));
        assert!(tags.contains(&"deep_sea".to_string()));
        assert!(tags.contains(&"bgm".to_string()));
    }

    #[test]
    fn test_promo_deterministic() {
        let e = BgmPromoEngine;
        assert_eq!(e.compose_promo("X", "y"), e.compose_promo("X", "y"));
    }
}
