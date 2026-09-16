//! MCP Tools for NT-GAME — exposes game operations as JSON-RPC tools.
//!
//! Tools:
//! - `game_list` — list available games with constellation/skill info
//! - `game_create` — create game session with config
//! - `game_step` — execute one game action
//! - `game_observe` — get current game state (text)
//! - `game_legal_actions` — get available actions
//! - `game_train` — start training loop
//! - `game_metrics` — get training metrics

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::builtin::game_2048::Game2048;
use super::builtin::hex_tictactoe::HexTicTacToe;
use super::env::NtGameEnv;
use super::framework::{Action, StepResult};
use super::hex_crucible::{HexCrucible, HexCrucibleConfig};

// ═══════════════════════════════════════════════════════════════════
// GameSession — holds mutable state across MCP calls
// ═══════════════════════════════════════════════════════════════════

/// A live game session that MCP tools operate on.
pub struct GameSession {
    pub game_name: String,
    pub turn: usize,
    pub state: Value,
    env: Box<dyn NtGameEnv>,
    history: Vec<StepResult>,
}

impl GameSession {
    pub fn new(game_name: String, env: Box<dyn NtGameEnv>, seed: u64) -> Self {
        let mut s = Self {
            game_name,
            turn: 0,
            state: Value::Null,
            env,
            history: Vec::new(),
        };
        let obs = s.env.reset(Some(seed));
        s.state = serde_json::json!({
            "turn": s.turn,
            "observation": obs.text,
            "is_terminal": s.env.is_terminal(),
            "current_player": s.env.current_player(),
            "hexagram": obs.hexagram,
            "phi": obs.phi,
        });
        s
    }

    pub fn step(&mut self, action: &Action) -> StepResult {
        let result = self.env.step(action);
        self.turn += 1;
        self.history.push(result.clone());
        self.state = serde_json::json!({
            "turn": self.turn,
            "observation": result.observation.text,
            "is_terminal": result.done,
            "current_player": self.env.current_player(),
            "reward": result.reward,
            "hexagram": result.observation.hexagram,
            "phi": result.observation.phi,
        });
        result
    }

    pub fn observe(&self) -> String {
        self.env.get_text_state(0)
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        self.env.legal_actions()
    }

    pub fn is_terminal(&self) -> bool {
        self.env.is_terminal()
    }

    pub fn current_player(&self) -> u32 {
        self.env.current_player()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Training Metrics
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpTrainingMetrics {
    pub episodes_completed: usize,
    pub total_steps: usize,
    pub avg_score: f64,
    pub best_score: f64,
    pub avg_reward: f64,
    pub avg_turns: f64,
}

// ═══════════════════════════════════════════════════════════════════
// GameTool — single tool trait
// ═══════════════════════════════════════════════════════════════════

/// Interface for a single game MCP tool.
pub trait GameTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Value) -> Result<Value, String>;
}

// ═══════════════════════════════════════════════════════════════════
// Concrete Tools
// ═══════════════════════════════════════════════════════════════════

// ─── game_list ───

struct GameListTool;

impl GameTool for GameListTool {
    fn name(&self) -> &str {
        "game_list"
    }

    fn description(&self) -> &str {
        "List available games with constellation and skill info"
    }

    fn execute(&self, _args: Value) -> Result<Value, String> {
        let reg = super::default_registry();
        let games: Vec<Value> = reg
            .list()
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "constellation": {
                        "min": m.min_constellation,
                        "max": m.max_constellation,
                    },
                    "skills": m.target_skills,
                })
            })
            .collect();
        Ok(Value::Array(games))
    }
}

// ─── game_create ───

struct GameCreateTool;

impl GameTool for GameCreateTool {
    fn name(&self) -> &str {
        "game_create"
    }

    fn description(&self) -> &str {
        "Create a new game session and return initial state"
    }

