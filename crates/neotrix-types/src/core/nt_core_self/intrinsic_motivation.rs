use super::silicon_self::SiliconSelfModel;
use super::thinking_trace::ReflectionGrade;
use super::attention_head::AttentionDomain;
use super::reasoning_strategy::StrategyKind;

pub struct MotivationState {
    pub intrinsic_reward: f64,
    pub confidence: f64,
    pub error_rate: f64,
    pub novelty_score: f64,
    pub should_explore: bool,
    pub suggested_domains: Vec<AttentionDomain>,
    pub suggested_strategies: Vec<StrategyKind>,
}

pub struct IntrinsicMotivation {
    pub window_size: usize,
    pub weight_confidence: f64,
    pub weight_error: f64,
    pub weight_novelty: f64,
    pub exploration_threshold: f64,
    pub last_reward: f64,
    pub reward_history: Vec<f64>,
}

impl Default for IntrinsicMotivation {
    fn default() -> Self {
        Self::new()
    }
}

impl IntrinsicMotivation {
    pub fn new() -> Self {
        Self {
            window_size: 10,
            weight_confidence: 0.4,
            weight_error: 0.3,
            weight_novelty: 0.3,
            exploration_threshold: 0.5,
            last_reward: 0.0,
            reward_history: Vec::new(),
        }
    }

    pub fn compute(&mut self, model: &SiliconSelfModel) -> MotivationState {
        let traces = model.recent_traces(self.window_size);
        let window_size = traces.len();
        if window_size == 0 {
            let state = MotivationState {
                intrinsic_reward: 0.5,
                confidence: 0.0,
                error_rate: 0.0,
                novelty_score: 0.0,
                should_explore: true,
                suggested_domains: Vec::new(),
                suggested_strategies: Vec::new(),
            };
            self.last_reward = 0.5;
            self.reward_history.push(0.5);
            return state;
        }

        let confidence: f64 = traces.iter().map(|t| t.grade.score()).sum::<f64>() / window_size as f64;

        let error_count = traces.iter()
            .filter(|t| matches!(t.grade, ReflectionGrade::Poor | ReflectionGrade::Failed))
            .count();
        let error_rate = error_count as f64 / window_size as f64;

        let all_traces = model.recent_traces(model.thinking_traces.len());
        let older_strategies: Vec<StrategyKind> = {
            let mut s: Vec<StrategyKind> = all_traces.iter().skip(window_size)
                .flat_map(|t| t.strategies_used())
                .collect();
            s.sort_by_key(|k| *k as u8);
            s.dedup();
            s
        };

        let window_strategies: Vec<StrategyKind> = {
            let mut s: Vec<StrategyKind> = traces.iter()
                .flat_map(|t| t.strategies_used())
                .collect();
            s.sort_by_key(|k| *k as u8);
            s.dedup();
            s
        };

        let novelty = if window_strategies.is_empty() || older_strategies.is_empty() {
            0.0
        } else {
            let new_count = window_strategies.iter()
                .filter(|sk| !older_strategies.contains(sk))
                .count();
            new_count as f64 / window_strategies.len() as f64
        };

        let r_int = self.weight_confidence * (1.0 - confidence)
            + self.weight_error * error_rate
            + self.weight_novelty * novelty;

        let should_explore = r_int > self.exploration_threshold || novelty > 0.5;

        let low_threshold = model.attention_manager.global_threshold * 0.5;
        let suggested_domains: Vec<AttentionDomain> = model.attention_manager.heads.iter()
            .filter(|h| h.activation < low_threshold)
            .map(|h| h.domain)
            .collect();

        let min_count = model.strategy_registry.strategies.values()
            .map(|s| s.use_count).min().unwrap_or(0);
        let suggested_strategies: Vec<StrategyKind> = model.strategy_registry.strategies.values()
            .filter(|s| s.use_count == min_count)
            .map(|s| s.kind)
            .collect();

        self.last_reward = r_int;
        self.reward_history.push(r_int);

        MotivationState {
            intrinsic_reward: r_int,
            confidence,
            error_rate,
            novelty_score: novelty,
            should_explore,
            suggested_domains,
            suggested_strategies,
        }
    }

    pub fn avg_reward(&self, n: usize) -> f64 {
        let n = n.min(self.reward_history.len());
        if n == 0 {
            return 0.0;
        }
        self.reward_history.iter().rev().take(n).sum::<f64>() / n as f64
    }

    pub fn motivation_trend(&self) -> f64 {
        self.last_reward - self.avg_reward(5)
    }

