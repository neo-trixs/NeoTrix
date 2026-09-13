use std::collections::HashMap;

use super::renderer::Vec2;

// ---------------------------------------------------------------------------
// NPCState
// ---------------------------------------------------------------------------

/// Behavioral state of an NPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NPCState {
    /// Standing still, playing idle animation.
    Idle,
    /// Following a predefined patrol route.
    Patrol,
    /// Moving randomly within a radius.
    Wander,
    /// Chasing a target entity.
    Chase,
    /// In combat, attacking a target.
    Attack,
    /// Running away from a threat.
    Flee,
    /// Dead (awaiting respawn or removal).
    Dead,
}

impl NPCState {
    /// Whether this state allows movement.
    pub fn can_move(&self) -> bool {
        matches!(self, Self::Patrol | Self::Wander | Self::Chase | Self::Flee)
    }

    /// Whether this state is in combat.
    pub fn in_combat(&self) -> bool {
        matches!(self, Self::Chase | Self::Attack)
    }
}

// ---------------------------------------------------------------------------
// NPCSchedule
// ---------------------------------------------------------------------------

/// Time-of-day schedule entry.
#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    /// Hour (0..23).
    pub hour: u8,
    /// Location name or coordinates.
    pub location: String,
    /// Activity description.
    pub activity: String,
    /// Optional: NPC state during this schedule.
    pub state: NPCState,
}

/// Daily schedule for an NPC.
#[derive(Debug, Clone)]
pub struct NPCSchedule {
    pub entries: Vec<ScheduleEntry>,
    /// The NPC's home position.
    pub home: Vec2,
    /// wander_radius when idle.
    pub wander_radius: f32,
}

impl NPCSchedule {
    pub fn new(home: Vec2) -> Self {
        Self {
            entries: Vec::new(),
            home,
            wander_radius: 64.0,
        }
    }

    pub fn with_entry(mut self, entry: ScheduleEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Get the current activity based on in-game hour.
    pub fn current_activity(&self, hour: u8) -> Option<&ScheduleEntry> {
        self.entries.iter().find(|e| e.hour == hour)
    }

    /// Get the activity for the nearest hour.
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

/// Relationship between the NPC and the player.
#[derive(Debug, Clone)]
pub struct NPCRelationship {
    /// Favorability (-100..100). Negative = hostile, positive = friendly.
    pub favorability: i32,
    /// Trust level (0..100). Higher = more willing to share info/quests.
    pub trust: i32,
    /// Whether the NPC has been met.
    pub met: bool,
    /// Times interacted.
    pub interaction_count: u32,
    /// Last interaction game-time.
    pub last_interaction: f64,
}

impl NPCRelationship {
    pub fn new() -> Self {
        Self {
            favorability: 0,
            trust: 0,
            met: false,
            interaction_count: 0,
            last_interaction: 0.0,
        }
    }

    pub fn with_favorability(mut self, val: i32) -> Self {
        self.favorability = val.clamp(-100, 100);
        self
    }

    pub fn with_trust(mut self, val: i32) -> Self {
        self.trust = val.clamp(0, 100);
        self
    }

    pub fn modify_favorability(&mut self, amount: i32) {
        self.favorability = (self.favorability + amount).clamp(-100, 100);
    }

    pub fn modify_trust(&mut self, amount: i32) {
        self.trust = (self.trust + amount).clamp(0, 100);
    }

    /// Record an interaction.
    pub fn interact(&mut self, game_time: f64) {
        self.met = true;
        self.interaction_count += 1;
        self.last_interaction = game_time;
    }

    /// Relationship tier based on favorability.
    pub fn tier(&self) -> RelationTier {
        match self.favorability {
            -100..=-50 => RelationTier::Hostile,
            -49..=-10 => RelationTier::Unfriendly,
            -9..=9 => RelationTier::Neutral,
            10..=49 => RelationTier::Friendly,
            50..=100 => RelationTier::Allied,
        }
    }
}

impl Default for NPCRelationship {
    fn default() -> Self {
        Self::new()
    }
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

/// Transition rule for the NPC state machine.
#[derive(Debug, Clone)]
pub struct BehaviorTransition {
    /// From state.
    pub from: NPCState,
    /// To state.
    pub to: NPCState,
    /// Condition name (checked against behavior context).
    pub condition: String,
}

/// NPC behavior state machine.
pub struct NPCBehavior {
    pub state: NPCState,
    pub transitions: Vec<BehaviorTransition>,
    /// Named flags for transition conditions.
    pub flags: HashMap<String, bool>,
    /// Named numeric values for conditions.
    pub stats: HashMap<String, f32>,
    /// Target entity ID (for Chase/Attack).
    pub target_id: Option<u64>,
    /// Patrol waypoints (for Patrol state).
    pub patrol_points: Vec<Vec2>,
    /// Current patrol index.
    pub patrol_index: usize,
    /// Wander center position.
    pub wander_center: Vec2,
    /// Wander radius.
    pub wander_radius: f32,
    /// Idle timer (seconds to wait before wandering).
    pub idle_timer: f32,
    /// Current idle countdown.
    pub idle_countdown: f32,
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
        }
    }

    pub fn with_transition(mut self, from: NPCState, to: NPCState, condition: &str) -> Self {
        self.transitions.push(BehaviorTransition {
            from, to, condition: condition.to_string(),
        });
        self
    }

    pub fn with_patrol(mut self, points: Vec<Vec2>) -> Self {
        self.patrol_points = points;
        self
    }

    pub fn with_wander_radius(mut self, radius: f32) -> Self {
        self.wander_radius = radius;
        self
    }

    /// Set a named flag.
    pub fn set_flag(&mut self, name: &str, value: bool) {
        self.flags.insert(name.to_string(), value);
    }

    /// Get a named flag.
    pub fn get_flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }

