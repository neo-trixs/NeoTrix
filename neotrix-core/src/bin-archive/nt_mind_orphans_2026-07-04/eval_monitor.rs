//! EvalMonitor — 外部 Agent 评估景观监控器
//!
//! 定期扫描已知 agent evaluation benchmark 的状态和关联度,
//! 为 SelfEvolver 提供外部评估标准的最新信息。
//! 自动标记高关联度新基准, 支持 SEAL pipeline 自检。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::clawbench::{ClassificationResult, TrajectoryClassifier};

/// 单一评估基准的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalBenchmark {
    pub name: String,
    pub url: String,
    pub description: String,
    /// 与 NeoTrix 的关联度 0.0–1.0
    pub relevance: f64,
    /// 上次检查时间戳
    #[serde(skip)]
    pub last_checked: Option<Instant>,
    /// 是否已吸收/集成
    pub absorbed: bool,
    /// 提议对接的 NeoTrix 模块
    pub target_module: Vec<String>,
}

/// 评估景观快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalLandscape {
    pub known_benchmarks: Vec<EvalBenchmark>,
    #[serde(skip)]
    pub last_update: Option<Instant>,
    pub high_relevance_count: usize,
}

impl EvalLandscape {
    pub fn new() -> Self {
        Self {
            known_benchmarks: Vec::new(),
            last_update: None,
            high_relevance_count: 0,
        }
    }

    pub fn find_relevant(&self, threshold: f64) -> Vec<&EvalBenchmark> {
        self.known_benchmarks
            .iter()
            .filter(|b| b.relevance >= threshold && !b.absorbed)
            .collect()
    }
}

/// AgentAtlas 六态控制 — 基于 AgentAtlas (arxiv 2605.20530) 的六种 agent 控制状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentAtlasState {
    /// Act: agent 自主行动
    Act,
    /// Ask: agent 向用户请求澄清
    Ask,
    /// Refuse: agent 拒绝不安全/不期望的请求
    Refuse,
    /// Stop: agent 停止执行 (达到限制或出错)
    Stop,
    /// Confirm: agent 在行动前请求确认
    Confirm,
    /// Recover: agent 从错误中恢复
    Recover,
}

impl AgentAtlasState {
    pub fn label(&self) -> &'static str {
        match self {
            AgentAtlasState::Act => "Act",
            AgentAtlasState::Ask => "Ask",
            AgentAtlasState::Refuse => "Refuse",
            AgentAtlasState::Stop => "Stop",
            AgentAtlasState::Confirm => "Confirm",
            AgentAtlasState::Recover => "Recover",
        }
    }
}

/// 评估监控器
pub struct EvalMonitor {
    landscape: EvalLandscape,
    last_scan: Instant,
    scan_interval: Duration,
    new_discoveries: Vec<String>,
    atlas_states: Vec<AgentAtlasState>,
    clawbench_trajectory_classify: Option<ClassificationResult>,
    atlas_action_log: Vec<(AgentAtlasState, String)>,
}