    fn execute(&self, args: Value) -> Result<Value, String> {
        let game_name = args["game_name"]
            .as_str()
            .ok_or("missing game_name")?
            .to_string();
        let seed = args["seed"].as_u64().unwrap_or(42);

        let env: Box<dyn NtGameEnv> = match game_name.as_str() {
            "HexCrucible" => {
                let cfg = HexCrucibleConfig::for_constellation(0);
                Box::new(HexCrucible::new(cfg, seed))
            }
            "HexTicTacToe" => Box::new(HexTicTacToe::new(seed)),
            "2048" => Box::new(Game2048::new(seed)),
            other => {
                return Err(format!(
                    "Unknown game: {}. Available: HexCrucible, HexTicTacToe, 2048",
                    other
                ));
            }
        };

        let session = GameSession::new(game_name.clone(), env, seed);
        let obs_text = session.observe();
        let legal: Vec<Value> = session
            .legal_actions()
            .iter()
            .map(|a| serde_json::json!({"kind": a.kind, "params": a.params}))
            .collect();

        Ok(serde_json::json!({
            "game_name": game_name,
            "turn": session.turn,
            "observation": obs_text,
            "legal_actions": legal,
            "is_terminal": session.is_terminal(),
            "state": session.state,
        }))
    }
}

// ─── game_step ───

struct GameStepTool;

impl GameTool for GameStepTool {
    fn name(&self) -> &str {
        "game_step"
    }

    fn description(&self) -> &str {
        "Execute one game action and return the result"
    }

    fn execute(&self, args: Value) -> Result<Value, String> {
        let action_kind = args["action_kind"]
            .as_str()
            .ok_or("missing action_kind")?
            .to_string();
        let params = args.get("params").cloned().unwrap_or(serde_json::json!({}));

        // The session is accessed via GameSessionManager; this tool
        // receives the session reference externally. For standalone use,
        // we reconstruct a minimal result.
        let action = Action {
            kind: action_kind,
            params,
            actor_id: 0,
        };

        Ok(serde_json::json!({
            "action": {"kind": action.kind, "params": action.params},
            "message": "action prepared — use GameSessionManager.step() to apply",
        }))
    }
}

// ─── game_observe ───

struct GameObserveTool;

impl GameTool for GameObserveTool {
    fn name(&self) -> &str {
        "game_observe"
    }

    fn description(&self) -> &str {
        "Get the current game state as text"
    }

    fn execute(&self, _args: Value) -> Result<Value, String> {
        Ok(serde_json::json!({
            "message": "use GameSessionManager.observe(session_id) to get game state",
        }))
    }
}

// ─── game_legal_actions ───

struct GameLegalActionsTool;

impl GameTool for GameLegalActionsTool {
    fn name(&self) -> &str {
        "game_legal_actions"
    }

    fn description(&self) -> &str {
        "Get all legal actions for the current game state"
    }

    fn execute(&self, _args: Value) -> Result<Value, String> {
        Ok(serde_json::json!({
            "message": "use GameSessionManager.legal_actions(session_id) to get legal actions",
        }))
    }
}

// ─── game_train ───

struct GameTrainTool;

impl GameTool for GameTrainTool {
    fn name(&self) -> &str {
        "game_train"
    }

    fn description(&self) -> &str {
        "Run a training loop for a game (N episodes, greedy policy)"
    }

