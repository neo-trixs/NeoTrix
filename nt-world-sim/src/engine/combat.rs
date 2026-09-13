use std::collections::HashMap;

use super::renderer::Vec2;
use super::skill::{SkillManager, SkillEffect};

// ---------------------------------------------------------------------------
// CombatEntity
// ---------------------------------------------------------------------------

/// Base stats for anything that can participate in combat.
#[derive(Debug, Clone)]
pub struct CombatEntity {
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
    pub mp: f32,
    pub max_mp: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
    pub crit_rate: f32,
    pub crit_damage: f32,
    pub level: u32,
    pub position: Vec2,
    pub skills: SkillManager,
    pub status_effects: Vec<StatusEffect>,
}

impl CombatEntity {
    pub fn new(name: &str, hp: f32) -> Self {
        Self {
            name: name.to_string(),
            hp, max_hp: hp,
            mp: 50.0, max_mp: 50.0,
            attack: 10.0, defense: 5.0,
            speed: 100.0, crit_rate: 0.05,
            crit_damage: 1.5,
            level: 1,
            position: Vec2::zero(),
            skills: SkillManager::new(),
            status_effects: Vec::new(),
        }
    }

    pub fn with_mp(mut self, mp: f32) -> Self { self.mp = mp; self.max_mp = mp; self }
    pub fn with_attack(mut self, atk: f32) -> Self { self.attack = atk; self }
    pub fn with_defense(mut self, def: f32) -> Self { self.defense = def; self }
    pub fn with_speed(mut self, spd: f32) -> Self { self.speed = spd; self }
    pub fn with_crit(mut self, rate: f32, damage: f32) -> Self { self.crit_rate = rate; self.crit_damage = damage; self }
    pub fn with_level(mut self, level: u32) -> Self { self.level = level; self }
    pub fn with_position(mut self, x: f32, y: f32) -> Self { self.position = Vec2::new(x, y); self }

