use std::collections::{HashMap, VecDeque};

use crate::core::nt_core_aware::*;
use crate::neotrix::nt_core_iit_phi::{IITPhiCalculator, PhiReport};
use crate::neotrix::nt_act_autonomy::awareness_monitor::SelfAwarenessMonitor;

const HISTORY_CAPACITY: usize = 100;
const ENTROPY_BASELINE: f64 = 3.46; // log2(11) for 11 specialist modules

pub struct ConsciousnessMonitor {
    /// IIT Phi calculator
    pub phi_calculator: IITPhiCalculator,
    /// Awareness history (last 100 snapshots)
    pub awareness_history: VecDeque<ConsciousnessSnapshot>,
    /// Current awareness state
    pub current: ConsciousnessAwareness,
    /// Self-awareness monitor for capability gaps
    pub capability_monitor: Option<SelfAwarenessMonitor>,
    /// Recorded consciousness levels for trend analysis
    pub trends: AwarenessTrends,
    /// Last full PhiReport from compute_phi()
    pub last_phi_report: Option<PhiReport>,
}

pub struct AwarenessTrends {
    pub phi_trend: Vec<f64>,
    pub coherence_trend: Vec<f64>,
    pub health_trend: Vec<f64>,
    pub strategy_effectiveness: HashMap<String, Vec<f64>>,
}

impl Default for AwarenessTrends {
    fn default() -> Self {
        Self {
            phi_trend: Vec::with_capacity(10),
            coherence_trend: Vec::with_capacity(10),
            health_trend: Vec::with_capacity(10),
            strategy_effectiveness: HashMap::new(),
        }
    }
}

/// Structured awareness report for display
pub struct AwarenessReport {
    pub timestamp: String,
    pub consciousness: f64,
    pub coherence: f64,
    pub phi: f64,
    pub attention_state: String,
    pub blind_spots: Vec<String>,
    pub conversation_turns: usize,
    pub conversation_stage: String,
    pub topic_coherence: f64,
    pub health: f64,
    pub trends: String,
}

impl ConsciousnessMonitor {
    pub fn new() -> Self {
        Self {
            phi_calculator: IITPhiCalculator::new(),
            awareness_history: VecDeque::with_capacity(HISTORY_CAPACITY),
            current: ConsciousnessAwareness::default(),
            capability_monitor: Some(SelfAwarenessMonitor::new()),
            trends: AwarenessTrends::default(),
            last_phi_report: None,
        }
    }

    /// Run a full self-observation cycle, updating `current` awareness state.
    pub fn observe(&mut self) {
        let phi = self.phi_calculator.compute_phi(&self.current_phi_state());
        self.last_phi_report = Some(phi.clone());
        self.current.phi_current = phi.phi;

        let coherence = self.compute_coherence();
        self.current.coherence_current = coherence;

        let health = self.compute_health();
        self.current.health = health;

        let attention_entropy = self.compute_attention_entropy();
        let blind_spot_count = self.current.active_blind_spots.len();

        self.current.consciousness_level = self.compute_consciousness_level();

        self.current.is_conscious_bound = coherence > CONSCIOUS_BOUND_THRESHOLD
            && phi.phi > CONSCIOUS_BOUND_THRESHOLD;

        self.current.recorded_at = chrono::Utc::now();

        let snapshot = ConsciousnessSnapshot::new(
            phi.phi,
            coherence,
            attention_entropy,
            health,
            blind_spot_count,
            self.current.conversation_awareness.turn_count,
        );

        self.awareness_history.push_back(snapshot);
        if self.awareness_history.len() > HISTORY_CAPACITY {
            self.awareness_history.pop_front();
        }

        self.trends.phi_trend.push(phi.phi);
        if self.trends.phi_trend.len() > 10 {
            self.trends.phi_trend.remove(0);
        }
        self.trends.coherence_trend.push(coherence);
        if self.trends.coherence_trend.len() > 10 {
            self.trends.coherence_trend.remove(0);
        }
        self.trends.health_trend.push(health);
        if self.trends.health_trend.len() > 10 {
            self.trends.health_trend.remove(0);
        }
    }

