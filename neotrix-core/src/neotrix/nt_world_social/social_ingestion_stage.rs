use super::tweet_stream::TweetStream;
use super::x_scraper::XScraper;
use crate::core::nt_core_time::unix_now;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_mind::self_iterating::pipeline::{BrainStage, StageDecision};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind_ingestion::scratchpad::IngestionScratchpad;
use crate::neotrix::nt_mind_ingestion::{IngestionConfig, IngestionSourceType};

/// SocialIngestionStage — 负熵驱动的社交媒体信息吸收
///
/// 设计:
/// 1. 好奇心触发: 当知识缺口 > 阈值 或 curiosity_bonus > 0.3 时激活
/// 2. 浏览器爬取: CamoFox 无头浏览器访问 X.com 时间线
/// 3. 负熵评分: TweetStream 去重 + novelty + signal purity + information gain
/// 4. 选择性吸收: 仅 negentropy > 0.25 的推文进入 IngestionScratchpad
/// 5. Reward 反馈: 信息增益 → reward bonus
pub struct SocialIngestionStage {
    scraper: std::sync::Mutex<XScraper>,
    stream: std::sync::Mutex<TweetStream>,
    last_scrape: std::sync::Mutex<u64>,
    scrape_interval_secs: u64,
    known_concepts: std::sync::Mutex<Vec<String>>,
}

