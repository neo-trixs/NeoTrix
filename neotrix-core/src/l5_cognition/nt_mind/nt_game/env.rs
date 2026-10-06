//! NtGameEnv trait — the core interface for all game environments.
//!
//! Every game implements this trait. Inspired by Gymnasium's Env interface
//! (from LLM-TeamGym + SIMPLE) plus NeoTrix-specific extensions for
//! consciousness mapping and E8 reasoning.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::framework::{Action, ActorId, Observation, StepResult, TurnIndex};

// ═══════════════════════════════════════════════════════════════════
// Game Types
// ═══════════════════════════════════════════════════════════════════

/// Rendering mode for game visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// No rendering (training).
    None,
    /// Text-based rendering (CLI / logs).
    Text,
    /// Visual rendering (Canvas / Tauri panel).
    Visual,
}

/// Difficulty tier that maps to constellation levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Tutorial = 0,
    Apprentice = 1,
    Journeyman = 2,
    Expert = 3,
    Master = 4,
    Transcendent = 5,
}

impl Difficulty {
    pub fn from_constellation(level: u8) -> Self {
        match level {
            0 => Self::Tutorial,
            1 => Self::Apprentice,
            2 => Self::Journeyman,
            3 => Self::Expert,
            4 => Self::Master,
            _ => Self::Transcendent,
        }
    }

    pub fn constellation(&self) -> u8 {
        *self as u8
    }
}

/// Cognitive skills that a game exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CognitiveSkill {
    /// Strategic planning / forward reasoning.
    Planning,
    /// Resource management / optimization.
    Optimization,
    /// Pattern recognition / hexagram matching.
    PatternRecognition,
    /// Emotional regulation under pressure.
    EmotionRegulation,
    /// Attention allocation / focus.
    AttentionAllocation,
    /// Multi-step reasoning / chains.
    MultiStepReasoning,
    /// Cooperation / negotiation.
    Cooperation,
    /// Adversarial thinking / opponent modeling.
    AdversarialThinking,
    /// Spatial reasoning.
    SpatialReasoning,
    /// Code understanding / debugging.
    CodeReasoning,
}

/// 终局结果。
///
/// ⭐ **本类型是为补 `NtGameEnv` 的合约缺口而引入的**（见 `outcome()`）。
/// 三个内置游戏**内部早就能算出胜负**（`HexTicTacToe.winner`、
/// `HexCrucibleState.winner`、`HexCrucible::determine_winner()`），
/// 但合约层没有任何出口 ⇒ 上层自对弈循环算不出胜率，`win_rate` 恒 0 ⇒
/// 难度自适应与 constellation 进阶在活路径上是**死逻辑**。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameOutcome {
    /// 获胜方玩家下标（0 起）。
    Win(usize),
    /// 终局但无胜者：和棋，或「记分制」游戏正常结束（如 2048 走不动了）。
    Draw,
}

/// Metadata about a game.
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct GameMeta {
    pub name: String,
    pub description: String,
    pub min_constellation: u8,
    pub max_constellation: u8,
    pub target_skills: Vec<CognitiveSkill>,
    /// Whether the game is built-in (Rust-native) or external.
    pub is_builtin: bool,
}

/// Action type enumeration — what players can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    Move,
    Claim,
    Transform,
    Pass,
    Attack,
    Defend,
    Trade,
    Build,
    Custom(u8),
}

/// The game state — a snapshot of the entire game at one moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current turn number (0-indexed).
    pub turn: TurnIndex,
    /// Current player's actor ID.
    pub current_player: ActorId,
    /// Whether the game is over.
    pub is_terminal: bool,
    /// Per-player scores.
    pub scores: HashMap<ActorId, f64>,
    /// Per-player energy/resources.
    pub energy: HashMap<ActorId, f64>,
    /// Hexagram states (if applicable, indexed by cell position).
    pub hexagram_states: Vec<u8>,
    /// Board dimensions (width, height).
    pub board_size: (usize, usize),
    /// Current difficulty.
    pub difficulty: Difficulty,
    /// Additional game-specific state as JSON.
    pub custom: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════
// NtGameEnv Trait
// ═══════════════════════════════════════════════════════════════════

/// Core trait for all NeoTrix game environments.
///
/// Every game implements this interface. It combines:
/// - Gymnasium-style reset/step (from SIMPLE, LLM-TeamGym)
/// - Text-native observations (from LLM-TeamGym)
/// - NeoTrix-specific consciousness mapping
pub trait NtGameEnv: Send + Sync {
    /// Game metadata.
    fn meta(&self) -> GameMeta;

    /// Reset the environment and return initial observation.
    fn reset(&mut self, seed: Option<u64>) -> Observation;

    /// Execute an action and return the result.
    fn step(&mut self, action: &Action) -> StepResult;

    /// Get the current game state.
    fn state(&self) -> GameState;

    /// Get all legal actions for the current player.
    fn legal_actions(&self) -> Vec<Action>;

    /// Check if the game is over.
    fn is_terminal(&self) -> bool;

    /// Get the current player's actor ID.
    fn current_player(&self) -> ActorId;

