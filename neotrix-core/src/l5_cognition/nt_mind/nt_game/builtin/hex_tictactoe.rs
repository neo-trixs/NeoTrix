//! HexTicTacToe — Simple 3×3 E8 hexagram tic-tac-toe.
//!
//! Tutorial-level game for constellation C0.
//! Two players take turns placing tokens on a 3×3 grid.
//! Each cell has a hexagram value that contributes to the score.
//! Win by getting 3 in a row (horizontal, vertical, or diagonal).

use std::collections::HashMap;

use super::super::env::{
    CognitiveSkill, Difficulty, GameMeta, GameRegistry, NtGameEnv, RenderMode,
};
use super::super::framework::{Action, ActorId, Observation, StepResult};

// ═══════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════

const GRID: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    X,
    O,
}

#[derive(Debug, Clone)]
pub struct HexTicTacToe {
    board: [[Cell; GRID]; GRID],
    hexagrams: [[u8; GRID]; GRID],
    current_player: usize,
    player_ids: [ActorId; 2],
    turn: usize,
    is_terminal: bool,
    winner: Option<usize>,
    max_turns: usize,
}

impl HexTicTacToe {
    pub fn new(seed: u64) -> Self {
        let mut hexagrams = [[0u8; GRID]; GRID];
        for r in 0..GRID {
            for c in 0..GRID {
                hexagrams[r][c] = ((r * GRID + c) as u64 * seed.wrapping_add(7) % 64) as u8;
            }
        }

        Self {
            board: [[Cell::Empty; GRID]; GRID],
            hexagrams,
            current_player: 0,
            player_ids: [0, 1],
            turn: 0,
            is_terminal: false,
            winner: None,
            max_turns: GRID * GRID,
        }
    }

    fn check_winner(&self) -> Option<usize> {
        // Check rows
        for r in 0..GRID {
            if self.board[r][0] != Cell::Empty
                && self.board[r][0] == self.board[r][1]
                && self.board[r][1] == self.board[r][2]
            {
                return Some(match self.board[r][0] {
                    Cell::X => 0,
                    Cell::O => 1,
                    _ => unreachable!(),
                });
            }
        }
        // Check columns
        for c in 0..GRID {
            if self.board[0][c] != Cell::Empty
                && self.board[0][c] == self.board[1][c]
                && self.board[1][c] == self.board[2][c]
            {
                return Some(match self.board[0][c] {
                    Cell::X => 0,
                    Cell::O => 1,
                    _ => unreachable!(),
                });
            }
        }
        // Check diagonals
        if self.board[0][0] != Cell::Empty
            && self.board[0][0] == self.board[1][1]
            && self.board[1][1] == self.board[2][2]
        {
            return Some(match self.board[0][0] {
                Cell::X => 0,
                Cell::O => 1,
                _ => unreachable!(),
            });
        }
        if self.board[0][2] != Cell::Empty
            && self.board[0][2] == self.board[1][1]
            && self.board[1][1] == self.board[2][0]
        {
            return Some(match self.board[0][2] {
                Cell::X => 0,
                Cell::O => 1,
                _ => unreachable!(),
            });
        }
        None
    }

    fn text_state(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("=== HexTicTacToe (Turn {}) ===\n", self.turn));
        s.push_str(&format!(
            "Player: {} | Opponent: {}\n\n",
            self.player_ids[0], self.player_ids[1]
        ));

        for r in 0..GRID {
            for c in 0..GRID {
                let marker = match self.board[r][c] {
                    Cell::X => "X",
                    Cell::O => "O",
                    Cell::Empty => ".",
                };
                s.push_str(&format!("{}({}) ", marker, self.hexagrams[r][c]));
            }
            s.push('\n');
        }

        if self.is_terminal {
            match self.winner {
                Some(p) => s.push_str(&format!("\nPlayer {} wins!\n", self.player_ids[p])),
                None => s.push_str("\nDraw!\n"),
            }
        }
        s
    }
}

impl NtGameEnv for HexTicTacToe {
    fn meta(&self) -> GameMeta {
        GameMeta {
            name: "HexTicTacToe".into(),
            description: "Simple 3×3 E8 hexagram tic-tac-toe".into(),
            min_constellation: 0,
            max_constellation: 0,
            target_skills: vec![CognitiveSkill::PatternRecognition],
            is_builtin: true,
        }
    }

