use std::collections::HashMap;

use super::Stat;

// ═══════════════════════════════════════════════════════════════════
// Cultivation Realms
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CultivationRealm {
    QiRefining,
    Foundation,
    CoreFormation,
    NascentSoul,
    SpiritSevering,
    DaoSeeking,
    ImmortalAscension,
}

impl CultivationRealm {
    pub fn all() -> &'static [CultivationRealm] {
        &[
            CultivationRealm::QiRefining,
            CultivationRealm::Foundation,
            CultivationRealm::CoreFormation,
            CultivationRealm::NascentSoul,
            CultivationRealm::SpiritSevering,
            CultivationRealm::DaoSeeking,
            CultivationRealm::ImmortalAscension,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CultivationRealm::QiRefining => "Qi Refining",
            CultivationRealm::Foundation => "Foundation Establishment",
            CultivationRealm::CoreFormation => "Core Formation",
            CultivationRealm::NascentSoul => "Nascent Soul",
            CultivationRealm::SpiritSevering => "Spirit Severing",
            CultivationRealm::DaoSeeking => "Dao Seeking",
            CultivationRealm::ImmortalAscension => "Immortal Ascension",
        }
    }

    pub fn sub_stages(&self) -> u32 {
        match self {
            CultivationRealm::QiRefining => 9,
            CultivationRealm::Foundation => 3,
            CultivationRealm::CoreFormation => 3,
            CultivationRealm::NascentSoul => 3,
            CultivationRealm::SpiritSevering => 3,
            CultivationRealm::DaoSeeking => 3,
            CultivationRealm::ImmortalAscension => 1,
        }
    }

    pub fn qi_required(&self) -> u64 {
        match self {
            CultivationRealm::QiRefining => 100,
            CultivationRealm::Foundation => 1_000,
            CultivationRealm::CoreFormation => 10_000,
            CultivationRealm::NascentSoul => 100_000,
            CultivationRealm::SpiritSevering => 1_000_000,
            CultivationRealm::DaoSeeking => 10_000_000,
            CultivationRealm::ImmortalAscension => 100_000_000,
        }
    }

    pub fn stat_multiplier(&self) -> f64 {
        match self {
            CultivationRealm::QiRefining => 1.0,
            CultivationRealm::Foundation => 1.5,
            CultivationRealm::CoreFormation => 2.5,
            CultivationRealm::NascentSoul => 4.0,
            CultivationRealm::SpiritSevering => 7.0,
            CultivationRealm::DaoSeeking => 12.0,
            CultivationRealm::ImmortalAscension => 20.0,
        }
    }

    pub fn next(&self) -> Option<CultivationRealm> {
        let idx = Self::all().iter().position(|r| r == self)?;
        Self::all().get(idx + 1).copied()
    }

    pub fn index(&self) -> usize {
        Self::all().iter().position(|r| r == self).unwrap_or(0)
    }
}

impl std::fmt::Display for CultivationRealm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ═══════════════════════════════════════════════════════════════════
// Spiritual Root Types
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpiritualRoot {
    Metal,
    Wood,
    Water,
    Fire,
    Earth,
    Mixed,
}

impl SpiritualRoot {
    pub fn all() -> &'static [SpiritualRoot] {
        &[
            SpiritualRoot::Metal,
            SpiritualRoot::Wood,
            SpiritualRoot::Water,
            SpiritualRoot::Fire,
            SpiritualRoot::Earth,
            SpiritualRoot::Mixed,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SpiritualRoot::Metal => "Metal",
            SpiritualRoot::Wood => "Wood",
            SpiritualRoot::Water => "Water",
            SpiritualRoot::Fire => "Fire",
            SpiritualRoot::Earth => "Earth",
            SpiritualRoot::Mixed => "Mixed",
        }
    }

    pub fn qi_affinity(&self, realm: CultivationRealm) -> f64 {
        let base = match self {
            SpiritualRoot::Metal
            | SpiritualRoot::Wood
            | SpiritualRoot::Water
            | SpiritualRoot::Fire
            | SpiritualRoot::Earth => 1.2,
            SpiritualRoot::Mixed => 0.8,
        };
        base * (1.0 + realm.index() as f64 * 0.1)
    }

    pub fn compatible_element(&self, other: SpiritualRoot) -> bool {
        match (self, other) {
            (a, b) if *a == b => true,
            (SpiritualRoot::Metal, SpiritualRoot::Earth)
            | (SpiritualRoot::Earth, SpiritualRoot::Metal) => true,
            (SpiritualRoot::Wood, SpiritualRoot::Water)
            | (SpiritualRoot::Water, SpiritualRoot::Wood) => true,
            (SpiritualRoot::Fire, SpiritualRoot::Wood)
            | (SpiritualRoot::Wood, SpiritualRoot::Fire) => true,
            (SpiritualRoot::Water, SpiritualRoot::Fire)
            | (SpiritualRoot::Fire, SpiritualRoot::Water) => false,
            _ => false,
        }
    }
}