    /// Is this entity alive?
    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
    }

    /// HP percentage (0.0..1.0).
    pub fn hp_pct(&self) -> f32 {
        self.hp / self.max_hp
    }

    /// Take raw damage. Returns actual damage dealt.
    pub fn take_damage(&mut self, raw: DamageResult) -> f32 {
        let actual = raw.final_damage;
        self.hp = (self.hp - actual).max(0.0);
        actual
    }

    /// Heal. Returns amount actually healed.
    pub fn heal(&mut self, amount: f32) -> f32 {
        let before = self.hp;
        self.hp = (self.hp + amount).min(self.max_hp);
        self.hp - before
    }

    /// Restore MP. Returns amount restored.
    pub fn restore_mp(&mut self, amount: f32) -> f32 {
        let before = self.mp;
        self.mp = (self.mp + amount).min(self.max_mp);
        self.mp - before
    }

    /// Consume MP. Returns true if successful.
    pub fn consume_mp(&mut self, amount: u32) -> bool {
        if self.mp >= amount as f32 {
            self.mp -= amount as f32;
            true
        } else {
            false
        }
    }

    /// Add a status effect.
    pub fn apply_status(&mut self, effect: StatusEffect) {
        self.status_effects.push(effect);
    }

    /// Process status effects (tick damage, check expiry).
    pub fn tick_status_effects(&mut self, dt: f32) -> Vec<StatusEvent> {
        let mut events = Vec::new();
        let mut to_remove = Vec::new();

        for (i, effect) in self.status_effects.iter_mut().enumerate() {
            match effect {
                StatusEffect::Poison { damage_per_sec, remaining, .. } => {
                    let tick_dmg = *damage_per_sec * dt;
                    self.hp = (self.hp - tick_dmg).max(0.0);
                    *remaining -= dt;
                    if *remaining <= 0.0 { to_remove.push(i); events.push(StatusEvent::PoisonExpired(self.name.clone())); }
                    else { events.push(StatusEvent::PoisonTick(self.name.clone(), tick_dmg)); }
                }
                StatusEffect::Regen { heal_per_sec, remaining, .. } => {
                    let tick_heal = *heal_per_sec * dt;
                    self.hp = (self.hp + tick_heal).min(self.max_hp);
                    *remaining -= dt;
                    if *remaining <= 0.0 { to_remove.push(i); }
                }
                StatusEffect::Stun { remaining } => {
                    *remaining -= dt;
                    if *remaining <= 0.0 { to_remove.push(i); events.push(StatusEvent::StunExpired(self.name.clone())); }
                }
                StatusEffect::Slow { remaining, .. } => {
                    *remaining -= dt;
                    if *remaining <= 0.0 { to_remove.push(i); }
                }
                StatusEffect::Shield { remaining, .. } => {
                    *remaining -= dt;
                    if *remaining <= 0.0 { to_remove.push(i); }
                }
                StatusEffect::Buff { remaining, .. } => {
                    *remaining -= dt;
                    if *remaining <= 0.0 { to_remove.push(i); events.push(StatusEvent::BuffExpired(self.name.clone())); }
                }
                StatusEffect::Debuff { remaining, .. } => {
                    *remaining -= dt;
                    if *remaining <= 0.0 { to_remove.push(i); }
                }
            }
        }
        for i in to_remove.into_iter().rev() {
            self.status_effects.remove(i);
        }
        events
    }

    /// Check if stunned.
    pub fn is_stunned(&self) -> bool {
        self.status_effects.iter().any(|e| matches!(e, StatusEffect::Stun { remaining } if *remaining > 0.0))
    }

    /// Get speed multiplier (accounting for slows).
    pub fn speed_multiplier(&self) -> f32 {
        let mut mult = 1.0f32;
        for effect in &self.status_effects {
            if let StatusEffect::Slow { amount, remaining } = effect {
                if *remaining > 0.0 { mult *= (1.0 - amount).max(0.1); }
            }
        }
        mult
    }

    /// Get effective attack (accounting for buffs/debuffs).
    pub fn effective_attack(&self) -> f32 {
        let mut atk = self.attack;
        for effect in &self.status_effects {
            match effect {
                StatusEffect::Buff { stat, amount, remaining, .. } if *remaining > 0.0 && stat == "attack" => atk += amount,
                StatusEffect::Debuff { stat, amount, remaining, .. } if *remaining > 0.0 && stat == "attack" => atk -= amount,
                _ => {}
            }
        }
        atk.max(0.0)
    }

    /// Get effective defense.
    pub fn effective_defense(&self) -> f32 {
        let mut def = self.defense;
        for effect in &self.status_effects {
            match effect {
                StatusEffect::Shield { amount, remaining, .. } if *remaining > 0.0 => def += amount,
                StatusEffect::Debuff { stat, amount, remaining, .. } if *remaining > 0.0 && stat == "defense" => def -= amount,
                _ => {}
            }
        }
        def.max(0.0)
    }
}

// ---------------------------------------------------------------------------
// StatusEffect
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum StatusEffect {
    Poison { damage_per_sec: f32, remaining: f32, source: String },
    Regen { heal_per_sec: f32, remaining: f32, source: String },
    Stun { remaining: f32 },
    Slow { amount: f32, remaining: f32 },
    Shield { amount: f32, remaining: f32 },
    Buff { stat: String, amount: f32, remaining: f32, source: String },
    Debuff { stat: String, amount: f32, remaining: f32, source: String },
}

#[derive(Debug, Clone)]
pub enum StatusEvent {
    PoisonTick(String, f32),
    PoisonExpired(String),
    StunExpired(String),
    BuffExpired(String),
}

// ---------------------------------------------------------------------------
// Damage calculation
// ---------------------------------------------------------------------------

/// Input parameters for damage calculation.
#[derive(Debug, Clone)]
pub struct DamageInput {
    pub base_damage: f32,
    pub attack: f32,
    pub defense: f32,
    pub crit_rate: f32,
    pub crit_damage: f32,
    pub level: u32,
    pub target_level: u32,
    pub random_factor: f32,
}

