# ADR-0002: Panic 债务治理（R-P3 落地，SQALE/SEI 技术债对齐）

**Status**: Accepted
**Date**: 2026-08-15
**Deciders**: NT-CORE / NT-MEMORY
**Technical Story**: 全面架构审查（R2）发现 2,213 处 `unwrap` + 1,167 处 `expect`；CI clippy `-D warnings` 门禁红灯（含 `unwrap_used` deny 模块）。

## Context

- **问题**：`#![deny(clippy::unwrap_used)]` 模块与 `-D warnings` 门禁下，已存在 ~100 处 clippy 违例 + 大量 panic 点。panic 债务高企导致：生产崩溃面不可控、不可恢复路径多、与"自愈/自适应"（C5）目标冲突。
- **可选方案**：
  - A. 一次性大规模替换（风险：行为改变、并发冲突）
  - B. **分档治理**（选定）：修复门禁阻断项 → 新增 panic 密度守卫 → 存量按优先级渐进替换
  - C. 维持现状

## Decision

采用方案 B，分三档落地：

1. **阻断项清零**（本次）：修复 CI 门禁 4 处 `unwrap_used`（deny 模块内）+ 9 处 capability_tree clippy 违例（`option_map_unit_fn`、`map_entry`、`from_str` 混淆、`manual_strip`、`useless_format`）+ 本 session 产出文件 7 处（`needless_borrow`、`redundant_closure`、`needless_match`、doc 缩进）。
2. **新增 `arch_fitness_panic_density` 守卫**：扫描 neotrix-core 源码统计 `unwrap`/`expect` 密度（每千行），输出结构化告警 + 趋势，接入 fitness 族与 SelfTest。
3. **存量渐进**：按模块优先级（网络/文件 IO → 锁 → 计算）以 `?`/`let-else`/`map_err` 替换，遵循 R-P3。

**约定**：`unwrap`/`expect` 仅允许在"不可达/静态不可错"处使用并带语义消息（如 `expect("static error response body is infallible")`）；`deny` 模块内禁止。

## Consequences

- **正面**：门禁转绿；panic 面收敛；与 SQALE/SEI 技术债测量对齐（可量化、可追踪）；为 C5 自愈奠基。
- **负面**：存量 3,000+ panic 点需长期分档消化；替换可能触及并发会话文件（需甄别归属）。
- **落地**：本次修复已含 R-P16 逐文件 re-read 验证；剩余存量列入 D 系审查长期项。

## Compliance / Fitness

- `arch_fitness_panic_density` 守卫输出密度指标，随 SEAL 周期跟踪趋势（单调递减目标）
- CI clippy `-D warnings` 为硬门禁，回归即红灯

## References

- SQALE（Software Quality Assessment based on Lifecycle Expectations）可维护性模型
- SEI：Technical Debt 测量与管理（Automated Measurement of Software Maintainability）
- Rust Clippy lints：`unwrap_used` / `option_map_unit_fn` / `map_entry` / `needless_borrow`
