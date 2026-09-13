use std::collections::HashMap;

use super::renderer::Vec2;

// ---------------------------------------------------------------------------
// NPCState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NPCState {
    Idle,
    Patrol,
    Wander,
    Chase,
    Attack,
    Flee,
    Dead,
}

impl NPCState {
    pub fn can_move(&self) -> bool {
        matches!(self, Self::Patrol | Self::Wander | Self::Chase | Self::Flee)
    }
    pub fn in_combat(&self) -> bool {
        matches!(self, Self::Chase | Self::Attack)
    }
    pub fn can_interact(&self) -> bool {
        matches!(self, Self::Idle | Self::Patrol | Self::Wander)
    }
}

// ---------------------------------------------------------------------------
// NPCSchedule
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub hour: u8,
    pub location: String,
    pub activity: String,
    pub state: NPCState,
    pub position: Option<Vec2>,
}

#[derive(Debug, Clone)]
pub struct NPCSchedule {
    pub entries: Vec<ScheduleEntry>,
    pub home: Vec2,
    pub wander_radius: f32,
}

impl NPCSchedule {
    pub fn new(home: Vec2) -> Self {
        Self { entries: Vec::new(), home, wander_radius: 64.0 }
    }

    pub fn with_entry(mut self, entry: ScheduleEntry) -> Self { self.entries.push(entry); self }

    pub fn current_activity(&self, hour: u8) -> Option<&ScheduleEntry> {
        self.entries.iter().find(|e| e.hour == hour)
    }

    pub fn nearest_activity(&self, hour: u8) -> Option<&ScheduleEntry> {
        self.entries.iter().min_by_key(|e| {
            let diff = (e.hour as i16 - hour as i16).abs();
            diff.min(24 - diff) as u16
        })
    }
}

// ---------------------------------------------------------------------------
// NPCRelationship
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NPCRelationship {
    pub favorability: i32,
    pub trust: i32,
    pub met: bool,
    pub interaction_count: u32,
    pub last_interaction: f64,
    pub dialogue_history: Vec<String>,
}

impl NPCRelationship {
    pub fn new() -> Self {
        Self { favorability: 0, trust: 0, met: false, interaction_count: 0, last_interaction: 0.0, dialogue_history: Vec::new() }
    }

    pub fn with_favorability(mut self, val: i32) -> Self { self.favorability = val.clamp(-100, 100); self }
    pub fn with_trust(mut self, val: i32) -> Self { self.trust = val.clamp(0, 100); self }

    pub fn modify_favorability(&mut self, amount: i32) { self.favorability = (self.favorability + amount).clamp(-100, 100); }
    pub fn modify_trust(&mut self, amount: i32) { self.trust = (self.trust + amount).clamp(0, 100); }

    pub fn interact(&mut self, game_time: f64) {
        self.met = true;
        self.interaction_count += 1;
        self.last_interaction = game_time;
    }

    pub fn add_dialogue_history(&mut self, dialogue_id: &str) {
        self.dialogue_history.push(dialogue_id.to_string());
    }

    pub fn tier(&self) -> RelationTier {
        match self.favorability {
            i32::MIN..=-50 => RelationTier::Hostile,
            -49..=-10 => RelationTier::Unfriendly,
            -9..=9 => RelationTier::Neutral,
            10..=49 => RelationTier::Friendly,
            50..=i32::MAX => RelationTier::Allied,
        }
    }

    pub fn can_offer_quest(&self) -> bool {
        self.tier() != RelationTier::Hostile
    }

    pub fn shop_discount_pct(&self) -> f32 {
        match self.tier() {
            RelationTier::Allied => 0.15,
            RelationTier::Friendly => 0.05,
            _ => 0.0,
        }
    }
}

impl Default for NPCRelationship {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationTier {
    Hostile,
    Unfriendly,
    Neutral,
    Friendly,
    Allied,
}

// ---------------------------------------------------------------------------
// NPCBehavior (state machine)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BehaviorTransition {
    pub from: NPCState,
    pub to: NPCState,
    pub condition: String,
}

