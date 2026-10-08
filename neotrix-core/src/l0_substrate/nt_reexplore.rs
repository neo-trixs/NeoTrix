//! `nt_reexplore` — 并行多 agent 调度的重探索成本计量（SquidAgent 2608.08647 吸收）。
//!
//! SquidAgent 的核心观察：并行 worker 往往浪费 token 重建
//! orchestrator 已经持有的上下文（先例：prior decisions、workspace 扫描）。
//! 本模块给出该成本的**纯函数定义**（measurement contract）：
//! 等将来真实的并行 spawn 实现接入时，按同一口径计数。
//!
//! 纯逻辑、零 I/O、零时钟；统计口径固定为：
//! - `orchestrator_tokens`：调度方/主 agent 持有的上下文体积（基线）
//! - 每个 worker 的 `w.context_tokens`：该 worker 自己重建的上下文体积
//! - **re-exploration 浪费 = Σ max(w.context_tokens − 共享基线, 0)**
//!   即：worker 重建得越多、越超出主 agent 基线，浪费越高。
//! - 0 浪费路径：全体 worker 只拿 orchestrator 已经准备好的上下文（继承）。

/// 单个 worker 的上下文用量记录。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerContext {
    /// 该 worker 为恢复上下文而消费的 token 数（估算值）。
    pub context_tokens: u64,
}

/// 一次并行调度的计量快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReexploreReport {
    /// 调度方基线（tokens）。
    pub orchestrator_tokens: u64,
    /// worker 总数。
    pub worker_count: usize,
    /// 重探索浪费总量（tokens）。
    pub wasted_tokens: u64,
    /// 浪费 / Σ worker 上下文 的比例（milli 分之一，0..=1000）。
    pub waste_ratio_millis: u64,
}

/// 计算重探索成本。
#[must_use]
pub fn measure(orchestrator_tokens: u64, workers: &[WorkerContext]) -> ReexploreReport {
    let mut total_worker = 0u64;
    let mut wasted = 0u64;
    for w in workers {
        total_worker = total_worker.saturating_add(w.context_tokens);
        wasted = wasted.saturating_add(w.context_tokens.saturating_sub(orchestrator_tokens));
    }
    let waste_ratio_millis = if total_worker == 0 {
        0
    } else {
        wasted.saturating_mul(1000) / total_worker
    };
    ReexploreReport {
        orchestrator_tokens,
        worker_count: workers.len(),
        wasted_tokens: wasted,
        waste_ratio_millis,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_waste_when_workers_inherit_orchestrator_context() {
        let workers = vec![
            WorkerContext { context_tokens: 100 },
            WorkerContext { context_tokens: 200 },
        ];
        let r = measure(500, &workers);
        assert_eq!(r.wasted_tokens, 0);
        assert_eq!(r.waste_ratio_millis, 0);
        assert_eq!(r.worker_count, 2);
    }

    #[test]
    fn waste_counts_excess_over_orchestrator_baseline() {
        let workers = vec![
            WorkerContext { context_tokens: 900 },
            WorkerContext { context_tokens: 300 },
        ];
        let r = measure(500, &workers);
        assert_eq!(r.wasted_tokens, 400); // (900-500) + max(300-500,0)
        assert_eq!(r.waste_ratio_millis, 400 * 1000 / 1200);
    }

    #[test]
    fn empty_workers_is_zero_and_report_is_serializable_shape() {
        let r = measure(1000, &[]);
        assert_eq!(r.wasted_tokens, 0);
        assert_eq!(r.worker_count, 0);
        assert_eq!(r.waste_ratio_millis, 0);
    }
}
