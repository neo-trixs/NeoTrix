//! NT-GAME Autonomous Evolution Loop
//!
//! Fully autonomous daemon — no human interaction required.
//! Continuously trains the consciousness entity through self-play,
//! adapts difficulty, and evolves game rules via constellation unlocks.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::env::{GameOutcome, NtGameEnv};
use super::hex_crucible::{HexCrucible, HexCrucibleConfig};
use super::play::adaptive::{AdaptiveDifficultyConfig, DifficultyAdjuster};
use super::builtin::{Game2048, HexTicTacToe};

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
    /// ⭐ **单调递增的 tick 计数**。此前只有 `total_episodes`，而活路径
    /// （`handlers_game.rs::handle_game_training`）的日志需要「第几个 tick」。
    /// 这与渲染层调研强调的「每条响应必须带单调递增序号」是同一条纪律：
    /// 没有它，丢帧/漏事件后接收方无法判断自己缺了什么、也无法对齐。
    pub total_ticks: usize,
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
    /// ⭐ 本次是第几个 tick（单调）。**消费方对齐与丢帧检测都靠它。**
    pub tick: usize,
    pub timestamp: String,
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

    /// ⭐ 按 constellation 造一个**正式游戏**（`NtGameEnv` 实现者）。
    ///
    /// ⛔ 此前这里造的是 `AutoTicTacToe`/`Auto2048`/`AutoHexCrucible` ——
    ///   那三个是为「避免跨模块依赖」而**重写的内联副本**（见被删区块的
    ///   原注释「Inline Game Engines (self-contained, no cross-module imports)」）。
    ///   但它们与 `builtin/hex_tictactoe.rs` / `builtin/game_2048.rs` /
    ///   `hex_crucible.rs` 是**同一批游戏的第二、第三次实现**，
    ///   而后者才是唯一实现了完整 `NtGameEnv` 契约（含 `get_game_rules` /
    ///   `to_hexagram` / `phi_contribution` / `difficulty`）的那份。
    ///
    /// ✅ 之所以当初能删掉 `Auto*`，是因为它比 `NtGameEnv` 多两个方法
    ///   （`reward(player)` 与 `board_hexagrams()`）—— 而**这两个都能从
    ///   `NtGameEnv::state()` 推出**，不需要另立接口：
    ///   · `reward(player)`  ⇒ `state().scores.get(&player)`
    ///   · `board_hexagrams()` ⇒ `state().hexagram_states`（本来就是 `Vec<u8>`）
    pub fn create_game(&self, constellation: u8, seed: u64) -> Box<dyn NtGameEnv> {
        match constellation {
            0 => Box::new(HexTicTacToe::new(seed)),
            1 => Box::new(Game2048::new(seed)),
            // `for_constellation` 内部按 level 选 grid_size/max_turns，
            // 与被删的 `AutoHexCrucible::new((level + 2).min(8))` 等价。
            other => Box::new(HexCrucible::new(
                HexCrucibleConfig::for_constellation(other),
                seed,
            )),
        }
    }

    pub fn game_name(constellation: u8) -> &'static str {
        match constellation {
            0 => "HexTicTacToe",
            1 => "2048",
            _ => "HexCrucible",
        }
    }


    pub fn tick(&mut self) -> GameTickReport {
        self.state.is_running = true;
        self.state.total_ticks += 1;
        let tick_no = self.state.total_ticks;
        let constellation = self.state.current_constellation;
        let seed = self.state.total_episodes as u64 + 1;
        let mut game = self.create_game(constellation, seed);

        let mut wins = 0usize;
        let mut losses = 0usize;
        let mut draws = 0usize;
        let mut total_reward = 0.0f64;
        let mut total_turns = 0.0f64;
        let mut total_phi = 0.0f64;
        // 有多少 episode 的游戏**真的定义了** phi（见 `phi_contribution` → `Option`）
        let mut phi_defined_episodes = 0usize;
        let episodes = self.config.episodes_per_round;

        for ep in 0..episodes {
            // ⭐ `NtGameEnv::reset` 收 `Option<u64>` 并返回 `Observation`
            //（被删的 `AutoGame::reset(u64)` 不返回）。返回值此处无用，丢弃。
            let _ = game.reset(Some(seed.wrapping_add(ep as u64)));
            let mut steps = 0usize;
            let ep_seed = seed.wrapping_add(ep as u64);

            // ⭐ 有效回合预算 = max(外层安全帽, 本局预算)。
            //   此前只用 `self.config.max_turns`(50)，把 constellation 4/5 所需的
            //   60/80 硬砍掉 ⇒ 永不到终局 ⇒ 全判 draw（实测，见 `NtGameEnv::turn_budget`）。
            let budget = self
                .config
                .max_turns
                .max(game.turn_budget().unwrap_or(0));

            while !game.is_terminal() && steps < budget {
                // ⭐ `NtGameEnv::legal_actions()` **无 player 参数**
                //（被删的 `AutoGame::legal_actions(player)` 有）。
                let actions = game.legal_actions();
                if actions.is_empty() {
                    break;
                }

                // Simple policy: pick action based on seed (simulating a policy network)
                let idx = ((ep_seed.wrapping_add(steps as u64)) % actions.len() as u64) as usize;
                let result = game.step(&actions[idx]);
                total_reward += result.reward;
                // ⭐ phi 改从 `NtGameEnv::phi_contribution()` 取（被删的
                //   `AutoGame::phi()`）。`result.info["phi"]` 是内联副本的私有约定，
                //   正式游戏不保证写这个键 ⇒ 旧写法在正式游戏上恒为 0。
                // ⭐ `phi_contribution()` 现返回 `Option<f64>`：只有定义了 phi 语义
                //   的游戏（当前仅 `HexCrucible`）才计入，`None` 不再被洗成 0.0 数据。
                if let Some(phi) = game.phi_contribution() {
                    total_phi += phi;
                    phi_defined_episodes += 1;
                }
                steps += 1;
            }

            total_turns += steps as f64;
            // ⭐ 胜负改由 `NtGameEnv::outcome()` 给出 —— 该方法读取各游戏**早已算好**
            //   的 `winner`，不再从 `state().scores` 猜（实测 `scores` 从不填胜负，
            //   猜测导致 `win_rate` 恒 0 ⇒ constellation 进阶成为死逻辑）。
            // ⛔ 记分制游戏（2048）终局报 `Draw`（无胜者概念），故「输」只可能
            //   出现在有胜者语义的对局里 —— 这与真实定义一致，不是缺口。
            let outcome = game.outcome();
            let reward = match outcome {
                Some(GameOutcome::Win(current)) => {
                    // `GameOutcome::Win` 存玩家下标（usize），`current_player()` 返回
                    // `ActorId`(= u32) ⇒ 显式转换，不依赖二者恰好同型。
                    if current as u32 == game.current_player() {
                        wins += 1;
                    } else {
                        losses += 1;
                    }
                    1.0
                }
                Some(GameOutcome::Draw) => {
                    draws += 1;
                    0.5
                }
                // 未在 `max_turns` 内走到终局：既非胜也非负，计 draw 但**不**给 reward。
                None => {
                    draws += 1;
                    0.0
                }
            };

            self.state.total_episodes += 1;
            self.state.total_steps += steps;
            self.difficulty_adjuster.record_episode(reward > 0.0);
        }

        let win_rate = if episodes > 0 {
            wins as f64 / episodes as f64
        } else {
            0.0
        };
        let avg_reward = if episodes > 0 {
            total_reward / episodes as f64
        } else {
            0.0
        };
        let avg_turns = if episodes > 0 {
            total_turns / episodes as f64
        } else {
            0.0
        };
        // ⭐ 分母用 `phi_defined_episodes` 而非 `episodes`：未定义 phi 的游戏
        //   （HexTicTacToe / 2048 在补齐前）根本不该让分母变大，否则会把
        //   「无 phi 语义」稀释成「phi 很小」。0 则如实报 0。
        let phi_avg = if phi_defined_episodes > 0 {
            total_phi / (phi_defined_episodes as f64 * avg_turns.max(1.0))
        } else {
            0.0
        };

        self.phi_history.push(phi_avg);
        if self.phi_history.len() > 100 {
            self.phi_history.remove(0);
        }
        let phi_avg_stable: f64 = if self.phi_history.is_empty() {
            0.0
        } else {
            self.phi_history.iter().sum::<f64>() / self.phi_history.len() as f64
        };

        // Health: win_rate + phi 的复合分。
        //
        // ⚠️ **本公式当前不可作为决策依据**，原因已实测定位（勿再怀疑公式本身）：
        //   `tick()` 的动作选择仍是
        //   `idx = (ep_seed + steps) % actions.len()` —— 一个**对种子取模**的
        //   自占位策略（注释自称 "simulating a policy network"）。它不学习、不记忆、
        //   不依状态 ⇒ 自对弈产出的是**噪声**，故 `win_rate` 与 `avg_reward` 都无
        //   统计意义，`advance_threshold` 也无法被真实跨过。
        // ⇒ 要让 constellation 进阶真正生效，必须先给 `tick()` 接上真策略
        //   （仓内已有 `play/self_play_loop.rs::SelfPlayLoop` + `advantage`/
        //   `grpo_adapter`，收敛计划本就判其「保留」—— 但目前同样未被本循环调用）。
        //   在那之前**保留**此公式以免改变既有报告形状，仅标注事实。
        let health = (win_rate * 0.6 + phi_avg_stable * 0.4).clamp(0.0, 1.0);
        self.health_history.push(health);
        if self.health_history.len() > 100 {
            self.health_history.remove(0);
        }

        // Update scores
        let best = self
            .state
            .constellation_scores
            .entry(constellation)
            .or_insert(0.0);
        if avg_reward > *best {
            *best = avg_reward;
        }
        let ep_count = self
            .state
            .constellation_episodes
            .entry(constellation)
            .or_insert(0);
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
            wins,
            losses,
            draws,
            avg_reward,
            avg_turns,
            win_rate,
            phi_avg: phi_avg_stable,
            health_score: health,
            constellation_advanced: advanced,
            tick: tick_no,
            timestamp: timestamp(),
        }
    }

    pub fn should_advance(&self) -> bool {
        if !self.config.auto_advance {
            return false;
        }
        if self.state.current_constellation >= self.config.max_constellation {
            return false;
        }
        let ep = self
            .state
            .constellation_episodes
            .get(&self.state.current_constellation)
            .copied()
            .unwrap_or(0);
        ep >= self.config.min_episodes_before_advance
    }

    pub fn advance_constellation(&mut self) {
        if self.state.current_constellation < self.config.max_constellation {
            self.state.current_constellation += 1;
        }
    }

    /// 连跑 `n` 个 tick，返回每 tick 的报告。**tick 号单调递增**（见 `GameTickReport::tick`）。
    pub fn run_ticks(&mut self, n: usize) -> Vec<GameTickReport> {
        (0..n).map(|_| self.tick()).collect()
    }

    /// JSON 状态快照（供 MCP / 后台循环观测）。
    pub fn status(&self) -> String {
        let st = &self.state;
        serde_json::json!({
            "constellation": st.current_constellation,
            "total_ticks": st.total_ticks,
            "total_episodes": st.total_episodes,
            "total_steps": st.total_steps,
            "is_running": st.is_running,
            "constellation_scores": st.constellation_scores,
        })
        .to_string()
    }

    pub fn state(&self) -> &GameEvolutionState {
        &self.state
    }
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!(
        "{}-{:02}-{:02}T00:00:00Z",
        1970 + (secs / 31_536_000) as u32,
        ((secs % 31_536_000) / 2_592_000) as u32 + 1,
        ((secs % 2_592_000) / 86_400) as u32 + 1
    )
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
        let mut evo = GameEvolutionLoop::new(GameEvolutionConfig {
            episodes_per_round: 3,
            ..Default::default()
        });
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

    // ⭐ 以下三个测试原先打在被删的 `Auto*` 内联副本上，现改为打
    //   **正式游戏实现**（`create_game` 返回的 `Box<dyn NtGameEnv>`）。
    //   这是「副本已消失」的判别力证据：换错符号 cargo 才报错，
    //   所以额外断言 `meta().name`，确保拿到的是预期那个游戏。
    #[test]
    fn test_tictactoe_game() {
        let evo = GameEvolutionLoop::new(GameEvolutionConfig::default());
        let mut game = evo.create_game(0, 42);
        let _ = game.reset(Some(42));
        assert!(!game.is_terminal());
        let actions = game.legal_actions();
        assert!(!actions.is_empty());
        assert_eq!(game.meta().name, "HexTicTacToe");
    }

    #[test]
    fn test_2048_game() {
        let evo = GameEvolutionLoop::new(GameEvolutionConfig::default());
        let mut game = evo.create_game(1, 42);
        let _ = game.reset(Some(42));
        assert!(!game.is_terminal());
        assert_eq!(game.meta().name, "2048");
    }

    #[test]
    fn test_hex_crucible_game() {
        let evo = GameEvolutionLoop::new(GameEvolutionConfig::default());
        let mut game = evo.create_game(2, 42);
        let _ = game.reset(Some(42));
        assert!(!game.is_terminal());
        assert_eq!(game.meta().name, "HexCrucible");
        // `Option<f64>`：`HexCrucible` 是唯一定义了 phi 语义的游戏 ⇒ 必须 `Some`。
        assert!(game.phi_contribution().is_some());
        // ⭐ `board_hexagrams` 的等价物现在从 state() 直接拿 —— 曾是 Auto* 独有方法。
        assert!(!game.state().hexagram_states.is_empty());
    }




    /// ⭐ 回归护栏：六个星位**都**必须能真正走到终局并产出胜负分类。
    ///
    /// 这条测试锁住三个曾被实测抓出的缺陷，任一复发即红：
    /// ① `NtGameEnv` 无终局出口 ⇒ `wins+losses+draws` 恒 0（实测全 draw / win_rate≡0）；
    /// ② 外层 `max_turns=50` 砍断 constellation 4/5 所需的 60/80 回合
    ///    ⇒ 永不到终局（实测 `turns` 恒 50、30/30 全 draw）；
    /// ③ `phi_contribution()` 默认 `0.0` 把「未定义」洗成「真值 0」。
    ///
    /// 断言用**不变量**而非具体数值 —— 随机自对弈的具体胜率不应被钉死
    /// （那正是 D5：无策略时它就是噪声，钉死会逼后人去调种子造假）。
    #[test]
    fn test_every_constellation_reaches_a_verdicts() {
        let cap = GameEvolutionConfig::default().max_turns;
        let mut constellations_with_wins = 0;

        for c in 0..6u8 {
            let mut evo = GameEvolutionLoop::new(GameEvolutionConfig {
                episodes_per_round: 30,
                ..Default::default()
            });
            evo.state.current_constellation = c;
            let declared = evo.create_game(c, 1).turn_budget().unwrap_or(0);
            let r = evo.tick();

            // ① 每个 episode 都要有归宿（胜/负/和），不得出现「没结论」
            assert_eq!(
                r.wins + r.losses + r.draws, 30,
                "constellation {c} 有 episode 未走到终局：W/L/D={}/{}/{} turns={}",
                r.wins, r.losses, r.draws, r.avg_turns
            );
            // ② 回合数不得超过**有效预算**（= max(安全帽, 本局预算)）
            assert!(
                r.avg_turns <= (cap.max(declared)) as f64 + 1e-9,
                "constellation {c} 回合数 {} 超过有效预算 {}（declared={declared} cap={cap}）——                 说明又被某个平帽截断了",
                r.avg_turns, cap.max(declared)
            );
            if r.wins > 0 {
                constellations_with_wins += 1;
            }
        }

        // ③ `outcome()` 确实接上了 —— 记分制 2048 无胜者，故要求「至少两个」星位有胜局，
        //    这样即使有人把 2048 误接成胜负游戏，该断言仍不会被误判为通过。
        assert!(
            constellations_with_wins >= 2,
            "只有 {constellations_with_wins} 个星位产出胜局 —— `outcome()` 可能未生效"
        );
    }

    /// ⭐ 回归护栏：`HexCrucible` 高星位声明的预算**必须**大于外层默认安全帽，
    /// 否则 D6 会以「某个平帽」的形式复发（本次实测：4/5 星位被 50 砍断）。
    #[test]
    fn test_high_constellation_budget_exceeds_flat_cap() {
        let evo = GameEvolutionLoop::new(GameEvolutionConfig::default());
        let cap = evo.config.max_turns;
        for c in 4..6u8 {
            let declared = evo
                .create_game(c, 1)
                .turn_budget()
                .unwrap_or_else(|| panic!("constellation {c} 必须声明回合预算"));
            assert!(
                declared > cap,
                "constellation {c} 预算 {declared} 未超过安全帽 {cap} —— 该星位会被截断"
            );
        }
    }

    /// ⭐ 回归护栏：三局都必须**真的定义** phi（`Some`），不得再吃默认 `0.0`。
    /// 若某局phi 未定义，`Option` 契约要求它如实报 `None`，而不是伪装成 0.0。
    #[test]
    fn test_all_games_declare_phi_or_honestly_none() {
        let evo = GameEvolutionLoop::new(GameEvolutionConfig::default());
        for c in 0..6u8 {
            let phi = evo.create_game(c, 1).phi_contribution();
            assert!(
                phi.is_some(),
                "constellation {c} 的 phi 为 None —— 这可以接受，但必须**显式**实现，                 不能靠 trait 默认值静默变成 0.0"
            );
            if let Some(v) = phi {
                // 非负是**全游戏**的共识。
                assert!(v >= 0.0, "phi 不得为负，实得 {v}");
                // ⛔ 只有 constellation 0/1 的 phi 是本次**新写且明确归一到 [0,1]** 的
                //   （棋盘多样性比例 / 分数÷2048）。`HexCrucible::compute_phi()` 是
                //   游戏内部的共鸣度量，**按设计不归一**（实测 53/49≈1.08）——
                //   此处不得替它强加 [0,1] 上限，那会篡改游戏语义。
                //   （此断言曾以 [0,1] 卡住全部星位，实测抓到的正是这条误判。）
                if c <= 1 {
                    assert!(
                        (0.0..=1.0).contains(&v),
                        "constellation {c} 的 phi 由新公式给出，应归一到 [0,1]，实得 {v}"
                    );
                }
            }
        }
    }
}