pub struct NPCBehavior {
    pub state: NPCState,
    pub transitions: Vec<BehaviorTransition>,
    pub flags: HashMap<String, bool>,
    pub stats: HashMap<String, f32>,
    pub target_id: Option<u64>,
    pub patrol_points: Vec<Vec2>,
    pub patrol_index: usize,
    pub wander_center: Vec2,
    pub wander_radius: f32,
    pub idle_timer: f32,
    pub idle_countdown: f32,
    pub target_position: Option<Vec2>,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
}

impl NPCBehavior {
    pub fn new() -> Self {
        Self {
            state: NPCState::Idle,
            transitions: Vec::new(),
            flags: HashMap::new(),
            stats: HashMap::new(),
            target_id: None,
            patrol_points: Vec::new(),
            patrol_index: 0,
            wander_center: Vec2::zero(),
            wander_radius: 64.0,
            idle_timer: 3.0,
            idle_countdown: 0.0,
            target_position: None,
            attack_cooldown: 1.0,
            attack_timer: 0.0,
        }
    }

    pub fn with_transition(mut self, from: NPCState, to: NPCState, condition: &str) -> Self {
        self.transitions.push(BehaviorTransition { from, to, condition: condition.to_string() });
        self
    }

    pub fn with_patrol(mut self, points: Vec<Vec2>) -> Self { self.patrol_points = points; self }
    pub fn with_wander_radius(mut self, radius: f32) -> Self { self.wander_radius = radius; self }

    pub fn set_flag(&mut self, name: &str, value: bool) { self.flags.insert(name.to_string(), value); }
    pub fn get_flag(&self, name: &str) -> bool { self.flags.get(name).copied().unwrap_or(false) }
    pub fn set_stat(&mut self, name: &str, value: f32) { self.stats.insert(name.to_string(), value); }
    pub fn get_stat(&self, name: &str) -> f32 { self.stats.get(name).copied().unwrap_or(0.0) }

    fn evaluate_condition(&self, condition: &str) -> bool {
        if let Some(flag_name) = condition.strip_prefix('!') {
            return !self.get_flag(flag_name);
        }
        if let Some(rest) = condition.strip_prefix("stat>=") {
            if let Some((stat_name, val_str)) = rest.split_once(':') {
                if let Ok(val) = val_str.parse::<f32>() { return self.get_stat(stat_name) >= val; }
            }
        }
        if let Some(rest) = condition.strip_prefix("stat<") {
            if let Some((stat_name, val_str)) = rest.split_once(':') {
                if let Ok(val) = val_str.parse::<f32>() { return self.get_stat(stat_name) < val; }
            }
        }
        self.get_flag(condition)
    }

    pub fn tick(&mut self, dt: f32) {
        // Check transitions from current state
        let transition = self.transitions.iter().find(|t| {
            t.from == self.state && self.evaluate_condition(&t.condition)
        }).cloned();

        if let Some(t) = transition {
            self.state = t.to;
        }

        // State-specific behavior
        match self.state {
            NPCState::Idle => {
                self.idle_countdown -= dt;
                if self.idle_countdown <= 0.0 {
                    self.state = NPCState::Wander;
                    self.idle_countdown = self.idle_timer;
                }
            }
            NPCState::Patrol => {
                if !self.patrol_points.is_empty() {
                    self.patrol_index = (self.patrol_index + 1) % self.patrol_points.len();
                }
            }
            NPCState::Chase | NPCState::Flee => {
                // Movement handled externally via target_position
            }
            NPCState::Attack => {
                self.attack_timer -= dt;
            }
            NPCState::Dead => {}
            _ => {}
        }
    }

    pub fn next_patrol_point(&self) -> Option<Vec2> {
        self.patrol_points.get(self.patrol_index).copied()
    }

    pub fn wander_target(&self) -> Vec2 {
        let angle = pseudo_random_f32() * std::f32::consts::TAU;
        let dist = pseudo_random_f32() * self.wander_radius;
        Vec2::new(
            self.wander_center.x + angle.cos() * dist,
            self.wander_center.y + angle.sin() * dist,
        )
    }