impl std::fmt::Display for SpiritualRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ═══════════════════════════════════════════════════════════════════
// Cultivation Techniques
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TechniqueType {
    BodyRefinement,
    QiGathering,
    CombatArt,
    Movement,
    Alchemy,
    Formation,
}

impl TechniqueType {
    pub fn display_name(&self) -> &'static str {
        match self {
            TechniqueType::BodyRefinement => "Body Refinement",
            TechniqueType::QiGathering => "Qi Gathering",
            TechniqueType::CombatArt => "Combat Art",
            TechniqueType::Movement => "Movement",
            TechniqueType::Alchemy => "Alchemy",
            TechniqueType::Formation => "Formation",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CultivationTechnique {
    pub id: String,
    pub name: String,
    pub technique_type: TechniqueType,
    pub element: SpiritualRoot,
    pub qi_per_tick: u64,
    pub stability_modifier: f64,
    pub required_realm: CultivationRealm,
    pub stat_bonuses: HashMap<Stat, i32>,
}

impl CultivationTechnique {
    pub fn new(
        id: &str,
        name: &str,
        technique_type: TechniqueType,
        element: SpiritualRoot,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            technique_type,
            element,
            qi_per_tick: 10,
            stability_modifier: 0.0,
            required_realm: CultivationRealm::QiRefining,
            stat_bonuses: HashMap::new(),
        }
    }

    pub fn with_qi_per_tick(mut self, qi: u64) -> Self {
        self.qi_per_tick = qi;
        self
    }

    pub fn with_stability_modifier(mut self, modifier: f64) -> Self {
        self.stability_modifier = modifier;
        self
    }

    pub fn with_required_realm(mut self, realm: CultivationRealm) -> Self {
        self.required_realm = realm;
        self
    }

    pub fn with_stat_bonus(mut self, stat: Stat, amount: i32) -> Self {
        self.stat_bonuses.insert(stat, amount);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════
// Technique Slots
// ═══════════════════════════════════════════════════════════════════

pub const MAX_TECHNIQUE_SLOTS: usize = 4;

#[derive(Debug, Clone)]
pub struct TechniqueSlots {
    pub slots: Vec<Option<CultivationTechnique>>,
}

impl TechniqueSlots {
    pub fn new() -> Self {
        Self {
            slots: vec![None; MAX_TECHNIQUE_SLOTS],
        }
    }

    pub fn equip(&mut self, technique: CultivationTechnique) -> Option<CultivationTechnique> {
        for slot in &mut self.slots {
            if slot.is_none() {
                *slot = Some(technique);
                return None;
            }
        }
        Some(technique)
    }

    pub fn unequip(&mut self, index: usize) -> Option<CultivationTechnique> {
        self.slots.get_mut(index)?.take()
    }

    pub fn active_techniques(&self) -> Vec<&CultivationTechnique> {
        self.slots.iter().filter_map(|s| s.as_ref()).collect()
    }

    pub fn total_qi_per_tick(&self) -> u64 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|t| t.qi_per_tick)
            .sum()
    }

    pub fn total_stability_modifier(&self) -> f64 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|t| t.stability_modifier)
            .sum()
    }

    pub fn total_stat_bonuses(&self) -> HashMap<Stat, i32> {
        let mut bonuses = HashMap::new();
        for technique in self.slots.iter().filter_map(|s| s.as_ref()) {
            for (stat, amount) in &technique.stat_bonuses {
                *bonuses.entry(*stat).or_insert(0) += amount;
            }
        }
        bonuses
    }
}

impl Default for TechniqueSlots {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Breakthrough System
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct BreakthroughAttempt {
    pub qi_accumulated: u64,
    pub qi_required: u64,
    pub stability: f64,
    pub base_success_rate: f64,
    pub spiritual_root_bonus: f64,
    pub technique_bonus: f64,
}

impl BreakthroughAttempt {
    pub fn can_attempt(&self) -> bool {
        self.qi_accumulated >= self.qi_required
    }

