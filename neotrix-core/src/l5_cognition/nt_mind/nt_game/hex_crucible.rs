//! HexCrucible — Primary training game for NT-GAME.
//!
//! A two-player zero-sum strategy game on a hexagonal board where each
//! cell is an E8 hexagram domain with 6 cognitive dimension lines.
//!
//! Players compete to control territory, build resonant chains, and
//! maximize integrated information (Phi).
//!
//! ## Game Rules
//!
//! **Setup**: Hexagonal grid of `n×n` cells. Each cell has 6 bits (lines)
//! representing cognitive dimensions. Players start with tokens on opposite
//! edges.
//!
//! **Turn Structure**:
//! 1. **Observe**: Scan hexagram states, GWT competition for focus
//! 2. **Decide**: Choose Move / Claim / Transform / Pass
//! 3. **Resolve**: Apply action, check resonance, update Phi
//! 4. **Score**: Territory + Resonance + Phi + Transfer bonus
//!
//! **Actions**:
//! - **Move**: Shift token to adjacent cell (free)
//! - **Claim**: Place token on unclaimed cell (costs 1 energy)
//! - **Transform**: Flip one line of a hexagram (costs 2 energy)
//! - **Pass**: Skip turn, gain 1 energy
//!
//! **Scoring**:
//! - Territory: 1 point per claimed cell
//! - Resonance: 2 points per resonant pair (hamming ≤ 2)
//! - Phi: bonus for integrated board state
//! - Transfer: bonus for strategies that generalize

use std::collections::HashMap;

use super::env::{CognitiveSkill, Difficulty, GameMeta, GameRegistry, NtGameEnv, RenderMode};
use super::framework::{Action, Actor, ActorId, Observation, Role, StepResult, TurnIndex};

// ═══════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════

/// Maximum number of lines (bits) per hexagram cell.
pub const MAX_LINES: usize = 6;
/// Resonance threshold — two cells resonate if hamming distance ≤ threshold.
pub const RESONANCE_THRESHOLD: u32 = 2;
/// Energy cost for claiming a cell.
pub const CLAIM_COST: f64 = 1.0;
/// Energy cost for transforming a hexagram line.
pub const TRANSFORM_COST: f64 = 2.0;
/// Energy gained per pass.
pub const PASS_GAIN: f64 = 1.0;
/// Maximum energy per player.
pub const MAX_ENERGY: f64 = 10.0;
/// Starting energy.
pub const STARTING_ENERGY: f64 = 5.0;

// ═══════════════════════════════════════════════════════════════════
// Hex Cell
// ═══════════════════════════════════════════════════════════════════

/// A single hex cell in the HexCrucible board.
#[derive(Debug, Clone)]
pub struct HexCell {
    /// 6-bit hexagram state (0-63), isomorphic to E8 reasoning states.
    pub hexagram: u8,
    /// Owner: None = unclaimed, Some(player_id) = claimed.
    pub owner: Option<ActorId>,
    /// Whether a token is present on this cell.
    pub has_token: bool,
}

impl HexCell {
    pub fn new(hexagram: u8) -> Self {
        Self {
            hexagram: hexagram & 0x3F,
            owner: None,
            has_token: false,
        }
    }

    /// Hamming distance to another cell's hexagram.
    pub fn hamming_dist(&self, other: &Self) -> u32 {
        (self.hexagram ^ other.hexagram).count_ones()
    }

    /// Whether this cell resonates with another (hamming ≤ threshold).
    pub fn resonates_with(&self, other: &Self) -> bool {
        self.hamming_dist(other) <= RESONANCE_THRESHOLD
    }

    /// Flip a specific line (bit) of the hexagram.
    pub fn flip_line(&mut self, line: usize) {
        if line < MAX_LINES {
            self.hexagram ^= 1 << line;
        }
    }

