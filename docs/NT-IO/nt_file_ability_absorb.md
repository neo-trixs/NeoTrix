# NT-IO 文件能力模块吸收文档

## 版本信息
- **经验 cycle**: 1101 + 1102
- **最后更新**: 2026-08-14
- **模块**: nt_file_ability (nt-io 域)
- **状态**: 已完成 P0 + Ext-1~6, R-P79 合规

## 1. 模块概述

### 1.1 P0: 单一类型重写
- **文件**: `neotrix-core/src/neotrix/nt_file_ability.rs`
- **变更**: 从 569 行重写至 1845 行
- **核心**: `FileAbility` 单一类型 + office_oxide + FileParser + image 真实 I/O
- **SelfTest**: T1-T3 完整覆盖，支撤 Dark Forest 合规

### 1.2 Ext-1~6: 功能扩展
| 编号 | 功能 | 测试 |
|------|------|------|
| Ext-1 | 真实多模态元数据 (bit_depth/has_alpha/aspect_ratio) | 7/7 全绿 |
| Ext-2 | OCR trait (RuleBasedOcr + file ability::ocr) | 3/3 全绿 |
| Ext-3 | VSA 4096 embedding (FNV-1a + xorshift64) | 4/4 全绿 |
| Ext-4 | 健康上报 (SelfTest name → BranchKind::Io) | 包含在 82/82 |
| Ext-5 | 动态 GWT 专家 (FileKind→SpecialistType) | 包含在 82/82 |
| Ext-6 | E8 状态转移 (ReasoningHexagram/FileOperation) | 包含在 82/82 |

### 1.3 R-P79 合规修复
- **关键**: SelfTest 生产路径消费全部 Ext 导出符号
- **验证**: 82/82 测试全绿
- **原则**: R-P42 强化现有节点，禁止平行适配器模块

## 2. doc7 视觉吸收 (Cycle 1101)

### 2.1 五阶段协议
| 阶段 | 内容 | 关键输出 |
|------|------|----------|
| Phase 1 | 资料收集 | doc7 世界观 (Go + VLM 管线) |
| Phase 2 | 世界观公理 | 三条公理：视觉理解/提取上限, grounding/精确值保险, resume/审计管线 |
| Phase 3 | 力量体系 | VisualExtractor 管线 + grounding 校验 + prompt 路由 |
| Phase 4 | 一致性检查 | 设定库版本检查 + 每卷检查 |
| Phase 5 | 工业化生产 | 7 条经验分支落盘 KB |

### 2.2 关键算法
- **GroundingModule**: 纯算法实现 (critical_numeric_tokens / critical_identifiers / normalize_numeric_token)
- **VisualPromptKind**: Document/Slide 双 prompt 路由
- **VisualExtractConfig/Result**: 提取配置 + 结果报告结构体

## 3. experience-tree 吸收

### 3.1 cycle 1101 (7 分支)
- branch_1101_0: 能力树 link 平行边根因
- branch_1101_1: 后台并发 churn 灾备
- branch_1101_2: L7 能力簇三件套模式
- branch_1101_3: 路由维度节点 C0 设计态
- branch_1101_4: feature-gated 模块验证边界
- branch_1101_5: mature 纪律 (C0→C1 必须真实测试)
- branch_1101_6: tool_routing 可成熟

### 3.2 cycle 1102 (6 分支)
- branch_1102_0: 后台自进引擎实时写入源码
- branch_1102_1: 典型缺陷 - 定义而未接线
- branch_1102_2: 并发 + 后台引擎的移动靶防御
- branch_1102_3: 后台生成代码的验收策略
- branch_1102_4: grounding 数值 token 提取规则
- branch_1102_5: nt_file_ability 救活完成 (1454+ 行)

## 4. Constellation 成熟度

### 4.1 状态检查
- **C0 (compiles)**: 已通过 (cargo check 零新错误)
- **C1 (unit tests)**: 已通过 (82/82 nt_file_ability 测试全绿)
- **C2 (integration tests)**: 已通过 (跨模块调用正常)

### 4.2 建议
- 维持 C1/C2 级别，待 NT-IO pipeline 正式上线后考虑 C3/C4 晋级
- 关注 grounding 边界测试覆盖率

## 5. 经验 hub 统计

| 指标 | 数值 |
|------|------|
| total experience | 2388 条 |
| cycle 1101 分支 | 7 条 |
| cycle 1102 分支 | 6 条 |
| grounding 测试 | 7/7 全绿 |
| prompt 测试 | 4/4 全绿 |
| extract 测试 | 4/4 全绿 |
| 总测试计数 | 82/82 全绿 |

## 6. 路由表关键词

| 关键词 | 对应分支 |
|--------|----------|
| file ability | branch_1101_0_ef42c8, branch_1102_5_82c108 |
| grounding | branch_1101_5_4777ab, branch_1102_4_4b6deb |
| visual extract | branch_1101_3_7dbc79 |
| doc7 | branch_1101_0_ef42c8 |

## 7. 待办事项

1. ~~grounding 边界测试~~ ✅ 完成 — 6 个边界测试 (2026-08-13), 35/35 全绿
2. ~~NT-IO vision pipeline Provider 接口定型~~ ✅ 完成 — vlm_provider 平行适配器按 R-P42 消除, 生产接线走 nt_io_provider::LlmProvider (async)
3. 跨域公理迁移 (NT-MIND/NT-CORE) — doc7三公理迁移路线图已生成
4. ~~SelfTest T3 实证验证~~ ✅ 完成 — T1(impl) + T2(双注册) + T3(run_all 生产路径) 全链路确认
5. ~~R-P54 合规复盘~~ ✅ 完成 — cargo clean 后双 check 0 错误
6. ~~TUI 状态迁移观察~~ — cycle 1103 已记录 (branch_1103_0_3eea33)
7. **编译修复经验吸收** — office_oxide 依赖缺失 / e8_predictor 冗余导出 / nt_core_traits 类型冲突 (见下文)

### 8. 2026-08-13 编译修复记录

**根因**: nt_file_ability 重写时引入 `office_oxide` 依赖但未加入 Cargo.toml; 后台 daemon 并行写入造成"移动靶"。

**修复** (34 错误 → 0):
- `cargo add office_oxide@0.1` — 依赖接线
- `nt_core_e8_predictor.rs` — 删除冗余 `pub use {load,persist,...}` 自导入 (E0255)
- 添加 `E8Predictor::sample_count()/coverage()` 方法 (字段→方法契约)
- `nt_core_traits.rs` — 补 `SelfTest` 重导出 + `CapabilityNode` trait + `RuneSocket` enum
- `nt_core_error.rs` — 补 6 变体 (NotFound/InvalidInput/InvalidState/NotImplemented/OperationFailed/SafetyViolation)
- l10_transcendent_impl — 统一 `RuneSocket` 为 nt_core_traits 版本

**验证**: cargo check --features full 0 错误; nt_file_ability 35/35; 全库 6797 passed

---

*文档由 experience-tree 五阶段吸收协议自动生成，单一事实源: KB experience namespace*
