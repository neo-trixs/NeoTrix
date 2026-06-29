use super::content::{ContentPlan, ContentPlanner, StrategyConfig};
use super::monetize::AiToEarnBridge;
use super::pipeline::EarnPipeline;
use super::publisher::{PublishResult, PublisherRegistry};
use super::tracker::{EarnStats, EarnTracker, EarningsRecord};
use super::video::VideoPipeline;
use crate::neotrix::nt_act_crypto::{ChainType, CryptoAgent, OpportunityType};

use super::financial_abstraction::FinancialAbstractionStack;
use super::knowledge_arbitrage::KnowledgeArbitrageEngine;
use super::wealth_model::{CapitalSource, WealthModel};
use std::sync::{Arc, Mutex};

/// Agent 运行状态
#[derive(Clone, Debug, PartialEq)]
pub enum AgentState {
    Idle,
    Running,
    Published,
    Error(String),
}

/// 一次循环的结果
#[derive(Clone, Debug)]
pub struct CycleResult {
    pub plan: ContentPlan,
    pub publish_results: Vec<PublishResult>,
    pub success_count: usize,
    pub fail_count: usize,
    pub earnings_delta: f64,
    pub video_path: Option<String>,
    pub platform_earnings: Vec<(String, f64)>,
}

/// 自包含赚钱 Agent — 第3代: 整合财富理论 + 金融抽象 + 知识套利
///
/// 三大引擎:
/// - WealthModel: 金字塔财富积累机制（杠杆×不对称性×复利）
/// - FinancialAbstractionStack: 实体→虚拟金融抽象堆叠（证券化/衍生品/做市）
/// - KnowledgeArbitrageEngine: 互联网信息套利（信息聚合→信号→变现）
pub struct EarnAgent {
    planner: ContentPlanner,
    publishers: PublisherRegistry,
    tracker: EarnTracker,
    state: AgentState,
    total_cycles: u64,
    llm: Option<super::LlmEngine>,
    video_pipeline: Option<VideoPipeline>,
    aitoearn_bridge: Option<AiToEarnBridge>,
    work_dir: String,

    // ── 三大新引擎 ──
    pub wealth_model: WealthModel,
    pub financial_stack: FinancialAbstractionStack,
    pub knowledge_engine: KnowledgeArbitrageEngine,

    // ── 加密引擎 ──
    pub nt_act_crypto: Option<Arc<Mutex<CryptoAgent>>>,
}

impl EarnAgent {
    pub fn new() -> Self {
        Self {
            planner: ContentPlanner::new(StrategyConfig::default()),
            publishers: PublisherRegistry::new(),
            tracker: EarnTracker::new(),
            state: AgentState::Idle,
            total_cycles: 0,
            llm: None,
            video_pipeline: None,
            aitoearn_bridge: None,
            work_dir: default_work_dir(),
            wealth_model: WealthModel::default(),
            financial_stack: FinancialAbstractionStack::default(),
            knowledge_engine: KnowledgeArbitrageEngine::default(),
            nt_act_crypto: None,
        }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        Self {
            planner: ContentPlanner::new(config),
            publishers: PublisherRegistry::new(),
            tracker: EarnTracker::new(),
            state: AgentState::Idle,
            total_cycles: 0,
            llm: None,
            video_pipeline: None,
            aitoearn_bridge: None,
            work_dir: default_work_dir(),
            wealth_model: WealthModel::default(),
            financial_stack: FinancialAbstractionStack::default(),
            knowledge_engine: KnowledgeArbitrageEngine::default(),
            nt_act_crypto: None,
        }
    }

    pub fn with_llm(mut self, engine: super::LlmEngine) -> Self {
        self.llm = Some(engine);
        self
    }

    pub fn with_video_pipeline(mut self, pipeline: VideoPipeline) -> Self {
        self.video_pipeline = Some(pipeline);
        self
    }

    pub fn with_aitoearn(mut self, bridge: AiToEarnBridge) -> Self {
        self.aitoearn_bridge = Some(bridge);
        self
    }

