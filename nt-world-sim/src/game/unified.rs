use crate::core::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode { Stardew, Moba, Evolution, Sandbox }
impl Default for GameMode { fn default() -> Self { GameMode::Stardew } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerRole { Fighter, Mage, Assassin }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvolutionStage { Awareness, Perception, Intelligence, Empathy, Transcendence }
impl EvolutionStage {
    pub fn level(&self) -> u32 { match self { Self::Awareness=>1, Self::Perception=>2, Self::Intelligence=>3, Self::Empathy=>4, Self::Transcendence=>5 } }
    pub fn name(&self) -> &str { match self { Self::Awareness=>"Awareness", Self::Perception=>"Perception", Self::Intelligence=>"Intelligence", Self::Empathy=>"Empathy", Self::Transcendence=>"Transcendence" } }
}

#[derive(Debug, Clone)]
pub struct Ability { pub id: u32, pub name: String, pub cooldown: f32, pub mana_cost: f32, pub damage: f32, pub current_cooldown: f32 }
impl Ability {
    pub fn new(id: u32, name: &str, cooldown: f32, mana_cost: f32, damage: f32) -> Self { Self { id, name: name.to_string(), cooldown, mana_cost, damage, current_cooldown: 0.0 } }
    pub fn can_cast(&self, mana: f32) -> bool { self.current_cooldown <= 0.0 && mana >= self.mana_cost }
    pub fn start_cooldown(&mut self) { self.current_cooldown = self.cooldown; }
    pub fn tick(&mut self, dt: f32) { if self.current_cooldown > 0.0 { self.current_cooldown = (self.current_cooldown - dt).max(0.0); } }
}

#[derive(Debug, Clone)]
pub struct Tower { pub position: Vec2, pub hp: f32, pub max_hp: f32, pub attack: f32, pub range: f32, pub team: u8, pub alive: bool }
impl Tower {
    pub fn new(position: Vec2, team: u8) -> Self { Self { position, hp: 2500.0, max_hp: 2500.0, attack: 100.0, range: 300.0, team, alive: true } }
}

#[derive(Debug, Clone)]
pub struct Minion { pub position: Vec2, pub hp: f32, pub max_hp: f32, pub attack: f32, pub speed: f32, pub team: u8, pub alive: bool, pub gold_value: u32 }
impl Minion {
    pub fn new(position: Vec2, team: u8) -> Self { Self { position, hp: 500.0, max_hp: 500.0, attack: 20.0, speed: 100.0, team, alive: true, gold_value: 20 } }
}

#[derive(Debug, Clone)]
pub struct Rival { pub position: Vec2, pub hp: f32, pub max_hp: f32, pub attack: f32, pub speed: f32, pub stage: EvolutionStage, pub alive: bool, pub respawn_timer: f32 }
impl Rival {
    pub fn new(position: Vec2, stage: EvolutionStage) -> Self {
        let (hp, atk) = match stage { EvolutionStage::Awareness=>(100.0,5.0), EvolutionStage::Perception=>(200.0,10.0), EvolutionStage::Intelligence=>(400.0,20.0), EvolutionStage::Empathy=>(800.0,40.0), EvolutionStage::Transcendence=>(1600.0,80.0) };
        Self { position, hp, max_hp: hp, attack: atk, speed: 80.0, stage, alive: true, respawn_timer: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct UnifiedPlayer {
    pub position: Vec2, pub hp: f32, pub max_hp: f32, pub mana: f32, pub max_mana: f32,
    pub energy: u32, pub max_energy: u32, pub level: u32, pub xp: u32, pub gold: u32,
    pub abilities: Vec<Ability>, pub evolution_stage: EvolutionStage, pub evolution_xp: u32,
    pub role: PlayerRole, pub alive: bool, pub respawn_timer: f32,
}
impl UnifiedPlayer {
    pub fn new_stardew(position: Vec2) -> Self {
        Self { position, hp:100.0, max_hp:100.0, mana:100.0, max_mana:100.0, energy:100, max_energy:100, level:1, xp:0, gold:500, abilities:Vec::new(), evolution_stage:EvolutionStage::Awareness, evolution_xp:0, role:PlayerRole::Fighter, alive:true, respawn_timer:0.0 }
    }
    pub fn new_moba(position: Vec2, role: PlayerRole) -> Self {
        let abs = match role {
            PlayerRole::Fighter => vec![Ability::new(1,"Strike",5.0,20.0,50.0),Ability::new(2,"Shield Bash",8.0,30.0,40.0),Ability::new(3,"War Cry",12.0,40.0,0.0),Ability::new(4,"Execute",60.0,100.0,200.0)],
            PlayerRole::Mage => vec![Ability::new(1,"Fireball",4.0,25.0,80.0),Ability::new(2,"Frost Nova",10.0,35.0,60.0),Ability::new(3,"Blink",15.0,40.0,0.0),Ability::new(4,"Meteor",70.0,120.0,300.0)],
            PlayerRole::Assassin => vec![Ability::new(1,"Backstab",4.0,20.0,70.0),Ability::new(2,"Shadow Step",6.0,25.0,0.0),Ability::new(3,"Poison Blade",8.0,30.0,50.0),Ability::new(4,"Death Mark",50.0,100.0,250.0)],
        };
        Self { position, hp:800.0, max_hp:800.0, mana:400.0, max_mana:400.0, energy:0, max_energy:0, level:1, xp:0, gold:500, abilities:abs, evolution_stage:EvolutionStage::Awareness, evolution_xp:0, role, alive:true, respawn_timer:0.0 }
    }
    pub fn new_evolution(position: Vec2) -> Self {
        Self { position, hp:100.0, max_hp:100.0, mana:50.0, max_mana:50.0, energy:0, max_energy:0, level:1, xp:0, gold:0,
            abilities:vec![Ability::new(1,"Pulse",2.0,10.0,20.0),Ability::new(2,"Dash",5.0,15.0,0.0),Ability::new(3,"Absorb",8.0,20.0,30.0),Ability::new(4,"Shield",12.0,25.0,0.0)],
            evolution_stage:EvolutionStage::Awareness, evolution_xp:0, role:PlayerRole::Fighter, alive:true, respawn_timer:0.0 }
    }
}

pub struct UnifiedGameWorld {
    pub mode: GameMode, pub player: UnifiedPlayer, pub towers: Vec<Tower>,
    pub minions: Vec<Minion>, pub rivals: Vec<Rival>, pub time: f64, pub tick_count: u64,
}
impl UnifiedGameWorld {
    pub fn new(mode: GameMode) -> Self {
        let player = match mode {
            GameMode::Stardew => UnifiedPlayer::new_stardew(Vec2::new(200.0,200.0)),
            GameMode::Moba => UnifiedPlayer::new_moba(Vec2::new(100.0,400.0), PlayerRole::Fighter),
            GameMode::Evolution => UnifiedPlayer::new_evolution(Vec2::new(400.0,400.0)),
            GameMode::Sandbox => UnifiedPlayer::new_stardew(Vec2::new(200.0,200.0)),
        };
        Self { mode, player, towers:Vec::new(), minions:Vec::new(), rivals:Vec::new(), time:0.0, tick_count:0 }
    }
    pub fn tick(&mut self, dt: f32) {
        self.time += dt as f64;
        self.tick_count += 1;
        for a in &mut self.player.abilities { a.tick(dt); }
        match self.mode {
            GameMode::Moba => self.tick_moba(dt),
            GameMode::Evolution => self.tick_evolution(dt),
            _ => {}
        }
    }
    fn tick_moba(&mut self, dt: f32) {
        for m in &mut self.minions {
            if m.alive {
                if let Some(t) = self.towers.iter().find(|t| t.team != m.team && t.alive) {
                    let dir = (t.position - m.position).normalized();
                    m.position += dir * m.speed * dt;
                }
            }
        }
        if !self.player.alive {
            self.player.respawn_timer -= dt;
            if self.player.respawn_timer <= 0.0 {
                self.player.alive = true;
                self.player.hp = self.player.max_hp;
                self.player.position = Vec2::new(100.0, 400.0);
            }
        }
    }
    fn tick_evolution(&mut self, dt: f32) {
        for r in &mut self.rivals {
            if r.alive {
                let dir = (self.player.position - r.position).normalized();
                r.position += dir * r.speed * dt;
            } else {
                r.respawn_timer -= dt;
                if r.respawn_timer <= 0.0 { r.alive = true; r.hp = r.max_hp; }
            }
        }
        self.check_evolution();
    }
    fn check_evolution(&mut self) {
        let xp = self.player.evolution_xp;
        let next = match self.player.evolution_stage {
            EvolutionStage::Awareness if xp >= 100 => Some(EvolutionStage::Perception),
            EvolutionStage::Perception if xp >= 300 => Some(EvolutionStage::Intelligence),
            EvolutionStage::Intelligence if xp >= 600 => Some(EvolutionStage::Empathy),
            EvolutionStage::Empathy if xp >= 1000 => Some(EvolutionStage::Transcendence),
            _ => None,
        };
        if let Some(s) = next { self.player.evolution_stage = s; }
    }
    pub fn spawn_moba_map(&mut self) {
        self.towers.push(Tower::new(Vec2::new(800.0,4000.0),0));
        self.towers.push(Tower::new(Vec2::new(4000.0,800.0),0));
        self.towers.push(Tower::new(Vec2::new(4000.0,4000.0),0));
        self.towers.push(Tower::new(Vec2::new(800.0,800.0),1));
        self.towers.push(Tower::new(Vec2::new(4000.0,800.0),1));
        self.towers.push(Tower::new(Vec2::new(800.0,4000.0),1));
    }
    pub fn spawn_evolution_rivals(&mut self) {
        for i in 0..5 { self.rivals.push(Rival::new(Vec2::new(100.0+i as f32*150.0, 300.0), EvolutionStage::Awareness)); }
    }
}
impl Default for UnifiedGameWorld { fn default() -> Self { Self::new(GameMode::Stardew) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_stardew() { let w = UnifiedGameWorld::new(GameMode::Stardew); assert_eq!(w.mode, GameMode::Stardew); assert!(w.player.alive); }
    #[test] fn test_moba() { let w = UnifiedGameWorld::new(GameMode::Moba); assert_eq!(w.player.abilities.len(), 4); }
    #[test] fn test_evolution() { let w = UnifiedGameWorld::new(GameMode::Evolution); assert_eq!(w.player.evolution_stage, EvolutionStage::Awareness); }
    #[test] fn test_evolution_progress() { let mut w = UnifiedGameWorld::new(GameMode::Evolution); w.player.evolution_xp = 150; w.tick(0.016); assert_eq!(w.player.evolution_stage, EvolutionStage::Perception); }
    #[test] fn test_ability_cooldown() { let mut a = Ability::new(1,"T",5.0,10.0,50.0); assert!(a.can_cast(20.0)); a.start_cooldown(); assert!(!a.can_cast(20.0)); a.tick(6.0); assert!(a.can_cast(20.0)); }
    #[test] fn test_tower() { let t = Tower::new(Vec2::new(0.0,0.0), 0); assert_eq!(t.hp, 2500.0); }
    #[test] fn test_minion() { let m = Minion::new(Vec2::new(0.0,0.0), 0); assert_eq!(m.gold_value, 20); }
    #[test] fn test_rival() { let r = Rival::new(Vec2::new(0.0,0.0), EvolutionStage::Intelligence); assert_eq!(r.hp, 400.0); }
    #[test] fn test_tick() { let mut w = UnifiedGameWorld::new(GameMode::Stardew); w.tick(0.016); assert_eq!(w.tick_count, 1); }
    #[test] fn test_moba_map() { let mut w = UnifiedGameWorld::new(GameMode::Moba); w.spawn_moba_map(); assert_eq!(w.towers.len(), 6); }
    #[test] fn test_rivals() { let mut w = UnifiedGameWorld::new(GameMode::Evolution); w.spawn_evolution_rivals(); assert_eq!(w.rivals.len(), 5); }
}
