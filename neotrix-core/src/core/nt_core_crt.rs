//! # CRT Multi-Scale Time Model
//!
//! Three cosmological time scales mapped to planning horizons:
//!
//! | Model | Chinese | Scale | Horizon | Numerics |
//! |-------|---------|-------|---------|----------|
//! | 盖天 | Gaitian | Tactical | Seconds-minutes | 80,000 li sky dome, 8-ft gnomon, 24h cycle |
//! | 浑天 | Huntian | Operational | Hours-days | 357,000 li celestial sphere, 365.25° |
//! | 宣夜 | Xuanye | Strategic | Weeks-years | Infinite void, 129,600-year Shao Yong cycle |
//!
//! Each scale inherits from the ancient Chinese cosmological model and
//! maps to a specific planning horizon in the GoalLoop system.

/// The three CRT time scales, ordered from fastest to slowest.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum CrtTimeScale {
    /// 盖天 (Covering Heaven) — tactical: immediate actions, seconds-minutes
    Gaitian,
    /// 浑天 (Spherical Heaven) — operational: planned cycles, hours-days
    Huntian,
    /// 宣夜 (Infinite Night) — strategic: broad vision, weeks-years
    Xuanye,
}

impl CrtTimeScale {
    pub fn all() -> [Self; 3] {
        [Self::Gaitian, Self::Huntian, Self::Xuanye]
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Gaitian => "gaitian",
            Self::Huntian => "huntian",
            Self::Xuanye => "xuanye",
        }
    }

    pub fn chinese_name(&self) -> &str {
        match self {
            Self::Gaitian => "盖天",
            Self::Huntian => "浑天",
            Self::Xuanye => "宣夜",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Gaitian => "Covering Heaven — flat dome, tactical horizon, seconds-minutes",
            Self::Huntian => "Spherical Heaven — celestial sphere, operational horizon, hours-days",
            Self::Xuanye => "Infinite Night — boundless void, strategic horizon, weeks-years",
        }
    }

    /// Base cycle length in seconds.
    pub fn cycle_seconds(&self) -> f64 {
        match self {
            Self::Gaitian => 86_400.0,                     // 1 day
            Self::Huntian => 31_557_600.0,                 // 365.25 days
            Self::Xuanye => 129_600.0 * 365.25 * 86_400.0, // Shao Yong cycle
        }
    }

    /// Recommended re-evaluation interval in seconds.
    pub fn re_eval_interval(&self) -> f64 {
        match self {
            Self::Gaitian => 60.0,    // Every minute
            Self::Huntian => 3_600.0, // Every hour
            Self::Xuanye => 86_400.0, // Every day
        }
    }

    /// Default max iterations per planning cycle.
    pub fn max_iterations(&self) -> u64 {
        match self {
            Self::Gaitian => 100, // Fast, many small iterations
            Self::Huntian => 50,  // Moderate
            Self::Xuanye => 10,   // Slow, deep iterations
        }
    }

    /// Convert a duration in seconds to this scale's "ticks".
    pub fn to_ticks(&self, seconds: f64) -> f64 {
        seconds / self.cycle_seconds()
    }

    /// Convert this scale's ticks back to seconds.
    pub fn from_ticks(&self, ticks: f64) -> f64 {
        ticks * self.cycle_seconds()
    }

    /// Determine the appropriate scale for a given duration in seconds.
    pub fn for_duration(seconds: f64) -> Self {
        if seconds < 3_600.0 {
            // < 1 hour
            Self::Gaitian
        } else if seconds < 604_800.0 {
            // < 1 week
            Self::Huntian
        } else {
            Self::Xuanye
        }
    }

    /// Scale hierarchy: higher scale subsumes lower.
    /// e.g., a Huntian goal can contain multiple Gaitian sub-goals.
    pub fn subsumes(&self, other: &Self) -> bool {
        self >= other
    }

    /// Convert to the corresponding 8×8 strategy matrix quadrant.
    /// Gaitian → Debug/Test/Analyze (analytical, concrete)
    /// Huntian → Design/Generate/Review (generative, mixed)
    /// Xuanye → Prototype/Meta (abstract, strategic)
    pub fn to_hexagram_bias(&self) -> &[u8] {
        match self {
            // Lower 24 hexagrams (0-23): analytical, concrete, focused
            Self::Gaitian => &[0, 1, 2, 3, 4, 5, 6, 7],
            // Middle hexagrams (24-47): balanced, generative
            Self::Huntian => &[24, 25, 26, 27, 28, 29, 30, 31],
            // Upper hexagrams (48-63): abstract, strategic, meta
            Self::Xuanye => &[48, 49, 50, 51, 52, 53, 54, 55],
        }
    }
}

/// A temporal plan spanning all three CRT scales.
#[derive(Debug, Clone)]
pub struct CrtPlan {
    pub scale: CrtTimeScale,
    pub time_budget_seconds: f64,
    pub max_ticks: f64,
    pub sub_plans: Vec<CrtPlan>,
    pub parent_scale: Option<CrtTimeScale>,
}

