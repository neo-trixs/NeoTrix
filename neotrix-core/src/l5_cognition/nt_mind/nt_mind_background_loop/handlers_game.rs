//! NT-GAME autonomous training handler for the background loop.
//!
//! Runs game evolution ticks — no human interaction required.
//! The consciousness entity trains itself through self-play,
//! evolving difficulty and strategy via constellation progression.

#![allow(dead_code)]

use log::warn;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Self-contained game types (avoids cross-module dependency issues)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTickReport {
    pub constellation: u8,
    pub game_name: String,
    pub episodes: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    pub avg_reward: f64,
    pub phi_avg: f64,
    pub health: f64,
    pub constellation_advanced: bool,
    pub tick: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _GameDaemonState {
    pub constellation: u8,
    pub total_ticks: usize,
    pub total_episodes: usize,
    pub win_rate_history: Vec<f64>,
    pub is_running: bool,
}

// ═══════════════════════════════════════════════════════════════════
// Inline mini-games (self-contained)
// ═══════════════════════════════════════════════════════════════════

fn run_tictactoe(_seed: u64, max_games: usize) -> (usize, usize, f64) {
    let mut wins = 0;
    let mut total_turns = 0usize;
    for _g in 0..max_games {
        let mut board = [0u8; 9];
        let mut turn = 0usize;
        let mut done = false;
        while !done && turn < 9 {
            let player = (turn % 2) as u8 + 1;
            // Simple heuristic: prefer center, then corners, then edges
            let priority = [4,0,2,6,8,1,3,5,7];
            let mut placed = false;
            for &pos in &priority {
                if board[pos] == 0 {
                    board[pos] = player;
                    turn += 1;
                    placed = true;
                    break;
                }
            }
            if !placed { break; }
            // Check win
            let lines = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];
            for line in lines {
                if board[line[0]] != 0 && board[line[0]] == board[line[1]] && board[line[1]] == board[line[2]] {
                    done = true;
                    if board[line[0]] == 1 { wins += 1; }
                    break;
                }
            }
        }
        total_turns += turn;
    }
    let avg_turns = total_turns as f64 / max_games as f64;
    (wins, max_games - wins, avg_turns)
}

fn run_2048(seed: u64, max_games: usize) -> (usize, usize, f64) {
    let mut total_score = 0.0f64;
    for _ in 0..max_games {
        let mut board = [[0u32; 4]; 4];
        let mut score = 0u32;
        let mut rng = seed;
        // Init 2 tiles
        for _ in 0..2 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let pos = (rng >> 33) as usize % 16;
            board[pos / 4][pos % 4] = if rng % 10 == 0 { 4 } else { 2 };
        }
        // Play 50 random moves
        for _ in 0..50 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let dir = (rng >> 33) as u8 % 4;
            let old = board;
            // Simple slide
            for _ in 0..dir {
                let b = board;
                for r in 0..4 { for c in 0..4 { board[c][3-r] = b[r][c]; } }
            }
            for r in 0..4 {
                let mut v: Vec<u32> = board[r].iter().copied().filter(|&x| x != 0).collect();
                v.resize(4, 0);
                for i in 0..3 {
                    if v[i] == v[i+1] && v[i] != 0 { v[i] *= 2; score += v[i]; v[i+1] = 0; }
                }
                let mut res: Vec<u32> = v.into_iter().filter(|&x| x != 0).collect();
                res.resize(4, 0);
                board[r] = [res[0], res[1], res[2], res[3]];
            }
            for _ in 0..(4-dir)%4 {
                let b = board;
                for r in 0..4 { for c in 0..4 { board[c][3-r] = b[r][c]; } }
            }
            if board == old {
                // Spawn tile
                let empty: Vec<(usize,usize)> = (0..4).flat_map(|r| (0..4).map(move |c| (r,c)))
                    .filter(|&(r,c)| board[r][c] == 0).collect();
                if !empty.is_empty() {
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let idx = (rng >> 33) as usize % empty.len();
                    let (r,c) = empty[idx];
                    board[r][c] = if rng % 10 == 0 { 4 } else { 2 };
                }
            }
        }
        total_score += score as f64;
    }
    let avg = total_score / max_games as f64;
    let wins = (avg / 1000.0) as usize;
    (wins, max_games, avg / 10000.0)
}

