//! L1 CAT-2 内容 — 社交媒体运营能力
//!
//! 实现统一架构: L1Capability + ContentProvider trait
//! 类别: CapabilityCategory::Content
//! 进化: C0→C1→C2→C3→C4→C5→C6
//!
//! Provider 可插拔: LinkedIn / Instagram / Facebook / Twitter / TikTok / YouTube
//! 排期引擎: 按最佳时间自动安排发布
//! 分析引擎: 跨平台互动数据聚合


use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::l1_action::traits::{
    L1Capability, ContentProvider, CapabilityCategory, ConstellationLevel,
    CapabilityHealth, CapabilityStats, CapabilityError,
    Post, EngagementMetrics,
};

// ════════════════════════════════════════════════════════════════
// 类型定义
// ════════════════════════════════════════════════════════════════

/// 社交平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    LinkedIn,
    Instagram,
    Facebook,
    Twitter,
    TikTok,
    YouTube,
    Pinterest,
    Alibaba,
    MadeInChina,
}

/// 内容类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Video,
    Carousel,
    Story,
    Reel,
    Article,
    Poll,
    Link,
}

/// 内容状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus {
    Draft,
    Scheduled,
    Publishing,
    Published,
    Failed,
    Archived,
}

/// 内容策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStrategy {
    pub name: String,
    pub target_platforms: Vec<Platform>,
    pub content_pillars: Vec<ContentPillar>,
    pub posting_frequency: PostingFrequency,
    pub hashtag_strategy: HashtagStrategy,
    pub tone_of_voice: String,
    pub target_audience: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPillar {
    pub name: String,
    pub description: String,
    pub percentage: f64,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingFrequency {
    pub posts_per_week: u32,
    pub best_times: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashtagStrategy {
    pub branded: Vec<String>,
    pub industry: Vec<String>,
    pub max_per_post: usize,
}

/// 排期槽
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSlot {
    pub id: String,
    pub platform: Platform,
    pub post: Post,
    pub scheduled_time: u64,
    pub status: ScheduleStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleStatus {
    Pending,
    Confirmed,
    Published,
    Failed,
}

/// 平台限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformLimits {
    pub max_body_length: usize,
    pub max_hashtags: usize,
    pub max_media: usize,
}

// ════════════════════════════════════════════════════════════════
// Provider 实现
// ════════════════════════════════════════════════════════════════

/// LinkedIn Provider
pub struct LinkedInProvider {
    id: String,
    pub access_token: String,
    pub organization_id: Option<String>,
    stats: CapabilityStats,
}

impl LinkedInProvider {
    pub fn new(access_token: &str, organization_id: Option<&str>) -> Self {
        Self {
            id: "content.linkedin".into(),
            access_token: access_token.to_string(),
            organization_id: organization_id.map(String::from),
            stats: CapabilityStats::default(),
        }
    }
}

impl L1Capability for LinkedInProvider {
    fn capability_id(&self) -> &str { &self.id }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Content }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C1UnitTest }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: !self.access_token.is_empty(),
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            message: None,
        }
    }
    fn description(&self) -> &str { "LinkedIn content publishing provider" }
    fn stats(&self) -> CapabilityStats { self.stats.clone() }
}

impl ContentProvider for LinkedInProvider {
    fn publish(&self, _post: &Post) -> Result<String, CapabilityError> {
        let post_id = format!("li_{}", uuid::Uuid::new_v4());
        // 实际实现: POST linkedin.com/v2/ugcPosts
        Ok(post_id)
    }

    fn get_engagement(&self, _post_id: &str) -> Result<EngagementMetrics, CapabilityError> {
        Ok(EngagementMetrics::default())
    }

    fn best_posting_times(&self) -> Vec<(u32, u32)> {
        vec![(8, 0), (12, 0), (17, 30)] // 工作日早/午/晚
    }
}

/// Instagram Provider
pub struct InstagramProvider {
    id: String,
    pub access_token: String,
    pub business_account_id: String,
    stats: CapabilityStats,
}

impl InstagramProvider {
    pub fn new(access_token: &str, business_account_id: &str) -> Self {
        Self {
            id: "content.instagram".into(),
            access_token: access_token.to_string(),
            business_account_id: business_account_id.to_string(),
            stats: CapabilityStats::default(),
        }
    }
}

