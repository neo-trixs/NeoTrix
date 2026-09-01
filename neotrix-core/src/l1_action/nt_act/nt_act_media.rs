//! L1 社交媒体运营 — 内容创作 + 多平台发布 + 排期 + 数据分析
//!
//! 通用能力: 任何域(外贸/营销/品牌/个人IP)都可调用
//! 设计原则: Platform trait 抽象 → 多平台可插拔, 内容生成 + 排期引擎 + 效果追踪

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════
// 平台抽象
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
    GlobalSources,
    Reddit,
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
    Document,
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

/// 内容帖子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub platform: Platform,
    pub content_type: ContentType,
    pub title: Option<String>,
    pub body: String,
    pub hashtags: Vec<String>,
    pub media_urls: Vec<String>,
    pub link_url: Option<String>,
    pub status: ContentStatus,
    pub scheduled_at: Option<u64>,
    pub published_at: Option<u64>,
    pub engagement: EngagementMetrics,
    pub metadata: HashMap<String, String>,
}

/// 互动数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub impressions: u64,
    pub reach: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub saves: u64,
    pub clicks: u64,
    pub ctr: f64,
    pub engagement_rate: f64,
}

/// 内容排期槽
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSlot {
    pub id: String,
    pub platform: Platform,
    pub post: Post,
    pub scheduled_time: u64,
    pub timezone: String,
    pub status: ScheduleStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleStatus {
    Pending,
    Confirmed,
    Published,
    Skipped,
    Failed,
}

// ════════════════════════════════════════════════════════════════
// Platform trait — 多平台可插拔
// ════════════════════════════════════════════════════════════════

/// 社交平台 Provider trait
pub trait SocialPlatform: Send + Sync {
    fn platform(&self) -> Platform;
    fn provider_name(&self) -> &str;
    fn is_available(&self) -> bool;

    /// 发布内容
    fn publish(&self, post: &Post) -> Result<String, String>;

    /// 获取帖子状态
    fn get_post_status(&self, post_id: &str) -> Result<ContentStatus, String>;

    /// 获取互动数据
    fn get_engagement(&self, post_id: &str) -> Result<EngagementMetrics, String>;

    /// 批量获取帖子
    fn list_posts(&self, since: Option<u64>, limit: usize) -> Result<Vec<Post>, String>;

    /// 获取最佳发布时间建议
    fn best_posting_times(&self) -> Vec<(u32, u32)>; // (hour, minute) UTC

    /// 获取平台限制
    fn platform_limits(&self) -> PlatformLimits;
}

/// 平台内容限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformLimits {
    pub max_body_length: usize,
    pub max_hashtags: usize,
    pub max_media: usize,
    pub supported_content_types: Vec<ContentType>,
    pub max_video_duration_secs: Option<u64>,
}

