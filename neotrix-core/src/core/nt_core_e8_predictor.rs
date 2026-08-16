//! E8 预测器模块
//!
//! 提供基于 E8 状态序列的在线学习预测器，用于意识核心的六阶段闭环跟踪。
//! 实现 `load`/`persist` 接口，供 `handlers_consciousness.rs` 与 `nt_core_task_dispatcher.rs` 调用。
//! 状态经 KB kv_store (namespace=`e8_predictor`) 持久化, 跨进程/重启保留累积样本。

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 核心结构体
// ---------------------------------------------------------------------------

/// E8 预测器实体
/// - 跨周期累积观测样本 (The Spice Must Flow)
/// - 提供 observe_trace / sample_count / coverage 用于闭环反馈
/// - 提供 predict_next 用于任务分发决策 (高置信本地执行 / 低置信分发 LLM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8Predictor {
    /// 观测的状态序列轨迹
    pub state_traces: Vec<Vec<u8>>,
    /// 样本计数
    pub sample_count: usize,
    /// 覆盖度指标 (0.0 ~ 1.0)
    pub coverage: f64,
    /// 64 态马尔可夫转移计数矩阵 [from][to] (E8 状态子空间)
    /// 使用嵌套 Vec (而非 [[u64;64];64]) 以支持 serde 序列化 (KB 持久化)。
    pub transition_counts: Vec<Vec<u64>>,
}

impl Default for E8Predictor {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Predictor {
    /// 创建新的 E8 预测器 (内部使用，load/persist 接口由外部调用)
    pub fn new() -> Self {
        Self {
            state_traces: Vec::new(),
            sample_count: 0,
            coverage: 0.0,
            transition_counts: vec![vec![0u64; 64]; 64],
        }
    }

    /// 记录一条状态轨迹观测 (对应 handlers_consciousness.rs 中的 observe_trace)
    ///
    /// 同时累积转移计数: 轨迹内相邻状态对 (from,to) 记入转移矩阵,
    /// 使预测器随观测增长获得可用的预测能力 (The Spice Must Flow)。
    pub fn observe_trace(&mut self, trace: &[u8]) {
        for pair in trace.windows(2) {
            let from = (pair[0] & 0x3f) as usize;
            let to = (pair[1] & 0x3f) as usize;
            self.transition_counts[from][to] += 1;
        }
        self.state_traces.push(trace.to_vec());
        self.sample_count += 1;
        // 根据累积样本重新计算覆盖度 (观测态数 / 样本数, 平滑防除零)
        let unique = self
            .state_traces
            .iter()
            .collect::<std::collections::HashSet<&Vec<u8>>>()
            .len();
        self.coverage = (unique as f64) / (self.sample_count as f64).max(1.0);
    }

    /// 预测从给定 E8 状态出发的最可能下一状态。
    ///
    /// 返回 `(next_state, confidence)`：
    /// - `next_state`: 转移计数最高的后继态 (无观测时返回当前态)
    /// - `confidence`: 该转移概率 (0.0 ~ 1.0)；样本不足时保守压低
    ///
    /// 供任务调度器做分发决策: 高置信 → 本地执行, 低置信 → 分发 LLM。
    pub fn predict_next(&self, current: u8) -> (u8, f64) {
        let idx = (current & 0x3f) as usize;
        let row = &self.transition_counts[idx];
        let total: u64 = row.iter().sum();
        if total == 0 {
            return (current, 0.0);
        }
        let mut best = 0usize;
        let mut best_count = 0u64;
        for (to, &count) in row.iter().enumerate() {
            if count > best_count {
                best_count = count;
                best = to;
            }
        }
        let mut conf = (best_count as f64) / (total as f64);
        // 样本不足时保守压低置信度, 避免过早信任稀疏转移
        if self.sample_count < 4 {
            conf *= self.sample_count as f64 / 4.0;
        }
        (best as u8, conf.clamp(0.0, 1.0))
    }

