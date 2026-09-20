//! 换源功能模块 — 自动寻找替代音源
//!
//! 整合 go-music-dl 的换源能力:
//! - 按歌名/歌手相似度匹配
//! - 时长差异过滤
//! - 可播放性验证
//! - 优先级排序

use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaEngine;

/// 换源配置
#[derive(Debug, Clone)]
pub struct SwitchConfig {
    /// 最大时长差异 (秒)
    pub max_duration_diff: u64,
    /// 最小相似度阈值 (0-1)
    pub min_similarity: f64,
    /// 排除的平台
    pub exclude_platforms: Vec<String>,
    /// 是否验证可播放性
    pub verify_playable: bool,
}

impl Default for SwitchConfig {
    fn default() -> Self {
        Self {
            max_duration_diff: 30,
            min_similarity: 0.6,
            exclude_platforms: vec!["soda".into(), "fivesing".into()],
            verify_playable: true,
        }
    }
}

/// 换源候选
#[derive(Debug, Clone)]
pub struct SwitchCandidate {
    pub item: MediaItem,
    pub similarity: f64,
    pub duration_diff: Option<i64>,
    pub source_priority: u32,
}

/// 换源结果
#[derive(Debug, Clone)]
pub struct SwitchResult {
    pub original: MediaItem,
    pub replacement: Option<MediaItem>,
    pub candidates_checked: usize,
    pub reason: String,
}

/// 换源器
pub struct SourceSwitcher {
    config: SwitchConfig,
}

impl SourceSwitcher {
    pub fn new(config: SwitchConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(SwitchConfig::default())
    }

    /// 为给定歌曲寻找替代音源
    pub async fn find_replacement(
        &self,
        original: &MediaItem,
        engine: &MediaEngine,
    ) -> SwitchResult {
        // 构建搜索查询
        let query = if original.artist.is_empty() || original.artist == "Unknown" {
            original.title.clone()
        } else {
            format!("{} {}", original.title, original.artist)
        };

        // 搜索多个源
        let mut candidates: Vec<SwitchCandidate> = Vec::new();

        for (source, priority, healthy) in engine.sources() {
            if !healthy {
                continue;
            }

            // 跳过排除的平台
            if self.config.exclude_platforms.contains(&source.id().to_string()) {
                continue;
            }

            if let Ok(result) = source.search(&query, 1).await {
                for item in result.data {
                    let similarity = self.calculate_similarity(&item.title, &item.artist, original);

                    if similarity < self.config.min_similarity {
                        continue;
                    }

                    // 计算时长差异
                    let duration_diff = match (&item.duration, &original.duration) {
                        (Some(d1), Some(d2)) => {
                            let diff = d1.as_secs() as i64 - d2.as_secs() as i64;
                            if diff.unsigned_abs() > self.config.max_duration_diff {
                                continue;
                            }
                            Some(diff)
                        }
                        _ => None,
                    };

                    candidates.push(SwitchCandidate {
                        item,
                        similarity,
                        duration_diff,
                        source_priority: priority,
                    });
                }
            }
        }

        // 按相似度和优先级排序
        candidates.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.source_priority.cmp(&b.source_priority))
        });

        // 验证最佳候选的可播放性
        if let Some(best) = candidates.first() {
            if self.config.verify_playable {
                if let Ok(view_source) = engine.play_url(&best.item, Quality::Standard).await {
                    // 简单验证 URL 是否可访问
                    if !view_source.url.is_empty() {
                        return SwitchResult {
                            original: original.clone(),
                            replacement: Some(best.item.clone()),
                            candidates_checked: candidates.len(),
                            reason: format!(
                                "Found replacement with {:.0}% similarity from {}",
                                best.similarity * 100.0,
                                best.item.id
                            ),
                        };
                    }
                }
            } else {
                return SwitchResult {
                    original: original.clone(),
                    replacement: Some(best.item.clone()),
                    candidates_checked: candidates.len(),
                    reason: format!(
                        "Found candidate with {:.0}% similarity",
                        best.similarity * 100.0
                    ),
                };
            }
        }

        SwitchResult {
            original: original.clone(),
            replacement: None,
            candidates_checked: candidates.len(),
            reason: if candidates.is_empty() {
                "No matching candidates found".to_string()
            } else {
                "Best candidate failed playability check".to_string()
            },
        }
    }

    /// 批量换源
    pub async fn batch_switch(
        &self,
        items: &[MediaItem],
        engine: &MediaEngine,
    ) -> Vec<SwitchResult> {
        let mut results = Vec::new();
        for item in items {
            results.push(self.find_replacement(item, engine).await);
        }
        results
    }

    /// 计算相似度 (标题 + 歌手)
    fn calculate_similarity(&self, title: &str, artist: &str, original: &MediaItem) -> f64 {
        let title_sim = self.fuzzy_match(title, &original.title);
        let artist_sim = if artist.is_empty() || artist == "Unknown" {
            0.5 // 无歌手信息时给中间分
        } else {
            self.fuzzy_match(artist, &original.artist)
        };

        // 标题权重 0.7，歌手权重 0.3
        title_sim * 0.7 + artist_sim * 0.3
    }

    /// 模糊匹配 (简单实现)
    fn fuzzy_match(&self, a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        if a_lower == b_lower {
            return 1.0;
        }

        if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
            return 0.8;
        }

        // 计算词重叠率
        let a_words: Vec<&str> = a_lower.split_whitespace().collect();
        let b_words: Vec<&str> = b_lower.split_whitespace().collect();

        if a_words.is_empty() || b_words.is_empty() {
            return 0.0;
        }

        let matches = a_words.iter().filter(|aw| b_words.contains(aw)).count();
        matches as f64 / a_words.len().max(b_words.len()) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_fuzzy_match() {
        let switcher = SourceSwitcher::default();
        assert_eq!(switcher.fuzzy_match("晴天", "晴天"), 1.0);
        assert!(switcher.fuzzy_match("周杰伦 - 晴天", "晴天") > 0.5);
    }

    #[test]
    fn test_similarity() {
        let switcher = SourceSwitcher::default();
        let original = MediaItem {
            id: "1".into(),
            title: "晴天".into(),
            artist: "周杰伦".into(),
            album: "叶惠美".into(),
            duration: Some(Duration::from_secs(269)),
            cover_url: None,
            media_type: MediaType::Audio,
            qualities: vec![Quality::Flac],
        };
        let sim = switcher.calculate_similarity("晴天", "周杰伦", &original);
        assert_eq!(sim, 1.0);
    }
}