    pub fn with_work_dir(mut self, dir: &str) -> Self {
        self.work_dir = dir.to_string();
        self
    }

    pub fn register_publisher(&mut self, name: &str, cmd: &str) {
        let publisher = super::publisher::CliPublisher::new(name, cmd);
        self.publishers.register(Box::new(publisher));
    }

    pub fn set_publishers(&mut self, registry: PublisherRegistry) {
        self.publishers = registry;
    }

    pub fn publishers(&self) -> &PublisherRegistry {
        &self.publishers
    }
    pub fn publishers_mut(&mut self) -> &mut PublisherRegistry {
        &mut self.publishers
    }

    pub fn configure_brand(&mut self, name: &str, tagline: &str) {
        self.planner = ContentPlanner::new(StrategyConfig {
            brand_name: name.to_string(),
            brand_tagline: tagline.to_string(),
            ..self.planner.config().clone()
        });
    }

    /// ── 财富模型操作 ──

    /// 应用杠杆策略
    pub fn deploy_leverage(&mut self, amount: f64, cost_rate: f64, source: CapitalSource) -> f64 {
        self.wealth_model.apply_leverage(amount, cost_rate, source)
    }

    /// 执行证券化 — 打包资产为分层产品
    pub fn securitize_assets(&mut self, assets: &[f64]) -> f64 {
        self.financial_stack.securitize(assets)
    }

    /// 开立套利头寸
    pub fn open_arbitrage_position(
        &mut self,
        instrument: &str,
        notional: f64,
        leverage: f64,
        direction: crate::neotrix::nt_act_earn::financial_abstraction::PositionDirection,
    ) {
        let pos = super::financial_abstraction::DerivativePosition {
            instrument: instrument.into(),
            direction,
            notional,
            entry_price: 0.0,
            mark_price: 0.0,
            leverage,
            margin: notional / leverage,
            liquidation_price: 0.0,
        };
        self.financial_stack.open_position(pos);
    }

    /// 扫描互联网信息套利机会
    pub fn scan_knowledge_opportunities(&mut self) -> usize {
        let opps = self.knowledge_engine.scan_opportunities();
        opps.len()
    }

    /// 模拟一天财富增长
    pub fn simulate_day(&mut self, labor_income: f64, capital_return: f64) {
        self.wealth_model.compound_day(labor_income, capital_return);
    }

    /// ── 核心赚钱循环（增强版）──

