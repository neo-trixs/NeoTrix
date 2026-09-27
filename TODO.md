# NeoTrix TODO 列表
> 智能同步生成，最后更新：2026-09-27（人工重建）

> **2026-09-27 目录架构统一轮（本轮，已 cargo 验证）**
> `cargo check -p neotrix --lib` **exit=0 / 1m07s**。完成 12 项：
> - 🔧 **修真 bug**：`KnowledgeBase::open` 把 SQLite 哨兵 `":memory:"` 当**文件路径**传，
>   在磁盘真建库 + `with_extension("lock")` 侧车再造假 `:memory:.lock`（落在仓库根）。
>   9 个调用点全修。**同族 `TemporalFactLedger::open`(`nt_temporal_facts.rs:63`) 早有正确处理，
>   唯独漏了此处** —— 按同一模式补齐。锚点 `l4_emotion/nt_memory/nt_memory_kb/kb_core.rs:101-111`
> - 删 `protocol/`（312 行 NIP-01 死代码，从未编译）与 `adapter/`（7 行陈旧桩）、`games/`（仅 `.DS_Store`）
> - `.gitignore`：`sessions/` 整目录忽略 → 按内容忽略 + 白名单（此前 11 个文件全靠 `git add -f` 硬塞）；
>   `HANDOFF*.md` → `/HANDOFF*.md` 锚定到根（无斜杠会匹配任意层级且在文件末尾覆盖白名单）；
>   清 3 条指向已删 `games/neotrix-guixu/` 的陈旧规则
> - `sessions/HANDOFF-TEMPLATE.md` **入库**（`AGENTS.md` 引用它做交接模板，模板不在库 = 规范不可执行）
> - 正典索引统一：`AGENTS.md` + `DOCUMENTATION-MAP.md` → 指向 `docs/architecture/NEOTRIX-MASTER-BLUEPRINT.md`
>   （`docs/architecture/README.md:7` 早已声明它是"唯一图纸入口"，不一致的是索引不是文档）
> - 规则沉淀：`AGENTS.md` 新增 R-SCAN-1/2/3（扫描器告警≠缺陷 / 手推≠实证 / 门记录必带时间戳）+
>   下刀前查 mtime；`RUST-STANDARDS.md` §17 新增 R-LOCK-4/5、R-BUILD-6、R-GIT-5；
>   门记录刷新为"22:52 实测 3 条 → 23:4x 复测 **0 条**"
> - 经验文档：`docs/architecture/LESSONS-2026-09-27-scanner-trust.md`

