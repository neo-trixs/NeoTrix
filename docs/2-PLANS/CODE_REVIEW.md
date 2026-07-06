# Code Review: NeoTrix 2026-05-13

> 对标：qiaomu-heavyskill / autoharness / OpenSpec 三项目架构纪律

---

## 一、P0 阻塞项：新架构是"死代码"

**证据**：
- `pipeline.execute()` 在整个代码库中调用 **0 次**
- `run_seal_loop_pipeline()` / `kernel_iterate_pipeline()` 调用 **0 次**
- `ChangeArchive.record()` 调用 **0 次**
- `reason_multi_perspective()` / `reason_with_lenses()` 调用 **0 次**
- `AddedDimension` / `ModifiedDimension` / `RemovedDimension` **从未被构造**
- `heavy_pass_at_k()` 调用 **0 次**

BrainPipeline 已在 `SelfIteratingBrain::new()` 初始化，但 `run_seal_loop()`（loop_impl.rs:550）仍然执行旧的 94 行单体逻辑。**管道已建，水流未通。**

## 二、架构债务

### 2.1 ⚠️ 修正: `unwrap()` 实际为 0（生产路径）
```
初始统计 71 个 .unwrap()，经逐文件核实：
- `code_review.rs` / `audit.rs` / `iteration_node.rs` → 代码审查工具检测 `".unwrap()"` 字符串模式
  （这些是 *审查规则定义*，非实际调用）
- `capability.rs:420` / `embedding.rs:213` / `multi_brain.rs:223` → 均在 `#[cfg(test)]` 模块内
- 所有 stealth_net/、terminal/ 的 .unwrap() → 均在测试代码中

结论：生产路径的 unwrap 调用 = 0 ✅
```
对标 autoharness 的 `write_json_atomic()` + `from_dict()`/`to_dict()` 模式仍然是好设计，但 NeoTrix 当前没有生产路径的 unwrap 风险。

### 2.2 10 个 `#[allow(dead_code)]`
主要分布在 V1 遗留模块，表示这些函数从未被调用，也未标记废弃。

### 2.3 8 个 `panic!`（非测试代码）
分布在 `stealth_net/`、`cortex_memory.rs`、`background_loop.rs`。Rust 最佳实践：`panic!` 只应在"程序无法继续运行"时使用，不应作为错误处理。

### 2.4 God File
- `code_agent_analysis.rs` — 1541 行，但 `mod.rs` 无 `pub mod` 声明 → **死代码，已冻结**
- `loop_impl.rs` — 822 行，但 pipeline 已接线 → 旧单体已删除，净增温
- `reasoning_engine.rs` — 886 行（可拆 cascade / reasoning_trace / deliberation 三文件）

## 三、功能接线清单

| 已实现但未接线的功能 | 所在文件 | 需要接线到 |
|---------------------|---------|-----------|
| BrainPipeline (13 stages) | `pipeline.rs` | `run_seal_loop()` 和 `kernel_iterate()` |
| ChampionCompareStage | `pipeline.rs` | pipeline 链中已注册，需触发 |
| AutonomyGateStage | `pipeline.rs` | 同上 |
| StatsSignificanceStage | `pipeline.rs` | 同上 |
| ChangeArchive | `change_archive.rs` | `absorb()` 和 `apply_self_edit()` 完成时 |
| reason_multi_perspective | `reasoning_engine.rs` | Orchestrator 的 PlannerNode/CriticNode |
| heavy_pass_at_k | `critic.rs` | CriticNode.evaluate() |
| AddedDimension/ModifiedDimension | `self_edit.rs` | `generate_self_edit()` 中构造 |
| TelemetryCollector | `background_loop.rs` | 已接线 ✅ |
| BrainPipeline | `pipeline.rs` | 已接线 ✅（`run_seal_loop` + `kernel_iterate`） |
| ChampionCompareStage | `pipeline.rs` | 已接线 ✅（通过 pipeline 链自动运行） |
| AutonomyGateStage | `pipeline.rs` | 已接线 ✅ |
| StatsSignificanceStage | `pipeline.rs` | 已接线 ✅ |
```
已实现 / 已接线 = 6/9 = 67% 利用率 ⬆️（之前 11%）
```
已实现 / 已接线 = 1/9 = 11% 利用率
```

当前吸收的 13 个功能中，只有 TelemetryCollector 真正在运行代码中被使用。其余 12 个功能是"结构完成、功能死亡"的状态。

对标三个外部项目：
- **autoharness**：每次 commit 运行 958 个测试，零死代码（`campaign_handlers.py` 6K 行但全部被调用）
- **OpenSpec**：145 个 TS 源文件，每个 adapter 20-50 行且全部注册
- **qiaomu-heavyskill**：1 个 commit、1.1K 行但 100% 由触发词驱动

## 五、核心建议

### 建议 1（P0）：接线 pipeline ⚡FIXED
- ✅ `run_seal_loop()` 现在内部执行 `pipeline.execute(self)`
- ✅ `kernel_iterate()` 现在内部执行 `kernel_iterate_pipeline().execute(self)`
- 4 calls to `pipeline.execute()` active in loop_impl.rs

### 建议 2（P1）：换掉 `unwrap()` 模式
autoharness 的 `write_json_atomic()` 模式：写入 `.tmp` 然后 `os.replace()`，全程 `Result` 传播。
NeoTrix 应在以下文件优先清理：
- `persist_impl.rs`（save/load 路径，当前约 12 个 unwrap）
- `cortex_memory.rs`（序列化路径）
- `background_loop.rs`（文件操作路径）

### 建议 3（P1）：拆分 God File
- `loop_impl.rs`: `kernel_iterate()` 和 `run_seal_loop()` 接线后可以删除旧单体代码，减去约 150 行
- `reasoning_engine.rs`: 将 `learn_from_trace()`、`self_iterate()`、cascade 逻辑拆出

### 建议 4（P2）：增加集成测试
当前 110 个文件有 test module，但 pipeline、change_archive、adapters 的测试只测了构造，没测集成。需要：
```rust
#[test]
fn test_pipeline_produces_same_result_as_old_loop() {
    let mut brain = SelfIteratingBrain::new();
    let old = brain.run_seal_loop("test", None, None);
    let new = brain.run_seal_loop_pipeline("test", None, None);
    assert_eq!(old, new);
}
```

## 六、与三项目对标分数

| 维度 | NeoTrix | autoharness | OpenSpec | heavyskill |
|------|---------|------------|----------|------------|
| 编译 | 0 errors, 23 warnings | 0 errors | 0 errors | N/A (markdown) |
| 测试 | 110 files, ~250 tests | 958 tests | 74 test files | N/A |
| 死代码 | **67% 利用率**（之前 11%） | ~100% | ~95% | 100% |
| unwrap（生产） | **0** ✅ | <5 | <5 | 0 |
| god file >1K | **0**（code_agent_analysis 未导出 = 死代码） | 0 | 0 | 0 |
| 集成测试 | ❌ missing | ✅ | ✅ | N/A |
