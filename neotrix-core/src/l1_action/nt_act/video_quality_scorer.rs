//! _VideoQualityScorer — 视频质量评分器
//!
//! 自动化质量评分 + 时序一致性检测 + 音视频同步检查。
//! 支持帧级、片段级、整体级多维度评分。

use std::collections::HashMap;
use std::time::Instant;

/// 质量维度
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum _QualityDimension {
    /// 时序一致性 (帧间稳定性)
    TemporalConsistency,
    /// 视觉保真度
    VisualFidelity,
    /// 音视频同步
    AudioVideoSync,
    /// 分辨率质量
    ResolutionQuality,
    /// 色彩一致性
    ColorConsistency,
    /// 运动平滑度
    MotionSmoothness,
    /// 整体质量
    Overall,
}

/// 帧质量评估
#[derive(Debug, Clone)]
pub(crate) struct _FrameQuality {
    /// 帧索引
    pub index: u32,
    /// 时间戳 (毫秒)
    pub timestamp_ms: f64,
    /// 各维度分数
    pub scores: HashMap<_QualityDimension, f64>,
    /// 帧大小 (字节)
    pub size_bytes: u32,
    /// 是否是关键帧
    pub is_keyframe: bool,
}

/// 片段质量评估
#[derive(Debug, Clone)]
pub(crate) struct _SegmentQuality {
    /// 片段 ID
    pub segment_id: String,
    /// 帧范围
    pub frame_range: (u32, u32),
    /// 各维度平均分数
    pub avg_scores: HashMap<_QualityDimension, f64>,
    /// 帧间一致性 (0-1)
    pub inter_frame_consistency: f64,
    /// 整体分数
    pub overall_score: f64,
}

/// 整体质量报告
#[derive(Debug, Clone)]
pub(crate) struct _QualityReport {
    /// 视频 ID
    pub video_id: String,
    /// 总帧数
    pub total_frames: u32,
    /// 各维度最终分数
    pub final_scores: HashMap<_QualityDimension, f64>,
    /// 整体质量分数
    pub overall_score: f64,
    /// 质量等级
    pub quality_grade: _QualityGrade,
    /// 问题列表
    pub issues: Vec<QualityIssue>,
    /// 评估时间
    pub evaluated_at: Instant,
}

/// 质量等级
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum _QualityGrade {
    Excellent,
    Good,
    Fair,
    Poor,
    Unacceptable,
}

/// 质量问题
#[derive(Debug, Clone)]
pub struct QualityIssue {
    /// 问题类型
    pub issue_type: IssueType,
    /// 严重程度
    pub severity: IssueSeverity,
    /// 描述
    pub description: String,
    /// 帧索引
    pub frame_index: Option<u32>,
    /// 时间戳
    pub timestamp_ms: Option<f64>,
}

/// 问题类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueType {
    /// 帧跳变
    FrameJump,
    /// 色彩偏移
    ColorShift,
    /// 模糊
    Blur,
    /// 音视频不同步
    AudioDesync,
    /// 分辨率下降
    ResolutionDrop,
    /// 运动不连贯
    MotionDiscontinuity,
}

/// 严重程度
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    Major,
    Minor,
    Cosmetic,
}

/// 视频质量评分器
pub(crate) struct _VideoQualityScorer {
    /// 帧质量历史
    frame_qualities: Vec<_FrameQuality>,
    /// 片段质量
    segment_qualities: Vec<_SegmentQuality>,
    /// 配置
    config: _ScorerConfig,
}

/// 评分器配置
#[derive(Debug, Clone)]
pub(crate) struct _ScorerConfig {
    /// 最小帧分数阈值
    pub min_frame_score: f64,
    /// 时序一致性阈值
    pub temporal_consistency_threshold: f64,
    /// 音视频同步阈值 (ms)
    pub av_sync_threshold_ms: f64,
    /// 问题检测灵敏度
    pub issue_sensitivity: f64,
}

impl Default for _ScorerConfig {
    fn default() -> Self {
        Self {
            min_frame_score: 0.5,
            temporal_consistency_threshold: 0.8,
            av_sync_threshold_ms: 40.0,
            issue_sensitivity: 0.7,
        }
    }
}

impl _VideoQualityScorer {
    pub fn new(config: _ScorerConfig) -> Self {
        Self {
            frame_qualities: Vec::new(),
            segment_qualities: Vec::new(),
            config,
        }
    }

    /// 评估单帧
    pub(crate) fn _evaluate_frame(&mut self, index: u32, timestamp_ms: f64, size_bytes: u32, is_keyframe: bool, scores: HashMap<_QualityDimension, f64>) -> _FrameQuality {
        let quality = _FrameQuality {
            index,
            timestamp_ms,
            scores,
            size_bytes,
            is_keyframe,
        };

        self.frame_qualities.push(quality.clone());
        quality
    }

