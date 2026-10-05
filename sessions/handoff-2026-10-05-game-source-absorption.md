# 交接 —— Rust 游戏源码吸收窗口（2026-10-05 15:5x ~ 18:52）

> 分支 `feat/capability-absorb-20260828` · 本窗口 5 个提交 · 无 worktree 新建
> **主树不是可信地面真相**：本窗口全程在他窗在途改动上工作。

---

## 1. 本窗口做了什么（一句话）

把 `ra2.exe` / `ra-ecs` / `ra-map` 的**实测源码**结论落成 5 个提交：
两个上一代 agent 留下的尾巴（P11/P12）收口，以及**三处由源码深读挖出、
且经判决实验证实的真实缺陷**（ECS 句柄契约、A\* 度量与穿角、A\* 自我复制）。

---

## 2. 提交清单（全部 `git commit --only`，共享 index 下零污染）

| hash | 内容 | 关键验证 |
|---|---|---|
| `2d574ffe` | **P11** 工具调用侧落库（`tool_calls` 表，645 行） | neobot **531 passed / 0 failed** |
| `648cffd1` | **P12** 删掉虚构的 `witness=` 旁证协议 | nt_provenance **44 passed**（附 mtime 一致性论证） |
| `3877336a` | `nt_ecs` 句柄契约：槽位回收 + 世代校验 + 堵幽灵组件 | 全量 **13263 passed / 0 failed** |
| `b2dae47c` | `nt_astar` octile 启发 + 禁穿角 + 合并两份复制实现 | 独立 crate **9 passed**（字节同一性论证） |

---

## 3. ⭐ 本窗口最贵的教训：**我带着错误前提准备重写死代码**

我上一轮断言「`nt_ecs.rs` 是 `HashMap<(EntityId, TypeId), Box<dyn Any>>`，
每组件一次装箱，要换成 dense+sparse」。**读码后发现该文件本来就是
`ArchetypeId` archetype 设计**，且：

- 全部公开 API **零真实消费者**（`nt_ecs::Chunk`/`SoAStorage`/
  `ParallelScheduler`/`Archetype` 外部引用均为 0；唯一那条 `ComponentStorage`
  命中是 `nt_game/ecs/mod.rs` 里的**注释**）。
- 1,362 行里活路径约 120 行，其余是装饰性结构。

⇒ **「优化一条没有调用方的热路径」是又一次装饰性重写。**
我把同一套判据（有没有真消费者？真的坏吗？）套到其余三个候选上，
当场又证伪两个：

- **候选④（统一 step_cost）早已完成** —— `nt_astar.rs:15` 的 `SQRT_2` 是共享
  常量，`nt_flow.rs:9-13` 明写两者「已统一为欧氏」，是更早会话做的。
- **候选③（越界返回 false）早已完成** —— `is_walkable`/`set_*` 都有显式
  四段边界检查。
- **候选②（世代戳复用）不存在对应缺陷** —— 真正的问题是 A\* 自身。

⇒ 真正值得做的是**让说谎的类型变成诚实的类型**，这才是 ra-ecs 那 17 行的
价值所在。

## 4. ⭐ 第二条教训：**手推 ≠ 实证，两次都差点改错方向**

**(a) 不可采纳启发式**：我推断「曼哈顿 + √2 对角边 ⇒ A\* 返回次优路径」。
按这个结论动手前先写测试 ⇒ **测试是绿的**，推断未被证实。
于是写**判决实验**：同一套边代价 + 同一套邻居规则的 Dijkstra 当 oracle，
对 60 个确定性 LCG 随机障碍布局逐个对账 ⇒ **28/60（47%）次优**，
最小一例差**恰好 2.0000**（L1 高估的特征值）。结论成立，但**是靠穷举
打出来的，不是靠推理**。

**(b) 穿角**：这条一次红就证实了（`(1,0)`/`(0,1)` 全堵仍返回
`Some([(0,0),(1,1)])`）。

**(c) 我自己的测试也编码了 bug**：`a_diagonal_still_works_when_one_orthogonal_neighbour_is_open`
断言 `len()==2` —— 那正是「直穿对角」。已改名并改为断言绕行 3 位置 / 代价 2.0。

