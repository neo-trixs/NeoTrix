---
name: law-expert
description: 占位描述（PLACEHOLDER，零实现）—— 见下方「状态」段
when_to_use: 暂勿使用
disable-model-invocation: true   # 占位技能，禁止模型自动触发
---

> **状态：PLACEHOLDER（2026-09-28 实测）**
>
> - 仓内**消费者 0 处**、目录内**只有本文件**（无 references/scripts/tests）、
>   最后提交 2026-07-06（84 天前，批量 checkpoint "Cycle 33 cleanup"）。
> - **不承担职责** —— 本仓**无**对应实现 —— 属「领域待建设」，不是错误资产。
> - 已 `disable-model-invocation: true`，模型不会自动加载它。
> - 处置：**保留不删**。它是「待建设」而非「错误资产」，且删除不可逆；
>   真要清理须同步改 `skills/index.json` + `skill_loader` + `check-skill-gate.sh`。
>   依据 AGENTS.md「导出 ≠ 调用」：0 消费者是删除的必要条件，不是充分条件。

# Law Expert Skill

## Description
Legal document analysis, contract review, and legal research capabilities.

## Skill Type
domain-specific

## Tags
- law
- legal
- contract
- compliance
- regulation

## Usage

This skill provides legal-related capabilities:

1. **Contract Analysis**: Review and analyze legal contracts
2. **Legal Research**: Search and analyze legal precedents and regulations
3. **Compliance Check**: Verify compliance with relevant laws and regulations

## Capability Vector Adjustment

When this skill is used, adjust the following capability dimensions:
- `legal_analysis`: +0.2
- `document_review`: +0.15
- `compliance_check`: +0.1
- `research`: +0.05

## Activation Triggers

- User mentions legal contracts
- User asks for legal research
- Task involves compliance verification
- User needs document review with legal implications

## Examples

### Contract Review
```
User: Review this employment contract for potential issues
Assistant: [Uses legal analysis capability]
```

### Compliance Check
```
User: Is this business practice compliant with GDPR?
Assistant: [Uses compliance check capability]
```

## Notes

This skill requires careful handling of sensitive legal information. Always recommend consulting with qualified legal professionals for critical decisions.
