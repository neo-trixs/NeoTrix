//! 2048 — Resource merging puzzle game.
//!
//! Classic 2048 adapted for NT-GAME with hexagram values.
//! Tiles are powers of 2, but their hexagram representation
//! provides cognitive dimension information.
//!
//! Constellation: C1 (Apprentice)

use std::collections::HashMap;

use super::super::env::{
    CognitiveSkill, Difficulty, GameMeta, GameRegistry, NtGameEnv, RenderMode,
};
use super::super::framework::{Action, ActorId, Observation, StepResult};

// ═══════════════════════════════════════════════════════════════════

const GRID: usize = 4;

#[derive(Debug, Clone)]
pub struct Game2048 {
    board: [[u32; GRID]; GRID],
    score: u32,
    best: u32,
    turn: usize,
    is_terminal: bool,
    #[allow(dead_code)]
    seed: u64,
    rng_state: u64,
}

impl Game2048 {
    pub fn new(seed: u64) -> Self {
        let mut game = Self {
            board: [[0; GRID]; GRID],
            score: 0,
            best: 0,
            turn: 0,
            is_terminal: false,
            seed,
            rng_state: seed,
        };
        game.spawn_tile();
        game.spawn_tile();
        game
    }

    fn next_random(&mut self) -> u32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        (self.rng_state >> 33) as u32
    }

    fn spawn_tile(&mut self) {
        let empty: Vec<(usize, usize)> = (0..GRID)
            .flat_map(|r| (0..GRID).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[r][c] == 0)
            .collect();
        if empty.is_empty() {
            return;
        }
        let idx = (self.next_random() as usize) % empty.len();
        let (r, c) = empty[idx];
        self.board[r][c] = if self.next_random() % 10 == 0 { 4 } else { 2 };
    }

    fn slide_row_left(row: &mut [u32; GRID]) -> u32 {
        let mut score = 0;
        // Compact
        let mut compacted: Vec<u32> = row.iter().copied().filter(|&x| x != 0).collect();
        compacted.resize(GRID, 0);
        // Merge
        for i in 0..compacted.len() - 1 {
            if compacted[i] == compacted[i + 1] && compacted[i] != 0 {
                compacted[i] *= 2;
                score += compacted[i];
                compacted[i + 1] = 0;
            }
        }
        // Compact again
        let mut result: Vec<u32> = compacted.into_iter().filter(|&x| x != 0).collect();
        result.resize(GRID, 0);
        for (i, &v) in result.iter().enumerate() {
            row[i] = v;
        }
        score
    }

    fn rotate_board(&mut self) {
        let mut new = [[0u32; GRID]; GRID];
        for r in 0..GRID {
            for c in 0..GRID {
                new[c][GRID - 1 - r] = self.board[r][c];
            }
        }
        self.board = new;
    }

    fn move_left(&mut self) -> u32 {
        let mut score = 0;
        for r in 0..GRID {
            score += Self::slide_row_left(&mut self.board[r]);
        }
        score
    }

    fn can_move(&self) -> bool {
        for r in 0..GRID {
            for c in 0..GRID {
                if self.board[r][c] == 0 {
                    return true;
                }
                if c + 1 < GRID && self.board[r][c] == self.board[r][c + 1] {
                    return true;
                }
                if r + 1 < GRID && self.board[r][c] == self.board[r + 1][c] {
                    return true;
                }
            }
        }
        false
    }

    fn text_state(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "=== 2048 (Score: {}, Best: {}) ===\n\n",
            self.score, self.best
        ));
        for r in 0..GRID {
            for c in 0..GRID {
                let v = self.board[r][c];
                if v == 0 {
                    s.push_str("    .");
                } else {
                    s.push_str(&format!("{:>5}", v));
                }
            }
            s.push('\n');
        }
        s
    }
}

impl NtGameEnv for Game2048 {
    fn meta(&self) -> GameMeta {
        GameMeta {
            name: "2048".into(),
            description: "Classic 2048 resource merging puzzle".into(),
            min_constellation: 1,
            max_constellation: 2,
            target_skills: vec![
                CognitiveSkill::Optimization,
                CognitiveSkill::PatternRecognition,
            ],
            is_builtin: true,
        }
    }

    fn reset(&mut self, seed: Option<u64>) -> Observation {
        self.rng_state = seed.unwrap_or(0);
        self.board = [[0; GRID]; GRID];
        self.score = 0;
        self.turn = 0;
        self.is_terminal = false;
        self.spawn_tile();
        self.spawn_tile();
        self.observation()
    }