> **2026-09-27 卡死/内存专项 + 长尾分诊（收口）**：全量 `--lib` 串行实跑
> **11493 绿 / 51 红**（起点 10113/125）。8 条卡死/内存根因全部除根并加三道闸；
> 21+25+27 = **73 处生产缺陷**已修（安全洞 5、解析/数据 11、逻辑 25、stub 补实现 9、
> 环境依赖去抖动 6、契约对齐 12、并发/治理 5）。详见
> `sessions/handoff-disease-list-20260927.md`（含 125 条分诊全表与 file:line）。
>
> **待办（按性质分组，非按模块）**
> - 🔴 **B 类·需设计裁决**（不可一行改，先定方向）：
>   1. `l0_substrate/nt_core_kb_primitives.rs:188` — `nodes.id` 单列 PK 使双时态版本化
>      不可能 → 需 PK→`UNIQUE(id,transaction_time)` + edges FK 重构 + 真实库迁移（2 测试）
>   2. `l4_emotion/nt_memory/cascade/cascade.rs:216,126` — `length_score = len/200` 过小，
>      且 `tick()` 永久丢弃未达阈样本（属设计缺陷，需重定晋升/丢弃策略）
>   3. `neotrix-types` 的 `Severity`/`FlagSeverity` 判别序"越严重越小"是**承重约定**
>      （`l2_perception/nt_world/osint/sweep.rs:225` 依赖它做 `min_severity` 过滤）→ 不要盲翻 `Ord`
> - 🟡 **C 类·未接线 stub**（实现或显式 `#[ignore]`，禁止改松断言凑绿）：
>   `nt_core_embed::TextEmbedder`（字节位置袋，任意文本相似度≈0.83）、
>   `nt_shield_ztnet/crypto/noise_handshake`（缺 `_create_message3`，握手无法完成）、
>   `publish_gateway`（YouTube 上传）、`nt_codegen::parse_yaml`（误用 serde_json，需引 serde_yaml）、
>   `nt_memory_kb::nt_memory_distill`（测试 teacher 与 student 恒等，`after<before` 不可满足）
> - 🟡 **环境依赖**（改确定性断言或 `#[ignore]`）：~~`l6_meta::runtime_monitor::test_get_health`（探针挂起）~~
>   ~~`nt_feel::writing_style`（2）~~ —— **已由 cycle `audit0927b` 修掉，非环境问题**：
>   runtime_monitor 是**真死锁**（monitor 持 metrics 守卫调 check_thresholds，后者再
>   锁同一把，std Mutex 不可重入）；writing_style 是该 1,241 LOC 模块**从未被 mod 声明**，
>   从未编译，接上即 0 error 0 warning。详见 `sessions/handoff-disease-list-20260927.md` §10。
>   仍待处理：`l6_meta::nt_core_aware`（4）、`nt_core_observer_error`（2）、
>   `nt_feel::cognitive_bridge::feedback`（2）
> - 🔴 **P0·本线遗留（cycle `audit0927`/`audit0927b`，按建议顺序接手）**：
>   1. **50 个失败测试** —— 多数是断言与实现漂移（夹具自带 `Always` 规则使被测分支
>      不可达、亚毫秒时长断言 `> 0`）。单条易改但 50 条一起动风险大，**独立成轮**。
>      分布：nt_shield 7 / nt_core_capability 6 / nt_memory 5 / nt_feel 5 /
>      nt_core_aware 4 / nt_meta 3 / healing 3 / 其余 17 个各 1-2。
>      复现：`cargo test -p neotrix --lib --no-run` 后跑二进制，`--test-threads=2`。
>   2. **`--test-threads=4` 时 SIGSEGV（退出码 139）** —— 崩在
>      `l6_meta::healing::predictive_maintenance::trend::tests` 之后；同模块单/双线程
>      跑均不复现（311 passed）。pre-existing 并发问题，**CI 暂用 `--test-threads=2`**。
>      根因需单独定位（共享资源竞态 / 栈深 / fd 耗尽）。
>   3. **三处同名类型双定义** —— `ExtractConfig` / `EmailConfig`
>      （`extractors/mod.rs` 与 `data_pipeline.rs` 各一份）、`PlatformRegistry`
>      （`data_pipeline.rs:211` 与 `platform_registry.rs:46` 各一份）。
>      `TradeDataPipeline::with_registry` 只认后者，收敛前须先确认二者语义是否本就该合并。
>   4. **4 个抽取 crate 的定位裁决** —— `crates/nt-lang` 只有 `[[bin]]` 无 `[lib]`，
>      **结构上无法被任何 crate 依赖**，却仍占 workspace member 槽（根 `Cargo.toml:12`）：
>      补 `[lib]` 并接入，或删掉。
>   5. **`neotrix-core/src/neotrix/nt_core_capability_tree` 住在 `src/` 里** ——
>      独立 crate（根 `Cargo.toml:11` member）却位于另一 crate 的源码目录，
>      4,670 LOC / ~30 处真实调用。已补 `[lints] workspace=true`（原 18 条 lint 全失效，
>      补上后暴露 20 条告警含 7 处可能 panic），但**归属未裁决**：是搬出去还是接受。

