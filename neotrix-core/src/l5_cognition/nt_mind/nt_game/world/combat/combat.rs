#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Fire,
    Ice,
    Lightning,
    Poison,
    Holy,
}

#[derive(Debug, Clone)]
pub struct CombatAction {
    pub name: String,
    pub damage: f64,
    pub damage_type: DamageType,
    pub cost: f64,
    pub range: f64,
    pub cooldown: f64,
    pub aoe: bool,
}

impl CombatAction {
    pub fn attack(name: &str, damage: f64) -> Self {
        Self {
            name: name.to_string(),
            damage,
            damage_type: DamageType::Physical,
            cost: 0.0,
            range: 1.5,
            cooldown: 0.0,
            aoe: false,
        }
    }
    pub fn with_type(mut self, dt: DamageType) -> Self {
        self.damage_type = dt;
        self
    }
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }
    pub fn with_range(mut self, range: f64) -> Self {
        self.range = range;
        self
    }
    pub fn with_cooldown(mut self, cd: f64) -> Self {
        self.cooldown = cd;
        self
    }
    pub fn with_aoe(mut self, aoe: bool) -> Self {
        self.aoe = aoe;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub damage_dealt: f64,
    pub damage_type: DamageType,
    pub blocked: bool,
    pub critical: bool,
    pub source_hp_after: f64,
    pub target_hp_after: f64,
}

#[derive(Debug, Clone)]
pub struct Combatant {
    pub id: u64,
    pub name: String,
    pub hp: f64,
    pub max_hp: f64,
    pub attack: f64,
    pub defense: f64,
    pub speed: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub cooldowns: std::collections::HashMap<String, f64>,
}

impl Combatant {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            hp: 100.0,
            max_hp: 100.0,
            attack: 10.0,
            defense: 5.0,
            speed: 1.0,
            energy: 100.0,
            max_energy: 100.0,
            cooldowns: std::collections::HashMap::new(),
        }
    }
    pub fn with_stats(mut self, hp: f64, attack: f64, defense: f64) -> Self {
        self.hp = hp;
        self.max_hp = hp;
        self.attack = attack;
        self.defense = defense;
        self
    }
    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
    }
    pub fn can_use(&self, action: &CombatAction) -> bool {
        self.energy >= action.cost && *self.cooldowns.get(&action.name).unwrap_or(&0.0) <= 0.0
    }
    pub fn take_damage(&mut self, amount: f64) -> f64 {
        let blocked = self.defense.min(amount);
        let actual = (amount - blocked).max(0.0);
        self.hp = (self.hp - actual).max(0.0);
        actual
    }
    pub fn heal(&mut self, amount: f64) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
    pub fn restore_energy(&mut self, amount: f64) {
        self.energy = (self.energy + amount).min(self.max_energy);
    }
    pub fn tick_cooldowns(&mut self, dt: f64) {
        for v in self.cooldowns.values_mut() {
            *v = (*v - dt).max(0.0);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CombatState {
    pub turn: u32,
    pub participants: Vec<Combatant>,
    pub is_active: bool,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            turn: 0,
            participants: Vec::new(),
            is_active: false,
        }
    }
    pub fn add_combatant(&mut self, c: Combatant) {
        self.participants.push(c);
    }
    pub fn start(&mut self) {
        self.is_active = true;
        self.turn = 1;
    }
    pub fn execute_action(
        &mut self,
        source_id: u64,
        target_id: u64,
        action: &CombatAction,
    ) -> Option<CombatResult> {
        let src_idx = self.participants.iter().position(|c| c.id == source_id)?;
        let tgt_idx = self.participants.iter().position(|c| c.id == target_id)?;
        if !self.participants[src_idx].can_use(action) {
            return None;
        }
        let src_attack = self.participants[src_idx].attack;
        let tgt_defense = self.participants[tgt_idx].defense;
        self.participants[src_idx].energy -= action.cost;
        self.participants[src_idx]
            .cooldowns
            .insert(action.name.clone(), action.cooldown);
        let raw_damage = action.damage + src_attack * 0.5;
        let blocked = tgt_defense.min(raw_damage);
        let actual = (raw_damage - blocked).max(0.0);
        let critical = rand_bool(0.1);
        let final_damage = if critical { actual * 1.5 } else { actual };
        self.participants[tgt_idx].hp = (self.participants[tgt_idx].hp - final_damage).max(0.0);
        Some(CombatResult {
            damage_dealt: final_damage,
            damage_type: action.damage_type,
            blocked: blocked > 0.0,
            critical,
            source_hp_after: self.participants[src_idx].hp,
            target_hp_after: self.participants[tgt_idx].hp,
        })
    }
    pub fn next_turn(&mut self) {
        self.turn += 1;
        for c in &mut self.participants {
            c.tick_cooldowns(1.0);
            c.restore_energy(10.0);
        }
    }
    pub fn is_over(&self) -> bool {
        self.participants.iter().filter(|c| c.is_alive()).count() <= 1
    }
    pub fn winner(&self) -> Option<&Combatant> {
        if self.is_over() {
            self.participants.iter().find(|c| c.is_alive())
        } else {
            None
        }
    }
}
impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_bool(prob: f64) -> bool {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    (h.finish() as f64 / u64::MAX as f64) < prob
}