impl CrtPlan {
    pub fn new(scale: CrtTimeScale, time_budget_seconds: f64) -> Self {
        let max_ticks = scale.to_ticks(time_budget_seconds);
        Self {
            scale,
            time_budget_seconds,
            max_ticks,
            sub_plans: Vec::new(),
            parent_scale: None,
        }
    }

    /// Decompose a Huntian/Xuanye plan into lower-scale sub-plans.
    pub fn decompose(&mut self) {
        if self.scale == CrtTimeScale::Xuanye && self.sub_plans.is_empty() {
            let sub_budget = self.time_budget_seconds / 12.0; // 12 months
            for _ in 0..12 {
                let mut sub = CrtPlan::new(CrtTimeScale::Huntian, sub_budget);
                sub.decompose();
                sub.parent_scale = Some(self.scale);
                self.sub_plans.push(sub);
            }
        } else if self.scale == CrtTimeScale::Huntian && self.sub_plans.is_empty() {
            let sub_budget = self.time_budget_seconds / 30.0; // 30 days
            for _ in 0..30 {
                let sub = CrtPlan::new(CrtTimeScale::Gaitian, sub_budget);
                self.sub_plans.push(sub);
            }
        }
    }

    pub fn total_sub_plans(&self) -> usize {
        let direct = self.sub_plans.len();
        direct
            + self
                .sub_plans
                .iter()
                .map(|s| s.total_sub_plans())
                .sum::<usize>()
    }

    pub fn depth(&self) -> usize {
        1 + self.sub_plans.iter().map(|s| s.depth()).max().unwrap_or(0)
    }
}

/// Multi-scale timeline tracking.
#[derive(Debug, Clone)]
pub struct CrtTimeline {
    pub gaitian_ticks: u64,
    pub huntian_ticks: u64,
    pub xuanye_ticks: u64,
    pub gaitian_cycle_start: f64,
    pub huntian_cycle_start: f64,
    pub xuanye_cycle_start: f64,
}

impl Default for CrtTimeline {
    fn default() -> Self {
        Self::new()
    }
}

impl CrtTimeline {
    pub fn new() -> Self {
        Self {
            gaitian_ticks: 0,
            huntian_ticks: 0,
            xuanye_ticks: 0,
            gaitian_cycle_start: 0.0,
            huntian_cycle_start: 0.0,
            xuanye_cycle_start: 0.0,
        }
    }

    /// Advance all three time scales by elapsed seconds.
    pub fn advance(&mut self, elapsed_seconds: f64) {
        self.gaitian_ticks = (CrtTimeScale::Gaitian
            .to_ticks(elapsed_seconds + self.gaitian_cycle_start)
            .floor() as u64)
            .max(self.gaitian_ticks);
        self.huntian_ticks = (CrtTimeScale::Huntian
            .to_ticks(elapsed_seconds + self.huntian_cycle_start)
            .floor() as u64)
            .max(self.huntian_ticks);
        self.xuanye_ticks = (CrtTimeScale::Xuanye
            .to_ticks(elapsed_seconds + self.xuanye_cycle_start)
            .floor() as u64)
            .max(self.xuanye_ticks);
    }

    /// Check if it's time to re-evaluate at a given scale.
    pub fn should_re_eval(&self, scale: CrtTimeScale, last_eval_seconds: f64) -> bool {
        let interval = scale.re_eval_interval();
        last_eval_seconds >= interval
    }

    /// Get a human-readable timeline summary.
    pub fn summary(&self) -> String {
        format!(
            "盖天:{}t 浑天:{}t 宣夜:{}t",
            self.gaitian_ticks, self.huntian_ticks, self.xuanye_ticks
        )
    }
}

/// A CRT-aware goal descriptor for the GoalLoop.
#[derive(Debug, Clone)]
pub struct CrtGoal {
    pub description: String,
    pub scale: CrtTimeScale,
    pub plan: CrtPlan,
    pub created_tick: u64,
    pub deadline_tick: Option<u64>,
    pub progress: f64,
}

impl CrtGoal {
    pub fn new(description: &str, scale: CrtTimeScale, time_budget_seconds: f64) -> Self {
        let mut plan = CrtPlan::new(scale, time_budget_seconds);
        plan.decompose();
        Self {
            description: description.to_string(),
            scale,
            plan,
            created_tick: 0,
            deadline_tick: None,
            progress: 0.0,
        }
    }

    pub fn time_remaining(&self, current_tick: u64, _scale: CrtTimeScale) -> Option<u64> {
        self.deadline_tick.map(|d| d.saturating_sub(current_tick))
    }

