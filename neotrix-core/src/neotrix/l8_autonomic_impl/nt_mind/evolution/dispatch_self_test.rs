//! 派单控制面端到端 SelfTest (P5, T3 生产接线) — 验证 P0-P4 共进化闭环。
//!
//! 单一多轮任务模拟同时验证:
//!   1. 技能级路由 bandit (RouteLearner) — 用真实派单结果学习, 有效路由从
//!      低成功率档案迁移到高成功率档案 (派单从仪式变控制面)。
//!   2. MANTA 派单拓扑 — trace 审计 + 有界修复, 结构边随行为自进化。
//!   3. MAGE 四子图共进化 — 同一 reward 流驱动 capability/task/experience/
//!      environment 子图与任务级搜索 bandit 同步更新。
//!   4. 跨轮持久化 — 学习成果落盘 KB 后可被新实例恢复 (跨会话存活)。
//!
//! 注册点: `handlers_consciousness::handle_architecture_audit` 的 SelfTestRegistry
//! (T3 inline self_test in production, 共享语言约定)。

use std::sync::Arc;

use crate::core::nt_core_self::attention_head::AttentionDomain;
use crate::core::nt_core_self_test::SelfTest;
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_mind::{
    AgentExecutionOutcome, AgentExecutor, CoEvoConfig, MetaAgentShell,
};

/// 脚本化执行器 — 可控环境: 指定档案一律失败, 其余一律成功。
/// `dispatch_and_execute` 走 `execute_with_strategy` (默认委托 execute),
/// 因此只用实现 `execute` 即可; 调用计数证明派单真的驱动了动作。
pub struct ScriptedExecutor {
    pub fail_agents: &'static [&'static str],
    pub calls: std::cell::Cell<usize>,
}

impl ScriptedExecutor {
    pub fn new(fail_agents: &'static [&'static str]) -> Self {
        Self {
            fail_agents,
            calls: std::cell::Cell::new(0),
        }
    }
}

impl AgentExecutor for ScriptedExecutor {
    fn execute(&self, agent: &str, _task: &str) -> AgentExecutionOutcome {
        self.calls.set(self.calls.get() + 1);
        if self.fail_agents.contains(&agent) {
            AgentExecutionOutcome::Failure(format!("scripted failure for {}", agent))
        } else {
            AgentExecutionOutcome::Success(format!("scripted success for {}", agent))
        }
    }
}

/// 任务模拟结果 — 供断言与审计。
#[derive(Debug, Clone, PartialEq)]
pub struct MissionReport {
    pub rounds: usize,
    pub repairs: usize,
    pub topology_revision: u64,
    pub coevo_rewards: u64,
    pub memories: usize,
    pub strategies_seen: usize,
    pub effective_agent: String,
    pub explorer_rate: f64,
    pub researcher_rate: f64,
}

/// 派单控制面 SelfTest — 注册进生产架构审计 registry, 周期性验证共进化闭环。
///
/// 场景: research_study 任务, 初始拓扑把 PatternMatch→researcher; 环境中
/// researcher 永远失败、explorer 永远成功。多轮真实派单后:
///   - learner 学会用 explorer 覆盖静态映射 (路由迁移);
///   - 拓扑 audit 把 PatternMatch 边修复为 explorer (结构自进化);
///   - coevo 四子图从同一 reward 流同步累积, 任务级搜索 bandit 覆盖多臂;
///   - 持久化 → 新实例恢复 (跨轮 playbook)。
#[derive(Debug, Clone, Copy)]
pub struct DispatchControlPlaneSelfTest {
    pub rounds: usize,
}

impl Default for DispatchControlPlaneSelfTest {
    fn default() -> Self {
        Self { rounds: 8 }
    }
}