    fn execute(&self, args: Value) -> Result<Value, String> {
        let game_name = args["game_name"]
            .as_str()
            .ok_or("missing game_name")?
            .to_string();
        let episodes = args["episodes"].as_u64().unwrap_or(10) as usize;
        let max_turns = args["max_turns"].as_u64().unwrap_or(50) as usize;
        let seed = args["seed"].as_u64().unwrap_or(42);

        let mut total_steps = 0usize;
        let mut total_reward = 0.0f64;
        let mut best_score = f64::NEG_INFINITY;
        let mut completed = 0usize;

        for ep_idx in 0..episodes {
            let ep_seed = seed.wrapping_add(ep_idx as u64);
            let env: Box<dyn NtGameEnv> = match game_name.as_str() {
                "HexCrucible" => {
                    let cfg = HexCrucibleConfig::for_constellation(0);
                    Box::new(HexCrucible::new(cfg, ep_seed))
                }
                "HexTicTacToe" => Box::new(HexTicTacToe::new(ep_seed)),
                "2048" => Box::new(Game2048::new(ep_seed)),
                other => return Err(format!("Unknown game: {}", other)),
            };

            let mut session = GameSession::new(game_name.clone(), env, ep_seed);
            let mut turns = 0usize;

            while !session.is_terminal() && turns < max_turns {
                let legal = session.legal_actions();
                if legal.is_empty() {
                    break;
                }
                let action = &legal[0]; // greedy
                let result = session.step(action);
                total_reward += result.reward;
                turns += 1;
                if result.done {
                    break;
                }
            }

            let score = turns as f64;
            if score > best_score {
                best_score = score;
            }
            total_steps += turns;
            completed += 1;
        }

        let avg_reward = if completed > 0 {
            total_reward / completed as f64
        } else {
            0.0
        };
        let avg_turns = if completed > 0 {
            total_steps as f64 / completed as f64
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "status": "training_complete",
            "game_name": game_name,
            "episodes_completed": completed,
            "total_steps": total_steps,
            "avg_reward": avg_reward,
            "avg_turns": avg_turns,
            "best_score": best_score,
        }))
    }
}

// ─── game_metrics ───

struct GameMetricsTool {
    metrics: std::sync::Mutex<McpTrainingMetrics>,
}

impl GameMetricsTool {
    fn new(metrics: std::sync::Mutex<McpTrainingMetrics>) -> Self {
        Self { metrics }
    }
}

impl GameTool for GameMetricsTool {
    fn name(&self) -> &str {
        "game_metrics"
    }

    fn description(&self) -> &str {
        "Get accumulated training metrics"
    }

    fn execute(&self, _args: Value) -> Result<Value, String> {
        let metrics = self
            .metrics
            .lock()
            .map_err(|e| format!("lock poisoned: {}", e))?;
        Ok(serde_json::to_value(&*metrics).map_err(|e| e.to_string())?)
    }
}

// ═══════════════════════════════════════════════════════════════════
// GameToolRegistry — holds HashMap of tool_name → execute fn
// ═══════════════════════════════════════════════════════════════════

/// Registry of all NT-GAME MCP tools, keyed by tool name.
pub struct GameToolRegistry {
    tools: HashMap<String, Box<dyn GameTool>>,
}

impl GameToolRegistry {
    pub fn new() -> Self {
        let metrics = std::sync::Mutex::new(McpTrainingMetrics::default());
        let mut tools: HashMap<String, Box<dyn GameTool>> = HashMap::new();

        let entries: Vec<Box<dyn GameTool>> = vec![
            Box::new(GameListTool),
            Box::new(GameCreateTool),
            Box::new(GameStepTool),
            Box::new(GameObserveTool),
            Box::new(GameLegalActionsTool),
            Box::new(GameTrainTool),
            Box::new(GameMetricsTool::new(metrics)),
        ];

        for tool in entries {
            tools.insert(tool.name().to_string(), tool);
        }

        Self { tools }
    }

    /// Get all tool definitions (for MCP `tools/list`).
    pub fn definitions(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                })
            })
            .collect()
    }

    /// Execute a tool by name with the given arguments.
    pub fn execute(&self, name: &str, args: Value) -> Result<Value, String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| format!("Unknown tool: {}", name))?;
        tool.execute(args)
    }

    /// Get the number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for GameToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// GameSessionManager — manages active sessions by session_id
// ═══════════════════════════════════════════════════════════════════

/// Manages active game sessions, keyed by session_id.
pub struct GameSessionManager {
    sessions: HashMap<String, GameSession>,
    next_id: u64,
}