    /// Get the number of players.
    fn num_players(&self) -> usize;

    // ─── Text-Native Interface (LLM-TeamGym) ───

    /// Get the text representation of the game state for a specific agent.
    fn get_text_state(&self, agent_id: ActorId) -> String;

    /// Get the game rules as text (for LLM system prompts).
    fn get_game_rules(&self) -> String;

    // ─── NeoTrix Extensions ───

    /// Map the current game state to an E8 hexagram (0-63).
    /// Returns None if the state doesn't map cleanly.
    fn to_hexagram(&self) -> Option<u8> {
        None
    }

    /// 该游戏是否定义了 phi 语义。
    ///
    /// ⭐ 改为 `Option<f64>`（原为 `-> f64` 默认 `0.0`）是**一次实测驱动的裁决**：
    /// `HexTicTacToe` / `Game2048` 都没覆写它，吃默认值 `0.0`，于是上层无法区分
    /// 「这个游戏的 phi 真的是 0」与「这个游戏根本没定义 phi」。
    /// 前者是真信息，后者是**缺口** —— 用 `0.0` 冒充会把缺口洗成数据
    /// （实测 `phi_avg` 恒 0.0000 却看不出原因）。`None` = 未定义。
    fn phi_contribution(&self) -> Option<f64> {
        None
    }

    /// 终局结果；`None` = 未终局。
    ///
    /// ⛔ 此前**本方法不存在**，这是三套游戏实现长期无法收敛到 `NtGameEnv` 的
    ///   真正原因：`nt_game/evolution.rs` 里的 `trait AutoGame` 自带一个
    ///   `reward(player)`，正是用来补这个洞的 —— 所谓「530 行冗余副本」
    ///   其实是**给缺失合约打的补丁**。删除副本而不补合约，只会把补丁连同
    ///   它掩盖的能力一起删掉。
    fn outcome(&self) -> Option<GameOutcome> {
        None
    }

    /// 本局自己要求的回合预算；`None` = 由游戏自行终局，外层无需代管。
    ///
    /// ⭐ 补这个出口是为了修一个**实测到的静默截断**：
    /// `GameEvolutionConfig::max_turns`（默认 50）是外层 `while steps < …` 的硬帽，
    /// 而 `HexCrucibleConfig::for_constellation` 给的预算是 20/30/40/50/60/80。
    /// 两者都是「回合上限」这一件事的**双重权威**且互相矛盾 ⇒ constellation 4/5
    /// （需 60/80）被外层 50 砍断，**永远走不到终局**，实测 30/30 episode 全判 draw、
    /// `win_rate ≡ 0`、`turns` 恒等于 50。
    /// ⇒ 有效预算改为 `max(外层安全帽, 本局预算)`：安全帽只在游戏**不自己管**时才生效。
    fn turn_budget(&self) -> Option<usize> {
        None
    }

    /// Get the constellation level required for this game.
    fn constellation_level(&self) -> u8 {
        self.meta().min_constellation
    }

    /// Get the current difficulty.
    fn difficulty(&self) -> Difficulty {
        self.state().difficulty
    }

    /// Render the game (text or visual).
    fn render(&self, _mode: RenderMode) -> String {
        String::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Game Registry
// ═══════════════════════════════════════════════════════════════════

/// Registry of available game environments.
pub struct GameRegistry {
    games: Vec<GameMeta>,
}

impl GameRegistry {
    pub fn new() -> Self {
        Self { games: Vec::new() }
    }

    pub fn register(&mut self, meta: GameMeta) {
        self.games.push(meta);
    }

    pub fn list(&self) -> &[GameMeta] {
        &self.games
    }

    pub fn find(&self, name: &str) -> Option<&GameMeta> {
        self.games.iter().find(|g| g.name == name)
    }

    pub fn for_constellation(&self, level: u8) -> Vec<&GameMeta> {
        self.games
            .iter()
            .filter(|g| level >= g.min_constellation && level <= g.max_constellation)
            .collect()
    }

    pub fn for_skill(&self, skill: &CognitiveSkill) -> Vec<&GameMeta> {
        self.games
            .iter()
            .filter(|g| g.target_skills.contains(skill))
            .collect()
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_constellation() {
        assert_eq!(Difficulty::from_constellation(0), Difficulty::Tutorial);
        assert_eq!(Difficulty::from_constellation(3), Difficulty::Expert);
        assert_eq!(Difficulty::from_constellation(99), Difficulty::Transcendent);
    }

    #[test]
    fn test_game_registry() {
        let mut reg = GameRegistry::new();
        reg.register(GameMeta {
            name: "HexCrucible".into(),
            description: "E8 strategy".into(),
            min_constellation: 0,
            max_constellation: 5,
            target_skills: vec![CognitiveSkill::Planning, CognitiveSkill::PatternRecognition],
            is_builtin: true,
        });
        assert_eq!(reg.list().len(), 1);
        assert!(reg.find("HexCrucible").is_some());
        assert!(reg.find("Nonexistent").is_none());
        let for_plan = reg.for_skill(&CognitiveSkill::Planning);
        assert_eq!(for_plan.len(), 1);
    }
}
