use std::collections::HashMap;
use crate::core::nt_core_self::attention_head::{AttentionDomain, AttentionManager, WeaponSet};

/// 双专精 — 每 session 两个 Weapon Set
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WeaponSetKind {
    /// 获取模式 (Acquisition) — CORE+WORLD 域优先
    Acquisition,
    /// 进化模式 (Evolution) — CORE+MIND 域优先
    Evolution,
}

impl WeaponSetKind {
    pub fn label(&self) -> &str {
        match self {
            Self::Acquisition => "Weapon Set I (获取: CORE+WORLD)",
            Self::Evolution => "Weapon Set II (进化: CORE+MIND)",
        }
    }

    pub fn all() -> Vec<WeaponSetKind> {
        vec![Self::Acquisition, Self::Evolution]
    }
}

/// 专精切换记录 — 跟踪 session 内的 Weapon Set 切换
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SwitchRecord {
    pub from_set: WeaponSetKind,
    pub to_set: WeaponSetKind,
    pub switch_count: u32,
}

/// Ascendancy 双专精路由 — AttentionManager 按任务类型路由
#[derive(Debug, Clone)]
pub struct AscendancyRouter {
    /// 当前激活的 Weapon Set
    current_set: WeaponSetKind,
    /// 切换记录
    switch_history: Vec<SwitchRecord>,
    /// 切换次数
    switch_count: u32,
    /// 每个域的激活强度 (用于路由决策)
    domain_scores: HashMap<AttentionDomain, f64>,
    /// AttentionManager 引用
    attention_mgr: AttentionManager,
}

impl AscendancyRouter {
    pub fn new() -> Self {
        let mgr = AttentionManager::new(0.5);
        Self {
            current_set: WeaponSetKind::Acquisition,
            switch_history: Vec::new(),
            switch_count: 0,
            domain_scores: HashMap::new(),
            attention_mgr: mgr,
        }
    }

    /// 获取当前专精
    pub fn current_set(&self) -> WeaponSetKind {
        self.current_set
    }

    /// 获取切换次数
    pub fn switch_count(&self) -> u32 {
        self.switch_count
    }

    /// 切换到指定专精
    pub fn switch_to(&mut self, set: WeaponSetKind) {
        let from = self.current_set;
        if from == set {
            return;
        }
        self.attention_mgr.activate_weapon_set(
            match set {
                WeaponSetKind::Acquisition => WeaponSet::Acquisition,
                WeaponSetKind::Evolution => WeaponSet::Evolution,
            },
            0.6,
        );
        self.switch_history.push(SwitchRecord {
            from_set: from,
            to_set: set,
            switch_count: self.switch_count,
        });
        self.current_set = set;
        self.switch_count += 1;
    }

    /// 根据任务类型自动路由到正确的专精
    pub fn route_by_task(&mut self, task: &str) {
        let set = WeaponSet::from_task_type(task);
        let target = match set {
            WeaponSet::Acquisition => WeaponSetKind::Acquisition,
            WeaponSet::Evolution => WeaponSetKind::Evolution,
        };
        self.switch_to(target);
    }

    /// 获取当前专精的优先级域
    pub fn active_priority_domains(&self) -> Vec<AttentionDomain> {
        match self.current_set {
            WeaponSetKind::Acquisition => {
                vec![
                    AttentionDomain::PatternMatch,
                    AttentionDomain::Code,
                    AttentionDomain::Temporal,
                    AttentionDomain::ToolUse,
                ]
            }
            WeaponSetKind::Evolution => {
                vec![
                    AttentionDomain::Semantic,
                    AttentionDomain::SelfReflection,
                    AttentionDomain::Creativity,
                    AttentionDomain::GoalAlignment,
                ]
            }
        }
    }

    /// 评估任务到当前专精的匹配度
    pub fn match_score(&self, task: &str) -> f64 {
        let task_lower = task.to_lowercase();
        let priority_domains = self.active_priority_domains();
        let mut score = 0.0;
        for domain in &priority_domains {
            if let Some(&domain_score) = self.domain_scores.get(domain) {
                score += domain_score;
            }
        }
        // 关键词匹配加分
        let mut domain_keywords: HashMap<AttentionDomain, Vec<&str>> = HashMap::new();
        match self.current_set {
            WeaponSetKind::Acquisition => {
                domain_keywords.insert(AttentionDomain::Code, vec!["code", "fix", "implement", "build", "refactor"]);
                domain_keywords.insert(AttentionDomain::PatternMatch, vec!["pattern", "match", "similar", "reuse"]);
                domain_keywords.insert(AttentionDomain::ToolUse, vec!["tool", "mcp", "api", "execute"]);
                domain_keywords.insert(AttentionDomain::Temporal, vec!["time", "schedule", "deadline"]);
            }
            WeaponSetKind::Evolution => {
                domain_keywords.insert(AttentionDomain::SelfReflection, vec!["reflect", "review", "meta", "self"]);
                domain_keywords.insert(AttentionDomain::Creativity, vec!["creative", "novel", "design", "brainstorm"]);
                domain_keywords.insert(AttentionDomain::Semantic, vec!["search", "learn", "understand", "knowledge"]);
                domain_keywords.insert(AttentionDomain::GoalAlignment, vec!["goal", "objective", "align", "strategy"]);
            }
        }
        for domain in &priority_domains {
            if let Some(keywords) = domain_keywords.get(domain) {
                for kw in keywords {
                    if task_lower.contains(kw) {
                        score += 0.1;
                    }
                }
            }
        }
        score.clamp(0.0, 1.0)
    }