    pub fn success_rate(&self) -> f64 {
        if !self.can_attempt() {
            return 0.0;
        }
        let qi_ratio = self.qi_accumulated as f64 / self.qi_required as f64;
        let qi_bonus = (qi_ratio - 1.0).max(0.0) * 0.5;
        let rate =
            self.base_success_rate + self.spiritual_root_bonus + self.technique_bonus + qi_bonus;
        rate.min(0.99)
    }

    pub fn stability_penalty(&self) -> f64 {
        if self.stability < 0.3 {
            0.4
        } else if self.stability < 0.6 {
            0.2
        } else if self.stability < 0.8 {
            0.1
        } else {
            0.0
        }
    }

    pub fn effective_success_rate(&self) -> f64 {
        (self.success_rate() - self.stability_penalty()).max(0.01)
    }
}

#[derive(Debug, Clone)]
pub struct BreakthroughResult {
    pub success: bool,
    pub realm_gained: Option<CultivationRealm>,
    pub sub_stage_advanced: bool,
    pub qi_cost: u64,
    pub instability_lost: f64,
    pub message: String,
}

// ═══════════════════════════════════════════════════════════════════
// Cultivation State
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct CultivationState {
    pub realm: CultivationRealm,
    pub sub_stage: u32,
    pub qi: u64,
    pub max_qi: u64,
    pub spiritual_root: SpiritualRoot,
    pub stability: f64,
    pub techniques: TechniqueSlots,
    pub total_qi_cultivated: u64,
    pub breakthroughs_attempted: u32,
    pub breakthroughs_succeeded: u32,
}

impl CultivationState {
    pub fn new() -> Self {
        Self {
            realm: CultivationRealm::QiRefining,
            sub_stage: 1,
            qi: 0,
            max_qi: 100,
            spiritual_root: SpiritualRoot::Mixed,
            stability: 1.0,
            techniques: TechniqueSlots::new(),
            total_qi_cultivated: 0,
            breakthroughs_attempted: 0,
            breakthroughs_succeeded: 0,
        }
    }

    pub fn with_root(mut self, root: SpiritualRoot) -> Self {
        self.spiritual_root = root;
        self
    }

    pub fn with_qi(mut self, qi: u64) -> Self {
        self.qi = qi.min(self.max_qi);
        self
    }

    pub fn cultivate_tick(&mut self) -> u64 {
        let base_qi = self.techniques.total_qi_per_tick();
        let affinity = self.spiritual_root.qi_affinity(self.realm);
        let qi_gained = (base_qi as f64 * affinity) as u64;
        let _before = self.qi;
        self.qi = (self.qi + qi_gained).min(self.max_qi);
        self.total_qi_cultivated += qi_gained;
        qi_gained
    }

    pub fn can_breakthrough(&self) -> bool {
        self.qi >= self.realm.qi_required() && self.sub_stage >= self.realm.sub_stages()
    }

    pub fn build_breakthrough_attempt(&self) -> BreakthroughAttempt {
        let root_bonus = match self.spiritual_root {
            SpiritualRoot::Metal
            | SpiritualRoot::Wood
            | SpiritualRoot::Water
            | SpiritualRoot::Fire
            | SpiritualRoot::Earth => 0.15,
            SpiritualRoot::Mixed => 0.05,
        };
        let tech_bonus = self.techniques.total_stability_modifier();

        BreakthroughAttempt {
            qi_accumulated: self.qi,
            qi_required: self.realm.qi_required(),
            stability: self.stability,
            base_success_rate: 0.6,
            spiritual_root_bonus: root_bonus,
            technique_bonus: tech_bonus,
        }
    }

