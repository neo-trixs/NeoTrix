use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttentionDomain {
    PatternMatch,
    Code,
    Semantic,
    Temporal,
    Planning,
    SelfReflection,
    ToolUse,
    GoalAlignment,
    RiskAssessment,
    Creativity,
}

impl AttentionDomain {
    pub fn all() -> Vec<AttentionDomain> {
        vec![
            AttentionDomain::PatternMatch,
            AttentionDomain::Code,
            AttentionDomain::Semantic,
            AttentionDomain::Temporal,
            AttentionDomain::Planning,
            AttentionDomain::SelfReflection,
            AttentionDomain::ToolUse,
            AttentionDomain::GoalAlignment,
            AttentionDomain::RiskAssessment,
            AttentionDomain::Creativity,
        ]
    }

    /// P1-13 确定性路由索引 (吸收 PrismSystem skills.json 六字段索引模式):
    /// 关键词 → 域 的确定性映射表, 供任务路由稳定分类 (不依赖 LLM 每次输出漂移)。
    /// PrismSystem 原文: "Router classifier + skills.json six-field index"。
    /// 返回匹配的域 (首个命中) 或 None。
    pub fn from_keywords(task: &str) -> Option<AttentionDomain> {
        let t = task.to_lowercase();
        // 有序: 越具体越靠前 (先匹配精确语义, 再匹配宽泛词)
        const ROUTES: &[(&[&str], AttentionDomain)] = &[
            (&["refactor", "code_review", "code review", "audit", "lint"], AttentionDomain::Code),
            (&["implement", "fix", "bug", "feature", "write code", "build"], AttentionDomain::Code),
            (&["search", "retrieve", "query", "find", "lookup", "explore"], AttentionDomain::Semantic),
            (&["plan", "architect", "design", "roadmap", "strategy"], AttentionDomain::Planning),
            (&["reflect", "review", "retro", "self", "meta"], AttentionDomain::SelfReflection),
            (&["tool", "mcp", "api", "call", "execute", "run"], AttentionDomain::ToolUse),
            (&["risk", "security", "threat", "danger", "guard"], AttentionDomain::RiskAssessment),
            (&["goal", "objective", "align", "priority"], AttentionDomain::GoalAlignment),
            (&["pattern", "match", "similar", "analogy", "reuse"], AttentionDomain::PatternMatch),
            (&["time", "schedule", "deadline", "history", "temporal"], AttentionDomain::Temporal),
            (&["creative", "novel", "generate", "imagine", "brainstorm"], AttentionDomain::Creativity),
        ];
        for (keywords, domain) in ROUTES {
            if keywords.iter().any(|k| t.contains(k)) {
                return Some(*domain);
            }
        }
        None
    }

    pub fn label(&self) -> &str {
        match self {
            AttentionDomain::PatternMatch => "pattern_match",
            AttentionDomain::Code => "code",
            AttentionDomain::Semantic => "semantic",
            AttentionDomain::Temporal => "temporal",
            AttentionDomain::Planning => "planning",
            AttentionDomain::SelfReflection => "self_reflection",
            AttentionDomain::ToolUse => "tool_use",
            AttentionDomain::GoalAlignment => "goal_alignment",
            AttentionDomain::RiskAssessment => "risk_assessment",
            AttentionDomain::Creativity => "creativity",
        }
    }
}

/// 规则强度等级 (来自 ponytail 吸收: R-P81 lazy ladder)
/// lite=探索/只读任务; full=生产修复/实现; ultra=架构重写/重构
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub enum RuleIntensity {
    #[default]
    Lite,
    Full,
    Ultra,
}

impl RuleIntensity {
    pub fn from_task_type(task: &str) -> Self {
        match task {
            t if t.contains("explore") || t.contains("read") || t.contains("search") => RuleIntensity::Lite,
            t if t.contains("implement") || t.contains("fix") || t.contains("refactor") => RuleIntensity::Full,
            t if t.contains("architect") || t.contains("design") || t.contains("rewrite") => RuleIntensity::Ultra,
            _ => RuleIntensity::Full,
        }
    }

