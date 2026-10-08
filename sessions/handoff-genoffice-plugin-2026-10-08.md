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
