use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::neotrix::nt_act_social::connector::{Platform, VideoInfo};

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub relevance_weight: f64,
    pub recency_weight: f64,
    pub engagement_weight: f64,
    pub diversity_weight: f64,
    pub quality_weight: f64,
    pub novelty_weight: f64,
    pub min_score: f64,
    pub max_results: usize,
}

impl Default for FilterConfig {
    fn default() -> Self {
        FilterConfig {
            relevance_weight: 0.30,
            recency_weight: 0.15,
            engagement_weight: 0.20,
            diversity_weight: 0.10,
            quality_weight: 0.10,
            novelty_weight: 0.15,
            min_score: 0.25,
            max_results: 50,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterContext {
    pub interest_keywords: Vec<String>,
    pub preferred_platforms: Vec<Platform>,
    pub max_age_hours: Option<f64>,
}

impl Default for FilterContext {
    fn default() -> Self {
        FilterContext {
            interest_keywords: Vec::new(),
            preferred_platforms: vec![
                Platform::YouTube,
                Platform::TikTok,
                Platform::Douyin,
                Platform::Twitter,
                Platform::Reddit,
                Platform::Instagram,
                Platform::Bilibili,
            ],
            max_age_hours: Some(168.0),
        }
    }
}

pub struct SelfControlledFilter {
    config: FilterConfig,
    seen_ids: Mutex<HashSet<String>>,
    user_feedback: Mutex<HashMap<String, f64>>,
    keyword_scores: Mutex<HashMap<String, f64>>,
}

impl SelfControlledFilter {
    pub fn new(config: FilterConfig) -> Self {
        SelfControlledFilter {
            config,
            seen_ids: Mutex::new(HashSet::new()),
            user_feedback: Mutex::new(HashMap::new()),
            keyword_scores: Mutex::new(HashMap::new()),
        }
    }

    pub fn filter_and_rank(&self, moments: Vec<VideoInfo>, context: &FilterContext) -> Vec<ScoredMoment> {
        let mut scored: Vec<ScoredMoment> = moments
            .into_iter()
            .map(|m| {
                let score = self.compute_score(&m, context);
                ScoredMoment { moment: m, score }
            })
            .filter(|s| s.score >= self.config.min_score)
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let mut seen = self.seen_ids.lock().expect("Mutex poisoned");
        for s in &scored {
            seen.insert(s.moment.id.clone());
        }

        scored.truncate(self.config.max_results);
        scored
    }

    pub fn apply_diversity_boost(&self, scored: Vec<ScoredMoment>) -> Vec<ScoredMoment> {
        let mut seen_sources: HashSet<String> = HashSet::new();
        let mut result = Vec::with_capacity(scored.len());

        for s in scored {
            let source_key = format!("{:?}:{}", s.moment.platform, s.moment.author.as_deref().unwrap_or("unknown"));
            if seen_sources.contains(&source_key) {
                let boosted = ScoredMoment {
                    score: s.score * 0.85,
                    ..s
                };
                result.push(boosted);
            } else {
                seen_sources.insert(source_key);
                result.push(s);
            }
        }

        result.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn record_feedback(&self, moment_id: &str, liked: bool) {
        let mut feedback = self.user_feedback.lock().expect("Mutex poisoned");
        let delta = if liked { 0.1 } else { -0.15 };
        let entry = feedback.entry(moment_id.to_string()).or_insert(0.0);
        *entry = (*entry + delta).clamp(-1.0, 1.0);

        drop(feedback);
        self.tune_weights();
    }

    pub fn record_keyword_feedback(&self, keyword: &str, positive: bool) {
        let mut scores = self.keyword_scores.lock().expect("Mutex poisoned");
        let delta = if positive { 0.05 } else { -0.05 };
        let entry = scores.entry(keyword.to_string()).or_insert(0.5);
        *entry = (*entry + delta).clamp(0.0, 1.0);
    }

    pub fn top_keywords(&self, n: usize) -> Vec<(String, f64)> {
        let scores = self.keyword_scores.lock().expect("Mutex poisoned");
        let mut kw: Vec<_> = scores.iter().map(|(k, v)| (k.clone(), *v)).collect();
        kw.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        kw.truncate(n);
        kw
    }

    pub fn config(&self) -> &FilterConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut FilterConfig {
        &mut self.config
    }

    fn compute_score(&self, moment: &VideoInfo, context: &FilterContext) -> f64 {
        let cfg = &self.config;
        let relevance = self.calc_relevance(moment, context) * cfg.relevance_weight;
        let recency = self.calc_recency(moment, context) * cfg.recency_weight;
        let engagement = self.calc_engagement(moment) * cfg.engagement_weight;
        let diversity = self.calc_diversity(moment) * cfg.diversity_weight;
        let quality = self.calc_quality(moment) * cfg.quality_weight;
        let novelty = self.calc_novelty(moment) * cfg.novelty_weight;

        relevance + recency + engagement + diversity + quality + novelty
    }

    fn calc_relevance(&self, moment: &VideoInfo, context: &FilterContext) -> f64 {
        if context.interest_keywords.is_empty() {
            return 0.5;
        }
        let text = format!("{} {} {}", moment.title, moment.description, moment.author.as_deref().unwrap_or(""));
        let text_lower = text.to_lowercase();

        let kw_scores = self.keyword_scores.lock().expect("Mutex poisoned");
        let mut total = 0.0;
        let mut matched = 0;
        for kw in &context.interest_keywords {
            if text_lower.contains(&kw.to_lowercase()) {
                let boost = kw_scores.get(kw).copied().unwrap_or(0.5);
                total += boost;
                matched += 1;
            }
        }
        if matched == 0 {
            0.1
        } else {
            (total / matched as f64).min(1.0)
        }
    }

    fn calc_recency(&self, moment: &VideoInfo, context: &FilterContext) -> f64 {
        if let Some(ts) = moment.published_at {
            let age = chrono::Utc::now().signed_duration_since(ts);
            let hours = age.num_hours() as f64;
            if let Some(max_h) = context.max_age_hours {
                if hours > max_h {
                    return 0.0;
                }
            }
            (1.0_f64 - (hours / 720.0_f64).min(1.0_f64)).max(0.1_f64)
        } else {
            0.3
        }
    }

    fn calc_engagement(&self, moment: &VideoInfo) -> f64 {
        let views = moment.view_count.unwrap_or(0) as f64;
        let likes = moment.like_count.unwrap_or(0) as f64;
        let comments = moment.comment_count.unwrap_or(0) as f64;

        let view_score = (views / 1_000_000.0).min(1.0);
        let like_score = (likes / 100_000.0).min(1.0);
        let comment_score = (comments / 10_000.0).min(1.0);

        if views == 0.0 && likes == 0.0 && comments == 0.0 {
            0.2
        } else {
            (view_score * 0.4 + like_score * 0.4 + comment_score * 0.2).min(1.0)
        }
    }

    fn calc_diversity(&self, moment: &VideoInfo) -> f64 {
        let seen = self.seen_ids.lock().expect("Mutex poisoned");
        if seen.contains(&moment.id) {
            0.1
        } else {
            1.0
        }
    }

    fn calc_quality(&self, moment: &VideoInfo) -> f64 {
        let mut score: f64 = 0.5;
        if moment.thumbnail_url.is_some() {
            score += 0.15;
        }
        if moment.duration_secs.is_some() {
            let d = moment.duration_secs.unwrap_or(0) as f64;
            if (30.0..=3600.0).contains(&d) {
                score += 0.15;
            }
        }
        if moment.description.len() > 20 {
            score += 0.1;
        }
        if moment.view_count.unwrap_or(0) > 100 {
            score += 0.1;
        }
        score.min(1.0)
    }

    fn calc_novelty(&self, _moment: &VideoInfo) -> f64 {
        0.8
    }

    fn tune_weights(&self) {
        let feedback = self.user_feedback.lock().expect("Mutex poisoned");
        if feedback.len() < 5 {
            return;
        }
        let avg: f64 = feedback.values().copied().sum::<f64>() / feedback.len() as f64;
        if avg > 0.3 {}
    }
}

#[derive(Debug, Clone)]
pub struct ScoredMoment {
    pub moment: VideoInfo,
    pub score: f64,
}

pub struct ScoredMomentsResult {
    pub moments: Vec<ScoredMoment>,
    pub total_input: usize,
    pub filtered_out: usize,
    pub avg_score: f64,
}

pub fn create_default_filter() -> SelfControlledFilter {
    SelfControlledFilter::new(FilterConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_video(id: &str, title: &str, platform: Platform, author: Option<String>, views: Option<u64>) -> VideoInfo {
        VideoInfo {
            id: id.into(), title: title.into(), description: "desc".into(),
            author, duration_secs: Some(120), view_count: views, like_count: Some(100),
            comment_count: Some(10), platform, published_at: None,
            thumbnail_url: None, url: format!("https://x.com/{}", id),
        }
    }

    #[test]
    fn test_filter_config_default() {
        let cfg = FilterConfig::default();
        assert!((cfg.relevance_weight - 0.30).abs() < 1e-9);
        assert_eq!(cfg.max_results, 50);
    }

    #[test]
    fn test_filter_context_default() {
        let ctx = FilterContext::default();
        assert!(!ctx.preferred_platforms.is_empty());
        assert_eq!(ctx.max_age_hours, Some(168.0));
    }

    #[test]
    fn test_filter_and_rank_empty_input() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        let ctx = FilterContext::default();
        let results = filter.filter_and_rank(vec![], &ctx);
        assert!(results.is_empty());
    }

    #[test]
    fn test_filter_and_rank_with_moments() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        let ctx = FilterContext::default();
        let moments = vec![
            make_video("1", "test title", Platform::YouTube, Some("author".into()), Some(1000)),
        ];
        let results = filter.filter_and_rank(moments, &ctx);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_filter_and_rank_min_score_filtering() {
        let mut cfg = FilterConfig::default();
        cfg.min_score = 1.5;
        let filter = SelfControlledFilter::new(cfg);
        let ctx = FilterContext::default();
        let moments = vec![
            make_video("1", "low quality", Platform::YouTube, None, Some(0)),
        ];
        let results = filter.filter_and_rank(moments, &ctx);
        assert!(results.is_empty());
    }

    #[test]
    fn test_apply_diversity_boost() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        let m1 = make_video("1", "a", Platform::YouTube, Some("same".into()), Some(100));
        let m2 = make_video("2", "b", Platform::YouTube, Some("same".into()), Some(100));
        let scored = vec![
            ScoredMoment { moment: m1, score: 1.0 },
            ScoredMoment { moment: m2, score: 0.9 },
        ];
        let result = filter.apply_diversity_boost(scored);
        assert_eq!(result.len(), 2);
        assert!((result[1].score - 0.9 * 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_record_feedback_positive() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        filter.record_feedback("video_1", true);
        let fb = filter.user_feedback.lock().unwrap();
        assert!((fb.get("video_1").copied().unwrap_or(0.0) - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_record_feedback_negative() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        filter.record_feedback("video_1", false);
        let fb = filter.user_feedback.lock().unwrap();
        assert!((fb.get("video_1").copied().unwrap_or(0.0) + 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_record_keyword_feedback() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        filter.record_keyword_feedback("rust", true);
        let kw = filter.keyword_scores.lock().unwrap();
        assert!((kw.get("rust").copied().unwrap_or(0.0) - 0.55).abs() < 1e-9);
    }

    #[test]
    fn test_top_keywords_ordered() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        filter.record_keyword_feedback("rust", true);
        filter.record_keyword_feedback("python", false);
        let top = filter.top_keywords(5);
        assert!(!top.is_empty());
        assert!(top[0].1 >= top.last().map(|x| x.1).unwrap_or(0.0));
    }

    #[test]
    fn test_config_accessor() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        assert_eq!(filter.config().max_results, 50);
    }

    #[test]
    fn test_config_mut_accessor() {
        let mut filter = SelfControlledFilter::new(FilterConfig::default());
        filter.config_mut().max_results = 100;
        assert_eq!(filter.config().max_results, 100);
    }

    #[test]
    fn test_calc_relevance_no_keywords() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        let ctx = FilterContext { interest_keywords: vec![], ..Default::default() };
        let video = make_video("1", "test", Platform::YouTube, None, None);
        let score = filter.calc_relevance(&video, &ctx);
        assert!((score - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_calc_relevance_with_match() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        let ctx = FilterContext { interest_keywords: vec!["test".into()], ..Default::default() };
        let video = make_video("1", "test video", Platform::YouTube, None, None);
        let score = filter.calc_relevance(&video, &ctx);
        assert!(score > 0.1);
    }

    #[test]
    fn test_calc_quality_with_thumbnail_and_duration() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        let video = VideoInfo {
            id: "1".into(), title: "t".into(), description: "long enough description".into(),
            author: None, duration_secs: Some(300), view_count: Some(200), like_count: Some(10),
            comment_count: Some(1), platform: Platform::YouTube, published_at: None,
            thumbnail_url: Some("https://x.com/thumb.jpg".into()), url: "https://x.com/v".into(),
        };
        let score = filter.calc_quality(&video);
        assert!(score > 0.5);
    }

    #[test]
    fn test_calc_engagement_no_engagement() {
        let filter = SelfControlledFilter::new(FilterConfig::default());
        let video = VideoInfo {
            id: "1".into(), title: "t".into(), description: "".into(),
            author: None, duration_secs: None, view_count: None,
            like_count: None, comment_count: None,
            platform: Platform::YouTube, published_at: None,
            thumbnail_url: None, url: "https://x.com/1".into(),
        };
        let score = filter.calc_engagement(&video);
        assert!((score - 0.2).abs() < 1e-9);
    }
}