> - ⚪ **结构性债务（PARK，有归属前置）**：L0 `CapabilityRegistry` ×4 + `SemanticRouter` ×2
>   正典收敛（异构，需专窗迁移）｜`proxy_pool.rs` 1757 行拆分（他人在途 1039+/24-）｜
>   剩余 God-file（`pdf.rs` 2142 / `nt_crystal_serve.rs` 2002 / gateway 1678 / hex 1530）｜
>   5 个内容型 worktree 裁决｜前端 `apps/neobot-desktop/frontend/{src,dist}` 归属与提交

> **2026-09-27 第三轮（4 代理并行）已改未验 — 交接给下一对话，优先收口**
> 30 项修改在 working tree，**全部未提交**（提交门禁需跑 cargo，当时内存门 BLOCKED）。
> 验证进度：一次跑 **802 passed / 2 failed**（29 项中 28 项绿），补修后再跑 **65 passed / 1 failed**。
> 唯一残留：`noise_handshake::full_handshake` —— 已从"构造即 panic"推进到
> `_consume_message2` 处 unwrap 失败（`noise_handshake.rs:420`），是真 crypto 缺口。
>
> **🔧 接手第一步（务必按序）**
> 1. `sh scripts/ops/nt_mem_gate.sh; echo $?` → 必须为 0 才继续
> 2. 定向验证：`CARGO_BUILD_JOBS=1 cargo test -p neotrix --lib -- <模块前缀> --test-threads=1`
> 3. 全绿后 `git commit -- <paths>` **pathspec 限定**（禁 `git add -A`、禁 `--no-verify`）：
>    `nt_core_capability/{dependency,integrator,monitor}.rs`、`nt_meta/gwt_router/{cost_weight,attention}.rs`、
>    `nt_core_self/dynamic_params.rs`、`nt_agent_identity.rs`、`nt_core_aware/mod.rs`、`nt_feel/{cognitive_bridge/feedback,writing_style,salesperson_profiling}.rs`、
>    `nt_memory/{cascade/cascade,consolidation/cache,distillation/distiller}.rs`、`nt_memory_kb/{memory_orchestrator,nt_memory_distill}.rs`、
>    `nt_shield_sandbox/stateful_bench.rs`、`nt_shield_ztnet/crypto/noise_handshake.rs`、
>    `nt_shield/{compliance/requirement,nt_shield_audit/threat_modeler,shield_core/audit,shield_core/safety_kernel}.rs`、
>    `nt_core_speculative_decoding.rs`、`nt_core_vector_store/store_hnsw.rs`、`nt_codegen.rs`、
>    `nt_core_guardian/repair.rs`、`crates/neotrix-types/src/{core/shared_types.rs,llm_types.rs}`、
>    `Cargo.toml`、`neotrix-core/Cargo.toml`
>
> **⚠️ 本线已做但未验证的高价值修复（接手方请优先确认）**
> - `nt_core_guardian/repair.rs` —— 自愈动作 `ClearCache` 原本执行 **`cargo clean`**
>   （会删整个 target/ 含 deps 活指纹）。已改为只删 `target/<profile>/incremental`。
> - `noise_handshake.rs` —— `hash[..27]` 拷 25 字节协议名，**任何构造都 panic**，
>   整个模块从未可用过。已按字面量自身长度自适应。
> - `stateful_bench.rs` S4 —— 原本把安全修复前的"有洞极性"写成断言（给漏洞背书），已互换策略布尔。
> - `store_hnsw.rs` —— 索引按余弦排序却报 Hamming 距离（自相矛盾），Hamming 配置改走图外精确扫描。
> - `check_ip` —— 原本无 CIDR 支持，IP 白名单形同虚设；已实现（blacklist 同步）。
> - `check_ip`/`serde_yaml` —— `serde_yaml` 已入 workspace + core 两处 manifest，
>   **首次构建会自动更新 Cargo.lock**（离线可解，`--locked` 构建会失败，需注意）。
>
> **📋 剩余 51 条待修**：全量 11493 绿 / 51 红，逐条根因见
> `sessions/handoff-disease-list-20260927.md` §9（P 17 / S 27 / U 4 / E 3，含 file:line）。
> **🧭 3 项需人工决策**：`sessions/handoff-decision-20260927.md`
> （双时态 PK 建议删 API / CAD 证据表建议砍到 4 个 / publish_gateway 建议加 dry_run）。


