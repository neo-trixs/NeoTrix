//! NT-GAME Autonomous Evolution Loop
//!
//! Fully autonomous daemon — no human interaction required.
//! Continuously trains the consciousness entity through self-play,
//! adapts difficulty, and evolves game rules via constellation unlocks.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::framework::{Action, Observation, StepResult};
use super::play::adaptive::{AdaptiveDifficultyConfig, DifficultyAdjuster};

// ═══════════════════════════════════════════════════════════════════
// Config
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvolutionConfig {
    pub episodes_per_round: usize,
    pub max_turns: usize,
    pub max_constellation: u8,
    pub auto_advance: bool,
    pub advance_threshold: f64,
    pub min_episodes_before_advance: usize,
}

impl Default for GameEvolutionConfig {
    fn default() -> Self {
        Self {
            episodes_per_round: 10,
            max_turns: 50,
            max_constellation: 5,
            auto_advance: true,
            advance_threshold: 0.6,
            min_episodes_before_advance: 10,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameEvolutionState {
    pub current_constellation: u8,
    pub total_episodes: usize,
    pub total_steps: usize,
    pub constellation_scores: HashMap<u8, f64>,
    pub constellation_episodes: HashMap<u8, usize>,
    pub is_running: bool,
}

// ═══════════════════════════════════════════════════════════════════
// Tick Report
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTickReport {
    pub constellation: u8,
    pub game_name: String,
    pub episodes_played: usize,
    pub wins: usize,
    pub losses: usize,
    pub draws: usize,
    pub avg_reward: f64,
    pub avg_turns: f64,
    pub win_rate: f64,
    pub phi_avg: f64,
    pub health_score: f64,
    pub constellation_advanced: bool,
    pub timestamp: String,
}

// ═══════════════════════════════════════════════════════════════════
// Inline Game Engines (self-contained, no cross-module imports)
// ═══════════════════════════════════════════════════════════════════

/// Minimal game engine for autonomous training.
trait AutoGame {
    fn reset(&mut self, seed: u64);
    fn step(&mut self, action: &Action) -> StepResult;
    fn legal_actions(&self, player: u32) -> Vec<Action>;
    fn is_terminal(&self) -> bool;
    fn reward(&self, player: u32) -> f64;
    fn phi(&self) -> f64;
    fn name(&self) -> &str;
    fn board_hexagrams(&self) -> Vec<u8>;
}

// ─── HexTicTacToe (C0) ───

struct AutoTicTacToe {
    board: [u8; 9], // 0=empty, 1=X, 2=O
    turn: usize,
    terminal: bool,
    winner: Option<u8>,
}

impl AutoTicTacToe {
    fn new() -> Self {
        Self { board: [0; 9], turn: 0, terminal: false, winner: None }
    }

    fn check(&mut self) {
        let lines = [
            [0,1,2],[3,4,5],[6,7,8],
            [0,3,6],[1,4,7],[2,5,8],
            [0,4,8],[2,4,6],
        ];
        for line in lines {
            let a = self.board[line[0]];
            if a != 0 && a == self.board[line[1]] && a == self.board[line[2]] {
                self.terminal = true;
                self.winner = Some(a);
                return;
            }
        }
        if self.turn >= 9 {
            self.terminal = true;
        }
    }
}

impl AutoGame for AutoTicTacToe {
    fn reset(&mut self, _seed: u64) {
        self.board = [0; 9];
        self.turn = 0;
        self.terminal = false;
        self.winner = None;
    }

    fn step(&mut self, action: &Action) -> StepResult {
        let target = action.params.get("pos").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        if target < 9 && self.board[target] == 0 {
            self.board[target] = if self.turn % 2 == 0 { 1 } else { 2 };
            self.turn += 1;
            self.check();
        }
        let reward = if self.terminal {
            match self.winner {
                Some(1) => 1.0,
                Some(2) => -1.0,
                None => 0.0,
                _ => 0.0,
            }
        } else {
            0.0
        };
        StepResult {
            observation: Observation {
                text: format!("TicTacToe turn={} board={:?}", self.turn, self.board),
                legal_actions: vec![],
                hexagram: None,
                phi: None,
            },
            reward,
            done: self.terminal,
            info: serde_json::json!({"turn": self.turn}),
        }
    }

    fn legal_actions(&self, player: u32) -> Vec<Action> {
        if self.terminal { return vec![]; }
        self.board.iter().enumerate()
            .filter(|(_, &v)| v == 0)
            .map(|(i, _)| Action {
                kind: "Place".into(),
                params: serde_json::json!({"pos": i}),
                actor_id: player,
            })
            .collect()
    }

    fn is_terminal(&self) -> bool { self.terminal }

    fn reward(&self, player: u32) -> f64 {
        match self.winner {
            Some(w) if w as u32 == player => 1.0,
            Some(_) => -1.0,
            None => 0.0,
        }
    }

    fn phi(&self) -> f64 {
        let filled = self.board.iter().filter(|&&v| v != 0).count() as f64;
        filled / 9.0
    }

    fn name(&self) -> &str { "HexTicTacToe" }

    fn board_hexagrams(&self) -> Vec<u8> {
        self.board.iter().map(|&v| v as u8 * 10).collect()
    }
}

// ─── Simplified 2048 (C1) ───

struct Auto2048 {
    board: [[u32; 4]; 4],
    score: u32,
    terminal: bool,
    rng: u64,
}

impl Auto2048 {
    fn new() -> Self {
        let mut g = Self { board: [[0; 4]; 4], score: 0, terminal: false, rng: 0 };
        g.spawn();
        g.spawn();
        g
    }

    fn spawn(&mut self) {
        let empty: Vec<(usize, usize)> = (0..4).flat_map(|r| (0..4).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[r][c] == 0).collect();
        if empty.is_empty() { return; }
        self.rng = self.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = (self.rng >> 33) as usize % empty.len();
        let (r, c) = empty[idx];
        self.board[r][c] = if self.rng % 10 == 0 { 4 } else { 2 };
    }

    fn slide(row: &mut [u32; 4]) -> u32 {
        let mut v: Vec<u32> = row.iter().copied().filter(|&x| x != 0).collect();
        v.resize(4, 0);
        let mut score = 0u32;
        for i in 0..3 {
            if v[i] == v[i + 1] && v[i] != 0 { v[i] *= 2; score += v[i]; v[i + 1] = 0; }
        }
        let mut r: Vec<u32> = v.into_iter().filter(|&x| x != 0).collect();
        r.resize(4, 0);
        for (i, &val) in r.iter().enumerate() { row[i] = val; }
        score
    }

    fn can_move(&self) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if self.board[r][c] == 0 { return true; }
                if c + 1 < 4 && self.board[r][c] == self.board[r][c + 1] { return true; }
                if r + 1 < 4 && self.board[r][c] == self.board[r + 1][c] { return true; }
            }
        }
        false
    }

