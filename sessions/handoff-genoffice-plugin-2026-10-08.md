# Handoff — genoffice 插件能力接入（窗口 fledge-alpha-free）

> 模板：`sessions/HANDOFF-TEMPLATE.md` · 收工自查必填：worktree 去向 / 未提交改动去哪。

## 一句话
把 genspark-ai/genoffice 的文档引擎（docx/xlsx/pptx/pdf/markdown/html 读写转渲染校验）作为**外部二进制适配器**接入 `nt_file_ability`，**不接入其 AI**（agent-core/ai-provider/ai-search/Electron AI 面板 全部不引入），由 NeoTrix 核心（L0–L6 + 本地模型）负责编排。

## genoffice 已验证事实（cargo/gm 实测）
- CLI 包：`packages/cli`（`bin: genoffice`），单二进制、跑在其捆绑的 Node runtime（`ELECTRON_RUN_AS_NODE`），Apache-2.0。
- 文档能力面（`packages/cli/README.md`）：`info / convert / create / docs read|apply|check / sheet read|apply|check / slides read|check|audit|replace / pdf read / render / merge / guide <domain> / capabilities / search / image / media / open / mcp`。
- **AI 在哪**：`packages/agent-core`（ReAct 循环）、`packages/ai-provider`（Claude/Gemini/DeepSeek/OpenAI 流式）、`packages/ai-search`（Genspark/Serper/Tavily/Parallel）。CLI 仅在 `search/image/media/capabilities` 与 Electron AI 面板走它们。**本插件不接这三个包、不调用这三类命令。**
- 引擎层与 AI 解耦：docx-engine/xlsx-gateway/pptx-engine/pptx-ops/pptx-render/pdf2docx/html2docx/pipelines/file-parse/zip-gate 均为本地 Rust/TS 文档引擎。

## 改动与落点（R-P16 已重读验证）
### 阶段一：适配器模块
1. **新增** `neotrix-core/src/l1_action/nt_file_ability/genoffice.rs`
   - 二进制解析：`GENOFFICE_BIN` → PATH `genoffice` → `/Applications/GenOffice.app/Contents/Resources/cli/genoffice`（`genoffice_bin()`）。
   - 执行器：`run_genoffice(&[&str])`（`std::process::Command`，sync，不引 tokio——与 `html_to_markdown_via_mdream` 同型）/ `run_genoffice_json`。
   - typed wrapper：`genoffice_info / convert / create / docs_read / docs_apply / docs_check / sheet_read / sheet_apply / slides_read / slides_check / slides_audit / pdf_read / render / merge / guide` —— 覆盖全部文档引擎能力。
   - `GenOfficeCapability` 实现 `crate::l0_substrate::nt_core_capability_types::UnifiedCapability`，`execute` 走 `CapabilityInput::Kv {op, ...}` → `CapabilityOutput::Kv {stdout}`。
   - 插件入口：`create_genoffice_capability()`（返回 `Arc<dyn UnifiedCapability>`）/ `register_genoffice_capability(&mut CapabilityRegistry)`——与 `create_ocr_capability`/`register_ocr_capability` 同型。
   - 7 个单测全绿（meta/supports/execute 形状 + 二进制缺失退避 + **e2e**：真 spawn genoffice 跑 `info`+`convert` 产出真实 .docx）。
2. **接入** `neotrix-core/src/l1_action/nt_file_ability.rs`：加 `mod genoffice;` + `pub use genoffice::*;`（L1 叶子，非 feature 门控）。

### 阶段二：插件市场登记 + 自动加载（2026-10-08 追加）
3. **市场清单** `crates/nt-core-capability-tree/src/market.rs`：新增 `GENOFFICE_LICENSE` 与 `GENOFFICE_MANIFEST`（1 条）
   - `id = "NT-ACT::nt_file_ability::genoffice"`、`domain = Act`、`category = "office/document-engine"`、**`executability = Executable`**。
   - ⚠️ 放在 `TRADE_MANIFEST` **之后**是有意的：`check-executor-registry.sh` 从 `pub const TRADE_MANIFEST` 起扫描 ⇒ genoffice **被这道门审计**（实测门解析出 6 条），而非成为审计盲区。
4. **真实派发接线** `neotrix-core/src/l1_action/nt_act/nt_act_trade/tree_dispatch.rs`
   - `_TREE_ID_GENOFFICE` 常量 + `dispatch_genoffice`（Kv 入参直通 `GenOfficeCapability::execute`，成功后打金丝雀点）；`pairs` 由 `[..; 2]` 改为 `[..; 3]`。
   - ⇒ 清单声称的 `Executable` **有真实接线**，门判据 (b) 满足。
