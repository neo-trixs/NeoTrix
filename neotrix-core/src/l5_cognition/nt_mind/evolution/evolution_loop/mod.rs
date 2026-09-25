//! 自进化循环引擎 — 持续迭代: 扫描 → 分析 → 修复 → 蒸馏
//!
//! 设计: 每个周期执行:
//!   1. 项目扫描 (代码度量, 文件大小, 测试覆盖率, 编译状态)
//!   2. 瓶颈分析 (慢模块, 循环依赖, unwrap 热点)
//!   3. Bug 检测 (编译警告, 测试失败, unsafe 使用)
//!   4. 自修复生成 (修复警告, 补全导入, 处理 unwrap)
//!   5. 模式蒸馏 (提取行为规则, 更新 AGENTS.md)
//!   6. 报告输出 (状态仪表盘)
//!
//! 融合 AGENTS.md 元认知自检 + MetaCognitive Self-Check 协议
//!
//! 单文件转目录门面 (God-file 拆分, 行为零变更; 原 `evolution_loop.rs` 1711 行):
//! - `nt_loop_types` — 常量 / Issue / EvolutionReport / Auditor / EvolutionLoop 类型根
//! - `nt_loop_evaluate` — 评估: 扫描 + 度量 + 健康评分
//! - `nt_loop_select` — 选择: 问题检测器 + 人工介入判断
//! - `nt_loop_mutate` — 变异: 自动修复周期
//! - `nt_loop_core` — 主循环: run_cycle + 仪表盘 + 诊断入口 + Provider 适配
//! - `nt_loop_tests` — 单测 (#[cfg(test)] 门控)
//! 外部 `evolution_loop::X` 路径经 `pub use` 保持不变。

pub mod nt_loop_core;
pub mod nt_loop_evaluate;
pub mod nt_loop_mutate;
pub mod nt_loop_select;
pub mod nt_loop_types;
pub mod nt_loop_tests;

// core/evaluate/select/mutate 仅含 `impl EvolutionLoop` 扩展块, 无新增公开条目,
// 随 `pub mod` 编译即生效, 无需重导出; 公开类型面 (= 原单文件全部 pub 项)
// 集中于 nt_loop_types, 此处一次性重导出以保持外部 `evolution_loop::X` 路径不变。
pub use nt_loop_types::*;
