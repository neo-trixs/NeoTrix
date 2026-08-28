//! 生成式视频技能 (NT-IO)
//!
//! 吸收源: github.com/Vincentwei1021/video-shotcraft
//! 成熟度: C1 (unit-tested stub, 无 Remotion 运行时集成)
//!
//! 核心能力: 把电影级产品视频编排为 shot card 驱动的 Remotion 渲染管线。
//! 本 stub 负责 shot card 选取校验与 Remotion 渲染规格生成。

use crate::core::nt_core_self_test::SelfTest;

/// Shot card: 一个镜头卡片 (来自 152 卡库), 含时长与序号。
#[derive(Debug, Clone, PartialEq)]
pub struct ShotCard {
    pub index: usize,
    pub duration_sec: f64,
    pub label: String,
}

/// 生成式视频技能 trait — shot card 选取 + Remotion 渲染接口 stub。
pub trait VideoShotcraft: Send + Sync {
    /// 选取有效 shot card: 序号非空、时长 > 0; 返回选取数量, 非法卡返回 None。
    fn select_cards(&self, cards: &[ShotCard]) -> Option<usize>;
    /// 生成 Remotion 渲染规格: 给定总时长返回 render 串, 时长 <= 0 返回 None。
    fn remotion_render(&self, total_duration_sec: f64) -> Option<String>;
}

/// 默认实现。
#[derive(Default)]
pub struct VideoShotcraftEngine;

impl VideoShotcraft for VideoShotcraftEngine {
    fn select_cards(&self, cards: &[ShotCard]) -> Option<usize> {
        if cards.is_empty() {
            return None;
        }
        let all_valid = cards.iter().all(|c| !c.label.trim().is_empty() && c.duration_sec > 0.0);
        if all_valid { Some(cards.len()) } else { None }
    }

    fn remotion_render(&self, total_duration_sec: f64) -> Option<String> {
        if total_duration_sec <= 0.0 {
            None
        } else {
            Some(format!("remotion://render?duration={}", total_duration_sec))
        }
    }
}

/// T1 SelfTest: 验证 shot 选取与渲染规格存在且生效。
#[derive(Default)]
pub struct VideoShotcraftSelfTest;

impl SelfTest for VideoShotcraftSelfTest {
    fn name(&self) -> &str {
        "nt_io_video_shotcraft"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = VideoShotcraftEngine;
        let cards = vec![
            ShotCard { index: 1, duration_sec: 3.0, label: "hook".into() },
            ShotCard { index: 2, duration_sec: 5.0, label: "reveal".into() },
        ];
        match v.select_cards(&cards) {
            Some(2) => {
                match v.remotion_render(8.0) {
                    Some(s) if s.contains("8") => Ok(()),
                    _ => Err(vec!["nt_io_video_shotcraft: render spec failed".into()]),
                }
            }
            _ => Err(vec!["nt_io_video_shotcraft: valid cards rejected".into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_valid_cards() {
        let v = VideoShotcraftEngine;
        let cards = vec![ShotCard { index: 1, duration_sec: 2.0, label: "a".into() }];
        assert_eq!(v.select_cards(&cards), Some(1));
    }

    #[test]
    fn test_rejects_bad_cards() {
        let v = VideoShotcraftEngine;
        assert_eq!(v.select_cards(&[]), None);
        let bad = vec![ShotCard { index: 1, duration_sec: 0.0, label: "a".into() }];
        assert_eq!(v.select_cards(&bad), None);
    }

    #[test]
    fn test_remotion_render_spec() {
        let v = VideoShotcraftEngine;
        assert_eq!(v.remotion_render(0.0), None);
        let s = v.remotion_render(12.5).unwrap();
        assert!(s.starts_with("remotion://render"));
        assert!(s.contains("12.5"));
    }
}