> **Batch3 吸收执行 (47 源)**: 四波 21 任务 20/20 闭环 · **交接 Wave 4: 10 任务待做** (🔴P0×3 越层修复/e8_state 合成值/测试抖动加固 · 🟡P1×3 情报工具接线/SEAL C0→C2/补全排序 · ⚪P2×4) → `docs/absorption-knowledge-base/batch3-2026-08-26-unified-evolution-todo.md` Wave 4 段 + 根 `HANDOFF.md` (2026-08-26 版)
---

# 统一进化清单（2026-09-27 重建）

> 本节由三份审计合并去重而成，是**唯一**的进化任务入口。
> 来源：`docs/architecture/DIR-AUDIT-2026-09-27.md`（目录/依赖）·
> `docs/architecture/EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md`（18 项支脉定位）·
> `docs/architecture/ABSORPTION-EXTERNAL-2026-09-27.md`（19 项外部吸收）。
> 三份文档保留推导过程，本节只保留**去重后的可执行条目**。
> 每项均带 `file:line`；无定点不改（RUST-STANDARDS §17）。

## 阶段 0 · 脱 stub 与收敛（1-2 天，最高杠杆）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 0.1 | **`UnifiedApi` 脱 stub** —— 桌面 chat 接真内核 | `src-tauri/src/stub.rs:275`（零状态单元结构体）· `:285-304`（`:289` 返回字面量）· `main.rs:52` 注册源 · `main.rs:387,407-408` 调用点 | ⬜ |
| 0.2 | **删 `ToolRegistry` 45 行 stub**，其余 3 份改 `pub use` | 删 `l5_cognition/nt_core_gate/nt_tool_registry.rs:11`；保留 `l1_action/nt_act/tool_registry.rs:85`（770 行唯一真实现） | ⬜ |
| 0.3 | **`CapabilityRegistry` 4→1** | `l0_substrate/nt_core_capability_types.rs:533` · `l5_cognition/nt_core/capability/registry.rs:453` · `neotrix/nt_file_ability/capability.rs:173` · `nt_core_capability_tree/src/registry.rs:53` | ⬜ |
| 0.4 | **`SkillRegistry` 5→1**：先删 `neotrix-types` **包内自重复**（零风险第一刀） | 自重复：`neotrix-types/src/core/skill.rs:54` + `core/skills/mod.rs:25`；另 3 份：`neotrix-core/src/skill_registry.rs:14`（正典）· `neotrix-gateway/src/skill_registry.rs:157` · `neotrix-multi-agent/src/skill_registry.rs:157` | ⬜ |
| 0.5 | **`maturity_audit()` 接 CI 门** —— 机制已完整实现且**带自愈**（`:484-485` 自动下调声称等级），缺的只是没人调它 | `nt_core_capability_tree/src/registry.rs:459` · 数据源 `.neotrix/capability_registry.json` → `nodes[]` | ⬜ |
| 0.6 | **CI 三断言基座**：① `detector_coverage ⊇ execution_scope` ②注册表唯一性防回潮 ③schema 指纹门（`sha256(DDL)[..12]`，环境派生字段拆独立门） | `security-audit.yml`（实测全文只有 `cargo deny check all`）· 挂靠点 `scripts/ops/` | ⬜ |