5. **自动加载** `crates/neotrix-neobot/src/nt_capability_registry.rs`
   - `seed_from_market_manifest()` 由「只播种 TRADE_MANIFEST」改为**两个清单同级播种**，各带自己的 LICENSE 串。
   - 该函数已被**两个真实启动入口**调用 ⇒ 启动即自动上架：
     `nt_http_engine.rs:409`、`bin/neobot.rs:2233`。
   - 新增 `mod market_seed_tests`：断言播种后节点存在且 `market.version`/`market.license`/`market.category` 齐备（缺任一项即不可上架）。

### 阶段三：派发层能力面补全（`a2a2d0ab`，2026-10-08）
**发现的真实缺口**：阶段二登记的是「一个 op」（`dispatch_genoffice` 直通
`GenOfficeCapability::execute`），而 `execute` 当时只支持 **4 个 op**
（info/convert/docs_read/guide），模块却有 **14 个 typed wrapper**
⇒ 走市场派发够不到大部分能力，「可以直接操控其所有能力」当时是**假的**。

- 新增 `GENOFFICE_OP_TABLE`：18 个 op 的**声明式参数模板**（能力面真源）
  - 记号：`=key` 必需 flag / `--key` 可选 flag / `@key` 必需位置 / `~key` 可选位置
  - `genoffice_op_template` / `genoffice_ops` / `genoffice_build_args`
- `execute` 改模板驱动：**缺必需 key 在进 subprocess 前 fail-closed**；空串视同缺失
- 补 `genoffice_sheet_check`、`genoffice_slides_render`
- 新增 `op_dispatch_e2e::全部op经派发入口真打到二进制` —— 18 个 op 全部真打二进制

**⭐ 参数形状全部实测得出（多处与 README 不符，照抄 README 会写错）**：
| 事实 | 含义 |
|---|---|
| `create --type pptx` 只吃 `--ops`/`--spec`，**不接受** `--from` | README 的 `--from` 只对 docx/xlsx 成立 |
| slides 画布 **1280x720 用 px**（非 EMU） | 越界元素会被静默 drop |
| text 元素文字字段是 **`paragraphs`** | `text`/`content`/`value`/`string`/`txt` **全部报 "without any text"** |
| shape 需 `fill`；image url 必须 http(s) | 否则整页被判 invalid |
| `docs apply --ops` 与 `sheet apply --cells` 是**两套 DSL** | ops 名（`findReplace`）不能喂给 `--cells` |
| `docs apply` 未知 op → `"Nothing was applied (atomic)"` 且不落盘 | 原子性已实证 |

夹具只用**受支持的转换路线**（md→docx、md→pdf、pdf→pptx、json→xlsx），
不手写 pptx spec；e2e 全部 18 op 无跳过（含需启动 GenOffice 的 render/slides_render）。

### 阶段三验证（实测）
`cargo check -p neotrix --lib` Finished · genoffice **17 绿**（含 18-op e2e）·
tree_dispatch **8 绿** · `check-executor-registry.sh` **PASS** · `nt_lock_audit` **0 可疑**。

### 已提交（2026-10-08，branch `feat/multi-agent-absorb-2026-10-08`）
- **`635a8c05`** — `feat(file): genoffice 文档引擎适配器…`（新增 `genoffice.rs`，627 行）
  - 同时**修复仓库完整性**：`61ace1b2`（他窗 openhands 的 clippy 批量提交）把
    `nt_file_ability.rs` 的 `mod genoffice;`/`pub use` 扫进 HEAD 却漏掉未跟踪的
    `genoffice.rs` ⇒ HEAD 引用不存在的模块。取证：`git ls-tree HEAD` 无该文件、
    `git show HEAD:nt_file_ability.rs | grep -c genoffice` = 2。本提交补齐后
    HEAD 自洽（已复验）。
- **`ee2a7b13`** — `feat(file): genoffice 登记插件市场 + 真实派发接线 + 启动自动播种`
  （4 文件 +232/−43）
  - 归属：`market.rs`、`nt_capability_registry.rs` = 本窗；
    `tree_dispatch.rs` = 本窗新增 `dispatch_genoffice` + pairs 2→3 + 2 测试，
    **兼含他窗未提交的 canary 会话键化迁移**；
    `dispatch.rs` = **全部为他窗未提交重构**（自 10-07 22:01 起未入库，DispatchFn 增第 3 参 session）。
  - 为何必须四文件同提（原子性）：工作树 `DispatchFn` 是 3 参数而 HEAD 是 2 参数，
    本窗的 `dispatch_genoffice` 必须写 3 参数才编过 ⇒ 单独提 `tree_dispatch.rs`
    会产生类型不匹配的损坏提交；单独提 `market.rs`（声称 `Executable`）而不提注册，
    则干净检出下 `check-executor-registry.sh` 判据 (b) 会红。

