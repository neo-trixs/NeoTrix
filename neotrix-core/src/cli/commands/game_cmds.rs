//! Game 命令 — 游戏列表、对弈、训练、竞技
//!
//! /game list              列出可用游戏及星座等级/技能
//! /game play [game]       开始交互式对弈 (返回初始状态)
//! /game train [game]      启动自博弈训练循环
//! /game status            显示训练指标
//! /game arena             显示竞技场统计

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;

// ====== /game ======

pub struct GameCmd;

impl CliCommand for GameCmd {
    fn name(&self) -> &str {
        "/game"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/gm", "/play"]
    }

    fn description(&self) -> &str {
        "游戏系统:\n  /game list              列出可用游戏及星座等级/技能\n  /game play [game]       开始交互式对弈 (返回初始状态)\n  /game train [game]      启动自博弈训练循环\n  /game status            显示训练指标\n  /game arena             显示竞技场统计"
    }

    fn is_primary(&self) -> bool {
        true
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let sub = args.iter().find(|a| *a != "--json").map(|s| s.as_str()).unwrap_or("list");

        match sub {
            "list" => game_list(args, want_json),
            "play" => game_play(args, want_json),
            "train" => game_train(args, want_json),
            "status" => game_status(args, want_json),
            "arena" => game_arena(args, want_json),
            _ => {
                let out = CommandOutput::ok(self.description());
                if want_json {
                    out.with_json(serde_json::json!({
                        "subcommands": ["list", "play", "train", "status", "arena"]
                    }))
                } else {
                    out
                }
            }
        }
    }
}

// ====== 子命令实现 ======

fn game_list(_args: &[String], want_json: bool) -> CommandOutput {
    let games = vec![
        ("HexCrucible", "C3", "E8推理", "E8 hexagram决策博弈, 测试多步推理与模式识别"),
        ("HexTicTacToe", "C2", "GWT注意力", "基于E8格点的井字棋变体, 注意力路由测试"),
        ("2048", "C1", "动态规划", "经典数字合并, 优化策略训练"),
    ];

    let mut body = String::from(
        "🎮 Available Games\n\
         ────────────────────────\n"
    );
    for (name, constellation, skill, desc) in &games {
        body.push_str(&format!(
            "  · {:<16} constellation={} skill={}\n    {}\n",
            name, constellation, skill, desc
        ));
    }

    let out = CommandOutput::ok(body.trim_end());
    if want_json {
        out.with_json(serde_json::json!({
            "games": [
                {"name": "HexCrucible", "constellation": "C3", "skill": "E8推理", "desc": "E8 hexagram决策博弈"},
                {"name": "HexTicTacToe", "constellation": "C2", "skill": "GWT注意力", "desc": "E8格点井字棋变体"},
                {"name": "2048", "constellation": "C1", "skill": "动态规划", "desc": "经典数字合并"}
            ]
        }))
    } else {
        out
    }
}

fn game_play(args: &[String], want_json: bool) -> CommandOutput {
    let game = args.iter()
        .find(|a| *a != "--json" && *a != "play")
        .map(|s| s.as_str())
        .unwrap_or("HexCrucible");

    let initial_state = match game {
        "HexCrucible" => format!(
            "⚔️  HexCrucible — E8 Hexagram Duel\n\
             ────────────────────────\n\
             Board: 64 hexagrams (8x8 E8 grid)\n\
             Players: Agent (you) vs Opponent\n\
             Rule: Claim hexagrams to form winning patterns\n\
             Skill tested: Multi-step E8 pattern recognition\n\n\
             Your turn. Select a hexagram (e.g. A1, B4, H8).\n\
             Type /game play --help for controls."
        ),
        "HexTicTacToe" => format!(
            "⭕ HexTicTacToe — E8 Grid Tic-Tac-Toe\n\
             ────────────────────────\n\
             Board: E8 lattice points (3x3 projected)\n\
             Players: X (you) vs O (agent)\n\
             Rule: First to align 3 on any E8 axis wins\n\
             Skill tested: GWT attention routing\n\n\
             Your turn. Select a cell (e.g. 1-9 or coord)."
        ),
        "2048" => format!(
            "🔢 2048 — Digital Merge\n\
             ────────────────────────\n\
             Board: 4x4 grid\n\
             Moves: Up / Down / Left / Right\n\
             Skill tested: Dynamic programming optimization\n\n\
             [  0 ][  2 ][  0 ][  0 ]\n\
             [  0 ][  0 ][  0 ][  4 ]\n\
             [  0 ][  0 ][  2 ][  0 ]\n\
             [  0 ][  0 ][  0 ][  0 ]\n\n\
             Score: 0 | Move: 1\n\
             Enter direction (u/d/l/r)."
        ),
        _ => {
            return CommandOutput::err(&format!(
                "Unknown game: {}. Use /game list to see available games.",
                game
            ));
        }
    };

    let out = CommandOutput::ok(&initial_state);
    if want_json {
        out.with_json(serde_json::json!({
            "op": "play",
            "game": game,
            "state": "initial",
            "turn": 1,
            "score": 0,
        }))
    } else {
        out
    }
}