/// Result of a damage calculation.
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub final_damage: f32,
    pub is_crit: bool,
    pub damage_before_defense: f32,
    pub raw_damage: f32,
}

/// Calculate damage between attacker and defender.
pub fn calculate_damage(input: DamageInput) -> DamageResult {
    let is_crit = pseudo_random() < input.crit_rate;
    let crit_mult = if is_crit { input.crit_damage } else { 1.0 };

    // Base damage = base + attack * multiplier
    let raw_damage = (input.base_damage + input.attack * 0.5) * crit_mult;

    // Level scaling
    let level_diff = (input.level as f32 - input.target_level as f32).max(0.0);
    let level_mult = 1.0 + level_diff * 0.02;

    // Defense reduction: damage * (100 / (100 + defense))
    let defense_mult = 100.0 / (100.0 + input.defense);

    let damage_before_defense = raw_damage * level_mult;
    let final_damage = (damage_before_defense * defense_mult * input.random_factor).max(1.0);

    DamageResult { final_damage, is_crit, damage_before_defense, raw_damage }
}

/// Simpler damage calculation between two entities.
pub fn calc_combat_damage(attacker: &CombatEntity, defender: &CombatEntity, skill_damage: f32) -> DamageResult {
    calculate_damage(DamageInput {
        base_damage: skill_damage,
        attack: attacker.effective_attack(),
        defense: defender.effective_defense(),
        crit_rate: attacker.crit_rate,
        crit_damage: attacker.crit_damage,
        level: attacker.level,
        target_level: defender.level,
        random_factor: 0.9 + pseudo_random() * 0.2,
    })
}

// ---------------------------------------------------------------------------
// CombatState
// ---------------------------------------------------------------------------

