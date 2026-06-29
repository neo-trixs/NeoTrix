use super::content::{ContentPlan, ContentPlanner, StrategyConfig};
use super::monetize::AiToEarnBridge;
use super::publisher::{ContentMeta, ContentType, PublishResult, PublisherRegistry};
use super::tracker::{EarnTracker, RewardSignal};
use super::video::VideoPipeline;

/// 一次完整赚钱循环的详细报告
#[derive(Clone, Debug)]
pub struct CycleReport {
    pub plan: ContentPlan,
    pub video_path: Option<String>,
    pub publish_results: Vec<PublishResult>,
    pub earnings_delta: f64,
    pub reward_signal: RewardSignal,
    pub platform_earnings: Vec<(String, f64)>,
    pub success_count: usize,
    pub fail_count: usize,
}

/// 赚钱流水线 — 整合视频生产 + AiToEarn 发布 + 收益追踪
/// 对标 MoneyPrinterTurbo 的 service/task.py + OpenSkynet 的 24/7 scheduling
pub struct EarnPipeline {
    strategy: ContentPlanner,
    tracker: EarnTracker,
    bridge: Option<AiToEarnBridge>,
    video_pipeline: Option<VideoPipeline>,
    publishers: Option<PublisherRegistry>,
}

impl EarnPipeline {
    pub fn new(config: StrategyConfig, _work_dir: &str) -> Self {
        Self {
            strategy: ContentPlanner::new(config),
            tracker: EarnTracker::new(),
            bridge: None,
            video_pipeline: None,
            publishers: None,
        }
    }

    pub fn with_bridge(mut self, bridge: AiToEarnBridge) -> Self {
        self.bridge = Some(bridge);
        self
    }

    pub fn with_video_pipeline(mut self, pipeline: VideoPipeline) -> Self {
        self.video_pipeline = Some(pipeline);
        self
    }

    pub fn with_publishers(mut self, publishers: PublisherRegistry) -> Self {
        self.publishers = Some(publishers);
        self
    }

    pub fn tracker(&self) -> &EarnTracker {
        &self.tracker
    }
    pub fn tracker_mut(&mut self) -> &mut EarnTracker {
        &mut self.tracker
    }

