# ADR Template — 架构决策记录

**Status**: Proposed | Accepted | Deprecated | Superseded by [ADR-NNNN](0000-adr-template.md)
**Date**: YYYY-MM-DD
**Deciders**: NT-CORE / NT-MIND / NT-MEMORY / 相关域
**Technical Story**: (若关联 issue/需求)

## Context

描述需要决策的**背景与约束**（尽力用可验证事实，含当前代码位置 file:line、指标、外部标准依据）。好的 Context 应让读者不需要读代码就能理解"为什么会有这个决策"。

- 问题陈述：我们在哪遇到张力/矛盾？
- 可选方案：
  - 方案 A：……
  - 方案 B：……
  - 方案 C：……

## Decision

明确写出决策内容（"我们采用 X，放弃 Y，因为……"）。避免模糊表述。决策必须可追溯：给出理由、权衡、以及**外部标准依据**（如 ISO 42010 / ATAM / OWASP / DORA / SQALE 等）。

## Consequences

- 正面（P 收益）：……
- 负面（T 成本/风险）：……
- 迁移/落地路径：如何接线到生产路径（遵循 R-P79 同 session 生产接线，禁止延期死代码）

## Compliance / Fitness

- 如何验证决策被遵守（fitness function、审查维度 D1-D50、Constellation 成熟度）
- 违反时的检测机制与回滚路径

## References

- 外部标准/链接
- 相关 ADR