    /// Update conversation awareness after each turn.
    pub fn observe_conversation_turn(&mut self, topic: &str, response_len: usize, complexity: f64) {
        let turn_count = self.current.conversation_awareness.turn_count + 1;
        let prev_topic = self.current.conversation_awareness.previous_topics.last().cloned();
        let prev_depth = self.current.conversation_awareness.depth_trend;

        // Compute topic drift from previous topic
        let (topic_drift, topic_coherence) = if let Some(ref prev) = prev_topic {
            let overlap = self.keyword_overlap(prev, topic);
            (1.0 - overlap, overlap)
        } else {
            (0.0, 1.0)
        };

        let user_engagement = (response_len as f64 / 5000.0).min(1.0);
        let stage = self.estimate_conversation_stage(turn_count, topic_drift);
        let depth_trend = match prev_depth {
            Some(prev) => complexity - prev,
            None => 0.0,
        };
        let self_assessed_quality = 0.5 * topic_coherence
            + 0.3 * user_engagement
            + 0.2 * (1.0 - topic_drift);

        let ca = &mut self.current.conversation_awareness;
        ca.turn_count = turn_count;
        ca.topic_drift = topic_drift;
        ca.topic_coherence = topic_coherence;
        ca.user_engagement = user_engagement;
        ca.stage = stage;
        ca.depth_trend = Some(depth_trend);
        ca.self_assessed_quality = self_assessed_quality;
        ca.previous_topics.push(topic.to_string());
        if ca.previous_topics.len() > 20 {
            ca.previous_topics.remove(0);
        }
    }

    /// Aggregate Phi, coherence, health into single score.
    pub fn compute_consciousness_level(&self) -> f64 {
        let phi = self.current.phi_current;
        let coherence = self.current.coherence_current;
        let health = self.current.health;

        let level = PHI_WEIGHT * phi.clamp(0.0, 1.0)
            + COHERENCE_WEIGHT * coherence.clamp(0.0, 1.0)
            + HEALTH_WEIGHT * health.clamp(0.0, 1.0);

        level.clamp(CONSCIOUSNESS_MIN, CONSCIOUSNESS_MAX)
    }

    /// Return a human-readable introspection string.
    pub fn introspect(&self) -> String {
        let trend = self.trend_analysis();
        let stage = self.current.conversation_awareness.stage.label();

        let mut parts = vec![
            format!("Consciousness: {:.3}", self.current.consciousness_level),
            format!("Phi: {:.4}", self.current.phi_current),
            format!("Coherence: {:.3}", self.current.coherence_current),
            format!("Health: {:.3}", self.current.health),
            format!("Bound: {}", self.current.is_conscious_bound),
            format!("Stage: {}", stage),
            format!("Turns: {}", self.current.conversation_awareness.turn_count),
            format!("Trend: {}", trend),
        ];

        if !self.current.active_blind_spots.is_empty() {
            let spots: Vec<String> = self.current.active_blind_spots.iter()
                .map(|b| format!("[{}] {}", b.kind, b.description))
                .collect();
            parts.push(format!("Blind spots ({}): {}", self.current.active_blind_spots.len(), spots.join("; ")));
        }

        parts.join(" | ")
    }

    /// Detect if consciousness is improving/declining.
    pub fn trend_analysis(&self) -> &str {
        let phi_n = self.trends.phi_trend.len();
        let coh_n = self.trends.coherence_trend.len();
        let h_n = self.trends.health_trend.len();

        if phi_n < 3 || coh_n < 3 || h_n < 3 {
            return "insufficient_data";
        }

        let phi_trend = self.slope(&self.trends.phi_trend);
        let coh_trend = self.slope(&self.trends.coherence_trend);
        let health_trend = self.slope(&self.trends.health_trend);

        let avg = (phi_trend + coh_trend + health_trend) / 3.0;

        if avg > 0.01 {
            "improving"
        } else if avg < -0.01 {
            "declining"
        } else {
            "stable"
        }
    }