    fn rotate(&mut self) {
        let b = self.board;
        for r in 0..4 { for c in 0..4 { self.board[c][3 - r] = b[r][c]; } }
    }

    fn move_dir(&mut self, dir: u8) -> u32 {
        let mut score = 0u32;
        for _ in 0..dir { self.rotate(); }
        for r in 0..4 { score += Self::slide(&mut self.board[r]); }
        for _ in 0..(4 - dir) % 4 { self.rotate(); }
        score
    }
}

impl AutoGame for Auto2048 {
    fn reset(&mut self, seed: u64) {
        self.board = [[0; 4]; 4];
        self.score = 0;
        self.terminal = false;
        self.rng = seed;
        self.spawn();
        self.spawn();
    }

    fn step(&mut self, action: &Action) -> StepResult {
        let dir = match action.kind.as_str() {
            "Left" => 0, "Up" => 1, "Right" => 2, "Down" => 3, _ => 0,
        };
        let old = self.board;
        let gain = self.move_dir(dir);
        if self.board == old {
            return StepResult {
                observation: Observation { text: "no change".into(), legal_actions: vec![], hexagram: None, phi: None },
                reward: 0.0, done: false, info: serde_json::json!({}),
            };
        }
        self.score += gain;
        self.spawn();
        if !self.can_move() { self.terminal = true; }
        StepResult {
            observation: Observation {
                text: format!("2048 score={} max={}", self.score, self.board.iter().flatten().max().unwrap_or(&0)),
                legal_actions: vec![], hexagram: Some((self.score as u8).min(63)), phi: None,
            },
            reward: gain as f64 / 2048.0,
            done: self.terminal,
            info: serde_json::json!({"score": self.score}),
        }
    }

    fn legal_actions(&self, player: u32) -> Vec<Action> {
        if self.terminal { return vec![]; }
        ["Left","Up","Right","Down"].iter().map(|d| Action {
            kind: d.to_string(), params: serde_json::json!({}), actor_id: player,
        }).collect()
    }

