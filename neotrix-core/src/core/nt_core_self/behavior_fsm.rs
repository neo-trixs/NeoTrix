use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// FSM 行为拓扑 + 失败预测
///
/// 参考: arXiv:2608.23670 "Emerging Digital Automata from Agent Traces"
/// 核心思想: 将 agent 执行轨迹压缩为紧凑 FSM (7-43 状态),
/// 用于行为监控、下一步预测和失败预测。
///
/// 架构:
/// - TraceCollector: 收集 agent 执行轨迹
/// - FsmBuilder: 轨迹 → FSM 转换
/// - NextStepPredictor: FSM 状态上下文预测下一步
/// - FailurePredictor: per-state 特征 → 失败概率

/// FSM 状态
#[derive(Debug, Clone)]
pub struct FsmState {
    pub id: String,
    pub label: String,
    pub features: HashMap<String, f64>,
}

/// FSM 转换
#[derive(Debug, Clone)]
pub struct FsmTransition {
    pub from: String,
    pub to: String,
    pub action: String,
    pub count: u32,
    pub probability: f64,
}

/// FSM 模型
#[derive(Debug, Clone)]
pub struct FsmModel {
    pub states: HashMap<String, FsmState>,
    pub transitions: Vec<FsmTransition>,
    pub initial_state: String,
    pub accepting_states: HashSet<String>,
}

impl FsmModel {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            transitions: Vec::new(),
            initial_state: String::new(),
            accepting_states: HashSet::new(),
        }
    }

    /// 添加状态
    pub fn add_state(&mut self, state: FsmState) {
        if self.initial_state.is_empty() {
            self.initial_state = state.id.clone();
        }
        self.states.insert(state.id.clone(), state);
    }

    /// 添加转换
    pub fn add_transition(&mut self, transition: FsmTransition) {
        self.transitions.push(transition);
    }

    /// 预测给定状态的失败概率
    pub fn predict_failure_probability(&self, state_id: &str) -> f64 {
        let mut predictor = FailurePredictor::new();
        // 从当前模型的状态转换学习失败率
        let traces: Vec<TraceEntry> = self.transitions.iter().map(|t| TraceEntry {
            timestamp: std::time::Instant::now(),
            action: t.action.clone(),
            state_before: Some(t.from.clone()),
            state_after: Some(t.to.clone()),
            result: TraceResult::Success,
            features: std::collections::HashMap::new(),
        }).collect();
        predictor.learn(&traces);
        predictor.predict_failure_probability(state_id)
    }

    /// 获取从给定状态出发的所有转换
    pub fn transitions_from(&self, state_id: &str) -> Vec<&FsmTransition> {
        self.transitions.iter()
            .filter(|t| t.from == state_id)
            .collect()
    }

    /// 计算转换概率
    pub fn compute_transition_probabilities(&mut self) {
        let mut from_counts: HashMap<String, u32> = HashMap::new();
        for t in &self.transitions {
            *from_counts.entry(t.from.clone()).or_default() += t.count;
        }

        for t in &mut self.transitions {
            if let Some(&total) = from_counts.get(&t.from) {
                t.probability = t.count as f64 / total as f64;
            }
        }
    }
}

/// Agent 执行轨迹点
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub timestamp: Instant,
    pub action: String,
    pub state_before: Option<String>,
    pub state_after: Option<String>,
    pub result: TraceResult,
    pub features: HashMap<String, f64>,
}