    /// 执行一次赚钱循环（完整流程：Plan → 视频生产 → 发布 → 收益追踪 → 财富模型更新）
    pub fn run_cycle(&mut self, best_platform: Option<&str>) -> CycleResult {
        self.state = AgentState::Running;
        self.total_cycles += 1;

        let bp = self.tracker.stats().best_platform.clone();
        let platform: Option<&str> = best_platform.or_else(|| {
            if bp == "none" {
                None
            } else {
                Some(bp.as_str())
            }
        });

        let mut plan = self.planner.plan_next(platform);

        // Step 1: LLM 驱动的内容生成
        if let Some(ref engine) = self.llm {
            if let Ok(gen) = Self::generate_content(engine, &plan) {
                plan.body = gen;
            }
        }

        // Step 2: 视频生产
        let video_path = if plan.content_type == super::publisher::ContentType::Video {
            self.video_pipeline.as_ref().and_then(|vp| {
                let result = vp.produce(&plan);
                if let Ok(ref path) = result {
                    plan.media_paths.push(path.clone());
                    plan.output_video_path = Some(path.clone());
                }
                result.ok()
            })
        } else {
            None
        };

        // Step 3: 发布（本地 publishers + AiToEarn 双通道）
        let mut all_results = Vec::new();

        let meta = super::publisher::ContentMeta {
            title: plan.title.clone(),
            body: plan.body.clone(),
            content_type: plan.content_type.clone(),
            media_paths: plan.media_paths.clone(),
            tags: plan.tags.clone(),
            schedule_at: plan.schedule_at.clone(),
        };
        let results = self.publishers.publish_all(&meta, &plan.platforms);
        all_results.extend(results);

        if let Some(ref bridge) = self.aitoearn_bridge {
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
                            all_results.push(PublishResult {
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

        let (earnings_delta, platform_earnings) = if let Some(ref bridge) = self.aitoearn_bridge {
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
                    self.tracker.record_earning(EarningsRecord {
                        date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                        platform: r.platform.clone(),
                        amount: earnings_delta / success_count.max(1) as f64,
                        currency: "USD".to_string(),
                        content_title: plan.title.clone(),
                    });
                }
            }
        }

        for (platform, amount) in &platform_earnings {
            self.tracker.record_earning(EarningsRecord {
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                platform: format!("aitoearn:{}", platform),
                amount: *amount,
                currency: "USD".to_string(),
                content_title: plan.title.clone(),
            });
        }

        // Step 5: 财富模型更新 — 将收入反馈到财富积累模型
        if earnings_delta > 0.0 {
            let capital_return = if self.wealth_model.capital_layers.total_capital() > 0.0 {
                earnings_delta / self.wealth_model.capital_layers.total_capital() * 365.0
            } else {
                0.05
            };
            self.wealth_model
                .compound_day(0.0, capital_return.max(0.01).min(1.0));
        }

        self.state = if success_count > 0 {
            AgentState::Published
        } else if !all_results.is_empty() {
            AgentState::Error("all publishes failed".to_string())
        } else {
            AgentState::Idle
        };

        CycleResult {
            plan,
            publish_results: all_results,
            success_count,
            fail_count,
            earnings_delta,
            video_path,
            platform_earnings,
        }
    }

    fn generate_content(engine: &super::LlmEngine, plan: &ContentPlan) -> Result<String, String> {
        let platform = plan
            .platforms
            .first()
            .map(|s| s.as_str())
            .unwrap_or("generic");
        let brand = "NeoTrix";

        let prompt = format!(
            "You are a content creator for {brand}.\n\
             Write a {content_type:?} for {platform} about: {title}.\n\n\
             Rules:\n- Be specific, technical, and insightful\n\
             - Use {brand}'s tone: sharp, precise, confident\n\
             - {multi} lines or less for this format\n\
             - Output ONLY the content body, no metadata\n\nContent body:",
            brand = brand,
            content_type = plan.content_type,
            platform = platform,
            title = plan.title,
            multi = if plan
                .platforms
                .iter()
                .any(|p| p == "wechat" || p == "bilibili")
            {
                "200"
            } else {
                "20"
            },
        );

        let request =
            crate::neotrix::nt_io_provider::types::LlmRequest::new(&engine.model, &prompt);
        let response = engine
            .runtime
            .block_on(engine.llm.complete(&request))
            .map_err(|e| format!("LLM generation failed: {}", e))?;
        Ok(crate::neotrix::nt_shield_prompt::default_output_screener().sanitize(&response.content))
    }

    /// 批量运行多次循环
    pub fn run_batch(&mut self, count: u32, best_platform: Option<&str>) -> Vec<CycleResult> {
        (0..count).map(|_| self.run_cycle(best_platform)).collect()
    }

    /// ── 战略洞察 ──

    /// 生成综合性赚钱战略报告
    pub fn strategic_report(&self) -> Vec<String> {
        let mut report = Vec::new();
        report.push("═══ EarnAgent 战略洞察 ═══".into());

        // 财富模型洞察
        report.push("── 财富积累金字塔 ──".into());
        report.extend(self.wealth_model.wealth_insight());
        let best_strats: Vec<String> = self
            .wealth_model
            .suggest_optimal_strategy()
            .iter()
            .map(|m| format!("  {:?}", m))
            .collect();
        if !best_strats.is_empty() {
            report.push("推荐策略:".into());
            report.extend(best_strats);
        }

        // 金融抽象洞察
        report.push("── 金融抽象堆叠 ──".into());
        report.extend(self.financial_stack.financial_insight());
        if self
            .financial_stack
            .derivative_book
            .open_positions
            .is_empty()
        {
            report.push("  无活跃衍生品头寸".into());
        }

        // 知识套利洞察
        report.push("── 互联网知识套利 ──".into());
        report.extend(self.knowledge_engine.arbitrage_insight());

        // 加密资产洞察
        if let Some(ref crypto_arc) = self.nt_act_crypto {
            let crypto = crypto_arc.lock().unwrap_or_else(|e| e.into_inner());
            report.push("── 加密资产 ──".into());
            report.extend(crypto.generate_strategic_insights());
        }

        // 综合评估
        let total_knowledge_capital = self.knowledge_engine.extract_knowledge_capital();
        let abstraction_mult = self.financial_stack.abstraction_multiplier();
        report.push("── 综合评估 ──".into());
        report.push(format!(
            "  知识资本化潜力: ${:.0}/年",
            total_knowledge_capital
        ));
        report.push(format!("  金融抽象倍率: {:.1}x", abstraction_mult));
        report.push(format!(
            "  资产: ${:.2} (自有 ${:.2}, 杠杆 ${:.2})",
            self.wealth_model.capital_layers.total_capital(),
            self.wealth_model.capital_layers.equity_capital,
            self.wealth_model.capital_layers.leveraged_capital,
        ));
        report.push(format!(
            "  资本/劳动收入比: {:.2}",
            self.wealth_model.compounding_tracker.capital_to_labor_ratio
        ));
        report.push(format!(
            "  总收益: ${:.2}",
            self.tracker.stats().total_earnings
        ));
        report.push(format!(
            "  信息源组合质量: {:.1}%",
            self.knowledge_engine.evaluate_source_portfolio() * 100.0
        ));

        report
    }

    /// ── 旧接口兼容 ──

    pub fn run_batch_old(&mut self, count: u32, best_platform: Option<&str>) -> Vec<CycleResult> {
        self.run_batch(count, best_platform)
    }

    pub fn into_pipeline(mut self) -> EarnPipeline {
        let mut pipe = EarnPipeline::new(self.planner.config().clone(), &self.work_dir);
        if let Some(bridge) = self.aitoearn_bridge.take() {
            pipe = pipe.with_bridge(bridge);
        }
        pipe = pipe.with_publishers(std::mem::take(&mut self.publishers));
        pipe
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self.tracker.stats())
            .map_err(|e| format!("Serialize failed: {}", e))?;
        let p = std::path::Path::new(path);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| format!("Write failed: {}", e))?;
        std::fs::rename(&tmp, p).map_err(|e| format!("Rename failed: {}", e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let json = std::fs::read_to_string(path).map_err(|e| format!("Read failed: {}", e))?;
        let stats: EarnStats =
            serde_json::from_str(&json).map_err(|e| format!("Deserialize failed: {}", e))?;
        self.tracker = EarnTracker::from_stats(stats);
        Ok(())
    }

    pub fn earnings_stats(&self) -> &EarnStats {
        self.tracker.stats()
    }
    pub fn state(&self) -> &AgentState {
        &self.state
    }
    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }
    pub fn available_platforms(&self) -> Vec<&str> {
        self.publishers.list()
    }
    pub fn tracker(&self) -> &EarnTracker {
        &self.tracker
    }