/// Tracks an active combat encounter.
pub struct CombatState {
    pub in_combat: bool,
    pub target: Option<String>,
    pub turn_order: Vec<String>,
    pub current_turn: usize,
    pub turn_count: u32,
    pub skill_queue: Vec<SkillCast>,
    pub combat_log: Vec<CombatLogEntry>,
    pub elapsed: f32,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            in_combat: false,
            target: None,
            turn_order: Vec::new(),
            current_turn: 0,
            turn_count: 0,
            skill_queue: Vec::new(),
            combat_log: Vec::new(),
            elapsed: 0.0,
        }
    }

    /// Start combat with a target.
    pub fn start_combat(&mut self, target_id: &str) {
        self.in_combat = true;
        self.target = Some(target_id.to_string());
        self.turn_count = 0;
        self.combat_log.clear();
    }

    /// End combat.
    pub fn end_combat(&mut self) {
        self.in_combat = false;
        self.target = None;
        self.turn_order.clear();
        self.current_turn = 0;
        self.skill_queue.clear();
    }

    /// Queue a skill cast.
    pub fn queue_skill(&mut self, caster_id: &str, skill_id: &str) {
        self.skill_queue.push(SkillCast {
            caster_id: caster_id.to_string(),
            skill_id: skill_id.to_string(),
        });
    }

    /// Drain skill queue.
    pub fn drain_skill_queue(&mut self) -> Vec<SkillCast> {
        std::mem::take(&mut self.skill_queue)
    }

    /// Add combat log entry.
    pub fn log(&mut self, entry: CombatLogEntry) {
        self.combat_log.push(entry);
    }

    /// Get combat log.
    pub fn log_entries(&self) -> &[CombatLogEntry] {
        &self.combat_log
    }

    pub fn tick(&mut self, dt: f32) {
        if self.in_combat {
            self.elapsed += dt;
        }
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SkillCast {
    pub caster_id: String,
    pub skill_id: String,
}

#[derive(Debug, Clone)]
pub struct CombatLogEntry {
    pub turn: u32,
    pub attacker: String,
    pub defender: String,
    pub skill: String,
    pub damage: f32,
    pub is_crit: bool,
    pub message: String,
}

// ---------------------------------------------------------------------------
// AutoCombatAI
// ---------------------------------------------------------------------------

/// Simple auto-combat AI that selects targets and uses skills.
pub struct AutoCombatAI {
    pub aggro_range: f32,
    pub flee_hp_threshold: f32,
    pub preferred_skills: Vec<String>,
}

impl AutoCombatAI {
    pub fn new() -> Self {
        Self {
            aggro_range: 200.0,
            flee_hp_threshold: 0.2,
            preferred_skills: Vec::new(),
        }
    }

    pub fn with_aggro_range(mut self, range: f32) -> Self { self.aggro_range = range; self }
    pub fn with_flee_threshold(mut self, threshold: f32) -> Self { self.flee_hp_threshold = threshold; self }
    pub fn with_preferred_skills(mut self, skills: Vec<String>) -> Self { self.preferred_skills = skills; self }

    /// Select the best skill to use.
    pub fn select_skill(&self, entity: &CombatEntity) -> Option<String> {
        if entity.is_stunned() { return None; }

        // Check preferred skills first
        for skill_id in &self.preferred_skills {
            if entity.skills.can_use(skill_id, entity.mp as u32) {
                return Some(skill_id.clone());
            }
        }

        // Fall back to any usable skill
        entity.skills.usable_skills(entity.mp as u32)
            .into_iter()
            .max_by(|a, b| a.effective_damage().partial_cmp(&b.effective_damage()).unwrap())
            .map(|s| s.id.clone())
    }

    /// Decide whether to flee.
    pub fn should_flee(&self, entity: &CombatEntity) -> bool {
        entity.hp_pct() < self.flee_hp_threshold
    }

    /// Find nearest hostile within aggro range.
    pub fn find_target(&self, entity: &CombatEntity, hostiles: &[&CombatEntity]) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, hostile) in hostiles.iter().enumerate() {
            if !hostile.is_alive() { continue; }
            let dx = entity.position.x - hostile.position.x;
            let dy = entity.position.y - hostile.position.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= self.aggro_range {
                match best {
                    None => best = Some((i, dist)),
                    Some((_, best_dist)) if dist < best_dist => best = Some((i, dist)),
                    _ => {}
                }
            }
        }
        best.map(|(i, _)| i)
    }

    /// Full AI tick: returns an action decision.
    pub fn decide(&self, entity: &CombatEntity, hostiles: &[&CombatEntity]) -> AIDecision {
        if !entity.is_alive() {
            return AIDecision::Dead;
        }

        if self.should_flee(entity) {
            return AIDecision::Flee;
        }

        if let Some(skill_id) = self.select_skill(entity) {
            if let Some(target_idx) = self.find_target(entity, hostiles) {
                return AIDecision::UseSkill {
                    skill_id,
                    target_idx,
                };
            }
        }

        if self.find_target(entity, hostiles).is_some() {
            AIDecision::Chase
        } else {
            AIDecision::Idle
        }
    }
}