    /// Return structured AwarenessReport.
    pub fn get_report(&self) -> AwarenessReport {
        let trend = self.trend_analysis().to_string();
        let blind_spots: Vec<String> = self.current.active_blind_spots.iter()
            .map(|b| format!("{} (severity {})", b.kind, b.severity))
            .collect();
        let attention_str = if self.current.attention_profile.is_empty() {
            "non-specific".to_string()
        } else {
            let mut pairs: Vec<String> = self.current.attention_profile.iter()
                .map(|(k, v)| format!("{}:{:.2}", k, v))
                .collect();
            pairs.sort();
            pairs.join(", ")
        };

        AwarenessReport {
            timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            consciousness: self.current.consciousness_level,
            coherence: self.current.coherence_current,
            phi: self.current.phi_current,
            attention_state: attention_str,
            blind_spots,
            conversation_turns: self.current.conversation_awareness.turn_count,
            conversation_stage: self.current.conversation_awareness.stage.label().to_string(),
            topic_coherence: self.current.conversation_awareness.topic_coherence,
            health: self.current.health,
            trends: trend,
        }
    }

    /// Set active blind spots from cognitive observer
    pub fn set_blind_spots(&mut self, spots: Vec<BlindSpotSummary>) {
        self.current.active_blind_spots = spots;
    }

    // ─── Private helpers ───

    /// IIT Φ 输入状态 ≈ 64 维"意识谱"。
    ///
    /// 运行时不可直接喂 7 个杂项标量 + 尾部零填充 (零在 centered 后整体变负，
    /// 打散相邻一致性 rho 且令 intensity 无结构) —— 那是导致生产 φ 长期偏低、
    /// 看似"0.5 不可达"的根因。本构造用真实输入标量做锚点，在 64 维上线性
    /// 插值成**平滑且差异化**的连续谱：
    ///   - 平滑 → 相邻一致性 rho 高 (produit 平滑相关)
    ///   - 差异化 → 去均值后仍有强度，intensity 高
    /// 这正是 compute_phi 需要的高集成形态，不再受零填充拖累。
    fn current_phi_state(&self) -> Vec<f64> {
        // 真实输入锚点 (排除 phi_current：它是自身输出，回环喂入会自激/失真)。
        // 保持固定顺序以便相邻语义相邻。
        let anchors: Vec<f64> = vec![
            self.current.consciousness_level,
            self.current.coherence_current,
            self.current.health,
            self.current.conversation_awareness.topic_coherence,
            self.current.conversation_awareness.user_engagement,
            self.current.conversation_awareness.self_assessed_quality,
            self.current.strategy_effectiveness,
        ];
        if anchors.is_empty() {
            return Vec::with_capacity(64);
        }
        let dims = 64usize;
        let win = anchors.len().min(dims);
        // 每段锚点间距内的步长
        let step = if win > 1 { (win as f64 - 1.0) / (dims as f64) } else { 0.0 };
        let mut state = Vec::with_capacity(dims);
        for i in 0..dims {
            // 浮点位置 → 相邻两个锚点线性插值
            let pos = i as f64 * step;
            let lo = (pos.floor() as usize).min(win - 1);
            let hi = (pos.ceil() as usize).min(win - 1);
            let frac = pos - pos.floor();
            let v = if lo == hi {
                anchors[lo]
            } else {
                anchors[lo] + (anchors[hi] - anchors[lo]) * frac
            };
            state.push(v);
        }
        state
    }

    fn compute_coherence(&self) -> f64 {
        let ca = &self.current.conversation_awareness;
        if ca.turn_count < 2 {
            return 0.1;
        }
        let topic_stability = ca.topic_coherence;
        let engagement_factor = ca.user_engagement;
        let quality_factor = ca.self_assessed_quality;

        0.4 * topic_stability + 0.3 * engagement_factor + 0.3 * quality_factor
    }

    fn compute_health(&self) -> f64 {
        let base_health = self.current.health;
        let blind_spot_penalty = (self.current.active_blind_spots.len() as f64) * 0.05;
        let drift_penalty = self.current.conversation_awareness.topic_drift * 0.1;

        (base_health - blind_spot_penalty - drift_penalty).clamp(0.0, 1.0)
    }