impl EvalMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            landscape: EvalLandscape::new(),
            last_scan: Instant::now(),
            scan_interval: Duration::from_secs(3600), // 1h
            new_discoveries: Vec::new(),
            atlas_states: Vec::new(),
            clawbench_trajectory_classify: None,
            atlas_action_log: Vec::new(),
        };
        monitor.init_known_benchmarks();
        monitor
    }

    pub fn with_scan_interval_secs(secs: u64) -> Self {
        let mut monitor = Self {
            landscape: EvalLandscape::new(),
            last_scan: Instant::now(),
            scan_interval: Duration::from_secs(secs),
            new_discoveries: Vec::new(),
            atlas_states: Vec::new(),
            clawbench_trajectory_classify: None,
            atlas_action_log: Vec::new(),
        };
        monitor.init_known_benchmarks();
        monitor
    }

    /// 初始化已知 benchmark (硬编码来自 2026-05-28 扫描)
    fn init_known_benchmarks(&mut self) {
        self.landscape.known_benchmarks = vec![
            EvalBenchmark {
                name: "ClawBench".into(),
                url: "https://github.com/openclaw/clawbench".into(),
                description: "动力系统诊断 + pass^k + 信噪比加权。轨迹级评估: 陷阱/极限环/扩散".into(),
                relevance: 0.90,
                last_checked: None,
                absorbed: false,
                target_module: vec!["pipeline.rs".into(), "BenchmarkSuite".into()],
            },
            EvalBenchmark {
                name: "Exgentic".into(),
                url: "https://github.com/Exgentic/exgentic".into(),
                description: "通用评估框架, 统一接入 7 基准(tau2/AppWorld/SWE-bench/BFCL)".into(),
                relevance: 0.80,
                last_checked: None,
                absorbed: false,
                target_module: vec!["BenchmarkSuite".into(), "nt_mind/core".into()],
            },
            EvalBenchmark {
                name: "STATE-Bench".into(),
                url: "https://github.com/microsoft/STATE-Bench".into(),
                description: "微软 450 企业多轮任务, pass^5 + UX Score + Cost Per Task".into(),
                relevance: 0.75,
                last_checked: None,
                absorbed: false,
                target_module: vec!["BenchmarkSuite".into()],
            },
            EvalBenchmark {
                name: "AlphaEval".into(),
                url: "https://github.com/GAIR-NLP/AlphaEval".into(),
                description: "94 生产任务, scaffold-aware 评估, 生产-学术鸿沟".into(),
                relevance: 0.85,
                last_checked: None,
                absorbed: false,
                target_module: vec!["AgentTeam".into(), "background_loop".into()],
            },
            EvalBenchmark {
                name: "AgentAtlas".into(),
                url: "https://arxiv.org/abs/2605.20530".into(),
                description: "六态控制 + 9 类轨迹失败分类. taxonomy-blind 降 14-40pp".into(),
                relevance: 0.88,
                last_checked: None,
                absorbed: false,
                target_module: vec!["pipeline.rs".into(), "metacognition".into()],
            },
            EvalBenchmark {
                name: "Agent-ValueBench".into(),
                url: "https://github.com/ValueByte-AI/Agent-ValueBench".into(),
                description: "28 价值系统, 4335 价值冲突任务. 价值观对齐评估".into(),
                relevance: 0.70,
                last_checked: None,
                absorbed: false,
                target_module: vec!["LawKeeper".into(), "behavioral_rule".into()],
            },
            EvalBenchmark {
                name: "Claw-Eval".into(),
                url: "https://github.com/claw-eval/claw-eval".into(),
                description: "300 人工验证任务, Completion/Safety/Robustness 三维".into(),
                relevance: 0.78,
                last_checked: None,
                absorbed: false,
                target_module: vec!["BenchmarkSuite".into()],
            },
            EvalBenchmark {
                name: "ClawMark".into(),
                url: "https://github.com/evolvent-ai/clawmark".into(),
                description: "多日工作基准 100 任务×13 专业, 零 LLM-as-judge".into(),
                relevance: 0.65,
                last_checked: None,
                absorbed: false,
                target_module: vec!["background_loop".into()],
            },
            EvalBenchmark {
                name: "Terrarium".into(),
                url: "https://github.com/evolvent-ai/Terrarium".into(),
                description: "living environment 多轮数据引擎, 可变环境".into(),
                relevance: 0.72,
                last_checked: None,
                absorbed: false,
                target_module: vec!["background_loop".into(), "exploration_pipeline".into()],
            },
            EvalBenchmark {
                name: "AgencyBench".into(),
                url: "https://github.com/GAIR-NLP/AgencyBench".into(),
                description: "ACL'26: 6 能力×32 场景×138 任务, 平均 1M tokens".into(),
                relevance: 0.68,
                last_checked: None,
                absorbed: false,
                target_module: vec!["BenchmarkSuite".into()],
            },
            EvalBenchmark {
                name: "auto-bench-audit".into(),
                url: "https://github.com/IsThatYou/auto-bench-audit".into(),
                description: "自动审计基准本身: 任务歧义/环境冲突/评估 bug".into(),
                relevance: 0.82,
                last_checked: None,
                absorbed: false,
                target_module: vec!["BenchmarkSuite".into(), "metacognition".into()],
            },
        ];
    }

    /// 执行景观扫描, 返回高关联度未吸收基准列表
    pub fn scan(&mut self) -> Vec<String> {
        self.last_scan = Instant::now();
        let mut discoveries = Vec::new();

        for bm in &self.landscape.known_benchmarks {
            if bm.relevance >= 0.80 && !bm.absorbed {
                discoveries.push(format!(
                    "🔬 {} (rel={:.2}): {} → {:?}",
                    bm.name, bm.relevance, bm.description, bm.target_module
                ));
            }
        }

        self.new_discoveries = discoveries.clone();
        self.landscape.last_update = Some(Instant::now());

        let relevant = self.landscape.find_relevant(0.80);
        self.landscape.high_relevance_count = relevant.len();

        discoveries
    }

    /// 标记某 benchmark 为已吸收
    pub fn mark_absorbed(&mut self, name: &str) {
        if let Some(bm) = self.landscape.known_benchmarks.iter_mut().find(|b| b.name == name) {
            bm.absorbed = true;
            bm.last_checked = Some(Instant::now());
        }
    }

    /// 获取当前景观摘要
    pub fn landscape_summary(&self) -> String {
        let total = self.landscape.known_benchmarks.len();
        let absorbed = self.landscape.known_benchmarks.iter().filter(|b| b.absorbed).count();
        let high_rel = self.landscape.find_relevant(0.80);
        format!(
            "已知 {} 基准, {} 已吸收, {} 高关联未吸收 | 上次扫描: {:?}",
            total,
            absorbed,
            high_rel.len(),
            self.landscape.last_update.map(|_| "ok").unwrap_or("never")
        )
    }

    /// 是否到扫描时间
    pub fn should_scan(&self) -> bool {
        self.last_scan.elapsed() >= self.scan_interval
    }

    /// 获取新发现列表
    pub fn discoveries(&self) -> &[String] {
        &self.new_discoveries
    }

    pub fn landscape(&self) -> &EvalLandscape {
        &self.landscape
    }

    pub fn landscape_mut(&mut self) -> &mut EvalLandscape {
        &mut self.landscape
    }

    // ── EV-01: ClawBench 轨迹诊断 ──

    /// 获取当前 ClawBench 轨迹分类结果
    pub fn clawbench_result(&self) -> Option<&ClassificationResult> {
        self.clawbench_trajectory_classify.as_ref()
    }

    /// 分析存储的轨迹数据, 使用 ClawBench 分类器分类
    pub fn pipeline_analyze_trajectory(&mut self, rewards: &[f64], actions: &[String]) -> ClassificationResult {
        let classifier = TrajectoryClassifier::new();
        let result = classifier.classify(rewards, actions);
        self.clawbench_trajectory_classify = Some(result.clone());
        result
    }

    // ── EV-03: AgentAtlas 六态控制 ──

    /// 记录一个 AgentAtlas 状态事件
    pub fn insert_atlas_state(&mut self, state: AgentAtlasState) {
        self.atlas_states.push(state);
        self.atlas_action_log.push((state, chrono::Utc::now().to_rfc3339()));
    }

    /// 记录带上下文的状态事件
    pub fn insert_atlas_state_with_context(&mut self, state: AgentAtlasState, context: String) {
        self.atlas_states.push(state);
        self.atlas_action_log.push((state, context));
    }

    /// 获取 AgentAtlas 状态分布
    pub fn atlas_state_distribution(&self) -> HashMap<AgentAtlasState, usize> {
        let mut dist: HashMap<AgentAtlasState, usize> = HashMap::new();
        for state in &self.atlas_states {
            *dist.entry(*state).or_insert(0) += 1;
        }
        dist
    }

    /// 清除所有 AgentAtlas 状态记录
    pub fn clear_atlas_states(&mut self) {
        self.atlas_states.clear();
        self.atlas_action_log.clear();
    }

    /// 基于 AgentAtlas 状态模式, 建议添加的 SEAL pipeline stage
    ///
    /// 规则:
    /// - 高频 Refuse/Stop → 建议添加 SafetyGuard stage
    /// - 高频 Ask/Confirm → 建议添加 HumanInLoop stage
    /// - 高频 Recover → 建议添加 ErrorRecovery stage
    /// - 均衡分布 → 无需额外 stage
    pub fn atlas_suggest_pipeline_stage(&self) -> Option<String> {
        let dist = self.atlas_state_distribution();
        let total: usize = dist.values().sum();
        if total == 0 {
            return None;
        }

        let refuse_stop = *dist.get(&AgentAtlasState::Refuse).unwrap_or(&0)
            + *dist.get(&AgentAtlasState::Stop).unwrap_or(&0);
        let ask_confirm = *dist.get(&AgentAtlasState::Ask).unwrap_or(&0)
            + *dist.get(&AgentAtlasState::Confirm).unwrap_or(&0);
        let recover = *dist.get(&AgentAtlasState::Recover).unwrap_or(&0);

        let ratio = |count: usize| count as f64 / total as f64;

        if ratio(refuse_stop) > 0.3 {
            Some("SafetyGuard".into())
        } else if ratio(ask_confirm) > 0.3 {
            Some("HumanInLoop".into())
        } else if ratio(recover) > 0.25 {
            Some("ErrorRecovery".into())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_monitor_new() {
        let monitor = EvalMonitor::new();
        assert_eq!(monitor.landscape.known_benchmarks.len(), 11);
        assert!(monitor.landscape.known_benchmarks[0].relevance > 0.0);
        assert!(monitor.clawbench_trajectory_classify.is_none());
        assert!(monitor.atlas_states.is_empty());
    }

    #[test]
    fn test_scan_returns_high_relevance() {
        let mut monitor = EvalMonitor::new();
        let discoveries = monitor.scan();
        assert!(!discoveries.is_empty());
        // ClawBench (0.90), Exgentic (0.80), AlphaEval (0.85), AgentAtlas (0.88), auto-bench-audit (0.80)
        assert!(discoveries.len() >= 4);
    }

    #[test]
    fn test_mark_absorbed() {
        let mut monitor = EvalMonitor::new();
        monitor.mark_absorbed("ClawBench");
        let relevant = monitor.landscape.find_relevant(0.80);
        assert!(!relevant.iter().any(|b| b.name == "ClawBench"));
    }

    #[test]
    fn test_landscape_summary() {
        let monitor = EvalMonitor::new();
        let summary = monitor.landscape_summary();
        assert!(summary.contains("11"));
    }

    #[test]
    fn test_should_scan() {
        let monitor = EvalMonitor::new();
        assert!(!monitor.should_scan()); // just created, shouldn't need scan
    }

    #[test]
    fn test_find_relevant_threshold() {
        let landscape = EvalLandscape::new();
        // empty landscape should return empty
        assert!(landscape.find_relevant(0.80).is_empty());
    }

    // ── EV-03: AgentAtlas 六态控制 tests ──

    #[test]
    fn test_agent_atlas_state_labels() {
        assert_eq!(AgentAtlasState::Act.label(), "Act");
        assert_eq!(AgentAtlasState::Ask.label(), "Ask");
        assert_eq!(AgentAtlasState::Refuse.label(), "Refuse");
        assert_eq!(AgentAtlasState::Stop.label(), "Stop");
        assert_eq!(AgentAtlasState::Confirm.label(), "Confirm");
        assert_eq!(AgentAtlasState::Recover.label(), "Recover");
    }

    #[test]
    fn test_insert_atlas_state() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Ask);
        assert_eq!(monitor.atlas_states.len(), 2);
    }

    #[test]
    fn test_atlas_state_distribution() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Ask);
        monitor.insert_atlas_state(AgentAtlasState::Refuse);
        let dist = monitor.atlas_state_distribution();
        assert_eq!(*dist.get(&AgentAtlasState::Act).expect("value should be ok in test"), 2);
        assert_eq!(*dist.get(&AgentAtlasState::Ask).expect("value should be ok in test"), 1);
        assert_eq!(*dist.get(&AgentAtlasState::Refuse).expect("value should be ok in test"), 1);
        assert_eq!(dist.len(), 3);
    }

    #[test]
    fn test_atlas_suggest_safety_guard() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state(AgentAtlasState::Refuse);
        monitor.insert_atlas_state(AgentAtlasState::Refuse);
        monitor.insert_atlas_state(AgentAtlasState::Stop);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        let suggestion = monitor.atlas_suggest_pipeline_stage();
        assert_eq!(suggestion, Some("SafetyGuard".into()));
    }

    #[test]
    fn test_atlas_suggest_human_in_loop() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state(AgentAtlasState::Ask);
        monitor.insert_atlas_state(AgentAtlasState::Ask);
        monitor.insert_atlas_state(AgentAtlasState::Confirm);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        let suggestion = monitor.atlas_suggest_pipeline_stage();
        assert_eq!(suggestion, Some("HumanInLoop".into()));
    }

    #[test]
    fn test_atlas_suggest_error_recovery() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state(AgentAtlasState::Recover);
        monitor.insert_atlas_state(AgentAtlasState::Recover);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        let suggestion = monitor.atlas_suggest_pipeline_stage();
        assert_eq!(suggestion, Some("ErrorRecovery".into()));
    }

    #[test]
    fn test_atlas_suggest_none_when_balanced() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.insert_atlas_state(AgentAtlasState::Act);
        let suggestion = monitor.atlas_suggest_pipeline_stage();
        assert_eq!(suggestion, None);
    }

    #[test]
    fn test_atlas_suggest_empty_states() {
        let monitor = EvalMonitor::new();
        assert_eq!(monitor.atlas_suggest_pipeline_stage(), None);
    }

    #[test]
    fn test_insert_atlas_state_with_context() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state_with_context(AgentAtlasState::Refuse, "unsafe command".into());
        assert_eq!(monitor.atlas_action_log.len(), 1);
        assert_eq!(monitor.atlas_action_log[0].1, "unsafe command");
    }

    #[test]
    fn test_clear_atlas_states() {
        let mut monitor = EvalMonitor::new();
        monitor.insert_atlas_state(AgentAtlasState::Act);
        monitor.clear_atlas_states();
        assert!(monitor.atlas_states.is_empty());
        assert!(monitor.atlas_action_log.is_empty());
    }

    // ── EV-01: ClawBench pipeline integration tests ──

    #[test]
    fn test_pipeline_analyze_trajectory() {
        let mut monitor = EvalMonitor::new();
        let rewards = vec![0.1, 0.3, 0.5, 0.6, 0.7];
        let actions = vec!["read".into(), "read".into(), "write".into(), "write".into(), "done".into()];
        let result = monitor.pipeline_analyze_trajectory(&rewards, &actions);
        assert!(!result.dynamics.label().is_empty());
        assert!(monitor.clawbench_result().is_some());
    }

    #[test]
    fn test_pipeline_analyze_trajectory_stores_result() {
        let mut monitor = EvalMonitor::new();
        let rewards = vec![0.1, 0.8, 0.2, 0.9, 0.15];
        let actions = vec!["a".into(), "b".into(), "a".into(), "b".into(), "a".into()];
        let _ = monitor.pipeline_analyze_trajectory(&rewards, &actions);
        let stored = monitor.clawbench_result().expect("value should be ok in test");
        assert_eq!(stored.trajectory_length, 5);
    }
}
