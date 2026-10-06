//! NT-GAME autonomous training handler for the background loop.
//!
//! Runs game evolution ticks — no human interaction required.
//! The consciousness entity trains itself through self-play,
//! evolving difficulty and strategy via constellation progression.

#![allow(dead_code)]

use super::*;
use log::warn;

// ═══════════════════════════════════════════════════════════════════
// Game Evolution Loop — the single source of truth
// ═══════════════════════════════════════════════════════════════════

use crate::l5_cognition::nt_mind::nt_game::evolution::{
    GameEvolutionConfig, GameEvolutionLoop,
};


fn new_loop() -> GameEvolutionLoop {
    let mut cfg = GameEvolutionConfig::default();
    // 后台循环每 5 分钟触发一次；单 tick 内跑足够多的 episode，
    // 让胜率/phi 在一次 tick 内就有统计意义（原 daemon 用 10）。
    cfg.episodes_per_round = 10;
    GameEvolutionLoop::new(cfg)
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_creates() {
        let loop_ = new_loop();
        assert_eq!(loop_.state().current_constellation, 0);
        assert_eq!(loop_.state().total_ticks, 0);
    }

    #[test]
    fn test_tick_runs() {
        let mut loop_ = new_loop();
        let r = loop_.tick();
        assert_eq!(r.tick, 1);
        assert_eq!(r.episodes_played, 10);
        assert!(r.health_score >= 0.0 && r.health_score <= 1.0);
    }

    #[test]
    fn test_constellation_advance() {
        let mut loop_ = new_loop();
        let start = loop_.state().current_constellation;
        for _ in 0..10 {
            let r = loop_.tick();
            if r.constellation_advanced {
                // ⚠️ `report.constellation` 是**本轮所玩**的星位；推进发生在其**之后**，
                //   所以进阶那一轮报告里它仍是旧值 —— 判据必须看**状态**。
                assert_eq!(r.constellation, loop_.state().current_constellation - 1);
                assert!(loop_.state().current_constellation > start);
                return;
            }
        }
        // 未进阶也是合法结果（胜率未达阈值），但状态必须自洽。
        assert!(loop_.state().current_constellation <= loop_.config.max_constellation);
    }

    #[test]
    fn test_multi_tick() {
        let mut loop_ = new_loop();
        let reports = loop_.run_ticks(3);
        assert_eq!(reports.len(), 3);
        // ⭐ tick 必须单调递增 —— 这是渲染层「响应带单调序号」纪律的同源要求。
        for w in reports.windows(2) {
            assert!(w[1].tick > w[0].tick, "tick 必须在多次 tick 间单调递增");
        }
        assert_eq!(loop_.state().total_ticks, 3);
    }

    #[test]
    fn test_games_are_the_canonical_ones() {
        // ⭐ 回归护栏：constellation 0/1/2 必须落到**正式游戏实现**上，
        // 而不是被删掉的内联副本。
        let loop_ = new_loop();
        assert_eq!(GameEvolutionLoop::game_name(0), "HexTicTacToe");
        assert_eq!(GameEvolutionLoop::game_name(1), "2048");
        assert_eq!(GameEvolutionLoop::game_name(2), "HexCrucible");
        let g = loop_.create_game(0, 7);
        assert_eq!(g.meta().name, "HexTicTacToe");
        assert!(g.meta().is_builtin);
    }

    #[test]
    fn test_status() {
        let mut loop_ = new_loop();
        loop_.tick();
        let s = loop_.status();
        assert!(s.contains("constellation"), "status JSON 必须含 constellation: {s}");
    }
}

static GAME_LOOP: std::sync::LazyLock<std::sync::Mutex<GameEvolutionLoop>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(new_loop()));

impl BackgroundLoopHandle {
    /// NT-PLAY autonomous training tick — called every 5 minutes by background loop.
    /// No human interaction — fully autonomous self-play training.
    pub(crate) async fn handle_game_training(&mut self) {
        let report = {
            let mut daemon = match GAME_LOOP.lock() {
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
            report.episodes_played,
            report.win_rate,
            report.phi_avg,
            report.health_score,
            if report.constellation_advanced { " ★ CONSTELLATION ADVANCED" } else { "" },
        );

        // Emit consciousness feedback event
        if let Some(ref bus) = self.event_bus {
            use crate::l0_substrate::nt_core_event::CoreEvent;
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
