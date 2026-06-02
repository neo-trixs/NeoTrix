use std::collections::HashMap;

/// 视频剧本 — 结构化场景序列
#[derive(Clone, Debug)]
pub struct VideoScript {
    pub title: String,
    pub scenes: Vec<VideoScene>,
    pub bgm_mood: String,
    pub total_duration_secs: f64,
}

/// 单个视频场景
#[derive(Clone, Debug)]
pub struct VideoScene {
    pub narration: String,
    pub visual_desc: String,
    pub duration_secs: f64,
    pub search_keywords: Vec<String>,
}

/// 视频脚本规划器
pub struct VideoScriptPlanner {
    llm: Option<(tokio::runtime::Runtime, Box<dyn crate::neotrix::provider::types::LlmProvider>)>,
}

impl VideoScriptPlanner {
    pub fn new() -> Self {
        Self { llm: None }
    }

    pub fn with_llm(provider: Box<dyn crate::neotrix::provider::types::LlmProvider>) -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        Self { llm: Some((runtime, provider)) }
    }

    pub fn generate(&self, topic: &str, platform: &str, duration_secs: f64) -> VideoScript {
        if let Some(ref state) = self.llm {
            let (ref runtime, ref provider) = *state;
            return self.generate_llm(runtime, provider.as_ref(), topic, platform, duration_secs);
        }
        self.generate_template(topic, duration_secs)
    }

    fn generate_llm(
        &self, runtime: &tokio::runtime::Runtime,
        provider: &dyn crate::neotrix::provider::types::LlmProvider,
        topic: &str, platform: &str, duration_secs: f64,
    ) -> VideoScript {
        let scene_count = (duration_secs / 8.0).ceil() as usize;
        let dur_str = format!("{}", duration_secs);
        let scene_str = format!("{}", scene_count);
        let prompt = format!(
            "Generate a video script for a {0}-second {1} video about '{2}'.\n\
             Return JSON with: title, scenes [{{narration, visual_desc, duration_secs, search_keywords}}], bgm_mood.\n\
             Exactly {3} scenes. Total duration must sum to {0}.\n\
             search_keywords are 2-3 English search terms for stock footage.\n\
             JSON only, no markdown.",
            dur_str, platform, topic, scene_str,
        );
        let request = crate::neotrix::provider::types::LlmRequest::new("gpt-4o", &prompt);
        match runtime.block_on(provider.complete(&request)) {
            Ok(resp) => self.parse_json_script(&resp.content, topic, duration_secs),
            Err(_) => self.generate_template(topic, duration_secs),
        }
    }

    fn parse_json_script(&self, json: &str, topic: &str, duration_secs: f64) -> VideoScript {
        let cleaned = json.trim().trim_start_matches("```json").trim_end_matches("```").trim();
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(cleaned) {
            let title = parsed.get("title").and_then(|v| v.as_str()).unwrap_or(topic).to_string();
            let bgm = parsed.get("bgm_mood").and_then(|v| v.as_str()).unwrap_or("neutral").to_string();
            let scenes = parsed.get("scenes").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().map(|s| VideoScene {
                    narration: s.get("narration").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    visual_desc: s.get("visual_desc").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    duration_secs: s.get("duration_secs").and_then(|v| v.as_f64()).unwrap_or(5.0),
                    search_keywords: s.get("search_keywords")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default(),
                }).collect()
            }).unwrap_or_else(|| vec![VideoScene {
                narration: format!("Exploring {}", topic), visual_desc: topic.to_string(),
                duration_secs, search_keywords: vec![topic.to_string()],
            }]);
            return VideoScript { title, scenes, bgm_mood: bgm, total_duration_secs: duration_secs };
        }
        self.generate_template(topic, duration_secs)
    }

    fn generate_template(&self, topic: &str, duration_secs: f64) -> VideoScript {
        let per_scene = (duration_secs / 5.0).ceil().min(10.0);
        let scenes = (0..(duration_secs / per_scene).ceil() as usize)
            .map(|i| {
                let angle = match i % 4 { 0 => "overview", 1 => "why it matters", 2 => "how it works", _ => "future outlook" };
                VideoScene {
                    narration: format!("{}: {} — {}", topic, angle, "built with Rust, evolving every cycle"),
                    visual_desc: format!("{} {}", topic, angle),
                    duration_secs: per_scene,
                    search_keywords: vec![topic.to_string(), angle.to_string()],
                }
            }).collect();
        VideoScript {
            title: topic.to_string(), scenes, bgm_mood: "neutral".into(),
            total_duration_secs: duration_secs,
        }
    }
}

