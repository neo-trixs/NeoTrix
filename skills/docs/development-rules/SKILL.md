# Development Rules

## Purpose
编码、构建、审查、吸收纪律的全量规则

## Trigger Words
- dev rules
- development rules
- coding rules
- 开发规则
- 编码规则

## Content
- R-P1 到 R-P101 规则
- 编码基线
- 构建规则
- 审查维度
- 吸收纪律

## File
`docs/standards/NEOTRIX-STD-1.0.md` (canonical since 2026-09-21, SIM-14;
legacy rule files live read-only under `docs/standards/archive/`)

## Usage
处理编码、构建、审查任务时加载。加载后内容为强制规则。

## Key Rules (canonical IDs first, legacy in parens — SIM-17)

- NTS-A01 (ex R-P1): `#![forbid(unsafe_code)]` — zero unsafe in core
- R-P6: Float clamping: `.max(0.0).min(1.0)` not `.clamp()` (style rule, no exact NTS clause; see NTS-C08)
- R-P8: `make_stage!` macro for SEAL pipeline stage definitions (legacy; see NTS-B08)
- R-P16: Re-read files after editing to verify persistence (workflow habit; see NTS-A08)
- NTS-B10 (ex R-P42): Absorption strengthens existing nodes, no parallel adapters
- NTS-B10 (ex R-P79): External tech must be wired to production in same session
