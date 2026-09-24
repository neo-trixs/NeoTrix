//! rst_flywheel — 从 `evolution_loop.rs` 拆分 (卫星件, 零生产引用, 行为零变更).

use serde::{Deserialize, Serialize};
/// RST 任务 — 递归合成的单元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RstTask {
    /// 任务 id。
    pub id: String,
    /// 任务描述。
    pub prompt: String,
    /// 合成代数 (种子 = 0)。
    pub generation: u32,
    /// 复杂度评分 (验证器用于对齐: 应单调不减)。
    pub complexity: f64,
    /// 是否已通过验证。
    pub verified: bool,
    /// 被复用的父任务 id (None = 种子)。
    pub parent: Option<String>,
}

/// RST 验证结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _RstVerdict {
    pub task_id: String,
    pub accepted: bool,
    /// 拒绝原因 (accepted=false 时)。
    pub reason: String,
}

/// RST 飞轮 — seed→extend→_realign→validate→reuse。
#[derive(Debug, Clone)]
pub struct _RstFlywheel {
    /// 已验证任务池 (reuse 源)。
    pub verified_pool: Vec<_RstTask>,
    /// 生成代数上限 (防止无界漂移)。
    pub max_generation: u32,
    /// 每代最大合成数。
    pub extend_per_gen: usize,
    /// 复杂度对齐阈值 — 新任务复杂度必须 ≥ 父任务 × 阈值。
    pub realign_threshold: f64,
    /// 复杂度上限 (超出即拒绝, 防发散)。
    pub complexity_cap: f64,
    /// 已拒绝计数。
    pub rejected_count: u64,
    /// 已接受计数。
    pub accepted_count: u64,
    /// A2 迭代预算 (autoresearch bounded-by-default): 单轮进化循环上限,
    /// 超出即停, 防止无界自进化消耗 (R-P38 retry_cap 语义)。
    pub iteration_budget: usize,
    /// A2 已消耗迭代数。
    pub iterations_used: usize,
}

impl Default for _RstFlywheel {
    fn default() -> Self {
        Self::new()
    }
}

impl _RstFlywheel {
    pub fn new() -> Self {
        Self {
            verified_pool: Vec::new(),
            max_generation: 4,
            extend_per_gen: 3,
            realign_threshold: 1.1,
            complexity_cap: 100.0,
            rejected_count: 0,
            accepted_count: 0,
            iteration_budget: 5,
            iterations_used: 0,
        }
    }

    /// 阶段 1 seed: 注入种子任务 (代数 0, 直接入池)。
    pub fn seed(&mut self, prompt: impl Into<String>) -> _RstTask {
        let task = _RstTask {
            id: format!("rst-{}", uuid::Uuid::new_v4()),
            prompt: prompt.into(),
            generation: 0,
            complexity: 1.0,
            verified: true,
            parent: None,
        };
        self.verified_pool.push(task.clone());
        self.accepted_count += 1;
        task
    }

    /// 阶段 2 extend: 从已验证池采样父任务, 合成新任务 (复杂度随代数放大)。
    pub fn extend(&self, parent: &_RstTask) -> Vec<_RstTask> {
        if parent.generation >= self.max_generation {
            return Vec::new();
        }
        let gen = parent.generation + 1;
        (0..self.extend_per_gen)
            .map(|i| _RstTask {
                id: format!("rst-{gen}-{i}-{}", uuid::Uuid::new_v4()),
                prompt: format!("extended[{}]: {}", parent.prompt, i),
                generation: gen,
                complexity: parent.complexity * 1.3,
                verified: false,
                parent: Some(parent.id.clone()),
            })
            .collect()
    }

    /// 阶段 3 _realign: 复杂度对齐 — 只保留复杂度 ∈ [父×阈值, cap] 的候选。
    /// 防生成器漂移 (分布外任务被淘汰)。
    pub(crate) fn _realign(&self, parent: &_RstTask, candidates: Vec<_RstTask>) -> Vec<_RstTask> {
        let floor = parent.complexity * self.realign_threshold;
        candidates
            .into_iter()
            .filter(|c| c.complexity >= floor && c.complexity <= self.complexity_cap)
            .collect()
    }

    /// 阶段 4 validate: 启发式验证器 — 任务必须可解 (提示非空) 且复杂度
    /// 在合理区间。通过 → 入池; 失败 → 记拒绝。
    pub fn validate(&mut self, candidates: Vec<_RstTask>) -> Vec<_RstTask> {
        let mut accepted = Vec::new();
        for c in candidates {
            let verdict = self.validate_one(&c);
            if verdict.accepted {
                accepted.push(c);
                self.accepted_count += 1;
            } else {
                self.rejected_count += 1;
            }
        }
        accepted
    }