    pub fn attempt_breakthrough(&mut self) -> BreakthroughResult {
        if !self.can_breakthrough() {
            return BreakthroughResult {
                success: false,
                realm_gained: None,
                sub_stage_advanced: false,
                qi_cost: 0,
                instability_lost: 0.0,
                message: "Insufficient Qi or sub-stage for breakthrough".to_string(),
            };
        }

        self.breakthroughs_attempted += 1;
        let attempt = self.build_breakthrough_attempt();
        let rate = attempt.effective_success_rate();
        let roll = pseudo_random_f64();

        if roll < rate {
            let qi_cost = self.realm.qi_required() / 2;
            self.qi = self.qi.saturating_sub(qi_cost);
            self.stability = (self.stability - 0.1).max(0.1);
            self.breakthroughs_succeeded += 1;

            if let Some(next_realm) = self.realm.next() {
                self.realm = next_realm;
                self.sub_stage = 1;
                self.max_qi = next_realm.qi_required();
                BreakthroughResult {
                    success: true,
                    realm_gained: Some(next_realm),
                    sub_stage_advanced: false,
                    qi_cost,
                    instability_lost: 0.1,
                    message: format!(
                        "Breakthrough succeeded! Advanced to {}",
                        next_realm.display_name()
                    ),
                }
            } else {
                BreakthroughResult {
                    success: true,
                    realm_gained: None,
                    sub_stage_advanced: false,
                    qi_cost,
                    instability_lost: 0.1,
                    message: "Already at peak realm — Immortal Ascension achieved!".to_string(),
                }
            }
        } else {
            let qi_cost = self.realm.qi_required() / 4;
            self.qi = self.qi.saturating_sub(qi_cost);
            self.stability = (self.stability - 0.3).max(0.0);
            BreakthroughResult {
                success: false,
                realm_gained: None,
                sub_stage_advanced: false,
                qi_cost,
                instability_lost: 0.3,
                message: "Breakthrough failed! Significant instability gained".to_string(),
            }
        }
    }

    pub fn advance_sub_stage(&mut self) {
        if self.sub_stage < self.realm.sub_stages() {
            self.sub_stage += 1;
        }
    }

    pub fn stabilize(&mut self) {
        self.stability = (self.stability + 0.1).min(1.0);
    }

    pub fn realm_display(&self) -> String {
        format!("{} Stage {}", self.realm.display_name(), self.sub_stage)
    }

    pub fn stat_multiplier(&self) -> f64 {
        let base = self.realm.stat_multiplier();
        base * (self.sub_stage as f64 / self.realm.sub_stages() as f64 * 0.3 + 0.7)
    }

    pub fn effective_stat_bonuses(&self) -> HashMap<Stat, i32> {
        let tech_bonuses = self.techniques.total_stat_bonuses();
        let multiplier = self.stat_multiplier();
        tech_bonuses
            .into_iter()
            .map(|(stat, amount)| (stat, (amount as f64 * multiplier) as i32))
            .collect()
    }
}

impl Default for CultivationState {
    fn default() -> Self {
        Self::new()
    }
}

fn pseudo_random_f64() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut h = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    h.finish() as f64 / u64::MAX as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_ordering() {
        assert!(CultivationRealm::QiRefining < CultivationRealm::Foundation);
        assert!(CultivationRealm::NascentSoul < CultivationRealm::SpiritSevering);
        assert_eq!(CultivationRealm::ImmortalAscension.next(), None);
    }

    #[test]
    fn test_cultivation_tick() {
        let mut state = CultivationState::new().with_qi(0);
        let mut slots = TechniqueSlots::new();
        let tech = CultivationTechnique::new(
            "basic_qi",
            "Basic Qi Gathering",
            TechniqueType::QiGathering,
            SpiritualRoot::Water,
        )
        .with_qi_per_tick(10);
        slots.equip(tech);
        state.techniques = slots;
        let gained = state.cultivate_tick();
        assert!(gained > 0);
        assert!(state.qi > 0);
    }

    #[test]
    fn test_breakthrough_not_ready() {
        let state = CultivationState::new();
        assert!(!state.can_breakthrough());
    }

    #[test]
    fn test_breakthrough_ready() {
        let mut state = CultivationState::new();
        state.qi = 100;
        state.sub_stage = 9;
        assert!(state.can_breakthrough());
    }

    #[test]
    fn test_technique_slots() {
        let mut slots = TechniqueSlots::new();
        let tech = CultivationTechnique::new(
            "basic_qi",
            "Basic Qi Gathering",
            TechniqueType::QiGathering,
            SpiritualRoot::Water,
        );
        assert!(slots.equip(tech).is_none());
        assert_eq!(slots.active_techniques().len(), 1);
    }

    #[test]
    fn test_spiritual_root_compatibility() {
        assert!(SpiritualRoot::Metal.compatible_element(SpiritualRoot::Earth));
        assert!(SpiritualRoot::Water.compatible_element(SpiritualRoot::Wood));
        assert!(!SpiritualRoot::Water.compatible_element(SpiritualRoot::Fire));
    }

    #[test]
    fn test_breakthrough_attempt() {
        let mut state = CultivationState::new()
            .with_root(SpiritualRoot::Fire)
            .with_qi(100);
        state.sub_stage = 9;
        let result = state.attempt_breakthrough();
        // Random, but we verify the structure works
        assert!(result.qi_cost > 0 || !result.success);
    }
}
