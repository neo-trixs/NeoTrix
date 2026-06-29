use crate::core::nt_core_self::{
    CognitiveEvaluator, CrystalRegistry, IntrinsicMotivation, SelfReferentialMonitor,
    SiliconArchive,
};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    Green,
    Yellow,
    Red,
}

impl AlertLevel {
    pub fn label(&self) -> &str {
        match self {
            AlertLevel::Green => "🟢 Green",
            AlertLevel::Yellow => "🟡 Yellow",
            AlertLevel::Red => "🔴 Red",
        }
    }

    pub fn needs_intervention(&self) -> bool {
        matches!(self, AlertLevel::Red)
    }
}

#[derive(Debug, Clone)]
pub struct BMonitorReport {
    pub health_score: f64,
    pub alert_level: AlertLevel,
    pub component_scores: ComponentScores,
    pub flags: Vec<String>,
    pub needs_intervention: bool,
    pub report_id: usize,
    pub iteration: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentScores {
    pub cognitive_health: f64,
    pub motivation_health: f64,
    pub plan_quality: f64,
    pub archive_depth: f64,
    pub skill_richness: f64,
}

impl ComponentScores {
    pub fn breakdown(&self) -> Vec<(&str, f64)> {
        vec![
            ("cognitive", self.cognitive_health),
            ("motivation", self.motivation_health),
            ("plan", self.plan_quality),
            ("archive", self.archive_depth),
            ("skills", self.skill_richness),
        ]
    }
}

pub struct BMonitor {
    pub report_count: usize,
    pub history: VecDeque<BMonitorReport>,
    pub max_history: usize,
}

impl Default for BMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl BMonitor {
    pub fn new() -> Self {
        Self {
            report_count: 0,
            history: VecDeque::new(),
            max_history: 20,
        }
    }

    pub fn evaluate(
        &mut self,
        evaluator: &CognitiveEvaluator,
        motivation: &IntrinsicMotivation,
        monitor: &SelfReferentialMonitor,
        archive: &SiliconArchive,
        skills: &CrystalRegistry,
        iteration: usize,
    ) -> BMonitorReport {
        let cognitive_health = self.score_cognitive(evaluator);
        let motivation_health = self.score_motivation(motivation);
        let plan_quality = self.score_plan_quality(monitor);
        let archive_depth = self.score_archive(archive);
        let skill_richness = self.score_skills(skills);

        let component_scores = ComponentScores {
            cognitive_health,
            motivation_health,
            plan_quality,
            archive_depth,
            skill_richness,
        };

        let health_score = cognitive_health * 0.30
            + motivation_health * 0.25
            + plan_quality * 0.20
            + archive_depth * 0.10
            + skill_richness * 0.15;

        let health_score = health_score.clamp(0.0, 100.0);
        let alert_level = if health_score >= 70.0 {
            AlertLevel::Green
        } else if health_score >= 40.0 {
            AlertLevel::Yellow
        } else {
            AlertLevel::Red
        };

        let mut flags: Vec<String> = Vec::new();
        if cognitive_health < 40.0 {
            flags.push(format!("cognitive health low: {:.1}", cognitive_health));
        }
        if motivation_health < 40.0 {
            flags.push(format!("motivation health low: {:.1}", motivation_health));
        }
        if plan_quality < 30.0 {
            flags.push(format!("plan quality degraded: {:.1}", plan_quality));
        }
        if monitor.needs_intervention() {
            flags.push("self-referential intervention needed".into());
        }

        let report = BMonitorReport {
            health_score,
            alert_level,
            component_scores,
            flags,
            needs_intervention: alert_level.needs_intervention(),
            report_id: self.report_count,
            iteration,
        };

        self.report_count += 1;
        self.history.push_back(report.clone());
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        report
    }

    fn score_cognitive(&self, evaluator: &CognitiveEvaluator) -> f64 {
        let report = match evaluator.latest_report() {
            Some(r) => r,
            None => return 50.0,
        };
        let raw = report.stability_score * 100.0;
        let penalty = report
            .flags
            .iter()
            .filter(|f| {
                matches!(
                    f.severity,
                    crate::core::nt_core_self::FlagSeverity::Critical
                )
            })
            .count() as f64
            * 15.0;
        (raw - penalty).clamp(0.0, 100.0)
    }

