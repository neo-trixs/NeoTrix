pub mod cultivation;

use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Character Stats
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stat {
    Str,
    Dex,
    Int,
    Wis,
    Con,
    Cha,
}

impl Stat {
    pub fn all() -> &'static [Stat] {
        &[
            Stat::Str,
            Stat::Dex,
            Stat::Int,
            Stat::Wis,
            Stat::Con,
            Stat::Cha,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Stat::Str => "Strength",
            Stat::Dex => "Dexterity",
            Stat::Int => "Intelligence",
            Stat::Wis => "Wisdom",
            Stat::Con => "Constitution",
            Stat::Cha => "Charisma",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CharacterStats {
    pub base: HashMap<Stat, u32>,
    pub bonuses: HashMap<Stat, i32>,
}

impl CharacterStats {
    pub fn new(base: HashMap<Stat, u32>) -> Self {
        let mut bonuses = HashMap::new();
        for stat in Stat::all() {
            bonuses.insert(*stat, 0);
        }
        Self { base, bonuses }
    }

    pub fn with_base(mut self, stat: Stat, value: u32) -> Self {
        self.base.insert(stat, value);
        self
    }

    pub fn get(&self, stat: Stat) -> u32 {
        let base = self.base.get(&stat).copied().unwrap_or(10);
        let bonus = self.bonuses.get(&stat).copied().unwrap_or(0);
        (base as i32 + bonus).max(1) as u32
    }

    pub fn add_bonus(&mut self, stat: Stat, amount: i32) {
        *self.bonuses.entry(stat).or_insert(0) += amount;
    }

    pub fn modifier(&self, stat: Stat) -> i32 {
        (self.get(stat) as i32 - 10) / 2
    }

    pub fn total(&self) -> u32 {
        Stat::all().iter().map(|s| self.get(*s)).sum()
    }
}

impl Default for CharacterStats {
    fn default() -> Self {
        let mut base = HashMap::new();
        for stat in Stat::all() {
            base.insert(*stat, 10);
        }
        Self::new(base)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Level System
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct LevelSystem {
    pub level: u32,
    pub xp: u64,
    pub stat_points: u32,
    pub skill_points: u32,
}

impl LevelSystem {
    pub fn new() -> Self {
        Self {
            level: 1,
            xp: 0,
            stat_points: 0,
            skill_points: 0,
        }
    }

    pub fn xp_for_level(level: u32) -> u64 {
        // Quadratic XP curve: level^2 * 100
        (level as u64)
            .saturating_mul(level as u64)
            .saturating_mul(100)
    }

    pub fn xp_to_next(&self) -> u64 {
        Self::xp_for_level(self.level + 1).saturating_sub(self.xp)
    }

    pub fn total_xp_for_current_level(&self) -> u64 {
        Self::xp_for_level(self.level)
    }

    pub fn add_xp(&mut self, amount: u64) -> Vec<u32> {
        self.xp += amount;
        let mut levels_gained = Vec::new();
        loop {
            let required = Self::xp_for_level(self.level + 1);
            if self.xp >= required {
                self.level += 1;
                self.stat_points += 5;
                self.skill_points += 1;
                levels_gained.push(self.level);
            } else {
                break;
            }
        }
        levels_gained
    }

    pub fn allocate_stat(&mut self, stat: Stat, stats: &mut CharacterStats, points: u32) -> bool {
        if self.stat_points < points {
            return false;
        }
        self.stat_points -= points;
        stats.add_bonus(stat, points as i32);
        true
    }

    pub fn level_progress(&self) -> f64 {
        let current = Self::xp_for_level(self.level);
        let next = Self::xp_for_level(self.level + 1);
        if next == current {
            return 1.0;
        }
        (self.xp - current) as f64 / (next - current) as f64
    }
}

impl Default for LevelSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Skill Tree
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillNodeType {
    Passive,
    Active,
    Toggle,
}

#[derive(Debug, Clone)]
pub struct SkillNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_type: SkillNodeType,
    pub max_rank: u32,
    pub current_rank: u32,
    pub cost: u32,
    pub prerequisites: Vec<String>,
    pub effects: Vec<SkillEffect>,
}

#[derive(Debug, Clone)]
pub struct SkillEffect {
    pub stat: Stat,
    pub amount_per_rank: i32,
}

impl SkillNode {
    pub fn new(id: &str, name: &str, max_rank: u32, cost: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            node_type: SkillNodeType::Active,
            max_rank,
            current_rank: 0,
            cost,
            prerequisites: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_type(mut self, node_type: SkillNodeType) -> Self {
        self.node_type = node_type;
        self
    }

    pub fn with_prerequisite(mut self, prereq_id: &str) -> Self {
        self.prerequisites.push(prereq_id.to_string());
        self
    }

    pub fn with_effect(mut self, stat: Stat, amount_per_rank: i32) -> Self {
        self.effects.push(SkillEffect {
            stat,
            amount_per_rank,
        });
        self
    }

    pub fn can_rank_up(&self) -> bool {
        self.current_rank < self.max_rank
    }

    pub fn total_effect(&self, stat: Stat) -> i32 {
        self.effects
            .iter()
            .filter(|e| e.stat == stat)
            .map(|e| e.amount_per_rank * self.current_rank as i32)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct SkillTree {
    pub nodes: HashMap<String, SkillNode>,
}

impl SkillTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: SkillNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn can_rank_up(&self, node_id: &str, level_system: &LevelSystem) -> bool {
        let Some(node) = self.nodes.get(node_id) else {
            return false;
        };
        if !node.can_rank_up() {
            return false;
        }
        if level_system.skill_points < node.cost {
            return false;
        }
        node.prerequisites.iter().all(|prereq_id| {
            self.nodes
                .get(prereq_id)
                .map_or(false, |p| p.current_rank > 0)
        })
    }

    pub fn rank_up(&mut self, node_id: &str, level_system: &mut LevelSystem) -> bool {
        if !self.can_rank_up(node_id, level_system) {
            return false;
        }
        let Some(node) = self.nodes.get_mut(node_id) else {
            return false;
        };
        level_system.skill_points -= node.cost;
        node.current_rank += 1;
        true
    }

    pub fn active_effects(&self) -> Vec<(String, SkillEffect)> {
        self.nodes
            .values()
            .flat_map(|node| {
                node.effects
                    .iter()
                    .map(|e| (node.id.clone(), e.clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn total_stat_bonus(&self, stat: Stat) -> i32 {
        self.nodes.values().map(|n| n.total_effect(stat)).sum()
    }
}

impl Default for SkillTree {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Equipment Slots
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,
    OffHand,
    Ring,
    Amulet,
}

impl EquipmentSlot {
    pub fn all() -> &'static [EquipmentSlot] {
        &[
            EquipmentSlot::Head,
            EquipmentSlot::Chest,
            EquipmentSlot::Legs,
            EquipmentSlot::Feet,
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Ring,
            EquipmentSlot::Amulet,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EquipmentSlot::Head => "Head",
            EquipmentSlot::Chest => "Chest",
            EquipmentSlot::Legs => "Legs",
            EquipmentSlot::Feet => "Feet",
            EquipmentSlot::MainHand => "Main Hand",
            EquipmentSlot::OffHand => "Off Hand",
            EquipmentSlot::Ring => "Ring",
            EquipmentSlot::Amulet => "Amulet",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EquipmentItem {
    pub item_id: u32,
    pub name: String,
    pub slot: EquipmentSlot,
    pub stat_bonuses: HashMap<Stat, i32>,
    pub required_level: u32,
    pub durability_max: u32,
    pub durability_current: u32,
}

impl EquipmentItem {
    pub fn new(item_id: u32, name: &str, slot: EquipmentSlot) -> Self {
        Self {
            item_id,
            name: name.to_string(),
            slot,
            stat_bonuses: HashMap::new(),
            required_level: 1,
            durability_max: 100,
            durability_current: 100,
        }
    }

    pub fn with_stat_bonus(mut self, stat: Stat, amount: i32) -> Self {
        self.stat_bonuses.insert(stat, amount);
        self
    }

    pub fn with_required_level(mut self, level: u32) -> Self {
        self.required_level = level;
        self
    }

    pub fn with_durability(mut self, max: u32) -> Self {
        self.durability_max = max;
        self.durability_current = max;
        self
    }

    pub fn is_broken(&self) -> bool {
        self.durability_current == 0
    }

    pub fn durability_pct(&self) -> f64 {
        if self.durability_max == 0 {
            return 0.0;
        }
        self.durability_current as f64 / self.durability_max as f64
    }

    pub fn take_damage(&mut self, amount: u32) -> bool {
        self.durability_current = self.durability_current.saturating_sub(amount);
        self.is_broken()
    }

    pub fn repair(&mut self, amount: u32) {
        self.durability_current = (self.durability_current + amount).min(self.durability_max);
    }

    pub fn stat_bonus(&self, stat: Stat) -> i32 {
        if self.is_broken() {
            0
        } else {
            self.stat_bonuses.get(&stat).copied().unwrap_or(0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct EquipmentLoadout {
    pub slots: HashMap<EquipmentSlot, EquipmentItem>,
}

impl EquipmentLoadout {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    pub fn equip(&mut self, item: EquipmentItem) -> Option<EquipmentItem> {
        self.slots.insert(item.slot, item)
    }

    pub fn unequip(&mut self, slot: EquipmentSlot) -> Option<EquipmentItem> {
        self.slots.remove(&slot)
    }

    pub fn get(&self, slot: EquipmentSlot) -> Option<&EquipmentItem> {
        self.slots.get(&slot)
    }

    pub fn total_stat_bonus(&self, stat: Stat) -> i32 {
        self.slots.values().map(|item| item.stat_bonus(stat)).sum()
    }

    pub fn equipped_count(&self) -> usize {
        self.slots.len()
    }

    pub fn all_equipped(&self) -> Vec<&EquipmentItem> {
        self.slots.values().collect()
    }

    pub fn broken_items(&self) -> Vec<&EquipmentItem> {
        self.slots.values().filter(|i| i.is_broken()).collect()
    }

    pub fn repair_all(&mut self) {
        for item in self.slots.values_mut() {
            item.repair(item.durability_max);
        }
    }
}

impl Default for EquipmentLoadout {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Character (combines all RPG systems)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Character {
    pub name: String,
    pub stats: CharacterStats,
    pub level: LevelSystem,
    pub skill_tree: SkillTree,
    pub equipment: EquipmentLoadout,
    pub cultivation: cultivation::CultivationState,
}

impl Character {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            stats: CharacterStats::default(),
            level: LevelSystem::new(),
            skill_tree: SkillTree::new(),
            equipment: EquipmentLoadout::new(),
            cultivation: cultivation::CultivationState::new(),
        }
    }

    pub fn effective_stat(&self, stat: Stat) -> i32 {
        let base = self.stats.get(stat) as i32;
        let skill_bonus = self.skill_tree.total_stat_bonus(stat);
        let equip_bonus = self.equipment.total_stat_bonus(stat);
        base + skill_bonus + equip_bonus
    }

    pub fn hp_max(&self) -> u32 {
        let con = self.effective_stat(Stat::Con) as u32;
        let level = self.level.level;
        con * 5 + level * 10
    }

    pub fn mp_max(&self) -> u32 {
        let int = self.effective_stat(Stat::Int) as u32;
        let wis = self.effective_stat(Stat::Wis) as u32;
        (int + wis) * 3
    }

    pub fn attack_power(&self) -> u32 {
        let str_val = self.effective_stat(Stat::Str) as u32;
        let dex = self.effective_stat(Stat::Dex) as u32;
        str_val + dex / 2
    }

    pub fn defense(&self) -> u32 {
        let con = self.effective_stat(Stat::Con) as u32;
        let str_val = self.effective_stat(Stat::Str) as u32;
        con / 2 + str_val / 4
    }

    pub fn skill_check(&self, stat: Stat, difficulty: u32) -> bool {
        let modifier = self.level.level as i32 / 5;
        let effective = self.effective_stat(stat) as i32 + modifier;
        effective as u32 >= difficulty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_stats() {
        let mut stats = CharacterStats::default();
        stats.add_bonus(Stat::Str, 5);
        assert_eq!(stats.get(Stat::Str), 15);
        assert_eq!(stats.modifier(Stat::Str), 2);
    }

    #[test]
    fn test_level_xp_curve() {
        let mut ls = LevelSystem::new();
        // Level 2 requires 2^2 * 100 = 400 XP
        let levels = ls.add_xp(400);
        assert_eq!(levels, vec![2]);
        assert_eq!(ls.level, 2);
        assert!(ls.stat_points >= 5);
    }

    #[test]
    fn test_skill_tree_rank_up() {
        let mut tree = SkillTree::new();
        tree.add_node(
            SkillNode::new("power_strike", "Power Strike", 3, 1).with_effect(Stat::Str, 2),
        );
        let mut ls = LevelSystem::new();
        ls.skill_points = 3;
        assert!(tree.can_rank_up("power_strike", &ls));
        assert!(tree.rank_up("power_strike", &mut ls));
        assert_eq!(tree.total_stat_bonus(Stat::Str), 2);
    }

    #[test]
    fn test_equipment_equip_unequip() {
        let mut loadout = EquipmentLoadout::new();
        let sword = EquipmentItem::new(1, "Iron Sword", EquipmentSlot::MainHand)
            .with_stat_bonus(Stat::Str, 5);
        assert!(loadout.equip(sword).is_none());
        assert_eq!(
            loadout.get(EquipmentSlot::MainHand).unwrap().name,
            "Iron Sword"
        );
        let removed = loadout.unequip(EquipmentSlot::MainHand);
        assert!(removed.is_some());
        assert!(loadout.get(EquipmentSlot::MainHand).is_none());
    }

    #[test]
    fn test_equipment_durability() {
        let mut item = EquipmentItem::new(1, "Shield", EquipmentSlot::OffHand).with_durability(50);
        assert_eq!(item.durability_pct(), 1.0);
        item.take_damage(30);
        assert_eq!(item.durability_current, 20);
        assert!(!item.is_broken());
        item.take_damage(20);
        assert!(item.is_broken());
        assert_eq!(item.stat_bonus(Stat::Str), 0);
    }

    #[test]
    fn test_character_effective_stat() {
        let mut char = Character::new("Hero");
        char.stats.add_bonus(Stat::Str, 3);
        let sword =
            EquipmentItem::new(1, "Sword", EquipmentSlot::MainHand).with_stat_bonus(Stat::Str, 5);
        char.equipment.equip(sword);
        assert_eq!(char.effective_stat(Stat::Str), 18); // 10 base + 3 bonus + 5 equip
    }
}