    fn validate_one(&self, task: &_RstTask) -> _RstVerdict {
        if task.prompt.trim().is_empty() {
            return _RstVerdict {
                task_id: task.id.clone(),
                accepted: false,
                reason: "empty prompt".into(),
            };
        }
        if task.complexity <= 0.0 {
            return _RstVerdict {
                task_id: task.id.clone(),
                accepted: false,
                reason: "non-positive complexity".into(),
            };
        }
        if task.complexity > self.complexity_cap {
            return _RstVerdict {
                task_id: task.id.clone(),
                accepted: false,
                reason: "complexity exceeds cap".into(),
            };
        }
        _RstVerdict {
            task_id: task.id.clone(),
            accepted: true,
            reason: String::new(),
        }
    }

    /// P0-8 keep-or-revert (awesome-autoresearch): 单次单改动判定 —
    /// 新方案得分 > 旧方案 × keep_threshold → keep; 否则 revert (回滚到旧方案)。
    /// 防止连续改动导致的累计漂移。无基准 (old=0) → 视为 keep。
    pub(crate) fn _keep_or_revert(&self, old_score: f64, new_score: f64, keep_threshold: f64) -> bool {
        if old_score <= 0.0 {
            return true;
        }
        new_score > old_score * keep_threshold
    }

    /// P0-8 seed 梯度付费 (PI-blog): 按噪声估计选择验证 seed 数。
    /// 低噪声 → 1 seed 足够; 中噪声 → 3; 高噪声 → 8 (付费上限)。
    /// 返回 (seed 数, 是否升级到下一档)。
    pub(crate) fn _seed_escalation(&self, noise_estimate: f64, current_seeds: usize) -> (usize, bool) {
        let target = if noise_estimate < 0.15 {
            1
        } else if noise_estimate < 0.4 {
            3
        } else {
            8
        };
        let escalated = current_seeds < target;
        (target, escalated)
    }

    /// 阶段 5 reuse: 从已验证池采样可复用任务 (round-robin 策略, 确定性)。
    pub fn reuse(&self, offset: usize) -> Option<&_RstTask> {
        if self.verified_pool.is_empty() {
            return None;
        }
        let idx = offset % self.verified_pool.len();
        self.verified_pool.get(idx)
    }

    /// 飞轮完整循环: 从指定父任务 extend → _realign → validate → 入池。
    /// 返回新接受任务数。
    pub(crate) fn _run_generation(&mut self, parent: &_RstTask) -> usize {
        let candidates = self.extend(parent);
        if candidates.is_empty() {
            return 0;
        }
        let aligned = self._realign(parent, candidates);
        let accepted = self.validate(aligned);
        let n = accepted.len();
        self.verified_pool.extend(accepted);
        n
    }

    /// 统计: 池规模 / 各代数分布。
    pub fn stats(&self) -> (usize, Vec<usize>) {
        let max_gen = self
            .verified_pool
            .iter()
            .map(|t| t.generation)
            .max()
            .unwrap_or(0) as usize;
        let mut dist = vec![0usize; max_gen + 1];
        for t in &self.verified_pool {
            dist[t.generation as usize] += 1;
        }
        (self.verified_pool.len(), dist)
    }

    // ── A2 (autoresearch, uditgoenka/autoresearch): 有界自进化循环 ──
    // "bounded-by-default / one change per iteration / mechanical verification
    // only / git is memory / commit before verify"。注入 RST 飞轮作为生产
    // 纪律: ①迭代有上限 (R-P38 retry_cap) ②先 commit 再验证 (改进保留,
    // 变差 revert) ③单次单改动。

    /// A2 迭代预算是否耗尽 (bounded-by-default)。
    pub fn budget_exhausted(&self) -> bool {
        self.iterations_used >= self.iteration_budget
    }

    /// A2 有界飞轮循环: 每次消耗 1 迭代预算; 耗尽 → 停 (返回 None)。
    /// 无界漂移的强制上限, 防止自进化循环吞噬资源。
    pub(crate) fn _bounded_run_generation(&mut self, parent: &_RstTask) -> Option<usize> {
        if self.budget_exhausted() {
            return None;
        }
        self.iterations_used += 1;
        Some(self._run_generation(parent))
    }

    /// A2 commit-then-verify 判定 (autoresearch "先 commit 再验证"):
    /// 新分 > 旧分 × keep_threshold → keep (commit 保留); 否则 revert (回滚)。
    /// 与 P0-8 _keep_or_revert 同构, 但显式表达 "验证在 commit 之后" 的纪律,
    /// 并返回 (keep, 是否触发回滚) 双元供上层事件分发 (R-P25 行为接地)。
    pub(crate) fn _commit_then_verify(
        &self,
        old_score: f64,
        new_score: f64,
        keep_threshold: f64,
    ) -> (bool, bool) {
        let keep = self._keep_or_revert(old_score, new_score, keep_threshold);
        (keep, !keep)
    }
}