// ════════════════════════════════════════════════════════════════
// 内容生成引擎
// ════════════════════════════════════════════════════════════════

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
    pub percentage: f64, // 占总内容的百分比
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingFrequency {
    pub posts_per_week: u32,
    pub best_times: Vec<(u32, u32)>,
    pub content_mix: HashMap<ContentType, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashtagStrategy {
    pub branded: Vec<String>,
    pub industry: Vec<String>,
    pub trending: Vec<String>,
    pub max_per_post: usize,
}

/// 内容生成器 — 基于策略自动生成帖子
pub struct ContentGenerator {
    strategy: ContentStrategy,
    content_queue: Vec<Post>,
}

impl ContentGenerator {
    pub fn new(strategy: ContentStrategy) -> Self {
        Self {
            strategy,
            content_queue: Vec::new(),
        }
    }

    /// 基于内容支柱生成帖子
    pub fn generate_post(
        &self,
        pillar: &ContentPillar,
        platform: Platform,
        custom_body: Option<String>,
    ) -> Post {
        let body = custom_body.unwrap_or_else(|| {
            format!(
                "📢 {} — {}\n\n#{}",
                pillar.name,
                pillar.description,
                pillar.keywords.join(" #")
            )
        });

        let hashtags: Vec<String> = self.strategy.hashtag_strategy.industry.iter()
            .take(self.strategy.hashtag_strategy.max_per_post)
            .cloned()
            .collect();

        Post {
            id: uuid::Uuid::new_v4().to_string(),
            platform,
            content_type: ContentType::Text,
            title: Some(pillar.name.clone()),
            body,
            hashtags,
            media_urls: Vec::new(),
            link_url: None,
            status: ContentStatus::Draft,
            scheduled_at: None,
            published_at: None,
            engagement: EngagementMetrics::default(),
            metadata: HashMap::new(),
        }
    }

    /// 生成一周内容日历
    pub fn generate_weekly_calendar(&self) -> Vec<Post> {
        let mut calendar = Vec::new();
        let posts_per_week = self.strategy.posting_frequency.posts_per_week as usize;
        let pillars = &self.strategy.content_pillars;

        for i in 0..posts_per_week {
            let pillar = &pillars[i % pillars.len()];
            for &platform in &self.strategy.target_platforms {
                let post = self.generate_post(pillar, platform, None);
                calendar.push(post);
            }
        }
        calendar
    }

    /// 获取内容队列
    pub fn queue(&self) -> &[Post] {
        &self.content_queue
    }

    /// 入队
    pub fn enqueue(&mut self, post: Post) {
        self.content_queue.push(post);
    }
}

// ════════════════════════════════════════════════════════════════
// 排期引擎
// ════════════════════════════════════════════════════════════════

/// 排期引擎 — 自动安排发布时间
pub struct ScheduleEngine {
    slots: Vec<ScheduleSlot>,
    timezone: String,
}

impl Default for ScheduleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleEngine {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            timezone: "UTC".into(),
        }
    }

    pub fn with_timezone(tz: &str) -> Self {
        Self {
            slots: Vec::new(),
            timezone: tz.to_string(),
        }
    }

    /// 添加帖子到排期
    pub fn schedule(&mut self, post: Post, time: u64) -> Result<&ScheduleSlot, String> {
        let slot = ScheduleSlot {
            id: format!("slot_{}", uuid::Uuid::new_v4()),
            platform: post.platform,
            post,
            scheduled_time: time,
            timezone: self.timezone.clone(),
            status: ScheduleStatus::Pending,
        };
        self.slots.push(slot);
        Ok(self.slots.last().unwrap())
    }

    /// 自动排期: 根据最佳时间安排
    pub fn auto_schedule(&mut self, post: Post, best_times: &[(u32, u32)]) -> Result<&ScheduleSlot, String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let tomorrow = now + 86400;

        if let Some(&(hour, _minute)) = best_times.first() {
            let scheduled = tomorrow + (hour as u64 * 3600);
            self.schedule(post, scheduled)
        } else {
            self.schedule(post, tomorrow + 3600) // default: tomorrow 1am
        }
    }

    /// 获取今日排期
    pub fn today_slots(&self) -> Vec<&ScheduleSlot> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let today_start = now - (now % 86400);
        let today_end = today_start + 86400;

        self.slots.iter()
            .filter(|s| s.scheduled_time >= today_start && s.scheduled_time < today_end)
            .collect()
    }

    /// 获取待发布排期
    pub fn pending_slots(&self) -> Vec<&ScheduleSlot> {
        self.slots.iter()
            .filter(|s| s.status == ScheduleStatus::Pending)
            .collect()
    }

    /// 确认排期
    pub fn confirm(&mut self, slot_id: &str) -> Result<(), String> {
        let slot = self.slots.iter_mut()
            .find(|s| s.id == slot_id)
            .ok_or_else(|| format!("Slot {} not found", slot_id))?;
        slot.status = ScheduleStatus::Confirmed;
        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════
// 分析引擎
// ════════════════════════════════════════════════════════════════

/// 社交媒体分析 — 跨平台数据聚合
pub struct SocialAnalytics {
    posts: Vec<Post>,
}

impl Default for SocialAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialAnalytics {
    pub fn new() -> Self {
        Self { posts: Vec::new() }
    }

    pub fn add_post(&mut self, post: Post) {
        self.posts.push(post);
    }

    /// 聚合所有帖子的互动数据
    pub fn aggregate_engagement(&self) -> EngagementMetrics {
        let mut total = EngagementMetrics::default();
        for post in &self.posts {
            total.impressions += post.engagement.impressions;
            total.reach += post.engagement.reach;
            total.likes += post.engagement.likes;
            total.comments += post.engagement.comments;
            total.shares += post.engagement.shares;
            total.saves += post.engagement.saves;
            total.clicks += post.engagement.clicks;
        }
        if total.impressions > 0 {
            total.ctr = total.clicks as f64 / total.impressions as f64;
            total.engagement_rate = (total.likes + total.comments + total.shares) as f64
                / total.impressions as f64;
        }
        total
    }

    /// 按平台分组统计
    pub fn by_platform(&self) -> HashMap<Platform, Vec<&Post>> {
        let mut grouped: HashMap<Platform, Vec<&Post>> = HashMap::new();
        for post in &self.posts {
            grouped.entry(post.platform).or_default().push(post);
        }
        grouped
    }

    /// Top performing posts
    pub fn top_posts(&self, n: usize) -> Vec<&Post> {
        let mut sorted: Vec<&Post> = self.posts.iter().collect();
        sorted.sort_by(|a, b| {
            let a_eng = a.engagement.likes + a.engagement.comments + a.engagement.shares;
            let b_eng = b.engagement.likes + b.engagement.comments + b.engagement.shares;
            b_eng.cmp(&a_eng)
        });
        sorted.into_iter().take(n).collect()
    }
}

// ════════════════════════════════════════════════════════════════
// 能力注册
// ════════════════════════════════════════════════════════════════

pub fn register_capability(registry: &mut nt_core_capability_tree::registry::CapabilityRegistry) -> Result<(), nt_core_capability_tree::registry::RegistryError> {
    use nt_core_capability_tree::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer};

    let mut node = CapabilityNode::new_primitive(
        "nt_act::media::social_media".into(),
        Domain::Act,
        vec![
            "media.publish".into(),
            "media.schedule".into(),
            "media.analytics".into(),
            "content.generation".into(),
        ],
    );
    node.layer = NodeLayer::L1Composite;
    node.constellation = ConstellationLevel::C1UnitTest;
    registry.register(node)
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_generator() {
        let strategy = ContentStrategy {
            name: "Trade Marketing".into(),
            target_platforms: vec![Platform::LinkedIn, Platform::Instagram],
            content_pillars: vec![
                ContentPillar {
                    name: "Product Showcase".into(),
                    description: "展示产品优势".into(),
                    percentage: 0.4,
                    keywords: vec!["product".into(), "quality".into()],
                },
                ContentPillar {
                    name: "Industry Insights".into(),
                    description: "行业趋势分析".into(),
                    percentage: 0.3,
                    keywords: vec!["industry".into(), "trend".into()],
                },
            ],
            posting_frequency: PostingFrequency {
                posts_per_week: 5,
                best_times: vec![(9, 0), (14, 0)],
                content_mix: HashMap::new(),
            },
            hashtag_strategy: HashtagStrategy {
                branded: vec!["#ACME".into()],
                industry: vec!["#manufacturing".into(), "#export".into()],
                trending: vec![],
                max_per_post: 10,
            },
            tone_of_voice: "Professional, friendly".into(),
            target_audience: "B2B buyers".into(),
        };

        let generator = ContentGenerator::new(strategy);
        let calendar = generator.generate_weekly_calendar();
        assert!(!calendar.is_empty());
    }

    #[test]
    fn test_schedule_engine() {
        let mut engine = ScheduleEngine::new();
        let post = Post {
            id: "test".into(),
            platform: Platform::LinkedIn,
            content_type: ContentType::Text,
            title: None,
            body: "Test post".into(),
            hashtags: vec![],
            media_urls: vec![],
            link_url: None,
            status: ContentStatus::Draft,
            scheduled_at: None,
            published_at: None,
            engagement: EngagementMetrics::default(),
            metadata: HashMap::new(),
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        engine.schedule(post, now + 3600).unwrap();
        assert_eq!(engine.pending_slots().len(), 1);
    }

    #[test]
    fn test_analytics_aggregation() {
        let mut analytics = SocialAnalytics::new();
        let post = Post {
            id: "test".into(),
            platform: Platform::LinkedIn,
            content_type: ContentType::Text,
            title: None,
            body: "Test".into(),
            hashtags: vec![],
            media_urls: vec![],
            link_url: None,
            status: ContentStatus::Published,
            scheduled_at: None,
            published_at: Some(1000),
            engagement: EngagementMetrics {
                impressions: 1000,
                reach: 500,
                likes: 50,
                comments: 10,
                shares: 5,
                saves: 3,
                clicks: 20,
                ctr: 0.02,
                engagement_rate: 0.065,
            },
            metadata: HashMap::new(),
        };
        analytics.add_post(post);

        let agg = analytics.aggregate_engagement();
        assert_eq!(agg.impressions, 1000);
        assert_eq!(agg.likes, 50);
    }
}
