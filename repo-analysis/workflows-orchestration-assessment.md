# P1-07: Claude Workflows 脚本化 Orchestration 可行性评估

> **Date**: 2026-05-29 | **基于**: NeoTrix orchestrator v1 (15 files, ~2300 lines)
> **Source**: Claude Workflows (CW) script-runtime pattern vs NeoTrix `orchestrator/`

---

## 1. CW Script-Runtime 核心模式

CW 的关键创新是**将编排逻辑（JS 脚本）与执行（subagents）解耦**：

1. Claude 从自然语言生成 `.js` 编排脚本
2. 脚本在隔离的 runtime 中运行（无 filesystem/shell 访问）
3. 脚本通过 IPC 协调 subagents — 最多 16 并发，1000 总计
4. **内置检查点/恢复** — 每次 `checkpoint()` 调用序列化运行时状态
5. **Adversarial verification** — 脚本内嵌验证步骤，subagent 间交叉质询
6. **Token 追踪** — 每次 IPC 调用计量 token 消耗，超额时暂停

## 2. NeoTrix 现有编排架构差距分析

| 维度 | CW Script-Runtime | NeoTrix Orchestrator | 差距评级 |
|------|--------------------|----------------------|----------|
| **编排定义** | JS 脚本（动态生成，灵活组合） | `flow.rs`: 链式 builder 定义静态 DAG | **中** — `Flow` 结构已存在但未集成 |
| **检查点/恢复** | `checkpoint()` 内置，中断后从检查点继续 | `run_recursive_loop()` 全程同步阻塞，中断=全重来 | **高** — 缺失 checkpoint 机制 |
| **Subagent 协调** | IPC 驱动，16 并发 agent，1000 总量 | `WorkerNode` 4 线程 `ParallelExecutor` + 可选 `AgentTeam` | **中** — AgentTeam 存在但容量有限 |
| **Adversarial 验证** | 脚本内嵌多 agent 交叉质询+共识 | `adversarial.rs`: 基于关键词的模式匹配，非 LLM 驱动 | **高** — 验证深度不足 |
| **Token 追踪** | 内建于 runtime，每次 IPC 计量 | 无 token 追踪，无配额管理 | **高** — 完全缺失 |
| **脚本持久化** | `.js` 文件可保存、共享、版本控制 | 编排是内存中的一次执行 | **高** — 无脚本化能力 |
| **状态管理** | 运行时状态可序列化为 JSON | `flow_state.rs::StateManager<T>` 支持 history+rollback | **低** — 已有基础，需扩展序列化 |
| **Flow 引擎** | 脚本是流程引擎 | `flow.rs:92-140` `FlowRuntime` 有 trigger-based execution | **低** — FlowRuntime 骨架已存在，未集成 |

## 3. `flow.rs` / `flow_state.rs` 检查点/恢复能力分析

### 已有基础（Low Effort Gap）

`flow_state.rs` 已提供：
- `StateManager<T>` 支持 `history` + `rollback()` + 序列化 (`Serialize` bound on `ConfigState`)
- `FlowStateId` (UUID) 唯一标识状态
- `FlowRuntime<S>` 持有 `completed_steps` + `current_step`

**可直接用于 checkpoint 的基础**：
```rust
// flow_state.rs:21-73  — StateManager 已有完整的 history tracking
// flow.rs:92-97        — FlowRuntime 结构体可 serialized
```

### 缺失（High Effort Gap）

1. **Checkpoint 写入/读取** — 无 `FlowRuntime::save_checkpoint(path)` / `load_checkpoint(path)`
2. **中断恢复** — 无从 `completed_steps` 反推 next step 的恢复逻辑（`ready_steps()` 已有但需测试中断后恢复）
3. **长期持久化** — 无基于磁盘的 checkpoint 存储，`StateManager` history 仅存在内存
4. **Flow 未集成到 Orchestrator** — `run_recursive_loop()` 不使用 `FlowRuntime`，而是直接操作 `StateGraph`

## 4. 间距量化评估

| 缺失能力 | 实现难度 | 预估人日 | 前置依赖 |
|----------|----------|----------|----------|
| Checkpoint 序列化 | 低 | 0.5 | `serde` 已就绪 |
| Checkpoint 恢复逻辑 | 低 | 0.5 | `ready_steps()` 已实现 |
| Flow+Orchestrator 集成 | 高 | 3 | 需重构 `run_recursive_loop()` |
| 脚本化编排定义 | 高 | 5 | 需 JS runtime 或 DSL 解释器 |
| Adversarial subagent | 高 | 5 | 需多 agent 原生 LLM 验证 |
| Token tracking | 高 | 3 | 新 telemetry 子系统 |

**总实现**: ~17 人日（3-4 周全人力）

## 5. 可行路径建议

### Tier 1（快速见效, 3-4 天）: Checkpoint 基础

1. 给 `FlowRuntime` 添加 `save_checkpoint()` / `load_checkpoint()` — 利用已有的 `Serialize` bound
2. 将 `FlowRuntime` 集成到 `run_recursive_loop()` 作为可选执行模式
3. 测试中断恢复路径

### Tier 2（组合编排, 1-2 周）: Flow + Script 混合

1. 将 `Flow` 定义从代码内联移到 JSON/YAML 配置文件
2. `Orchestrator::run_flow(flow_path)` 从文件加载 Flow 定义
3. 每个 `FlowStep.action` 映射到 `PlannerNode` / `WorkerNode` / `CriticNode` 调用

### Tier 3（完整 CW 等价, 3-4 周）: Script Runtime

1. 集成轻量 JS runtime（boa_engine 或 rhai）
2. 定义 NeoTrix IPC API（spawn_subagent, checkpoint, get_result）
3. 实现 adversarial verification 的 LLM 驱动版本（复用 `CriticNode` + 多 perspective）

## 6. 结论

**可行，但不要追求 CW 完整的 JS script-runtime 等价。**

NeoTrix 的 `flow.rs` + `flow_state.rs` 已有 **检查点架构的 60% 基础**。当前最务实的路径是 Tier 1（Checkpoint 集成）快速获得中断恢复能力，然后评估 Tier 2 的 JSON flow 定义是否满足编排需求。

**无需实现完整 JS runtime** — NeoTrix 的 Flow builder API (`flow.rs:38-88`) 通过链式调用已经提供了等价的声明式编排能力，缺的是序列化和集成。
