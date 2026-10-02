//! Spin(11,3) 64-fermion decomposition: `FermionState` and generators.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

// ─── Spin(11,3) 64-Fermion Decomposition ────────────────────────────

/// `FermionState` 及其两个生成函数收敛到低层唯一实现（2026-10-02）。
///
/// 与同批的 `Hexagram` 同因：`62cb175f` 记下的阻塞是
/// 「返回类型在两 crate 各定义一份 ⇒ 不可收敛」⇒ **阻塞根因在类型**。
///
/// 收敛前的核实（**这一笔差点出事，见下**）：
/// · 字段逐字相同、方法集相同（1 个，无缺失）；
/// · 归一化（去注释 + 归空白）后 `all_sm_fermions` **0 差异**；
/// · ⛔ 但 `fermion_states_for_generation` **原本并不一致**：
///   types 的兜底臂是 `unreachable!()`（**panic**），core 是 `("unknown",0,0,0)`。
///   而 `all_sm_fermions` **调用各自那份 helper** ⇒ 若直接转出，
///   就会把 core 的活路径接到 **types 的 panic 实现**上。
/// ⇒ 故先在 types 侧把两处 `unreachable!()` 降级为优雅兜底，
///   并证明这两臂**可证明不可达**（`color_bits` 恒 0..7、`weak_bits` 恒 0..3，
///   且臂齐全）⇒ 该改动**行为等价**，同时消除生产 panic
///   （AGENTS.md 明禁生产 `panic!`）。
pub use neotrix_types::core::nt_core_e8::FermionState;

/// 单一代的 64 个费米子态 —— 转出低层唯一实现。
pub use neotrix_types::core::nt_core_e8::fermion_states_for_generation;

/// 三代共 192 个标准模型费米子态 —— 转出低层唯一实现。
pub use neotrix_types::core::nt_core_e8::all_sm_fermions;

// 单点真身收敛（2026-09-30）：此前本文件与 `neotrix-types/core/nt_core_e8.rs`
// 各有一份**逐字相同**的 `verify_total_fermions`（`nt_diverge.py` 实测 owner 同为
// `(free)`、归一化后全等）⇒ 收敛为引用低层唯一实现。
// 判定依据：返回值是 `bool`，**跨 crate 同一类型** ⇒ 可安全收敛。
//（`all_sm_fermions` 曾因此被判「不可收敛」，因返回类型 `FermionState`
//  在两 crate 各自定义。2026-10-02 类型收敛后**该阻塞已解除**，
//  `all_sm_fermions` / `fermion_states_for_generation` 一并转出，
//  故本条括号注释**已于同日更新** —— 留着旧断言会让下一个 agent
//  以为「仍不可收敛」而重复当年的绕行。）
// 必要性：逐字相同的副本只会各自漂移（本会话已实测 `now_ts` 13 份里 2 份 panic）。
pub use neotrix_types::core::nt_core_e8::verify_total_fermions;