## 5. ⭐ 第三条教训：**换对启发反而暴露了一个潜伏缺陷，并引入了新 bug**

- 换成 octile 后查询**跑不完**。逐次打迭代轨迹发现：旧 `closed` 只在**弹出**时
  记账 ⇒ 还在 `open` 里的邻居查不到 ⇒ 每次重展开都**重复 push**；
  闸门 `current.g > best_g` 是严格大于 ⇒ g 相等照样重展开。
  曼哈顿让 f 有梯度、能较快撞到终点，**把这个缺陷掩盖了**；octile 在无障碍格上
  f 恒等 ⇒ 堆退化 ⇒ 重复 push 指数膨胀。
  ⇒ 换成标准 A\* 模式（`g_score` 在**生成边时**更新）。
- 修好后**更糟**：连 10x10 空网格走 5 步都挂。轨迹显示迭代**已抵达终点、
  g 已是最终值**却仍在打转 ⇒ 卡在路径回溯 ⇒ `parents` 成环。
  根因是**我漏了 `g_score[start] = 0`**：起点能被当作邻居重新生成、
  `parents[start]` 被改写。症状极具欺骗性。

> 这三条都不是靠读码发现的。**只有把真实输入喂进去跑**才看得见。

---

## 6. 待办 / 建议下一步（**都还没动**）

| 项 | 依据 | 为何未动 |
|---|---|---|
| `neighbors()` 每次调用分配一个 `Vec` | 每弹出一个节点一次堆分配 | 正确性已修；`nt_astar` 零消费者，无吞吐动机 |
| `set_movement_cost` 不校验负值 | 负边破坏 A\* 不变量 | 属未验证输入面，需先有调用方 |
| `find_or_create_archetype` 线性扫全表比 `HashSet` | O(n) per insert | 同上，`nt_ecs` 零消费者 |
| `nt_game/ecs`（20,844 行中的一块） | 与 L0 `nt_ecs` 职责重叠 | 按「无定点不改」等真出现消费者 |
| ra-map 的 `PassGrid` 每次寻路**深拷贝整张网格** | `navigation.rs:123`，约 10 字节/格 × 4 次分配 | 本仓尚无对应调用点，属ra-ecs 侧的问题 |

**建议下一窗口先做「消费者审计」再动手** —— 本窗口 4 个候选里 3 个因
「零消费者 / 已完成」被当场撤下。这个判据比任何实现技巧都省事。

---

## 7. 未解决的外部阻塞（**不是本窗口引入**）

共享树被**他窗在途 WIP** 弄坏，本窗口**未触碰**任何他窗文件：

- 18:37 实测 12 个编译错误，全在两个文件：
  `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/{mod,policy_engine}.rs`
  与 `neotrix-core/src/l6_meta/nt_approval.rs`
  （`E0753 expected outer doc comment`、`E0255 ViolationSeverity 重复定义`、
  `E0599 ApprovalDecision 无 Deny/Ask 变体`…）。
- 17:0x 期间还两次被 `consciousness_runtime.rs` 阻断：
  `json!` 值位置塞语句块（E0753 类）；随后 `with_registry` 签名改为返回
  `Result` 而 `unwrap_or_else(||…)` 未跟上（E0593）。

⇒ **`cargo test -p neotrix --lib` 目前跑不了**。`b2dae47c` 的验证因此改用
独立 crate（文件**字节同一**，`sha256` 前缀 `927c88d0d2b07c43f4ddff5b`）。
基线参考：今日 17:0x 全量 **13263 passed / 0 failed / 38 ignored**。

---

## 8. 收工自查（**必填**）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` → **exit 4**，摘要：

