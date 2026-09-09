use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;

/// 搜索结果标准化器
pub struct SearchNormalizer;

impl SearchNormalizer {
    /// 标准化搜索结果
    pub fn normalize(results: Vec<MediaItem>, source: &str) -> Vec<MediaItem> {
        results.into_iter().map(|item| {
            MediaItem {
                id: item.id,
                title: Self::normalize_title(&item.title),
                artist: Self::normalize_artist(&item.artist),
                album: item.album,
                duration: item.duration,
                cover_url: item.cover_url,
                media_type: item.media_type,
                qualities: Self::normalize_qualities(&item.qualities, source),
            }
        }).collect()
    }

    /// 标准化标题 (去除多余空格, 统一大小写)
    fn normalize_title(title: &str) -> String {
        title.split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// 标准化艺术家 (去除 "Unknown" 变体)
    fn normalize_artist(artist: &str) -> String {
        match artist {
            "Unknown" | "unknown" | "未知" | "佚名" => "Unknown".to_string(),
            _ => artist.to_string(),
        }
    }

    /// 标准化音质列表
    fn normalize_qualities(qualities: &[Quality], source: &str) -> Vec<Quality> {
        let mut normalized: Vec<Quality> = qualities.to_vec();
        // 根据源的能力调整
        match source {
            "kugou" => {
                if !normalized.contains(&Quality::Flac) {
                    normalized.push(Quality::Flac);
                }
            }
            "kuwo" => {
                if !normalized.contains(&Quality::Flac24bit) {
                    normalized.push(Quality::Flac24bit);
                }
            }
            _ => {}
        }
        normalized.sort_by(|a, b| b.bitrate().cmp(&a.bitrate()));
        normalized
    }

    /// 去重 (按 title + artist)
    pub fn deduplicate(items: Vec<MediaItem>) -> Vec<MediaItem> {
        let mut seen = std::collections::HashSet::new();
        items.into_iter().filter(|item| {
            let key = format!("{}:{}", item.title, item.artist);
            seen.insert(key)
        }).collect()
    }
}
