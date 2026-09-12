use crate::core::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineralType {
    ThoughtFragment, CuriosityShard, LogicCrystal, EmpathyGem,
    CreativityOrb, FocusStone, WisdomOre, VoidEssence,
}

#[derive(Debug, Clone)]
pub struct Mineral {
    pub mineral_type: MineralType,
    pub hardness: u32,
    pub xp_reward: u32,
    pub drop_id: u32,
    pub drop_quantity: u32,
}

impl Mineral {
    pub fn all() -> Vec<Self> {
        vec![
            Self { mineral_type: MineralType::ThoughtFragment, hardness: 1, xp_reward: 5, drop_id: 3001, drop_quantity: 1 },
            Self { mineral_type: MineralType::CuriosityShard, hardness: 2, xp_reward: 8, drop_id: 3002, drop_quantity: 1 },
            Self { mineral_type: MineralType::LogicCrystal, hardness: 3, xp_reward: 12, drop_id: 3003, drop_quantity: 1 },
            Self { mineral_type: MineralType::EmpathyGem, hardness: 2, xp_reward: 10, drop_id: 3004, drop_quantity: 1 },
            Self { mineral_type: MineralType::CreativityOrb, hardness: 4, xp_reward: 15, drop_id: 3005, drop_quantity: 1 },
            Self { mineral_type: MineralType::FocusStone, hardness: 3, xp_reward: 12, drop_id: 3006, drop_quantity: 1 },
            Self { mineral_type: MineralType::WisdomOre, hardness: 5, xp_reward: 20, drop_id: 3007, drop_quantity: 1 },
            Self { mineral_type: MineralType::VoidEssence, hardness: 6, xp_reward: 30, drop_id: 3008, drop_quantity: 1 },
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineTile { Empty, Mineral(MineralType), Ladder, Staircase, Enemy(EnemyType), Chest }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType { KnowledgeGoblin, ConfusionSlime, DoubtBat, FearSpider }

#[derive(Debug, Clone)]
pub struct MineFloor {
    pub depth: u32,
    pub tiles: Vec<Vec<MineTile>>,
    pub width: u32,
    pub height: u32,
    pub cleared: bool,
    pub has_ladder: bool,
}

impl MineFloor {
    pub fn generate(depth: u32) -> Self {
        let width = 20;
        let height = 15;
        let mut tiles = vec![vec![MineTile::Empty; width as usize]; height as usize];

        let mineral_count = 10 + depth as usize * 2;
        let minerals = Mineral::all();

        for i in 0..mineral_count.min((width * height / 4) as usize) {
            let x = pseudo_random(depth * 1000 + i as u32) % width;
            let y = pseudo_random(depth * 2000 + i as u32) % height;
            let mineral_idx = pseudo_random(depth * 3000 + i as u32) as usize % minerals.len();
            tiles[y as usize][x as usize] = MineTile::Mineral(minerals[mineral_idx].mineral_type);
        }

        let enemy_count = 3 + depth as usize;
        for i in 0..enemy_count.min(8) {
            let x = pseudo_random(depth * 4000 + i as u32 * 100) % width;
            let y = pseudo_random(depth * 5000 + i as u32 * 100) % height;
            let enemy_type = match pseudo_random(depth * 6000 + i as u32) % 4 {
                0 => EnemyType::KnowledgeGoblin,
                1 => EnemyType::ConfusionSlime,
                2 => EnemyType::DoubtBat,
                _ => EnemyType::FearSpider,
            };
            tiles[y as usize][x as usize] = MineTile::Enemy(enemy_type);
        }

        tiles[1][1] = MineTile::Ladder;

        Self { depth, tiles, width, height, cleared: false, has_ladder: false }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> MineTile {
        if x < self.width && y < self.height {
            self.tiles[y as usize][x as usize]
        } else {
            MineTile::Empty
        }
    }

    pub fn mine_tile(&mut self, x: u32, y: u32) -> Option<MineTile> {
        if x < self.width && y < self.height {
            let tile = self.tiles[y as usize][x as usize];
            if tile != MineTile::Empty {
                self.tiles[y as usize][x as usize] = MineTile::Empty;
                Some(tile)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct MineState {
    pub current_floor: u32,
    pub floors: HashMap<u32, MineFloor>,
    pub player_x: u32,
    pub player_y: u32,
    pub in_mine: bool,
}

impl Resource for MineState {}

impl MineState {
    pub fn new() -> Self {
        Self {
            current_floor: 1,
            floors: HashMap::new(),
            player_x: 1,
            player_y: 1,
            in_mine: false,
        }
    }

    pub fn enter_mine(&mut self) {
        self.in_mine = true;
        self.current_floor = 1;
        self.player_x = 1;
        self.player_y = 1;
        self.get_or_create_floor(1);
    }

    pub fn descend(&mut self) {
        self.current_floor += 1;
        self.player_x = 1;
        self.player_y = 1;
        self.get_or_create_floor(self.current_floor);
    }

    pub fn ascend(&mut self) {
        if self.current_floor > 1 {
            self.current_floor -= 1;
            self.player_x = 1;
            self.player_y = 1;
        } else {
            self.in_mine = false;
        }
    }

    fn get_or_create_floor(&mut self, depth: u32) {
        if !self.floors.contains_key(&depth) {
            self.floors.insert(depth, MineFloor::generate(depth));
        }
    }

    pub fn current_floor_mut(&mut self) -> Option<&mut MineFloor> {
        self.floors.get_mut(&self.current_floor)
    }
}

impl Default for MineState {
    fn default() -> Self { Self::new() }
}

fn pseudo_random(seed: u32) -> u32 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mineral_count() {
        let minerals = Mineral::all();
        assert_eq!(minerals.len(), 8);
    }

    #[test]
    fn test_floor_generation() {
        let floor = MineFloor::generate(1);
        assert_eq!(floor.width, 20);
        assert_eq!(floor.height, 15);
        assert_eq!(floor.get_tile(1, 1), MineTile::Ladder);
    }

    #[test]
    fn test_mine_tile() {
        let mut floor = MineFloor::generate(1);
        let mut found_mineral = false;
        for y in 0..floor.height {
            for x in 0..floor.width {
                if let MineTile::Mineral(_) = floor.get_tile(x, y) {
                    let mined = floor.mine_tile(x, y);
                    assert!(mined.is_some());
                    assert_eq!(floor.get_tile(x, y), MineTile::Empty);
                    found_mineral = true;
                    break;
                }
            }
            if found_mineral { break; }
        }
        assert!(found_mineral);
    }

    #[test]
    fn test_mine_state_enter_exit() {
        let mut state = MineState::new();
        assert!(!state.in_mine);
        state.enter_mine();
        assert!(state.in_mine);
        assert_eq!(state.current_floor, 1);
        state.ascend();
        assert!(!state.in_mine);
    }

    #[test]
    fn test_mine_state_descend() {
        let mut state = MineState::new();
        state.enter_mine();
        state.descend();
        assert_eq!(state.current_floor, 2);
        assert!(state.floors.contains_key(&2));
    }

    #[test]
    fn test_floor_depth_scaling() {
        let floor1 = MineFloor::generate(1);
        let floor5 = MineFloor::generate(5);
        let count_minerals = |floor: &MineFloor| -> usize {
            let mut c = 0;
            for row in &floor.tiles {
                for tile in row {
                    if matches!(tile, MineTile::Mineral(_)) { c += 1; }
                }
            }
            c
        };
        assert!(count_minerals(&floor5) >= count_minerals(&floor1));
    }
}