/// 内容计划
#[derive(Clone, Debug)]
pub struct ContentPlan {
    pub title: String,
    pub body: String,
    pub content_type: super::publisher::ContentType,
    pub platforms: Vec<String>,
    pub media_paths: Vec<String>,
    pub tags: Vec<String>,
    pub schedule_at: Option<String>,
    pub video_script: Option<VideoScript>,
    pub output_video_path: Option<String>,
}

/// 内容主题
#[derive(Clone, Debug)]
pub struct ContentTopic {
    pub pillar: String,
    pub angle: String,
    pub target_platforms: Vec<String>,
}

/// 内容策略
#[derive(Clone, Debug)]
pub struct StrategyConfig {
    pub brand_name: String,
    pub brand_tagline: String,
    pub posting_interval_hours: u32,
    pub default_video_duration_secs: f64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            brand_name: "NeoTrix".into(),
            brand_tagline: "The Self-Improving Reasoning Agent".into(),
            posting_interval_hours: 48,
            default_video_duration_secs: 30.0,
        }
    }
}

/// 内容规划器 — 决定发什么、发到哪
pub struct ContentPlanner {
    config: StrategyConfig,
    published_count: HashMap<String, u64>,
    script_planner: VideoScriptPlanner,
}

impl ContentPlanner {
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            config,
            published_count: HashMap::new(),
            script_planner: VideoScriptPlanner::new(),
        }
    }

    pub fn with_script_planner(mut self, planner: VideoScriptPlanner) -> Self {
        self.script_planner = planner;
        self
    }

    pub fn config(&self) -> &StrategyConfig { &self.config }
    pub fn script_planner(&self) -> &VideoScriptPlanner { &self.script_planner }

    /// 生成下一个内容计划
    pub fn plan_next(&mut self, best_platform: Option<&str>) -> ContentPlan {
        let platform = best_platform
            .filter(|p| !p.is_empty())
            .unwrap_or("twitter");
        let count = self.published_count.get(platform).copied().unwrap_or(0);
        *self.published_count.entry(platform.to_string()).or_insert(0) += 1;

        let is_video_platform = platform == "youtube" || platform == "bilibili" || platform == "douyin"
            || platform == "kuaishou" || platform == "xiaohongshu" || platform == "tiktok";

        let (title, body) = match (platform, count) {
            ("twitter", 0) => (
                format!("{}: The Core Claim", self.config.brand_name),
                format!("Every AI coding tool today: LLM in → tokens out → diff applied.\nNone measure their own reasoning. None learn. None improve.\n{} is the first open-source agent that does.\nBuilt with Rust. Zero unsafe. 1,715+ tests.", self.config.brand_name),
            ),
            ("twitter", 1) => (
                format!("{}: The Architecture", self.config.brand_name),
                "No separate experts. No MoE routing.\nA single 23-dim capability vector that encodes everything.\nEvery task updates it. Every success absorbed. Every failure traced.".to_string(),
            ),
            ("wechat", _) => (
                format!("{} 深度解析：从工具到 Agent 的进化之路", self.config.brand_name),
                format!("{} — {}\n\n## Why all AI coding tools are wrong\nAgents that cannot examine their own reasoning are tools, not agents.\n\n## The SEAL loop\nSelf-Editing Architecture Learning — every output is measured, absorbed, or corrected.", self.config.brand_name, self.config.brand_tagline),
            ),
            _ if is_video_platform => (
                format!("{} — {}", self.config.brand_name, ["Architecture Deep Dive", "SEAL Loop Explained", "Building Self-Improving AI", "Rust Cognitive OS"][count as usize % 4]),
                format!("In this video: {} — {}. Full architecture breakdown with E8 reasoning engine, HyperCube knowledge representation, and GWT attention routing.", self.config.brand_name, self.config.brand_tagline),
            ),
            _ => (
                format!("{} — Day {}", self.config.brand_name, count + 1),
                format!("{}: {}. Open source. Self-improving. Built with Rust.", self.config.brand_name, self.config.brand_tagline),
            ),
        };

        let video_script = if is_video_platform {
            let platform_label = if platform == "bilibili" || platform == "douyin" { "short" } else { "youtube" };
            Some(self.script_planner.generate(&title, platform_label, self.config.default_video_duration_secs))
        } else { None };

        ContentPlan {
            title,
            body,
            content_type: if is_video_platform {
                super::publisher::ContentType::Video
            } else if platform == "instagram" {
                super::publisher::ContentType::Image
            } else if platform == "wechat" {
                super::publisher::ContentType::Article
            } else {
                super::publisher::ContentType::Text
            },
            platforms: vec![platform.to_string()],
            media_paths: vec![],
            tags: vec![],
            schedule_at: None,
            video_script,
            output_video_path: None,
        }
    }

    /// 从已有的内容日历中选择下一个
    pub fn next_from_calendar(&self, calendar: &[ContentTopic], video: bool) -> Option<ContentPlan> {
        calendar.first().map(|topic| {
            let is_video = video || topic.target_platforms.iter().any(|p| matches!(p.as_str(), "youtube" | "bilibili" | "douyin" | "tiktok"));
            ContentPlan {
                title: format!("{} — {}", self.config.brand_name, topic.angle),
                body: format!("Exploring {} from the perspective of {}.", topic.pillar, self.config.brand_name),
                content_type: if is_video { super::publisher::ContentType::Video } else { super::publisher::ContentType::Article },
                platforms: topic.target_platforms.clone(),
                media_paths: vec![],
                tags: vec![topic.pillar.clone()],
                schedule_at: None,
                video_script: if is_video {
                    Some(self.script_planner.generate(&topic.angle, "youtube", self.config.default_video_duration_secs))
                } else { None },
                output_video_path: None,
            }
        })
    }
}