    /// 评估片段
    pub(crate) fn _evaluate_segment(&mut self, segment_id: &str, frame_start: u32, frame_end: u32) -> _SegmentQuality {
        let segment_frames: Vec<_FrameQuality> = self.frame_qualities.iter()
            .filter(|f| f.index >= frame_start && f.index <= frame_end)
            .cloned()
            .collect();

        let mut avg_scores = HashMap::new();
        for dim in &[
            _QualityDimension::TemporalConsistency,
            _QualityDimension::VisualFidelity,
            _QualityDimension::AudioVideoSync,
            _QualityDimension::ResolutionQuality,
            _QualityDimension::ColorConsistency,
            _QualityDimension::MotionSmoothness,
        ] {
            let sum: f64 = segment_frames.iter()
                .filter_map(|f| f.scores.get(dim))
                .sum();
            let count = segment_frames.iter()
                .filter(|f| f.scores.contains_key(dim))
                .count();
            if count > 0 {
                avg_scores.insert(dim.clone(), sum / count as f64);
            }
        }

        let inter_frame_consistency = self.calculate_inter_frame_consistency(&segment_frames);
        let overall_score = avg_scores.values().sum::<f64>() / avg_scores.len() as f64;

        let quality = _SegmentQuality {
            segment_id: segment_id.to_string(),
            frame_range: (frame_start, frame_end),
            avg_scores,
            inter_frame_consistency,
            overall_score,
        };

        self.segment_qualities.push(quality.clone());
        quality
    }

    /// 计算帧间一致性
    fn calculate_inter_frame_consistency(&self, frames: &[_FrameQuality]) -> f64 {
        if frames.len() < 2 {
            return 1.0;
        }

        let mut total_diff = 0.0;
        for i in 1..frames.len() {
            if let (Some(score_a), Some(score_b)) = (
                frames[i-1].scores.get(&_QualityDimension::TemporalConsistency),
                frames[i].scores.get(&_QualityDimension::TemporalConsistency),
            ) {
                total_diff += (score_a - score_b).abs();
            }
        }

        1.0 - (total_diff / (frames.len() - 1) as f64)
    }

    /// 生成整体质量报告
    pub fn generate_report(&self, video_id: &str) -> _QualityReport {
        let mut final_scores = HashMap::new();
        let mut issues = Vec::new();

        // 计算各维度最终分数
        for dim in &[
            _QualityDimension::TemporalConsistency,
            _QualityDimension::VisualFidelity,
            _QualityDimension::AudioVideoSync,
            _QualityDimension::ResolutionQuality,
            _QualityDimension::ColorConsistency,
            _QualityDimension::MotionSmoothness,
        ] {
            let sum: f64 = self.frame_qualities.iter()
                .filter_map(|f| f.scores.get(dim))
                .sum();
            let count = self.frame_qualities.iter()
                .filter(|f| f.scores.contains_key(dim))
                .count();
            if count > 0 {
                final_scores.insert(dim.clone(), sum / count as f64);
            }
        }

        // 计算整体分数
        let overall_score = if final_scores.is_empty() {
            0.0
        } else {
            final_scores.values().sum::<f64>() / final_scores.len() as f64
        };

        // 检测问题
        for frame in &self.frame_qualities {
            if let Some(score) = frame.scores.get(&_QualityDimension::TemporalConsistency) {
                if *score < self.config.min_frame_score {
                    issues.push(QualityIssue {
                        issue_type: IssueType::FrameJump,
                        severity: IssueSeverity::Major,
                        description: format!("Frame {} has low temporal consistency: {:.2}", frame.index, score),
                        frame_index: Some(frame.index),
                        timestamp_ms: Some(frame.timestamp_ms),
                    });
                }
            }
        }

        // 确定质量等级
        let quality_grade = match overall_score {
            s if s >= 0.9 => _QualityGrade::Excellent,
            s if s >= 0.75 => _QualityGrade::Good,
            s if s >= 0.6 => _QualityGrade::Fair,
            s if s >= 0.4 => _QualityGrade::Poor,
            _ => _QualityGrade::Unacceptable,
        };

        _QualityReport {
            video_id: video_id.to_string(),
            total_frames: self.frame_qualities.len() as u32,
            final_scores,
            overall_score,
            quality_grade,
            issues,
            evaluated_at: Instant::now(),
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> _ScorerStats {
        _ScorerStats {
            total_frames: self.frame_qualities.len() as u32,
            total_segments: self.segment_qualities.len() as u32,
            avg_frame_score: self.frame_qualities.iter()
                .filter_map(|f| f.scores.get(&_QualityDimension::Overall))
                .sum::<f64>() / self.frame_qualities.len() as f64,
        }
    }
}

impl Default for _VideoQualityScorer {
    fn default() -> Self {
        Self::new(_ScorerConfig::default())
    }
}

/// 评分统计
#[derive(Debug, Clone, Default)]
pub(crate) struct _ScorerStats {
    pub total_frames: u32,
    pub total_segments: u32,
    pub avg_frame_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_frame() {
        let mut scorer = _VideoQualityScorer::default();
        let mut scores = HashMap::new();
        scores.insert(_QualityDimension::TemporalConsistency, 0.9);
        scores.insert(_QualityDimension::VisualFidelity, 0.85);

        let quality = scorer._evaluate_frame(0, 0.0, 1024, true, scores);
        assert_eq!(quality.index, 0);
    }

    #[test]
    fn test_generate_report() {
        let mut scorer = _VideoQualityScorer::default();
        let mut scores = HashMap::new();
        scores.insert(_QualityDimension::Overall, 0.85);

        scorer._evaluate_frame(0, 0.0, 1024, true, scores.clone());
        scorer._evaluate_frame(1, 33.33, 1024, false, scores);

        let report = scorer.generate_report("test-video");
        assert_eq!(report.total_frames, 2);
    }
}
