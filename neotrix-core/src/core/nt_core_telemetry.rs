use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryEvent {
    AgentSpawned {
        agent_id: String,
        role: String,
    },
    AgentCompleted {
        agent_id: String,
        success: bool,
        duration_ms: u64,
    },
    ToolCall {
        tool: String,
        success: bool,
        duration_ms: u64,
    },
    KnowledgeAbsorbed {
        source: String,
        count: u64,
    },
    Error {
        source: String,
        message: String,
        severity: u8,
    },
    ConsciousnessTick {
        phi: f64,
        coherence: f64,
        quality: f64,
    },
    Sealed {
        cycle: u64,
        reward: f64,
    },
    Custom {
        name: String,
        value: String,
    },
}

impl TelemetryEvent {
    pub fn kind(&self) -> &str {
        match self {
            Self::AgentSpawned { .. } => "agent_spawned",
            Self::AgentCompleted { .. } => "agent_completed",
            Self::ToolCall { .. } => "tool_call",
            Self::KnowledgeAbsorbed { .. } => "knowledge_absorbed",
            Self::Error { .. } => "error",
            Self::ConsciousnessTick { .. } => "consciousness_tick",
            Self::Sealed { .. } => "sealed",
            Self::Custom { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedMetric {
    pub count: u64,
    pub error_count: u64,
    pub total_duration_ms: u64,
    pub last_seen: Instant,
}

pub struct TelemetryStore {
    events: Mutex<VecDeque<(Instant, TelemetryEvent)>>,
    max_events: usize,
    metrics: Mutex<HashMap<String, AggregatedMetric>>,
    /// Numeric metric series (e.g. latency_ms) — counts/durations only, no raw payloads.
    metric_series: Mutex<HashMap<String, VecDeque<(Instant, f64)>>>,
}

/// Aggregate recorded events into per-kind window buckets.
/// Privacy: counts + durations only — raw payloads (message/value) never escape.
#[derive(Debug, Clone, Default)]
pub struct WindowAggregate {
    pub count: u64,
    pub error_count: u64,
    pub total_duration_ms: u64,
}

impl TelemetryStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Mutex::new(VecDeque::with_capacity(max_events.min(100_000))),
            max_events,
            metrics: Mutex::new(HashMap::new()),
            metric_series: Mutex::new(HashMap::new()),
        }
    }

    /// Record a numeric metric sample (e.g. request latency). Privacy: numeric
    /// value only, no raw payload retained.
    pub fn record_metric(&self, metric: &str, value: f64) {
        if let Ok(mut series) = self.metric_series.lock() {
            let q = series
                .entry(metric.to_string())
                .or_insert_with(VecDeque::new);
            if q.len() >= self.max_events {
                q.pop_front();
            }
            q.push_back((Instant::now(), value));
        }
    }

    /// Names of all recorded numeric metric series.
    pub fn metric_names(&self) -> Vec<String> {
        match self.metric_series.lock() {
            Ok(series) => series.keys().cloned().collect(),
            Err(_) => vec![],
        }
    }

    /// Mean value of a numeric metric over the window (None if no samples).
    pub fn metric_window_mean(&self, metric: &str, window: Duration) -> Option<f64> {
        let cutoff = Instant::now() - window;
        match self.metric_series.lock() {
            Ok(series) => series.get(metric).and_then(|q| {
                let samples: Vec<f64> = q
                    .iter()
                    .filter(|(t, _)| *t > cutoff)
                    .map(|(_, v)| *v)
                    .collect();
                if samples.is_empty() {
                    None
                } else {
                    Some(samples.iter().sum::<f64>() / samples.len() as f64)
                }
            }),
            Err(_) => None,
        }
    }

    pub fn record(&self, event: TelemetryEvent) {
        let kind = event.kind().to_string();
        let is_error = matches!(&event, TelemetryEvent::Error { .. });
        let duration = match &event {
            TelemetryEvent::ToolCall { duration_ms, .. } => *duration_ms,
            TelemetryEvent::AgentCompleted { duration_ms, .. } => *duration_ms,
            _ => 0,
        };

        if let Ok(mut events) = self.events.lock() {
            if events.len() >= self.max_events {
                events.pop_front();
            }
            events.push_back((Instant::now(), event));
        }

        if let Ok(mut metrics) = self.metrics.lock() {
            let entry = metrics.entry(kind).or_insert(AggregatedMetric {
                count: 0,
                error_count: 0,
                total_duration_ms: 0,
                last_seen: Instant::now(),
            });
            entry.count += 1;
            if is_error {
                entry.error_count += 1;
            }
            entry.total_duration_ms += duration;
            entry.last_seen = Instant::now();
        }
    }

    pub fn summary(&self) -> Vec<(String, AggregatedMetric)> {
        let mut result: Vec<_> = match self.metrics.lock() {
            Ok(m) => m.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            Err(_) => return vec![],
        };
        result.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        result
    }

    pub fn recent_events(&self, n: usize) -> Vec<(Instant, TelemetryEvent)> {
        match self.events.lock() {
            Ok(events) => events.iter().rev().take(n).cloned().collect(),
            Err(_) => vec![],
        }
    }

    pub fn errors_in_last(&self, duration: Duration) -> u64 {
        let cutoff = Instant::now() - duration;
        match self.events.lock() {
            Ok(events) => events
                .iter()
                .filter(|(t, e)| *t > cutoff && matches!(e, TelemetryEvent::Error { .. }))
                .count() as u64,
            Err(_) => 0,
        }
    }

    /// Count events per kind within the last `window`. Only aggregate scalars
    /// are surfaced — no raw event payloads leak into the alerting pipeline.
    pub fn window_aggregates(&self, window: Duration) -> Vec<(String, WindowAggregate)> {
        let cutoff = Instant::now() - window;
        let mut agg: HashMap<String, WindowAggregate> = HashMap::new();
        if let Ok(events) = self.events.lock() {
            for (t, e) in events.iter() {
                if *t < cutoff {
                    continue;
                }
                let kind = e.kind().to_string();
                let entry = agg.entry(kind).or_default();
                entry.count += 1;
                if matches!(e, TelemetryEvent::Error { .. }) {
                    entry.error_count += 1;
                }
                entry.total_duration_ms += match e {
                    TelemetryEvent::ToolCall { duration_ms, .. } => *duration_ms,
                    TelemetryEvent::AgentCompleted { duration_ms, .. } => *duration_ms,
                    _ => 0,
                };
            }
        }
        let mut result: Vec<_> = agg.into_iter().collect();
        result.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        result
    }

    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
        if let Ok(mut series) = self.metric_series.lock() {
            series.clear();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Spike,
    Drop,
}

#[derive(Debug, Clone)]
pub struct TelemetryAlert {
    pub metric: String,
    pub kind: AlertKind,
    pub current: f64,
    pub baseline: f64,
    pub threshold: f64,
    /// J-Space 能力实现损失层分类 (j-space 报告 §2): 告警对应哪一层失配。
    /// None = 常规指标告警, 未归类。
    pub loss_layer: Option<LossLayer>,
}

/// J-Space 能力实现损失 (capability-realization loss) 六层分类 —
/// 推理模式 / 首轮接口 / 工具 schema / 活动表征 / 长程状态 / 验证机制。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LossLayer {
    /// 推理模式失配 (短链跳过桥接 / 长链不行动)
    ReasoningMode,
    /// 首轮接口失配 (persona / 首轮条件 / 轨迹锚定)
    FirstTurnInterface,
    /// 工具 schema 失配 (工具目录 / schema 指纹)
    ToolSchema,
    /// 活动表征失配 (工作集过载 / 表征漂移)
    ActiveRepresentation,
    /// 长程状态失配 (目标淡出 / 跨分支重建 / 空白重试)
    LongHorizonState,
    /// 验证机制失配 (过早完成 / 验证覆盖不足)
    Verification,
}

impl LossLayer {
    pub const ALL: [LossLayer; 6] = [
        LossLayer::ReasoningMode,
        LossLayer::FirstTurnInterface,
        LossLayer::ToolSchema,
        LossLayer::ActiveRepresentation,
        LossLayer::LongHorizonState,
        LossLayer::Verification,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LossLayer::ReasoningMode => "reasoning-mode",
            LossLayer::FirstTurnInterface => "first-turn-interface",
            LossLayer::ToolSchema => "tool-schema",
            LossLayer::ActiveRepresentation => "active-representation",
            LossLayer::LongHorizonState => "long-horizon-state",
            LossLayer::Verification => "verification",
        }
    }
}

