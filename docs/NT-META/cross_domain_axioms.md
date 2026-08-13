# 跨域公理迁移文档

## 目标
将 doc7 世界观构建中的三大公理迁移到 NeoTrix 多域架构，建立“视觉理解=提取上限”在 NeoTrix 语境下的等价表达。

## 1. doc7 三大公理

### 1.1 公理一：视觉理解是提取上限
**原文**: "Visual understanding is the upper bound of extraction"

**迁移说明**:
- **doc7 语境**: Go + VLM 管线的提取上限，受限于模型能力和数据质量
- **NeoTrix 语境**: `nt_file_ability` 的 Ext-3 VSA embedding 容量上限 (4096 维)
- **等价表达**: `VSA_dimension ≤ Extraction_capability(model, data_quality)`
- **在 NeoTrix 中的实现**:
  - E8VsaEmbedding 使用 4096 维超向量
  - content_similarity 函数量化提取质量
  - VSA 相似度 (cosine) 作为提取可靠性指标

**行动项**:
- [x] 正式化 `ExtractionUpperBound` 概念到 NT-CORE
- [x] 将 VSA 维度与提取质量建模关联
- [x] 建立 grounding 可靠性评分体系

**落地 (2026-08-13)**:
- `GroundingReport::reliability_score` (0.0~1.0) — 公理二量化 (nt_file_ability.rs:1138)
- `VisualExtractResult::extraction_bound_ok` — 公理一量化, 可靠性 < 0.5 标记超限 (nt_file_ability.rs:1345)
- `visual_extract` 在生产路径计算 `extraction_bound_ok`, 调用方据此二次校正 (R-P79)
- 5 个新测试覆盖: test_axiom1_* / test_axiom2_*

### 1.2 公理二：grounding 是精确值保险
**原文**: "Grounding is the exact value insurance"

**迁移说明**:
- **doc7 语境**: grounding.go / grounding_numeric.go 中的数值锚定，防止 VLM 响应的幻觉
- **NeoTrix 语境**: `nt_file_ability` 的 P1 GroundingModule，纯算法校验
- **等价表达**: `Grounded_value ← Numerical_token_extraction ∧ Consistency_check`
- **在 NeoTrix 中的实现**:
  - `critical_numeric_tokens` / `critical_identifiers` 函数
  - `ground_missing_tokens` → 缺失 token / 标识符保护
  - 序列一致性校验 / math 行保护

**行动项**:
- [x] 将 grounding 校验步骤 formalize 到 NT-IO pipeline
- [x] 建立“grounding 失败 → LLM 分发”机制 (R-P79 合规)
- [x] 设计数值提取可靠性评分

**落地 (2026-08-13)**:
- `reliability_score` 计算: 1 - (未接地 token / 源关键 token 总数), math_guard_skipped 豁免 (nt_file_ability.rs:1199)
- 分发机制: `extraction_bound_ok = false` → 调用方二次校正/人工复核 (nt_file_ability.rs:1413)
- 空源无风险 → 可靠性 1.0

### 1.3 公理三：resume 是审计管线
**原文**: "Resume is the audit pipeline"

**迁移说明**:
- **doc7 语境**: 履历/履历文档的审计追踪，确保变更可审计
- **NeoTrix 语境**: 经验吸收的五阶段协议 + route_table 持久化
- **等价表达**: `Audit_pipeline ← Experience_absorption ∧ Route_table_persistence`
- **在 NeoTrix 中的实现**:
  - cycle 1101 + 1102 的五阶段吸收 (Phase 1-5)
  - experience KB `experience` namespace 持久化
  - route_table 关键词路由持久化
  - `neotrix-experience close --cycle N` 闭合快照

**行动项**:
- [x] 文档化经验吸收协议的每个阶段要点
- [x] 建立跨 session 的 experience 跟踪机制
- [x] 验证 route_table 持久化的幂等性

**落地 (2026-08-13)**:
- route-verify --clean 幂等验证: 两次均 0 ghost, 217 routes 不变
- cycle 1101/1102/1103/1104 全部落盘 KB (28 分支)

## 2. 迁移路线图

### 2.1 第一阶段: NT-IO 基础 (当前)
- grounding 数值 token 提取规则正式化
- VisualExtractor Provider 接口定型 (已完成 vlm_provider.rs)
- 可靠性评分体系原型

### 2.2 第二阶段: NT-MIND 扩展
- 将公理二 (grounding) 扩展到能力树健康监控
- 公理三 (resume) 迁移到能力树进化日志
- 跨模块一致性检查

### 2.2 第三阶段: NT-CORE 理论
- 公理一 (提取上限) 形式化为数学模型
- 与 VSA HyperCube 容量模型关联
- 算法子strate 一致性审计

### 2.4 第四阶段: 统一框架
- 三公理在 NeoTrix 全局治理中的统一体现
- 元认知层面的公理验证机制
- 自演闭环: 公理 → 模型 → 验证 → 公理微调

## 2. 关键概念定义

| 概念 | doc7 定义 | NeoTrix 定义 | 对应模块 |
|------|-----------|--------------|----------|
| Extraction upper bound | VLM 模型能力上限 | VSA 4096维 + 数据质量 | nt_file_ability Ext-3 |
| Exact value insurance | grounding 防幻觉 | critical_numeric_tokens 校验 | nt_file_ability P1 |
| Audit pipeline | 履历审计文档 | experience 五阶段协议 | experience-tree |
| Extraction upper limit | 提取上限 | VSA_dimension / model_quality | Ext-3 / Ext-1 |
| Reliability score | 无 | grounding 可靠性评分 | P1 / Ext-1 |

## 3. 行动计划

### 3.1 即时 (本 sprint)
1. ~~完成 vlm_provider.rs 框架验证~~ ✅ — vlm_provider 平行适配器按 R-P42 消除, 生产接线走 `nt_io_provider::LlmProvider` (async)
2. ✅ grounding 可靠性评分 (reliability_score) + 提取上限分发 (extraction_bound_ok)
3. ✅ 公理一 NeoTrix 等价表达已文档化 + 代码化 (5 测试)

### 3.2 短期 (2-3 sprint)
1. 将公理一、二、三映射到 NT-MIND 能力树 — ✅ 部分: `nt_file_ability::unified_file_ops [C1]` 已注册能力树 (R-P100); 公理量化已纳入 FileAbilitySelfTest 第 9 项 (R-P79 生产接地)
2. 建立跨模块一致性检查 — 待 (跨 session)
3. 经验吸收协议的公理化验证 — ✅ 部分: route_table 幂等性验证 + cycle 1101-1105 落盘

### 3.3 长期 (5-10 sprint)
1. 统一框架设计与实现 — 待 (架构设计决策)
2. 自演闭环机制 — 待
3. 治理层面的公理版本管理 — 待

---

*本文档由 experience-tree cycle 1101 + 1102 的吸收经验生成，单一事实源: KB experience namespace*