fn run_hex_crucible(grid: usize, seed: u64, max_games: usize) -> (usize, usize, f64, f64) {
    let mut wins = 0;
    let mut total_phi = 0.0f64;
    let n = grid * grid;
    for g in 0..max_games {
        let mut cells: Vec<u8> = (0..n).map(|i| ((i as u64 * seed.wrapping_add(g as u64 + 7)) % 64) as u8).collect();
        let mut owners = vec![-1i8; n];
        let mut territory = [0usize; 2];
        let mut energy = [5.0f64; 2];
        owners[0] = 0; territory[0] = 1;
        owners[n-1] = 1; territory[1] = 1;
        let mut turn = 0;
        let max_turns = n * 2;
        while turn < max_turns {
            let player = turn % 2;
            // Simple policy: claim random unclaimed cell or pass
            rng_step(&mut cells, seed, turn);
            let target = (seed.wrapping_add(g as u64 + turn as u64) as usize) % n;
            if energy[player] >= 1.0 && owners[target] == -1 {
                owners[target] = player as i8;
                territory[player] += 1;
                energy[player] -= 1.0;
            } else {
                energy[player] = (energy[player] + 1.0).min(10.0);
            }
            turn += 1;
        }
        // Compute phi
        let mut phi_total = 0u32;
        for i in 0..n {
            for j in (i+1)..n {
                let dist = (cells[i] ^ cells[j]).count_ones();
                if dist <= 2 { phi_total += 6 - dist; }
            }
        }
        let phi = phi_total as f64 / (n as f64 * 6.0);
        total_phi += phi;
        if territory[0] > territory[1] { wins += 1; }
    }
    (wins, max_games, 0.0, total_phi / max_games as f64)
}

fn rng_step(cells: &mut [u8], seed: u64, step: usize) {
    // No-op placeholder for deterministic behavior
    let _ = (cells, seed, step);
}

// ═══════════════════════════════════════════════════════════════════
// Game Training Daemon
// ═══════════════════════════════════════════════════════════════════

pub(crate) struct _GameTrainingDaemon {
    pub state: _GameDaemonState,
    pub episodes_per_tick: usize,
    pub max_constellation: u8,
    pub advance_threshold: f64,
    pub min_episodes_before_advance: usize,
}

impl _GameTrainingDaemon {
    pub fn new() -> Self {
        Self {
            state: _GameDaemonState {
                constellation: 0,
                total_ticks: 0,
                total_episodes: 0,
                win_rate_history: Vec::new(),
                is_running: false,
            },
            episodes_per_tick: 10,
            max_constellation: 5,
            advance_threshold: 0.6,
            min_episodes_before_advance: 10,
        }
    }

    pub fn tick(&mut self) -> GameTickReport {
        self.state.is_running = true;
        self.state.total_ticks += 1;
        let c = self.state.constellation;
        let seed = self.state.total_ticks as u64;
        let eps = self.episodes_per_tick;

        let (wins, losses, avg_reward, _avg_turns, phi_avg, game_name) = match c {
            0 => {
                let (w, l, t) = run_tictactoe(seed, eps);
                (w, l, w as f64 / eps as f64, t, 0.1, "HexTicTacToe")
            }
            1 => {
                let (w, _l, avg) = run_2048(seed, eps);
                (w, eps, avg, 50.0, avg, "2048")
            }
            _ => {
                let grid = (c as usize + 2).min(8);
                let (w, l, _losses, phi) = run_hex_crucible(grid, seed, eps);
                (w, l, w as f64 / eps as f64, grid as f64 * 2.0, phi, "HexCrucible")
            }
        };

        let win_rate = wins as f64 / eps as f64;
        self.state.total_episodes += eps;
        self.state.win_rate_history.push(win_rate);
        if self.state.win_rate_history.len() > 50 {
            self.state.win_rate_history.remove(0);
        }

        let health = (win_rate * 0.6 + phi_avg * 0.4).clamp(0.0, 1.0);

        // Check constellation advance
        let mut advanced = false;
        let total_eps_for_c = self.state.win_rate_history.len();
        if c < self.max_constellation && win_rate >= self.advance_threshold
            && total_eps_for_c >= self.min_episodes_before_advance
        {
            self.state.constellation = c + 1;
            advanced = true;
        }

        self.state.is_running = false;

        GameTickReport {
            constellation: c,
            game_name: game_name.to_string(),
            episodes: eps,
            wins, losses,
            win_rate,
            avg_reward,
            phi_avg,
            health,
            constellation_advanced: advanced,
            tick: self.state.total_ticks,
        }
    }

