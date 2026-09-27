//! `nt_stale_guard` — stale running 防御性复位（只增不改）.
//!
//! 背景（2026-09-27 D lane 定点）：
//! - 前端 hero `运行中 · X` 取 `tasks.find(status == "running")` 首条；
//!   后端 `running` 残留（崩溃/杀进程落在 Running save 与终态 save 之间，
//!   租约过期无人扫）→ 前端 15s 轮询只读 `list_tasks`，永不触发回收 → 卡死。
//! - `recover_stale_running` 只在 `run_local_turn_inner` 起点调；
//!   读路径（`neobot_tasks` / `neobot_convos`）从不调。
//! - routine 侧无 steal 逻辑（grep 全仓无 `steal`），`MIN_INTERVAL` /
//!   `FATIGUE_LIMIT` 只管注册地板与连败自停，与本次卡死无关。
//!
//! 本模块只做加法：纯判定 + best-effort 复位，不改任何已有签名/调度语义。
//! 接入（主线程做，D lane 不动已有文件行为）：
//! - 跑轮起点已覆盖（`nt_agent` 内）；读路径如需自愈，在 `neobot_tasks`
//!   内首行加 `let _ = recover_stale_best_effort(&store, &now);` 即可。

use crate::nt_store::{CLAIM_TTL_SECS, NeobotStore};

/// stale 判定上限秒（与 `nt_agent::LEASE_SECS` 同值同义；单轮最长 10 分钟）。
/// 改租约先改 `nt_agent::LEASE_SECS`，此处跟随（刻意不用跨模块常量，
/// 避免调度语义被展示层反向牵引）。
pub const STALE_RUNNING_SECS: i64 = 600;

/// 纯判定：running 任务是否已 stale（展示层/doctor 用，无 DB 写）。
/// - `lease_until` 为空 → stale（老数据/崩溃残留：无租约即无心跳）；
/// - 任一时间解析失败 → stale（fail-visible，不让脏时间卡死展示）；
/// - 否则租约早于 now 即 stale。
pub fn is_stale_running(lease_until: Option<&str>, now_rfc3339: &str) -> bool {
    let Some(lease) = lease_until else {
        return true;
    };
    let (Ok(lease_ts), Ok(now_ts)) = (
        chrono::DateTime::parse_from_rfc3339(lease),
        chrono::DateTime::parse_from_rfc3339(now_rfc3339),
    ) else {
        return true;
    };
    lease_ts < now_ts
}

/// 防御性复位（best-effort）：stale running → pending + 过期认领释放。
/// 返回 `(recovered, swept)`；任一步失败记 0，不抛错（调用方绝不因此挡路）。
pub fn recover_stale_best_effort(store: &NeobotStore, now_rfc3339: &str) -> (usize, usize) {
    let recovered: usize = store
        .recover_stale_running(now_rfc3339)
        .unwrap_or(0);
    let swept: usize = store
        .sweep_stale_claims(now_rfc3339, CLAIM_TTL_SECS)
        .unwrap_or(0);
    (recovered, swept)
}

/// 列出 stale running 任务 id（doctor/巡检用；只读，不写库）。
/// 读失败回空（调用方按“无异常”处理，不炸展示）。
pub fn stale_running_ids(store: &NeobotStore, now_rfc3339: &str) -> Vec<String> {
    let tasks: Vec<crate::nt_types::AgentTask> = store.list_tasks(200).unwrap_or_default();
    tasks
        .into_iter()
        .filter(|t| t.status == crate::nt_types::TaskStatus::Running)
        .filter(|t| is_stale_running(t.lease_until.as_deref(), now_rfc3339))
        .map(|t| t.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::is_stale_running;

    #[test]
    fn stale_judgement() {
        let now = "2026-09-27T01:00:00+00:00";
        // 无租约 → stale
        assert!(is_stale_running(None, now));
        // 脏时间 → stale（fail-visible）
        assert!(is_stale_running(Some("not-a-time"), now));
        assert!(is_stale_running(Some("2026-09-27T00:00:00+00:00"), "bad-now"));
        // 过期租约 → stale；未来租约 → 不 stale
        assert!(is_stale_running(Some("2026-09-27T00:00:00+00:00"), now));
        assert!(!is_stale_running(Some("2026-09-27T02:00:00+00:00"), now));
    }
}