    pub fn check_deps() -> Vec<String> {
        VideoPipeline::check_deps()
    }

    /// 发送 RL 奖励信号到 ReasoningBrain（外部验证回路）
    pub fn feed_reward_to_brain(&self, brain: &mut crate::neotrix::nt_mind::ReasoningBrain) {
        use crate::core::nt_core_traits::BrainProvider;
        let content_earnings = self.tracker.stats().total_earnings;
        let crypto_earnings = self.crypto_earnings_total();
        let reward_usd = content_earnings + crypto_earnings;

        let boost = (reward_usd * 0.01).min(0.5).max(0.0);
        if boost > 0.01 {
            let mut v = brain.capability_vector().clone();
            v.set_creativity((v.creativity() + boost * 0.1).min(1.0));
            v.set_analysis((v.analysis() + boost * 0.05).min(1.0));
            v.set_synthesis((v.synthesis() + boost * 0.08).min(1.0));
            brain.register_knowledge_source("nt_act_earn::real_earnings", v);
            let _ = brain.absorb_from_custom("nt_act_earn::real_earnings");
        }
    }

    // ── CryptoAgent 集成 (Arc<Mutex<>> 共享) ──

    pub fn with_nt_act_crypto(mut self, crypto: Arc<Mutex<CryptoAgent>>) -> Self {
        self.nt_act_crypto = Some(crypto);
        self
    }

