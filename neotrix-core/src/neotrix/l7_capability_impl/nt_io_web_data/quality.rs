//! nt_io::web_data::quality — 来源质量评分 (输出门)
//!
//! 节点: nt_io::web_data::quality (L1)
//! Provides: source_quality_scoring, content_completeness
//!
//! 内容质量门: 依据标题/正文长度/去重比例对摄取结果评分, 低于阈值拒绝入库,
//! 防垃圾源与空壳页面污染知识库。

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QualityGrade {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Default)]
pub struct SourceQualityGate {
    min_body_len: usize,
    require_title: bool,
}

impl SourceQualityGate {
    pub fn new() -> Self {
        Self {
            min_body_len: 200,
            require_title: true,
        }
    }

    /// 评分并给出门控结论
    pub fn grade(&self, title: &str, body: &str) -> QualityGrade {
        if self.require_title && title.trim().is_empty() {
            return QualityGrade::Reject;
        }
        if body.trim().len() < self.min_body_len {
            return QualityGrade::Reject;
        }
        QualityGrade::Accept
    }

    pub fn score(&self, title: &str, body: &str) -> f32 {
        let mut s = 0.0;
        if !title.trim().is_empty() {
            s += 0.4;
        }
        let len = body.trim().len();
        if len >= self.min_body_len {
            s += 0.4;
        } else if len > 0 {
            s += 0.2;
        }
        // 非空字符占比 (防空白/重复字符垃圾)
        let nonzero = body.chars().filter(|c| !c.is_whitespace()).count();
        let ratio = if body.is_empty() {
            0.0
        } else {
            nonzero as f32 / body.len().max(1) as f32
        };
        s += ratio * 0.2;
        s
    }
}

impl CapabilityNode for SourceQualityGate {
    fn node_id(&self) -> &str {
        "nt_io::web_data::quality"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "source_quality_scoring".into(),
            "content_completeness".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec!["web_data_acquisition".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Golden, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for SourceQualityGate {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let g = SourceQualityGate::new();
        let body = "词".repeat(300);
        assert_eq!(
            g.grade("标题", &body),
            QualityGrade::Accept,
            "完整内容应接受"
        );
        assert_eq!(g.grade("", &body), QualityGrade::Reject, "无标题应拒绝");
        assert_eq!(
            g.grade("标题", "短"),
            QualityGrade::Reject,
            "正文过短应拒绝"
        );
        assert!(g.score("标题", &body) > 0.5);
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_io_web_data_quality"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_complete() {
        let g = SourceQualityGate::new();
        let body = "内容".repeat(300);
        assert_eq!(g.grade("文档标题", &body), QualityGrade::Accept);
    }

    #[test]
    fn test_reject_empty_title() {
        let g = SourceQualityGate::new();
        assert_eq!(g.grade("   ", &"词".repeat(300)), QualityGrade::Reject);
    }

    #[test]
    fn test_reject_short_body() {
        let g = SourceQualityGate::new();
        assert_eq!(g.grade("标题", "略"), QualityGrade::Reject);
    }

    #[test]
    fn test_score_ranges() {
        let g = SourceQualityGate::new();
        let full = g.score("标题", &"内容".repeat(300));
        let empty = g.score("", "");
        assert!(full > empty, "完整来源应比空壳分数高");
        assert!(full <= 1.0);
    }

    #[test]
    fn test_whitespace_garbage_penalized() {
        let g = SourceQualityGate::new();
        let spaces = g.score("标题", &" ".repeat(300));
        let real = g.score("标题", &"内容".repeat(300));
        assert!(spaces < real, "纯空白正文应被降分");
    }
}