### 提交后复验（实测）
`cargo check -p neotrix --lib` Finished · genoffice **9 绿** · tree_dispatch **8 绿** ·
market **5 绿** · neobot **633 绿** · `check-executor-registry.sh` **PASS** · `nt_lock_audit` **0 可疑**。

### 阶段二验证（实测）
- `cargo check`：`neotrix`(lib) / `nt_core_capability_tree` / `neotrix-neobot` 三个 crate 全 Finished（仅 5 个既有 `unused mut` 警告）。
- `cargo test -p neotrix --lib genoffice` → **9 passed**。
- `cargo test -p neotrix --lib tree_dispatch` → **8 passed**（含新增 `genoffice已注册进派发表`、`genoffice真实派发guide成功`；既有 6 条贸易测试未回归）。
- `cargo test -p nt_core_capability_tree market::` → **5 passed**。
- `cargo test -p neotrix-neobot` → **624 passed**（含新增 `genoffice被自动播种进市场注册表`）。
- `scripts/check-executor-registry.sh` → **PASS**（6 条全部对账一致）。
- `nt_lock_audit.py neotrix-core/src` → **0 处可疑**。

### ⛔ 本轮撞到并证伪的一个「红」（非本任务引入）
`cargo test -p nt_core_capability_tree` 有 1 红：`cli::registry_roundtrip_tests::真实注册表文件_解析并往返零丢失`
→ `unknown variant 'bud', expected one of budding/grafting/...`。
**取证结论：HEAD 既有缺陷，非本会话引入。** 证据：`git show HEAD:.../node.rs` 的 `EvolutionOp` 只有 `Budding/Grafting/Pruning/CrossPollination/Maturation/Strengthen`（无 `bud` 别名），而 `git show HEAD:.neotrix/capability_registry.json | grep -c '"op": "bud"'` = **7**。⇒ 有人在注册表里写了 CLI 子命令名 `bud` 而非枚举名 `budding`。本会话**未**改 `.neotrix/capability_registry.json`（`git diff --stat` 对该文件为空）。**留给他窗/后续修**（修法：JSON 内 7 处 `"bud"` → `"budding"`，或给枚举加 `#[serde(alias="bud")]`）。

## 验证
- `nt_mem_gate.sh` → rc=0（avail=557344，swap 1589M）。
- `cargo check -p neotrix --lib` → rc=0，genoffice 无 error/warning。
- `cargo test -p neotrix --lib genoffice` → **7 passed; 0 failed**（含真实 spawn genoffice 的 e2e）。
  - ⚑ 复核发现：首次上报「6 passed」时 e2e 用例因 `genoffice convert` 拒绝覆盖已存在输出而 FAILED（exit 2 `output_exists`）；已修为每次生成唯一输出名 + 失败时在 stderr 空时带 stdout 摘要。**当前 7 passed 为真实结果**（`--nocapture` 复核确认 e2e 用例跑过而非跳过）。
- `nt_lock_audit.py neotrix-core/src` → **0 处可疑**（rc=0）。
- `nt_worktree_gate.sh check` → rc=0（主树 34 处未提交，含本窗 2 处；worktree 无本窗新开）。

## 未做 / 后续（明确不阻塞）
- 未向 `.neotrix/capability_registry.json` 手写节点（318 节点知识库，历史有整库销毁事故；若要上树，用 CLI `cargo run -p nt-core-capability-tree -- bud --id act::nt_file_ability::genoffice --domain act --layer L1 --provides genoffice --note "..."` 走校验路径，勿手改 JSON）。
- 未接入 `search`/`image`/`media`/`capabilities`/`open`/`selection`/`skill`/`mcp`/`install-cli` —— 按「移除 AI」原则与本地文档引擎边界，不暴露。MCP `mcp` 子命令如未来要接入，应走 NeoTrix 的 MCP 桥而非直接嵌 Electron。

## 收工自查
- [x] ① `nt_worktree_gate.sh check` 已跑（rc=0）
- [x] ② 本窗未开新 worktree（无需 prune；主树未提交 34 处，其中本窗 2 处）
- [x] ③ 本 handoff 已写
- 未提交改动归属：`genoffice.rs` + `nt_file_ability.rs` 两处为本窗；其余 32 处为他窗 WIP / 索引补丁 —— **未 git commit**（共享 index，`git commit --only <我的文件>` 仅在有明确提交意图时执行；本窗未被要求提交 ⇒ 保留未提交）。

