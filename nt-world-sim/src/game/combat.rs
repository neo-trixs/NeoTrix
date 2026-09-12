use crate::core::Resource;
use super::mining::EnemyType;

#[derive(Debug, Clone)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub xp_reward: u32,
    pub drop_id: Option<u32>,
    pub x: u32,
    pub y: u32,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, depth: u32) -> Self {
        let (hp, attack, defense, xp, drop) = match enemy_type {
            EnemyType::KnowledgeGoblin => (20 + depth * 5, 5 + depth, 2, 15, Some(3001)),
            EnemyType::ConfusionSlime => (15 + depth * 4, 3 + depth, 1, 10, Some(3002)),
            EnemyType::DoubtBat => (10 + depth * 3, 4 + depth, 1, 12, Some(3003)),
            EnemyType::FearSpider => (25 + depth * 6, 6 + depth, 3, 20, Some(3004)),
        };
        Self {
            enemy_type, hp, max_hp: hp, attack, defense,
            xp_reward: xp, drop_id: drop, x: 0, y: 0,
        }
    }

    pub fn take_damage(&mut self, damage: u32) -> bool {
        let actual = damage.saturating_sub(self.defense).max(1);
        self.hp = self.hp.saturating_sub(actual);
        self.hp == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAction { Attack, Defend, UseItem(u32), Flee }

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub player_damage_taken: u32,
    pub enemy_damage_taken: u32,
    pub enemy_killed: bool,
    pub xp_gained: u32,
    pub drops: Vec<u32>,
    pub player_fled: bool,
}

pub struct CombatSystem;

impl CombatSystem {
    pub fn calculate_attack(player_attack: u32, weapon_bonus: u32, skill_level: u32) -> u32 {
        let base = player_attack + weapon_bonus;
        let skill_bonus = skill_level as f32 * 0.1;
        (base as f32 * (1.0 + skill_bonus)) as u32
    }

    pub fn resolve_combat(
        player_attack: u32, player_defense: u32,
        enemy: &mut Enemy, action: CombatAction,
        skill_level: u32,
    ) -> CombatResult {
        match action {
            CombatAction::Attack => {
                let damage = Self::calculate_attack(player_attack, 0, skill_level);
                let enemy_killed = enemy.take_damage(damage);
                let player_dmg = enemy.attack.saturating_sub(player_defense).max(1);

                let mut drops = Vec::new();
                if enemy_killed {
                    if let Some(drop_id) = enemy.drop_id {
                        drops.push(drop_id);
                    }
                }

                CombatResult {
                    player_damage_taken: player_dmg,
                    enemy_damage_taken: damage,
                    enemy_killed,
                    xp_gained: if enemy_killed { enemy.xp_reward } else { 0 },
                    drops,
                    player_fled: false,
                }
            }
            CombatAction::Defend => {
                let reduced = enemy.attack / 2;
                CombatResult {
                    player_damage_taken: reduced,
                    enemy_damage_taken: 0,
                    enemy_killed: false,
                    xp_gained: 0,
                    drops: Vec::new(),
                    player_fled: false,
                }
            }
            CombatAction::Flee => {
                let flee_chance = 0.5 + skill_level as f32 * 0.05;
                let fled = pseudo_random_f32() < flee_chance;
                CombatResult {
                    player_damage_taken: if fled { 0 } else { enemy.attack },
                    enemy_damage_taken: 0,
                    enemy_killed: false,
                    xp_gained: 0,
                    drops: Vec::new(),
                    player_fled: fled,
                }
            }
            CombatAction::UseItem(_item_id) => {
                CombatResult {
                    player_damage_taken: 0, enemy_damage_taken: 0,
                    enemy_killed: false, xp_gained: 0, drops: Vec::new(), player_fled: false,
                }
            }
        }
    }
}

pub struct CombatState {
    pub in_combat: bool,
    pub current_enemy: Option<Enemy>,
    pub player_attack: u32,
    pub player_defense: u32,
    pub player_hp: u32,
    pub player_max_hp: u32,
    pub total_xp: u32,
}

impl Resource for CombatState {}

impl CombatState {
    pub fn new() -> Self {
        Self {
            in_combat: false,
            current_enemy: None,
            player_attack: 10,
            player_defense: 5,
            player_hp: 100,
            player_max_hp: 100,
            total_xp: 0,
        }
    }

    pub fn start_combat(&mut self, enemy_type: EnemyType, depth: u32, x: u32, y: u32) {
        let mut enemy = Enemy::new(enemy_type, depth);
        enemy.x = x;
        enemy.y = y;
        self.current_enemy = Some(enemy);
        self.in_combat = true;
    }

    pub fn resolve(&mut self, action: CombatAction, skill_level: u32) -> Option<CombatResult> {
        if !self.in_combat { return None; }
        let enemy = self.current_enemy.as_mut()?;
        let result = CombatSystem::resolve_combat(
            self.player_attack, self.player_defense, enemy, action, skill_level,
        );
        self.player_hp = self.player_hp.saturating_sub(result.player_damage_taken);
        self.total_xp += result.xp_gained;
        if result.enemy_killed {
            self.in_combat = false;
            self.current_enemy = None;
        }
        if result.player_fled {
            self.in_combat = false;
            self.current_enemy = None;
        }
        Some(result)
    }
}

impl Default for CombatState {
    fn default() -> Self { Self::new() }
}

fn pseudo_random_f32() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_creation() {
        let enemy = Enemy::new(EnemyType::KnowledgeGoblin, 1);
        assert_eq!(enemy.hp, 25);
        assert_eq!(enemy.max_hp, 25);
        assert_eq!(enemy.attack, 6);
        assert!(enemy.hp > 0);
    }

    #[test]
    fn test_enemy_take_damage() {
        let mut enemy = Enemy::new(EnemyType::ConfusionSlime, 1);
        let killed = enemy.take_damage(20);
        assert!(killed);
        assert_eq!(enemy.hp, 0);
    }

    #[test]
    fn test_enemy_defense_reduces_damage() {
        let mut enemy = Enemy::new(EnemyType::FearSpider, 1);
        enemy.take_damage(5);
        assert!(enemy.hp > 0);
        assert!(enemy.hp < enemy.max_hp);
    }

    #[test]
    fn test_attack_damage() {
        let damage = CombatSystem::calculate_attack(10, 0, 1);
        assert!(damage >= 11);
    }

    #[test]
    fn test_resolve_attack() {
        let mut enemy = Enemy::new(EnemyType::KnowledgeGoblin, 1);
        let result = CombatSystem::resolve_combat(100, 5, &mut enemy, CombatAction::Attack, 1);
        assert!(result.enemy_damage_taken > 0);
        assert!(!result.player_fled);
    }

    #[test]
    fn test_resolve_defend() {
        let mut enemy = Enemy::new(EnemyType::DoubtBat, 1);
        let result = CombatSystem::resolve_combat(10, 5, &mut enemy, CombatAction::Defend, 1);
        assert_eq!(result.enemy_damage_taken, 0);
        assert!(result.player_damage_taken < enemy.attack);
    }

    #[test]
    fn test_combat_state_flow() {
        let mut state = CombatState::new();
        state.start_combat(EnemyType::ConfusionSlime, 1, 5, 5);
        assert!(state.in_combat);
        let result = state.resolve(CombatAction::Attack, 1).unwrap();
        assert!(result.enemy_damage_taken > 0);
    }

    #[test]
    fn test_combat_state_not_in_combat() {
        let mut state = CombatState::new();
        assert!(state.resolve(CombatAction::Attack, 1).is_none());
    }
}
