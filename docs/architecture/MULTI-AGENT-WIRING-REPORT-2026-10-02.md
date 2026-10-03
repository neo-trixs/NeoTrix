# `multi_agent` 接线实测报告（2026-10-02）

## 结论：**已修好 5 处使它可编译，但 177 个测试跑出 11 个真实失败 ⇒ 本轮不接线**

## 为什么先测不裁决
`MIRROR-BANK-2026-09-30` 记的是「15 文件 / 4,655 行 / 177 测试**全部从未编译**」。
「从未编译」意味着**里面是什么完全未知** ⇒ 不测就裁决等于猜。

## 实测结果（临时声明 `mod multi_agent;` 后）

| 阶段 | 结果 |
|---|---|
| 修编译错前 | **8 个错误**（3 真类型错 + 5 未用导入/变量） |
| 修完编译错 | `cargo check -p neotrix --lib` **0 error** |
| 跑 177 个测试 | **169 passed / 11 failed / 0 ignored** |

⇒ **接线在技术上可行**（比几天前记录的「8 个错误」乐观：那 8 个里 5 个只是未用导入），
**但代价是 11 个从未被执行过的真实缺陷**。

## 已修的 5 处（**真实缺陷，已提交，即使暂不接线也保留**）

### ① `crew.rs` E0382 —— 真逻辑 bug：`else` 分支不可达
```rust
if let Some(aggregated) = worker_results.into_iter().find(|r| r.success) { ... }
else if let Some(fallback) = worker_results.into_iter().next() { ... }  // ⛔ 已被消耗
```
`into_iter()` 第一次就**消耗**了 `worker_results` ⇒ `else if` 永不执行。
本函数意图是「优先取首个 success，否则退回首个」⇒ 需**先定位再取走**。
已改为 `position()` + `swap_remove()`。

### ② `graph_orch/dag.rs` E0308 —— 作者把 `Vec::remove` 当成 `HashMap::remove`
```rust
while let Some(node) = queue.remove(0) {   // ⛔ Vec::remove 返回 T，不是 Option<T>
```
⇒ 「expected `str`, found `Option<_>`」。已改为 `while !queue.is_empty() { let node = queue.remove(0); ... }`。

ⓘ **我在这行上连续错 3 次**：先假设 `Vec::remove` 返回 `Option`（实际返回 `T`），
试了 `drain(..1)`（引入 E0499 借用错误）、又试 `Some(&node)`（仍不对）。
**拦下它的是「停下来恢复原状、重新精确读编译器输出」**，不是继续试变体。
⇒ 教训：**对语言 API 的假设也要验证**；连错 3 次时该做的是重新测量，不是第 4 次试变体。

### ③ `graph_orch/scheduler.rs` E0277 ×2 —— 多剥了一层引用
`nodes: &Vec<&str>` ⇒ `for x in nodes` 已得 `&&str`，原代码写 `for &&node_id`
⇒ 局部变量成了 unsized 的 `str`。改为 `for &node_id in nodes`。

### ④ `graph_orch/scheduler.rs` 测试导入漏 `DagEdge`
同文件 8 处 `DagEdge::new(..)` 无法解析；`optimizer.rs` 的同类导入本就含 `DagEdge`，
`dag.rs` 靠 `use super::*` 拿到 ⇒ **只有这一处漏了**。
这正是「孤儿测试从未编译」的直接后果：**编译错误从未被任何人看见**。

### ⑤ `graph_orch/monitor.rs` `DagMonitor` 漏 `Serialize`/`Deserialize`
同文件 `NodeStatus`、`StatusTransition` **都已派生**，只有它漏 ⇒ JSON 往返测试无法编译。
顺带标注一句**与代码不符**的注释：原文写「Thread-safe via interior mutability
(all state is in the HashMap)」，但字段是**普通** `HashMap`/`Vec`，**无任何 Mutex**
⇒ 该「线程安全」描述是假的。⛔ 本轮**不加** Mutex（那是设计变更），只标注。

## 已修的 1 个**真实产品缺陷**（由 177 个测试抓出）

### `identify_critical_path`：空 DAG 返回**幽灵节点**
```rust
let end_node = dist.iter().max_by_key(..).unwrap_or("");  // ⛔ 空串当哨兵
while let Some(node) = current {   // current = Some("") ⇒ 仍执行一次
    path.push(node.to_string());    // ⛔ 把 "" 压进结果
```
空 DAG ⇒ `path == [""]`（长度 1）而非空 ⇒ 任何调用方 `path.first()` 得到 `Some("")`。
已改为 `let Some(end_node) = ... else { return 空路径 }`。

## 剩余 11 个失败（**未修**，需要逐条裁决）
```
coordinator::coordinator::tests::complete_task_updates_load
crew::tests::parallel_execution
delegation::tests::delegation_request_builder
graph_orch::optimizer::tests::critical_path_diamond
graph_orch::optimizer::tests::merge_preserves_outgoing_edges
graph_orch::optimizer::tests::merge_sequential_tasks
graph_orch::optimizer::tests::optimize_full_pipeline
graph_orch::optimizer::tests::parallelization_no_suggestions_for_linear
graph_orch::scheduler::tests::entries_for_turn_filter
graph_orch::scheduler::tests::makespan_calculation
graph_orch::scheduler::tests::schedule_diamond
```
集中在 **`graph_orch`（optimizer 5 + scheduler 3）**，即 DAG 调度与关键路径。
已知形态：`makespan_calculation` 实测 `left: 2 right: 3`（步数算少）。

## 本轮为何**不接线**
保持接线会让 `cargo test -p neotrix --lib` **变红**（11 失败），
打破本会话一直维持的 **12,497 passed / 0 failed** 基线。
⛔ **不擅自让仓库变红** —— 该状态需显式裁决，不是顺手能做的。
⇒ 已回退 `mod multi_agent;`，基线恢复 12,497 全绿；
**但上述 5 处修复 + 1 处真实缺陷保留并提交**（它们是对真实代码的真实修复）。

## 与本计划的关系
`EMERGENCE-PLAN-2026-10-02.md` §五写过：
> **不接线 `multi_agent`**：…零验证。⇒ 与 P2 的「可证伪」要求冲突：
> 未编译的东西无法提供证据。

⇒ 本轮**部分推翻**该判断：接线是**可行的**，且一接线就暴露了 1 个真实产品缺陷
（空 DAG 幽灵节点）—— 这本身就是「不编译 ⇒ 无法提供证据」的直接证明。
⇒ 但也**证实了它的代价**：11 个缺陷需逐条裁决。
⇒ 下一步应是**逐条分诊这 11 条**，而不是整包接线或整包归档。