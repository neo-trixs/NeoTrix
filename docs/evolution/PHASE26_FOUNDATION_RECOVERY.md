# Phase 26: Foundation Recovery + Consciousness Wiring

## 诊断

| 瓶颈 | 阻塞 | 根因 |
|------|------|------|
| 编译失败 (69 errors) | 一切进化 | API 不兼容 (VsaTagged/self_category 改名, Debug 缺失, 借用检查) |
| 认知模块未接线 (M0) | 推理能力不升级 | ModuleRegistry 不存在, ~8 模块孤立 |
| 自我修改不安全 | RSI 自举 | 无回滚/事务/验证门控 |
| 网络感知未集成 | 外部感知 | nt_core_network 创建但未接入 ConsciousnessPipeline |

## Gate 递进

```
Gate 0 ──→ Gate 1 ──→ Gate 2 ──→ Gate 3
 编译零错误   认知模块接线   自我修改安全   RSI 自举加速
```

### Gate 0: Zero-Error Compilation (当前)
修复 4 类编译错误:
- **Debug 缺失** (~12 处) — `#[derive(Debug)]` 批量添加
- **VsaTagged API 不一致** (~11 处) — 统一调用约定
- **借用检查器** (~15 处) — clone/restructure
- **类型不匹配** (~15 处) — 对齐签名

**通过标准**: `cargo check -p neotrix --lib` 0 errors

### Gate 1: Consciousness Module Wiring
- 实现 `ModuleRegistry` trait — ~200 行编排代码
- 将 8 个孤立模块接线到 `ConsciousnessPipeline::run_full_cycle()`
- Draft-Edit-Refine 外层编排现有 refinery loop

### Gate 2: Edit Safety Infrastructure (P0.5)
- TransactionScope — 原子提交/回滚
- CompileVerifyGate — 修改后自动 cargo check
- TestVerifyGate — 修改后自动 cargo test

### Gate 3: RSI Bootstrap Integration
- 网络层 (nt_core_network) 接线到 ConsciousnessPipeline
- Ne 自举管道 (SelfInspectable→SystemCard→CodegenBridge) 接线到自我修改循环
- RSI Metrics (code_auto_rate, engineer_multiplier, task_autonomy_hours)