> **0.1 的隐藏工作量**：`UnifiedResponse.metadata`（`stub.rs:292-303`）**已预留**
> `consciousness_state{phi,coherence,gwt_resonance}` + `capabilities_used` + `confidence`，
> 与 L5 意识核**类型级吻合** —— 契约已定好，填值即可。

## 已知债 · 主 CI workflow 曾无法解析（2026-09-27 修复）

`.github/workflows/ci.yml` **在 HEAD 就是非法 YAML**：`Truth-surface gate (ratchet: blocks ...)`
这一行 `- name:` 的值未加引号且含冒号，`yaml.safe_load` 报
`mapping values are not allowed here (line 28)`。

**影响**：`ci.yml` 是主 workflow（12 个 job 中的 11 个在此）。它无法被解析 ⇒
**`check-truth-surface.sh --strict` 这个门从未在 CI 里跑过**。而本项目多份文档
（包括本文件此前）把「`truth-surface-baseline.txt` 基线 0 条 ✅」当作门有效的证据 ——
**基线再准，门没跑就等于没有门**。这与 R-SCAN-3（门记录会腐化）是同一族问题的更严重形态：
不是记录过时，而是**门本身从未接线**。

**已修**：该行加引号；全 14 个 workflow 用 `yaml.safe_load` 扫过，其余 13 个均合法。
**新增** `capability-truth` job（报告态，见「已知债 · 能力成熟度虚标」）。

**遗留动作**：核实 `check-truth-surface.sh` / `check-layer-deps.sh` 在修好 workflow 后
是否真的通过（本次未在 CI 环境验证，只做了本地 YAML 合法性检查）。

## 已知债 · 能力成熟度虚标 24 项（2026-09-27 实测，暂不自动降标）

`./target/debug/neotrix-capability audit-maturity` 实测 **24 个虚标节点，全部在 NT-ACT**
（`nt_file_ability::*`，声称 C2/C3 而"证据"只支撑 C1）。

**已做**：给 CLI 加 `--strict`（`nt_core_capability_tree/src/cli.rs` `Commands::AuditMaturity`），
使 `maturity_audit()` 可被 CI 引用 —— 此前该命令**恒返回 `Ok(())`**，机制齐备却无法被任何流水线
引用，于是 `demote_mislabeled` 自愈路径从不触发。

**为何不执行 `--apply` 自动降标**：`evidence_supported_constellation`
（`nt_core_capability_tree/src/node.rs:465-485`）**不检查真实测试文件**，只读 `metadata` 三个声明字段：

| 等级 | 判据 |
|---|---|
| C1 | `provides` 非空 |
| C2 | `metadata.wiring_evidence` 非空字符串 |
| C3 | `metadata.evidence_gated == "passed"` |

抽样 3 个模块（`tables::xlsx_read` / `merge::merge_tables_with_mode` / `ocr::OcrEngine`）在
`neotrix-core/tests/` 与 `benches/` **各 0 次出现** → C1 对它们确实正确。
**但该机制无法区分「真没集成测试」与「测了但元数据没填」**。而 `nt_file_ability` 按外部吸收分析
是已接入生产的（xlsx-data skill 分支）。用声明元数据这个代理指标下调 24 个节点的项目语义，
属于用指标代替内容。

**正确修法**：为这些节点补真实 `wiring_evidence`（file:line）或跑 D16 promotion gate，
**而非**降标。补完后 `--strict` 即可转绿并正式设为阻塞门。

