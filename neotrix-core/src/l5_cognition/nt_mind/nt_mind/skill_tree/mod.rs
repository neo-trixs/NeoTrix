pub mod node;
pub mod rune;
pub mod ascendancy;

pub use node::{
    SkillNode, SkillTreeRegistry, Tier, UpgradeCondition, NodeEffect,
};
pub use rune::{
    RuneColor, RuneSlot, Rune, Runeword, ModuleRunes, RuneSystem,
};
pub use ascendancy::{
    AscendancyRouter, WeaponSetKind, SwitchRecord,
};

/// 技能树配置 — 组合三大核心组件
#[derive(Debug, Clone, Default)]
pub struct SkillTreeConfig {
    pub tree: SkillTreeRegistry,
    pub rune_system: RuneSystem,
    pub ascendancy: AscendancyRouter,
}

impl SkillTreeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// 执行一次完整的技能树周期: 记录使用 → 自动晋升 → 符文检查 → 专精路由
    pub fn tick(&mut self, task: &str) {
        // 1. 记录使用
        for node in self.tree.nodes.iter_mut() {
            if node.unlocked {
                node.record_use();
            }
        }
        // 2. 自动晋升
        self.tree.auto_promote_all();
        // 3. 符文检查
        self.rune_system.check_all_runewords();
        // 4. 专精路由
        self.ascendancy.route_by_task(task);
    }

    /// 获取完整状态摘要
    pub fn status(&self) -> String {
        format!(
            "{}\n{}\n{}",
            self.tree.summary(),
            self.rune_system.total_bonus(),
            self.ascendancy.ascendancy_summary(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = SkillTreeConfig::new();
        assert!(config.tree.nodes.is_empty());
        assert!(config.rune_system.modules.is_empty());
    }

    #[test]
    fn test_config_tick() {
        let mut config = SkillTreeConfig::new();
        let mut n1 = super::node::SkillNode::new(
            "n1", "Test", "Test",
            super::node::Tier::SmallPassive, "NT-CORE",
        );
        n1.unlocked = true;
        config.tree.add_node(n1);
        config.rune_system.register_module(
            "nt-core",
            vec![RuneColor::Crimson, RuneColor::Indigo],
        );
        let rune = Rune::new("r1", "Fire", RuneColor::Crimson, 0.8);
        config.rune_system.insert_rune("nt-core", RuneColor::Crimson, &rune).unwrap();
        config.tick("fix code");
        // tick should not panic
        assert!(config.tree.nodes[0].use_count > 0);
    }

    #[test]
    fn test_config_status() {
        let config = SkillTreeConfig::new();
        let s = config.status();
        assert!(!s.is_empty());
    }
}