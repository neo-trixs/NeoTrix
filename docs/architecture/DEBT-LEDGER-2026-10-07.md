# 债务清单（权威单一来源）

> **取代**散落在 `FOLLOWUP-TASKS-2026-10-06.md` 里的债务段落。
> 本文件只存**实测数字 + 修法 + 判据**，不存过程叙事（过程见 `FOLLOWUP-TASKS`）。
> ⛔ 数字必须来自**门/工具输出**，⛔ 禁止凭记忆填写（AGENTS.md R-SCAN-3）。

## 实时状态（2026-10-07 实测）

| 项 | 实测值 | 门/工具 |
|---|---|---|
| `check-unwrap` NEW | ⭐ **0** —— `--strict` **首次全绿**（`PASS: 0 new violation(s); 514 known`） | `bash scripts/check-unwrap.sh --strict` |
| `check-unwrap` 存量 | **514** 站点，全部记账在 `scripts/unwrap-baseline.txt`（**基线已瘦身 705 → 514**） | 同上 |
| 死配置待判定 | **184**（bool 133 + numeric 51） | `python3 scripts/ops/nt_dead_flag.py --root . --types {bool,numeric}` |
| 死配置「未接线规格」 | ⭐ **0**（bool 与 numeric **均归零**） | 同上 |
| 名字债 | 133 条已定性、**全部留痕**（含 3 处「门报告错误」的撤标与 1 处「门是对的」裁定） | 同上 |
| `check-naming` | **1615** offender（advisory） | `check-naming.sh --strict` |
| 能力未接线 | **3 / 5** DeclaredOnly + **1 / 5** Scaffold | `market.rs` 的 `executability` |
| `check-executor-registry` | **RC=0** | 本会话新建 |
| 元门 | 未登记 **0** · 恒红 **1** · 探针失败 **0** | `check-gate-satisfiable.sh --strict` |

## ⭐ D1 —— unwrap/expect 债务：**已清零**（`check-unwrap --strict` 首次全绿）

### ✅ 本会话已清 28 处（27 → 0）

#### ⭐ 最后 4 处（4 → 0）：**根因互不相同，无一是「删个 expect」**

| 提交 | 站点 | 真实根因 | 修法 |
|---|---|---|---|
| `30c2c9f1` | `nt_io_output_style.rs` | `expect` 把「不可达」写成**运行时不变量**（未来加 `remove()` 即变可达） | `resolve → Option<&dyn>`；`apply` 三级兜底，末级**恒等透传**（没样式就该保持原文） |
| `a8915b61` | `social_access/traits.rs` | **零调用方**的 panic 包装 | 删函数 + 迁移 2 个调用方到 `?` |
| `4795e209` | `nt_mind_background_loop/run.rs` | ⭐ **无效 fallback**：失败后**重开完全相同的路径**，而失败原因（不可写/磁盘满/锁冲突）**不因重试消失** ⇒ 真实故障下必 panic；且 `log::warn!("creating temp")` **谎称在降级** | `kb: Arc<KB> → Option<Arc<KB>>` + 访问器返回 `Err`；handler 降级为 no-op |
| `7e421b9a` | `goal_loop/loop_impl/core.rs` | **借用检查器限制**（⛔ 不是「可能失败」：上一行刚 `= Some(...)`，`.expect` 逻辑上恒成立） | 返回 `Option<&GoalTracker>`（13 处调用方全忽略返回值 ⇒ 零破坏） |

**承重原则**（本轮反复用到）：
- ⛔ 明确**不**fabricate 空 KB/空库 —— 那会让 `weave_patterns` 返回 `Ok(0)`，
  把「KB 缺失」**伪装成「已挖掘但无模式」** ⇒ 必须用 `Option` 区分两种「没有」。
- ⛔ **不**给门加豁免来「修」违规（那把症状变成谎言）。

#### ⭐ 门本身的缺陷：`check-unwrap` 有一条**隐形债通道**（`690903a7`）

| 证据 | 数值 |
|---|---|
| 基线文件真实条目 | **705** |
| 门实际认账 | **514** |
| **隐形（静默丢弃）** | **191 条（27%）** |