## 阶段 1 · 安全封口（3-5 天，事故已真实发生）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 1.1 | **DNS qtype 白名单** + qname 长度上限 + 把 `dns_allow` 从"学习提示"改成"过滤器" | `l1_action/nt_io/nt_io_provider/common/egress_types.rs:14-21` —— `SandboxEgressRule` **只有 `host`/`port`/`allow` 三字段，全文件零 DNS 概念** | ⬜ |
| 1.2 | **"以尝试为检测单元，结果是独立字段"** —— 任何情况下不得让 outcome 衰减 attempt 严重性 | `l0_substrate/nt_core_telemetry.rs`（事件 schema）· `l6_meta/nt_safety_monitor.rs` · `l6_meta/nt_core_guardian/health.rs` | ⬜ |
| 1.3 | **严重性校准对 + 职责分离**（借 cloudflare/security-audit-skill，MIT） | 写入 `RUST-STANDARDS.md` §17.5 或 audit 规则 | ⬜ |
| 1.4 | **非不可宽化策略地板**：把"允许绕过批准的能力集合"变成测试钉死的冻结数据 | `crates/neotrix-neobot/src/nt_policy.rs:75` `evaluate_policy`（现 deny 全集：`human-control:77` `computer-allow:84` `computer-host:90` `workspace-jail:98,106` `unknown-tool:119` `default-deny:121`；`evaluate_extra_deny:137`） | ⬜ |

> **1.1 的行业空白**：深挖 7 个沙箱/安全仓，**没有一个做 qtype 过滤**。
> `microsandbox` 唯一真做 host 侧 DNS 管控（smoltcp，guest 不持 resolver socket）但也无 qtype；
> `CubeSandbox` 的 eBPF `dns_allow` **只学 A 记录**，不在名单内的查询不被拦且 gateway 常放行
> —— **与 OpenAI 2026-09-20 事故同构的隐蔽信道**。

## 阶段 2 · 记忆正确性（1-2 周）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 2.1 | **supersession 形态**（绕开主键重写，今天可迁） | `l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs` —— 5 段即 `:169 snapshot` / `:195 distill` / `:253 classify` / `:306 persist` / `:403 feedback` | ⬜ |
| 2.2 | **真双时间四列迁移**（依赖 2.1 先落地） | 同上 `:29 ExperienceEntry` + `l4_emotion/nt_memory/{kb_kb,paged_kv}`。先例：`l0_substrate/nt_core_kb_primitives.rs:188`（即上方 B 类第 1 项） | ⬜ |
| 2.3 | **`Provenance` 第二轴**（**不要重载 `Source`**） | `experience_tree/mod.rs:101` `Source{Dialogue,Audit,Research,Absorption}` 是**渠道**语义，与"证据等级"正交 | ⬜ |

> **2.2 最值得抄的细节**：`valid_to_precision = 'unknown'` 三态
> （`valid_to IS NULL` + `precision IS NULL` = 持续；`+ precision='unknown'` = 已结束但日期未知）。
> 教科书做法（把文档日期塞进 `valid_to` 当上界）被 utopia 显式拒绝，理由是
> *"看起来像个确定的时间戳；每个读者都得先查精度，而不撒谎才是产品。"*

## 阶段 3 · 可观测与成本（1-2 周）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 3.1 | **成本归因：在链路上抓** —— canonical block 记录（`zone`/`section`/`bucket`/`tool`/**`skill`**/`role`/`tokens`/`hash`），skills 与 MCP server 作为一等维度 | **唯一插点已存在**：`l1_action/nt_io/nt_io_provider/anthropic/anthropic.rs:92,237`（注释 "P0-4 prefix caching"）= 请求时组装、从不落 transcript 的不可见前缀。已有 `Usage`：`crates/neotrix-decision-engine/src/types.rs:304` | ⬜ |
| 3.2 | **能力 manifest 加安全信封**（声明式爆炸半径 + 构建期校验） | `.neotrix/capability_registry.json`（形状已对：`provides`/`requires`/`rune_sockets`/`evolution_log`）· 弃用机制已在 `capability_overrides.json`（`deprecated`/`deprecated_reason`） | ⬜ |

> **3.1 为何必须走链路**：系统提示、注入的 tool schema、MCP schema 都在**请求时组装、从不写进
> transcript**，那段前缀"可以占到上下文的一半甚至更多"。**基于日志的归因对最大的成本中心结构性失明。**
> 120 个 skill + 28 个 Tauri plugin 正在这个盲区里。