    /// Set a named stat.
    pub fn set_stat(&mut self, name: &str, value: f32) {
        self.stats.insert(name.to_string(), value);
    }

    /// Get a named stat.
    pub fn get_stat(&self, name: &str) -> f32 {
        self.stats.get(name).copied().unwrap_or(0.0)
    }

    /// Evaluate a condition string against current flags/stats.
    fn evaluate_condition(&self, condition: &str) -> bool {
        // Simple flag check: "flag_name" = true, "!flag_name" = false
        if let Some(flag_name) = condition.strip_prefix('!') {
            return !self.get_flag(flag_name);
        }
        if let Some(flag_name) = condition.strip_prefix("stat>=") {
            if let Some((stat_name, val_str)) = flag_name.split_once(':') {
                if let Ok(val) = val_str.parse::<f32>() {
                    return self.get_stat(stat_name) >= val;
                }
            }
        }
        if let Some(flag_name) = condition.strip_prefix("stat<") {
            if let Some((stat_name, val_str)) = flag_name.split_once(':') {
                if let Ok(val) = val_str.parse::<f32>() {
                    return self.get_stat(stat_name) < val;
                }
            }
        }
        // Default: check as flag
        self.get_flag(condition)
    }

    /// Run the state machine: check transitions, apply the first matching one.
    pub fn tick(&mut self, dt: f32) {
        // Check transitions from current state
        let transition = self.transitions.iter().find(|t| {
            t.from == self.state && self.evaluate_condition(&t.condition)
        });

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
            NPCState::Wander | NPCState::Chase | NPCState::Flee => {
                // Movement is handled externally via target positions
            }
            NPCState::Dead => {
                // No-op, awaiting external handling
            }
            _ => {}
        }
    }

    /// Get the next patrol waypoint.
    pub fn next_patrol_point(&self) -> Option<Vec2> {
        self.patrol_points.get(self.patrol_index).copied()
    }

    /// Get a random wander target within radius.
    pub fn wander_target(&self) -> Vec2 {
        let angle = pseudo_random_f32() * std::f32::consts::TAU;
        let dist = pseudo_random_f32() * self.wander_radius;
        Vec2::new(
            self.wander_center.x + angle.cos() * dist,
            self.wander_center.y + angle.sin() * dist,
        )
    }

    /// Check if the NPC is in combat state.
    pub fn in_combat(&self) -> bool {
        self.state.in_combat()
    }
}

