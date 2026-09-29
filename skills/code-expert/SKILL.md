---
name: code-expert
description: 占位描述（PLACEHOLDER，零实现）—— 见下方「状态」段
when_to_use: 暂勿使用
disable-model-invocation: true   # 占位技能，禁止模型自动触发
---

> **状态：PLACEHOLDER（2026-09-28 实测）**
>
> - 仓内**消费者 0 处**、目录内**只有本文件**（无 references/scripts/tests）、
>   最后提交 2026-07-06（84 天前，批量 checkpoint "Cycle 33 cleanup"）。
> - **不承担职责** —— 代码分析/生成/审查能力已由 **L5 `nt_mind_skill_engine/`**（16 文件）与
`l2_perception/nt_core_code_search.rs`（670 行，含 RRF 融合）承接。
> - 已 `disable-model-invocation: true`，模型不会自动加载它。
> - 处置：**保留不删**。它是「待建设」而非「错误资产」，且删除不可逆；
>   真要清理须同步改 `skills/index.json` + `skill_loader` + `check-skill-gate.sh`。
>   依据 AGENTS.md「导出 ≠ 调用」：0 消费者是删除的必要条件，不是充分条件。

# Code Expert Skill

## Description
Code analysis, generation, and review capabilities for multiple programming languages.

## Skill Type
automation

## Tags
- code
- analysis
- generation
- review
- programming

## Usage

This skill provides code-related capabilities:

1. **Code Analysis**: Analyze code structure, quality, and potential issues
2. **Code Generation**: Generate code based on requirements
3. **Code Review**: Review code for best practices and security issues

## Capability Vector Adjustment

When this skill is used, adjust the following capability dimensions:
- `code_generation`: +0.15
- `analysis`: +0.1
- `debugging`: +0.1
- `inference_depth`: +0.05

## Activation Triggers

- User requests code generation
- User asks for code review
- User wants to analyze code structure
- Task involves programming languages

## Examples

### Code Generation
```
User: Write a function to calculate Fibonacci numbers
Assistant: [Uses code generation capability]
```

### Code Review
```
User: Review this PR for security issues
Assistant: [Uses code review capability]
```

## Notes

This skill integrates with ReasoningBrain's capability vector system. Each use updates the brain's capability based on success rate and task complexity.