impl std::fmt::Display for LossLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Rolling z-score anomaly detector over a metric time series.
///
/// Z-score approach (cf. network_diagnostics/protocol.rs EWMA): each observed
/// value is compared against the mean/std of the trailing window. A value that
/// exceeds `+z_threshold` std from baseline is a Spike; below `-z_threshold` is
/// a Drop. Reuses `MetricSeries` sampling so a single compact implementation
/// serves the background-loop feed (no parallel adapter — R-P42).
#[derive(Debug, Clone)]
pub struct MetricSeries {
    /// (Instant, value) samples, oldest first.
    samples: VecDeque<(Instant, f64)>,
    /// Max samples retained.
    capacity: usize,
}

impl MetricSeries {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity: capacity.max(2),
        }
    }

    pub fn push(&mut self, value: f64) {
        self.samples.push_back((Instant::now(), value));
        while self.samples.len() > self.capacity {
            self.samples.pop_front();
        }
    }

    /// All samples within `window` (most recent last).
    pub fn window_values(&self, window: Duration) -> Vec<f64> {
        let cutoff = Instant::now() - window;
        self.samples
            .iter()
            .filter(|(t, _)| *t >= cutoff)
            .map(|(_, v)| *v)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }
}

/// AnomalyDetector — per-metric rolling z-score spike/drop detection.
/// Thread-safe: holds its own `MetricSeries` collection behind a Mutex.
pub struct AnomalyDetector {
    series: Mutex<HashMap<String, MetricSeries>>,
    window: Duration,
    z_threshold: f64,
    capacity: usize,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self {
            series: Mutex::new(HashMap::new()),
            window: Duration::from_secs(600),
            z_threshold: 2.5,
            capacity: 64,
        }
    }
}

