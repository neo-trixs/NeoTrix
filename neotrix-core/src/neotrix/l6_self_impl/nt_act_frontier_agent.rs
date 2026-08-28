//! L6 / NT-ACT — frontier-agent (github.com/ApodexAI/FrontierAgent) 吸收节点 (C2)。
//!
//! 源: FrontierAgent — Agent Team 框架 (ReAct / Agent Team) + 沙箱化评估 harness。
//! 多 agent 协作完成任务, 评估器在沙箱内打分。NeoTrix 视角: Agent Team 编排 +
//! 沙箱评估 harness, 评估结果与 team 运行落 SQLite (`PRAGMA busy_timeout=5000` 防并发写)。

use crate::core::nt_core_self_test::SelfTest;
use rusqlite::Connection;

/// 一个 agent team 成员。
#[derive(Debug, Clone)]
pub struct AgentMember {
    pub name: String,
    pub role: String,
}

/// 沙箱评估 harness 对一次 team 运行的评分。
#[derive(Debug, Clone, Default)]
pub struct EvalScore {
    pub passed: bool,
    pub score: f32,
}

/// Agent Team 编排 + 评估。
pub trait AgentTeam {
    fn spawn_team(&self, members: &[AgentMember]) -> usize;
    fn evaluate(&self, ran_steps: usize, expected: usize) -> EvalScore;
}

/// 纯计算评估器 (无副作用)。
pub struct FrontierAgentTeam;

impl AgentTeam for FrontierAgentTeam {
    fn spawn_team(&self, members: &[AgentMember]) -> usize {
        members.len()
    }

    fn evaluate(&self, ran_steps: usize, expected: usize) -> EvalScore {
        compute_eval(ran_steps, expected)
    }
}

/// 计算一次 team 运行的评估分数。
pub fn compute_eval(ran_steps: usize, expected: usize) -> EvalScore {
    let ratio = if expected == 0 {
        0.0
    } else {
        (ran_steps as f32 / expected as f32).clamp(0.0, 1.0)
    };
    EvalScore {
        passed: ran_steps >= expected,
        score: ratio,
    }
}

/// 打开 (或创建) 评估 harness 的 SQLite 库: 设 busy_timeout + 建 eval_runs 表。
pub fn open_db(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS eval_runs (
            id INTEGER PRIMARY KEY,
            team_name TEXT NOT NULL,
            ran_steps INTEGER NOT NULL,
            expected INTEGER NOT NULL,
            passed INTEGER NOT NULL,
            score REAL NOT NULL
        );",
    )?;
    Ok(conn)
}

/// 沙箱评估 harness: team 运行结果持久化到 SQLite。
pub struct FrontierAgentHarness {
    conn: Connection,
}

impl FrontierAgentHarness {
    /// 内存库 harness (建表 + busy_timeout)。
    pub fn in_memory() -> rusqlite::Result<Self> {
        Ok(Self::open(open_db(":memory:")?))
    }

    /// 打开 (或创建) 文件型评估库。
    pub fn open(conn: Connection) -> Self {
        Self { conn }
    }

    /// 计算评分并把本次运行落 SQLite, 返回自增 run id。
    pub fn record_run(&self, team: &str, ran_steps: usize, expected: usize) -> rusqlite::Result<i64> {
        let e = compute_eval(ran_steps, expected);
        self.conn.execute(
            "INSERT INTO eval_runs (team_name, ran_steps, expected, passed, score) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                team,
                ran_steps as i64,
                expected as i64,
                e.passed as i64,
                e.score as f64
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 读取某 run 的评分 (供后续校验/审计)。
    pub fn score_of(&self, run_id: i64) -> rusqlite::Result<Option<EvalScore>> {
        let mut stmt = self
            .conn
            .prepare("SELECT passed, score FROM eval_runs WHERE id = ?1")?;
        let mut rows = stmt.query_map([run_id], |r| {
            Ok(EvalScore {
                passed: r.get::<_, i64>(0)? != 0,
                score: r.get::<_, f64>(1)? as f32,
            })
        })?;
        match rows.next() {
            Some(Ok(e)) => Ok(Some(e)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[derive(Default)]
pub struct FrontierAgentSelfTest;

impl SelfTest for FrontierAgentSelfTest {
    fn name(&self) -> &str {
        "nt_act_frontier_agent"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let tm = FrontierAgentTeam;
        let mut errs = Vec::new();
        let n = tm.spawn_team(&[
            AgentMember {
                name: "p".into(),
                role: "planner".into(),
            },
            AgentMember {
                name: "e".into(),
                role: "executor".into(),
            },
        ]);
        if n != 2 {
            errs.push(format!("frontier: team size {n}, expected 2"));
        }
        if !tm.evaluate(5, 5).passed {
            errs.push("frontier: 5/5 steps must pass".into());
        }
        if tm.evaluate(3, 5).passed {
            errs.push("frontier: 3/5 steps must not pass".into());
        }
        if (tm.evaluate(5, 5).score - 1.0).abs() > 1e-6 {
            errs.push("frontier: full run must score 1.0".into());
        }

        // SQLite harness 端到端校验。
        let h = match FrontierAgentHarness::in_memory() {
            Ok(h) => h,
            Err(e) => return Err(vec![format!("frontier: init failed: {e}")]),
        };
        let id = match h.record_run("team-a", 5, 5) {
            Ok(id) => id,
            Err(e) => return Err(vec![format!("frontier: record_run failed: {e}")]),
        };
        match h.score_of(id) {
            Ok(Some(e)) => {
                if !e.passed || (e.score - 1.0).abs() > 1e-6 {
                    errs.push("frontier: stored run inconsistent".into());
                }
            }
            Ok(None) => errs.push("frontier: stored run missing".into()),
            Err(e) => errs.push(format!("frontier: score_of failed: {e}")),
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_size_matches() {
        let tm = FrontierAgentTeam;
        assert_eq!(tm.spawn_team(&[AgentMember { name: "a".into(), role: "r".into() }]), 1);
    }

    #[test]
    fn eval_pass_threshold() {
        assert!(FrontierAgentTeam.evaluate(4, 4).passed);
        assert!(!FrontierAgentTeam.evaluate(2, 4).passed);
    }

    #[test]
    fn eval_score_clamped() {
        assert_eq!(FrontierAgentTeam.evaluate(10, 5).score, 1.0);
    }

    #[test]
    fn harness_persists_eval_run() {
        let h = FrontierAgentHarness::in_memory().unwrap();
        let id = h.record_run("t", 5, 5).unwrap();
        let e = h.score_of(id).unwrap().unwrap();
        assert!(e.passed);
        assert_eq!(e.score, 1.0);
    }

    #[test]
    fn file_backed_harness_roundtrip() {
        let path = std::env::temp_dir().join(format!("neotrix_frontier_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let id = {
            let h = FrontierAgentHarness::open(open_db(path.to_str().unwrap()).unwrap());
            h.record_run("t", 3, 5).unwrap()
        };
        {
            let h = FrontierAgentHarness::open(open_db(path.to_str().unwrap()).unwrap());
            let e = h.score_of(id).unwrap().unwrap();
            assert!(!e.passed);
            assert_eq!(e.score, 0.6);
        }
        let _ = std::fs::remove_file(&path);
    }
}
