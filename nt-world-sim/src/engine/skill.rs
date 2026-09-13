use std::collections::HashMap;

// ---------------------------------------------------------------------------
// SkillType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    Basic,
    Special,
    Movement,
    Ultimate,
}

// ---------------------------------------------------------------------------
// SkillTarget
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillTarget {
    SingleEnemy,
    AllEnemies,
    Self,
    Ally,
    AllAllies,
    Area,
}

// ---------------------------------------------------------------------------
// SkillAffix
// ---------------------------------------------------------------------------

/// Prefix/suffix modifiers that add stat bonuses to skills.
#[derive(Debug, Clone)]
pub struct SkillAffix {
    pub name: String,
    pub affix_type: AffixType,
    pub bonuses: Vec<AffixBonus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffixType {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone)]
pub enum AffixBonus {
    DamageFlat(f32),
    DamagePercent(f32),
    CooldownReduction(f32),
    ManaCostReduction(f32),
    RangeIncrease(f32),
    StatBonus { stat: String, amount: f32 },
}

impl SkillAffix {
    pub fn new(name: &str, affix_type: AffixType) -> Self {
        Self { name: name.to_string(), affix_type, bonuses: Vec::new() }
    }

    pub fn with_bonus(mut self, bonus: AffixBonus) -> Self {
        self.bonuses.push(bonus);
        self
    }

    /// Calculate total damage modifier from this affix.
    pub fn damage_modifier(&self) -> f32 {
        let mut modifier = 0.0;
        for b in &self.bonuses {
            match b {
                AffixBonus::DamageFlat(f) => modifier += f,
                AffixBonus::DamagePercent(p) => modifier += p,
                _ => {}
            }
        }
        modifier
    }

    /// Calculate cooldown reduction (clamped 0..1).
    pub fn cooldown_reduction(&self) -> f32 {
        let mut reduction = 0.0;
        for b in &self.bonuses {
            if let AffixBonus::CooldownReduction(r) = b {
                reduction += r;
            }
        }
        reduction.clamp(0.0, 0.8)
    }

    /// Calculate mana cost reduction (clamped 0..1).
    pub fn mana_cost_reduction(&self) -> f32 {
        let mut reduction = 0.0;
        for b in &self.bonuses {
            if let AffixBonus::ManaCostReduction(r) = b {
                reduction += r;
            }
        }
        reduction.clamp(0.0, 0.8)
    }
}

// ---------------------------------------------------------------------------
// Skill
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub skill_type: SkillType,
    pub base_damage: f32,
    pub mp_cost: u32,
    pub base_cooldown: f32,
    pub range: f32,
    pub target: SkillTarget,
    pub effects: Vec<SkillEffect>,
    pub description: String,
    pub icon: String,
    pub level_required: u32,
    pub max_level: u32,
    pub current_level: u32,
    /// Affixes applied to this skill.
    pub affixes: Vec<SkillAffix>,
    /// Scaling per level.
    pub damage_per_level: f32,
    pub cooldown_per_level: f32,
    pub mp_cost_per_level: i32,
}

#[derive(Debug, Clone)]
pub enum SkillEffect {
    Stun { duration: f32 },
    Slow { amount: f32, duration: f32 },
    Poison { damage_per_sec: f32, duration: f32 },
    Heal(f32),
    Shield { amount: f32, duration: f32 },
    Buff { stat: String, amount: f32, duration: f32 },
    Debuff { stat: String, amount: f32, duration: f32 },
    Lifesteal(f32),
    Knockback(f32),
}