    fn reset(&mut self, seed: Option<u64>) -> Observation {
        let s = seed.unwrap_or(0);
        for r in 0..GRID {
            for c in 0..GRID {
                self.board[r][c] = Cell::Empty;
                self.hexagrams[r][c] = ((r * GRID + c) as u64 * s.wrapping_add(7) % 64) as u8;
            }
        }
        self.current_player = 0;
        self.turn = 0;
        self.is_terminal = false;
        self.winner = None;

        Observation {
            text: self.text_state(),
            legal_actions: self.legal_actions(),
            hexagram: None,
            phi: None,
        }
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

        let target = action
            .params
            .get("target")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let row = target / GRID;
        let col = target % GRID;

        if row >= GRID || col >= GRID || self.board[row][col] != Cell::Empty {
            return StepResult {
                observation: self.observation(),
                reward: -0.1,
                done: false,
                info: serde_json::json!({"error": "illegal move"}),
            };
        }

        self.board[row][col] = if self.current_player == 0 {
            Cell::X
        } else {
            Cell::O
        };
        self.turn += 1;

        if let Some(winner) = self.check_winner() {
            self.is_terminal = true;
            self.winner = Some(winner);
        } else if self.turn >= self.max_turns {
            self.is_terminal = true;
        } else {
            self.current_player = 1 - self.current_player;
        }

        let reward = if self.is_terminal {
            match self.winner {
                Some(w) if w == 0 => 1.0,
                Some(_) => -1.0,
                None => 0.0,
            }
        } else {
            0.0
        };

        StepResult {
            observation: self.observation(),
            reward,
            done: self.is_terminal,
            info: serde_json::json!({
                "action": "place",
                "target": target,
                "player": self.current_player,
            }),
        }
    }

    fn state(&self) -> super::super::env::GameState {
        let mut scores = HashMap::new();
        scores.insert(self.player_ids[0], 0.0);
        scores.insert(self.player_ids[1], 0.0);

        super::super::env::GameState {
            turn: self.turn,
            current_player: self.player_ids[self.current_player],
            is_terminal: self.is_terminal,
            scores,
            energy: HashMap::new(),
            hexagram_states: self.hexagrams.iter().flatten().copied().collect(),
            board_size: (GRID, GRID),
            difficulty: Difficulty::Tutorial,
            custom: serde_json::json!({"winner": self.winner}),
        }
    }

    fn legal_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.is_terminal {
            return actions;
        }
        for r in 0..GRID {
            for c in 0..GRID {
                if self.board[r][c] == Cell::Empty {
                    actions.push(Action {
                        kind: "Place".into(),
                        params: serde_json::json!({"target": r * GRID + c}),
                        actor_id: self.player_ids[self.current_player],
                    });
                }
            }
        }
        actions
    }

    fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    fn current_player(&self) -> ActorId {
        self.player_ids[self.current_player]
    }

    fn num_players(&self) -> usize {
        2
    }

    fn get_text_state(&self, _agent_id: ActorId) -> String {
        self.text_state()
    }

    fn get_game_rules(&self) -> String {
        r#"HexTicTacToe — Simple E8 Tic-Tac-Toe
3×3 grid. Take turns placing X or O.
First to get 3 in a row wins.
"#
        .into()
    }

    fn constellation_level(&self) -> u8 {
        0
    }

    fn render(&self, mode: RenderMode) -> String {
        match mode {
            RenderMode::Text | RenderMode::Visual => self.text_state(),
            RenderMode::None => String::new(),
        }
    }
}

impl HexTicTacToe {
    fn observation(&self) -> Observation {
        Observation {
            text: self.text_state(),
            legal_actions: self.legal_actions(),
            hexagram: None,
            phi: None,
        }
    }
}

/// Register HexTicTacToe into the game registry.
pub fn register_hex_tictactoe(registry: &mut GameRegistry) {
    registry.register(GameMeta {
        name: "HexTicTacToe".into(),
        description: "Simple 3×3 E8 hexagram tic-tac-toe".into(),
        min_constellation: 0,
        max_constellation: 0,
        target_skills: vec![CognitiveSkill::PatternRecognition],
        is_builtin: true,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = HexTicTacToe::new(42);
        assert_eq!(game.turn, 0);
        assert!(!game.is_terminal);
    }

    #[test]
    fn test_win_row() {
        let mut game = HexTicTacToe::new(1);
        // Player X places at 0,1,2
        game.step(&Action {
            kind: "Place".into(),
            params: serde_json::json!({"target": 0}),
            actor_id: 0,
        });
        game.step(&Action {
            kind: "Place".into(),
            params: serde_json::json!({"target": 3}),
            actor_id: 1,
        });
        game.step(&Action {
            kind: "Place".into(),
            params: serde_json::json!({"target": 1}),
            actor_id: 0,
        });
        game.step(&Action {
            kind: "Place".into(),
            params: serde_json::json!({"target": 4}),
            actor_id: 1,
        });
        game.step(&Action {
            kind: "Place".into(),
            params: serde_json::json!({"target": 2}),
            actor_id: 0,
        });
        assert!(game.is_terminal);
        assert_eq!(game.winner, Some(0));
    }

    #[test]
    fn test_draw() {
        let mut game = HexTicTacToe::new(1);
        // Fill the board without a winner
        let moves = [0, 1, 2, 4, 3, 5, 7, 6, 8]; // O plays odd positions
        for (i, &m) in moves.iter().enumerate() {
            game.step(&Action {
                kind: "Place".into(),
                params: serde_json::json!({"target": m}),
                actor_id: i as ActorId % 2,
            });
        }
        assert!(game.is_terminal);
        assert_eq!(game.winner, None);
    }

    #[test]
    fn test_legal_actions() {
        let game = HexTicTacToe::new(1);
        let actions = game.legal_actions();
        assert_eq!(actions.len(), 9); // 3x3 = 9 empty cells
    }
}