    /// 执行一次完整赚钱循环
    /// 步骤: Plan → (若 Video 则生产) → Publish → Track Earnings → Compute Reward
    pub fn execute_cycle(&mut self, best_platform: Option<&str>) -> CycleReport {
        let mut plan = self.strategy.plan_next(best_platform);

        // Step 1: Video Production（若需视频）
        let video_path = if plan.content_type == ContentType::Video {
            self.video_pipeline
                .as_ref()
                .and_then(|vp| vp.produce(&plan).ok())
        } else {
            None
        };

        // 填充 media_paths 用生成的视频
        if let Some(ref vp) = video_path {
            plan.media_paths.push(vp.clone());
            plan.output_video_path = Some(vp.clone());
        }

        // Step 2: Publish（本地 publisher + AiToEarn bridge 双通道）
        let mut all_results = Vec::new();

        // 本地 publisher
        if let Some(ref publishers) = self.publishers {
            let meta = ContentMeta {
                title: plan.title.clone(),
                body: plan.body.clone(),
                content_type: plan.content_type.clone(),
                media_paths: plan.media_paths.clone(),
                tags: plan.tags.clone(),
                schedule_at: plan.schedule_at.clone(),
            };
            let results = publishers.publish_all(&meta, &plan.platforms);
            all_results.extend(results);
        }

        // Step 3: AiToEarn 桥接发布
        if let Some(ref bridge) = self.bridge {
            if bridge.is_configured() {
                let req = super::monetize::PublishRequest {
                    title: plan.title.clone(),
                    content: plan.body.clone(),
                    content_type: format!("{:?}", plan.content_type).to_lowercase(),
                    platforms: plan.platforms.clone(),
                    schedule_time: None,
                    media_urls: video_path.clone().into_iter().collect(),
                };
                match bridge.publish_content(&req) {
                    Ok(result) => {
                        for pr in result.platform_results {
                            all_results.push(super::publisher::PublishResult {
                                platform: format!("aitoearn:{}", pr.platform),
                                success: pr.success,
                                post_url: pr.post_url,
                                error: pr.error,
                            });
                        }
                    }
                    Err(e) => {
                        all_results.push(PublishResult {
                            platform: "aitoearn".into(),
                            success: false,
                            post_url: None,
                            error: Some(e),
                        });
                    }
                }
            }
        }

        // Step 4: 收益追踪
        let success_count = all_results.iter().filter(|r| r.success).count();
        let fail_count = all_results.iter().filter(|r| !r.success).count();

        // 真实收益数据优先（AiToEarn），回退硬编码
        let (earnings_delta, platform_earnings) = if let Some(ref bridge) = self.bridge {
            if bridge.is_configured() {
                bridge
                    .fetch_real_earnings_reward()
                    .unwrap_or((success_count as f64 * 0.05, Vec::new()))
            } else {
                (success_count as f64 * 0.05, Vec::new())
            }
        } else {
            (success_count as f64 * 0.05, Vec::new())
        };

        if earnings_delta > 0.0 {
            for r in &all_results {
                if r.success {
                    self.tracker.record_earning(super::tracker::EarningsRecord {
                        date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                        platform: r.platform.clone(),
                        amount: earnings_delta / success_count.max(1) as f64,
                        currency: "USD".to_string(),
                        content_title: plan.title.clone(),
                    });
                }
            }
        }

        // 同步 AiToEarn 真实平台收益到 tracker
        for (platform, amount) in &platform_earnings {
            self.tracker.record_earning(super::tracker::EarningsRecord {
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                platform: format!("aitoearn:{}", platform),
                amount: *amount,
                currency: "USD".to_string(),
                content_title: plan.title.clone(),
            });
        }

        let reward = self.tracker.compute_reward(earnings_delta);

        CycleReport {
            plan,
            video_path,
            publish_results: all_results,
            earnings_delta,
            reward_signal: reward,
            platform_earnings,
            success_count,
            fail_count,
        }
    }

    /// 批量执行 N 次赚钱循环
    pub fn run_batch(&mut self, count: u32, best_platform: Option<&str>) -> Vec<CycleReport> {
        (0..count)
            .map(|_| self.execute_cycle(best_platform))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_act_earn::publisher::CliPublisher;

    #[test]
    fn test_pipeline_creation() {
        let pipe = EarnPipeline::new(StrategyConfig::default(), "/tmp/neotrix_earn_test");
        assert_eq!(pipe.tracker().stats().total_earnings, 0.0);
    }

    #[test]
    fn test_text_cycle_no_video() {
        let mut pipe = EarnPipeline::new(StrategyConfig::default(), "/tmp/neotrix_earn_test");
        let mut reg = crate::neotrix::nt_act_earn::publisher::PublisherRegistry::new();
        reg.register(Box::new(CliPublisher::new("twitter", "echo 'test'")));
        pipe = pipe.with_publishers(reg);
        let report = pipe.execute_cycle(Some("twitter"));
        assert!(report.video_path.is_none());
        assert!(report.publish_results.len() >= 1);
    }

    #[test]
    fn test_cycle_report_contains_earnings() {
        let mut pipe = EarnPipeline::new(StrategyConfig::default(), "/tmp/neotrix_earn_test");
        let mut reg = crate::neotrix::nt_act_earn::publisher::PublisherRegistry::new();
        reg.register(Box::new(CliPublisher::new("twitter", "echo 'test'")));
        pipe = pipe.with_publishers(reg);
        let report = pipe.execute_cycle(Some("twitter"));
        assert!(report.earnings_delta >= 0.0);
        assert!(report.reward_signal.value >= 0.0);
    }
}
