# 需人工决策项 · 技术决策书 (2026-09-27)

> 三项都不是"一行能改"的病。此处给出**事实、代价、推荐**，供人拍板，不代替决策。
> 背景与全量分诊见 `sessions/handoff-disease-list-20260927.md` §9。

---

## 决策 1 · KB 双时态节点 schema（阻塞 2 个测试）

### 事实

- `l0_substrate/nt_core_kb_primitives.rs:188` — `nodes.id TEXT PRIMARY KEY`。
  主键不含 `transaction_time`，因此**同一节点无法存在多个版本**。
  节点表本身已带 `valid_start_time` / `valid_end_time` / `transaction_time`
  （v9 引入，v10 建索引），即 schema 已经是双时态的，唯独主键没跟上。
- 4 处外键指向 `nodes(id)`：`:237` `:238`（edges.source_id/target_id）、
  `:258` `:277`（另两张表的 `node_id PRIMARY KEY`）、`:347`（无 cascade 的引用）。
- `nodes_as_of()`（`:685-691`）与 `node_history()`（测试 `:1025-1035`）
  **全仓零生产调用方** —— 只被自己的测试调用。
- 附带发现：`SCHEMA_VERSION = 10`（`:165`），但代码里已有 `if version < 11` 的迁移分支
  （`:579`，加 `parent_id`/`depth`）。新库被写成 10 号版本却带着 v11 列，
  属于版本号与迁移脚本的漂移，应一并校正。

### 代价对比

| 方案 | 改动 | 风险 | 后果 |
|---|---|---|---|
| **A. 复合主键** `(id, transaction_time)` | v12 迁移：建 nodes_new → 复制 → 换名 → 4 处 FK 重建 → 旧 FK 表重建；`nodes_as_of` 需加"每 id 取 MAX(transaction_time) <= ?1"窗口 | **高**。SQLite 改主键必须重建表；4 个 FK 表也要重建（否则 FK 指向失效）；真实库有数据 | 双时态真正可用；需要写迁移+回滚+在真实库副本上验证 |
| **B. 承认契约虚构** | 删掉 `nodes_as_of` / `node_history` 两个 API 及其 2 个测试 | **低**。零生产调用方，删了不影响任何行为 | 诚实：告诉世界"我们不做节点版本历史"；未来真需要时再按 A 做 |
| **C. 只让 `nodes_as_of` 正确** | 不动 PK，改查询加 per-id MAX 窗口 | 低。但 `test_node_history`（插 3 个版本）**仍然过不了**，PK 拦着 | 只能修一半，另一条仍红 |

### 推荐：**B**

理由：这套"双时态"从来没有被任何生产路径使用过——主键设计证明它从落地起就是半成品
（列加了、约束没改）。为一个**零调用方**的 API 付"重建 5 张表 + 迁移真实库"的代价不划算，
而且 C 方案会让两个测试一红一绿、留下"看起来支持版本历史其实不支持"的假象。
若产品确实需要节点修订历史，再单独立项做 A。

---

## 决策 2 · CAD self-test 接线证据是自我欺骗面

### 事实

- `l5_cognition/nt_core_cad_consciousness.rs:332-336` 的 `CAD_SELFTESTS` 列 12 个名字，
  其中 8 个（`cad_csr`、`cad_generator` 等）指向 `neotrix/l2_world_impl/*.rs` ——
  **本仓不存在这些文件**。注册处 `nt_core_self_test_integration.rs:26,34`
  早已被注释掉并注明 "module not found"。
- 列表里还有字面垃圾条目 `"// // cad_generator"`（`:335`）。
- 测试 `cad_full_wiring_verification` 在第一步就 panic：
  `run_one("cad_csr")` 返回 `None`。
- **更严重**：`cad_wiring_map()`（`:164-209`）把这些**死 file:line 字符串**当作
  "接线证据"输出，供 D16 晋升门消费。一个恒过的门比红测试更危险。

### 选项

| 方案 | 说明 | 风险 |
|---|---|---|
| **A. 重新吸收那 8 个模块** | 找回或重建 `l2_world_impl` 下的 CAD self-test | 高；等于重新做一轮吸收，且不知原实现是否还适配现架构 |
| **B. 砍到真实的 4 个** | `CAD_SELFTESTS` 只留确实注册了的（`cad_seal_stage`、`cad_runeword`、`cad_wiring_evidence`、`cad_absorption`），**并同步删掉 `cad_wiring_map()` 里对应的死证据行** | 低；D16 晋升门的判据会变严（这才是正确的方向） |

### 推荐：**B**

关键点不在测试红不红，而在于**那份证据表在骗人**。留着不修，晋升门会基于不存在的
接线证据放行。修的时候必须**证据表与测试清单同改**，否则只是把一个假门换成一个半假门。

---

## 决策 3 · publish_gateway 五个平台臂全是"not wired"

### 事实

- `l1_action/nt_act/actions/orchestration/publish_gateway.rs:205-212`：
  YouTube 臂硬编码 `success: false`，注释 "upload not wired"。
  `PublishConfig`（`:49-62`）只有 `oauth_token` / `auto_publish`，**没有 `dry_run`**，
  模块内也没有 mock 路径。测试 `test_publish_gateway` 直接断言真实上传成功。
- 该模块 5 个平台臂全是同一形态。

### 选项

| 方案 | 说明 | 代价 |
|---|---|---|
| **A. 真接 YouTube Data API v3** | OAuth2 + `reqwest` 可续传上传 + 配额/错误处理 | 真实工作量；还要凭证与网络；测试仍需离线替身 |
| **B. 加一等公民 `dry_run`** | `PublishConfig { dry_run: bool }`，`dry_run` 时走完整任务生命周期但不发出网络请求，返回结构化结果 | 小；且**让测试能测真正该测的东西**（任务状态机/幂等/错误映射），而不是测一个硬编码 false |
| **C. 删测试** | — | ❌ 否决：等于把文档里承诺的能力静默藏起来 |

### 推荐：**B**，A 另立专项

理由：现在这个测试**测不到任何东西**（断言的是一个硬编码值）。
B 用很小代价换来"发布任务生命周期真的被测到"，同时不假装能力已具备。
若产品近期真要投 YouTube，再在 B 之上做 A——B 的 dry_run 分支也不会白写。

---

## 附：本决策书产出时的环境阻塞

验证上述改动所需的 cargo 构建当时被内存门拦住（另两个窗口的 agent 进程 + 供 App 使用的
9B `llama-server` 常驻，free 内存 < 1.6G 门限）。这 29 项修改处于**已改未验**状态，
恢复方式是：等内存让出 → 单模块定向跑 → 绿了再提交。详见 handoff 文档的"Resume"段。