    pub fn is_overdue(&self, current_tick: u64, _scale: CrtTimeScale) -> bool {
        self.deadline_tick.is_some_and(|d| current_tick > d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_ordering() {
        assert!(CrtTimeScale::Gaitian < CrtTimeScale::Huntian);
        assert!(CrtTimeScale::Huntian < CrtTimeScale::Xuanye);
        assert!(CrtTimeScale::Gaitian < CrtTimeScale::Xuanye);
    }

    #[test]
    fn test_cycle_seconds() {
        let one_day = 86_400.0;
        assert!((CrtTimeScale::Gaitian.cycle_seconds() - one_day).abs() < 1.0);
        assert!((CrtTimeScale::Huntian.cycle_seconds() - 365.25 * one_day).abs() < 1.0);
        let xuanye_expected = 129_600.0 * 365.25 * one_day;
        let xuanye_actual = CrtTimeScale::Xuanye.cycle_seconds();
        let rel_error = (xuanye_actual - xuanye_expected).abs() / xuanye_expected;
        assert!(rel_error < 1e-10, "Xuanye cycle rel error: {}", rel_error);
    }

    #[test]
    fn test_for_duration() {
        assert_eq!(CrtTimeScale::for_duration(300.0), CrtTimeScale::Gaitian); // 5 min
        assert_eq!(CrtTimeScale::for_duration(7200.0), CrtTimeScale::Huntian); // 2 hours
        assert_eq!(CrtTimeScale::for_duration(864_000.0), CrtTimeScale::Xuanye);
        // 10 days
    }

    #[test]
    fn test_to_from_ticks() {
        let s = CrtTimeScale::Gaitian;
        let ticks = s.to_ticks(172_800.0); // 2 days
        assert!((ticks - 2.0).abs() < 0.01);
        let back = s.from_ticks(ticks);
        assert!((back - 172_800.0).abs() < 1.0);
    }

    #[test]
    fn test_plan_decompose() {
        let mut plan = CrtPlan::new(CrtTimeScale::Xuanye, 365.25 * 86400.0 * 5.0); // 5 years
        assert_eq!(plan.depth(), 1);
        plan.decompose();
        assert_eq!(plan.depth(), 3);
        assert!(!plan.sub_plans.is_empty());
    }

    #[test]
    fn test_plan_total_sub_plans() {
        let mut plan = CrtPlan::new(CrtTimeScale::Huntian, 30.0 * 86400.0); // 30 days
        plan.decompose();
        assert_eq!(plan.sub_plans.len(), 30);
        // Each Gaitian sub-plan has no further sub-plans
        assert_eq!(plan.total_sub_plans(), 30);
    }

    #[test]
    fn test_timeline_advance() {
        let mut tl = CrtTimeline::new();
        tl.advance(86_400.0); // 1 day
        assert_eq!(tl.gaitian_ticks, 1);
        tl.advance(31_557_600.0); // 1 year
        assert_eq!(tl.huntian_ticks, 1);
    }

    #[test]
    fn test_timeline_summary() {
        let tl = CrtTimeline::new();
        let s = tl.summary();
        assert!(s.contains("盖天"));
        assert!(s.contains("浑天"));
        assert!(s.contains("宣夜"));
    }

    #[test]
    fn test_goal_basic() {
        let goal = CrtGoal::new("Explore deep space", CrtTimeScale::Xuanye, 86400.0 * 365.25);
        assert_eq!(goal.scale, CrtTimeScale::Xuanye);
        assert!((goal.progress - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_goal_overdue() {
        let mut goal = CrtGoal::new("Quick task", CrtTimeScale::Gaitian, 3600.0);
        goal.created_tick = 0;
        goal.deadline_tick = Some(2);
        assert!(!goal.is_overdue(1, CrtTimeScale::Gaitian));
        assert!(goal.is_overdue(3, CrtTimeScale::Gaitian));
    }

    #[test]
    fn test_scale_subsumes() {
        assert!(CrtTimeScale::Xuanye.subsumes(&CrtTimeScale::Huntian));
        assert!(CrtTimeScale::Huntian.subsumes(&CrtTimeScale::Gaitian));
        assert!(!CrtTimeScale::Gaitian.subsumes(&CrtTimeScale::Huntian));
    }

    #[test]
    fn test_re_eval_interval() {
        assert!((CrtTimeScale::Gaitian.re_eval_interval() - 60.0).abs() < 1.0);
        assert!((CrtTimeScale::Huntian.re_eval_interval() - 3600.0).abs() < 1.0);
        assert!((CrtTimeScale::Xuanye.re_eval_interval() - 86400.0).abs() < 1.0);
    }

    #[test]
    fn test_hexagram_bias_disjoint() {
        // Each scale's bias should target different regions
        let g = CrtTimeScale::Gaitian.to_hexagram_bias();
        let h = CrtTimeScale::Huntian.to_hexagram_bias();
        let x = CrtTimeScale::Xuanye.to_hexagram_bias();
        for &gv in g {
            assert!(gv < 24);
        }
        for &hv in h {
            assert!(hv >= 24 && hv < 48);
        }
        for &xv in x {
            assert!(xv >= 48);
        }
    }

    #[test]
    fn test_max_iterations() {
        assert!(CrtTimeScale::Gaitian.max_iterations() > CrtTimeScale::Huntian.max_iterations());
        assert!(CrtTimeScale::Huntian.max_iterations() > CrtTimeScale::Xuanye.max_iterations());
    }
}