impl TraceEntry {
    /// 序列化为 JSON (手动实现, 因为 Instant 不支持 serde)
    pub fn to_json(&self) -> String {
        let features_json: Vec<String> = self.features.iter()
            .map(|(k, v)| format!("\"{}\":{}", k, v))
            .collect();
        format!(
            r#"{{"action":"{}","state_before":{},"state_after":{},"result":"{}","features":{{{}}}}}"#,
            self.action,
            self.state_before.as_ref().map(|s| format!("\"{}\"", s)).unwrap_or_else(|| "null".to_string()),
            self.state_after.as_ref().map(|s| format!("\"{}\"", s)).unwrap_or_else(|| "null".to_string()),
            match self.result {
                TraceResult::Success => "Success",
                TraceResult::Failure(_) => "Failure",
                TraceResult::Timeout => "Timeout",
            },
            features_json.join(",")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceResult {
    Success,
    Failure(String),
    Timeout,
}

/// 轨迹收集器
pub struct TraceCollector {
    entries: Vec<TraceEntry>,
    max_entries: usize,
}

impl TraceCollector {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// 记录轨迹点
    pub fn record(&mut self, entry: TraceEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// 获取所有轨迹
    pub fn get_entries(&self) -> &[TraceEntry] {
        &self.entries
    }

    /// 清空轨迹
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// FSM 构建器 — 从轨迹构建 FSM
pub struct FsmBuilder {
    min_transitions: u32,
    similarity_threshold: f64,
}

impl FsmBuilder {
    pub fn new(min_transitions: u32, similarity_threshold: f64) -> Self {
        Self {
            min_transitions,
            similarity_threshold,
        }
    }

    /// 从轨迹构建 FSM
    pub fn build(&self, traces: &[TraceEntry]) -> FsmModel {
        let mut fsm = FsmModel::new();

        // 1. 聚类相似状态
        let state_clusters = self.cluster_states(traces);

        // 2. 为每个聚类创建 FSM 状态
        for (cluster_id, entries) in &state_clusters {
            let features = self.aggregate_features(entries);
            let state = FsmState {
                id: cluster_id.clone(),
                label: format!("State_{}", cluster_id),
                features,
            };
            fsm.add_state(state);
        }

        // 3. 统计转换
        let mut transition_counts: HashMap<(String, String, String), u32> = HashMap::new();
        for window in traces.windows(2) {
            if let (Some(before), Some(after)) = (&window[0].state_after, &window[1].state_before) {
                let key = (before.clone(), after.clone(), window[1].action.clone());
                *transition_counts.entry(key).or_default() += 1;
            }
        }

        // 4. 添加满足最小频率的转换
        for ((from, to, action), count) in transition_counts {
            if count >= self.min_transitions {
                fsm.add_transition(FsmTransition {
                    from,
                    to,
                    action,
                    count,
                    probability: 0.0,
                });
            }
        }

        // 5. 计算转换概率
        fsm.compute_transition_probabilities();

        fsm
    }

    /// 聚类相似状态
    fn cluster_states<'a>(&self, traces: &'a [TraceEntry]) -> HashMap<String, Vec<&'a TraceEntry>> {
        let mut clusters: HashMap<String, Vec<&TraceEntry>> = HashMap::new();
        let mut cluster_centroids: Vec<HashMap<String, f64>> = Vec::new();

        for entry in traces {
            let features = &entry.features;
            let mut assigned = false;

            for (i, centroid) in cluster_centroids.iter().enumerate() {
                if self.similarity(features, centroid) >= self.similarity_threshold {
                    let cluster_id = format!("s_{}", i);
                    clusters.entry(cluster_id).or_default().push(entry);
                    assigned = true;
                    break;
                }
            }

            if !assigned {
                let cluster_id = format!("s_{}", cluster_centroids.len());
                cluster_centroids.push(features.clone());
                clusters.entry(cluster_id).or_default().push(entry);
            }
        }

        clusters
    }

    /// 计算特征相似度 (余弦相似度)
    fn similarity(&self, a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
        let all_keys: HashSet<_> = a.keys().chain(b.keys()).collect();
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for key in all_keys {
            let va = a.get(key).unwrap_or(&0.0);
            let vb = b.get(key).unwrap_or(&0.0);
            dot_product += va * vb;
            norm_a += va * va;
            norm_b += vb * vb;
        }

        let denominator = (norm_a * norm_b).sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            dot_product / denominator
        }
    }

    /// 聚合特征 (取平均)
    fn aggregate_features(&self, entries: &[&TraceEntry]) -> HashMap<String, f64> {
        let mut sums: HashMap<String, f64> = HashMap::new();
        let count = entries.len() as f64;

        for entry in entries {
            for (k, v) in &entry.features {
                *sums.entry(k.clone()).or_default() += v;
            }
        }

        for v in sums.values_mut() {
            *v /= count;
        }

        sums
    }
}

/// 下一步预测器 — 基于 FSM 状态上下文
pub struct NextStepPredictor {
    fsm: FsmModel,
    history: Vec<String>, // 状态 ID 历史
}

impl NextStepPredictor {
    pub fn new(fsm: FsmModel) -> Self {
        Self {
            fsm,
            history: Vec::new(),
        }
    }

    /// 记录当前状态
    pub fn observe(&mut self, state_id: String) {
        self.history.push(state_id);
    }

    /// 预测下一步最可能的转换
    pub fn predict(&self) -> Option<&FsmTransition> {
        let current_state = self.history.last()?;
        self.fsm
            .transitions_from(current_state)
            .into_iter()
            .max_by(|a, b| a.probability.partial_cmp(&b.probability).unwrap())
    }

    /// 预测 Top-K 下一步
    pub fn predict_top_k(&self, k: usize) -> Vec<&FsmTransition> {
        let current_state = match self.history.last() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut transitions: Vec<_> = self.fsm.transitions_from(current_state);
        transitions.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());
        transitions.into_iter().take(k).collect()
    }
}

/// 失败预测器 — 基于 per-state 特征
pub struct FailurePredictor {
    /// 每个状态的失败率
    state_failure_rates: HashMap<String, f64>,
    /// 全局失败率
    global_failure_rate: f64,
}

impl FailurePredictor {
    pub fn new() -> Self {
        Self {
            state_failure_rates: HashMap::new(),
            global_failure_rate: 0.0,
        }
    }

