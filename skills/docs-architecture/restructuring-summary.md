# NeoTrix 架构重构 - 执行摘要

## 项目背景

NeoTrix 是一个 776K 行、2,347 文件的 Rust 单体仓库，实现了一个 6 层 "自进化推理内核"。

---

## 核心发现

### 冗余度: 72/100 (高)

**主要问题**:
1. FFI 模块完全重复 (13 文件)
2. SelfModel 4+ 变体分散在 3 个位置
3. ConsciousnessState 定义在 7 个地方
4. Answer Engine 在 core/ 和 l5_cognition/ 重复
5. 6 个 facade stub 文件 (单行 re-export)

### 架构健康: 45/100 (严重问题)

**主要问题**:
1. L5 Cognition 是单体 (696 文件/240K 行)
2. 循环依赖通过 re-export 变通
3. L0 从 L6 re-export (层次倒置)
4. core/ 目录作为幽灵层
5. 392 个 nt_core_* 文件

---

## 重构方案

### Phase 1: 快速去重 (1-2 天)

**任务**:
- 删除 src/ffi/ 目录
- 删除 Answer Engine 重复
- 删除 Consciousness Crystal 重复
- 删除 6 个 facade stub 文件
- 清理 legacy/ 目录

### Phase 2: 类型整合 (1 周)

**任务**:
- 合并 4+ SelfModel 变体
- 合并 7 个 ConsciousnessState 定义
- 创建统一错误层次
- 替换 100+ 独立错误枚举

### Phase 3: L5 分解 (2-3 周)

**任务**:
- 拆分 neotrix-consciousness
- 拆分 neotrix-reasoning
- 拆分 neotrix-gateway
- 拆分 neotrix-multi-agent

### Phase 4: 层次修复 (1 周)

**任务**:
- 吸收 core/ 到 L0
- 修复层次倒置
- 整理 facade

### Phase 5: 优化 (1 周)

**任务**:
- 消除循环依赖
- 清理 nt_core_* 命名
- 死代码清理
- 最终验证

---

## 多 Agent 巡检系统

| Agent | 职责 | 频率 |
|-------|------|------|
| agent-redundancy | 冗余检测 | 每次提交 |
| agent-architecture | 架构合规 | 每次提交 |
| agent-types | 类型一致性 | 每次提交 |
| agent-layers | 层次边界 | 每次提交 |
| agent-deadcode | 死代码检测 | 每周 |

---

## 预期成果

| 指标 | 当前 | 目标 | 改进 |
|------|------|------|------|
| 冗余度 | 72/100 | <30/100 | -58% |
| 架构健康 | 45/100 | >80/100 | +78% |
| L5 文件数 | 696 | <200 | -71% |
| 循环依赖 | 多处 | 0 | -100% |

---

*版本: v1.0*
*创建: 2026-09-19*