    fn is_terminal(&self) -> bool { self.terminal }
    fn reward(&self, _player: u32) -> f64 { self.score as f64 / 10000.0 }
    fn phi(&self) -> f64 {
        let max = *self.board.iter().flatten().max().unwrap_or(&1) as f64;
        (max.log2() / 12.0).min(1.0)
    }
    fn name(&self) -> &str { "2048" }
    fn board_hexagrams(&self) -> Vec<u8> {
        self.board.iter().flatten().map(|&v| (v as u8).min(63)).collect()
    }
}

// ─── Simplified HexCrucible (C2+) ───

struct AutoHexCrucible {
    cells: Vec<u8>,
    grid: usize,
    owners: Vec<i8>, // -1=none, 0=player, 1=opponent
    tokens: Vec<i8>, // -1=none, 0=player, 1=opponent
    turn: usize,
    terminal: bool,
    energy: [f64; 2],
    territory: [usize; 2],
}

impl AutoHexCrucible {
    fn new(grid_size: usize) -> Self {
        let n = grid_size * grid_size;
        Self {
            cells: (0..n).map(|i| (i % 64) as u8).collect(),
            grid: grid_size,
            owners: vec![-1; n],
            tokens: vec![-1; n],
            turn: 0,
            terminal: false,
            energy: [5.0; 2],
            territory: [0; 2],
        }
    }

    fn neighbors(&self, idx: usize) -> Vec<usize> {
        let r = idx / self.grid;
        let c = idx % self.grid;
        let mut out = Vec::new();
        let offsets = if r % 2 == 0 {
            [(-1,-1),(-1,0),(0,-1),(0,1),(1,-1),(1,0)]
        } else {
            [(-1,0),(-1,1),(0,-1),(0,1),(1,0),(1,1)]
        };
        for (dr, dc) in offsets {
            let nr = r as isize + dr;
            let nc = c as isize + dc;
            if nr >= 0 && nr < self.grid as isize && nc >= 0 && nc < self.grid as isize {
                out.push(nr as usize * self.grid + nc as usize);
            }
        }
        out
    }

    fn hamming(a: u8, b: u8) -> u32 { (a ^ b).count_ones() }

    fn compute_phi(&self) -> f64 {
        let mut total = 0u32;
        let n = self.grid * self.grid;
        for i in 0..n {
            for &j in &self.neighbors(i) {
                if j > i { total += 6 - Self::hamming(self.cells[i], self.cells[j]); }
            }
        }
        let max = (n as u32) * 6;
        if max == 0 { 0.0 } else { total as f64 / max as f64 }
    }
}

impl AutoGame for AutoHexCrucible {
    fn reset(&mut self, seed: u64) {
        let n = self.grid * self.grid;
        for i in 0..n {
            self.cells[i] = ((i as u64 * seed.wrapping_add(7)) % 64) as u8;
            self.owners[i] = -1;
            self.tokens[i] = -1;
        }
        // Place initial tokens
        self.owners[0] = 0;
        self.tokens[0] = 0;
        self.territory[0] = 1;
        let last = n - 1;
        self.owners[last] = 1;
        self.tokens[last] = 1;
        self.territory[1] = 1;
        self.turn = 0;
        self.terminal = false;
        self.energy = [5.0; 2];
    }

    fn step(&mut self, action: &Action) -> StepResult {
        let player = self.turn % 2;
        let target = action.params.get("target").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let n = self.grid * self.grid;

        match action.kind.as_str() {
            "Claim" if self.energy[player] >= 1.0 && target < n && self.owners[target] == -1 => {
                self.energy[player] -= 1.0;
                self.owners[target] = player as i8;
                self.territory[player] += 1;
            }
            "Transform" if self.energy[player] >= 2.0 && target < n => {
                self.energy[player] -= 2.0;
                let line = action.params.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                if line < 6 { self.cells[target] ^= 1 << line; }
            }
            "Pass" => {
                self.energy[player] = (self.energy[player] + 1.0).min(10.0);
            }
            _ => {}
        }

        self.turn += 1;
        let max_turns = self.grid * self.grid * 2;
        if self.turn >= max_turns { self.terminal = true; }

        let phi = self.compute_phi();
        StepResult {
            observation: Observation {
                text: format!("HexCrucible turn={} phi={:.3}", self.turn, phi),
                legal_actions: vec![],
                hexagram: Some((phi * 63.0) as u8),
                phi: Some(phi),
            },
            reward: 0.0,
            done: self.terminal,
            info: serde_json::json!({"phi": phi, "turn": self.turn}),
        }
    }