    fn step(&mut self, action: &Action) -> StepResult {
        if self.is_terminal {
            return StepResult {
                observation: self.observation(),
                reward: 0.0,
                done: true,
                info: serde_json::json!({"error": "game over"}),
            };
        }

        let dir = match action.kind.as_str() {
            "Left" => 0,
            "Up" => 1,
            "Right" => 2,
            "Down" => 3,
            _ => {
                return StepResult {
                    observation: self.observation(),
                    reward: -0.1,
                    done: false,
                    info: serde_json::json!({"error": "invalid direction"}),
                };
            }
        };

        let old_board = self.board;
        let score_gain;

        // Rotate so the desired direction becomes "left", slide, rotate back
        for _ in 0..dir {
            self.rotate_board();
        }
        score_gain = self.move_left();
        for _ in 0..(4 - dir) % 4 {
            self.rotate_board();
        }

        // Check if board changed
        if self.board == old_board {
            return StepResult {
                observation: self.observation(),
                reward: 0.0,
                done: false,
                info: serde_json::json!({"error": "no change"}),
            };
        }

        self.score += score_gain;
        self.turn += 1;
        self.spawn_tile();

        if !self.can_move() {
            self.is_terminal = true;
            if self.score > self.best {
                self.best = self.score;
            }
        }

        StepResult {
            observation: self.observation(),
            reward: score_gain as f64 / 2048.0,
            done: self.is_terminal,
            info: serde_json::json!({
                "score_gain": score_gain,
                "total_score": self.score,
                "max_tile": self.board.iter().flatten().copied().max().unwrap_or(0),
            }),
        }
    }

    fn state(&self) -> super::super::env::GameState {
        let mut scores = HashMap::new();
        scores.insert(0, self.score as f64);

        super::super::env::GameState {
            turn: self.turn,
            current_player: 0,
            is_terminal: self.is_terminal,
            scores,
            energy: HashMap::new(),
            hexagram_states: self
                .board
                .iter()
                .flatten()
                .map(|&v| (v as u8).min(63))
                .collect(),
            board_size: (GRID, GRID),
            difficulty: Difficulty::Apprentice,
            custom: serde_json::json!({
                "score": self.score,
                "best": self.best,
                "max_tile": self.board.iter().flatten().copied().max().unwrap_or(0),
            }),
        }
    }

    fn legal_actions(&self) -> Vec<Action> {
        if self.is_terminal {
            return vec![];
        }
        vec![
            Action {
                kind: "Left".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
            Action {
                kind: "Up".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
            Action {
                kind: "Right".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
            Action {
                kind: "Down".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
        ]
    }

    fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    fn current_player(&self) -> ActorId {
        0
    }

    fn num_players(&self) -> usize {
        1
    }

    fn get_text_state(&self, _agent_id: ActorId) -> String {
        self.text_state()
    }

    fn get_game_rules(&self) -> String {
        r#"2048 — Resource Merging Puzzle

Slide tiles in 4 directions. Same-value tiles merge.
Goal: reach 2048 or maximize score before the board fills.
"#
        .into()
    }

    fn constellation_level(&self) -> u8 {
        1
    }

    fn render(&self, mode: RenderMode) -> String {
        match mode {
            RenderMode::Text | RenderMode::Visual => self.text_state(),
            RenderMode::None => String::new(),
        }
    }
}

impl Game2048 {
    fn observation(&self) -> Observation {
        Observation {
            text: self.text_state(),
            legal_actions: self.legal_actions(),
            hexagram: Some((self.score as u8).min(63)),
            phi: None,
        }
    }
}

/// Register 2048 into the game registry.
pub fn register_game_2048(registry: &mut GameRegistry) {
    registry.register(GameMeta {
        name: "2048".into(),
        description: "Classic 2048 resource merging puzzle".into(),
        min_constellation: 1,
        max_constellation: 2,
        target_skills: vec![
            CognitiveSkill::Optimization,
            CognitiveSkill::PatternRecognition,
        ],
        is_builtin: true,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game2048::new(42);
        assert_eq!(game.turn, 0);
        assert!(!game.is_terminal);
        let empty = game.board.iter().flatten().filter(|&&v| v == 0).count();
        assert_eq!(empty, 14); // 16 - 2 initial tiles
    }

    #[test]
    fn test_slide_row_left() {
        let mut row = [2, 2, 0, 0];
        let score = Game2048::slide_row_left(&mut row);
        assert_eq!(row, [4, 0, 0, 0]);
        assert_eq!(score, 4);
    }

    #[test]
    fn test_slide_row_merge_chain() {
        let mut row = [2, 2, 2, 2];
        let score = Game2048::slide_row_left(&mut row);
        assert_eq!(row, [4, 4, 0, 0]);
        assert_eq!(score, 8);
    }

    #[test]
    fn test_move_left() {
        let mut game = Game2048::new(1);
        game.board = [[2, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        game.move_left();
        assert_eq!(game.board[0], [2, 0, 0, 0]);
    }

    #[test]
    fn test_legal_actions() {
        let game = Game2048::new(1);
        let actions = game.legal_actions();
        assert_eq!(actions.len(), 4);
    }
}
