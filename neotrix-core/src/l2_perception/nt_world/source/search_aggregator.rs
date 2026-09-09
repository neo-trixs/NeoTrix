use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;
use std::collections::HashMap;

/// 多源搜索聚合器
pub struct SearchAggregator;

impl SearchAggregator {
    /// 并发搜索多个源，合并去重排序
    pub async fn aggregate_search(
        query: &str,
        sources: &[Box<dyn MediaSource>],
        page: u32,
    ) -> Vec<MediaItem> {
        let mut results: Vec<MediaItem> = Vec::new();
        let mut seen: HashMap<String, bool> = HashMap::new();

        for source in sources {
            if let Ok(search_result) = source.search(query, page).await {
                for item in search_result.data {
                    let key = format!("{}:{}", item.title, item.artist);
                    if !seen.contains_key(&key) {
                        seen.insert(key, true);
                        results.push(item);
                    }
                }
            }
        }

        // 按相关性排序 (简单实现: 标题匹配度)
        results.sort_by(|a, b| {
            let a_score = Self::relevance_score(&a.title, query);
            let b_score = Self::relevance_score(&b.title, query);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// 计算相关性分数
    fn relevance_score(title: &str, query: &str) -> f64 {
        let title_lower = title.to_lowercase();
        let query_lower = query.to_lowercase();

        if title_lower == query_lower {
            1.0
        } else if title_lower.contains(&query_lower) {
            0.8
        } else {
            // 简单的词匹配
            let query_words: Vec<&str> = query_lower.split_whitespace().collect();
            let title_words: Vec<&str> = title_lower.split_whitespace().collect();
            let matches = query_words.iter().filter(|qw| title_words.contains(qw)).count();
            matches as f64 / query_words.len().max(1) as f64 * 0.6
        }
    }
}