impl Default for NPCBehavior {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// NPC (full entity)
// ---------------------------------------------------------------------------

/// A complete NPC definition.
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
    /// Dialogue tree IDs this NPC can trigger.
    pub dialogue_ids: Vec<String>,
    /// Quest IDs this NPC can offer.
    pub offered_quests: Vec<String>,
    /// Respawn time in seconds (None = no respawn).
    pub respawn_time: Option<f32>,
    /// Respawn countdown.
    pub respawn_countdown: f32,
}

impl NPC {
    pub fn new(id: &str, name: &str, position: Vec2) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            position,
            behavior: NPCBehavior::new(),
            schedule: None,
            relationship: NPCRelationship::new(),
            max_health: 100.0,
            health: 100.0,
            attack_power: 10.0,
            defense: 5.0,
            speed: 100.0,
            dialogue_ids: Vec::new(),
            offered_quests: Vec::new(),
            respawn_time: None,
            respawn_countdown: 0.0,
        }
    }

    pub fn with_health(mut self, hp: f32) -> Self {
        self.max_health = hp;
        self.health = hp;
        self
    }

    pub fn with_combat_stats(mut self, attack: f32, defense: f32) -> Self {
        self.attack_power = attack;
        self.defense = defense;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_schedule(mut self, schedule: NPCSchedule) -> Self {
        self.schedule = Some(schedule);
        self
    }

    pub fn with_dialogue(mut self, dialogue_id: &str) -> Self {
        self.dialogue_ids.push(dialogue_id.to_string());
        self
    }

    pub fn with_quest(mut self, quest_id: &str) -> Self {
        self.offered_quests.push(quest_id.to_string());
        self
    }

    pub fn with_respawn(mut self, seconds: f32) -> Self {
        self.respawn_time = Some(seconds);
        self
    }

    /// Take damage. Returns actual damage dealt.
    pub fn take_damage(&mut self, raw_damage: f32) -> f32 {
        let actual = (raw_damage - self.defense).max(0.0);
        self.health = (self.health - actual).max(0.0);
        if self.health <= 0.0 {
            self.behavior.state = NPCState::Dead;
        }
        actual
    }

    /// Heal the NPC.
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Is the NPC alive?
    pub fn is_alive(&self) -> bool {
        self.health > 0.0 && self.behavior.state != NPCState::Dead
    }

    /// Update respawn timer.
    pub fn update_respawn(&mut self, dt: f32) {
        if self.behavior.state == NPCState::Dead {
            if let Some(respawn_time) = self.respawn_time {
                self.respawn_countdown += dt;
                if self.respawn_countdown >= respawn_time {
                    self.respawn();
                }
            }
        }
    }

    /// Respawn the NPC.
    pub fn respawn(&mut self) {
        self.health = self.max_health;
        self.behavior.state = NPCState::Idle;
        self.respawn_countdown = 0.0;
    }

    /// Tick the NPC's behavior.
    pub fn tick(&mut self, dt: f32) {
        if self.is_alive() {
            self.behavior.tick(dt);
        } else {
            self.update_respawn(dt);
        }
    }
}

// ---------------------------------------------------------------------------
// NPCManager
// ---------------------------------------------------------------------------

/// Manages all NPCs in the world.
pub struct NPCManager {
    npcs: HashMap<String, NPC>,
}

impl NPCManager {
    pub fn new() -> Self {
        Self { npcs: HashMap::new() }
    }

    pub fn add_npc(&mut self, npc: NPC) {
        self.npcs.insert(npc.id.clone(), npc);
    }