## 阶段 4 · 决策面与自治（2-3 周）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 4.1 | **JEV 四件套**：场景指纹 / 硬 `call_budget` / 过期 / 非阻塞 worker + 三路置信门 | 三原语**已忠实建模**：`neotrix-decision-engine/src/types.rs:85 QuestionType` `:146 NoulAnswer` `:166 ChoiceAnswer` `:189 ScoreAnswer`；后门 `engine.rs`/`router.rs`；已有 stale 机制 `crates/neotrix-neobot/src/nt_stale_guard.rs` | ⬜ |
| 4.2 | **"未解析"建模为一等状态**（`epistemic: exact \| lower-bound`） | `nt_core_capability_tree/src/node.rs` · 雏形已在 `registry.rs:15-18 MaturityFinding{claimed,supported}` | ⬜ |
| 4.3 | **自治循环 git 化 + 固定墙钟预算**（`results.tsv` 5 列，变好推进/变差 `git reset`） | `Makefile`（现有目标 `run:4` `project-locate:151`）· 反馈判据 `experience_tree/mod.rs:403 feedback()` | ⬜ |
| 4.4 | **覆盖率账本状态机**（hunters 不能写自己的覆盖率） | `l3_embodiment/nt_shield/nt_shield_audit/`（10 文件）· `l6_meta/nt_core_self/self_audit.rs` · 基线 `scripts/truth-surface-baseline.txt`（现 0 条 ✅） | ⬜ |

> **4.1 最高价值的一条方法论**（jev-drone 自评，MIT）：
> *"a state-design bug, not a model failure"* —— JEV 从不选 `climb`，因为状态是 5 个**水平**距离扇区、
> **没有垂直信息**，"飞过去"根本不可推断；加入障碍顶边高度后 `climb` 从"从未被选"→ **p=0.93**。
> **规则：如果一个决策从不触发，先审状态形状，再审模型。**
> ⚠️ 同时记住反面：该仓头条数字是 **n=1 单次运行**，3-seed 配对比较**无优势**。别把单次胜利当机制有效性。

## 阶段 5 · GUI 与 MCP（2-4 周，可全程并行）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 5.1 | **元素寻址 GUI 驱动**（B 路：a11y 树优先，坐标仅 fallback） | 现状：`l3_embodiment/nt_computer.rs`(477 行，**filesystem/process trait，不是 GUI**) · `nt_computer_fleet.rs`(262) · `crates/neotrix-neobot/src/nt_computer.rs`(155，`NoopBackend` 唯一后端) · `l1_action/nt_io/nt_io_desktop/` **只有 2 文件**（`mod.rs` 9 行 + `updater_signing.rs`） | ⬜ |
| 5.2 | **MCP per-request capability 协商** + 客户端提生产 | 协议层 2026 已稳定：每请求必带 `protocolVersion`+`clientCapabilities`，缺字段 `-32602`；`MissingRequiredClientCapabilityError`(`-32021`)。本地：`l1_action/nt_act/mcp_protocol/`(775 行 ✅) · `l1_action/nt_io/mcp_server.rs:20`（内存 Vec，**无 wire transport**）· **客户端 `McpRegistry` 还在 `neotrix-core/src/agent.rs:566` 的 `#[cfg(test)]` 块里** | ⬜ |
| 5.3 | **无 API 重放**（审计路径） | `crates/neotrix-neobot/src/nt_audit.rs`（**全仓 `replay` 零命中**）· CLI `bin/neobot.rs` 已有 `audit` 子命令 | ⬜ |
| 5.4 | **UI 组件 + 动效 token** | `src-tauri/frontend/src/canvas/`(9 文件/1793) · **SolidJS**（5 个候选 UI 站里 4 个是 React，需 1-3h/个移植）。`rareui.com` **已死**（HTTP 402 `DEPLOYMENT_DISABLED`） | ⬜ |

