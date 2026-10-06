# OPEN-DEFECTS —— 遗留待修缺陷（单一入口）

> 2026-10-06 建。此前缺陷散落在 70 份 handoff 正文里（`sessions/README.md` 有索引），
> 随窗口关闭而沉底。**新发现的缺陷必须同时登记到这里**，否则等于没报。
>
> **核实状态**逐条标注，含义严格：
> - `已核实` = 本轮用命令读过现场/跑过实测
> - `未核实` = 转述自某份 handoff，本轮**没有**复现，⛔ 别当既成事实直接动手
> - `已修` = 本轮修掉，附验证判据

## P0

| # | 缺陷 | 位置 | 状态 | 判据 / 备注 |
|---|---|---|---|---|
| 0 | ⛔ **`neotrix` crate 编译不过**：`unused variable: interp` | `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/input_validator.rs:589` | **已核实，2026-10-06 12:00 仍红** | 来自提交 `0c5b1bc9`（11:01）。**证据**：`cargo check`（**不调用链接器**）同样失败 ⇒ 与构建配置改动无关。该文件仍在被持续编辑 ⇒ 未代改。⚠️ 修法：`let Some(interp)` 那个 `interp` 未使用 ⇒ 改名 `_interp` 或补上使用。**⚠️ author 字段全部是 `openhands`（含我自己）⇒ 不能用 author 区分窗口，归属只能看文件清单。** |

| 1 | 一次写命令把能力注册表从 **318 节点/43 边**覆盖成 **41 节点/0 边**，**退出码 0** | `crates/nt-core-capability-tree/src/{node,cli}.rs` | **已修** `83bc568b` | 三处叠加：`kind` 缺 `#[serde(default)]`（318 节点全无此键）⇒ `from_str` 整体失败；`load_registry` 用 `Err(_)` 吞错误走老 schema 迁移（0 节点）；`save_registry` 在 `run()` 末尾无条件执行。门：`scripts/ops/nt-registry-determinism.sh` |
| 2 | 注册表快照**跨进程不确定**：直接序列化照抄 `metadata: HashMap` 迭代序 | 同上 | **已修** `83bc568b` | 实测同一输入 5 次写盘 ⇒ 5 个不同 md5。改走 `serde_json::Value` 中转。⛔ 单进程测不出（曾写 3 条单测，变异后 8/8 照样绿）⇒ 必须靠跨进程门 |

## P1

| # | 缺陷 | 位置 | 状态 | 判据 / 备注 |
|---|---|---|---|---|
| 3 | `capability_invoke` **不执行能力本体**，只做市场校验+计数+回执 | `crates/neotrix-neobot/src/nt_agent.rs` | 已核实（设计缺口） | trade 执行入口在 core L1，neobot 不依赖 core ⇒ 循环依赖。刻意不编造假结果。接法 A/B/C 见 `handoff-2026-10-06-capability-invoke.md` §3 |
| 4 | `dispatch_by_capability` **只解析节点就 `counts += 1`** ⇒ rationale 路径污染 `registered_never_invoked()` | `crates/neotrix-neobot/src/nt_capability_registry.rs` | 已核实（读码） | 应拆 `resolve_by_capability()`（无副作用）+ `record_invocation(id)`（仅执行成功后）。⛔ 注意 `record_dispatch`（`82824585` 新增）已具「成功后记账」语义，收敛时须一并处理，否则计数规则仍有两处 |
| 5 | 金丝雀窗口**进程全局**，多会话互相 `reset()` | `crates/neotrix-neobot/src/nt_capability_canary.rs` | **已核实** | `static CANARY: OnceLock<Mutex<Canary>>` + `static WINDOW_TICKS: AtomicUsize`，均无 `convo_id` 键。⇒ 需按会话键化 |
| 6 | `maybe_compact_context` **只有测试调用**，未接生产 | `neotrix-core/.../nt_io_agent_loop/` | **已核实** | 全仓调用点仅 `nt_loop_tests.rs:527,555`。接它会新增 LLM 调用 ⇒ 须先纳入 `nt_cost.rs` 预算，否则等于悄悄加钱 |
| 7 | core 侧旧 `distill_output` **预算过小时死循环** | `neotrix-core/src/l1_action/nt_io/nt_io_agent_loop/nt_loop_step.rs:156-161` | **已核实** | `while estimate_tokens(&clipped) > max_tokens && clipped.len() > 8`，每轮 `truncate(0.7*len)` 后 `push_str(15 字符)` ⇒ 存在 `len≈50` 的不动点；当 `max_tokens` 小到 50 字符都装不下即**永不退出**。循环后的兜底分支在该情形**不可达**。neobot 侧已修（`8196dd10`），core 这份属他窗未改 |

## P2