    fn score_motivation(&self, motivation: &IntrinsicMotivation) -> f64 {
        if motivation.reward_history.is_empty() {
            return 50.0;
        }
        let avg = motivation.avg_reward(5);
        if avg <= 0.0 {
            return 100.0;
        }
        let raw = (1.0 - avg) * 100.0;
        raw.clamp(0.0, 100.0)
    }

    fn score_plan_quality(&self, monitor: &SelfReferentialMonitor) -> f64 {
        if monitor.plan_history.is_empty() {
            return 50.0;
        }
        let recent: Vec<f64> = monitor
            .plan_history
            .iter()
            .rev()
            .take(5)
            .map(|r| r.execution_quality)
            .collect();
        if recent.is_empty() {
            return 50.0;
        }
        let avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let trend = monitor.plan_quality_trend();
        let base = avg * 100.0;
        let trend_penalty = if trend < -0.05 {
            (trend.abs() * 100.0).min(30.0)
        } else {
            0.0
        };
        (base - trend_penalty).clamp(0.0, 100.0)
    }

    fn score_archive(&self, archive: &SiliconArchive) -> f64 {
        let count = archive.snapshots.len();
        if count == 0 {
            return 20.0;
        }
        let max_useful = 20.0_f64;
        let raw = (count as f64 / max_useful * 100.0).min(100.0);
        raw.clamp(0.0, 100.0)
    }

    fn score_skills(&self, skills: &CrystalRegistry) -> f64 {
        if skills.crystals.is_empty() {
            return 15.0;
        }
        let count_score = (skills.crystals.len() as f64 / 20.0 * 100.0).min(100.0) * 0.4;
        let best_eff = skills
            .crystals
            .iter()
            .map(|c| c.effectiveness)
            .fold(0.0_f64, f64::max);
        let eff_score = best_eff * 100.0 * 0.4;
        let total_use = skills.crystals.iter().map(|c| c.use_count).sum::<usize>();
        let use_score = (total_use as f64 / 50.0 * 100.0).min(100.0) * 0.2;
        (count_score + eff_score + use_score).clamp(0.0, 100.0)
    }

    pub fn latest_report(&self) -> Option<&BMonitorReport> {
        self.history.back()
    }

    pub fn health_trend(&self) -> f64 {
        let n = self.history.len();
        if n < 2 {
            return 0.0;
        }
        let recent: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(5)
            .map(|r| r.health_score)
            .collect();
        if recent.len() < 2 {
            return 0.0;
        }
        let first = recent.last().expect("recent.len() >= 2 per guard");
        let last = recent.first().expect("recent.len() >= 2 per guard");
        last - first
    }

    pub fn dashboard(&self) -> String {
        match self.latest_report() {
            None => "B-Brain Monitor: no report yet".to_string(),
            Some(report) => {
                let comp: Vec<String> = report
                    .component_scores
                    .breakdown()
                    .iter()
                    .map(|(name, score)| format!("{}={:.1}", name, score))
                    .collect();
                let flags = if report.flags.is_empty() {
                    "no flags".to_string()
                } else {
                    report.flags.join("; ")
                };
                format!(
                    "B-Brain #{} [{}] score={:.1} | {} | trend={:+.1} | iter={} | {}",
                    report.report_id,
                    report.alert_level.label(),
                    report.health_score,
                    comp.join(" "),
                    self.health_trend(),
                    report.iteration,
                    flags,
                )
            }
        }
    }

    pub fn summary_for_repl(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        lines.push("╭─ B-Brain Monitor ─────────────────────────────╮".into());
        match self.latest_report() {
            None => {
                lines.push("│  No report available yet                       │".into());
            }
            Some(report) => {
                let alert = report.alert_level.label();
                lines.push(format!(
                    "│  Health Score: {:>5.1}/100  [{}]              │",
                    report.health_score, alert
                ));
                lines.push(format!(
                    "│  Report #{} | Iteration #{}                    │",
                    report.report_id, report.iteration
                ));
                lines.push("│  Components:                                    │".into());
                for (name, score) in report.component_scores.breakdown() {
                    let bar = Self::bar(score, 20);
                    lines.push(format!("│    {:>10}: {:>5.1} {} │", name, score, bar));
                }
                let trend = self.health_trend();
                let trend_sign = if trend > 0.0 { "+" } else { "" };
                lines.push(format!(
                    "│  Trend: {}{:.1} pts                              │",
                    trend_sign, trend
                ));
                if !report.flags.is_empty() {
                    lines.push("│  ⚠ Flags:                                       │".into());
                    for flag in &report.flags {
                        lines.push(format!("│    • {}                  │", flag));
                    }
                }
            }
        }
        lines.push("╰──────────────────────────────────────────────╯".into());
        lines
    }