impl SelfTest for DispatchControlPlaneSelfTest {
    fn name(&self) -> &str {
        "dispatch_control_plane"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        let tmp = std::env::temp_dir().join(format!(
            "neotrix_selftest_dispatch_{}_{}",
            std::process::id(),
            now_nanos(),
        ));
        let kb = match KnowledgeBase::open(Some(tmp.into())) {
            Ok(kb) => Arc::new(kb),
            Err(e) => return Err(vec![format!("kb open: {}", e)]),
        };

        // 共进化配置: epsilon=0 (确定性 bandit), min_evidence=2 (加速证据积累)。
        let mut shell = MetaAgentShell::with_coevo_config(
            "research_study",
            CoEvoConfig {
                epsilon: 0.0,
                max_memories: 200,
                min_evidence: 2,
                mastery_gate: 0.5,
            },
        );
        shell.learner.config.min_evidence = 2;
        let executor = ScriptedExecutor::new(&["researcher"]);

        let report = run_mission(&mut shell, &executor, self.rounds);

        // 1) 派单真实驱动了动作: 每轮都 dispatch 到执行器。
        if executor.calls.get() != self.rounds {
            failures.push(format!(
                "expected {} executions, got {}",
                self.rounds,
                executor.calls.get()
            ));
        }
        // 2) MANTA 拓扑自进化: PatternMatch 边从 researcher 修复为 explorer。
        if shell.topology.agent_for(AttentionDomain::PatternMatch) != "explorer" {
            failures.push(format!(
                "topology edge should repair to explorer, got {}",
                shell.topology.agent_for(AttentionDomain::PatternMatch)
            ));
        }
        if report.repairs == 0 || report.topology_revision == 0 {
            failures.push("no topology repair applied during mission".into());
        }
        // 3) 有效路由改善: 学习后 route_to_catalog 指向高成功率档案。
        if shell.route_to_catalog() != Some("explorer") {
            failures.push(format!(
                "effective route should be explorer, got {:?}",
                shell.route_to_catalog()
            ));
        }
        // 4) learner 行为证据: explorer 成功率高于 researcher。
        if report.explorer_rate <= report.researcher_rate {
            failures.push(format!(
                "learner should prefer explorer ({:.2}) over researcher ({:.2})",
                report.explorer_rate, report.researcher_rate
            ));
        }
        // 5) MAGE 四子图共进化: 同一 reward 流同步累积。
        if report.coevo_rewards != self.rounds as u64 {
            failures.push(format!(
                "coevo rewards should equal rounds ({}), got {}",
                self.rounds, report.coevo_rewards
            ));
        }
        if report.memories != self.rounds {
            failures.push(format!(
                "append-only memories should equal rounds ({}), got {}",
                self.rounds, report.memories
            ));
        }
        if report.strategies_seen < 3 {
            failures.push(format!(
                "task search bandit should cover multiple strategies, seen {}",
                report.strategies_seen
            ));
        }
        if (shell.coevo.mastery(AttentionDomain::PatternMatch, "explorer") - 1.0).abs() > 1e-9 {
            failures.push("coevo capability mastery for explorer should be 1.0".into());
        }

        // 6) 跨轮持久化: learner + topology + coevo 落盘 → 新实例恢复。
        if let Err(e) = shell.learner.persist(&kb) {
            failures.push(format!("learner persist: {}", e));
        }
        if let Err(e) = shell.persist_topology(&kb) {
            failures.push(format!("topology persist: {}", e));
        }
        if let Err(e) = shell.persist_coevo(&kb) {
            failures.push(format!("coevo persist: {}", e));
        }
        if failures.is_empty() {
            let mut restored = MetaAgentShell::new("research_study");
            if let Err(e) = restored.learner.load(&kb) {
                failures.push(format!("learner load: {}", e));
            }
            if let Err(e) = restored.load_topology(&kb) {
                failures.push(format!("topology load: {}", e));
            }
            if let Err(e) = restored.load_coevo(&kb) {
                failures.push(format!("coevo load: {}", e));
            }
            if failures.is_empty() {
                // 恢复校验: 拓扑边、修订号、共进化里程全部跨会话还原。
                if restored.topology.agent_for(AttentionDomain::PatternMatch) != "explorer" {
                    failures.push("restored topology edge lost".into());
                }
                if restored.topology.revision != report.topology_revision {
                    failures.push("restored topology revision lost".into());
                }
                if restored.coevo.total_rewards != report.coevo_rewards {
                    failures.push("restored coevo rewards lost".into());
                }
                if restored.coevo.evolution_revision != report.coevo_rewards {
                    failures.push("restored coevo evolution revision lost".into());
                }
                if restored.learner.rates(AttentionDomain::PatternMatch).is_empty() {
                    failures.push("restored learner evidence lost".into());
                }
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 多轮真实派单任务 — 返回共进化观测报告。
///
/// 每轮: 刺激 PatternMatch → 交替 research/explore 提示 → 真实 dispatch_and_execute
/// → MANTA audit+repair (镜像生产 handlers_core 派单后的审计块)。
pub fn run_mission(shell: &mut MetaAgentShell, executor: &ScriptedExecutor, rounds: usize) -> MissionReport {
    let mut repairs = 0usize;
    for i in 0..rounds {
        shell.stimulate(AttentionDomain::PatternMatch, 0.9);
        let hint = if i % 2 == 0 {
            "research co-evolution knowledge graph papers"
        } else {
            "explore the codebase layout for agents"
        };
        let _ = shell.dispatch_and_execute(executor, hint);
        repairs += shell.audit_and_repair_topology().len();
    }
    let rates = shell.learner.rates(AttentionDomain::PatternMatch);
    let rate_of = |a: &str| {
        rates
            .iter()
            .find(|(agent, _, _)| *agent == a)
            .map(|(_, r, _)| *r)
            .unwrap_or(0.0)
    };
    let strategies_seen = shell
        .coevo
        .bandit()
        .stats("research_study")
        .len();
    MissionReport {
        rounds,
        repairs,
        topology_revision: shell.topology.revision,
        coevo_rewards: shell.coevo.total_rewards,
        memories: shell.coevo.graph.memories.len(),
        strategies_seen,
        effective_agent: shell
            .route_to_catalog()
            .map(|a| a.to_string())
            .unwrap_or_default(),
        explorer_rate: rate_of("explorer"),
        researcher_rate: rate_of("researcher"),
    }
}

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_shell() -> MetaAgentShell {
        let mut shell = MetaAgentShell::with_coevo_config(
            "research_study",
            CoEvoConfig {
                epsilon: 0.0,
                max_memories: 200,
                min_evidence: 2,
                mastery_gate: 0.5,
            },
        );
        shell.learner.config.min_evidence = 2;
        shell
    }

    #[test]
    fn scripted_executor_fails_only_designated_agents() {
        let e = ScriptedExecutor::new(&["researcher"]);
        assert!(!e.execute("researcher", "t").is_success());
        assert!(e.execute("explorer", "t").is_success());
        assert!(e.execute("generalist", "t").is_success());
        assert_eq!(e.calls.get(), 3);
    }

    #[test]
    fn mission_coevolves_all_three_systems() {
        let mut shell = test_shell();
        let executor = ScriptedExecutor::new(&["researcher"]);
        let report = run_mission(&mut shell, &executor, 8);
        assert_eq!(report.rounds, 8);
        // 路由迁移: learner 学会 explorer 优于 researcher。
        assert!(report.explorer_rate > report.researcher_rate);
        assert_eq!(report.effective_agent, "explorer");
        // MANTA 拓扑修复。
        assert!(report.repairs >= 1);
        assert_eq!(shell.topology.agent_for(AttentionDomain::PatternMatch), "explorer");
        // MAGE 共进化: 同一 reward 流同步累积。
        assert_eq!(report.coevo_rewards, 8);
        assert_eq!(report.memories, 8);
        assert!(report.strategies_seen >= 3, "bandit should cover multiple arms, got {}", report.strategies_seen);
        assert_eq!(shell.coevo.mastery(AttentionDomain::PatternMatch, "explorer"), 1.0);
        assert_eq!(shell.coevo.mastery(AttentionDomain::PatternMatch, "researcher"), 0.0);
    }

    #[test]
    fn selftest_passes_end_to_end() {
        assert!(DispatchControlPlaneSelfTest::default().self_test().is_ok());
    }

    #[test]
    fn selftest_name_unique() {
        assert_eq!(DispatchControlPlaneSelfTest::default().name(), "dispatch_control_plane");
    }
}
