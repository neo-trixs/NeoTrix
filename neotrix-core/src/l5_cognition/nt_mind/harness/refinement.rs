//! Continual Harness Refinement — 基于 PrimeAgent 模式
//!
//! `/refine` 命令：审查轨迹，应用小的有证据支持的状态更新。快照回滚。

#![allow(dead_code)]

use std::collections::HashMap;

/// 快照（用于回滚）
#[derive(Debug, Clone)]
pub struct HarnessSnapshot {
    pub snapshot_id: String,
    pub timestamp: i64,
    pub state: HashMap<String, String>,
    pub description: String,
}

/// 精炼更新
#[derive(Debug, Clone)]
pub struct RefinementUpdate {
    pub update_id: String,
    pub target: String, // 要更新的配置/提示词/规则
    pub before: String,
    pub after: String,
    pub evidence: String, // 支持更新的证据
    pub confidence: f64,  // 0.0-1.0
}

/// 精炼结果
#[derive(Debug, Clone)]
pub struct RefinementResult {
    pub snapshot_id: String,
    pub updates_applied: Vec<RefinementUpdate>,
    pub updates_rejected: Vec<(RefinementUpdate, String)>,
    pub rolled_back: bool,
}

/// Continual Harness 精炼器
#[derive(Debug, Clone)]
pub struct ContinualRefiner {
    snapshots: Vec<HarnessSnapshot>,
    updates: Vec<RefinementUpdate>,
    current_state: HashMap<String, String>,
    next_snapshot_id: u32,
}

impl ContinualRefiner {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            updates: Vec::new(),
            current_state: HashMap::new(),
            next_snapshot_id: 0,
        }
    }

    /// 创建快照
    pub fn snapshot(&mut self, description: &str) -> String {
        self.next_snapshot_id += 1;
        let id = format!("snap_{}", self.next_snapshot_id);

        self.snapshots.push(HarnessSnapshot {
            snapshot_id: id.clone(),
            timestamp: now_ts(),
            state: self.current_state.clone(),
            description: description.to_string(),
        });

        id
    }

    /// 应用精炼更新
    pub fn refine(
        &mut self,
        updates: Vec<RefinementUpdate>,
        min_confidence: f64,
    ) -> RefinementResult {
        let snapshot_id = self.snapshot("refine_start");
        let mut applied = Vec::new();
        let mut rejected = Vec::new();

        for update in updates {
            if update.confidence >= min_confidence {
                self.current_state
                    .insert(update.target.clone(), update.after.clone());
                self.updates.push(update.clone());
                applied.push(update);
            } else {
                rejected.push((
                    update.clone(),
                    format!("置信度 {:.2} < {:.2}", update.confidence, min_confidence),
                ));
            }
        }

        RefinementResult {
            snapshot_id,
            updates_applied: applied,
            updates_rejected: rejected,
            rolled_back: false,
        }
    }

    /// 回滚到快照
    pub fn rollback(&mut self, snapshot_id: &str) -> bool {
        if let Some(snapshot) = self.snapshots.iter().find(|s| s.snapshot_id == snapshot_id) {
            self.current_state = snapshot.state.clone();
            true
        } else {
            false
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> &HashMap<String, String> {
        &self.current_state
    }

    /// 获取快照列表
    pub fn snapshots(&self) -> &Vec<HarnessSnapshot> {
        &self.snapshots
    }

    /// 获取统计
    pub fn stats(&self) -> (usize, usize, usize) {
        let total_updates = self.updates.len();
        let snapshots = self.snapshots.len();
        let state_items = self.current_state.len();
        (total_updates, snapshots, state_items)
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