    fn legal_actions(&self, player: u32) -> Vec<Action> {
        if self.terminal { return vec![]; }
        let mut actions = Vec::new();
        let n = self.grid * self.grid;
        actions.push(Action { kind: "Pass".into(), params: serde_json::json!({}), actor_id: player });
        if self.energy[player as usize] >= 1.0 {
            for i in 0..n {
                if self.owners[i] == -1 {
                    actions.push(Action { kind: "Claim".into(), params: serde_json::json!({"target": i}), actor_id: player });
                }
            }
        }
        if self.energy[player as usize] >= 2.0 {
            for i in 0..n {
                for line in 0..6 {
                    actions.push(Action { kind: "Transform".into(), params: serde_json::json!({"target": i, "line": line}), actor_id: player });
                }
            }
        }
        actions
    }

    fn is_terminal(&self) -> bool { self.terminal }

    fn reward(&self, player: u32) -> f64 {
        let p = player as usize;
        let opp = 1 - p;
        let territory_diff = self.territory[p] as f64 - self.territory[opp] as f64;
        let max = (self.grid * self.grid) as f64;
        (territory_diff / max).clamp(-1.0, 1.0)
    }

    fn phi(&self) -> f64 { self.compute_phi() }
    fn name(&self) -> &str { "HexCrucible" }
    fn board_hexagrams(&self) -> Vec<u8> { self.cells.clone() }
}

// ═══════════════════════════════════════════════════════════════════
// Evolution Loop
// ═══════════════════════════════════════════════════════════════════

pub struct GameEvolutionLoop {
    pub config: GameEvolutionConfig,
    pub state: GameEvolutionState,
    difficulty_adjuster: DifficultyAdjuster,
    health_history: Vec<f64>,
    phi_history: Vec<f64>,
}

impl GameEvolutionLoop {
    pub fn new(config: GameEvolutionConfig) -> Self {
        Self {
            config,
            state: GameEvolutionState::default(),
            difficulty_adjuster: DifficultyAdjuster::new(AdaptiveDifficultyConfig::default()),
            health_history: Vec::new(),
            phi_history: Vec::new(),
        }
    }

    fn create_game(&self, constellation: u8, seed: u64) -> Box<dyn AutoGame> {
        match constellation {
            0 => Box::new(AutoTicTacToe::new()),
            1 => {
                let mut g = Auto2048::new();
                g.rng = seed;
                Box::new(g)
            }
            _ => Box::new(AutoHexCrucible::new((constellation as usize + 2).min(8))),
        }
    }

