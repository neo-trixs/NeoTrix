//! 元认知校准权重 —— **全仓唯一真身**。
//!
//! ## 为什么单独一个文件（而不是塞进某个实现模块）
//! 这三个常量此前在**两个不同的层**各存一份，且靠注释提醒人工同步：
//!
//! | 位置 | 层 | 编译条件 |
//! |---|---|---|
//! | `neotrix-core/src/l5_cognition/nt_core_consciousness_tree/ops.rs` | L5 | 总编译 |
//! | `neotrix-core/src/l0_substrate/ffi/consciousness_tree.rs` | L0 | `#[cfg(feature = "ios-bridge")]` |
//!
//! 两边都要用，但 **L0/ffi 那份默认构建根本不编译** ⇒ 真身**不能**放在 L0/ffi，
//! 也不能放在 L5（L0 不能反向依赖 L5）。唯一同时满足两个约束的位置是
//! **契约层 `neotrix-types`**：core 依赖它，且它不受任何 feature 门控。
//!
//! ## 漂移风险是真实的，不是理论上的
//! l5 侧的旧注释写着「(与 ffi/consciousness_tree.rs 一致)」—— 即**靠人盯住**两个
//! 魔数相等。任何一侧调参而另一侧漏改，就是一处静默的行为分叉。
//!
//! ## ⚠️ 权重同源，但**公式不同**，且这是有意的
//! 收敛的是**权重**，不是公式。两条路径的惩罚公式并不相同：
//!
//! * L0 `apply_metacognitive_calibration`：
//!   `penalty = w_ece*ece + w_hce*hce`，输入含 `ece` 与 `high_conf_error_rate`。
//! * L5 `ops.rs`：`penalty = 只 w_ece*ece` —— 该路径的 ece 由
//!   `metacalib::expected_calibration_error` 从 SelfTest 样本算出，
//!   **没有 hce 指标可取**，故省略该项。
//!
//! ⇒ 旧注释里的「一致」是**错的**。权重取同一真身，公式差异**保留**并在
//! 使用处显式说明。不要为了「对齐」给 L5 补 hce 依赖。

/// ECE（期望校准误差）惩罚权重。
pub const CALIB_W_ECE: f32 = 0.6;

/// 高置信错误率（HCE）惩罚权重。**仅 L0 FFI 路径使用**（它才有 hce 指标）。
pub const CALIB_W_HCE: f32 = 0.4;

/// 惩罚上限（`penalty = min(…, CALIB_MAX_PENALTY)`）。
pub const CALIB_MAX_PENALTY: f32 = 0.35;