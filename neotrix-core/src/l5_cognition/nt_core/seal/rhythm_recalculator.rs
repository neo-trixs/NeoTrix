//! 节奏重算模块
//! 基于目标时长的节段重算和分配
//! 服务于动态漫节奏设计和分镜脚本生成

use serde::{Serialize, Deserialize};

// 叙事类型定义在 core 层，此处 re-export 保持 L5 内部向后兼容
pub use crate::core::nt_core_narrative_types::{SegmentType, SegmentData};

// ============================================================================
// 节段分配结果

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _AllocatedSegment {
    pub r#type: SegmentType,
    pub allocated_length: f32,
    pub content_priority: f32,
    pub is_core_scuang: bool,
    pub requires_strong_dynamic: bool,
}

// ============================================================================
// 默认分段比例

pub const DEFAULT_SEGMENT_RATIOS: [f32; 4] = [0.3, 0.4, 0.3, 0.0];
pub const QUICK_SEGMENT_RATIOS: [f32; 3] = [0.2, 0.3, 0.5];

pub const SERIES_10COL_RATIOS: [[f32; 4]; 10] = [
    [0.4, 0.3, 0.3, 0.0],
    [0.3, 0.4, 0.3, 0.0],
    [0.2, 0.3, 0.5, 0.0],
    [0.25, 0.35, 0.4, 0.0],
    [0.25, 0.35, 0.4, 0.0],
    [0.25, 0.35, 0.4, 0.0],
    [0.3, 0.4, 0.3, 0.0],
    [0.3, 0.4, 0.3, 0.0],
    [0.3, 0.3, 0.4, 0.0],
    [0.2, 0.3, 0.5, 0.0],
];

// ============================================================================
// 核心重算函数

pub(crate) fn _recalculate_rhythm_segments(
    default_duration: f32,
    target_duration: f32,
    segments: &[SegmentData],
) -> Vec<_AllocatedSegment> {
    let ratio = target_duration / default_duration;
    segments.iter().map(|seg| _AllocatedSegment {
        r#type: seg.r#type,
        allocated_length: seg.base_length * ratio.powf(0.8),
        content_priority: seg.content_priority,
        is_core_scuang: seg.is_core_scuang,
        requires_strong_dynamic: seg.is_core_scuang && seg.base_length * ratio.powf(0.8) > 30.0,
    }).collect()
}

pub(crate) fn _generate_default_segments(default_duration: f32) -> Vec<SegmentData> {
    let ratios = DEFAULT_SEGMENT_RATIOS;
    let total_ratio: f32 = ratios.iter().sum();
    (0..ratios.len())
        .map(|i| SegmentData {
            r#type: match i {
                0 => SegmentType::Setup,
                1 => SegmentType::Conflict,
                2 => SegmentType::Climax,
                _ => SegmentType::Transition,
            },
            base_length: default_duration * ratios[i] / total_ratio,
            content_priority: 1.0,
            is_core_scuang: i == 2,
        })
        .collect()
}

pub(crate) fn _generate_quick_segments(default_duration: f32) -> Vec<SegmentData> {
    let ratios = QUICK_SEGMENT_RATIOS;
    let total_ratio: f32 = ratios.iter().sum();
    (0..ratios.len())
        .map(|i| SegmentData {
            r#type: match i {
                0 => SegmentType::Setup,
                1 => SegmentType::Conflict,
                _ => SegmentType::Climax,
            },
            base_length: default_duration * ratios[i] / total_ratio,
            content_priority: 1.0,
            is_core_scuang: i == 2,
        })
        .collect()
}

// ============================================================================
// 系列剧节奏布局

pub(crate) fn _get_series_segment_raters(episode: usize) -> [f32; 4] {
    if episode > 0 && episode <= 10 {
        SERIES_10COL_RATIOS[episode - 1]
    } else {
        DEFAULT_SEGMENT_RATIOS
    }
}

// ============================================================================
// 验证与工具

pub(crate) fn _validate_segments(segments: &[SegmentData], total_duration: f32) -> bool {
    let allocated_total: f32 = segments.iter().map(|s| s.base_length).sum();
    let tolerance = 0.1;
    let length_ok = (allocated_total - total_duration).abs() / total_duration < tolerance;
    let content_ok = segments.iter().all(|s| s.base_length > 0.0 && s.content_priority > 0.0);
    length_ok && content_ok
}

pub(crate) fn _get_core_scuang_index(segments: &[SegmentData]) -> Option<usize> {
    segments.iter().position(|s| s.is_core_scuang)
}

