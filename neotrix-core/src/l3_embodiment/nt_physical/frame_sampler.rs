//! FrameSampler — 帧采样器
//!
//! 视频帧采样和去重，支持均匀采样、关键帧检测、场景变化检测。
//! 优化视频处理性能，减少冗余计算。

use std::time::Instant;

/// 采样策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SamplingStrategy {
    /// 均匀采样 (每N帧取1帧)
    Uniform,
    /// 关键帧采样 (基于I帧)
    Keyframe,
    /// 场景变化采样 (基于帧差异)
    SceneChange,
    /// 自适应采样 (根据内容复杂度)
    Adaptive,
}

/// 帧信息
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// 帧索引
    pub index: u32,
    /// 时间戳 (毫秒)
    pub timestamp_ms: f64,
    /// 帧大小 (字节)
    pub size_bytes: u32,
    /// 是否是关键帧
    pub is_keyframe: bool,
    /// 帧差异分数 (0-1)
    pub diff_score: f64,
}

/// 采样结果
#[derive(Debug, Clone)]
pub struct SampledFrame {
    pub original_index: u32,
    pub reason: String,
}

/// 帧采样器
pub struct FrameSampler {
    /// 采样策略
    strategy: SamplingStrategy,
    /// 采样率 (0-1)
    sample_rate: f64,
    /// 关键帧阈值
    keyframe_threshold: f64,
    /// 场景变化阈值
    scene_change_threshold: f64,
    /// 统计信息
    stats: SamplingStats,
}

impl FrameSampler {
    pub fn new(strategy: SamplingStrategy, sample_rate: f64) -> Self {
        Self {
            strategy,
            sample_rate,
            keyframe_threshold: 0.8,
            scene_change_threshold: 0.3,
            stats: SamplingStats::default(),
        }
    }

    /// 采样帧
    pub fn sample(&mut self, frames: &[FrameInfo]) -> Vec<SampledFrame> {
        let start = Instant::now();
        let mut sampled = Vec::new();

        match self.strategy {
            SamplingStrategy::Uniform => {
                let step = (1.0 / self.sample_rate) as u32;
                for (i, frame) in frames.iter().enumerate() {
                    if i as u32 % step == 0 {
                        sampled.push(SampledFrame {
                            original_index: frame.index,
                            reason: "uniform".to_string(),
                        });
                    }
                }
            }
            SamplingStrategy::Keyframe => {
                for frame in frames {
                    if frame.is_keyframe || frame.diff_score > self.keyframe_threshold {
                        sampled.push(SampledFrame {
                            original_index: frame.index,
                            reason: "keyframe".to_string(),
                        });
                    }
                }
            }
            SamplingStrategy::SceneChange => {
                for frame in frames {
                    if frame.diff_score > self.scene_change_threshold {
                        sampled.push(SampledFrame {
                            original_index: frame.index,
                            reason: "scene_change".to_string(),
                        });
                    }
                }
            }
            SamplingStrategy::Adaptive => {
                for frame in frames {
                    let score = frame.diff_score * if frame.is_keyframe { 1.5 } else { 1.0 };
                    if score > (1.0 - self.sample_rate) {
                        sampled.push(SampledFrame {
                            original_index: frame.index,
                            reason: "adaptive".to_string(),
                        });
                    }
                }
            }
        }

        self.stats.total_frames += frames.len() as u32;
        self.stats.sampled_frames += sampled.len() as u32;
        self.stats.total_time_ms += start.elapsed().as_millis() as f64;

        sampled
    }

    /// 去重 (基于帧相似度)
    pub fn dedup(&self, frames: &[FrameInfo], similarity_threshold: f64) -> Vec<u32> {
        let mut unique_indices = Vec::new();
        let mut last_diff_score = 0.0;

        for frame in frames {
            if (frame.diff_score - last_diff_score).abs() > similarity_threshold {
                unique_indices.push(frame.index);
                last_diff_score = frame.diff_score;
            }
        }

        unique_indices
    }

    /// 获取统计信息
    pub fn stats(&self) -> SamplingStats {
        self.stats.clone()
    }
}

impl Default for FrameSampler {
    fn default() -> Self {
        Self::new(SamplingStrategy::Uniform, 0.1)
    }
}

/// 采样统计
#[derive(Debug, Clone, Default)]
pub struct SamplingStats {
    pub total_frames: u32,
    pub sampled_frames: u32,
    pub total_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_sampling() {
        let mut sampler = FrameSampler::new(SamplingStrategy::Uniform, 0.5);
        let frames: Vec<FrameInfo> = (0..10).map(|i| FrameInfo {
            index: i,
            timestamp_ms: i as f64 * 33.33,
            size_bytes: 1024,
            is_keyframe: false,
            diff_score: 0.1,
        }).collect();

        let sampled = sampler.sample(&frames);
        assert_eq!(sampled.len(), 5);
    }
}