    /// 从轨迹学习失败率
    pub fn learn(&mut self, traces: &[TraceEntry]) {
        let mut state_totals: HashMap<String, u32> = HashMap::new();
        let mut state_failures: HashMap<String, u32> = HashMap::new();
        let mut total = 0u32;
        let mut total_failures = 0u32;

        for entry in traces {
            if let Some(ref state) = entry.state_before {
                *state_totals.entry(state.clone()).or_default() += 1;
                total += 1;

                if matches!(entry.result, TraceResult::Failure(_)) {
                    *state_failures.entry(state.clone()).or_default() += 1;
                    total_failures += 1;
                }
            }
        }

        for (state, &failures) in &state_failures {
            if let Some(&totals) = state_totals.get(state) {
                self.state_failure_rates
                    .insert(state.clone(), failures as f64 / totals as f64);
            }
        }

        if total > 0 {
            self.global_failure_rate = total_failures as f64 / total as f64;
        }
    }

    /// 预测给定状态的失败概率
    pub fn predict_failure_probability(&self, state_id: &str) -> f64 {
        self.state_failure_rates
            .get(state_id)
            .copied()
            .unwrap_or(self.global_failure_rate)
    }

    /// 识别高风险状态 (失败率 > 阈值)
    pub fn high_risk_states(&self, threshold: f64) -> Vec<(&str, f64)> {
        self.state_failure_rates
            .iter()
            .filter(|(_, &rate)| rate > threshold)
            .map(|(s, &rate)| (s.as_str(), rate))
            .collect()
    }

    /// 早期停止: 部分轨迹 → 排序失败运行
    pub fn should_early_stop(&self, current_state: &str, confidence_threshold: f64) -> bool {
        self.predict_failure_probability(current_state) > confidence_threshold
    }
}

/// FSM 分析结果
#[derive(Debug, Clone)]
pub struct FsmAnalysis {
    pub state_count: usize,
    pub transition_count: usize,
    pub initial_state: String,
    pub accepting_states: usize,
    pub avg_transition_probability: f64,
    pub high_risk_states: Vec<(String, f64)>,
}

/// 综合 FSM 分析器
pub struct BehaviorFsmAnalyzer {
    collector: TraceCollector,
    builder: FsmBuilder,
    predictor: Option<NextStepPredictor>,
    failure_predictor: FailurePredictor,
}

impl BehaviorFsmAnalyzer {
    pub fn new(max_entries: usize, min_transitions: u32, similarity_threshold: f64) -> Self {
        Self {
            collector: TraceCollector::new(max_entries),
            builder: FsmBuilder::new(min_transitions, similarity_threshold),
            predictor: None,
            failure_predictor: FailurePredictor::new(),
        }
    }

    /// 记录轨迹点
    pub fn record(&mut self, entry: TraceEntry) {
        self.collector.record(entry);
    }

    /// 从当前轨迹构建 FSM
    pub fn build_fsm(&mut self) -> FsmModel {
        let traces = self.collector.get_entries().to_vec();
        let fsm = self.builder.build(&traces);
        self.predictor = Some(NextStepPredictor::new(fsm.clone()));
        self.failure_predictor.learn(&traces);
        fsm
    }