    fn bar(value: f64, width: usize) -> String {
        let filled = ((value / 100.0) * width as f64).round() as usize;
        let filled = filled.min(width);
        let empty = width.saturating_sub(filled);
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self::{
        AttentionDomain, CognitiveEvaluator, CrystalRegistry, IntrinsicMotivation,
        SelfReferentialMonitor, SiliconArchive, SiliconSelfModel, StrategyKind,
    };

    fn make_evaluator_with_stability(stability: f64) -> CognitiveEvaluator {
        let model = SiliconSelfModel::new();
        let mut eval = CognitiveEvaluator::new();
        for _ in 0..3 {
            eval.evaluate(&model);
        }
        if let Some(report) = eval.history.last_mut() {
            report.stability_score = stability;
            report.attention_health = stability;
            report.strategy_diversity = stability * 0.8;
            report.trace_quality = stability * 0.9;
            report.flags.clear();
        }
        eval
    }

    fn make_motivation_with_reward(rewards: &[f64]) -> IntrinsicMotivation {
        let mut m = IntrinsicMotivation::new();
        for r in rewards {
            m.last_reward = *r;
            m.reward_history.push(*r);
        }
        m
    }

    fn make_monitor_with_plans(qualities: &[f64]) -> SelfReferentialMonitor {
        let mut m = SelfReferentialMonitor::new();
        for (i, &q) in qualities.iter().enumerate() {
            m.record_plan(&format!("plan_{}", i), q);
        }
        m
    }

    fn make_archive_with_count(n: usize) -> SiliconArchive {
        let model = SiliconSelfModel::new();
        let mut a = SiliconArchive::with_max(50);
        for i in 0..n {
            a.snapshot(&format!("snap_{}", i), &model);
        }
        a
    }

    fn make_skills_with_count(n: usize) -> CrystalRegistry {
        let mut reg = CrystalRegistry::new();
        for i in 0..n {
            reg.crystals
                .push(crate::core::nt_core_self::SkillCrystal::new(
                    i,
                    &format!("skill_{}", i),
                    "test",
                    StrategyKind::Direct,
                    AttentionDomain::Code,
                    1,
                ));
        }
        if n > 0 {
            reg.crystals[0].effectiveness = 0.9;
        }
        reg
    }

    #[test]
    fn test_bmonitor_new() {
        let bm = BMonitor::new();
        assert_eq!(bm.report_count, 0);
        assert!(bm.history.is_empty());
        assert_eq!(bm.max_history, 20);
        assert!(bm.latest_report().is_none());
    }

    #[test]
    fn test_evaluate_green() {
        let mut bm = BMonitor::new();
        let eval = make_evaluator_with_stability(0.9);
        let mot = make_motivation_with_reward(&[0.2, 0.25, 0.3]);
        let mon = make_monitor_with_plans(&[0.8, 0.85, 0.9]);
        let archive = make_archive_with_count(15);
        let skills = make_skills_with_count(10);

        let report = bm.evaluate(&eval, &mot, &mon, &archive, &skills, 1);
        assert_eq!(report.alert_level, AlertLevel::Green);
        assert!(report.health_score >= 70.0);
        assert!(!report.needs_intervention);
        assert!(report.flags.is_empty());
        assert_eq!(report.report_id, 0);
    }

    #[test]
    fn test_evaluate_red() {
        let mut bm = BMonitor::new();
        let eval = make_evaluator_with_stability(0.1);
        let mut mot = IntrinsicMotivation::new();
        mot.last_reward = 0.9;
        mot.reward_history.push(0.9);
        let mon = SelfReferentialMonitor::new();
        let archive = SiliconArchive::new();
        let skills = CrystalRegistry::new();

        let report = bm.evaluate(&eval, &mot, &mon, &archive, &skills, 1);
        assert_eq!(report.alert_level, AlertLevel::Red);
        assert!(report.health_score < 40.0);
        assert!(report.needs_intervention);
        assert!(!report.flags.is_empty());
    }

    #[test]
    fn test_evaluate_yellow() {
        let mut bm = BMonitor::new();
        let eval = make_evaluator_with_stability(0.5);
        let mot = make_motivation_with_reward(&[0.5, 0.55, 0.6]);
        let mon = make_monitor_with_plans(&[0.5, 0.45, 0.4]);
        let archive = make_archive_with_count(3);
        let skills = make_skills_with_count(2);

        let report = bm.evaluate(&eval, &mot, &mon, &archive, &skills, 1);
        assert_eq!(report.alert_level, AlertLevel::Yellow);
        assert!(report.health_score >= 40.0 && report.health_score < 70.0);
    }

    #[test]
    fn test_health_trend_positive() {
        let mut bm = BMonitor::new();
        let _eval = make_evaluator_with_stability(0.7);
        let mot = make_motivation_with_reward(&[0.3, 0.35, 0.4]);
        let mon = make_monitor_with_plans(&[0.7, 0.75]);
        let archive = make_archive_with_count(10);
        let skills = make_skills_with_count(5);

        for i in 0..3 {
            let e = make_evaluator_with_stability(0.5 + i as f64 * 0.2);
            bm.evaluate(&e, &mot, &mon, &archive, &skills, i);
        }
        let trend = bm.health_trend();
        assert!(trend >= 0.0 || (trend).abs() < 0.1);
    }

    #[test]
    fn test_dashboard_format() {
        let mut bm = BMonitor::new();
        let eval = make_evaluator_with_stability(0.8);
        let mot = make_motivation_with_reward(&[0.2, 0.3]);
        let mon = make_monitor_with_plans(&[0.8, 0.85, 0.9]);
        let archive = make_archive_with_count(12);
        let skills = make_skills_with_count(8);

        bm.evaluate(&eval, &mot, &mon, &archive, &skills, 1);
        let dash = bm.dashboard();
        assert!(dash.contains("B-Brain"));
        assert!(dash.contains("score="));
    }

    #[test]
    fn test_summary_for_repl() {
        let mut bm = BMonitor::new();
        let eval = make_evaluator_with_stability(0.6);
        let mot = make_motivation_with_reward(&[0.4, 0.45]);
        let mon = make_monitor_with_plans(&[0.6]);
        let archive = make_archive_with_count(5);
        let skills = make_skills_with_count(3);

        bm.evaluate(&eval, &mot, &mon, &archive, &skills, 1);
        let lines = bm.summary_for_repl();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("B-Brain Monitor"));
    }

