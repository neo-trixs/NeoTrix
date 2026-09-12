use std::collections::HashMap;

/// Skill Tree 三层节点类型 (POE-inspired)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Tier {
    /// 微节点 (Small Passive) — 基础能力片断
    SmallPassive,
    /// 显节点 (Notable Passive) — 组合能力
    NotablePassive,
    /// 基石 (Keystone) — 决定性能力
    Keystone,
}

impl Tier {
    pub fn label(&self) -> &str {
        match self {
            Self::SmallPassive => "Small Passive",
            Self::NotablePassive => "Notable Passive",
            Self::Keystone => "Keystone",
        }
    }

    /// 升级所需前置节点数 (SmallPassive→NotablePassive 需1, NotablePassive→Keystone 需2)
    pub(crate) fn _upgrade_cost(&self) -> u8 {
        match self {
            Self::SmallPassive => 1,
            Self::NotablePassive => 2,
            Self::Keystone => 3,
        }
    }
}

/// 升级条件 — 满足条件自动晋升
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpgradeCondition {
    /// 激活的前置节点数量
    pub prerequisite_count: u8,
    /// 最低激活分数
    pub min_activation_score: f64,
    /// 所需域 (可选，空=不限)
    pub required_domain: Option<String>,
    /// 最小使用次数
    pub min_use_count: usize,
}

impl Default for UpgradeCondition {
    fn default() -> Self {
        Self {
            prerequisite_count: 1,
            min_activation_score: 0.5,
            required_domain: None,
            min_use_count: 0,
        }
    }
}

/// Skill Tree 单个节点
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tier: Tier,
    pub domain: String,
    /// 前置节点 ID 列表
    pub prerequisites: Vec<String>,
    /// 当前是否已激活
    pub unlocked: bool,
    /// 激活进度 [0.0, 1.0]
    pub progress: f64,
    /// 激活分数 (0.0-1.0)
    pub activation_score: f64,
    /// 使用次数
    pub use_count: usize,
    /// 升级条件
    pub upgrade_condition: UpgradeCondition,
    /// 节点效果
    pub effects: Vec<NodeEffect>,
}

/// 节点效果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeEffect {
    pub effect_type: String,
    pub target: String,
    pub value: f64,
    pub description: String,
}

impl SkillNode {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        tier: Tier,
        domain: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            tier,
            domain: domain.into(),
            prerequisites: Vec::new(),
            unlocked: false,
            progress: 0.0,
            activation_score: 0.0,
            use_count: 0,
            upgrade_condition: UpgradeCondition::default(),
            effects: Vec::new(),
        }
    }

    /// 添加前置节点
    pub(crate) fn _add_prerequisite(&mut self, prereq_id: impl Into<String>) {
        self.prerequisites.push(prereq_id.into());
    }

    /// 添加效果
    pub fn add_effect(&mut self, effect: NodeEffect) {
        self.effects.push(effect);
    }

    /// 检查是否满足升级条件
    pub fn can_upgrade(&self, activated_ids: &[String]) -> bool {
        if self.unlocked {
            return false;
        }
        let condition = &self.upgrade_condition;
        if condition.prerequisite_count > 0 {
            let mut met = 0;
            for prereq_id in &self.prerequisites {
                if activated_ids.contains(prereq_id) {
                    met += 1;
                }
            }
            if met < condition.prerequisite_count {
                return false;
            }
        }
        if self.activation_score < condition.min_activation_score {
            return false;
        }
        if let Some(ref domain) = condition.required_domain {
            if self.domain != *domain {
                return false;
            }
        }
        if self.use_count < condition.min_use_count {
            return false;
        }
        true
    }

    /// 尝试自动晋升到下一层
    pub(crate) fn _try_auto_promote(
        &self,
        activated_ids: &[String],
    ) -> Option<Tier> {
        if !self.can_upgrade(activated_ids) {
            return None;
        }
        let next_tier = match self.tier {
            Tier::SmallPassive => Tier::NotablePassive,
            Tier::NotablePassive => Tier::Keystone,
            Tier::Keystone => return None,
        };
        Some(next_tier)
    }

    /// 晋升到指定层
    pub(crate) fn _promote_to(&mut self, new_tier: Tier) {
        self.tier = new_tier;
        self.unlocked = true;
        self.progress = 1.0;
        self.activation_score = (self.activation_score + 0.3).min(1.0);
    }

    /// 记录使用
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.activation_score = (self.activation_score + 0.05).min(1.0);
        self.progress = (self.progress + 0.1).min(1.0);
    }
}