    /// 当前累积样本数 (对齐 handlers_consciousness.rs 的方法调用)
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// 当前状态覆盖度 (0.0 ~ 1.0, 对齐 handlers_consciousness.rs 的方法调用)
    pub fn coverage(&self) -> f64 {
        self.coverage
    }
}

// ---------------------------------------------------------------------------
// load / persist 接口 (KB 持久化)
// ---------------------------------------------------------------------------

/// E8 预测器在 KB kv_store 中的 namespace 与 key。
const KB_NAMESPACE: &str = "e8_predictor";
const KB_KEY: &str = "core";

/// 测试注入的 KB 路径 (进程内静态, 线程安全) — 替代 env 覆盖,
/// 消除 `set_var` 在并行测试间的全局 env 竞争 (Rust env 多线程不安全)。
/// 生产路径恒为 None → 解析回落 env/默认, 零开销。
static TEST_KB_PATH: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

/// 打开 KB 连接 (默认 `~/.neotrix/knowledge.db`), 复用 NT-MEMORY 统一 schema 初始化。
/// 单一 schema 事实源: 不在此处维护 kv_store 本地 DDL, 避免漂移 (对齐 consciousness_core)。
///
/// 路径解析优先级:
/// 1. 测试注入静态 `TEST_KB_PATH` (线程安全, 无 env 竞争)
/// 2. `NEOTRIX_KB_PATH` env (生产/外部覆盖)
/// 3. `~/.neotrix/knowledge.db` 默认
fn open_kb() -> Result<rusqlite::Connection, String> {
    let path = TEST_KB_PATH
        .lock()
        .map(|g| g.clone())
        .ok()
        .flatten()
        .or_else(|| {
            std::env::var("NEOTRIX_KB_PATH")
                .map(std::path::PathBuf::from)
                .ok()
        })
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home)
                .join(".neotrix")
                .join("knowledge.db")
        });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("KB dir: {}", e))?;
    }
    let conn = rusqlite::Connection::open(&path).map_err(|e| format!("KB open: {}", e))?;
    crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema::initialize(&conn)
        .map_err(|e| format!("KB init: {}", e))?;
    Ok(conn)
}

/// 从持久化存储加载 E8 预测器实例
///
/// 对应 `handlers_consciousness.rs` 线 356-358 的调用：
/// `use crate::core::nt_core_e8_predictor::{load as predictor_load, persist as predictor_persist};`
/// `let mut predictor = predictor_load();`
///
/// 行为：尝试从 KB kv_store (namespace=`e8_predictor`) 读取先前状态；
/// 无记录/损坏时返回空默认实例 (优雅降级, 不 panic)。
/// 确保跨周期累积 (The Spice Must Flow)，防止预测器成为孤儿模块 (Dark Forest)。
pub fn load() -> E8Predictor {
    let conn = match open_kb() {
        Ok(c) => c,
        Err(_) => return E8Predictor::new(),
    };
    let raw = match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::kv_get(
        &conn,
        KB_NAMESPACE,
        KB_KEY,
    ) {
        Ok(Some(v)) => v,
        _ => return E8Predictor::new(),
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| E8Predictor::new())
}

/// 将预测器状态持久化到存储
///
/// 对应 `handlers_consciousness.rs` 线 388 的调用：
/// `let _ = predictor_persist(&predictor);`
///
/// 参数：`&predictor` - 要持久化的预测器实例引用
/// 行为：将当前预测器的状态轨迹和转移矩阵写入 KB kv_store,
/// 保证跨进程/重启保留累积样本 (The Spice Must Flow)。
pub fn persist(predictor: &E8Predictor) {
    let conn = match open_kb() {
        Ok(c) => c,
        Err(_) => return,
    };
    let json = match serde_json::to_string(predictor) {
        Ok(j) => j,
        Err(_) => return,
    };
    let _ = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::kv_set(
        &conn,
        KB_NAMESPACE,
        KB_KEY,
        &json,
    );
}

// ---------------------------------------------------------------------------
// 语法修正：u8::from(i) → i as u8
// ---------------------------------------------------------------------------

/// 将整数安全转换为 u8
///
/// 修复潜在的 `u8::from(i)` 语法错误，改用 `as u8` 语法，
/// 符合 NeoTrix R-P1 零 unsafe 规范。
pub fn safe_u8(value: i32) -> u8 {
    // as u8 语法，显式且安全（值范围由调用者保证）
    value as u8
}

