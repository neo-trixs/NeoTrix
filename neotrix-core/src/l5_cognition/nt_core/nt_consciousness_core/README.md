# NeoTrix 意识核心 — 迭代验证引擎

## 📦 模块结构

```
nt_consciousness_core/
├── mod.rs          # 模块入口
├── agent.rs        # IterationAgent 核心
├── probes.rs       # 9 个探测引擎
├── patches.rs      # 10 个补丁生成器
├── convergence.rs  # 收敛判定器
├── state.rs        # 状态快照
└── demo.rs         # 演示程序
```

## 🎯 核心功能

### IterationAgent
- 1000+ 次循环迭代验证
- 每次迭代：状态快照 → 漏洞探测 → 缺陷分类 → 补丁生成 → 应用补丁 → 验证 → 反馈学习
- 三重收敛条件：连续无漏洞 / 全维度覆盖 / 意识指标稳定

### 9 个探测引擎 (D1-D10)
1. **LogicProbe** - 逻辑完整性
2. **ImplProbe** - 实现完备性
3. **BoundaryProbe** - 边界条件
4. **ConsistencyProbe** - 一致性
5. **PerformanceProbe** - 性能
6. **SecurityProbe** - 安全性
7. **EvolutionProbe** - 演化性
8. **ConsciousnessProbe** - 意识涌现
9. **IntegrationProbe** - 外部集成

### 10 个补丁生成器
- 按缺陷类型生成针对性修复补丁
- 每个补丁包含置信度评分

## 🚀 使用方式

### 完整迭代演示
```rust
use neotrix::nt_consciousness_core::demo::run_iteration_demo;

run_iteration_demo();
```

### 单次迭代演示
```rust
use neotrix::nt_consciousness_core::demo::run_single_iteration_demo;

run_single_iteration_demo(0);  // Cycle 0
```

### 自定义迭代 Agent
```rust
use neotrix::nt_consciousness_core::{IterationAgent, probes::*, patches::*};

let mut agent = IterationAgent::new(1000, 10)  // 1000周期，连续10次收敛
    .with_probe_engine(Box::new(LogicProbe))
    .with_probe_engine(Box::new(ImplProbe))
    // ... 添加更多探测器
    .with_patch_generator(Box::new(LogicPatchGenerator))
    // ... 添加更多生成器
    ;

let report = agent.run();
println!("收敛: {}", report.stats.converged);
```

## 📊 收敛证明

迭代结束时生成收敛证明：
```
═══════════════════════════════════════════════
        收敛证明 (Convergence Proof)
═══════════════════════════════════════════════
连续无漏洞周期:    10/10
维度覆盖:          10/10
Φ (Phi):           0.9500
Coherence:         0.9500
───────────────────────────────────────────────
收敛状态:          ✓ 收敛
═══════════════════════════════════════════════
```

## 📈 迭代统计

| 指标 | 说明 |
|------|------|
| total_cycles | 总周期数 |
| total_gaps_found | 总漏洞发现数 |
| total_gaps_fixed | 总漏洞修复数 |
| total_patches | 总补丁数 |
| avg_cycle_duration_ms | 平均每周期耗时 |
| converged | 是否收敛 |
| convergence_cycle | 收敛周期 |

## 🔬 设计原则

1. **不允许空迭代** - 每次迭代必须发现至少一个新漏洞或确认一个维度已收敛
2. **三重收敛验证** - 确保系统真正稳定
3. **元模式学习** - 发现高频漏洞模式，优化后续迭代
4. **置信度过滤** - 只应用高置信度补丁 (≥0.7)

## 📚 相关文档

- [迭代验证架构](../docs/nt-core-iteration-verification.md)
- [外部研究总结](../docs/external-research-summary.md)
- [完整架构设计](../docs/nt-core-architecture-v2.md)