/// Skill Tree 注册表
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillTreeRegistry {
    pub nodes: Vec<SkillNode>,
    pub next_id: u64,
}

impl SkillTreeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, node: SkillNode) -> String {
        let id = node.id.clone();
        self.nodes.push(node);
        id
    }

    pub fn get_node(&self, id: &str) -> Option<&SkillNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub(crate) fn _get_node_mut(&mut self, id: &str) -> Option<&mut SkillNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// 获取所有已激活节点
    pub fn activated_nodes(&self) -> Vec<&SkillNode> {
        self.nodes.iter().filter(|n| n.unlocked).collect()
    }

    /// 获取指定域的节点
    pub(crate) fn _nodes_by_domain(&self, domain: &str) -> Vec<&SkillNode> {
        self.nodes.iter().filter(|n| n.domain == domain).collect()
    }

    /// 执行自动晋升: 遍历所有未激活节点，尝试晋升
    pub fn auto_promote_all(&mut self) -> Vec<String> {
        let mut promoted = Vec::new();
        let activated_ids: Vec<String> = self
            .nodes
            .iter()
            .filter(|n| n.unlocked)
            .map(|n| n.id.clone())
            .collect();

        // 第一遍: 找出所有可晋升的节点 ID
        let mut to_promote: Vec<(String, Tier)> = Vec::new();
        for node in &self.nodes {
            if node.unlocked {
                continue;
            }
            if node.can_upgrade(&activated_ids) {
                if let Some(next_tier) = node._try_auto_promote(&activated_ids) {
                    to_promote.push((node.id.clone(), next_tier));
                }
            }
        }

        // 第二遍: 晋升节点
        for (id, next_tier) in to_promote {
            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
                node._promote_to(next_tier);
                promoted.push(id);
            }
        }
        promoted
    }

    /// 构建激活节点的依赖图 (返回 topological order)
    pub fn topological_order(&self) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut order = Vec::new();
        for node in &self.nodes {
            self.dfs_topo(node, &mut visited, &mut order);
        }
        order
    }

    fn dfs_topo(
        &self,
        node: &SkillNode,
        visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) {
        if visited.contains(&node.id) {
            return;
        }
        visited.insert(node.id.clone());
        for prereq_id in &node.prerequisites {
            if let Some(prereq) = self.get_node(prereq_id) {
                self.dfs_topo(prereq, visited, order);
            }
        }
        order.push(node.id.clone());
    }

    pub fn summary(&self) -> String {
        let total = self.nodes.len();
        let activated = self.nodes.iter().filter(|n| n.unlocked).count();
        let tiers: HashMap<Tier, usize> = self.nodes.iter().fold(
            HashMap::new(),
            |mut acc, n| {
                *acc.entry(n.tier).or_insert(0) += 1;
                acc
            },
        );
        format!(
            "SkillTree: {}/{} activated (small_passive={}, notable_passive={}, keystone={})",
            activated,
            total,
            tiers.get(&Tier::SmallPassive).unwrap_or(&0),
            tiers.get(&Tier::NotablePassive).unwrap_or(&0),
            tiers.get(&Tier::Keystone).unwrap_or(&0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, tier: Tier, domain: &str) -> SkillNode {
        SkillNode::new(id, id, id, tier, domain)
    }

    #[test]
    fn test_node_creation() {
        let n = make_node("test-1", Tier::SmallPassive, "NT-CORE");
        assert_eq!(n.id, "test-1");
        assert_eq!(n.tier, Tier::SmallPassive);
        assert_eq!(n.domain, "NT-CORE");
        assert!(!n.unlocked);
        assert_eq!(n.use_count, 0);
    }

    #[test]
    fn test_node_add_prerequisite() {
        let mut n = make_node("test-2", Tier::NotablePassive, "NT-MIND");
        n._add_prerequisite("test-1");
        assert_eq!(n.prerequisites, vec!["test-1"]);
    }

    #[test]
    fn test_node_record_use_increases_score() {
        let mut n = make_node("test-3", Tier::SmallPassive, "NT-CORE");
        let initial = n.activation_score;
        n.record_use();
        assert!(n.activation_score > initial);
        assert_eq!(n.use_count, 1);
    }

    #[test]
    fn test_node_can_upgrade_when_prereqs_met() {
        let mut reg = SkillTreeRegistry::new();
        let mut n1 = make_node("n1", Tier::SmallPassive, "NT-CORE");
        n1.unlocked = true;
        let mut n2 = make_node("n2", Tier::NotablePassive, "NT-CORE");
        n2._add_prerequisite("n1");
        n2.activation_score = 0.5;
        n2.upgrade_condition.prerequisite_count = 1;
        reg.add_node(n1);
        reg.add_node(n2);

        let activated: Vec<String> = reg.nodes.iter().filter(|n| n.unlocked).map(|n| n.id.clone()).collect();
        assert!(reg.get_node("n2").unwrap().can_upgrade(&activated));
    }

    #[test]
    fn test_node_cannot_upgrade_without_prereqs() {
        let mut reg = SkillTreeRegistry::new();
        let n2 = make_node("n2", Tier::NotablePassive, "NT-CORE");
        reg.add_node(n2);

        let activated: Vec<String> = Vec::new();
        assert!(!reg.get_node("n2").unwrap().can_upgrade(&activated));
    }

    #[test]
    fn test_auto_promote() {
        let mut reg = SkillTreeRegistry::new();
        let mut n1 = make_node("n1", Tier::SmallPassive, "NT-CORE");
        n1.unlocked = true;
        n1.activation_score = 0.8;
        let mut n2 = make_node("n2", Tier::SmallPassive, "NT-CORE");
        n2._add_prerequisite("n1");
        n2.upgrade_condition.prerequisite_count = 1;
        n2.activation_score = 0.8;
        reg.add_node(n1);
        reg.add_node(n2);

        let promoted = reg.auto_promote_all();
        assert!(promoted.contains(&"n2".to_string()));
        assert!(reg.get_node("n2").unwrap().unlocked);
        assert_eq!(reg.get_node("n2").unwrap().tier, Tier::NotablePassive);
    }

    #[test]
    fn test_no_promote_when_prereq_not_activated() {
        let mut reg = SkillTreeRegistry::new();
        let mut n2 = make_node("n2", Tier::SmallPassive, "NT-CORE");
        n2._add_prerequisite("n1");
        n2.upgrade_condition.prerequisite_count = 1;
        n2.activation_score = 0.8;
        reg.add_node(n2);

        let promoted = reg.auto_promote_all();
        assert!(promoted.is_empty());
    }

    #[test]
    fn test_keystone_cannot_promote_further() {
        let mut n = make_node("ks", Tier::Keystone, "NT-CORE");
        n.unlocked = true;
        n.activation_score = 1.0;
        let node_id = n.id.clone();
        let activated: Vec<String> = vec![node_id];
        assert!(n._try_auto_promote(&activated).is_none());
    }

    #[test]
    fn test_topological_order() {
        let mut reg = SkillTreeRegistry::new();
        let n1 = make_node("n1", Tier::SmallPassive, "NT-CORE");
        let mut n2 = make_node("n2", Tier::NotablePassive, "NT-CORE");
        n2._add_prerequisite("n1");
        let mut n3 = make_node("n3", Tier::Keystone, "NT-CORE");
        n3._add_prerequisite("n2");
        reg.add_node(n1);
        reg.add_node(n2);
        reg.add_node(n3);

        let order = reg.topological_order();
        assert_eq!(order.len(), 3);
        let idx_n1 = order.iter().position(|x| x == "n1").unwrap();
        let idx_n2 = order.iter().position(|x| x == "n2").unwrap();
        let idx_n3 = order.iter().position(|x| x == "n3").unwrap();
        assert!(idx_n1 < idx_n2);
        assert!(idx_n2 < idx_n3);
    }

    #[test]
    fn test_summary() {
        let mut reg = SkillTreeRegistry::new();
        let mut n1 = make_node("n1", Tier::SmallPassive, "NT-CORE");
        n1.unlocked = true;
        reg.add_node(n1);
        reg.add_node(make_node("n2", Tier::SmallPassive, "NT-CORE"));
        let s = reg.summary();
        assert!(s.contains("1/2 activated"));
    }
}