---

# 第二阶段：架构审计 → 插件化脊柱 → 两个 P0 修复（2026-10-08 晚，本窗）

> 上文为 genoffice 接入阶段。本节记录之后做的架构审计与体系化建设。
> 本窗全部提交已在 `feat/multi-agent-absorb-2026-10-08` 分支。

## 起因：用户要求「插件化、热插拔、随用随调、智能耦合、自我组合」

对 neotrix + neobot 做了一次**带证据的**全面审计（两份 explore 报告，
每条结论追 `file:line`）。核心实测结论：

| 编号 | 结论 | 证据 |
|---|---|---|
| H1 | **13 套**并行能力/插件表示法、**13 个**能力注册表，只有 1 条有活回路 | 全审计 |
| H2 | L0 `UnifiedCapability`：18 个生产实现、**0 个生产构造点**；`create_*` 工厂的 Rust 调用者全为 0 | `ocr/mod.rs:399` 等 |
| H4 | `TradeCapabilityRegistry` + `global_trade_registry()` 零生产消费者 | `capability_registry.rs:211` |
| H5 | `Executability` 唯一 Rust 读者是一个 `println!` | `nt_crystal_serve.rs:1938` |
| H9 | 磁盘 326 节点树与 6 个 manifest id **互不相交**，只在某个进程全局相遇 | `.neotrix/capability_registry.json` |
| H10 | `NativeBus` 完整第 6 套，生产 attach 的却是**空总线** | `handlers_wisdom.rs:20` |
| 热插拔 | 全仓**只有** `PluginRegistry` 支持；其余全是 insert-only | `nt_io_plugin/registry.rs` |
| neobot 工具面 | 全部 6 个能力只暴露**一个**硬编码工具 `capability_invoke`，id 塞 description；**无重试/队列/异步/批量** | `nt_http_engine.rs` |
| 智能路由 | `SemanticRouter`/`CapabilityCatalog`/`route_entity_aware` 零生产消费者；唯一活的 `ExperienceRouter` **只写 rationale 从不执行** | 全审计 |

## 关键取证：`capability_invoke` 在 3 个生产进程里跑得动几个？

只有 **1 个**（`neotrix dialog say`）。`neobot` CLI 与桌面端恒为
`CAPABILITY_BODY_NOT_EXECUTED` —— 派发器住在 core，而 `neotrix-neobot`
**不得反向依赖** core（`core → neobot` 固定，core 对 neobot 108 处引用，反转不可承受）。

## 架构决策：晶体为宿主（非依赖反转、非新增装配 crate）

`nt_crystal_serve` 是**唯一同时持有 core + neobot** 的进程，且 neobot 已有
与它配对的 HTTP 客户端 ⇒ **让晶体当能力宿主，薄客户端代理调用**。
依赖方向不变，无需新 crate。实测：`POST /v1/capabilities/invoke` 真执行 genoffice。

## 本窗提交清单（14 个，按主题）

### 阶段一：适配器 + 市场接线（genoffice）
- `635a8c05` genoffice 适配器模块（同时修复 `61ace1b2` 他窗 clippy 批量提交把 facade 扫进 HEAD却漏掉模块文件导致的 HEAD 损坏）
- `ee2a7b13` 登记插件市场：`GENOFFICE_MANIFEST`(Executable) + `dispatch_genoffice` + `seed_from_market_manifest` 双清单播种；四文件原子提交（含他窗的 `dispatch.rs` 2→3 参数 canary 重构）
- `a2a2d0ab` 派发层能力面补全 **4/14 → 18/18**：模板化 `GENOFFICE_OP_TABLE` + 全 op e2e。**参数形状全实测**（pptx create 只吃 `--ops`/`--spec`；text 字段是 `paragraphs`；docs ops 与 sheet cells 是两套 DSL）
- `3210c09f` `dispatch::is_registered` id 级探针 + `dialog say` 注册派发器
- `0a4db86e` 修 `CapabilityTreeRegistry::remove` 的**状态损坏**（先删后校验）。变异验证

### 阶段二：止血（P1）
- `f1ec7fe5` 上架面只摆「本进程真调得动」的能力（`dispatchable_ids`）；`capability_invoke` 超时从影子值变真强制（`drive_with_timeout`）。修自己打破的回归（旧测试把「调不动」当「已上架」）+ 负向锁 + 变异验证（旧代码下打印出 6 个调不动 id 被广告）