    /// 更新域的激活强度
    pub fn update_domain_score(&mut self, domain: AttentionDomain, score: f64) {
        self.domain_scores.insert(domain, score.clamp(0.0, 1.0));
    }

    /// 获取 AttentionManager 引用
    pub fn attention_manager(&self) -> &AttentionManager {
        &self.attention_mgr
    }

    pub fn attention_manager_mut(&mut self) -> &mut AttentionManager {
        &mut self.attention_mgr
    }

    /// 获取切换历史
    pub fn switch_history(&self) -> &[SwitchRecord] {
        &self.switch_history
    }

    /// 获取当前会话的两个专精状态摘要
    pub fn ascendancy_summary(&self) -> String {
        format!(
            "Ascendancy: current={}, switch_count={}, priority_domains={:?}",
            self.current_set.label(),
            self.switch_count,
            self.active_priority_domains()
                .iter()
                .map(|d| d.label())
                .collect::<Vec<_>>(),
        )
    }
}

impl Default for AscendancyRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 WeaponSetKind 转换为 AttentionDomain (用于域匹配)
impl WeaponSetKind {
    pub fn to_attention_domain(&self) -> AttentionDomain {
        match self {
            Self::Acquisition => AttentionDomain::Code,
            Self::Evolution => AttentionDomain::SelfReflection,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
     use crate::core::nt_core_self::attention_head::AttentionDomain;

    #[test]
    fn test_ascendancy_router_new() {
        let router = AscendancyRouter::new();
        assert_eq!(router.current_set(), WeaponSetKind::Acquisition);
        assert_eq!(router.switch_count(), 0);
    }

    #[test]
    fn test_switch_to_evolution() {
        let mut router = AscendancyRouter::new();
        router.switch_to(WeaponSetKind::Evolution);
        assert_eq!(router.current_set(), WeaponSetKind::Evolution);
        assert_eq!(router.switch_count(), 1);
    }

    #[test]
    fn test_switch_same_set_no_op() {
        let mut router = AscendancyRouter::new();
        router.switch_to(WeaponSetKind::Acquisition);
        assert_eq!(router.switch_count(), 0);
    }

    #[test]
    fn test_route_by_task() {
        let mut router = AscendancyRouter::new();
        router.route_by_task("distill knowledge");
        assert_eq!(router.current_set(), WeaponSetKind::Evolution);
        router.route_by_task("fix bug");
        assert_eq!(router.current_set(), WeaponSetKind::Acquisition);
    }

    #[test]
    fn test_active_priority_domains_acquisition() {
        let router = AscendancyRouter::new();
        let domains = router.active_priority_domains();
        assert!(domains.contains(&AttentionDomain::Code));
        assert!(domains.contains(&AttentionDomain::ToolUse));
        assert!(!domains.contains(&AttentionDomain::SelfReflection));
    }

    #[test]
    fn test_active_priority_domains_evolution() {
        let mut router = AscendancyRouter::new();
        router.switch_to(WeaponSetKind::Evolution);
        let domains = router.active_priority_domains();
        assert!(domains.contains(&AttentionDomain::SelfReflection));
        assert!(domains.contains(&AttentionDomain::Creativity));
        assert!(!domains.contains(&AttentionDomain::Code));
    }

    #[test]
    fn test_match_score() {
        let router = AscendancyRouter::new();
        let score = router.match_score("fix the code");
        assert!(score > 0.0);
    }

    #[test]
    fn test_match_score_evolution() {
        let mut router = AscendancyRouter::new();
        router.switch_to(WeaponSetKind::Evolution);
        let score = router.match_score("reflect on the design");
        assert!(score > 0.0);
    }

    #[test]
    fn test_update_domain_score() {
        let mut router = AscendancyRouter::new();
        router.update_domain_score(AttentionDomain::Code, 0.8);
        router.update_domain_score(AttentionDomain::Creativity, 0.6);
        assert_eq!(router.domain_scores[&AttentionDomain::Code], 0.8);
    }

    #[test]
    fn test_switch_history() {
        let mut router = AscendancyRouter::new();
        router.switch_to(WeaponSetKind::Evolution);
        router.switch_to(WeaponSetKind::Acquisition);
        assert_eq!(router.switch_count(), 2);
        assert_eq!(router.switch_history().len(), 2);
        assert_eq!(router.switch_history()[0].from_set, WeaponSetKind::Acquisition);
        assert_eq!(router.switch_history()[0].to_set, WeaponSetKind::Evolution);
        assert_eq!(router.switch_history()[1].from_set, WeaponSetKind::Evolution);
        assert_eq!(router.switch_history()[1].to_set, WeaponSetKind::Acquisition);
    }

    #[test]
    fn test_ascendancy_summary() {
        let router = AscendancyRouter::new();
        let summary = router.ascendancy_summary();
        assert!(summary.contains("Weapon Set I"));
    }

#[test]
    fn test_attention_manager_access() {
        let mut router = AscendancyRouter::new();
        assert_eq!(router.current_set(), WeaponSetKind::Acquisition);
        router.switch_to(WeaponSetKind::Evolution);
        assert_eq!(router.current_set(), WeaponSetKind::Evolution);
    }

    #[test]
    fn test_weapon_set_kind_to_attention_domain() {
        assert_eq!(
            WeaponSetKind::Acquisition.to_attention_domain(),
            AttentionDomain::Code
        );
        assert_eq!(
            WeaponSetKind::Evolution.to_attention_domain(),
            AttentionDomain::SelfReflection
        );
    }
}