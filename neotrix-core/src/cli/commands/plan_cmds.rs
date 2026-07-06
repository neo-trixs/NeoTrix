use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_plan::{E8Plan, PlanGenerator, StepStatus};

fn plan_manager() -> &'static Mutex<PlanManager> {
    static MANAGER: OnceLock<Mutex<PlanManager>> = OnceLock::new();
    MANAGER.get_or_init(|| Mutex::new(PlanManager::load()))
}

struct PlanManager {
    plans: Vec<E8Plan>,
    active_id: Option<String>,
}

impl PlanManager {
    fn load() -> Self {
        let path = Self::path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(plans) = serde_json::from_str::<Vec<E8Plan>>(&data) {
                let active_id = plans.last().map(|p| p.id.clone());
                return Self { plans, active_id };
            }
        }
        Self { plans: Vec::new(), active_id: None }
    }

    fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self.plans) {
            let _ = std::fs::write(&path, &data);
        }
    }

    fn path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".neotrix").join("plans.json")
    }

    fn active_plan(&self) -> Option<&E8Plan> {
        self.active_id.as_ref().and_then(|id| self.plans.iter().find(|p| p.id == *id))
    }

    fn add_plan(&mut self, plan: E8Plan) {
        let id = plan.id.clone();
        self.plans.push(plan);
        self.active_id = Some(id);
        self.save();
    }

    fn complete_active(&mut self) -> bool {
        if let Some(ref id) = self.active_id.clone() {
            if let Some(plan) = self.plans.iter_mut().find(|p| p.id == *id) {
                for step in plan.steps.iter_mut() {
                    if matches!(step.status, StepStatus::Pending) {
                        step.status = StepStatus::Skipped;
                    }
                }
                self.active_id = None;
                self.save();
                return true;
            }
        }
        false
    }
}

pub struct PlanCmd;
impl CliCommand for PlanCmd {
    fn name(&self) -> &str { "/plan" }
    fn aliases(&self) -> Vec<&str> { vec!["/plan-mode"] }
    fn description(&self) -> &str { "E8 Plan Mode: /plan create <goal> | /plan status | /plan step [index] | /plan list | /plan complete" }
    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("E8 Plan Mode:\n  /plan create <goal>    创建新计划\n  /plan status           查看当前计划状态\n  /plan step [index]     执行/查看计划步骤\n  /plan list             列出所有计划\n  /plan complete          标记计划完成");
        }
        match args[0].as_str() {
            "create" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /plan create <goal>");
                }
                let goal = args[1..].join(" ");
                let gen = PlanGenerator::new();
                let plan = gen.generate_plan(&goal, &[]);
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                let summary = format!(
                    "📋 计划已创建 [{}]\n  目标: {}\n  步骤: {}\n  E8序列: {:?}\n  平均PRM: {:.3}\n  创建时间: {}",
                    &plan.id[..8], plan.goal, plan.metrics.total_steps, plan.e8_sequence, plan.metrics.avg_prm_score, now
                );
                let mut mgr = plan_manager().lock().unwrap_or_else(|e| e.into_inner());
                mgr.add_plan(plan);
                CommandOutput::ok(&summary)
            }
            "status" => {
                let mgr = plan_manager().lock().unwrap_or_else(|e| e.into_inner());
                match mgr.active_plan() {
                    Some(plan) => {
                        let mut out = format!(
                            "📋 活跃计划 [{}]\n  目标: {}\n  进度: {:.0}% ({}/{})\n  平均PRM: {:.3}",
                            &plan.id[..8], plan.goal,
                            plan.completion_pct() * 100.0,
                            plan.metrics.completed_steps, plan.metrics.total_steps,
                            plan.metrics.avg_prm_score
                        );
                        for (i, step) in plan.steps.iter().enumerate() {
                            let status_char = match step.status {
                                StepStatus::Completed => "✅",
                                StepStatus::InProgress => "🔄",
                                StepStatus::Failed(_) => "❌",
                                StepStatus::Skipped => "⏭️",
                                StepStatus::Pending => "⬜",
                            };
                            out.push_str(&format!("\n  {}. [E8:{}] {} {} (score: {:.2})", i + 1, step.e8_mode, step.action, status_char, step.prm_score));
                        }
                        CommandOutput::ok(&out)
                    }
                    None => CommandOutput::ok("当前无活跃计划。使用 /plan create <goal> 创建。")
                }
            }
            "step" => {
                let mgr = plan_manager().lock().unwrap_or_else(|e| e.into_inner());
                match mgr.active_plan() {
                    Some(plan) => {
                        if plan.is_complete() {
                            return CommandOutput::ok("计划已完成。创建新计划或使用 /plan list 查看。");
                        }
                        let next = plan.next_pending();
                        match next {
                            Some(step) => {
                                let idx = step.index;
                                CommandOutput::ok(&format!(
                                    "下一步: 步骤 {} [E8:{}] {}\n  预期: {}\n  评分: {:.2}\n  执行后使用 /plan complete 标记完成。",
                                    idx + 1, step.e8_mode, step.action, step.expected_outcome, step.prm_score
                                ))
                            }
                            None => CommandOutput::ok("所有步骤已完成。使用 /plan complete 结束计划。")
                        }
                    }
                    None => CommandOutput::ok("无活跃计划。使用 /plan create <goal> 创建。")
                }
            }
            "list" => {
                let mgr = plan_manager().lock().unwrap_or_else(|e| e.into_inner());
                if mgr.plans.is_empty() {
                    return CommandOutput::ok("尚无计划。使用 /plan create <goal> 创建。");
                }
                let mut out = format!("计划列表 (共{}个):\n", mgr.plans.len());
                for plan in &mgr.plans {
                    let is_active = mgr.active_id.as_ref() == Some(&plan.id);
                    let active_mark = if is_active { " ← 活跃" } else { "" };
                    out.push_str(&format!(
                        "  [{}] {} — {:.0}% ({}/{}){}{}",
                        &plan.id[..8], plan.goal,
                        plan.completion_pct() * 100.0,
                        plan.metrics.completed_steps, plan.metrics.total_steps,
                        active_mark, if plan.is_complete() { " ✅" } else { "" }
                    ));
                    out.push('\n');
                }
                CommandOutput::ok(&out)
            }
            "complete" => {
                let mut mgr = plan_manager().lock().unwrap_or_else(|e| e.into_inner());
                if mgr.complete_active() {
                    CommandOutput::ok("计划已标记完成。")
                } else {
                    CommandOutput::ok("无活跃计划可完成。")
                }
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: create, status, step, list, complete", args[0])),
        }
    }
}
