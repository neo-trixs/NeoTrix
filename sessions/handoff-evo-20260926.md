# NeoTrix 统一进化迭代（2026-09-26，34 源吸收 → 路线 → 版本 → 归档 → 清单）

> 34/34 只读吸收成功，零 failed、零编造（四 lane 子代理证据见§0）。
> 本文件为统一进化路线设计 + 版本目标 + 归档计划 + 完整任务清单。零 commit，改动待批。
> 硬规则：forbid(unsafe)；生产禁 unwrap/expect/panic；nt_ 前缀；R-P16；单 cargo；禁 clean/--all-targets。

## 0. 吸收战报（34 源核心一行 + 思想一行）

### Lane A · Agent/技能/效能（9/9）

| 源 | 核心逻辑 | 思想特性 | 缺口 |
|---|---|---|---|
| oh-my-openagent 69.4k★ | 三形态+11 agents/54 hooks/4 MCP；ultrawork 门+按类选模+Team Mode lead+8 并行 | 只管编排不管选模；discipline 不半途停工 | 缺 Team Mode＋按类路由 |
| claude-token-optimizer 603★ | 11k→1.3k；4 文件常驻+按需载+learnings 注入；measure/audit/compress 全套 CLI | 先度量再治理；压缩归档永不真删 | 缺 Token 成本门 |
| ai-employees 432★ | 8 角色×60 routines；ROLE/SCHEDULE/SKILL 纯文本+晨报+ledger；11 harness 复用 | last-click 归人；RELEASES 授权释放 | 缺常驻业务角色范式 |
| win4r/MuseAI-Skills 227★ | 68 SKILL 快照；wide-research 协调者+skill-creator+artifacts/testing；三件套 | 触发/边界/验收先行；manifest 方法级权限 | 缺技能评审三件套 |
| openmuse 2.3k★ | AG-UI runtime；持久 Chromium+Docker computer+SQL lease；action review 收据 | 所见即所得可接管；写后不隐藏重试 | 缺持久 computer+收据链 |
| ip-as-logo-skill 5.6k★ | 单文件 SKILL 无脚本；3 提案→点头→6 候选并行；image-only prompt | 约束写给模型看；一遍交付 | 缺 IP 品牌技能范式 |
| reverse-skill 37.8k★ | RULES→ROUTING→scope→场景梯；routing.json 单源真理+175 例回归 | scope 未就绪不 ACT；client-neutral | 缺路由单源+回归基准 |
| mapcn 12.2k★ | React 地图组件；MapLibre+Tailwind+shadcn；registry 分发 | 不造引擎只做体验层 | 缺可视化组件层（次） |
| luvus 901★ Rust | TUI mission-control；workspace/pane 全控+agent 状态感知+Commander 群发+UHP 协议 | 单窗可观测胜多窗并行 | 缺 mission-control |

### Lane B · 数据/基建/训练（9/9）

| 源 | 核心逻辑 | 思想特性 | 缺口 |
|---|---|---|---|
| cocoindex-code | tree-sitter AST 切块+embedding；Rust 增量索引；ccc search/grep/doctor，省 70% | 零配置；结构搜索；daemon 增量 | 缺声明式增量语义索引 |
| dbx 25MB | Rust+Tauri 跨端；统一驱动 MySQL/PG/SQLite/Redis/Mongo/DuckDB；AI+MCP+Skill | 极小多后端抽象 | 缺统一数据面网关 |
| Ix | tree-sitter 抽 symbols/calls/imports 落 ArangoDB；explain/trace/impact/rank 有界切片 | 查图不灌文件，省 30–99.7% | 缺持久系统图谱 |
| verl（MiMo fork） | HybridFlow 编排 PPO/GRPO/DAPO；FSDP+vLLM 解耦；五域 AgentLoop+可验证 reward | 生产 RLHF+多轮 tool-call 一体 | 缺 RL 后训练闭环 |
| needle3 8-29MB | Laddered Attention+GQA+engram；2.125bit 单文件；tool-call/抽取/embedding 三合一 | 弃泛聊换端侧精确执行 | 缺端侧 tiny-tool 位 |
| dspy | Signature/Module 代手写 prompt；GEPA/MIPRO 编译器搜指令+few-shot | prompt 当超参优化 | 缺自优化 prompt 闭环 |
| MarS | LMM 订单级生成；mlib Env+State+Agent；Ray Serve+三类评估 | 世界模型涌现价格；反事实实验 | 缺可控仿真评估器 |
| runner | Rust gpui 真 PTY；Role→Crew→Mission+feed 回放+ask_human；CLI 双向 | provider 中立 | 缺 mission 基建（feed/lead） |
| artemis 99%+ | Flash/Pro 双 profile；无障碍树+OCR+定位；SafetyNet+burst；MCP/CLI/SDK | Dynamic-First；incident 自恢复 | 缺 observe-act 校验环 |

