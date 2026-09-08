# ADR-0001: 架构 Fitness Functions 守卫族（Evolutionary Architecture 门禁）

**Status**: Accepted
**Date**: 2026-08-15
**Deciders**: NT-CORE（NT-MIND / NT-MEMORY 协审）
**Technical Story**: 全面架构审查（D1-D63 + 外部标准对标）发现：演进型架构缺乏自动化"纪律守卫"，结构退化（越层依赖、平行适配器、重复能力边、TreeSingleton 泛滥）仅靠人工审查检出。

## Context

NeoTrix 采用 EvoArch（Building Evolutionary Architectures, Ford et al.）方法论，但缺乏对应落地：
- **问题**：`l1_body_impl` 直接引用 `l8_/l9_/l10_` 模块（越层）；`capability_registry.json` 出现重复依赖边；自持有/工厂模式外出现多处 `Box::new` 单例实例化；dead code 积累无人追踪。
- **可选方案**：
  - A. 仅人工审查（现状）——检出滞后、不可持续
  - B. 一次性 CI 脚本（grep 检查）——脆弱、无法复用核心数据
  - C. **fitness function 族 + SelfTest T3 接线**（选定）——把纪律编码为可运行守卫，接入 SEAL pipeline 与 7 域统一生命周期

## Decision

采用方案 C，实现 5 个 fitness 守卫（`neotrix-core/src/core/nt_core_arch_fitness.rs`）：

| 守卫 | 验证对象 | 阈值 |
|------|---------|------|
| `layer_boundary` | L1 文件不得引用 `l8_/l9_/l10_` 模块 | 0 违规 |
| `capability_acyclic` | 能力树 DAG 无环 | 0 环 |
| `capability_idempotent` | registry 无重复依赖边 | 0 重复 |
| `tree_singleton` | 非工厂处 `Box::new` 单例 ≤1 处生产实例化 | ≤1 |
| `dead_code` | 注册型死代码检出 | 告警级 |

**接线**：`run.rs` + `pipeline.rs` SelfTest 注册（T3），`nt_core_self_test_integration` count=13；核心数据经 `CapabilityRegistry` 读取而非文本 grep——守卫与进化引擎共享单一事实源。

## Consequences

- **正面**：架构纪律自动化；退化即红灯；与 SEAL 生命周期、审查维度 D 系形成闭环；符合 EvoArch fitness function 模式。
- **负面**：守卫需维护（豁免清单如 factory host / register 模式）；修复越层可能涉及跨域重构。
- **落地**：守卫经 `/tmp/fitness_probe` 与 `cargo test` 验证；`dead_code` 为告警级，其余为阻断级。

## Compliance / Fitness

- 守卫注册于 `nt_core_arch_fitness_tests` 与 `self_test_integration`（T3 生产接线，count=14，含 `PanicDensityFitness`）
- 违反时：layer_boundary 阻断越层，其余产生结构化告警进入 SEAL 审查输入
- **2026-08-15 蜕变补记**：本守卫族暴露并驱动清理了两组 theater 模块（零生产消费者、未注册能力树）——
  `l8_capability_impl`（retrieval/alignment/prompting/knowledge_graph/benchmark_suite）与
  `l9_capability_impl`（nt_safety）。按 Dark Forest（无消费者即删）与 R-P42（禁平行适配器）删除 13 文件，
  `neotrix/` 层目录收敛为每层唯一：L1-L10 与 `core/` 规范命名对齐（R1 层号冲突消解）。

## References

- Ford, N., Parsons, R., Kua, P. (2017). *Building Evolutionary Architectures* — Fitness Functions
- ISO/IEC/IEEE 42010:2022 架构描述框架
- 全量审查报告 D1-D63 输出（`docs/` 审查档案）