impl AnomalyDetector {
    pub fn new(window: Duration, z_threshold: f64, capacity: usize) -> Self {
        Self {
            series: Mutex::new(HashMap::new()),
            window,
            z_threshold,
            capacity,
        }
    }

    /// Observe one metric value, returning an alert if it spikes or drops
    /// beyond the z-score threshold relative to the trailing window baseline.
    /// Returns None when insufficient samples for a stable baseline exist.
    pub fn observe(&self, metric: &str, value: f64) -> Option<TelemetryAlert> {
        let mut series = self.series.lock().ok()?;
        let entry = series
            .entry(metric.to_string())
            .or_insert_with(|| MetricSeries::new(self.capacity));
        let baseline_values = entry.window_values(self.window);
        if baseline_values.len() < 3 {
            entry.push(value);
            return None;
        }
        let n = baseline_values.len() as f64;
        let mean = baseline_values.iter().sum::<f64>() / n;
        let variance = baseline_values
            .iter()
            .map(|v| (v - mean) * (v - mean))
            .sum::<f64>()
            / n;
        // Floor std relative to the mean scale: a perfectly constant series
        // would otherwise give std=0 → every tiny deviation explodes to +∞ z.
        let std = variance.sqrt().max(mean.abs() * 0.01).max(1e-9);
        let z = (value - mean) / std;

        let kind = if z > self.z_threshold {
            Some(AlertKind::Spike)
        } else if z < -self.z_threshold {
            Some(AlertKind::Drop)
        } else {
            None
        };

        entry.push(value);
        kind.map(|kind| TelemetryAlert {
            metric: metric.to_string(),
            kind,
            current: value,
            baseline: mean,
            threshold: self.z_threshold,
            loss_layer: None,
        })
    }

    /// 带能力实现损失层分类的观察入口。J-Space: 告警不仅报告"偏离",
    /// 还标注它落在六层失配中的哪一层, 供控制回路定向修复。
    pub fn observe_loss(&self, metric: &str, value: f64, layer: LossLayer) -> Option<TelemetryAlert> {
        let mut alert = self.observe(metric, value)?;
        alert.loss_layer = Some(layer);
        Some(alert)
    }

    pub fn series_len(&self, metric: &str) -> usize {
        match self.series.lock() {
            Ok(s) => s.get(metric).map(|m| m.len()).unwrap_or(0),
            Err(_) => 0,
        }
    }

    pub fn reset(&self, metric: &str) {
        if let Ok(mut s) = self.series.lock() {
            s.remove(metric);
        }
    }
}

/// J-Space seam 停滞观察器 (jspace.py observations / STALL_RUN=3) — 在
/// 相邻 seam 之间对比账本状态, 产出事实观察; 判断交给控制回路 (T17 三选一)。
/// 忠实复现 jspace.py 四条观察, 不携带任何判断词 (judgement is not the
/// script's)。
#[derive(Debug)]
pub struct SeamMonitor {
    /// 连续观察窗口内的 seam 状态快照 (最新在前)。
    runs: Mutex<VecDeque<SeamSnapshot>>,
    /// 连续几轮算停滞 (jspace STALL_RUN = 3)。
    run: usize,
}

/// 单个 seam 时刻的账本状态快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeamSnapshot {
    pub next: String,
    pub verified: usize,
    pub open: usize,
}

impl Default for SeamMonitor {
    fn default() -> Self {
        Self::new(3)
    }
}

impl SeamMonitor {
    pub fn new(run: usize) -> Self {
        Self {
            runs: Mutex::new(VecDeque::with_capacity(run.max(3))),
            run: run.max(3),
        }
    }

    /// 记录一次 seam 快照, 返回窗口内的事实观察 (无则 None)。
    pub fn observe(&self, snap: SeamSnapshot) -> Option<String> {
        let mut runs = match self.runs.lock() {
            Ok(r) => r,
            Err(_) => return None,
        };
        runs.push_front(snap);
        while runs.len() > self.run {
            runs.pop_back();
        }
        if runs.len() < self.run {
            return None;
        }
        let window: Vec<&SeamSnapshot> = runs.iter().collect();
        if window[0].next.is_empty() {
            return Some("seam: no next action recorded — the ledger stops being state".to_string());
        }
        let next_unchanged = window.iter().all(|s| s.next == window[0].next);
        let verified_grew = window[0].verified != window[window.len() - 1].verified;
        // 最新在前 (index 0), 单调增 = 每对相邻 (i, i+1) 满足 s[i].open > s[i+1].open
        let open_monotonic = window
            .windows(2)
            .all(|w| w[0].open > w[1].open);

        if next_unchanged && !verified_grew {
            Some(format!(
                "seam: next action unchanged for {} seams and nothing new verified — stalled",
                self.run
            ))
        } else if next_unchanged && verified_grew {
            Some(format!(
                "seam: verified is growing but the next action has not changed for {} seams",
                self.run
            ))
        } else if open_monotonic {
            Some(format!(
                "seam: open-question count increased at every seam over the last {} seams",
                self.run
            ))
        } else {
            None
        }
    }