    pub fn nt_act_crypto_arc(&self) -> Option<Arc<Mutex<CryptoAgent>>> {
        self.nt_act_crypto.clone()
    }

    pub fn init_nt_act_crypto(&mut self) -> Arc<Mutex<CryptoAgent>> {
        if self.nt_act_crypto.is_none() {
            self.nt_act_crypto = Some(Arc::new(Mutex::new(CryptoAgent::new())));
        }
        self.nt_act_crypto.clone().unwrap()
    }

    pub fn scan_crypto_opportunities(&mut self) -> Vec<crate::neotrix::nt_act_crypto::Opportunity> {
        match self.nt_act_crypto.clone() {
            Some(crypto_arc) => {
                let mut crypto = crypto_arc.lock().unwrap_or_else(|e| e.into_inner());
                let opps = crypto.scan_opportunities();
                for opp in &opps {
                    let _ = self.tracker.record_earning(EarningsRecord {
                        date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                        platform: format!("crypto:{}:{}", opp.chain, opp.opportunity_type.name()),
                        amount: opp.estimated_value_usd,
                        currency: "USD".to_string(),
                        content_title: opp.title.clone(),
                    });
                }
                opps
            }
            None => Vec::new(),
        }
    }

    pub fn check_crypto_balances(&self) -> std::collections::HashMap<String, f64> {
        match self.nt_act_crypto.as_ref() {
            Some(crypto_arc) => {
                let crypto = crypto_arc.lock().unwrap_or_else(|e| e.into_inner());
                crypto.check_all_balances()
            }
            None => std::collections::HashMap::new(),
        }
    }

    pub fn crypto_earnings_total(&self) -> f64 {
        self.nt_act_crypto
            .as_ref()
            .map(|c| {
                c.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .collector
                    .total_earnings()
            })
            .unwrap_or(0.0)
    }

    pub fn record_crypto_earning(
        &mut self,
        chain: ChainType,
        opp_type: OpportunityType,
        token: &str,
        amount: f64,
        value_usd: f64,
        label: &str,
    ) {
        if let Some(ref crypto_arc) = self.nt_act_crypto {
            let mut crypto = crypto_arc.lock().unwrap_or_else(|e| e.into_inner());
            crypto.collector.record_simple(
                chain.clone(),
                opp_type,
                token,
                amount,
                value_usd,
                label,
            );

            let _ = self.tracker.record_earning(EarningsRecord {
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                platform: format!("crypto:{}", chain),
                amount: value_usd,
                currency: "USD".to_string(),
                content_title: format!("crypto earning: {} {} {}", amount, token, label),
            });
        }
    }
}

impl Default for EarnAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn default_work_dir() -> String {
    crate::core::nt_core_util::home_dir()
        .join(".neotrix/earn")
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn echo_registry() -> PublisherRegistry {
        let mut reg = PublisherRegistry::new();
        reg.register(Box::new(super::super::publisher::CliPublisher::new(
            "twitter",
            "echo 'mock publish'",
        )));
        reg.register(Box::new(super::super::publisher::CliPublisher::new(
            "github",
            "echo 'mock publish'",
        )));
        reg
    }

    #[test]
    fn test_agent_creation() {
        let agent = EarnAgent::new();
        assert_eq!(*agent.state(), AgentState::Idle);
        assert_eq!(agent.total_cycles(), 0);
    }

