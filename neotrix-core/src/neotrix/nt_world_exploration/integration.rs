//! # 探索-好奇心桥接
//!
//! 连接 CuriosityDrive → ExplorationOrchestrator:
//! 1. 好奇心信号 → 搜索词 → API/Search 源入队
//! 2. 好奇心水平 → 决定探索调度优先级
//! 3. 吸收统计 → 反馈到好奇心校准

use super::content::NegentropyScore;
use super::orchestrator::ExplorationOrchestrator;
use crate::neotrix::nt_mind::curiosity_drive::{CuriosityDrive, CuriosityLevel};

/// 将好奇心信号注入探索编排器的种子队列
pub fn seed_orchestrator_from_curiosity(
    drive: &CuriosityDrive,
    orchestrator: &mut ExplorationOrchestrator,
    max_signals: usize,
) -> usize {
    let mut seeded = 0;
    for signal in drive.top_signals(max_signals) {
        for term in &signal.suggested_search_terms {
            orchestrator.api.enqueue(term.clone());
            orchestrator.search.search(term.clone());
            seeded += 1;
        }
    }
    seeded
}

/// 根据好奇心水平决定探索节奏
pub fn exploration_urgency(curiosity: CuriosityLevel) -> f64 {
    match curiosity {
        CuriosityLevel::Calm => 0.05,
        CuriosityLevel::Interested => 0.2,
        CuriosityLevel::Curious => 0.5,
        CuriosityLevel::IntenselyCurious => 0.85,
    }
}

/// 一轮完整的探索-好奇心循环:
/// 1. 好奇心信号 → 种子查询
/// 2. 执行探索 (按好奇心水平调度)
/// 3. 返回吸收结果 (用于反馈给好奇心校准)
pub fn exploration_curiosity_cycle(
    drive: &CuriosityDrive,
    orchestrator: &mut ExplorationOrchestrator,
) -> Vec<NegentropyScore> {
    let urgency = exploration_urgency(drive.curiosity_level);
    if urgency < 0.15 {
        return Vec::new();
    }

    // 种子
    seed_orchestrator_from_curiosity(drive, orchestrator, 5);

    // 执行探索
    match orchestrator.explore_cycle(urgency) {
        Ok(scores) => scores,
        Err(_) => Vec::new(),
    }
}