impl Default for AutoCombatAI {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum AIDecision {
    Idle,
    Chase,
    UseSkill { skill_id: String, target_idx: usize },
    Flee,
    Dead,
}

// ---------------------------------------------------------------------------
// CombatManager
// ---------------------------------------------------------------------------

/// High-level combat manager that coordinates entities and auto-combat.
pub struct CombatManager {
    pub state: CombatState,
    pub ai: AutoCombatAI,
    pub entities: HashMap<String, CombatEntity>,
    pub loot_table: LootTable,
    pub event_log: Vec<String>,
}

impl CombatManager {
    pub fn new() -> Self {
        Self {
            state: CombatState::new(),
            ai: AutoCombatAI::new(),
            entities: HashMap::new(),
            loot_table: LootTable::new(),
            event_log: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: CombatEntity) {
        let name = entity.name.clone();
        self.entities.insert(name, entity);
    }

    pub fn get_entity(&self, id: &str) -> Option<&CombatEntity> {
        self.entities.get(id)
    }

    pub fn get_entity_mut(&mut self, id: &str) -> Option<&mut CombatEntity> {
        self.entities.get_mut(id)
    }

    /// Execute an attack between two entities.
    pub fn execute_attack(&mut self, attacker_id: &str, defender_id: &str) -> Option<DamageResult> {
        let attacker = self.entities.get(attacker_id)?.clone();
        let defender = self.entities.get(defender_id)?.clone();

        if !attacker.is_alive() || !defender.is_alive() { return None; }
        if attacker.is_stunned() { return None; }

        let result = calc_combat_damage(&attacker, &defender, attacker.effective_attack());

        if let Some(d) = self.entities.get_mut(defender_id) {
            d.take_damage(result.clone());
        }

        let msg = format!(
            "{} attacks {} for {:.0} damage{}!",
            attacker.name, defender.name, result.final_damage,
            if result.is_crit { " (CRIT)" } else { "" }
        );
        self.event_log.push(msg.clone());
        self.state.log(CombatLogEntry {
            turn: self.state.turn_count,
            attacker: attacker.name.clone(),
            defender: defender.name.clone(),
            skill: "basic_attack".into(),
            damage: result.final_damage,
            is_crit: result.is_crit,
            message: msg,
        });

        Some(result)
    }

    /// Execute a skill between two entities.
    pub fn execute_skill(&mut self, attacker_id: &str, defender_id: &str, skill_id: &str) -> Option<DamageResult> {
        // Clone attacker to avoid borrow conflicts
        let mut attacker = self.entities.get(attacker_id)?.clone();
        let defender = self.entities.get(defender_id)?.clone();

        if !attacker.is_alive() || !defender.is_alive() { return None; }
        if attacker.is_stunned() { return None; }

        // Check if skill can be used and get cost
        let skill = attacker.skills.get(skill_id)?.clone();
        let cost = attacker.skills.use_skill(skill_id, attacker.mp as u32)?;

        // Apply MP cost
        if let Some(a) = self.entities.get_mut(attacker_id) {
            a.consume_mp(cost);
        }

        let result = calc_combat_damage(&attacker, &defender, skill.effective_damage());

        // Apply damage
        if let Some(d) = self.entities.get_mut(defender_id) {
            d.take_damage(result.clone());
        }

        // Apply skill effects
        let attacker_name = attacker.name.clone();
        let defender_name = defender.name.clone();
        let skill_name = skill.name.clone();
        let skill_effects = skill.effects.clone();

        for effect in &skill_effects {
            match effect {
                SkillEffect::Stun { duration } => {
                    if let Some(d) = self.entities.get_mut(defender_id) {
                        d.apply_status(StatusEffect::Stun { remaining: *duration });
                    }
                }
                SkillEffect::Slow { amount, duration } => {
                    if let Some(d) = self.entities.get_mut(defender_id) {
                        d.apply_status(StatusEffect::Slow { amount: *amount, remaining: *duration });
                    }
                }
                SkillEffect::Poison { damage_per_sec, duration } => {
                    if let Some(d) = self.entities.get_mut(defender_id) {
                        d.apply_status(StatusEffect::Poison {
                            damage_per_sec: *damage_per_sec,
                            remaining: *duration,
                            source: attacker_name.clone(),
                        });
                    }
                }
                SkillEffect::Heal(amount) => {
                    if let Some(a) = self.entities.get_mut(attacker_id) {
                        a.heal(*amount);
                    }
                }
                SkillEffect::Shield { amount, duration } => {
                    if let Some(a) = self.entities.get_mut(attacker_id) {
                        a.apply_status(StatusEffect::Shield { amount: *amount, remaining: *duration });
                    }
                }
                SkillEffect::Buff { stat, amount, duration } => {
                    if let Some(a) = self.entities.get_mut(attacker_id) {
                        a.apply_status(StatusEffect::Buff {
                            stat: stat.clone(),
                            amount: *amount,
                            remaining: *duration,
                            source: attacker_name.clone(),
                        });
                    }
                }
                SkillEffect::Debuff { stat, amount, duration } => {
                    if let Some(d) = self.entities.get_mut(defender_id) {
                        d.apply_status(StatusEffect::Debuff {
                            stat: stat.clone(),
                            amount: *amount,
                            remaining: *duration,
                            source: attacker_name.clone(),
                        });
                    }
                }
                SkillEffect::Lifesteal(pct) => {
                    let heal = result.final_damage * pct;
                    if let Some(a) = self.entities.get_mut(attacker_id) {
                        a.heal(heal);
                    }
                }
                SkillEffect::Knockback(force) => {
                    let attacker_pos = self.entities.get(attacker_id).map(|a| a.position);
                    if let (Some(d), Some(a_pos)) = (
                        self.entities.get_mut(defender_id),
                        attacker_pos
                    ) {
                        let dx = d.position.x - a_pos.x;
                        let dy = d.position.y - a_pos.y;
                        let len = (dx * dx + dy * dy).sqrt().max(1.0);
                        d.position.x += dx / len * force;
                        d.position.y += dy / len * force;
                    }
                }
            }
        }

        let msg = format!(
            "{} uses {} on {} for {:.0} damage{}!",
            attacker_name, skill_name, defender_name, result.final_damage,
            if result.is_crit { " (CRIT)" } else { "" }
        );
        self.event_log.push(msg.clone());
        self.state.log(CombatLogEntry {
            turn: self.state.turn_count,
            attacker: attacker_name,
            defender: defender_name,
            skill: skill_name,
            damage: result.final_damage,
            is_crit: result.is_crit,
            message: msg,
        });

        Some(result)
    }

    /// Auto-combat tick: AI decides actions for all entities.
    pub fn auto_tick(&mut self, dt: f32) {
        self.state.tick(dt);
        if !self.state.in_combat { return; }

        let alive: Vec<String> = self.entities.iter()
            .filter(|(_, e)| e.is_alive())
            .map(|(k, _)| k.clone())
            .collect();

        for id in &alive {
            let entity = self.entities.get(id).cloned().unwrap();
            let hostiles: Vec<&CombatEntity> = self.entities.values()
                .filter(|e| e.is_alive() && e.name != entity.name)
                .collect();

            match self.ai.decide(&entity, &hostiles) {
                AIDecision::UseSkill { skill_id, target_idx } => {
                    let target_name = hostiles.get(target_idx).map(|e| e.name.clone());
                    if let Some(target) = target_name {
                        self.execute_skill(id, &target, &skill_id);
                    }
                }
                AIDecision::Flee => {
                    self.event_log.push(format!("{} attempts to flee!", entity.name));
                }
                _ => {}
            }

            // Tick status effects
            if let Some(e) = self.entities.get_mut(id) {
                let events = e.tick_status_effects(dt);
                for event in &events {
                    self.event_log.push(format!("{:?}", event));
                }
            }
        }
    }

    /// Roll loot from loot table.
    pub fn roll_loot(&self) -> Vec<LootDrop> {
        self.loot_table.roll()
    }
}

impl Default for CombatManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LootTable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LootEntry {
    pub item_id: String,
    pub count_min: u32,
    pub count_max: u32,
    pub drop_rate: f32,
}

pub struct LootTable {
    pub entries: Vec<LootEntry>,
}

impl LootTable {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: LootEntry) {
        self.entries.push(entry);
    }

