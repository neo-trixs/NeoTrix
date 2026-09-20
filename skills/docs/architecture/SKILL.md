# Architecture Documentation

## Purpose
NeoTrix 6层意识架构文档

## Trigger Words
- architecture
- 6-layer
- consciousness architecture
- 架构
- 6层架构

## Content
- 6层意识架构设计
- 依赖规则（向下依赖）
- 目录结构映射
- Model Gateway 集成
- 类型定义

## Key Files
| File | Purpose |
|------|---------|
| `ARCHITECTURE.md` | 主架构文档 |
| `ARCHITECTURE_OVERVIEW.md` | 架构概览 |
| `0-ARCHITECTURE/*.md` | 详细架构文档 |

## 6-Layer Architecture
```
L6 Meta-Cognition (元认知层)    →  nt_meta + nt_repair + nt_nexus
L5 Cognition (认知层)           →  nt_core + nt_mind + ModelGateway
L4 Emotion (情感层)             →  nt_feel (core emotion engine)
L3 Embodiment (具身层)          →  nt_physical + nt_shield + nt_feel
L2 Perception (感知层)          →  nt_world + nt_sense
L1 Action (行动层)              →  nt_act + nt_io + nt_memory
L0 Substrate (基底层)           →  SelfTest + Time + EventBus + Types
```

## Dependency Rule
**Downward only**: `L(n)` may import from `L(n-1)` but never `L(n+1)`.

## Location
`docs/0-ARCHITECTURE/` and `docs/ARCHITECTURE.md`

## Usage
研究 NeoTrix 架构设计时加载此技能。