```
[worktree-gate] worktree=6 个 | 合计 58866M | target 占 58439M
[worktree-gate] 带未提交改动: 4 个 | 近3h有改动: 2 个
[worktree-gate] ⚠️  2 个 worktree 近 3 小时仍有 .rs 改动 ⇒ 可能他窗在用，勿删
[worktree-gate] ⛔ 4 个 worktree 的未提交改动不在任何提交里：
[worktree-gate]      ⛔ /private/tmp/nt-probe
[worktree-gate]      ⛔ .worktrees/merge-b
[worktree-gate]      ⛔ .worktrees/nt-v3
[worktree-gate]      ⛔ .worktrees/nt-verify
[worktree-gate] ⛔  主树未提交: 15 处
```

- 本会话**新建 worktree 数量：0**。
- 临时验证 crate（非 git worktree，不在门视野内）：
  `/var/.../T/opencode/astar-probe`（21M）—— **已 `rm -rf` 删除**。
- 上表 6 个 worktree **均非本窗口创建**，且 4 个带未提交改动、2 个近 3h 有活动
  ⇒ 按 R-DISK-5 与并发公约，**本窗口一律未删、未 prune**。是否回收由用户裁决。

### 8.2 未提交改动的去向

**本窗口的 5 个代码文件全部已入库**（`git status --porcelain` 里
`nt_store`/`nt_provenance`/`nt_ecs`/`nt_astar` 零命中）：

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `crates/neotrix-neobot/src/nt_store/mod.rs` | P11 建表/索引 | ☑ 已提交 `2d574ffe` |
| `crates/neotrix-neobot/src/nt_store/nt_store_tool_calls.rs` | P11 新表（579 行） | ☑ 已提交 `2d574ffe` |
| `neotrix-core/…/nt_core_gate/nt_provenance.rs` | P12 删虚构协议 | ☑ 已提交 `648cffd1` |
| `neotrix-core/src/l0_substrate/nt_ecs.rs` | 句柄契约 + 7 测试 | ☑ 已提交 `3877336a` |
| `neotrix-core/src/l3_embodiment/nt_astar.rs` | octile + 禁穿角 + 合并 | ☑ 已提交 `b2dae47c` |

**本窗口遗留的两个 untracked patch —— 明确弃用**：

| 文件 | 理由 |
|---|---|
| `.neotrix/patches/2026-10-05-batch3-mechanisms.patch`（8,880 行） | 更早会话的**提交前快照**，内容已在 `f91b3763`/`942a62fd`。实测 `git apply --reverse --check` **已失败**（内容漂移）⇒ **零恢复价值**。属 AGENTS.md §5 说的「陈旧记录比没有更危险」，建议删除，但删 untracked 文件不由本窗口独断。 |
| `.neotrix/patches/2026-10-05-miu2d-ra2-determinism.patch`（769 行） | 同上 |

其余 13 项未提交（`.neotrix/capability_*.json`、`Cargo.lock`、
`entry/headless.rs`、`actions/security/sandbox.rs`、`nt_shield/guard/mod.rs`、
`consciousness_runtime.rs`、`nt_approval.rs`、`eval_engine/llm_judge.rs`、
`capability_registry.json.bak-legacy`、`knowledge.db`、
`nt_meta_integration.rs.disabled`、`handoff-…tui-wiring…md`）
**全部是他窗在途 WIP，本窗口未触碰、不声明去向。**

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**4**（带未提交改动的 worktree 存在 —— 全部他窗）
- 提交前是否跑过 `cargo test -p neotrix --lib`：
  - `2d574ffe`：✅ `cargo test -p neotrix-neobot --lib` 531 passed
  - `648cffd1`：✅ `nt_provenance` 44 passed（16:04–16:10，且文件 mtime 15:58:38 早于该次运行、此后未变 ⇒ 测的是同一份内容）
  - `3877336a`：✅ **全量 13263 passed / 0 failed / 38 ignored**
  - `b2dae47c`：☐ **否** —— 共享树被 §7 的他窗错误阻断；改用**字节同一**的独立 crate 9 passed，并已在提交信息里写明该缺口
- `nt_lock_audit.py`：本窗口每个 `.rs` 提交前均重跑，`neotrix-core/src` 0 处（RC=0）、`crates/neotrix-neobot/src` 0 处（RC=0）
- 门红归属：`b2dae47c` 的「全量未跑」= **他窗 WIP**，非本会话引入