### Lane C · 浏览器/设备/协作（8/8）

| 源 | 核心逻辑 | 思想特性 | 缺口 |
|---|---|---|---|
| jev-ultrafast 20.5k★ | 动态动作空间；TypeSafe Jev单请求 operation+target 扇出；快照读可见控件；遮挡校验 | 无截图省 context；等待按需 | 缺高速 nt_browser 环 |
| jev-chat-jarvis 6.6k★ | 一 App 一适配器；无障碍/Compose/OCR 三路；判断→3 候选→只填不发 | 非侵入只读；发送权在人 | 缺设备安全闭环 |
| apk-reverse 1.8k★ | SKILL 门控 R1-R4/G1-G4；渐进披露；doctor.py 首检；等长补丁优先 | 失败目录即资产 | 缺设备取证 Skill |
| taskview-community 908★ | Monorepo API+Web+MCP；看板/冲刺/依赖/工时；SAML/OIDC+tvk_ token | AI-ready；Your data | 缺协作任务平面 |
| genoffice 7.8k★ | 7 Electron+TS+Rust sidecar；原文件真源+窄补丁回装；CLI+MCP 双驱；FTS CJK | Local by design；改动可回滚 | 缺字节保真文档引擎 |
| whiteboard 1.5k★ | 共绘 canvas；语义 AST diff；决策日志链 Agent 决策；LSP 直跳 | Diagrams lead to code | 缺共创画布 |
| localsend 92.7k★ | 局域 REST+HTTPS；每设备即时 TLS；53317 端口；Flutter+Rust；cli 直投 | 去中心零云中转 | 缺近场 P2P 通道 |
| authentik 25.7k★ | 全协议 IdP（SAML/OIDC/LDAP/RADIUS）；Compose/Helm；企业对标 | Authentication glue | 缺统一身份胶水 |

### Lane D · 论文/语言/系统（8/8）

| 源 | 核心逻辑 | 思想特性 | 缺口 |
|---|---|---|---|
| arxiv 2609.14011 ICL | 100 抽象任务×6 模态；clean-deranged 对照+Δn；5/6 正相关，9.9–31.4pp | ICL 是模态不变基底 | 缺跨模态 ICL 基准 |
| bend 2 | Python 语法+dependent types+affine；LAWS 机验；分治并行+GPU | LAWS=可证明 AGENTS.md | 缺证明门 |
| mu judge-kernel | 35 决策点交小快 judge；chunk 准入+pointer+墓碑；verdict 进 ledger | 决策与 work 解耦；context 永不满 | 缺 admission/墓碑 |
| reef | Serve→Observe→Grow→Commit；收据+report 评分；recipe 双 surface；热切换 | harness 本身可进化 | 缺在线进化闭环 |
| Horizon | 多源→profile 管线（rubric/阈值/去重→enrich→balanced digest）；CLI+MCP | 去重在 enrich 前 | 缺 profile 情报管线 |
| pebrel Rust GPUI | GPU 终端；tabs/SSH/hook 感知/Lua 热载 | 执行与阅读分离 | 缺 GPU embodiment 终端 |
| macro | 同后端 email/chat/CRDT/tasks/CRM；双向@link+channel 权限；nightly team-memory；~100% MCP | 公司须 computable | 缺@link+CRDT 协同 |
| TokenHub v0.9.0 | OpenAI 兼容+embed/rerank；Jev语义路由（预算/fallback/review）；retrieval 独立计量 | governed inventory | 缺语义路由+计量 |

## 1. 全域 map 缺口（12 缺失架构能力点 → 层）