// ---------------------------------------------------------------------------
// 模块导出
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试隔离: 为每个调用分配唯一临时 KB 路径,
    /// 避免污染生产 KB 且测试间状态互不干扰。
    /// 通过进程内静态注入 (非 env) — 并行测试间无 env 竞争。
    fn isolate_home() {
        static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tmp = std::env::temp_dir().join(format!("neotrix-e8p-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&tmp).ok();
        *TEST_KB_PATH.lock().unwrap() = Some(tmp.join("knowledge.db"));
    }

    /// 串行化所有触碰隔离 DB 的测试: 共享同库下并行写会互相覆盖基线,
    /// 使 roundtrip 断言不可判定。用锁保证一次仅一个测试持有 DB。
    /// 抗中毒: 前一测试 panic 时锁标记中毒, 用 into_inner 恢复继续。
    fn with_kb_lock<T>(f: impl FnOnce() -> T) -> T {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let guard = match LOCK.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(), // 前一测试 panic 污染: 恢复锁
        };
        let _guard = guard;
        f()
    }

    #[test]
    fn test_load_creates_instance() {
        with_kb_lock(|| {
            isolate_home();
            let p = load();
            assert!(matches!(
                p,
                E8Predictor {
                    sample_count: 0,
                    ..
                }
            ));
        });
    }

    #[test]
    fn test_observe_trace() {
        with_kb_lock(|| {
            isolate_home();
            let mut p = load();
            let trace = vec![1u8, 2, 3];
            p.observe_trace(&trace);
            assert_eq!(p.sample_count, 1);
            assert_eq!(p.state_traces.len(), 1);
        });
    }

    #[test]
    fn test_persist_no_panic() {
        with_kb_lock(|| {
            isolate_home();
            let p = load();
            persist(&p); // 不应 panic
        });
    }

    #[test]
    fn test_kb_roundtrip_preserves_samples() {
        with_kb_lock(|| {
            isolate_home();
            // 累积 4 条样本后持久化
            let mut p = load();
            p.observe_trace(&[1, 2]);
            p.observe_trace(&[1, 2]);
            p.observe_trace(&[1, 2]);
            p.observe_trace(&[1, 2]);
            persist(&p);

            // 新进程语义: 重新 load 应恢复累积样本 (跨进程保留)
            let reloaded = load();
            assert_eq!(reloaded.sample_count, 4);
            assert_eq!(reloaded.state_traces.len(), 4);
            // 预测能力也保留
            let (next, conf) = reloaded.predict_next(1);
            assert_eq!(next, 2);
            assert!(conf > 0.9);
        });
    }

    #[test]
    fn test_predict_next_after_observations() {
        with_kb_lock(|| {
            isolate_home();
            let mut p = load();
        // 建立一致转移: 1 → 2 出现 4 次 (>= 保守阈值, 达满置信)
        p.observe_trace(&[1, 2]);
        p.observe_trace(&[1, 2]);
        p.observe_trace(&[1, 2]);
        p.observe_trace(&[1, 2]);
        let (next, conf) = p.predict_next(1);
        assert_eq!(next, 2);
        assert!(conf > 0.9);
        });
    }

    #[test]
    fn test_predict_next_sparse_conservative() {
        with_kb_lock(|| {
            isolate_home();
            let mut p = load();
        // 仅 1 次观测: 置信度被保守压低 (< 1.0)
        p.observe_trace(&[1, 2]);
        let (next, conf) = p.predict_next(1);
        assert_eq!(next, 2);
        assert!(conf < 0.5);
        assert!(conf > 0.0);
        });
    }

    #[test]
    fn test_predict_next_no_data() {
        isolate_home();
        let p = load();
        let (next, conf) = p.predict_next(5);
        assert_eq!(next, 5); // 无观测时返回当前态
        assert_eq!(conf, 0.0); // 零置信
    }

    #[test]
    fn test_safe_u8() {
        let result = safe_u8(42);
        assert_eq!(result, 42);
    }
}