impl Skill {
    pub fn new(id: &str, name: &str, skill_type: SkillType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            skill_type,
            base_damage: 0.0,
            mp_cost: 0,
            base_cooldown: 1.0,
            range: 100.0,
            target: SkillTarget::SingleEnemy,
            effects: Vec::new(),
            description: String::new(),
            icon: format!("skills/{}.png", id),
            level_required: 1,
            max_level: 10,
            current_level: 1,
            affixes: Vec::new(),
            damage_per_level: 0.0,
            cooldown_per_level: 0.0,
            mp_cost_per_level: 0,
        }
    }

    pub fn with_damage(mut self, dmg: f32) -> Self { self.base_damage = dmg; self }
    pub fn with_mp_cost(mut self, cost: u32) -> Self { self.mp_cost = cost; self }
    pub fn with_cooldown(mut self, cd: f32) -> Self { self.base_cooldown = cd; self }
    pub fn with_range(mut self, range: f32) -> Self { self.range = range; self }
    pub fn with_target(mut self, target: SkillTarget) -> Self { self.target = target; self }
    pub fn with_description(mut self, desc: &str) -> Self { self.description = desc.to_string(); self }
    pub fn with_level_required(mut self, level: u32) -> Self { self.level_required = level; self }
    pub fn with_max_level(mut self, level: u32) -> Self { self.max_level = level; self }
    pub fn with_damage_scaling(mut self, per_level: f32) -> Self { self.damage_per_level = per_level; self }
    pub fn with_cooldown_scaling(mut self, per_level: f32) -> Self { self.cooldown_per_level = per_level; self }
    pub fn with_mp_scaling(mut self, per_level: i32) -> Self { self.mp_cost_per_level = per_level; self }

    pub fn with_effect(mut self, effect: SkillEffect) -> Self { self.effects.push(effect); self }
    pub fn with_affix(mut self, affix: SkillAffix) -> Self { self.affixes.push(affix); self }

    /// Calculate effective damage (level scaling + affixes).
    pub fn effective_damage(&self) -> f32 {
        let level_dmg = self.base_damage + self.damage_per_level * (self.current_level - 1) as f32;
        let mut total = level_dmg;
        for affix in &self.affixes {
            total += affix.damage_modifier();
        }
        total.max(0.0)
    }

    /// Calculate effective cooldown after level scaling and affixes.
    pub fn effective_cooldown(&self) -> f32 {
        let level_cd = (self.base_cooldown + self.cooldown_per_level * (self.current_level - 1) as f32).max(0.1);
        let mut reduction = 0.0f32;
        for affix in &self.affixes {
            reduction += affix.cooldown_reduction();
        }
        (level_cd * (1.0 - reduction)).max(0.05)
    }

    /// Calculate effective mana cost after level scaling and affixes.
    pub fn effective_mp_cost(&self) -> u32 {
        let level_mp = (self.mp_cost as i32 + self.mp_cost_per_level * (self.current_level - 1) as i32).max(0);
        let mut reduction = 0.0f32;
        for affix in &self.affixes {
            reduction += affix.mana_cost_reduction();
        }
        (level_mp as f32 * (1.0 - reduction)).max(0.0) as u32
    }

    /// Check if skill can be leveled up.
    pub fn can_level_up(&self) -> bool {
        self.current_level < self.max_level
    }

    /// Level up the skill. Returns true if successful.
    pub fn level_up(&mut self) -> bool {
        if self.can_level_up() {
            self.current_level += 1;
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// SkillCooldowns
// ---------------------------------------------------------------------------

/// Tracks cooldown timers for all skills.
pub struct SkillCooldowns {
    cooldowns: HashMap<String, f32>,
}

impl SkillCooldowns {
    pub fn new() -> Self {
        Self { cooldowns: HashMap::new() }
    }

    /// Start cooldown for a skill.
    pub fn start_cooldown(&mut self, skill_id: &str, duration: f32) {
        self.cooldowns.insert(skill_id.to_string(), duration);
    }

    /// Check if a skill is on cooldown.
    pub fn is_on_cooldown(&self, skill_id: &str) -> bool {
        self.cooldowns.get(skill_id).map_or(false, |cd| *cd > 0.0)
    }

    /// Get remaining cooldown time.
    pub fn remaining(&self, skill_id: &str) -> f32 {
        self.cooldowns.get(skill_id).copied().unwrap_or(0.0)
    }

    /// Progress all cooldowns by dt.
    pub fn tick(&mut self, dt: f32) {
        for cd in self.cooldowns.values_mut() {
            if *cd > 0.0 {
                *cd = (*cd - dt).max(0.0);
            }
        }
        self.cooldowns.retain(|_, cd| *cd > 0.0);
    }

    /// Reset a specific cooldown.
    pub fn reset(&mut self, skill_id: &str) {
        self.cooldowns.remove(skill_id);
    }

    /// Reset all cooldowns.
    pub fn reset_all(&mut self) {
        self.cooldowns.clear();
    }
}

impl Default for SkillCooldowns {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SkillManager
// ---------------------------------------------------------------------------

/// Manages learned skills, their levels, and cooldown tracking.
pub struct SkillManager {
    skills: HashMap<String, Skill>,
    cooldowns: SkillCooldowns,
    skill_points: u32,
}

impl SkillManager {
    pub fn new() -> Self {
        Self { skills: HashMap::new(), cooldowns: SkillCooldowns::new(), skill_points: 0 }
    }

    /// Learn a new skill.
    pub fn learn(&mut self, skill: Skill) -> bool {
        if self.skills.contains_key(&skill.id) {
            return false;
        }
        self.skills.insert(skill.id.clone(), skill);
        true
    }

    /// Forget a skill.
    pub fn forget(&mut self, skill_id: &str) -> Option<Skill> {
        self.skills.remove(skill_id)
    }

    /// Get a skill by id.
    pub fn get(&self, skill_id: &str) -> Option<&Skill> {
        self.skills.get(skill_id)
    }

    /// Get a mutable skill by id.
    pub fn get_mut(&mut self, skill_id: &str) -> Option<&mut Skill> {
        self.skills.get_mut(skill_id)
    }

    /// Use a skill (check cooldown + MP cost). Returns effective cost.
    pub fn use_skill(&mut self, skill_id: &str, current_mp: u32) -> Option<u32> {
        let skill = self.skills.get(skill_id)?;
        let cost = skill.effective_mp_cost();

        if self.cooldowns.is_on_cooldown(skill_id) {
            return None;
        }
        if current_mp < cost {
            return None;
        }

        self.cooldowns.start_cooldown(skill_id, skill.effective_cooldown());
        Some(cost)
    }

    /// Check if a skill can be used (off cooldown and MP check).
    pub fn can_use(&self, skill_id: &str, current_mp: u32) -> bool {
        if let Some(skill) = self.skills.get(skill_id) {
            !self.cooldowns.is_on_cooldown(skill_id) && current_mp >= skill.effective_mp_cost()
        } else {
            false
        }
    }

    /// Upgrade a skill level (costs 1 skill point).
    pub fn upgrade(&mut self, skill_id: &str) -> bool {
        if self.skill_points == 0 { return false; }
        if let Some(skill) = self.skills.get_mut(skill_id) {
            if skill.level_up() {
                self.skill_points -= 1;
                return true;
            }
        }
        false
    }

    /// Add skill points.
    pub fn add_skill_points(&mut self, points: u32) {
        self.skill_points += points;
    }

    /// Remaining skill points.
    pub fn skill_points(&self) -> u32 {
        self.skill_points
    }

    /// Progress all cooldowns.
    pub fn tick(&mut self, dt: f32) {
        self.cooldowns.tick(dt);
    }

    /// Get all learned skills.
    pub fn all_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// Get skills of a specific type.
    pub fn skills_by_type(&self, skill_type: SkillType) -> Vec<&Skill> {
        self.skills.values().filter(|s| s.skill_type == skill_type).collect()
    }

    /// Get usable skills (off cooldown).
    pub fn usable_skills(&self, current_mp: u32) -> Vec<&Skill> {
        self.skills.values()
            .filter(|s| self.can_use(&s.id, current_mp))
            .collect()
    }

    /// Total learned skill count.
    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fireball() -> Skill {
        Skill::new("fireball", "Fireball", SkillType::Basic)
            .with_damage(25.0)
            .with_mp_cost(10)
            .with_cooldown(2.0)
            .with_range(150.0)
            .with_effect(SkillEffect::Poison { damage_per_sec: 3.0, duration: 3.0 })
    }

    fn ultimate_strike() -> Skill {
        Skill::new("ult", "Ultimate Strike", SkillType::Ultimate)
            .with_damage(200.0)
            .with_mp_cost(80)
            .with_cooldown(30.0)
            .with_max_level(5)
            .with_damage_scaling(20.0)
    }

    #[test]
    fn test_skill_basics() {
        let s = fireball();
        assert_eq!(s.effective_damage(), 25.0);
        assert_eq!(s.effective_mp_cost(), 10);
        assert!((s.effective_cooldown() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_skill_level_up() {
        let mut s = ultimate_strike();
        assert_eq!(s.current_level, 1);
        assert!(s.level_up());
        assert_eq!(s.current_level, 2);
        assert!((s.effective_damage() - 245.0).abs() < 0.01);
        for _ in 0..8 { s.level_up(); }
        assert!(!s.level_up()); // max level
    }

    #[test]
    fn test_skill_affixes() {
        let s = fireball().with_affix(
            SkillAffix::new("Vicious", AffixType::Prefix)
                .with_bonus(AffixBonus::DamagePercent(10.0))
                .with_bonus(AffixBonus::CooldownReduction(0.1))
        );
        assert!((s.effective_damage() - 35.0).abs() < 0.01);
        assert!((s.effective_cooldown() - 1.8).abs() < 0.01);
    }

    #[test]
    fn test_cooldowns() {
        let mut cd = SkillCooldowns::new();
        assert!(!cd.is_on_cooldown("fb"));
        cd.start_cooldown("fb", 2.0);
        assert!(cd.is_on_cooldown("fb"));
        assert!((cd.remaining("fb") - 2.0).abs() < 0.01);
        cd.tick(1.5);
        assert!((cd.remaining("fb") - 0.5).abs() < 0.01);
        cd.tick(1.0);
        assert!(!cd.is_on_cooldown("fb"));
    }

    #[test]
    fn test_skill_manager() {
        let mut mgr = SkillManager::new();
        assert!(mgr.learn(fireball()));
        assert!(!mgr.learn(fireball())); // duplicate
        assert_eq!(mgr.skill_count(), 1);

        // Use skill
        let cost = mgr.use_skill("fireball", 50).unwrap();
        assert_eq!(cost, 10);
        assert!(mgr.cooldowns.is_on_cooldown("fireball"));
        assert!(mgr.use_skill("fireball", 50).is_none()); // on cooldown

        // Cooldown tick
        mgr.tick(2.5);
        assert!(mgr.can_use("fireball", 50));
    }

    #[test]
    fn test_upgrade() {
        let mut mgr = SkillManager::new();
        mgr.learn(ultimate_strike());
        mgr.add_skill_points(3);
        assert!(mgr.upgrade("ult"));
        assert_eq!(mgr.get("ult").unwrap().current_level, 2);
        assert_eq!(mgr.skill_points(), 2);
        assert!(!mgr.upgrade("nonexistent"));
    }

    #[test]
    fn test_mp_insufficient() {
        let mut mgr = SkillManager::new();
        mgr.learn(fireball());
        assert!(mgr.use_skill("fireball", 5).is_none()); // need 10
    }

    #[test]
    fn test_skill_effects() {
        let s = fireball();
        assert_eq!(s.effects.len(), 1);
        match &s.effects[0] {
            SkillEffect::Poison { damage_per_sec, duration } => {
                assert!((*damage_per_sec - 3.0).abs() < 0.01);
                assert!((*duration - 3.0).abs() < 0.01);
            }
            _ => panic!("expected poison"),
        }
    }

    #[test]
    fn test_skills_by_type() {
        let mut mgr = SkillManager::new();
        mgr.learn(fireball());
        mgr.learn(Skill::new("slash", "Slash", SkillType::Basic).with_damage(15.0));
        mgr.learn(Skill::new("blink", "Blink", SkillType::Movement));
        assert_eq!(mgr.skills_by_type(SkillType::Basic).len(), 2);
        assert_eq!(mgr.skills_by_type(SkillType::Movement).len(), 1);
    }
}