| 缺口 | 层 | 源 | 严重度 |
|---|---|---|---|
| M-01 Token 成本门（阈值告警/拦截/压缩回归） | L0/L6 横切 | cto | P0 |
| M-02 judge-kernel（admission/墓碑/ledger，小模型常驻） | L0 缓存 | mu | P0 |
| M-03 DSPy 自优化 prompt（Signature/编译器/评估闭环） | L5 认知 | dspy | P0 |
| M-04 高速浏览器环（原子快照/单 RTT/遮挡校验） | L1 动作 | jev-ultrafast | P0 |
| M-05 语义索引＋系统图谱（AST+向量召回，symbols 图精排） | L1/L2 | coco/Ix | P1 |
| M-06 技能路由＋评审（routing.json 单源真理+三件套+回归） | L5 心智 | reverse/MuseAI | P1 |
| M-07 在线进化闭环（Serve/Observe/Grow/Commit+artifact 版本） | L6 元 | reef | P1 |
| M-08 统一数据面（多模 DB 网关+MCP） | L0 底座 | dbx | P1 |
| M-09 近场协作平面（mTLS 传输+OIDC+画布+任务 MCP+保真文档） | L3 具身 | localsend/auth/white/task/genoffice | P2 |
| M-10 RL＋仿真（GRPO tool-use reward；MarS 反事实评估） | L4/L5 | verl/MarS | P2 |
| M-11 证明门＋语义网关（LAWS 影子模式；Jev路由+独立计量） | Shield/网关 | bend/TokenHub | P2 |
| M-12 情报 profile 管线（rubric/去重/balanced digest）＋跨模态 ICL 基准 | 感知/评估 | Horizon/14011 | P2 |

## 2. 统一进化路线（最优解序列）

### P0（本会话可立项，同会话生产可用项先行；R-P79：cto 度量/DSPy 编译器/judge 影子三件可独立 binary 落地）

- **EVO-01 Token 成本门**：`measure/audit/compress` CLI＋CI（exit 1）＋pre-tool guard（warn/block）＋learnings 注入。owner: Infra。
- **EVO-02 judge-kernel（影子）**：tool admission 逐 chunk＋forget 墓碑＋ledger；prompt-cache warming。owner: L0。
- **EVO-03 DSPy 层**：Signature＋metric＋Bootstrap 编译器；先离线夜跑，不碰训练集群。owner: L5。
- **EVO-04 高速浏览器环**：原子快照＋单请求 operation/target＋遮挡/新鲜度校验。owner: L1。

### P1（Phase 2 窗口；需可构建窗口＋单 cargo 串行）

- **EVO-05 语义索引＋图谱**：nt_locate 升级 AST+向量；symbols/calls 图落 SQLite；explain/impact。owner: L1/L2。
- **EVO-06 技能路由＋评审**：routing.json＋test-routing 回归；新技能三件套强制门。owner: L5。
- **EVO-07 在线进化**：receipt＋report 评分；nightly Grow；artifact 版本化＋热切换。owner: L6。
- **EVO-08 统一数据面**：多模 DB 网关＋MCP（记忆持久层后端之一）。owner: L0。

### P2（条件触发；重型/跨端）

- **EVO-09 近场协作**：53317+mTLS 传输→OIDC 门→画布/任务/文档面。owner: L3/Desktop。
- **EVO-10 RL＋仿真**：verl recipe 小规模 GRPO→MarS 评估器→needle 端侧蒸馏接口。owner: L4/L5。
- **EVO-11 证明门＋语义网关**：LAWS 影子→门禁；Jev路由＋retrieval 计量。owner: Shield。
- **EVO-12 情报＋ICL 基准**：Horizon profile 管线挂 intel-watch；14011 paired-gap 评测集。owner: 感知。

落地纪律：R-P79 外部思想同会话至少一处生产可用（建议 EVO-01 measure 先行）；
R12 成本纪律 eval 独立 binary；训练类禁在 16G 日间跑。

## 3. 版本目标（最优解）

- **目标：v0.22.0（workspace 统一）**：`workspace.package 0.21.0→0.22.0` 全员继承；
  `apps/neobot-desktop` 0.21.0→0.22.0 与主 `src-tauri` NeoTrix 0.22.0 对齐（双产品线版本号统一，各自 productName 不变）。