**变异实验**（判定依据，非推断）：把一条**已失效**的 v1 条目改指向任意文件
⇒ 门仍输出 `baseline entries: 514` + `PASS: 0 new` ⇒ **完全无感**。

根因：v1 `path:line` 条目**仅当此刻该行号仍被检出**才折算成内容锚点，
否则**静默丢弃** —— 既不计入 honoured、不报失效、也不算 NEW。

修复：新增 `stale_baseline` 记录 + 显式输出 STALE 计数/样例/修复命令；
⛔ **不**改原「绝不无条件折算」的反洗白语义。修复后同一变异**立即被捕获**。
随后 `--update-baseline` 从**活站点**派生（⛔ 非手工删）⇒ **705 → 514**。

### ✅ 此前已清 24 处（27 → 4）

| 批次 | 内容 | 手法 |
|---|---|---|
| 1 | `nt_governance.rs` 6→0 | `re_opt -> Option<Regex>`，**消除「需要fallback 正则」的前提** |
| 2 | `nt_ecs.rs` | `match downcast()`（unwrap 纯冗余） |
| 3 | `main.rs` | `Runtime::new()` → 可读错误 |
| 4 | `nt_pet.rs` 2 | `be32_at() -> Option`（`get(..)` 去两层 panic） |
| 5 | `nt_io_output_style.rs` | core 侧同步 Option 化（消除**panic 分歧**） |
| 6 | `shanhai_query.rs` 3 | `Result` 传播 + 含**库路径**的可读错误 |
| 7 | `coverage_ledger.rs` 3 | 锁投毒按**本仓范式** `PoisonError::into_inner` 恢复 |
| 8 | memory_pack / pure_fns / rag / orchestrator / guardian / memory_filesystem / experience_memory 8 | 逐类：统一错误风格 / 去掉冗余 unwrap / `filter_map` 跳过不一致 / `Path::parent()` 用 `if let` / 时钟回拨用 `unwrap_or_default` |

### ✅ 那 3 处「需 API 变更」的阻塞，**本轮已各寻得正解**

| 原判 | 当时的结论 | 本轮实测的正解 |
|---|---|---|
| `core.rs:290` | 签名 `-> &GoalTracker` 表达不了失败 | ⛔ **判断错了**：`.expect` 逻辑上**恒成立**（上一行刚 `= Some(...)`）⇒ 正解是返回 `Option<&T>`，**13 处调用方全忽略返回值** ⇒ 零破坏 |
| `run.rs:712` | 「同一失败原因下的二次 open」 | ⛔ **低估了它**：不仅二次 open 无效，还在 `log::warn!("creating temp")` **谎称在降级** ⇒ 正解是 `Option` 表达缺失 + handler 降级为 no-op |
| `traits.rs:304` | 「刻意契约：失败即崩」 | ⛔ **漏了调用方全貌**：2 个调用方（`reddit.rs`/`instagram.rs`）本就返回 `Result` ⇒ 可迁 `?` ⇒ 函数变**零调用方** ⇒ 删除 |

⚠️ **我在这三处各试了 4+ 版**，包括发明 `open_transient`、用 `Default`、用下标索引、
`unwrap_or_else`、`unreachable!()` 换皮、以及批量正则改 `self.kb.*`
（误伤同文件 `ExperienceQuery.kb` ⇒ 23 编译错误 ⇒ 回滚）。

⇒ **教训**：反复失败说明**切面选错了**，不在于「改写不够巧妙」。
   反复失败的信号应当触发**换切面**（改调用方 / 改字段类型 / 删函数），
   ⛔ 而不是继续在函数体内试第 5 版。

### 逐类判据（供后续沿用）

1. 编译期常量正则 ⇒ **可以** `expect`（但本仓已统一 `Option`）
2. **运行时拼装**的正则（`format!` + 数据）⇒ 必须 `Option`/`Result`
3. Mutex锁投毒 ⇒ 按**本仓范式** `PoisonError::into_inner` 恢复，⛔ 不改 panic
4. 刚 `push`/`insert` 后的 `last()`/`get()` ⇒ 冗余，改用**已记录的下标/idx**
5. `SystemTime::duration_since` 失败（时钟回拨）⇒ `unwrap_or_default`（只是个临时目录名）
6. `Path::parent()` 为 `None` ⇒ `if let`，⛔ 不 `expect`
7. `expect` 出现在**函数签名不支持传播**处 ⇒ ⛔ 不硬改，归「需 API 变更」