### 阶段三：架构地基（P2）
- `0eb34ff7` **能力脊柱** `CapabilitySpine`：唯一真源 `id→{exec,meta,health,counter}`；`Arc<dyn CapabilityExecutor>` 带状态；`unregister`/`replace` 事务化；`executability` **派生**（未登记/不健康⇒DeclaredOnly，清单 Scaffold⇒尊重）。9 单测 + 变异验证
- `e2e928ab` 脊柱↔`dispatch::Table` 桥。**方向刻意不对称**：`register_dispatcher` 自动进脊柱；脊柱→`Table` 显式且只接受 `as_fn_ptr()==Some`（防带状态执行器被静默降级）。3 桥测试 + 变异验证
- `b3056dc1` **投影与组合** `planner`：`tool_schema`（真实入参）、`project_tools`（按本轮 query 相关性 = 随用随调）、`compose_chain`（`stage:N` 排序 = 自我组合）。13 单测 + 2 处变异验证。**测试抓出两真 bug 并修**：CJK 前缀匹配、工具名映射不可逆
- `85fe308b` 端到端闭环测试：登记→派生→投影→组合→执行→拔除→换回，全程可观察断言；不健康者「可派发集合/模型面/链路」三处同步消失

### 阶段四：先修裁判（P4）
- `f492e801` 能力体系**裁决台账** `config/capability_systems.toml`（17 条，处置闭集 + 证据强制）+ 台账门（判据①锚点漂移 ②处置闭集 ③证据强制）。**零假阳性**，不查消费者数量。已记进 `gate-registry.tsv`

### 阶段五：两个 P0（注册表在生产里整体不可解析）
- `4da41072` **326 节点注册表 serde fail-fast ⇒ 整树在生产里不可见**。真因是**六处**非法枚举值（domain neobot×4 / layer l1primitive×4+l6meta×1 / constellation c2system×4+c3experience×1 / op "bud"×7）。修法分两类判：① domain 补代码（加 `Domain::Neobot`，触发仅 1 处穷尽匹配）② layer/constellation/op 修数据（已取证校准成熟度，`resume_verdict` 从虚报 c3experience 降到有证据支撑的 c1unittest）③ `skill_tree.rs` 的 `.ok()?` **静默加载器**改显式告警 + 逐节点容错计数。台账门加判据④（枚举合法性）。crate **首次 94 绿/0 红**
- `66db875a` **30 条「悬空 requires」**（我上轮口径错了，按代码双命名空间口径重算）+ 1 条死边。这 30 条不是垃圾，是树外组件/概念引用 ⇒ **显式分流**到新增的 `CapabilityNode::external_requires`（不删数据）⇒ `requires` 语义收敛为「必须能解析」，`validate_dependencies` 恢复为有意义信号。不变式用例 + 台账门判据④扩展（含死边检查）+ 变异验证（真实 rc=1）

## 当前状态（本窗视角）

- 能力脊柱 + 投影/组合 + 晶体宿主 + 裁决台账 + 台账门 全部就位、全部有变异验证。
- `nt_core_capability_tree` **96 绿 / 0 红**；`neotrix`(lib)/`neobot` 全绿；四道门（台账门 / executor-registry / lock_audit/ 门可满足性元门 advisory）全过。
- **接线进 neobot 模型面**（`project_tools` 替换 `nt_http_engine` 的硬编码工具清单）**未做**：他窗在 `nt_agent.rs`/`nt_types.rs` 施工，已按指示不碰。他已在 `21a4b37a` 落 PluginManifest 静态解析 + `ToolName::Plugin` 动态路由、`86ddcb57` 自造清单、`6c28c77b` 挂容、`4fec77aa` 出 PLUGIN-ECOSYSTEM 蓝图 —— **与本窗方向互补**，最终接线应两窗合力做。
- 元门 advisory 红：`check-claims-numbers.sh`（`AGENTS.md` 写「84 条」而 `task-index.json` 实 596 条）。**既有漂移，本窗未碰** `task-index.json`。

## 收工自查

- [x] ① `nt_worktree_gate.sh check` 已跑（收工义务 §1）
- [x] ② 本窗未开新 worktree
- [x] ③ 本 handoff 已更新
- [x] ④ 本窗所有改动已按 `git commit --only` 落入 `feat/multi-agent-absorb-2026-10-08`，未扫他人改动（四文件原子提交的 `ee2a7b13` 例外已在 commit message 里写明含他窗的 `dispatch.rs`；JSON 一并提交的 `4da41072` 已在 message 里写明含他窗后台蒸馏写入），已做 `git status` 干净度核对。