    pub fn attention_threshold(&self) -> f64 {
        match self {
            RuleIntensity::Lite => 0.2,
            RuleIntensity::Full => 0.4,
            RuleIntensity::Ultra => 0.6,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            RuleIntensity::Lite => "lite",
            RuleIntensity::Full => "full",
            RuleIntensity::Ultra => "ultra",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttentionHead {
    pub id: usize,
    pub domain: AttentionDomain,
    pub receptive_field: usize,
    pub activation: f64,
    pub specialization: Vec<f64>,
    pub focus: Vec<String>,
    pub decay_rate: f64,
    pub priority: u8,
}

impl AttentionHead {
    pub fn new(id: usize, domain: AttentionDomain) -> Self {
        Self {
            id,
            domain,
            receptive_field: 10,
            activation: 0.0,
            specialization: Vec::new(),
            focus: Vec::new(),
            decay_rate: 0.1,
            priority: 5,
        }
    }

    pub fn salience(&self, novelty: f64, coherence: f64) -> f64 {
        self.activation * novelty * coherence
    }

    pub fn stimulate(&mut self, amount: f64) {
        self.activation = (self.activation + amount).min(1.0);
    }

    pub fn decay(&mut self) {
        self.activation = (self.activation - self.decay_rate).max(0.0);
    }

    pub fn focus_on(&mut self, concept: &str) {
        if !self.focus.contains(&concept.to_string()) {
            self.focus.push(concept.to_string());
        }
        self.stimulate(0.1);
    }

    pub fn is_activated(&self, threshold: f64) -> bool {
        self.activation >= threshold
    }
}

#[derive(Debug, Clone)]
pub struct AttentionProfile {
    pub dominant: AttentionDomain,
    pub distribution: HashMap<AttentionDomain, f64>,
    pub num_activated_heads: usize,
}

impl AttentionProfile {
    pub fn new(dominant: AttentionDomain, distribution: HashMap<AttentionDomain, f64>, num_activated_heads: usize) -> Self {
        Self { dominant, distribution, num_activated_heads }
    }
}

/// Ascendancy Weapon Set — 双专精 (AGENTS.md):
/// 每 session 两个 Weapon Set, 经 AttentionManager 按任务类型路由。
/// Weapon Set I = 获取 (acquisition): CORE+WORLD 域优先
/// Weapon Set II = 进化 (evolution): CORE+MIND 域优先
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WeaponSet {
    /// 获取模式: PatternMatch/Code/Temporal/ToolUse 优先 (采集+执行)
    Acquisition,
    /// 进化模式: Semantic/SelfReflection/Creativity/GoalAlignment 优先 (蒸馏+进化)
    Evolution,
}

impl WeaponSet {
    /// 双专精域映射 — 该专精下获得激活加成的 attention 域
    pub fn priority_domains(&self) -> Vec<AttentionDomain> {
        match self {
            Self::Acquisition => vec![
                AttentionDomain::PatternMatch,
                AttentionDomain::Code,
                AttentionDomain::Temporal,
                AttentionDomain::ToolUse,
            ],
            Self::Evolution => vec![
                AttentionDomain::Semantic,
                AttentionDomain::SelfReflection,
                AttentionDomain::Creativity,
                AttentionDomain::GoalAlignment,
            ],
        }
    }

    /// 从任务类型路由专精 (与 RuleIntensity::from_task_type 协同)
    pub fn from_task_type(task: &str) -> Self {
        let t = task.to_lowercase();
        if t.contains("evolve") || t.contains("distill") || t.contains("absorb")
            || t.contains("reflect") || t.contains("crystallize") || t.contains("learn")
        {
            Self::Evolution
        } else {
            Self::Acquisition
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Acquisition => "Weapon Set I (获取: CORE+WORLD)",
            Self::Evolution => "Weapon Set II (进化: CORE+MIND)",
        }
    }
}

pub struct AttentionManager {
    pub heads: Vec<AttentionHead>,
    pub global_threshold: f64,
    pub rule_intensity: RuleIntensity,
    /// Ascendancy 当前专精 (Weapon Set)
    pub weapon_set: WeaponSet,
}

impl AttentionManager {
    pub fn new(threshold: f64) -> Self {
        let heads: Vec<AttentionHead> = AttentionDomain::all().into_iter()
            .enumerate()
            .map(|(i, domain)| AttentionHead::new(i, domain))
            .collect();
        Self { heads, global_threshold: threshold, rule_intensity: RuleIntensity::default(), weapon_set: WeaponSet::Acquisition }
    }

    pub fn with_intensity(threshold: f64, intensity: RuleIntensity) -> Self {
        let mut mgr = Self::new(threshold);
        mgr.set_intensity(intensity);
        mgr
    }

    pub fn set_intensity(&mut self, intensity: RuleIntensity) {
        self.rule_intensity = intensity;
        self.global_threshold = intensity.attention_threshold();
    }

    pub fn from_task_type(threshold: f64, task: &str) -> Self {
        let intensity = RuleIntensity::from_task_type(task);
        let mut mgr = Self::with_intensity(threshold, intensity);
        mgr.weapon_set = WeaponSet::from_task_type(task);
        mgr
    }

    /// Ascendancy: 切换 Weapon Set 并给予优先级域启动激活加成
    pub fn activate_weapon_set(&mut self, set: WeaponSet, boost: f64) {
        self.weapon_set = set;
        for domain in set.priority_domains() {
            self.stimulate_domain(domain, boost);
        }
    }

    /// 当前专精的优先级域列表
    pub fn active_priority_domains(&self) -> Vec<AttentionDomain> {
        self.weapon_set.priority_domains()
    }

    pub fn stimulate_domain(&mut self, domain: AttentionDomain, amount: f64) {
        if let Some(head) = self.heads.iter_mut().find(|h| h.domain == domain) {
            head.stimulate(amount);
        }
    }

    pub fn decay_all(&mut self) {
        for head in &mut self.heads {
            head.decay();
        }
    }

    pub fn active_heads(&self) -> Vec<&AttentionHead> {
        self.heads.iter().filter(|h| h.activation >= self.global_threshold).collect()
    }

    pub fn dominant_domain(&self) -> Option<AttentionDomain> {
        self.heads.iter().max_by(|a, b| a.activation.partial_cmp(&b.activation).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|h| h.activation > 0.0)
            .map(|h| h.domain)
    }

    pub fn profile(&self) -> AttentionProfile {
        let distribution: HashMap<AttentionDomain, f64> = self.heads.iter()
            .map(|h| (h.domain, h.activation))
            .collect();
        let dominant = self.dominant_domain().unwrap_or(AttentionDomain::PatternMatch);
        let num_activated = self.active_heads().len();
        AttentionProfile::new(dominant, distribution, num_activated)
    }

    pub fn reset(&mut self) {
        for head in &mut self.heads {
            head.activation = 0.0;
            head.focus.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_head_new() {
        let h = AttentionHead::new(0, AttentionDomain::Code);
        assert_eq!(h.domain, AttentionDomain::Code);
        assert_eq!(h.activation, 0.0);
        assert_eq!(h.id, 0);
    }

    #[test]
    fn test_stimulate_and_decay() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(0.5);
        assert!((h.activation - 0.5).abs() < 1e-6);
        h.decay();
        assert!((h.activation - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_activation_capped() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(1.5);
        assert!((h.activation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_salience_formula() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(0.8);
        let s = h.salience(0.5, 0.5);
        assert!((s - 0.8 * 0.5 * 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_attention_manager_decay_all() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.7);
        let code_idx = AttentionDomain::all().iter().position(|d| *d == AttentionDomain::Code).expect("value should be ok in test");
        let plan_idx = AttentionDomain::all().iter().position(|d| *d == AttentionDomain::Planning).expect("value should be ok in test");
        assert_eq!(mgr.active_heads().len(), 2);
        mgr.decay_all();
        assert!((mgr.heads[code_idx].activation - 0.8).abs() < 1e-6);
        assert!((mgr.heads[plan_idx].activation - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_domain() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.3);
        assert_eq!(mgr.dominant_domain(), Some(AttentionDomain::Code));
    }

    #[test]
    fn test_attention_profile() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::SelfReflection, 0.8);
        let profile = mgr.profile();
        assert_eq!(profile.dominant, AttentionDomain::SelfReflection);
        assert!(profile.num_activated_heads >= 1);
        assert!(profile.distribution.get(&AttentionDomain::SelfReflection).copied().unwrap_or(0.0) > 0.0);
    }

    #[test]
    fn test_focus_on_concept() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.focus_on("rust");
        assert!(h.focus.contains(&"rust".to_string()));
        assert!(h.activation > 0.0);
        let act_before = h.activation;
        h.focus_on("rust");
        assert_eq!(h.focus.len(), 1);
        assert!(h.activation >= act_before);
    }

    #[test]
    fn test_reset_manager() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.7);
        assert!(mgr.active_heads().len() > 0);
        mgr.reset();
        assert_eq!(mgr.active_heads().len(), 0);
    }

    #[test]
    fn test_all_domains_count() {
        let domains = AttentionDomain::all();
        assert_eq!(domains.len(), 10);
    }

    #[test]
    fn test_weapon_set_from_task_type() {
        assert_eq!(WeaponSet::from_task_type("absorb knowledge"), WeaponSet::Evolution);
        assert_eq!(WeaponSet::from_task_type("distill session"), WeaponSet::Evolution);
        assert_eq!(WeaponSet::from_task_type("crawler fix"), WeaponSet::Acquisition);
        assert_eq!(WeaponSet::from_task_type("implement feature"), WeaponSet::Acquisition);
    }

    #[test]
    fn test_weapon_set_priority_domains() {
        let acq = WeaponSet::Acquisition.priority_domains();
        assert!(acq.contains(&AttentionDomain::Code));
        assert!(acq.contains(&AttentionDomain::ToolUse));
        assert!(!acq.contains(&AttentionDomain::SelfReflection));
        let evo = WeaponSet::Evolution.priority_domains();
        assert!(evo.contains(&AttentionDomain::SelfReflection));
        assert!(evo.contains(&AttentionDomain::Creativity));
        assert!(!evo.contains(&AttentionDomain::Code));
    }

    #[test]
    fn test_activate_weapon_set_boosts_priority_domains() {
        let mut mgr = AttentionManager::new(0.5);
        mgr.activate_weapon_set(WeaponSet::Evolution, 0.6);
        assert_eq!(mgr.weapon_set, WeaponSet::Evolution);
        for domain in mgr.active_priority_domains() {
            let head = mgr.heads.iter().find(|h| h.domain == domain).expect("head exists");
            assert!(head.activation >= 0.6, "priority domain {} should be boosted", domain.label());
        }
        // 非优先级域不应被提升
        let code_head = mgr.heads.iter().find(|h| h.domain == AttentionDomain::Code).unwrap();
        assert!(code_head.activation < 0.6);
    }

    #[test]
    fn test_from_task_type_sets_weapon_set() {
        let mgr = AttentionManager::from_task_type(0.4, "distill knowledge");
        assert_eq!(mgr.weapon_set, WeaponSet::Evolution);
        let mgr2 = AttentionManager::from_task_type(0.4, "write code");
        assert_eq!(mgr2.weapon_set, WeaponSet::Acquisition);
    }

    #[test]
    fn test_attention_head_is_activated() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        assert!(!h.is_activated(0.5));
        h.stimulate(0.6);
        assert!(h.is_activated(0.5));
    }

    #[test]
    fn test_rule_intensity_from_task_type() {
        assert_eq!(RuleIntensity::from_task_type("explore codebase"), RuleIntensity::Lite);
        assert_eq!(RuleIntensity::from_task_type("search for pattern"), RuleIntensity::Lite);
        assert_eq!(RuleIntensity::from_task_type("read file"), RuleIntensity::Lite);
        assert_eq!(RuleIntensity::from_task_type("implement feature"), RuleIntensity::Full);
        assert_eq!(RuleIntensity::from_task_type("fix bug"), RuleIntensity::Full);
        assert_eq!(RuleIntensity::from_task_type("refactor module"), RuleIntensity::Full);
        assert_eq!(RuleIntensity::from_task_type("architect system"), RuleIntensity::Ultra);
        assert_eq!(RuleIntensity::from_task_type("design api"), RuleIntensity::Ultra);
        assert_eq!(RuleIntensity::from_task_type("rewrite core"), RuleIntensity::Ultra);
        assert_eq!(RuleIntensity::from_task_type("unknown"), RuleIntensity::Full);
    }

    #[test]
    fn test_rule_intensity_threshold() {
        assert_eq!(RuleIntensity::Lite.attention_threshold(), 0.2);
        assert_eq!(RuleIntensity::Full.attention_threshold(), 0.4);
        assert_eq!(RuleIntensity::Ultra.attention_threshold(), 0.6);
    }

    #[test]
    fn test_attention_manager_with_intensity() {
        let mgr = AttentionManager::with_intensity(0.3, RuleIntensity::Lite);
        assert_eq!(mgr.global_threshold, 0.2);
        let mgr = AttentionManager::with_intensity(0.3, RuleIntensity::Full);
        assert_eq!(mgr.global_threshold, 0.4);
        let mgr = AttentionManager::with_intensity(0.3, RuleIntensity::Ultra);
        assert_eq!(mgr.global_threshold, 0.6);
    }

    #[test]
    fn test_attention_manager_from_task_type() {
        let mgr = AttentionManager::from_task_type(0.5, "explore");
        assert_eq!(mgr.global_threshold, 0.2);
        let mgr = AttentionManager::from_task_type(0.5, "implement feature");
        assert_eq!(mgr.global_threshold, 0.4);
        let mgr = AttentionManager::from_task_type(0.5, "architect");
        assert_eq!(mgr.global_threshold, 0.6);
    }

    #[test]
    fn test_attention_manager_set_intensity() {
        let mut mgr = AttentionManager::new(0.5);
        mgr.set_intensity(RuleIntensity::Ultra);
        assert_eq!(mgr.global_threshold, 0.6);
    }
}
