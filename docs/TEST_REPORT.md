# NeoTrix 测试报告

**生成时间**: 2026-09-18  
**测试命令**: `cargo test -p neotrix --lib`  
**状态**: 编译失败，测试未执行

---

## 1. 测试概览

| 指标 | 值 |
|------|-----|
| 编译状态 | 失败 |
| 编译错误 | 111 |
| 编译警告 | 133 |
| 通过测试 | 0 |
| 失败测试 | 0 |
| 测试覆盖率 | 0% (无法执行) |

---

## 2. 错误分类

### 2.1 模块文件未找到 (E0583) — 4个

| 模块名 | 位置 | 说明 |
|--------|------|------|
| `nt_core_ws` | `core/mod.rs:152` | 工作空间模块 |
| `nt_core_cache` | `core/mod.rs:153` | 缓存模块 |
| `nt_core_answer_engine` | `core/mod.rs:155` | 答案引擎模块 |
| `nt_core_qtest` | `core/mod.rs:156` | 快速测试模块 |

### 2.2 类型不匹配 (E0308) — 8+个

**主要问题：`KnowledgeSource` 双重定义**

`neotrix_types::KnowledgeSource` (crates/neotrix-types/src/core/nt_core_knowledge/types.rs:50) 与 `l2_perception::nt_core_knowledge::types::KnowledgeSource` (neotrix-core/src/l2_perception/nt_core_knowledge/types.rs:83) 存在冲突。

**受影响文件**:
- `l5_cognition/nt_mind/nt_mind/infrastructure/tests/brain.rs` (6处)
- `l5_cognition/nt_mind/nt_mind/evolution/agent_capability/tests.rs` (SearchResult vs WebSearchResult)

### 2.3 未声明类型 (E0433/E0425) — 6个

| 类型 | 文件 | 建议导入路径 |
|------|------|-------------|
| `FullReasoningState` | `engine_core/executor.rs:1092` | `crate::l5_cognition::nt_core_hex` 或 `neotrix_types` |
| `ReasoningHexagram` | `engine_core/executor.rs:1093` | `crate::l5_cognition::nt_core_hex` 或 `neotrix_types` |
| `CrtPlan` | `evolution/goal_loop/types.rs:259` | `crate::l5_cognition::nt_core::nt_crt` 或 `neotrix_types` |
| `ReasoningType` | `engine_core/cache.rs:20` | `crate::neotrix::nt_crystal_core` 或 `crate::nt_mind::infrastructure` |
| `ExternalKnowledgeAbsorbStage` | `pipeline/preprocess.rs:78` | 未定义的类型 |

### 2.4 方法未找到 (E0599) — 5个

| 方法 | 结构体 | 建议 |
|------|--------|------|
| `call_llm` | `ReasoningEngine` | 添加到 `ReasoningEngine` impl 或通过 trait |
| `generate_cot` | `DefaultCoTGenerator` | 导入 `CoTGenerator` trait |

### 2.5 私有字段/方法访问 (E0616/E0624) — 4个

| 成员 | 结构体 | 建议 |
|------|--------|------|
| `frames` | `VideoFeatureSummary` | 改为 pub 或添加 getter |
| `key_frames` | `VideoFeatureSummary` | 同上 |
| `classifications` | `VideoFeatureSummary` | 同上 |
| `broadcast_and_finalize` | `ReasoningEngine` | 改为 pub(crate) |

### 2.6 其他错误 — 5个

- 类型注解缺失 (E0282): 2处
- trait bound 不满足 (E0277): 1处
- 未定义变量/导入 (E0432): 2处

---

## 3. 警告分类 (133个)

| 类型 | 数量 | 说明 |
|------|------|------|
| unused imports | ~60 | 未使用的导入 |
| unused variables | ~40 | 未使用的变量 |
| unused mut | ~15 | 不必要的 mutable |
| deprecated | 2 | 使用了已弃用的 `consolidate_tables` |
| duplicate_macro_attributes | 4 | 重复的 `#[test]` 属性 |
| unexpected_cfgs | 3 | 未定义的 feature flag |
| unused_doc_comments | 5 | 文档注释未被使用 |

---

## 4. 根因分析

### 4.1 文件拆分引入的断裂

v0.25.5 进行了文件拆分（consciousness_core 4656行、engine_core 3004行、pipeline 3226行），但模块声明与实际文件不匹配。4个模块（ws/cache/answer_engine/qtest）在 `core/mod.rs` 中声明但文件不存在。

### 4.2 类型统一不完整

`KnowledgeSource` 在 `neotrix-types` 和 `neotrix-core` 中各有一份定义，测试代码使用了 `neotrix_types` 版本但被测方法期望 `neotrix_core` 版本。

### 4.3 API 边界不清晰

`ReasoningEngine` 缺少 `call_llm` 方法，`VideoFeatureSummary` 字段为私有但测试代码尝试直接访问。

---

## 5. 修复优先级

### P0 — 必须修复（阻塞编译）

1. **创建缺失模块文件**: `nt_core_ws.rs`, `nt_core_cache.rs`, `nt_core_answer_engine.rs`, `nt_core_qtest.rs`
2. **统一 `KnowledgeSource`**: 统一到 `neotrix_types` 或添加 re-export
3. **添加缺失类型导入**: `FullReasoningState`, `ReasoningHexagram`, `CrtPlan`, `ReasoningType`

### P1 — 应该修复

4. **为 `ReasoningEngine` 添加 `call_llm` 方法**
5. **公开 `VideoFeatureSummary` 字段或添加 getter**
6. **公开 `broadcast_and_finalize` 方法**

### P2 — 可以延后

7. 清理 133 个警告（unused imports/variables）
8. 修复重复 `#[test]` 属性
9. 添加 `network_tests` 和 `integration_tests` feature flags

---

## 6. 下一步建议

1. **立即**: 修复 P0 错误使编译通过
2. **短期**: 运行完整测试套件，更新本报告
3. **中期**: 添加 CI 编译检查，防止模块声明与文件脱节
4. **长期**: 考虑 `#[cfg(test)]` 模块的自动化验证

---

## 7. 测试输出

完整编译输出保存于: `docs/test-output.txt`