门已可信（2026-10-07 修好两处盲区：测试块不检测 + 打印截断）。
⚠️ **`check-unwrap` 之前报 16 是打印假象**，真实值一直是 24+。

| 文件 | 行 | 备注 |
|---|---|---|
| `neotrix-core/src/bin/shanhai_query.rs` | 36, 37, 131 | bin 入口 |
| `neotrix-core/src/main.rs` | 1054 | bin 入口 |
| `crates/neotrix-neobot/src/nt_pet.rs` | 225, 226 | |
| `neotrix-core/src/l4_emotion/nt_memory/coverage_ledger.rs` | 388, 447, 464 | |
| `.../nt_memory_kb/nt_memory_pack.rs` | 554 | |
| `.../nt_memory_kb/nt_memory_search/nt_pure_fns.rs` | 4 | |
| `.../nt_memory_knowledge_graph/experience_memory/*` | 2 处 | |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/nt_core_rag.rs` | 227 | |
| `.../nt_mind/foundation/guardian.rs` | 931 | |
| `.../nt_mind/nt_mind/evolution/goal_loop/loop_impl/core.rs` | 1 处 | |
| `.../nt_mind/nt_mind_background_loop/run.rs` | 712 | |
| `neotrix-core/src/l2_perception/nt_world/social_access/traits.rs` | 295 | |
| `neotrix-core/src/l0_substrate/nt_ecs.rs` | 528 | |
| `.../nt_crystal_core/agent_orchestrator.rs` | 91 | |
| `.../nt_mind/self_improvement/memory_filesystem.rs` | 181 | |

**修法判据**（照 `nt_governance` 的成功路径）：
1. ⛔ **不要**用「fallback 成永不匹配的值」来消除 `unwrap` —— 那需要再构造一次
   ⇒ 要么 `expect`、要么 `unreachable!()`（**自身即 panic**）。⇒ 应**消除该前提**。
2. 构造可能失败的运行时值 ⇒ 返回 `Option`/`Result` 并让**消费点显式早退**。
3. 降级语义必须区分「**无操作**」（返回空/None）与「**破坏**」（清空/默认值）。
4. 每批修完必跑：目标包测试 + `check-unwrap` NEW 数下降 + **探针 RC=0**。

✅ 已完成（棘轮 27 → **17**，逐批验证 NEW 下降 + 探针 RC=0）：

| 批次 | 内容 | 手法 |
|---|---|---|
| 1 | `nt_governance.rs` **6 → 0** | `re_opt -> Option<Regex>` 全链路，**消除「需要 fallback 正则」这个前提** |
| 2 | `nt_ecs.rs:528` | `is::<T>() + downcast().unwrap()` → **`match downcast()`**（unwrap 本就冗余） |
| 3 | `main.rs:1054` | `Runtime::new().expect("tokio")` → `match` + **可读错误**（裸 panic 信息量为零） |
| 4 | `nt_pet.rs:225/226` | 新增 `be32_at() -> Option<u32>`（`get(..)` 而非 `[..]`，⛔ 切片越界本身也是 panic） |
| 5 | `nt_io_output_style.rs` 全套 | core 侧与 neobot 侧**分歧**（一份 panic 一份降级）⇒ 同步 `re_opt` |
| 6 | `shanhai_query.rs`×3 | `open_kb() -> Result` + 可读错误（含**库路径**）；`fs::write` 失败给路径 |
| 7 | `coverage_ledger.rs`×3 | 锁投毒按**本仓既有范式** `PoisonError::into_inner` 恢复（见 c022bfab）；`entries.last().unwrap()` ⇒ 用已记录的 `idx` |

## D2 —— 死配置（**184 待判定**：bool 133 + numeric 51）

⭐ **本轮把「未接线规格」从 5 降到 0**，且发现其中多数标注**本身是错的**。

零读点的 `pub` 配置字段。**批量 grep 会误判**（AGENTS.md R-SCAN-1b）：
必须逐个**读现场**确认，且**同名 ≠ 同一符号**（`window_width` 在
`nt_game/render` 是死配置，在 `nt_shield_stealth_net` 有外部消费者 ⇒ 不可连坐）。

✅ 已完成：`nt_game/render` 的 `window_width/height`（零读点，全仓引用 0）。

## D2-a ✅ 分类：`memory_types.rs` 的工作流/子任务子系统**整体已死**（实测）

### 证据链（逐级核实，非推测）

| 判定对象 | 实测 | 结论 |
|---|---|---|
| `WorkflowMemory` / `SubtaskMemory` / `WorkflowStep` | 全仓**无构造点**（`success:` 的命中全属别的类型） | 从未被实例化 |
| `_record_workflow` / `_record_subtask` / `_search_workflow` / `_search_subtask` | 类型文件外引用各 **0** | 死方法 |
| 该文件的 `MemoryStore` **struct** | 全仓引用 **0**（唯2 处命中是`memory_bank.rs` 里**另一个同名 trait**） | 死类型 |

⚠️ **`MemoryStore` 是同名双胞胎**：`memory_types.rs` 的 struct
与 `l5_cognition/nt_mind/foundation/memory_bank.rs:89` 的 trait **同名但不同物**
⇒ 这正是 AGENTS.md 「**同名 ≠ 同一符号**」的又一次实例。

### ⭐ 我自己踩的坑：grep 被 `.worktrees/` 污染

第一次统计 `WorkflowStep` 得到「56 处引用」⇒ 差点得出「它很常用，不能删」。
**真相**：那 56 处里大量来自 `.worktrees/**` 的**其他工作树副本**。
⇒ 排除 worktrees/target 后是 **10 处**，且**全部指向另一个定义**
（`production_orchestrator.rs:32`，字段是 `id`/名称…，与本处不同）。

⇒ **纪律补充（判据第 4 条）**：`grep -rn` **必须显式限定目录**
（`neotrix-core/src crates/ apps/`），⛔ 不能用 `.`，
否则 `.worktrees/` / `target/` 会把计数**放大数倍** ⇒ 结论直接反向。

### ✅ 处置：**已删 47 行**（2026-10-07）

| 删除 | 依据 |
|---|---|
| `WorkflowMemory` / `WorkflowStep` / `SubtaskMemory` | 全仓零消费者、**零构造点** |
| `TripleMemoryStore.workflows` / `.subtasks` 字段 | 元素类型已删 ⇒ 恒空容器 |
| `_record_workflow` / `_record_subtask` / `_search_workflows` | 文件外引用各 **0** |
| `stats()` 三元组 → `usize` | 前两项**恒为 0** ⇒ 「看起来是统计，实际是假的」 |

**保留**：`FunctionMemory` + `_record_function_call` + `_function_stats`
（`success_count` **确实被读**，算 success_rate）· `pub mod memory_types;` 声明。

⚠️ **又一次踩到 E0119**（孤立 `#[derive]` 残留）—— 这是配平法的**第三次**实证
⇒ 判据第 8 条已两次补充：**删除后必须 build**，
且**配平法只解决花括号，解决不了注释归属**。

⇒ 已从「188 待判定」中**精确分类出这 3 个 `pub success`**：
它们是**只写不读**的标记（对照：`FunctionMemory.success_count` 确实被读）。

## D2-b ⚠️ 「死配置」有**两种相反含义** —— 批量删除会销毁规格

### 最大单文件簇：`BackgroundConfig`（**18 / 42 字段装饰性**）

取门的裁决（非我自写扫描器 —— 我第一版自写扫描器把声明本身算成读点，
**全部误判为「有读点」** ⇒ 又一次「不要重写已有的门」）。
位置：`l5_cognition/nt_mind/nt_mind_background_loop/config.rs`

| 类型 | 死字段 | 明细 |
|---|---|---|
| bool | **6** | `enabled` · `proxy_enabled` · `system_proxy_enabled` · `geo_auto_update` · `agent_protocol_enabled` · `enable_exploration` |
| numeric | **12** | `save_interval_secs` · `consolidate_interval_secs` · `evolve_interval_secs` · `cleanup_interval_secs` · `mine_interval_secs` · `metacog_interval_secs` · `thinking_interval_secs` · `geo_update_interval_hours` · `telemetry_interval_secs` · `nt_world_crawl_interval_secs` · `prediction_interval_secs` · `panorama_interval_secs` |

⇒ **43% 的配置面在控制空气。** 结构体本身是活的（5 处外部构造引用）。

### ⭐⭐ 关键区分：「死配置」有**两种相反**成因

| 成因 | 含义 | 正确处置 |
|---|---|---|
| **A. 已废弃** | 该功能已被替代/移除，字段是残留 | ✅ **删除** |
| **B. 未接线规格** | 功能**声明了但从未实现**，字段是「意图声明」 | ⛔ **必须保留**并标注 |

⇒ **B 类占多数**（如 `enable_exploration`、`agent_protocol_enabled`
明显对应真实功能名，只差读者没写）。
⇒ **批量删除 B 类 = 把「要实现什么」的规格从代码里抹掉**
⇒ 下一个 agent 读不到这些字段，就**再也不会知道探索功能本该存在**。

⇒ ⇒ **因此D2 不能按「门报了就删」推进。**
每个字段必须先回答：**「这个功能是没做，还是不做了？」**
—— 这是**意图问题**，门无法回答（门只能回答「有没有人读」）。

### ✅ 门已增强：第四种分类「**未接线规格**」（2026-10-07）

 新增第4 类，判据 = **字段上方 6 行内**出现
`nt-unwired-spec` 或「未接线规格」。

⭐ **标记必须长在字段旁边**（⛔ 不用外部清单文件）
⇒ 否则清单会与代码漂移，变成**第二份「关于代码的说法」** —— 那正是本仓的主要缺陷形态。

**实测效果**：

| 类型 | 原待判定 | 现待判定 | 未接线规格 |
|---|---|---|---|
| bool | 132 | **126** | **6** |
| numeric | 52 | **40** | **11** |

⇒ 已把 `BackgroundConfig` 的 19 个字段标注（其中 `enable_auto_crystallize`
**有读点** ⇒ 门如实不计入未接线规格 ⇒ **19 标注 / 17 生效**，
差额来自门的事实裁决，**不是我手改清单**）。

⭐ **变异验证**（同一字段 `system_identity.rs` 的 `enabled`）：
- 加标记 ⇒ 判「**未接线规格**（⛔ 不要删）」
- 去标记 ⇒ 判「**待人工判定**」
⇒ 证明新分类**真的改变判定**，⛔ 不是空转。

**剩余处置**：仍需逐个判定 A（已废弃 ⇒ 删）/ B（未接线 ⇒ 实现）。
本提交只做了**分类与标注**，⛔ 未删任何字段。

## ⭐ D3 —— 冗余：**已清 ~347 行**（原为「输出治理规则在两个 crate 各存一份」）

✅ 已完成：治理规则下沉到 neobot 真身（`nt_io_output_style.rs` 1091 → **791**，
删 22 项逐字重复、约 300 行）+ `memory_types.rs` 132 → **85**（删死类型/字段/方法）。
⚠️ 保留：`OutputStyleId` 容器 D3-Phase-2 未做（属 D3 剩余项，见下）。

### 实测：逐字相同的重复项共**22 个**

| 类别 | 数量 | 验证方式 |
|---|---|---|
| 规则本体 `r1_answer_first` … `r8_hallucinated_paths` | **8** | 逐函数 diff **为空** |
| 助手 `extract_path_refs` / `re_opt` / `has_known_ext` / `mask_code_fences` / `strip_pure_placeholder_lines` | 5 | 同上 |
| 常量 `EXTS` / `PLACEHOLDER_PURE_RE` / `PLACEHOLDER_INLINE_RE` | 3 | 同上 |
| `RuleResult` 结构体 | 1 | 字段逐字相同（`rule_id/passed/detail`） |
| `build_rules` 本体 | 1 | 72 vs 75 行，**仅闭包 arity 不同** |

位置：`crates/neotrix-neobot/src/nt_governance.rs` 与
`neotrix-core/src/l1_action/nt_io/nt_io_output_style.rs`。

### ⭐ 关键发现：闭包差异是**历史遗留**，不是设计

core 侧 8 个闭包都写成 `|text, _style|`（`_` 前缀 ⇒ **从未被使用**），
而调用点 `(rule.check_fn)(text, style)` 确实传了 `style`。
⇒ `OutputStyleId` 是**为未来 style-aware 规则预留**的，当前零消费。
⇒ ⇒ **规则逻辑其实完全相同**，只有**容器类型**因多一个参数而不同。

### ⛔ 下沉的三个真实阻塞（我逐一撞过，非推测）

1. **E0116**：`impl RuleResult { fn pass/fail }` 在 core 侧 ——
   类型若改为从 neobot 引入，则**不能在本 crate 定义 inherent impl**。
   ⇒ 必须把 `pass`/`fail` 一起搬进 neobot 并 `pub`。
2. **E0624**：`RuleResult::pass/fail` 当前是**私有** ⇒ 跨 crate 不可用。
3. **E0774**：删本地 `RuleResult` 时误吞了相邻 `#[derive(...)]`
   ⇒ ⇒ **删除必须按花括号配平定位**，不可按行号区间（我按行号删过一次，炸了）。

### 建议的下沉路径（下一窗口可直接执行）

**✅ Phase 1 已完成（2026-10-07）**

| 项 | 结果 |
|---|---|
| 删除的重复项 | **16 项**（8 `r*` + 4 助手 + `RuleResult` + `impl{pass,fail}` + `EXTS` + …） |
| `nt_io_output_style.rs` | **1091 → 791 行（-300）** |
| 行为变更 | **零**（core lib +13,589 绿 · neobot 569 绿 · 4 门 RC=0） |
| `pass/fail` | 已 `pub`（core 跨 crate 需要） |
| 依赖 | `core → neobot` 已存在 ⇒ **零新 crate** |

⚠️ 执行中踩到 **E0119**（孤立 `#[derive]` 被留下）：我的配平删除器
把 `RuleResult` 上方的 `///单条规则的检查结果。` 当成它自己的注释，
于是留下一条**孤立 derive** 挂在 `GovernanceReport` 上。
⇒ 教训：配平法对**注释归属**仍然会误判 ⇒ 删除后必须 **build**，
⛔ 不能只看「括号配平成功」。

**Phase 2（需决策）**
`GovernorRule.check_fn` 是否**去掉 `OutputStyleId`**？
· 去掉 ⇒ 容器也统一，**22 项重复全部消除**
· 保留 ⇒ 容器仍是两份（但只是**壳**，逻辑零重复）
⇒ 我倾向**保留**（参数是为未来预留，删它会阻断后续 style-aware 规则），
但需在文档写明「当前零消费」，避免下个人以为它在起作用。

⚠️ **依赖方向天然合适**：`neotrix-core` 已依赖 `neobot` ⇒ **零新 crate**。

## D4 —— 命名（1615，advisory）

`check-naming.sh` 是 advisory：clean-HEAD 实测 1615 无前缀文件。
⇒ **规约 vs 现实差 1615 ⇒ 该规约当前无约束力**。
⇒ **不要**直接设为门。**先抽 1 层**（如 `l0_substrate`）验证可行性再谈门禁。

## D5 —— 能力未接线（3 DeclaredOnly + 1 Scaffold）

| 能力 | `executability` | 阻塞 |
|---|---|---|
| `trade_quote_negotiation` | `Executable` | —（已接线） |
| `foreign_trade_full_cycle` | `Scaffold` | 17 阶段业务逻辑**全是注释** |
| `trade_production_logistics` | `DeclaredOnly` | 顶层执行器数 0 |
| `trade_finance_compliance` | `DeclaredOnly` | 顶层执行器数 0 |
| `trade_product_spec` | `DeclaredOnly` | 只有知识包工厂 |

⛔ **需权威输入 schema**，发明即造假 ⇒ **不能靠适配器绕过**。
✅ 现状已在**三处**诚实标注（代码头 / 市场描述 / 本清单）。

## ✅ D7 已修：baseline 键改为**内容锚点**（行号漂移免疫）

**病根**：账本以 **`路径:行号`** 为键 ⇒ 上游任何编辑都可能让「同一位点」
看起来是「新增」（假警报），或让既有的一条看起来「已修」（棘轮失真）。
⇒ 实测旧键已陈旧 **132** 条。

**修法**：键改为 `路径 \x1f 所属函数名 \x1f 归一化代码行`
（`\x1f` 是不可见分隔符，输出时还原成可读形式）。
· 所属函数 = 向上找最近的 `fn` 定义行（找不到记`<toplevel>`）
· 归一化 = 去掉所有空白 ⇒ **纯格式调整不触发新增**

**迁移安全设计**（关键）：
旧账本条目**仅当该位点此刻仍被检出**时才折算成锚点。
⛔ 绝不无条件折算 —— 那 132 条「已不存在」的若被折算，
会凭空造出 never-seen 的锚点 ⇒ **等于洗白债务**。

⚠️ **我自己引入的回归（已修）**：第一版输出直接打印**原始锚点**
（含不可见 `\x1f`）⇒「按 `path:line` 查」这一最常见动作失效，
而我却在提交信息里声称「已还原成可读形式」—— **那次替换随断言失败一起回滚了，
我却没核对**。⇒ 现输出 `路径:行号  [函数]  归一化代码`，**比原来更好定位**。

**验收（含漂移实验）**：

| 实验 | 结果 |
|---|---|
| 探针 | **RC=0** |
| 基线 | 532 站点 / **515** 基线（= 只折算仍存在者） |
| NEW | **17**（与换键前一致 ⇒ 未洗白任何债） |
| **在已记账文件顶部插入 1 行** | NEW 仍 **17** ✅ 漂移免疫 |
| core 测试 | **13,589 绿**（连跑 3 次） |

⚠️ 一次测试曾出现 `13588 passed; 1 failed`，连跑 3 次均 `13,589 passed`
⇒ 判定为 **flaky**（时序相关），**如实记录，不当绿灯**。

## D6 —— 需外部裁决（非代码可解）

- `.neotrix/LICENSE-EXCEPTIONS.md` `status: void` ⇒ 需**所有者签署**
- `.project-map/` 354M ⇒ 需**体积策略**（改 `.gitignore` 属另一窗口的未提交改动）
- 3 个能力的业务逻辑缺口 ⇒ 需**产品/协议侧**给定 schema

## 执行顺序（按「价值 ÷ 风险」·2026-10-07 刷新）

| # | 项 | 状态 | 说明 |
|---|---|---|---|
| 1 | **D1** unwrap | ✅ **已完成** | `--strict` 首绿；存量 514 全部记账 |
| 2 | **D3** 冗余 | ✅ **主体完成**（-347 行） | 余 `OutputStyleId` 容器（D3-Phase-2） |
| 3 | **门本身** | ✅ **本轮新增** | `check-unwrap` 隐形债通道已封（`690903a7`） |
| 4 | **D2** 死配置 | ⏳ **剩 184 待判定** | ⛔ 禁止批量 grep 判定；实测比率约 **1/4 会翻车** |
| 5 | **D4** 命名 | ⏳ 1615（advisory） | 先切片验证；⛔ 规约与现实差 1616 ⇒ 不宜直接门禁 |
| 6 | **D5** 能力 | ⛔ 缺权威 schema | 3 DeclaredOnly + 1 Scaffold |
| 7 | **D6** 外部裁决 | ⛔ 非代码可解 | `LICENSE-EXCEPTIONS` 仍 `void` |

⭐ **D2 的下一步为什么不是「继续扫」**：
本轮实测每个真接线需读 2–6 个文件的证据链，且**约 1/4 会翻车**
（同名异 struct / 跨文件互救活 / 注释触发门标记）。
⇒ 184 条预计需读 500+ 文件，**批量误判会污染基线**，比不做事更糟。

⇒ **更优路径**：把 184 条按**目录**分片，每片先出「读现场清单」再动手，
   ⛔ 禁止一次性批量判定。

## 本清单的纪律

- ⛔ 数字只能来自门/工具输出（R-SCAN-3）
- ⛔ 每条修法必须写**判据**，不只写「改成 X」
- ⛔ 阻塞项要写明「**为什么不能靠代码解决**」

---

## 架构 / code map 状态刷新（2026-10-07 实测）

### 门矩阵（14 个）

🟢 **12 绿**：`unwrap` ⭐·`executor-registry` · `silent-failure` · `doc-drift`
· `dead-config-flag` · `orphan-dirs` · `doc-claims` · `claims-numbers`
· `agent-config` · `layer-deps` · `map-check` · `test-baseline`
🔴 **1 红**：`naming` 1615（**advisory**，规约 vs 现实差 1616 ⇒ 该规约无约束力，
advisory PASS ≠ 合规；⛔ 不宜直接升级为门禁）
🟢 元门：**未登记 0 · 恒红 0 · 探针失败 0**

⭐ **本轮变化**：`check-unwrap` 由红转绿（`--strict` 首次 PASS）⇒
元门「恒红 1」归零 ⇒ 这是本会话在**门维度**最实质的变化
（此前 4 个门共 24 次提交都在治「门报得对不对」，本轮是**门本身变绿**）。

### 测试

| 包 | 结果 |
|---|---|
| `neotrix`（core） | ⭐ **13,599 绿** / 38 ignored |
| `neotrix-neobot` | **569 绿** / 1 ignored |
| `nt-core-capability-tree` | **63 绿** |

### 本会话对「审计/测试元能力」的增强（**这是元认知基础设施的实质变化**）

| # | 增强 | 性质 |
|---|---|---|
| 1 | `check-executor-registry`（新建） | 让「能力能否执行」成为**可对账字段** |
| 2 | `nt_ok_audit.py`（新建） | `.ok()` 审计工具化，**排除 `#[cfg(test)]` 区间** |
| 3 | `unwrap` 门**测试块检测**修复 | `in_test_block` 永不复位 ⇒ **盲区** |
| 4 | `unwrap` 门**打印截断**移除 | `[:8]` 使探针**只在 NEW ≤ 8 时有效** |
| 5 | `unwrap` baseline**内容锚点** | 行号漂移免疫（实测插入 1 行 NEW 不变） |
| 6 | `doc-drift` **账本分离** | 两类别塞一文件 ⇒ 恒红是**格式碰撞** |
| 7 | `silent-failure` **opener 放宽** ×2 | 只认 `let _ =` ⇒ 裸语句/命名绑定不可见 |
| 8 | 6 个门/工具登记 `task-index` + `gate-registry` | 新建资产**可被发现** |
| 9 | `DEBT-LEDGER-2026-10-07.md` | 债务**权威单一来源**（取代散落记录） |

⇒ **9 项中 5 项是「修门本身的盲区」** —— 而不是我修了什么业务 bug。

### 本会话沉淀的判据（已全部写入清单，可执行）

D1 的 **7 类错误判据** + 全局 8 条：
1. 结论自洽时→ `git log -S` 追首次出现
2. 承重断言 → **变异测试**（去掉它是否真的失败）
3. 门红了 → 先问「**门坏还是债错**」
4. grep 命中 → **读那一行**（同名≠同一符号）
5. 入口能用 → 先确认**实现与入口在同一依赖闭包**
6. 从门输出推导数字 → 先确认**没截断**
7. 连续 N 次失败 → **先怀疑实验装置**
8. 删除代码 → **花括号配平**，⛔ 不按行号区间

### 未完成 / 阻塞（诚实）

| 项 | 阻塞原因 |
|---|---|
| D1 剩 3 处 | **正解在函数签名上**（需 API 变更，见清单逐一说明） |
| D2 188 死配置 | 需**逐个读现场**（⛔ 禁止批量 grep 判定） |
| D3 冗余 22 项 | 3 个具体阻塞已记录 + 两阶段路径 |
| D4 命名 1615 | **规约 vs 现实差 1615** ⇒ 当前无约束力，先切片 |
| D5 能力 4/5 未接线 | **缺权威输入 schema** ⇒ 发明即造假 |
| 外部技术吸收 | 许可证边界 + 上述 schema 缺口 |