pub(crate) fn _check_emotion_beat_interval(segments: &[SegmentData], _total_duration: f32) -> bool {
    let core_count = segments.iter().filter(|s| s.is_core_scuang).count();
    if core_count == 0 {
        return false;
    }
    let has_micro = segments.iter().any(|s| !s.is_core_scuang && s.content_priority > 0.5);
    let has_middle = segments.iter().any(|s| !s.is_core_scuang && s.content_priority > 0.3);
    core_count >= 1 && (has_micro || has_middle)
}

// ============================================================================
// 测试模块

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_recalculate_rhythm_segments() {
        let default_dur = 240.0;
        let target_dur = 300.0;
        
        let segments = vec![
            SegmentData { r#type: SegmentType::Setup, base_length: 60.0, content_priority: 1.0, is_core_scuang: false },
            SegmentData { r#type: SegmentType::Conflict, base_length: 80.0, content_priority: 1.0, is_core_scuang: false },
            SegmentData { r#type: SegmentType::Climax, base_length: 80.0, content_priority: 1.0, is_core_scuang: true },
        ];
        
        let allocated = _recalculate_rhythm_segments(default_dur, target_dur, &segments);
        assert_eq!(allocated.len(), 3);
        let expected_factor = (target_dur / default_dur).powf(0.8);
        assert!(allocated[0].allocated_length > 0.0);
        assert!(allocated[2].allocated_length > 0.0);
        assert!(allocated[2].requires_strong_dynamic);
    }
    
    #[test]
    fn test_generate_default_segments() {
        let segments = _generate_default_segments(240.0);
        assert_eq!(segments.len(), 4);
        let total: f32 = segments.iter().map(|s| s.base_length).sum();
        assert!((total - 240.0).abs() / 240.0 < 0.1);
        assert_eq!(segments[0].r#type, SegmentType::Setup);
        assert_eq!(segments[1].r#type, SegmentType::Conflict);
        assert_eq!(segments[2].r#type, SegmentType::Climax);
        assert_eq!(segments[3].r#type, SegmentType::Transition);
        assert!(segments[2].is_core_scuang);
    }
    
    #[test]
    fn test_quick_segments() {
        let segments = _generate_quick_segments(180.0);
        assert_eq!(segments.len(), 3);
        let total: f32 = segments.iter().map(|s| s.base_length).sum();
        assert!((total - 180.0).abs() / 180.0 < 0.1);
        assert!(segments[2].is_core_scuang);
    }
    
    #[test]
    fn test_validate_segments() {
        let segments = vec![
            SegmentData { r#type: SegmentType::Setup, base_length: 60.0, content_priority: 1.0, is_core_scuang: false },
            SegmentData { r#type: SegmentType::Conflict, base_length: 80.0, content_priority: 1.0, is_core_scuang: false },
            SegmentData { r#type: SegmentType::Climax, base_length: 100.0, content_priority: 1.0, is_core_scuang: true },
        ];
        assert!(_validate_segments(&segments, 240.0));
    }
    
    #[test]
    fn test_get_core_scuang_index() {
        let segments = vec![
            SegmentData { r#type: SegmentType::Setup, base_length: 50.0, content_priority: 0.5, is_core_scuang: false },
            SegmentData { r#type: SegmentType::Climax, base_length: 80.0, content_priority: 1.0, is_core_scuang: true },
            SegmentData { r#type: SegmentType::Transition, base_length: 70.0, content_priority: 0.5, is_core_scuang: false },
        ];
        let idx = _get_core_scuang_index(&segments);
        assert_eq!(idx, Some(1));
    }
    
    #[test]
    fn test_series_ratios() {
        for i in 1..=10 {
            let ratios = _get_series_segment_raters(i);
            let total: f32 = ratios.iter().sum();
            assert!((total - 1.0).abs() < 0.001, "Episode {} ratios sum to {}", i, total);
        }
    }
    
    #[test]
    fn test_emotion_beat_interval() {
        let segments = vec![
            SegmentData { r#type: SegmentType::Setup, base_length: 60.0, content_priority: 0.8, is_core_scuang: false },
            SegmentData { r#type: SegmentType::Climax, base_length: 80.0, content_priority: 1.0, is_core_scuang: true },
            SegmentData { r#type: SegmentType::Transition, base_length: 60.0, content_priority: 0.6, is_core_scuang: false },
        ];
        assert!(_check_emotion_beat_interval(&segments, 200.0));
    }
}