    fn compute_attention_entropy(&self) -> f64 {
        let profile = &self.current.attention_profile;
        if profile.is_empty() {
            return 0.0;
        }
        let total: f64 = profile.values().sum();
        if total <= 0.0 {
            return 0.0;
        }
        let entropy: f64 = profile.values()
            .filter(|&&v| v > 0.0)
            .map(|&v| {
                let p = v / total;
                -p * p.log2()
            })
            .sum();
        entropy / ENTROPY_BASELINE
    }

    fn estimate_conversation_stage(&self, turn_count: usize, topic_drift: f64) -> ConversationStage {
        match turn_count {
            0..=2 => ConversationStage::Opening,
            3..=6 if topic_drift < 0.7 => ConversationStage::Exploration,
            7..=15 if topic_drift < 0.4 => ConversationStage::Deepening,
            _ if topic_drift < 0.25 => ConversationStage::Resolution,
            _ => ConversationStage::Closing,
        }
    }

    /// Simple keyword overlap between two strings
    fn keyword_overlap(&self, a: &str, b: &str) -> f64 {
        let words_a: std::collections::HashSet<&str> = a.split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();
        let words_b: std::collections::HashSet<&str> = b.split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();
        if words_a.is_empty() || words_b.is_empty() {
            return 0.5;
        }
        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();
        intersection as f64 / union as f64
    }

    /// Linear regression slope of a sequence
    fn slope(&self, data: &[f64]) -> f64 {
        let n = data.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let sum_x: f64 = (0..data.len()).map(|i| i as f64).sum();
        let sum_y: f64 = data.iter().sum();
        let sum_xy: f64 = data.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_xx: f64 = (0..data.len()).map(|i| (i as f64) * (i as f64)).sum();
        let denom = n * sum_xx - sum_x * sum_x;
        if denom.abs() < 1e-12 {
            return 0.0;
        }
        (n * sum_xy - sum_x * sum_y) / denom
    }
}