> **5.2 别投钱的部分**：registry 只是元数据 —— 其自身 roadmap 白纸黑字
> *"Unified runtime: Not solving how servers are executed"* · *"Quality rankings: No built-in server
> quality assessments"*；schema 里**根本没有 `capabilities` 字段**。**行业自己都还没做，别自建。**
> 长任务已被移出 core 到扩展（SEP-2663），协议**无服务端 task store**。
> 2026-07-28 弃用 `roots`/`sampling`/`logging` → **客户端必须处理三个协议世代**。
>
> **5.4 该建的那一叠**：`beautifului.dev`(MIT) + `ui.shadcn.com`(MIT) 是唯一连贯的一对。
> **别把 beUI 动效混进 transitions.dev 的 token scale** —— 会正好产出 `transitions.dev refine`
> 命令存在的意义所在的那种 ad-hoc 硬编码时长债务。

## 成本感知路由 —— 2026 全行业空位（强依赖 3.1）

深挖 8 个编排仓的模型选择机制清点：Orca 的 per-worker `--model/--effort`（手动覆盖）、
AX 的 `Model` CRD（凭据包不是选择器）、MAF 的 `MagenticProgressLedger`（进度感知非成本）、
dsh 的 `TeamMemberSnapshot.provider`、Symphony 的单一固定 `codex.executable`。
**没有一个有价格表、token 成本模型，或把成本/质量前沿放进路由路径。**
Symphony 记 token 与 rate limit —— 但**只用于显示，从不用于路由**。

**顺序是强依赖的：先能归因（3.1），再能路由。**

---

## 编排隔离单元的实测（决策依据，非待办）

8 个编排仓**无共识**，实测如下：

| 隔离单元 | 谁在用 | 判定 |
|---|---|---|
| 一进程一 agent | **8/8 无异议** | 不是选择，是地板 |
| **一 worktree 一任务** | 只有 Orca（worktree-native）。**Symphony SPEC.md §9.3 明文反共识**："The spec does not require any built-in VCS or repository bootstrap behavior" | 有争议 |
| 一容器一任务 | 只有集群派（AX / CubeSandbox），非谈判项 | 集群专属 |
| 共享队列 + 租约 | **没有一个有分布式租约**，全是本地 FS 存活探针或内存 claim | 空白 |

**NeoTrix 已做对**：`l1_action/nt_act/nt_act_workspace_isolator.rs`(719 行，git-worktree-per-task)
正是唯一有真实共识基础的那一档。**别动它。** 缺的是 reconciliation，不是 worktree。

---

## 附：已废止，不得再实现

| 来源 | 原因 |
|---|---|
| `rareui.com` | 站点已死：HTTP 402 `x-vercel-error: DEPLOYMENT_DISABLED` |
| `ARCHITECTURE-EVOLUTION-ROADMAP.md` | 零引用，55 行 |
| `FUSION-ARCHITECTURE.md` | 零有效引用，其"下一步"含**已被证伪**的"解决预存编译错误" |
| `ARCHITECTURE.md §1-§12` | 已被 §13（neobot 融合，2026-09-26）推翻。读 §13 起的实测部分 |
| `protocol/`（已删） | 312 行 NIP-01 事件总线，从未编译（不在 `lib.rs`）→ 那 7 个测试**从未真正跑过**；Nostr/NIP-01/Buzz 在 914k 行代码库零足迹。git history 可追回 |
| `neotrix-core/src/adapter/`（已删） | 7 行陈旧桩，自述功能已并入 `nt_io_provider`（后者已完全消失，坐实合并残留） |
| `nt-lang` 的"删掉"建议 | **已自我纠正**：`docs/plans/2026-09-20-nt-lang-evolution-roadmap.md` 显示它是"测试生成器→声明式 DSL"的在制品。补 `[lib]` 或接线，**不要删** |
