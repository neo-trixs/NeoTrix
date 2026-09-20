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
`dev-rules.md`

## Usage
处理编码、构建、审查任务时加载。加载后内容为强制规则。

## Key Rules
- R-P1: `#![forbid(unsafe_code)]` — zero unsafe in core
- R-P6: Float clamping: `.max(0.0).min(1.0)` not `.clamp()`
- R-P8: `make_stage!` macro for SEAL pipeline stage definitions
- R-P16: Re-read files after editing to verify persistence
- R-P42: Absorption strengthens existing nodes, no parallel adapters
- R-P79: External tech must be wired to production in same session