    pub fn roll(&self) -> Vec<LootDrop> {
        self.entries.iter().filter_map(|e| {
            if pseudo_random() < e.drop_rate {
                let count = e.count_min + (pseudo_random() * (e.count_max - e.count_min) as f32) as u32;
                Some(LootDrop { item_id: e.item_id.clone(), count })
            } else {
                None
            }
        }).collect()
    }
}

impl Default for LootTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LootDrop {
    pub item_id: String,
    pub count: u32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pseudo_random() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_entity_basics() {
        let mut e = CombatEntity::new("Hero", 200.0).with_mp(100.0);
        assert!(e.is_alive());
        assert!((e.hp_pct() - 1.0).abs() < 0.01);
        e.take_damage(DamageResult { final_damage: 50.0, is_crit: false, damage_before_defense: 50.0, raw_damage: 50.0 });
        assert!((e.hp - 150.0).abs() < 0.01);
        e.heal(30.0);
        assert!((e.hp - 180.0).abs() < 0.01);
    }

    #[test]
    fn test_damage_calculation() {
        let result = calculate_damage(DamageInput {
            base_damage: 20.0, attack: 15.0, defense: 5.0,
            crit_rate: 0.0, crit_damage: 1.5,
            level: 5, target_level: 5,
            random_factor: 1.0,
        });
        assert!(result.final_damage > 0.0);
        assert!(!result.is_crit); // 0% crit rate
    }