impl SocialIngestionStage {
    pub fn new() -> Self {
        Self {
            scraper: std::sync::Mutex::new(XScraper::new()),
            stream: std::sync::Mutex::new(TweetStream::new()),
            last_scrape: std::sync::Mutex::new(0),
            scrape_interval_secs: 300,
            known_concepts: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn with_interval(mut self, secs: u64) -> Self {
        self.scrape_interval_secs = secs;
        self
    }

    /// 更新 known_concepts 从 brain 的知识库
    fn refresh_known_concepts(&self, brain: &SelfIteratingBrain) {
        if let Some(ref kb) = brain._nt_memory_kb {
            if let Ok(results) = kb.search("", 50) {
                let titles: Vec<String> = results
                    .iter()
                    .map(|r| r.node.title.clone())
                    .filter(|t| t.len() > 3)
                    .collect();
                if let Ok(mut kc) = self.known_concepts.lock() {
                    *kc = titles;
                }
            }
        }
    }

    /// 触发知识缺口 → 好奇心信号
    fn curiosity_signal(brain: &SelfIteratingBrain) -> f64 {
        let entropy = brain.entropy_crisis_level;
        let curiosity = brain.curiosity_bonus;
        let gap = brain
            .goal_state
            .goal_contract
            .as_ref()
            .map(|g| {
                let done = g.phases.iter().filter(|p| p.verified).count();
                let total = g.phases.len().max(1);
                done as f64 / total as f64
            })
            .unwrap_or(0.5);
        let stagnation = brain.stagnation.stats().consecutive_zero_reward as f64 * 0.05;
        (entropy * 0.3 + curiosity * 0.4 + (1.0 - gap) * 0.2 + stagnation * 0.1).clamp(0.0, 1.0)
    }

    /// 将推文注入 IngestionScratchpad
    fn inject_to_scratchpad(
        brain: &mut SelfIteratingBrain,
        text: &str,
        tag: &str,
        negentropy: f64,
    ) {
        let tagged = format!("[{}] {}", tag, text);
        let pad = IngestionScratchpad::new(
            tagged,
            IngestionSourceType::Social,
            IngestionConfig {
                source_type: IngestionSourceType::Social,
                max_rounds: 3,
                convergence_threshold: 0.05,
                quality_threshold: 0.5,
                auto_store: true,
            },
        );
        brain._ingestion_scratchpad = Some(pad);

        log::info!(
            "[social-ingest] negentropy={:.3} | {}",
            negentropy,
            text.chars().take(80).collect::<String>()
        );
    }
}

impl Default for SocialIngestionStage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainStage for SocialIngestionStage {
    fn name(&self) -> &str {
        "social_ingestion"
    }

    fn frequency(&self) -> usize {
        10
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // 1. 好奇心门控 — 只在外界信号足够强时执行
        let curiosity = Self::curiosity_signal(brain);
        if curiosity < 0.15 {
            return Ok(StageDecision::Skip(format!(
                "curiosity too low ({:.3})",
                curiosity
            )));
        }

        // 2. 频率门控 — 避免高频轮询
        let now = unix_now() as u64;
        if let Ok(last) = self.last_scrape.lock() {
            if now - *last < self.scrape_interval_secs {
                return Ok(StageDecision::Skip("scrape interval not elapsed".into()));
            }
        }

        log::info!(
            "[social-ingestion] curiosity={:.3} — activating scrape",
            curiosity
        );

        // 3. 更新已知概念
        self.refresh_known_concepts(brain);

        // 4. 爬取 X.com 时间线
        let mut guard = self
            .scraper
            .lock()
            .map_err(|e| NeoTrixError::Brain(format!("scraper lock: {}", e)))?;
        let timeline = match guard.scrape_home_timeline(20) {
            Ok(tl) => tl,
            Err(e) => {
                log::warn!("[social-ingestion] scrape failed: {}", e);
                return Ok(StageDecision::Continue);
            }
        };
        drop(guard);

        if timeline.tweets.is_empty() {
            log::debug!("[social-ingestion] no tweets found");
            return Ok(StageDecision::Continue);
        }

        // 5. 记录爬取时间
        if let Ok(mut last) = self.last_scrape.lock() {
            *last = now;
        }

        // 6. 负熵处理管线
        let known = self
            .known_concepts
            .lock()
            .map(|kc| kc.clone())
            .unwrap_or_default();

        let mut stream = self
            .stream
            .lock()
            .map_err(|e| NeoTrixError::Brain(format!("stream lock: {}", e)))?;

        let absorbed = stream.process_timeline(&timeline.tweets, &known, curiosity, 5);

        if absorbed.is_empty() {
            log::debug!("[social-ingestion] all tweets filtered out (low negentropy)");
            return Ok(StageDecision::Continue);
        }

        // 7. 将最高负熵的推文注入 consciousness
        for score in &absorbed {
            let ingest_text = super::tweet_stream::TweetStream::score_to_ingestion_text(score);
            Self::inject_to_scratchpad(
                brain,
                &ingest_text,
                &format!("X/{}", score.raw_tweet.author_handle),
                score.negentropy,
            );

            // 提取关键词反馈到 known_concepts
            let keywords =
                super::tweet_stream::TweetStream::extract_keywords(&score.raw_tweet.text);
            if let Ok(mut kc) = self.known_concepts.lock() {
                for kw in keywords {
                    if !kc.contains(&kw) {
                        kc.push(kw);
                    }
                }
                kc.truncate(200);
            }
        }

        // 8. Reward: 信息增益 → 内在奖励
        let total_gain: f64 = absorbed.iter().map(|s| s.information_gain).sum();
        let reward_bonus = (total_gain * 0.1).min(0.3);
        brain._set_reward(brain._reward() + reward_bonus);

        log::info!(
            "[social-ingestion] absorbed {} tweets, total_gain={:.3}, reward_bonus={:.3}",
            absorbed.len(),
            total_gain,
            reward_bonus,
        );

        // 9. 更新 curiosity_bonus — 信息增益越高, curiosity 越满足
        let curiosity_decay = (total_gain * 0.5).min(0.2);
        brain.curiosity_bonus = (brain.curiosity_bonus - curiosity_decay).max(0.0);

        // 10. 统计
        let stats = stream.stats();
        let _ = stats;

        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_ingestion_stage_construction() {
        let stage = SocialIngestionStage::new();
        assert_eq!(stage.name(), "social_ingestion");
        assert_eq!(stage.frequency(), 10);
    }

    #[test]
    fn test_with_interval() {
        let stage = SocialIngestionStage::new().with_interval(600);
        assert_eq!(stage.scrape_interval_secs, 600);
    }

    #[test]
    fn test_inject_to_scratchpad_sets_pad() {
        let mut brain = SelfIteratingBrain::new();
        SocialIngestionStage::inject_to_scratchpad(
            &mut brain,
            "test tweet with enough content for ingestion",
            "X/testuser",
            0.5,
        );
        assert!(brain._ingestion_scratchpad.is_some());
        if let Some(ref pad) = brain._ingestion_scratchpad {
            assert!(pad.input.contains("test tweet"));
            assert_eq!(pad.source_type, IngestionSourceType::Social);
        }
    }
}