    /// 预测下一步
    pub fn predict_next(&self) -> Option<&FsmTransition> {
        self.predictor.as_ref()?.predict()
    }

    /// 预测失败概率
    pub fn predict_failure(&self, state_id: &str) -> f64 {
        self.failure_predictor.predict_failure_probability(state_id)
    }

    /// 分析 FSM
    pub fn analyze(&self, fsm: &FsmModel) -> FsmAnalysis {
        let avg_prob = if fsm.transitions.is_empty() {
            0.0
        } else {
            fsm.transitions.iter().map(|t| t.probability).sum::<f64>() / fsm.transitions.len() as f64
        };

        let high_risk = self
            .failure_predictor
            .high_risk_states(0.5)
            .into_iter()
            .map(|(s, r)| (s.to_string(), r))
            .collect();

        FsmAnalysis {
            state_count: fsm.states.len(),
            transition_count: fsm.transitions.len(),
            initial_state: fsm.initial_state.clone(),
            accepting_states: fsm.accepting_states.len(),
            avg_transition_probability: avg_prob,
            high_risk_states: high_risk,
        }
    }

    /// 清空收集器
    pub fn reset(&mut self) {
        self.collector.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trace(action: &str, state_before: Option<&str>, state_after: Option<&str>, result: TraceResult) -> TraceEntry {
        TraceEntry {
            timestamp: Instant::now(),
            action: action.to_string(),
            state_before: state_before.map(|s| s.to_string()),
            state_after: state_after.map(|s| s.to_string()),
            result,
            features: HashMap::new(),
        }
    }

    #[test]
    fn test_fsm_builder_basic() {
        let builder = FsmBuilder::new(1, 0.8);
        let traces = vec![
            make_trace("init", None, Some("s0"), TraceResult::Success),
            make_trace("fetch", Some("s0"), Some("s1"), TraceResult::Success),
            make_trace("process", Some("s1"), Some("s2"), TraceResult::Success),
            make_trace("init", None, Some("s0"), TraceResult::Success),
            make_trace("fetch", Some("s0"), Some("s1"), TraceResult::Success),
            make_trace("process", Some("s1"), Some("s2"), TraceResult::Success),
        ];

        let fsm = builder.build(&traces);
        assert!(fsm.states.len() >= 3);
        assert!(!fsm.transitions.is_empty());
    }

    #[test]
    fn test_failure_predictor() {
        let mut predictor = FailurePredictor::new();
        let traces = vec![
            make_trace("a", Some("s0"), Some("s1"), TraceResult::Success),
            make_trace("b", Some("s1"), Some("s2"), TraceResult::Failure("err".to_string())),
            make_trace("c", Some("s0"), Some("s1"), TraceResult::Success),
            make_trace("d", Some("s1"), Some("s2"), TraceResult::Failure("err".to_string())),
        ];

        predictor.learn(&traces);
        let prob = predictor.predict_failure_probability("s1");
        assert!(prob > 0.0);
    }

    #[test]
    fn test_next_step_predictor() {
        let mut fsm = FsmModel::new();
        fsm.add_state(FsmState {
            id: "s0".to_string(),
            label: "Start".to_string(),
            features: HashMap::new(),
        });
        fsm.add_state(FsmState {
            id: "s1".to_string(),
            label: "End".to_string(),
            features: HashMap::new(),
        });
        fsm.add_transition(FsmTransition {
            from: "s0".to_string(),
            to: "s1".to_string(),
            action: "go".to_string(),
            count: 10,
            probability: 1.0,
        });

        let mut predictor = NextStepPredictor::new(fsm);
        predictor.observe("s0".to_string());

        let prediction = predictor.predict();
        assert!(prediction.is_some());
        assert_eq!(prediction.unwrap().to, "s1");
    }

    #[test]
    fn test_behavior_fsm_analyzer() {
        let mut analyzer = BehaviorFsmAnalyzer::new(100, 1, 0.8);

        for i in 0..20 {
            analyzer.record(make_trace(
                &format!("action_{}", i % 5),
                Some(&format!("s{}", i % 3)),
                Some(&format!("s{}", (i + 1) % 3)),
                TraceResult::Success,
            ));
        }

        let fsm = analyzer.build_fsm();
        assert!(!fsm.states.is_empty());

        let analysis = analyzer.analyze(&fsm);
        assert!(analysis.state_count > 0);
    }
}