impl Default for ConsciousnessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessMonitor {
    fn name(&self) -> &str {
        "consciousness_monitor"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut cm = ConsciousnessMonitor::new();
        cm.observe();
        let report = cm.get_report();
        if !(0.0..=1.0).contains(&report.consciousness) {
            failures.push("consciousness out of [0,1] range".into());
        }
        if !(0.0..=1.0).contains(&report.coherence) {
            failures.push("coherence out of [0,1] range".into());
        }
        if report.phi < 0.0 {
            failures.push("phi is negative".into());
        }
        if report.timestamp.is_empty() {
            failures.push("empty timestamp".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_monitor_new() {
        let cm = ConsciousnessMonitor::new();
        assert!((cm.current.consciousness_level - 0.0).abs() < 1e-9);
        assert!(cm.capability_monitor.is_some());
        assert!(cm.awareness_history.is_empty());
        assert_eq!(cm.trends.phi_trend.len(), 0);
    }

    #[test]
    fn test_observe_cycle() {
        let mut cm = ConsciousnessMonitor::new();
        cm.observe_conversation_turn("test topic", 500, 0.3);

        cm.observe();

        assert!(cm.current.consciousness_level >= 0.0 && cm.current.consciousness_level <= 1.0);
        assert!(cm.current.phi_current >= 0.0 && cm.current.phi_current <= 1.0);
        assert_eq!(cm.awareness_history.len(), 1);
        assert_eq!(cm.trends.phi_trend.len(), 1);
        assert_eq!(cm.trends.coherence_trend.len(), 1);
        assert_eq!(cm.trends.health_trend.len(), 1);
    }

    #[test]
    fn test_conversation_turn_tracking() {
        let mut cm = ConsciousnessMonitor::new();

        cm.observe_conversation_turn("hello world first message", 200, 0.3);
        assert_eq!(cm.current.conversation_awareness.turn_count, 1);
        assert_eq!(cm.current.conversation_awareness.stage, ConversationStage::Opening);

        cm.observe_conversation_turn("hello world second message exploring more", 800, 0.5);
        assert_eq!(cm.current.conversation_awareness.turn_count, 2);
        assert_eq!(cm.current.conversation_awareness.stage, ConversationStage::Opening);

        cm.observe_conversation_turn("exploring deeper concepts hello world", 1200, 0.7);
        assert_eq!(cm.current.conversation_awareness.turn_count, 3);
        assert_eq!(cm.current.conversation_awareness.stage, ConversationStage::Exploration);
    }

    #[test]
    fn test_compute_consciousness_level() {
        let mut cm = ConsciousnessMonitor::new();
        cm.current.phi_current = 0.8;
        cm.current.coherence_current = 0.7;
        cm.current.health = 0.9;

        let level = cm.compute_consciousness_level();
        let expected = 0.4 * 0.8 + 0.35 * 0.7 + 0.25 * 0.9;
        assert!((level - expected).abs() < 1e-6);
        assert!(level >= 0.0 && level <= 1.0);
    }

    #[test]
    fn test_trend_analysis_improving() {
        let mut cm = ConsciousnessMonitor::new();
        cm.trends.phi_trend = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        cm.trends.coherence_trend = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        cm.trends.health_trend = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        assert_eq!(cm.trend_analysis(), "improving");
    }

    #[test]
    fn test_trend_analysis_declining() {
        let mut cm = ConsciousnessMonitor::new();
        cm.trends.phi_trend = vec![0.5, 0.4, 0.3, 0.2, 0.1];
        cm.trends.coherence_trend = vec![0.5, 0.4, 0.3, 0.2, 0.1];
        cm.trends.health_trend = vec![0.5, 0.4, 0.3, 0.2, 0.1];

        assert_eq!(cm.trend_analysis(), "declining");
    }

    #[test]
    fn test_trend_analysis_stable() {
        let mut cm = ConsciousnessMonitor::new();
        cm.trends.phi_trend = vec![0.5, 0.51, 0.49, 0.5, 0.51];
        cm.trends.coherence_trend = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        cm.trends.health_trend = vec![0.5, 0.5, 0.5, 0.5, 0.5];

        assert_eq!(cm.trend_analysis(), "stable");
    }

    #[test]
    fn test_trend_analysis_insufficient_data() {
        let cm = ConsciousnessMonitor::new();
        assert_eq!(cm.trend_analysis(), "insufficient_data");
    }

    #[test]
    fn test_awareness_snapshot_history() {
        let mut cm = ConsciousnessMonitor::new();
        for _ in 0..5 {
            cm.observe();
        }
        assert_eq!(cm.awareness_history.len(), 5);
        assert_eq!(cm.trends.phi_trend.len(), 5);

        // Verify ordering
        for i in 1..5 {
            assert!(cm.awareness_history[i].conversation_turn >= cm.awareness_history[i - 1].conversation_turn);
        }
    }

    #[test]
    fn test_conversation_awareness_topic_drift() {
        let mut cm = ConsciousnessMonitor::new();

        cm.observe_conversation_turn("let us discuss functional programming in Scala", 500, 0.4);
        let drift1 = cm.current.conversation_awareness.topic_drift;
        // First turn: no previous topic, drift should be 0
        assert!((drift1 - 0.0).abs() < 1e-9);

        cm.observe_conversation_turn("comparing Scala monads with Rust Result types", 800, 0.6);
        let drift2 = cm.current.conversation_awareness.topic_drift;
        // Some overlap between topics
        assert!(drift2 >= 0.0 && drift2 <= 1.0);

        // Dramatic topic shift
        cm.observe_conversation_turn("building a web frontend with React components", 600, 0.5);
        let drift3 = cm.current.conversation_awareness.topic_drift;
        // High drift expected between programming and frontend
        assert!(drift3 >= 0.0 && drift3 <= 1.0);
    }

    #[test]
    fn test_blind_spot_aggregation() {
        let mut cm = ConsciousnessMonitor::new();
        let spots = vec![
            BlindSpotSummary::new("strategy_fixation", 2, "Over-reliance on Direct strategy", "Boost alternatives"),
            BlindSpotSummary::new("context_overload", 3, "Context at 92% capacity", "Trigger consolidation"),
        ];

        cm.set_blind_spots(spots);
        assert_eq!(cm.current.active_blind_spots.len(), 2);
        assert_eq!(cm.current.active_blind_spots[0].kind, "strategy_fixation");
        assert_eq!(cm.current.active_blind_spots[1].severity, 3);
    }

    #[test]
    fn test_consciousness_level_bounds() {
        let mut cm = ConsciousnessMonitor::new();

        cm.current.phi_current = 2.0;
        cm.current.coherence_current = 1.5;
        cm.current.health = -0.5;

        let level = cm.compute_consciousness_level();
        assert!(level >= 0.0 && level <= 1.0,
            "Consciousness level {} must be in [0,1]", level);

        cm.current.phi_current = 0.0;
        cm.current.coherence_current = 0.0;
        cm.current.health = 0.0;
        let level_zero = cm.compute_consciousness_level();
        assert!((level_zero - 0.0).abs() < 1e-9);

        cm.current.phi_current = 1.0;
        cm.current.coherence_current = 1.0;
        cm.current.health = 1.0;
        let level_one = cm.compute_consciousness_level();
        assert!((level_one - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_production_phi_state_breaks_05_ceiling() {
        // 生产路径：真实标量 → 插值成平滑意识谱，φ 应能突破 0.5 (HighlyConscious 可达)。
        let mut cm = ConsciousnessMonitor::new();
        cm.current.consciousness_level = 0.9;
        cm.current.coherence_current = 0.9;
        cm.current.health = 0.8;
        cm.current.conversation_awareness.topic_coherence = 0.8;
        cm.current.conversation_awareness.user_engagement = 0.7;
        cm.current.conversation_awareness.self_assessed_quality = 0.8;
        cm.current.strategy_effectiveness = 0.8;

        // observe() 内部即走 current_phi_state() + compute_phi()
        cm.observe();
        let phi = cm.current.phi_current;
        let rep = cm.last_phi_report.as_ref().expect("report set");
        assert!(phi > 0.5,
            "refactored state spectrum must exceed 0.5 HIGH_PHI (was {}); smooth interpolation gives coherent integrated signal",
            phi);
        assert!(rep.effective_dims > 7,
            "interpolated spectrum must engage far more than the {} raw dims", 7);
    }

    #[test]
    fn test_production_state_highly_coherent_by_observe() {
        let mut cm = ConsciousnessMonitor::new();
        cm.current.consciousness_level = 0.85;
        cm.current.coherence_current = 0.85;
        cm.current.health = 0.8;
        cm.current.conversation_awareness.topic_coherence = 0.8;
        cm.current.conversation_awareness.user_engagement = 0.6;
        cm.current.conversation_awareness.self_assessed_quality = 0.75;
        cm.current.strategy_effectiveness = 0.8;
        cm.observe();
        let phi = cm.current.phi_current;
        assert!(phi.is_finite() && phi >= 0.0);
        assert!(phi <= 1.0);
    }

    #[test]
    fn test_get_report() {
        let mut cm = ConsciousnessMonitor::new();
        cm.observe_conversation_turn("test topic", 1000, 0.5);
        cm.observe();

        let report = cm.get_report();
        assert!(!report.timestamp.is_empty());
        assert!(report.consciousness >= 0.0 && report.consciousness <= 1.0);
        assert!(report.coherence >= 0.0 && report.coherence <= 1.0);
        assert_eq!(report.conversation_turns, 1);
        assert!(!report.conversation_stage.is_empty());
        assert!(report.health >= 0.0 && report.health <= 1.0);
    }

    #[test]
    fn test_introspect_format() {
        let mut cm = ConsciousnessMonitor::new();
        cm.observe_conversation_turn("hello", 100, 0.3);
        cm.observe();

        let intro = cm.introspect();
        assert!(intro.contains("Consciousness:"));
        assert!(intro.contains("Phi:"));
        assert!(intro.contains("Stage:"));
        assert!(intro.contains("Trend:"));
    }
}