    pub fn summary(&self) -> String {
        format!(
            "IntrinsicMotivation | R_last={:.4} | history={} | window={} | trend={:.4}",
            self.last_reward,
            self.reward_history.len(),
            self.window_size,
            self.motivation_trend(),
        )
    }

    pub fn reset_window(&mut self) {
        self.reward_history.clear();
        self.last_reward = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::silicon_self::SiliconSelfModel;
    use super::super::thinking_trace::{ThinkingTrace, ThinkingStep, ReflectionGrade};
    use super::super::reasoning_strategy::StrategyKind;
    use super::super::attention_head::AttentionDomain;

    fn make_model_with_grades(grades: &[(ReflectionGrade, StrategyKind)]) -> SiliconSelfModel {
        let mut model = SiliconSelfModel::new();
        for (i, (grade, strategy)) in grades.iter().enumerate() {
            let mut trace = ThinkingTrace::new(i, &format!("task{}", i));
            trace.grade = grade.clone();
            let step = ThinkingStep::new(1, "step", *strategy);
            trace.add_step(step);
            model.add_thinking_trace(trace);
        }
        model
    }

    #[test]
    fn test_intrinsic_motivation_new() {
        let im = IntrinsicMotivation::new();
        assert_eq!(im.window_size, 10);
        assert!((im.weight_confidence - 0.4).abs() < 1e-6);
        assert!((im.weight_error - 0.3).abs() < 1e-6);
        assert!((im.weight_novelty - 0.3).abs() < 1e-6);
        assert!((im.exploration_threshold - 0.5).abs() < 1e-6);
        assert!(im.reward_history.is_empty());
    }

    #[test]
    fn test_compute_initial() {
        let mut im = IntrinsicMotivation::new();
        let model = SiliconSelfModel::new();
        let state = im.compute(&model);
        assert!((state.intrinsic_reward - 0.5).abs() < 1e-6);
        assert!(state.should_explore);
    }

    #[test]
    fn test_compute_after_good_traces() {
        let mut im = IntrinsicMotivation::new();
        im.window_size = 3;
        let model = make_model_with_grades(&[
            (ReflectionGrade::Excellent, StrategyKind::Direct),
            (ReflectionGrade::Excellent, StrategyKind::Direct),
            (ReflectionGrade::Excellent, StrategyKind::Direct),
        ]);
        let state = im.compute(&model);
        assert!((state.confidence - 1.0).abs() < 1e-6);
        assert!((state.error_rate - 0.0).abs() < 1e-6);
        assert!(!state.should_explore);
    }

    #[test]
    fn test_compute_after_poor_traces() {
        let mut im = IntrinsicMotivation::new();
        im.window_size = 3;
        let model = make_model_with_grades(&[
            (ReflectionGrade::Poor, StrategyKind::Direct),
            (ReflectionGrade::Failed, StrategyKind::Direct),
            (ReflectionGrade::Poor, StrategyKind::Direct),
        ]);
        let state = im.compute(&model);
        assert!(state.confidence < 0.3);
        assert!(state.error_rate > 0.5);
        assert!(state.intrinsic_reward > 0.3);
    }

    #[test]
    fn test_suggested_domains() {
        let mut model = SiliconSelfModel::new();
        model.attention_manager.stimulate_domain(AttentionDomain::Code, 0.9);
        let mut im = IntrinsicMotivation::new();
        im.window_size = 1;
        let mut trace = ThinkingTrace::new(0, "test");
        trace.grade = ReflectionGrade::Excellent;
        trace.add_step(ThinkingStep::new(1, "step", StrategyKind::Direct));
        model.add_thinking_trace(trace);

        let state = im.compute(&model);
        assert!(state.suggested_domains.iter().any(|d| *d != AttentionDomain::Code));
    }

    #[test]
    fn test_motivation_trend() {
        let mut im = IntrinsicMotivation::new();
        for v in &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6] {
            im.last_reward = *v;
            im.reward_history.push(*v);
        }
        let trend = im.motivation_trend();
        assert!((trend - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_summary_format() {
        let im = IntrinsicMotivation::new();
        let s = im.summary();
        assert!(s.contains("IntrinsicMotivation"));
        assert!(s.contains("R_last="));
        assert!(s.contains("history="));
    }

    #[test]
    fn test_reset_window() {
        let mut im = IntrinsicMotivation::new();
        im.reward_history.push(0.5);
        im.last_reward = 0.5;
        im.reset_window();
        assert!(im.reward_history.is_empty());
        assert!((im.last_reward - 0.0).abs() < 1e-6);
    }
}