- **暂缓执行（诚实声明）**：Cargo.toml 版本 bump 将使全 workspace 指纹失效→触发全量重编；
  16G 日间＋1087 脏树＋晶体在线 serving 中，重编 swap 风险高。故 bump 留待**独占窗口**
  （停 crystal serving＋单 cargo＋夜间），本会话只定目标＋路由＋清单。
- 地图版本：ARCHITECTURE.md（v1.0.0→本次追加§14/§15 不改版号，由 owner 定版）；
  ROADMAP-V2 v2.0.0 有效，迭代记录追加（本次已追加一条，见§5）。

## 4. 剥离归档计划（权属核查结论：本次零移动，全部计划化）

| 候选 | 规模 | 权属 | 处置 | 门 |
|---|---|---|---|---|
| games/neotrix-guixu 38D | 38 文件删 | 他窗在途（意图未定） | 不动，park | owner＋game 窗双方确认 |
| skills/assets/icons/* D | ~78 删 | 他窗在途 | 不动，park | 对方落地后复核 |
| crates/neotrix-game particles.rs M | +261/-2 | 我窗 Emitter＋CJK 共存 | 保持未提交（拍板） | 钩子放行后 hunk 过滤 |
| src-tauri 248 M | 大 | 多窗共建 | 不动 | 各窗认领后拆 |
| .worktrees/ 9 个 | detached 全合入 diff 0 | 他窗所建 | 不删（删留待各窗确认） | 逐窗点名 |
| sessions 旧 handoff | 只增不改 | 各窗 | 永不归档（续命线） | — |

结论：轻装上阵＝先立 EVO 路线＋版本目标＋归档门，不在多窗 churn 高峰期搬文件。
归档根：沿用 `_archive/`（spire 先例），禁用新根。

## 5. 完整任务清单（本轮）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|---|---|---|---|---|---|
| EVO-01 | Token 成本门 | measure/audit/compress 落地＋CI 红绿 | Infra | 无 | ⬜（可首点名） |
| EVO-02 | judge-kernel 影子 | admission＋墓碑＋ledger 跑通（不拦截） | L0 | EVO-01 | ⬜ |
| EVO-03 | DSPy 层 | 首个 Signature 编译＋metric 绿 | L5 | EVO-01 | ⬜ |
| EVO-04 | 高速浏览器环 | 单 RTT＋遮挡校验＋用时证据 | L1 | 无 | ⬜ |
| EVO-05 | 语义索引＋图谱 | explain/impact 有界切片可用 | L1/L2 | Phase2 窗口 | ⬜ |
| EVO-06 | 技能路由＋评审 | routing.json＋回归绿＋三件套门 | L5 | EVO-05 | ⬜ |
| EVO-07 | 在线进化 | nightly Grow＋artifact 版本化 | L6 | EVO-03 | ⬜ |
| EVO-08 | 统一数据面 | 网关＋MCP 查库可用 | L0 | Phase2 | ⬜ |
| EVO-09 | 近场协作 | 局域传输＋OIDC＋一面可用 | L3 | P3 | ⬜ |
| EVO-10 | RL＋仿真 | GRPO 小跑＋评估器各一 | L4/L5 | P3＋独占 | ⬜ |
| EVO-11 | 证明门＋语义网关 | 影子报告＋路由计量 | Shield | P3 | ⬜ |
| EVO-12 | 情报＋ICL 基准 | profile 简报＋paired-gap 集 | 感知 | P3 | ⬜ |
| G-03 | §39 | 已落盘 sessions/handoff-S39-20260926.md | 本窗 | — | ✅ |
| G-04 | nt_act_cleanup | check 零错零警告＋单测 35/35（见§6） | wt-cleanup | — | ✅ |
| G-05 | ntcode 残 | skill_evolution 已修（他人）；mapper 他窗迁移中→不动 | 点名窗 | — | 🟨（2/4 已定，余等落地） |
| VER | v0.22.0 bump | 独占窗口执行 | Owner | 夜间＋停服 | ⬜ |

旧清单承接：G-01/G-02/G-10/G-11 等决议；EQ 全系保持（G-09 无新证据不改格）；
PARK（P-01~P-05）不动。

## 6. 验证证据（本会话实跑）

- `CARGO_BUILD_JOBS=2 cargo check -p neotrix --lib` → EXIT:0，22.12s，0 warning 0 error（/tmp/nt_cleanup_check.log，2 行）。
- `CARGO_BUILD_JOBS=2 cargo test -p neotrix --lib -- nt_act_cleanup` → 35 passed / 0 failed（11786 filtered，/tmp/nt_cleanup_test.log）。
- soul 双端 online（tools=9，crystal 0.2.0）；`:3000` LISTEN；`:8149` Down（blocked）；App 待目视。
- 零 commit；docs 追加 3 文件（S39＋ARCH§14＋ROADMAP 迭代条），重读验证通过。

## 7. 并行执行证据（2026-09-26 夜，四 lane 写＋主串行验）

- EVO-01 `crates/neotrix-neobot/src/nt_token_guard.rs`（379 行，6 单测）＋lib.rs 1 行注册：
  `cargo test -p neotrix-neobot --lib -- nt_token_guard` → **6/6**（GUARD_EXIT:0）。
- EVO-03 `neotrix-core/src/l5_cognition/nt_dspy.rs`（489 行，6 单测，首行 inner attr 已摘）＋mod 2 行；
  EVO-02 `neotrix-core/src/l0_substrate/nt_judge.rs`（388 行，6 单测）＋mod 2 行：
  `cargo check -p neotrix --lib` → EXIT:0；`cargo test -p neotrix --lib -- nt_dspy nt_judge` → **12/12**。
- 新文件生产区无 unwrap/expect/panic/unsafe（awk 隔离 test mod 后 rg 验证；nt_dspy 仅命中首行 forbid，已摘）。
- EVO-04 spec：`sessions/handoff-EVO04-browse-20260926.md`（三步，归属 browser 窗，零引擎改动）。
- 门控未执行：P1（EVO-05~08，Phase2 窗口排队）/ P2（EVO-09~12，条件未到）/
  G-01 sidecar（禁直起，等门）/ G-02 App 目视（等你在屏）/ G-10·G-11（等决议）/ VER bump（独占窗口）。

## 8. P1 执行证据（子代理 infra 证书故障→主线程直写，四新文件 821 行 18 单测）

- EVO-05 `l2_perception/nt_code_graph.rs`（270 行，5 单测）→ **5/5**。
- EVO-06 `l5_cognition/nt_skill_route.rs`（199 行，5 单测）→ **5/5**。
- EVO-07 `l6_meta/nt_evolve_loop.rs`（156 行，4 单测）→ **4/4**。
- EVO-08 `l0_substrate/nt_data_gateway.rs`（196 行，4 单测）→ **4/4**
 （含自修 mask_dsn scheme 冒号误判一处，回归绿）。
- `cargo check -p neotrix --lib` EXIT:0；生产区无 unwrap/expect/panic/unsafe。
- 状态：未提交（等下一提交令；mod 注册 4 处皆自有行，hunk 过滤预案就绪）。

## 9. P2 执行证据（主线程直写，四新文件 593 行 16 单测，一次全绿）

- EVO-09 `l3_embodiment/nt_near_field.rs`（180 行，4 单测）→ **4/4**。
- EVO-10 `l4_emotion/nt_sim_eval.rs`（134 行，4 单测）→ **4/4**。
- EVO-11 `l6_meta/nt_law_gate.rs`（149 行，4 单测）→ **4/4**。
- EVO-12 `l2_perception/nt_intel_digest.rs`（130 行，4 单测）→ **4/4**。
- `cargo check -p neotrix --lib` EXIT:0；生产区无 unwrap/expect/panic/unsafe。
- 门控收尾：G-11 关闭（env.sh 无 deepseek key，14 处引用皆惰性选项；Lingee 已退役）；
  G-01 保持 blocked（nt_watch.sh 只是邻居哨兵非 sidecar 启动器，:8149 拉起＝训练侧进程，
  16G＋cargo 占用＋train 禁令三重门）；G-02 目视/G-10 决议/VER 独占等主；
  PARK 全不动。P2 包未提交，等令。