    pub fn chase_target(&self, target_pos: Vec2, current_pos: Vec2, speed: f32, dt: f32) -> Vec2 {
        let dx = target_pos.x - current_pos.x;
        let dy = target_pos.y - current_pos.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1.0 { return current_pos; }
        let move_speed = speed * dt;
        Vec2::new(
            current_pos.x + dx / dist * move_speed.min(dist),
            current_pos.y + dy / dist * move_speed.min(dist),
        )
    }

    pub fn flee_from(&self, threat_pos: Vec2, current_pos: Vec2, speed: f32, dt: f32) -> Vec2 {
        let dx = current_pos.x - threat_pos.x;
        let dy = current_pos.y - threat_pos.y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let move_speed = speed * dt;
        Vec2::new(
            current_pos.x + dx / dist * move_speed,
            current_pos.y + dy / dist * move_speed,
        )
    }

    pub fn in_combat(&self) -> bool { self.state.in_combat() }

    pub fn can_attack(&self) -> bool {
        self.state == NPCState::Attack && self.attack_timer <= 0.0
    }

    pub fn reset_attack_timer(&mut self) {
        self.attack_timer = self.attack_cooldown;
    }
}

impl Default for NPCBehavior {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// NPC (full entity)
// ---------------------------------------------------------------------------

pub struct NPC {
    pub id: String,
    pub name: String,
    pub position: Vec2,
    pub behavior: NPCBehavior,
    pub schedule: Option<NPCSchedule>,
    pub relationship: NPCRelationship,
    pub max_health: f32,
    pub health: f32,
    pub attack_power: f32,
    pub defense: f32,
    pub speed: f32,
    pub dialogue_ids: Vec<String>,
    pub offered_quests: Vec<String>,
    pub respawn_time: Option<f32>,
    pub respawn_countdown: f32,
    pub loot_table: Vec<(String, u32)>,
    pub xp_reward: u32,
    pub aggro_range: f32,
    pub deaggro_range: f32,
    pub faction: Option<String>,
}

impl NPC {
    pub fn new(id: &str, name: &str, position: Vec2) -> Self {
        Self {
            id: id.to_string(), name: name.to_string(), position,
            behavior: NPCBehavior::new(), schedule: None, relationship: NPCRelationship::new(),
            max_health: 100.0, health: 100.0, attack_power: 10.0, defense: 5.0, speed: 100.0,
            dialogue_ids: Vec::new(), offered_quests: Vec::new(),
            respawn_time: None, respawn_countdown: 0.0,
            loot_table: Vec::new(), xp_reward: 0,
            aggro_range: 150.0, deaggro_range: 300.0, faction: None,
        }
    }

    pub fn with_health(mut self, hp: f32) -> Self { self.max_health = hp; self.health = hp; self }
    pub fn with_combat_stats(mut self, atk: f32, def: f32) -> Self { self.attack_power = atk; self.defense = def; self }
    pub fn with_speed(mut self, spd: f32) -> Self { self.speed = spd; self }
    pub fn with_schedule(mut self, schedule: NPCSchedule) -> Self { self.schedule = Some(schedule); self }
    pub fn with_dialogue(mut self, dialogue_id: &str) -> Self { self.dialogue_ids.push(dialogue_id.to_string()); self }
    pub fn with_quest(mut self, quest_id: &str) -> Self { self.offered_quests.push(quest_id.to_string()); self }
    pub fn with_respawn(mut self, seconds: f32) -> Self { self.respawn_time = Some(seconds); self }
    pub fn with_loot(mut self, item_id: &str, count: u32) -> Self { self.loot_table.push((item_id.to_string(), count)); self }
    pub fn with_xp(mut self, xp: u32) -> Self { self.xp_reward = xp; self }
    pub fn with_aggro(mut self, aggro: f32, deaggro: f32) -> Self { self.aggro_range = aggro; self.deaggro_range = deaggro; self }
    pub fn with_faction(mut self, faction: &str) -> Self { self.faction = Some(faction.to_string()); self }

    pub fn take_damage(&mut self, raw_damage: f32) -> f32 {
        let actual = (raw_damage - self.defense).max(0.0);
        self.health = (self.health - actual).max(0.0);
        if self.health <= 0.0 { self.behavior.state = NPCState::Dead; }
        actual
    }