    fn game_name(constellation: u8) -> &'static str {
        match constellation {
            0 => "HexTicTacToe",
            1 => "2048",
            _ => "HexCrucible",
        }
    }

    pub fn tick(&mut self) -> GameTickReport {
        self.state.is_running = true;
        let constellation = self.state.current_constellation;
        let seed = self.state.total_episodes as u64 + 1;
        let mut game = self.create_game(constellation, seed);

        let mut wins = 0usize;
        let mut losses = 0usize;
        let mut draws = 0usize;
        let mut total_reward = 0.0f64;
        let mut total_turns = 0.0f64;
        let mut total_phi = 0.0f64;
        let episodes = self.config.episodes_per_round;

        for ep in 0..episodes {
            game.reset(seed.wrapping_add(ep as u64));
            let mut steps = 0usize;
            let ep_seed = seed.wrapping_add(ep as u64);

            while !game.is_terminal() && steps < self.config.max_turns {
                let player = (steps % 2) as u32;
                let actions = game.legal_actions(player);
                if actions.is_empty() { break; }

                // Simple policy: pick action based on seed (simulating a policy network)
                let idx = ((ep_seed.wrapping_add(steps as u64)) % actions.len() as u64) as usize;
                let result = game.step(&actions[idx]);
                total_reward += result.reward;
                total_phi += result.info.get("phi").and_then(|v| v.as_f64()).unwrap_or(0.0);
                steps += 1;
            }

            total_turns += steps as f64;
            let reward = game.reward(0);
            if reward > 0.0 { wins += 1; }
            else if reward < 0.0 { losses += 1; }
            else { draws += 1; }

            self.state.total_episodes += 1;
            self.state.total_steps += steps;
            self.difficulty_adjuster.record_episode(reward > 0.0);
        }

        let win_rate = if episodes > 0 { wins as f64 / episodes as f64 } else { 0.0 };
        let avg_reward = if episodes > 0 { total_reward / episodes as f64 } else { 0.0 };
        let avg_turns = if episodes > 0 { total_turns / episodes as f64 } else { 0.0 };
        let phi_avg = if episodes > 0 { total_phi / (episodes as f64 * avg_turns.max(1.0)) } else { 0.0 };

        self.phi_history.push(phi_avg);
        if self.phi_history.len() > 100 { self.phi_history.remove(0); }
        let phi_avg_stable: f64 = if self.phi_history.is_empty() { 0.0 }
            else { self.phi_history.iter().sum::<f64>() / self.phi_history.len() as f64 };

        // Health: composite of win_rate + phi
        let health = (win_rate * 0.6 + phi_avg_stable * 0.4).clamp(0.0, 1.0);
        self.health_history.push(health);
        if self.health_history.len() > 100 { self.health_history.remove(0); }

        // Update scores
        let best = self.state.constellation_scores.entry(constellation).or_insert(0.0);
        if avg_reward > *best { *best = avg_reward; }
        let ep_count = self.state.constellation_episodes.entry(constellation).or_insert(0);
        *ep_count += episodes;

        // Check constellation advance
        let mut advanced = false;
        if self.config.auto_advance && constellation < self.config.max_constellation {
            if win_rate >= self.config.advance_threshold
                && *ep_count >= self.config.min_episodes_before_advance
            {
                self.state.current_constellation = constellation + 1;
                advanced = true;
            }
        }

        self.state.is_running = false;

        GameTickReport {
            constellation,
            game_name: Self::game_name(constellation).to_string(),
            episodes_played: episodes,
            wins, losses, draws,
            avg_reward,
            avg_turns,
            win_rate,
            phi_avg: phi_avg_stable,
            health_score: health,
            constellation_advanced: advanced,
            timestamp: timestamp(),
        }
    }

    pub fn should_advance(&self) -> bool {
        if !self.config.auto_advance { return false; }
        if self.state.current_constellation >= self.config.max_constellation { return false; }
        let ep = self.state.constellation_episodes.get(&self.state.current_constellation).copied().unwrap_or(0);
        ep >= self.config.min_episodes_before_advance
    }

    pub fn advance_constellation(&mut self) {
        if self.state.current_constellation < self.config.max_constellation {
            self.state.current_constellation += 1;
        }
    }

    pub fn state(&self) -> &GameEvolutionState {
        &self.state
    }
}

fn timestamp() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("{}-{:02}-{:02}T00:00:00Z", 1970 + (secs / 31_536_000) as u32,
        ((secs % 31_536_000) / 2_592_000) as u32 + 1,
        ((secs % 2_592_000) / 86_400) as u32 + 1)
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_creates() {
        let evo = GameEvolutionLoop::new(GameEvolutionConfig::default());
        assert_eq!(evo.state.current_constellation, 0);
        assert_eq!(evo.state.total_episodes, 0);
    }

    #[test]
    fn test_tick_runs() {
        let mut evo = GameEvolutionLoop::new(GameEvolutionConfig { episodes_per_round: 3, ..Default::default() });
        let report = evo.tick();
        assert_eq!(report.episodes_played, 3);
        assert!(report.avg_turns > 0.0);
        assert_eq!(evo.state.total_episodes, 3);
    }

    #[test]
    fn test_constellation_advance() {
        let mut evo = GameEvolutionLoop::new(GameEvolutionConfig {
            episodes_per_round: 5,
            min_episodes_before_advance: 5,
            advance_threshold: 0.0, // Always advance
            ..Default::default()
        });
        evo.tick();
        assert_eq!(evo.state.current_constellation, 1);
    }

    #[test]
    fn test_tictactoe_game() {
        let mut game = AutoTicTacToe::new();
        game.reset(42);
        assert!(!game.is_terminal());
        let actions = game.legal_actions(0);
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_2048_game() {
        let mut game = Auto2048::new();
        game.reset(42);
        assert!(!game.is_terminal());
    }

    #[test]
    fn test_hex_crucible_game() {
        let mut game = AutoHexCrucible::new(3);
        game.reset(42);
        assert!(!game.is_terminal());
        assert!(game.compute_phi() >= 0.0);
    }
}