impl GameSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create a new session for the given game, return its session_id.
    pub fn create_session(&mut self, game_name: &str, seed: u64) -> Result<String, String> {
        let env: Box<dyn NtGameEnv> = match game_name {
            "HexCrucible" => {
                let cfg = HexCrucibleConfig::for_constellation(0);
                Box::new(HexCrucible::new(cfg, seed))
            }
            "HexTicTacToe" => Box::new(HexTicTacToe::new(seed)),
            "2048" => Box::new(Game2048::new(seed)),
            other => {
                return Err(format!(
                    "Unknown game: {}. Available: HexCrucible, HexTicTacToe, 2048",
                    other
                ));
            }
        };

        let id = format!("session_{}", self.next_id);
        self.next_id += 1;
        let session = GameSession::new(game_name.to_string(), env, seed);
        self.sessions.insert(id.clone(), session);
        Ok(id)
    }

    /// Get a reference to a session by id.
    pub fn get_session(&self, id: &str) -> Option<&GameSession> {
        self.sessions.get(id)
    }

    /// Get a mutable reference to a session by id.
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut GameSession> {
        self.sessions.get_mut(id)
    }

    /// Step the game in the given session.
    pub fn step(&mut self, id: &str, action_kind: &str, params: Value) -> Result<Value, String> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| format!("No session: {}", id))?;

        if session.is_terminal() {
            return Err("Game is already over".into());
        }

        let action = Action {
            kind: action_kind.to_string(),
            params,
            actor_id: session.current_player(),
        };

        let result = session.step(&action);
        let legal: Vec<Value> = session
            .legal_actions()
            .iter()
            .map(|a| serde_json::json!({"kind": a.kind, "params": a.params}))
            .collect();

        Ok(serde_json::json!({
            "observation": result.observation.text,
            "reward": result.reward,
            "done": result.done,
            "turn": session.turn,
            "legal_actions": legal,
            "hexagram": result.observation.hexagram,
            "phi": result.observation.phi,
            "info": result.info,
        }))
    }

    /// Observe the current state of a session.
    pub fn observe(&self, id: &str) -> Result<Value, String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("No session: {}", id))?;

        let text = session.observe();
        Ok(serde_json::json!({
            "game_name": session.game_name,
            "turn": session.turn,
            "text_state": text,
            "is_terminal": session.is_terminal(),
            "current_player": session.current_player(),
            "state": session.state,
        }))
    }

    /// Get legal actions for a session.
    pub fn legal_actions(&self, id: &str) -> Result<Value, String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("No session: {}", id))?;

        let actions: Vec<Value> = session
            .legal_actions()
            .iter()
            .map(|a| {
                serde_json::json!({
                    "kind": a.kind,
                    "params": a.params,
                    "actor_id": a.actor_id,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "game_name": session.game_name,
            "turn": session.turn,
            "count": actions.len(),
            "actions": actions,
        }))
    }

    /// Close (remove) a session.
    pub fn close_session(&mut self, id: &str) -> Result<Value, String> {
        let session = self
            .sessions
            .remove(id)
            .ok_or_else(|| format!("No session: {}", id))?;

        Ok(serde_json::json!({
            "closed": id,
            "game_name": session.game_name,
            "turns_played": session.turn,
            "is_terminal": session.is_terminal(),
        }))
    }

    /// List all active session ids.
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for GameSessionManager {
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
    fn test_registry_tool_count() {
        let reg = GameToolRegistry::new();
        assert_eq!(reg.tool_count(), 7);
    }

    #[test]
    fn test_game_list() {
        let reg = GameToolRegistry::new();
        let out = reg.execute("game_list", serde_json::json!({})).unwrap();
        let games: Vec<Value> = serde_json::from_value(out).unwrap();
        assert!(games.len() >= 3);
        let names: Vec<&str> = games.iter().map(|g| g["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"HexCrucible"));
        assert!(names.contains(&"HexTicTacToe"));
        assert!(names.contains(&"2048"));
    }

    #[test]
    fn test_game_list_structure() {
        let reg = GameToolRegistry::new();
        let out = reg.execute("game_list", serde_json::json!({})).unwrap();
        let games: Vec<Value> = serde_json::from_value(out).unwrap();
        for game in &games {
            assert!(game["name"].is_string());
            assert!(game["constellation"]["min"].is_number());
            assert!(game["constellation"]["max"].is_number());
            assert!(game["skills"].is_array());
        }
    }

    #[test]
    fn test_game_create() {
        let reg = GameToolRegistry::new();
        let out = reg
            .execute(
                "game_create",
                serde_json::json!({"game_name": "HexTicTacToe", "seed": 42}),
            )
            .unwrap();
        assert_eq!(out["game_name"], "HexTicTacToe");
        assert!(out["observation"].is_string());
        assert!(out["legal_actions"].is_array());
        assert!(out["state"].is_object());
    }

    #[test]
    fn test_game_create_unknown() {
        let reg = GameToolRegistry::new();
        let err = reg
            .execute("game_create", serde_json::json!({"game_name": "Nope"}))
            .unwrap_err();
        assert!(err.contains("Unknown game"));
    }

    #[test]
    fn test_game_train() {
        let reg = GameToolRegistry::new();
        let out = reg
            .execute(
                "game_train",
                serde_json::json!({
                    "game_name": "HexTicTacToe",
                    "episodes": 3,
                    "max_turns": 10,
                    "seed": 42,
                }),
            )
            .unwrap();
        assert_eq!(out["status"], "training_complete");
        assert_eq!(out["episodes_completed"], 3);
    }

    #[test]
    fn test_game_metrics() {
        let reg = GameToolRegistry::new();
        let out = reg.execute("game_metrics", serde_json::json!({})).unwrap();
        assert!(out["episodes_completed"].is_number());
    }

    #[test]
    fn test_unknown_tool() {
        let reg = GameToolRegistry::new();
        let err = reg.execute("nope", serde_json::json!({})).unwrap_err();
        assert!(err.contains("Unknown tool"));
    }

    #[test]
    fn test_session_manager_create_and_observe() {
        let mut mgr = GameSessionManager::new();
        let id = mgr.create_session("HexTicTacToe", 42).unwrap();
        let obs = mgr.observe(&id).unwrap();
        assert_eq!(obs["game_name"], "HexTicTacToe");
        assert!(obs["text_state"].is_string());
    }

    #[test]
    fn test_session_manager_step() {
        let mut mgr = GameSessionManager::new();
        let id = mgr.create_session("HexTicTacToe", 42).unwrap();
        let result = mgr
            .step(&id, "Move", serde_json::json!({"row": 0, "col": 0}))
            .unwrap();
        assert!(result["reward"].is_number());
        assert!(result["done"].is_boolean());
        assert_eq!(result["turn"], 1);
    }

    #[test]
    fn test_session_manager_legal_actions() {
        let mut mgr = GameSessionManager::new();
        let id = mgr.create_session("2048", 7).unwrap();
        let out = mgr.legal_actions(&id).unwrap();
        assert!(out["actions"].is_array());
        assert!(out["count"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_session_manager_close() {
        let mut mgr = GameSessionManager::new();
        let id = mgr.create_session("HexTicTacToe", 42).unwrap();
        assert_eq!(mgr.session_count(), 1);
        let closed = mgr.close_session(&id).unwrap();
        assert_eq!(closed["closed"], id);
        assert_eq!(mgr.session_count(), 0);
    }

    #[test]
    fn test_session_manager_unknown_session() {
        let mut mgr = GameSessionManager::new();
        let err = mgr.observe("nonexistent").unwrap_err();
        assert!(err.contains("No session"));
    }

    #[test]
    fn test_session_manager_list_sessions() {
        let mut mgr = GameSessionManager::new();
        let id1 = mgr.create_session("HexTicTacToe", 1).unwrap();
        let id2 = mgr.create_session("2048", 2).unwrap();
        let ids = mgr.list_sessions();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }
}