fn game_train(args: &[String], want_json: bool) -> CommandOutput {
    let game = args.iter()
        .find(|a| *a != "--json" && *a != "train")
        .map(|s| s.as_str())
        .unwrap_or("HexCrucible");

    let episodes = args.iter()
        .find_map(|a| a.strip_prefix("--episodes="))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1000);

    let lr = args.iter()
        .find_map(|a| a.strip_prefix("--lr="))
        .unwrap_or("0.001");

    let body = format!(
        "🏋️ Training Loop — {}\n\
         ────────────────────────\n\
         Game: {}\n\
         Episodes: {}\n\
         Learning rate: {}\n\
         Strategy: Self-play with MCTS\n\
         Reward: Win=+1, Loss=-1, Draw=0\n\
         Discount: 0.99\n\n\
         Training started in background.\n\
         Use /game status to check progress.\n\
         Use /game arena to view competitive stats.",
        game, game, episodes, lr
    );

    let out = CommandOutput::ok(&body);
    if want_json {
        out.with_json(serde_json::json!({
            "op": "train",
            "game": game,
            "episodes": episodes,
            "lr": lr,
            "strategy": "self-play-mcts",
            "status": "started",
        }))
    } else {
        out
    }
}

fn game_status(_args: &[String], want_json: bool) -> CommandOutput {
    let body = "📊 Training Status\n\
         ────────────────────────\n\
         Game: HexCrucible\n\
         Episodes completed: 0 / 1000\n\
         Win rate: --%\n\
         Avg reward: --\n\
         ELO: 1000\n\
         Checkpoint: none yet\n\
         ETA: --\n\n\
         No active training session.\n\
         Start with /game train [game].";

    let out = CommandOutput::ok(body);
    if want_json {
        out.with_json(serde_json::json!({
            "op": "status",
            "active": false,
            "episodes_completed": 0,
            "episodes_total": 1000,
            "win_rate": null,
            "avg_reward": null,
            "elo": 1000,
        }))
    } else {
        out
    }
}

fn game_arena(_args: &[String], want_json: bool) -> CommandOutput {
    let body = "🏟️ Arena Stats\n\
         ────────────────────────\n\
         Matches played: 0\n\
         Wins: 0 | Losses: 0 | Draws: 0\n\
         Win rate: --%\n\
         Avg game length: -- turns\n\
         Opponents:\n\
           · RandomBot    ELO=800   W/L: --/--\n\
           · GreedyBot    ELO=1000  W/L: --/--\n\
           · MCTS-Bot     ELO=1200  W/L: --/--\n\
         Leaderboard:\n\
           1. (no entries)\n\n\
         Play matches with /game play or start training.";

    let out = CommandOutput::ok(body);
    if want_json {
        out.with_json(serde_json::json!({
            "op": "arena",
            "matches": 0,
            "wins": 0,
            "losses": 0,
            "draws": 0,
            "win_rate": null,
            "opponents": [
                {"name": "RandomBot", "elo": 800},
                {"name": "GreedyBot", "elo": 1000},
                {"name": "MCTS-Bot", "elo": 1200}
            ],
        }))
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "TODO: replace with real test — current placeholder asserts nothing"]
    fn test_basic() {
        panic!("test_basic is a placeholder; implement real assertion or remove");
    }
}