    pub fn heal(&mut self, amount: f32) { self.health = (self.health + amount).min(self.max_health); }
    pub fn is_alive(&self) -> bool { self.health > 0.0 && self.behavior.state != NPCState::Dead }
    pub fn hp_pct(&self) -> f32 { self.health / self.max_health }

    pub fn update_respawn(&mut self, dt: f32) {
        if self.behavior.state == NPCState::Dead {
            if let Some(respawn_time) = self.respawn_time {
                self.respawn_countdown += dt;
                if self.respawn_countdown >= respawn_time { self.respawn(); }
            }
        }
    }

    pub fn respawn(&mut self) {
        self.health = self.max_health;
        self.behavior.state = NPCState::Idle;
        self.behavior.target_id = None;
        self.behavior.target_position = None;
        self.respawn_countdown = 0.0;
    }

    /// Check if NPC should aggro on a player position.
    pub fn should_aggro(&self, player_pos: Vec2) -> bool {
        if !self.is_alive() { return false; }
        if self.behavior.state.in_combat() { return false; }
        let dx = self.position.x - player_pos.x;
        let dy = self.position.y - player_pos.y;
        (dx * dx + dy * dy) <= self.aggro_range * self.aggro_range
    }

    /// Check if NPC should deaggro.
    pub fn should_deaggro(&self, player_pos: Vec2) -> bool {
        let dx = self.position.x - player_pos.x;
        let dy = self.position.y - player_pos.y;
        (dx * dx + dy * dy) > self.deaggro_range * self.deaggro_range
    }

    /// Get movement target based on current state.
    pub fn movement_target(&self) -> Option<Vec2> {
        match self.behavior.state {
            NPCState::Patrol => self.behavior.next_patrol_point(),
            NPCState::Wander => Some(self.behavior.wander_target()),
            NPCState::Chase | NPCState::Flee => self.behavior.target_position,
            _ => None,
        }
    }

    /// Update position toward target. Returns new position.
    pub fn move_toward(&self, target: Vec2, dt: f32) -> Vec2 {
        let speed = self.speed * self.behavior.state.state_maybe_speed_mult();
        let dx = target.x - self.position.x;
        let dy = target.y - self.position.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1.0 { return self.position; }
        let move_amount = speed * dt;
        Vec2::new(
            self.position.x + dx / dist * move_amount.min(dist),
            self.position.y + dy / dist * move_amount.min(dist),
        )
    }