    #[test]
    fn test_alert_level_order() {
        assert!(AlertLevel::Red > AlertLevel::Yellow);
        assert!(AlertLevel::Yellow > AlertLevel::Green);
        assert!(AlertLevel::Red > AlertLevel::Green);
    }

    #[test]
    fn test_needs_intervention_only_red() {
        assert!(AlertLevel::Red.needs_intervention());
        assert!(!AlertLevel::Yellow.needs_intervention());
        assert!(!AlertLevel::Green.needs_intervention());
    }

    #[test]
    fn test_empty_archive_scores_low() {
        let mut bm = BMonitor::new();
        let eval = make_evaluator_with_stability(0.5);
        let mot = make_motivation_with_reward(&[0.3]);
        let mon = make_monitor_with_plans(&[0.5]);
        let archive = SiliconArchive::new();
        let skills = CrystalRegistry::new();

        let report = bm.evaluate(&eval, &mot, &mon, &archive, &skills, 1);
        assert!(report.component_scores.archive_depth < 30.0);
        assert!(report.component_scores.skill_richness < 30.0);
    }

    #[test]
    fn test_max_history_enforced() {
        let mut bm = BMonitor::new();
        bm.max_history = 3;
        let eval = make_evaluator_with_stability(0.5);
        let mot = make_motivation_with_reward(&[0.3]);
        let mon = make_monitor_with_plans(&[0.5]);
        let archive = make_archive_with_count(5);
        let skills = make_skills_with_count(3);

        for i in 0..5 {
            bm.evaluate(&eval, &mot, &mon, &archive, &skills, i);
        }
        assert_eq!(bm.history.len(), 3);
    }
}