    #[test]
    fn test_wealth_model_accessible() {
        let agent = EarnAgent::new();
        assert_eq!(agent.wealth_model.capital_layers.total_capital(), 1000.0);
    }

    #[test]
    fn test_financial_stack_accessible() {
        let agent = EarnAgent::new();
        assert_eq!(agent.financial_stack.layers.len(), 4);
    }

    #[test]
    fn test_knowledge_engine_accessible() {
        let agent = EarnAgent::new();
        assert_eq!(agent.knowledge_engine.sources.len(), 5);
    }

    #[test]
    fn test_deploy_leverage() {
        let mut agent = EarnAgent::new();
        let total = agent.deploy_leverage(5000.0, 0.05, CapitalSource::BankDebt);
        assert!(total > 1000.0);
    }

    #[test]
    fn test_securitize_assets() {
        let mut agent = EarnAgent::new();
        let pool = agent.securitize_assets(&[1000.0, 2000.0, 3000.0]);
        assert_eq!(pool, 6000.0);
    }

    #[test]
    fn test_open_arbitrage_position() {
        let mut agent = EarnAgent::new();
        agent.open_arbitrage_position(
            "BTC/USD",
            10000.0,
            10.0,
            crate::neotrix::nt_act_earn::financial_abstraction::PositionDirection::Long,
        );
        assert_eq!(
            agent.financial_stack.derivative_book.total_notional,
            10000.0
        );
    }

    #[test]
    fn test_scan_knowledge_opportunities() {
        let mut agent = EarnAgent::new();
        let count = agent.scan_knowledge_opportunities();
        assert!(count > 0);
    }

    #[test]
    fn test_simulate_day() {
        let mut agent = EarnAgent::new();
        agent.simulate_day(100.0, 0.1);
        assert!(agent.wealth_model.compounding_tracker.time_horizon_days >= 1);
    }

    #[test]
    fn test_strategic_report() {
        let mut agent = EarnAgent::new();
        agent.scan_knowledge_opportunities();
        let report = agent.strategic_report();
        assert!(!report.is_empty());
    }

    #[test]
    fn test_cycle_with_default_publishers() {
        let mut agent = EarnAgent::new();
        agent.set_publishers(echo_registry());
        let result = agent.run_cycle(Some("twitter"));
        assert_eq!(agent.total_cycles(), 1);
        assert_eq!(result.success_count, 1);
    }

    #[test]
    fn test_cycle_no_platform() {
        let mut agent = EarnAgent::new();
        let result = agent.run_cycle(None);
        assert!(result.publish_results.is_empty());
        assert_eq!(*agent.state(), AgentState::Idle);
    }

    #[test]
    fn test_batch_cycles() {
        let mut agent = EarnAgent::new();
        agent.set_publishers(echo_registry());
        let results = agent.run_batch(3, Some("twitter"));
        assert_eq!(results.len(), 3);
        assert_eq!(agent.total_cycles(), 3);
    }

    #[test]
    fn test_earnings_tracking() {
        let mut agent = EarnAgent::new();
        agent.set_publishers(echo_registry());
        agent.run_cycle(Some("twitter"));
        assert!(agent.earnings_stats().total_earnings > 0.0);
    }

    #[test]
    fn test_register_custom_publisher() {
        let mut agent = EarnAgent::new();
        agent.register_publisher("custom", "echo 'custom publish'");
        assert!(agent.available_platforms().contains(&"custom"));
    }

    #[test]
    fn test_save_load_state() {
        let path = "/tmp/test_earn_state_agent_v3.json";
        let _ = std::fs::remove_file(path);
        {
            let mut agent = EarnAgent::new();
            agent.set_publishers(echo_registry());
            agent.run_cycle(Some("twitter"));
            agent.save_state(path).expect("save should work");
        }
        let mut agent = EarnAgent::new();
        agent.load_state(path).expect("load should work");
        assert!(agent.earnings_stats().total_earnings > 0.0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_into_pipeline() {
        let agent = EarnAgent::new();
        let _pipeline = agent.into_pipeline();
    }
}
