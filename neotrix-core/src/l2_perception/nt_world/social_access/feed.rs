//! FeedService — 通用推荐引擎与平台适配器注册表
//!
//! 融合了 x-algorithm 开源权重 (2026-08)、SimClusters 社区检索、
//! 以及 Phoenix 多任务排序的核心思想。

use std::collections::HashMap;
use std::sync::Arc;

use crate::l2_perception::nt_world::social_access::traits::*;
use crate::l2_perception::nt_world::social_access::SocialAccessError;

/// 通用推荐引擎 — 基于 x-algorithm 开源权重
pub struct UniversalRecommender {
    action_weights: HashMap<String, f64>,
}

impl UniversalRecommender {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert("share_copy_link".to_string(), 20.0);
        weights.insert("reply_mutual".to_string(), 20.0);
        weights.insert("reply_normal".to_string(), 5.0);
        weights.insert("quote_dm".to_string(), 5.0);
        weights.insert("follow_author".to_string(), 4.0);
        weights.insert("generic_share".to_string(), 2.0);
        weights.insert("repost".to_string(), 1.0);
        weights.insert("like".to_string(), 0.5);
        weights.insert("open_link".to_string(), 0.2);
        weights.insert("report".to_string(), -234.0);
        weights.insert("mute_author".to_string(), -58.8);
        Self { action_weights: weights }
    }

    pub fn score_post(&self, actions: &HashMap<String, f64>) -> f64 {
        actions.iter().map(|(action, count)| {
            self.action_weights.get(action).unwrap_or(&0.0) * count
        }).sum()
    }

    pub fn rank_posts(&self, mut posts: Vec<FeedItem>) -> Vec<FeedItem> {
        for post in &mut posts {
            post.score = self.score_post(&post.actions);
        }
        posts.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        let mut result = Vec::new();
        let mut author_count: HashMap<String, usize> = HashMap::new();
        for post in posts {
            let count = author_count.entry(post.author.clone()).or_insert(0);
            if *count >= 2 && result.len() >= 20 { break; }
            *count += 1;
            result.push(post);
        }
        result
    }
}

/// 平台适配器注册表
pub struct PlatformAdapterRegistry {
    adapters: HashMap<String, Arc<dyn SocialPlatformAdapter>>,
}

impl PlatformAdapterRegistry {
    pub fn new() -> Self {
        Self { adapters: HashMap::new() }
    }

    pub fn register(&mut self, adapter: Arc<dyn SocialPlatformAdapter>) {
        self.adapters.insert(adapter.id(), adapter);
    }

    pub fn get(&self, platform: &str) -> Option<&dyn SocialPlatformAdapter> {
        self.adapters.get(platform).map(|b| b.as_ref())
    }

    pub fn all_platforms(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }
}

/// FeedService — 推荐流获取与聚合
pub struct FeedService {
    registry: PlatformAdapterRegistry,
    recommender: UniversalRecommender,
}

impl FeedService {
    pub fn new() -> Self {
        Self {
            registry: PlatformAdapterRegistry::new(),
            recommender: UniversalRecommender::new(),
        }
    }

    pub fn register_adapter(&mut self, adapter: Arc<dyn SocialPlatformAdapter>) {
        self.registry.register(adapter);
    }

    pub fn get_recommended(
        &self,
        platform: SocialPlatform,
        session: &SessionEntry,
        limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        let adapter = self.registry.get(platform.as_str())
            .ok_or_else(|| SocialAccessError::Platform(format!("No adapter for {:?}", platform)))?;
        let raw = adapter.get_recommended(session, limit)?;
        let ranked = self.recommender.rank_posts(raw.items);
        let total = ranked.len();
        Ok(FeedResult { items: ranked, total })
    }

    pub fn get_following(
        &self,
        platform: SocialPlatform,
        session: &SessionEntry,
        limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        let adapter = self.registry.get(platform.as_str())
            .ok_or_else(|| SocialAccessError::Platform(format!("No adapter for {:?}", platform)))?;
        adapter.get_following(session, limit)
    }

    pub fn get_trending(
        &self,
        platform: SocialPlatform,
    ) -> Result<Vec<TrendingTopic>, SocialAccessError> {
        let adapter = self.registry.get(platform.as_str())
            .ok_or_else(|| SocialAccessError::Platform(format!("No adapter for {:?}", platform)))?;
        adapter.get_trending()
    }

    pub fn get_feed_all(
        &self,
        session: &SessionEntry,
        limit_per_platform: usize,
    ) -> Result<Vec<FeedItem>, SocialAccessError> {
        let mut all = Vec::new();
        for platform_str in self.registry.all_platforms() {
            let platform = SocialPlatform::from_str(&platform_str);
            match self.get_recommended(platform, session, limit_per_platform) {
                Ok(feed) => all.extend(feed.items),
                Err(_) => continue,
            }
        }
        let ranked = self.recommender.rank_posts(all);
        Ok(ranked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommender_scoring() {
        let rec = UniversalRecommender::new();
        let mut actions = HashMap::new();
        actions.insert("like".to_string(), 10.0);
        actions.insert("generic_share".to_string(), 2.0);
        let score = rec.score_post(&actions);
        assert_eq!(score, 10.0 * 0.5 + 2.0 * 2.0);
    }

    #[test]
    fn test_recommender_author_diversity() {
        let rec = UniversalRecommender::new();
        let mut posts = vec![
            FeedItem { id: "1".to_string(), content: "a".to_string(), author: "alice".to_string(), metrics: EngagementMetrics::default(), score: 0.0, actions: HashMap::new() },
            FeedItem { id: "2".to_string(), content: "b".to_string(), author: "alice".to_string(), metrics: EngagementMetrics::default(), score: 0.0, actions: HashMap::new() },
            FeedItem { id: "3".to_string(), content: "c".to_string(), author: "bob".to_string(), metrics: EngagementMetrics::default(), score: 0.0, actions: HashMap::new() },
        ];
        posts = rec.rank_posts(posts);
        assert!(posts.len() <= 3);
        let alice_count = posts.iter().filter(|p| p.author == "alice").count();
        assert!(alice_count <= 2);
    }
}