| # | 缺陷 | 位置 | 状态 | 判据 / 备注 |
|---|---|---|---|---|
| 8 | 2 处 **U+FFFD 编码损坏**在 core | `neotrix-core/src/l0_substrate/nt_core_event_bus.rs`、`neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/kb_search.rs` | **已核实，各 1 处** | 纯文本修复（1 行内）。建议加个门，否则编码损坏会静默传播 |
| 9 | `mod_orphan::scan_tree_sorted_by_lines_desc` 失败 | `neotrix-core/src/l0_substrate/nt_core_platform/mod_orphan.rs:682` | **已核实**（他窗引入） | 来自 `64913227`；已在**无本轮改动**的检出上复现 ⇒ 不是本轮引入。报「两个孤儿」实得 3 个 |
| 10 | 商业许可阻断：例外条目 `status: void` | `.neotrix/LICENSE-EXCEPTIONS.md` | 未核实（本轮未读该文件现状） | ⛔ 不得改门规避 |
| 11 | 能力市场无 API/UI；KB namespace/sensitivity → `ring_inner` 结果层门未接 | 多处 | 未核实 | 承接自 `handoff-2026-10-05-tui-wiring-and-six-defects.md` 等，⛔ 动手前先复现 |
| 12 | UI 门存在既有失败；macOS 截图受屏幕录制权限阻塞（AX 验证可用） | UI 相关 | 未核实 | ⛔ 复现前不要改门 |

## 编译与磁盘（本轮专项）

| # | 事项 | 状态 | 判据 / 备注 |
|---|---|---|---|
| C1 | `target/debug/incremental` 曾占 **14G / 22G（67%）**，是本仓最大的「无界增长文件」 | **已处理** | 删掉回收 14.1G，代价仅 **10.17s 且只重编 1 个 crate**（依赖不重编）。⇒ 这是 `cargo clean` 的正确替代品：`rm -rf target/debug/incremental`。⚠️ 上界：`incremental` 随编辑次数累积，需定期清 |
| C2 | 改 profile / rustflags 后**旧产物不删**，实测同 crate 多个哈希并存（`rand` 17 个、500 个 crate 重名） | 机制已记录 | 指纹失效 ≠ 旧产物删除。⚠️ `cargo-sweep` 按天龄**治不了**（超 1 天仅 9 个文件）⇒ 这类要靠 C1 或 `cargo clean` |
| C3 | `cargo-sweep` 未装 | **有意不装** | crates.io 最新 v0.8.0 **无 release 资产** ⇒ 只能源码编译；且当时 `target` 超 7 天仅 4 文件 ⇒ 回收≈0。等真有陈旧件再装 |
| C4 | `cargo-nextest` 已装但**不采用** | 已实测否决 | `group_contracts::*` **13/13 在 nextest 下失败、`cargo test` 下全绿**。定位：`test_sync_group` 单独跑 PASS、与 12 个兄弟同跑 FAIL ⇒ 测试**共用固定临时目录**，nextest 多进程并行互相踩。属测试写法问题，`cargo test` 保持默认 |
| C5 | lld 替代 ld64 | **已接** `df490a75` | 需 `brew upgrade lld`（22.1.7 → 23.1.2 才可用；22.1.7 报 `libSystem.tbd: unknown architecture: arm64e.x1-macos`）。⚠️ mold 走同源 TAPI 解析器，很可能吃同一刀 |
| C6 | `dev` profile 调试信息过重 | **已修** `df490a75` | 9.7G → 6.8G（−30%）。`[profile.dev] debug = "line-tables-only"` + `[profile.dev.package."*"] debug = 0`。⛔ 已撤回「−45% 编译耗时」：四次 clean 构建 2m55s/3m17s/4m31s/5m14s 不随 debug 递减 ⇒ 本机耗时是噪声，只有体积是确定性的 |

## 本轮新发现（未修，登记备查）

| # | 事项 | 位置 | 判据 |
|---|---|---|---|
| N1 | 集成测试被**禁用**（`.disabled` 后缀，文件有完整文档头 ⇒ 有意为之非垃圾） | `neotrix-core/tests/nt_meta_integration.rs.disabled` | nt_meta ⨯ nt_core 跨模块集成测试全部不参与 `cargo test` |
| N2 | 2 处 **U+FFFD 编码损坏仍在**（上轮已报，他窗未修） | `neotrix-core/src/l0_substrate/nt_core_event_bus.rs`、`neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/kb_search.rs` | 各 1 处；仍未加防复发门 |
| N3 | `nt-core-capability-tree` 3 个文件仍有 48 个装饰性 `⭐` | `registry.rs` 15 / `node.rs` 27 / `cli.rs` 6 | ⏸ **本轮有意跳过**：`registry.rs` 的 mtime 比当前时间还晚 54 分钟 ⇒ 疑似他窗带偏时钟在写，按并发纪律不撞车 |

## ⛔ 不要重复踩的坑（本轮实测得到）

1. **「序列化两次比对」抓不到跨进程缺陷** —— 同一进程内 `HashMap` 迭代序稳定。
   我为此写过 3 条单测，变异验证**全部照样绿**。唯一判据是起多个真进程比对字节。
2. **测试里重写一遍生产逻辑 = 零区分力** —— 我第一版 legacy 判别测试在测试里
   重抄了判别式，把生产代码改成恒真后测试仍全绿。⇒ 判别逻辑必须抽成**唯一**函数供测试调用。
3. **`$var` 后紧跟全角标点**会被 bash 并进变量名 ⇒ `unbound variable`。
   本轮在自写门脚本里踩到（`$first_md5）`）。⇒ 写 `${var}`。
4. **门必须双向验证** —— 我自写的门在二进制陈旧时 `FAIL`，修好后 `PASS`；
   若只跑一次「看起来绿」，无法区分「修好了」与「测的根本不是新代码」。
5. **裸 grep 命中不构成证据**（R-SCAN-1b）—— 本轮查 U+FFFD 时，
   `grep -c` 给出的是**文件级计数**，必须再 `grep -n` 看那一行本身。