    pub fn get_npc(&self, id: &str) -> Option<&NPC> {
        self.npcs.get(id)
    }

    pub fn get_npc_mut(&mut self, id: &str) -> Option<&mut NPC> {
        self.npcs.get_mut(id)
    }

    pub fn remove_npc(&mut self, id: &str) -> Option<NPC> {
        self.npcs.remove(id)
    }

    pub fn npc_count(&self) -> usize {
        self.npcs.len()
    }

    pub fn alive_npcs(&self) -> Vec<&NPC> {
        self.npcs.values().filter(|n| n.is_alive()).collect()
    }

    pub fn npcs_in_range(&self, center: Vec2, range: f32) -> Vec<&NPC> {
        self.npcs.values()
            .filter(|n| {
                let dx = n.position.x - center.x;
                let dy = n.position.y - center.y;
                dx * dx + dy * dy <= range * range
            })
            .collect()
    }

    /// Tick all NPCs.
    pub fn tick_all(&mut self, dt: f32) {
        for npc in self.npcs.values_mut() {
            npc.tick(dt);
        }
    }
}

impl Default for NPCManager {
    fn default() -> Self {
        Self::new()
    }
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
    }

    #[test]
    fn test_npc_relationship() {
        let mut rel = NPCRelationship::new();
        assert_eq!(rel.tier(), RelationTier::Neutral);
        rel.modify_favorability(60);
        assert_eq!(rel.tier(), RelationTier::Allied);
        rel.modify_favorability(-150);
        assert_eq!(rel.tier(), RelationTier::Hostile);
        rel.interact(100.0);
        assert!(rel.met);
        assert_eq!(rel.interaction_count, 1);
    }

    #[test]
    fn test_npc_schedule() {
        let schedule = NPCSchedule::new(Vec2::zero())
            .with_entry(ScheduleEntry {
                hour: 8, location: "market".to_string(),
                activity: "sell goods".to_string(), state: NPCState::Idle,
            })
            .with_entry(ScheduleEntry {
                hour: 12, location: "tavern".to_string(),
                activity: "eat lunch".to_string(), state: NPCState::Idle,
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
        behavior.idle_countdown = 10.0; // won't auto-trigger
        behavior.tick(0.1);
        assert_eq!(behavior.state, NPCState::Wander);
    }

    #[test]
    fn test_npc_combat() {
        let mut npc = NPC::new("goblin", "Goblin", Vec2::zero())
            .with_health(30.0)
            .with_combat_stats(5.0, 2.0);
        let dmg = npc.take_damage(10.0);
        assert!((dmg - 8.0).abs() < 0.01);
        assert!((npc.health - 22.0).abs() < 0.01);
        npc.take_damage(25.0);
        assert!(!npc.is_alive());
        assert_eq!(npc.behavior.state, NPCState::Dead);
    }

    #[test]
    fn test_npc_respawn() {
        let mut npc = NPC::new("goblin", "Goblin", Vec2::zero())
            .with_health(10.0)
            .with_respawn(1.0);
        npc.take_damage(20.0);
        assert!(!npc.is_alive());
        npc.update_respawn(1.5);
        assert!(npc.is_alive());
        assert_eq!(npc.health, 10.0);
    }

    #[test]
    fn test_npc_manager() {
        let mut mgr = NPCManager::new();
        mgr.add_npc(NPC::new("a", "A", Vec2::new(10.0, 10.0)));
        mgr.add_npc(NPC::new("b", "B", Vec2::new(100.0, 100.0)));
        assert_eq!(mgr.npc_count(), 2);

        let nearby = mgr.npcs_in_range(Vec2::new(0.0, 0.0), 20.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].id, "a");
    }

    #[test]
    fn test_npc_wander_target() {
        let behavior = NPCBehavior::new();
        let target = behavior.wander_target();
        let dist = (target.x * target.x + target.y * target.y).sqrt();
        assert!(dist <= behavior.wander_radius);
    }
}