    #[test]
    fn test_damage_always_at_least_1() {
        let result = calculate_damage(DamageInput {
            base_damage: 0.0, attack: 0.0, defense: 999.0,
            crit_rate: 0.0, crit_damage: 1.5,
            level: 1, target_level: 99,
            random_factor: 0.9,
        });
        assert!(result.final_damage >= 1.0);
    }

    #[test]
    fn test_status_effects() {
        let mut e = CombatEntity::new("Test", 100.0);
        e.apply_status(StatusEffect::Poison { damage_per_sec: 10.0, remaining: 2.0, source: "poison".into() });
        let events = e.tick_status_effects(1.0);
        assert_eq!(events.len(), 1);
        assert!((e.hp - 90.0).abs() < 0.01);

        let events = e.tick_status_effects(1.5);
        assert!(!events.is_empty());
        assert!((e.hp - 75.0).abs() < 0.01); // 10 * 0.5 remaining
    }

    #[test]
    fn test_stun_prevents_action() {
        let mut e = CombatEntity::new("Stunned", 100.0);
        e.apply_status(StatusEffect::Stun { remaining: 2.0 });
        assert!(e.is_stunned());
        e.tick_status_effects(2.5);
        assert!(!e.is_stunned());
    }

    #[test]
    fn test_ai_flee() {
        let mut ai = AutoCombatAI::new().with_flee_threshold(0.3);
        let mut entity = CombatEntity::new("Weak", 100.0);
        entity.hp = 20.0; // 20%
        assert!(ai.should_flee(&entity));
    }

    #[test]
    fn test_ai_target_selection() {
        let ai = AutoCombatAI::new().with_aggro_range(100.0);
        let mut entity = CombatEntity::new("Player", 100.0).with_position(0.0, 0.0);
        let mut enemy1 = CombatEntity::new("Goblin", 50.0).with_position(50.0, 0.0);
        let mut enemy2 = CombatEntity::new("Orc", 50.0).with_position(10.0, 0.0);

        let hostiles: Vec<&CombatEntity> = vec![&enemy1, &enemy2];
        let idx = ai.find_target(&entity, &hostiles);
        assert_eq!(idx, Some(1)); // Orc is closer
    }

    #[test]
    fn test_combat_manager() {
        let mut mgr = CombatManager::new();
        mgr.add_entity(CombatEntity::new("Hero", 200.0).with_attack(20.0).with_position(0.0, 0.0));
        mgr.add_entity(CombatEntity::new("Goblin", 50.0).with_defense(2.0).with_position(50.0, 0.0));
        mgr.state.start_combat("Goblin");

        let result = mgr.execute_attack("Hero", "Goblin").unwrap();
        assert!(result.final_damage > 0.0);
        assert!(mgr.get_entity("Goblin").unwrap().hp < 50.0);
    }

    #[test]
    fn test_loot_table() {
        let mut table = LootTable::new();
        table.add_entry(LootEntry { item_id: "gold".into(), count_min: 5, count_max: 10, drop_rate: 1.0 });
        table.add_entry(LootEntry { item_id: "sword".into(), count_min: 1, count_max: 1, drop_rate: 0.0 });
        let loot = table.roll();
        assert!(loot.iter().any(|d| d.item_id == "gold"));
    }
}
