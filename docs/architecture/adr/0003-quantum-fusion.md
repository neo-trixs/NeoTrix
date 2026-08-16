# ADR-0003: 量子态检测与最优融合（Quantum State Detection & Optimal Fusion）

**Status**: Accepted
**Date**: 2026-08-15
**Deciders**: NT-CORE（L4 认知层）
**Technical Story**: 架构升级需求「量子态检测与最优融合」——多检测源（GWT 共振、arch_fitness 守卫、E8 预测器、遥测异常检测）输出各自独立信号，缺少统一的最优融合层，导致多源信号冲突时无法产生单一高可靠决策信号。

## Context

NeoTrix 已有多种检测/评估能力（D1-D63 审查维度的 6 守卫族、GWT resonance、E8 预测、telemetry），但：
- **问题**：各检测器独立输出 `(value, confidence)`，无统一融合契约；信号冲突时无从判断可信度；无纠缠一致性（多源佐证强度）度量。
- **可选方案**：
  - A. 保持各检测器独立输出，人工仲裁——检出滞后、冲突无法自动消解
  - B. 简单平均/投票融合——忽略置信度与一致性，噪声信号权重过高
  - C. **量子态范式最优融合**（选定）——借鉴量子叠加/纠缠/坍缩概念（确定性实现），把多源信号视为叠加态，经纠缠一致性度量与置信度加权坍缩，产出单一高可靠信号

## Decision

采用方案 C，实现 `neotrix-core/src/core/nt_core_quantum_fusion.rs`（L4 认知层）：

| 概念 | 量子隐喻 | 确定性实现 |
|------|---------|-----------|
| `QuantumSignal` | 叠加态分量 | 检测源输出 `(value∈[0,1], confidence∈[0,1], source)` |
| `QuantumSuperposition` | 叠加态 | 多源信号集合，支持 `push`/`from_signals` |
| `entanglement()` | 纠缠一致性 | `1 - 加权标准差`（多源佐证强度，0.0=完全冲突 → 1.0=完全一致） |
| `entropy()` | 测量不确定性 | 信号值 8 桶香农熵归一化（高熵=分散） |
| `fuse()`/`fuse_with()` | 坍缩选优 | 置信度加权平均 + 纠缠调节（冲突线性惩罚 / 高纠缠加成）+ 主导源识别 |

**融合算法**（确定性，无真量子随机性，不引入外部量子库）：
1. 排除置信度低于 `confidence_floor=0.2` 的噪声信号
2. 置信度加权平均得基础值
3. 纠缠度调节置信度：`entanglement < conflict_threshold=0.6` → 线性惩罚；`> high_entanglement=0.75` → 加成
4. 主导源 = 最高置信度信号来源

**接线**（T3 生产）：
- `QuantumFusionSelfTest` 注册进 `register_absorbed_modules`（SelfTest 检测族 14→15）
- 生产路径 `handlers_consciousness::handle_architecture_audit` 经 `register_absorbed_modules` → `run_all()` 实际执行探针
- 能力树注册 `nt_core::nt_core_quantum_fusion::optimal_fusion`（bud C0 → mature C1）

## Consequences

- **正面**：多源信号获得统一最优融合契约；冲突自动降置信（而非误导）；纠缠一致性能量化多源佐证强度；可作 GWT resonance 与 arch_fitness 的未来融合前端；与 E8 预测、telemetry 输出天然兼容（同 `(value, confidence)` 契约）。
- **负面**：新增一认知层模块需维护；阈值超参（floor/conflict/high_entanglement）需随实测校准；当前以库函数形式存在，尚未被 GWT/arch_fitness 生产消费（待后续接线）。
- **验证**：7 单测（信号钳制/纠缠高一致/纠缠低冲突/熵/冲突惩罚/主导源/自定义阈值）+ 集成注册测试 3/3 全绿；`--features full` 编译通过；clippy 0 新警告。

## Compliance / Fitness

- 层归属：L4 认知层（`core/mod.rs:66` 声明，L4 区块内）——符合 9 层架构
- R-P1：`#![forbid(unsafe_code)]`，0 unsafe
- R-P11：超参走 `QuantumFusionConfig` config struct + `Default`
- R-P42：强化现有检测能力，无平行适配器
- R-P79：T3 生产接线（SelfTest 检测族 → 后台自愈循环）
- R-P100：能力树 bud C0 → mature C1 注册完成

## References

- ADR-0001 架构 Fitness Functions 守卫族（融合前端的既有检测源）
- ADR-0002 panic 债务治理（检测信号质量基线）
- Quantum-inspired 融合方法论：叠加/纠缠/坍缩概念的确定性算法映射