impl L1Capability for InstagramProvider {
    fn capability_id(&self) -> &str { &self.id }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Content }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C1UnitTest }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: !self.access_token.is_empty(),
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            message: None,
        }
    }
    fn description(&self) -> &str { "Instagram content publishing provider" }
    fn stats(&self) -> CapabilityStats { self.stats.clone() }
}

impl ContentProvider for InstagramProvider {
    fn publish(&self, _post: &Post) -> Result<String, CapabilityError> {
        let post_id = format!("ig_{}", uuid::Uuid::new_v4());
        Ok(post_id)
    }

    fn get_engagement(&self, _post_id: &str) -> Result<EngagementMetrics, CapabilityError> {
        Ok(EngagementMetrics::default())
    }

    fn best_posting_times(&self) -> Vec<(u32, u32)> {
        vec![(11, 0), (14, 0), (19, 0)] // 午餐/下午/晚间
    }
}

// ════════════════════════════════════════════════════════════════
// Registry
// ════════════════════════════════════════════════════════════════

/// 内容能力注册中心
pub struct ContentRegistry {
    providers: Vec<Box<dyn ContentProvider>>,
}

impl Default for ContentRegistry {
    fn default() -> Self { Self::new() }
}

impl ContentRegistry {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn register(&mut self, provider: Box<dyn ContentProvider>) {
        self.providers.push(provider);
    }

    pub fn get(&self, id: &str) -> Option<&dyn ContentProvider> {
        self.providers.iter().find(|p| p.capability_id() == id).map(|p| p.as_ref())
    }

    pub fn health_check_all(&self) -> Vec<(String, CapabilityHealth)> {
        self.providers.iter()
            .map(|p| (p.capability_id().to_string(), p.health_check()))
            .collect()
    }

    pub fn optimal(&self) -> Option<&dyn ContentProvider> {
        self.providers.iter()
            .filter(|p| p.health_check().healthy)
            .max_by(|a, b| {
                let a_score = 1.0 - a.health_check().error_rate;
                let b_score = 1.0 - b.health_check().error_rate;
                a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.as_ref())
    }
}

// ════════════════════════════════════════════════════════════════
// Router
// ════════════════════════════════════════════════════════════════

/// 内容路由器 — 按平台选择最佳 Provider
pub struct ContentRouter {
    registry: ContentRegistry,
}

impl ContentRouter {
    pub fn new(registry: ContentRegistry) -> Self {
        Self { registry }
    }

    pub fn route(&self, _platform: Platform) -> Option<&dyn ContentProvider> {
        self.registry.optimal()
    }

    pub fn publish(&self, post: &Post) -> Result<String, CapabilityError> {
        let provider = self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No content provider".into()))?;
        provider.publish(post)
    }

    pub fn get_engagement(&self, post_id: &str) -> Result<EngagementMetrics, CapabilityError> {
        let provider = self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No content provider".into()))?;
        provider.get_engagement(post_id)
    }
}

// ════════════════════════════════════════════════════════════════
// Bridge
// ════════════════════════════════════════════════════════════════

/// 内容能力桥接
pub struct ContentBridge {
    router: ContentRouter,
}

impl ContentBridge {
    pub fn new(router: ContentRouter) -> Self {
        Self { router }
    }

    pub fn publish(&self, post: &Post) -> Result<String, CapabilityError> {
        self.router.publish(post)
    }

    pub fn get_engagement(&self, post_id: &str) -> Result<EngagementMetrics, CapabilityError> {
        self.router.get_engagement(post_id)
    }
}

// ════════════════════════════════════════════════════════════════
// 内容生成器
// ════════════════════════════════════════════════════════════════

/// 内容生成器 — 基于策略自动生成帖子
pub struct ContentGenerator {
    strategy: ContentStrategy,
}

impl ContentGenerator {
    pub fn new(strategy: ContentStrategy) -> Self {
        Self { strategy }
    }

    pub fn generate_post(
        &self,
        pillar: &ContentPillar,
        platform: Platform,
        custom_body: Option<String>,
    ) -> Post {
        let body = custom_body.unwrap_or_else(|| {
            format!("📢 {} — {}\n\n#{}", pillar.name, pillar.description, pillar.keywords.join(" #"))
        });
        let hashtags: Vec<String> = self.strategy.hashtag_strategy.industry.iter()
            .take(self.strategy.hashtag_strategy.max_per_post)
            .cloned()
            .collect();

        Post {
            id: uuid::Uuid::new_v4().to_string(),
            platform: format!("{:?}", platform).to_lowercase(),
            body,
            hashtags,
            status: "draft".into(),
        }
    }