    pub fn tick(&mut self, dt: f32) {
        if self.is_alive() {
            self.behavior.tick(dt);
        } else {
            self.update_respawn(dt);
        }
    }
}

// Helper on NPCState
impl NPCState {
    fn state_maybe_speed_mult(&self) -> f32 {
        match self {
            NPCState::Flee => 1.3,
            NPCState::Chase => 0.9,
            _ => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// NPCManager
// ---------------------------------------------------------------------------

pub struct NPCManager {
    npcs: HashMap<String, NPC>,
    interaction_cooldowns: HashMap<String, f32>,
}

impl NPCManager {
    pub fn new() -> Self {
        Self { npcs: HashMap::new(), interaction_cooldowns: HashMap::new() }
    }

    pub fn add_npc(&mut self, npc: NPC) { self.npcs.insert(npc.id.clone(), npc); }
    pub fn get_npc(&self, id: &str) -> Option<&NPC> { self.npcs.get(id) }
    pub fn get_npc_mut(&mut self, id: &str) -> Option<&mut NPC> { self.npcs.get_mut(id) }
    pub fn remove_npc(&mut self, id: &str) -> Option<NPC> { self.npcs.remove(id) }
    pub fn npc_count(&self) -> usize { self.npcs.len() }

    pub fn alive_npcs(&self) -> Vec<&NPC> {
        self.npcs.values().filter(|n| n.is_alive()).collect()
    }

    pub fn npcs_in_range(&self, center: Vec2, range: f32) -> Vec<&NPC> {
        self.npcs.values().filter(|n| {
            let dx = n.position.x - center.x;
            let dy = n.position.y - center.y;
            dx * dx + dy * dy <= range * range
        }).collect()
    }

    pub fn get_nearby_npcs(&self, x: f32, y: f32, radius: f32) -> Vec<&NPC> {
        self.npcs_in_range(Vec2::new(x, y), radius)
    }

    pub fn interact(&mut self, npc_id: &str, game_time: f64) -> Option<&NPC> {
        if let Some(npc) = self.npcs.get_mut(npc_id) {
            if npc.is_alive() && npc.behavior.state.can_interact() {
                npc.relationship.interact(game_time);
                self.interaction_cooldowns.insert(npc_id.to_string(), 0.5);
                Some(&*npc)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Update all NPCs. Returns events (aggro, deaggro, etc.).
    pub fn update_all(&mut self, dt: f32, player_pos: Vec2, game_time: f64) -> Vec<NPCEvent> {
        let mut events = Vec::new();

        // Update interaction cooldowns
        for cd in self.interaction_cooldowns.values_mut() {
            *cd -= dt;
        }
        self.interaction_cooldowns.retain(|_, cd| *cd > 0.0);

        let npc_ids: Vec<String> = self.npcs.keys().cloned().collect();
        for id in &npc_ids {
            // Get NPC's alive status and current state before mutable borrow
            let (is_alive, can_aggro, should_deaggro, has_no_target) = {
                if let Some(npc) = self.npcs.get(id) {
                    let alive = npc.is_alive();
                    let agg = npc.should_aggro(player_pos);
                    let deag = npc.should_deaggro(player_pos);
                    let no_target = npc.behavior.target_id.is_none();
                    (alive, agg, deag, no_target)
                } else {
                    continue;
                }
            };

            if !is_alive { continue; }

            // Aggro logic
            if can_aggro && !self.npcs[id].behavior.in_combat() {
                if let Some(npc) = self.npcs.get_mut(id) {
                    npc.behavior.state = NPCState::Chase;
                    npc.behavior.target_position = Some(player_pos);
                    events.push(NPCEvent::Aggro { npc_id: id.clone() });
                }
            }

            // Deaggro logic
            if should_deaggro && self.npcs[id].behavior.in_combat() {
                if let Some(npc) = self.npcs.get_mut(id) {
                    npc.behavior.state = NPCState::Idle;
                    npc.behavior.target_id = None;
                    npc.behavior.target_position = None;
                    events.push(NPCEvent::Deaggro { npc_id: id.clone() });
                }
            }

            // Tick behavior
            if let Some(npc) = self.npcs.get_mut(id) {
                npc.tick(dt);

                // Flee if low health
                if npc.is_alive() && npc.hp_pct() < 0.2 && npc.behavior.state == NPCState::Attack {
                    npc.behavior.state = NPCState::Flee;
                    events.push(NPCEvent::Flee { npc_id: id.clone() });
                }

                // Update movement
                if let Some(target) = npc.movement_target() {
                    let new_pos = npc.move_toward(target, dt);
                    npc.position = new_pos;
                }

                // Apply schedule
                if let Some(ref schedule) = npc.schedule {
                    if let Some(entry) = schedule.nearest_activity(game_time as u8) {
                        if let Some(pos) = entry.position {
                            npc.behavior.target_position = Some(pos);
                        }
                    }
                }
            }
        }

        events
    }
}

impl Default for NPCManager {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// NPCEvent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum NPCEvent {
    Aggro { npc_id: String },
    Deaggro { npc_id: String },
    Flee { npc_id: String },
    Die { npc_id: String },
    Respawn { npc_id: String },
    Interact { npc_id: String },
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pseudo_random_f32() -> f32 {
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
    fn test_npc_state() {
        assert!(NPCState::Patrol.can_move());
        assert!(!NPCState::Idle.can_move());
        assert!(NPCState::Chase.in_combat());
        assert!(!NPCState::Idle.in_combat());
        assert!(NPCState::Idle.can_interact());
        assert!(!NPCState::Dead.can_interact());
    }

    #[test]
    fn test_npc_relationship() {
        let mut rel = NPCRelationship::new();
        assert_eq!(rel.tier(), RelationTier::Neutral);
        rel.modify_favorability(60);
        assert_eq!(rel.tier(), RelationTier::Allied);
        assert!(rel.can_offer_quest());
        assert!((rel.shop_discount_pct() - 0.15).abs() < 0.01);
        rel.modify_favorability(-150);
        assert_eq!(rel.tier(), RelationTier::Hostile);
        assert!(!rel.can_offer_quest());
    }

    #[test]
    fn test_npc_schedule() {
        let schedule = NPCSchedule::new(Vec2::zero())
            .with_entry(ScheduleEntry {
                hour: 8, location: "market".into(), activity: "sell goods".into(),
                state: NPCState::Idle, position: Some(Vec2::new(100.0, 200.0)),
            });
        let entry = schedule.current_activity(8);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().activity, "sell goods");
    }

    #[test]
    fn test_npc_behavior_transitions() {
        let mut behavior = NPCBehavior::new()
            .with_transition(NPCState::Idle, NPCState::Wander, "should_wander")
            .with_wander_radius(50.0);
        behavior.set_flag("should_wander", true);
        behavior.idle_countdown = 10.0;
        behavior.tick(0.1);
        assert_eq!(behavior.state, NPCState::Wander);
    }

    #[test]
    fn test_npc_combat() {
        let mut npc = NPC::new("goblin", "Goblin", Vec2::zero())
            .with_health(30.0).with_combat_stats(5.0, 2.0);
        let dmg = npc.take_damage(10.0);
        assert!((dmg - 8.0).abs() < 0.01);
        npc.take_damage(25.0);
        assert!(!npc.is_alive());
        assert_eq!(npc.behavior.state, NPCState::Dead);
    }

    #[test]
    fn test_npc_respawn() {
        let mut npc = NPC::new("goblin", "Goblin", Vec2::zero())
            .with_health(10.0).with_respawn(1.0);
        npc.take_damage(20.0);
        assert!(!npc.is_alive());
        npc.update_respawn(1.5);
        assert!(npc.is_alive());
        assert_eq!(npc.health, 10.0);
    }

    #[test]
    fn test_npc_aggro_deaggro() {
        let npc = NPC::new("g", "Guard", Vec2::new(0.0, 0.0)).with_aggro(100.0, 200.0);
        assert!(npc.should_aggro(Vec2::new(50.0, 0.0)));
        assert!(!npc.should_aggro(Vec2::new(150.0, 0.0)));
        assert!(npc.should_deaggro(Vec2::new(250.0, 0.0)));
    }

    #[test]
    fn test_npc_chase() {
        let mut behavior = NPCBehavior::new();
        behavior.state = NPCState::Chase;
        let new_pos = behavior.chase_target(Vec2::new(100.0, 0.0), Vec2::new(0.0, 0.0), 100.0, 0.1);
        assert!(new_pos.x > 0.0);
    }

    #[test]
    fn test_npc_flee() {
        let behavior = NPCBehavior::new();
        let new_pos = behavior.flee_from(Vec2::new(100.0, 0.0), Vec2::new(50.0, 0.0), 100.0, 0.1);
        assert!(new_pos.x < 50.0);
    }

    #[test]
    fn test_npc_manager() {
        let mut mgr = NPCManager::new();
        mgr.add_npc(NPC::new("a", "A", Vec2::new(10.0, 10.0)));
        mgr.add_npc(NPC::new("b", "B", Vec2::new(100.0, 100.0)));
        assert_eq!(mgr.npc_count(), 2);

        let nearby = mgr.get_nearby_npcs(0.0, 0.0, 20.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].id, "a");
    }

    #[test]
    fn test_npc_manager_update() {
        let mut mgr = NPCManager::new();
        let mut npc = NPC::new("g", "Guard", Vec2::new(0.0, 0.0))
            .with_health(50.0)
            .with_aggro(100.0, 200.0);
        npc.behavior.set_flag("should_wander", true);
        mgr.add_npc(npc);

        let events = mgr.update_all(0.1, Vec2::new(50.0, 0.0), 8.0);
        // Guard should aggro
        assert!(events.iter().any(|e| matches!(e, NPCEvent::Aggro { .. })));
    }

    #[test]
    fn test_npc_loot_and_xp() {
        let npc = NPC::new("g", "Goblin", Vec2::zero())
            .with_loot("gold", 5)
            .with_xp(25);
        assert_eq!(npc.loot_table.len(), 1);
        assert_eq!(npc.xp_reward, 25);
    }
}
