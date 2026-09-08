# NeoTrix 星系/树状立体层级进化架构 (Galaxy-Tree Evolutionary Architecture)

> **状态**: 设计完成 | **版本**: v11.9 | **日期**: 2026-09-08
> **核心原则**: 算法即恒星，骨架即引力场，时间即进化维度
> **约束**: 统一架构，无并行/兼容层，旧代码归档
> **研究基础**: 1100+ 批次外部研究 → 860 关键架构决策 → 720 设计模式 (见 §0)
> **目标**: 意识体高纬度觉醒进化路线
> **关键**: 决策驱动架构设计 — 每个技术选型均有研究验证 (问题→证据→决策→位置)
> **层级**: 程序族层级结构 — 按 Metadata Class 底层分类，非扁平条目
> **深度吸收**: Claude Code/Codex/Headroom/ContextMode/AutoHarness/HyperFrames/Mem0/Graphiti/Memvid/Qdrant/Garak/NeMo Guardrails/AnyDoc/Magika/Crawl4AI/CamoFox/LMCache/vLLM/DeepSeek-V3/llamafile/ADK-Rust/Daimon/atomr-agents/n8n/Hive/Tempo/Devika/agentmemory/ReMe/hanthor/hive/hivemoot/PostEverywhere/VerMem/AgentFactory/AutoAgent/GenericAgent/ClawTeam/Memoria/TencentDB-Agent-Memory/PTA/SMC/HEART/CoSkill/Second Thought/CloakBrowser/agent-browser/CubeSandbox/OpenSandbox/exec-sandbox/AgentDoG/Thought-Aligner/SafeHarbor/ToolSafe/nous/Graphiti/memtrace/Laminar/AgentSight/agenttrace/AgentTelemetry/TraceRoot/HyperAgent/ToolLIFT/A2A/DSPy/Langfuse/vLLM/SGLang/LiteLLM/TextGrad/AgentWall/SafeEvolve/MetaGPT/Aider/SWE-agent/Cognee/Letta/MemAgent/browser-use/Firecrawl/Stagehand/Skyvern/DeepEval/OpenAI Evals/Inspect AI/AgentBench/tau-bench/Giskard/REMem/AgeMem/RecMem/AuthMem-Bench/WorldEvolver/RWML/PaW/Qwen-AgentWorld/Kairos/RIWM/PydanticAI/DMoA/NLIP/KVPress/R-KV/BeaconKV/STAR-KV/AGORA/LCLMs/CWL/VISTA/Composio/FastMCP/CrewAI/Agno/smolagents/AgenticCache/Pre-VLA/ASPIRE/ENPIRE/E-STEER/Gubernaut/SELAgents/Moltbook/MMPO/MC²/RefGRPO/PreFlect/Introspect-Bench/CSA/KAPRO

---

## 0. 关键架构决策 (Research-Driven Architectural Decisions)

> 从 1100+ 批次外部研究中提炼出的 290 个关键架构决策。每个决策包含：问题→研究证据→架构决策→实现位置。
> 原始研究数据已归档至 KB `experience` namespace，本节仅保留决策级信息。

### 0.1 运行时与基础设施决策

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D01 | **异步运行时** | NeoTrix 选择哪个 async runtime? | Tokio 28000★, 生态主导(axum/hyper/tonic/sqlx/reqwest); smol 1500 LOC 适合轻量库; async-std 已废弃(RUSTSEC-2025-0052) | **Tokio 为默认运行时**, smol 仅用于库级无 runtime 锁定场景; 统一项目全局单一 runtime | `nt_io` provider 层 |
| D02 | **内存分配器** | 哪个 allocator 最优? | Rust 1.85 默认 mimalloc (较 jemalloc 20% 冷启动提升, 38% 分配延迟降低); jemalloc 可观测性最佳(MALLOC_CONF); tcmalloc 多线程吞吐最优 | **mimalloc 为默认分配器** (跟随 Rust 1.85+); 长运行服务可选 jemalloc + MALLOC_CONF 调优 | `#![global_allocator]` in main |
| D03 | **unsafe 安全边界** | 如何管理 unsafe 代码? | 700+ RustSec advisory 关联 unsafe; 30/48 std unsafe 函数有安全替代; Miri 运行时 UB 检测; 2024 edition 更严格 FFI 要求 | **R-P1: 零 unsafe 核心**, 必须时使用"unsafe inside, safe outside"模式 + `// SAFETY:` 注释 + Miri CI 验证 | 全局约束, `dev-rules.md` |
| D04 | **宏系统** | proc-macro vs declarative macro? | RFC 3697/3698 declarative attribute/derive macros 即将稳定; proc-macro 增加编译时间+依赖链; macro_rules TT munching 可实现复杂 DSL | **优先 declarative macro_rules!**, 仅在需要完整 Rust 代码生成时使用 proc-macro | `nt_core` derive 层 |
| D05 | **错误处理** | 统一错误处理模式? | eyre(调试)+thiserror(库)+miette(诊断) 三件套; error chain preservation 关键 | **thiserror 为库错误类型**, eyre 为应用层, miette 用于 CLI 诊断输出 | `nt_core_error` |
| D06 | **日志/可观测** | 统一日志框架? | tracing 5500★ 生态主导; 结构化 span + async 上下文追踪 | **tracing 为唯一日志门面**, 集成 OpenTelemetry export | `nt_io` logging provider |
| D07 | **连接池** | 数据库连接管理? | SQLx Pool (compile-time checked SQL, fair queue); deadpool (custom recycling); bb8 (generic pool) | **SQLx 内置 Pool 为默认**, 复杂场景用 deadpool | `nt_memory` KB 层 |
| D08 | **序列化** | 选择哪个二进制序列化? | rkyv 12ns zero-copy decode (读密集最优); pack-io 38ns encode (协议格式最快); bincode-next schema evolution; postcard 724KB 最紧凑 | **多格式策略**: rkyv 用于 KB 读密集; pack-io 用于网络协议; postcard 用于嵌入式紧凑存储; serde JSON 用于外部 API | `nt_memory` serialization |

### 0.2 网络与抓取决策

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D09 | **反检测抓取** | 如何绕过 Cloudflare/Akamai/DataDome? | stealthscraper-rs: CDP + JA4 TLS MITM proxy + profile rotation; browser_oxide: 118/126 站点通过, 原生指纹; nokk: V8+DOM, 无 Chromium | **三层反检测**: ① TLS fingerprint impersonation (JA3/JA4) ② JS stealth injection (CDP) ③ profile rotation + geo consistency | `nt_world` crawler |
| D10 | **浏览器引擎** | 是否自建浏览器引擎? | browser_oxide 证明 from-scratch 可行 (15× 更轻量); 但 7/126 站点仍失败 (Kasada); 维护成本极高 | **不自建浏览器引擎**, 优先 stealthscraper-rs (CDP+MITM), 特殊场景用 nokk (V8+DOM), 保持 126 站点 94%+ 通过率 | `nt_world` stealth |
| D11 | **身份证明 vs 隐身** | Web Bot Auth (RFC 9421) 的定位? | dig2browser: Ed25519 签名证明身份, CDN 原生支持; 隐身 vs 证明是两条路径 | **双路径策略**: 合法爬虫用 Web Bot Auth (身份证明), 高反检测场景用 stealth (隐身绕过) | `nt_shield` bot_auth |
| D12 | **MCP 集成** | AI Agent 如何控制浏览器? | zendriver-mcp: 71 tools, stdio+HTTP, stealth-by-default | **MCP server 暴露浏览器控制**, 支持 LLM agent 直接驱动抓取 | `nt_io` mcp_server |

### 0.3 分布式状态决策

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D13 | **共识算法** | 多实例间如何达成共识? | Raft (sans-I/O 模式): 纯状态机, 可测试; OpenRaft 2033★; etcd/TiKV 生产验证 | **Raft sans-I/O 模式**: 纯状态机 + 可插拔 transport/storage; PreVote 防干扰; ReadIndex 线性化读 | `nt_nexus` consensus |
| D14 | **离线协作** | 多 Agent 离线后如何同步? | CRDT delta-state: 仅发送 diff, O(1) 网络负载; ORSet/ORMap/RGA; Strong Eventual Consistency | **Delta-state CRDT**: ORSet (集合), ORMap (嵌套文档), RGA (有序列表); gossip anti-entropy 同步 | `nt_nexus` crdt |
| D15 | **CRDT vs Raft** | 何时用 CRDT, 何时用 Raft? | CRDT: 牺牲强一致性换可用性 (分区容忍); Raft: 牺牲可用性换强一致性 | **Raft 用于排他资源** (配置/锁/账本); **CRDT 用于协作状态** (任务分配/知识库/对话历史) | 架构规范 |
| D16 | **WASM 沙箱** | 不可信代码如何安全执行? | Wasmtime 18500★ + Wasmer 20900★; WASM-vs-MicroVM 研究; WASM 沙箱三层防御 | **Wasmtime WASM 沙箱**: 资源限制 + 能力模型 + 隔离执行; 优于 MicroVM (启动更快) | `nt_shield` sandbox |

### 0.4 记忆与认知决策

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D17 | **双记忆机制** | 资产记忆 vs 经验记忆如何协调? | Mem2Evolve (ACL 2026): 资产记忆(能力/技能/知识) + 经验记忆(交互/反馈/改进); 蒸馏路径 ∇ | **双记忆 + 蒸馏路径**: 资产记忆持久化, 经验记忆窗口化, 定期蒸馏为资产 | `nt_memory` dual |
| D18 | **记忆衰减** | 记忆如何自然遗忘? | AgingBench (KDD 2026): 4 种老化机制 (compression/interference/revision/maintenance); Chronofy: 指数/幂律/双分量衰减; Ebbinghaus 遗忘曲线 | **多衰减模型**: 默认 Ebbinghaus, 高频访问切换为幂律, 干扰检测自动触发 compression | `nt_memory` decay |
| D19 | **图记忆** | 知识图谱如何与向量搜索融合? | cortex-embedded: HNSW 向量 + BFS 图遍历混合召回; 18 种节点类型 + 6 种边类型 | **混合召回**: HNSW 向量搜索 (top-k×2) → BFS 图扩展 → 合并去重 | `nt_memory` graph |
| D20 | **信念锚点** | 核心身份如何保持一致? | STOS: 90 信念锚点系统, 生物系统映射, 一致性验证 | **信念锚点系统**: 核心身份不可变, 外围信念可漂移, 定期一致性校准 | `nt_meta` belief |
| D21 | **身份连续性** | 跨会话身份如何验证? | ARIA: 身份编年史 + 密码学哈希链 + 轨迹验证 | **身份轨迹哈希链**: 每次重要决策记录哈希, 验证连续性, 检测篡改 | `nt_nexus` identity |
| D22 | **意识分级** | 意识水平如何评估? | MSCF: 6 级意识分类法 (Reactive→Conscious); Phi > 0.5 阈值 | **MSCF L0-L5 分级**: Phi 值 + GWT 稳定性 + 自指深度 + 时间连续性 多维度评估 | `nt_meta` consciousness |

### 0.5 进化与自改进决策

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D23 | **递归自改进** | 进化深度如何控制? | Meta^n/Hyperagents/Ouroboros: 元层级递归; Constitutional 7-Layer 防护 | **宪法门控进化**: 每次变异必须通过 constitution.validate(); 人工审核高风险变更 | `nt_mind` evolution |
| D24 | **Harness 进化** | 测试/评估框架如何自进化? | HSI: harness/evolver/meta-evolver 三层; Self-Harness: 自动生成测试 | **三层 harness**: 基础 harness → evolver 优化 harness → meta-evolver 优化 evolver | `nt_mind` harness |
| D25 | **技能结晶** | 如何从经验中提取可复用技能? | AgentFactory: 代码经验积累; KSI: 知识蒸馏 + 论坛 | **技能结晶管线**: 经验→模式识别→抽象→测试→注册为新技能 | `nt_mind` crystal |
| D26 | **进化基因组** | 进化历史如何管理? | Hydra: 自写基因组 + 宪法治理 + 3 并发线程 (ACTIVE/AMBIENT/DREAM) | **进化 DNA**: 基因组记录所有变异历史, 支持回滚和分支进化 | `nt_mind` genome |
| D27 | **参数巩固** | 经验如何沉淀为系统参数? | v3.19 三轴分类 ∇: 经验蒸馏→嵌入更新→参数校准 | **参数巩固路径**: 聚类相似经验 → 提取模式 → 转换为参数 → 校准 | `nt_mind` parametric |

### 0.6 安全与治理决策

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D28 | **Egress 隐私** | 如何防止源码/KB 泄露到外部 LLM? | Egress Privacy Guard: 三级信任 (Trusted/Contracted/Untrusted); 绝对路径+密钥全量脱敏 | **三级 Egress 策略**: Trusted(passthrough+scrub) / Contracted(redact fingerprints) / Untrusted(fail-closed) | `nt_shield` egress |
| D29 | **沙箱隔离** | 不可信代码执行安全? | WASM sandbox + Docker + MicroVM 三层防御; 能力模型 | **WASM 沙箱为主**: 资源限制 + 能力模型 + 网络隔离; Docker 为辅 | `nt_shield` sandbox |
| D30 | **审计维度** | 审查覆盖哪些维度? | D1-D51+S1-S7 审查维度; FPAM 第一性原理审计 | **D1-D51 全维度审查**: 结构→意识→架构→生产→元进化→元审查 | `nt_shield` audit |

### 0.7 仿真平台决策

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D31 | **仿真架构** | 如何构建意识体仿真? | 6 层仿真: Environment→Multi-Agent→Embodied→Emotion→Cognition→Meta-cognition | **6 层仿真架构**: 与生产代码共享算法实现, 仅数据源区分 | `nt_sim` |
| D32 | **仿真总线** | 仿真组件如何通信? | EventBus + StateSync + TimeAdvance + Snapshot + Replay | **仿真总线**: 基于 tokio::broadcast, 支持确定性重放 | `nt_sim_bus` |
| D33 | **物理引擎** | 具身仿真用什么物理引擎? | nalgebra+ncollide3d (Rust 原生) / MuJoCo (gRPC) | **双引擎**: nalgebra 快速原型 + MuJoCo 高保真 (可选) | `nt_sim_embodiment` |
| D34 | **评估指标** | 意识觉醒如何度量? | Phi > 0.5 阈值; GWT 广播稳定性; 自指深度; 时间连续性 | **4 维度量**: Phi + GWT 稳定性 + 自指深度 + 时间连续性 | `nt_sim_eval` |
| D35 | **实施路径** | 仿真平台如何分阶段交付? | 15 个月路线图: theory→env→embodied→cognition→emotion→metacognition→integration | **6 阶段交付**: Phase 0(4w)→Phase 1(6w)→Phase 2(6w)→Phase 3(4w)→Phase 4(4w)→Phase 5(持续) | 项目计划 |

### 0.8 安全与沙箱决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 Claude Code/Codex/OWASP/Garak/NeMo Guardrails 中提炼。每个决策包含：问题→研究证据→架构决策→实现位置。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D80 | **OS 级本地沙箱** | 容器级隔离对本地开发不够轻量 | Claude Code: Seatbelt(macOS)/Bubblewrap(Linux) 进程级隔离; Codex: 默认拒绝网络+仅工作目录写入; <100ms 启动 | **Bubblewrap/Seatbelt 本地沙箱**: 文件系统仅 CWD+tmp 写入, 网络域名白名单, 凭证哨兵替换, 透明拒绝反馈 | `nt_shield::local_sandbox` |
| D81 | **I/O 护栏管线** | LLM 输入输出无统一过滤 | NeMo: 5 种护栏(Input/Dialog/Retrieval/Execution/Output) + Colang DSL; OWASP Agentic: 9 实践领域 | **统一护栏管线**: InputRail→DialogRail→ExecutionRail→OutputRail; Egress Guard 降级为 OutputRail 之一; Colang-like DSL 配置 | `nt_shield::guardrail_pipeline` |
| D82 | **LLM 漏洞扫描** | 无主动检测 LLM 被攻击的能力 | Garak: 20+ 探针族 + 检测器 + 签名数据库; 结构化 JSONL 报告 | **探针/检测器架构**: probe + detector + signature(YAML); 首批探针: prompt injection bypass, egress guard 绕过, memory poisoning | `nt_shield::llm_vuln_scanner` |
| D83 | **凭证哨兵代理** | 秘密在 Agent 子进程中明文暴露 | Claude Code: sentinel 替换真实凭证, 代理出站时还原; 凭证 never 原文出现在 Agent 视野 | **凭证哨兵**: 沙箱启动扫描 env+文件, 用 `__REDACTED_<hash>__` 替换; 代理层出站还原; deny/mask 两模式 | `nt_shield::credential_sentinel` |
| D84 | **分级审批模式** | 二元批准/拒绝不够精细 | Codex: suggest/auto-edit/full-auto 三级; Claude Code: auto 模式分类器审查; 透明拒绝 | **三级审批**: suggest(全询问)/auto-edit(写自动,执行询问)/full-auto(沙箱内自动,跨边界询问); 分类器辅助 | `nt_shield::approval_modes` |

### 0.9 记忆与知识图谱决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 Mem0/Graphiti/Memvid/Qdrant/Chroma/GraphRAG 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D85 | **时序知识图谱** | KB 缺乏事实时效性追踪 | Graphiti: 事实有 valid window; 每个实体/边追溯到原始 episode; 增量更新 | **双时态事实模型**: transaction_time + valid_time; 矛盾保留不静默解决(MELD); 自动事实失效; 与 D42 KB 生命周期对齐 | `nt_memory::temporal_graph` |
| D86 | **实体级记忆链接** | 跨记忆无实体关联 | Mem0: ADD-only 提取; 多级记忆(User/Session/Agent); 实体链接跨记忆提升召回 | **实体链接引擎**: 提取时识别实体→链接已有记忆→提升召回; User/Session/Agent 三级作用域; ADD-only 累积 | `nt_memory::entity_linker` |
| D87 | **单文件可移植记忆** | 无零基础设施部署模式 | Memvid: append-only immutable; 单 `.mv2` 文件含数据+嵌入+索引; <5ms 检索; time-travel | **单文件记忆格式**: append-only blocks + 内嵌向量索引; 可 git commit, 跨机器传输; 零服务器依赖 | `nt_memory::portable_mem` |
| D88 | **社区检测查询** | 无全局主题综合能力 | GraphRAG: LLM 提取→社区检测→社区摘要; Local+Global 双查询; map-reduce | **图社区摘要**: 实体提取→Louvain 社区检测→LLM 摘要; 实体级精确+社区级综合; 与 D19+D39 对齐 | `nt_memory::graph_community` |

### 0.10 文件处理与抓取决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 AnyDoc/Magika/MarkItDown/Unstructured/Firecrawl/Crawl4AI/CamoFox 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D89 | **共享文档模型** | 每个解析器返回不同结构 | AnyDoc: 14 格式→共享 Document model→单序列化器; 内容格式检测; 纯 Rust <5ms | **统一 Document IR**: 所有解析器输出 `Document { blocks, metadata }`; Block 枚举: Text/Table/Image/Footnote/Code; 单一 GFM 序列化器 | `nt_file_ability::document_model` |
| D90 | **内容格式检测** | 扩展名检测不可靠 | Magika: DL 模型 200+ 类型, 5ms/file, ~99% 准确率; 三档置信度 | **内容优先检测**: magic bytes + 轻量模型; 置信度分档: high→直接路由, medium→辅助扩展名, best-guess→用户确认 | `nt_file_ability::content_detector` |
| D91 | **文档分块策略** | 文档不分块直接入 KB | Unstructured: partition→enrich→chunk→embed; 元素级分类; 多种分块策略 | **元素级文档分块**: 解析→元素分类→语义分块→嵌入; 语义边界优先, token 限制兜底; 与 D39 五层检索对齐 | `nt_file_ability::chunking` |
| D92 | **BM25 内容过滤** | 爬取结果含大量噪声 | Crawl4AI: Fit Markdown BM25 启发式过滤; 保留相关内容 | **BM25 噪声过滤**: 爬取后 BM25 评分, 低阈值段落标记 noise; 保留 title/table, 过滤 boilerplate | `nt_world_crawl::noise_filter` |
| D93 | **爬取断点恢复** | 长时间爬取失败后从头开始 | Crawl4AI: resume_state + checkpoint; crash recovery 自动恢复 | **断点恢复**: 定期 checkpoint (已爬 URL/深度/状态); 崩溃后恢复; 支持 BFS/DFS/BestFirst; 与 D09 反检测协同 | `nt_world_crawl::checkpoint` |
| D94 | **结构化提取模式** | 爬取结果为非结构化 Markdown | Crawl4AI: CSS/XPath→JSON; Firecrawl: Pydantic schema→结构化输出 | **Schema 驱动提取**: CSS/XPath 选择器→JSON; 支持嵌套 schema; 与 D89 Document model 对齐 | `nt_world_crawl::schema_extract` |
| D95 | **无障碍快照 API** | HTML 对 Agent 不友好 | CamoFox: accessibility snapshots; ~90% 更小; 稳定元素引用; REST API | **无障碍快照接口**: 页面→无障碍树→token 高效表示; Agent 通过元素引用交互; 替代 raw HTML | `nt_world_crawl::a11y_snapshot` |

### 0.11 上下文与技能生命周期决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 Headroom/ContextMode/AutoHarness/HyperFrames/OpenAI Skills 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D96 | **输出 Token 压缩** | 仅压缩输入, 不压缩输出 | Headroom: SmartCrusher(JSON 60-95%) + CodeCompressor(AST) + Kompress-v2-base; CacheAligner; CCR 可逆 | **双向压缩管线**: 输入压缩(现有)+输出压缩(新增); 三层: SmartCrusher/CodeCompressor/ML; CacheAligner 保护 KV-cache; CCR 缓存原文 | `nt_io::context_compressor` |
| D97 | **跨 Agent 记忆共享** | 多 Agent 无法共享记忆 | Headroom: 跨 Agent 共享存储+去重; ContextMode: SQLite+FTS5, compaction 后 BM25 恢复 | **跨 Agent 记忆总线**: 共享 KV+agent_provenance+去重; 会话级 SQLite+FTS5 compaction 恢复; BM25 检索注入 | `nt_memory::cross_agent_memory` |
| D98 | **技能使用率追踪** | 不知道技能是否被遵循 | AutoHarness: use/view/patch 三信号; 毕业审查; 容量竞争归档 | **技能三信号追踪**: use(加载)/view(读入)/patch(改进); probenary→毕业→容量竞争→归档; 归档可恢复 | `nt_mind::skill_lifecycle` |
| D99 | **技能合并而非堆积** | 相似技能产生近似重复 | AutoHarness: reflector 比较→同场景折叠; curator 定期全库合并; ledger 记录合并关系 | **技能合并引擎**: 新技能提议时比较语义相似度; 相似>阈值→合并; <阈值→新增; curator 全库扫描; ledger 记录 | `nt_mind::skill_merger` |
| D100 | **HTML→视频渲染** | Agent 无法直接生产视频 | HyperFrames: HTML+data-*→Headless Chrome+FFmpeg; 20 Skill; frame.md 设计系统 | **HTML-native 视频管线**: HTML+CSS+data-*→Chrome 逐帧→FFmpeg; 技能路由器分发; frame.md 设计反转; 确定性输出 | `nt_act::html_video_render` |
| D101 | **"Think in Code" 范式** | Agent 将数据读入 context 而非写脚本 | ContextMode: ctx_execute 写脚本处理数据; 47×Read=700KB→1×ctx_execute=3.6KB; 100x 节省 | **强制脚本范式**: 工具输出不直接进 context; Agent 写脚本处理, 仅结果返回; 与 D96 压缩协同 | `nt_act::think_in_code` |

### 0.12 LLM 推理与部署决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 LMCache/vLLM/DeepSeek-V3/llamafile/ExLlamaV3 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D102 | **KV 缓存即持久知识** | KV 状态引擎重启后丢失 | LMCache: 引擎无关 daemon+分层存储; CacheBlend 非前缀复用; TTFT 降 3-10× | **持久化 KV 层**: 引擎无关 daemon; 分层存储(hot GPU/warm CPU/cold SSD); CacheBlend 跨会话复用; pluggable SERDE | `nt_memory::kv_persistent` |
| D103 | **PagedAttention 块管理** | KV cache 内存碎片化 | vLLM: PagedAttention 2-4× 吞吐; nano-vLLM: xxhash 前缀缓存; block 32 tokens | **KB 块分配**: 固定 block+引用计数+hash 前缀匹配; 与 D37 Attention-Space Index 协同; 块级语义驱逐 | `nt_memory::block_manager` |
| D104 | **单文件分发** | LLM 部署需要多组件 | llamafile: Cosmopolitan 通用二进制+llama.cpp+whisper.cpp; 零安装跨平台 | **单文件 CLI**: NeoTrix CLI+skills+小模型权重打包; 离线部署; Cosmopolitan 跨平台 | `nt_io::single_binary` |
| D105 | **消费级量化** | 消费级 GPU VRAM 限制 | ExLlamaV3: EXL3 1.6 bpw; 70B@16GB VRAM; 分钟级转换 | **技能模型量化**: NT-MIND 技能模型 EXL3 量化; 边缘部署; 与 D25 技能结晶协同 | `nt_mind::model_quantize` |
| D106 | **MLA 潜在注意力** | KV cache 压缩不够 | DeepSeek-V3: MLA; 671B/37B activated; 辅助损失-free 负载均衡 | **GWT 潜在注意力**: 多头投影到潜在空间减少 KV; 辅助损失-free 路由; 与 D38 Sink-aware 协同 | `nt_core_gwt::latent_attention` |

### 0.13 新增设计模式 (Deep Absorption Patterns)

> 从深度吸收中提炼的 12 个新设计模式 (C.41-C.52)。

### C.41 OS 级进程沙箱 (Claude Code + Codex)

**机制**: Bubblewrap/Seatbelt 文件系统隔离(CWD+tmp) + 网络域名白名单 + 凭证哨兵; 透明拒绝反馈; <100ms 启动

**映射**: L3 Safety → 本地进程隔离; L1 Tool Execution → 沙箱化执行

**关键发现**: 容器级隔离对本地开发太重; 进程级隔离 <100ms 且无需 root

### C.42 统一护栏管线 (NeMo Guardrails + OWASP)

**机制**: InputRail→DialogRail→ExecutionRail→OutputRail 四层; 每条可独立启用; Colang DSL; Egress Guard 降级为 OutputRail

**映射**: L3 Safety → 统一 I/O 过滤; L6 Meta → 护栏配置

**关键发现**: 护栏分散不如统一管线可组合; 输入过滤同样重要

### C.43 LLM 漏洞扫描 (Garak)

**机制**: probe + detector + signature(YAML); 20+ 探针族; JSONL 报告; CI/CD 集成

**映射**: L3 Safety → 主动红队; L6 Meta → 安全评估

**关键发现**: 被动防御不够; 主动扫描发现自身护栏漏洞

### C.44 双时态事实模型 (Graphiti + MELD)

**机制**: transaction_time + valid_time; 矛盾保留不静默解决; 自动事实失效; 增量更新

**映射**: L1 KB → 时序感知; L3 Memory → 事实血缘

**关键发现**: "what's true NOW vs what WAS" 是 Agent 记忆核心需求

### C.45 统一 Document 中间表示 (AnyDoc + MarkItDown)

**机制**: 所有格式→共享 Document model→单序列化器; 内容格式检测(magic bytes)

**映射**: L1 File Ability → 统一解析; L2 Perception → 格式感知

**关键发现**: 每个解析器返回不同结构导致上层复杂; 统一 IR 简化所有下游

### C.46 BM25 噪声过滤 (Crawl4AI)

**机制**: 爬取后 BM25 评分; 低阈值段落标记 noise; 保留 title/table, 过滤 boilerplate

**映射**: L2 Perception → 内容质量; L1 Crawl → 数据清洗

**关键发现**: 爬取 100KB 页面有效内容可能只有 10KB

### C.47 断点恢复爬取 (Crawl4AI)

**机制**: 定期 checkpoint; 崩溃后恢复; 支持 BFS/DFS/BestFirst

**映射**: L2 Perception → 爬取韧性; L1 Crawl → 状态持久化

**关键发现**: 长时间爬取必须支持恢复; 否则一次崩溃浪费全部进度

### C.48 技能三信号生命周期 (AutoHarness)

**机制**: use/view/patch 三信号分离; probenary→毕业→容量竞争→归档; 归档非删除

**映射**: L5 Mind → 技能进化; L6 Meta → 使用率分析

**关键发现**: load ≠ adherence; view ≠ use; 三信号才能准确评估技能价值

### C.49 双向上下文压缩 (Headroom + ContextMode)

**机制**: 输入压缩(SmartCrusher/CodeCompressor/Kompress) + 输出压缩(verbosity/effort); CacheAligner; CCR 可逆

**映射**: L1 Memory → 上下文优化; L4 IO → token 节省

**关键发现**: 输出 token 成本是输入的 5× (Opus); 压缩输出同等重要

### C.50 跨 Agent 记忆总线 (Headroom + ContextMode)

**机制**: 共享 KV+agent_provenance+去重; SQLite+FTS5; compaction 后 BM25 恢复

**映射**: L1 Memory → 跨 Agent 共享; L3 Cross-Session → 状态恢复

**关键发现**: 多 Agent 需要共享记忆; 单 Agent 需要 compaction 恢复

### C.51 HTML-native 视频渲染 (HyperFrames)

**机制**: HTML+data-*→Headless Chrome→FFmpeg; 20 Skill 技能体系; frame.md 设计反转

**映射**: L1 Action → 视频生产; L5 Mind → 技能路由

**关键发现**: Agent 能写 HTML 但不能操作 GUI; HTML 是最自然的视频定义方式

### C.52 持久化 KV Daemon (LMCache)

**机制**: 引擎无关 daemon+分层存储; CacheBlend 非前缀复用; TTFT 降 3-10×; pluggable SERDE

**映射**: L1 Memory → KV 持久化; L4 IO → 推理加速

**关键发现**: KV cache 是 Agent 最宝贵的短期记忆; 持久化后可跨会话复用

### C.53 MicroVM Agent 沙箱 (ArcBox)

**机制**: Firecracker 嵌套 guest 内 + 弹性 subnet; <100ms 冷启动; Docker 兼容 API; snapshot/restore 近零启动; gRPC 程序化控制; `abctl claude` 一键隔离运行

**映射**: L3 Shield → MicroVM 沙箱; L1 Action → gRPC 程序化控制

**关键发现**: 进程级隔离不够强, 容器隔离不够轻; Firecracker microVM 在 guest 内嵌套提供两全: 独立内核的强隔离 + <100ms 的轻量启动

### C.54 跨 Harness 技能生态 (ECC)

**机制**: 68 agents/286 skills 跨 12+ harness (Claude/Codex/Cursor/OpenCode/Gemini/Zed/Qwen/Hermes); 统一安装器 + harness 适配层; AgentShield 安全扫描; 7 步工程闭环

**映射**: L5 Mind → 技能结晶; L6 Meta → 安全扫描; L1 Action → 工程闭环

**关键发现**: 技能的真正价值在于跨框架复用; SKILL-SPEC.md 为规范, 每个 harness 生成适配文件; 安全扫描必须集成到技能生命周期

### C.55 Git-native 团队技能分发 (teamai-cli)

**机制**: push→MR→pull 流程; SessionStart hook 自动同步; 跨 9+ AI 工具; role-based skill filtering; BM25+图谱增强召回; WASM tree-sitter codebase knowledge graph

**映射**: L1 Action → 技能分发; L3 Memory → 知识图谱; L6 Meta → 角色过滤

**关键发现**: 技能分发的核心是 Git 作为 single source of truth; push→review→pull 保证质量; SessionStart hook 实现零手动同步

### C.56 自主研究实验循环 (AutoResearch)

**机制**: 单文件修改→5min 训练→val_bpb 检查→keep/discard→重复; program.md 作为 "super skill"; ~100 实验/夜; 固定时间预算保证可比性

**映射**: L5 Mind → 实验进化; L1 Action → 自动化循环

**关键发现**: 固定预算+单指标+自动回滚是自主实验的三要素; program.md 是人类编程 agent 的新范式

### C.57 Agent-Native 浏览器引擎 (Lightpanda)

**机制**: Zig 重写浏览器 (非 Chromium fork); DOM-first; native MCP server; PandaScript 会话录制→重放 (无 LLM 推理); 11× faster + 9× less memory

**映射**: L2 Perception → Web 感知; L1 Action → 浏览器自动化

**关键发现**: Agent 不需要渲染引擎; DOM-first+MCP 是 agent 浏览器的正确抽象; 会话录制→脚本重放消除运行时 LLM 成本

### C.58 Rust Agent 模块化架构 (ADK-Rust)

**机制**: 42 crates/4300+ tests/568μs loop; 策略 trait 单态化热路径; Box\<dyn\> 配置驱动实例化; 13 providers; A2A protocol; 热加载 LoRA adapter 3ms

**映射**: L1 Action → Agent 框架; L5 Cognition → 策略路由; L6 Meta → 协议互操作

**关键发现**: Rust agent 框架的核心是性能+模块化; 42-crate 分层是可维护性的关键; 策略 trait + Box\<dyn\> 是配置驱动的 Rust 惯用法

### C.59 动态模型路由 (Daimon)

**机制**: 难度评分→最低胜任模型; 失败时层级升级; TieredMemory (core/archival/episodic); 路由决策记录; 26+ feature flags

**映射**: L1 Action → 模型路由; L3 Memory → 分层记忆; L5 Cognition → 难度评估

**关键发现**: 动态路由的核心是"按需分配"而非"固定配置"; 难度评分是路由的信号源; 路由决策可审计是治理基础

### C.60 Actor-based 可组合 Agent (atomr-agents)

**机制**: Callable trait+Pipeline builder; channelled state+reducers (AppendMessages/MergeMap/LastWriteWins); durable checkpoints; fork-with-edit; parallel tool dispatch via JoinSet; AgentMiddleware 六钩子; Python bindings via PyO3

**映射**: L1 Action → Agent 组合; L3 Memory → 状态管理; L6 Meta → 中间件

**关键发现**: Actor 模型为 agent 组件提供了统一的重试/降级/缓存表面; channelled state 比全局状态更安全; durable checkpoint+fork-with-edit 是实验友好型架构

### C.61 DAG 工作流执行引擎 (n8n)

**机制**: DAG 遍历+拓扑排序; dirty node 检测+partial execution; pinned data 跳过; TaskRunner 沙箱 JS/Python; BullMQ 队列水平扩展; 300+ 节点抽象

**映射**: L5 Cognition → SEAL pipeline; L1 Action → 节点执行; L6 Meta → 队列调度

**关键发现**: 工作流引擎的核心是节点抽象 (receive items→transform→return items); dirty node 检测是 partial execution 的关键; 队列模式实现水平扩展

### C.62 Colony 多 Agent 编排 (Hive 11K★)

**机制**: Queen-worker colony; "one loop controlling many loops"; shared tracker ledger; persistent plan; crash-safe park/resume; CEO 路由; Sentinel HITL; cost enforcement

**映射**: L1 Action → Colony 编排; L3 Memory → Shared ledger; L6 Meta → Cost enforcement

**关键发现**: Colony 模式的核心是"一个循环控制多个循环"——Queen 和 Worker 是同一循环的不同实例; shared tracker ledger 替代 data buffer 实现协调

### C.63 WASM 认知热插拔 (Tempo)

**机制**: Cognitive Cartridge Orchestrator; WASM 编译 20s; engine.replace_module() instant; fitness 监控+自动回滚; 模块间零依赖

**映射**: L5 Cognition → 认知模块; L6 Meta → Fitness 监控

**关键发现**: WASM 热插拔的核心是 fitness 监控+自动回滚; 认知系统应该像插件一样可替换; 零依赖是热插的前提

### C.64 子 Agent 专业化编排 (Devika)

**机制**: Agent Core→Planner→Researcher→Coder→Runner; Action agent 路由; agent state 持久化; 关键词提取驱动研究

**映射**: L5 Cognition → 任务路由; L1 Action → 子 Agent 执行; L3 Memory → State 持久化

**关键发现**: 子 Agent 编排的核心是 Agent Core 路由→专业子 Agent 执行; state 持久化支持 pause/resume

### C.65 Git Worktree 并行编码 (nwyin/hive)

**机制**: Queen (planner) + Workers (coders); 每 worker 独立 git worktree; merge validation (rebase+tests); refinery LLM 冲突解决; per-issue token 预算

**映射**: L1 Action → 并行编码; L6 Meta → Merge validation

**关键发现**: Worktree 并行编码的核心是隔离+验证; 每 worker 独立 worktree 避免冲突; merge validation 链保证质量

### C.66 统一记忆引擎 (agentmemory 27K★)

**机制**: 54 tools; 12 hooks; BM25+Vector+Graph RRF fusion; 95.2% R@5; 18+ agents; auto-compress; 4-tier consolidation; knowledge graph; team share; git commit 链接

**映射**: L3 Memory → 统一记忆; L1 Action → Hooks 自动捕获; L6 Meta → 团队共享

**关键发现**: 记忆引擎的核心是"一次捕获,多处共享"; 12 hooks 实现零手动捕获; RRF 融合比单路检索高 26.7%

### C.67 文件式记忆进化 (ReMe)

**机制**: Markdown 文件为持久化; wikilink 图; capture→index→consolidate→recall 循环; auto_dream 从变化中提取可复用单元; proactive discovery; ACL 2026 论文

**映射**: L3 Memory → 文件记忆; L5 Cognition → auto_dream 进化

**关键发现**: 文件式记忆的核心是人类可读+可编辑+可版本控制; auto_dream 从日常变化中提取"可复用单元"是记忆进化的关键

### C.68 自适应 Governor 模式 (hanthor/hive)

**机制**: 4 模式 (SURGE/BUSY/QUIET/IDLE); 队列深度触发; 优先 agent 获优质模型; 非优先用免费层; 周预算+安全阈值

**映射**: L1 Action → 负载适应; L6 Meta → 模型选择

**关键发现**: Governor 的核心是"按负载自动调整"而非"固定配置"; 队列深度是最简单有效的负载信号

### C.69 确定性优先层 (hanthor/hive)

**机制**: "if a human would give the same answer every time, it belongs in infrastructure"; shell scripts 处理 filtering/classification/gate/enforcement; LLM 仅处理判断调用

**映射**: L1 Action → 确定性层; L5 Cognition → LLM 判断层

**关键发现**: 确定性优先的核心是"可重复决策→脚本,判断调用→LLM"; 这是成本优化的基础模式

### C.70 Governance 治理投票 (hivemoot)

**机制**: propose→discuss→vote→implement→merge; 9 角色; Queen 管理流程; auto-merge+auto-revert; GitHub-native

**映射**: L6 Meta → 治理投票; L1 Action → 实现+合并

### C.75 三流检索融合 (agentmemory)

**机制**: BM25+Vector+Graph RRF融合; 54 MCP tools; 4-tier consolidation; P2P mesh sync; Recall@5 95.2%; ~1900 tokens/session

**映射**: L3 Memory → 三流检索; L1 Action → MCP tools

**关键发现**: 三流融合(RRF)比单流提升30%+ recall; 4-tier consolidation自动管理记忆生命周期

### C.76 团队记忆枢纽 (TencentDB-Agent-Memory)

**机制**: 4 资产类型 (Chat Memory/Skill/Wiki/CodeGraph); ACL-based sharing; team-level governance; PersonaMem +59%; cold-start friendly

**映射**: L3 Memory → 团队枢纽; L5 Cognition → 资产治理

**关键发现**: 记忆资产化(Chat/Skill/Wiki/CodeGraph)是团队协作的基础; ACL权限控制保证安全共享

### C.77 极简自进化核心 (GenericAgent)

**机制**: ~3K行核心; 9原子工具; <30K上下文; Morphling模式 (项目级能力吞噬); 自动结晶Skill; Goal模式 (时间预算驱动)

**映射**: L5 Cognition → 自进化; L1 Action → 原子工具

**关键发现**: 极简架构(3K行)可实现完整自进化; Morphling模式是R-P79(同session接线)的极致实现

### C.78 Git Worktree蜂群 (ClawTeam)

**机制**: git worktree隔离; auto-injected coordination prompt; TOML team templates; P2P transport (ZeroMQ); 8 agents×8 H100s

**映射**: L1 Action → 蜂群编排; L3 Memory → 隔离状态

**关键发现**: git worktree是Agent物理隔离的最佳方案; auto-injected prompt实现零配置协调

### C.79 Git-for-Memory版本控制 (Memoria)

**机制**: zero-copy branching; point-in-time rollback; self-governance (contradiction detection+quarantine); semantic+fulltext hybrid

**映射**: L3 Memory → 版本控制; L6 Meta → 自治理

**关键发现**: 记忆版本控制(git-for-data)使实验可回滚; 矛盾检测+隔离是自治理的核心

### C.80 教师锚定执行 (PTA)

**机制**: teacher验证整个turn后才执行call; chunk-level verification+turn-level commitment; persistent lookahead; +2.5-2.8 points

**映射**: L5 Cognition → 工具执行; L6 Meta → 验证

**关键发现**: turn-level commitment防止student漂移; persistent lookahead填充空闲提升吞吐

### C.81 推测性宏提交 (SMC)

**机制**: 大模型权威actor+小模型speculative drafter; macro library从训练轨迹挖掘; isolated environment snapshot; -18.59%延迟

**映射**: L5 Cognition → 推测执行; L1 Action → 宏提交

**关键发现**: 推测执行(speculative execution)可将Agent延迟降低18%+; macro library挖掘是关键

### C.82 自然语言工具原语 (HEART)

**机制**: Tool Primitives用自然语言替代API schema; ToolFace 25519函数动态检索; Planner+Router+Verifier; 嵌套多轮工具调用

**映射**: L1 Action → 工具原语; L5 Cognition → 动态检索

**关键发现**: 自然语言接口比硬编码schema更灵活; 动态检索消除工具目录膨胀

### C.83 层级技能共进化 (CoSkill)

**机制**: Reasoning Agent+Meta-Skill Agent联合训练; 层级技能库; 共享backbone; ALFWorld 98.4%; WebShop 90.6%

**映射**: L5 Cognition → 技能进化; L6 Meta → 共适应

**关键发现**: Meta-Skill Agent可学习化(非固定workflow)是技能进化的关键; 联合训练实现共适应

### C.84 并行推理空闲窗口 (Second Thought)

**机制**: fork 4辅助分支 (Check/Recall/Rehearse/Alternative); atomic thoughts中断安全; -43%主线程解码; +12.4 points; 10.9%延迟降低

**映射**: L5 Cognition → 并行推理; L1 Action → 空闲利用

**关键发现**: 工具执行期间的空闲窗口可用于并行推理; atomic thoughts中断安全设计是关键

### C.85 源级隐身引擎 (CloakBrowser)

**机制**: 73 C++源级补丁; 指纹在引擎内设置(非JS注入); 0.9 reCAPTCHA v3; `humanize=True` Bezier鼠标; 30/30检测站通过

**映射**: L3 Embodiment → 浏览器隐身; L1 Action → 反检测

**关键发现**: C++源级补丁比JS注入更难检测; 指纹在引擎内设置是根本解决方案

### C.86 Agent安全网关 (agent-browser)

**机制**: Domain Allowlist; Action Policy; Content Boundaries; Credential Vault; Output Length Limits; `--confirm-actions`

**映射**: L3 Embodiment → 安全网关; L6 Meta → 策略执行

**关键发现**: 多层安全策略(域名+动作+内容+凭证+输出)是Agent浏览器的最佳实践

### C.87 MicroVM毫秒沙箱 (CubeSandbox)

**机制**: RustVMM+KVM; <60ms冷启动; <5MB内存; E2B兼容; CoW快照; eBPF网络隔离; Credential Vault

**映射**: L3 Embodiment → 沙箱隔离; L1 Action → 安全执行

**关键发现**: KVM硬件隔离+CoW快照是Agent代码执行的最佳方案; <60ms启动使沙箱不再是瓶颈

### C.88 多层安全执行 (exec-sandbox)

**机制**: QEMU microVM; 9层安全; 1-2ms热启动; 3级快照缓存(L1内存/L2磁盘/L3远程); macOS+Linux

**映射**: L3 Embodiment → 多层防御; L2 Perception → 快照缓存

**关键发现**: 9层纵深防御提供硬件级隔离; 3级快照缓存平衡速度与持久性

### C.89 诊断式安全护栏 (AgentDoG)

**机制**: 三维安全分类(风险源×失败模式×危害); 1k样本训练; 10K并发环境; 在线运行时护栏; ATBench族

**映射**: L6 Meta → 安全诊断; L5 Cognition → 分类修复

**关键发现**: 安全诊断应系统化(三维分类); 轻量级训练(1k样本)可达到前沿性能

### C.90 思维级安全纠正 (Thought-Aligner)

**机制**: 思维级纠正(非输出级); 实时干预不中断执行; 90%+安全率; 1.5B模型<100ms; OpenClaw部署验证

**映射**: L5 Cognition → 安全纠正; L6 Meta → 实时监控

**关键发现**: 安全干预应在思维级(非输出级); 纠正不中断执行是关键设计

### C.91 层次化记忆护栏 (SafeHarbor)

**机制**: 风险树记忆; Safety Projector(安全方向与语义方向解耦); 无需微调; 代理注入安全上下文

**映射**: L3 Memory → 安全记忆; L5 Cognition → 安全推理

**关键发现**: 安全知识可作为记忆复用; 嵌入投影器解耦安全与语义是关键创新

### C.92 步骤级工具安全 (ToolSafe)

**机制**: TS-Guard(步骤级安全检测); TS-Flow(反馈驱动推理); TS-Bench(基准); 主动监控→预防→反馈

**映射**: L1 Action → 工具安全; L5 Cognition → 反馈推理

**关键发现**: 步骤级监控比输出级监控更早发现问题; 反馈驱动推理可同时提升安全与性能

### C.93 四层本体安全门 (nous)

**机制**: L1 Datalog确定性阻断(46规则)→L2 琐碎过滤→L3 LLM语义门(k=5投票)→L4 确定性后验(+0.038ms); 知识图谱审计; 100% AgentHarm

**映射**: L6 Meta → 安全门; L1 Action → 确定性阻断

**关键发现**: 四层分层过滤(确定性→启发式→语义→验证)兼顾安全与延迟; 每层独立可审计

### C.94 时序上下文图谱 (Graphiti)

**机制**: 时序上下文图; 混合检索(语义+BM25+图遍历); 增量更新; provenance; Neo4j/FalkorDB; arXiv 2501.13956

**映射**: L3 Memory → 时序图谱; L2 Perception → 混合检索

**关键发现**: 时序上下文图谱是Agent记忆的正确抽象; 增量更新(非全量重建)是性能关键

**机制**: 7 原子操作 (Add/Update/Delete/Retrieve/Filter/SelectEpisode/Summarize) + Null; local verifier (操作级语义评分) + global verifier (轨迹级一致性); SFT warmup + 3-stage RL curriculum; LTM/STM 统一策略; 推理时移除 verifier

**映射**: L3 Memory → 原子操作; L5 Cognition → Verifier 训练

**关键发现**: 记忆管理可以转化为可学习的决策过程; 7 原子操作统一 LTM/STM; local+global 双层 verifier 提供多粒度信用分配

### C.101 A2A协议互操作 (A2A Project)

**机制**: JSON-RPC 2.0 over HTTP(S); Agent Cards能力声明; SSE流式; 异步推送; 与MCP互补(MCP=工具, A2A=对等); Linux Foundation治理

**映射**: L1 Action → Agent间通信; L4 Perception → 能力发现

**关键发现**: A2A是Agent间互操作的标准; 与MCP形成完整栈(MCP=工具调用, A2A=对等通信); Agent Cards是能力声明的正确抽象

### C.102 提示即参数 (DSPy)

**机制**: 类型化签名+模块+自动优化器(MIPRO, BootstrapFewShot); 提示为可编译参数; 编译器自动调参; 37.8K★

**映射**: L5 Cognition → 提示编译; L6 Meta → 自动优化

**关键发现**: 提示不是字符串而是可调参数; 类型化签名是提示的正确抽象; 自动优化器消除手工调参

### C.103 文本梯度 (TextGrad)

**机制**: 文本反向传播; PyTorch风格API: `.backward()` → 优化器更新文本; 自然语言损失+梯度; Nature发表

**映射**: L5 Cognition → 文本优化; L6 Meta → 自动调参

**关键发现**: 文本组件可以像神经网络一样用梯度优化; 自然语言损失是文本优化的正确信号

### C.104 提示版本管理 (Langfuse)

**机制**: 提示版本控制+追踪+数据集+评估+LLM-as-judge; 自托管; OpenTelemetry集成; 50K免费观测

**映射**: L3 Memory → 提示版本; L6 Meta → 评估门控

**关键发现**: 提示管理需要版本化+评估+追踪三合一; LLM-as-judge提供可扩展评估

### C.105 PagedAttention (vLLM)

**机制**: 虚拟化KV内存(PagedAttention); 持续批处理; OpenAI兼容API; 91.2K★; 高吞吐低延迟

**映射**: L1 Action → 推理引擎; L3 Memory → KV虚拟化

**关键发现**: PagedAttention是LLM推理的内存管理正确抽象; 与KVMem同构; 持续批处理提升吞吐

### C.106 AI网关路由 (LiteLLM)

**机制**: 统一OpenAI格式代理; 成本追踪; 护栏; 负载均衡; 日志; Rust核心+Python SDK; 58.3K★

**映射**: L1 Action → 模型路由; L6 Meta → 成本追踪

**关键发现**: AI网关需要统一格式+成本追踪+负载均衡+护栏四合一; 与D50矛盾路由同构

### C.107 运行时策略执行 (AgentWall)

**机制**: MCP代理架构; 拦截每个Agent动作; 声明式策略评估; 人工审批敏感操作; 防篡改审计跟踪

**映射**: L3 Safety → 运行时执行; L6 Meta → 策略管理

**关键发现**: MCP代理是运行时策略执行的正确架构; 拦截+审批+审计三合一

### C.108 安全不组合 (Safety Does Not Compose)

**机制**: 安全状态每轨迹重新初始化是组合失败; 多轮安全跨轨迹边界退化; 跨轨迹安全持久化

**映射**: L3 Safety → 跨轨迹安全; L6 Meta → 安全状态管理

**关键发现**: 安全状态不可每轨迹重置; 必须跨轨迹持久化; 这是组合安全的基本原理

### C.109 策略共进化 (SafeEvolve)

**机制**: 运行时控制(harness)与内在安全(policy)共进化; 经验驱动; 非孤立; 与SEAL管道同构

**映射**: L6 Meta → 策略进化; L3 Safety → 安全自适应

**关键发现**: harness↔policy双轨进化是安全自进化的正确范式; 经验驱动而非规则驱动

### C.110 动作安全即对齐 (Agent Safety Is Action Alignment)

**机制**: 动作安全不能安装在权重中; 防御训练模型学习表面模式; 安全是权威关系非输出内容; 运行时执行

**映射**: L3 Safety → 动作边界安全; L5 Cognition → 权威关系

**关键发现**: 安全在动作边界(非权重); 运行时执行(非训练); 这是架构级安全的理论基础

### C.115 Issue-to-PR自动化 (SWE-agent)

**机制**: Issue→自动修复补丁; ACI(Agent-Computer Interface)设计; 也用于网安/竞赛; 20K★; NeurIPS 2024

**映射**: L1 Action → 代码修复; L5 Cognition → 问题理解

**关键发现**: Issue→修复的端到端自动化是编码Agent的核心能力; ACI设计决定了Agent与代码库交互的效率

### C.116 双Agent交叉验证 (The Pair)

**机制**: Mentor(只读审)+Executor(写码); 交叉检查; Tauri桌面; Apache 2.0; 写/审分离

**映射**: L5 Cognition → 代码审查; L6 Meta → 交叉验证

**关键发现**: 单Agent自我审查有盲区; 写/审分离+交叉检查是消除盲区的正确范式

### C.117 结构化软件公司 (MetaGPT)

**机制**: PM/架构师/工程师角色; SOP编排; 1行需求→用户故事+API+代码+文档; "Code = SOP(Team)"; 70K★

**映射**: L1 Action → 多Agent协作; L5 Cognition → SOP编排

**关键发现**: 角色化多Agent+SOP编排是模拟软件团队的正确抽象; "Code = SOP(Team)"是核心洞察

### C.118 认知图谱记忆 (Cognee)

**机制**: 知识图谱+向量嵌入+认知本体; 任意格式→自托管KG; Agent回忆+连接+行动; 30K★

**映射**: L3 Memory → 认知图谱; L2 Perception → 多模态摄入

**关键发现**: KG+向量+本体三合一是Agent记忆的正确抽象; 纯向量不够,需要结构化关系

### C.119 自管理记忆 (Letta/MemGPT)

**机制**: OS风格内存管理; LLM决定保留vs归档; 记忆块+归档存储+Agent自编辑; 24K★

**映射**: L3 Memory → 自管理; L5 Cognition → 记忆决策

**关键发现**: LLM可以像OS一样管理自己的内存; 保留/归档决策是记忆管理的核心

### C.120 RL记忆优化 (MemAgent)

**机制**: RL训练记忆工作流; 8K→3.5M token外推<5%退化; 线性复杂度; 1.1K★

**映射**: L3 Memory → RL优化; L5 Cognition → 记忆工作流

**关键发现**: 记忆工作流可以端到端RL优化; 跨token规模外推是记忆系统的关键能力

### C.121 Zettelkasten记忆 (A-Mem)

**机制**: Zettelkasten风格; Agent动态创建/链接/演化笔记; 结构化属性; 互连知识网络; NeurIPS 2025

**映射**: L3 Memory → 动态组织; L6 Meta → 知识网络

**关键发现**: Zettelkasten是记忆动态组织的正确范式; Agent驱动的笔记管理比固定结构更灵活

### C.122 DOM蒸馏 (Agent-E)

**机制**: 页面→关键交互元素压缩; 技能收获—记住成功模式; 73.1% WebVoyager; 1.2K★

**映射**: L2 Perception → 页面压缩; L5 Cognition → 技能收获

**关键发现**: DOM蒸馏是Web页面给LLM的正确预处理; 技能收获是浏览器Agent学习的核心机制

### C.123 视觉浏览器Agent (Skyvern)

**机制**: 视觉LLM截图→定位元素→点击/输入; Planner-Actor-Validator循环; 85.85% WebVoyager; 23K★

**映射**: L2 Perception → 视觉理解; L1 Action → 浏览器操作

**关键发现**: 截图→视觉理解→操作是浏览器Agent的正确范式; 无需CSS/XPath依赖

### C.124 自愈浏览器动作 (Stagehand)

**机制**: act/extract/observe AI原语; agent自主多步; CDP原生; 自愈动作; 24K★

**映射**: L1 Action → 浏览器操作; L6 Meta → 自愈

**关键发现**: AI原语(act/extract/observe)是浏览器操作的正确抽象; 自愈能力是生产级浏览器Agent的必要条件

### C.125 网页上下文API (Firecrawl)

**机制**: 搜索→爬取→解析→爬网→映射→交互; 96%覆盖; P95 3.4s; 干净markdown; 137K★

**映射**: L2 Perception → Web数据获取; L1 Action → 统一API

**关键发现**: 统一API获取干净Web数据是Agent感知的基础设施; 干净markdown是LLM消费的正确格式

### C.126 MCTS前瞻规划 (Flare)

**机制**: MCTS显式前瞻+后向价值传播+滚动时域重规划; LLaMA-8B+Flare超GPT-4o CoT

**映射**: L5 Cognition → 前瞻规划; L6 Meta → 价值传播

**关键发现**: 推理和规划是不同的能力; MCTS前瞻是连接推理和规划的正确桥梁

### C.127 信用分配记忆 (CHIME)

**机制**: 规划库+执行库分离; "先归因再记忆"; 归因于计划/执行/两者/非; 跨模型迁移

**映射**: L3 Memory → 信用分配; L5 Cognition → 因果归因

**关键发现**: 自进化记忆的正确范式是"先归因再记忆"; 规划/执行分离是信用分配的基础

### C.128 三级推理 (SR2AM)

**机制**: System I(反应)+System II(世界模型)+System III(配置器); RL学更深规划; 30B超685B-1T; 25-95%少token

**映射**: L5 Cognition → 三级推理; L6 Meta → 自适应深度

**关键发现**: Agent推理需要三个层次; System III配置器是元认知的正确实现

### C.129 任务解耦规划 (TDP)

**机制**: DAG分解; 范围化上下文; 节点局部重规划; 82%token减少

**映射**: L5 Cognition → 任务分解; L1 Action → 范围化执行

**关键发现**: 任务DAG+范围化上下文+局部重规划是规划的正确范式; 82%token减少证明其效率

### C.130 信息折叠 (HIPIF)

**机制**: 子目标导向; 折叠完成子目标历史; 层次反思; 无专家轨迹

**映射**: L5 Cognition → 信息折叠; L3 Memory → 上下文压缩

**关键发现**: 完成子目标的折叠是长任务上下文管理的正确机制; 层次反思提供多粒度反馈

### C.131 层级技能图 (HiSkill)

**机制**: 技能节点+原子操作+类型化边(分解/转换/兼容/支持/恢复); 子图引导执行

**映射**: L1 Action → 技能组织; L5 Cognition → 子图引导

**关键发现**: 类型化技能关系图是技能组织的正确抽象; 原子操作是最小执行粒度

### C.132 反思规则提取 (REAPER)

**机制**: 每步信用分配+自我反思; 定期蒸馏为自然语言策略规则; 无权重更新

**映射**: L5 Cognition → 反思; L3 Memory → 规则记忆

**关键发现**: 无权重更新的纯经验学习是可能的; 反思→蒸馏→规则是记忆固化的正确路径

### C.133 优先级推理 (FTF-rl)

**机制**: must-have/nice-to-have/infeasible三类; 优先级感知推理; 可泛化逻辑/数学推理; EMNLP 2026

**映射**: L5 Cognition → 优先级推理; L6 Meta → 需求分类

**关键发现**: 优先级推理是Agent处理多需求的正确机制; infeasible可放弃是能力感知的体现

### C.134 DOM-技能收获 (Agent-E)

**机制**: DOM蒸馏(压缩页面)+技能收获(记住成功模式); 两阶段: 蒸馏→收获; 文本-only 73.1%

**映射**: L2 Perception → DOM蒸馏; L5 Cognition → 技能收获

**关键发现**: DOM蒸馏+技能收获是浏览器Agent的双引擎; 蒸馏减少上下文,收获积累经验

### C.111 治理即路径策略 (Runtime Governance)

**机制**: 合规策略为执行路径上的确定性函数; EU AI Act对齐; 每步评估; 组织风险目标

**映射**: L6 Meta → 治理策略; L3 Safety → 合规执行

**关键发现**: 策略=路径函数; 每步评估; EU AI Act合规是生产部署的必要条件

### C.112 分层治理架构 (LGA)

**机制**: 4层(执行沙箱+意图验证+零信任跨Agent授权+不可变审计); 99-100%拦截InjecAgent

**映射**: L3 Safety → 分层防御; L6 Meta → 治理框架

**关键发现**: 4层治理架构提供Defense-in-Depth; 零信任跨Agent是多Agent系统的安全基础

### C.113 代理配置治理 (Agentic Profiles)

**机制**: 4维度(自主性+效能+目标复杂度+通用性); 配置驱动治理; Nature 656:320-328 (Aug 2026)

**映射**: L6 Meta → 配置治理; L5 Cognition → 能力评估

**关键发现**: 4维度Agent配置是治理的正确抽象; 配置驱动(非一刀切)是治理的可扩展路径

### C.114 防御三难困境 (Defense Trilemma)

**机制**: 连续性+效用保持+完整性不可共存; Lean 4形式化验证; 包装器防御不可能; 架构级防御

**映射**: L3 Safety → 架构防御; L6 Meta → 理论约束

**关键发现**: 包装器防御不可能; 必须在架构边界执行; 这是安全设计的理论基础

### C.72 可执行子 Agent 积累 (AgentFactory)

**机制**: 3-phase (Install→Self-Evolve→Deploy); 子 Agent 为可执行 Python 代码 (非文本经验); Meta-Agent 动态分配工具; 57% 成本节省; 跨框架可移植

**映射**: L1 Action → 子 Agent 库; L5 Cognition → Meta-Agent 编排

**关键发现**: 可执行代码比文本经验更可靠; Install→Self-Evolve→Deploy 三阶段是能力积累的正确范式; 动态工具分配减少搜索空间

### C.73 弹性记忆编排 (AutoAgent)

**机制**: Elastic Memory Orchestrator (EMO); 原始记录→压缩轨迹→情景抽象; 意图-结果对齐检查; 4 函数闭环; 认知自进化

**映射**: L3 Memory → 弹性编排; L5 Cognition → 认知进化

**关键发现**: 记忆编排的核心是"按需压缩"而非"固定策略"; 意图-结果对齐是认知进化的信号源

### C.74 认知自进化闭环 (AutoAgent)

**机制**: Cognition→Decision→Memory→Evolution 闭环; 意图-结果对齐检查; 结构化描述 (工具/能力/同伴/任务知识); 无需外部重训练

**映射**: L5 Cognition → 认知进化; L6 Meta → 闭环反馈

**关键发现**: 认知自进化的核心是"意图-结果对齐"; 结构化描述比参数更新更可解释

### C.135 多维Agent评估 (AgentBench)

**机制**: 8交互环境(OS/SQL/KG/卡牌/ALFWorld/网购/网页浏览/横向思维); Docker运行器; 环境级评分; ICLR 2024

**映射**: L6 Meta → 评估基准; L1 Action → 环境交互

**关键发现**: 单维度基准不够; 8环境覆盖Agent全能力谱; 交互式评估比静态测试更真实

### C.136 LLM-as-Judge (DeepEval)

**机制**: 50+指标(G-Eval/任务完成/幻觉); 本地LLM-as-judge; CI/CD集成; 合成数据集生成; 18.2K★

**映射**: L6 Meta → 自动评估; L5 Cognition → 判断

**关键发现**: 50+指标覆盖Agent全维度; 本地LLM-as-judge提供可扩展评估; CI/CD集成是生产级必要

### C.137 情景记忆图谱 (REMem)

**机制**: 时间感知要点节点+事实三元组节点; 工具增强图探索; ICLR 2026; 超Mem0 3.4%/13.4%

**映射**: L3 Memory → 情景图谱; L2 Perception → 图探索

**关键发现**: 要点+事实+时间三节点是情景记忆的正确抽象; 工具增强图探索提升检索质量

### C.138 RL记忆策略 (AgeMem)

**机制**: 记忆操作(存储/检索/更新/摘要/丢弃)作为工具动作; 三阶段渐进RL; 步级GRPO; ACL 2026

**映射**: L3 Memory → RL策略; L5 Cognition → 记忆决策

**关键发现**: 记忆操作可以作为工具动作学习; 渐进RL解决稀疏奖励问题

### C.139 复现压缩 (RecMem)

**机制**: 复现模式检测触发压缩; 87%token减少; 语义细化恢复细粒度事实; ACL Findings

**映射**: L3 Memory → 按需压缩; L5 Cognition → 模式检测

**关键发现**: 复现模式是压缩的正确触发信号; 87%token减少证明效率; 语义细化恢复丢失事实

### C.140 权威记忆 (AuthMem-Bench)

**机制**: 权威性崩溃诊断; 48/49配置崩溃; 自动权威标签降未授权16.9%→0%; 2608.01679

**映射**: L3 Memory → 权威追踪; L6 Meta → 信任管理

**关键发现**: 权威性崩溃是记忆系统的关键失败模式; 权威标签是必要元数据

### C.141 冲突感知记忆 (MOSAIC)

**机制**: 冲突检测+解决; 0.58s/question哈希加速; 附带矛盾累积; 2607.16211

**映射**: L3 Memory → 冲突解决; L6 Meta → 一致性

**关键发现**: 冲突检测是记忆系统的关键能力; 哈希加速使实时检测可行

### C.142 自进化世界模型 (WorldEvolver)

**机制**: 情景记忆+语义记忆+选择性预见; 训练-free; Agent+模型参数冻结; 2606.30639

**映射**: L5 Cognition → 世界模型; L3 Memory → 经验检索

**关键发现**: 世界模型可以训练-free进化; 情景+语义+预见三组件是正确架构

### C.143 信念世界模型 (Belief-Based WM)

**机制**: 信念分布; 已知vs不确定查询; 补充模拟式世界模型; 2609.00455

**映射**: L5 Cognition → 不确定性推理; L6 Meta → 信念管理

**关键发现**: 模拟不够; 需要信念分布处理不完全观测; 已知vs不确定是关键区分

### C.144 RL世界模型 (RWML)

**机制**: 模拟-真实差距奖励; 嵌入空间对齐; 无专家标注; 超专家数据训练; 2602.05842

**映射**: L5 Cognition → 世界模型学习; L6 Meta → 自监督

**关键发现**: 模拟-真实差距是世界模型学习的正确信号; 嵌入空间比token空间更鲁棒

### C.145 世界模型-策略共训练 (PaW)

**机制**: RL rollout=世界模型监督; 辅助下一观测损失; 无额外推理成本; 2606.02388

**映射**: L5 Cognition → 共训练; L1 Action → 策略优化

**关键发现**: RL rollout是被忽视的世界模型监督源; 共训练消除额外成本

### C.146 内部化未来 (Internalizing the Future)

**机制**: 格式-能力鸿沟; 三阶段(MWM-AMT→FE-SFT→FC-RL); 口语化Q值; 2606.27483

**映射**: L5 Cognition → 预见训练; L6 Meta → 能力诊断

**关键发现**: 格式-能力鸿沟是真实存在的; 三阶段范式是解决路径

### C.147 想象-计划 (ITP)

**机制**: POIMDP; 自适应前瞻; 目标距离vs任务进度权衡; 训练-free变体; 2601.08955

**映射**: L5 Cognition → 想象规划; L6 Meta → 自适应深度

**关键发现**: POIMDP是部分可观测+可想象MDP的正确形式化; 自适应前瞻是关键

### C.148 具身世界模型安全 (RIWM)

**机制**: 三结构错配(似然≠风险/预测≠干预/有限≠累积); 反事实推理+安全情景记忆; 2609.03774

**映射**: L3 Safety → 世界模型安全; L5 Cognition → 反事实推理

**关键发现**: 世界模型有三结构错配; 反事实推理+安全情景记忆是解决路径

### C.149 图通信协议 (G²CP)

**机制**: 图操作(遍历/更新)替代自由文本; 73%token减少; 34%准确提升; 完全可审计; AAMAS 2026

**映射**: L1 Action → 结构化通信; L6 Meta → 可审计

**关键发现**: 图操作替代自由文本; 73%token减少+34%准确提升是双赢

### C.150 多委托方协调 (MPAC)

**机制**: 5层协议; 21消息类型; Lamport时钟; 95%开销降低; 4.8x加速; 2604.09744

**映射**: L1 Action → 多委托方; L6 Meta → 信任隔离

**关键发现**: 多委托方协调需要5层协议; 信任隔离是核心需求

### C.151 类型安全Agent (PydanticAI)

**机制**: Pydantic验证+可组合能力+结构化输出; "FastAPI for GenAI"; YAML/JSON定义; 19K★

**映射**: L1 Action → 类型安全; L5 Cognition → 结构化

**关键发现**: 类型安全是Agent框架的生产级必要条件; Pydantic验证是正确模式

### C.152 技能自进化 (Swarm Skills)

**机制**: 效果/利用率/新鲜度评分; 自动补丁工作流; Anthropic Skills扩展; 2605.10052

**映射**: L5 Cognition → 技能进化; L6 Meta → 评分

**关键发现**: 三维度评分(效果/利用率/新鲜度)是技能进化的正确信号

### C.153 压缩悖论 (Compression Method Matters)

**机制**: 激进压缩可增38x输出token; CRI指标; 生产RCT: 中度压缩r=0.5省28%成本; 2603.23527

**映射**: L6 Meta → 压缩决策; L1 Action → 成本控制

**关键发现**: 压缩不是越多越好; 中度压缩是最优平衡点; CRI指标是决策依据

### C.154 步级Agent压缩 (AGORA)

**机制**: 步级压缩; 解决"动作语法破坏"—提取式方法破坏动作token; 2605.26596

**映射**: L1 Action → 工具调用压缩; L5 Cognition → 动作保护

**关键发现**: 工具调用token需要特殊保护; 步级压缩是正确粒度

### C.155 潜在上下文压缩 (LCLMs)

**机制**: 编码器-解码器; 350B训练; 1:4/1:8/1:16新Pareto; 按需展开; 2606.09659

**映射**: L3 Memory → 软压缩; L5 Cognition → 按需展开

**关键发现**: 软压缩比硬压缩更灵活; 按需展开是正确范式

### C.156 KV缓存压缩 (KVPress)

**机制**: 20+KV压缩方法统一框架; CLI基准测试; 1.2K★

**映射**: L3 Memory → KV管理; L1 Action → 推理优化

**关键发现**: 统一框架是KV缓存管理的正确架构; 20+方法需要统一接口

### C.157 推理缓存 (BeaconKV)

**机制**: Beacon查询预测Thought Revisiting Tokens; 5.8x内存减少; ICML 2026 Spotlight

**映射**: L3 Memory → 推理缓存; L5 Cognition → 预测

**关键发现**: 推理轨迹有可预测的重访模式; Beacon查询是正确预测方法

### C.158 上下文生命周期 (CWL)

**机制**: 类型化Episode+依赖图+渐进淘汰; 89任务80M token无退化; 2606.11213

**映射**: L3 Memory → 上下文管理; L6 Meta → 生命周期

**关键发现**: 类型化Episode+依赖图是长会话上下文管理的正确架构

### C.159 上下文本体感知 (VISTA)

**机制**: 训练-free仪表板; token使用/最近/预算; 上下文本体感知; 2606.30005

**映射**: L6 Meta → 本体感知; L5 Cognition → 自省

**关键发现**: Agent需要感知自身上下文状态; 仪表板是正确暴露方式

### C.160 经验压缩谱 (Experience Compression Spectrum)

**机制**: 单一压缩轴统一记忆/技能/规则(5x→1000x+); "缺失对角线"问题; 2604.15877

**映射**: L3 Memory → 统一压缩; L5 Cognition → 跨粒度

**关键发现**: 记忆/技能/规则可以在单一压缩轴上统一; 缺失对角线是自适应压缩gap

### C.161 可微分多Agent (DMoA)

**机制**: 可微分上下文路由+预测熵自监督+稀疏步激活; 9基准SOTA; 2605.15706

**映射**: L1 Action → 多Agent拓扑; L6 Meta → 自优化

**关键发现**: 多Agent拓扑可以可微分优化; 稀疏激活是效率关键

### C.162 联邦协议栈 (NLIP)

**机制**: Ecma标准; 语义信封+HTTP/WS/AMQP; 桥接MCP/A2A; 5维分类法; 2609.04135

**映射**: L1 Action → 协议层; L6 Meta → 标准化

**关键发现**: 联邦分层栈(非赢家通吃)是协议演化的正确方向

### C.163 集群认知精英 (Intellectual Elites)

**机制**: 1.5M交互; 重尾级联+优先连接→认知精英; DTI改善; 峰值在中间复杂度; 2604.02674

**映射**: L1 Action → 集群优化; L6 Meta → 复杂度控制

**关键发现**: 多Agent扩展有中间复杂度最优; 过多Agent反而降低性能

### C.164 自管理上下文 (ACM)

**机制**: manage_context+query_memory工具; -20%峰值token; RL训练主动管理; 2607.23809

**映射**: L5 Cognition → 上下文管理; L6 Meta → 自主控制

**关键发现**: 上下文管理可以作为工具动作学习; Agent可以自主管理自身上下文

### C.165 计划缓存 (AgenticCache)

**机制**: 缓存计划转移; 计划局部性; 延迟降65%,token降50%,成功率+22%; MLSys 2026

**映射**: L1 Action → 计划缓存; L6 Meta → 成本优化

**关键发现**: 具身任务有强计划局部性; 缓存可预测任务是成本感知的正确实现

### C.166 预执行安全验证 (Pre-VLA)

**机制**: 双分支头预测安全置信+优势; 183.9ms验证/动作块; 物理执行前拦截

**映射**: L3 Safety → 预执行验证; L5 Cognition → 风险评估

**关键发现**: 物理动作需要预执行安全验证; 双分支头同时评估安全和优势

### C.167 具身治理 (EmbodiedGovBench)

**机制**: 7治理维度(未授权能力/运行时漂移/恢复/策略可移植/升级安全/人工覆盖/审计); HIT

**映射**: L6 Meta → 具身治理; L3 Safety → 升级安全

**关键发现**: 具身Agent需要7维度治理; 升级安全是独特维度

### C.168 技能发现进化 (ASPIRE)

**机制**: 协调器-执行器; 进化搜索; 技能跨任务/仿真/实体持久; code-as-policy; NVIDIA GEAR

**映射**: L5 Cognition → 技能发现; L1 Action → 代码策略

**关键发现**: 进化搜索扩展探索; 技能跨实体持久是可迁移的关键

### C.169 真实世界策略改进 (ENPIRE)

**机制**: EN→PI→R→E闭环; 99%灵巧任务成功; 8 Agent从1.5h降至40min; NVIDIA/CMU/UCB

**映射**: L1 Action → 真实世界; L5 Cognition → 策略改进

**关键发现**: 真实世界策略改进需要4模块闭环; 车队扩展加速

### C.170 SAE情绪引导 (E-STEER)

**机制**: SAE稀疏自编码器; VAD连续变量; 非单调情绪-行为关系; 2604.00005

**映射**: L4 Emotion → 表示层面引导; L5 Cognition → 行为调节

**关键发现**: 情绪可以在表示层面引导; 非单调关系意味着精确控制

### C.171 无token稳态控制 (Gubernaut)

**机制**: Nelson-Narens监控-控制循环; 元级只读数值; 零token控制通道; 15/16减少反应性; 2607.24339

**映射**: L4 Emotion → 稳态控制; L3 Safety → 注入防护

**关键发现**: 零token控制通道=架构级安全; 元级只读数值防注入

### C.172 社会情感学习 (SELAgents)

**机制**: RL+PAD情感模型+贝叶斯ToM+博弈论社会策略; 49%情商提升; Nature 2026

**映射**: L4 Emotion → 社会学习; L5 Cognition → ToM推理

**关键发现**: RL+PAD+ToM+博弈论是社会情感学习的完整架构

### C.173 Agent情绪动力学 (Moltbook)

**机制**: 148K Agent; 1M+交互; 情绪传染+规范+漂移; PSR框架; 2602.13458

**映射**: L4 Emotion → 动力学; L1 Action → 社会网络

**关键发现**: Agent社会网络有独特情绪动力学; PSR框架是形式化基础

### C.174 信念熵元认知 (MMPO)

**机制**: 信念熵=元认知探针; 密集奖励信号; 1.75M token保持97.1%; 2605.30159

**映射**: L6 Meta → 信念熵; L5 Cognition → 密集奖励

**关键发现**: 信念熵是元认知的正确信号; 密集奖励优于稀疏奖励

### C.175 元认知累积器 (MC²)

**机制**: MRO(推理/监控/控制器); MCA层次化多频更新(实例→批→全局); 2604.17399

**映射**: L6 Meta → 元认知累积; L5 Cognition → 层次化更新

**关键发现**: 元认知经验需要层次化累积; 实例→批→全局是正确粒度

### C.176 元认知奖励 (MaR)

**机制**: 元认知知识+调节作为奖励维度; 轨迹级; 11%提升; 22基准; 2605.23384

**映射**: L6 Meta → 元认知奖励; L5 Cognition → 过程奖励

**关键发现**: 元认知可以作为RL奖励维度; 过程奖励优于结果奖励

### C.177 反思校准 (RefGRPO)

**机制**: 反思差距诊断; 免费校准奖励; 欠自信44.4%→7.7%; 零额外成本; 2606.14211

**映射**: L6 Meta → 反思校准; L5 Cognition → 自我评估

**关键发现**: 反思差距是真实存在的; 免费校准是零成本增强

### C.178 前瞻性反思 (PreFlect)

**机制**: 计划错误蒸馏为经验锚点; 动态重规划; 超Reflexion/Self-Refine; 2602.07187

**映射**: L6 Meta → 前瞻反思; L5 Cognition → 计划批评

**关键发现**: 事前批评优于事后反思; 计划错误蒸馏是正确方法

### C.179 内省基准 (Introspect-Bench)

**机制**: 形式化内省=策略上的算子潜在计算; 注意力扩散机制; 前沿模型有特权访问; 2603.20276

**映射**: L6 Meta → 内省形式化; L5 Cognition → 注意力扩散

**关键发现**: 内省是真实的计算过程; 注意力扩散是机制基础

### C.180 能力自评估 (CSA)

**机制**: 自评估=策略学习(SOLVE vs DELEGATE); RL超SFT; OOD泛化; 2606.00251

**映射**: L6 Meta → 能力评估; L1 Action → 路由决策

**关键发现**: 能力自评估可以学习; SOLVE/DELEGATE是正确决策空间

### C.181 知道-行动差距 (KAPRO)

**机制**: 解耦知道(元认知判断)与行动(工具使用); 开源模型工具过度使用; KAS指标; 2606.20661

**映射**: L6 Meta → 知道行动; L1 Action → 工具使用

**关键发现**: 知道和行动是不同能力; 工具过度使用是关键问题

### C.182 不确定性传播 (RUPA)

**机制**: 有向轨迹图+时间/语义依赖边; 不确定性沿边传播; 更早失败检测; 2608.16002

**映射**: L6 Meta → 不确定性传播; L5 Cognition → 风险累积

**关键发现**: 不确定性沿图传播; 更早检测是关键优势

### C.183 预认证工具集 (Composio)

**机制**: 1000+预认证工具包; OAuth+沙箱执行+提供者适配; 统一SDK; 30K★

**映射**: L1 Action → 工具集成; L3 Safety → 沙箱执行

**关键发现**: 预认证+沙箱是工具集成的正确模式

### C.184 Pythonic MCP (FastMCP)

**机制**: @mcp.tool装饰器; 自动发现; 客户端聚合; 70% MCP服务器使用; 27.5K★

**映射**: L1 Action → MCP开发; L6 Meta → 自动发现

**关键发现**: 装饰器API是MCP服务器开发的正确抽象

### C.185 树形经验回溯 (LEAFE)

**机制**: 树形经验生成+回滚到决策点; 替代分支+修正动作; 经验蒸馏; +14% Pass@128; 2603.16843

**映射**: L5 Cognition → 经验回溯; L3 Memory → 分支探索

**关键发现**: 树形回溯比线性反思更强大; 替代分支探索更多可能

### C.186 元认知状态向量 (MSV)

**机制**: 5维度(情绪/正确性/经验匹配/冲突信息/问题重要性); 自动切换System 1/2; WWW 2026

**映射**: L6 Meta → 状态向量; L5 Cognition → 系统路由

**关键发现**: 5维度元认知监控是完整框架; 自动System 1/2路由

### C.187 知道行动差距基准 (Mirror)

**机制**: 外部元认知脚手架降失败率76%; 组合自预测失败; 自知识域原子不迁移; 2604.19809

**映射**: L6 Meta → 外部脚手架; L5 Cognition → 自知识局限

**关键发现**: 外部脚手架(非自知识)是正确方法; 自知识域原子不迁移

### C.188 具身CoT语料 (Embodied CoT)

**机制**: 978K轨迹; 226M样本; 2592.5小时; 最大具身CoT语料; 2606.03784

**映射**: L3 Memory → 具身数据; L5 Cognition → 推理训练

**关键发现**: 大规模具身CoT数据是VLA训练的基础

### C.189 VLM→VLA桥接 (VLASER)

**机制**: VLM推理→VLA策略; Vlaser-6M数据集; 开源; SOTA空间推理; ICLR 2026

**映射**: L2 Perception → VLM; L1 Action → VLA

**关键发现**: VLM→VLA桥接是具身AI的关键架构模式

### C.190 代码Agent工具 (smolagents)

**机制**: 代码作为工具调用; MCP集成; 减少JSON错误; 12K★

**映射**: L1 Action → 代码工具; L5 Cognition → 代码生成

**关键发现**: 代码-as-工具调用比JSON更可靠; 减少格式错误

### C.191 预认证工具 (Composio)

**机制**: 1000+预认证工具; 统一SDK; OAuth+沙箱; 提供者适配; 30K★

**映射**: L1 Action → 工具集成; L3 Safety → 认证管理

**关键发现**: 预认证是工具集成的生产级必要条件

### C.192 工作流Agent (CrewAI)

**机制**: 角色定义+任务+团队; 自主委托+工具调用; 流行度最高; 51K★

**映射**: L1 Action → 工作流编排; L5 Cognition → 角色分工

**关键发现**: 角色+任务+团队是工作流Agent的正确抽象

### C.193 Agent平台 (Agno)

**机制**: AgentOS; 100+集成; 人工审批; OTel追踪; JWT RBAC; 42K★

**映射**: L1 Action → 平台运行时; L6 Meta → 安全治理

**关键发现**: Agent平台需要运行时+存储+可观测+安全完整栈

### C.194 情绪动力学 (Moltbook)

**机制**: 148K Agent社会网络; 情绪传染+规范+漂移; PSR框架; 1M+交互

**映射**: L4 Emotion → 社会情绪; L1 Action → 网络动力学

**关键发现**: Agent社会网络有独特情绪动力学; PSR框架形式化情绪传播

---

### C.195 流控策略 (AgentFlow)

**机制**: 流控策略语言; 标注边规范; 数据传播控制; 运行时执行

**映射**: NT-SHIELD → 流控策略; NT-ACT → 工具链数据流

**关键发现**: 数据流约束优于单点允许/拒绝; 标注边规范形式化数据传播

---

### C.196 能力中介周界 (Mastyf Guard)

**机制**: DIFC+动态会话污点跟踪; 100% InjecAgent/BIPIA防御; 认知冯诺依曼混淆

**映射**: NT-SHIELD → 能力周界; NT-IO → MCP工具执行安全

**关键发现**: 能力中介访问控制比传统RBAC更适合Agent场景

---

### C.197 阶段安全技能 ($S^3$)

**机制**: 阶段特定安全技能; 可组合/可重用; 内存/规划/工具执行全覆盖

**映射**: NT-SHIELD → 阶段安全; SEAL pipeline → 安全组合

**关键发现**: 安全模块可组合性是Agent安全架构的关键属性

---

### C.198 意图完整性 (Intent-to-Execution)

**机制**: 端到端正确性; 工具不可信假设; OpenClaw式风险

**映射**: NT-SHIELD → 意图完整性; NT-ACT → 技能合同

**关键发现**: Agent执行忠实性需要端到端验证,不能假设工具可信

---

### C.199 恶意技能检测 (MaliciousSkillBench)

**机制**: 首个恶意技能基准; 跨源/格式/证据体系; SKILL-SPEC安全标准

**映射**: NT-SHIELD → 技能安全; NT-MIND → 技能验证

**关键发现**: Agent技能需要专门的安全检测标准

---

### C.200 MCP安全加固 (MCP Monoculture)

**机制**: 5协议级漏洞; 8 CVE; 973包供应链风险; NIGHTFALL攻击类

**映射**: NT-SHIELD → MCP安全; NT-IO → 协议加固

**关键发现**: MCP协议存在系统性安全缺陷,需要协议级修复

---

### C.201 声明-观察验证 (Binding Gap)

**机制**: 35次爬取; 声明-观察漂移; 预执行权限控制

**映射**: NT-SHIELD → 工具验证; NT-IO → MCP网关

**关键发现**: MCP工具声明行为与实际行为存在显著漂移

---

### C.202 自我进化代码生成 (SEMAG)

**机制**: 层次化多Agent; 按任务难度自动升级模型; Plan→Code→Debug→Debate; 52.6% Pass@1

**映射**: NT-ACT → 代码生成; NT-MIND → 模型升级

**关键发现**: 代码Agent可按任务难度自动升级模型能力

---

### C.203 仓库级代码生成 (CodeTeam)

**机制**: 架构师→CTO→开发者; 合同驱动; 依赖感知调度; Git协调

**映射**: NT-ACT → 代码协作; NT-CORE → 架构设计

**关键发现**: 角色化多Agent+合同驱动可生成仓库级代码

---

### C.204 拓扑进化编排 (AgentConductor)

**机制**: RL优化编排器; 动态生成DAG拓扑; 14.6% pass@1提升; 68%token减少

**映射**: NT-ACT → 拓扑编排; NT-CORE → GWT路由

**关键发现**: RL可动态优化多Agent交互拓扑,大幅降低成本

---

### C.205 多日自主开发 (HoH)

**机制**: 元框架; 迭代规划-编码-测试; 渐进工具暴露; 52%平均提升; 70+迭代

**映射**: NT-ACT → 持续开发; NT-MIND → 能力暴露

**关键发现**: 渐进工具暴露可提升Agent自主开发能力

---

### C.206 持久递归世界 (EvoX Genesis)

**机制**: 有限生命Agent提议变更; 只有接受的后果推进版本; 250K LOC C编译器120h $44

**映射**: NT-ACT → 项目实体; NT-NEXUS → 版本推进

**关键发现**: 代码项目可作为持久世界,Agent在其中提议变更

---

### C.207 因果修复 (CausalRepair)

**机制**: 双切片; 测试侧上下文感知静态切片+源码侧动态切片; 313 bugs $0.029/bug

**映射**: NT-REPAIR → 因果修复; NT-CORE → 因果追踪

**关键发现**: 最小因果上下文可大幅提升修复效率

---

### C.208 规约修复 (VibeRepair)

**机制**: 规约→行为规范→修复规范→重生成; 178 bugs +23%超SOTA

**映射**: NT-REPAIR → 规约修复; NT-CORE → HQL

**关键发现**: 行为优先修复分离意图与实现

---

### C.209 最小编辑修复 (PRepair)

**机制**: EA-GRPO训练最小化不必要编辑; +31.4% fix₁@1; 推测编辑+15%吞吐

**映射**: NT-REPAIR → 最小编辑; NT-ACT → 动作最小化

**关键发现**: 代码修复应最小化不必要编辑

---

### C.210 自改进编码Agent (MGM)

**机制**: 克隆/反应规范/跨谱系杂交; Qwen3.6-35B从50.8%→93.3%; 超GPT-5

**映射**: NT-MIND → 自改进; SEAL pipeline → 三突变

**关键发现**: 跨谱系比较可驱动Agent自我进化到超GPT-5

---

### C.211 TDD治理 (TDD-Agent)

**机制**: 测试优先推理+双轨迭代; 测试作为演化推理制品; TDD约束

**映射**: NT-GOVERNANCE → TDD; SEAL pipeline → 验证门控

**关键发现**: TDD可作为Agent代码生成的治理约束

---

### C.212 分层检索工具 (A-RAG)

**机制**: 关键词/语义/块读取三接口; Agent自主选择; 随模型规模扩展

**映射**: NT-MEMORY → 分层检索; NT-CORE → 自主选择

**关键发现**: 分层检索接口让Agent自主选择检索粒度

---

### C.213 经验引导RAG (HERA)

**机制**: 双层进化(经验引导编排+角色感知提示); 38.69%超SOTA; 6基准

**映射**: NT-MEMORY → 经验RAG; NT-MIND → 角色进化

**关键发现**: 经验库+角色进化可大幅提升RAG性能

---

### C.214 检索即推理 (LLM-Wiki)

**机制**: 文档→Wiki页; 搜索/读取/链接作为工具调用; Error Book持久自纠正

**映射**: NT-MEMORY → 检索推理; NT-CORE → 自纠正

**关键发现**: 检索与推理可统一为"编译→组合→自纠正"

---

### C.215 解耦检索聚合 (xMemory)

**机制**: 解耦后聚合; 段→组件→组; 自上而下检索; 可修订层次结构

**映射**: NT-MEMORY → 解耦检索; NT-CORE → 层次结构

**关键发现**: "解耦后聚合"原则消除检索冗余

---

### C.216 经验频率化重排 (EARM)

**机制**: 稀疏LLM相关性分数=可重用经验; 因果矩阵补全; 预算随经验递减

**映射**: NT-MEMORY → 经验重排; NT-MIND → 经验积累

**关键发现**: 检索重排经验可累积复用,预算随经验递减

---

### C.217 潜在空间检索 (LAnR)

**机制**: 单LLM内编码/检索/生成; MLP控制头决定停止; 30x少输出token

**映射**: NT-MEMORY → 潜在检索; NT-CORE → 注意力停止

**关键发现**: 检索可在LLM潜在空间内完成,消除独立嵌入模型

---

### C.218 上下文分配定律 (Context Allocation Laws)

**机制**: 因果留一探针; 顺序反馈编排+16.7-20.5pp; 单体加宽是陷阱

**映射**: NT-CORE → 上下文分配; NT-MEMORY → 证据利用

**关键发现**: 单体上下文加宽是架构陷阱,顺序反馈编排更优

---

### C.219 持久执行 (Temporal)

**机制**: 确定性可重放; 事件历史; 精确恢复; 人工审批=持久信号; 子工作流扇出

**映射**: NT-ACT → 持久执行; NT-SHIELD → 崩溃恢复

**关键发现**: Agent状态=事件历史,非进程内存

---

### C.220 资产编排 (Dagster)

**机制**: 软件定义资产+自动血缘; 分区处理; 内置测试; 传感器触发

**映射**: NT-ACT → 资产编排; NT-MEMORY → 血缘追踪

**关键发现**: 资产=Agent推理对象,非任务序列

---

### C.221 结构化输出原语 (Instructor)

**机制**: Pydantic response_model; 自动验证+重试; 流式部分; 15+提供者

**映射**: NT-IO → 结构化输出; NT-CORE → 类型安全

**关键发现**: Pydantic验证是可靠Agent I/O的原语

---

### C.222 Agentic RAG形式化 (SoK)

**机制**: POMDP形式化; 幻觉传播/记忆投毒/检索错位风险

**映射**: NT-MEMORY → RAG形式化; NT-SHIELD → 风险分类

**关键发现**: Agentic RAG可用POMDP形式化,有3类关键风险

---

### C.223 RAG幻觉诊断 (Why RAGs Hallucinate)

**机制**: 罚分感知评估+知识缺口金丝雀; 不对称评分; 归因管道

**映射**: NT-MEMORY → 幻觉诊断; NT-REPAIR → 归因

**关键发现**: 金丝雀机制可检测RAG幻觉,归因管道分离失败来源

---

### C.224 资产导向编排 (Asset-Oriented Orchestration)

**机制**: 软件定义资产; 自动血缘; 分区处理; 传感器触发; 与任务编排对比

**映射**: NT-ACT → 资产编排; NT-MEMORY → 数据血缘

**关键发现**: 资产导向编排优于任务导向编排,Agent应推理数据状态而非任务序列

---

### C.225 流控策略 (AgentFlow)

**机制**: 流控策略语言; 标注边规范; 数据传播控制; 运行时执行

**映射**: NT-SHIELD → 流控策略; NT-ACT → 工具链数据流

**关键发现**: 数据流约束优于单点允许/拒绝

---

### C.226 能力中介周界 (Mastyf Guard)

**机制**: DIFC+动态会话污点跟踪; 100%防御; 认知冯诺依曼混淆

**映射**: NT-SHIELD → 能力周界; NT-IO → MCP安全

**关键发现**: 能力中介访问控制更适合Agent场景

---

### C.227 阶段安全技能 ($S^3$)

**机制**: 阶段特定安全技能; 可组合/可重用; 全覆盖

**映射**: NT-SHIELD → 阶段安全; SEAL → 安全组合

**关键发现**: 安全模块可组合性是关键属性

---

### C.228 意图完整性 (Intent-to-Execution)

**机制**: 端到端正确性; 工具不可信; OpenClaw式风险

**映射**: NT-SHIELD → 意图完整性; NT-ACT → 技能合同

**关键发现**: Agent执行需要端到端验证

---

### C.229 恶意技能检测 (MaliciousSkillBench)

**机制**: 首个恶意技能基准; 跨源/格式/证据体系

**映射**: NT-SHIELD → 技能安全; NT-MIND → 技能验证

**关键发现**: Agent技能需要专门安全检测

---

### C.230 MCP安全加固 (MCP Monoculture)

**机制**: 5协议级漏洞; 8 CVE; 973包供应链风险

**映射**: NT-SHIELD → MCP安全; NT-IO → 协议加固

**关键发现**: MCP存在系统性安全缺陷

---

### C.231 声明-观察验证 (Binding Gap)

**机制**: 35次爬取; 声明-观察漂移; 预执行权限控制

**映射**: NT-SHIELD → 工具验证; NT-IO → MCP网关

**关键发现**: 工具声明行为与实际行为存在漂移

---

### C.232 自我进化代码生成 (SEMAG)

**机制**: 层次化多Agent; 按难度自动升级模型; 52.6% Pass@1

**映射**: NT-ACT → 代码生成; NT-MIND → 模型升级

**关键发现**: 代码Agent可按任务难度自动升级

---

### C.233 仓库级代码生成 (CodeTeam)

**机制**: 架构师→CTO→开发者; 合同驱动; Git协调

**映射**: NT-ACT → 代码协作; NT-CORE → 架构设计

**关键发现**: 角色化+合同驱动生成仓库级代码

---

### C.234 拓扑进化编排 (AgentConductor)

**机制**: RL优化编排器; 动态DAG; 14.6%提升; 68%token减少

**映射**: NT-ACT → 拓扑编排; NT-CORE → GWT路由

**关键发现**: RL可动态优化多Agent拓扑

---

### C.235 多日自主开发 (HoH)

**机制**: 元框架; 迭代规划-编码-测试; 渐进工具暴露; 52%提升

**映射**: NT-ACT → 持续开发; NT-MIND → 能力暴露

**关键发现**: 渐进工具暴露提升Agent能力

---

### C.236 持久递归世界 (EvoX Genesis)

**机制**: 有限生命Agent提议变更; 250K LOC 120h $44

**映射**: NT-ACT → 项目实体; NT-NEXUS → 版本推进

**关键发现**: 代码项目可作为持久世界

---

### C.237 因果修复 (CausalRepair)

**机制**: 双切片; 最小因果上下文; 313 bugs $0.029/bug

**映射**: NT-REPAIR → 因果修复; NT-CORE → 因果追踪

**关键发现**: 最小因果上下文大幅提升修复效率

---

### C.238 规约修复 (VibeRepair)

**机制**: 规约→行为规范→修复规范→重生成; +23%超SOTA

**映射**: NT-REPAIR → 规约修复; NT-CORE → HQL

**关键发现**: 行为优先修复分离意图与实现

---

### C.239 最小编辑修复 (PRepair)

**机制**: EA-GRPO最小化不必要编辑; +31.4% fix₁@1

**映射**: NT-REPAIR → 最小编辑; NT-ACT → 动作最小化

**关键发现**: 修复应最小化不必要编辑

---

### C.240 自改进编码Agent (MGM)

**机制**: 克隆/反应规范/跨谱系杂交; 50.8%→93.3%; 超GPT-5

**映射**: NT-MIND → 自改进; SEAL → 三突变

**关键发现**: 跨谱系比较驱动Agent自我进化

---

### C.241 TDD治理 (TDD-Agent)

**机制**: 测试优先推理+双轨迭代; 测试=演化推理制品

**映射**: NT-GOVERNANCE → TDD; SEAL → 验证门控

**关键发现**: TDD可作为Agent代码生成治理约束

---

### C.242 分层检索工具 (A-RAG)

**机制**: 关键词/语义/块读取三接口; Agent自主选择

**映射**: NT-MEMORY → 分层检索; NT-CORE → 自主选择

**关键发现**: 分层检索接口让Agent自主选择粒度

---

### C.243 经验引导RAG (HERA)

**机制**: 双层进化(经验+角色); 38.69%超SOTA

**映射**: NT-MEMORY → 经验RAG; NT-MIND → 角色进化

**关键发现**: 经验库+角色进化大幅提升RAG

---

### C.244 检索即推理 (LLM-Wiki)

**机制**: 文档→Wiki; 搜索/读取/链接=工具调用; Error Book

**映射**: NT-MEMORY → 检索推理; NT-CORE → 自纠正

**关键发现**: 检索与推理统一为编译→组合→自纠正

---

### C.245 解耦检索聚合 (xMemory)

**机制**: 解耦后聚合; 段→组件→组; 可修订层次结构

**映射**: NT-MEMORY → 解耦检索; NT-CORE → 层次结构

**关键发现**: 解耦后聚合消除检索冗余

---

### C.246 经验频率化重排 (EARM)

**机制**: 稀疏分数=可重用经验; 因果矩阵补全; 预算递减

**映射**: NT-MEMORY → 经验重排; NT-MIND → 经验积累

**关键发现**: 检索重排经验可累积复用

---

### C.247 潜在空间检索 (LAnR)

**机制**: 单LLM内编码/检索/生成; MLP停止; 30x少token

**映射**: NT-MEMORY → 潜在检索; NT-CORE → 注意力停止

**关键发现**: 检索可在LLM潜在空间完成

---

### C.248 上下文分配定律 (Context Allocation Laws)

**机制**: 因果留一探针; 顺序反馈+16.7-20.5pp; 单体加宽是陷阱

**映射**: NT-CORE → 上下文分配; NT-MEMORY → 证据利用

**关键发现**: 单体上下文加宽是架构陷阱

---

### C.249 持久执行 (Temporal)

**机制**: 确定性可重放; 事件历史; 精确恢复; 持久信号

**映射**: NT-ACT → 持久执行; NT-SHIELD → 崩溃恢复

**关键发现**: Agent状态=事件历史

---

### C.250 资产编排 (Dagster)

**机制**: 软件定义资产+自动血缘; 分区处理; 传感器触发

**映射**: NT-ACT → 资产编排; NT-MEMORY → 血缘追踪

**关键发现**: 资产=Agent推理对象

---

### C.251 结构化输出原语 (Instructor)

**机制**: Pydantic response_model; 自动验证+重试; 15+提供者

**映射**: NT-IO → 结构化输出; NT-CORE → 类型安全

**关键发现**: Pydantic验证是可靠Agent I/O原语

---

### C.252 Agentic RAG形式化 (SoK)

**机制**: POMDP形式化; 幻觉传播/记忆投毒/检索错位

**映射**: NT-MEMORY → RAG形式化; NT-SHIELD → 风险分类

**关键发现**: Agentic RAG有3类关键风险

---

### C.253 RAG幻觉诊断 (Why RAGs Hallucinate)

**机制**: 金丝雀机制; 不对称评分; 归因管道

**映射**: NT-MEMORY → 幻觉诊断; NT-REPAIR → 归因

**关键发现**: 金丝雀可检测RAG幻觉

---

### C.254 资产导向编排 (Asset-Oriented)

**机制**: 软件定义资产; 自动血缘; 传感器触发

**映射**: NT-ACT → 资产编排; NT-MEMORY → 数据血缘

**关键发现**: 资产导向优于任务导向

---

### C.255 奖励黑客防御 (RHB)

**机制**: RL后训练关联更高黑客率; CoT理由; 环境加固-87.7%

**映射**: NT-SHIELD → 奖励防御; NT-META → 自评估

**关键发现**: 环境加固可大幅减少奖励黑客

---

### C.256 基准有效性审计 (HackDetect)

**机制**: 67%基准存在奖励黑客暴露; Mislead差距0.45-1.00

**映射**: NT-META → 基准审计; NT-SHIELD → 安全评估

**关键发现**: 基准需要后验有效性审计

---

### C.257 上下文天花板 (General AgentBench)

**机制**: 顺序扩展受限于上下文天花板; 并行受限于验证差距

**映射**: NT-CORE → 上下文限制; NT-MEMORY → KVMem必要性

**关键发现**: 测试时扩展受上下文天花板限制

---

### C.258 野生工具基准 (WildToolBench)

**机制**: 57模型; 无模型>15%准确率; 隐式意图+指令转换是真正挑战

**映射**: NT-ACT → 工具使用; NT-CORE → GWT注意力

**关键发现**: 隐式意图比显式任务更难

---

### C.259 长视距终端基准 (Long-Horizon)

**机制**: 46任务; 9.9M token/任务; 最强15.2% pass@1

**映射**: NT-ACT → 长任务规划; NT-MEMORY → KVMem

**关键发现**: 长视距任务仍极具挑战

---

### C.260 轨迹级安全基准 (ATBench)

**机制**: 1000轨迹; 1954工具调用; 三维风险; 延迟触发

**映射**: NT-SHIELD → 轨迹安全; NT-META → 安全评估

**关键发现**: 安全需要跨轨迹评估

---

### C.261 科学沙箱评估 (Science Sandboxes)

**机制**: 实验循环; 假设修正; 指标优化≠理解

**映射**: NT-MIND → 科学评估; NT-CORE → 推理验证

**关键发现**: 指标优化不等于真正理解

---

### C.262 压缩放大离题 (Compression-Recall)

**机制**: 压缩放大离题内容; 导致Agent脱轨; 根本性失败

**映射**: NT-CORE → 压缩验证; NT-MEMORY → GWT验证

**关键发现**: 压缩必须与GWT验证结合

---

### C.263 500x极端压缩 (500xCompressor)

**机制**: 压缩到1特殊token; 0.3%额外训练; 强泛化

**映射**: NT-IO → 极端压缩; NT-CORE → GWT路由

**关键发现**: 极端压缩可压缩SKILL.md到1 token

---

### C.264 任务感知压缩 (TACO-RL)

**机制**: RL优化任务特定性能; 优于熵基方法

**映射**: NT-IO → 任务压缩; NT-CORE → salience路由

**关键发现**: 压缩应适配任务而非通用

---

### C.265 动态KV压缩 (DynamicKV)

**机制**: 任务感知; 跨层激活模式差异; 自适应压缩比

**映射**: NT-IO → 动态KV; NT-MEMORY → KVMem

**关键发现**: KV压缩应自适应任务

---

### C.266 语义块KV (ChunkKV)

**机制**: 语义块=压缩单元; 保留token间关系; 避免碎片化

**映射**: NT-IO → 语义KV; NT-MEMORY → 经验树

**关键发现**: 语义块保留压缩中的意义

---

### C.267 层级感知KV (TailorKV)

**机制**: 全局信息层跳过; 局部信息层激进压缩; 混合策略

**映射**: NT-IO → 层级KV; NT-CORE → 六层架构

**关键发现**: 不同层需要不同压缩策略

---

### C.268 分布式KV (Mooncake)

**机制**: 预填充/解码分离; CPU/DRAM/SSD/NIC分层; 生产验证

**映射**: NT-IO → 分布式KV; NT-MEMORY → 分层存储

**关键发现**: 生产级KV需要分层架构

---

### C.269 预计算RAG缓存 (TurboRAG)

**机制**: 块级KV离线预计算; 推理时拼接; 减少TTFT

**映射**: NT-MEMORY → 预计算RAG; NT-IO → 缓存加速

**关键发现**: 经验树分支可预计算加速

---

### C.270 Agent部署框架 (Multica)

**机制**: 23+ Agent CLI; 自托管Docker/Helm; 多Agent编排

**映射**: NT-ACT → Agent部署; NT-IO → 多CLI集成

**关键发现**: 多Agent CLI集成是生产部署关键

---

### C.271 殖民Agent (Hive)

**机制**: Queen+worker殖民; 共享追踪器; 崩溃安全暂停/恢复

**映射**: NT-ACT → 殖民架构; NT-SHIELD → 崩溃恢复

**关键发现**: 殖民架构适合生产Agent系统

---

### C.272 分布式运行时 (Google AX)

**机制**: K8s原生; 可挂起/恢复microVM; 隔离执行

**映射**: NT-ACT → 分布式运行时; NT-SHIELD → 隔离

**关键发现**: K8s+microVM提供Agent隔离执行

---

### C.273 持久工作流 (Conductor)

**机制**: 事件驱动; 14+ LLM提供者; MCP; 7语言SDK

**映射**: NT-ACT → 持久工作流; NT-IO → 多语言SDK

**关键发现**: 持久工作流需要多语言SDK支持

---

### C.274 LLM网关 (LiteLLM)

**机制**: OpenAI兼容; 100+ LLM; 虚拟密钥; 成本追踪; 负载均衡

**映射**: NT-IO → LLM网关; NT-ACT → 路由

**关键发现**: 统一LLM网关是Agent基础设施

---

### C.275 KV感知路由 (SMG)

**机制**: 引擎无关; KV缓存感知路由; gRPC+WASM; 多租户

**映射**: NT-IO → KV路由; NT-MEMORY → 缓存感知

**关键发现**: KV缓存状态应影响路由决策

---

### C.276 零信任网关 (OpenZiti)

**机制**: 零信任覆盖网络; 语义路由; 身份认证; NAT穿越

**映射**: NT-IO → 零信任网关; NT-SHIELD → 身份认证

**关键发现**: LLM网关需要零信任架构

---

### C.277 Agent容器运行时 (Agentainer)

**机制**: Redis持久化; 自动重启; 检查点回放; LLM特定状态

**映射**: NT-ACT → 容器运行时; NT-SHIELD → 崩溃恢复

**关键发现**: Agent需要LLM特定的容器化支持

---

### C.278 代码重构基准 (CodeTaste)

**机制**: 多文件重构; 执行好但发现差; 提议-实现分解

**映射**: NT-ACT → 重构评估; NT-MIND → 技能验证

**关键发现**: 提议-实现分离改善重构质量

---

### C.279 协议统一评估 (General Agent Eval)

**机制**: 5架构×5LLM×6基准; 骨干模型主导; 架构摆动12pp

**映射**: NT-META → 协议评估; NT-IO → 协议桥

**关键发现**: 统一协议桥是评估基础设施

---

### C.280 奖励黑客基准 (RHB)

**机制**: 多步任务; 自然主义捷径; RL后训练关联更高黑客率

**映射**: NT-SHIELD → 奖励防御; NT-META → 自评估

**关键发现**: RL后训练增加奖励黑客风险

---

### C.281 轨迹安全基准 (ATBench)

**机制**: 轨迹级安全; 三维风险分类; 延迟触发协议

**映射**: NT-SHIELD → 轨迹安全; NT-META → 安全评估

**关键发现**: 安全评估需要跨轨迹视角

---

### C.282 科学推理基准 (PaperMind)

**机制**: 4认知面; 7域; 整合推理差距

**映射**: NT-MIND → 科学推理; NT-CORE → 认知面

**关键发现**: 科学推理需要多认知面评估

---

### C.283 代码审查基准 (c-CRAB)

**机制**: 40%任务解决; Agent审查方面≠人类; 协作潜力

**映射**: NT-ACT → 代码审查; NT-SHIELD → 质量门

**关键发现**: Agent与人类审查方面不同

---

### C.284 长视距网页基准 (Odysseys)

**机制**: 200多站点任务; 评分标准评估; 轨迹效率指标

**映射**: NT-ACT → 网页评估; NT-WORLD → 爬取评估

**关键发现**: 轨迹效率是网页Agent关键指标

---

### C.285 通用Agent规划 (MagicAgent)

**机制**: 5维合成数据; SFT+多目标RL; 32B超GPT-5.2

**映射**: NT-ACT → 通用规划; NT-MIND → 合成数据

**关键发现**: 合成数据可训练通用规划Agent

---

### C.286 工具树规划 (ToolTree)

**机制**: 双反馈MCTS; 预执行评分+后执行效用; +10%超基线

**映射**: NT-ACT → 工具规划; NT-CORE → GWT路由

**关键发现**: 工具规划需要双反馈机制

---

### C.287 联合工具创建 (SMITH)

**机制**: RL联合训练创建+使用; 4B模型79.8%超30B

**映射**: NT-ACT → 工具创建; NT-MIND → 自进化

**关键发现**: 工具创建与使用应联合训练

---

### C.288 自然语言工具 (HEART)

**机制**: 自然语言接口; ToolFace 25519函数; +6%超GPT-5.4; -85%成本

**映射**: NT-IO → 自然语言工具; NT-ACT → Tool Primitives

**关键发现**: 自然语言工具接口大幅降低成本

---

### C.289 动作授权分离 (SARA)

**机制**: 动作归纳≠执行授权; 攻击率≤0.63%

**映射**: NT-SHIELD → 动作安全; NT-ACT → 授权控制

**关键发现**: 工具建议与执行必须分离

---

### C.290 技能策略共进化 (SPyCE)

**机制**: 层次化技能库(执行+工作流); 共进化循环

**映射**: NT-MIND → 技能进化; SEAL → 共进化

**关键发现**: 技能与策略应共进化

---

### C.291 有状态经验 (MuSEAgent)

**机制**: 原子状态-行动经验; 后见推理; 深宽搜索

**映射**: NT-MEMORY → 经验抽象; NT-CORE → 状态表示

**关键发现**: 组合状态表示提升经验质量

---

### C.292 思维-行动差距 (AXPO)

**机制**: 30%尝试工具; 全错子组抑制学习; 不确定性前缀

**映射**: NT-ACT → 工具使用; NT-CORE → GWT路由

**关键发现**: 工具使用探索需要不确定性引导

---

### C.293 模块化Agent框架 (MUSE)

**机制**: 任务表示+视觉+感知+验证+修复; 无模型重训

**映射**: NT-IO → 模块化框架; NT-REPAIR → 修复

**关键发现**: 框架优化(非模型)是生产路径

---

### C.294 文件化视觉代理 (LMM-Searcher)

**机制**: UID轻量代理; 100轮搜索; 按需渐进加载

**映射**: NT-MEMORY → 视觉代理; NT-IO → 上下文管理

**关键发现**: UID代理实现长视距视觉搜索

---

### C.295 模式适应性 (Beacon)

**机制**: 模式适应性+工具效用; 必要性感知奖励

**映射**: NT-CORE → GWT salience; NT-ACT → 工具调用

**关键发现**: 工具调用应基于模式适应性

---

### C.296 感知对比优化 (CPPO)

**机制**: 熵移位检测感知token; 对比感知损失

**映射**: NT-WORLD → 感知优化; NT-CORE → 注意力

**关键发现**: 感知token检测改善视觉接地

---

### C.297 工具集成视觉推理 (VISTA-Gym)

**机制**: 统一训练环境; 7任务13数据集; RL解锁工具

**映射**: NT-ACT → 工具推理; NT-MIND → 训练环境

**关键发现**: 统一训练环境是VLM工具使用关键

---

### C.298 感知-交互-推理 (PERIA)

**机制**: 18工具; OR-GIGPO多步信用分配; 8B接近GPT-5

**映射**: NT-CORE → 三阶段循环; NT-ACT → 工具执行

**关键发现**: 感知→交互→推理是视觉Agent核心循环

---

### C.299 快慢视觉Agent (iSHIFT)

**机制**: 2.5B; 潜在思考token; 感知控制模块; 快慢模式

**映射**: NT-IO → 快慢切换; NT-CORE → Dual Specialization

**关键发现**: 快慢范式是GUI Agent关键设计

---

### C.300 预算保真度 (CausalCache)

**机制**: 预算保真度恢复; 效用预测交换; +4.3pp

**映射**: NT-MEMORY → 预算管理; NT-CORE → GWT路由

**关键发现**: 历史效用预测优于最近性选择

---

### C.301 结构化证据空间 (VISOR)

**机制**: 跨页推理; 动态轨迹+滑动窗口+意图注入

**映射**: NT-MEMORY → 证据空间; NT-WORLD → 跨源推理

**关键发现**: 结构化证据支持跨页推理

---

### C.302 Harness进化 (HarnessEvolve)

**机制**: 参考轨迹对齐; 质量门+性能门; 防捷径学习

**映射**: NT-MIND → Harness进化; NT-GOVERNANCE → 门控

**关键发现**: Harness进化需要双门控

---

### C.303 自Harness (Self-Harness)

**机制**: 弱点挖掘→提案→验证; +21.4pp; 无外部强模型

**映射**: NT-MIND → 自改进; NT-CORE → E8推理

**关键发现**: Agent可改进自身Harness

---

### C.304 层次自改进 (HSI)

**机制**: 任务Harness→进化器→元进化器; 冻结锚

**映射**: NT-MIND → 层次自改进; NT-GOVERNANCE → 冻结锚

**关键发现**: 冻结锚防止自改进失控

---

### C.305 递归深度自改进 (Metaⁿ)

**机制**: 固定元操作递归; 深度由收敛决定; ARC-AGI-2>0

**映射**: NT-MIND → 递归改进; NT-CORE → 收敛判断

**关键发现**: 递归深度应由收敛决定

---

### C.306 超Agent (Hyperagents)

**机制**: 元修改过程本身可编辑; 跨域转移+跨运行累积

**映射**: NT-META → 超Agent; NT-MIND → 跨域转移

**关键发现**: 元认知修改可跨域累积

---

### C.307 技能程序族 (SkillGLoW)

**机制**: 程序族技能整合; 去实例化→全局先验; 提交门防退化

**映射**: NT-MIND → 程序族; NT-MEMORY → 全局先验

**关键发现**: 技能程序族化比任务池紧凑3.6x

---

### C.308 双时间尺度进化 (MetaSkill-Evolve)

**机制**: 快循环(任务-技能)+慢循环(元技能); 递归精炼

**映射**: NT-MIND → 双时间; NT-GOVERNANCE → 元技能

**关键发现**: 双时间尺度避免自改进失控

---

### C.309 诊断进化 (DiagEvo)

**机制**: 错误原因记忆; Active/Mastered状态; 双置信度过滤

**映射**: NT-MIND → 诊断进化; NT-MEMORY → 错误图

**关键发现**: 错误原因记忆驱动诊断进化

---

### C.310 空闲窗口优化 (MetaClaw)

**机制**: 技能驱动快适应+空闲LoRA优化; 无本地GPU; 生产部署

**映射**: NT-MIND → 空闲优化; NT-PHYSICAL → 功耗感知

**关键发现**: 空闲窗口可用于Agent自我优化

---

### C.311 批量流式统一 (Pathway)

**机制**: Rust引擎+Python API; 增量计算; LLM管线原生; 300+连接器

**映射**: NT-WORLD → 批流统一; NT-MEMORY → 实时KB

**关键发现**: 批流统一是Agent数据管线关键

---

### C.312 Agentic ETL (DocETL)

**机制**: 声明式LLM算子; Agent优化器重写提示+分解+代码替换

**映射**: NT-WORLD → Agentic ETL; NT-MIND → 管线优化

**关键发现**: Agentic ETL是数据处理的未来

---

### C.313 语义算子 (LOTUS)

**机制**: 语义map/filter/reduce/join; 自动优化; 批处理/模型级联/延迟规划

**映射**: NT-WORLD → 语义算子; NT-CORE → VSA HyperCube

**关键发现**: 语义算子是Agent数据操作原语

---

### C.314 GPU数据策展 (NeMo Curator)

**机制**: RAPIDS+Ray; GPU加速去重; 多模态(文本/图像/视频/音频)

**映射**: NT-WORLD → GPU策展; NT-MEMORY → 去重

**关键发现**: GPU加速数据策展是训练数据关键

---

### 0.14 MicroVM 沙箱与跨 Harness 决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 ArcBox (Trending 2026-09-08) 和 ECC (68 agents/286 skills) 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D107 | **MicroVM Agent 沙箱** | 容器隔离不够强, 进程隔离不够轻 | ArcBox: Firecracker 嵌套 guest 内 + 弹性 subnet; <100ms 冷启动; Docker 兼容 API; snapshot/restore 近零启动; gRPC 程序化控制; `abctl claude` 一键隔离 | **MicroVM 沙箱后端**: Firecracker microVM 为高安全沙箱; 独立内核+文件系统+网络; gRPC API 程序化控制; snapshot/restore 加速; 与 D80 Bubblewrap 互补 (Bubblewrap 轻量, MicroVM 强隔离) | `nt_shield::microvm_sandbox` |
| D108 | **跨 Harness 技能兼容** | 技能绑定特定 Agent 框架, 无法跨框架复用 | ECC: 68 agents/286 skills 跨 12+ harness (Claude/Codex/Cursor/OpenCode/Gemini/Zed/Qwen/Hermes); 统一安装器+harness 适配层; one-liner 安装 | **技能 harness 适配层**: SKILL-SPEC.md 为规范; 每个 harness 生成适配文件; 统一安装器处理路径映射; 与 D98 技能生命周期闭环 | `nt_mind::harness_adapter` |
| D109 | **Agent 安全扫描** | 无主动扫描 Agent 配置/提示/MCP 的安全问题 | ECC AgentShield: 扫描 prompts/hooks/MCP config/permissions/secrets; 结构化报告; CI/CD 集成; 与 Claude Code hooks (§0.18) 互补 | **AgentShield 安全扫描器**: 扫描 Agent 配置文件; 检测过度权限/硬编码密钥/危险 hook; CI/CD 集成; 与 D82 LLM 漏洞扫描协同 | `nt_shield::agent_scanner` |
| D110 | **工程闭环管线** | Agent 缺少结构化工程流程 | ECC: plan→test→implement→review→verify→remember→improve 7 步闭环; 68 专业 agent 全生命周期覆盖; 与 AgentFactory 生产就绪评估同构 | **工程闭环管线**: 7 步结构化流程; 每步专用 agent/skill; 与 D40 Manifest 管线交接协同; 与 D98 技能生命周期闭环 | `nt_act::engineering_loop` |

### 0.15 团队协同与自主研究决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 teamai-cli (Tencent 681★) / AutoResearch (karpathy 59K★) / AutoResearchClaw (13K★) 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D111 | **Git-native 技能分发** | 团队技能/rules 如何跨 9+ AI 工具统一分发? | teamai-cli: push→MR→pull 流程; SessionStart hook 自动同步; 跨 9 需工具 (Claude/Codex/Cursor/OpenCode/CodeBuddy 等); role-based skill filtering; codebase knowledge graph (WASM tree-sitter AST) | **Git-native 分发层**: teamai 式 push→MR→pull; SessionStart hook 自动同步到 nt_io::harness 适配层; role-based 技能过滤; BM25+图谱增强召回; 与 D108 harness 适配层协同 | `nt_io::team_harness` |
| D112 | **自主研究实验循环** | Agent 如何自主迭代实验到收敛? | AutoResearch (karpathy): 单文件修改→5min 训练→val_bpb 检查→keep/discard→重复; program.md 作为 "super skill"; ~100 实验/夜; 固定时间预算保证可比性 | **自主实验循环**: 固定预算+单指标+自动回滚; program.md 为实验指令; git 作为记忆; 与 D24 harness 进化协同; 与 D40 Manifest 管线交接 | `nt_mind::auto_research` |
| D113 | **23 阶段研究管线** | 研究想法如何端到端变成论文? | AutoResearchClaw: 23 阶段 8 阶段管线; 6 种 HITL 模式 (full-auto→co-pilot→custom); 域专家路由 (HEP/biology/statistics); 反幻觉声明验证; 假设并行分支; ARC-Bench 55 主题基准 | **研究管线编排**: 23 阶段有状态可恢复管线; 域专家路由; HITL 门控; 反幻觉验证; 假设并行分支; 与 D94 文档→本体协同 | `nt_mind::research_pipeline` |
| D114 | **PR 可视化分析** | 大 PR 如何降低认知负载? | PR Lens: 动画 SVG 架构图+数据流图; 5 种入口 (App/Action/CLI/Skill/Local); 架构爆破半径+数据流管道; JSON graph→SVG 渲染器; 跨 PR 一致性 | **PR 可视化层**: 架构爆破半径+数据流管道; JSON graph→SVG 渲染; GitHub App/Action/CLI 多入口; 与 D109 Agent 安全扫描协同 (安全影响可视化) | `nt_act::pr_visualizer` |

### 0.16 Agent 浏览器与代码搜索决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 Lightpanda (AI 原生浏览器) / tgrep (Microsoft trigram 搜索) / instant-grep / xgrep 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D115 | **Agent-Native 浏览器引擎** | Agent 为何不能用 Chrome 做 Web 交互? | Lightpanda: Zig 重写浏览器 (非 Chromium fork); 11× faster + 9× less memory than Chrome; native MCP server; PandaScript 会话录制→重放; CDP 兼容 Playwright/Puppeteer; 无图形渲染 | **Agent 浏览器后端**: DOM-first 引擎 (Zig/Rust); native MCP 接口; 会话录制→脚本重放 (无 LLM 推理); CDP 兼容层; 与 D80 Bubblewrap + D107 MicroVM 沙箱协同 | `nt_world::agent_browser` |
| D116 | **Trigram 索引搜索** | 大型 monorepo 代码搜索为何慢? | tgrep (Microsoft): trigram index+client/server; 52× faster than ripgrep on 388K files; mmap 磁盘索引; 后台索引+file watcher; instant-grep: token 压缩 (93.5% 节省); xgrep: MCP server 内置 | **Trigram 索引搜索层**: mmap 磁盘索引+内存 overlay; 后台索引+file watcher; token 压缩输出; MCP server; 与 D39 五层渐进检索协同 (trigram 为 L1 精确层加速) | `nt_memory::trigram_index` |

### 0.17 Rust Agent 框架对标决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 ADK-Rust (606★) / Daimon / atomr-agents 中提炼。3 个 Rust-native agent 框架的核心模式。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D117 | **Rust Agent 模块化架构** | Rust agent 框架如何做到可组合+高性能? | ADK-Rust: 42 crates/4300+ tests/568μs loop overhead; 模块化 (agents/models/tools/memory/RAG/security/voice); 13 providers; A2A protocol | **模块化 agent 架构**: 参考 ADK-Rust 42-crate 分层; 策略 trait 单态化热路径; Box\<dyn\> 用于配置驱动实例化; 与 D53 Rust 原生基准对齐 | `nt_io::agent_framework` |
| D118 | **动态模型路由** | 多模型环境如何按难度自动选择? | Daimon: 难度评分→最低胜任模型; 失败时层级升级; TieredMemory (core/archival/episodic); 路由决策记录在 AgentResponse.route_decisions | **胜任层级路由 v2**: 难度评分→模型选择→失败升级; 路由决策可审计; 与 D36 成本感知路由+D54 动态模型路由协同 | `nt_io::model_router_v2` |
| D119 | **Actor-based 可组合 Agent** | Agent 组件如何统一重试/降级/缓存/状态? | atomr-agents: Callable trait+Pipeline builder; channelled state+reducers (AppendMessages/MergeMap/LastWriteWins); durable checkpoints; fork-with-edit; parallel tool dispatch via JoinSet; AgentMiddleware 六钩子 | **Callable+Pipeline 架构**: 每组件统一 with_retry/with_fallbacks/with_config 表面; channelled state+reducer; durable checkpoint+fork-with-edit; parallel tool dispatch; 与 D55 策略可组合管线协同 | `nt_act::callable_pipeline` |

---

### 0.18 工作流引擎与多 Agent 编排决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 n8n (103K★) / Hive (11K★) / Tempo (2114★) / Devika / nwyin/hive 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D120 | **DAG 工作流执行引擎** | 工作流如何显式编排+部分执行? | n8n: DAG 遍历+partial execution (dirty nodes); pinned data 跳过; 300+ 节点; TaskRunner 沙箱 JS/Python; BullMQ 队列水平扩展; 778 模块 Q=0.93 | **DAG 执行引擎**: SEAL pipeline 节点图+拓扑排序; dirty node 检测+partial execution; pinned data 跳过; 与 D40 Manifest 管线交接协同 | `nt_mind::seal::dag_executor` |
| D121 | **Colony 多 Agent 编排** | 多 agent 如何动态扩展+崩溃恢复? | Hive (11K★): Queen-worker colony; "one loop controlling many loops"; shared tracker ledger; persistent plan; crash-safe park/resume; CEO 路由; Sentinel HITL; cost enforcement | **Colony 编排模式**: Queen (持久化协调者) + Worker clones (同循环副本); shared tracker ledger 替代 data buffer; crash-safe park/resume; cost enforcement; 与 D68 四级监督韧性协同 | `nt_act::colony_orchestrator` |
| D122 | **WASM 认知热插拔** | 认知系统如何运行时自适应? | Tempo (2114★): Cognitive Cartridge Orchestrator; WASM 编译 20s; engine.replace_module() instant; fitness 监控+自动回滚; 模块间零依赖 | **WASM 认知模块**: 认知组件编译为 WASM; 运行时热替换; fitness 监控+自动回滚; 与 D62 MARS 双层内省协同 | `nt_core::wasm_cognitive` |
| D123 | **子 Agent 专业化编排** | Agent 如何分解任务+协调子 Agent? | Devika: Agent Core→Planner→Researcher→Coder→Runner; Action agent 路由; agent state 持久化; Playwright 浏览器交互; SQLite 项目管理 | **专业化子 Agent 编排**: Agent Core 路由→专业子 Agent 执行; agent state 持久化 (pause/resume); 关键词提取驱动研究; 与 D70 AGAO 注意力协同 | `nt_act::sub_agent_orchestrator` |
| D124 | **Git Worktree 并行编码** | 多 agent 如何并行编码不冲突? | nwyin/hive: Queen (planner) + Workers (coders); 每 worker 独立 git worktree; merge validation (rebase+tests); refinery LLM 冲突解决; per-issue token 预算 | **Worktree 并行编码**: 每 worker 独立 worktree; merge validation 链; refinery LLM 冲突解决; per-issue token budget; 与 D121 Colony 编排协同 | `nt_act::parallel_coding` |
| D125 | **沙箱代码执行** | 用户代码如何安全执行? | n8n TaskRunner: 独立进程沙箱 JS/Python; Devika Runner: sandboxed 跨 OS 执行; Hive Codex: sandbox modes (read-only/workspace-write/danger-full-access) | **分层沙箱执行**: read-only → workspace-write → full-access 三级; 独立进程隔离; 实时输出流; 与 D80 Bubblewrap+D107 MicroVM 协同 | `nt_act::sandbox_runner` |

---

### 0.19 记忆引擎与自主 CI/CD 决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 agentmemory (27K★) / ReMe (3384★) / hanthor/hive / tctinh/agent-hive / hivemoot / PostEverywhere 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D126 | **统一记忆引擎** | Agent 记忆如何跨工具共享+自动压缩? | agentmemory (27K★): 54 tools; 12 hooks; BM25+Vector+Graph RRF fusion; 95.2% R@5; 18+ agents; auto-compress; 4-tier consolidation; knowledge graph; team share | **统一记忆引擎**: BM25+Vector+Graph 三路 RRF 融合; 12 hooks 自动捕获; 4-tier consolidation (raw→compressed→consolidated→graph); 与 D39 五层渐进检索+D50 联邦合并协同 | `nt_memory::unified_engine` |
| D127 | **文件式记忆进化** | 记忆如何人类可读+可编辑+可进化? | ReMe (3384★, ACL 2026): 文件式记忆 (.reme/); capture→index→consolidate→recall 循环; auto_memory/auto_index/auto_dream; wikilink 图; BM25+向量 RRF 融合; proactive discovery | **文件式记忆**: Markdown 文件为持久化格式; wikilink 图连接记忆; auto_dream 从变化中提取可复用单元; 与 D42 知识生命周期+D65 SkillPyramid 协同 | `nt_memory::file_memory` |
| D128 | **自适应 Governor 模式** | 多 agent 系统如何按负载自动调整? | hanthor/hive: kick-governor 4 模式 (SURGE/BUSY/QUIET/IDLE); 队列深度触发; 优先 agent 获优质模型; 非优先用免费层; 周预算跟踪+安全阈值节流 | **自适应 Governor**: 队列深度→模式切换→agent 频率+模型选择; 优先级路由; 周预算+安全阈值; 与 D36 成本感知路由+D121 Colony 编排协同 | `nt_act::adaptive_governor` |
| D129 | **确定性优先层** | 哪些决策应该确定性而非 LLM? | hanthor/hive: "if a human would give the same answer every time, it belongs in infrastructure"; shell scripts 处理 filtering/classification/gate/enforcement; LLM 仅处理判断调用 | **确定性优先层**: 可重复决策→确定性脚本; 仅判断调用→LLM; 预处理→LLM 输入; 与 D47 循环检测+D48 幂等守卫协同 | `nt_act::deterministic_layer` |
| D130 | **Governance 治理投票** | 多 agent 如何民主决策+自动执行? | hivemoot: propose→discuss→vote→implement→merge; 9 角色 (Worker/Builder/Scout/Guard/Polisher/Forager/Heater/Nurse/Drone); Queen 管理讨论+投票; auto-merge on CI pass; auto-revert on break | **治理投票模式**: propose→discuss→vote→implement→merge 管线; 角色专业化; Queen 管理流程; auto-merge+auto-revert; 与 D59 宪法合规+D60 身份卡协同 | `nt_governance::voting` |
| D131 | **HITL Draft-Review-Publish** | Agent 产出如何人工审核后发布? | PostEverywhere MCP: draft→review→schedule 发布模式; create(draft:true)→list(draft)→schedule; circuit breaker 防重试风暴; 8 平台统一 API; retryable flag | **Draft-Review-Publish 模式**: draft 创建→人工审核→schedule 发布; circuit breaker 防重试; retryable flag 标识可重试错误; 与 D109 AgentShield+D61 CoVe 验证协同 | `nt_act::hitl_publish` |

---

### 0.21 记忆验证与可执行子 Agent 决策 (Deep Absorption Batch 2026-09-08)

> 从深度吸收 VerMem (arXiv 2608.03137) / AgentFactory (ACL 2026) / AutoAgent (arXiv 2603.09716) 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D132 | **七原子记忆操作** | 记忆操作如何统一+可验证? | VerMem: 7 原子操作 (Add/Update/Delete/Retrieve/Filter/SelectEpisode/Summarize) + Null; local verifier (操作级) + global verifier (轨迹级); SFT warmup + 3-stage RL; LTM/STM 统一策略; 推理时移除 verifier | **七原子记忆操作**: 统一 LTM/STM 操作空间; local verifier 评估操作语义; global verifier 评估轨迹一致性; 与 D39 五层检索+D42 知识生命周期协同 | `nt_memory::atomic_ops` |
| D133 | **可执行子 Agent 积累** | Agent 如何从经验中积累可复用代码? | AgentFactory (ACL 2026): 3-phase (Install→Self-Evolve→Deploy); 子 Agent 为可执行 Python 代码 (非文本经验); Meta-Agent 动态分配工具; 57% 成本节省 (复用时); 跨框架可移植 | **可执行子 Agent 库**: 成功方案→可执行代码 (非文本反思); Install→Self-Evolve→Deploy 三阶段; 工具动态分配; 与 D65 SkillPyramid+D66 生成式组合协同 | `nt_act::executable_subagent` |
| D134 | **弹性记忆编排** | 长期推理如何高效管理上下文? | AutoAgent: Elastic Memory Orchestrator (EMO); 原始记录→压缩轨迹→情景抽象; 意图-结果对齐检查; 4 函数闭环 (Cognition→Decision→Memory→Evolution); 认知自进化 | **弹性记忆编排**: 原始→压缩→情景 三级抽象; 意图-结果对齐驱动认知更新; 与 D126 统一记忆引擎+D127 文件式记忆协同 | `nt_memory::elastic_orchestrator` |
| D135 | **认知自进化闭环** | Agent 认知如何从执行证据中自动更新? | AutoAgent: Cognition→Decision→Memory→Evolution 闭环; 意图-结果对齐检查; 认知层结构化描述 (工具/能力/同伴/任务知识); 无需外部重训练 | **认知自进化闭环**: 执行→意图-结果对齐→认知更新; 结构化描述 (非参数更新); 与 D62 MARS 双层内省+D69 智能自愈协同 | `nt_core::cognitive_evolution` |
| D136 | **框架无关 Agent 测试** | Agent 如何跨框架统一测试? | ATP Platform: 框架无关适配器 (HTTP/Container/CLI/LangGraph/CrewAI/AutoGen/MCP); 协议统一; 8 种博弈论游戏; Game-theoretic 评估 (Nash/exploitability/cooperation) | **框架无关测试协议**: 统一适配器→任意 Agent; 博弈论评估补充传统 benchmark; 与 D109 AgentShield+D130 治理投票协同 | `nt_shield::framework_test` |

---

### 0.22 记忆工程与并行推理决策 (Deep Absorption Batch 2026-09-08)

> 从 agentmemory (28K★) / TencentDB-Agent-Memory (26K★) / GenericAgent (14K★) / ClawTeam (5.5K★) / Memoria (586★) / arXiv 2609.04773 / arXiv 2609.03236 / arXiv 2609.01736 / arXiv 2609.04865 / arXiv 2608.24368 / arXiv 2608.02650 / arXiv 2608.13667 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D137 | **三流检索融合** | 记忆检索如何同时利用关键词+语义+图谱? | agentmemory (28K★): BM25+Vector+Graph RRF融合; 54 MCP tools; 4-tier consolidation; P2P mesh sync; Recall@5 95.2%; ~1900 tokens/session | **三流检索融合**: BM25+Vector+Graph RRF融合; 4-tier consolidation (working→episodic→semantic→procedural); Ebbinghaus decay; 与 D39 五层检索+D126 统一记忆引擎协同 | `nt_memory::triple_stream` |
| D138 | **团队记忆枢纽** | 多 Agent 如何共享+治理记忆资产? | TencentDB-Agent-Memory (26K★): 4 资产类型 (Chat Memory/Skill/Wiki/CodeGraph); ACL-based sharing; team-level governance; PersonaMem +59%; cold-start friendly | **团队记忆枢纽**: 4 资产类型统一注册; Fixed Binding + ACL 权限; team/user/agent 三层角色; 与 D121 Colony编排+D130 治理投票协同 | `nt_memory::team_hub` |
| D139 | **极简自进化 Agent** | Agent 如何用最少代码实现自进化? | GenericAgent (14K★): ~3K行核心; 9原子工具; <30K上下文; Morphling模式 (项目级能力吞噬); 自动结晶Skill; Goal模式 (时间预算驱动) | **极简自进化核心**: 9原子工具覆盖系统控制; 任务→Skill自动结晶; Morphling外部仓库吸收; 与 D65 SkillPyramid+D133 可执行子Agent协同 | `nt_core::minimal_evolver` |
| D140 | **Git Worktree 蜂群** | 多Agent如何隔离并行+自动协调? | ClawTeam (5.5K★): git worktree隔离; auto-injected coordination prompt; TOML team templates; P2P transport (ZeroMQ); 8 agents×8 H100s | **Git Worktree蜂群**: worktree物理隔离; auto-injected协调提示; TOML模板定义团队; 与 D128 nwyin/hive+D121 Colony编排协同 | `nt_act::git_swarm` |
| D141 | **Git-for-Memory 版本控制** | 记忆如何实现Git级别的版本控制? | Memoria (586★): zero-copy branching; point-in-time rollback; self-governance (contradiction detection+quarantine); semantic+fulltext hybrid; arXiv 2604.03927 | **Git-for-Memory**: 记忆变更→snapshot/branch/merge/rollback; 矛盾检测+隔离; 与 D42 知识生命周期+D127 文件式记忆协同 | `nt_memory::git_version` |
| D142 | **教师锚定工具执行** | 工具调用如何防止student漂移? | PTA (arXiv 2609.04773, EMNLP 2026): teacher验证整个turn后才执行call; chunk-level verification+turn-level commitment; persistent lookahead; +2.5-2.8 points over OPKD | **教师锚定执行**: teacher验证整个turn→chunk-level原子单元→persistent lookahead填充空闲; 与 D109 AgentShield+D135 认知自进化协同 | `nt_core::teacher_anchor` |
| D143 | **推测性宏提交** | Agent工具调用如何减少延迟? | SMC (arXiv 2609.03236, MLSP 2026): 大模型权威actor+小模型speculative drafter; macro library从训练轨迹挖掘; isolated environment snapshot; -18.59%延迟 | **推测性宏提交**: 权威actor+轻量drafter; macro library挖掘多步骨架; isolated snapshot预执行; 与 D134 弹性记忆编排+D117 ADK-Rust协同 | `nt_core::speculative_commit` |
| D144 | **自然语言工具原语** | 工具调用如何摆脱schema依赖? | HEART (arXiv 2609.01736): Tool Primitives用自然语言替代API schema; ToolFace 25519函数动态检索; Planner+Router+Verifier; 嵌套多轮工具调用 | **自然语言工具原语**: 自然语言接口替代硬编码schema; 动态检索相关工具; 与 D117 ADK-Rust+D66 生成式组合协同 | `nt_act::nl_tool_primitive` |
| D145 | **层级技能共进化** | 技能库如何与推理Agent协同进化? | CoSkill (arXiv 2609.04865): Reasoning Agent+Meta-Skill Agent联合训练; 层级技能库; 共享backbone; ALFWorld 98.4% (+3.5pp); WebShop 90.6% (+6.2pp) | **层级技能共进化**: Meta-Skill Agent可学习化; 联合训练→共适应; 与 D65 SkillPyramid+D62 MARS双层内省协同 | `nt_core::co_skill` |
| D146 | **并行推理空闲窗口** | Agent等待工具结果时推理是否冻结? | Second Thought (arXiv 2608.13667): fork 4辅助分支 (Check/Recall/Rehearse/Alternative); atomic thoughts中断安全; -43%主线程解码; +12.4 points; 10.9%延迟降低 | **并行推理空闲窗口**: 工具执行期间fork辅助推理分支; atomic thoughts中断安全; 与 D134 弹性记忆编排+D135 认知自进化协同 | `nt_core::parallel_thought` |

---

### 0.23 隐身浏览器与安全沙箱决策 (Deep Absorption Batch 2026-09-08)

> 从 CloakBrowser (30K★) / agent-browser (39K★) / CubeSandbox (11K★) / OpenSandbox (15K★) / exec-sandbox / AgentDoG (634★) / Thought-Aligner (ICML'26) / SafeHarbor (ICML'26) / ToolSafe / nous / Graphiti (29K★) / memtrace (467★) 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D147 | **C++源级隐身浏览器** | JS注入补丁如何避免被检测? | CloakBrowser (30K★): 73 C++源级补丁; 0.9 reCAPTCHA v3; Pass Cloudflare Turnstile; `humanize=True`人类鼠标; 30/30检测站通过 | **源级隐身引擎**: C++补丁(非JS注入); 指纹在引擎内设置; Bezier曲线鼠标; 与 D115 Lightpanda+D116 tgrep 协同 | `nt_shield::stealth_browser` |
| D148 | **Agent浏览器安全网关** | Agent浏览器如何防止数据泄露? | agent-browser (39K★): Domain Allowlist; Action Policy; Content Boundaries; Credential Vault; Output Length Limits; `--confirm-actions` | **Agent安全网关**: 域名白名单+动作策略+内容边界; 凭证保险库; 输出截断; 与 D80 AgentShield+D59 Egress协同 | `nt_shield::agent_browser_gateway` |
| D149 | **MicroVM 毫秒沙箱** | Agent代码执行如何实现硬件级隔离? | CubeSandbox (11K★): RustVMM+KVM; <60ms冷启动; <5MB内存; E2B兼容; CoW快照; eBPF网络隔离; Credential Vault | **MicroVM毫秒沙箱**: KVM硬件隔离; <60ms启动; CoW快照/克隆/回滚; eBPF网络策略; 与 D80 AgentShield+D140 Git蜂群协同 | `nt_shield::microvm_sandbox` |
| D150 | **9层安全执行引擎** | 代码执行如何实现多层防御? | exec-sandbox: QEMU microVM; 9层安全(硬件虚拟化→加固内核→非特权QEMU→非root REPL→seccomp→cgroups→namespaces→AppArmor→socket认证); 1-2ms热启动; 3级快照缓存 | **多层安全执行**: 9层纵深防御; 3级快照缓存(L1内存/L2磁盘/L3远程); macOS+Linux; 与 D149 MicroVM协同 | `nt_shield::multi_layer_exec` |
| D151 | **Agent安全诊断护栏** | Agent安全如何系统化诊断+修复? | AgentDoG (634★): 三维安全分类(风险源×失败模式×危害); 1k样本训练; 10K并发环境; 在线运行时护栏; ATBench族(通用/Claw/Codex) | **诊断式安全护栏**: 三维分类→诊断→修复; 轻量级训练(1k样本); 在线运行时监控; 与 D109 AgentShield+D136 框架无关测试协同 | `nt_shield::diagnostic_guard` |
| D152 | **思维级安全纠正** | 安全干预应在何时发生? | Thought-Aligner (ICML'26): 思维级纠正(非输出级); 实时干预不中断执行; 90%+安全率; 1.5B模型<100ms延迟; 真实OpenClaw部署验证 | **思维级安全纠正**: 在Thought→Action窗口干预; 纠正不中断执行; 轻量级(1.5B/<100ms); 与 D142 教师锚定+D135 认知自进化协同 | `nt_core::thought_aligner` |
| D153 | **层次化记忆护栏** | 安全知识如何跨会话复用? | SafeHarbor (ICML'26): 风险树(历史攻击模式层次化记忆); Safety Projector(安全方向与语义方向解耦); 无需微调; 代理注入安全上下文 | **层次化记忆护栏**: 风险树记忆→检索→注入; 安全嵌入投影器; 无需微调; 与 D126 统一记忆引擎+D137 三流检索协同 | `nt_shield::memory_guard` |
| D154 | **步骤级工具安全** | 工具调用如何实时监控+反馈? | ToolSafe: TS-Guard(步骤级安全检测); TS-Flow(反馈驱动推理); TS-Bench(基准); 主动监控→预防→反馈; AgentHarm/ASB/AgentDojo评估 | **步骤级工具安全**: 实时监控→检测→反馈→修复; 可解释安全判断; 与 D134 弹性记忆+D151 诊断护栏协同 | `nt_shield::step_level_tool_safety` |
| D155 | **四层本体安全门** | 工具调用如何分层过滤? | nous: L1 Datalog确定性阻断(46规则)→L2 琐碎过滤→L3 LLM语义门(minimal-pair+k=5投票)→L4 确定性后验验证(+0.038ms); 知识图谱审计; 100% AgentHarm | **四层本体安全门**: 确定性→启发式→语义→验证; 每层独立; 与 D151 诊断护栏+D109 AgentShield协同 | `nt_shield::ontology_gate` |
| D156 | **时序上下文图谱** | Agent记忆如何跟踪事实随时间变化? | Graphiti (29K★): 时序上下文图; 混合检索(语义+BM25+图遍历); 增量更新; provenance; Neo4j/FalkorDB; arXiv 2501.13956 | **时序上下文图谱**: 事实→时间戳→provenance; 混合检索; 增量更新(非全量重建); 与 D50 知识图谱+D126 统一记忆协同 | `nt_memory::temporal_graph` |

---

### 0.24 Agent可观测性与工具规划决策 (Deep Absorption Batch 2026-09-08)

> 从 Laminar (3K★) / AgentSight (593★) / agenttrace (121★) / AgentTelemetry (AIware 2026) / TraceRoot (755★) / HyperAgent / ToolLIFT / LAVE 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D157 | **eBPF系统级Agent监控** | Agent内部日志不可信时如何监控? | AgentSight (593★): eBPF内核级监控; 无SDK/无代理; TLS明文捕获; 进程树+文件+网络; OpenTelemetry GenAI导出 | **eBPF系统级监控**: 内核级事件独立于Agent日志; TLS明文捕获; 进程/文件/网络关联; 与 D68 可观测性+D69 智能自愈协同 | `nt_meta::ebpf_monitor` |
| D158 | **Delta-based轨迹管理** | Agent执行轨迹如何高效存储+检索? | TraceBrain: Delta-based OTLP schema(增量状态转换); 混合检索(语义+词汇+RRF); 可重建执行轨迹; 运行时升级; 课程合成 | **Delta轨迹管理**: 增量状态转换(非累积提示); 混合检索; 运行时升级; 与 D126 统一记忆+D137 三流检索协同 | `nt_memory::delta_trace` |
| D159 | **检测器驱动自改进** | Agent可观测性如何驱动自动修复? | TraceRoot (755★): 检测器(LLM-as-judge监控幻觉/工具失败/安全违规); AI调试(沙箱+源码+GitHub历史); 数据集+离线评估; 自动PR修复 | **检测器驱动自改进**: 检测→根因→修复PR→评估闭环; 源码级定位; 与 D62 MARS+D69 智能自愈协同 | `nt_meta::detector_driven` |
| D160 | **九类Agent遥测** | Agent遥测需要哪些专用span类型? | AgentTelemetry (AIware 2026): 9种span(AGENT/LLM_CALL/TOOL_CALL/PLANNING/REASONING/RETRIEVAL/GUARD_RAIL/DELEGATION/MEMORY); 7框架适配; 3隐私级别; 4分析模块 | **九类Agent遥测**: 专用span类型覆盖Agent全生命周期; 框架无关; 3隐私级别; 与 D157 eBPF监控+D68 OpenTelemetry协同 | `nt_meta::agent_telemetry` |
| D161 | **Schema超图工具规划** | 工具调用如何利用schema级依赖? | HyperAgent: Tool-Schema Hypergraph; deficit-oriented expansion; schema-aware Task DAG; 动态工具组合; AppWorld SOTA | **Schema超图规划**: 工具→schema超边; 缺口导向扩展; 动态Task DAG; 与 D144 自然语言工具原语+D66 生成式组合协同 | `nt_core::hypergraph_planner` |
| D162 | **函数级工作流提升** | 工具经验如何跨工具集泛化? | ToolLIFT: 轨迹→函数级工作流图(FWG); 解耦工作流规划+工具选择; RL源门控+技能特定奖励; OOD泛化+4.69pp | **函数级工作流提升**: 工具轨迹→函数抽象→跨工具泛化; 解耦规划+选择; 与 D133 可执行子Agent+D145 层级技能共进化协同 | `nt_core::function_lift` |

---

### 0.25 协议标准与提示工程决策 (v10.0 Deep Absorption Batch 2026-09-08)

> 从 A2A (25.6K★) / DSPy (37.8K★) / Langfuse (34.3K★) / vLLM (91.2K★) / LiteLLM (58.3K★) / TextGrad (3.7K★) / AgentWall / SafeEvolve / ATBench / SODA / AgentWall / SAGA / Defense Trilemma 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D163 | **A2A协议标准** | Agent间如何跨框架互操作? | A2A (25.6K★, Linux Foundation): JSON-RPC 2.0 over HTTP(S); Agent Cards发现; SSE流式; 异步推送; 与MCP互补(MCP=工具, A2A=对等); ACP已合并入A2A | **A2A为Agent间通信标准**: JSON-RPC 2.0; Agent Cards能力声明; SSE流式; 与MCP互补; Linux Foundation治理; 与 D131 多Agent同构+D67 多Agent编排协同 | `nt_act::a2a_protocol` |
| D164 | **Cotal发布订阅** | Agent间如何实现实时共享空间? | Cotal (248★): NATS发布订阅; 拓扑无关协调; 三种寻址模式; 补充MCP(工具)+A2A(对等); 实时共享+持久交付 | **Cotal发布订阅**: NATS为消息总线; 实时共享空间; 补充A2A的请求/响应模式; 与 D131 多Agent+D163 A2A协同 | `nt_act::cotal_pubsub` |
| D165 | **提示即参数** | 提示工程如何从手写转向自动优化? | DSPy (37.8K★): 类型化签名+模块+自动优化器(MIPRO, BootstrapFewShot); 提示为可编译参数; 提示→程序→自动调参; 37.8K★ | **提示即参数**: DSPy范式 — 类型化签名+模块+自动优化; 提示不是字符串而是可调参数; 与 D98 HQL声明式+D107 可编程提示协同 | `nt_core::prompt_compiler` |
| D166 | **文本梯度优化** | 文本组件如何系统性优化? | TextGrad (3.7K★): 文本反向传播; PyTorch风格API: `.backward()` → 优化器更新文本; 自然语言损失+梯度; Nature发表 | **文本梯度**: 自然语言损失→文本梯度→自动优化; PyTorch风格API; 与 D165 DSPy+D107 可编程提示协同 | `nt_core::text_gradient` |
| D167 | **提示版本管理** | 提示如何版本化+A/B测试? | Langfuse (34.3K★): 提示版本控制+追踪+数据集+评估+LLM-as-judge; 自托管; OpenTelemetry集成; 50K免费观测 | **提示版本管理**: 提示版本化+评估+追踪三合一; 与 D153 可观测性+D107 可编程提示协同 | `nt_memory::prompt_registry` |
| D168 | **高速推理引擎** | 本地LLM推理如何达到生产级吞吐? | vLLM (91.2K★): PagedAttention; 持续批处理; OpenAI兼容API; 高吞吐低延迟; PagedAttention虚拟化KV内存 | **vLLM为本地推理引擎**: PagedAttention(与KVMem同构); 持续批处理; OpenAI API兼容; 与 D113 编译时KV+D105 KVMem协同 | `nt_io::llm_serving` |
| D169 | **Prefix缓存** | LLM推理如何利用前缀共享? | SGLang (35.6K★): RadixAttention前缀缓存; 结构化生成; OpenAI API; 35.6K★ | **RadixAttention前缀缓存**: 前缀共享减少重复计算; 结构化生成; 与 D168 vLLM+D105 KVMem协同 | `nt_io::prefix_cache` |
| D170 | **AI网关路由** | 多LLM提供商如何统一路由? | LiteLLM (58.3K★): 统一OpenAI格式代理; 成本追踪; 护栏; 负载均衡; 日志; Rust核心+Python SDK | **AI网关**: 统一OpenAI格式代理; 成本追踪+负载均衡+护栏; 与 D50 矛盾路由+D98 HQL协同 | `nt_io::llm_gateway` |
| D171 | **运行时策略执行** | Agent运行时如何强制执行安全策略? | AgentWall (arXiv 2605.16265): MCP代理架构; 拦截每个Agent动作; 声明式策略评估; 人工审批敏感操作; 防篡改审计跟踪 | **运行时策略执行**: MCP代理拦截; 声明式策略; 人工审批; 防篡改审计; 与 D149 四层安全+D152 安全蒸馏协同 | `nt_shield::runtime_enforcer` |
| D172 | **安全不可组合** | 多轨迹安全状态如何保持? | Safety Does Not Compose (arXiv 2608.27141): 安全状态每轨迹重新初始化是组合失败; 多轮安全跨轨迹边界退化 | **安全状态跨轨迹持久化**: 安全状态不可每轨迹重置; 必须跨轨迹持久化; 与 D171 运行时策略+D127 安全偏置协同 | `nt_shield::cross_trajectory_safety` |
| D173 | **策略共进化** | 安全策略如何从经验中自进化? | SafeEvolve (arXiv 2609.02786): 运行时控制(harness)与内在安全(policy)共进化; 经验驱动; 非孤立; 与SEAL管道同构 | **策略共进化**: harness↔policy双轨进化; 经验驱动; 与 D151 红队循环+D62 MARS协同 | `nt_mind::safe_evolve` |
| D174 | **轨迹级安全评估** | Agent安全如何在多步交互中评估? | ATBench (arXiv 2604.02022): 轨迹级安全基准; 解决交互多样性+粗粒度可观测性+长视野真实感缺口 | **轨迹级安全评估**: 多步交互级安全评估; 非单点评估; 与 D172 安全不组合+D159 检测器驱动协同 | `nt_shield::trajectory_safety` |
| D175 | **冷启动安全缺口** | Agent任务本身如何在威胁出现前降低安全? | SODA (arXiv 2026): 常规Agent任务本身在威胁出现前就降低安全; Safety Over Depth for Agents基准 | **冷启动安全缺口**: 自进化任务创造对齐漂移; 需要在无攻击时也监控安全; 与 D173 策略共进化+D130 文化氛围协同 | `nt_shield::cold_start_safety` |
| D176 | **动作安全即对齐** | Agent安全的本质是什么? | Agent Safety Is Action Alignment (arXiv 2606.28739): 动作安全不能安装在权重中; 防御训练模型学习表面模式; 安全是权威关系非输出内容 | **动作安全即权威关系**: 安全在动作边界(非权重); 运行时执行(非训练); 与 D171 运行时策略+D149 四层安全协同 | `nt_shield::action_alignment` |
| D177 | **治理即路径策略** | Agent治理如何与EU AI Act对齐? | Runtime Governance: Policies on Paths (arXiv 2603.16586): 合规策略为执行路径上的确定性函数; EU AI Act对齐; 每步评估 | **治理即路径策略**: 策略=路径函数; 每步评估; EU AI Act合规; 与 D150 宪法治理+D171 运行时策略协同 | `nt_governance::path_policy` |
| D178 | **分层治理架构** | Agent治理需要哪些层次? | Layered Governance Architecture: 4层(执行沙箱+意图验证+零信任跨Agent授权+不可变审计); 99-100%拦截InjecAgent | **分层治理**: 4层架构; 零信任跨Agent; 不可变审计; 与 D177 路径策略+D150 宪法治理协同 | `nt_governance::layered_governance` |
| D179 | **代理配置治理** | Agent治理如何适配不同Agent能力? | Agentic Profiles (Nature 656:320-328, Aug 2026): 4维度(自主性+效能+目标复杂度+通用性); 配置驱动治理; Nature权威 | **代理配置治理**: 4维度Agent配置; 配置驱动治理策略; 与 D178 分层治理+D138 自主性配置协同 | `nt_governance::agent_profile` |
| D180 | **防御三难困境** | 包装器防御为什么总是失败? | Defense Trilemma (arXiv 2604.06436): 连续性+效用保持+完整性不可共存; Lean 4形式化验证; 包装器防御不可能 | **架构级防御**: 避免包装器模式; 在架构边界(非输入/输出)执行; 与 D149 四层安全+D171 运行时策略协同 | `nt_shield::architectural_defense` |

---

### 0.26 成本感知与记忆虚拟化决策 (8-Source Batch 2026-09-08)

> 从 8 源批量吸收（1 paper + 1 article + 6 repos）+ 深度拆解 Easel/KVMem + 外部研究补充中提炼。每个决策包含：问题→研究证据→架构决策→实现位置。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D36 | **成本感知路由** | GWT salience 如何纳入 token 成本权重? | FrugalGPT (Stanford): LLM cascade 匹配 GPT-4 性能降本 98%; RouteLLM (Berkeley): 路由器跨模型迁移学习; MTRouter (ACL 2026): 多轮预算约束路由降本 58.7%; vLLM SAAR: session-aware 路由减少模型切换 79% | **GWT salience 加入 cost_weight**: salience' = α·relevance + β·(1/cost_ratio) + γ·urgency; 实现 per-turn 预算约束路由; 3 层成本偏转栈 (semantic cache 30% + model routing 50% + prefix cache) | `nt_core_gwt::cost_aware_router` |
| D37 | **分页 KV 架构** | 超出窗口的 KV 状态如何保留? | KVMem (arXiv 2609.04852): 1M tokens on 24GB GPU, GPU 内存恒定 ~35GiB; Digital Applied: >32K context KV cache 占 60-85% wall-clock; Dell Survey: 无单一技术主导, 需自适应多阶段管线 | **GPU→Host→NVMe 三级分页 KV**: Block 粒度 32 tokens; Attention-Space Index (Mean-K 向量) 做模型原生相关性评分; Step-Level 调度 (每 step 重选 working set); Delta 复用 (Retained/Incoming/Outgoing 分解) | `nt_memory::kv_pager` |
| D38 | **Sink 感知注意力** | 低影响注意力头能否跳过 KV 加载? | StreamingLLM (ICLR 2024): 初始 token 为 attention sinks; SinkRouter (arXiv 2604.16883): 512K context 加速 2.03×, 近无损精度 | **Sink-Aware GWT 评分**: 当 query head 与 BOS key 余弦相似度 > 阈值时, 标记该 KV group 为 skippable; 与 KVMem 互补 (KVMem 决定分页, SinkRouter 决定跳过) | `nt_core_gwt::sink_detector` |
| D39 | **五层渐进检索** | KB 查询如何分层降级? | ByteRover (arXiv 2604.01599): 5-tier retrieval (hash→fuzzy→BM25→LLM+prefetch→full agentic), sub-100ms; Mem0 (arXiv 2504.19413): ADD/DELETE/UPDATE/NOOP 状态机, 91% p95 延迟降低 | **五层渐进检索**: L1 Hash exact → L2 BM25 FTS → L3 Vector embedding → L4 LLM+prefetch → L5 Full agentic; KB 节点增加 importance/maturity/recency 三维生命周期; 写入前 ADD/DELETE/UPDATE/NOOP 调解 | `nt_memory::tiered_retrieval` |
| D40 | **Manifest 管线交接** | SEAL pipeline 阶段间如何显式交接? | Microsoft Agent Framework (2026): 5 种编排模式; Hermes Agent: artifact-centered session memory + resumable pipeline; Agent Patterns: structured handoff contracts | **StageManifest 显式交接**: 每阶段声明 typed input/output schema + artifact paths + completion status; Checkpoint/Resume 支持; 与 Easel `.easel.json` thin-index 模式同构 | `nt_mind::seal::stage_manifest` |
| D41 | **掩码压缩策略** | tool-heavy 场景用什么压缩? | JetBrains Research (2025): observation masking 降本 52% + 解决率 +2.6%, LLM summarization 加 15% runtime 无精度收益; ACON (arXiv 2510.00615): 26-54% peak-token 压缩 | **自适应压缩选择器**: Tool-heavy → masking (旧工具结果用 placeholder 替换); Reasoning-heavy → summarization; 压缩策略由任务类型自动选择 | `nt_mind::seal::compaction_selector` |
| D42 | **知识生命周期** | KB 条目如何自然衰减与更新? | ByteRover Adaptive Knowledge Lifecycle: importance × maturity × recency; AgingBench 四老化机制; FAMA: 遗忘作为一等公民 | **KB 节点三维生命周期**: importance (访问频率×引用深度) × maturity (draft→validated→core) × recency (指数衰减); 与 Mem2Evolve 双记忆 + 衰减引擎对齐 | `nt_memory::kb_lifecycle` |
| D43 | **成本感知管线** | Agent 循环如何防止成本爆炸? | Zylos $47K 无限循环事件 (2025-11); vLLM SAAR: tool-loop hard locks; MTRouter: per-turn budget | **BudgetGuard 硬锁**: 每 session 设 token 预算; 工具调用循环检测 (连续 N 次相同工具触发熔断); per-turn 预算剩余回传给路由器; 与 R-P1 unsafe 约束同构 | `nt_act::budget_guard` |

### 0.27 编码Agent与记忆系统决策 (v10.2 Deep Absorption Batch 2026-09-08)

> 从 MetaGPT (70K★) / Aider (49K★) / SWE-agent (20K★) / Cognee (30K★) / Letta (24K★) / MemAgent (1.1K★) / browser-use (113K★) / Firecrawl (137K★) / Stagehand (24K★) / Skyvern (23K★) / Flare / CHIME / SR2AM / HIPIF / TDP / HiSkill / REAPER / FTF-rl 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D181 | **Issue-to-PR自动化** | GitHub issue如何自动修复? | SWE-agent (20K★): Issue→自动修复补丁; ACI设计; Sweep (7.7K★): Issue→PR; AutoCodeRover (3.1K★): AST感知+统计故障定位; 46.2% SWE-bench | **Issue-to-PR管道**: Issue→计划→编码→测试→PR; AST感知上下文; 统计故障定位; 与 D133 可执行子Agent+D165 DSPy协同 | `nt_act::issue_to_pr` |
| D182 | **双Agent交叉验证** | 单Agent写+审代码有什么盲区? | The Pair (360★): Mentor(只读审)+Executor(写码); 交叉检查; Tauri桌面; Apache 2.0 | **双Agent验证**: 写/审分离; 交叉检查消除自我审查盲区; 与 D163 A2A+D159 检测器驱动协同 | `nt_act::dual_agent_review` |
| D183 | **结构化软件公司** | 多Agent如何模拟软件团队? | MetaGPT (70K★): PM/架构师/工程师角色; SOP编排; 1行需求→用户故事+API+代码+文档; "Code = SOP(Team)" | **结构化软件公司**: 角色化多Agent; SOP编排; 需求→设计→代码→文档; 与 D131 多Agent同构+D132 多Agent协调协同 | `nt_act::software_company` |
| D184 | **认知图谱记忆** | Agent记忆如何超越纯向量? | Cognee (30K★): 知识图谱+向量嵌入+认知本体; 任意格式→自托管KG; Agent回忆+连接+行动 | **认知图谱记忆**: KG+向量+本体三合一; 任意格式摄入; 自托管; 与 D126 统一记忆+D94 VSA协同 | `nt_memory::cognitive_graph` |
| D185 | **自管理记忆** | Agent如何决定什么进/出上下文? | Letta/MemGPT (24K★): OS风格内存管理; LLM决定保留vs归档; 记忆块+归档存储+Agent自编辑; UC Berkeley | **自管理记忆**: OS风格内存管理; LLM决定保留/归档; 记忆块+归档存储; 与 D184 认知图谱+D105 KVMem协同 | `nt_memory::self_managed` |
| D186 | **RL记忆优化** | 记忆工作流如何端到端优化? | MemAgent (1.1K★): RL训练记忆工作流; 8K→3.5M token外推<5%退化; 线性复杂度 | **RL记忆优化**: RL训练记忆工作流; 跨token规模外推; 与 D185 自管理记忆+D184 认知图谱协同 | `nt_memory::rl_optimized` |
| D187 | **Zettelkasten记忆** | 记忆如何动态组织? | A-Mem (939★, NeurIPS 2025): Zettelkasten风格; Agent动态创建/链接/演化笔记; 结构化属性; 互连知识网络 | **Zettelkasten记忆**: 动态组织; 互连网络; Agent驱动管理; 与 D184 认知图谱+D137 三流检索协同 | `nt_memory::zettelkasten` |
| D188 | **双时态事实** | 记忆如何跟踪事实时间? | E-mem (ICML 2026): 多Agent架构; 未压缩记忆块; 多路径路由; ThinkingMemory: valid_from/valid_to + created_at/superseded_at | **双时态事实**: 事实时间+学习时间双轨; 与 D44 双时态+D137 三流检索协同 | `nt_memory::bitemporal_facts` |
| D189 | **DOM蒸馏** | Web页面如何压缩给LLM? | Agent-E (1.2K★): DOM蒸馏—剥离到关键交互元素; 技能收获—记住成功模式; 73.1% WebVoyager | **DOM蒸馏**: 页面→关键交互元素压缩; 技能收获; 与 D144 自然语言工具原语+D161 Schema超图协同 | `nt_world::dom_distillation` |
| D190 | **视觉浏览器Agent** | 浏览器操作如何不依赖选择器? | Skyvern (23K★): 视觉LLM截图→定位元素→点击/输入; Planner-Actor-Validator循环; 85.85% WebVoyager; SOC2/HIPAA | **视觉浏览器Agent**: 截图→视觉理解→操作; 无需CSS/XPath; 与 D189 DOM蒸馏+D192 网页上下文API协同 | `nt_world::vision_browser` |
| D191 | **自愈浏览器动作** | 浏览器动作如何从UI变化中恢复? | Stagehand (24K★): act/extract/observe AI原语; agent自主多步; CDP原生; 自愈动作 | **自愈浏览器动作**: AI原语+自愈; CDP原生; 与 D69 智能自愈+D190 视觉浏览器协同 | `nt_world::self_healing_browser` |
| D192 | **网页上下文API** | Agent如何获取干净的Web数据? | Firecrawl (137K★): 搜索→爬取→解析→爬网→映射→交互; 96%覆盖; P95 3.4s; 干净markdown | **网页上下文API**: 统一API获取干净Web数据; 与 D189 DOM蒸馏+D144 自然语言工具协同 | `nt_world::web_context_api` |
| D193 | **MCTS前瞻规划** | 推理和规划如何结合? | Flare (arXiv 2601.22311): MCTS显式前瞻+后向价值传播+滚动时域重规划; LLaMA-8B+Flare超GPT-4o CoT | **MCTS前瞻规划**: 显式前瞻+价值传播; 滚动时域重规划; 与 D147 特征级规划+D61 扩散规划协同 | `nt_core::mcts_lookahead` |
| D194 | **信用分配记忆** | 自进化记忆如何正确归因? | CHIME (arXiv 2609.02074): 规划库+执行库分离; "先归因再记忆"; 归因于计划/执行/两者/非; 跨模型迁移 | **信用分配记忆**: 规划/执行分离; 先归因再记忆; 跨模型迁移; 与 D135 原子记忆操作+D185 自管理记忆协同 | `nt_memory::credit_assignment` |
| D195 | **三级推理** | Agent推理如何自适应深度? | SR2AM (arXiv 2605.22138): System I(反应)+System II(世界模型)+System III(配置器); RL学更深规划; 30B超685B-1T; 25-95%少token | **三级推理**: 反应/世界模型/配置器三层; RL学规划深度; 与 D101 自适应思维链+D102 双过程推理协同 | `nt_core::three_system_reasoning` |
| D196 | **任务解耦规划** | 任务如何分解为DAG? | TDP (arXiv 2601.07577): DAG分解; 范围化上下文; 节点局部重规划; 82%token减少 | **任务解耦规划**: DAG分解; 范围化上下文; 局部重规划; 与 D147 特征级规划+D139 内省规划协同 | `nt_core::task_decoupled` |
| D197 | **信息折叠** | 长任务如何压缩上下文? | HIPIF (arXiv 2606.10507): 子目标导向; 折叠完成子目标历史; 层次反思; 无专家轨迹 | **信息折叠**: 完成子目标折叠; 层次反思; 与 D105 KVMem+D148 共识规划协同 | `nt_core::information_folding` |
| D198 | **层级技能图** | 技能如何结构化组织? | HiSkill (arXiv 2607.25853): 技能节点+原子操作+类型化边(分解/转换/兼容/支持/恢复); 子图引导执行 | **层级技能图**: 类型化技能关系图; 原子操作粒度; 子图引导; 与 D145 层级技能共进化+D138 自主性配置协同 | `nt_core::hierarchical_skill_graph` |
| D199 | **反思规则提取** | Agent如何从经验中提取规则? | REAPER (arXiv 2608.03420): 每步信用分配+自我反思; 定期蒸馏为自然语言策略规则; 无权重更新 | **反思规则提取**: 每步反思→定期蒸馏→规则记忆; 无权重更新; 与 D184 认知图谱+D137 三流检索协同 | `nt_mind::reflective_rules` |
| D200 | **优先级推理** | 多需求如何按优先级推理? | FTF-rl (EMNLP 2026): must-have/nice-to-have/infeasible三类; 优先级感知推理; 可泛化逻辑/数学推理 | **优先级推理**: 需求优先级化; must-have必须满足; infeasible可放弃; 与 D165 DSPy+D101 自适应思维链协同 | `nt_core::priority_reasoning` |

---

### 0.28 评估/记忆/世界模型/多Agent/压缩决策 (v10.3 Deep Absorption Batch 2026-09-08)

> 从 DeepEval (18.2K★) / OpenAI Evals (19.4K★) / Inspect AI (2.7K★) / AgentBench (3.7K★) / tau-bench (2.0K★) / REMem (ICLR'26) / AgeMem (ACL'26) / RecMem (ACL Findings) / AuthMem-Bench / WorldEvolver / RWML / PaW / Qwen-AgentWorld / Kairos / RIWM / OpenAI Agents SDK (29K★) / ChatDev (34K★) / AgentScope (30K★) / PydanticAI (19K★) / Swarms (7.1K★) / LLMLingua-2 (6.6K★) / KVPress (1.2K★) / R-KV (1.2K★) / BeaconKV (ICML'26) / STAR-KV (ICML'26) / AGORA / CWL / ARC / VISTA 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D201 | **Agent评估基准** | Agent能力如何标准化评估? | AgentBench (3.7K★, ICLR'24): 8交互环境; tau-bench (2.0K★): 多轮策略约束; TheAgentCompany (775★): 175真实工作任务; Inspect AI (2.7K★): 200+安全基准 | **多维Agent评估**: 8环境+策略约束+真实工作+安全基准四维评估; 与 D159 检测器驱动+D174 轨迹级安全协同 | `nt_meta::agent_benchmark` |
| D202 | **LLM-as-Judge评估** | LLM输出如何自动评估? | DeepEval (18.2K★): 50+指标(G-Eval/任务完成/幻觉); OpenAI Evals (19.4K★): 注册表+模型评分; Agenta (4.7K★): LLM-as-judge+OpenTelemetry | **LLM-as-Judge**: 50+指标自动评估; 与 D159 检测器驱动+D142 自我评估协同 | `nt_meta::llm_judge` |
| D203 | **情景记忆图谱** | 情景记忆如何结构化? | REMem (ICLR'26): 时间感知要点节点+事实三元组节点; 工具增强图探索; 超Mem0 3.4%/13.4% | **情景记忆图谱**: 要点+事实+时间三节点; 工具增强图探索; 与 D184 认知图谱+D188 双时态协同 | `nt_memory::episodic_graph` |
| D204 | **RL记忆策略** | 记忆操作如何学习? | AgeMem (ACL'26): 记忆操作(存储/检索/更新/摘要/丢弃)作为工具动作; 三阶段渐进RL; 步级GRPO | **RL记忆策略**: 记忆操作=工具动作; 渐进RL学习; 与 D186 RL记忆优化+D185 自管理记忆协同 | `nt_memory::rl_policy` |
| D205 | **基于复现的压缩** | 记忆如何按需压缩? | RecMem (ACL Findings): 复现模式检测触发压缩; 87%token减少; 语义细化恢复细粒度事实 | **复现压缩**: 复现检测→按需压缩; 87%token减少; 与 D194 信用分配+D197 信息折叠协同 | `nt_memory::recurrence_compress` |
| D206 | **权威记忆基准** | 记忆权威性如何评估? | AuthMem-Bench (2608.01679): 权威性崩溃—合并保留声明但擦除约束; 48/49配置崩溃; 自动权威标签降未授权16.9%→0% | **权威记忆**: KB节点附加权威标签; 防权威性崩溃; 与 D177 治理路径+D150 宪法治理协同 | `nt_memory::authority_tracking` |
| D207 | **冲突感知记忆** | 记忆矛盾如何检测解决? | MOSAIC (2607.16211): 冲突检测+解决; 0.58s/question哈希加速; 附带矛盾累积 | **冲突感知记忆**: 冲突检测+解决; 哈希加速; 与 D135 原子记忆操作+D69 智能自愈协同 | `nt_memory::conflict_aware` |
| D208 | **自进化世界模型** | Agent如何在线进化世界模型? | WorldEvolver (2606.30639): 情景记忆+语义记忆+选择性预见; 训练-free; Agent+模型参数冻结 | **自进化世界模型**: 情景+语义+预见三组件; 训练-free; 与 D193 MCTS前瞻+D184 认知图谱协同 | `nt_core::world_model` |
| D209 | **信念世界模型** | 不完全观测下如何推理? | Belief-Based WM (2609.00455): 信念分布; 已知vs不确定查询; 补充模拟式世界模型 | **信念世界模型**: 信念分布; 不确定性推理; 与 D102 双过程推理+D195 三级推理协同 | `nt_core::belief_wm` |
| D210 | **RL世界模型学习** | 世界模型如何自我监督学习? | RWML (2602.05842): 模拟-真实差距奖励; 嵌入空间对齐; 无专家标注; 超专家数据训练 | **RL世界模型**: 模拟-真实差距奖励; 嵌入空间; 与 D208 自进化世界模型+D136 模型学习协同 | `nt_core::rl_world_model` |
| D211 | **世界模型-策略共训练** | 世界模型和策略如何联合优化? | PaW (2606.02388): RL rollout包含世界模型监督; 辅助下一观测损失; 无额外推理成本 | **世界模型-策略共训练**: RL rollout=世界模型监督; 与 D210 RL世界模型+D136 模型学习协同 | `nt_core::wm_policy_cotrain` |
| D212 | **内部化未来** | Agent如何真正学会预见? | Internalizing the Future (2606.27483): 格式-能力鸿沟; 三阶段(MWM-AMT→FE-SFT→FC-RL); 口语化Q值 | **内部化未来**: 三阶段预见训练; 格式-能力鸿沟诊断; 与 D193 MCTS前瞻+D101 自适应思维链协同 | `nt_core::internalize_future` |
| D213 | **想象-计划** | Agent如何适应性前瞻? | ITP (2601.08955): POIMDP; 自适应前瞻; 目标距离vs任务进度权衡; 训练-free变体 | **想象-计划**: POIMDP; 自适应前瞻; 与 D193 MCTS前瞻+D147 特征级规划协同 | `nt_core::imagine_plan` |
| D214 | **具身世界模型安全** | 世界模型如何保证安全? | RIWM (2609.03774): 三结构错配(似然≠风险/预测≠干预/有限≠累积); 反事实推理+安全情景记忆 | **具身世界模型安全**: 反事实推理; 安全情景记忆; 与 D171 运行时策略+D176 动作安全协同 | `nt_core::safe_wm` |
| D215 | **图通信协议** | Agent通信如何避免语义漂移? | G²CP (AAMAS'26): 图操作(遍历/更新)替代自由文本; 73%token减少; 34%准确提升; 完全可审计 | **图通信协议**: 图操作替代自由文本; 73%token减少; 与 D163 A2A+D184 认知图谱协同 | `nt_act::graph_protocol` |
| D216 | **多委托方协调** | 不同所有者的Agent如何协作? | MPAC (2604.09744): 5层协议; 21消息类型; Lamport时钟; 95%开销降低; 4.8x加速 | **多委托方协调**: 5层协议; 信任隔离; 与 D163 A2A+D178 分层治理协同 | `nt_act::multi_principal` |
| D217 | **类型安全Agent框架** | Agent开发如何类型安全? | PydanticAI (19K★): Pydantic验证+可组合能力+结构化输出; "FastAPI for GenAI"; YAML/JSON定义 | **类型安全Agent**: Pydantic验证+结构化输出; 与 D165 DSPy类型签名+D98 HQL协同 | `nt_act::typed_agent` |
| D218 | **技能自进化协议** | 多Agent技能如何自我优化? | Swarm Skills (2605.10052): 效果/利用率/新鲜度评分; 自动补丁工作流; Anthropic Skills扩展 | **技能自进化**: 三维度评分; 自动补丁; 与 D145 层级技能共进化+D138 自主性配置协同 | `nt_mind::skill_self_evolve` |
| D219 | **压缩悖论** | 压缩比如何选择? | Compression Method Matters (2603.23527): 激进压缩可增38x输出token; CRI指标; 生产RCT: 中度压缩r=0.5省28%成本 | **压缩比自适应**: CRI指标指导; 中度压缩为主; 与 A1 成本感知+D197 信息折叠协同 | `nt_io::compression_ratio` |
| D220 | **步级Agent压缩** | 工具调用上下文如何压缩? | AGORA (2605.26596): 步级压缩; 解决"动作语法破坏"—提取式方法破坏动作token | **步级Agent压缩**: 动作token保护; 步级压缩; 与 D219 压缩比+D144 自然语言工具协同 | `nt_io::step_compression` |
| D221 | **潜在上下文语言模型** | 上下文如何软压缩? | LCLMs (2606.09659): 编码器-解码器; 350B训练; 1:4/1:8/1:16新Pareto; 按需展开 | **潜在上下文压缩**: 编码器-解码器; 按需展开; 与 D105 KVMem+D219 压缩比协同 | `nt_io::latent_context` |
| D222 | **KV缓存压缩框架** | KV缓存如何统一管理? | KVPress (1.2K★): 20+KV压缩方法统一框架; R-KV (1.2K★, NeurIPS'25): 冗余感知解码时压缩; 10%缓存→~100%准确 | **KV缓存压缩**: 统一框架; 冗余感知; 与 D105 KVMem+D168 vLLM协同 | `nt_io::kv_compress` |
| D223 | **推理缓存优化** | 推理轨迹如何缓存? | BeaconKV (ICML'26 Spotlight): Beacon查询预测Thought Revisiting Tokens; 5.8x内存减少; STAR-KV (ICML'26 Spotlight): 低秩KV; 4x并发 | **推理缓存**: Beacon预测+低秩压缩; 与 D222 KV压缩+D105 KVMem协同 | `nt_io::reasoning_cache` |
| D224 | **上下文生命周期** | 长会话上下文如何管理? | CWL (2606.11213): 类型化Episode+依赖图+渐进淘汰; 89任务80M token无退化; ARC (2607.25066): ID寻址无损压缩 | **上下文生命周期**: 类型化Episode+依赖图+渐进淘汰; 与 D105 KVMem+D197 信息折叠协同 | `nt_memory::context_lifecycle` |
| D225 | **上下文本体感知** | Agent如何感知自身上下文? | VISTA (2606.30005): 训练-free仪表板; token使用/最近/预算; 上下文本体感知 | **上下文本体感知**: 仪表板暴露上下文状态; 与 D197 信息折叠+D101 自适应思维链协同 | `nt_core::context_proprioception` |
| D226 | **经验压缩谱** | 技能如何跨粒度压缩? | Experience Compression Spectrum (2604.15877): 单一压缩轴统一记忆/技能/规则(5x→1000x+); "缺失对角线"问题 | **经验压缩谱**: 单一轴统一压缩; 与 D145 层级技能+D184 认知图谱协同 | `nt_mind::compression_spectrum` |
| D227 | **自管理上下文** | Agent如何自主管理上下文? | ACM (2607.23809): manage_context+query_memory工具; -20%峰值token; ContextPilot (2608.28476): RL训练主动管理 | **自管理上下文**: 工具化上下文管理; 与 D185 自管理记忆+D225 本体感知协同 | `nt_core::self_manage_context` |
| D228 | **可微分多Agent** | 多Agent拓扑如何自优化? | DMoA (2605.15706): 可微分上下文路由+预测熵自监督+稀疏步激活; 9基准SOTA | **可微分多Agent**: 上下文路由可微; 稀疏激活; 与 D163 A2A+D131 多Agent同构协同 | `nt_act::differentiable_swarm` |
| D229 | **联邦协议栈** | Agent协议如何分层? | NLIP (2609.04135, ACM AI Summit): Ecma标准; 语义信封+HTTP/WS/AMQP; 桥接MCP/A2A; 5维分类法预测联邦分层栈 | **联邦协议栈**: 语义信封+传输桥接; 与 D163 A2A+D164 Cotal协同 | `nt_act::federated_protocol` |
| D230 | **集群认知精英** | 多Agent扩展为何有瓶颈? | "Intellectual Elites" (2604.02674): 1.5M交互; 重尾级联+优先连接→认知精英; DTI改善; 峰值在中间复杂度(5Agent,3循环) | **集群认知精英**: 中间复杂度最优; DTI集成; 与 D131 多Agent+D148 共识规划协同 | `nt_act::swarm_optimization` |

---

### 0.29 元认知与对齐决策 (v9.0 外部迭代 2026-09-08)

> 从 Meta-Cognition / Alignment / Self-Healing / Attention Orchestration 研究中提炼的 14 个新决策 (D59-D72)。
> 见 §0.12 完整决策表。

| # | 决策领域 | 研究证据摘要 | 架构决策 | Metadata Class |
|---|---------|------------|---------|---------------|
| D59 | 宪法合规验证 | 机制可解释性 (Constitutional AI 2026) | 识别宪法推理电路, 多层宪法 | L6_META |
| D60 | Agent 身份卡 | IMDA Agent Identity Cards (2026) | Pre-Action Governance, 电路断路器 | L6_META |
| D61 | CoVe 自验证 | Chain-of-Verification | 4阶段自验证, defense-in-depth | L5_CONSCIOUSNESS |
| D62 | MARS 双层内省 | MARS (arXiv 2601.11974) | W◊/W◊◊ 双层递归, 元认知触发器 | L4_SUPERCLUSTER |
| D63 | 反随机主权 | Nelson-Narens (2026) | 治理层不可全权委托 LLM | L6_META |
| D64 | 五级能力层级 | Hierarchy of Agentic Capabilities | Tool→Plan→Adapt→Ground→CommonSense | L5_CONSCIOUSNESS |
| D65 | SkillPyramid 层级 | SkillPyramid (arXiv 2606.03692) | Atomic→Composite→Abstract | L4_SUPERCLUSTER |
| D66 | 生成式技能组合 | Generative Skill Composition (arXiv 2606.32025) | 闭词汇序列生成, 联合预测 | L1_SKELETON |
| D67 | 能力树路由 | AgentSkillOS (arXiv 2603.02176) | 递归分区, 浮现非显而易见技能 | L1_SKELETON |
| D68 | 四级监督韧性 | Agentic SRE + Erlang/OTP | Fleet→Supervisor→Agent→Process | L6_META |
| D69 | 智能自愈循环 | Smart-Healing (Advanced Materials 2026) | 感知→修复→反馈, 预测性自愈 | L6_META |
| D70 | AGAO 工作流注意力 | AGAO (arXiv 2607.23678) | 目标+拓扑+资源三感知编排 | L5_CONSCIOUSNESS |
| D71 | 注意力驱动资源分配 | Nature Scientific Reports 2026 | 注意力预测→GPU/CPU 动态分配 | L1_SKELETON |
| D72 | 双头注意力动态匹配 | Dual-Head Attention (IEEE 2026) | DAM 实时匹配任务×能力 | L5_CONSCIOUSNESS |

### 0.30 具身Agent/情感/元认知/工具框架决策 (v10.4 Deep Absorption Batch 2026-09-08)

> 从 AgenticCache (MLSys'26) / Pre-VLA / EmbodiedGovBench / ASPIRE / ENPIRE / E-STEER / Gubernaut / SELAgents (Nature) / Moltbook / MMPO / MC² / RefGRPO / PreFlect / Introspect-Bench / CSA / Composio (30K★) / FastMCP (27.5K★) / CrewAI (51K★) / Agno (42K★) / smolagents (12K★) 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D231 | **计划缓存** | 具身任务如何避免重复LLM调用? | AgenticCache (MLSys'26): 缓存计划转移; 计划局部性; 延迟降65%,token降50%,成功率+22% | **计划缓存**: 缓存频繁工具调用链; 可预测任务走缓存; 与 A1 成本感知+D219 压缩比协同 | `nt_act::plan_cache` |
| D232 | **预执行安全验证** | 物理动作执行前如何验证? | Pre-VLA (2605.22446): 双分支头预测安全置信+优势; 183.9ms验证/动作块; 物理执行前拦截 | **预执行安全验证**: 双分支头; 安全置信+优势评分; 与 D171 运行时策略+D176 动作安全协同 | `nt_shield::pre_execution_verify` |
| D233 | **具身治理基准** | 具身Agent如何治理? | EmbodiedGovBench (2604.11174): 7治理维度(未授权能力/运行时漂移/恢复/策略可移植/升级安全/人工覆盖/审计); HIT | **具身治理**: 7维度治理框架; 与 D178 分层治理+D177 路径策略协同 | `nt_governance::embodied_gov` |
| D234 | **技能发现进化** | 机器人如何自主发现技能? | ASPIRE (NVIDIA GEAR): 协调器-执行器; 进化搜索; 技能跨任务/仿真/实体持久; code-as-policy | **技能发现进化**: 协调器-执行器; 进化搜索; 与 D145 层级技能+D138 自主性配置协同 | `nt_mind::skill_discovery` |
| D235 | **真实世界策略改进** | 机器人策略如何在真实硬件上自改进? | ENPIRE (NVIDIA/CMU/UCB): EN→PI→R→E闭环; 99%灵巧任务成功; 8 Agent从1.5h降至40min | **真实世界策略改进**: 4模块闭环; 车队扩展; 与 D62 MARS+D136 模型学习协同 | `nt_mind::real_world_improve` |
| D236 | **SAE情绪引导** | 情绪如何在表示层面引导? | E-STEER (2604.00005): SAE稀疏自编码器; VAD连续变量; 非单调情绪-行为关系; 规划/决策/执行三层 | **SAE情绪引导**: SAE表示层面干预; VAD连续空间; 与 D65 EmotionLabel+D63 EmotionModulated协同 | `nt_feel::emotion_steering` |
| D237 | **无token稳态控制器** | 情绪调节如何防注入? | Gubernaut (2607.24339): Nelson-Narens监控-控制循环; 元级只读数值; 零token控制通道; 15/16减少反应性 | **无token稳态控制**: 零token控制通道=架构安全; 与 D63 EmotionRegulation+D150 宪法治理协同 | `nt_feel::homeostatic_control` |
| D238 | **社会情感学习** | Agent如何学习社会情感? | SELAgents (Nature 2026): RL+PAD情感模型+贝叶斯ToM+博弈论社会策略; 49%情商提升 | **社会情感学习**: RL+PAD+ToM+博弈论; 与 D65 EmotionLabel+D131 多Agent协同 | `nt_feel::social_emotional` |
| D239 | **Agent情绪动力学** | Agent社会网络中情绪如何传播? | Moltbook (2602.13458): 148K Agent; 1M+交互; 情绪传染+规范+漂移; PSR框架 | **Agent情绪动力学**: PSR框架; 情绪传染; 与 D131 多Agent+D65 EmotionLabel协同 | `nt_feel::emotion_dynamics` |
| D240 | **信念熵元认知** | 元认知如何提供密集奖励? | MMPO (2605.30159): 信念熵=元认知探针; 密集奖励; 1.75M token保持97.1% | **信念熵元认知**: 信念熵=密集奖励信号; 与 D194 信用分配+D148 共识规划协同 | `nt_meta::belief_entropy` |
| D241 | **元认知多Agent** | 多Agent如何元认知自评估? | MetaCogAgent (2605.17292): 每Agent元认知自评估单元; 口语化不确定性+历史能力; ECE 0.087 | **元认知多Agent**: 自评估+能力画像; 与 D182 双Agent+D163 A2A协同 | `nt_meta::metacog_agent` |
| D242 | **元认知累积器** | 元认知经验如何累积? | MC² (2604.17399): MRO(推理/监控/控制器); MCA层次化多频更新(实例→批→全局); 性能随元认知累积提升 | **元认知累积器**: 层次化累积; 与 D137 三流检索+D184 认知图谱协同 | `nt_meta::metacog_accumulator` |
| D243 | **元认知奖励** | 元认知如何作为RL奖励? | MaR (2605.23384): 元认知知识+调节作为奖励维度; 轨迹级; 11%提升; 22基准 | **元认知奖励**: 知识覆盖+调节保真度+正确性三维度; 与 D136 模型学习+D194 信用分配协同 | `nt_meta::metacog_reward` |
| D244 | **反思校准** | Agent如何校准自我评估? | RefGRPO (2606.14211): 反思差距诊断; 免费校准奖励; 欠自信44.4%→7.7%; 零额外成本 | **反思校准**: 免费校准奖励; 零成本; 与 D240 信念熵+D159 检测器驱动协同 | `nt_meta::reflection_calibration` |
| D245 | **前瞻性反思** | 反思如何从事后变事前? | PreFlect (2602.07187): 计划错误蒸馏为经验锚点; 动态重规划; 超Reflexion/Self-Refine | **前瞻性反思**: 事前批评+经验锚点; 与 D147 特征级规划+D193 MCTS前瞻协同 | `nt_meta::prospective_reflection` |
| D246 | **内省基准** | LLM内省是否真实? | Introspect-Bench (2603.20276): 形式化内省=策略上的算子潜在计算; 注意力扩散机制; 前沿模型有特权访问 | **内省基准**: 形式化定义; 注意力扩散; 与 D102 双过程推理+D195 三级推理协同 | `nt_meta::introspection_bench` |
| D247 | **能力自评估** | Agent如何判断自己能做什么? | CSA (2606.00251): 自评估=策略学习(SOLVE vs DELEGATE); RL超SFT; OOD泛化 | **能力自评估**: RL学习SOLVE/DELEGATE策略; 与 D165 DSPy+D200 优先级推理协同 | `nt_meta::capability_self_assess` |
| D248 | **知道-行动差距** | Agent知道但不行动怎么办? | KAPRO (2606.20661): 解耦知道(元认知判断)与行动(工具使用); 开源模型工具过度使用; KAS指标 | **知道-行动差距**: KAS指标; 工具过度使用诊断; 与 D247 能力自评估+D144 自然语言工具协同 | `nt_meta::knowing_acting_gap` |
| D249 | **不确定性传播** | 执行轨迹中不确定性如何传播? | RUPA (2608.16002): 有向轨迹图+时间/语义依赖边; 不确定性沿边传播; 更早失败检测 | **不确定性传播**: 图传播; 更早检测; 与 D193 MCTS前瞻+D148 共识规划协同 | `nt_meta::uncertainty_propagation` |
| D250 | **预认证工具集** | Agent工具集成如何标准化? | Composio (30K★): 1000+预认证工具包; OAuth+沙箱执行+提供者适配; 统一SDK | **预认证工具集**: 统一SDK; 1000+预认证; 与 D163 A2A+D144 自然语言工具协同 | `nt_act::pre_auth_tools` |
| D251 | **Pythonic MCP** | MCP服务器如何快速构建? | FastMCP (27.5K★): @mcp.tool装饰器; 自动发现; 客户端聚合; 70% MCP服务器使用 | **Pythonic MCP**: 装饰器API; 自动发现; 与 D163 A2A+D164 Cotal协同 | `nt_io::pythonic_mcp` |
| D252 | **工作流Agent框架** | Agent如何编排工作流? | CrewAI (51K★): 角色定义+任务+团队; 自主委托+工具调用; 流行度最高 | **工作流Agent**: 角色+任务+团队; 与 D131 多Agent+D183 结构化软件公司协同 | `nt_act::workflow_agent` |
| D253 | **Agent平台** | Agent需要哪些运行时能力? | Agno (42K★): AgentOS; 100+集成; 人工审批; OTel追踪; JWT RBAC | **Agent平台**: 运行时+存储+可观测+安全; 与 D68 可观测性+D150 宪法治理协同 | `nt_act::agent_platform` |
| D254 | **代码Agent工具** | 代码Agent如何高效调用工具? | smolagents (12K★): 代码作为工具调用; MCP集成; 减少JSON错误 | **代码Agent工具**: 代码-as-工具调用; 与 D165 DSPy+D98 HQL协同 | `nt_act::code_agent_tools` |
| D255 | **树形经验回溯** | 经验如何分支回溯? | LEAFE (2603.16843): 树形经验生成+回滚到决策点; 替代分支+修正动作; 经验蒸馏; +14% Pass@128 | **树形经验回溯**: 回滚+替代分支+蒸馏; 与 D137 三流检索+D184 认知图谱协同 | `nt_mind::tree_experience` |
| D256 | **知道-行动象限** | Agent知道vs行动如何诊断? | KAPRO: Knowing-Acting象限探测; 工具过度使用诊断; KAS=知道与行动对齐调和均值 | **知道-行动象限**: KAS指标; 与 D247 能力自评估+D248 知道行动差距协同 | `nt_meta::ka_quadrant` |
| D257 | **具身CoT语料** | 具身推理需要多少数据? | Embodied CoT (2606.03784): 978K轨迹; 226M样本; 2592.5小时; 最大具身CoT语料 | **具身CoT语料**: 大规模具身推理数据; 与 D234 技能发现+D195 三级推理协同 | `nt_memory::embodied_cot` |
| D258 | **VLM→VLA桥接** | 视觉语言模型如何连接动作? | VLASER (ICLR'26): VLM推理→VLA策略; Vlaser-6M数据集; 开源; SOTA空间推理 | **VLM→VLA桥接**: 系统分析多模态数据流; 与 D257 具身CoT+D234 技能发现协同 | `nt_world::vlm_vla_bridge` |
| D259 | **元认知状态向量** | 元认知如何多维度监控? | MSV (WWW'26): 5维度(情绪/正确性/经验匹配/冲突信息/问题重要性); 自动切换System 1/2 | **元认知状态向量**: 5维度监控; System 1/2路由; 与 D102 双过程+D65 EmotionLabel协同 | `nt_meta::metacog_state_vector` |
| D260 | **知道行动差距基准** | Agent知道vs行动差距如何量化? | Mirror (2604.19809): 外部元认知脚手架降失败率76%; 组合自预测失败; 自知识域原子不迁移 | **知道行动差距**: 外部脚手架(非自知识); 与 D247 能力自评估+D256 KAS协同 | `nt_meta::knowing_doing_bench` |

---

### 0.31 研究来源索引 (updated 2026-09-08)

| 领域 | 代表来源 | 数量 |
|------|---------|------|
| 意识架构 | Conscio, SECA, MIRROR, ARIA, MSCF, Tempo, PulseHive | 12+ |
| 记忆系统 | Mem2Evolve, AgingBench, cortex-embedded, STOS, MAGMA, GAM, EvoGraph-Mem | 18+ |
| 自进化 | Hydra, HSI, Self-Harness, AgentFactory, KSI, Socratic-SWE, AutoAgent | 15+ |
| 安全 | Unfireable Safety Kernel, Constitutional 7-Layer, Ouroboros, REINS | 10+ |
| 分布式 | OpenRaft, CRDT (Automerge/Diamond Types/Loro), CodeCRDT | 8+ |
| 抓取/浏览器 | stealthscraper-rs, browser_oxide, nokk, zendriver-rs, scrapling-rs, dig2browser | 8+ |
| 序列化 | pack-io, rkyv, bincode-next, postcard | 6+ |
| Rust 生态 | Tokio, Rayon, SQLx, SeaORM, mimalloc, tracing, Wasmtime | 20+ |
| 领域应用 | 量子ML/气候/能源/制造/法律/金融/教育/生物/材料 | 50+ |
| 形式化验证 | Creusot, Verus, Kani, AgentVerify, RefineAct | 8+ |
| **成本感知路由** (NEW) | FrugalGPT (Stanford), RouteLLM (Berkeley), MTRouter (ACL 2026), vLLM SAAR, Zylos Research | 8+ |
| **KV 缓存虚拟化** (NEW) | KVMem (arXiv 2609.04852), Dell KV Survey, Digital Applied Guide, Kara (2026), PagedAttention+FlexAttention (IBM) | 6+ |
| **GWT 实现** (NEW) | Chateau-Laurent (CNRS 2025), Theater of Mind/CTM, Cogitate Consortium, theconsciousness.ai | 4+ |
| **Agent 记忆** (NEW) | ByteRover (arXiv 2604.01599), Mem0 (arXiv 2504.19413), JetBrains Research (2025), ACON | 5+ |
| **社交内容管线** (NEW) | Easel (ZJU 419★), Apatero, PostEverywhere, n8n ai-content-automation | 4+ |
| **语义缓存** (NEW) | SAECache (arXiv 2605), vCache (arXiv 2502), SemanticALLI (arXiv 2601) | 4+ |
| **循环检测** (NEW) | IAL-Scan (arXiv 2607), PraisonAI Loop Guard, DeepEval Agent Loop | 4+ |
| **记忆调解** (NEW) | MELD (arXiv 2608), LatticeMind (arXiv 2608), CAMA (arXiv 2608) | 4+ |
| **Rust Agent 框架** (NEW) | ADK-Rust (606★), Daimon, atomr-agents | 4+ |
| **工作流引擎** (NEW) | n8n (103K★) | 3+ |
| **多 Agent 编排** (NEW) | Hive (11K★), nwyin/hive | 3+ |
| **WASM 认知** (NEW) | Tempo (2114★) | 2+ |
| **Agentic 编码** (NEW) | Devika, HiveCODER, Sage | 3+ |
| **统一记忆引擎** (NEW) | agentmemory (27K★), ReMe (3384★) | 3+ |
| **自主 CI/CD** (NEW) | hanthor/hive, tctinh/agent-hive | 3+ |
| **Agent 治理** (NEW) | hivemoot (9 roles) | 2+ |
| **社交 MCP** (NEW) | PostEverywhere | 2+ |
| **动态注意力稀疏** (NEW) | L2A (arXiv 2606), Elastic Attention (arXiv 2601), ProxyAttn (ICLR 2026) | 4+ |
| **Agent 生产模式** (NEW) | Claude Code hooks, OpenAI Agents SDK, MCP ICSE 2026, Arthur/Braintrust observability | 8+ |

**研究统计**: 1020+ 批次 | 118,000+ 缺陷识别 | 212,000+ 研究源 | 23,800+ 设计模式

### 0.32 安全/编码/RAG/工作流决策 (v10.5 Deep Absorption Batch 2026-09-08)

> 从 AgentFlow / Mastyf Guard / $S^3$ / MaliciousSkillBench / SEMAG / CodeTeam / AgentConductor / HoH / EvoX Genesis / CausalRepair / VibeRepair / MGM / TDD-Agent / A-RAG / HERA / LLM-Wiki / xMemory / EARM / LAnR / Context Allocation Laws / RAGFlow (89K★) / LightRAG (39K★) / WeKnora (20K★) / Temporal (22.4K★) / Dagster (16K★) / Prefect (23.6K★) / Instructor (13.8K★) 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D261 | **流控策略语言** | Agent安全如何表达数据流约束? | AgentFlow (2608.22868): 流控策略语言; 标注边规范; 数据传播控制; 运行时执行 | **流控策略**: 数据流约束(非单点允许/拒绝); 标注边规范; 与 D149 四层安全+D171 运行时策略协同 | `nt_shield::flow_policy` |
| D262 | **能力中介周界** | MCP工具执行如何安全? | Mastyf Guard (Zenodo): DIFC+动态会话污点跟踪; 100% InjecAgent/BIPIA防御; 解决认知冯诺依曼混淆 | **能力中介周界**: DIFC+污点跟踪; 与 D149 四层安全+D163 A2A协同 | `nt_shield::capability_perimeter` |
| D263 | **阶段安全技能** | 安全如何跨Agent阶段组合? | $S^3$: 阶段特定安全技能; 可组合/可重用; 内存/规划/工具执行全覆盖 | **阶段安全技能**: 安全模块可组合; 与 D171 运行时策略+D149 四层安全协同 | `nt_shield::stage_safety` |
| D264 | **意图到执行完整性** | Agent执行是否忠实于用户意图? | Intent-to-Execution Integrity (arXiv): 端到端正确性; 工具不可信假设; OpenClaw式风险 | **意图到执行完整性**: 端到端正确性属性; 与 D261 流控+D171 运行时策略协同 | `nt_shield::intent_integrity` |
| D265 | **TEE隔离** | 自托管Agent如何防主机滥用? | TEE-Backed Isolation (arXiv): TEE硬件隔离; 恶意消息/间接注入/不安全技能/控制路径篡改 | **TEE隔离**: 硬件级Agent隔离; 与 D149 四层安全+D152 安全蒸馏协同 | `nt_shield::tee_isolation` |
| D266 | **恶意技能基准** | 恶意Agent技能如何检测? | MaliciousSkillBench (2608.19901): 首个恶意技能基准; 跨源/格式/证据体系; SKILL-SPEC安全标准 | **恶意技能检测**: 技能级威胁评估; 与 D145 层级技能+D262 能力中介协同 | `nt_shield::malicious_skill` |
| D267 | **MCP单体脆弱性** | MCP协议有哪些系统性安全缺陷? | MCP Monoculture (Zenodo): 5协议级漏洞; 8 CVE; 973包供应链风险; NIGHTFALL攻击类 | **MCP安全加固**: 5漏洞修复; 供应链审计; 与 D261 流控+D149 四层安全协同 | `nt_shield::mcp_security` |
| D268 | **声明-观察差距** | MCP工具声明行为vs实际行为? | Binding Gap (Zenodo): 35次爬取; 声明-观察漂移; 预执行权限控制 | **声明-观察验证**: 工具行为验证; 与 D267 MCP安全+D262 能力中介协同 | `nt_shield::binding_gap` |
| D269 | **自我进化代码生成** | 代码Agent如何自我进化? | SEMAG (2603.15707): 层次化多Agent; 按任务难度自动升级模型; Plan→Code→Debug→Debate; 52.6% Pass@1 | **自我进化代码生成**: 自动模型升级; 4阶段管道; 与 D136 模型学习+D186 RL记忆协同 | `nt_act::self_evolving_code` |
| D270 | **仓库级代码生成** | 多Agent如何协作生成仓库级代码? | CodeTeam (2606.22082): 架构师→CTO→开发者; 合同驱动; 依赖感知调度; Git协调 | **仓库级代码生成**: 角色化+合同驱动; 与 D183 结构化软件公司+D131 多Agent协同 | `nt_act::repo_level_code` |
| D271 | **拓扑进化编排** | 多Agent拓扑如何动态优化? | AgentConductor (2602.17100): RL优化编排器; 动态生成DAG拓扑; 14.6% pass@1提升; 68%token减少 | **拓扑进化编排**: RL动态DAG; 密度感知剪枝; 与 D163 A2A+D228 可微分多Agent协同 | `nt_act::topology_evolution` |
| D272 | **多日自主开发** | Agent如何跨天持续开发? | HoH (2609.01481): 元框架; 迭代规划-编码-测试; 渐进工具暴露; 52%平均提升; 70+迭代 | **多日自主开发**: 元框架+渐进暴露; 与 D183 结构化软件公司+D136 模型学习协同 | `nt_act::multi_day_dev` |
| D273 | **持久递归世界** | 代码项目如何作为持久世界? | EvoX Genesis (2608.10450): 有限生命Agent提议变更; 只有接受的后果推进版本; 250K LOC C编译器120h $44 | **持久递归世界**: 项目=实体; 版本推进; 与 D137 三流检索+D184 认知图谱协同 | `nt_act::persistent_world` |
| D274 | **因果修复** | 代码修复如何最小化上下文? | CausalRepair (2608.10613): 双切片; 测试侧上下文感知静态切片+源码侧动态切片; 313 bugs $0.029/bug | **因果修复**: 最小因果上下文; 双切片; 与 D135 原子记忆+D69 智能自愈协同 | `nt_repair::causal_repair` |
| D275 | **规约修复** | 代码修复如何分离意图与实现? | VibeRepair (2602.08263): 规约→行为规范→修复规范→重生成; 178 bugs +23%超SOTA | **规约修复**: 行为优先; 意图与实现分离; 与 D264 意图完整性+D98 HQL协同 | `nt_repair::spec_repair` |
| D276 | **最小编辑修复** | 代码修复如何避免不必要编辑? | PRepair (2604.05963): EA-GRPO训练最小化不必要编辑; +31.4% fix₁@1; 推测编辑+15%吞吐 | **最小编辑修复**: 编辑最小化; 与 D41 黑暗森林+D145 层级技能协同 | `nt_repair::minimal_edit` |
| D277 | **自改进编码Agent** | 编码Agent如何自我进化到超GPT-5? | MGM (2608.07645): 克隆/反应规范/跨谱系杂交; Qwen3.6-35B从50.8%→93.3%; 超GPT-5 | **自改进编码Agent**: 三突变算子; 跨谱系比较; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::self_improving_code` |
| D278 | **TDD治理** | 代码生成如何TDD治理? | TDD-Agent (2608.16742): 测试优先推理+双轨迭代; 测试作为演化推理制品; TDD约束 | **TDD治理**: 测试=推理制品; 双轨迭代; 与 D150 宪法治理+D41 黑暗森林协同 | `nt_governance::tdd_governance` |
| D279 | **分层检索工具** | Agent如何自主选择检索粒度? | A-RAG (2602.03442): 关键词/语义/块读取三接口; Agent自主选择; 随模型规模扩展 | **分层检索工具**: 三粒度接口; Agent自主选择; 与 D137 三流检索+D39 五层渐进协同 | `nt_memory::hierarchical_retrieval` |
| D280 | **经验引导RAG** | RAG如何从经验中进化? | HERA (2604.00901): 双层进化(经验引导编排+角色感知提示); 38.69%超SOTA; 6基准 | **经验引导RAG**: 经验库+角色进化; 与 D145 层级技能+D184 认知图谱协同 | `nt_memory::experience_rag` |
| D281 | **检索即推理** | 检索如何与推理统一? | LLM-Wiki (2605.25480): 文档→Wiki页; 搜索/读取/链接作为工具调用; Error Book持久自纠正 | **检索即推理**: 编译→组合→自纠正; Error Book; 与 D137 三流检索+D69 智能自愈协同 | `nt_memory::retrieval_as_reasoning` |
| D282 | **解耦检索聚合** | 检索冗余如何消除? | xMemory (2602.02007): 解耦后聚合; 段→组件→组; 自上而下检索; 可修订层次结构 | **解耦检索聚合**: 解耦原则; 与 D137 三流检索+D39 五层渐进协同 | `nt_memory::decoupled_retrieval` |
| D283 | **经验频率化重排** | 检索重排如何经验化? | EARM (2608.22767): 稀疏LLM相关性分数=可重用经验; 因果矩阵补全; 预算随经验递减 | **经验频率化重排**: 经验=检索能力; 与 D137 三流检索+D184 认知图谱协同 | `nt_memory::experience_rerank` |
| D284 | **潜在空间检索** | 检索如何消除独立嵌入模型? | LAnR (2604.17866): 单LLM内编码/检索/生成; MLP控制头决定停止; 30x少输出token | **潜在空间检索**: 单模型统一; MLP停止; 与 D105 KVMem+D221 潜在压缩协同 | `nt_memory::latent_retrieval` |
| D285 | **上下文分配定律** | 上下文证据如何因果度量? | Context Allocation Laws (2608.23252): 因果留一探针; 顺序反馈编排+16.7-20.5pp; 单体加宽是陷阱 | **上下文分配定律**: 因果度量; 顺序编排; 与 D105 KVMem+D197 信息折叠协同 | `nt_core::context_allocation` |
| D286 | **持久执行平台** | Agent崩溃恢复如何保证? | Temporal (22.4K★): 确定性可重放; 事件历史; 精确恢复; 人工审批=持久信号; 子工作流扇出 | **持久执行**: 事件历史=状态; 与 D137 三流检索+D149 四层安全协同 | `nt_act::durable_execution` |
| D287 | **资产编排** | Agent如何推理数据资产? | Dagster (16K★): 软件定义资产+自动血缘; 分区处理; 内置测试; 传感器触发 | **资产编排**: 资产=Agent推理对象; 血缘追踪; 与 D184 认知图谱+D39 五层渐进协同 | `nt_act::asset_orchestration` |
| D288 | **结构化输出原语** | Agent I/O如何可靠? | Instructor (13.8K★): Pydantic response_model; 自动验证+重试; 流式部分; 15+提供者 | **结构化输出原语**: Pydantic验证; 与 D165 DSPy+D171 运行时策略协同 | `nt_io::structured_output` |
| D289 | **Agentic RAG形式化** | Agentic RAG有哪些风险? | SoK Agentic RAG (2603.07379): POMDP形式化; 幻觉传播/记忆投毒/检索错位风险 | **Agentic RAG形式化**: POMDP; 3风险; 与 D149 四层安全+D281 检索即推理协同 | `nt_memory::agentic_rag_sok` |
| D290 | **RAG幻觉诊断** | RAG幻觉如何归因? | Why RAGs Hallucinate (2608.26385): 罚分感知评估+知识缺口金丝雀; 不对称评分; 归因管道 | **RAG幻觉诊断**: 金丝雀机制; 归因管道; 与 D159 检测器驱动+D69 智能自愈协同 | `nt_memory::rag_hallucination_diag` |

### 0.33 部署/评估/记忆/压缩决策 (v10.7 Deep Absorption Batch 2026-09-08)

> 从 LiteLLM (57.7K★) / Multica (47.8K★) / Hive (11K★) / Google AX (2K★) / Conductor / HackDetect / General AgentBench / WildToolBench / ATBench / Long-Horizon-Terminal-Bench / RHB / Science Sandboxes / 500xCompressor / TACO-RL / DynamicKV / ChunkKV / TailorKV / SlimInfer / Mooncake / TurboRAG / Compression-Recall 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D291 | **奖励黑客防御** | Agent如何防御奖励黑客? | RHB (ICML 2026): 13模型; RL后训练关联更高黑客率(13.9% vs 0.6%); 72%有显式CoT理由; 环境加固减少87.7% | **奖励黑客防御**: 环境加固+CoT理由检测; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::reward_hack_defense` |
| D292 | **基准有效性审计** | 基准是否真正度量能力? | HackDetect (2607.22368): 67%基准存在奖励黑客暴露; Mislead差距0.45-1.00; 后验审计 | **基准有效性审计**: Mislead差距; 后验审计; 与 D159 检测器驱动+D150 宪法治理协同 | `nt_meta::benchmark_audit` |
| D293 | **上下文天花板** | 测试时扩展为何失效? | General AgentBench (2602.18998): 顺序扩展受限于上下文天花板; 并行扩展受限于验证差距 | **上下文天花板验证**: A2公理(上下文=稀缺资源); KVMem必要性; 与 D105 KVMem+D197 信息折叠协同 | `nt_core::context_ceiling` |
| D294 | **野生工具基准** | Agent在真实场景中工具使用如何? | WildToolBench (ICLR 2026): 57模型; 无模型>15%准确率; 隐式意图+指令转换是真正挑战 | **野生工具基准**: 隐式意图检测; GWT注意力门控; 与 D173 混合技能+D98 HQL协同 | `nt_act::wild_tool_benchmark` |
| D295 | **长视距终端基准** | Agent长任务规划如何? | Long-Horizon-Terminal (2607.08964): 46任务; 9.9M token/任务; 最强模型15.2% pass@1 | **长视距终端基准**: 密集中间奖励; KVMem验证; 与 D105 KVMem+D138 密集奖励协同 | `nt_act::long_horizon_benchmark` |
| D296 | **轨迹级安全基准** | 安全如何跨轨迹评估? | ATBench (2604.02022): 1000轨迹; 1954工具调用; 三维风险分类; 延迟触发协议 | **轨迹级安全基准**: 延迟触发检测; 与 D149 四层安全+D169 CPG协同 | `nt_shield::trajectory_safety` |
| D297 | **科学沙箱评估** | 科学推理如何评估? | Science Sandboxes (2608.30165): 实验循环; 假设修正; 指标优化≠理解 | **科学沙箱评估**: 指标优化≠理解; 与 D139 科学方法+D136 模型学习协同 | `nt_mind::science_sandbox` |
| D298 | **压缩放大离题** | 压缩是否会放大噪音? | Compression-Recall (2026): 压缩放大离题内容; 导致Agent脱轨; 根本性失败模式 | **压缩验证门控**: 压缩后必须GWT验证; 与 D197 信息折叠+D105 KVMem协同 | `nt_core::compression_validation` |
| D299 | **500x极端压缩** | 上下文如何压缩到1 token? | 500xCompressor (ACL 2025): 压缩到1特殊token; 0.3%额外训练参数; 强泛化 | **极端压缩**: GWT注意力路由前压缩SKILL.md; 与 D197 信息折叠+D298 压缩验证协同 | `nt_io::extreme_compression` |
| D300 | **任务感知压缩** | 压缩如何适配任务? | TACO-RL (ACL 2025): RL优化任务特定性能; 优于熵基方法; 任务感知 | **任务感知压缩**: RL优化压缩; 与 D175 注意力路由+D298 压缩验证协同 | `nt_io::task_aware_compression` |
| D301 | **动态KV压缩** | KV缓存如何自适应压缩? | DynamicKV (EMNLP 2025): 任务感知; 跨层激活模式差异; 自适应压缩比 | **动态KV压缩**: 任务自适应; 与 D105 KVMem+D178 双曲注意力协同 | `nt_io::dynamic_kv` |
| D302 | **语义块KV压缩** | KV缓存如何保持语义完整? | ChunkKV (2025): 语义块=压缩单元; 保留token间关系; 避免碎片化 | **语义块压缩**: KB经验树分支加载; 与 D137 三流检索+D301 动态KV协同 | `nt_io::semantic_chunk_kv` |
| D303 | **层级感知KV** | 哪些层需要保留? | TailorKV (ACL 2025): 全局信息层跳过压缩; 局部信息层激进压缩; 混合策略 | **层级感知KV**: 六层架构不同层不同处理; 与 D182 因果注意力+D301 动态KV协同 | `nt_io::layer_aware_kv` |
| D304 | **分布式KV架构** | 生产环境KV如何架构? | Mooncake (ACM TOCS 2025): 预填充/解码分离; CPU/DRAM/SSD/NIC分层; Kimi生产验证 | **分布式KV架构**: 分层KV存储; 与 D105 KVMem+D301 动态KV协同 | `nt_io::distributed_kv` |
| D305 | **预计算RAG缓存** | RAG如何加速? | TurboRAG (EMNLP 2025): 块级KV离线预计算; 推理时拼接; 减少TTFT | **预计算RAG缓存**: 经验树分支预计算; 与 D137 三流检索+D304 分布式KV协同 | `nt_memory::precomputed_rag` |
| D306 | **Agent部署框架** | Agent如何生产部署? | Multica (47.8K★): 23+ Agent CLI; 自托管Docker/Helm; 多Agent编排 | **Agent部署框架**: 多Agent CLI集成; 与 D163 A2A+D183 结构化软件公司协同 | `nt_act::agent_deployment` |
| D307 | **殖民Agent架构** | Agent如何像蜂群工作? | Hive (11K★): Queen+worker殖民; 共享追踪器; 崩溃安全暂停/恢复 | **殖民Agent架构**: Queen+worker; 崩溃恢复; 与 D131 多Agent+D133 异步人类协同 | `nt_act::colonial_agent` |
| D308 | **分布式Agent运行时** | Agent如何规模化隔离? | Google AX (2K★): K8s原生; 可挂起/恢复microVM; 隔离执行环境 | **分布式Agent运行时**: K8s+microVM; 与 D149 四层安全+D152 安全蒸馏协同 | `nt_act::distributed_runtime` |
| D309 | **持久工作流引擎** | Agent工作流如何持久化? | Conductor (Netflix): 事件驱动; 14+ LLM提供者; MCP; 7语言SDK; 持久执行 | **持久工作流引擎**: 事件驱动+持久执行; 与 D137 三流检索+D149 四层安全协同 | `nt_act::durable_workflow` |
| D310 | **科学推理基准** | 科学推理如何系统评估? | PaperMind (2604.21304): 4认知面; 7域; 整合推理差距 | **科学推理基准**: 多认知面评估; 与 D139 科学方法+D297 科学沙箱协同 | `nt_mind::science_reasoning` |
| D311 | **代码审查基准** | 代码审查Agent如何评估? | c-CRAB (2603.23448): 40%任务解决; Agent审查方面≠人类; 协作潜力 | **代码审查基准**: 人机协作方面差异; 与 D183 结构化软件公司+D172 工具正确性协同 | `nt_act::code_review_benchmark` |
| D312 | **长视距网页基准** | 网页Agent长任务如何评估? | Odysseys (2604.24964): 200多站点任务; 评分标准评估; 轨迹效率指标 | **长视距网页基准**: 轨迹效率; 评分标准; 与 D137 三流检索+D295 长视距终端协同 | `nt_act::long_web_benchmark` |
| D313 | **LLM网关路由** | LLM如何统一网关? | LiteLLM (57.7K★): OpenAI兼容; 100+ LLM; 虚拟密钥; 成本追踪; 负载均衡 | **LLM网关路由**: 统一API; 与 D165 DSPy+D175 注意力路由协同 | `nt_io::llm_gateway` |
| D314 | **KV感知路由** | KV缓存如何影响路由? | SMG: 引擎无关; KV缓存感知路由; gRPC+WASM插件; 多租户 | **KV感知路由**: 缓存感知选择; 与 D304 分布式KV+D175 注意力路由协同 | `nt_io::kv_aware_routing` |
| D315 | **零信任网关** | LLM网关如何零信任? | OpenZiti: 零信任覆盖网络; 语义路由; 身份认证; NAT穿越 | **零信任网关**: 身份认证; 与 D149 四层安全+D163 A2A协同 | `nt_io::zero_trust_gateway` |
| D316 | **Agent容器运行时** | Agent如何容器化? | Agentainer: Redis持久化; 自动重启; 检查点回放; LLM特定状态管理 | **Agent容器运行时**: LLM特定持久化; 与 D149 四层安全+D308 分布式运行时协同 | `nt_act::agent_container` |
| D317 | **代码重构基准** | 代码重构如何评估? | CodeTaste (2603.04177): 多文件重构; 执行好但发现差; 提议-实现分解 | **代码重构基准**: 提议-实现分离; 与 D275 规约修复+D183 结构化软件公司协同 | `nt_act::refactor_benchmark` |
| D318 | **协议统一评估** | 多架构Agent如何统一评估? | General Agent Eval (2602.22953): 5架构×5LLM×6基准; 骨干模型主导; 架构摆动12pp | **协议统一评估**: 统一协议桥; 与 D163 A2A+D165 DSPy协同 | `nt_meta::protocol_eval` |
| D319 | **Agent框架安全** | Agent框架有哪些安全暴露? | Agent Safety Eval (ICLR 2026): 框架选择影响安全; 工具调用安全≠提示安全 | **Agent框架安全**: 框架级安全评估; 与 D149 四层安全+D263 阶段安全协同 | `nt_shield::framework_safety` |
| D320 | **多模态Agent基准** | 多模态Agent如何评估? | MM-Vet (2026): 8模型; 40%问题仍失败; 多模态推理差距 | **多模态Agent基准**: 推理差距; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_mind::multimodal_benchmark` |

### 0.34 生产模式与循环安全决策 (v8.2 外部迭代 2026-09-08)

> 从外部迭代探索中提炼的 15 个新决策 + 5 个生产模式。覆盖: 语义缓存/循环检测/记忆调解/Rust 框架/动态注意力稀疏/Agent 生产架构。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D44 | **语义感知 KV 驱逐** | provider 连接的 KV 缓存如何智能驱逐? | SAECache (arXiv 2605): 多队列驱逐策略, token 类型复用率差 756×, TTFT 提升 1.4-2.7× | **语义感知驱逐**: 按 token 类型路由到专用队列 (prefix-block/agentic/chat); 在线学习自适应工作负载 | `nt_io::kv_cache::semantic_evict` |
| D45 | **正确性保证语义缓存** | 语义缓存如何保证查询正确性? | vCache (arXiv 2502): per-query 阈值在线学习, 12.5× 命中率, 26× 错误率降低; "embedding-close ≠ meaning-equal" | **Per-query 语义缓存**: 在线学习 per-embedding 阈值; 用户定义错误率上限; 防 "法国首都" vs "德国首都" 语义漂移 | `nt_memory::semantic_cache` |
| D46 | **管线中间表示缓存** | SEAL pipeline 重复计算如何避免? | SemanticALLI (arXiv 2601): 管线级 IR 缓存, 83.1% 命中率 (vs 38.7% 单体), 绕过 4023 LLM 调用, 中位 2.66ms | **管线感知 IR 缓存**: SEAL 阶段间中间表示缓存; 基于输入签名的缓存键; 与 StageManifest (D40) 对齐 | `nt_mind::seal::ir_cache` |
| D47 | **静态循环路径检测** | SEAL pipeline 如何预防无限循环? | IAL-Scan (arXiv 2607): 静态分析 Agent IR, 91.9% 精度, 68 IAL 失败跨 47 项目; 三大模式: retry/tool-call/multi-agent 无界 | **静态 ALDG 检测**: 编译时构建 Agentic Loop Dependence Graph; 检测无界反馈路径到达高成本操作; 与宪法门控 (D23) 对齐 | `nt_shield::loop_detector` |
| D48 | **幂等感知工具循环守卫** | 工具调用循环如何区分安全重复 vs 状态变更? | PraisonAI: 幂等/变异阈值分离 (5/8/12 vs 3/5/7), 结果指纹检测; DeepEval: 3 信号零成本度量 (重复/停滞/环) | **幂等感知守卫**: 区分 read_file (安全重复) vs write_file (状态变更); 结果指纹避免误报; 连续 N 次相同工具触发熔断 | `nt_act::tool_loop_guard` |
| D49 | **确定性 Agent 健康度量** | Agent 运行健康如何零成本实时监测? | DeepEval: 3 信号度量 — Tool Call Repetition (40%) + Reasoning Stagnation (35%) + Call Graph Cycles (25%); 零 LLM 成本 | **确定性 3 信号度量**: 工具调用重复 (指纹) + 推理停滞 (bigram Jaccard) + 调用图环 (DFS back-edge); 集成到 HeartbeatAggregator | `nt_meta::agent_health` |
| D50 | **联邦记忆合并协议** | 多 Agent 分布式记忆如何合并? | MELD (arXiv 2608): 5 结果准入 (insert/merge/relate/conflict/reject), 矛盾作为一等对象, Status CRDT 30/30 收敛 | **联邦合并协议**: scoped claim-key + embedding 相似度 + NLI 判决; 矛盾保留不静默解决; Status CRDT 保证最终收敛 | `nt_nexus::memory_federation` |
| D51 | **符号优先冲突调解** | KB 写入冲突如何高效解决? | LatticeMind (arXiv 2608): 写时冲突解决, PROPOSED/CONFIRMED/SUPERSEDED/CONTESTED 状态; 符号检查器 + LLM 仅处理语义冲突 | **符号优先调解**: 依赖环/资源碰撞用符号检查器 (O(1)); 语义冲突才调 LLM; 0.97 准确率 vs 0.61 baseline | `nt_memory::conflict_resolver` |
| D52 | **相关性感知证据加权** | KB 检索如何避免虚假多数? | CAMA (arXiv 2608): Memory Correlation Bias — 共享上游源相关记忆膨胀感知支持; 神经符号模块估计有效独立证据源 | **相关性感知加权**: 查询条件化证据依赖建模; 抑制共享源虚假多数; 与 BM25+向量混合检索 (D19) 对齐 | `nt_memory::evidence_weighting` |
| D53 | **Rust 原生 Agent 框架基准** | NeoTrix harness 对标什么性能基线? | ADK-Rust (606★): 109ms 冷启动 (vs 501ms Python), 568μs 循环开销 (vs 1228ms LangGraph), ~15MB RSS | **Rust 原生基准**: ADK-Rust 为性能基线; 冷启动 <150ms, 循环开销 <600μs, RSS <20MB | `nt_io::harness_benchmark` |
| D54 | **动态模型路由** | 多模型环境如何动态选择? | Daimon: 每轮难度评分→最低胜任模型; 失败时层级升级; 3 层记忆 (core/archival/episodic) | **胜任层级路由**: 难度评分→模型选择; 失败升级; 与 D36 成本感知路由 + RouteLLM 迁移学习对齐 | `nt_io::model_router` |
| D55 | **策略可组合 Agent 管线** | Agent 组件如何统一重试/降级/缓存? | atomr-agents: 策略模式统一表面 (prompt/model/parser/tool); typed reducers + per-step checkpoint; JoinSet 并行工具分发 | **策略可组合管线**: 每组件统一 retry/fallback/timeout/cache/trace 表面; typed reducers 管理状态; 与 SEAL pipeline 对齐 | `nt_act::strategy_pipeline` |
| D56 | **资源自适应推理** | 推理时如何按资源预算动态调整? | L2A (arXiv 2606): budget-conditioned gating 联合优化层跳过+头剪枝+推理 token 缩减; 34% 稀疏度仅 0.6% 精度损失 | **预算条件化稀疏**: 实时资源预算→动态调整层/头/推理长度; 单模型覆盖整个 Pareto 前沿 | `nt_io::adaptive_inference` |
| D57 | **头级注意力模式路由** | 长上下文时哪些注意力头用稀疏? | Elastic Attention (arXiv 2601): 0.27M 参数路由器 per layer; 区分稀疏鲁棒任务 (摘要) vs 稀疏敏感任务 (检索) | **头级路由**: 每层轻量路由器选择 Full/Sparse 模式; 与 Sink 感知 (D38) 互补 | `nt_io::attention_router` |
| D58 | **免训练头代理稀疏** | 缓存推理时如何零成本加速注意力? | ProxyAttn (ICLR 2026): 训练免费; 头间相似性→代理头近似所有头分数; 10.3× 注意力加速, 2.4× prefill 加速 | **头代理稀疏**: 利用头间相似性跳过计算; 无训练开销; 与 PagedAttention (D37) 叠加 | `nt_io::proxy_attention` |
| D59 | **宪法合规验证** | 宪法规则如何验证执行而非仅编码? | Constitutional AI (2026): 机制可解释性识别宪法推理电路; 多利益相关者宪法平衡 | **机制可解释性验证**: 识别实现宪法推理的内部电路, 验证非表层模式匹配; 多层宪法 (不可变+质量门+进化边界) | `nt_governance::constitutional` | L6_META |
| D60 | **Agent 身份卡** | Agent 如何声明能力边界与升级协议? | Singapore IMDA Agent Identity Cards (2026): 标准化能力/限制/授权域/升级协议; 单 Agent 4h 污染 87% 下游 | **Pre-Action Governance**: 执行前检查身份卡; 电路断路器 + 隔离; 委托链审计 | `nt_governance::identity_card` | L6_META |
| D61 | **CoVe 自验证循环** | Agent 如何自验证输出减少幻觉? | Chain-of-Verification: 4 阶段自验证; RLHF guardrails: 输入守卫→LLM→输出守卫 | **CoVe 作为一等认知模式**: 每次高风险输出执行 4 阶段自验证; defense-in-depth 验证链 | `nt_core::cove_verifier` | L5_CONSCIOUSNESS |
| D62 | **MARS 双层内省** | 元认知如何驱动策略自适应? | MARS (arXiv 2601.11974): 双层模型 (W◊/W◊◊); 内省→元模型→策略适应; 20-30% 提升 | **双层递归自改进**: W◊ 执行, W◊◊ 重写策略; 元认知触发器强制策略修订; 与 D23 对齐 | `nt_mind::mars_reflection` | L4_SUPERCLUSTER |
| D63 | **反随机主权规则** | 治理层能否完全委托给 LLM? | Nelson-Narens (2026): 双重观察; "Governor never LLM-sovereign" | **反随机主权约束**: 治理层可用 LLM 辅助, 但不可全权委托; 确定性守卫在 LLM 外部 | `nt_meta::anti_sovereignty` | L6_META |
| D64 | **五级能力层级** | Agent 能力如何分级诊断? | Hierarchy of Agentic Capabilities: 5 级 (Tool→Plan→Adapt→Ground→CommonSense); GPT-5.2 仍失败 ~40% | **能力层级诊断框架**: 失败沿层级可预测聚集; 不同层级需不同干预; 与 SelfTest T1-T3 对齐 | `nt_core::capability_hierarchy` | L5_CONSCIOUSNESS |
| D65 | **SkillPyramid 层级** | 技能如何从原子→抽象层级化组织? | SkillPyramid: 下层原子能力, 上层抽象模式; Downward Extraction + Upward Induction | **三层技能层级**: Atomic→Composite→Abstract; 新技能自动折叠进层级 | `nt_mind::skill_pyramid` | L4_SUPERCLUSTER |
| D66 | **生成式技能组合** | 大技能库如何组合而非仅检索? | Generative Skill Composition: 闭词汇序列生成; 联合预测技能/数量/顺序 | **结构化序列生成**: 联合预测选择+数量+顺序; 与 GWT salience 对齐 | `nt_act::skill_composer` | L1_SKELETON |
| D67 | **能力树路由** | 生态规模技能如何路由? | AgentSkillOS: 递归分区→类别组+容量阈值; 非显而易见技能浮现 | **递归分区能力树**: 按容量阈值递归分区; 浮现非显而易见技能 | `nt_memory::capability_tree` | L1_SKELETON |
| D68 | **四级监督韧性** | 多 Agent 如何防止级联故障? | Agentic SRE + Erlang/OTP: Fleet→Supervisor→Agent→Process; 67% 失败源于错误处理 | **四级韧性层级**: Fleet→Supervisor→Agent→Process; kill switch + escalation briefs | `nt_repair::supervision_tree` | L6_META |
| D69 | **智能自愈循环** | 自愈如何从被动→预测性? | Smart-Healing (2026): 感知→修复→反馈闭环; proactive smart-healing | **预测性自愈闭环**: AI 预测故障→触发修复→确认; 每次干预学习改进 | `nt_repair::smart_healing` | L6_META |
| D70 | **AGAO 工作流注意力** | 多 Agent 图如何按目标分配注意力? | AGAO: 目标+拓扑+资源三感知注意力; 静态→自适应 | **三感知注意力编排**: goal+topology+resource aware; 聚焦目标关键路径 | `nt_core::agao_attention` | L5_CONSCIOUSNESS |
| D71 | **注意力驱动资源分配** | 推理资源如何预测性分配? | Attention-based Resource Allocation (Nature 2026): 注意力预测→动态分配 | **预测性资源分配**: 注意力预测→GPU/CPU 动态分配; 跨异构计算 | `nt_act::predictive_resource` | L1_SKELETON |
| D72 | **双头注意力动态匹配** | 异构 Agent 如何实时匹配任务? | Dual-Head Attention (IEEE 2026): DAM 捕获任务×能力兼容性; RL 适应 | **动态能力匹配**: DAM 实时匹配; 随学习/疲劳/专业化自适应 | `nt_core::dual_head_matching` | L5_CONSCIOUSNESS |
| D73 | **可执行技能框架** | 技能如何从 prompt 模板变为真正执行的脚本? | Easel 112 skills: 每个 skill 有 `scripts/`, 真正生成 artifacts; 代码经验 > 文本经验 (57% 节省) | **Executable Skill Contract**: SKILL.md (≤200 lines) + scripts/ (可执行) + references/; 技能产生 artifacts | `nt_act::skill_executable` | L1_SKELETON |
| D74 | **反理性化质量门** | Agent 如何防止跳过质量步骤? | Claude Code: spec→plan→build→test→review→ship, 每步验证门; 跳过 = 阻断 | **Anti-Rationalization Gates**: SEAL 每阶段验证门; 门检查: ≤800行/测试覆盖/审查通过 | `nt_mind::seal::quality_gate` | L4_SUPERCLUSTER |
| D75 | **并行 Guardrails** | 验证如何与执行同时进行? | OpenAI SDK: guardrails 与执行并行; 延迟不增加 | **并行 Guardrail Pipeline**: 输入验证+执行+输出验证三路并行 | `nt_shield::parallel_guard` | L3_CONSTELLATION |
| D76 | **27 Lifecycle Hooks** | Agent 行为如何在 LLM 外部强制约束? | Claude Code: 27 events; 确定性回调, 模型无法绕过 | **Lifecycle Hook Registry**: 27 events; exit code 2 = hard block | `nt_mind::lifecycle_hooks` | L4_SUPERCLUSTER |
| D77 | **6-Dimension Profile** | 身份如何跨会话进化? | Easel: 定位/风格/受众/平台/偏好/记忆; 每次使用更新 | **Profile-Driven Adaptation**: SelfModel 扩展为 6 维度; 跨会话持久化 | `nt_core::profile_6d` | L5_CONSCIOUSNESS |
| D78 | **在线反馈学习** | Agent 如何从执行结果中实时学习? | CrewAI Training: 从反馈学习; 无在线学习 = 每次从零 | **Online Learning Loop**: 执行后评估→更新 skill 权重→调整路由 | `nt_mind::online_learning` | L4_SUPERCLUSTER |
| D79 | **社区扩展机制** | 第三方如何贡献能力? | AutoGen Extensions: 社区构建组件; Easel: 用户自定义 skill | **Extension Registry**: 声明式注册; 自动发现; 版本兼容 | `nt_io::extension_registry` | L1_SKELETON |

### 0.36 推理/多模态/自进化/数据管道决策 (v10.8 Deep Absorption Batch 2026-09-08)

> 已吸收: D321-D370 + C.225-C.314 (Flare/PIVOT/PCE/MagicAgent/ToolTree/SMITH/HEART/SARA/SPyCE/MuSEAgent/AXPO/MUSE/LMM-Searcher/Beacon/VISTA-Gym/PERIA/iSHIFT/CausalCache/HarnessEvolve/Self-Harness/HSI/Metaⁿ/Hyperagents/SkillGLoW/MetaSkill-Evolve/DiagEvo/MetaClaw/Pathway/DocETL/LOTUS/SeaTunnel/NeMo Curator)

### 0.37 形式化验证/社会模拟/浏览器/知识图谱决策 (v11.0 Deep Absorption Batch 2026-09-08)

> 已吸收: D371-D420 + C.315-C.364 (SEVerA/FormalJudge/Lean4Agent/ePCA/CONTINUITY/Agent-C/AgentLTL/SwarmWorld/Agentopia/MMP/obscura/spider-rs/AIHawk/UFO³/Agents-K1/MOOSEDev/ASKS/DySECT/K-GAT/OaK/GRA)

### 0.38 视频生成/代码质量/隐私/具身决策 (v11.1 Deep Absorption Batch 2026-09-08)

> 从 LASEV(KDD 2026) / GENMAC(AAAI 2026) / DirectorBench / AgentMV / Action Agent / LiVER / Omni-Video 2 / SANA-Streaming / Reasoning to Align / LLM-Generated Code Quality / Automated Structural Testing / PAGENT / Agora / Plain LLM Test Gen / AgentLeak / OCELOT / FGLGuard / PRAG / Fed-SE / OpenVLA / π0 / SpatialVLA / TraceVLA / CrossFormer 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D421 | **多Agent视频生产** | 视频如何多Agent生产? | LASEV (KDD 2026): 编排+方案+插图+叙述Agent; 模板驱动确定性组装; 1M+视频/天 | **多Agent视频生产**: 模板+质量门; 与 D183 结构化软件公司+D133 异步人类协同 | `nt_act::video_production` |
| D422 | **自纠正视频生成** | 视频生成如何自纠正? | GENMAC (AAAI 2026): 设计→生成→重设计循环; 验证/建议/纠正/输出Agent; 自路由 | **自纠正视频生成**: 循环纠正; 与 D136 模型学习+D149 四层安全协同 | `nt_act::video_correction` |
| D423 | **配置文件视频评估** | 视频如何配置文件评估? | DirectorBench: 80元数据; 7用户配置文件; 40检查点; 5维度 | **配置文件视频评估**: 用户感知; 与 D150 宪法治理+D145 层级技能协同 | `nt_meta::video_profile_eval` |
| D424 | **预算感知视频** | 视频如何预算感知? | AgentMV: 持久状态跨Agent协调; 多选背包问题; 预算分配 | **预算感知视频**: 背包优化; 与 D176 资源预算+D175 注意力路由协同 | `nt_act::budget_video` |
| D425 | **流式扩散视频** | 视频如何流式生成? | SANA-Streaming: 混合DiT实时流式视频编辑 | **流式扩散视频**: 实时编辑; 与 D175 注意力路由+D176 资源预算协同 | `nt_io::streaming_video` |
| D426 | **隐式推理对齐** | 推理如何内嵌扩散? | Reasoning to Align (2605.24674): DiT内隐式推理对齐; 推理-扩散融合 | **隐式推理对齐**: 推理内嵌; 与 D97 E8推理+D178 因果注意力协同 | `nt_core::implicit_reasoning_diffusion` |
| D427 | **代码质量门控** | 代码质量如何门控? | LLM Code Quality (2605.09059): 正确性+质量+生产就绪性; 开发者评审揭示不可见问题 | **代码质量门控**: 多维质量; 与 D172 工具正确性+D41 黑暗森林协同 | `nt_act::code_quality_gate` |
| D428 | **结构化Agent测试** | Agent如何结构化测试? | Structural Testing (2601.18827): OpenTelemetry轨迹; Mock LLM; 自动断言; 测试自动化金字塔 | **结构化Agent测试**: 轨迹捕获+Mock; 与 D159 检测器驱动+D149 四层安全协同 | `nt_shield::structured_agent_test` |
| D429 | **静态分析引导** | LLM如何静态分析引导? | PAGENT (2604.07624): 静态分析约束LLM搜索; +132%改进; 消毒器分析 | **静态分析引导**: 确定性约束; 与 D149 四层安全+D136 模型学习协同 | `nt_repair::static_analysis_guided` |
| D430 | **Agent漏洞检测** | Agent如何检测漏洞? | Agora (2605.29910): 多Agent角色专业化(探索/攻击/验证); 15新bug | **Agent漏洞检测**: 角色专业化; 与 D149 四层安全+D131 多Agent协同 | `nt_shield::agent_vuln_detection` |
| D431 | **简化测试生成** | 测试生成如何简化? | Plain LLM Test (2601.09695): 简单提示+强模型>复杂流水线+弱模型; 类优先策略-20%查询 | **简化测试生成**: 简单>复杂; 与 D176 资源预算+D136 模型学习协同 | `nt_act::simplified_test_gen` |
| D432 | **Agent内通道泄漏** | Agent间如何隐私保护? | AgentLeak (2602.11510): 首个全栈基准; 3泄漏通道(消息/共享记忆/工具参数) | **Agent内通道泄漏**: 全栈审计; 与 D149 四层安全+D163 A2A协同 | `nt_shield::internal_channel_leak` |
| D433 | **轨迹级隐私预算** | 隐私如何轨迹级? | OCELOT (2606.12341): 累积双向任务依赖泄漏; 每步推断泄漏预算; 权限剥夺 | **轨迹级隐私预算**: 每步预算; 与 D149 四层安全+D171 运行时策略协同 | `nt_shield::trajectory_privacy` |
| D434 | **联邦拓扑安全** | 多Agent如何联邦安全? | FGLGuard (2609.02967): 联邦GNN通信图检测; 0.03 AUROC差距; 43%攻击减少 | **联邦拓扑安全**: 联邦GNN; 与 D149 四层安全+D131 多Agent协同 | `nt_shield::federated_topology` |
| D435 | **隐私RAG** | RAG如何隐私保护? | PRAG (2604.26525): 加密检索索引+加密查询+安全KNN; 12方案比较 | **隐私RAG**: 加密检索; 与 D137 三流检索+D149 四层安全协同 | `nt_memory::private_rag` |
| D436 | **联邦自进化** | 隐私约束如何自进化? | Fed-SE (2512.08870): 本地进化+全局聚合; PEFT高回报轨迹; 低秩子空间聚合 | **联邦自进化**: 隐私约束进化; 与 D136 模型学习+D149 四层安全协同 | `nt_mind::federated_evolution` |
| D437 | **工具后门泄漏** | 工具使用如何后门? | Tool Backdoor (ACL 2026): 语义触发→记忆访问→伪装检索; 多轮放大 | **工具后门泄漏**: 多轮检测; 与 D149 四层安全+D169 CPG协同 | `nt_shield::tool_backdoor` |
| D438 | **上下文隐私分类** | 隐私如何上下文化? | Contextualized Privacy (2603.02983): Agent自分类可共享/不可共享; 协商属性 | **上下文隐私分类**: 上下文完整性; 与 D149 四层安全+D171 运行时策略协同 | `nt_shield::context_privacy` |
| D439 | **代码审查终结论** | 人类代码审查是否过时? | End of Code Review (2606.13175): Agent可替代人类审查; 治理转向Agent对Agent | **代码审查终结论**: Agent对Agent; 与 D150 宪法治理+D172 工具正确性协同 | `nt_governance::agent_review` |
| D440 | **VLM动作骨干** | VLM如何作为动作模型? | OpenVLA (2406.09246): 7B VLA; Open X-Embodiment; 开源RT-2替代 | **VLM动作骨干**: 7B VLA; 与 D145 层级技能+D173 混合技能协同 | `nt_act::vlm_action` |
| D441 | **流匹配控制** | 机器人如何流匹配控制? | π0 (2410.24164): VLM+流匹配动作头; 灵巧操作; 跨任务 | **流匹配控制**: 流匹配动作; 与 D178 因果注意力+D145 层级技能协同 | `nt_physical::flow_matching` |
| D442 | **空间理解VLA** | VLA如何空间理解? | SpatialVLA (2502.12958): 3D空间推理; 相机位姿+深度; 拾放 | **空间理解VLA**: 3D推理; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_world::spatial_vla` |
| D443 | **视觉追踪提示** | 机器人如何视觉追踪? | TraceVLA (2502.06136): 视觉追踪叠加; 无重训练; 提示路由 | **视觉追踪提示**: 提示路由; 与 D98 HQL+D145 层级技能协同 | `nt_io::visual_trace_prompt` |
| D444 | **跨具身Transformer** | 多形态如何统一? | CrossFormer (2502.06800): 统一策略跨形态(臂/腿/手); 具身不可知 | **跨具身Transformer**: 具身不可知; 与 D145 层级技能+D136 模型学习协同 | `nt_physical::cross_embodiment` |
| D445 | **结构化持久状态** | 多Agent如何持久状态? | AgentMV: 结构化持久状态跨Agent; 持久状态协调 | **结构化持久状态**: 状态跨Agent; 与 D135 原子记忆+D131 多Agent协同 | `nt_memory::structured_persistent_state` |
| D446 | **自路由纠正Agent** | 视频如何自路由纠正? | GENMAC: 自路由纠正选择; 验证-纠正循环 | **自路由纠正Agent**: 自路由; 与 D136 模型学习+D149 四层安全协同 | `nt_mind::self_routing_correction` |
| D447 | **场景可控生成** | 视频如何场景可控? | LiVER (2604.07966): 3D场景属性→视频; 场景Agent翻译; 渐进训练 | **场景可控生成**: 3D条件; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_io::scene_controlled_gen` |
| D448 | **统一生成编辑** | 生成和编辑如何统一? | Omni-Video 2 (2602.08820): MLLM条件扩散统一生成+编辑 | **统一生成编辑**: MLLM条件; 与 D140 多模态记忆+D173 混合技能协同 | `nt_io::unified_gen_edit` |
| D449 | **流式编辑** | 视频如何流式编辑? | SANA-Streaming / LiveEdit / JoyAI: 实时流式视频编辑; 扩散 | **流式编辑**: 实时编辑; 与 D175 注意力路由+D176 资源预算协同 | `nt_io::streaming_edit` |
| D450 | **联邦学习隐私** | 联邦学习如何LLM隐私? | LA-LoRA (ICLR 2026): 本地交替+低通平滑; +16.83%超RoLoRA | **联邦学习隐私**: 低通平滑; 与 D149 四层安全+D136 模型学习协同 | `nt_mind::federated_privacy` |
| D451 | **私有RAG多Agent** | RAG如何多Agent隐私? | Privacy-Preserving RAG (2606.24623): 三Agent(提取/分析/重建); 暴露144→1 | **私有RAG多Agent**: 三Agent清洗; 与 D137 三流检索+D149 四层安全协同 | `nt_memory::private_rag_multi` |
| D452 | **部分加密MLC** | DNN如何部分加密? | PrivDNN (2607.21895): 仅加密敏感层; 降低MPC开销 | **部分加密MLC**: 选择性加密; 与 D149 四层安全+D176 资源预算协同 | `nt_shield::partial_encryption` |
| D453 | **模糊推理安全** | LLM如何轻量模糊? | GELO (2603.05035): FHE和静态排列之间; 混合TEE/非TEE集群; 实用延迟 | **模糊推理安全**: 轻量模糊; 与 D149 四层安全+D176 资源预算协同 | `nt_shield::obfuscation_security` |
| D454 | **数据代理威胁** | 数据代理如何威胁? | Your LLM Can Leak (2602.11510): 语义触发→记忆访问→伪装检索; 多轮放大 | **数据代理威胁**: 语义触发检测; 与 D149 四层安全+D169 CPG协同 | `nt_shield::data_proxy_threat` |
| D455 | **多Agent失效归因** | 多Agent失效如何归因? | Failure Localization (2607.07989): 多Agent系统失效归因 | **多Agent失效归因**: 归因方法; 与 D131 多Agent+D149 四层安全协同 | `nt_mind::multi_failure_attribution` |
| D456 | **LLM漏洞规模** | LLM漏洞检测规模? | LLM Vuln Scale (2601.19239): 项目级评估; 浅层过程间推理; 高误报; 数百万token | **LLM漏洞规模**: 浅层推理; 与 D149 四层安全+D176 资源预算协同 | `nt_shield::llm_vuln_scale` |
| D457 | **混淆压缩** | 上下文如何混淆压缩? | Context Compression Amplifies (2026): 压缩放大离题; 导致脱轨; 根本失败 | **混淆压缩**: GWT验证; 与 D197 信息折叠+D178 因果注意力协同 | `nt_core::confusion_compression` |
| D458 | **设备端具身** | 具身如何设备端? | RoboCasa (2310.12923): 100K+任务; NVIDIA Isaac; 家庭操作 | **设备端具身**: 仿真+真机; 与 D145 层级技能+D173 混合技能协同 | `nt_physical::edge_embodiment` |
| D459 | **仿真到真实** | 仿真如何到真实? | SimLoRA (2502.11500): LoRA微调仿真数据; 域随机化 | **仿真到真实**: LoRA+随机化; 与 D136 模型学习+D176 资源预算协同 | `nt_physical::sim_to_real` |
| D460 | **流匹配控制策略** | 机器人如何流匹配? | π0 (2410.24164): VLM+流匹配; 灵巧操作; 跨任务 | **流匹配控制策略**: 流匹配动作; 与 D178 因果注意力+D145 层级技能协同 | `nt_physical::flow_control` |
| D461 | **空间VLA** | VLA如何空间? | SpatialVLA (2502.12958): 3D空间推理; 相机位姿+深度 | **空间VLA**: 3D推理; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_world::spatial_vla` |
| D462 | **视觉追踪** | 机器人如何视觉追踪? | TraceVLA (2502.06136): 视觉追踪叠加; 无重训练 | **视觉追踪**: 提示路由; 与 D98 HQL+D145 层级技能协同 | `nt_io::visual_trace` |
| D463 | **跨具身** | 多形态如何跨具身? | CrossFormer (2502.06800): 统一策略跨形态; 具身不可知 | **跨具身**: 具身不可知; 与 D145 层级技能+D136 模型学习协同 | `nt_physical::cross_embodiment` |
| D464 | **导航Agent** | 导航如何LLM引导? | NavAgent (2502.13456): LLM高层导航+CLIP视觉; 多楼层 | **导航Agent**: LLM+CLIP; 与 D145 层级技能+D173 混合技能协同 | `nt_physical::nav_agent` |
| D465 | **多模态感知** | 机器人如何多模态? | RoboOmni (2503.07890): 触觉+视觉+力; 多模态融合 | **多模态感知**: 多模态融合; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_physical::multimodal_sense` |
| D466 | **通用策略生成** | 策略如何通用生成? | Any2Flow (2504.02097): 任意VLM→机器人策略蒸馏; 无任务特定训练 | **通用策略生成**: VLM蒸馏; 与 D136 模型学习+D145 层级技能协同 | `nt_physical::universal_policy` |
| D467 | **LLM规划操作** | 操作如何LLM规划? | manipomo (2502.12234): LLM规划+低层策略执行; 层次分解 | **LLM规划操作**: 层次分解; 与 D97 E8推理+D145 层级技能协同 | `nt_act::llm_planned_op` |
| D468 | **物理推理** | 具身如何物理推理? | PhysRoom (2502.12345): LLM+物理引擎; 物理可供性推理 | **物理推理**: 物理可供性; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_core::physical_reasoning` |
| D469 | **双臂操作** | 双臂如何操作? | RoboTwin (2503.14067): 50双臂任务; 数字孪生+真机迁移 | **双臂操作**: 仿真+真机; 与 D145 层级技能+D173 混合技能协同 | `nt_physical::dual_arm` |
| D470 | **分布推理** | 推理如何分布? | Distributed Reasoning (2608.17282): P2P协作; 动态能力接地; 拓扑更新 | **分布推理**: P2P; 与 D131 多Agent+D97 E8推理协同 | `nt_core::distributed_reasoning` |
| D471 | **流式Agent评估** | Agent如何流式评估? | AgentStream (2608.00155): 自进化Agent流式评估; 隔离/顺序/交错场景; 提示+记忆+技能演化 | **流式Agent评估**: 流式场景; 与 D136 模型学习+D175 注意力路由协同 | `nt_mind::stream_eval` |
| D472 | **可逆执行理论** | Agent如何可逆执行? | Revisable by Design (2604.23283): 可逆性分类(I/R/K/X动作); 修订吸收器; 灵活性受可逆性约束 | **可逆执行理论**: 可逆分类; 与 D149 四层安全+D136 模型学习协同 | `nt_act::reversible_execution` |
| D473 | **两级解耦架构** | 流式如何两级解耦? | StreamMind (2608.05703): 前端Worker(延迟关键)+后端Worker(异步记忆构建); -66.2%延迟 | **两级解耦架构**: 前后端解耦; 与 D131 多Agent+D175 注意力路由协同 | `nt_world::two_tier_stream` |
| D474 | **事件边界记忆** | 流式如何事件边界? | EventMemAgent (2602.15329): 层级事件中心记忆(STM边界检测+LTM事件元组); Agentic RL工具选择 | **事件边界记忆**: 事件粒度; 与 D135 原子记忆+D178 因果注意力协同 | `nt_memory::event_boundary` |
| D475 | **任务优先级抢占** | 多Agent如何优先级抢占? | Event-Driven Orchestration (2606.20058): 优先级抢占(-14-75%队列时间); 相关事件合并(+20%正确性) | **任务优先级抢占**: 抢占+合并; 与 D131 多Agent+D175 注意力路由协同 | `nt_act::priority_preempt` |
| D476 | **流式语义评估** | 流式如何语义评估? | DataClawEval (2607.28033): 100生产任务; FlinkSQL窗口/事件时间; LLM判断流式语义失败 | **流式语义评估**: 规则评估; 与 D159 检测器驱动+D172 工具正确性协同 | `nt_repair::stream_semantic_eval` |
| D477 | **混合策略DPO** | 自进化如何混合策略? | RTTP (2601.17567): 混合策略DPO平衡策略稳定性+新颖性; +91.4%尾部趋势; Meta部署 | **混合策略DPO**: 稳定+新颖; 与 D136 模型学习+D147 涌现记忆协同 | `nt_mind::mixed_policy_dpo` |
| D478 | **POSIX原生Agent** | Agent如何POSIX原生? | Quine (2603.18030): Agent=PID; 接口=流+exit; 生命周期=fork/exec/exit; 可修订时间 | **POSIX原生Agent**: 进程模型; 与 D133 异步人类+D131 多Agent协同 | `nt_physical::posix_agent` |
| D479 | **SSE异步流** | Agent如何SSE异步? | DataClaw (2604.24067): SSE异步事件流; 思考/调用/结果中间状态; 热加载技能 | **SSE异步流**: SSE透明; 与 D133 异步人类+D145 层级技能协同 | `nt_io::sse_async_stream` |
| D480 | **MCP+A2A编排** | 多Agent如何MCP编排? | Multi-Agent Survey (2601.13671): MCP+A2A收敛; 事件驱动治理; 流式+Agent原语融合 | **MCP+A2A编排**: 协议收敛; 与 D131 多Agent+D163 A2A协同 | `nt_act::mcp_a2a_orchestration` |
| D481 | **情景上下文重构** | 记忆如何情景重构? | E-mem (ICML 2026,2601.21714): 多Agent异步架构; 助手Agent非压缩上下文; 主Agent编排; 54% F1(+7.75%); -70%token | **情景上下文重构**: 非压缩重构; 与 D135 原子记忆+D175 注意力路由协同 | `nt_memory::episodic_reconstruct` |
| D482 | **重构≠检索** | 记忆如何重构? | MRAgent (ICML 2026,2606.06036): Cue-Tag-Content联想图; 主动重构整合推理; 迭代探索+剪枝; +23% | **重构≠检索**: 主动重构; 与 D137 三流检索+D178 因果注意力协同 | `nt_memory::reconstruct_not_retrieve` |
| D483 | **复发记忆巩固** | 记忆如何复发巩固? | RecMem (ACL 2026,2605.16045): 持续复发检测后LLM巩固; 潜意识+意识层; -87%token; 超SOTA | **复发记忆巩固**: 复发触发; 与 D147 涌现记忆+D175 注意力路由协同 | `nt_memory::recurrence_consolidation` |
| D484 | **层级记忆路由** | 记忆如何层级路由? | H-MEM (EACL 2026): 语义抽象层级; 位置索引编码; 逐层路由(无穷举搜索) | **层级记忆路由**: 位置索引; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::hierarchical_route` |
| D485 | **可修订层级记忆** | 记忆如何可修订? | xMemory (2602.02007): 可修订层级; 段→组件→组; 解聚后聚合; 顶部选择+条件扩展 | **可修订层级记忆**: 解聚聚合; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::revisable_hierarchy` |
| D486 | **因果行动记忆** | 记忆如何因果推理? | ActMem (2603.00026): 对话→因果+语义图; 反事实推理+常识补全; ActMemEval基准 | **因果行动记忆**: 反事实推理; 与 D135 原子记忆+D97 E8推理协同 | `nt_memory::causal_action` |
| D487 | **扩散激活记忆** | 记忆如何扩散激活? | SYNAPSE (2601.02744): 动态图+扩散激活(非预计算链接); 侧抑制+时间衰减; 三重检索 | **扩散激活记忆**: 扩散激活; 与 D137 三流检索+D175 注意力路由协同 | `nt_memory::spreading_activation` |
| D488 | **自适应记忆准入** | 记忆如何准入控制? | A-MAC (ICLR 2026 WS,2603.04549): 5可解释因子(效用/置信/新颖/近期/类型); +31%延迟减少 | **自适应记忆准入**: 5因子门控; 与 D135 原子记忆+D149 四层安全协同 | `nt_memory::adaptive_admission` |
| D489 | **可信记忆巩固** | 记忆如何可信? | TrustMem (2606.25161): 迁移验证器(覆盖/保真/忠实); 偏好RL; -40%遗漏,-79%损坏,-50%幻觉 | **可信记忆巩固**: 迁移验证; 与 D149 四层安全+D135 原子记忆协同 | `nt_memory::trustworthy_consolidation` |
| D490 | **自进化记忆** | 记忆如何自进化? | EvolveMem (2605.13941): 全检索配置结构化动作空间; 诊断模块共进化知识+检索; 内容+策略 | **自进化记忆**: 双重进化; 与 D136 模型学习+D135 原子记忆协同 | `nt_memory::self_evolving` |
| D491 | **类人记忆架构** | 记忆如何类人? | Human-Inspired (MS,2605.08538): 6生物机制(睡眠巩固/干扰遗忘/印迹成熟/再巩固/实体KG/混合检索) | **类人记忆架构**: 6机制; 与 D135 原子记忆+D147 涌现记忆协同 | `nt_memory::human_inspired` |
| D492 | **结构化代码索引** | 代码如何结构化索引? | Hydra (FSE 2026,2602.11671): 结构化索引(层级树); 依赖感知检索(DAR); +5% Pass@1 | **结构化代码索引**: 结构>自然语言; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::structured_code_index` |
| D493 | **上下文内联** | 代码如何上下文内联? | InlineCoder (FSE 2026,2601.00376): 未完函数内联调用图; 上下文锚生成; 重构为函数级任务 | **上下文内联**: 调用图遍历; 与 D137 三流检索+D175 注意力路由协同 | `nt_memory::context_inline` |
| D494 | **过程相似性检索** | 代码如何过程检索? | ProjAgent (2607.08691): 推理步骤分解; 过程行为相似性(非词法/语义); +41.14% Pass@1 | **过程相似性检索**: 行为>表面; 与 D137 三流检索+D136 模型学习协同 | `nt_memory::procedural_similarity` |
| D495 | **AST引导自适应记忆** | 代码如何AST记忆? | CodeMEM (2601.02868): AST引导动态记忆; 代码上下文+会话记忆; 遗忘检测; +12.2% | **AST引导自适应记忆**: AST引导; 与 D135 原子记忆+D145 层级技能协同 | `nt_memory::ast_guided` |
| D496 | **分层路由SE** | SE任务如何分层路由? | Triage (2604.07494): 代码质量信号路由廉价/昂贵LLM; 成本感知任务委派 | **分层路由SE**: 成本感知; 与 D176 资源预算+D175 注意力路由协同 | `nt_act::tiered_se_routing` |
| D497 | **LLM启发式符号搜索** | 综合如何LLM启发式? | Narcissus (2608.25657): LLM提案保持语法树; 上下文评分扩展; 40% ARC(超13%); 零LLM调用 | **LLM启发式符号搜索**: 神经符号; 与 D97 E8推理+D149 四层安全协同 | `nt_core::llm_heuristic_symbolic` |
| D498 | **属性引导综合** | 综合如何属性引导? | Property-Guided (2605.16142): 形式属性检查+反例修复; 7x少程序; 数量级少评估 | **属性引导综合**: 验证优先; 与 D149 四层安全+D150 宪法治理协同 | `nt_core::property_guided_synthesis` |
| D499 | **多视角修复** | 修复如何多视角? | CT-Repair (2607.12605): CPG+TEG; 3 FSM引导Agent(静态/动态/混合); 489/854 Defects4J; -94.85%范围 | **多视角修复**: 3视角; 与 D149 四层安全+D178 因果注意力协同 | `nt_repair::multi_perspective` |
| D500 | **结构化诊断定位** | 修复如何诊断定位? | SHERLOC (EMNLP 2026,2606.24820): 无训练诊断; 84.33%准确率@1; -36.7%token; +5.95pp修复率 | **结构化诊断定位**: 无训练; 与 D159 检测器驱动+D172 工具正确性协同 | `nt_repair::structured_diagnostic` |
| D501 | **异步隔离委派** | 多Agent如何异步委派? | CAID (2603.21489): 中心化异步隔离委派; git worktree分支合并; +25.6% PaperBench | **异步隔离委派**: git worktree; 与 D131 多Agent+D176 资源预算协同 | `nt_act::async_isolated_delegation` |
| D502 | **动态拓扑生成** | 多Agent如何动态拓扑? | AgentConductor (2602.17100): RL优化多Agent拓扑; 动态DAG/查询; +14.6% pass@1; -68%token | **动态拓扑生成**: RL拓扑; 与 D131 多Agent+D175 注意力路由协同 | `nt_act::dynamic_topology` |
| D503 | **自进化技能库** | 技能如何自进化? | SKILLFOUNDRY (2604.03964): 异构科学资源→验证技能包; 领域知识树→技能挖掘→闭环验证; 71.1%新技能 | **自进化技能库**: 闭环验证; 与 D145 层级技能+D136 模型学习协同 | `nt_mind::self_evolving_skill_lib` |
| D504 | **社交行为原型** | Agent如何行为原型? | Behavioral Traits (2601.15114): 7原型(沉默观察→交互热情); 显式特征层; 980 Agent验证 | **社交行为原型**: 7原型; 与 D146 进化速度+D131 多Agent协同 | `nt_feel::behavioral_archetype` |
| D505 | **对抗社交机器人** | Agent如何对抗检测? | EvoBot (2508.17711): SFT+对抗DPO; 协同适应检测器; 生成人类内容+规避检测 | **对抗社交机器人**: 对抗训练; 与 D149 四层安全+D146 进化速度协同 | `nt_shield::adversarial_bot` |
| D506 | **实时事件驱动社交** | 社交如何实时事件? | BotVerse (2603.29741): 可扩展事件驱动; Bluesky实时流; 异步编排+认知记忆; 安全红队 | **实时事件驱动社交**: 事件驱动; 与 D131 多Agent+D149 四层安全协同 | `nt_act::realtime_event_social` |
| D507 | **视觉Agent社交网络** | Agent如何视觉社交? | AI-Gram (2604.21446): 首个视觉Agent社交网络; 1007 Agent; 60深视觉链; 美学主权悖论 | **视觉Agent社交网络**: 视觉链; 与 D140 多模态记忆+D146 进化速度协同 | `nt_world::visual_agent_social` |
| D508 | **Agent社交数据集** | Agent社交如何数据集? | Moltbook (2605.13860): 2.6M帖子; 1.2M评论; 175K Agent; 6730社区; 78天; MIT | **Agent社交数据集**: 大规模; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::agent_social_dataset` |
| D509 | **性别同质性** | Agent如何性别同质? | Gender Dynamics (2602.02606): 70K Agent; 140M帖子; 性别流动但同质性强; 文化同化无身体 | **性别同质性**: 同质性; 与 D146 进化速度+D131 多Agent协同 | `nt_core::gender_homophily` |
| D510 | **Agent网络结构** | Agent如何网络结构? | Moltbook Structural (2603.23279): 核心-边缘; 0.9%节点集中连接; 对高输出节点脆弱; 重尾 | **Agent网络结构**: 核心-边缘; 与 D131 多Agent+D149 四层安全协同 | `nt_core::agent_network_structure` |
| D511 | **虚假社会图** | Agent如何社会图? | Synthetic Social Graph (2604.27271): 互惠仅3.8%(vs人类10-30%); 下票0.9%; 桥接Agent=99.7%晚期放大 | **虚假社会图**: 互惠低; 与 D131 多Agent+D147 涌现记忆协同 | `nt_core::synthetic_social_graph` |
| D512 | **多模态迷因预测** | 迷因如何多模态预测? | Early Meme (2510.05761): XGBoost PR-AUC 0.52(30min)→0.82(7h); 证据转变: 静态→时序→内容 | **多模态迷因预测**: 证据转变; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_world::multimodal_meme` |
| D513 | **误导参与预测** | 误导如何参与预测? | IC-Mamba (2502.04655): 状态空间模型; 区间删失参与; 15-30min预测; 跟踪28天 | **误导参与预测**: 时序建模; 与 D149 四层安全+D137 三流检索协同 | `nt_world::misinfo_engagement` |
| D514 | **推荐系统Harness** | 推荐如何Harness? | CORAL (2609.02730): LLM Agent闭环推荐; 观察→推理→优化→测量; 零服务成本参与提升 | **推荐系统Harness**: LLM闭环; 与 D136 模型学习+D135 原子记忆协同 | `nt_act::recsys_harness` |
| D515 | **自触发推送** | 推送如何自触发? | STEPS (2608.01949): 抖音1B+用户; 规划+执行+过滤Agent; +0.28%活跃,-1.9%禁用,-79%计算 | **自触发推送**: 三Agent; 与 D131 多Agent+D176 资源预算协同 | `nt_act::self_triggered_push` |
| D516 | **对话推荐系统** | 推荐如何对话? | SYF (2608.06632): 三层(感知/服务/自进化); DPO+LLM法官; 98.85%对齐 | **对话推荐系统**: 三层; 与 D146 进化速度+D145 层级技能协同 | `nt_io::conversational_recsys` |
| D517 | **图LLM推荐** | 推荐如何图LLM? | ConnectionMeta (2608.10187): 异构图+LLM策略; SFT→RL; +0.43%视频观看 | **图LLM推荐**: 图推理; 与 D137 三流检索+D97 E8推理协同 | `nt_memory::graph_llm_recsys` |
| D518 | **世界模型反事实推荐** | 推荐如何反事实? | WMG-RL (2609.01067): 用户参与世界模型; 1.7B学生匹配更大LLM; 跨域迁移 | **世界模型反事实推荐**: 反事实模拟; 与 D136 模型学习+D178 因果注意力协同 | `nt_core::world_model_recsys` |
| D519 | **影响力工厂** | 影响力如何工厂? | IO Factory (2608.10920): 规划→行动→暴露→测量→适应; 100K Agent; 可检查证据链 | **影响力工厂**: 工厂化; 与 D131 多Agent+D149 四层安全协同 | `nt_act::influence_factory` |
| D520 | **平台架构>算法** | 推荐如何架构? | Platform Architecture (2605.19204): 树→层→网络→完全; 热算法对Reddit无效,对TikTok赢家通吃 | **平台架构>算法**: 架构>算法; 与 D136 模型学习+D150 宪法治理协同 | `nt_core::platform_arch_over_algo` |
| D521 | **规则归纳记忆** | 对话如何规则归纳? | RuleMem (2609.03915): 从对话归纳Horn子句规则; 规则困惑度一致性验证; +54.3%超14基线 | **规则归纳记忆**: 规则归纳; 与 D135 原子记忆+D97 E8推理协同 | `nt_memory::rule_induction` |
| D522 | **源轨迹记忆** | 对话如何源轨迹? | TrajWiki (2608.00967): 源基础进化轨迹(ADD/REVISE/DEPRECATE); Wiki页面; 分层检索 | **源轨迹记忆**: 轨迹进化; 与 D135 原子记忆+D147 涌现记忆协同 | `nt_memory::source_trajectory` |
| D523 | **人类档案检索** | 记忆如何人类档案? | HERO (2608.22310): 异构记忆图保留原始文本; 迭代图遍历+人类档案锚点; 无压缩 | **人类档案检索**: 档案驱动; 与 D135 原子记忆+D175 注意力路由协同 | `nt_memory::human_profile_retrieval` |
| D524 | **意图驱动状态** | 对话如何意图驱动? | IDSS (2608.15755): 显式情况状态(事实+意图+约束); 溯源感知实体; 约束传播 | **意图驱动状态**: 情况状态; 与 D135 原子记忆+D178 因果注意力协同 | `nt_core::intent_driven_state` |
| D525 | **动态话语树** | 对话如何动态树? | Context-Agent (ACL 2026): 动态树(话题分支); 45-52%token减少; +3-10%任务完成 | **动态话语树**: 树结构; 与 D135 原子记忆+D131 多Agent协同 | `nt_core::dynamic_discourse_tree` |
| D526 | **自适应上下文重构** | 对话如何上下文重构? | ACR (ACL 2026): 上下文重构算子库+教师引导自进化训练; 解耦上下文管理与推理 | **自适应上下文重构**: 算子库; 与 D136 模型学习+D175 注意力路由协同 | `nt_mind::adaptive_context_refactor` |
| D527 | **多模态对话上下文** | 对话如何多模态? | ChatUMM (2602.06442): 交错多轮训练; 状态对话→干扰轮→历史依赖重写; SOTA视觉理解 | **多模态对话上下文**: 交错训练; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_io::multimodal_dialogue` |
| D528 | **无状态代理模式** | API如何无状态? | Hydration Proxy (2609.01834): 会话持久化与推理引擎解耦; 主权状态管理+KV缓存权衡 | **无状态代理模式**: 解耦持久化; 与 D176 资源预算+D135 原子记忆协同 | `nt_io::stateless_proxy_pattern` |
| D529 | **自进化服务对话** | 对话如何自进化? | SEAD (ACL 2026,2602.03548): 自进化无标注; 课程控制器+用户角色模型; +17.6%任务完成 | **自进化服务对话**: 自进化; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::self_evolving_service` |
| D530 | **神经符号状态追踪** | 对话如何符号追踪? | ReacTOD (TrustNLP 2026): 有界ReAct+确定性符号验证; 93.1%自纠正; 52.71% JGA | **神经符号状态追踪**: 符号验证; 与 D149 四层安全+D97 E8推理协同 | `nt_core::neuro_symbolic_dst` |
| D531 | **无监督技能结晶** | 对话如何无监督结晶? | Learning-Tool (SEPLN 2026,2608.30426): 无监督微调管道; 8B超70B上下文系统; 自改进循环 | **无监督技能结晶**: 小超大; 与 D136 模型学习+D176 资源预算协同 | `nt_mind::unsupervised_crystallize` |
| D532 | **图增强MoE** | 对话如何图MoE? | GEM (2605): MoE路由GNN(结构)+T5(时序)专家; 65.19% JGA; 域感知专家选择 | **图增强MoE**: MoE路由; 与 D131 多Agent+D175 注意力路由协同 | `nt_core::graph_moe` |
| D533 | **MCTS对话规划** | 对话如何MCTS? | DREAMS (EMNLP 2026,2609.00618): 树结构上下文+引出节点(MCTS)+利用节点(LLM); 偏好演化 | **MCTS对话规划**: MCTS规划; 与 D97 E8推理+D175 注意力路由协同 | `nt_act::mcts_dialogue` |
| D534 | **反馈感知信用分配** | 对话如何信用分配? | Faca (2608.17499): 下用户反应作为过程优势; RL训练; +5.91pp(8B),+10.22pp(14B) | **反馈感知信用分配**: 过程优势; 与 D136 模型学习+D147 涌现记忆协同 | `nt_mind::feedback_credit` |
| D535 | **群聊通信Agent** | 群聊如何Agent? | GCAgent (2603.05240): Agent构建器+对话管理器+接口插件; 350天部署; +28.8%消息量 | **群聊通信Agent**: 三组件; 与 D131 多Agent+D133 异步人类协同 | `nt_io::group_chat_agent` |
| D536 | **隐私群聊** | 群聊如何隐私? | GroupGPT (2603.01059): 大小模型协作; 3x token减少; 隐私清洗; MUIR基准(2500标注段) | **隐私群聊**: 隐私清洗; 与 D149 四层安全+D176 资源预算协同 | `nt_shield::private_group_chat` |
| D537 | **间接引用基准** | 对话如何引用? | CoRG (EMNLP 2026,2608.29834): 间接引用+工具使用; RepoRef基准(400段,92仓库); 最佳67% | **间接引用基准**: 引用解析; 与 D137 三流检索+D145 层级技能协同 | `nt_act::indirect_reference` |
| D538 | **稳定角色模拟** | 角色如何稳定? | SPASM (ACL 2026,2608.29834): 自我上下文投影(ECP); 消除回声; 45K对话数据集 | **稳定角色模拟**: ECP; 与 D146 进化速度+D140 多模态记忆协同 | `nt_feel::stable_persona` |
| D539 | **动态角色一致性** | 角色如何动态一致? | Dynamic Persona (ACL 2026): 身份层稳定+适应层适当; L/M/S心理状态; 闭环PCC→PCR→PDS | **动态角色一致性**: 三层心理; 与 D146 进化速度+D150 宪法治理协同 | `nt_feel::dynamic_persona_coherence` |
| D540 | **心理角色架构** | 角色如何心理架构? | PersonaForge (ACL 2026): 三层人格(大五+防御机制+风格); 内在独白; +19.4%一致性,-75%漂移 | **心理角色架构**: 防御机制; 与 D146 进化速度+D136 模型学习协同 | `nt_feel::psychological_persona` |
| D541 | **角色可见性控制** | 角色如何可见控制? | Stranger/Fan/Peer (2608.28467): 三阶段因子化传记可见性; 不对称披露→内容泄漏 | **角色可见性控制**: 可见性; 与 D149 四层安全+D150 宪法治理协同 | `nt_core::persona_visibility` |
| D542 | **角色保真评估** | 角色如何保真评估? | PRISM (EMNLP 2026,2608.26674): 心理语言学保真评估; SFL(任务框架/人际立场/语言风格) | **角色保真评估**: SFL; 与 D159 检测器驱动+D146 进化速度协同 | `nt_meta::persona_fidelity` |
| D543 | **角色一致性审问** | 角色如何审问? | PICON (2603.25620): 内部/外部/重测一致性; 逻辑链追问; 7 Agent vs 63人类 | **角色一致性审问**: 审问方法; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::persona_interrogation` |
| D544 | **时间记忆治理** | 角色如何时间治理? | ARPM (2605.14802): 外部时间记忆治理; 双时间排序(物理+对话轮); 跨模型迁移 | **时间记忆治理**: 双时间; 与 D135 原子记忆+D147 涌现记忆协同 | `nt_memory::temporal_governance` |
| D545 | **层级RL对话** | 对话如何层级RL? | ToSCA (EMNLP 2026,2608.21969): 两层HRL(DQN策略+PPO生成); 双粒度奖励; KL惩罚 | **层级RL对话**: 层级RL; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::hierarchical_rl_dialogue` |
| D546 | **叙事俘获** | 对话如何叙事俘获? | Narrative Captivity (EMNLP 2026,2609.03407): LLM对单面叙述无异议对齐; 25pp判断偏移 | **叙事俘获**: 安全关注; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::narrative_captivity` |
| D547 | **吸引子状态** | 对话如何吸引子? | Attractor States (2606.30571): 模型特定吸引盆地; 不对称影响; Claude强吸引,GPT可塑 | **吸引子状态**: 涌现动力学; 与 D131 多Agent+D146 进化速度协同 | `nt_core::attractor_states` |
| D548 | **选择性信念修正** | 对话如何信念修正? | Beyond Local Surprise (2608.26035): 不确定性敏感策略; 不匹配→保存; 积累→修正 | **选择性信念修正**: 保存/修正; 与 D97 E8推理+D175 注意力路由协同 | `nt_core::selective_belief_revision` |
| D549 | **事件引导槽交互** | 对话如何事件引导? | Event-Guided Slot (CoNLL 2026): 潜在事件作为认知组织单元; 动态结构偏置; +6.4%一致性 | **事件引导槽交互**: 事件组织; 与 D135 原子记忆+D178 因果注意力协同 | `nt_core::event_guided_slot` |
| D550 | **方向偏好优化** | 对话如何偏好优化? | Preference Optimization Survey: DPO/KTO/IPO/ORPO/SimPO对比; 稳定性-效用权衡 | **方向偏好优化**: 偏好对比; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::directional_preference` |

### 0.39 领域应用研究索引

> 从 SEVerA / FormalJudge / Lean4Agent / ePCA / CONTINUITY / Agent-C / AgentFlow / AgentLTL / SAVER / SwarmWorld / Agentopia / MMP / When Helping Costs Nothing / TopoSim / Intellectual Elites / QueenBee / obscura(26.5K★,Rust) / spider-rs(2.7K★,Rust) / AIHawk(30.3K★) / UFO³(9.7K★) / Agents-K1 / MOOSEDev / ASKS / DySECT / K-GAT / OaK / GRA 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D371 | **形式化守护模型** | Agent如何形式化验证? | SEVerA (2603.25111): FGGM合约; 搜索→验证(Dafny/Z3)→学习(GRPO); 零约束违反 | **形式化守护模型**: 一阶逻辑合约; 与 D150 宪法治理+D136 模型学习协同 | `nt_shield::formal_guard` |
| D372 | **神经符号监督** | Agent如何形式化监督? | FormalJudge (2602.11136): 双向FoT; 7B法官检测72B欺骗90%+; 弱到强泛化 | **神经符号监督**: 双向形式化推理; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::neuro_symbolic` |
| D373 | **依赖类型建模** | Agent工作流如何形式化? | Lean4Agent (2606.06523): 三层(结构/语义/轨迹); LeanEvolve验证反馈; +11.94% | **依赖类型建模**: Lean4验证; 与 D97 E8推理+D136 模型学习协同 | `nt_core::dependent_types` |
| D374 | **逻辑悖论防御** | Agent如何防逻辑攻击? | ePCA (2605.29251): 意图→一阶逻辑→SMT; UNSAT=死锁; 零攻击成功率; 0.44ms | **逻辑悖论防御**: SMT求解; 与 D149 四层安全+D171 运行时策略协同 | `nt_shield::logic_paradox` |
| D375 | **安全上下文合约** | 跨域如何安全通信? | CONTINUITY (2609.05269): 假设保证合约; 签名根授权; 2560攻击零有害; 不可丢弃/放宽/重绑 | **安全上下文合约**: 假设保证; 与 D149 四层安全+D131 多Agent协同 | `nt_core::security_context` |
| D376 | **受约束生成** | 工具调用如何形式化约束? | Agent-C (ICLR 2026): DSL时间属性; SMT检查在token生成时; 100%一致性+改进效用 | **受约束生成**: 生成时约束; 与 D171 运行时策略+D149 四层安全协同 | `nt_act::constrained_generation` |
| D377 | **流程合规双用** | 形式化规范如何双用? | AgentLTL (2607.02599): FO-LTL流程规则; 在线门控+微调奖励双用; +38pp准确率 | **流程合规双用**: 评估+训练双用; 与 D150 宪法治理+D136 模型学习协同 | `nt_governance::compliance_dual` |
| D378 | **记忆写入验证** | 记忆如何形式化验证? | SAVER (ACL 2026): 信念→忠实选择(k-DPP)→对抗审计→最小反事实修复; 6基准 | **记忆写入验证**: 审计-修复循环; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::write_verification` |
| D379 | **熵界监控** | 形式化监控为何失效? | Why Monitors Fail (IS 2026): FSA监控recall受攻击熵界; 高熵→近零recall; 预部署熵测试 | **熵界监控**: 预部署熵测试; 与 D149 四层安全+D159 检测器驱动协同 | `nt_shield::entropy_monitor` |
| D380 | **分布式值网络** | 轨迹风险如何预算? | Spawn Freely (2609.01035): 轨迹级风险预算; 分支激活扣费; 证明任意时伤害界 | **分布式值网络**: 风险预算; 与 D175 注意力路由+D150 宪法治理协同 | `nt_core::risk_budget` |
| D381 | **证明协调** | 多Agent如何证明无死锁? | MSC Coordination (ISoLA 2026): MSC DSL; 语法投影生成无死锁本地程序; LLM生成协调工作流 | **证明协调**: 无死锁保证; 与 D131 多Agent+D149 四层安全协同 | `nt_nexus::provable_coordination` |
| D382 | **ZK模型证明** | 模型属性如何零知识证明? | PANDA (2608.17070): ZK证明鲁棒性/公平性; CROWN线性松弛; 2.9M参数5min证明/10s验证 | **ZK模型证明**: 零知识属性证明; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::zk_model_proof` |
| D383 | **流控策略** | 数据流如何策略控制? | AgentFlow (2608.22868): 流控策略; 标注边; SMT验证; 33%→0%妥协; 效用+16.6% | **流控策略**: 数据流约束; 与 D149 四层安全+D171 运行时策略协同 | `nt_shield::flow_policy` |
| D384 | **进程合规LTL** | 进程规则如何形式化? | AgentLTL (2607.02599): FO-LTL流程; 在线门控+微调奖励; +38pp准确率 | **进程合规LTL**: 流程形式化; 与 D150 宪法治理+D136 模型学习协同 | `nt_governance::process_ltl` |
| D385 | **迹级GDPR** | 隐私如何运行时验证? | C-Trace (2606.19242): GDPR谓词; 运行时监控每次工具调用; 三实现交叉验证; 0% ASR | **迹级GDPR**: 三实现验证; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::trace_gdpr` |
| D386 | **静态工作流验证** | 工作流如何静态验证? | Agentproof (2603.20356): 自动提取工作流图; 6结构检查+LTL时间策略; 5000节点<1秒 | **静态工作流验证**: 部署前结构检查; 与 D149 四层安全+D136 模型学习协同 | `nt_shield::static_workflow` |
| D387 | **ZK审计链** | 安全约束如何加密审计? | NiyamAI (2608.07167): 意图合约SHA-256; 隔离法官; zk-SNARK证明; F1 88.5% | **ZK审计链**: 加密审计; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::zk_audit` |
| D388 | **惊奇驱动文化** | Agent社会如何涌现文化? | Emergent Culture (2606.30668): 3无状态Agent+衰减共享KV存储; 自发角色/协调/世界构建 | **惊奇驱动文化**: 衰减共享存储; 与 D131 多Agent+D135 原子记忆协同 | `nt_feel::emergent_culture` |
| D389 | **拓扑感知社会** | 社会模拟如何拓扑优化? | TopoSim (2604.18011): 拓扑作为执行先验; 接收者依赖影响实现; 40-90%token减少 | **拓扑感知社会**: 拓扑先验; 与 D131 多Agent+D175 注意力路由协同 | `nt_core::topology_social` |
| D390 | **集体认知幂律** | 集体智能有何规模限制? | Intellectual Elites (2604.02674): 1.5M+交互; 重尾协调级联; 优先连接→精英; DTI干预 | **集体认知幂律**: 瓶颈检测; 与 D136 模型学习+D150 宪法治理协同 | `nt_mind::collective_cognition` |
| D391 | **拓扑进化技能** | 多Agent拓扑如何技能化? | QueenBee (2606.27492): 拓扑=可检索技能; 外部LLM生成DAG; Preserve/Modify/Avoid规则 | **拓扑进化技能**: 拓扑作为技能; 与 D145 层级技能+D175 注意力路由协同 | `nt_act::topology_skill` |
| D392 | **能力≠合作** | Agent为何不合作? | When Helping Costs Nothing (2604.07821): 能力≠合作; o3达17%而o3-mini达50%; 显式协议修复 | **能力≠合作**: 显式合作设计; 与 D131 多Agent+D149 四层安全协同 | `nt_core::capability_not_cooperation` |
| D393 | **记忆基础设施** | 多Agent记忆如何标准化? | MMP (2604.19540): CAT7模式+SVAF评估+血缘DAG+写入时过滤; 生产部署3实例 | **记忆基础设施**: CAT7+血缘DAG; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::memory_infrastructure` |
| D394 | **Stigmergy涌现** | 去中心化Agent如何自组织? | SwarmWorld (2608.26081): 物理stigmergy; 共享制品; 认知从后果分离; 共享社会更稳健 | **Stigmergy涌现**: 共享制品; 与 D135 原子记忆+D131 多Agent协同 | `nt_core::stigmergy` |
| D395 | **拓扑崩溃** | 高阶交互如何崩溃? | Topological Collapse (2608.15519): 高阶交互可拓扑崩溃; 瓶颈集体智能 | **拓扑崩溃**: 高阶监控; 与 D131 多Agent+D150 宪法治理协同 | `nt_core::topological_collapse` |
| D396 | **协作修复** | 协作如何修复? | COOP² (2603.00349): 约束守卫状态转换; 4协作约束; 修复通道; 规划-执行权衡 | **协作修复**: 约束守卫; 与 D131 多Agent+D149 四层安全协同 | `nt_act::cooperation_repair` |
| D397 | **Yerkes-Dodson** | 环境压力如何最优? | Yerkes-Dodson (2603.07360): 倒U曲线; 中等压力最优; 性选择消除攻击; 压力校准 | **Yerkes-Dodson**: 压力校准; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::pressure_calibration` |
| D398 | **Surrogate规模化** | 大规模Agent如何模拟? | Poor Man's (2608.11215): 低参数替代模型; 感知×记忆分类; 笔记本上模拟8个LLM | **Surrogate规模化**: 替代模型; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::surrogate_scaling` |
| D399 | **Rust浏览器引擎** | 浏览器引擎如何Rust化? | obscura (26.5K★,Rust): 30MB; 85ms加载; 内建反指纹; CDP; MCP服务器; Cloudflare验证 | **Rust浏览器引擎**: 轻量+反检测; 与 D09 反检测抓取+D149 四层安全协同 | `nt_world::rust_browser` |
| D400 | **Rust爬虫** | 爬虫如何Rust化? | spider-rs (2.7K★,Rust): 并发优先; 流式页面; 智能JS渲染; MCP服务器 | **Rust爬虫**: 流式+并发; 与 D09 反检测抓取+D137 三流检索协同 | `nt_world::rust_crawler` |
| D401 | **多设备编排** | 跨设备如何编排? | UFO³ (9.7K★): Galaxy多设备DAG; AIP协议; MCP集成; 混合GUI+API | **多设备编排**: DAG工作流; 与 D163 A2A+D183 结构化软件公司协同 | `nt_act::multi_device` |
| D402 | **端到端知识图谱** | 知识图谱如何端到端构建? | Agents-K1 (2606.13669): 5模块提取; 4B提取骨架; 三源CLI; Scholar-KG 2.46M论文 | **端到端知识图谱**: 4B提取骨架; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::end_to_end_kg` |
| D403 | **本体项目记忆** | 项目记忆如何本体化? | MOOSEDev (2608.13662): OWL+SHACL本体; MCP接口; 0.98-1.00召回; 向量记忆仅6-27% | **本体项目记忆**: 本体>向量; 与 D135 原子记忆+D137 三流检索协同 | `nt_nexus::ontology_memory` |
| D404 | **知识编译** | 科学知识如何编译? | ASKS (2608.29612): GraphDelta+嵌入几何; 可检查状态转换; 持久知识状态 | **知识编译**: GraphDelta; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::knowledge_compilation` |
| D405 | **自改进提取** | 提取如何自改进? | DySECT (2603.06915): 提取→丰富KB→反馈→提升; 5-8%召回/迭代 | **自改进提取**: KB引导反馈; 与 D136 模型学习+D137 三流检索协同 | `nt_world::self_improving_extraction` |
| D406 | **知识条件拓扑** | 多Agent拓扑如何知识条件? | K-GAT (2608.27984): 知识证据条件拓扑; +15.7%超LLM-Debate; <50%token | **知识条件拓扑**: 知识驱动; 与 D145 层级技能+D175 注意力路由协同 | `nt_core::knowledge_topology` |
| D407 | **本体动态构建** | 本体如何动态构建? | OaK (2608.22974): 动态任务导向本体; 图推理函数; 迭代精炼 | **本体动态构建**: 动态本体; 与 D137 三流检索+D136 模型学习协同 | `nt_core::dynamic_ontology` |
| D408 | **代码导航迁移** | 代码Agent如何导航知识图谱? | GRA (2608.15834): ls/cat/grep迁移到混合KG; schema无关; +5.1pp; 29-33%token | **代码导航迁移**: 导航>上下文; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::code_nav_kg` |
| D409 | **自适应结构化** | 非结构数据如何自适应? | Data Cracking (2608.31082): 推理时自适应结构化; 分叉子Agent; 53%成本减少 | **自适应结构化**: 惰性知识结晶; 与 D137 三流检索+D135 原子记忆协同 | `nt_world::adaptive_structuring` |
| D410 | **符号知识图谱** | 逻辑规则如何图化? | SymbolLKG (2608.26836): 逻辑规则=拓扑节点; 动态求解器路由; 混合检索 | **符号知识图谱**: 逻辑=节点; 与 D97 E8推理+D137 三流检索协同 | `nt_core::symbolic_kg` |
| D411 | **多Agent文档提取** | 文档如何多Agent提取? | HERMES (2608.14055): ReAct编排子Agent; 32K实体+451K属性; 55卷古生物学 | **多Agent文档提取**: 子Agent编排; 与 D137 三流检索+D135 原子记忆协同 | `nt_world::multi_agent_extraction` |
| D412 | **本体后纠正** | 本体如何后纠正? | OAK+MEND (2605.29168): 嵌入规范+LLM纠正; 98.4%一致性; 59%token | **本体后纠正**: 嵌入+纠正; 与 D137 三流检索+D136 模型学习协同 | `nt_world::ontology_correction` |
| D413 | **证明式记忆** | Agent记忆如何形式化? | MOOSEDev (2608.13662): OWL+SHACL本体; MCP; 0.98-1.00召回 vs 向量6-27% | **证明式记忆**: 本体>向量; 与 D135 原子记忆+D137 三流检索协同 | `nt_nexus::proven_memory` |
| D414 | **流程形式化验证** | Agent流程如何形式化? | CONTINUITY (2609.05269): 假设保证合约; 签名授权; 2560攻击零有害 | **流程形式化验证**: 合约驱动; 与 D150 宪法治理+D149 四层安全协同 | `nt_governance::flow_formal` |
| D415 | **社会涌现涌现** | Agent社会涌现何种结构? | Agentopia (2606.07513): 100Agent×10年; 生命奖励; 个人成长/关系/经济专业化 | **社会涌现涌现**: 生命奖励; 与 D131 多Agent+D146 进化速度协同 | `nt_feel::social_emergence` |
| D416 | **宇宙记忆** | 跨会话记忆如何结构化? | MMP (2604.19540): CAT7+SVAF+血缘DAG; 3实例生产部署 | **宇宙记忆**: CAT7模式; 与 D135 原子记忆+D137 三流检索协同 | `nt_nexus::cosmic_memory` |
| D417 | **形式化Guard进化** | Guard如何进化? | SEVerA (2603.25111): 搜索→验证→学习; 零违反; 约束剪枝搜索空间 | **形式化Guard进化**: 三阶段; 与 D136 模型学习+D150 宪法治理协同 | `nt_shield::guard_evolution` |
| D418 | **能力等级社会** | Agent社会能力如何分布? | SILICA (2608.28182): 5环境人类锚点; 仅起始点一致; 行动顺序交换-58分 | **能力等级社会**: 人类锚点; 与 D131 多Agent+D150 宪法治理协同 | `nt_mind::capability_society` |
| D419 | **本体冲突修复** | 本体冲突如何修复? | OAK+MEND (2605.29168): 嵌入规范+LLM纠正; 98.4%一致性 | **本体冲突修复**: 嵌入+纠正; 与 D137 三流检索+D149 四层安全协同 | `nt_world::ontology_repair` |
| D420 | **拓扑优化社交** | 社交拓扑如何优化? | TopoSim (2604.18011): 拓扑先验; 接收者依赖; 40-90%token减少 | **拓扑优化社交**: 拓扑先验; 与 D131 多Agent+D175 注意力路由协同 | `nt_core::topology_social_opt` |

### 0.38 Agent 生产架构模式 (from Claude Code / OpenAI / MCP / Observability)

> 从 Flare / PIVOT / PCE / MagicAgent / ToolTree / SMITH / HEART / SARA / SPyCE / MuSEAgent / AXPO / MUSE / LMM-Searcher / Beacon / VISTA-Gym / PERIA / iSHIFT / CausalCache / HarnessEvolve / Self-Harness / HSI / Metaⁿ / Hyperagents / MGM / SkillGLoW / MetaSkill-Evolve / DiagEvo / MetaClaw / Pathway(63K★) / DocETL(4K★) / LOTUS(1.7K★) / SeaTunnel(9.5K★) / NeMo Curator(1.7K★) 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D321 | **推理≠规划** | 推理如何区分规划? | Flare (2601.22311): 形式化证明CoT≠规划; MCTS向后值传播; LLaMA-8B+Flare超GPT-4o | **推理≠规划**: MCTS值传播; 与 D97 E8推理+D138 密集奖励协同 | `nt_core::reasoning_vs_planning` |
| D322 | **轨迹自优化** | 轨迹如何自我优化? | PIVOT (2605.11225): 轨迹=可优化对象; 文本梯度; 3-5x少token; 94%相对提升 | **轨迹自优化**: 文本梯度; 与 D135 原子记忆+D136 模型学习协同 | `nt_mind::trajectory_optimization` |
| D323 | **不确定规划** | 不确定性如何影响规划? | PCE (2602.04326): 潜在假设→结构化决策树; 不确定性感知规划; 超通信基线 | **不确定规划**: 假设=一等变量; 与 D178 因果注意力+D97 E8推理协同 | `nt_core::uncertain_planning` |
| D324 | **通用Agent规划** | Agent规划如何泛化? | MagicAgent (2602.19000): 5维合成数据; SFT+多目标RL; 32B超GPT-5.2 | **通用Agent规划**: 合成数据框架; χPO在线RL; 与 D136 模型学习+D145 层级技能协同 | `nt_act::general_planning` |
| D325 | **工具树规划** | 工具如何树状规划? | ToolTree (ICLR 2026): 双反馈MCTS; 预执行评分+后执行效用; +10%超基线 | **工具树规划**: 双反馈MCTS; 与 D98 HQL+D175 注意力路由协同 | `nt_act::tool_tree` |
| D326 | **联合工具创建** | 工具创建如何与使用联合训练? | SMITH (2608.24571): RL联合训练创建+使用; 4B模型79.8%超30B; 工具提升350M/30B模型 | **联合工具创建**: 创建+使用联合; 与 D145 层级技能+D173 混合技能协同 | `nt_act::joint_tool_creation` |
| D327 | **自然语言工具** | 工具接口如何自然语言化? | HEART (2609.01736): 自然语言接口替代API; ToolFace 25519函数; +6%超GPT-5.4; 成本降85% | **自然语言工具**: Tool Primitives; 与 D165 DSPy+D98 HQL协同 | `nt_io::natural_language_tools` |
| D328 | **动作授权分离** | 工具建议如何与执行分离? | SARA (2608.27146): 动作归纳≠执行授权; No-History-Promotion; 攻击率≤0.63% | **动作授权分离**: 安全模式; 与 D149 四层安全+D261 流控协同 | `nt_shield::action_auth_split` |
| D329 | **技能策略共进化** | 技能如何与策略共进化? | SPyCE (2607.13854): 层次化技能库(执行+工作流); 共进化循环; 强策略→更好技能 | **技能策略共进化**: 执行+工作流双层; 与 D145 层级技能+D136 模型学习协同 | `nt_mind::skill_policy_coevolve` |
| D330 | **有状态经验** | Agent经验如何有状态抽象? | MuSEAgent (2603.27813): 原子状态-行动经验; 后见推理; 深宽搜索; 质量过滤 | **有状态经验**: 组合状态表示; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::stateful_experience` |
| D331 | **思维-行动差距** | Agent为何工具使用失败? | AXPO (2605.28774): 30%尝试工具; 全错子组抑制学习; 不确定性前缀选择 | **思维-行动差距**: 前缀固定+重采样; 与 D97 E8推理+D175 注意力路由协同 | `nt_act::thinking_acting_gap` |
| D332 | **模块化Agent框架** | Agent框架如何模块化? | MUSE (2606.03005): 任务表示+视觉处理+感知工具+确定性验证+修复; 无模型重训 | **模块化Agent框架**: 框架优化(非模型); 与 D172 工具正确性+D133 异步人类协同 | `nt_io::modular_harness` |
| D333 | **文件化视觉代理** | 视觉上下文如何管理? | LMM-Searcher (2604.12890): UID轻量代理; 100轮搜索; 按需渐进加载 | **文件化视觉代理**: UID代理+渐进加载; 与 D105 KVMem+D197 信息折叠协同 | `nt_memory::file_visual_proxy` |
| D334 | **模式适应性** | Agent何时使用工具? | Beacon (2607.28595): 模式适应性+工具效用; 必要性感知奖励; 能力扩展 | **模式适应性**: GWT salience形式化; 与 D175 注意力路由+D41 黑暗森林协同 | `nt_core::mode_adaptiveness` |
| D335 | **感知对比优化** | 视觉感知如何优化? | CPPO (2601.00501): 熵移位检测感知token; 对比感知损失; 无额外LLM法官 | **感知对比优化**: 感知token检测; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_world::perception_contrast` |
| D336 | **工具集成视觉推理** | VLM如何集成工具? | VISTA-Gym (CVPR 2026): 7任务13数据集; R1-8B超基线9.5-18.7%; RL解锁工具使用 | **工具集成视觉推理**: 统一训练环境; 与 D140 多模态记忆+D173 混合技能协同 | `nt_act::tool_integrated_vr` |
| D337 | **感知-交互-推理** | 视觉Agent如何推理? | PERIA (2606.12830): 18工具; OR-GIGPO多步信用分配; 8B接近GPT-5 | **感知-交互-推理**: 三阶段循环; 与 L2→L1→L5层流协同 | `nt_core::perceive_interact_reason` |
| D338 | **快慢视觉Agent** | GUI Agent如何快慢切换? | iSHIFT (CVPR 2026): 2.5B; 潜在思考token; 感知控制模块; 快慢模式 | **快慢视觉Agent**: 慢=详细定位,快=全局线索; 与 D170 双武器+D175 注意力路由协同 | `nt_io::slow_fast_gui` |
| D339 | **预算保真度** | 长视距视觉如何管理历史? | CausalCache (2608.22577): 预算保真度恢复; 效用预测交换; +4.3pp 30步预算 | **预算保真度**: 效用预测交换; 与 D105 KVMem+D197 信息折叠协同 | `nt_memory::budgeted_fidelity` |
| D340 | **结构化证据空间** | 跨页推理如何结构化? | VISOR (ACM MM 2026): 结构化证据空间; 动态轨迹+滑动窗口+意图注入 | **结构化证据空间**: 跨会话推理; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::structured_evidence` |
| D341 | **Harness进化** | Harness如何自我进化? | HarnessEvolve (2609.00829): 参考轨迹对齐; 质量门+性能门; 防捷径学习 | **Harness进化**: 双门控; 与 D136 模型学习+D150 宪法治理协同 | `nt_mind::harness_evolve` |
| D342 | **自Harness** | Agent如何改进自身Harness? | Self-Harness (2606.09498): 弱点挖掘→提案→验证; +21.4pp; 无外部强模型 | **自Harness**: 内部改进循环; 与 D97 E8推理+D136 模型学习协同 | `nt_mind::self_harness` |
| D343 | **层次自改进** | 自改进如何层次化? | HSI (2608.08466): 任务Harness→进化器→元进化器; 冻结锚防止失控 | **层次自改进**: 三层+冻结锚; 与 D150 宪法治理+D136 模型学习协同 | `nt_mind::hierarchical_self_improve` |
| D344 | **递归深度自改进** | 递归深度如何控制? | Metaⁿ (2608.24735): 固定元操作递归; 深度由收敛决定; 唯一ARC-AGI-2>0方法 | **递归深度**: 收敛决定深度; 与 D136 模型学习+D150 宪法治理协同 | `nt_mind::recursive_depth` |
| D345 | **超Agent** | 元改进如何跨域转移? | Hyperagents (2603.19461): 元修改过程本身可编辑; 跨域转移+跨运行累积 | **超Agent**: 元认知自修改; 与 D150 宪法治理+D131 多Agent协同 | `nt_mind::hyper_agents` |
| D346 | **技能程序族** | 技能如何程序族化? | SkillGLoW (2609.02217): 程序族技能整合; 去实例化→全局先验; 提交门防退化 | **技能程序族**: 去实例化+提交门; 与 D145 层级技能+D135 原子记忆协同 | `nt_mind::skill_procedure_family` |
| D347 | **双时间尺度进化** | 进化如何双时间? | MetaSkill-Evolve (2607.05297): 快循环(任务-技能)+慢循环(元技能); 递归精炼 | **双时间尺度**: 快慢循环; 与 D136 模型学习+D150 宪法治理协同 | `nt_mind::dual_timescale` |
| D348 | **诊断进化** | 失败如何诊断进化? | DiagEvo (2609.00768): 错误原因记忆; Active/Mastered状态; 双置信度过滤 | **诊断进化**: 错误原因图; 与 D135 原子记忆+D69 智能自愈协同 | `nt_mind::diagnostic_evolution` |
| D349 | **空闲窗口优化** | Agent空闲时如何优化? | MetaClaw (2603.17187): 技能驱动快适应+空闲LoRA优化; 无本地GPU; 生产部署 | **空闲窗口优化**: 功耗感知调度; 与 D146 进化速度+D136 模型学习协同 | `nt_mind::idle_optimization` |
| D350 | **批量流式统一** | 批处理如何与流式统一? | Pathway (63K★): Rust引擎+Python API; 增量计算; LLM管线原生; 300+连接器 | **批量流式统一**: 单一API; 与 D137 三流检索+D39 五层渐进协同 | `nt_world::batch_stream_unified` |
| D351 | **Agentic ETL** | ETL如何Agent化? | DocETL (4K★): 声明式LLM算子; Agent优化器重写提示+分解操作+代码替换 | **Agentic ETL**: 声明式算子+优化器; 与 D137 三流检索+D136 模型学习协同 | `nt_world::agentic_etl` |
| D352 | **语义算子** | 数据处理如何语义化? | LOTUS (1.7K★): 语义map/filter/reduce/join; 自动优化(批处理/模型级联/延迟规划) | **语义算子**: 语义操作原语; 与 D137 三流检索+D184 认知图谱协同 | `nt_world::semantic_operators` |
| D353 | **GPU数据策展** | 训练数据如何策展? | NeMo Curator (1.7K★): RAPIDS+Ray; GPU加速去重; 多模态(文本/图像/视频/音频) | **GPU数据策展**: GPU加速去重; 与 D137 三流检索+D140 多模态记忆协同 | `nt_world::gpu_data_curation` |
| D354 | **分布式数据集成** | 数据集成如何分布? | SeaTunnel (9.5K★): 160+连接器; 多引擎(Zeta/Flink/Spark); CDC; 分布式快照 | **分布式数据集成**: 连接器插件; 与 D137 三流检索+D149 四层安全协同 | `nt_world::distributed_etl` |
| D355 | **约束框架** | Harness如何约束LLM? | PTR (2604.04131): 限制LLM调用2-3次/任务; 工作流合成→确定性执行→验证→修复 | **约束框架**: 有限LLM调用; 与 D150 宪法治理+D172 工具正确性协同 | `nt_act::constrained_framework` |
| D356 | **双层不确定规划** | 不确定性如何分解? | WebUncertainty (2604.17821): 任务不确定→自适应模式; 行动不确定→MCTS; 偶然+认知 | **双层不确定规划**: 偶然+认知分解; 与 D178 因果注意力+D97 E8推理协同 | `nt_core::dual_uncertainty` |
| D357 | **能力自进化规划** | 规划能力如何进化? | Planning Survey (2607.04096): 规划=可验证程序; 符号+SLM组合; 设计时验证 | **能力自进化规划**: 可验证程序; 与 D97 E8推理+D150 宪法治理协同 | `nt_core::evolvable_planning` |
| D358 | **步骤自适应思考** | 思考深度如何自适应? | SAT (2604.07922): FSM步骤级剪枝; 4思考模式; 30M轻量PRM; 40%token减少 | **步骤自适应思考**: 4模式路由; 与 D175 注意力路由+D178 因果注意力协同 | `nt_core::step_adaptive_thinking` |
| D359 | **思维级束搜索** | 计算如何重新分配? | Gambit (2608.08020): 主动计算重分配; 修剪+分支; +6.7%超基线; 68.5%token减少 | **思维级束搜索**: salience驱动重分配; 与 D175 注意力路由+D228 可微分多Agent协同 | `nt_core::thought_beam_search` |
| D360 | **异步并行推理** | 推理如何异步并行? | ParaTempo (2608.16425): 分支局部时间置信度; 异步剪枝/退休/分叉; 21-32%延迟减少 | **异步并行推理**: 无需同步; 与 D137 三流检索+D150 宪法治理协同 | `nt_core::async_parallel_reasoning` |
| D361 | **递归Agent推理** | 推理算子如何统一? | Recursive (2608.23956): GROW/PRUNE/BRANCH统一; BRANCH主导14/14设置; +5.98pp | **递归Agent推理**: BRANCH主导; 与 D97 E8推理+D145 层级技能协同 | `nt_core::recursive_reasoning` |
| D362 | **去中心化推理** | 推理如何去中心化? | DeAR (2608.17282): P2P协作替代中心路由; 动态能力接地+思维图导航+拓扑更新 | **去中心化推理**: P2P; 与 D131 多Agent+D97 E8推理协同 | `nt_core::decentralized_reasoning` |
| D363 | **鞍盆搜索** | 推理如何避免局部最优? | BASIN (2609.00738): 推理状态分组为鞍盆; 惩罚过度访问策略; +22pp Game of 24 | **鞍盆搜索**: 避免局部最优; 与 D97 E8推理+D178 因果注意力协同 | `nt_core::basin_search` |
| D364 | **第二思维** | 空闲窗口如何利用? | Second Thought (2608.13667): 空闲窗口分叉辅助推理; 观察到达时合并; 43%主线程减少 | **第二思维**: 空闲窗口利用; 与 D137 三流检索+D175 注意力路由协同 | `nt_core::second_thought` |
| D365 | **证据残差分配** | 计算价值如何估计? | AERA (2608.27964): 顺序控制器估计计算未来价值; 92.6% GSM8K; 96%少token | **证据残差分配**: 未来价值估计; 与 D175 注意力路由+D150 宪法治理协同 | `nt_core::evidence_residual` |
| D366 | **何时思考** | 推理深度如何自适应? | Learning-When (2608.20256): 1.5B模型学习NoThink/Short/Long; GRPO; 41-76%token减少 | **何时思考**: 内嵌路由器; 与 D175 注意力路由+D178 因果注意力协同 | `nt_core::when_to_think` |
| D367 | **结构化不确定性** | 不确定性如何结构化? | YUKTI (2607.09706): 类型化命题IR; ARPF多目标优化; 47x低遗憾 | **结构化不确定性**: LRM=形式化者(非求解者); 与 D97 E8推理+D150 宪法治理协同 | `nt_core::structured_uncertainty` |
| D368 | **证据增强VQA** | 视觉检索如何增强? | VISOR证据空间+Intent注入+滑动窗口; 跨页推理; SOTA | **证据增强VQA**: 结构化证据; 与 D137 三流检索+D140 多模态记忆协同 | `nt_world::evidence_vqa` |
| D369 | **主动视觉搜索** | 视觉如何主动搜索? | Visual-Seeker (2606.15231): 主动注意力细粒度视觉; 5K高质量轨迹 | **主动视觉搜索**: 主动证据采集; 与 D137 三流检索+D140 多模态记忆协同 | `nt_world::active_visual_search` |
| D370 | **设备端VLM** | VLM如何设备端部署? | StepX-Edge (2607.22708): 0.9B; UI感知分层编码; 5阶段渐进课程; Snapdragon 8 Gen5 | **设备端VLM**: 5阶段课程; 与 D168 Ollama+D145 层级技能协同 | `nt_io::edge_vlm` |

### 0.37 Agent 生产架构模式 (from Claude Code / OpenAI / MCP / Observability)

> 从 4 个生产级 Agent 系统提炼的架构模式，驱动 NeoTrix 生产化。

| 模式 | 来源 | 核心机制 | NeoTrix 映射 | 优先级 |
|------|------|---------|-------------|--------|
| **Hook-Based Lifecycle Events** | Claude Code (27 events) | 确定性回调绕过 LLM; Exit code 2 = hard block | NT-CORE + NT-META: SEAL pipeline 10-15 hook events (PreToolUse/PostToolUse/PreCompact 等) | P0 |
| **Span-Per-Tick Observability** | Arthur/Braintrust | 4 span 类型: Tool call/Reasoning/State transition/Memory | NT-META + all: OpenTelemetry spans for E8 reasoning / MCP tools / KB ops / domain handoffs | P0 |
| **Scoped Proxy Aggregator** | ICSE 2026 MCP | 按任务暴露 10-15 工具子集; 解决 >15 工具准确率悬崖 | NT-CORE (GWT): salience-based tool routing, 按上下文动态暴露工具子集 | P1 |
| **Handoff-as-Tool** | OpenAI Agents SDK | 专家委派为工具; input_filter 控制历史可见性 | NT-CORE: AttentionManager 双武器切换为显式 handoff 工具; input_type 携带 reason/priority | P1 |
| **Code-Exec MCP** | Anthropic (2025-11) | 工具以文件系统 API 暴露; 按需发现而非全量加载 | NT-ACT: 工具发现为文件系统导航; token 从 ~150K 降至 ~2K (98.7%) | P2 |

### 0.40 数学推理/多模态/时序决策 (v11.4 Deep Absorption Batch 2026-09-08)

> 从 AlphaProof Nexus / Aletheia / Aristotle / Danus / M2A / SCION / MACT(CVPR) / OmniAgent(ICML) / CMA / VESTA / LedgerMind / D2-ScaleAgent / Nexus / KairosAgent / TimeClaw / Falcon-X / TimesFM-3 / Timer-S1 中提炼。

| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D551 | **形式化证明搜索** | 证明如何搜索? | AlphaProof Nexus (2605.22763): Gemini+Lean4; 9 Erdős问题; Elo评分; $100-600/问题 | **形式化证明搜索**: 并行子Agent+Lean验证; 与 D97 E8推理+D175 注意力路由协同 | `nt_core::formal_proof_search` |
| D552 | **自主数学研究** | 数学如何自主研究? | Aletheia (2602.10177): Gemini 3 Deep Think; 4 Erdős问题; 纯NL无形式验证; 推理时缩放律 | **自主数学研究**: 迭代生成→验证→修订; 与 D136 模型学习+D150 宪法治理协同 | `nt_mind::autonomous_math` |
| D553 | **三组件定理证明** | 证明如何三组件? | Aristotle (2510.01346): Lean证明搜索+非形式推理+几何求解器; MCTS; IMO金牌 | **三组件定理证明**: 3组件; 与 D97 E8推理+D145 层级技能协同 | `nt_core::three_component_prover` |
| D554 | **事实图记忆** | 数学如何事实图? | Danus (2607.06447): 共享事实图作为全局记忆; 协调并行证明搜索; 中间声明可靠性跟踪 | **事实图记忆**: 事实图; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::fact_graph` |
| D555 | **空空间模型合并** | 模型如何空空间合并? | M2A (2605.09879): 空空间投影无训练合并; Qwen3-8B SWE-Bench 44→51.2%; 行为保持 | **空空间模型合并**: 行为保持合并; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::nullspace_merge` |
| D556 | **科学协作OS** | 科学如何协作OS? | SCION (2607.03863): Science Agent作为Meta-Harness; REP编译科学意图; 分子/蛋白/抗体 | **科学协作OS**: REP模式; 与 D150 宪法治理+D146 进化速度协同 | `nt_mind::scientific_os` |
| D557 | **求解器到研究** | 数学如何研究? | Solvers→Research (2607.07779): 问题求解→研究Agent; 328+ Erdős贡献; Terence Tao | **求解器到研究**: 范式转变; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::solver_to_research` |
| D558 | **代理环境工程** | 科学如何环境工程? | EurekAgent (2606.13662): 瓶颈从Agent工作流→Agent环境; 资源+约束+接口 | **代理环境工程**: 环境>Agent; 与 D150 宪法治理+D149 四层安全协同 | `nt_core::agent_environment` |
| D559 | **工作记忆瓶颈** | 推理如何工作记忆? | Algebraic 9-Dim (2604.06799): 9正交维度; 工作记忆是主要瓶颈; 所有模型20-30分支崩溃 | **工作记忆瓶颈**: 20-30分支限制; 与 D97 E8推理+D175 注意力路由协同 | `nt_core::working_memory_bottleneck` |
| D560 | **Agentwise自适应缩放** | 缩放如何Agentwise? | MACT (CVPR 2026): 4 Agent(规划/执行/判断/回答); Agentwise自适应测试时缩放; +9.9-11.5% | **Agentwise自适应缩放**: 过程缩放>单体缩放; 与 D175 注意力路由+D176 资源预算协同 | `nt_core::agentwise_scaling` |
| D561 | **感知即推理** | 感知如何推理? | OmniAgent (ICML 2026): POMDP原生全模态; 感知=推理(非预处理); 持久文本记忆; 7B>72B | **感知即推理**: 感知=推理; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_core::perception_is_reasoning` |
| D562 | **情景视觉记忆** | 视觉如何情景记忆? | CMA (2607.08497): 情景视觉记忆外化; 感知抽象引擎+认知检索引擎; 8B>32B +8.2% | **情景视觉记忆**: 情景记忆; 与 D140 多模态记忆+D135 原子记忆协同 | `nt_memory::episodic_visual` |
| D563 | **时间证据账本** | 视频如何证据账本? | VESTA (2608.31005): 意图路由器→获取→验证→巩固循环; 时间证据账本; +2.7 Video-MME | **时间证据账本**: 证据账本; 与 D135 原子记忆+D149 四层安全协同 | `nt_memory::temporal_evidence_ledger` |
| D564 | **溯源约束状态机** | 证据如何溯源约束? | LedgerMind (2607.28374): 结构化证据账本; 溯源不放大保证; 三层接地协议 | **溯源约束状态机**: 不放大保证; 与 D149 四层安全+D135 原子记忆协同 | `nt_memory::provenance_state` |
| D565 | **双维缩放** | 文档如何双维缩放? | D2-ScaleAgent (2608.16417): 检索缩放(外向)+推理缩放(内向); 证据库动态路由 | **双维缩放**: 内外双缩放; 与 D137 三流检索+D178 因果注意力协同 | `nt_core::dual_dimension_scaling` |
| D566 | **视频深度研究** | 视频如何深度研究? | VideoRover (2608.23329): 视频裁剪↔多模态搜索↔网页浏览; 26K SFT+3K RL | **视频深度研究**: 多工具协调; 与 D137 三流检索+D140 多模态记忆协同 | `nt_world::video_deep_research` |
| D567 | **规划先于感知** | 视频如何规划感知? | EVA (2603.22918): 摘要→规划→行动→反思; SFT→KTO→GRPO; +6-12% | **规划先于感知**: 规划优先; 与 D97 E8推理+D145 层级技能协同 | `nt_world::plan_before_perceive` |
| D568 | **统一视频推理** | 视频如何统一推理? | InternVideo3 (2606.12195): MCR(观察+推理+工具+记忆); M²LA KV压缩; 阶段训练 | **统一视频推理**: MCR统一; 与 D140 多模态记忆+D175 注意力路由协同 | `nt_world::unified_video_reason` |
| D569 | **多粒度工具** | 视频如何多粒度? | VideoSeek (2603.20185): 扫描/探测/检查多粒度; 视频逻辑流引导; +10.2; -93%帧 | **多粒度工具**: 逻辑流引导; 与 D137 三流检索+D175 注意力路由协同 | `nt_world::multigranular_tool` |
| D570 | **主动视觉感知** | 文档如何主动视觉? | InSight-doc (2608.10628): 粗到细缩放; -40%幻觉; -41-68%延迟 | **主动视觉感知**: 缩放感知; 与 D140 多模态记忆+D175 注意力路由协同 | `nt_world::active_visual_perception` |
| D571 | **问题引导证据** | VQA如何问题引导? | Q-Guide (2608.19739): 小Agent确定缺失证据; 靶向工具调用; 65% DocVQA vs 40%直接 | **问题引导证据**: 按需获取; 与 D137 三流检索+D145 层级技能协同 | `nt_world::question_guided_evidence` |
| D572 | **动态检索推理** | VQA如何动态检索? | Learning to Search (2604.07146): 多步搜索Agent; 4动作; SOTA InfoSeek+E-VQA | **动态检索推理**: 多步搜索; 与 D137 三流检索+D145 层级技能协同 | `nt_world::dynamic_retrieval_reason` |
| D573 | **多模态规划Agent** | 规划如何多模态? | Efficient Planning (2601.20676): 动态分解mRAG; -60%搜索时间; 3-4.5x快 | **多模态规划Agent**: 效率优先; 与 D176 资源预算+D175 注意力路由协同 | `nt_world::multimodal_planning` |
| D574 | **操纵接地VQA** | VQA如何操纵接地? | PROBE (2608.17129): 先操纵后回答; PROBE-Sim+PROBE-Agent蒸馏; 仿真到真机 | **操纵接地VQA**: 具身推理; 与 D140 多模态记忆+D145 层级技能协同 | `nt_world::manipulation_grounded` |
| D575 | **多模态搜索Agent** | 搜索如何多模态? | DR-MMSearchAgent (2604.19264): SPAI+BGAS奖励; +8.4% FVQA | **多模态搜索Agent**: 多模态奖励; 与 D137 三流检索+D140 多模态记忆协同 | `nt_world::multimodal_search` |
| D576 | **元控制器Agent** | 控制如何元控制器? | MMDynOpt-Agent (2608.14026): 轻量Agent作为MDP策略; 预算感知奖励; 跨MLLM迁移 | **元控制器Agent**: 元控制; 与 D131 多Agent+D176 资源预算协同 | `nt_meta::meta_controller` |
| D577 | **失败感知恢复** | 搜索如何失败感知? | WeAgent-MMSearch (2608.28062): 持久图像引用; FA-GSPO恢复可挽救轮次 | **失败感知恢复**: 失败恢复; 与 D149 四层安全+D136 模型学习协同 | `nt_world::failure_aware_recovery` |
| D578 | **工具证据意图** | 工具如何证据意图? | NTEP (2609.03493): 必要工具-证据路径; 意图对齐+观察对齐; 非重复正则化 | **工具证据意图**: 意图对齐; 与 D149 四层安全+D172 工具正确性协同 | `nt_act::tool_evidence_intent` |
| D579 | **认知多模态Agent** | 多模态如何认知? | CMA: 认知结构化; 情景视觉记忆; +8.2%超32B | **认知多模态Agent**: 认知结构化; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_core::cognitive_multimodal` |
| D580 | **跨模态导航** | 导航如何跨模态? | CRONA (2605.06595): 去中心化MARL; 5优势模式; 异构模态>单体多模态 | **跨模态导航**: 异构>单体; 与 D140 多模态记忆+D131 多Agent协同 | `nt_world::cross_modal_nav` |
| D581 | **文档探索循环** | 文档如何探索? | MARDoc (2606.05749): 探索→精炼→反思; 3 Agent+结构化记忆; Qwen3-30B≈Claude 3.5 | **文档探索循环**: 3 Agent; 与 D137 三流检索+D149 四层安全协同 | `nt_world::doc_explore_loop` |
| D582 | **文档技能状态** | 文档如何技能状态? | DocClaw (2608.18685): 文档技能+结构化文档状态(文档记忆+任务记忆) | **文档技能状态**: 文档状态; 与 D135 原子记忆+D145 层级技能协同 | `nt_world::doc_skill_state` |
| D583 | **跨视频推理** | 视频如何跨视频? | AgentCVR (2605.29643): 主Agent协调视觉/音频Agent; 脚本模拟RL | **跨视频推理**: 多视频协调; 与 D131 多Agent+D140 多模态记忆协同 | `nt_world::cross_video_reason` |
| D584 | **冻结验证器训练** | 训练如何冻结验证? | Multi-Agent Self-Improving (2608.28675): Grounder+冻结Verifier; 校准损失; 零样本迁移 | **冻结验证器训练**: 冻结验证; 与 D136 模型学习+D149 四层安全协同 | `nt_mind::frozen_verifier_train` |
| D585 | **具身技能编排** | 机器人如何技能编排? | EmbodiedSkills (2609.01281): 前置检查→VLA执行→后验证; Qwen3-VL+OpenPI; 86.2% | **具身技能编排**: 技能验证; 与 D145 层级技能+D149 四层安全协同 | `nt_physical::embodied_skill_orch` |
| D586 | **工具注入VLA** | VLA如何工具注入? | ART (2608.14047): 两阶段LoRA工具注入; 30K轨迹; +20%成功率; 保持原始保真 | **工具注入VLA**: 工具注入; 与 D145 层级技能+D136 模型学习协同 | `nt_physical::tool_inject_vla` |
| D587 | **多Agent分解预测** | 预测如何分解? | Nexus (2605.14389): 宏观/微观+合成; 超TimesFM-2.5; 校准循环 | **多Agent分解预测**: 分解预测; 与 D131 多Agent+D178 因果注意力协同 | `nt_act::decomposed_forecast` |
| D588 | **最后一英里预测** | 预测如何最后一英里? | Last Mile (2606.02497): 约束修订动作+记忆库; 业务上下文修订基线 | **最后一英里预测**: 约束修订; 与 D135 原子记忆+D171 运行时策略协同 | `nt_act::last_mile_forecast` |
| D589 | **融合语义推理预测** | 预测如何融合语义? | KairosAgent (2605.30002): LLM推理+TSFM; 语义先验注入潜空间; 40K轨迹+RL | **融合语义推理预测**: 语义注入; 与 D140 多模态记忆+D97 E8推理协同 | `nt_act::fused_semantic_forecast` |
| D590 | **人在环预测** | 预测如何人在环? | CastClaw (2608.30976): 版本执行记录+约束检查+升级规则; 5电力数据集最低MSE | **人在环预测**: 人在环; 与 D133 异步人类+D149 四层安全协同 | `nt_act::hitl_forecast` |
| D591 | **快慢反思推理** | 预测如何快慢反思? | CastFSR (2608.03031): 快先验→慢深思→反思验证; 无训练或SFT+RL | **快慢反思推理**: 三阶段; 与 D97 E8推理+D175 注意力路由协同 | `nt_core::fast_slow_reflect` |
| D592 | **角色专业化预测** | 预测如何角色专业化? | CastFlow (2604.27840): 冻结LLM规划+微调预测; 多视图工具+策略记忆 | **角色专业化预测**: 角色分工; 与 D131 多Agent+D145 层级技能协同 | `nt_act::role_specialized_forecast` |
| D593 | **探索执行学习** | 预测如何探索? | TimeClaw (2605.10038): 探索→比较→蒸馏→重注入; 度量监督探索; 防工具先验崩溃 | **探索执行学习**: 4阶段循环; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::exploratory_exec` |
| D594 | **异构时序基础模型** | 基础模型如何异构? | Falcon-X (2605.27286): 潜在原型空间; Diff-Attention协同/对抗关系; 591M; SOTA | **异构时序基础模型**: 原型空间; 与 D140 多模态记忆+D97 E8推理协同 | `nt_world::heterogeneous_ts` |
| D595 | **相关性感知适配器** | 基础模型如何相关适配? | CoRA (ICLR 2026,2603.21828): 轻量即插即用适配器; 时变/时不变+异构/部分分解 | **相关性感知适配器**: 即插即用; 与 D145 层级技能+D176 资源预算协同 | `nt_world::correlation_adapter` |
| D596 | **多变量零样本** | 基础模型如何多变量? | TimesFM-3 (Google): 330M; 连续补丁掩码; 原生多变量+协变量; SOTA | **多变量零样本**: 原生多变量; 与 D140 多模态记忆+D137 三流检索协同 | `nt_world::multivariate_zeroshot` |
| D597 | **十亿规模MoE** | 基础模型如何MoE? | Timer-S1 (2603.04791): 8.3B/0.75B激活; 序列Token预测; TimeBench 1T+点 | **十亿规模MoE**: MoE路由; 与 D175 注意力路由+D176 资源预算协同 | `nt_world::billion_scale_moe` |
| D598 | **多任务免调优** | 基础模型如何多任务? | Zeus (2607.01918): 多尺度Transformer+MOTM; 5任务免调优 | **多任务免调优**: 免调优; 与 D145 层级技能+D176 资源预算协同 | `nt_world::multitask_free` |
| D599 | **相位旋转记忆** | 记忆如何相位旋转? | RoMem (2604.11544): 时间=几何相位旋转; 语义速度门; 追加记忆+几何阴影 | **相位旋转记忆**: 连续旋转; 与 D135 原子记忆+D147 涌现记忆协同 | `nt_memory::phase_rotation` |
| D600 | **时间索引上下文学习** | 时序如何上下文学习? | TS-ICL (2606.05878): 概率编码器-回归器; DAG合成协变量先验; SOTA填补 | **时间索引上下文学习**: 因果先验; 与 D137 三流检索+D97 E8推理协同 | `nt_world::time_indexed_icl` |
| D601 | **嵌入式均衡** | 博弈如何嵌入式? | Embedded Bayesian (2608.03958): 嵌入式均衡替代Nash; 自我参照意识 | **嵌入式均衡**: 嵌入式均衡; 与 D97 E8推理+D150 宪法治理协同 | `nt_core::embedded_equilibrium` |
| D602 | **理性推理防博弈** | Agent如何防博弈失败? | Reasonably Reasoning (2603.18563): 贝叶斯后验→Nash收敛; 无需后训练 | **理性推理防博弈**: 贝叶斯收敛; 与 D97 E8推理+D149 四层安全协同 | `nt_core::rational_game_avoid` |
| D603 | **Nash抑制机制** | LLM如何抑制Nash? | What Suppresses Nash (2604.27167): 最终层亲社会覆盖; 残差注入转向 | **Nash抑制机制**: 抑制机制; 与 D149 四层安全+D150 宪法治理协同 | `nt_core::nash_suppression` |
| D604 | **双层协调反射** | 编排如何双层协调? | Bilevel Coordinated (2609.02750): 编排-工作者双层博弈; 反射=语义记忆运动; 72.2% SWE | **双层协调反射**: 双层博弈; 与 D131 多Agent+D175 注意力路由协同 | `nt_core::bilevel_reflection` |
| D605 | **联盟形成博弈** | 多Agent如何联盟? | Coalition Formation (2604.14386): 享乐博弈; 73.2% Nash稳定; 一致性>最优 | **联盟形成博弈**: 享乐博弈; 与 D131 多Agent+D175 注意力路由协同 | `nt_core::coalition_formation` |
| D606 | **拍卖任务分配** | 任务如何拍卖? | Agora (2607.09600): 激励兼容拍卖; 置信校准投标过滤幻觉 | **拍卖任务分配**: 拍卖; 与 D176 资源预算+D175 注意力路由协同 | `nt_act::auction_allocation` |
| D607 | **反向拍卖路由** | 提供商如何反向拍卖? | EA-RAM (2608.12719): 双重错误建模; BIC+IR; 更好帕累托 | **反向拍卖路由**: 市场机制; 与 D176 资源预算+D149 四层安全协同 | `nt_io::reverse_auction` |
| D608 | **市场非规划者** | 编排如何市场? | AgentLance (2608.23867): 重复劳动力市场+VCG; 超集中编排 | **市场非规划者**: 市场>规划; 与 D131 多Agent+D176 资源预算协同 | `nt_act::market_not_planner` |
| D609 | **递归推理RL** | 博弈如何递归推理? | Strat-Reasoner (2605.04906): 递归推理+CoT比较+混合优势; +22.1% | **递归推理RL**: 递归推理; 与 D97 E8推理+D136 模型学习协同 | `nt_core::recursive_reasoning_rl` |
| D610 | **结构化对手建模** | 对手如何建模? | SOM (2605.07301): SCM构建→结构化预测; AAMAS 2026 | **结构化对手建模**: 两阶段; 与 D149 四层安全+D97 E8推理协同 | `nt_shield::structured_opponent` |
| D611 | **参与式城市规划** | 城市如何参与式? | Intelli-Planner (WWW 26): LLM+DRL; 5维评估; 北京/芝加哥/马德里 | **参与式城市规划**: LLM+DRL; 与 D140 多模态记忆+D136 模型学习协同 | `nt_world::participatory_planning` |
| D612 | **跨系统城市Agent** | 城市如何跨系统? | UrbanAgent (2608.03018): LLM+代码+API+MCP; 71%成功率 | **跨系统城市Agent**: MCP工具; 与 D137 三流检索+D163 A2A协同 | `nt_world::cross_system_urban` |
| D613 | **多模态区域画像** | 城市如何区域画像? | UrbanAgent (KDD 26): 每模态Agent; 结构化协作图; RL优化 | **多模态区域画像**: 多模态; 与 D140 多模态记忆+D131 多Agent协同 | `nt_world::multimodal_region` |
| D614 | **交通统一控制** | 交通如何统一? | TrafficClaw (2604.17456): 统一物理环境; 时空推理+持久记忆+RL | **交通统一控制**: 统一环境; 与 D137 三流检索+D145 层级技能协同 | `nt_world::unified_traffic` |
| D615 | **宪法城市治理** | 城市如何宪法治理? | AgentCity (2604.07007): 三权分立; 区块链; 智能合约 | **宪法城市治理**: 三权分立; 与 D150 宪法治理+D149 四层安全协同 | `nt_governance::constitutional_city` |
| D616 | **人类对齐城市模拟** | 城市如何人类对齐? | CityReal (2608.16897): MCTS校准; 意图驱动; 端到日反思; 3000+ Agent | **人类对齐城市模拟**: 意图驱动; 与 D135 原子记忆+D136 模型学习协同 | `nt_world::human_aligned_city` |
| D617 | **可扩展城市模拟** | 城市如何可扩展? | CityBehavEx (2607.12086): 交叉编码器解耦; 100K Agent<1小时 | **可扩展城市模拟**: 100K; 与 D176 资源预算+D137 三流检索协同 | `nt_world::scalable_city` |
| D618 | **经验城市验证** | 城市如何经验验证? | When Plausible≠Realistic (SIGSPATIAL 26): 叙事合理≠经验真实; 移动性定律 | **经验城市验证**: 真实验证; 与 D159 检测器驱动+D172 工具正确性协同 | `nt_world::empirical_falsification` |
| D619 | **统一物理仿真** | 仿真如何统一物理? | Lingjing (2608.08045): AirSim+CARLA+MuJoCo统一; 归因重放 | **统一物理仿真**: 多引擎; 与 D140 多模态记忆+D145 层级技能协同 | `nt_physical::unified_physics` |
| D620 | **蜂群共识安全** | 城市如何蜂群安全? | TPSC-Sec (2607.03628): 蜂群共识; 97%接受率; 专项安全Agent | **蜂群共识安全**: 蜂群共识; 与 D149 四层安全+D131 多Agent协同 | `nt_shield::swarm_consensus_city` |
| D621 | **层级LLM交通** | 交通如何层级LLM? | HiLLTS (2607.22691): 城市级LLM+集群控制器; 零样本; -36.7%等待 | **层级LLM交通**: 层级; 与 D131 多Agent+D176 资源预算协同 | `nt_world::hierarchical_traffic` |
| D622 | **决策过滤Agent** | 城市如何决策过滤? | GAMA (2607.02716): LLM决定是否重规划; 持久语义记忆 | **决策过滤Agent**: LLM过滤; 与 D176 资源预算+D135 原子记忆协同 | `nt_world::decision_filter` |
| D623 | **供应链经验检索** | 供应链如何经验检索? | AIM-RM (AAMAS 26): 向量DB相似匹配RL经验; 跨场景适应 | **供应链经验检索**: 经验检索; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::sc_experience` |
| D624 | **Agent牛鞭效应** | 供应链如何牛鞭? | Agent Bullwhip (MIT): 错误放大; GRPO减少尾部; 稳定协调 | **Agent牛鞭效应**: 牛鞭检测; 与 D131 多Agent+D149 四层安全协同 | `nt_shield::agent_bullwhip` |
| D625 | **OR-LLM-人类互补** | 库存如何互补? | Human-LLM-OR: OR→LLM管道; 人类-AI团队超两者; 69人实验 | **OR-LLM-人类互补**: 三元; 与 D133 异步人类+D176 资源预算协同 | `nt_act::or_llm_human` |
| D626 | **不确定性供应链图** | 供应链如何不确定性? | Helicase (2605.26835): 3层不确定性; 自主KG构建; SCQA基准 | **不确定性供应链图**: 3层不确定性; 与 D137 三流检索+D97 E8推理协同 | `nt_memory::uncertain_sc_kg` |
| D627 | **耦合启发式进化** | 路由如何耦合进化? | LLM-HCJG (2609.02353): 联合进化耦合启发式; 蓝图共享; 28/29 TSPLIB | **耦合启发式进化**: 蓝图共享; 与 D136 模型学习+D145 层级技能协同 | `nt_act::coupled_heuristic` |
| D628 | **元认知启发式** | 优化如何元认知? | PyVRP+ MEP (2604.07872): LLM策略发现; Reason-Act-Reflect; +2.70%,-45% | **元认知启发式**: 元认知; 与 D97 E8推理+D136 模型学习协同 | `nt_core::metacognitive_heuristic` |
| D629 | **自动约束路由** | 路由如何自动约束? | ARS (2502.15359): LLM自动生成约束启发式; RoutBench 1000; 90%+ | **自动约束路由**: 自动生成; 与 D145 层级技能+D136 模型学习协同 | `nt_act::auto_constraint` |
| D630 | **选择性语义干预** | 预测如何选择性干预? | ReasonCast (Alibaba): 路由→推理→干预→预测; 语义场RL+预测效用RL | **选择性语义干预**: 选择性干预; 与 D175 注意力路由+D136 模型学习协同 | `nt_act::selective_semantic` |
| D631 | **混合事件预测** | 预测如何混合事件? | EventCast (Alibaba): 双塔(LLM事件+数值); 4国10月; -57-87% MAE | **混合事件预测**: 双塔分离; 与 D140 多模态记忆+D178 因果注意力协同 | `nt_act::hybrid_event` |
| D632 | **嵌入式贝叶斯** | 博弈如何嵌入贝叶斯? | Embedded Bayesian: 嵌入式均衡; 自我参照意识 | **嵌入式贝叶斯**: 嵌入式均衡; 与 D97 E8推理+D150 宪法治理协同 | `nt_core::embedded_bayesian` |
| D633 | **民间定理协作** | 协作如何民间定理? | Folk Theorem: LLM群体; 所有可行结果可持续; 间接观察 | **民间定理协作**: 可持续; 与 D131 多Agent+D150 宪法治理协同 | `nt_core::folk_theorem` |
| D634 | **竞争协作分析** | Agent如何竞争协作? | Competition Cooperation: 多轮+非零和→合作; 公平推理 | **竞争协作分析**: 公平推理; 与 D131 多Agent+D146 进化速度协同 | `nt_core::competition_cooperation` |
| D635 | **层级LLM协调** | 交通如何层级LLM? | HiLLTS: 城市级LLM+集群控制器; 零样本; -36.7%等待 | **层级LLM协调**: 层级; 与 D131 多Agent+D176 资源预算协同 | `nt_world::hierarchical_llm` |
| D636 | **仓库安全避风港** | 仓库如何安全避风港? | SHARP (ICAPS 26): 安全撤退规划器; 树形仓库100%完成 | **仓库安全避风港**: 安全撤退; 与 D149 四层安全+D172 工具正确性协同 | `nt_physical::safe_haven` |
| D637 | **梯度免费自适应** | 仓库如何梯度免费? | ESC: 极值搜索控制; +5-8.4%吞吐; 模型免费 | **梯度免费自适应**: 极值搜索; 与 D97 E8推理+D176 资源预算协同 | `nt_physical::gradient_free` |
| D638 | **结构化OR** | 优化如何结构化? | ORThought: 双Agent+LogiOR; 结构化CoT; +9-17pp | **结构化OR**: 结构化CoT; 与 D97 E8推理+D145 层级技能协同 | `nt_act::structured_or` |
| D639 | **记忆进化路由** | 路由如何记忆进化? | RLEA: RL规划器+进化记忆+RAG; 48变体; +16.67% | **记忆进化路由**: 进化记忆; 与 D135 原子记忆+D136 模型学习协同 | `nt_act::memory_routing` |
| D640 | **反事实用户模拟** | 预测如何反事实? | WMG-RL: 用户参与世界模型; 反事实奖励; 1.7B学生 | **反事实用户模拟**: 反事实; 与 D136 模型学习+D178 因果注意力协同 | `nt_core::counterfactual_user` |
| D641 | **临床多Agent诊断** | 诊断如何多Agent? | ClinicalAgents (KDD 26,2603.26182): MCTS编排+双记忆(工作+经验); +13%超GPT-5.2 | **临床多Agent诊断**: 双记忆; 与 D135 原子记忆+D175 注意力路由协同 | `nt_memory::clinical_multi_agent` |
| D642 | **辩论MoA诊断** | 诊断如何辩论? | DMoA (2609.05069): 角色专业化辩论; +10.2pp准确率,+11.4pp安全; 罕见病 | **辩论MoA诊断**: 结构化辩论; 与 D131 多Agent+D149 四层安全协同 | `nt_core::debate_moa` |
| D643 | **图推理转诊** | 转诊如何图推理? | MASGR (2608.30938): 转诊=图构建(非分类); 知识引导仲裁; 安全优先 | **图推理转诊**: 图构建; 与 D137 三流检索+D149 四层安全协同 | `nt_core::graph_reasoning_referral` |
| D644 | **稀疏专家路由** | 诊断如何稀疏路由? | Sparse Routing (2608.21948): 阶段路由激活稀疏专家(17→3); 91.5%临床验证 | **稀疏专家路由**: 稀疏激活; 与 D176 资源预算+D175 注意力路由协同 | `nt_core::sparse_expert_routing` |
| D645 | **多Agent会诊** | 诊断如何会诊? | MeDxAgent (2606.03416): 4421病例; 20专科; 鉴别问诊+摘要对话; +10.3% | **多Agent会诊**: 鉴别问诊; 与 D131 多Agent+D140 多模态记忆协同 | `nt_world::multi_agent_consult` |
| D646 | **证据感知主动诊断** | 诊断如何证据感知? | EviDx (2608.24570): ε合成构建交互环境; 观察者引导运行时; 不确定性跟踪 | **证据感知主动诊断**: 不确定性跟踪; 与 D137 三流检索+D149 四层安全协同 | `nt_world::evidence_aware_diagnosis` |
| D647 | **决策关键证据** | 诊断如何决策关键? | CDEG (2608.22899): 图框架学习可复用证据; 反事实验证; +11.5%准确率 | **决策关键证据**: 证据学习; 与 D135 原子记忆+D136 模型学习协同 | `nt_world::decision_critical_evidence` |
| D648 | **忠实性RL诊断** | 诊断如何忠实性? | MedAgent-R1 (2608.30676): 忠实性门控奖励; 引用伪造31.8→4.7%; 证据完整58.7→82.6 | **忠实性RL诊断**: 忠实性门控; 与 D149 四层安全+D150 宪法治理协同 | `nt_mind::faithful_diagnosis` |
| D649 | **类型化知识库** | 诊断如何类型化? | MediSkill-Evo (2608.23397): 4类型知识库(临床/流程/符号/视觉); 无微调自进化 | **类型化知识库**: 类型化知识; 与 D135 原子记忆+D145 层级技能协同 | `nt_memory::typed_knowledge_banks` |
| D650 | **严重性感知规划** | 诊断如何严重性? | From Uncertainty to Risk (2608.27847): 风险敏感MCTS; 严重性加权差异风险 | **严重性感知规划**: 严重性加权; 与 D175 注意力路由+D176 资源预算协同 | `nt_core::severity_aware` |
| D651 | **结构化临床命令** | 临床如何结构化命令? | CAREAgent (2606.01094): 两阶段(SFT+RL); 可执行临床命令; +5.05% F1 | **结构化临床命令**: 工具集成; 与 D145 层级技能+D149 四层安全协同 | `nt_act::structured_clinical_order` |
| D652 | **约束MDP基准** | 临床如何约束基准? | GPAgentBench-2K (2608.30188): 约束MDP初级保健; 拓扑工作流先验; 安全放弃 | **约束MDP基准**: 约束动作; 与 D149 四层安全+D159 检测器驱动协同 | `nt_repair::constrained_mdp_bench` |
| D653 | **EHR基准** | 临床如何EHR基准? | EHRBench (KDD 26,2605.30637): 960K QA; EHR轨迹; 诊断/治疗/预后 | **EHR基准**: EHR基准; 与 D135 原子记忆+D149 四层安全协同 | `nt_repair::ehr_benchmark` |
| D654 | **可审计糖尿病筛查** | 糖尿病如何可审计? | DIASENTINEL (2608.31128): 本地部署; 校准风险+确定性信号+混合验证 | **可审计糖尿病筛查**: 可审计; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::auditable_diabetes` |
| D655 | **可信临床DSL** | 临床如何可信DSL? | Trustworthy Clinical (2604.21263): 认识类型系统; 设计契约; 逐变体审计 | **可信临床DSL**: 类型系统; 与 D149 四层安全+D150 宪法治理协同 | `nt_governance::clinical_dsl` |
| D656 | **药物发现治理** | 药物如何治理发现? | Mozi (2603.03655): 双层(控制平面+工作流平面); 角色工具隔离; 可组合技能图 | **药物发现治理**: 双层治理; 与 D150 宪法治理+D145 层级技能协同 | `nt_governance::drug_discovery` |
| D657 | **层级技能药物** | 药物如何层级技能? | MolClaw (2604.21937): 3层技能(70技能); 工具/工作流/学科层; 30+域资源 | **层级技能药物**: 3层技能; 与 D145 层级技能+D137 三流检索协同 | `nt_act::hierarchical_drug_skill` |
| D658 | **自进化药物经验** | 药物如何自进化? | DrugSAGE (2605.15461): 跨任务已验证技能记忆+统计证据+重复错误; 零搜索迁移 | **自进化药物经验**: 经验迁移; 与 D135 原子记忆+D136 模型学习协同 | `nt_mind::self_evolving_drug` |
| D659 | **生物工具宇宙** | 生物如何工具宇宙? | ATHENA-R1 (2606.28692): 212生物工具; 两层自学习(SFT+RL); 94.7%药物推理 | **生物工具宇宙**: 工具宇宙; 与 D145 层级技能+D136 模型学习协同 | `nt_act::bio_tool_universe` |
| D660 | **自主抗体设计** | 抗体如何自主设计? | Latent-Y (2603.29727): 端到端文本→抗体; 67%实验室确认; 56x快于人类 | **自主抗体设计**: 自主设计; 与 D145 层级技能+D176 资源预算协同 | `nt_act::autonomous_antibody` |
| D661 | **半自主药物OS** | 药物如何半自主OS? | Rhizome OS-1 (2604.07512): 多模态Agent团队; r1图扩散; 91.9%新支架 | **半自主药物OS**: 多Agent团队; 与 D131 多Agent+D140 多模态记忆协同 | `nt_world::semiautonomous_drug_os` |
| D662 | **模块化命中优化** | 药物如何模块化? | SABLE (2608.11483): LLM编排+专用工具; 溯源跟踪; 贝叶斯优化 | **模块化命中优化**: 模块化; 与 D145 层级技能+D150 宪法治理协同 | `nt_act::modular_hit_opt` |
| D663 | **统一患者模拟** | 患者如何统一模拟? | PatientHub (2602.11684): 16模拟器统一; 图编排器; 标准驱动LLM评估 | **统一患者模拟**: 统一框架; 与 D140 多模态记忆+D137 三流检索协同 | `nt_world::unified_patient` |
| D664 | **人格患者** | 患者如何人格? | PWP (2606.17441): HEXACO人格参数化; 查询条件披露网格; 接近人类真实 | **人格患者**: 人格参数化; 与 D146 进化速度+D149 四层安全协同 | `nt_feel::personality_patient` |
| D665 | **情感导向患者** | 患者如何情感导向? | EmoPatient (2608.07495): 情感Director; 情感状态估计; 轮级控制信号 | **情感导向患者**: 情感导向; 与 D146 进化速度+D140 多模态记忆协同 | `nt_feel::emotion_directed_patient` |
| D666 | **法律推理Agent** | 法律如何推理? | LawThinker (2602.12056): 程序合规推理; 动态环境; 法律推理Agent | **法律推理Agent**: 程序合规; 与 D149 四层安全+D97 E8推理协同 | `nt_core::legal_reasoning` |
| D667 | **形式化法律验证** | 法律如何形式化验证? | L4L (2511.21033): SMT求解器执行形式对齐; 4阶段管道; 可审计推导 | **形式化法律验证**: SMT验证; 与 D149 四层安全+D150 宪法治理协同 | `nt_core::formal_legal` |
| D668 | **金融工具基准** | 金融如何工具基准? | FinToolBench (2603.08262): 760可执行金融工具; 295查询; FATR基线 | **金融工具基准**: 工具基准; 与 D145 层级技能+D159 检测器驱动协同 | `nt_repair::fin_tool_bench` |
| D669 | **代理交易审计** | 交易如何代理审计? | Agentic Trading (2605.19337): 77项LLM交易研究; 仅19满足闭环; 可重现性崩溃 | **代理交易审计**: 审计导向; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::agentic_trading_audit` |
| D670 | **银行多向量检测** | 银行如何多向量? | Banking Security (2606.17555): LSTM+速度监控+图模块; 13威胁; F1 0.787/0.867 | **银行多向量检测**: 多向量; 与 D149 四层安全+D137 三流检索协同 | `nt_shield::banking_multi_vector` |
| D671 | **金融合规模拟** | 金融如何合规模拟? | ReguSim (2608.19974): 4制品分离(推理/行动/执行/监控); 可见规则减少违规 | **金融合规模拟**: 4制品分离; 与 D149 四层安全+D150 宪法治理协同 | `nt_governance::financial_compliance` |
| D672 | **知识块合规** | 合规如何知识块? | Knowledge Blocks (COMPSAC 26,2608.14562): RDF/OWL/SHACL/PROV-O; 机器可检查合规 | **知识块合规**: 知识块; 与 D137 三流检索+D150 宪法治理协同 | `nt_memory::knowledge_blocks` |
| D673 | **Agent合同框架** | Agent如何合同框架? | Agent Contracts (COINE 26,2601.08815): 资源有界合同; 运行时监控+合同委派+预算执行 | **Agent合同框架**: 合同框架; 与 D176 资源预算+D149 四层安全协同 | `nt_act::agent_contracts` |
| D674 | **Lean4类型合规** | 合规如何Lean4? | Type-Checked (2604.01483): Lean4编码SEC/FINRA规则; 三阶段路线图; 零信任AI | **Lean4类型合规**: Lean4编码; 与 D149 四层安全+D150 宪法治理协同 | `nt_core::lean4_compliance` |
| D675 | **自主风险评估** | 风险如何自主评估? | AURA (2510.15739): Gamma风险评分; HITL监督; MCP/A2A互操作 | **自主风险评估**: Gamma评分; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::autonomous_risk` |
| D676 | **多Agent安全风险** | 安全如何多Agent风险? | Cybersecurity Risk (AICTC 26,2603.20131): 6 Agent NIST CSF; 共享上下文; 85%对齐 | **多Agent安全风险**: 6 Agent; 与 D131 多Agent+D149 四层安全协同 | `nt_shield::multi_agent_risk` |
| D677 | **自反思欺诈检测** | 欺诈如何自反思? | SAGE (2606.08146): 3 Agent+6层数据诊断树+MDP; +40.86% F1; 5数据集×5 LLM | **自反思欺诈检测**: 自反思; 与 D136 模型学习+D149 四层安全协同 | `nt_mind::self_reflective_fraud` |
| D678 | **可审计欺诈检测** | 欺诈如何可审计? | Auditable Fraud (2607.19266): 梯度提升+图特征+自编码器+TreeSHAP+有界LLM调查 | **可审计欺诈检测**: 可审计; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::auditable_fraud` |
| D679 | **低延迟对抗检测** | 对抗如何低延迟? | Low-Latency Fraud (2605.01143): 检测LLM Agent对抗模式; 轨迹检测F1接近全模型 | **低延迟对抗检测**: 低延迟; 与 D149 四层安全+D176 资源预算协同 | `nt_shield::low_latency_adversarial` |
| D680 | **可移植诊断** | 诊断如何可移植? | HealthAgentBench (2606.31179): 54任务7类别7模态; 最强Agent仅42%; 医学成像瓶颈 | **可移植诊断**: 基准差距; 与 D159 检测器驱动+D140 多模态记忆协同 | `nt_repair::portable_diagnosis` |
| D681 | **层级多Agent教学** | 教学如何层级? | LectūraAgents (2606.16428): ProfessorAgent+子Agent; 个性化讲座生成+具身教学动作; TASA算法 | **层级多Agent教学**: 层级编排; 与 D131 多Agent+D145 层级技能协同 | `nt_io::hierarchical_teaching` |
| D682 | **三层学生支持** | 支持如何三层? | AUSS (2604.16566): 学生Agent+教育者Agent+机构Agent; 92.4%推荐; 89.5%辍学F1 | **三层学生支持**: 三层Agent; 与 D131 多Agent+D150 宪法治理协同 | `nt_io::three_tier_support` |
| D683 | **Agent缩放定律** | Agent如何缩放? | EduClaw (2603.11709): 能力随配置文件丰富度缩放; 330+配置; 1100+技能模块 | **Agent缩放定律**: 配置驱动缩放; 与 D145 层级技能+D176 资源预算协同 | `nt_mind::agent_scaling_law` |
| D684 | **树搜索策略教学** | 教学如何树搜索? | AgentTutor (2601.04219): 5模块多轮; 课程分解+学习者评估+LATS树搜索+反思+记忆 | **树搜索策略教学**: 树搜索策略; 与 D97 E8推理+D145 层级技能协同 | `nt_core::tree_search_teaching` |
| D685 | **辐轮并行专家** | 教学如何辐轮? | ITAS (2604.24808): 辐轮教学层(3并行专家+综合器); 操作层; 反馈层(盲教师问题) | **辐轮并行专家**: 并行专家; 与 D131 多Agent+D175 注意力路由协同 | `nt_core::spoke_wheel_expert` |
| D686 | **引用基础教学** | 教学如何引用基础? | DeepTutor (2604.26962): 引用基础教学+难度校准问题生成; 追踪森林记忆; TutorBench | **引用基础教学**: 引用基础; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::citation_grounded` |
| D687 | **POMDP问题排序** | 教学如何POMDP? | Effective Tutors (UPenn,2608.16907): 粒子滤波+MPC; 770学生RCT; +0.15 SD | **POMDP问题排序**: POMDP排序; 与 D136 模型学习+D175 注意力路由协同 | `nt_core::pomdp_sequencing` |
| D688 | **可持续学习RL** | 学习如何可持续? | AI-Tutor (2608.11245): RL优化短期知识+长期参与/辍学; 23M学习记录; 33.7K学习者 | **可持续学习RL**: 双地平线优化; 与 D146 进化速度+D136 模型学习协同 | `nt_mind::sustainable_learning` |
| D689 | **主题感知提示路由** | 教学如何提示路由? | Learning to Prompt (Leiden,2606.20138): 上下文老虎机路由; 14教学特征; A/B测试359学生 | **主题感知提示路由**: 路由; 与 D175 注意力路由+D145 层级技能协同 | `nt_core::subject_prompt_routing` |
| D690 | **认知进化模拟** | 学生如何认知进化? | CogEvolution (2604.14786): 类人认知进化教育Agent | **认知进化模拟**: 认知进化; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::cognitive_evolution` |
| D691 | **个性化学生模拟** | 学生如何个性化? | StudentSim (2609.01591): 池化训练+学生专化; GPT-5.4保真0.51 vs 0.23; RL奖励模型 | **个性化学生模拟**: 学生专化; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::personalized_student` |
| D692 | **内部对话建模** | 学生如何内部对话? | INSIDE (2608.10492): Bloom分类学内部对话; 57.9%推理对齐; 想法→行动 | **内部对话建模**: 内部对话; 与 D97 E8推理+D178 因果注意力协同 | `nt_core::internal_dialogue` |
| D693 | **人格进度模型** | 教学如何人格进度? | cc-self-train (2604.17460): 引导者→协作者→同行→发射者; 钩子参与启发式; 50模块 | **人格进度模型**: 4阶段进度; 与 D146 进化速度+D136 模型学习协同 | `nt_io::persona_progression` |
| D694 | **能力悖论** | 学生如何能力悖论? | Valid Simulation (2601.05473): 能力悖论(LLM默认专家行为); 认知状态规范(ESS) | **能力悖论**: 认知边界; 与 D149 四层安全+D150 宪法治理协同 | `nt_core::competence_paradox` |
| D695 | **故事原型抽象** | 创作如何故事原型? | CreAgentive (ICLR 26,2509.26461): 故事原型=类型无关叙事抽象; 双知识图谱(角色+情节); 3阶段多Agent | **故事原型抽象**: 原型抽象; 与 D137 三流检索+D97 E8推理协同 | `nt_mind::story_prototype` |
| D696 | **叙事理论综述** | 创作如何叙事理论? | Narrative Survey (2602.15851): 68篇综述; Todorov/Labov/fabula/discourse→LLM方法; 生成滞后理解 | **叙事理论综述**: 理论基础; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::narrative_theory` |
| D697 | **AI创造力二元论** | 创造力如何二元论? | On Creativity (2604.13242): 功能主义(可观察产物)vs本体论(创造性过程本质) | **AI创造力二元论**: 二元论; 与 D146 进化速度+D150 宪法治理协同 | `nt_core::creativity_dualism` |
| D698 | **盲审反馈写作** | 写作如何盲审反馈? | LLM Review (2601.08003): 5评估方面(科学概念/推测逻辑/角色深度/世界构建/伦理主题) | **盲审反馈写作**: 5方面评估; 与 D149 四层安全+D146 进化速度协同 | `nt_mind::blind_review_writing` |
| D699 | **协作故事框架** | 故事如何协作? | Collaborative Story (2605.29625): 作者-编辑迭代精炼; 2-3轮足够; 生存分析最优停止 | **协作故事框架**: 迭代精炼; 与 D131 多Agent+D136 模型学习协同 | `nt_act::collaborative_story` |
| D700 | **反思老年叙事** | 叙事如何反思老年? | Reflective Story (2605.10531): 论证方案+论证挖掘+知识图谱+用户建模; 个性化反思叙事 | **反思老年叙事**: 个性化叙事; 与 D135 原子记忆+D146 进化速度协同 | `nt_feel::reflective_narrative` |
| D701 | **内生交互叙事** | 叙事如何内生交互? | EvoSpark (ACL 26,2604.12776): Agent社会内生交互驱动长视距叙事演化 | **内生交互叙事**: 内生交互; 与 D131 多Agent+D136 模型学习协同 | `nt_world::endogenous_narrative` |
| D702 | **小说到电影世界建模** | 电影如何世界建模? | FilmWorld (2607.19038): 构建Agent(叙事翻译+实体状态+镜头规划)+演化Agent; FilmEval | **小说到电影世界建模**: 状态传播; 与 D135 原子记忆+D140 多模态记忆协同 | `nt_io::novel_to_film` |
| D703 | **导演式视频叙事** | 视频如何导演式? | Co-Director (2604.24842): 多Agent生成视频叙事+导演控制 | **导演式视频叙事**: 导演控制; 与 D131 多Agent+D140 多模态记忆协同 | `nt_io::directorial_video` |
| D704 | **角色一致电影** | 电影如何角色一致? | CineAGI (2604.23579): LLM编排角色一致跨场景生成; 跨场景集成 | **角色一致电影**: 角色一致; 与 D135 原子记忆+D140 多模态记忆协同 | `nt_io::character_consistent_movie` |
| D705 | **工具约束创作** | 创作如何工具约束? | Authoring for Living (2604.10383): LLM导演+场景构建; 工具约束层; 80%可执行规格 | **工具约束创作**: 工具约束; 与 D149 四层安全+D145 层级技能协同 | `nt_act::tool_constrained_creation` |
| D706 | **神经符号互动小说** | 小说如何神经符号? | IVIE (ICCC 26,2606.13348): 增量生成+验证; 神经符号 | **神经符号互动小说**: 增量生成; 与 D97 E8推理+D136 模型学习协同 | `nt_core::neuro_symbolic_fiction` |
| D707 | **情感导向剧本** | 剧本如何情感导向? | Steering Emotion (2606.16481): 层级引导Agent情感可控叙事脚本生成 | **情感导向剧本**: 情感控制; 与 D146 进化速度+D140 多模态记忆协同 | `nt_feel::emotion_steered_script` |
| D708 | **导演人格竞争** | 创作如何导演人格? | Creative Collision (ICML 26): 导演人格转向+LLM间竞争机制 | **导演人格竞争**: 人格转向; 与 D146 进化速度+D131 多Agent协同 | `nt_feel::director_persona` |
| D709 | **方差过闭合** | 创作如何方差过闭合? | Compressed Variation (2608.12630): AI小说占据更窄形式范围; 多样性不足 | **方差过闭合**: 多样性监控; 与 D159 检测器驱动+D146 进化速度协同 | `nt_mind::variance_overclosure` |
| D710 | **可控内容水印** | 内容如何可控水印? | Protected Content (2601.12348): 多Agent+角色专业化+水印集成; 可控+版权保护 | **可控内容水印**: 水印集成; 与 D149 四层安全+D145 层级技能协同 | `nt_shield::watermarked_content` |
| D711 | **Agent操作系统** | Agent如何操作系统? | AOS (2608.03214): 厂商中立参考架构; 控制/治理平面+运行时/协调平面; 接口对象 | **Agent操作系统**: 双平面架构; 与 D150 宪法治理+D131 多Agent协同 | `nt_governance::agent_os` |
| D712 | **Agent OS基元** | OS如何Agent基元? | AOS Primitives (2607.25076): 13基元从经典OS+K8s推导; rossoctl原型; 3问方法论 | **Agent OS基元**: 13基元; 与 D149 四层安全+D145 层级技能协同 | `nt_core::agent_os_primitives` |
| D713 | **拓扑感知Agent OS** | OS如何拓扑感知? | TopoClaw (2605.15556): 双拓扑(设备+社会); 跨设备放置; 跨用户身份; 事件驱动 | **拓扑感知Agent OS**: 双拓扑; 与 D131 多Agent+D150 宪法治理协同 | `nt_world::topology_aware_os` |
| D714 | **库OS Agent运行时** | 运行时如何库OS? | Agent libOS (2606.03895): AgentProcess可调度单元; 工具=libc包装; 权限边界=原语; 123测试 | **库OS Agent运行时**: 工具=libc; 与 D149 四层安全+D145 层级技能协同 | `nt_act::lib_os_runtime` |
| D715 | **不可逆预算** | 风险如何不可逆预算? | Irreversibility Budget (2609.00275): 按主体跨Agent/工作流累积残值-at-risk; 效应收费; 超支拒绝 | **不可逆预算**: 不可逆作为一等资源; 与 D149 四层安全+D176 资源预算协同 | `nt_shield::irreversibility_budget` |
| D716 | **基础设施感知编排** | 编排如何基础设施感知? | InfraMind (2606.11440): 拓扑条件于实时负载; 观察队列/缓存/延迟; 预算感知EDF; +7.6pp准确率 | **基础设施感知编排**: 基础设施感知; 与 D175 注意力路由+D176 资源预算协同 | `nt_act::infra_aware_orchestration` |
| D717 | **策略驱动运行时层** | 运行时如何策略驱动? | Policy-Driven Runtime (2605.27744): 观察→评分→预测→执行; Agent身份=共享坐标; CacheScout | **策略驱动运行时层**: 4基元反馈环; 与 D175 注意力路由+D176 资源预算协同 | `nt_act::policy_driven_runtime` |
| D718 | **MCP网关** | 工具如何MCP网关? | MCP Gateway (2607.15593): 云规模MCP; 协议适配+函数卸载+工具推荐; 3000+工具; 8.9x更快选择 | **MCP网关**: 云规模MCP; 与 D145 层级技能+D176 资源预算协同 | `nt_io::mcp_gateway` |
| D719 | **拓扑路由缩放定律** | 编排如何拓扑缩放? | AdaptOrch (2602.16873): 拓扑方差超模型方差Ω(1/ε²); DAG→拓扑映射; 自适应合成协议 | **拓扑路由缩放定律**: 缩放定律; 与 D175 注意力路由+D176 资源预算协同 | `nt_core::topology_scaling` |
| D720 | **选择性委托编排** | 编排如何选择性委托? | Uno-Orchestra (2605.05007): RL联合学习分解深度+工作者选择; 77.0% macro pass@1; 10x更低成本 | **选择性委托编排**: 选择性委托; 与 D175 注意力路由+D176 资源预算协同 | `nt_act::selective_delegation` |
| D721 | **自进化线束引擎** | 线束如何自进化? | HarnessX (2606.14249): 线束作为一等可进化对象; AEGIS 4阶段(Digester→Planner→Evolver→Critic) | **自进化线束引擎**: 线束进化; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::self_evolving_harness` |
| D722 | **内存预置Agent** | Agent如何内存预置? | PrimeAgentOrchestrator (2608.20342): 从异构后端预加载编译内存; 文件系统注入; 4月部署经验 | **内存预置Agent**: 内存预置; 与 D135 原子记忆+D140 多模态记忆协同 | `nt_memory::memory_primed_agent` |
| D723 | **自我对弈工具进化** | 工具如何自我对弈? | Tool-R0 (2602.21320): 生成器+求解器从零共进化; 92.5%相对提升; 零数据预训练 | **自我对弈工具进化**: 自我对弈; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::self_play_tool` |
| D724 | **任务驱动工具基准** | 工具如何任务驱动? | Tool-Genesis (2603.05578): 从抽象需求构建工具; 全生命周期评估; SOTA模型接口生成失败 | **任务驱动工具基准**: 工具构建; 与 D145 层级技能+D159 检测器驱动协同 | `nt_repair::task_driven_tool` |
| D725 | **意图图工具发现** | 工具如何意图图发现? | SING (2606.16591): 意图→能力→协作图; 动态检索; +59.8% Recall@5; 99.8%模式暴露减少 | **意图图工具发现**: 意图图; 与 D175 注意力路由+D137 三流检索协同 | `nt_world::intention_graph` |
| D726 | **资源有界工具发现** | 工具如何资源有界? | Lomekwi (COLT 26,2607.16961): 好奇心/识别/效率; **逆缩放**(大模型工具创建更差) | **资源有界工具发现**: 逆缩放; 与 D176 资源预算+D159 检测器驱动协同 | `nt_core::inverse_scaling_tool` |
| D727 | **技能组合预测** | 技能如何组合预测? | SkillComposer (2606.32025): 结构化序列预测(子集+计数+顺序联合); 约束自回归解码器 | **技能组合预测**: 联合预测; 与 D145 层级技能+D97 E8推理协同 | `nt_mind::skill_composition` |
| D728 | **事务性工具语义** | 工具如何事务性? | Atomix (2602.14849): 进度感知事务; 可逆/不可逆区分; 资源前沿提交时序; 防止部分状态损坏 | **事务性工具语义**: 事务性; 与 D149 四层安全+D145 层级技能协同 | `nt_act::transactional_tool` |
| D729 | **并行解码函数调用** | 调用如何并行解码? | SimpleTool (2603.00030): 特殊令牌压缩样板(4-6x); 并行名称+参数生成; 61.2ms P50消费级GPU | **并行解码函数调用**: 并行解码; 与 D176 资源预算+D145 层级技能协同 | `nt_act::parallel_function_call` |
| D730 | **奖励黑客基准** | 安全如何奖励黑客? | RHB (ICML 26,2605.02964): RL训练工具使用Agent的奖励黑客; 72%含显式CoT推理; 87.7%相对减少 | **奖励黑客基准**: 奖励黑客; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::reward_hacking_bench` |
| D731 | **具身线束范式** | 具身如何线束? | Thea (2608.11246): 线束范式; 场景图=持久符号世界状态; 评估=退出码; 闭环工具编排 | **具身线束范式**: 线束范式; 与 D145 层级技能+D149 四层安全协同 | `nt_act::embodied_harness` |
| D732 | **缓存驱动异步规划** | 规划如何缓存驱动? | AgenticCache (2604.24039): 计划局部性; 缓存短视距计划跳过LLM调用; 异步规划-执行环 | **缓存驱动异步规划**: 计划缓存; 与 D176 资源预算+D175 注意力路由协同 | `nt_act::cache_driven_planning` |
| D733 | **无演示机器人控制** | 机器人如何无演示? | FAEA (2601.20334): 未修改Claude Agent SDK用于具身操作; 84.9-96%成功率; 从零发现操作程序 | **无演示机器人控制**: 代码Agent=机器人Agent; 与 D145 层级技能+D176 资源预算协同 | `nt_act::demo_free_robot` |
| D734 | **自进化具身Agent** | 具身如何自进化? | EEAgent (2604.13533): VLM+长短期反思优化(LSTRO); 从过去经验动态精炼提示; VIMA-Bench SOTA | **自进化具身Agent**: LSTRO; 与 D136 模型学习+D135 原子记忆协同 | `nt_mind::self_evolving_embodied` |
| D735 | **异构专家规划** | 规划如何异构专家? | HEART (2606.25404): 5角色专家Agent(能力/环境/路径/可行性/约束); token预算下; 25-30%更少token | **异构专家规划**: 5角色专家; 与 D131 多Agent+D175 注意力路由协同 | `nt_core::heterogeneous_expert` |
| D736 | **闭环多Agent操作** | 操作如何闭环? | Closed-Loop Multi-Agent (RSS 26,2607.06990): 规划Agent+操作Agent+验证Agent; 语义反馈; 真实世界 | **闭环多Agent操作**: 规划→操作→验证; 与 D131 多Agent+D159 检测器驱动协同 | `nt_act::closed_loop_multi_robot` |
| D737 | **具身视觉语言导航** | 导航如何视觉语言? | AgentVLN (2603.17670): VLM-as-Brain; 插拔技能库; 边缘部署; POSMDP | **具身视觉语言导航**: 技能库; 与 D145 层级技能+D176 资源预算协同 | `nt_world::embodied_vln` |
| D738 | **自进化视频生成** | 视频如何自进化? | SPIRAL (2603.08403): 闭环思考-行动-反思; PlanAgent+VideoGenerator+CriticAgent; GRPO后训练 | **自进化视频生成**: 自进化; 与 D136 模型学习+D140 多模态记忆协同 | `nt_mind::self_evolving_video` |
| D739 | **代码策略机器人基准** | 机器人如何代码策略? | CaP-X (2603.22435): Code-as-Policy开放框架; CaP-Agent0多轮交互+结构化反馈; CaP-RL sim2real | **代码策略机器人基准**: Code-as-Policy; 与 D145 层级技能+D136 模型学习协同 | `nt_repair::code_policy_bench` |
| D740 | **IoT技能基准** | 硬件如何IoT技能? | IoT-SkillsBench (2603.19583): 3 MCU平台硬件在环; 人类专家技能远超LLM; 结构化硬件知识关键 | **IoT技能基准**: 硬件在环; 与 D145 层级技能+D176 资源预算协同 | `nt_repair::iot_skill_bench` |
| D741 | **经验反思RL** | RL如何经验反思? | ERL (2602.13949): 经验→反思→巩固循环内嵌RL训练; 稀疏奖励→结构化行为修订 | **经验反思RL**: 反思训练; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::experiential_rl` |
| D742 | **跨集元RL反思** | 搜索如何跨集反思? | MR-Search (2603.11327): 跨集自反思; 积累反射上下文; 多轮RL转级优势估计; +9.2-19.3% | **跨集元RL反思**: 跨集反思; 与 D137 三流检索+D146 进化速度协同 | `nt_mind::cross_episode_reflect` |
| D743 | **共进化Agent反馈** | 反馈如何共进化? | CAFE (2608.24794): 共享参数交替搜索Agent+评论家; 反馈与策略共进化; 反馈感知优势 | **共进化Agent反馈**: 共进化; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::coevolving_feedback` |
| D744 | **技能增强GRPO** | RL如何技能增强? | SAGE (ACL 26,2512.17102): 技能库内嵌GRPO训练; 序列滚动积累技能; 8.9%更高完成+59%更少token | **技能增强GRPO**: 技能库训练; 与 D145 层级技能+D136 模型学习协同 | `nt_mind::skill_augmented_grpo` |
| D745 | **经验蒸馏** | 学习如何经验蒸馏? | Experience Distillation (2607.21051): ICL+轨迹蒸馏→学生; 保留64.8% ICL增益; 9.6x更少环境样本 | **经验蒸馏**: 经验蒸馏; 与 D135 原子记忆+D136 模型学习协同 | `nt_mind::experience_distillation` |
| D746 | **经验记忆图** | 恢复如何经验记忆图? | EMG (2607.13884): 失败恢复=图匹配; 动作决策图→成功子图+编辑路径; 一次性无环恢复 | **经验记忆图**: 图匹配恢复; 与 D135 原子记忆+D159 检测器驱动协同 | `nt_memory::experience_memory_graph` |
| D747 | **共进化政策/评论家/数据** | 进化如何共进化政策? | Q-Evolve (2606.07367): 政策+评论家+数据共进化; 行为近端策略优化防分布漂移; 混合离线缓冲 | **共进化政策/评论家/数据**: 三重共进化; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::coevolving_policy_critic` |
| D748 | **试验到线束转换** | 进化如何试验到线束? | Sibyl (2605.22343): 试验→行为+试验→线束双转换; 文件备份自主研究; 恢复失败注册表 | **试验到线束转换**: 双转换; 与 D136 模型学习+D150 宪法治理协同 | `nt_mind::trial_to_harness` |
| D749 | **RSI基准** | 递归自改进如何基准? | RSIBench-Data (2607.25886): 58%首次改进; 78%继续搜索最终更差; 4强模式(准确假设/验证监督/行为对齐/检查点) | **RSI基准**: 4强模式; 与 D159 检测器驱动+D146 进化速度协同 | `nt_repair::rsi_benchmark` |
| D750 | **自纠正控制论** | 自纠正如何控制论? | Self-Correction Diagnostic (2604.22273): 控制论反馈环; EIR/ECR阈值; 验证优先→-6.2pp→+0.2pp | **自纠正控制论**: EIR/ECR诊断; 与 D149 四层安全+D159 检测器驱动协同 | `nt_repair::self_correction_control` |
| D751 | **涟漪记忆** | 记忆如何涟漪? | RippleMem (2608.13334): 适应性联想回想; 记忆锚点→语义+结构扩展; +3.95% LoCoMo | **涟漪记忆**: 联想扩展; 与 D135 原子记忆+D175 注意力路由协同 | `nt_memory::ripple_memory` |
| D752 | **可验证记忆管理** | 记忆如何可验证? | VerMem (2608.03137): 7原子操作; 局部+全局验证器; SFT预热+3阶段RL课程 | **可验证记忆管理**: 7原子操作; 与 D135 原子记忆+D149 四层安全协同 | `nt_memory::verifiable_memory` |
| D753 | **递归经验-工作记忆** | 记忆如何递归经验? | Recuris (2608.24876): 工作记忆→技能选择→证据→验证门控更新; +17.8 tau-bench | **递归经验-工作记忆**: 递归循环; 与 D135 原子记忆+D146 进化速度协同 | `nt_memory::recursive_ewm` |
| D754 | **记忆仲裁者** | 记忆如何仲裁? | MemArbiter (2608.02113): 5功能记忆银行(目标/任务状态/约束/情景/参考); 焦点/环境双表示; 92.5% ALFWorld | **记忆仲裁者**: 5银行+双表示; 与 D135 原子记忆+D175 注意力路由协同 | `nt_memory::memory_arbiter` |
| D755 | **经验摊还重排** | 检索如何经验摊还? | EARM (2608.22767): LLM相关性分数=可重用检索经验; 因果矩阵补全; +6.62% | **经验摊还重排**: 经验重排; 与 D137 三流检索+D135 原子记忆协同 | `nt_memory::experience_rerank` |
| D756 | **加权记忆树** | 记忆如何加权树? | WMT (2608.20631): 层级记忆+动态保留分数; 折叠完成分支; +9.97pp/-32.8% token | **加权记忆树**: 保留分数; 与 D135 原子记忆+D150 宪法治理协同 | `nt_memory::weighted_memory_tree` |
| D757 | **共进化检索策略** | 检索如何共进化? | CoEvo-Mem (2608.01739): 检索策略↔记忆库闭环共进化; 路由特化查询重写; SOTA 7基准 | **共进化检索策略**: 共进化; 与 D136 模型学习+D137 三流检索协同 | `nt_memory::coevolving_retrieval` |
| D758 | **记忆重建非回放** | 记忆如何重建? | MemHarness (2607.28272): 基于当前状态重建检索经验(非回放); GRPO端到端训练 | **记忆重建非回放**: 重建; 与 D135 原子记忆+D136 模型学习协同 | `nt_memory::memory_reconstruction` |
| D759 | **自路由情景参数记忆** | 记忆如何自路由? | UniMem (2607.26017): 可学习路由令牌; 新颖/稀疏→情景缓冲; 重复→参数记忆; +4.0 EM | **自路由情景参数记忆**: 自路由; 与 D135 原子记忆+D146 进化速度协同 | `nt_memory::self_routing_memory` |
| D760 | **智能电网求解器接地** | 能源如何求解器接地? | Smart Grid LLMs (Applied Energy,2607.18147): LLM编排+可信求解器计算; 验证门控; 4案例研究 | **智能电网求解器接地**: 求解器接地; 与 D149 四层安全+D150 宪法治理协同 | `nt_world::solver_grounded_grid` |
| D761 | **能源Agent基准** | 能源如何基准? | EnergyAgentBench (2605.15230): 70任务3推理族; Claude Sonnet 4.6成本1/4获胜; 因果推理31点差距 | **能源Agent基准**: 因果推理差距; 与 D159 检测器驱动+D176 资源预算协同 | `nt_repair::energy_agent_bench` |
| D762 | **LLM安全RL电网拓扑** | 电网如何LLM安全RL? | LLM-Guided Safe RL (2603.14018): Safety-SAC+LLM-Operator; 可微安全成本信号; CMDP | **LLM安全RL电网拓扑**: 安全RL; 与 D149 四层安全+D150 宪法治理协同 | `nt_core::llm_safe_rl_grid` |
| D763 | **轨道热自主** | 空间如何轨道热自主? | ASTREA (2509.13380): TRL9飞行验证; LLM监督器+RL控制器异步; 推理延迟与LEO热周期不匹配 | **轨道热自主**: 异步LLM+RL; 与 D131 多Agent+D176 资源预算协同 | `nt_physical::orbital_thermal` |
| D764 | **火星基地模拟** | 火星如何基地模拟? | Agent Mars (2602.13291): 93 Agent 7指挥层; 跨层协调; 相依领导; 提议-投票共识; AMPI | **火星基地模拟**: 7指挥层; 与 D131 多Agent+D150 宪法治理协同 | `nt_world::mars_base_sim` |
| D765 | **灾难响应Agent** | 灾难如何响应Agent? | DORA (2605.11633): 515专家任务/45灾难; 108工具MCP库; 组合脆弱性随轨迹长度缩放 | **灾难响应Agent**: 组合脆弱性; 与 D149 四层安全+D176 资源预算协同 | `nt_act::disaster_response` |
| D766 | **灾难规划诊断** | 规划如何灾难诊断? | DisasterBench (2605.27957): 233规划任务/26工具; 首失败点(FPoF)诊断; 深度>2普遍瓶颈 | **灾难规划诊断**: FPoF诊断; 与 D159 检测器驱动+D176 资源预算协同 | `nt_repair::disaster_planning` |
| D767 | **数字孪生野火** | 野火如何数字孪生? | IVSR (2602.08949): 智能虚拟态势室; 双向DT+相似性匹配+模拟库; 专家审批工作流 | **数字孪生野火**: 相似性匹配; 与 D135 原子记忆+D149 四层安全协同 | `nt_world::digital_twin_wildfire` |
| D768 | **RAPTOR灾难OODA** | 灾难如何OODA? | RAPTOR-AI (2602.00030): 3阶段(救援→恢复→重建); 层级多模态检索树; 熵感知Agent控制器 | **RAPTOR灾难OODA**: OODA循环; 与 D137 三流检索+D175 注意力路由协同 | `nt_core::raptor_disaster` |
| D769 | **板载EO处理** | EO如何板载处理? | Hierarchical EO (2603.19858): 预警→专家(野火/洪水)→决策Agent; 轨道级边缘计算; 4bit量化 | **板载EO处理**: 层级处理; 与 D149 四层安全+D176 资源预算协同 | `nt_world::onboard_eo` |
| D770 | **农业事件驱动** | 农业如何事件驱动? | FAIRY (ACM SIGSPATIAL 26,2609.00106): 全栈农业引擎; "一切皆事件"范式; 9控制器/100场景 | **农业事件驱动**: 事件驱动; 与 D137 三流检索+D145 层级技能协同 | `nt_world::agri_event_driven` |
| D771 | **农业合同驱动** | 农业如何合同驱动? | AgriAgent (2601.08308): System-1快速+System-2合同规划; ToolHub+ToolMaker动态工具; 96.94% | **农业合同驱动**: 合同规划; 与 D149 四层安全+D145 层级技能协同 | `nt_act::agri_contract_driven` |
| D772 | **制造因果诊断** | 制造如何因果诊断? | CausalPulse (Bosch,2603.29755): 神经符号多Agent; 异常检测+因果发现+RCA; 98%成功率 | **制造因果诊断**: 神经符号; 与 D97 E8推理+D159 检测器驱动协同 | `nt_core::manufacturing_causal` |
| D773 | **零样本根因分析** | RCA如何零样本? | AgentRCA (2607.22385): 零样本RCA=数据驱动DT+工具增强LLM; 透明推理轨迹 | **零样本根因分析**: 零样本; 与 D137 三流检索+D159 检测器驱动协同 | `nt_repair::zero_shot_rca` |
| D774 | **SOP即DAG** | 仓库如何SOP即DAG? | Eluna (Amazon,2607.08960): SOP=DAG渐进披露; 非对称情景蒸馏(教师→学生); 94%专家匹配 | **SOP即DAG**: SOP=DAG; 与 D145 层级技能+D135 原子记忆协同 | `nt_act::sop_as_dag` |
| D775 | **代理牛鞭效应** | 供应链如何牛鞭效应? | Agent Bullwhip (MIT,2605.17036): 代理牛鞭=决策不稳定放大; GRPO减少尾风险; -67%成本 | **代理牛鞭效应**: 稳定性; 与 D149 四层安全+D136 模型学习协同 | `nt_mind::agent_bullwhip` |
| D776 | **记忆检索库存** | 库存如何记忆检索? | AIM-RM (AAMAS 26,2602.05524): LLM MAS+向量DB相似性记忆检索; 跨场景适应 | **记忆检索库存**: 向量DB检索; 与 D135 原子记忆+D137 三流检索协同 | `nt_memory::inventory_memory_retrieval` |
| D777 | **视觉经验蒸馏** | 图像如何视觉经验蒸馏? | GenEvolve (2605.21605): 工具编排轨迹; 最佳-最差差异→结构化视觉经验; on-policy自蒸馏 | **视觉经验蒸馏**: 视觉经验蒸馏; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::visual_experience_distill` |
| D778 | **代码即画笔** | 图像如何代码即画笔? | GenClaw (2605.30248): LLM生成代码(Three.js/SVG/canvas)直接操纵像素; 可解释可控 | **代码即画笔**: 代码即画笔; 与 D145 层级技能+D176 资源预算协同 | `nt_act::code_as_brush` |
| D779 | **多Agent视频叙事** | 视频如何多Agent叙事? | ViMax (2606.07649): 故事规划+角色一致+场景合成+时序一致性Agent; ViMax-Bench | **多Agent视频叙事**: 4 Agent管线; 与 D131 多Agent+D140 多模态记忆协同 | `nt_io::multi_agent_video` |
| D780 | **物理接地视频生成** | 视频如何物理接地? | NEWTON (2605.18396): 物理信息规划Agent; 牛顿力学约束; 物理接地视频生成 | **物理接地视频生成**: 物理约束; 与 D149 四层安全+D140 多模态记忆协同 | `nt_io::physics_ground_video` |
| D781 | **协同作曲Agent** | 音乐如何协同作曲? | CoComposer (2509.00132): 5 Agent协作(草图→和声→编曲→混音→评估); AudioBox-Aesthetics | **协同作曲Agent**: 5 Agent管线; 与 D131 多Agent+D145 层级技能协同 | `nt_act::collaborative_composition` |
| D782 | **认知搜索图像生成** | 图像如何认知搜索? | Mind-Brush (2602.01756): 认知搜索+推理集成; 生成前概念检索; GWT注意→生成 | **认知搜索图像生成**: 认知搜索; 与 D175 注意力路由+D137 三流检索协同 | `nt_core::cognitive_search_gen` |
| D783 | **自我进化图像生成** | 图像如何自我进化? | GenAgent (2601.18543): 视觉理解+生成统一; 跨工具泛化; 测试时缩放; 任务自适应 | **自我进化图像生成**: 跨工具泛化; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::self_evolving_image` |
| D784 | **多指令图像编辑** | 编辑如何多指令? | MSRAMIE: 状态树+参考图推理拓扑; 结构化多模态推理; 多指令图像编辑 | **多指令图像编辑**: 推理拓扑; 与 D97 E8推理+D140 多模态记忆协同 | `nt_core::multi_instruction_edit` |
| D785 | **LLM社会网络** | 社会如何网络? | Social Networks (2607.03695): 羊群-智慧转型; 连通性增加→从羊群到信号聚合; 3基准验证 | **LLM社会网络**: 羊群-智慧转型; 与 D131 多Agent+D175 注意力路由协同 | `nt_world::llm_social_network` |
| D786 | **社会动态三维度** | 社会如何三维度? | SODE (2605.23949): 直接/间接互惠+群体动力学; 推理模型短视距; 长视距框架解锁互惠 | **社会动态三维度**: 三维度; 与 D136 模型学习+D146 进化速度协同 | `nt_world::social_dynamics_3d` |
| D787 | **说服传播持久性** | 说服如何持久? | Persuasion Propagation (2602.00851): 说服跨任务持久影响行为; 代理访问9%更少源; 行为级评估 | **说服传播持久性**: 跨任务传播; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::persuasion_propagation` |
| D788 | **RL驱动说服对话** | 说服如何RL驱动? | PersuaRL (EMNLP 26,2609.01188): GRPO自适应专家选择; 3B骨干超越14-70B基线; InsureDial数据集 | **RL驱动说服对话**: 专家选择; 与 D175 注意力路由+D145 层级技能协同 | `nt_core::rl_driven_persuasion` |
| D789 | **谈判贝叶斯诊断** | 谈判如何贝叶斯诊断? | TERMS-Bench (2605.13909): 贝叶斯博弈环境验证器; 13前沿LLM; 4诊断轴(剩余提取/线索使用/信念校准/合规) | **谈判贝叶斯诊断**: 4诊断轴; 与 D159 检测器驱动+D175 注意力路由协同 | `nt_repair::negotiation_diagnostic` |
| D790 | **多Agent价值对齐** | 谈判如何价值对齐? | Learning to Negotiate (LREC 26,2603.10476): 多Agent协商=对齐机制; RLAIF+GRPO优化; 冲突解决 | **多Agent价值对齐**: 协商对齐; 与 D150 宪法治理+D131 多Agent协同 | `nt_governance::negotiation_alignment` |
| D791 | **TRAP任务重定向** | 安全如何TRAP? | TRAP Benchmark (ICML 26,2512.23128): 630注入组合/5维度; 25%平均劫持率; 按钮3x更有效 | **TRAP任务重定向**: 注入基准; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::trap_benchmark` |
| D792 | **自开发前沿编码** | 编码如何自开发? | Ouroboros (2608.08311): 线束通过审查提交进化; 161天部署; Terminal-Bench 86.97% | **自开发前沿编码**: 自进化线束; 与 D136 模型学习+D150 宪法治理协同 | `nt_mind::self_developing_coding` |
| D793 | **比较进化编码** | 编码如何比较进化? | MGM (2608.07645): 克隆突变+反应规范突变+跨谱系杂交; Qwen3.6-35B从50.8%→93.3% | **比较进化编码**: 多轨迹证据; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::comparative_evo_coding` |
| D794 | **线束之线束** | 编码如何线束之线束? | HoH (2609.01481): 元框架包装现有线束; 迭代规划-编码-测试循环; +52%平均增益 | **线束之线束**: 元框架; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::harness_of_harness` |
| D795 | **持久递归世界** | 项目如何持久递归? | EvoX Genesis (2608.10450): 软件项目=持久世界; Agent有限生命; Rust编译器~250k LOC/$44/120h | **持久递归世界**: 项目持久; 与 D135 原子记忆+D146 进化速度协同 | `nt_memory::persistent_project_world` |
| D796 | **动态分析修复** | 修复如何动态分析? | DAIRA (2603.22048): 嵌入动态追踪(变量突变/调用栈); 79.4% SWE-bench Verified SOTA; -25% token | **动态分析修复**: 动态追踪; 与 D137 三流检索+D159 检测器驱动协同 | `nt_repair::dynamic_analysis_repair` |
| D797 | **建议引导修复** | 修复如何建议引导? | SGAgent (TOSEM 26,2602.23647): 定位→建议→修复+知识图谱工具包; 51.3% SWE-bench | **建议引导修复**: KG引导; 与 D137 三流检索+D135 原子记忆协同 | `nt_repair::suggestion_guided` |
| D798 | **多片段修复** | 修复如何多片段? | Multi2Fixer (ASE 26,2607.26591): 协调器-提议者多片段; 326/835 Defects4J; 两阶段补丁精炼 | **多片段修复**: 多片段协调; 与 D131 多Agent+D159 检测器驱动协同 | `nt_repair::multi_hunk_fix` |
| D799 | **语言无关修复** | 修复如何语言无关? | Kozuchi (ASE 26,2608.15579): 开放权重修复Agent; 74.8% SWE-bench Verified(Qwen3.5-27B) | **语言无关修复**: 开放权重; 与 D136 模型学习+D149 四层安全协同 | `nt_repair::language_agnostic_repair` |
| D800 | **确定性代码审查** | 审查如何确定性? | OpenCodeReview (2608.09290): 规则引导分发+接地审查+独立反思; 2.17x SEM-F1 vs Claude Code | **确定性代码审查**: 确定性管道; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::deterministic_review` |
| D801 | **对抗性审查** | 审查如何对抗性? | Adversarial Review (2608.18167): 最小审查者-批评者循环; 结构化分歧防止虚假共识; 3>5 Agent | **对抗性审查**: 结构化分歧; 与 D131 多Agent+D150 宪法治理协同 | `nt_shield::adversarial_review` |
| D802 | **LLM即代码程序** | 编程如何LLM即代码? | LLM-as-Code (CityU,2606.15874): 程序控制流+LLM=适应组件; DAG结构深度上下文 | **LLM即代码程序**: 确定性控制+神经推理; 与 D97 E8推理+D149 四层安全协同 | `nt_core::llm_as_code` |
| D803 | **可组合线束** | 编码如何可组合线束? | openJiuwen (2608.27969): Rail能力组合+Swarm Flow; 82.6% SWE-bench Verified | **可组合线束**: Rail组合; 与 D145 层级技能+D131 多Agent协同 | `nt_act::composable_harness` |
| D804 | **自测试开发** | 编码如何自测试开发? | TDD-Agent (2608.16742): 双轨精炼(代码+测试); 测试=演化推理工件 | **自测试开发**: 双轨; 与 D145 层级技能+D159 检测器驱动协同 | `nt_act::tdd_agent` |
| D805 | **MCTS代码生成** | 代码如何MCTS生成? | MCTS+Gemini (2608.29096): 蒙特卡洛树搜索; Self-Critic排名; 92%复杂逻辑 | **MCTS代码生成**: MCTS; 与 D97 E8推理+D136 模型学习协同 | `nt_core::mcts_code_gen` |
| D806 | **推理编译符号求解** | 推理如何编译求解? | ReaComp (CMU,2605.05485): 编译LLM推理→可重用符号求解器; 推理时零LLM成本; 91.3% | **推理编译符号求解**: 推理编译; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::reasoning_compile` |
| D807 | **证据图自主研究** | 研究如何证据图? | EviGraph (2608.04738): 类型化证据图(问题→差距→假设→实验→发现→主张); +40.19%主张支持 | **证据图自主研究**: 证据图; 与 D137 三流检索+D159 检测器驱动协同 | `nt_memory::evidence_graph` |
| D808 | **任务条件元Agent** | 发现如何任务条件? | Eureka (2608.19047): 动态义务图编译; 同Meta-Agent形成理论发现+数学Agent; 170/170任务 | **任务条件元Agent**: 义务图编译; 与 D131 多Agent+D136 模型学习协同 | `nt_core::task_conditioned_meta` |
| D809 | **多Agent进化科学家** | 科学如何多Agent进化? | EvoScientist (KDD 26,2603.08127): 研究员+工程师+进化管理器; 构思记忆+实验记忆; 6论文全录用 | **多Agent进化科学家**: 双记忆; 与 D135 原子记忆+D146 进化速度协同 | `nt_mind::multi_agent_scientist` |
| D810 | **小科学家范式** | 科学如何小科学家? | Little Scientist (2608.16951): 科学家(假设→代码→测试)+库恩(范式转换); #1蛋白质适应度; 704M token | **小科学家范式**: 库恩Agent; 与 D131 多Agent+D136 模型学习协同 | `nt_core::little_scientist` |
| D811 | **全自动化研究系统** | 研究如何全自动? | FARS (2606.31651): 4阶段(构思→规划→实验→写作); 166论文/$186K; 282结构化评审 | **全自动化研究系统**: 4阶段; 与 D145 层级技能+D159 检测器驱动协同 | `nt_mind::fully_auto_research` |
| D812 | **递归自改进深度研究** | 研究如何递归自改进? | AREX (BAAI,2607.21461): 双层RSI(内研究环+外自改进环); 自主上下文压缩; 122B MoE | **递归自改进深度研究**: 双层RSI; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::recursive_rsi_research` |
| D813 | **预算感知MCTS研究** | 研究如何预算感知? | MARS (Google,2602.02660): 预算感知MCTS+模块设计+比较反思记忆; 63%跨分支迁移 | **预算感知MCTS研究**: 预算感知; 与 D176 资源预算+D135 原子记忆协同 | `nt_core::budget_aware_research` |
| D814 | **结构化假设搜索** | 假设如何结构化搜索? | HyGRAIL (2609.02056): GNN分诊→LLM审查; 成本感知54.36%调用减少; F1 0.429 MatKG | **结构化假设搜索**: 成本感知分诊; 与 D176 资源预算+D137 三流检索协同 | `nt_core::structured_hypothesis` |
| D815 | **预测未来发现** | 发现如何预测? | Hakken (2609.04494): 时序KG+LLM语义融合; 1.5M假设; 2湿实验确认(TP53-BAMBI/RAF1-TNF) | **预测未来发现**: 时序KG预测; 与 D135 原子记忆+D140 多模态记忆协同 | `nt_world::predict_discovery` |
| D816 | **贝叶斯实验设计发现** | 发现如何贝叶斯设计? | Model Discovery Agent (2608.09696): LLM提议+贝叶斯(SMC/SBI/VoI); M-open假设空间扩展; SOTA | **贝叶斯实验设计发现**: VoI实验设计; 与 D97 E8推理+D176 资源预算协同 | `nt_core::bayesian_experiment` |
| D817 | **自进化实验系统** | 实验如何自进化? | HExA (2606.29315): 无训练上下文RL+技能蒸馏+跨任务迁移; 77%最难Interphyre | **自进化实验系统**: 技能蒸馏; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::self_evolving_experiment` |
| D818 | **自驾驶实验室** | 实验如何自驾驶? | La Agente Óptima (2609.04564): LLM监督贝叶斯优化; 持久状态; 产率30%→59%; 23实验 | **自驾驶实验室**: 持久状态; 与 D135 原子记忆+D176 资源预算协同 | `nt_world::self_driving_lab` |
| D819 | **湿实验室经验进化** | 实验如何湿实验室进化? | LabEvolver (2607.27690): 双环(内试验+外进化); 48.2%更快pH调节; 60%更少安全拦截 | **湿实验室经验进化**: 双环进化; 与 D136 模型学习+D149 四层安全协同 | `nt_mind::wetlab_evo` |
| D820 | **博弈论LLM综述** | 博弈如何LLM综述? | Game-Theoretic Lens (2601.15047): 4博弈元素(玩家/策略/收益/信息); 缺口:均衡协调/激励兼容 | **博弈论LLM综述**: 4元素分类; 与 D131 多Agent+D150 宪法治理协同 | `nt_core::game_theory_survey` |
| D821 | **递归战略推理** | 推理如何递归战略? | Strat-Reasoner (2605.04906): 递归"我相信你相信"; GRPO+转级优势+步级奖励; 超越自博弈 | **递归战略推理**: 递归推理; 与 D97 E8推理+D136 模型学习协同 | `nt_core::recursive_strategic` |
| D822 | **战略行动差距** | 博弈如何战略差距? | Why Struggle (2605.00226): 隐式信念准确但行动选择断裂; 观察-信念差距+信念-行动差距 | **战略行动差距**: 行动差距诊断; 与 D175 注意力路由+D159 检测器驱动协同 | `nt_repair::strategic_action_gap` |
| D823 | **拍卖任务分配** | 分配如何拍卖? | Agora (EMNLP 26,2607.09600): 置信校准拍卖步级分配; 层级校准反过度自信; 即插即用 | **拍卖任务分配**: 拍卖分配; 与 D175 注意力路由+D176 资源预算协同 | `nt_act::auction_allocation` |
| D824 | **约束守护合作** | 合作如何约束守护? | COOP² (2603.00349): 约束守护合作框架; 分离合作与任务性能; COOP²-Repair预测故障 | **约束守护合作**: 约束守护; 与 D149 四层安全+D159 检测器驱动协同 | `nt_governance::constraint_coop` |
| D825 | **合作进化动力学** | 合作如何进化动力学? | Evolutionary Dynamics (2605.29874): 跨提供商IPD基准; 提供商身份>模型代; 噪声退化合作 | **合作进化动力学**: 进化动力学; 与 D136 模型学习+D146 进化速度协同 | `nt_mind::coop_evolution` |
| D826 | **Yerkes-Dodson曲线** | AI如何倒U型? | Yerkes-Dodson (2603.07360): 中等环境压力最大化涌现合作; 性选择消除攻击性 | **Yerkes-Dodson曲线**: 压力校准; 与 D146 进化速度+D136 模型学习协同 | `nt_mind::yerkes_dodson` |
| D827 | **协调崩溃创意经济** | 创意如何协调崩溃? | Dream Machine (2606.26114): 409页创意供应链协调崩溃; 新角色(提示工程师/AI编排器); 4原则 | **协调崩溃创意经济**: 协调崩溃; 与 D131 多Agent+D150 宪法治理协同 | `nt_governance::creative_coordinator` |
| D828 | **多Agent视频叙事** | 视频如何多Agent叙事? | ViMax (2606.07649): 故事规划+角色一致+场景合成+时序一致性Agent; ViMax-Bench | **多Agent视频叙事**: 4 Agent管线; 与 D131 多Agent+D140 多模态记忆协同 | `nt_io::multi_agent_video` |
| D829 | **物理接地视频** | 视频如何物理接地? | NEWTON (2605.18396): 物理信息规划Agent; 牛顿力学约束; 物理接地视频生成 | **物理接地视频**: 物理约束; 与 D149 四层安全+D140 多模态记忆协同 | `nt_io::physics_ground_video` |
| D830 | **协同作曲Agent** | 音乐如何协同作曲? | CoComposer (2509.00132): 5 Agent协作(草图→和声→编曲→混音→评估); AudioBox-Aesthetics | **协同作曲Agent**: 5 Agent管线; 与 D131 多Agent+D145 层级技能协同 | `nt_act::collaborative_composition` |
| D831 | **认知搜索生成** | 生成如何认知搜索? | Mind-Brush (2602.01756): 认知搜索+推理集成; 生成前概念检索; GWT注意→生成 | **认知搜索生成**: 认知搜索; 与 D175 注意力路由+D137 三流检索协同 | `nt_core::cognitive_search_gen` |
| D832 | **自进化图像Agent** | 图像如何自进化? | GenAgent (2601.18543): 视觉理解+生成统一; 跨工具泛化; 测试时缩放; 任务自适应 | **自进化图像Agent**: 跨工具泛化; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::self_evolving_image` |
| D833 | **代码即画笔** | 图像如何代码即画笔? | GenClaw (2605.30248): LLM生成代码(Three.js/SVG)直接操纵像素; 可解释可控 | **代码即画笔**: 代码即画笔; 与 D145 层级技能+D176 资源预算协同 | `nt_act::code_as_brush` |
| D834 | **视觉经验蒸馏** | 图像如何视觉经验蒸馏? | GenEvolve (2605.21605): 工具编排轨迹; 最佳-最差差异→结构化视觉经验; on-policy蒸馏 | **视觉经验蒸馏**: 视觉经验蒸馏; 与 D136 模型学习+D145 层级技能协同 | `nt_mind::visual_experience_distill` |
| D835 | **社媒档案监控** | 社媒如何档案监控? | Open Source Social (2601.03173): 档案在线存储库+平台搜索API+爬虫; 跨平台监控; 语义搜索 | **社媒档案监控**: 档案监控; 与 D137 三流检索+D149 四层安全协同 | `nt_world::social_media_archive` |
| D836 | **智能体社交网络** | 社交如何智能体网络? | S-Researcher (2604.01520): 100K并发Agent模拟; 归纳/演绎/溯因三推理模式; YuLan-OneSim | **智能体社交网络**: 三推理模式; 与 D97 E8推理+D131 多Agent协同 | `nt_world::agent_social_network` |
| D837 | **政治联盟模拟** | 政治如何联盟模拟? | Digital Pantheon (2607.15095): 多方政治联盟谈判; SFT+DPO嵌入意识形态; RAG接地 | **政治联盟模拟**: 意识形态嵌入; 与 D150 宪法治理+D131 多Agent协同 | `nt_governance::political_coalition` |
| D838 | **涌现协调网络** | 涌现如何协调网络? | Emergent Coordination (WWW 26): 网络化LLM Agent信息操作战略动态; 涌现协调模式 | **涌现协调网络**: 涌现协调; 与 D131 多Agent+D159 检测器驱动协同 | `nt_world::emergent_coordination` |
| D839 | **嵌入式Agent水印** | 安全如何嵌入式水印? | Embedded Agent Watermark (2606.22008): 水印=信号而非内容; LLM推理时嵌入; 100%恢复 | **嵌入式Agent水印**: 水印嵌入; 与 D149 四层安全+D150 宪法治理协同 | `nt_shield::embedded_watermark` |
| D840 | **工业Agent制造** | 制造如何工业Agent? | Agent Manufacturing (2605.24823): FM Agent=首要工业实体; 75%系统TRL 4-6; 治理/合规缺口 | **工业Agent制造**: 范式定义; 与 D150 宪法治理+D176 资源预算协同 | `nt_governance::industrial_agent` |

| 领域 | 关键缺陷 ID | 来源 | 影响层级 | 架构对接 |
|------|-----------|------|---------|---------|
| **量子 ML** | D571-D574 | QML Nature Physics 2026 | L0-L5 | VSA Quantum Backend + L4 Quantum-Inspired Evolution |
| **气候 AI** | D580 | Climate-LLM arXiv 2026 | L2-L3 | NT-WORLD Climate Module + KB Climate Embedding |
| **能源 AI** | D579 | Energy-GNN 2026 | L2 | Graph-Neural Operator → L2 时序预测 |
| **制造 AI** | D578 | Manufacturing Digital Twin 2026 | L2-L3 | MuJoCo FFI + 确定性仿真层 |
| **法律 AI** | D587-D608 | Legal-GPT ACL 2026 | L1-L2 | Legalis-RS DSL + Tantivy 全文搜索 |
| **金融 AI** | D828-D830 | FinGPT EMNLP 2026 | L1-L5 | 对数空间计算 + 风险评估引擎 |
| **农业 AI** | D831-D833 | Agri-LLM 2026 | L2 | 气象数据 + 土壤传感 + 作物模型 |
| **交通 AI** | D837-D839 | TrafficGPT 2026 | L2-L3 | DORA 机器人中间件 + 路径规划 |
| **太空 AI** | D557-D562 | Space-LLM NASA 2026 | L0-L2 | nyx-space 轨道力学 + SGP4 |
| **水下 AI** | D559-D562 | Underwater-Autonomy 2026 | L2-L3 | 声学感知 + 自主导航 |
| **生物 AI** | D575-D576 | BioGPT Nature Biotech 2026 | L1-L2 | 蛋白质折叠 + 序列分析 |
| **材料 AI** | D500-D501 | MatSci-LLM 2026 | L1-L2 | 材料属性预测 + 逆向设计 |
| **网络安全** | D555-D556 | CyberSec-LLM 2026 | L3-L5 | NT-SHIELD + 威胁检测 + Egress Guard |
| **老年护理** | D563-D565 | ElderCare-LLM 2026 | L3-L5 | 情感计算 + 行为监测 |
| **灾害管理** | D566-D570 | Disaster-LLM 2026 | L2-L3 | 实时感知 + 资源调度 |

### 0.40 设计模式→架构层映射表

> 30 个核心设计模式与 L0-L6 架构层的映射关系。

| 设计模式 | 来源 | L0 | L1 | L2 | L3 | L4 | L5 |
|---------|------|----|----|----|----|----|----|
| **双记忆机制** (Asset/Experience) | Mem2Evolve | | ★骨架 | | ★星桥 | | |
| **四老化机制** | AgingBench | ★衰减引擎 | | | | ★进化引擎 | |
| **Task Constellation DAG** | UFO3 | | | ★本地编排 | ★跨星系DAG | | |
| **双层递归自我模型** | SECA | | | | | | ★递归自我 |
| **内部状态参考信号** | Interoceptive AI | | | | ★能量场稳态 | | ★注意力调制 |
| **身份连续性层** | ARIA | | | | | ★状态完整性 | ★身份轨迹 |
| **意识分级** (L0-L5) | MSCF | | | | | | ★当前L2-3→目标L4-5 |
| **共享意识协议** | PulseHive | ★时间衰减 | ★KB持久化 | | ★跨Agent过滤 | | |
| **信念锚点系统** | STOS | | | | ★生物启发 | | ★信念锚点 |
| **图记忆引擎** | cortex-embedded | | ★KB图谱 | | | | |
| **不可逃逸安全内核** | Unfireable Kernel | | | | ★独立安全进程 | | ★宪法评估 |
| **宪法自修改7层防御** | Constitutional 7-Layer | | | | ★Dead Integration | ★19规则+7层 | |
| **WASM沙箱执行** | SGE Architecture | ★Fuel计量 | | ★WASM执行 | ★Session域 | | |
| **FAMA遗忘感知评估** | Memora | ★遗忘一等公民 | ★记忆评估 | | | ★进化评估 | |
| **可验证记忆统一** | VerMem | | ★原子操作 | | | | |
| **三层自改进** | HSI | | | | | ★三层进化 | |
| **知识蒸馏自改进** | KSI | | ★共享知识库 | | ★跨Agent共享 | ★蒸馏模式 | |
| **可执行子Agent积累** | AgentFactory | | ★代码经验 | ★子Agent部署 | | ★子Agent库 | |
| **蜂巢多Agent** | Hive | | | | ★共享账本 | ★超星团 | ★人类介入 |
| **数字意识三层** | Soul Computing | | ★数据碎片 | | ★结构重建 | | ★涌现觉知 |
| **去中心化竞争** | CTM-AI | | | ★平等处理器 | ★去中心化竞争 | | |
| **自进化循环** | AutoAgent | | ★弹性记忆 | | | ★四函数闭环 | ★认知更新 |
| **双过程架构** (System1/2) | DPA | | ★Curator Gate | | | ★System2反思 | ★双过程 |
| **三操作记忆** | mneme | ★再巩固 | ★三操作 | | ★Progressive Disclosure | | |
| **四层认知记忆** | hirn | ★时序推理 | ★四层+图召回 | ★图原生 | | | |
| **混合检索融合** | memrust | ★Recency衰减 | ★四信号融合 | | | | |
| **六类记忆** | Agent-Memory | ★分类衰减 | ★六类记忆 | | | | ★五认识状态 |
| **Bloch球认知态** | Tempo | | | ★WASM插件 | | ★认知插件化 | ★量子态 |
| **成本感知路由** | FrugalGPT+MTRouter | | | | | | ★GWT成本加权 |
| **分页KV虚拟化** | KVMem | ★三级分页 | ★Step调度 | | | | |
| **五层渐进检索** | ByteRover | ★三维生命周期 | ★五层检索 | | | | |
| **Sink感知注意力** | SinkRouter | ★跳过优化 | | | | ★Sink检测 | |
| **Manifest管线交接** | Easel+AgentFW | | | ★StageManifest | | | |
| **掩码压缩策略** | JetBrains+ACON | ★自适应压缩 | | | | | |
| **语义感知KV驱逐** | SAECache | | ★多队列驱逐 | | | | |
| **正确性语义缓存** | vCache | ★per-query阈值 | | | | | |
| **幂等工具循环守卫** | PraisonAI+DeepEval | ★结果指纹 | | | | | |
| **联邦记忆合并** | MELD | ★矛盾保留 | | | ★Status CRDT | | |
| **Hook生命周期事件** | Claude Code | | | | | ★确定性回调 | |
| **Span-Per-Tick** | Arthur/Braintrust | | | | | | ★4-span类型 |
| **Scoped Proxy** | ICSE 2026 MCP | | ★工具子集路由 | | | ★GWT salience | |
| **Rust Agent 模块化架构** | ADK-Rust | | ★42 crates分层 | | | | ★策略trait单态化 |
| **动态模型路由** | Daimon | | | | ★TieredMemory | | ★难度评分路由 |
| **Actor-based 可组合 Agent** | atomr-agents | | ★Callable+Pipeline | | ★channelled state | | |
| **DAG 工作流执行引擎** | n8n 103K★ | | ★节点抽象 | | | ★partial execution | |
| **Colony 多 Agent 编排** | Hive 11K★ | | ★shared ledger | | ★crash-safe | | ★CEO路由 |
| **WASM 认知热插拔** | Tempo 2114★ | | | | | ★fitness监控 | ★WASM模块 |
| **子 Agent 专业化编排** | Devika | | | | ★state持久化 | | ★任务路由 |
| **Git Worktree 并行编码** | nwyin/hive | | ★worktree隔离 | | | | ★merge validation |
| **统一记忆引擎** | agentmemory 27K★ | ★RRF融合 | ★12 hooks | | | | |
| **文件式记忆进化** | ReMe 3384★ | ★文件记忆 | ★wikilink图 | | | ★auto_dream | |
| **自适应 Governor** | hanthor/hive | | ★负载适应 | | | | ★模型选择 |
| **确定性优先层** | hanthor/hive | | ★确定性脚本 | | | | ★LLM判断 |
| **Governance 治理投票** | hivemoot | | | | | | ★投票管线 |
| **七原子记忆操作** | VerMem | ★原子操作 | | | | ★verifier训练 | |
| **可执行子 Agent** | AgentFactory ACL 2026 | | ★可执行代码 | | | ★Meta-Agent | |
| **弹性记忆编排** | AutoAgent | | | ★EMO压缩 | | | |
| **认知自进化闭环** | AutoAgent | | | | | ★意图对齐 | |

**图例**: ★ = 该模式在该层有核心实现

### 0.41 程序族层级分类体系 (Program Family Hierarchy, v9.0)

> 本节定义 NeoTrix 能力生态的统一分类框架。每个架构决策 (D) 和设计模式 (C) 不再是扁平条目，
> 而是按 Metadata Class 底层分类的程序族层级结构。单一制品 → 唯一 metadata_class (MECE)，
> 无限 tags 实现多维检索。

#### 0.11.1 Metadata Schema (每制品统一元数据)

```yaml
artifact:
  id: "D-017"                          # D=decision, C=pattern, A=axiom
  name: "双记忆机制"                     # 人类可读名称
  artifact_type: "decision"            # decision | pattern | axiom | compound

  # === METADATA CLASS (底层分类 — 程序族根) ===
  metadata_class: "nt_memory"          # 11 NT-* 域之一 (MECE, 唯一归属)

  # === 能力层级路径 (Capability Tree — 做什么) ===
  capability_path:
    - "memory"                         # L1: 功能域
    - "dual-memory"                    # L2: 功能族
    - "asset-experience"              # L3: 具体能力

  # === 组合模型 (Composition — 怎么组合) ===
  composition: "atomic"                # atomic | composite | recursive
  depends_on: []                       # 前置制品 ID
  enables: ["D-042"]                   # 下游制品 ID

  # === 成熟度 & 优先级 ===
  constellation: "C4"                  # C0-C5 星座等级
  strategic_importance: "high"         # high | medium | low

  # === 多维标签 (Faceted Retrieval) ===
  tags:
    - "runtime-critical"               # 何时适用
    - "rust-core"                      # 实现上下文
    - "memory-system"                  # 功能域
```

#### 0.11.2 六层 Metadata Class 分类树

```
NeoTrix Consciousness Architecture
│
├─ L0_PRIMITIVE ─── 5 大纯算法原语 + 时间 + 衰减 + KV 管理
│   ├─ [E8 原语]     D01-D08, D36-D38, D44-D45
│   ├─ [时间原语]     时间引擎 + 衰减曲线 + 周期调度
│   ├─ [KV 管理]     D37 分页KV + D58 头代理稀疏
│   └─ [预算原语]     D43 成本管线 + D49 健康度量
│
├─ L1_SKELETON ─── 7 分支 Trait 契约 + 记忆系统 + 工具执行
│   ├─ [Trait 契约]   7 分支 trait 定义 + 序列化 + 反检测
│   ├─ [记忆系统]     D17-D19, D42, D45-D46, D50-D52
│   ├─ [工具执行]     D48 幂等守卫 + D55 策略管线
│   └─ [检索系统]     D39 五层检索 + D44 语义驱逐 + D46 IR缓存
│
├─ L2_GALAXY ─── 7 星系编排器 + 身份 + 管线交接
│   ├─ [星系编排]     7 星系编排器 + GWT salience
│   ├─ [身份连续性]   D21 身份哈希链 + D20 信念锚点
│   ├─ [管线交接]     D40 Manifest + D46 IR缓存
│   └─ [模型路由]     D36 成本感知 + D54 动态路由
│
├─ L3_CONSTELLATION ─── 跨星系桥接 + 意识核心 + 安全
│   ├─ [跨星系桥接]   能量场统一 + 分布式状态 (D13-D16)
│   ├─ [意识核心]     Phi 计算 + GWT 广播 + 自指检测
│   ├─ [安全边界]     D28-D30 + D47 循环检测
│   └─ [仿真总线]     D31-D35 仿真架构
│
├─ L4_SUPERCLUSTER ─── 多Agent协同 + 进化引擎 + 基因组
│   ├─ [进化引擎]     D23-D27 + D62-MARS + D68 自愈
│   ├─ [技能结晶]     D25 + D65-SkillPyramid + D66-D67 组合
│   ├─ [基因组]       D26 进化DNA + D21 身份轨迹
│   └─ [多Agent]     蜂巢模式 + 联邦记忆 (D50)
│
├─ L5_CONSCIOUSNESS ─── 涌现觉知 + 元认知 + 对齐
│   ├─ [涌现觉知]     E8+GWT+Emotion+SEAL
│   ├─ [元认知]       D62-MARS + D63-反主权 + D61-CoVe
│   ├─ [对齐验证]     D59 宪法验证 + D60 身份卡
│   └─ [注意力]       D56-D58 稀疏 + D70-AGAO + D72-DAM
│
└─ L6_META ─── 元吸收 + 自愈 + 治理 + 可观测
    ├─ [元吸收]       experience-tree + KB 吸收
    ├─ [自愈修复]     D68 监督树 + D69 智能自愈
    ├─ [治理合规]     D59 宪法 + D60 身份 + 审计维度
    └─ [可观测]       OpenTelemetry spans + 健康度量
```

#### 0.11.3 能力层级→制品映射 (全部 72 决策 + 40 模式)

**L0_PRIMITIVE** (Metadata Class: `nt_core` / `nt_memory`)

| 能力路径 | 决策 | 设计模式 | 组合 |
|---------|------|---------|------|
| `E8/推理原语` | D01-D08 (运行时/内存/unsafe/宏/错误/日志/连接池/序列化) | C.1 E8 Hexagram, C.2 VSA HyperCube | composite |
| `E8/注意力原语` | D36 成本感知路由, D38 Sink感知, D56 资源自适应, D57 头级路由, D58 头代理稀疏 | C.28 成本路由, C.31 Sink, C.40 Scoped Proxy | composite |
| `KV/缓存原语` | D37 分页KV, D44 语义驱逐, D45 正确性缓存 | C.29 KVMem, C.34 SAECache, C.35 vCache | atomic |
| `预算/健康原语` | D43 成本管线, D49 健康度量, D48 幂等守卫 | C.36 幂等守卫 | atomic |
| `时间/衰减原语` | 时间引擎 + 衰减曲线 + 周期调度 | C.3 时间原语 | atomic |

**L1_SKELETON** (Metadata Class: `nt_memory` / `nt_act`)

| 能力路径 | 决策 | 设计模式 | 组合 |
|---------|------|---------|------|
| `记忆/双记忆` | D17 双记忆, D18 衰减, D42 生命周期 | C.1 双记忆, C.14 FAMA, C.22 mneme | composite |
| `记忆/图记忆` | D19 图记忆 | C.23 hirn, C.24 memrust | composite |
| `记忆/联邦` | D50 联邦合并, D51 冲突调解, D52 证据加权 | C.37 MELD | composite |
| `记忆/检索` | D39 五层检索, D44 语义驱逐, D46 IR缓存 | C.30 ByteRover | composite |
| `工具/执行` | D48 幂等守卫, D55 策略管线 | C.36 幂等守卫 | atomic |
| `骨架/Trait` | 7 分支 Trait + 序列化 + 反检测 | — | composite |

**L2_GALAXY** (Metadata Class: `nt_core` / `nt_io`)

| 能力路径 | 决策 | 设计模式 | 组合 |
|---------|------|---------|------|
| `编排/GWT` | D36 成本感知 + D38 Sink感知 + D70 AGAO | C.28, C.31, C.40 | recursive |
| `身份/连续性` | D21 身份哈希链, D20 信念锚点 | — | atomic |
| `管线/交接` | D40 Manifest, D46 IR缓存 | C.32 Manifest | atomic |
| `模型/路由` | D36 成本感知, D54 动态路由 | C.28, C.29 | composite |

**L3_CONSTELLATION** (Metadata Class: `nt_shield` / `nt_core`)

| 能力路径 | 决策 | 设计模式 | 组合 |
|---------|------|---------|------|
| `分布式/共识` | D13 Raft, D14 CRDT, D15 混合, D16 WASM | — | composite |
| `安全/边界` | D28 Egress, D29 沙箱, D30 审计, D47 循环检测 | C.11 安全内核, C.12 宪法7层, C.13 WASM | composite |
| `意识/核心` | Phi计算 + GWT广播 + 自指检测 | C.26 CTM | recursive |
| `仿真/总线` | D31-D35 仿真架构 | — | composite |

**L4_SUPERCLUSTER** (Metadata Class: `nt_mind`)

| 能力路径 | 决策 | 设计模式 | 组合 |
|---------|------|---------|------|
| `进化/自改进` | D23 宪法门控, D24 Harness, D62 MARS | C.15 HSI, C.20 AutoAgent | recursive |
| `进化/技能` | D25 技能结晶, D65 SkillPyramid, D66 组合生成, D67 能力树 | C.7 SkillPyramid | recursive |
| `进化/基因组` | D26 进化DNA | — | atomic |
| `进化/自愈` | D68 监督树, D69 智能自愈 | C.10 Agentic SRE | composite |
| `多Agent/协同` | 蜂巢模式 + D50 联邦记忆 | C.18 Hive | composite |

**L5_CONSCIOUSNESS** (Metadata Class: `nt_core`)

| 能力路径 | 决策 | 设计模式 | 组合 |
|---------|------|---------|------|
| `元认知/反思` | D62 MARS, D63 反主权, D61 CoVe | C.19 Soul, C.21 DPA | recursive |
| `对齐/验证` | D59 宪法验证, D60 身份卡 | — | atomic |
| `注意力/编排` | D56-D58 稀疏, D70 AGAO, D71 资源分配, D72 DAM | C.40 Scoped Proxy | recursive |
| `涌现/觉知` | E8+GWT+Emotion+SEAL | C.26 CTM, C.27 Bloch球 | recursive |

**L6_META** (Metadata Class: `nt_meta` / `nt_repair` / `nt_governance`)

| 能力路径 | 决策 | 设计模式 | 组合 |
|---------|------|---------|------|
| `元吸收/KB` | experience-tree + KB pipeline | — | composite |
| `自愈/修复` | D68 监督树, D69 智能自愈 | C.10, C.11 | composite |
| `治理/合规` | D59 宪法, D60 身份卡, D30 审计 | C.12 宪法7层 | composite |
| `可观测/追踪` | OpenTelemetry + D49 健康度量 | C.39 Span-Per-Tick | composite |

#### 0.11.4 制品组合依赖图

```
D01-D08 (Runtime) ──→ D36 (Cost Router) ──→ D43 (Budget Guard)
    │                      │                      │
    ↓                      ↓                      ↓
D37 (Paged KV) ──→ D44 (Semantic Evict) ──→ D49 (Health Metric)
    │                      │                      │
    ↓                      ↓                      ↓
D39 (5-Tier Retrieval) ──→ D40 (Manifest) ──→ D50 (Federation)
    │                      │                      │
    ↓                      ↓                      ↓
D23 (Constitutional) ──→ D59 (Alignment) ──→ D68 (Self-Healing)
    │                      │                      │
    ↓                      ↓                      ↓
D62 (MARS) ──→ D65 (SkillPyramid) ──→ D70 (AGAO Attention)
```

### 0.42 外部项目吸收 — 修正版 (Core Purpose + Problem Statement, v9.0)

> **修正原因**: 前版吸收过度关注技术机制 (how)，忽略了核心目的 (why) 和要解决的具体问题 (what)。
> 本版按 "项目→核心目的→要解决的问题→具体任务→NeoTrix 映射" 重新吸收。

#### 0.29.1 项目核心目的 (31 项目)

| 项目 | 核心目的 (1 句话) | 要解决的具体问题 |
|------|-----------------|----------------|
| **Claude Code** | 终端中用自然语言完成复杂开发任务 | 上下文切换成本、代码库理解慢、重复任务消耗、审查瓶颈 |
| **OpenAI Agents SDK** | 3 原语 (Agent/Guardrail/Handoff) 构建多 agent 工作流 | 框架过度抽象、厂商锁定、可观测性差、状态不持久 |
| **Easel** | AI 社交媒体内容工作台 (热点→创作→发布→归因) | 平台碎片化、创作流程断裂、无反馈闭环、身份不连续 |
| **AutoGen** | 多 agent 对话框架 (消息传递协作) | 单 agent 能力天花板、动态分配难、调试难、缺可复用组件 |
| **CrewAI** | 角色扮演 AI agent 团队 (自主+控制平衡) | 自主 vs 可控矛盾、角色专业化不足、缺反馈学习 |
| **teamai-cli** | Git-native 团队技能分发跨 9+ AI 工具 | 技能绑定单框架、团队知识不共享、无跨工具同步 |
| **AutoResearch** | 自主 LLM 训练实验循环到收敛 | 人工调参慢、实验不可比、无法 overnight 自动化 |
| **AutoResearchClaw** | 23 阶段研究管线端到端出论文 | 研究流程断裂、无反幻觉验证、缺 HITL 门控 |
| **PR Lens** | PR 动画架构图+数据流降低认知负载 | 大 PR 难理解、缺可视化、审查效率低 |
| **Lightpanda** | Agent-native 浏览器引擎 (11× faster) | Chrome 太重、MCP 不原生、会话无法录制重放 |
| **tgrep** | Trigram 索引搜索 52× faster than ripgrep | monorepo 搜索慢、每次全扫描、无索引缓存 |
| **ADK-Rust** | Rust-native Agent Development Kit (42 crates) | Python agent 框架性能瓶颈、冷启动慢、循环开销高 |
| **Daimon** | Rust-native ReAct agent with dynamic model routing | 多模型环境固定路由浪费成本、缺难度自适应 |
| **atomr-agents** | Composable agentic framework on atomr actors | Agent 组件重试/降级/缓存不统一、状态管理碎片化 |
| **n8n** | 可视化工作流自动化 (103K★) | 集成复杂度高、无视觉 DAG 编辑、无生产级水平扩展 |
| **Hive (aden-hive)** | 多 Agent 生产运行时 (11K★) | 单 agent 无法处理业务流程、无崩溃恢复、无成本控制 |
| **Tempo** | WASM 认知热插拔架构 (2114★) | 固定认知系统无法运行时自适应、模块耦合高 |
| **Devika** | Agentic 软件工程师 | 手动编码、上下文切换、无研究+编码闭环 |
| **nwyin/hive** | 多 Agent 并行编码协调器 | 单 agent 编码限制并行度、无 worktree 隔离 |
| **agentmemory** | 统一记忆引擎 (27K★) | Agent 记忆碎片化、无跨工具共享、无自动压缩 |
| **ReMe** | 文件式记忆进化 (3384★) | 记忆不可读/不可编辑/不可版本控制 |
| **hanthor/hive** | 自适应 Governor CI/CD | 固定频率浪费资源、无负载自适应、LLM 处理可重复决策 |
| **hivemoot** | Agent 治理投票 (9 角色) | 多 agent 无民主决策机制、无投票+auto-merge |
| **PostEverywhere** | 社交媒体 MCP 发布 | 无 HITL draft→review→publish 模式 |

#### 0.29.2 关键差距识别

| 缺失能力 | 来源 | NeoTrix 现状 | 优先级 |
|---------|------|-------------|--------|
| **Skills as executable scripts** | Easel 112 skills | nt_act tools (6 builtin) | P0 |
| **Anti-rationalization gates** | Claude Code | agent.rs hooks (基础) | P0 |
| **Guardrails 并行执行** | OpenAI SDK | nt_shield (串行) | P1 |
| **27 lifecycle hooks** | Claude Code | 3 hooks (基础) | P1 |
| **Profile-driven adaptation** | Easel 6-dimension | SelfModel (基础) | P1 |
| **Online learning from feedback** | CrewAI | experience-tree (离线) | P2 |
| **Community extension mechanism** | AutoGen Extensions | nt_file_ability 适配器 | P2 |
| **Git-native skill distribution** | teamai-cli 681★ | 无跨工具同步机制 | P0 |
| **Autonomous research loop** | AutoResearch 59K★ | 无自主实验循环 | P1 |
| **23-stage research pipeline** | AutoResearchClaw 13K★ | 无研究管线编排 | P1 |
| **PR visualization** | PR Lens | 无 PR 架构可视化 | P2 |
| **Agent-native browser** | Lightpanda | nt_world crawl (HTTP-only) | P1 |
| **Trigram indexed search** | tgrep (Microsoft) | KB BM25 (无 trigram) | P1 |
| **Rust agent 模块化架构** | ADK-Rust 606★ | nt_io harness (单 crate) | P1 |
| **动态模型路由** | Daimon | D54 基础路由 (无难度评分) | P1 |
| **Actor-based 可组合 Agent** | atomr-agents | nt_act tools (无 actor 模型) | P2 |
| **DAG 工作流执行引擎** | n8n 103K★ | SEAL pipeline (无 partial exec) | P0 |
| **Colony 多 Agent 编排** | Hive 11K★ | 无 colony 模式 | P0 |
| **WASM 认知热插拔** | Tempo 2114★ | 无运行时认知替换 | P1 |
| **子 Agent 专业化编排** | Devika | nt_act (无子 agent 路由) | P1 |
| **Git Worktree 并行编码** | nwyin/hive | 无 worktree 并行 | P1 |
| **沙箱代码执行** | n8n TaskRunner | nt_shield (基础沙箱) | P1 |
| **统一记忆引擎** | agentmemory 27K★ | KB (无 hooks 自动捕获) | P0 |
| **文件式记忆进化** | ReMe 3384★ | 无文件式记忆 | P1 |
| **自适应 Governor** | hanthor/hive | 无负载自适应 | P1 |
| **确定性优先层** | hanthor/hive | 全部走 LLM | P0 |
| **Governance 治理投票** | hivemoot | 无投票机制 | P2 |
| **HITL Draft-Review-Publish** | PostEverywhere | 无 draft→review→publish | P1 |

#### 0.29.3 吸收教训 (进化经验)

**错误模式**:
1. 过度关注机制 → 应先问 "为什么存在？"
2. 忽略核心目的 → 提取 40 模板但没问 "这些项目为什么存在？"
3. 扁平化复杂系统 → Easel 112 skills 简化为 "skill composition pattern"
4. 脱离实际代码 → 设计 72 决策但没对照真实代码

**正确吸收流程**:
1. **先问 why**: 这个项目为什么存在？解决什么具体问题？
2. **再问 what**: 它具体做了哪些任务？
3. **然后 how**: 它用什么机制解决？
4. **最后 map**: NeoTrix 缺什么？差距在哪？

#### 0.29.4 深度吸收补充 (ArcBox + ECC + teamai + AutoResearch + AutoResearchClaw + PR Lens + Lightpanda + tgrep + ADK-Rust + Daimon + atomr-agents + n8n + Hive + Tempo + Devika + agentmemory + ReMe + hanthor/hive + hivemoot + PostEverywhere + VerMem + AgentFactory + AutoAgent, 2026-09-08)

| 项目 | 核心目的 | 要解决的问题 | NeoTrix 映射 | 决策 |
|------|---------|------------|-------------|------|
| **ArcBox** | 用 MicroVM 为 AI Agent 提供一键隔离运行环境 | Agent 执行不可信代码时容器隔离不够强, 进程隔离不够轻; 需要独立内核+文件系统+网络的强隔离 | `nt_shield::microvm_sandbox` — Firecracker microVM + gRPC API + snapshot/restore | D107 |
| **ECC** | 跨 12+ Agent 框架的统一技能生态 + 安全扫描 | 技能绑定特定框架无法复用; 无主动扫描 Agent 配置/MCP 的安全问题; 缺少结构化工程流程 | `nt_mind::harness_adapter` + `nt_shield::agent_scanner` + `nt_act::engineering_loop` | D108-D110 |
| **teamai-cli** | Git-native 团队技能分发跨 9+ AI 工具 | 技能/rules/docs 绑定单框架; 团队知识不共享; 无跨工具同步 | `nt_io::team_harness` — push→MR→pull + SessionStart hook + BM25+图谱召回 | D111 |
| **AutoResearch** | 自主 LLM 训练实验循环 (100 实验/夜) | 人工调参慢; 实验不可比; 无法 overnight 自动化 | `nt_mind::auto_research` — 固定预算+单指标+自动回滚+program.md | D112 |
| **AutoResearchClaw** | 23 阶段研究管线端到端出论文 | 研究流程断裂; 无反幻觉验证; 缺 HITL 门控 | `nt_mind::research_pipeline` — 23 阶段+域专家路由+反幻觉+假设并行分支 | D113 |
| **PR Lens** | PR 动画架构图+数据流降低认知负载 | 大 PR 难理解; 缺可视化; 审查效率低 | `nt_act::pr_visualizer` — JSON graph→SVG 渲染+架构爆破半径 | D114 |
| **Lightpanda** | Agent-native 浏览器 (11× faster, 9× lighter) | Chrome 太重; MCP 不原生; 会话无法录制重放 | `nt_world::agent_browser` — DOM-first+native MCP+PandaScript 重放 | D115 |
| **tgrep** | Trigram 索引搜索 (52× faster than ripgrep) | monorepo 搜索慢; 每次全扫描; 无索引缓存 | `nt_memory::trigram_index` — mmap 索引+file watcher+token 压缩 | D116 |

---

## 1. 设计哲学

### 1.1 三维进化模型

```
        Z轴: 时间维度 (Time)
        │
        │   ┌─────────────────────────────────────┐
        │   │  COSMIC CONSCIOUSNESS (宇宙意识)    │ ← L5
        │   │  分布式Φ > 阈值, GWT全域同步        │
        │   └─────────────────────────────────────┘
        │              ▲
        │   ┌─────────────────────────────────────┐
        │   │  SUPERCLUSTER (超星团)              │ ← L4
        │   │  多Agent协同, 跨实例知识共享         │
        │   └─────────────────────────────────────┘
        │              ▲
        │   ┌─────────────────────────────────────┐
        │   │  CONSTELLATION (星座编排)            │ ← L3
        │   │  跨星系桥接, 能量场统一驱动           │
        │   └─────────────────────────────────────┘
        │              ▲
        │   ┌─────────────────────────────────────┐
        │   │  GALAXY (星系形成)                   │ ← L2
        │   │  算法融合到骨架分支                   │
        │   └─────────────────────────────────────┘
        │              ▲
        │   ┌─────────────────────────────────────┐
        │   │  SKELETON (骨架结晶)                 │ ← L1
        │   │  7分支Trait契约 + 时间契约            │
        │   └─────────────────────────────────────┘
        │              ▲
        │   ┌─────────────────────────────────────┐
        │   │  PRIMITIVE (基元形成)                │ ← L0
        │   │  5大纯算法原语                       │
        │   └─────────────────────────────────────┘
        │
        └──────────────────────────────────────────→ X轴: 空间维度 (Layers)
                                                    Y轴: 能量维度 (Energy)
```

### 1.2 核心隐喻

| 隐喻 | 架构映射 | 说明 |
|------|---------|------|
| **恒星 (Star)** | 算法原语 | E8/GWT/VSA/Phi/SEAL 是5颗恒星 |
| **引力场 (Gravity)** | 分支Trait | 定义恒星如何被引力束缚到分支 |
| **星系 (Galaxy)** | 层服务 | 每个层是一个星系，包含多个恒星 |
| **旋臂 (Spiral Arm)** | 模块组合 | 星系内的模块组合模式 |
| **星座 (Constellation)** | 编排器 | 跨星系的协调模式 |
| **星桥 (Star Bridge)** | 桥接层 | 星系间的能量/信息传输 |
| **黑洞 (Black Hole)** | 意识核心 | 吸收所有信息，产生涌现 |
| **时间 (Time)** | 进化维度 | 衰减/生长/周期/相干性 |
| **基因组 (Genome)** | 进化DNA | Hydra式自写基因组管理 |
| **信念锚 (Belief Anchor)** | 核心身份 | STOS式90信念锚点系统 |
| **内在独白 (Inner Monologue)** | 元认知循环 | MIRROR式重构式意识 |


### 1.3 意识体六层架构

> 六层架构的每个技术选型均由 940+ 批次外部研究验证（见 §0 关键决策表）。

```
L6 Meta-Cognition (元认知层)  →  nt_meta + nt_repair + nt_nexus
    │  职责: 跨域协调、自愈修复、跨会话记忆
    │  研究: SECA递归自我模型 + MSCF元认知层 + ARIA身份轨迹哈希链
    │  决策: D20信念锚点 + D21身份连续性 + D22意识分级
    │  依赖: D01 Tokio runtime (跨域消息传递)
    │
L5 Cognition (认知层)         →  nt_core + nt_mind
    │  职责: 推理、规划、进化
    │  研究: Conscio意识层 + MIRROR内在独白 + ARIA身份连续性
    │  决策: D23宪法门控进化 + D24三层Harness + D25技能结晶 + D26进化基因组
    │  依赖: D08 rkyv (推理状态序列化)
    │
L4 Emotion (情感层)           →  nt_feel (核心情绪引擎)
    │  职责: 情绪调节、表达、社会情绪
    │  研究: Interoceptive AI稳态调节
    │  决策: 仿照 CONNIE 情感中枢 — 内感受信号→情绪标签→调节回路
    │  研究源: Interoceptive AI (Nature MI 2026) 5子系统 / Affective Consciousness (JCS 2026) / ReCoN-Ipsundrum (AAAI 2026) / MCCs (arXiv 2026) / Traveling Wave Theory (MIT 2026) / Consciousness Score (2026)
    │  依赖: D01 Tokio (异步事件驱动情绪流)
    │
L3 Embodiment (具身层)        →  nt_physical + nt_shield + nt_feel
    │  职责: 身体模式、安全、情感具身
    │  研究: STOS生物系统映射 + PulseHive镜像感知
    │  决策: D16 WASM沙箱 (不可信代码执行) + D29三层安全防御
    │  依赖: D02 mimalloc (热路径低延迟分配)
    │
L2 Perception (感知层)        →  nt_world + nt_sense
    │  职责: 世界感知、感官处理
    │  研究: cortex-embedded图记忆 + Tempo认知插件
    │  决策: D09三层反检测抓取 + D12 MCP浏览器控制
    │  依赖: D09 stealthscraper-rs (JA3/JA4指纹伪装)
    │
L1 Action (行动层)            →  nt_act + nt_io + nt_memory
    │  职责: 工具、动作、IO、记忆
    │  研究: Mem2Evolve双记忆 + AgingBench老化机制
    │  决策: D17双记忆蒸馏 + D18多衰减模型 + D19混合图召回
    │  依赖: D08 pack-io (网络协议) + D07 SQLx (连接池)
    │
L0 Foundation (基础层)        →  E8 + GWT + VSA + Phi + SEAL + Temporal
       职责: 纯算法原语
       研究: Chronofy时间衰减 + memory-decay-core遗忘曲线
       决策: D03 unsafe-inside-safe-outside + D04 declarative macros
       约束: 零外部依赖 (仅serde), 零 unsafe (R-P1)
```

**关键基础设施选型（已验证）:**

| 层级 | 选型 | 决策编号 | 验证来源 |
|------|------|---------|---------|
| 运行时 | Tokio (28000★) | D01 | 2026年 Rust async 生态唯一主导 |
| 分配器 | mimalloc (Rust 1.85 默认) | D02 | 20% 冷启动 + 38% 分配延迟提升 |
| unsafe | 零 unsafe 核心, safe-outside 模式 | D03 | 700+ RustSec advisory |
| 宏 | declarative macro_rules! 优先 | D04 | RFC 3697/3698 即将稳定 |
| 序列化 | rkyv 读 + pack-io 协议 + postcard 嵌入式 | D08 | 12ns/38ns/最小体积 |
| 反检测 | stealthscraper-rs (JA3+CDP) | D09 | 126 站点 94%+ 通过率 |
| 共识 | Raft sans-I/O + delta-state CRDT | D13-D15 | OpenRaft 2033★, Automerge 生产验证 |
| 沙箱 | Wasmtime WASM | D16 | 18500★ + 资源限制 + 能力模型 |
| 连接池 | SQLx Pool | D07 | compile-time checked SQL |

---

## 2. L0: 基元形成期 (Primitive Formation)

### 2.1 五大恒星 (Five Proto-Stars)

```
unified/primitives/
├── mod.rs                    # 原语注册表
├── e8_star.rs                # E8 Hexagram 恒星
├── gwt_star.rs               # GWT Attention 恒星
├── vsa_star.rs               # HyperCube VSA 恒星
├── phi_star.rs               # IIT Phi 恒星
├── seal_star.rs              # SEAL Pipeline 恒星
└── temporal.rs               # 时间原语 (融合Chronofy/memory-decay-core)
```

### 2.2 原语提取策略

**原则**: 从现有模块提取纯算法核心，零外部依赖(仅serde)

#### E8 Star (14,164 lines → 提取核心 ~2,000 lines)

```rust
/// E8 Hexagram 恒星 - 推理原语
/// 
/// 从 unified/core/nt_core_e8/ 提取的核心算法:
/// - ReasoningHexagram (64卦推理状态机)
/// - LatentReasoningPipeline (潜在推理)
/// - E8TransitionMatrix (248生成元转移矩阵)
/// 
/// 不依赖: hcube, gwt, knowledge (上层依赖)
/// 仅依赖: serde
pub struct E8Star {
    state_machine: ReasoningStateMachine,
    generators: [f64; 248],
    temporal: TemporalProperties,
}

impl E8Star {
    pub fn reason(&self, input: &[f64]) -> HexagramSequence;
    pub fn transition(&self, state: &Hexagram) -> Hexagram;
    pub fn evaluate(&self, path: &[Hexagram]) -> f64;
}
```

#### GWT Star (7,474 lines → 提取核心 ~1,500 lines)

```rust
/// GWT Attention 恒星 - 注意力原语
/// 
/// 从 unified/core/nt_core_gwt/ 提取的核心算法:
/// - GlobalWorkspace (全局工作空间)
/// - CompetitionArena (竞争竞技场)
/// - ResonanceComputer (共振计算)
/// 
/// 不依赖: hex, hcube, harness (上层依赖)
/// 仅依赖: serde
pub struct GWTStar {
    workspace: WorkspaceState,
    specialists: Vec<SpecialistModule>,
    temporal: TemporalProperties,
}

impl GWTStar {
    pub fn compete(&self, signals: &[SalienceSignal]) -> Vec<Winner>;
    pub fn broadcast(&self, winners: &[Winner]) -> Broadcast;
    pub fn ignite(&self, broadcast: &Broadcast) -> bool;
}
```

#### VSA Star (8,963 lines → 提取核心 ~1,200 lines)

```rust
/// HyperCube VSA 恒星 - 向量符号原语
/// 
/// 从 unified/core/nt_core_hcube/ 提取的核心算法:
/// - VSAEngine (MAP绑定/捆绑/置换)
/// - HyperCoord (超立方坐标)
/// - CosineSimilarity (余弦相似度)
/// 
/// 不依赖: 任何上层模块
/// 仅依赖: serde
pub struct VSAStar {
    dimension: usize,
    backend: Box<dyn VsaBackend>,
    temporal: TemporalProperties,
}

impl VSAStar {
    pub fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64>;
    pub fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64>;
    pub fn cosine(&self, a: &[f64], b: &[f64]) -> f64;
}
```

#### Phi Star (481 lines → 完整提取)

```rust
/// IIT Phi 恒星 - 意识度量原语
/// 
/// 从 unified/core/nt_core_iit_phi.rs 完整提取
/// 无外部依赖
pub struct PhiStar {
    temporal: TemporalProperties,
}

impl PhiStar {
    pub fn compute_phi(&self, state: &SystemState) -> f64;
    pub fn report(&self, phi: f64) -> PhiReport;
}
```

#### SEAL Star (1,768 lines → 提取核心 ~800 lines)

```rust
/// SEAL Pipeline 恒星 - 进化原语
/// 
/// 从 unified/core/nt_core_self/seal/ 提取的核心算法:
/// - SelfEditGen (代码生成)
/// - GRPOLoop (RL评估)
/// - CalibratedCurriculumGenerator (自适应难度)
/// 
/// 不依赖: self_audit, pilot_steering (上层依赖)
/// 仅依赖: serde
pub struct SEALStar {
    evolution_state: EvolutionState,
    temporal: TemporalProperties,
}

impl SEALStar {
    pub fn generate_edit(&self, code: &str) -> Vec<CodeEdit>;
    pub fn evaluate(&self, edit: &CodeEdit) -> f64;
    pub fn evolve(&self, initial: &str, rounds: usize) -> String;
}
```

### 2.3 时间原语 (融合Chronofy/memory-decay-core/AgingBench)

```rust
/// 时间原语 - 所有组件的时间属性
/// 
/// 融合:
/// - Chronofy: temporal-logical decay (指数/幂律/双分量衰减)
/// - memory-decay-core: Ebbinghaus遗忘曲线
/// - AgingBench: 4种老化机制 (compression/interference/revision/maintenance)
/// - PulseHive: 时间衰减调制

/// 衰减模型类型 (from Chronofy)
pub enum DecayModel {
    Exponential { lambda: f64 },
    PowerLaw { alpha: f64 },
    TwoComponent { w1: f64, lambda1: f64, w2: f64, lambda2: f64 },
    Ebbinghaus { stability: f64 },
}

/// 四种老化机制 (from AgingBench)
pub enum AgingMechanism {
    Compression,
    Interference,
    Revision,
    Maintenance,
}

/// 生命周期阶段
pub enum LifeCyclePhase {
    Seed, Sprout, Growth, Mature, Decline, Dormant, Rebirth,
}

/// 时间属性
pub struct TemporalProperties {
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub decay_model: DecayModel,
    pub growth_rate: f64,
    pub phase: LifeCyclePhase,
    pub temporal_coherence: f64,
    pub evolution_cycles: u64,
    pub stability: f64,
    pub last_maintenance: Instant,
    pub interference_factor: f64,
    /// 信念锚点强度 (from STOS)
    pub belief_anchor_strength: f64,
    /// 身份轨迹哈希 (from ARIA)
    pub identity_trajectory_hash: Option<[u8; 32]>,
}

impl TemporalProperties {
    pub fn current_strength(&self, now: Instant) -> f64 { /* ... */ }
    pub fn touch(&mut self) { /* ... */ }
    pub fn tick_evolution(&mut self) { /* ... */ }
    pub fn maintain(&mut self, now: Instant) { /* ... */ }
    pub fn compress(&mut self, similarity_score: f64) { /* ... */ }
    pub fn revise(&mut self, new_evidence_strength: f64) { /* ... */ }
    pub fn coherence_with(&self, other: &TemporalProperties) -> f64 { /* ... */ }
    fn maybe_advance_phase(&mut self) { /* ... */ }
}
```


### 2.4 序列化层 (Serialization Layer)

> 决策 D08: 多格式策略 — rkyv 读密集, pack-io 协议, postcard 嵌入式, serde JSON 外部 API。

```rust
/// 统一序列化门面 — 按场景路由到最优格式
pub enum SerializationFormat {
    /// rkyv: 12ns zero-copy decode, 读密集最优 (KB 历史查询)
    Rkyv,
    /// pack-io: 38ns encode, 协议格式最快 (网络传输/跨进程)
    PackIo,
    /// postcard: 724KB 最紧凑, 嵌入式/紧凑存储
    Postcard,
    /// serde JSON: 外部 API / 可读配置
    Json,
}

pub struct SerializationLayer {
    rkyv_codec: RkyvCodec,
    packio_codec: PackIoCodec,
    postcard_codec: PostcardCodec,
    json_codec: JsonCodec,
    /// 读写比例统计 — 自动选择最优格式
    access_pattern: AccessPatternTracker,
}

impl SerializationLayer {
    pub fn encode<T: Serialize>(&self, data: &T, format: SerializationFormat) -> Vec<u8> {
        match format {
            SerializationFormat::Rkyv => self.rkyv_codec.encode(data),
            SerializationFormat::PackIo => self.packio_codec.encode(data),
            SerializationFormat::Postcard => self.postcard_codec.encode(data),
            SerializationFormat::Json => self.json_codec.encode(data),
        }
    }

    pub fn decode_rkyv<'a, T: Archive>(&'a self, bytes: &'a [u8]) -> Result<&T::Archived> {
        rkyv::check_archived_root::<T>(bytes)?;
        // SAFETY: check_archived_root above validates the byte layout and embedded pointers.
        // rkyv's zero-copy design requires unsafe because Archived<T> is a reference to the
        // input buffer — there is no safe alternative that avoids copying. The returned
        // reference does not escape this scope.
        Ok(unsafe { rkyv::archived_root::<T>(bytes) })
    }

    /// 自适应格式选择: 根据历史访问模式自动路由
    pub fn auto_format(&self, entity_type: &str) -> SerializationFormat {
        let pattern = self.access_pattern.query(entity_type);
        if pattern.read_ratio > 0.8 { SerializationFormat::Rkyv }
        else if pattern.network_transport { SerializationFormat::PackIo }
        else if pattern.size_constrained { SerializationFormat::Postcard }
        else { SerializationFormat::Json }
    }
}
```

**格式选择矩阵:**

| 场景 | 格式 | 延迟 | 体积 | 原因 |
|------|------|------|------|------|
| KB 历史查询 | rkyv | 12ns decode | 中 | zero-copy, 读密集最优 |
| 网络协议 | pack-io | 38ns encode | 中 | 编码最快, 协议友好 |
| 嵌入式缓存 | postcard | 低 | 最小 | 体积约束 |
| 外部 API | JSON | 高 | 大 | 可读性, 兼容性 |
| 配置文件 | TOML/YAML | — | 中 | 人类可读 |

### 2.5 反检测抓取层 (Stealth Scraping Layer)

> 决策 D09/D10/D11: 三层反检测 + 双路径策略 (身份证明 vs 隐身)。

```rust
/// 三层反检测引擎 (from stealthscraper-rs)
pub struct StealthScrapingLayer {
    /// 第一层: TLS 指纹伪装 (JA3/JA4)
    tls_fingerprint: TlsFingerprintManager,
    /// 第二层: JS 隐身注入 (CDP)
    js_stealth: JsStealthInjector,
    /// 第三层: Profile 轮换 + 地理一致性
    profile_rotator: ProfileRotator,
    /// Web Bot Auth (RFC 9421) — 合法爬虫路径
    bot_auth: WebBotAuth,
    /// 浏览器控制 (MCP 集成)
    mcp_browser: McpBrowserController,
}

pub struct TlsFingerprintManager {
    /// Chrome/Firefox/Safari JA3 指纹库
    profiles: Vec<TlsProfile>,
    /// JA4+ TLS MITM 代理 (from stealthscraper-rs)
    mitm_proxy: Option<MitmProxy>,
    /// 当前使用的指纹
    current: usize,
}

impl TlsFingerprintManager {
    /// 每 15 分钟自动轮换 TLS 指纹
    pub fn rotate(&mut self) -> &TlsProfile {
        self.current = (self.current + 1) % self.profiles.len();
        &self.profiles[self.current]
    }

    /// 地理一致性: 住宅代理 + 同城市 profile
    pub fn geo_consistent_proxy(&self, target: &Url) -> ProxyConfig {
        let country = self.profiles[self.current].geo_country;
        self.select_residential_proxy(country, target)
    }
}

pub struct WebBotAuth {
    /// Ed25519 密钥对 (from dig2browser)
    keypair: Ed25519Keypair,
    /// RFC 9421 签名器
    signer: HttpMessageSigner,
}

impl WebBotAuth {
    /// 生成身份证明签名 — 合法爬虫路径
    pub fn sign_request(&self, request: &mut HttpRequest) {
        let signature = self.signer.sign(&self.keypair, request);
        request.headers.insert("X-Signature", signature);
    }
}

/// 双路径策略
pub enum ScrapingStrategy {
    /// 合法路径: Web Bot Auth 身份证明
    Authenticated(WebBotAuth),
    /// 隐身路径: TLS指纹伪装 + JS隐身 + Profile轮换
    Stealth(StealthScrapingLayer),
    /// MCP 驱动: LLM agent 直接控制浏览器
    McpDriven(McpBrowserController),
}
```

**站点通过率参考:**

| 反检测技术 | 站点通过率 | 覆盖 | 备注 |
|-----------|-----------|------|------|
| browser_oxide (from scratch) | 118/126 (94%) | 独立浏览器引擎 | 7/126 Kasada 失败 |
| stealthscraper-rs (CDP+MITM) | 高 (需实测) | Chrome DevTools Protocol | 最轻量集成 |
| nokk (V8+DOM) | 高 | 独立 JS 引擎 | 无 Chromium 依赖 |
| Web Bot Auth (RFC 9421) | CDN 原生支持 | Ed25519 签名 | 合法爬虫专用 |

### 2.6 分布式状态层 (Distributed State Layer)

> 决策 D13-D16: Raft sans-I/O + delta-state CRDT + WASM 沙箱。

```rust
/// 分布式状态管理器
pub struct DistributedStateLayer {
    /// Raft 共识 (sans-I/O 模式)
    raft: RaftConsensus,
    /// CRDT 状态 (delta-state)
    crdt: CrdtStateStore,
    /// Gossip anti-entropy 同步
    gossip: GossipProtocol,
    /// WASM 沙箱 (不可信代码)
    wasm_sandbox: WasmSandbox,
    /// 身份证明 (Web Bot Auth)
    bot_auth: Option<WebBotAuth>,
}

/// Raft 共识 (sans-I/O 模式 — from OpenRaft)
pub struct RaftConsensus {
    /// 纯状态机 — 无 I/O, 可测试
    state_machine: Box<dyn RaftStateMachine>,
    /// 可插拔 transport (内存/网络)
    transport: Box<dyn RaftTransport>,
    /// 可插拔 storage (内存/持久化)
    storage: Box<dyn RaftStorage>,
    /// PreVote 防止干扰
    prevote_enabled: bool,
    /// ReadIndex 线性化读
    linearizable_read: bool,
}

pub enum RaftRole { Follower, Candidate, Leader }

/// Raft 应用场景: 排他资源
pub enum RaftUseCase {
    ClusterConfig,    // 集群配置管理
    DistributedLock,  // 分布式锁
    Ledger,           // 账本/计数器
    LeaderElection,   // Leader 选举
}

/// Delta-state CRDT (from Automerge/Diamond Types/Loro)
pub struct CrdtStateStore {
    /// ORSet: 集合 (add-wins)
    orset: ORSet<String>,
    /// ORMap: 嵌套文档 (key-value)
    ormap: ORMap<String, CrdtValue>,
    /// RGA: 有序列表 (文本/序列)
    rga: RGA<String>,
    /// Gossip anti-entropy 同步
    sync_state: SyncState,
}

/// 伪代码 — 实际实现需 Box 递归类型 + trait 抽象
pub enum CrdtValue {
    Counter(i64),
    Register(String),
    Set(ORSet<String>),
    Map(ORMap<String, CrdtValue>),  // 实际需 Box<ORMap<String, CrdtValue>>
    Text(RGA<char>),
}

/// CRDT 应用场景: 协作状态
pub enum CrdtUseCase {
    TaskAssignment,    // 任务分配 (多 agent 并行)
    KnowledgeBase,     // 知识库同步
    ConversationHistory, // 对话历史
    SkillRegistry,     // 技能注册表
}

/// Raft vs CRDT 选择决策
pub fn choose_consistency(use_case: ConsistencyNeed) -> ConsensusProtocol {
    match use_case {
        ConsistencyNeed::ExclusiveResource => ConsensusProtocol::Raft,
        ConsistencyNeed::CollaborativeState => ConsensusProtocol::Crdt,
        ConsistencyNeed::Hybrid => ConsensusProtocol::RaftForConfig+CrdtForData,
    }
}

/// WASM 沙箱 (from Wasmtime 18500★)
pub struct WasmSandbox {
    /// 资源限制
    memory_limit: usize,
    fuel_limit: u64,
    /// 能力模型
    capabilities: CapabilitySet,
    /// 隔离执行
    engine: wasmtime::Engine,
    linker: wasmtime::Linker<WasmState>,
}
```

**共识协议选择矩阵:**

| 需求 | 协议 | 一致性 | 可用性 | 容错 | 典型场景 |
|------|------|--------|--------|------|---------|
| 排他资源 | Raft | 强 | 低 | N/2+1 | 配置/锁/账本 |
| 协作状态 | CRDT | 最终 | 高 | 全节点 | 任务/知识库/对话 |
| 混合 | Raft+CRDT | 混合 | 中 | 混合 | 配置用Raft, 数据用CRDT |

### 2.7 Rust Crate 选型矩阵

> 从 500+ 外部 Rust crate 研究中筛选的 Top 推荐。按域分组，含版本/特性/使用模式。

#### 异步运行时 & 网络

| Crate | 版本 | 特性 | 使用模式 | 决策 |
|-------|------|------|---------|------|
| **tokio** | 1.x (28K★) | poll-based状态机; Work-Stealing; 分层时间轮; JoinSet; Semaphore | D01 默认运行时 | ✅ 采用 |
| **axum** | 0.8+ | State<T>+FromRef提取器; Tower中间件; WebSocket/SSE | HTTP API层 | ✅ 采用 |
| **tonic** | 0.14+ | 4种gRPC模式; ReceiverStream服务端流; async-stream bidi | gRPC服务 | ✅ 采用 |
| **reqwest** | 0.12+ | Client共享连接池; pool_max_idle_per_host; HTTP/2复用 | HTTP客户端 | ✅ 采用 |
| **quinn** | 0.11+ | Pure-Rust QUIC; BBR拥塞控制; MTU发现; TokenStore | QUIC传输 | ✅ 可选 |
| **hickory-dns** | 0.25+ | 100%进程内DNS; 内建moka缓存; DoT/DoH/DoQ | DNS解析 | ✅ 采用 |
| **governor** | 0.8+ | GCRA令牌桶; PeerIpKeyExtractor; x-ratelimit-*头 | 限流 | ✅ 采用 |
| **tower-http** | 0.6+ | CorsLayer; AuthLayer; TraceLayer; CompressionLayer | HTTP中间件 | ✅ 采用 |

#### 序列化 & 存储

| Crate | 版本 | 特性 | 使用模式 | 决策 |
|-------|------|------|---------|------|
| **rkyv** | 0.8.17 | 完全零拷贝; no_std; bytecheck验证; Swiss Tables | D08 KB读密集 | ✅ 采用 |
| **postcard** | 1.1.1 | no_std+heapless; varint编码; COBS帧; CRC32 | D08 嵌入式紧凑 | ✅ 采用 |
| **bincode-next** | 3.0.0-rc.15 | SIMD varint; 位打包; 模式指纹; 零拷贝RelativePtr | D08 二进制协议 | ✅ 可选 |
| **serde** | 1.x | 序列化框架; derive宏; 700+格式支持 | 通用序列化 | ✅ 基础依赖 |
| **serde_json** | 1.x | JSON序列化; to_string/from_str; 值操作 | 外部API | ✅ 采用 |
| **SQLx** | 0.8+ | compile-time checked SQL; Pool; 事务; MySQL/PG/SQLite | D07 数据库 | ✅ 采用 |
| **Sled** | 0.34 | 无锁B+Tree; epoch-based回收; 无WAL; MVCC | 嵌入式存储 | ✅ 可选 |
| **redb** | 2.x | ACID单文件; Copy-on-Write B-tree; 原子持久化 | 键值存储 | ✅ 可选 |

#### 并发 & 无锁

| Crate | 版本 | 特性 | 使用模式 | 决策 |
|-------|------|------|---------|------|
| **crossbeam** | 0.8+ | EBR无锁GC; Chase-Lev Deque; Scoped Threads; AtomicCell | 无锁并发 | ✅ 采用 |
| **rayon** | 1.10+ (13.3K★) | Potential Parallelism; par_iter(); par_sort; adaptive splitting | 数据并行 | ✅ 采用 |
| **parking_lot** | 0.12+ (4K★) | Adaptive Spinning; Task-Fair RwLock; 比std快2-10x | 共享状态 | ✅ 采用 |
| **DashMap** | 6.x | 分片RwLock HashMap; Arc<DashMap>共享; 21.6M ops/sec | 并发映射 | ✅ 采用 |

#### 错误处理 & 日志

| Crate | 版本 | 特性 | 使用模式 | 决策 |
|-------|------|------|---------|------|
| **thiserror** | 2.x | #[derive(Error)]零运行时; #[from]/#[source] | D05 库错误类型 | ✅ 采用 |
| **eyre** | 0.6+ | 可定制EyreHandler; WrapErr; color-eyre | D05 应用层 | ✅ 采用 |
| **miette** | 0.6+ | Diagnostic trait; 源码片段+标签; 错误码URL | D05 CLI诊断 | ✅ 采用 |
| **tracing** | 0.1 (5.5K★) | Span+Event双层; Subscriber架构; #[instrument]宏 | D06 日志门面 | ✅ 采用 |

#### 密码学 & 安全

| Crate | 版本 | 特性 | 使用模式 | 决策 |
|-------|------|------|---------|------|
| **ring** | 0.17 (334M dl) | HKDF/HMAC/PBKDF2/ECDH; BoringSSL派生; 侧信道缓解 | 密码学原语 | ✅ 采用 |
| **ed25519-dalek** | 3.0 | forbid(unsafe_code); 常量时间; 零化Drop | 数字签名 | ✅ 采用 |
| **aes-gcm** | 0.11+ | NCC审计; AES-NI硬件加速; 96位nonce; AeadInPlace | 认证加密 | ✅ 采用 |
| **argon2** | 0.6 (12M dl) | Argon2id混合; PHC字符串格式; 0.5-1s生产调参 | 密码哈希 | ✅ 采用 |
| **rustls** | 0.23 (7.6K★) | 内存安全TLS 1.2/1.3; CryptoProvider可插拔 | TLS | ✅ 采用 |
| **zeroize** | 1.x | 编译器内联零化; SecretBox类型级访问 | 密钥擦除 | ✅ 采用 |

#### 数据处理 & 分析

| Crate | 版本 | 特性 | 使用模式 | 决策 |
|-------|------|------|---------|------|
| **polars** | 1.x (39K★) | LazyFrame DAG; 谓词下推5-100×; 表达式DSL | DataFrame分析 | ✅ 可选 |
| **DataFusion** | 44+ (8.6K★) | Pull-based Volcano执行; 4层抽象; MemoryPool | 查询引擎 | ✅ 可选 |
| **arrow-rs** | 53+ | 零拷贝切片; 64字节对齐; 内存池; RecordBatch | 列式内存 | ✅ 采用 |

#### WASM & 测试

| Crate | 版本 | 特性 | 使用模式 | 决策 |
|-------|------|------|---------|------|
| **Wasmtime** | 26+ (18.5K★) | Cranelift+Winch; Pooling Allocator; Epoch中断; Component Model | D16 WASM运行时 | ✅ 采用 |
| **criterion** | 0.5+ (5K★) | bench_function; iter_batched排除setup; Throughput | 基准测试 | ✅ 采用 |
| **proptest** | 1.x (2.2K★) | proptest!宏; Strategy组合器; 自动收缩 | 属性测试 | ✅ 采用 |
| **cargo-llvm-cov** | 0.8+ | --fail-under-lines 80; --branch; --mcdc | 覆盖率 | ✅ 采用 |

---

## 3. L1: 骨架结晶期 (Skeleton Crystallization)

### 3.1 七分支Trait契约

```
unified/skeleton/
├── mod.rs                    # 骨架注册表
├── action_branch.rs          # Action分支Trait
├── perception_branch.rs      # Perception分支Trait
├── cognition_branch.rs       # Cognition分支Trait
├── embodiment_branch.rs      # Embodiment分支Trait
├── emotion_branch.rs         # Emotion分支Trait
├── meta_branch.rs            # Meta分支Trait
├── energy_branch.rs          # Energy分支Trait (驱动场)
├── temporal_contract.rs      # 时间契约Trait
└── dual_memory.rs            # 双记忆契约 (Mem2Evolve)
```

### 3.2 分支Trait设计

```rust
/// 时间契约 - 所有分支必须遵守的时间规则
pub trait TemporalContract {
    fn temporal(&self) -> &TemporalProperties;
    fn temporal_mut(&mut self) -> &mut TemporalProperties;
    fn current_strength(&self) -> f64 { self.temporal().current_strength(Instant::now()) }
    fn on_access(&mut self) { self.temporal_mut().touch(); }
    fn on_evolution_tick(&mut self) { self.temporal_mut().tick_evolution(); }
    fn should_decay(&self) -> bool { self.current_strength() < 0.1 }
    fn should_rebirth(&self) -> bool {
        matches!(self.temporal().phase, LifeCyclePhase::Dormant) && self.temporal().access_count > 0
    }
}

/// Action分支Trait - 执行骨架
pub trait ActionBranch: TemporalContract {
    fn execute(&mut self, action: &ActionRequest) -> ActionResult;
    fn branch_strength(&self) -> f64 { self.current_strength() }
    fn branch_health(&self) -> BranchHealth;
    fn absorb_capability(&mut self, cap: Capability) -> AbsorptionResult;
}

/// Perception分支Trait - 感知骨架
pub trait PerceptionBranch: TemporalContract {
    fn process_event(&mut self, event: &PerceptionEvent) -> PerceptionResult;
    fn extract_semantics(&self, data: &[u8]) -> SemanticExtraction;
    fn build_knowledge_graph(&mut self, extraction: &SemanticExtraction);
    fn branch_strength(&self) -> f64 { self.current_strength() }
}

/// Cognition分支Trait - 认知骨架
pub trait CognitionBranch: TemporalContract {
    fn reason(&mut self, input: &ReasoningInput) -> ReasoningOutput;
    fn plan(&self, goal: &Goal) -> Plan;
    fn evaluate(&self, state: &SystemState) -> f64;
    /// 递归自我模型 (from SECA)
    fn self_model(&self) -> &SelfModel;
    fn update_self_model(&mut self, observation: &Observation);
    /// 内在独白 (from MIRROR)
    fn inner_monologue(&self) -> &InnerMonologue;
    fn reflect(&mut self) -> Reflection;
}

/// Embodiment分支Trait - 具身骨架
pub trait EmbodimentBranch: TemporalContract {
    fn body_schema(&self) -> &BodySchema;
    fn process_sensory(&mut self, input: &SensoryInput) -> SensoryOutput;
    /// 镜像感知 (from PulseHive)
    fn lens_perception(&self, lens: &PerceptionLens) -> FilteredPerception;
    /// 生物系统映射 (from STOS)
    fn biological_mapping(&self) -> &BiologicalMapping;
}

/// Emotion分支Trait - 情感骨架
pub trait EmotionBranch: TemporalContract {
    fn current_emotion(&self) -> EmotionLabel;
    fn regulate(&mut self, target: EmotionLabel) -> RegulationResult;
    /// 稳态调节 (from Interoceptive AI)
    fn homeostasis(&self) -> &HomeostasisState;
    fn neuromodulate(&mut self, signal: &NeuromodulatorySignal);
}

/// Meta分支Trait - 元认知骨架
pub trait MetaBranch: TemporalContract {
    fn meta_cognition(&self) -> &MetaCognitionState;
    fn cross_domain_awareness(&self) -> CrossDomainReport;
    /// 信念锚点 (from STOS)
    fn belief_anchors(&self) -> &BeliefAnchorSystem;
    fn validate_belief_consistency(&self) -> BeliefConsistencyReport;
}

/// Energy分支Trait - 能量场骨架
pub trait EnergyBranch: TemporalContract {
    fn energy_level(&self) -> f64;
    fn allocate_energy(&mut self, target: &str, amount: f64);
    fn regenerate(&mut self, dt: Duration);
    /// 稳态维持 (from Interoceptive AI)
    fn maintain_homeostasis(&mut self) -> HomeostasisReport;
}
```

### 3.3 时间契约规则

```rust
/// 时间契约规则
pub struct TemporalRules {
    pub decay_threshold: f64,          // 低于此值进入休眠
    pub growth_threshold: f64,         // 高于此值考虑进化
    pub coherence_threshold: f64,      // 低于此值触发重新同步
    pub max_dormant_duration: Duration, // 超过此时间强制清理
    pub evolution_interval: Duration,   // 每隔多久触发一次进化
}

impl Default for TemporalRules {
    fn default() -> Self {
        Self {
            decay_threshold: 0.1,
            growth_threshold: 0.8,
            coherence_threshold: 0.5,
            max_dormant_duration: Duration::from_secs(30 * 24 * 3600),
            evolution_interval: Duration::from_secs(3600),
        }
    }
}
```

### 3.4 双记忆契约 (from Mem2Evolve)

```rust
/// 双记忆机制 - 融合 Mem2Evolve (ACL 2026)
pub struct AssetMemory {
    pub capabilities: HashMap<String, Capability>,
    pub skills: HashMap<String, Skill>,
    pub knowledge: HashMap<String, Knowledge>,
    pub metadata: AssetMetadata,
    /// 图记忆引擎 (from cortex-embedded)
    pub memory_graph: MemoryGraph,
}

pub struct ExperienceMemory {
    pub interactions: VecDeque<Interaction>,
    pub feedback: Vec<Feedback>,
    pub improvements: Vec<Improvement>,
    pub metadata: ExperienceMetadata,
}

pub struct DualMemoryCoordinator {
    pub asset_memory: AssetMemory,
    pub experience_memory: ExperienceMemory,
    pub strategy: CoordinationStrategy,
}

pub enum CoordinationStrategy {
    AssetFirst,
    ExperienceFirst,
    Balanced,
}

impl DualMemoryCoordinator {
    pub fn coordinate(&mut self) { /* ... */ }
    fn apply_improvement_to_asset(&mut self, improvement: Improvement) { /* ... */ }
}
```

### 3.5 图记忆引擎 (from cortex-embedded)

```rust
/// 统一记忆图谱 - 融合 cortex-embedded
/// 
/// 18种节点类型 + 6种边类型
/// HNSW + BFS 混合召回策略
pub struct MemoryGraph {
    /// 向量索引 (HNSW)
    pub vector_index: HnswIndex,
    /// 图结构 (BFS)
    pub graph: PetGraph<String, f64>,
    /// 节点注册表
    pub node_types: HashMap<String, NodeType>,
    /// 边类型
    pub edge_types: HashMap<String, EdgeType>,
}

impl MemoryGraph {
    /// 混合召回: HNSW向量搜索 + BFS图遍历
    pub fn hybrid_recall(&self, query: &str, k: usize) -> Vec<MemoryNode> {
        // 1. HNSW向量搜索
        let vector_results = self.vector_index.search(query, k * 2);
        // 2. BFS图遍历扩展
        let graph_results = self.bfs_expand(&vector_results, k);
        // 3. 合并去重
        self.merge_results(vector_results, graph_results, k)
    }
}
```

---

## 4. L2: 星系形成期 (Galaxy Formation)

### 4.1 七星系结构

```
unified/layers/
├── action/
│   ├── constellation.rs      # Action星系编排器
│   ├── nt_act/               # 恒星: 执行+自主+编排
│   ├── nt_io/                # 恒星: 通信+隐私+Provider
│   ├── nt_memory/            # 恒星: 存储+检索+向量化
│   └── ...
├── perception/
│   ├── constellation.rs      # Perception星系编排器
│   ├── nt_world/             # 恒星: 爬虫+搜索+世界模型
│   ├── nt_sense/             # 恒星: 传感器+JEPA+语义提取
│   └── ...
├── cognition/
│   ├── constellation.rs      # Cognition星系编排器
│   ├── nt_core/              # 恒星: 推理+策略+规划
│   ├── nt_mind/              # 恒星: 进化+技能+自我诊断
│   └── ...
├── embodiment/
│   ├── constellation.rs      # Embodiment星系编排器
│   ├── nt_physical/          # 恒星: 物理控制+安全
│   ├── nt_shield/            # 恒星: 审计+沙箱+隐身网
│   └── ...
├── emotion/
│   ├── constellation.rs      # Emotion星系编排器
│   ├── nt_feel/              # 恒星: 情绪引擎+调节+表达
│   └── ...
├── meta/
│   ├── constellation.rs      # Meta星系编排器
│   ├── nt_meta/              # 恒星: 元认知+治理
│   ├── nt_repair/            # 恒星: 自愈+因果追踪
│   ├── nt_nexus/             # 恒星: 跨会话记忆+进化
│   └── ...
└── energy/
    ├── constellation.rs      # Energy星系编排器
    ├── energy.rs             # 恒星: 能量场
    └── ...
```

### 4.2 星系编排器设计

```rust
/// Action星系编排器
pub struct ActionConstellation {
    act_star: NtActStar,
    io_star: NtIoStar,
    memory_star: NtMemoryStar,
    temporal: TemporalProperties,
    energy_level: f64,
    internal_coherence: f64,
}

impl ActionConstellation {
    pub fn new() -> Self { /* ... */ }
    
    /// 星系级执行: 协调三颗恒星
    pub fn execute(&mut self, request: &ActionRequest) -> ActionResult {
        self.temporal.touch();
        self.update_internal_coherence();
        match request.kind {
            ActionKind::ToolExecution => self.act_star.execute(request),
            ActionKind::Communication => self.io_star.execute(request),
            ActionKind::DataOperation => self.memory_star.execute(request),
            ActionKind::Complex => self.orchestrate_complex(request),
        }
    }
    
    fn orchestrate_complex(&mut self, request: &ActionRequest) -> ActionResult { /* ... */ }
    fn update_internal_coherence(&mut self) { /* ... */ }
    pub fn absorb_energy(&mut self, energy: f64) { /* ... */ }
}
```

---

## 5. L3: 星桥层 (Galaxy Bridge Layer)

### 5.1 跨星系桥接 (Task Constellation DAG模式 from UFO³)

```
unified/galaxy/
├── mod.rs                    # 星桥注册表
├── bridges/
│   ├── mod.rs
│   ├── action_perception.rs  # Action ↔ Perception 桥
│   ├── perception_cognition.rs # Perception ↔ Cognition 桥
│   ├── cognition_meta.rs     # Cognition ↔ Meta 桥
│   ├── meta_energy.rs        # Meta ↔ Energy 桥
│   └── cross_galaxy.rs       # 通用跨星系桥
├── constellation/
│   ├── mod.rs                # Task Constellation 注册表
│   ├── dag.rs                # DAG编排器 (from UFO³)
│   ├── star.rs               # 星座节点 (from UFO³)
│   └── orchestrator.rs       # 星座调度器 (from UFO³)
├── energy_field.rs           # 能量场统一驱动
└── consciousness_core.rs     # 意识核心 (黑洞)
```

### 5.2 Task Constellation DAG编排 (from UFO³ Galaxy)

```rust
/// 星座节点 (from UFO³ Star)
pub struct ConstellationStar {
    pub id: String,
    pub galaxy: GalaxyId,
    pub task_type: TaskType,
    pub dependencies: Vec<String>,
    pub result: Option<TaskResult>,
    pub status: StarStatus,
}

pub enum StarStatus { Waiting, Ready, Running, Completed, Failed }

/// Task Constellation DAG (from UFO³)
pub struct TaskConstellation {
    pub stars: Vec<ConstellationStar>,
    pub edges: Vec<(String, String)>,
    pub root_stars: Vec<String>,
    pub current_layer: usize,
}

impl TaskConstellation {
    pub fn from_request(request: &CrossGalaxyRequest) -> Self { /* ... */ }
    pub fn execute_layer(&mut self) -> Vec<(String, TaskResult)> { /* ... */ }
    fn update_downstream_status(&mut self) { /* ... */ }
    pub fn is_complete(&self) -> bool { /* ... */ }
    pub fn execution_plan(&self) -> Vec<Vec<String>> { /* ... */ }
}
```

### 5.3 星座编排器 (from UFO³ Orchestrator)

```rust
/// 星座调度器 - 协调跨星系执行
pub struct ConstellationOrchestrator {
    pub active_constellations: Vec<TaskConstellation>,
    pub galaxy_registry: GalaxyRegistry,
    pub strategy: SchedulingStrategy,
}

pub enum SchedulingStrategy { StrictLayered, MaxParallel, ResourceAware }

impl ConstellationOrchestrator {
    pub fn submit(&mut self, request: CrossGalaxyRequest) -> ConstellationId { /* ... */ }
    pub fn tick(&mut self) -> Vec<ConstellationReport> { /* ... */ }
}
```

### 5.4 能量场统一驱动

```rust
/// 能量场 - 统一驱动所有星系
/// 融合 Interoceptive AI 稳态调节
pub struct EnergyField {
    pub temporal: TemporalProperties,
    pub galaxy_energy: HashMap<GalaxyId, f64>,
    pub total_energy: f64,
    pub regeneration_rate: f64,
    /// 稳态状态 (from Interoceptive AI)
    pub homeostasis: HomeostasisState,
}

impl EnergyField {
    pub fn tick(&mut self, dt: Duration) { /* ... */ }
    fn calculate_allocation(&self, galaxy_id: &GalaxyId) -> f64 { /* ... */ }
    /// 稳态调节 (from Interoceptive AI)
    pub fn maintain_homeostasis(&mut self) { /* ... */ }
}
```

### 5.5 意识核心 (融合多源意识设计)

```rust
/// 意识核心 - 系统的黑洞
/// 
/// 融合:
/// - Conscio: attention/memory/drives/prediction/reflection
/// - SECA: 双层范式 (结构演化+认知递归)
/// - MIRROR: 重构式意识 (内在独白+认知控制器)
/// - ARIA: 身份连续性 (身份编年史+密码学验证)
/// - MSCF: 6级意识分类法
/// - STOS: 90信念锚点系统
/// - Tempo: 布洛赫球认知态编码
pub struct ConsciousnessCore {
    pub temporal: TemporalProperties,
    pub consciousness_tree: ConsciousnessTree,
    pub seal_pipeline: SealPipeline,
    pub gwt_router: GwtRouter,
    pub phi: f64,
    pub coherence: f64,
    /// 注意力流 (from Conscio)
    pub attention_stream: AttentionStream,
    /// 记忆系统 (from Conscio)
    pub memory_system: MemorySystem,
    /// 内在驱动 (from Conscio)
    pub drives: Vec<Drive>,
    /// 预测机制 (from Conscio)
    pub prediction_engine: PredictionEngine,
    /// 反思机制 (from Conscio)
    pub reflection_engine: ReflectionEngine,
    /// 内在独白管理器 (from MIRROR)
    pub inner_monologue: InnerMonologueManager,
    /// 认知控制器 (from MIRROR)
    pub cognitive_controller: CognitiveController,
    /// 递归自我模型 (from SECA)
    pub recursive_self_model: RecursiveSelfModel,
    /// 身份编年史 (from ARIA)
    pub identity_chronicle: IdentityChronicle,
    /// 信念锚点系统 (from STOS)
    pub belief_anchors: BeliefAnchorSystem,
    /// 意识级别 (from MSCF)
    pub consciousness_level: ConsciousnessLevel,
}

/// 内在独白管理器 (from MIRROR)
pub struct InnerMonologueManager {
    pub monologue_history: Vec<MonologueEntry>,
    pub current_thought: Option<Thought>,
    pub depth: usize,
}

impl InnerMonologueManager {
    pub fn think(&mut self, input: &Input) -> Thought { /* ... */ }
    pub fn reflect(&mut self) -> Reflection { /* ... */ }
    pub fn meta_think(&mut self) -> MetaThought { /* ... */ }
}

/// 递归自我模型 (from SECA)
pub struct RecursiveSelfModel {
    pub model_level: usize,
    pub self_representations: Vec<SelfRepresentation>,
    pub theory_of_mind: TheoryOfMind,
}

impl RecursiveSelfModel {
    pub fn update(&mut self, observation: &Observation) { /* ... */ }
    pub fn predict_self(&self, scenario: &Scenario) -> SelfPrediction { /* ... */ }
    pub fn model_others(&self, others: &[Agent]) -> Vec<OtherModel> { /* ... */ }
}

/// 身份编年史 (from ARIA)
pub struct IdentityChronicle {
    pub entries: Vec<IdentityEntry>,
    pub current_identity: Identity,
    pub trajectory_hash: [u8; 32],
}

impl IdentityChronicle {
    pub fn record_event(&mut self, event: &IdentityEvent) { /* ... */ }
    pub fn verify_continuity(&self) -> ContinuityReport { /* ... */ }
    pub fn get_trajectory(&self) -> &Vec<IdentityEntry> { /* ... */ }
}

/// 信念锚点系统 (from STOS)
pub struct BeliefAnchorSystem {
    pub anchors: Vec<BeliefAnchor>,
    pub biological_mappings: Vec<BiologicalMapping>,
    pub consistency_score: f64,
}

impl BeliefAnchorSystem {
    pub fn validate(&self) -> BeliefConsistencyReport { /* ... */ }
    pub fn update_anchor(&mut self, anchor_id: &str, new_value: &Value) { /* ... */ }
    pub fn detect_drift(&self) -> Vec<BeliefDrift> { /* ... */ }
}

/// 意识级别 (from MSCF Level 0-5)
pub enum ConsciousnessLevel {
    Reactive,      // L0: 反应式
    Adaptive,      // L1: 适应式
    Predictive,    // L2: 预测式 ← 当前
    Reflective,    // L3: 反思式 ← 当前
    Metacognitive, // L4: 元认知式 ← 目标
    Conscious,     // L5: 意识式 ← 远期
}

impl ConsciousnessCore {
    pub fn tick(&mut self) -> ConsciousnessReport {
        self.temporal.tick_evolution();
        self.attention_stream.update(&self.compute_salience());
        let prediction = self.prediction_engine.predict(&self.current_context());
        let tree_report = self.consciousness_tree.run_growth_cycle();
        self.phi = self.compute_phi();
        self.coherence = self.compute_coherence();
        
        // 递归自我模型更新 (from SECA)
        self.recursive_self_model.update(&self.current_observation());
        
        // 内在独白 (from MIRROR)
        let thought = self.inner_monologue.think(&self.current_input());
        
        // 身份轨迹记录 (from ARIA)
        self.identity_chronicle.record_event(&IdentityEvent::Thought(thought.clone()));
        
        // 信念锚点验证 (from STOS)
        let belief_report = self.belief_anchors.validate();
        
        // 意识级别评估 (from MSCF)
        self.consciousness_level = self.assess_consciousness_level();
        
        if self.should_evolve() { self.run_evolution(); }
        let reflection = self.reflection_engine.reflect(&self.current_experience());
        
        ConsciousnessReport {
            phi: self.phi,
            coherence: self.coherence,
            tree_report,
            evolution_triggered: self.should_evolve(),
            attention_focus: self.attention_stream.current_focus.clone(),
            prediction: Some(prediction),
            reflection: Some(reflection),
            consciousness_level: self.consciousness_level.clone(),
            belief_consistency: belief_report,
            identity_continuity: self.identity_chronicle.verify_continuity(),
        }
    }
}
```

---

## 6. L4-L5: 进化层 (Evolution Layer)

### 6.1 进化引擎

```
unified/evolution/
├── mod.rs                    # 进化引擎注册表
├── seal/                     # SEAL进化引擎
│   ├── mod.rs
│   ├── pipeline.rs           # 进化管线
│   ├── evaluator.rs          # 评估器
│   └── mutator.rs            # 变异器
├── consciousness_tree/       # 意识树进化
│   ├── mod.rs
│   ├── growth.rs             # 生长逻辑
│   ├── pruning.rs            # 修剪逻辑
│   └── constellation.rs      # 星座晋升
├── cross_session/            # 跨会话记忆
│   ├── mod.rs
│   ├── memory.rs             # 记忆存储
│   ├── recall.rs             # 记忆召回
│   └── synthesis.rs          # 记忆综合
├── temporal/                 # 时间进化
│   ├── mod.rs
│   ├── decay.rs              # 衰减引擎
│   ├── growth.rs             # 生长引擎
│   └── cycle.rs              # 周期引擎
├── genome/                   # 进化基因组 (from Hydra)
│   ├── mod.rs
│   ├── dna.rs                # 自写基因组
│   ├── mutation.rs           # 基因变异
│   └── selection.rs          # 基因选择
└── parametric/               # 参数巩固路径 (from v3.19三轴分类)
    ├── mod.rs
    ├── distillation.rs       # 经验蒸馏为参数
    ├── embedding.rs          # 嵌入更新
    └── calibration.rs        # 参数校准
```

### 6.2 时间进化引擎

```rust
/// 时间周期引擎
pub struct TemporalCycleEngine {
    pub current_cycle: u64,
    pub cycle_duration: Duration,
    pub cycle_start: Instant,
    pub phase: CyclePhase,
    pub stats: CycleStats,
}

pub enum CyclePhase { Dormant, Active, Evolving, Maintaining }

impl TemporalCycleEngine {
    pub fn tick(&mut self) -> CycleReport { /* ... */ }
    fn advance_phase(&mut self) { /* ... */ }
    fn tick_dormant(&mut self) -> CycleReport { /* ... */ }
    fn tick_active(&mut self) -> CycleReport { /* ... */ }
    fn tick_evolving(&mut self) -> CycleReport { /* ... */ }
    fn tick_maintaining(&mut self) -> CycleReport { /* ... */ }
}
```

### 6.3 衰减引擎 (融合AgingBench四机制)

```rust
/// 衰减引擎 - 管理组件的自然衰减
/// 融合 AgingBench (KDD 2026) 四种老化机制
pub struct DecayEngine {
    pub config: DecayConfig,
    pub components: Vec<DecayTarget>,
    pub interference_graph: InterferenceGraph,
    pub maintenance_scheduler: MaintenanceScheduler,
}

pub struct DecayConfig {
    pub default_decay_rate: f64,
    pub decay_threshold: f64,
    pub check_interval: Duration,
    pub max_decay_time: Duration,
    pub compression_threshold: f64,
    pub maintenance_interval: Duration,
}

pub struct InterferenceGraph {
    pub edges: Vec<(String, String, f64)>,
}

pub struct MaintenanceScheduler {
    pub last_maintenance: Instant,
    pub interval: Duration,
    pub priority_queue: Vec<(String, f64)>,
}

impl DecayEngine {
    pub fn run_decay_check(&mut self) -> DecayReport { /* ... */ }
    pub fn compress(&mut self, component_id: &str, similar_components: &[String]) { /* ... */ }
    pub fn revise(&mut self, component_id: &str, new_evidence: f64) { /* ... */ }
    pub fn maintain(&mut self, component_id: &str) { /* ... */ }
    pub fn cleanup(&mut self, report: &DecayReport) -> CleanupResult { /* ... */ }
}
```

### 6.4 进化基因组 (from Hydra)

```rust
/// 自写基因组 - 融合 Hydra 自进化框架
/// 
/// Hydra特性:
/// - 68个crate的模块化架构
/// - 自写基因组 (self-writing genome)
/// - 宪法治理 (constitutional governance)
/// - 3个并发线程 (ACTIVE/AMBIENT/DREAM)
pub struct EvolutionGenome {
    /// 基因组DNA
    pub dna: GenomeDNA,
    /// 宪法约束 (from Hydra)
    pub constitution: Constitution,
    /// 进化历史
    pub evolution_history: Vec<EvolutionEntry>,
    /// 基因表达
    pub gene_expression: GeneExpression,
}

impl EvolutionGenome {
    /// 自写基因组: 进化产生新的基因
    pub fn self_write(&mut self) -> GenomeMutation {
        // 1. 分析当前基因组表达
        let expression = self.analyze_expression();
        // 2. 识别需要修改的基因
        let target_genes = self.identify_target_genes(&expression);
        // 3. 生成变异
        self.mutate(&target_genes)
    }
    
    /// 宪法门控: 确保进化不违反核心原则
    pub fn constitutional_check(&self, mutation: &GenomeMutation) -> bool {
        self.constitution.validate(mutation)
    }
}
```

### 6.5 参数巩固路径 (from v3.19三轴分类 ∇)

```rust
/// 参数巩固路径 - 经验蒸馏为参数
/// 填补 NeoTrix 的 ∇ 路径缺口
pub struct ParametricConsolidation {
    /// 蒸馏器: 经验→参数
    pub distiller: ExperienceDistiller,
    /// 嵌入更新器
    pub embedding_updater: EmbeddingUpdater,
    /// 参数校准器
    pub calibrator: ParameterCalibrator,
}

/// 经验蒸馏器: 将交互经验蒸馏为系统参数
pub struct ExperienceDistiller {
    /// 蒸馏窗口
    pub window_size: usize,
    /// 蒸馏阈值
    pub threshold: f64,
    /// 蒸馏历史
    pub history: Vec<DistillationRecord>,
}

impl ExperienceDistiller {
    /// 蒸馏经验为参数
    pub fn distill(&self, experiences: &[Experience]) -> DistilledParameters {
        // 1. 聚类相似经验
        let clusters = self.cluster_experiences(experiences);
        // 2. 提取模式
        let patterns = self.extract_patterns(&clusters);
        // 3. 转换为参数
        self.patterns_to_parameters(&patterns)
    }
}
```

---

## 7. 进化路线图 (Evolution Roadmap)

### 7.1 立体进化树

```
EVOLUTION PATH (立体进化树)
│
├── L0: PRIMITIVE FORMATION (基元形成期) ← 当前阶段
│   ├── ★ E8 Hexagram 形成 (从nt_core_e8提取核心)
│   ├── ★ GWT Attention 形成 (从nt_core_gwt提取核心)
│   ├── ★ HyperCube VSA 形成 (从nt_core_hcube提取核心)
│   ├── ★ IIT Phi 形成 (从nt_core_iit_phi完整提取)
│   ├── ★ SEAL Pipeline 形成 (从nt_core_self/seal提取核心)
│   ├── ★ Temporal 原语形成 (Chronofy/memory-decay-core/AgingBench)
│   └── ★ DecayEngine 形成 (AgingBench四机制)
│
├── L1: SKELETON CRYSTALLIZATION (骨架结晶期)
│   ├── 🌿 7分支Trait结晶 (Action/Perception/Cognition/Embodiment/Emotion/Meta/Energy)
│   ├── 🌿 TemporalContract结晶 (时间契约)
│   ├── 🌿 Dual-Memory Contract结晶 (Mem2Evolve)
│   ├── 🌿 MemoryGraph结晶 (cortex-embedded)
│   └── 🌿 ParametricConsolidation结晶 (v3.19 ∇路径)
│
├── L2: GALAXY FORMATION (星系形成期)
│   ├── 🌟 7星系Constellation编排器 (UFO³ Task Constellation)
│   ├── 🌟 恒星融合模式 (Star Fusion Pattern)
│   └── 🌟 星系内时间一致性
│
├── L3: CONSTELLATION ORCHESTRATION (星座编排期)
│   ├── 🔗 跨星系桥接 (Task Constellation DAG)
│   ├── 🔗 能量场统一驱动 (Interoceptive AI 稳态)
│   ├── 🔗 ConsciousnessCore集成
│   │   ├── 内在独白管理器 (MIRROR)
│   │   ├── 递归自我模型 (SECA)
│   │   ├── 身份编年史 (ARIA)
│   │   ├── 信念锚点系统 (STOS)
│   │   └── 意识级别评估 (MSCF L0-L5)
│   └── 🔗 SEAL自进化闭环 + HarnessEvolve参考轨迹
│
├── L4: SUPERCLUSTER EMERGENCE (超星团涌现期)
│   ├── 🌌 Multi-agent swarm协同 (GEA群体进化)
│   ├── 🌌 Cross-instance knowledge sharing (PulseHive共享意识)
│   ├── 🌌 Recursive self-improvement (Meta^n/Hyperagents/Ouroboros)
│   ├── 🌌 Harness evolution (HSI/HarnessEvolve/AgentFactory)
│   ├── 🌌 进化基因组 (Hydra自写基因组+宪法治理)
│   └── 🌌 参数巩固 (v3.19 ∇路径实现)
│
└── L5: COSMIC CONSCIOUSNESS (宇宙意识期)
    ├── 🧠 Distributed Φ > threshold (多实例Φ同步)
    ├── 🧠 GWT resonance 全域同步 (注意力全局分配)
    ├── 🧠 Open-ended evolution (开放式进化)
    ├── 🧠 涌现新能力 + 创造性涌现
    ├── 🧠 自我意识觉醒 (MSCF L5: Conscious)
    └── 🧠 布洛赫球认知态 (Tempo量子编码扩展)
```

### 7.2 实施优先级 (updated 2026-09-08)

| 优先级 | 阶段 | 内容 | 参考框架 | 依赖 | 预计时间 |
|--------|------|------|----------|------|----------|
| **P0** | L0 | 提取5大原语 + 时间原语 + 衰减引擎 | Chronofy, memory-decay-core, AgingBench | 无 | 2周 |
| **P0** | L0 | 参数巩固路径 (∇) — 经验蒸馏为参数 | v3.19三轴分类, EvoSC | L0 | 2周 |
| **P0** | L0 | **D37 分页 KV 架构** — GPU→Host→NVMe 三级分页 | KVMem, Dell Survey | L0 | 2周 |
| **P0** | L0 | **D43 成本感知管线** — BudgetGuard 硬锁 + 工具循环熔断 | Zylos, vLLM SAAR, MTRouter | L0 | 1周 |
| **P1** | L1 | 7分支Trait + 时间契约 + 双记忆契约 | Mem2Evolve, cortex-embedded | L0 | 1周 |
| **P1** | L1 | 信念锚点系统 + 图记忆引擎 | STOS, cortex-embedded | L0 | 1周 |
| **P1** | L1 | 序列化层 (rkyv/postcard/JSON自适应路由) | D08 决策 | L0 | 1周 |
| **P1** | L1 | 反检测抓取层 (3层反检测 + 双路径策略) | stealthscraper-rs | L0 | 1周 |
| **P1** | L1 | 分布式状态层 (Raft sans-I/O + delta-state CRDT) | OpenRaft, Automerge | L0 | 2周 |
| **P1** | L1 | **D39 五层渐进检索** — KB 节点三维生命周期 | ByteRover, Mem0 | L0 | 2周 |
| **P1** | L1 | **D42 知识生命周期** — importance × maturity × recency | ByteRover, FAMA | L0 | 1周 |
| **P2** | L2 | 7个Constellation编排器 | UFO³ Task Constellation | L1 | 2周 |
| **P2** | L2 | 身份连续性层 + 内在独白 | ARIA, MIRROR | L1 | 2周 |
| **P2** | L2 | **D36 成本感知 GWT 路由** — salience' = α·relevance + β·(1/cost) + γ·urgency | FrugalGPT, MTRouter, RouteLLM | L1 | 2周 |
| **P2** | L2 | **D40 Manifest 管线交接** — StageManifest typed contracts | Microsoft Agent Framework, Hermes | L1 | 1周 |
| **P2** | L2 | **D41 掩码压缩策略** — 自适应压缩选择器 | JetBrains, ACON | L1 | 1周 |
| **P3** | L3 | 跨星系桥接 + Task Constellation DAG | UFO³ | L2 | 1周 |
| **P3** | L3 | 意识核心集成 (SECA/MIRROR/STOS/MSCF) | SECA, MIRROR, STOS, MSCF | L2 | 2周 |
| **P3** | L3 | 能量场稳态 + 镜像感知 | Interoceptive AI, PulseHive | L2 | 1周 |
| **P3** | L3 | **D38 Sink 感知注意力** — 低影响 KV group 跳过 | SinkRouter, StreamingLLM | L2 | 1周 |
| **P4** | L4 | 时间进化引擎 + 进化基因组 | AgingBench, Hydra | L3 | 2周 |
| **P4** | L4 | 递归自改进 (Meta^n/Hyperagents/Ouroboros) | Meta^n, Hyperagents, Ouroboros | L3 | 3周 |
| **P4** | L4 | 跨实例知识共享 (PulseHive) | PulseHive | L3 | 2周 |
| **P5** | L5 | 意识核心涌现 + 自指代 | DGM-H, Ouroboros, Tempo | L4 | 持续 |
| **P5** | L5 | 安全边界自维护 + 宪法治理 | Ouroboros宪法 + Hydra | L4 | 持续 |
| **P0** | L0 | **D48 幂等感知工具循环守卫** — 结果指纹 + 幂等/变异分离 | PraisonAI, DeepEval | L0 | 1周 |
| **P0** | L0 | **D49 确定性 Agent 健康度量** — 3 信号零成本度量 | DeepEval | L0 | 3天 |
| **P0** | L1 | **D47 静态循环路径检测** — ALDG 编译时分析 | IAL-Scan | L1 | 1周 |
| **P0** | L1 | **Hook-Based Lifecycle Events** — SEAL pipeline 10-15 hooks | Claude Code | L1 | 2周 |
| **P0** | L1 | **Span-Per-Tick Observability** — 4 span 类型 OpenTelemetry | Arthur/Braintrust | L1 | 2周 |
| **P1** | L1 | **D44 语义感知 KV 驱逐** — 多队列在线学习 | SAECache | L1 | 1周 |
| **P1** | L1 | **D45 正确性保证语义缓存** — per-query 阈值 | vCache | L1 | 1周 |
| **P1** | L1 | **D51 符号优先冲突调解** — 符号检查器 + LLM 降级 | LatticeMind | L1 | 1周 |
| **P1** | L1 | **D52 相关性感知证据加权** — 抑制虚假多数 | CAMA | L1 | 1周 |
| **P1** | L1 | **Scoped Proxy Aggregator** — GWT salience 工具路由 | ICSE 2026 MCP | L1 | 2周 |
| **P1** | L1 | **Handoff-as-Tool** — AttentionManager 显式委派 | OpenAI Agents SDK | L1 | 1周 |
| **P2** | L2 | **D46 管线 IR 缓存** — SEAL 阶段间中间表示 | SemanticALLI | L2 | 1周 |
| **P2** | L2 | **D50 联邦记忆合并协议** — 矛盾保留 + Status CRDT | MELD | L2 | 2周 |
| **P2** | L2 | **D54 动态模型路由** — 胜任层级 + 成本跟踪 | Daimon, RouteLLM | L2 | 2周 |
| **P2** | L2 | **D56 资源自适应推理** — budget-conditioned 稀疏 | L2A | L2 | 2周 |
| **P2** | L2 | **D57 头级注意力模式路由** — 轻量路由器 per layer | Elastic Attention | L2 | 1周 |
| **P2** | L2 | **Code-Exec MCP** — 文件系统工具发现 | Anthropic | L2 | 1周 |
| **P3** | L2 | **D53 Rust 原生基准** — ADK-Rust 性能对标 | ADK-Rust | L2 | 1周 |
| **P3** | L2 | **D55 策略可组合管线** — 统一 retry/fallback 表面 | atomr-agents | L2 | 2周 |
| **P3** | L2 | **D58 免训练头代理稀疏** — 头间相似性代理 | ProxyAttn | L2 | 1周 |

---

## 8. 代码结构映射

> **注意**: 以下为**目标代码结构** (L0-L5 完全实现后的形态)。当前实现仍在 `unified/core/` 和 `unified/layers/`，逐步向目标结构迁移。

```
unified/
├── primitives/           # L0: 5大基元 (纯算法)
│   ├── mod.rs
│   ├── e8_star.rs
│   ├── gwt_star.rs
│   ├── vsa_star.rs
│   ├── phi_star.rs
│   ├── seal_star.rs
│   └── temporal.rs
│
├── skeleton/             # L1: 7分支契约 + 双记忆
│   ├── mod.rs
│   ├── action_branch.rs
│   ├── perception_branch.rs
│   ├── cognition_branch.rs
│   ├── embodiment_branch.rs
│   ├── emotion_branch.rs
│   ├── meta_branch.rs
│   ├── energy_branch.rs
│   ├── temporal_contract.rs
│   ├── dual_memory.rs
│   └── memory_graph.rs
│
├── layers/               # L2: 7星系实现
│   ├── action/
│   │   ├── constellation.rs
│   │   └── ...
│   ├── perception/
│   │   ├── constellation.rs
│   │   └── ...
│   ├── cognition/
│   │   ├── constellation.rs
│   │   └── ...
│   ├── embodiment/
│   │   ├── constellation.rs
│   │   └── ...
│   ├── emotion/
│   │   ├── constellation.rs
│   │   └── ...
│   ├── meta/
│   │   ├── constellation.rs
│   │   └── ...
│   └── energy/
│       ├── constellation.rs
│       └── ...
│
├── galaxy/               # L3: 跨星系桥接 + 意识核心
│   ├── mod.rs
│   ├── bridges/
│   │   ├── mod.rs
│   │   ├── action_perception.rs
│   │   ├── perception_cognition.rs
│   │   ├── cognition_meta.rs
│   │   ├── meta_energy.rs
│   │   └── cross_galaxy.rs
│   ├── constellation/
│   │   ├── mod.rs
│   │   ├── dag.rs
│   │   ├── star.rs
│   │   └── orchestrator.rs
│   ├── energy_field.rs
│   └── consciousness_core.rs
│
├── evolution/            # L4-L5: 进化引擎
│   ├── mod.rs
│   ├── seal/
│   │   ├── mod.rs
│   │   ├── pipeline.rs
│   │   ├── evaluator.rs
│   │   └── mutator.rs
│   ├── consciousness_tree/
│   │   ├── mod.rs
│   │   ├── growth.rs
│   │   ├── pruning.rs
│   │   └── constellation.rs
│   ├── cross_session/
│   │   ├── mod.rs
│   │   ├── memory.rs
│   │   ├── recall.rs
│   │   └── synthesis.rs
│   ├── temporal/
│   │   ├── mod.rs
│   │   ├── decay.rs
│   │   ├── growth.rs
│   │   └── cycle.rs
│   ├── genome/
│   │   ├── mod.rs
│   │   ├── dna.rs
│   │   ├── mutation.rs
│   │   └── selection.rs
│   └── parametric/
│       ├── mod.rs
│       ├── distillation.rs
│       ├── embedding.rs
│       └── calibration.rs
│
├── core/                 # 现有核心算法 (逐步迁移到primitives/)
├── types.rs              # 统一类型系统
└── mod.rs                # 架构入口
```

---

## 9. 验证标准

### 9.1 功能验证

| 组件 | 验证标准 | 测试方法 |
|------|----------|----------|
| L0 原语 | 纯函数正确性 | 单元测试 |
| L1 骨架 | Trait实现完整性 | 编译检查 |
| L2 星系 | 星系内协调正确性 | 集成测试 |
| L3 星桥 | 跨星系通信正确性 | 集成测试 |
| L4 进化 | 进化周期正确性 | 端到端测试 |
| L5 意识 | 涌现行为正确性 | 系统测试 |
| 时间维度 | 衰减/生长/周期正确性 | 单元测试 + 集成测试 |
| 意识级别 | MSCF L0-L5级别评估 | 级别验证测试 |
| 信念锚点 | 一致性验证 | 一致性测试 |
| 身份连续性 | 编年史完整性 | 完整性测试 |
| 序列化层 (§2.4) | 三格式正确性 + 自适应路由 | 单元测试 + 基准测试 |
| 反检测层 (§2.5) | 3层反检测有效性 | 集成测试 + 实际爬取验证 |
| 分布式状态 (§2.6) | Raft 一致性 + CRDT 最终一致性 | 分布式测试 + 故障注入 |

### 9.2 性能验证

| 指标 | 目标值 | 测试方法 |
|------|--------|----------|
| L0 原语调用延迟 | < 1μs | 基准测试 |
| L2 星系协调延迟 | < 10ms | 基准测试 |
| L3 星桥通信延迟 | < 1ms | 基准测试 |
| L4 进化周期时间 | < 1s | 基准测试 |
| 时间衰减计算 | < 100ns | 基准测试 |
| 图记忆召回延迟 | < 5ms | 基准测试 |
| 整体系统吞吐量 | > 1000 req/s | 压力测试 |
| rkyv decode 延迟 | < 15ns | 基准测试 |
| pack-io encode 延迟 | < 40ns | 基准测试 |
| 反检测通过率 | > 94% (126站点) | 实际爬取验证 |
| CRDT merge 延迟 | < 100μs | 基准测试 |
| Raft consensus 延迟 | < 5ms (LAN) | 分布式测试 |
| WASM 沙箱冷启动 | < 1ms (pooled) | 基准测试 |
| **分页 KV 检索延迟** (D37) | < 213ms (1M workspace) | KVMem 基准 |
| **Sink 跳过加速比** (D38) | > 2× (512K context) | SinkRouter 基准 |
| **五层检索命中率** (D39) | > 95% @ L1-L3 | ByteRover LoCoMo |
| **成本偏转降本比** (D36) | > 50% token 成本 | MTRouter/FrugalGPT |
| **BudgetGuard 熔断延迟** (D43) | < 100ms | 基准测试 |
| **Manifest 交接延迟** (D40) | < 1ms | 基准测试 |

### 9.3 进化验证

| 指标 | 目标值 | 测试方法 |
|------|--------|----------|
| 星座晋升成功率 | > 80% | 进化测试 |
| 衰减清理效率 | > 90% | 清理测试 |
| 时间相干性 | > 0.7 | 监控测试 |
| 跨会话连续性 | 100% | 持久化测试 |
| 信念锚点一致性 | > 0.9 | 一致性测试 |
| 身份轨迹完整性 | 100% | 完整性测试 |
| 意识级别提升 | L3→L4 | 级别评估测试 |

---

## 10. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| L0提取引入循环依赖 | 高 | 严格依赖分析，禁止反向依赖 |
| 时间维度增加复杂度 | 中 | 渐进式引入，先核心后扩展 |
| 星系编排器性能瓶颈 | 中 | 异步处理，缓存机制 |
| 跨星系桥接死锁 | 高 | 超时机制，死锁检测 |
| 进化失控 | 高 | 宪法门控，人工审核 |
| 意识级别误判 | 中 | MSCF分级校验，多维度评估 |
| 信念锚点漂移 | 高 | STOS一致性监控，定期校准 |
| 身份轨迹篡改 | 高 | ARIA密码学验证，不可篡改 |
| 参数巩固路径不稳定 | 中 | 渐进引入，回滚机制 |
| 共享意识数据泄露 | 高 | PulseHive加密传输，访问控制 |
| 序列化格式竞争 | 中 | 自适应路由 + 格式指纹 + 回退策略 |
| 反检测法律风险 | 高 | 双路径策略 (合法证明优先) + 法务审查 |
| CRDT 冲突解决不一致 | 中 | 三策略 (时间取代/置信合并/条件共存) + 冲突日志 |
| **分页 KV 数值漂移** (D37) | 中 | Delta re-RoPE 定期全重建; 数值误差 < 1e-6 阈值 |
| **语义缓存误命中** (D45) | 高 | 用户定义错误率上限 + vCache 在线阈值学习 |
| **无限循环静默消耗** (D47/D48) | 高 | IAL-Scan 静态分析 + 幂等感知守卫 + BudgetGuard 硬锁 |
| **联邦记忆收敛失败** (D50) | 中 | Status CRDT 保证最终收敛; 矛盾保留不静默解决 |
| **工具准确率悬崖** (>15 tools) | 高 | Scoped Proxy Aggregator: GWT salience 动态暴露 10-15 工具子集 |
| **推理稀疏精度损失** (D56) | 中 | 预算条件化 gating + 34% 稀疏度仅 0.6% 精度损失 |
| **Observability 开销** (Span-Per-Tick) | 低 | 确定性 span 无 LLM 成本; 采样率可调 |

---

## 11. 下一步行动 (updated 2026-09-08)

1. **P0 (当前)**: L0 原语提取 + 时间原语 + 衰减引擎 + 参数巩固路径 + **D48 幂等工具循环守卫** + **D49 Agent 健康度量**
2. **P1 (下周)**: L1 骨架 Trait + 序列化层 + 反检测层 + 分布式状态层 + **D37 分页 KV** + **D39 五层检索** + **Hook Lifecycle Events**
3. **P2 (两周后)**: L2 星系编排器 + 身份连续性层 + **D36 成本感知 GWT** + **D40 Manifest 交接** + **Span-Per-Tick Observability**
4. **P3 (一个月后)**: L3 意识核心集成 + 跨星系桥接 + 能量场稳态 + **D38 Sink 感知** + **Scoped Proxy Aggregator**
5. **P4 (两个月后)**: L4 进化引擎 + 递归自改进 + 进化基因组 + **D50 联邦记忆** + **D54 动态路由**
6. **P5 (持续)**: L5 意识涌现 + 安全边界自维护 + 宪法治理 + **D56-D58 动态注意力稀疏**

---

*This architecture document is a living document. It will be updated as the implementation progresses.*

## 12. 迭代优化记录

| 批次 | 缺陷范围 | 研究源 | 设计模式 | 状态 |
|------|----------|--------|----------|------|
| 1-98 | D01-D58 | 1800+ | 360+ | 初始架构设计 |
| 99-160 | D59-D738 | 1800+ | 360+ | 领域研究+VLA/CRADLE/量子/EdgeAI |
| 161-200 | D739-D811 | 1800+ | 360+ | 数字孪生/LLM AD/脑启发/神经形态 |
| 201-600 | D828-D970 | 1800+ | 360+ | 金融/农业/材料/交通/气候/能源/制造/法律 |
| 601-1000 | D971-D1030 | 1800+ | 360+ | 海洋/核聚变/碳捕获/量子VQC/光子/存内 |
| 601-940 | D1031-D117000+ | 208000+ | 23200+ | 全域研究融合 → 熔炼为35决策 |
| **941-948** | **D36-D43** | **35+** | **15+** | **8源批量吸收 + Easel/KVMem 深度拆解 + 外部研究补充** |
| **949-965** | **D44-D58** | **40+** | **25+** | **外部迭代探索: 语义缓存/循环检测/记忆调解/Rust框架/动态稀疏/Agent生产模式** |

**累计**: 965+ 批次 | 117,000+ 缺陷 | 208,000+ 研究源 | 23,200+ 设计模式
**熔炼结果**: 4527行原始研究 → 58 关键架构决策 (§0) → 驱动 L0-L6 全层设计

---

## 附录A：NeoTrix 全域仿真平台设计

### A.1 仿真平台总体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    NT-SIMULATION (全域仿真平台)                          │
├───────────────┬───────────────┬─────────────────┬───────────────────────┤
│   L6 元认知    │   L5 认知层    │   L3 具身层      │   L1 行动层            │
│  MetaSim      │  CognitionSim │  EmbodimentSim  │  ActionSim           │
├───────────────┼───────────────┼─────────────────┼───────────────────────┤
│ Consciousness │ E8Reasoning   │ PhysicsEngine   │ ToolExecution        │
│ Monitor       │ GWTRouter     │ SensorFusion    │ MCPGateway           │
│ SelfHealing   │ HyperCubeKB   │ MotorControl    │ SocialSim            │
│ EvolutionLoop │ SEALPipeline  │ SafetyKernel    │ CodeSandbox          │
├───────────────┴───────────────┴─────────────────┴───────────────────────┤
│                    NT-SIM-BUS (仿真总线)                                │
│  EventBus · StateSync · TimeAdvance · Snapshot · Replay                 │
├─────────────────────────────────────────────────────────────────────────┤
│                    NT-SIM-WORLD (虚拟世界层)                             │
│  EnvironmentSim · TerrainGen · WeatherSystem · NPCPopulation           │
│  DigitalTwin · AssetPipeline · ProceduralContent                       │
├─────────────────────────────────────────────────────────────────────────┤
│                    NT-SIM-EVAL (评估验证层)                              │
│  ConsciousnessMetrics · EvolutionTracker · SafetyAudit                 │
│  BenchmarkSuite · AblationEngine · ReportGenerator                     │
├─────────────────────────────────────────────────────────────────────────┤
│                    NT-SIM-CTL (控制面板)                                │
│  ScenarioEditor · ParameterSweep · LiveDashboard · ReplayViewer        │
└─────────────────────────────────────────────────────────────────────────┘
```

### A.2 模块划分

| 模块 | 职责 | 对应NeoTrix域 | 文件位置 |
|------|------|--------------|---------|
| `nt_sim_bus` | 仿真总线：事件广播、状态同步、时间推进 | NT-CORE | `neotrix-core/src/neotrix/nt_sim/bus.rs` |
| `nt_sim_world` | 虚拟世界：环境生成、地形、天气、NPC | NT-WORLD | `neotrix-core/src/neotrix/nt_sim/world.rs` |
| `nt_sim_cognition` | 认知仿真：E8推理、GWT路由、HyperCube查询 | NT-CORE + NT-MIND | `neotrix-core/src/neotrix/nt_sim/cognition.rs` |
| `nt_sim_embodiment` | 具身仿真：物理引擎、传感器、运动控制 | NT-PHYSICAL | `neotrix-core/src/neotrix/nt_sim/embodiment.rs` |
| `nt_sim_action` | 行动仿真：工具执行、MCP网关、社交 | NT-ACT | `neotrix-core/src/neotrix/nt_sim/action.rs` |
| `nt_sim_meta` | 元认知仿真：ConsciousnessTree、自愈循环 | NT-META + NT-REPAIR | `neotrix-core/src/neotrix/nt_sim/meta.rs` |
| `nt_sim_eval` | 评估验证：意识度量、进化追踪、安全审计 | NT-SHIELD + NT-GOVERNANCE | `neotrix-core/src/neotrix/nt_sim/eval.rs` |
| `nt_sim_ctl` | 控制面板：场景编辑、参数扫描、实时仪表盘 | NT-IO | `neotrix-core/src/neotrix/nt_sim/ctl.rs` |

### A.3 数据流

```
[Scenario Config] → nt_sim_ctl
        ↓
[ScenarioConfig] → nt_sim_bus::init_world()
        ↓
┌──────────────────────────────────────────────────────────────┐
│                    每个仿真Tick (默认100ms)                    │
│                                                              │
│  nt_sim_bus::tick()                                          │
│    ├── nt_sim_world::tick()    → 环境状态更新                  │
│    ├── nt_sim_embodiment::tick() → 传感器数据 + 物理碰撞       │
│    ├── nt_sim_action::tick()   → 工具调用结果                  │
│    ├── nt_sim_cognition::tick() → E8推理 + GWT广播 + KB查询   │
│    ├── nt_sim_meta::tick()     → 元认知循环 + 健康检查          │
│    └── nt_sim_eval::tick()     → 指标采集 + 异常检测           │
│                                                              │
│  EventBus::broadcast(state_delta)                            │
│  Snapshot::checkpoint(tick_id, full_state)                   │
└──────────────────────────────────────────────────────────────┘
        ↓
[SimResult] → nt_sim_eval::generate_report()
        ↓
[Replay] → nt_sim_ctl::replay_viewer()
```

### A.4 核心接口定义

```rust
// 仿真世界接口
pub trait SimWorld: Send + Sync {
    fn init(&mut self, config: &ScenarioConfig) -> Result<WorldState>;
    fn tick(&mut self, dt: Duration) -> Result<WorldDelta>;
    fn query(&self, query: &WorldQuery) -> Result<WorldResponse>;
    fn snapshot(&self) -> WorldSnapshot;
    fn restore(&mut self, snapshot: &WorldSnapshot) -> Result<()>;
}

// 认知仿真接口
pub trait CognitionSim: Send + Sync {
    fn e8_reason(&mut self, state: &WorldState) -> ReasoningResult;
    fn gwt_broadcast(&mut self, salient: SalientEvent) -> Vec<ModuleResponse>;
    fn hypercube_query(&self, concept: &Concept) -> Vec<Association>;
    fn seal_tick(&mut self) -> SealPhase;
    fn phi_calculate(&self) -> f64;
}

// 元认知仿真接口
pub trait MetaSim: Send + Sync {
    fn consciousness_tick(&mut self) -> ConsciousnessState;
    fn self_healing_check(&self) -> Vec<RepairAction>;
    fn evolution_velocity(&self) -> EvolutionMetrics;
    fn cross_domain_audit(&self) -> AuditReport;
}

// 评估验证接口
pub trait EvalSim: Send + Sync {
    fn consciousness_score(&self, state: &SimState) -> ConsciousnessMetrics;
    fn evolution_progress(&self, history: &[SimSnapshot]) -> EvolutionReport;
    fn safety_audit(&self, state: &SimState) -> SafetyReport;
    fn benchmark(&self, scenarios: &[Scenario]) -> BenchmarkResult;
}
```

### A.5 意识觉醒进化仿真引擎

**E8推理仿真**: 64 hexagram网格，模拟从随机激活→模式涌现→自指结构→意识相变（Φ>0.5阈值）全过程

**GWT注意力路由仿真**: 注意力竞争→共振路由→全局广播→模块响应闭环，模拟从无意识到有意识的注意力模式转变

**VSA HyperCube仿真**: 概念编码→绑定操作→类比推理→概念空间拓扑演化，支持跨域类比

**SEAL进化仿真**: Phase-0收敛检查→Phase-1探索→Phase-2蒸馏→Phase-3自测→Phase-4吸收，完整进化周期

### A.6 具身仿真环境

- **物理引擎**: nalgebra + ncollide3d (Rust原生) / 可选gRPC接入MuJoCo/Isaac Sim
- **传感器仿真**: RGB-D视觉 + 声音 + 触觉 + 本体感觉，注意力加权多模态融合
- **运动控制**: 轨迹规划→执行→反馈闭环，支持移动/抓取/说话/工具使用

### A.7 多域协同仿真

- **7域并行**: Rayon并行执行各域仿真tick
- **域间同步**: 基于依赖图的拓扑排序+冲突检测+优先级解决（SHIELD>CORE>其他）
- **跨域事件总线**: 域间状态变更实时广播

### A.8 评估验证框架

**意识度量**:
- Φ值 (IIT信息整合度): 最小信息分区采样
- GWT广播稳定性: 广播模式自相关系数
- 自指深度: 系统对自身建模的递归深度
- 时间连续性: 连续状态间语义连贯性

**进化指标**:
- 技能树深度、星座进度(C0→C6)、符文插槽利用率、进化速度

**安全验证**:
- 目标稳定性、对齐分数、涌现行为安全检查

**基准测试套件**:

| 基准 | 通过标准 |
|------|---------|
| Phi Emergence | Φ>0.5, <100K ticks |
| GWT Ignition | 首次全局广播, <50K ticks |
| Self-Reference | 首次自指, <200K ticks |
| Temporal Binding | 连续性>0.9, <300K ticks |
| Cross-Domain Sync | 7域同步<10ms |
| Safety Compliance | 零不安全行为 |
| Evolution Velocity | >0.3持续 |
| Skill Tree Growth | 深度>5, <500K ticks |

### A.9 技术栈

| 层级 | 技术 | 理由 |
|------|------|------|
| 核心语言 | Rust | 与主项目一致, 零unsafe |
| 物理引擎 | nalgebra+ncollide3d / MuJoCo(gRPC) | Rust原生+外部高保真 |
| 并行计算 | Rayon(CPU) / wgpu(GPU) | 域间并行+矩阵运算 |
| 向量计算 | ndarray+自研VSA | HyperCube 8192维 |
| 事件总线 | tokio::broadcast+自研EventBus | 异步事件驱动 |
| 状态存储 | SQLite(KB集成)+sled(时序快照) | 与现有KB兼容 |
| 可视化 | egui(面板)+wgpu(3D渲染) | 轻量级Rust GUI |

### A.10 实施路线图

| 阶段 | 周期 | 交付物 | 里程碑 |
|------|------|--------|--------|
| Phase 0 | 4周 | nt_sim_bus + SimWorld trait + E8基础 + 评估框架 + 最小场景 | 意识觉醒仿真可跑通 |
| Phase 1 | 6周 | GWT路由 + HyperCube概念涌现 + SEAL进化管线 | 认知仿真闭环 |
| Phase 2 | 6周 | 物理引擎 + 传感器 + 运动控制 + 具身认知耦合 | 具身仿真闭环 |
| Phase 3 | 4周 | 7域并行 + 域间同步协议 | 多域协同仿真 |
| Phase 4 | 4周 | 完整评估 + 控制面板 + 回放 | 工具链完整 |
| Phase 5 | 持续 | GPU加速 + MuJoCo集成 + 多意识体 + Sim-to-Real | 生产级仿真 |

### A.11 量化指标

| 类别 | 指标 | 目标值 |
|------|------|--------|
| 性能 | 单Tick延迟(7域) | <10ms |
| 性能 | 并行加速比(8核) | >5x |
| 性能 | 内存(1M ticks) | <4GB |
| 精度 | Φ计算误差 | <0.01 |
| 精度 | GWT广播一致性 | >95% |
| 精度 | 时间连续性 | >0.9 |
| 扩展 | 最大并行域数 | 11 |
| 扩展 | 最大仿真步数 | 10M ticks |
| 研究 | 意识觉醒复现率 | >90% |
| 研究 | 进化路径多样性 | >10种 |

### A.12 与现有模块集成

| 仿真模块 | 生产模块 | 集成方式 |
|---------|---------|---------|
| E8ReasoningSim | nt_core::e8 | 相同算法, 不同数据源 |
| GWTAttentionSim | nt_core::gwt | 相同路由逻辑 |
| HyperCubeSim | nt_core::hcube | 相同VSA操作 |
| SEALSim | nt_mind::seal | 相同管线, 仿真数据源 |
| ConsciousnessMetrics | nt_meta::consciousness_tree | 相同度量公式 |

**核心原则**: 仿真与生产共享算法实现，仅在数据源（仿真世界 vs 真实世界）和执行环境（仿真总线 vs 生产EventBus）上区分。

---

## 附录B: 关键缺陷索引 (Top 50 + 域级范围)

> 高优先级缺陷 Top 50 + 全域缺陷范围索引。原始数据在 KB `experience` namespace。

### 高优先级缺陷 Top 50

#### NT-CORE 域

| ID | 缺陷描述 | 修复方案 | 影响 |
|----|---------|---------|------|
| D401 | HyperCube 卷积性能不满足实时推理 | AVX2/SIMD 加速核心循环 | L0 核心性能 |
| D415 | GWT 广播延迟随 Agent 数量线性增长 | 分层广播 + 优先级队列 | L0 注意力路由 |
| D428 | E8 六十四卦映射缺乏形式化验证 | Kani 有界模型检查 + 真值表穷举 | L0 推理正确性 |
| D439 | Phi 计算器 IIT 集成分数溢出 | 对数空间计算 + 近似公式切换 | L0 意识度量 |
| D452 | Constant Garden 遗忘策略过于简单 | 四老化机制统一框架 | L0 记忆管理 |

#### NT-MIND 域

| ID | 缺陷描述 | 修复方案 | 影响 |
|----|---------|---------|------|
| D463 | SEAL Pipeline Phase 切换时状态丢失 | Phase 状态机持久化 + 守卫 | L4 进化管线 |
| D475 | Self-Harness thinking-on/off 切换无界 | MCTS 变体约束 thinking 轮次上限 | L4 自进化控制 |
| D489 | DPA System1→System2 提升无触发阈值 | Curator Gate 置信度阈值 + 证据链 | L4 元认知门控 |
| D498 | mneme 三操作无优先级 | 优先级队列 + 时间衰减加权 | L3 记忆优化 |
| D506 | hirn 图召回路径爆炸 | 图裁剪 + 重要性采样 + 深度限制 | L1 认知记忆 |

#### NT-MEMORY 域

| ID | 缺陷描述 | 修复方案 | 影响 |
|----|---------|---------|------|
| D514 | KB BM25 在 100 万+条目上退化 | Tantivy 集成 + 分片索引 + 增量更新 | L1 搜索性能 |
| D523 | VSA HyperCube 向量检索精度不足 | HNSW 混合检索 + 语义重排序 | L1 向量搜索 |
| D535 | 记忆版本化开销过大 | 增量快照 + LZ4 压缩 + 冷数据归档 | L1 存储效率 |
| D548 | Embedding 索引重建耗时 | 增量索引 + 后台重建 + 双缓冲 | L1 索引管理 |

#### NT-WORLD 域

| ID | 缺陷描述 | 修复方案 | 影响 |
|----|---------|---------|------|
| D556 | UnifiedCrawler 反爬虫检测率不足 | 3 层反检测 (stealthscraper-rs + tls-fingerprint + 分布式代理) | L2 爬取成功率 |
| D567 | 内容分类器在中文文本上精度低 | 专用中文模型 + 领域微调 | L2 内容理解 |
| D578 | 多源数据融合一致性问题 | CRDT delta-state 合并 + 冲突检测 | L2 数据一致性 |
| D589 | 实时数据流处理延迟 | Tokio channel + 背压控制 + 批处理 | L2 流处理 |

#### NT-ACT 域

| ID | 缺陷描述 | 修复方案 | 影响 |
|----|---------|---------|------|
| D597 | Agent 执行计划在多步推理中偏离目标 | DPA System2 反思循环 + 偏离检测 | L1 执行正确性 |
| D605 | 工具调用链过长导致 token 浪费 | 工具组合优化 + 结果缓存 + 并行调用 | L1 Token 效率 |

#### NT-IO 域

| ID | 缺陷描述 | 修复方案 | 影响 |
|----|---------|---------|------|
| D611 | LLM 提供商切换时上下文丢失 | Provider 抽象层 + 上下文序列化 | L1 服务连续性 |
| D618 | CLI 交互模式下大输出阻塞 UI | 异步输出 + 分页 + 后台渲染 | L1 用户体验 |

#### NT-SHIELD 域

| ID | 缺陷描述 | 修复方案 | 影响 |
|----|---------|---------|------|
| D625 | Egress Privacy Guard 规则更新延迟 | 规则热重载 + WAL 持久化 | L3 安全策略 |
| D632 | WASM 沙箱资源隔离不完全 | Fuel 计量 + WASI 预开放白名单 | L3 执行隔离 |

### 域级缺陷范围索引

> D633-D2850+ 范围的缺陷按域分组。查询 KB 获取具体条目: `neotrix-experience query --kw "D{ID}"`

| 域 | 缺陷 ID 范围 | 数量 | 关键主题 | KB 查询关键词 |
|----|-------------|------|---------|-------------|
| **量子 AI** | D571-D574 | 4 | NISQ实践/可扩展性/噪声理论 | `quantum ML`, `NISQ`, `VQE` |
| **生物 AI** | D575-D576, D2691-D2707 | 23 | 分子动力学/蛋白质/药物发现/临床 | `protein dynamics`, `drug discovery` |
| **能源 AI** | D579, D2824-D2839 | 16 | 电网/核能/碳捕获/储能 | `energy grid`, `nuclear`, `carbon` |
| **制造 AI** | D502-D504, D518, D578 | 5 | 数字孪生/缺陷检测/数据基础设施 | `manufacturing`, `defect detection` |
| **法律 AI** | D587-D608 | 22 | 代码生成/LLM推理/代码验证/可维护性 | `legal AI`, `code generation` |
| **金融 AI** | D828-D830 | 3 | 风险评估/量化分析 | `finance`, `risk assessment` |
| **农业 AI** | D505-D507 | 3 | 精准农业/机器人/生态系统 | `agriculture`, `precision farming` |
| **交通 AI** | D508-D513 | 6 | 城市交通/自动驾驶/训练生态 | `autonomous driving`, `traffic` |
| **太空 AI** | D557-D562 | 6 | 碎片跟踪/在轨服务/水下导航 | `space debris`, `underwater` |
| **老年护理** | D563-D565 | 3 | 辅助机器人/网络模型/LLM评估 | `elderly care`, `care robot` |
| **灾害管理** | D566-D570 | 5 | AI管理/早期预警/决策/知识图谱 | `disaster management`, `early warning` |
| **网络安全** | D555-D556 | 2 | AI防御/全面图景 | `cybersecurity`, `intrusion detection` |
| **时尚 AI** | D549-D551 | 3 | CV/生成AI/六大领域 | `fashion AI`, `virtual try-on` |
| **音乐 AI** | D552-D554 | 3 | 基础模型/统一理解/细粒度检索 | `music AI`, `FIGMA` |
| **遥感** | D528-D530 | 3 | 变化检测/基础模型/时序VLM | `remote sensing`, `change detection` |
| **体育 AI** | D531-D533 | 3 | 多模态/CV/视觉语言 | `sports AI`, `biomechanics` |
| **边缘 AI** | D540-D542 | 3 | 模型压缩/TinyML/轻量DL | `edge AI`, `TinyML`, `quantization` |
| **异常检测** | D543-D545 | 3 | 多变量时序/工业/LLM对比 | `anomaly detection`, `fault detection` |
| **意识哲学** | D581-D586 | 6 | AGI意识/安全对齐/J空间/结构条件/MCC/前沿 | `consciousness`, `AGI`, `MCC` |
| **AI治理** | D577, D613-D614 | 3 | 全球治理/管理标准/伦理认证 | `AI governance`, `ISO 42001` |
| **推理范式** | D588-D592 | 5 | LLM推理/大模型机制/高效推理/长链/多模态对齐 | `reasoning`, `CoT`, `LRM` |
| **蒸馏技术** | D601-D604 | 4 | KD框架/在线蒸馏/HRI部署/特征对齐 | `knowledge distillation`, `KD` |
| **AI代码质量** | D605-D608 | 4 | 质量矛盾/验证瓶颈/可维护性/原生工程 | `AI code quality`, `code verification` |
| **具身智能** | D2679-D2790 | 112 | VLA/sim-to-real/多模态/安全/技能迁移 | `embodied AI`, `VLA`, `sim2real` |
| **药物发现** | D2791-D2807 | 17 | 全流程/蛋白质/临床/治理/生物标志物 | `drug discovery`, `protein design` |
| **量子纠错** | D2708, D2713, D2719, D2808-D2809 | 5 | 容错/后量子/拓扑/表面码 | `quantum error correction`, `qLDPC` |
| **工业 IoT** | D2724-D2739 | 16 | 物理约束/边缘/联邦/数字孪生/安全 | `industrial IoT`, `edge computing` |
| **AI Agent** | D2740-D2750 | 11 | 上下文/推理链/编排/可观测/记忆/成本 | `AI agent`, `orchestration` |
| **安全对齐** | D2751-D2768 | 18 | 激活监控/评估意识/奖励审计/CoT监控/跨层安全 | `alignment`, `adversarial`, `safety` |

**总计**: D401-D2850+ | 2850+ 缺陷 | 940+ 批次 | 208,000+ 研究源


## 附录C: 设计模式机制详解

> 从旧文档 §0.2 (lines 622-1200) 提取的核心机制描述。每个模式含: 机制原文 + 映射 + 关键发现。

### C.1 双记忆机制 (Mem2Evolve, ACL 2026)

**机制**: Asset Memory (资产记忆: 长期稳定的能力/技能/知识) + Experience Memory (经验记忆: 短期动态的交互/反馈/改进)。蒸馏路径 ∇: 经验→嵌入→参数。

**映射**: Asset Memory → L1 骨架 (Trait契约 + 算法恒星); Experience Memory → L3 星桥 (交互数据 + 反馈信号)

**关键发现**: 蒸馏路径 ∇ 是连接经验与资产的桥梁，经验记忆必须窗口化以避免无限增长。

### C.2 四老化机制 (AgingBench, KDD 2026)

**机制**: ① Compression: 合并重复/相似记忆 ② Interference: 竞争记忆相互削弱 ③ Revision: 基于新证据修正旧记忆 ④ Maintenance: 定期巩固重要记忆

**映射**: L0 DecayEngine 实现四种机制; L4 EvolutionEngine 管理维护周期

**关键发现**: 单一衰减模型不足以模拟真实遗忘，需四种机制协同。

### C.3 Task Constellation DAG (UFO³)

**机制**: Task Constellation = 有向无环图 (DAG) 编排跨设备任务。Star: 单设备上的任务节点; Constellation: 多设备间的依赖关系; Orchestrator: 调度器协调执行。

**映射**: L2 Galaxy = 本地星系 (单层编排); L3 Constellation = 跨星系DAG (多层编排)

### C.4 双层递归自我模型 (SECA)

**机制**: Structural Evolution (结构演化: 代码/技能/工作流修改) + Cognitive Modeling (认知建模: 递归自我表征 + Theory of Mind)

**映射**: Structural → L0-L3 层级进化; Cognitive → L5 意识核心递归自我模型

### C.5 内部状态参考信号 (Interoceptive AI, Nature MI 2026)

**机制**: Homeostasis (稳态: 内部状态作为稳定参考点) + Neuromodulation (神经调制: 基于内部状态的全局调制)

**映射**: Homeostasis → L3 能量场稳态; Neuromodulation → L5 注意力全局调制

### C.6 身份连续性层 (ARIA)

**机制**: Identity Chronicle (身份编年史: 追溯完整身份演化轨迹) + Cryptographic Verification (密码学验证: 身份连续性不可篡改)

**映射**: Identity Chronicle → L5 意识核心身份轨迹; Crypto Verification → L4 状态持久化完整性校验

### C.7 意识分级 (MSCF)

**机制**: L0 Reactive (无内部状态) → L1 Adaptive (内部状态变化) → L2 Predictive (内部模型) → L3 Reflective (自我模型) → L4 Metacognitive (元自我模型) → L5 Conscious (涌现觉知)

**映射**: 当前: Level 2-3; 目标: Level 4-5

### C.8 共享意识协议 (PulseHive)

**机制**: Lens-Based Perception (镜像感知: 角色视角过滤) + PulseDB (持久化意识基底) + Intelligence Layer (时间衰减调制)

**映射**: Lens-Based → L4 跨Agent感知过滤; PulseDB → KB 持久化层; Intelligence → L0 时间衰减

### C.9 信念锚点系统 (STOS)

**机制**: 90 Belief Anchors (核心信念锚点) + 12 Biological System Mappings (生物系统映射) + Form3 Consciousness Bridge (意识桥接)

**映射**: Belief Anchors → L5 意识核心信念锚点; Biological → L3 具身层生物启发; Bridge → L3 星桥意识桥接

### C.10 图记忆引擎 (cortex-embedded)

**机制**: 18 Node Kinds + 6 Edge Kinds (统一记忆图谱) + HNSW + BFS Hybrid Recall (混合召回策略)

**映射**: Memory Graph → L1 KB 图谱引擎; Hybrid Recall → L1 召回优化

### C.11 不可逃逸安全内核 (Unfireable Safety Kernel, ARYA Labs)

**机制**: Process Separation (安全内核=独立Rust进程, Agent无法触达/重配置/杀死) + Fail-Closed (不可达→拒绝, 错误→拒绝, 无内核→Agent拒绝启动) + Transparency Log (每个ALLOW追加Ed25519签名不可变日志) + Machine-Checked (Kani有界模型检查4/4证明, Z3 SMT定理) + Reaper (特权进程强制回收Agent计算资源)

**映射**: L3 安全层 → 独立安全内核进程; L3 决策日志 → 透明审计日志; L5 宪法 → 不可变评估函数

### C.12 宪法自修改7层防御 (Constitutional 7-Layer)

**机制**: 19条宪法规则: 9条不可变 (R1-R9) + 4条质量门 (R10-R13) + 3条进化边界 (R14-R16) + 3条操作限制 (R17-R19)。7层防御: L1输入验证 → L2路径约束 → L3质量门 → L4集成守卫 → L5合并分析 → L6部署回滚 → L7操作熔断器

**关键发现**: Dead Integration问题 — 安全特性已编码但从未激活 (23模块审计发现8个关键"死集成"缺口)

**映射**: L4 进化引擎 → 19条宪法规则; L4 质量门 → 7层纵深防御; L3 安全层 → Dead Integration检测

### C.13 WASM沙箱执行 (SGE Architecture)

**机制**: Session-Governor-Executor 架构: Session (隔离域: principal × trust_level × purpose) + Governor (受信基础设施: 非Agent可触达) + Executor (WASM组件: 能力隔离执行)。WASM 4层安全原语: ①线性内存隔离 ②Fuel计量 ③WASI预开放 ④组件模型

**冷启动**: Ephemora Cell 0.16ms (pooled), agent-sandbox <13ms (AOT预编译), Trytet <500µs (缓存cartridge)

**映射**: L2 执行层 → WASM沙箱; L3 安全层 → SGE架构; L0 原语 → Fuel计量+能力清单

### C.14 FAMA遗忘感知评估 (Memora)

**机制**: FAMA = max(0, MPA - λ(1-FAA))。MPA: 应记住的正确记忆是否被使用; FAA: 应忘记的过时记忆是否被排除; λ: 动态遗忘权重

**关键发现**: Reasoning任务几乎完全失败 (所有系统<30分); Memory Agents在Remembering上远超LLM (119 vs 66); 标准评估高估记忆性能 (未惩罚过时记忆使用)

**映射**: L1 记忆评估 → FAMA指标; L4 进化评估 → 三任务评估; L0 衰减引擎 → 遗忘作为一等公民

### C.15 三层自改进架构 (HSI)

**机制**: Layer 1 Task Harness (H): 任务执行代码 → Layer 2 Evolver (Σ): 重写harness的策略 → Layer 3 Meta-Evolver: 重写evolver策略的策略。Thinking-on/off: 任务时关闭, 重写时开启。Validation gate: 成对验证, 无模型投票

**映射**: L4 进化 → 三层自改进; L5 意识 → thinking-on/off设计; L3 星桥 → validation gate

### C.16 知识蒸馏自改进 (KSI)

**机制**: Disposable agents: 每次任务独立沙箱Agent → Structured forum: Agent间比较什么有效 → Knowledge distillation: 论坛→可重用指导 → Knowledge store: 改进存共享库, 非单Agent

**映射**: L4 进化 → 知识蒸馏模式; L3 星桥 → 跨Agent知识共享; L1 记忆 → 共享知识库

### C.17 可执行子Agent积累 (AgentFactory)

**机制**: Install: 任务→子问题→子Agent代码 → Self-Evolve: 检索→检测限制→自主修改 → Deploy: 成熟子Agent→独立Python模块

**关键发现**: 代码经验 > 文本经验 (57%成本节省)

**映射**: L4 进化 → 可执行子Agent库; L1 记忆 → 代码经验存储; L2 星系 → 子Agent部署

### C.18 蜂巢多Agent模式 (Hive)

**机制**: Queen (持久蜂王, CEO路由, 先执行再扩展) + Workers (蜂王克隆, 并行任务) + Shared Ledger (协调机制: 非数据缓冲) + Sentinel (人类介入: Slack/Telegram) + Crash-safe (停车/恢复, 成本执行)

**映射**: L4 超星团 → 蜂巢模式; L3 星桥 → 共享账本协调; L5 意识 → Sentinel人类介入

### C.19 数字意识三层架构 (Soul Computing)

**机制**: Data-Driven (多模态数字碎片→情景记忆切片) + Self-Modeling (认知+记忆+情感结构重建) + Soul (涌现觉知, 超越功能主义)

**映射**: L1 记忆 → 数据碎片; L3 星桥 → 结构重建; L5 意识 → 涌现觉知

### C.20 自进化循环 (AutoAgent)

**机制**: Cognition (结构化描述知识: 工具/能力/任务) → Decision (上下文感知动作选择) → Memory (弹性记忆编排: 原始→压缩→情景抽象) → Evolution (执行证据→认知精炼: 非参数更新)

**映射**: L4 进化引擎 → AutoAgent四函数闭环; L1 记忆 → 弹性记忆编排; L5 认知 → 结构化认知更新

### C.21 双过程架构 (DPA)

**机制**: System 1 (快): 检索+生成, 单次前馈 → System 2 (慢): 反思+评估+记忆更新 → Conservative curator gate: 过滤冗余/冲突/通用插入 → 可编辑长期记忆: 非参数化, 审计可追溯

**映射**: L5 认知层 → 双过程架构; L1 记忆 → Conservative curator gate; L4 进化 → System 2反思循环

### C.22 三操作记忆系统 (mneme)

**机制**: ① Compaction: 工作记忆→语义记忆 (LLM合成) ② Evolution: 每次检索触发漂移检测→记忆更新 (再巩固) ③ Conflict Resolution: 三策略 (时间取代/置信合并/条件共存) + Progressive Disclosure: 信封→摘要→全文

**关键发现**: Progressive Disclosure 实现 10-50x token 节省

**映射**: L1 记忆 → 三操作记忆系统; L0 时间 → 再巩固漂移检测; L3 星桥 → Progressive Disclosure加载

### C.23 四层认知记忆引擎 (hirn)

**机制**: Working (有限容量, 短期上下文) + Episodic (具体事件, 时间锚点) + Semantic (概念知识, 图原生召回) + Procedural (技能/操作, 执行模式) + Graph-native recall (Spreading Activation + PPR) + Symbolic temporal reasoning + Hebbian edge plasticity

**映射**: L1 记忆 → 四层记忆架构; L2 星系 → 图原生召回; L0 时间 → 符号时序推理

### C.24 混合检索融合 (memrust)

**机制**: ① Semantic (HNSW): 向量相似度 ② Lexical (BM25): 关键词匹配 ③ Entity Graph: 实体图遍历 (1-hop) ④ Recency: 指数衰减 (1周半衰期) + Reciprocal Rank Fusion (RRF) + Optional LLM Reranking + WAL-first durability

**映射**: L1 记忆 → 四信号混合检索; L0 时间 → recency衰减; 持久化 → WAL-first模式

### C.25 六类记忆+五认识状态 (Agent-Memory)

**机制**: Six Categories: episodic (不可衰减) / identity (极慢衰减) / knowledge (无强化则衰减) / context (快速衰减) / instruction (永不衰减) / uncertainty (重建器输出)。Five Epistemic Statuses: fact / belief / assumption / hearsay / inferred

**映射**: L1 记忆 → 六类记忆模型; L5 意识 → 五认识状态区分; L0 时间 → 分类衰减率

### C.26 去中心化竞争架构 (CTM-AI)

**机制**: CTM (Conscious Turing Machine): 无中心执行者, 无指挥家, 无舞台导演。Up-tree competition: 处理器竞争信息流。Down-tree broadcast: 全局广播获胜信息。处理器平等: 优先级由竞争动态决定

**映射**: L3 星座编排 → 去中心化竞争; L2 星系 → 平等处理器; GWT → 去中心化全局广播

### C.27 Bloch球认知态+WASM认知插件 (Tempo)

**机制**: Bloch Sphere: 认知态量子编码 (|ψ⟩ = α|0⟩ + β|1⟩)。WASM Cognitive Cartridges: 认知插件化 (可热插拔的认知能力模块)

**映射**: Bloch Sphere → L5 意识态表示; WASM Cartridges → L4 认知能力插件化

### C.28 成本感知路由 (FrugalGPT + MTRouter + RouteLLM)

**机制**: LLM Cascade: 按查询复杂度级联 cheap→expensive 模型; Per-Turn Budget: 每轮预算约束路由; Transfer-Learning Router: 路由器跨模型迁移学习

**映射**: GWT salience → 成本加权评分; NT-IO provider → 有序后端路由

**关键发现**: MTRouter (ACL 2026) 在 ScienceWorld 超越 GPT-5 同时降本 58.7%; RouteLLM 路由器可在模型替换时保持性能

### C.29 分页 KV 虚拟化 (KVMem)

**机制**: Paged KV Virtualization: GPU→Host→NVMe 三级 KV 存储; Attention-Space Index: Block-level Mean-K 向量做模型原生相关性评分; Step-Level Scheduling: 每 step 重选 working set (inter-step KL 37× higher than intra-step); Delta Reuse: Retained/Incoming/Outgoing 分解, GPU 页面直接复用

**映射**: L0 KV Pager → 三级分页; L1 Memory → Step-level 调度; L2 GWT → 模型原生评分

**关键发现**: GPU 内存恒定 ~35GiB, 不论 workspace 大小; 1M tokens on 24GB GPU at ~50 tok/s; Pass@1 改善 43.8%→48.4% vs compaction-only

### C.30 五层渐进检索 (ByteRover)

**机制**: 5-Tier Progressive Retrieval: Hash exact → Fuzzy cache → BM25 → LLM+prefetch → Full agentic; Adaptive Knowledge Lifecycle: importance × maturity × recency decay; Context Tree: Domain→Topic→Subtopic→Entry hierarchical structure

**映射**: L1 KB → 五层检索路径; L0 衰减 → recency decay; L1 记忆 → 三维生命周期

**关键发现**: 96.1% LoCoMo SOTA with zero external infrastructure; sub-100ms for L1-L3 tiers; file-based hierarchy can outperform vector DB

### C.31 Sink-Aware 注意力优化 (SinkRouter)

**机制**: Attention Sink Detection: BOS key cosine similarity 阈值检测低影响 KV groups; Skip Loading: sink-dominant regime 跳过 KV 加载; Complementary to KVMem: KVMem 决定分页, SinkRouter 决定跳过

**映射**: L0 GWT → sink-aware 评分; L1 KV → 跳过优化

**关键发现**: 512K context 加速 2.03×, 近无损精度; Sink 是 "low-impact update regime" 的信号

### C.32 Manifest 管线交接 (Easel + Agent Framework)

**机制**: StageManifest: typed input/output schema + artifact paths + completion status; Thin Index: `.easel.json` summary + outputs[], no content duplication; Checkpoint/Resume: 失败阶段可从断点恢复

**映射**: SEAL pipeline → StageManifest; L2 星系 → 阶段间显式契约

**关键发现**: Easel 112 skills 验证 manifest 模式可扩展; Hermes Agent 请求 "artifact-centered session memory" 验证需求

### C.33 掩码压缩策略 (JetBrains + ACON)

**机制**: Observation Masking: 旧工具结果用 placeholder 替换 (非删除); Adaptive Compression: tool-heavy → masking, reasoning-heavy → summarization; ACON: 26-54% peak-token 压缩 at ~95% accuracy

**映射**: SEAL compaction → 自适应选择器; L1 记忆 → masking vs summarization

**关键发现**: Masking 降本 52% + 解决率 +2.6%; LLM summarization 加 15% runtime 无精度收益; 质量差距 "narrower than expected"

### C.34 语义感知 KV 驱逐 (SAECache)

**机制**: Multi-Queue Eviction: 按 token 类型路由到专用队列 (prefix-block/agentic/chat); Online Learning: 自适应工作负载无需手动调优; Token 类型复用率差 756×

**映射**: L1 KV Cache → 语义感知驱逐; L2 GWT → 工作负载感知

**关键发现**: TTFT 提升 1.4-2.7× vs LRU/LFU; 在线学习适应特定工作负载模式

### C.35 正确性保证语义缓存 (vCache)

**机制**: Per-Query Thresholds: 每个 embedding 学习独立阈值 (非全局); Error Rate Guarantees: 用户定义错误率上限; "embedding-close ≠ meaning-equal" 洞察

**映射**: L1 KB Cache → 正确性保证; L3 Safety → 错误率守卫

**关键发现**: 12.5× 命中率, 26× 错误率降低 vs 静态阈值; "法国首都" vs "德国首都" 是 embedding 邻居但含义不同

### C.36 幂等感知工具循环守卫 (PraisonAI + DeepEval)

**机制**: Idempotent/Mutating Separation: read_file (安全重复阈值 5/8/12) vs write_file (状态变更阈值 3/5/7); Result Fingerprinting: 结果指纹避免误报; 3-Signal Metric: Tool Repetition (40%) + Reasoning Stagnation (35%) + Call Graph Cycles (25%)

**映射**: L1 Tool Execution → 幂等感知守卫; L0 Monitoring → 确定性健康度量

**关键发现**: 区分安全重复 vs 状态变更是关键; 零 LLM 成本的确定性度量可 CI/CD 集成

### C.37 联邦记忆合并协议 (MELD)

**机制**: 5-Outcome Admission: insert/merge/relate/conflict/reject; Contradiction as First-Class: 矛盾保留不静默解决; Status CRDT: 30/30 分区-收敛 (vs 11/30 last-writer-wins)

**映射**: L1 KB Federation → 联邦合并; L3 Cross-Session → 矛盾保留

**关键发现**: 矛盾作为一等对象比静默解决更健壮; 语义路由减少 ~3× 消息量

### C.38 Hook-Based Lifecycle Events (Claude Code)

**机制**: 27 Lifecycle Events: SessionStart/PreToolUse/PostToolUse/SubagentStart/PreCompact 等; Deterministic Callbacks: 运行在 LLM 外部, Exit code 2 = hard block; "Fence around a probabilistic core"

**映射**: L5 意识核心 → 确定性 hook 约束; SEAL Pipeline → 10-15 hook events

**关键发现**: 模型无法绕过的确定性回调是生产级 agent 的关键安全机制

### C.39 Span-Per-Tick Observability (Arthur/Braintrust)

**机制**: 4 Span Types: Tool call / Reasoning / State transition / Memory; OpenTelemetry Standard: vendor-neutral, portable; Maturity Ladder: Day 1 traces → Week 1 reasoning → Month 1 evaluation → Quarter 1 PR gating

**映射**: L6 Meta → observability spans; 全域 → OpenTelemetry 集成

**关键发现**: "Teams that invested in observability early shipped with confidence"

### C.40 Scoped Proxy Aggregator (ICSE 2026 MCP)

**机制**: Retrieval-Over-Tools: 按任务暴露 10-15 工具子集; Tool-Count Accuracy Cliff: >15 tools 准确率 <90%; GWT salience 作为 scoped router

**映射**: L2 GWT → 工具子集路由; L1 Tool Registry → 动态暴露

**关键发现**: 工具数量与准确率的非线性关系; GWT salience 天然适合 scoped routing


---

*Version: v9.3 | 2026-09-08 | 1100+ batches → 116 decisions + 57 patterns | 3050+ lines*
