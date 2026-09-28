# handoff — 算法萃取 + 目录归档（2026-09-28）

> 本会话在共享工作区与另一窗口并行作业。**凡标「他窗」的都不是本会话的改动。**

## 1. 一句话

从 `crates/` 的 2 个孤立 crate 里萃取出 **8 个通用算法**进主代码 L0–L6，
把 4 个 crate + 2 个顶层目录归档到 `~/Downloads/Neo/neotrix-archive/`，
裁决 2 笔架构债。顶层目录 **17 → 11**，`crates/` **13 → 9**。

## 2. 萃取清单（7 个文件被 git 识别为 rename，非增删）

| 萃取物 | 行 | 落点 | 主代码此前的空白 |
|---|---|---|---|
| `nt_fov.rs` | 284 | `l2_perception/` | 无任何 FOV 实现 |
| `nt_flow.rs` | 398 | `l3_embodiment/` | 只有单源 A*，无多源流场 |
| `nt_utility.rs` | 111 | `l5_cognition/` | 无通用效用选择器 |
| `dungeongen.rs` | 527 | `nt_game/world/` | — |
| `nt_net.rs` | 287 | `l6_meta/` | 权威房间协议 |
| `nt_commands.rs` | 141 | `nt_game/` | `persistence/replay.rs` 全文件 **0 个 sort** |
| `nt_clock.rs` | 86 | `nt_game/` | 无固定步长 |
| `nt_sampler.rs` | 244 | `l0_substrate/` | `top_k`/`top_p`/`repetition_penalty` **全是 config 透传，零实现** |

## 3. 两笔债的裁决

### 债1 — 度量冲突：统一到欧氏

`nt_flow` 原对角步不乘 √2（Chebyshev），但它自己的 A\* 启发式是 octile
（`a + 0.41421356b` = √2−1）——**Chebyshev 边代价配 octile 启发会高估，
返回次优路径**。这不是风格问题，是它出厂自带的 bug。

处置：`nt_flow` 两处边代价 ×`SQRT_2`；`nt_astar` 两处字面量 `1.414` 改用同一常量。
3 个断言具体距离值的测试改为**推导式 + 容差**。改动前实测两者**均 0 消费者**。

### 债2 — ECS「重复」：前提有误，划清作用域

`nt_game/ecs/`(291) vs L0 `nt_ecs.rs`(1,362) **不是重复**：
前者独有 tag 系统与 priority 调度，后者独有 SoA/ArchetypeId。
`render/scene.rs` 还靠它做场景图父子结构。**删掉会丢能力**，故不合并，
在该文件顶部立裁决文档。

## 4. 归档（`rsync -an -c` 逐字节 **0 差异**）

`crates/neotrix-abilities`(30) `neotrix-game`(37) `neotrix-decision-engine`(24)
`nt-lang`(6) `games/`(38) `fuzz/`(4) `thirdparty/`(7) `datasets/`(101M)
`models/{agent-jev,qwen3-06b,qwen35-4b-uncensored}`(7.2G) `sessions/` 未入库 232 件

## 5. ⚠️ 踩过的坑（都已有回归门或注释锁定）

1. **`models/training/jev_platts.json` 是雷** —— `nt_jev_calibration.rs:476` 用
   `include_str!` 编译期嵌入，且 `:590-595` 有回归门注释「曾被归档误搬后剩 `{}`
   空壳」。该文件**原地钉死**，勿动。
2. **归档顺序**：必须**先抽 `NtSampler` 再归档 decision-engine**，否则唯一真空算法
   一起丢。
3. **`nt_gen_model.rs` 973 行从未编译** —— 批量抢救提交 `e75272be` 加了文件漏了
   `mod` 声明。这是「入库 ≠ 编译」，与 layer-map `_rule` 的「导出 ≠ 调用」同源。
4. **`check-forbid-coverage.sh` 硬编码 crate 名单** —— 归档任何 crate 都必须同步
   改它，否则门报 `NO-FORBID`/漏检。本轮改了 2 次。
5. **反查消费者不能只扫 `Cargo.toml`** —— `neotrix-audit` 与
   `nt-core-capability-tree` 的 Cargo 反查显示「零消费者」，实为
   **CI 门在用**（`nt-audit.yml:30-31`、`ci.yml:224-226`）。须一并 grep
   `.github/workflows/`。

## 6. 遗留 / 待办

| 项 | 状态 |
|---|---|
| **`l6_meta/nt_core_capability/` 14 个死引擎 4,945 行** | 已审计出**零外部消费者**（15 个符号逐个开 import 核实，非零的 3 个是同名异物）。其 91 个测试全是死代码互测，且 `mod.rs:37-148` 与 `151-262` 是**逐字重复的两份 `inline_tests`**。**本轮未动** |
| 主代码重名副本 | `LoadBalancer` **3 份**（`l6_meta/nt_core_capability/loadbalancer.rs:59` / `l5_cognition/nt_core_gwt/load_balancer.rs:15` / `l5_cognition/nt_core/multi_agent/coordinator/load_balancer.rs:37`）；`VersionManager` **2 份**（`l6_meta/.../versioning.rs:121` / `l2_perception/nt_core_knowledge/versioning.rs:141`） |
| `layer-map.json` 的 `_rule` 写「CapabilityRegistry x4」 | **已过期，实为 3 套**（第 4 份 `nt_file_ability/capability.rs` 已于 2026-09-28 删除） |
| `Cargo.lock` | 未入库（`.gitignore:4` 的 `*.lock`）⇒ 构建不可复现。本轮改过 3 份 `Cargo.toml`，锁文件应随之更新入库 |
| `target/` | 52G 构建产物，未清 |

## 7. 门状态（本会话实测）

```
check-layer-deps.sh --strict   PASS: 0 new violation(s); 102 known/recorded
check-naming.sh                PASS (advisory)
nt_lock_audit.py               0 处
check-forbid-coverage.sh       OK        ← 曾因他窗删 src-tauri 而恒红，已修
cargo metadata --no-deps       exit 0，12 成员清单与 path 依赖全解析
pre-commit cargo check --tests 全仓仅 1 error（**他窗 model_pool.rs 的 unused
                               imports**，非本会话改动；详见该文件 model_pool.rs:22）
```