    /// Get a specific line value.
    pub fn line(&self, line: usize) -> u8 {
        if line < MAX_LINES {
            (self.hexagram >> line) & 1
        } else {
            0
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// HexCrucible State
// ═══════════════════════════════════════════════════════════════════

/// Configuration for a HexCrucible game.
#[derive(Debug, Clone)]
pub struct HexCrucibleConfig {
    /// Grid size (n×n cells).
    pub grid_size: usize,
    /// Constellation level (0-5).
    pub constellation: u8,
    /// Maximum turns.
    pub max_turns: usize,
}

impl HexCrucibleConfig {
    /// Create config for a given constellation level.
    pub fn for_constellation(level: u8) -> Self {
        let grid_size = match level {
            0 => 3,
            1 => 4,
            2 => 5,
            3 => 6,
            4 => 7,
            _ => 8,
        };
        let max_turns = match level {
            0 => 20,
            1 => 30,
            2 => 40,
            3 => 50,
            4 => 60,
            _ => 80,
        };
        Self {
            grid_size,
            constellation: level.min(5),
            max_turns,
        }
    }
}

/// Internal state of HexCrucible.
#[derive(Debug, Clone)]
pub struct HexCrucibleState {
    /// The hexagonal grid (flat array, row-major).
    pub cells: Vec<HexCell>,
    /// Grid dimensions.
    pub grid_size: usize,
    /// Current turn number.
    pub turn: TurnIndex,
    /// Current player index (0 or 1).
    pub current_player: usize,
    /// Player actor IDs.
    pub player_ids: [ActorId; 2],
    /// Per-player energy.
    pub energy: [f64; 2],
    /// Per-player territory count.
    pub territory: [usize; 2],
    /// Per-player resonance count.
    pub resonance_count: [usize; 2],
    /// Total resonance pairs on the board.
    pub total_resonance_pairs: usize,
    /// Whether the game is over.
    pub is_terminal: bool,
    /// Winner: None = draw, Some(player_index) = winner.
    pub winner: Option<usize>,
    /// Maximum turns.
    pub max_turns: usize,
    /// Random seed.
    pub seed: u64,
}

// ═══════════════════════════════════════════════════════════════════
// HexCrucible Implementation
// ═══════════════════════════════════════════════════════════════════

/// HexCrucible game environment.
pub struct HexCrucible {
    pub state: HexCrucibleState,
    pub config: HexCrucibleConfig,
}

impl HexCrucible {
    /// Create a new HexCrucible game.
    pub fn new(config: HexCrucibleConfig, seed: u64) -> Self {
        let grid_size = config.grid_size;
        let total_cells = grid_size * grid_size;

        // Initialize cells with pseudo-random hexagrams
        let mut cells = Vec::with_capacity(total_cells);
        for i in 0..total_cells {
            let hex = ((i as u64).wrapping_mul(seed.wrapping_add(1)) % 64) as u8;
            cells.push(HexCell::new(hex));
        }

        let player_a = Actor::new(0, Role::Player, "Red player");
        let player_b = Actor::new(1, Role::Opponent, "Blue player");

        Self {
            state: HexCrucibleState {
                cells,
                grid_size,
                turn: 0,
                current_player: 0,
                player_ids: [player_a.id, player_b.id],
                energy: [STARTING_ENERGY; 2],
                territory: [0; 2],
                resonance_count: [0; 2],
                total_resonance_pairs: 0,
                is_terminal: false,
                winner: None,
                max_turns: config.max_turns,
                seed,
            },
            config,
        }
    }

    /// Get cell index from (row, col).
    pub fn cell_index(&self, row: usize, col: usize) -> usize {
        row * self.state.grid_size + col
    }

    /// Get (row, col) from cell index.
    pub fn cell_pos(&self, idx: usize) -> (usize, usize) {
        (idx / self.state.grid_size, idx % self.state.grid_size)
    }

    /// Get adjacent cell indices (hexagonal neighbors).
    pub fn neighbors(&self, idx: usize) -> Vec<usize> {
        let (row, col) = self.cell_pos(idx);
        let mut neighbors = Vec::new();
        let size = self.state.grid_size;

        // Hex grid: even/row offset adjacency
        let offsets = if row % 2 == 0 {
            [(-1, -1), (-1, 0), (0, -1), (0, 1), (1, -1), (1, 0)]
        } else {
            [(-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0), (1, 1)]
        };

        for (dr, dc) in offsets {
            let nr = row as isize + dr;
            let nc = col as isize + dc;
            if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                neighbors.push(self.cell_index(nr as usize, nc as usize));
            }
        }
        neighbors
    }

    /// Compute resonance pairs on the board.
    pub fn compute_resonance_pairs(&self) -> Vec<(usize, usize, u32)> {
        let mut pairs = Vec::new();
        let n = self.state.grid_size;
        for i in 0..n * n {
            let neighbors = self.neighbors(i);
            for j in neighbors {
                if j > i {
                    let dist = self.state.cells[i].hamming_dist(&self.state.cells[j]);
                    if dist <= RESONANCE_THRESHOLD {
                        pairs.push((i, j, 6 - dist));
                    }
                }
            }
        }
        pairs
    }

    /// Compute board Phi (simplified IIT measure).
    pub fn compute_phi(&self) -> f64 {
        let pairs = self.compute_resonance_pairs();
        if pairs.is_empty() {
            return 0.0;
        }
        let total_resonance: u32 = pairs.iter().map(|(_, _, strength)| *strength).sum();
        let max_possible = (self.state.grid_size * self.state.grid_size) as u32 * 6;
        total_resonance as f64 / max_possible as f64
    }

    /// Find the hex cell closest to a given actor's token.
    pub fn find_token(&self, player_idx: usize) -> Option<usize> {
        let pid = self.state.player_ids[player_idx];
        self.state
            .cells
            .iter()
            .enumerate()
            .find(|(_, c)| c.has_token && c.owner == Some(pid))
            .map(|(i, _)| i)
    }

    /// Place initial tokens for both players.
    pub fn place_initial_tokens(&mut self) {
        let n = self.state.grid_size;
        let pid_a = self.state.player_ids[0];
        let pid_b = self.state.player_ids[1];

        // Player A: top-left corner
        let idx_a = self.cell_index(0, 0);
        self.state.cells[idx_a].owner = Some(pid_a);
        self.state.cells[idx_a].has_token = true;

        // Player B: bottom-right corner
        let idx_b = self.cell_index(n - 1, n - 1);
        self.state.cells[idx_b].owner = Some(pid_b);
        self.state.cells[idx_b].has_token = true;

        self.state.territory[0] = 1;
        self.state.territory[1] = 1;
    }

    /// Try to move a token to an adjacent cell.
    pub fn try_move(&mut self, player_idx: usize, target: usize) -> Result<StepResult, String> {
        if self.state.is_terminal {
            return Err("Game is over".into());
        }
        if player_idx != self.state.current_player {
            return Err("Not your turn".into());
        }

        let token_pos = self.find_token(player_idx).ok_or("No token found")?;

        let neighbors = self.neighbors(token_pos);
        if !neighbors.contains(&target) {
            return Err("Target is not adjacent".into());
        }

        let pid = self.state.player_ids[player_idx];

        // Move token
        self.state.cells[token_pos].has_token = false;
        self.state.cells[target].has_token = true;
        self.state.cells[target].owner = Some(pid);

        // If unclaimed, claim it
        if self.state.cells[target].owner == Some(pid) && self.state.territory[player_idx] == 0
            || self.state.cells[target].owner.is_none()
        {
            self.state.cells[target].owner = Some(pid);
            self.state.territory[player_idx] += 1;
        }

        self.advance_turn();

        let phi = self.compute_phi();
        let pairs = self.compute_resonance_pairs();
        self.state.total_resonance_pairs = pairs.len();

        Ok(StepResult {
            observation: self.make_observation(),
            reward: 0.0,
            done: self.state.is_terminal,
            info: serde_json::json!({
                "action": "move",
                "from": token_pos,
                "to": target,
                "phi": phi,
                "resonance_pairs": pairs.len(),
            }),
        })
    }

    /// Try to claim an unclaimed cell.
    pub fn try_claim(&mut self, player_idx: usize, target: usize) -> Result<StepResult, String> {
        if self.state.is_terminal {
            return Err("Game is over".into());
        }
        if player_idx != self.state.current_player {
            return Err("Not your turn".into());
        }
        if self.state.energy[player_idx] < CLAIM_COST {
            return Err("Not enough energy".into());
        }

        let cell = &self.state.cells[target];
        if cell.owner.is_some() {
            return Err("Cell already claimed".into());
        }

        self.state.energy[player_idx] -= CLAIM_COST;
        let pid = self.state.player_ids[player_idx];
        self.state.cells[target].owner = Some(pid);
        self.state.territory[player_idx] += 1;

        self.advance_turn();

        let phi = self.compute_phi();
        Ok(StepResult {
            observation: self.make_observation(),
            reward: 0.0,
            done: self.state.is_terminal,
            info: serde_json::json!({
                "action": "claim",
                "target": target,
                "phi": phi,
                "remaining_energy": self.state.energy[player_idx],
            }),
        })
    }

    /// Try to transform (flip a line of) a hexagram.
    pub fn try_transform(
        &mut self,
        player_idx: usize,
        target: usize,
        line: usize,
    ) -> Result<StepResult, String> {
        if self.state.is_terminal {
            return Err("Game is over".into());
        }
        if player_idx != self.state.current_player {
            return Err("Not your turn".into());
        }
        if self.state.energy[player_idx] < TRANSFORM_COST {
            return Err("Not enough energy".into());
        }
        if line >= MAX_LINES {
            return Err("Invalid line".into());
        }

        self.state.energy[player_idx] -= TRANSFORM_COST;
        self.state.cells[target].flip_line(line);

        // Check for new resonance pairs
        let old_pairs = self.state.total_resonance_pairs;
        let new_pairs = self.compute_resonance_pairs().len();
        let bonus = if new_pairs > old_pairs {
            (new_pairs - old_pairs) as f64 * 0.5
        } else {
            0.0
        };

        self.state.total_resonance_pairs = new_pairs;
        self.advance_turn();

        let phi = self.compute_phi();
        Ok(StepResult {
            observation: self.make_observation(),
            reward: bonus,
            done: self.state.is_terminal,
            info: serde_json::json!({
                "action": "transform",
                "target": target,
                "line": line,
                "new_resonance_pairs": new_pairs - old_pairs,
                "phi": phi,
            }),
        })
    }

    /// Pass the turn.
    pub fn try_pass(&mut self, player_idx: usize) -> Result<StepResult, String> {
        if self.state.is_terminal {
            return Err("Game is over".into());
        }
        if player_idx != self.state.current_player {
            return Err("Not your turn".into());
        }

        self.state.energy[player_idx] = (self.state.energy[player_idx] + PASS_GAIN).min(MAX_ENERGY);
        self.advance_turn();

        Ok(StepResult {
            observation: self.make_observation(),
            reward: 0.0,
            done: self.state.is_terminal,
            info: serde_json::json!({
                "action": "pass",
                "remaining_energy": self.state.energy[player_idx],
            }),
        })
    }

    /// Advance to the next turn.
    fn advance_turn(&mut self) {
        self.state.turn += 1;
        self.state.current_player = 1 - self.state.current_player;

        // Check termination
        if self.state.turn >= self.state.max_turns {
            self.state.is_terminal = true;
            self.determine_winner();
        }
    }

    /// Determine the winner based on scores.
    fn determine_winner(&mut self) {
        let score_a = self.compute_score(0);
        let score_b = self.compute_score(1);

        if score_a > score_b {
            self.state.winner = Some(0);
        } else if score_b > score_a {
            self.state.winner = Some(1);
        } else {
            self.state.winner = None; // Draw
        }
    }

    /// Compute the score for a player.
    pub fn compute_score(&self, player_idx: usize) -> f64 {
        let territory = self.state.territory[player_idx] as f64;
        let resonance = self.state.resonance_count[player_idx] as f64 * 2.0;
        let phi = self.compute_phi() * 10.0;
        territory + resonance + phi
    }

    /// Make an observation from the current state.
    fn make_observation(&self) -> Observation {
        let text = self.text_state(self.state.player_ids[self.state.current_player]);
        let legal = self.generate_legal_actions(self.state.current_player);
        let hex = self.to_hexagram();
        let phi = Some(self.compute_phi());

        Observation {
            text,
            legal_actions: legal,
            hexagram: hex,
            phi,
        }
    }

    /// Generate legal actions for a player.
    fn generate_legal_actions(&self, player_idx: usize) -> Vec<Action> {
        let mut actions = Vec::new();
        let pid = self.state.player_ids[player_idx];

        // Can always pass
        actions.push(Action {
            kind: "Pass".into(),
            params: serde_json::json!({}),
            actor_id: pid,
        });

        // Can claim unclaimed cells if enough energy
        if self.state.energy[player_idx] >= CLAIM_COST {
            for (i, cell) in self.state.cells.iter().enumerate() {
                if cell.owner.is_none() {
                    actions.push(Action {
                        kind: "Claim".into(),
                        params: serde_json::json!({"target": i}),
                        actor_id: pid,
                    });
                }
            }
        }

        // Can move token to adjacent cells
        if let Some(token_pos) = self.find_token(player_idx) {
            for neighbor in self.neighbors(token_pos) {
                actions.push(Action {
                    kind: "Move".into(),
                    params: serde_json::json!({"target": neighbor}),
                    actor_id: pid,
                });
            }
        }

        // Can transform if enough energy
        if self.state.energy[player_idx] >= TRANSFORM_COST {
            for i in 0..self.state.cells.len() {
                for line in 0..MAX_LINES {
                    actions.push(Action {
                        kind: "Transform".into(),
                        params: serde_json::json!({"target": i, "line": line}),
                        actor_id: pid,
                    });
                }
            }
        }

        actions
    }

    /// Text representation of the game state.
    fn text_state(&self, agent_id: ActorId) -> String {
        let mut s = String::new();
        let n = self.state.grid_size;
        let pid = if agent_id == self.state.player_ids[0] {
            0
        } else {
            1
        };

        s.push_str(&format!(
            "=== HexCrucible (Turn {}/{}) ===\n",
            self.state.turn, self.state.max_turns
        ));
        s.push_str(&format!(
            "You: {} | Energy: {:.0}\n",
            agent_id, self.state.energy[pid]
        ));
        s.push_str(&format!(
            "Territory: {} | Resonance pairs: {}\n",
            self.state.territory[0], self.state.total_resonance_pairs
        ));
        s.push_str(&format!("Phi: {:.3}\n\n", self.compute_phi()));

        // Board
        for row in 0..n {
            let indent = if row % 2 == 1 { " " } else { "" };
            s.push_str(indent);
            for col in 0..n {
                let idx = self.cell_index(row, col);
                let cell = &self.state.cells[idx];
                let marker = match cell.owner {
                    Some(id) if id == self.state.player_ids[0] => "R",
                    Some(_) => "B",
                    None => ".",
                };
                let token = if cell.has_token { "*" } else { " " };
                s.push_str(&format!("{}{}{} ", marker, cell.hexagram, token));
            }
            s.push('\n');
        }

        s
    }
}

// ═══════════════════════════════════════════════════════════════════
// NtGameEnv Implementation
// ═══════════════════════════════════════════════════════════════════

impl NtGameEnv for HexCrucible {
    fn meta(&self) -> GameMeta {
        GameMeta {
            name: "HexCrucible".into(),
            description:
                "E8 hexagram strategy game — two-player territory control with resonance mechanics"
                    .into(),
            min_constellation: 0,
            max_constellation: 5,
            target_skills: vec![
                CognitiveSkill::Planning,
                CognitiveSkill::PatternRecognition,
                CognitiveSkill::Optimization,
            ],
            is_builtin: true,
        }
    }

    fn reset(&mut self, seed: Option<u64>) -> Observation {
        if let Some(s) = seed {
            self.state.seed = s;
        }
        self.state.turn = 0;
        self.state.current_player = 0;
        self.state.energy = [STARTING_ENERGY; 2];
        self.state.territory = [0; 2];
        self.state.resonance_count = [0; 2];
        self.state.total_resonance_pairs = 0;
        self.state.is_terminal = false;
        self.state.winner = None;

        // Re-initialize cells
        for (i, cell) in self.state.cells.iter_mut().enumerate() {
            cell.hexagram = ((i as u64).wrapping_mul(self.state.seed.wrapping_add(1)) % 64) as u8;
            cell.owner = None;
            cell.has_token = false;
        }

        self.place_initial_tokens();
        self.make_observation()
    }

    fn step(&mut self, action: &Action) -> StepResult {
        let pid = self.state.current_player;
        match action.kind.as_str() {
            "Move" => {
                let target = action
                    .params
                    .get("target")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                self.try_move(pid, target).unwrap_or_else(|e| StepResult {
                    observation: self.make_observation(),
                    reward: -0.1, // Small penalty for illegal action
                    done: false,
                    info: serde_json::json!({"error": e}),
                })
            }
            "Claim" => {
                let target = action
                    .params
                    .get("target")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                self.try_claim(pid, target).unwrap_or_else(|e| StepResult {
                    observation: self.make_observation(),
                    reward: -0.1,
                    done: false,
                    info: serde_json::json!({"error": e}),
                })
            }
            "Transform" => {
                let target = action
                    .params
                    .get("target")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let line = action
                    .params
                    .get("line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                self.try_transform(pid, target, line)
                    .unwrap_or_else(|e| StepResult {
                        observation: self.make_observation(),
                        reward: -0.1,
                        done: false,
                        info: serde_json::json!({"error": e}),
                    })
            }
            "Pass" | _ => self.try_pass(pid).unwrap_or_else(|e| StepResult {
                observation: self.make_observation(),
                reward: -0.1,
                done: false,
                info: serde_json::json!({"error": e}),
            }),
        }
    }

    fn state(&self) -> super::env::GameState {
        let mut scores = HashMap::new();
        scores.insert(self.state.player_ids[0], self.compute_score(0));
        scores.insert(self.state.player_ids[1], self.compute_score(1));

        let mut energy = HashMap::new();
        energy.insert(self.state.player_ids[0], self.state.energy[0]);
        energy.insert(self.state.player_ids[1], self.state.energy[1]);

        super::env::GameState {
            turn: self.state.turn,
            current_player: self.state.player_ids[self.state.current_player],
            is_terminal: self.state.is_terminal,
            scores,
            energy,
            hexagram_states: self.state.cells.iter().map(|c| c.hexagram).collect(),
            board_size: (self.state.grid_size, self.state.grid_size),
            difficulty: Difficulty::from_constellation(self.config.constellation),
            custom: serde_json::json!({
                "territory": self.state.territory,
                "resonance_count": self.state.resonance_count,
                "total_resonance_pairs": self.state.total_resonance_pairs,
                "winner": self.state.winner,
            }),
        }
    }

    fn legal_actions(&self) -> Vec<Action> {
        self.generate_legal_actions(self.state.current_player)
    }

    fn is_terminal(&self) -> bool {
        self.state.is_terminal
    }

    fn current_player(&self) -> ActorId {
        self.state.player_ids[self.state.current_player]
    }

    fn num_players(&self) -> usize {
        2
    }

    fn get_text_state(&self, agent_id: ActorId) -> String {
        self.text_state(agent_id)
    }

    fn get_game_rules(&self) -> String {
        r#"HexCrucible — E8 Strategy Game

Rules:
- Two players take turns on a hexagonal grid
- Each cell has a 6-bit hexagram (0-63)
- Actions: Move (free), Claim (1 energy), Transform (2 energy), Pass (gain 1 energy)

Scoring:
- Territory: 1 point per claimed cell
- Resonance: 2 points per pair of cells with hamming distance ≤ 2
- Phi: bonus for integrated board state (0-10)

Winning:
- Highest score after max_turns wins
"#
        .into()
    }

    fn to_hexagram(&self) -> Option<u8> {
        // Map the board's average hexagram state
        let avg: u32 = self.state.cells.iter().map(|c| c.hexagram as u32).sum();
        let count = self.state.cells.len() as u32;
        Some((avg / count) as u8)
    }

    fn phi_contribution(&self) -> f64 {
        self.compute_phi()
    }

    fn constellation_level(&self) -> u8 {
        self.config.constellation
    }

    fn render(&self, mode: RenderMode) -> String {
        match mode {
            RenderMode::None => String::new(),
            RenderMode::Text => self.text_state(self.state.player_ids[self.state.current_player]),
            RenderMode::Visual => {
                // For visual mode, return JSON data for the frontend
                serde_json::json!({
                    "grid_size": self.state.grid_size,
                    "cells": self.state.cells.iter().map(|c| serde_json::json!({
                        "hexagram": c.hexagram,
                        "owner": c.owner,
                        "has_token": c.has_token,
                    })).collect::<Vec<_>>(),
                    "turn": self.state.turn,
                    "max_turns": self.state.max_turns,
                    "energy": self.state.energy,
                    "territory": self.state.territory,
                    "phi": self.compute_phi(),
                })
                .to_string()
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Registry Helper
// ═══════════════════════════════════════════════════════════════════

/// Register HexCrucible into the game registry.
pub fn register_hex_crucible(registry: &mut GameRegistry) {
    registry.register(GameMeta {
        name: "HexCrucible".into(),
        description:
            "E8 hexagram strategy game — two-player territory control with resonance mechanics"
                .into(),
        min_constellation: 0,
        max_constellation: 5,
        target_skills: vec![
            CognitiveSkill::Planning,
            CognitiveSkill::PatternRecognition,
            CognitiveSkill::Optimization,
        ],
        is_builtin: true,
    });
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> HexCrucible {
        let config = HexCrucibleConfig::for_constellation(0);
        let mut game = HexCrucible::new(config, 42);
        game.place_initial_tokens();
        game
    }

    #[test]
    fn test_hex_cell_hamming() {
        let a = HexCell::new(0b000000);
        let b = HexCell::new(0b000011);
        assert_eq!(a.hamming_dist(&b), 2);
        assert!(a.resonates_with(&b));

        let c = HexCell::new(0b111111);
        assert_eq!(a.hamming_dist(&c), 6);
        assert!(!a.resonates_with(&c));
    }

    #[test]
    fn test_hex_cell_flip_line() {
        let mut cell = HexCell::new(0b000000);
        cell.flip_line(0);
        assert_eq!(cell.hexagram, 0b000001);
        cell.flip_line(0);
        assert_eq!(cell.hexagram, 0b000000);
    }

    #[test]
    fn test_game_creation() {
        let game = make_game();
        assert_eq!(game.state.grid_size, 3);
        assert_eq!(game.state.turn, 0);
        assert_eq!(game.state.energy, [STARTING_ENERGY; 2]);
        assert!(!game.state.is_terminal);
    }

    #[test]
    fn test_initial_tokens() {
        let game = make_game();
        let token_a = game.find_token(0);
        let token_b = game.find_token(1);
        assert!(token_a.is_some());
        assert!(token_b.is_some());
        assert_eq!(game.state.territory, [1, 1]);
    }

    #[test]
    fn test_neighbors() {
        let game = make_game();
        // Corner cell (0,0) should have fewer neighbors
        let neighbors_00 = game.neighbors(0);
        assert!(neighbors_00.len() >= 2);
        assert!(neighbors_00.len() <= 4);

        // Center cell should have more neighbors
        let center = game.cell_index(1, 1);
        let neighbors_center = game.neighbors(center);
        assert_eq!(neighbors_center.len(), 6);
    }

    #[test]
    fn test_move_action() {
        let mut game = make_game();
        let token_a = game.find_token(0).unwrap();
        let neighbors = game.neighbors(token_a);
        let target = neighbors[0];

        let result = game.try_move(0, target);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(game.state.turn, 1);
        assert_eq!(game.state.current_player, 1);
        assert!(!result.done);
    }

    #[test]
    fn test_claim_action() {
        let mut game = make_game();
        // Find an unclaimed cell
        let target = game
            .state
            .cells
            .iter()
            .position(|c| c.owner.is_none())
            .unwrap();
        let result = game.try_claim(0, target);
        assert!(result.is_ok());
        assert!(game.state.energy[0] < STARTING_ENERGY);
    }

    #[test]
    fn test_transform_action() {
        let mut game = make_game();
        let result = game.try_transform(0, 0, 3);
        assert!(result.is_ok());
        assert!(game.state.energy[0] < STARTING_ENERGY);
    }

    #[test]
    fn test_pass_action() {
        let mut game = make_game();
        let energy_before = game.state.energy[0];
        let result = game.try_pass(0);
        assert!(result.is_ok());
        assert!(game.state.energy[0] > energy_before);
    }

    #[test]
    fn test_legal_actions() {
        let game = make_game();
        let actions = game.legal_actions();
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.kind == "Pass"));
    }

    #[test]
    fn test_game_rules() {
        let game = make_game();
        let rules = game.get_game_rules();
        assert!(rules.contains("HexCrucible"));
        assert!(rules.contains("Move"));
    }

    #[test]
    fn test_text_state() {
        let game = make_game();
        let text = game.get_text_state(0);
        assert!(text.contains("HexCrucible"));
        assert!(text.contains("Turn"));
    }

    #[test]
    fn test_phi_computation() {
        let game = make_game();
        let phi = game.compute_phi();
        assert!(phi >= 0.0);
        assert!(phi <= 1.0);
    }

    #[test]
    fn test_score_computation() {
        let game = make_game();
        let score_a = game.compute_score(0);
        let score_b = game.compute_score(1);
        // Both should have some base score from territory
        assert!(score_a > 0.0);
        assert!(score_b > 0.0);
    }
}