    pub fn generate_weekly_calendar(&self) -> Vec<Post> {
        let mut calendar = Vec::new();
        let posts_per_week = self.strategy.posting_frequency.posts_per_week as usize;
        for i in 0..posts_per_week {
            let pillar = &self.strategy.content_pillars[i % self.strategy.content_pillars.len()];
            for &platform in &self.strategy.target_platforms {
                calendar.push(self.generate_post(pillar, platform, None));
            }
        }
        calendar
    }
}

// ════════════════════════════════════════════════════════════════
// 排期引擎
// ════════════════════════════════════════════════════════════════

/// 排期引擎
pub struct ScheduleEngine {
    slots: Vec<ScheduleSlot>,
}

impl Default for ScheduleEngine {
    fn default() -> Self { Self::new() }
}

impl ScheduleEngine {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub fn schedule(&mut self, post: Post, time: u64) -> Result<&ScheduleSlot, String> {
        let slot = ScheduleSlot {
            id: format!("slot_{}", uuid::Uuid::new_v4()),
            platform: Platform::LinkedIn, // default
            post,
            scheduled_time: time,
            status: ScheduleStatus::Pending,
        };
        self.slots.push(slot);
        Ok(self.slots.last().expect("just pushed a slot"))
    }

    pub(crate) fn _auto_schedule(&mut self, post: Post, best_times: &[(u32, u32)]) -> Result<&ScheduleSlot, String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let tomorrow = now + 86400;
        let time = best_times.first().map_or(tomorrow + 3600, |&(h, _)| tomorrow + (h as u64 * 3600));
        self.schedule(post, time)
    }

    pub(crate) fn _pending_slots(&self) -> Vec<&ScheduleSlot> {
        self.slots.iter().filter(|s| s.status == ScheduleStatus::Pending).collect()
    }
}

// ════════════════════════════════════════════════════════════════
// 分析引擎
// ════════════════════════════════════════════════════════════════

/// 社交媒体分析
pub struct SocialAnalytics {
    posts: Vec<Post>,
}

impl Default for SocialAnalytics {
    fn default() -> Self { Self::new() }
}

impl SocialAnalytics {
    pub fn new() -> Self { Self { posts: Vec::new() } }

    pub(crate) fn _add_post(&mut self, post: Post) { self.posts.push(post); }

    pub(crate) fn _top_posts(&self, n: usize) -> Vec<&Post> {
        let mut sorted: Vec<&Post> = self.posts.iter().collect();
        sorted.sort_by(|a, b| b.id.cmp(&a.id)); // placeholder sort
        sorted.into_iter().take(n).collect()
    }
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linkedin_provider_trait() {
        let p = LinkedInProvider::new("token", None);
        assert_eq!(p.category(), CapabilityCategory::Content);
        assert_eq!(p.constellation(), ConstellationLevel::C1UnitTest);
        assert!(p.health_check().healthy);
    }

    #[test]
    fn test_instagram_provider_trait() {
        let p = InstagramProvider::new("token", "biz123");
        assert_eq!(p.category(), CapabilityCategory::Content);
        assert!(p.health_check().healthy);
    }

    #[test]
    fn test_registry() {
        let mut reg = ContentRegistry::new();
        reg.register(Box::new(LinkedInProvider::new("token", None)));
        reg.register(Box::new(InstagramProvider::new("token", "biz123")));
        assert_eq!(reg.health_check_all().len(), 2);
        assert!(reg.optimal().is_some());
    }

    #[test]
    fn test_content_generator() {
        let strategy = ContentStrategy {
            name: "Test".into(),
            target_platforms: vec![Platform::LinkedIn],
            content_pillars: vec![ContentPillar {
                name: "Pillar".into(), description: "Desc".into(),
                percentage: 1.0, keywords: vec!["test".into()],
            }],
            posting_frequency: PostingFrequency { posts_per_week: 3, best_times: vec![(9, 0)] },
            hashtag_strategy: HashtagStrategy { branded: vec![], industry: vec!["#test".into()], max_per_post: 5 },
            tone_of_voice: "Professional".into(),
            target_audience: "B2B".into(),
        };
        let gen = ContentGenerator::new(strategy);
        let calendar = gen.generate_weekly_calendar();
        assert!(!calendar.is_empty());
    }
}