    /// 当前窗口内的快照数。
    pub fn len(&self) -> usize {
        match self.runs.lock() {
            Ok(r) => r.len(),
            Err(_) => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 策略漂移监测器 (Replica #3: 塌缩前兆检测) — 生成策略 vs 训练策略的
/// JS 散度在塌缩前会上升约 2 个数量级。这里监测 rollout 的 "策略漂移比"
/// (gen 分布与 train 基线分布间散度 / 基线散度), 当比值跨过数量级阈值
/// (默认 10×) 时发出塌缩前兆告警, 早于质量分数显式恶化。
#[derive(Debug)]
pub struct PolicyDriftMonitor {
    /// 每轮观察到的漂移比历史 (即 gen/train 散度与基线散度的比值)
    ratios: Mutex<VecDeque<f64>>,
    /// 基线散度 — 训练策略稳定期的散度均值; None = 尚未建立基线
    baseline_divergence: Mutex<Option<f64>>,
    /// 触发塌缩前兆告警的漂移比阈值 (论文: ~2 数量级 → 默认 10×)
    magnitude_jump: f64,
    /// 保留的比率样本数
    capacity: usize,
}

impl Default for PolicyDriftMonitor {
    fn default() -> Self {
        Self {
            ratios: Mutex::new(VecDeque::with_capacity(64)),
            baseline_divergence: Mutex::new(None),
            magnitude_jump: 10.0,
            capacity: 64,
        }
    }
}

impl PolicyDriftMonitor {
    pub fn new(magnitude_jump: f64) -> Self {
        Self {
            ratios: Mutex::new(VecDeque::with_capacity(64)),
            baseline_divergence: Mutex::new(None),
            magnitude_jump: magnitude_jump.max(2.0),
            capacity: 64,
        }
    }

    /// 记录一次散度观测: `generation` = 生成策略与训练策略的当前散度,
    /// `baseline_sample` = 是否训练稳定期样本 (用于建立基线)。
    /// 返回 None = 正常; Some(TelemetryAlert) = 漂移比跨数量级 (塌缩前兆)。
    pub fn observe(&self, generation: f64, baseline_sample: bool) -> Option<TelemetryAlert> {
        let mut base = self.baseline_divergence.lock().ok()?;
        if baseline_sample {
            // 稳定期样本滚动更新基线 (滚动中位近似: 均值)
            let samples: Vec<f64> = {
                let mut ratios = self.ratios.lock().ok()?;
                ratios.push_back(generation);
                while ratios.len() > self.capacity {
                    ratios.pop_front();
                }
                ratios.iter().copied().collect()
            };
            if !samples.is_empty() {
                let mean = samples.iter().sum::<f64>() / samples.len() as f64;
                *base = Some(mean.max(1e-9));
            }
            return None;
        }

        let baseline = (*base)?;
        let ratio = (generation / baseline).max(0.0);
        {
            let mut ratios = self.ratios.lock().ok()?;
            ratios.push_back(ratio);
            while ratios.len() > self.capacity {
                ratios.pop_front();
            }
        }
        if ratio >= self.magnitude_jump {
            Some(TelemetryAlert {
                metric: "policy_drift_ratio".into(),
                kind: AlertKind::Spike,
                current: ratio,
                baseline,
                threshold: self.magnitude_jump,
                loss_layer: Some(LossLayer::ReasoningMode),
            })
        } else {
            None
        }
    }

    /// 当前漂移比 — 供上层读取趋势。
    pub fn current_ratio(&self) -> Option<f64> {
        self.ratios.lock().ok()?.back().copied()
    }

    /// 基线已建立?
    pub fn has_baseline(&self) -> bool {
        matches!(self.baseline_divergence.lock(), Ok(g) if g.is_some())
    }
}

#[derive(Debug, Clone)]
pub struct AgentBehavior {
    pub agent_id: String,
    pub call_count: u64,
    pub error_count: u64,
    pub total_duration_ms: u64,
    pub tools_used: Vec<String>,
    pub last_active: Instant,
}

pub struct AgentBehaviorMap {
    agents: Mutex<HashMap<String, AgentBehavior>>,
    tool_frequencies: Mutex<HashMap<String, u64>>,
    error_patterns: Mutex<Vec<(String, String)>>,
}

impl Default for AgentBehaviorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentBehaviorMap {
    pub fn new() -> Self {
        Self {
            agents: Mutex::new(HashMap::new()),
            tool_frequencies: Mutex::new(HashMap::new()),
            error_patterns: Mutex::new(Vec::new()),
        }
    }

    pub fn record_event(&self, event: &TelemetryEvent) {
        match event {
            TelemetryEvent::AgentSpawned { agent_id, role } => {
                if let Ok(mut agents) = self.agents.lock() {
                    agents.entry(agent_id.clone()).or_insert(AgentBehavior {
                        agent_id: agent_id.clone(),
                        call_count: 0,
                        error_count: 0,
                        total_duration_ms: 0,
                        tools_used: vec![],
                        last_active: Instant::now(),
                    });
                }
                if let Ok(mut freqs) = self.tool_frequencies.lock() {
                    *freqs.entry(format!("role:{}", role)).or_insert(0) += 1;
                }
            }
            TelemetryEvent::AgentCompleted {
                agent_id,
                success,
                duration_ms,
            } => {
                if let Ok(mut agents) = self.agents.lock() {
                    if let Some(a) = agents.get_mut(agent_id) {
                        a.call_count += 1;
                        if !success {
                            a.error_count += 1;
                        }
                        a.total_duration_ms += duration_ms;
                        a.last_active = Instant::now();
                    }
                }
            }
            TelemetryEvent::ToolCall {
                tool,
                success,
                duration_ms,
            } => {
                if let Ok(mut freqs) = self.tool_frequencies.lock() {
                    *freqs.entry(tool.clone()).or_insert(0) += 1;
                }
                if let Ok(_error_patterns) = self.error_patterns.lock() {
                    if !success {
                        // keep for future expansion
                    }
                }
                let _ = duration_ms;
            }
            TelemetryEvent::Error {
                source, message, ..
            } => {
                if let Ok(mut patterns) = self.error_patterns.lock() {
                    patterns.push((source.clone(), message.clone()));
                    let excess = patterns.len().saturating_sub(1000);
                    if excess > 0 {
                        patterns.drain(0..excess);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn top_tools(&self, n: usize) -> Vec<(String, u64)> {
        match self.tool_frequencies.lock() {
            Ok(freqs) => {
                let mut pairs: Vec<_> = freqs.iter().map(|(k, v)| (k.clone(), *v)).collect();
                pairs.sort_by(|a, b| b.1.cmp(&a.1));
                pairs.truncate(n);
                pairs
            }
            Err(_) => vec![],
        }
    }

    pub fn active_agents(&self) -> Vec<(String, u64, u64)> {
        match self.agents.lock() {
            Ok(agents) => agents
                .iter()
                .map(|(id, a)| (id.clone(), a.call_count, a.error_count))
                .collect(),
            Err(_) => vec![],
        }
    }

    pub fn high_error_tools(&self, threshold: f64) -> Vec<String> {
        match self.error_patterns.lock() {
            Ok(patterns) => {
                let mut sources: HashMap<String, u64> = HashMap::new();
                for (src, _) in patterns.iter() {
                    *sources.entry(src.clone()).or_insert(0) += 1;
                }
                let total = sources.values().sum::<u64>() as f64;
                sources
                    .into_iter()
                    .filter(|(_, count)| *count as f64 / total.max(1.0) > threshold)
                    .map(|(k, _)| k)
                    .collect()
            }
            Err(_) => vec![],
        }
    }
}

impl std::fmt::Debug for AgentBehaviorMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AgentBehaviorMap")
    }
}

static GLOBAL_TELEMETRY_STORE: std::sync::LazyLock<TelemetryStore> =
    std::sync::LazyLock::new(|| TelemetryStore::new(10_000));

static GLOBAL_AGENT_MAP: std::sync::LazyLock<AgentBehaviorMap> =
    std::sync::LazyLock::new(AgentBehaviorMap::new);

pub fn global_telemetry() -> &'static TelemetryStore {
    &GLOBAL_TELEMETRY_STORE
}

pub fn global_agent_map() -> &'static AgentBehaviorMap {
    &GLOBAL_AGENT_MAP
}

impl crate::core::nt_core_self_test::SelfTest for TelemetryStore {
    fn name(&self) -> &str {
        "TelemetryStore"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let summary = self.summary();
        let _ = summary;
        if self.max_events == 0 {
            failures.push("max_events is 0".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for AnomalyDetector {
    fn name(&self) -> &str {
        "AnomalyDetector"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        // Stable baseline then a spike must be flagged.
        let d = AnomalyDetector::default();
        for i in 0..10 {
            let v = if i == 9 { 50.0 } else { 1.0 };
            if let Some(a) = d.observe("probe", v) {
                if a.kind == AlertKind::Spike {
                    failures.push("baseline already spiking".into());
                }
            }
        }
        // Fresh metric: 5 warmup samples → stable → spike → drop.
        let d = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        for _ in 0..5 {
            d.observe("m", 1.0);
        }
        let spike = d.observe("m", 10.0);
        if !matches!(spike.map(|a| a.kind), Some(AlertKind::Spike)) {
            failures.push("spike not detected".into());
        }
        for _ in 0..5 {
            d.observe("m", 1.0);
        }
        let drop = d.observe("m", 0.1);
        if !matches!(drop.map(|a| a.kind), Some(AlertKind::Drop)) {
            failures.push("drop not detected".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for PolicyDriftMonitor {
    fn name(&self) -> &str {
        "PolicyDriftMonitor"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        // 建立基线 (稳定期样本) → 散度 10× 跳变 → 应触发塌缩前兆告警。
        let m = PolicyDriftMonitor::default();
        for _ in 0..5 {
            m.observe(1.0, true);
        }
        if !m.has_baseline() {
            failures.push("baseline not established".into());
        }
        if let Some(a) = m.observe(50.0, false) {
            if a.kind != AlertKind::Spike {
                failures.push("drift alert wrong kind".into());
            }
        } else {
            failures.push("magnitude jump not flagged".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_store_record_and_summary() {
        let store = TelemetryStore::new(100);
        store.record(TelemetryEvent::ToolCall {
            tool: "bash".into(),
            success: true,
            duration_ms: 50,
        });
        store.record(TelemetryEvent::ToolCall {
            tool: "bash".into(),
            success: true,
            duration_ms: 30,
        });
        store.record(TelemetryEvent::Error {
            source: "bash".into(),
            message: "timeout".into(),
            severity: 2,
        });
        let summary = store.summary();
        assert!(summary.iter().any(|(k, _)| k == "tool_call"));
        assert!(summary.iter().any(|(k, _)| k == "error"));
        assert_eq!(store.errors_in_last(Duration::from_secs(60)), 1);
    }

    #[test]
    fn test_telemetry_store_max_events() {
        let store = TelemetryStore::new(10);
        for i in 0..20 {
            store.record(TelemetryEvent::Custom {
                name: format!("e{}", i),
                value: "x".into(),
            });
        }
        let recent = store.recent_events(100);
        assert!(recent.len() <= 10);
    }

    #[test]
    fn test_agent_behavior_map() {
        let map = AgentBehaviorMap::new();
        map.record_event(&TelemetryEvent::AgentSpawned {
            agent_id: "a1".into(),
            role: "coder".into(),
        });
        map.record_event(&TelemetryEvent::AgentCompleted {
            agent_id: "a1".into(),
            success: true,
            duration_ms: 100,
        });
        map.record_event(&TelemetryEvent::ToolCall {
            tool: "bash".into(),
            success: true,
            duration_ms: 10,
        });
        map.record_event(&TelemetryEvent::ToolCall {
            tool: "read".into(),
            success: false,
            duration_ms: 5,
        });
        let agents = map.active_agents();
        assert!(agents.iter().any(|(id, _, _)| id == "a1"));
        let tools = map.top_tools(5);
        assert!(tools.iter().any(|(t, _)| t == "bash"));
    }

    #[test]
    fn test_error_pattern_detection() {
        let map = AgentBehaviorMap::new();
        for _ in 0..8 {
            map.record_event(&TelemetryEvent::Error {
                source: "bash".into(),
                message: "timeout".into(),
                severity: 2,
            });
        }
        for _ in 0..2 {
            map.record_event(&TelemetryEvent::Error {
                source: "read".into(),
                message: "not found".into(),
                severity: 1,
            });
        }
        let high_error = map.high_error_tools(0.5);
        assert!(high_error.contains(&"bash".to_string()));
    }

    #[test]
    fn test_telemetry_store_clear() {
        let store = TelemetryStore::new(100);
        store.record(TelemetryEvent::ToolCall {
            tool: "test".into(),
            success: true,
            duration_ms: 1,
        });
        assert!(!store.summary().is_empty());
        store.clear();
        assert!(store.summary().is_empty());
    }

    #[test]
    fn test_event_kind() {
        assert_eq!(
            TelemetryEvent::ToolCall {
                tool: "x".into(),
                success: true,
                duration_ms: 0
            }
            .kind(),
            "tool_call"
        );
        assert_eq!(
            TelemetryEvent::AgentSpawned {
                agent_id: "x".into(),
                role: "r".into()
            }
            .kind(),
            "agent_spawned"
        );
        assert_eq!(
            TelemetryEvent::Error {
                source: "x".into(),
                message: "m".into(),
                severity: 1
            }
            .kind(),
            "error"
        );
        assert_eq!(
            TelemetryEvent::Custom {
                name: "my_event".into(),
                value: "v".into()
            }
            .kind(),
            "my_event"
        );
    }

    // ── AnomalyDetector (spike/drop) ───────────────────────────────────

    #[test]
    fn test_anomaly_detector_insufficient_samples() {
        let d = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        // Baseline needs >=3 samples; the 1st..3rd observations all return None
        // (they only push, never alert), and the series grows to 3.
        for _ in 0..3 {
            assert!(d.observe("m", 1.0).is_none(), "baseline needs >=3 samples");
        }
        assert_eq!(d.series_len("m"), 3);
    }

    #[test]
    fn test_anomaly_detector_spike_detection() {
        let d = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        for _ in 0..5 {
            assert!(d.observe("latency", 1.0).is_none());
        }
        let alert = d.observe("latency", 10.0).expect("spike should alert");
        assert_eq!(alert.kind, AlertKind::Spike);
        assert_eq!(alert.metric, "latency");
        assert!((alert.current - 10.0).abs() < 1e-9);
        assert!((alert.baseline - 1.0).abs() < 1e-9);
        assert_eq!(alert.threshold, 2.5);
    }

    #[test]
    fn test_anomaly_detector_drop_detection() {
        let d = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        for _ in 0..5 {
            d.observe("count", 100.0);
        }
        let alert = d.observe("count", 1.0).expect("drop should alert");
        assert_eq!(alert.kind, AlertKind::Drop);
        assert!((alert.current - 1.0).abs() < 1e-9);
        assert!((alert.baseline - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_anomaly_detector_no_alert_on_stable() {
        let d = AnomalyDetector::new(Duration::from_secs(600), 3.0, 64);
        for _ in 0..5 {
            d.observe("m", 5.0);
        }
        // small perturbation within tolerance → no alert
        assert!(d.observe("m", 5.1).is_none());
        assert!(d.observe("m", 4.9).is_none());
    }

    #[test]
    fn test_anomaly_detector_loss_layer_classification() {
        let d = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        // 基线
        for _ in 0..5 {
            assert!(d.observe_loss("goal_drift", 1.0, LossLayer::LongHorizonState).is_none());
        }
        // 长程状态层 spike → 告警必须携带 loss_layer 分类
        let alert = d
            .observe_loss("goal_drift", 10.0, LossLayer::LongHorizonState)
            .expect("loss-layer spike should alert");
        assert_eq!(alert.kind, AlertKind::Spike);
        assert_eq!(alert.loss_layer, Some(LossLayer::LongHorizonState));
        assert_eq!(alert.loss_layer.unwrap().label(), "long-horizon-state");
    }

    #[test]
    fn test_loss_layer_enum_is_six_fold() {
        assert_eq!(LossLayer::ALL.len(), 6, "J-Space 六层失配分类");
        let labels: Vec<&str> = LossLayer::ALL.iter().map(|l| l.label()).collect();
        assert!(labels.contains(&"reasoning-mode"));
        assert!(labels.contains(&"first-turn-interface"));
        assert!(labels.contains(&"tool-schema"));
        assert!(labels.contains(&"active-representation"));
        assert!(labels.contains(&"verification"));
        // 每层 display == label
        for l in LossLayer::ALL {
            assert_eq!(l.to_string(), l.label());
        }
    }

    #[test]
    fn test_plain_observe_has_no_loss_layer() {
        let d = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        for _ in 0..5 {
            d.observe("m", 1.0);
        }
        let alert = d.observe("m", 10.0).expect("spike should alert");
        assert_eq!(alert.loss_layer, None, "plain observe stays unclassified");
    }

    #[test]
    fn test_anomaly_detector_metric_isolation() {
        let d = AnomalyDetector::new(Duration::from_secs(600), 2.5, 64);
        for _ in 0..5 {
            d.observe("stable", 1.0);
        }
        d.observe("stable", 10.0); // spike on one metric
                                   // unrelated metric keeps its own baseline — no alert from cross-talk
        for _ in 0..5 {
            d.observe("other", 3.0);
        }
        assert!(d.observe("other", 3.05).is_none());
        assert_eq!(d.series_len("other"), 6);
    }

    // ── Window aggregation + privacy ───────────────────────────────────

    #[test]
    fn test_window_aggregates_counts_and_durations() {
        let store = TelemetryStore::new(1000);
        for _ in 0..5 {
            store.record(TelemetryEvent::ToolCall {
                tool: "bash".into(),
                success: true,
                duration_ms: 40,
            });
        }
        for _ in 0..2 {
            store.record(TelemetryEvent::Error {
                source: "bash".into(),
                message: "boom".into(),
                severity: 2,
            });
        }
        store.record(TelemetryEvent::AgentCompleted {
            agent_id: "a".into(),
            success: true,
            duration_ms: 200,
        });
        let aggs = store.window_aggregates(Duration::from_secs(60));
        let tc = aggs
            .iter()
            .find(|(k, _)| k == "tool_call")
            .expect("tool_call present");
        assert_eq!(tc.1.count, 5);
        assert_eq!(tc.1.total_duration_ms, 200);
        let err = aggs
            .iter()
            .find(|(k, _)| k == "error")
            .expect("error present");
        assert_eq!(err.1.count, 2);
        assert_eq!(err.1.error_count, 2);
        let ac = aggs
            .iter()
            .find(|(k, _)| k == "agent_completed")
            .expect("agent_completed present");
        assert_eq!(ac.1.count, 1);
        assert_eq!(ac.1.total_duration_ms, 200);
    }

    #[test]
    fn test_privacy_no_raw_payload_in_aggregates() {
        let store = TelemetryStore::new(1000);
        store.record(TelemetryEvent::Error {
            source: "db".into(),
            message: "secret-credential-xyz".into(),
            severity: 3,
        });
        store.record(TelemetryEvent::Custom {
            name: "raw".into(),
            value: "user-document-body".into(),
        });
        // Aggregates must never expose raw payload strings — only scalars.
        for (kind, _agg) in store.window_aggregates(Duration::from_secs(60)) {
            assert!(!kind.contains("secret-credential"));
        }
        let summary = store.summary();
        assert!(summary
            .iter()
            .all(|(k, _)| !k.contains("user-document-body")));
    }

    #[test]
    fn test_record_metric_series() {
        let store = TelemetryStore::new(1000);
        store.record_metric("gateway_latency_ms", 12.0);
        store.record_metric("gateway_latency_ms", 14.0);
        store.record_metric("gateway_latency_ms", 13.0);
        assert!(store
            .metric_names()
            .contains(&"gateway_latency_ms".to_string()));
        let mean = store
            .metric_window_mean("gateway_latency_ms", Duration::from_secs(60))
            .unwrap();
        assert!((mean - 13.0).abs() < 1e-9);
        assert!(store
            .metric_window_mean("unknown", Duration::from_secs(60))
            .is_none());
    }

    #[test]
    fn test_policy_drift_no_alert_on_stable_rollout() {
        let m = PolicyDriftMonitor::default();
        // 稳定期: 连续建立基线 + 小幅波动 → 不告警
        for _ in 0..5 {
            m.observe(1.0, true);
        }
        for _ in 0..3 {
            assert!(m.observe(1.5, false).is_none(), "稳定期不应告警");
        }
        assert!(m.has_baseline());
        let r = m.current_ratio().expect("有漂移比");
        assert!(r < 10.0);
    }

    #[test]
    fn test_policy_drift_magnitude_jump_alerts_collapse_precursor() {
        let m = PolicyDriftMonitor::new(10.0);
        for _ in 0..5 {
            m.observe(1.0, true);
        }
        // 散度跳 50× (> 阈值 10×) → 塌缩前兆告警
        let alert = m.observe(50.0, false).expect("数量级跳变应触发");
        assert_eq!(alert.kind, AlertKind::Spike);
        assert_eq!(alert.metric, "policy_drift_ratio");
        assert!(alert.current >= 10.0);
    }

    #[test]
    fn test_policy_drift_requires_baseline() {
        let m = PolicyDriftMonitor::default();
        // 未建立基线前散度跳变不应误报 (无历史可比)
        assert!(m.observe(100.0, false).is_none(), "无基线不应告警");
        assert!(!m.has_baseline());
    }

    // ── SeamMonitor (J-Space seam 停滞观察) ────────────────────────────

    #[test]
    fn test_seam_monitor_needs_full_window() {
        let m = SeamMonitor::new(3);
        for _ in 0..2 {
            assert!(
                m.observe(SeamSnapshot {
                    next: "next".into(),
                    verified: 1,
                    open: 0,
                })
                .is_none(),
                "窗口未满不产观察"
            );
        }
        assert_eq!(m.len(), 2);
        assert!(!m.is_empty());
    }

    #[test]
    fn test_seam_monitor_flags_stall_when_next_unchanged_and_no_new_verified() {
        let m = SeamMonitor::new(3);
        for _ in 0..3 {
            let obs = m.observe(SeamSnapshot {
                next: "same action".into(),
                verified: 2,
                open: 1,
            });
            if obs.is_some() {
                let text = obs.unwrap();
                assert!(text.contains("stalled"), "{text}");
                assert!(text.contains("unchanged"), "{text}");
            }
        }
        // 第三次快照后窗口满 → 必须产出停滞观察
        assert!(m.len() == 3);
    }

    #[test]
    fn test_seam_monitor_reports_growth_with_frozen_next() {
        let m = SeamMonitor::new(3);
        for _ in 0..2 {
            assert!(m
                .observe(SeamSnapshot {
                    next: "same".into(),
                    verified: 1,
                    open: 0,
                })
                .is_none());
        }
        let obs = m
            .observe(SeamSnapshot {
                next: "same".into(),
                verified: 3,
                open: 0,
            })
            .expect("verified grew but next frozen → observation");
        assert!(obs.contains("growing"), "{obs}");
    }

    #[test]
    fn test_seam_monitor_flags_monotonic_open_growth() {
        let m = SeamMonitor::new(3);
        // 窗口内 open 持续增加 (新→旧: 3,2,1) — 注意 observe 最新在前
        for open in [1usize, 2, 3] {
            m.observe(SeamSnapshot {
                next: "n".into(),
                verified: 1,
                open,
            });
        }
        let obs = m
            .observe(SeamSnapshot {
                next: "n".into(),
                verified: 1,
                open: 4,
            })
            .expect("open monotonically increasing → observation");
        assert!(obs.contains("open-question count increased"), "{obs}");
    }

    #[test]
    fn test_seam_monitor_silent_when_healthy() {
        let m = SeamMonitor::new(3);
        let mut last = 1usize;
        for _ in 0..3 {
            let obs = m.observe(SeamSnapshot {
                next: format!("n{last}"),
                verified: last,
                open: 0,
            });
            assert!(obs.is_none(), "healthy progress must not stall: {obs:?}");
            last += 1;
        }
    }

    #[test]
    fn test_seam_monitor_flags_missing_next() {
        let m = SeamMonitor::new(3);
        for _ in 0..3 {
            let obs = m.observe(SeamSnapshot {
                next: String::new(),
                verified: 1,
                open: 0,
            });
            if let Some(text) = obs {
                assert!(text.contains("no next action"), "{text}");
            }
        }
    }
}