    pub fn run(&mut self, ticks: usize) -> Vec<GameTickReport> {
        (0..ticks).map(|_| self.tick()).collect()
    }

    pub fn status(&self) -> String {
        serde_json::json!({
            "constellation": self.state.constellation,
            "total_ticks": self.state.total_ticks,
            "total_episodes": self.state.total_episodes,
            "avg_win_rate": if self.state.win_rate_history.is_empty() { 0.0 }
                else { self.state.win_rate_history.iter().sum::<f64>() / self.state.win_rate_history.len() as f64 },
            "is_running": self.state.is_running,
        }).to_string()
    }
}

impl Default for _GameTrainingDaemon {
    fn default() -> Self { Self::new() }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_creates() {
        let d = _GameTrainingDaemon::new();
        assert_eq!(d.state.constellation, 0);
    }

    #[test]
    fn test_tick_runs() {
        let mut d = _GameTrainingDaemon::new();
        let report = d.tick();
        assert_eq!(report.episodes, 10);
        assert!(report.avg_reward >= 0.0);
    }

    #[test]
    fn test_constellation_advance() {
        let mut d = _GameTrainingDaemon::new();
        d.advance_threshold = 0.0;
        d.min_episodes_before_advance = 1;
        let report = d.tick();
        assert!(report.constellation_advanced);
        assert_eq!(d.state.constellation, 1);
    }

    #[test]
    fn test_multi_tick() {
        let mut d = _GameTrainingDaemon::new();
        d.advance_threshold = 0.0;
        d.min_episodes_before_advance = 1;
        let reports = d.run(5);
        assert_eq!(reports.len(), 5);
        assert_eq!(d.state.total_ticks, 5);
    }

    #[test]
    fn test_tictactoe() {
        let (w, l, _) = run_tictactoe(42, 10);
        assert!(w + l == 10);
    }

    #[test]
    fn test_status() {
        let d = _GameTrainingDaemon::new();
        let s = d.status();
        assert!(s.contains("constellation"));
    }
}

// ═══════════════════════════════════════════════════════════════════
// BackgroundLoopHandle integration
// ═══════════════════════════════════════════════════════════════════

use super::*;

/// Static daemon instance — persists across ticks
static GAME_DAEMON: std::sync::LazyLock<std::sync::Mutex<_GameTrainingDaemon>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(_GameTrainingDaemon::new()));

impl BackgroundLoopHandle {
    /// NT-PLAY autonomous training tick — called every 5 minutes by background loop.
    /// No human interaction — fully autonomous self-play training.
    pub(crate) async fn handle_game_training(&mut self) {
        let report = {
            let mut daemon = match GAME_DAEMON.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    warn!("[bg] game_training: lock poisoned: {}", poisoned);
                    poisoned.into_inner()
                }
            };
            daemon.tick()
        };

        log::info!(
            "[bg] game_training tick #{}: constellation={} game={} episodes={} win_rate={:.2} phi={:.3} health={:.3}{}",
            report.tick,
            report.constellation,
            report.game_name,
            report.episodes,
            report.win_rate,
            report.phi_avg,
            report.health,
            if report.constellation_advanced { " ★ CONSTELLATION ADVANCED" } else { "" },
        );

        // Emit consciousness feedback event
        if let Some(ref bus) = self.event_bus {
            use crate::core::nt_core_event::CoreEvent;
            bus.emit(CoreEvent::GameTrainingUpdate {
                game_name: report.game_name,
                iteration: report.tick,
                policy_loss: report.avg_reward,
                win_rate: report.win_rate,
            });
            bus.emit(CoreEvent::GameConsciousnessFeedback {
                phi_delta: report.phi_avg,
                emotion_label: if report.win_rate > 0.6 { "Joy" }
                    else if report.win_rate < 0.3 { "Frustration" }
                    else { "Thinking" }.into(),
                attention_shift: format!("constellation={}", report.constellation),
            });
        }
    }
}