/// ContentPlan 简易 builder
pub fn plan_for_video(title: &str, platforms: &[&str], script: VideoScript) -> ContentPlan {
    ContentPlan {
        title: title.to_string(),
        body: String::new(),
        content_type: super::publisher::ContentType::Video,
        platforms: platforms.iter().map(|s| s.to_string()).collect(),
        media_paths: vec![],
        tags: vec![],
        schedule_at: None,
        video_script: Some(script),
        output_video_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_act_earn::ContentType;

    #[test]
    fn test_video_script_template() {
        let planner = VideoScriptPlanner::new();
        let script = planner.generate("NeoTrix Architecture", "youtube", 30.0);
        assert!(script.scenes.len() >= 3);
        assert!((script.total_duration_secs - 30.0).abs() < 10.0);
    }

    #[test]
    fn test_plan_video_platform_generates_script() {
        let mut planner = ContentPlanner::new(StrategyConfig::default());
        let plan = planner.plan_next(Some("youtube"));
        assert_eq!(plan.content_type, ContentType::Video);
        assert!(plan.video_script.is_some());
    }

    #[test]
    fn test_plan_twitter_no_video_script() {
        let mut planner = ContentPlanner::new(StrategyConfig::default());
        let plan = planner.plan_next(Some("twitter"));
        assert_eq!(plan.content_type, ContentType::Text);
        assert!(plan.video_script.is_none());
    }

    #[test]
    fn test_content_type_mapping() {
        let mut planner = ContentPlanner::new(StrategyConfig::default());
        assert_eq!(planner.plan_next(Some("youtube")).content_type, ContentType::Video);
        assert_eq!(planner.plan_next(Some("douyin")).content_type, ContentType::Video);
        assert_eq!(planner.plan_next(Some("instagram")).content_type, ContentType::Image);
        assert_eq!(planner.plan_next(Some("twitter")).content_type, ContentType::Text);
    }

    #[test]
    fn test_plan_for_video_builder() {
        let script = VideoScriptPlanner::new().generate("Test", "youtube", 15.0);
        let plan = plan_for_video("Test Video", &["youtube", "bilibili"], script.clone());
        assert!(plan.video_script.is_some());
        assert_eq!(plan.platforms.len(), 2);
    }
}
