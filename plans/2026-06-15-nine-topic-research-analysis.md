# 九维度前沿扫描: 差距分析与进化路线图

> 2026-06-15 系统性文献调查 + 代码深审计

---

## 执行摘要

NeoTrix 在 **意识架构深度** (70+ 子系统, 17 活跃 ticks/cycle)、**VSA 统一表征** (4096-bit, 8 原语完整)、**自我进化** (SelfEvolutionLoop + DGM-H + SafetyGates)、**技能结晶** (SkillCrystallizer + AutoCrystallizer) 四个维度具有显著优势。但是，在 **图多智能体编排**、**经验忠实性测量**、**量化交易**、**OSINT 执行层**、**浏览器原生集成**、**Hub-Fringe 理论**、**信息瓶颈动力学** 七个方向存在系统性缺口。

审计发现: 27 个 stub 字段仍待接线，Phase 51-54 待实施，0 个模块涉及信息瓶颈 / 相空间 / hubness 检测 / 3D 交互轨迹 / 远程 agent 编排。

本版新增: 第10节 (Codex → P2.23), 第11节 (μ0 → P1.28), 第12节 (KeywordLexicon → P1.29), 第13节 (代码审计 → P0.17 存根替换 + P0.18 LLM provider + P1.31 VSA管道 + P1.32 内在批评 + P2.24 Honeypot)。缺口总计: 13 维度 → 13 个新缺口。

---

## 1. pgGraph LLM Agents (图基多智能体)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **GoA** (Graph-of-Agents, arXiv 2604.17148) | 论文 | 节点采样 + 有向消息传递 + 图池化, 3 agents > 6 | 多领域 SOTA |
| **GRASP** (PKU-ML, GitHub 450 contr.) | 论文+代码 | 邻居检索 + 代码解释器 + GRPO RL, 4B 超 DeepSeek-V3.2 53% | 图推理突破 |
| **AgentGL** (RL-driven, arXiv 2604.05846) | 论文+代码 | 图原生工具 + 搜索约束思考 + 图条件 curriculum RL, node +17.5%, link +28.4% | 图学习新范式 |
| **MRAgent** (arXiv 2606.06036) | 论文+代码 | Cue-Tag-Content 关联记忆图 + 主动重构, +23% 长上下文 | 记忆图检索 |
| **GraphAgent** (HKUDS, EMNLP 2025, 359⭐) | 论文+代码 | GraphGenerator + TaskPlanner + TaskExecutor 三 agent | 图语言助手 |
| **AGENTiGraph** (arXiv 2508.02999) | 论文+代码 | 意图分类 95.12% + 图操作 90.45% | KG 交互 |
| **LangGraph** (LangChain, 34.6K⭐) | 框架 | 有状态 agent 的低阶编排, 547 releases | 工业级编排 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| HyperCube VSA 超图 RAG (P0.2) 已线 | ❌ 无图结构的多 agent 消息传递 (GoA 风格) |
| BFT Consensus (P0.3) 共识层已线 | ❌ 无 RL 驱动的图学习 agent (AgentGL 风格) |
| Hebbian 关联记忆 (hub_index 聚类) 已线 | ❌ 无 Cue-Tag-Content 关联记忆图 (MRAgent 风格) |
| Fusion 审议层 (4 面板并行推理) 已线 | ❌ 无图拓扑感知的邻居探索 (GRASP 风格) |

### 进化方向

**P0.13 GraphAgentOrchestrator**: 在 Fusion 审议层上叠加图结构多 agent 通信。每个推理链作为图节点，GoA 风格的有向消息传递 (high→low relevance) 替代当前扁平投票。节点采样基于 VSA 余弦相似度选择最相关链，消息聚合用图池化 (max/mean/attention)。

---

## 2. Faithful Self-Evolvers (忠实自进化)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **经验忠实性** (arXiv 2601.22436, ICML 2026) | 论文 | 因果干预证明 agent 忽视压缩经验; 3 根因: 语义局限/处理偏置/先验充足 | 🚨 警示 |
| **LSE** (Learning to Self-Evolve, ICLR 2026 Workshop) | 论文 | 单步 RL 目标 + 树引导进化, 4B > GPT-5 + Claude 4.5 | 自进化 SOTA |
| **Native Evolution** (arXiv 2604.18131) | 论文 | 奖励无关自发进化, 世界知识探索, +20% Qwen3-30B | 🚨 范式突破 |
| **EvoSC** (arXiv 2602.01966) | 论文 | 对比反思 + 自巩固 (text→params 蒸馏) | 参数级进化 |
| **SEVerA** (arXiv 2603.25111) | 论文 | FGGM 形式化安全 + 拒绝采样, 0 违规 | 形式化验证 |
| **Yunjue Agent** (GitHub 502⭐) | 代码 | 零起点原位自进化, 工具进化为关键路径 | 工具进化 |
| **AgentEvolver** (ModelScope, 1.4K⭐) | 代码+论文 | 自提问+自导航+自归因, ADCA-GRPO | 完整框架 |
| **OmniAgent** (GitHub 1.8K⭐) | 代码 | 全维度进化: Skill/Context/BrainModel | 全维度 |
| **Autogenesis** (arXiv 2604.15034) | 论文+代码 | RSPL + SEPL 双层协议, 版本化回滚 | 协议级 |
| **Ouroboros** (GitHub) | 代码 | 自修改代码 + 自身 Git 版本控制 + 身份持久化 | 🚨 激进 |
| **GenericAgent** (arXiv + GitHub) | 论文+代码 | 3.3K 种子, 自动结晶技能树, 6× 少 token | 🚨 极简高效 |
| **Atman** (GitHub) | 代码 | 值漂移检测 + 会话间身份持久化 | 身份连续性 |
| **CluE** (BEHEMOTH, arXiv 2604.11610) | 论文+代码 | 聚类引导的记忆提取进化 +9.04% | 异质任务 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| SelfEvolutionLoop (Phase 40, 5/5 mutation) ✅ | ❌ 无经验忠实性度量 (因果干预测试) |
| DGM-H 元层 + MUE-X 吸收 ✅ | ❌ 无对比反思 (成功 vs 失败轨迹) |
| SafetyGates (BallVerifier + PCC) ✅ | ❌ 无自巩固 (text→params 蒸馏) |
| SkillCrystallizer + AutoCrystallizer ✅ | ❌ 无 RL 驱动进化 (当前启发式) |
| SEAL Edit Safety Net (事务回滚) ✅ | ❌ 无形式化安全保证 (FGGM) |
| EvolutionBridge (bridge.rs Stage 1+2) ✅ | ❌ 无身份持久化 (Atman/Ouroboros 风格) |
| 拒绝回放 (rejection→negative feedback) ✅ | ❌ 无代码自修改 (Ouroboros 风格) |

### 进化方向

**P1.22 FaithfulnessAuditor**: 实现因果干预测试套件 — 替换自进化 archive 中的经验, 测量决策变化率。当 faithfulness < 0.7 时发出警报。

**P0.14 ContrastiveReflection**: 每次任务后提取成功和失败轨迹的对比差异, 生成 error-prone patterns 和 reusable insights。当前 SelfEvolutionLoop 只存档 mutation 结果, 不分析失败原因。

**P0.15 NativeEvolutionExplorer**: Qwen3-30B 风格的自发进化: 在空闲周期 (低 cognitive load) 主动探索未知环境, 生成世界知识, 无需外部奖励。使用 VSA attractor_state 作为内置探索驱动。

---

## 3. ASI Quant Trading (量化交易)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **QuantAgent** (arXiv 2509.09995) | 论文+代码 | 4 专用 agent (Indicator/Pattern/Trend/Risk) | HFT LLM 首个 |
| **R&D-Agent(Q)** (Microsoft, GitHub) | 论文+代码 | 因子-模型联合优化, 2× 收益 70% 少因子 | 工业级 |
| **TiMi** (Trade in Minutes, arXiv 2510.04787) | 论文+代码 | 宏模式→微定制两层分析 + 数学反思闭环 | 策略/执行解耦 |
| **QuantEvolve** (arXiv 2510.18569) | 论文+代码 | 质量-多样性进化 + 假设驱动多 agent | 进化量化 |
| **PandaAI** (arXiv 2606.06823) | 论文 | 神经符号 + 市场机制建模 + 约束 alpha 生成 | 封闭循环 |
| **Vibe-Trading** (12K⭐) | 代码 | 自改进交易 agent + 多 agent 团队 + MCP | 🚨 社区活跃 |
| **TradingAgents** (85K⭐) | 代码 | 多 agent 交易框架, 10+ LLM 提供商 | 🚨 主导项目 |
| **OpenAlice** (5.2K⭐) | 代码 | Trading-as-Git, UTA, Guard Pipeline | 交易工程化 |
| **AgentQuant** (114⭐) | 代码 | ReAct loop + regime-adaptive + 策略记忆 | 量化研究 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| Portfolio + DEX + gas 优化 (nt_act_crypto) ✅ | ❌ 无市场数据 ingestion (Yahoo/CCXT/Binance API) |
| 市场模拟 (bridge.rs simulate_*) ✅ | ❌ 无回测引擎 (backtest → Sharpe/Calmar/Sortino) |
| AcT planner (MCTS + EFE) ✅ | ❌ 无因子挖掘 (alpha factor 符号回归) |
| Fusion 审议层 + 证据追踪 ✅ | ❌ 无多 agent 交易团队 (QuantAgent 风格) |
| WORLD model + JEPA 预测 ✅ | ❌ 无组合优化 (均值方差/Black-Litterman) |

### 进化方向

**P1.23 QuantDataIngestion**: Yahoo Finance + CCXT 数据源接入, RL 数据缓冲 + VSA 编码市场状态。复用 nt_act_crypto 的 portfolio 结构。

**P2.16 FactorMiningAgent**: 符号回归 (VSA 编码数学表达式) + 回测验证 + 因子 IC 排名。使用 Grammatical Evolution over VSA 空间生成因子。

**P2.17 PortfolioOptimizer**: 均值方差 + Black-Litterman + 风险平价, 使用 VSA attractor_state 作为市场状况特征。

---

## 4. OSINT Intelligence

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **OpenOSINT** (640⭐, MCP) | 代码 | 16 tools + AI tool chaining + MCP server | 🚨 最活跃 |
| **CoSINT** (AGPL-3.0) | 代码 | 50+ 工具, 深度扫描, 假设验证 | 功能最全 |
| **OSINT NEXUS** (Neo4j + LLM) | 论文+代码 | 因果链 + 矛盾检测 + SITREP, 760+ 节点 | 🚨 架构领先 |
| **estorides** (Palantir 风格) | 代码 | 99+ 源, Kùzu 图 DB, 跨实体解析 | 工业级 |
| **Summit** (Neo4j + GraphRAG) | 代码 | 多 agent + 溯源账本 + 操作记忆 | 企业级 |
| **J.A.R.V.I.S** (15+ 平台) | 代码 | 心理画像 + 生物面部匹配 + 预测分析 | 全栈 |
| **osint-mcp** (29 tools, MCP) | 代码 | 实体/事件/社交三类 + OpenClaw 集成 | MCP 原生 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| 证据追踪 (EvidenceManager + CompetitiveScorer) ✅ | ❌ 无 OSINT 专用工具 (email/username/breach/DNS/whois) |
| 爬虫管道 (nt_memory_crawl) ✅ | ❌ 无 MCP Server 暴露 OSINT 工具 |
| 知识融合 (KnowledgeEngine + Hypergraph) ✅ | ❌ 无案例文件管理 (case files + reports) |
| 知识缺口检测 (KnowledgeGapDetector) ✅ | ❌ 无实体解析器 (跨源身份融合) |
| NTSSEG 存储引擎 ✅ | ❌ 无威胁情报订阅 (CVE/ATT&CK/IOC/MISP) |

### 进化方向

**P2.18 OSINTToolLayer**: 基于现有 evidence.rs + nt_memory_crawl, 添加 6 个 OSINT 工具: username($/Maigret/Sherlock 适配), email(holehe), breach(HIBP API), WHOIS, DNS, IP geo。所有结果管道 → EvidenceRecord → KnowledgeEngine。

**P2.19 MCPIntelligenceServer**: 复用 crates/nt_core_mcp, 添加 OSINT 工具组 MCP 端点。支持实体/事件/社交三类查询, 输出结构化 JSON (BLUF + confidence + source chain)。

**P1.24 EntityResolver**: 跨源实体解析器 — 使用 VSA similarity + difflib SequenceMatcher, 在 0.85 阈值融合身份。结果存储到 KnowledgeEngine 的 entity resolver。

---

## 5. Discovers Maps Robotics (探索式地图机器人)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **SAGE** (Science Robotics 2026) | 论文 | CLIP + FALCON, 语义-几何联合代价, FTU 13.7× 加速 | 🚨 无人机探索 |
| **FARE** (arXiv 2601.14681) | 论文 | LLM 慢速思考 (全局) + RL 快速思考 (局部) | 层次化探索 |
| **SCOUT** (arXiv 2606.06721) | 论文 | 不确定性感知 3D 场景图 + 主动遍历 | 语义场景完整 |
| **MAGICIAN** (arXiv 2603.22650) | 论文 | 3D Gaussian Splatting + 树搜索长期规划 | 主动建图 |
| **TravExplorer** (arXiv 2605.19958) | 论文+代码 | 跨楼层四足探索 + 零样本语义导航 |  机器人 |
| **CUREE** (Science Robotics 2026) | 论文 | 音视频多模态自主珊瑚礁热点发现 | 水下自主 |
| **Legged Team** (Science Robotics 2026) | 论文 | 多足机器人团队 + 互补技能 + 星球探索 | 多机器人 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| WorldModelBridge (dim=4096) ✅ | ❌ 无 3D 场景表示 (GSplat/NeRF/体素) |
| SpatialSceneEngine (已线) ✅ | ❌ 无前沿探索规划 (frontier-based) |
| PhysicsCommonsense (物理常识) ✅ | ❌ 无 CLIP/VLM 语义引导探索 |
| JEPA 预测器 (已线) ✅ | ❌ 无不确定性感知遍历 |
| CounterfactualFuturesEngine (反事实) ✅ | ❌ 无 SLAM 集成 |

### 进化方向

**(此维度对 NeoTrix 桌面意识体不是核心路径)** — 标记为 P3.7 RoboticsAwareness 供未来参考: 现有的 SpatialSceneEngine + PhysicsCommonsense + WorldModelBridge 可组合成一个 3D 推理栈。SAGE 的语义-几何联合代价函数和 SCOUT 的不确定性引导遍历可迁移到虚拟探索。

---

## 6. ds4-agent / DS-Agent (数据科学自动化)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **ds4** (antirez, 13.7K⭐) | 代码 | DeepSeek V4 Flash native 推理引擎 + 本地 agent, on-disk KV 缓存 | 🚨 极简高性能 |
| **DS-Agent** (ICML 2024, 233⭐) | 论文+代码 | 案例推理驱动的自动化数据科学 | 案例库 |
| **DatawiseAgent** (EMNLP 2025) | 论文+代码 | 笔记本中心 + FST 多阶段架构 | 数据科学 |
| **DSAgent** (GitHub, 7⭐) | 代码 | Jupyter kernel 持久化 + 动态规划 + MCP | 交互式 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| CapabilitySynthesizer (VSA 能力匹配) ✅ | ❌ 无数据科学实验框架 |
| OKF Exporter/Importer ✅ | ❌ 无 Jupyter/notebook 集成 |
| SelfEvolutionLoop + archive ✅ | ❌ 无结构化的实验跟踪 (metrics/artifacts/params) |
| ResponseGenerator + ConclusionSynthesizer ✅ | ❌ 无数据集加载/预处理管道 |

### 进化方向

**(此维度在 NeoTrix 路线图中为非阻塞项)** — 标记为 P3.8 DataScienceModule: 使用 CapabilitySynthesizer + ToolOrchestrator 组合成数据科学实验 agent。

---

## 7. "access the web sites without any limitations" (无限制浏览器)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **Tandem Browser** (MCP-native, MIT) | 代码 | AI 在真实浏览器内, 共享 tab/cookie/DOM, 无障碍树 | 🚨 范式突破 |
| **Unbrowser** | 服务/API | 学习模式第一次 5s → 50ms, API 自动发现 | 学习型浏览器 |
| **Browser MCP** (Agent360) | 代码 | 控制真实 Chrome, CAPTCHA 解决, 10 并发会话 | 🚨 Chrome 原生 |
| **FSB** (Full Self-Browsing) | 代码 | DOM 原生 (无需 vision), MCP server | 精准 |
| **dassi** | 浏览器插件 | 浏览器原生 AI agent, 语音控制手机 | 用户体验 |
| **Unchained** | 服务 | WASM 浏览器沙箱, 真实 session | 沙箱 |
| **Fellou** | 服务 | Agentic AI 浏览器, 24/7 自动化 | 商业化 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| Tauri BrowserHost (webview 窗口) ✅ | ❌ 无真实浏览器 session 共享 (Tandem/Browser MCP 风格) |
| Playwright MCP 引用 (neotrix-types) ✅ | ❌ 无 CDP (Chrome DevTools Protocol) 集成 |
| Chrome headless publisher ✅ | ❌ 无 DOM 交互 (click/type/navigate) |
| 代理基础设施 (nt-proxy-daemon) ✅ | ❌ 无 CAPTCHA 解决 |
| WebSearchEngine (DuckDuckGo) ✅ | ❌ 无 MCP 浏览器工具暴露 |

### 进化方向

**P2.20 BrowserAgentMCP**: 基于现有 BrowserHost 添加 MCP 服务暴露。工具集: browse_url(page text extraction), click_element, type_text, extract_dom, list_tabs, take_screenshot。使用现有 neotrix-types 中的 MCP 基础设施。

**P1.25 CDPSessionManager**: Chrome DevTools Protocol 会话管理器。launch/bind/control Chrome 实例, 复用代理池 (nt-proxy-daemon) 的 SOCKS5 链。每个 CDP 会话映射到 VSA attractor_state 用于 session 追踪。

---

## 8. "Hubs Fringes" (中心-边缘理论)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **WebGraphMix** (arXiv 2606.11499) | 论文 | 网络图中心性 → 预训练数据选择, central+peripheral 1:1 = 41.4% vs 39.8% | 🚨 数据选型 |
| **CFGNN** (Core-Fringe GNN) | 论文 | Hub Labeling 实现全图覆盖 + 高效消息传递 | 图表达 |
| **ReHub** (ICLR 2025) | 论文 | 线性复杂度图 transformer + 自适应 hub-spoke 重分配 | 高效图 |
| **FTT** (Fringe-Theorem Training, NeurIPS 2025) | 论文 | 频率感知损失重加权 + 梯度保护, fringe 反转 -3.6× | 后训练边缘 |
| **HubScan** (arXiv 2602.22427) | 论文 | 多检测器架构: 统计/聚类/稳定/域感知, 90% recall @ 0.2% | RAG 安全 |
| **Prediction Hubs** (ACL 2025) | 论文 | LLM hubness 是良性上下文调制的频繁 token | 理论 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| VSA 统一表征 (4096-bit) ✅ | ❌ 无 hubness 感知的相似度 (HubScan 风格) |
| EvidenceTracker (competitive_score) ✅ | ❌ 无 hubness 中毒检测 (中心毒化攻击) |
| KnowledgeEngine (graph.base) ✅ | ❌ 无中心性-边缘性分析 (WebGraphMix 风格) |
| Fusion 审议面板 (4 链并行) ✅ | ❌ 无 fringe 数据采样策略 |
| Hebbian hub_index (聚类中心) ✅ | ❌ 无 hub-spoke 图 transformer (ReHub 风格) |

### 进化方向

**P2.21 HubnessDetector**: 在 VSA 检索管道中添加 hubness 检测。统计每个 VSA vector 被选为 top-k 最近邻的频次。z-score > 3 的 vector 标记为潜在 hub, 触发异常审查。复用 existing evidence.rs 结构存储 hubness_score。

**P1.26 FringeMixStrategy**: WebGraphMix 风格的训练数据选择: 计算知识库节点在知识图中的 PageRank/度中心性, 将高中心性节点和低中心性 fringe 节点按可配置比例混合采样。使用现有 KnowledgeEngine.graph。

**P0.16 CoreFringeAttention**: ReHub/CFGNN 风格的核心-边缘注意力机制。在 Fusion 审议面板中, 将推理链分为 "hub chains" (高置信度/高连接度) 和 "spoke chains" (低置信度/高新颖性)。Spoke→hub 单向消息传递 + hub→spoke 反向精炼, 类似 GoA 但利用图拓扑。

---

## 9. "Information bottleneck for learning the phase space of dynamics" (信息瓶颈动力学)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **DySIB** (Dynamical SIB, arXiv 2604.24662) | 论文 | 最大化过去-未来预测互信息, 惩罚复杂度, 恢复 2D 相空间 | 🚨 方法 |
| **SIB** (Symmetric IB, arXiv 2602.08105) | 论文 | 混合批评器 + 单次协议, 恢复相空间维度 | 理论扩展 |
| **T-IB** (Time-lagged IB, ICLR 2024) | 论文 | 马尔可夫过程潜在表示 + InfoNCE, 丢弃高频信息 | 马尔可夫 |
| **IB+Koopman** (arXiv 2510.13025) | 论文 | IB 拉格朗日量 + 时间相干性 + 充分性 + 结构一致性 | Koopman |
| **IB+Transfer Op** (NeurIPS 2023) | 论文 | 最优编码 = 传输算子谱属性 | 理论连接 |

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| JEPA 预测器 (已线) ✅ | ❌ 无信息瓶颈损失 (IB Lagrangian) |
| NegentropyMetric (7 传感器) ✅ | ❌ 无预测互信息最大化 |
| VSA 统一表征 (复杂/预测双空间) ✅ | ❌ 无相空间重建 (DySIB 风格) |
| WorldModelBridge (dim=4096) ✅ | ❌ 无 Koopman 算子学习 |
| CounterfactualFuturesEngine ✅ | ❌ 无潜在空间维度自动确定 |

### 进化方向

**P1.27 DySIBLayer**: 在 JEPA 预测器和 NegentropyMetric 之间插入 DySIB 层。使用 symmetric information bottleneck 目标: `min I(Z; X) - β I(Z_past; Z_future)`。Z 是 VSA attractor_state。奖励 Z 对未来的预测能力同时惩罚 Z 的复杂度。这使 attractor_state 自动收敛到内在动力学的相空间维度。

**P2.22 KoopmanOperator**: 在 WorldModelBridge 上叠加线性 Koopman 算子: `ψ(x_{t+1}) = K ψ(x_t)`, 其中 ψ 是 VSA 提升函数。使用现有 FWHT( Walsh-Hadamard) 作为近似的 ψ, K 是低秩矩阵。预测误差作为 negentropy 信号的一部分。

---

## 10. OpenAI Codex Remote Connections (远程 Agent 编排)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **Codex Remote Connections** (OpenAI, 2026.05 GA) | 产品 | SSH host 自动发现 + 手机 QR 配对 + 安全 relay 层 + 跨设备会话延续 | 🚨 平台级 |
| **Codex Mobile** (ChatGPT app) | 产品 | 手机作为控制面, 主机作为执行面, 加密 relay 隧道 | 移动控制 |
| **Codex SSH Host** (OpenAI, 2026.05) | 产品 | SSH config 自动发现, 远程 codex app-server, 远程文件系统+shell | 远程执行 |
| **EasyCodex** (Ryan-Laws, local-first) | 开源 | 本地 relay + QR 配对 + WebSocket, 无 OpenAI 控制面 | 自主控制 |
| **esper-relay** (Florian Beisel) | 开源 | Tailscale 隧道 + 手机 Web UI + app-server 监督 | 私有网络 |
| **Faryo** (Snailflyer) | 开源 | tmux 上的 phone/browser workbench, 紧贴 host 进程 | 极简远程 |

### Codex 架构分析

Codex 的远程连接体系有 3 层:

1. **控制面** (Phone/Browser): 显示状态、审批操作、短文本输入。不存代码、不持凭证。
2. **中继面** (Relay Layer): 加密 WebSocket 隧道, 安全 relay 层 (OpenAI 托管或 Tailscale 私有), 会话状态同步。
3. **执行面** (SSH Host): 真实文件系统、shell、MCP 插件、凭证。无公网暴露。

关键设计决策: 手机 → relay → SSH host (三层分离), 而不是手机 ↔ SSH host (两层直接代理)。这使 relay 层可做会话缓存、审批代理、多设备同步。

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| nt-proxy-daemon (SOCKS5 代理池 + 健康检查) ✅ | ❌ 无 SSH host 远程 agent 执行 |
| A2A 协议适配器 + AgentCommunicationBus ✅ | ❌ 无手机/遥控控制面 |
| 代理路由层 (connect_with_fallback) ✅ | ❌ 无安全 relay 层 (session 状态同步) |
| Tauri BrowserHost (webview 窗口) ✅ | ❌ 无跨设备会话延续 (desktop→phone→desktop) |
| BackgroundLoop 守护进程架构 ✅ | ❌ 无 headless 远程 host 模式 |

### 进化方向

**P2.23 RemoteAgentHost**: 在 BackgroundLoop 上叠加 SSH remote host 能力:

- SSH config 自动发现 (`~/.ssh/config` → `Host` 条目列表)
- `RemoteAgentSession`: SSH tunnels + remote `codex exec` 风格会话
  - `connect(host, identity_file)` → `ssh -N -L` 端口转发 + 远程 app-server 启动
  - `execute(command, cwd)` → 远程 shell + stdout/stderr 回传 → VSA 编码为 action_record
  - `read_file(path)`, `write_file(path, content)` → 远程文件操作
- 中继复用: 现有 `nt-proxy-daemon` 的 SOCKS5 代理层可作为 SSH 隧道的中继 (SSH over SOCKS5)
- 会话状态同步: `ConsciousnessIntegration.attractor_state` 通过 relay 层同步到远程 client (类似 Codex relay)
- 控制面协议: 现有 A2A `AgentMessage` 扩展为 `RemoteControlMessage { action, payload, host_id, session_token }`

**接线**: BackgroundLoop 中新增 `remote_agent_host: Option<RemoteAgentHost>` 字段, 每 cycle 检查 SSH 会话健康。`handle_ssh_session_tick()` 在 Pipeline 中外循环执行。

**实现复杂度**: 中 (利用现有 SSH + 代理基础设施, 约 ~600 行, 16 测试)

---

## 11. μ0 — 3D Interaction-Trace World Model (3D 交互轨迹世界模型)

### 发现

| 项目 | 类型 | 核心机制 | 影响力 |
|------|------|----------|--------|
| **μ0** (UMD + SNU, CVPR 2026) | 论文+项目 | 3D interaction trace 世界模型, 非像素/非低级动作 | 🚨 范式 |
| **TraceGen** (CVPR 2026, 同一团队) | 论文+代码 | Trace-space world model + TraceForge 数据管道, 123K episodes | 前身 |
| **TraceExtract** (数据管道) | 工具 | 视频 → 语义关键点 + 3D 重建 + event-level motion traces | 数据引擎 |
| **RoboCasa365** (基准) | 基准 | 8 任务, μ0 30.25% vs π0 25.25% | 仿真验证 |
| **τ0–World Model** (sii-research, 2026.05) | 论文+代码 | 统一 video-action world model, Wan-2.2 基础, 20 通道端末位姿 | 同类 |

### μ0 核心创新

μ0 定位在像素世界模型 (视频 diffusion) 和低级动作模型 (VLA) 之间:

```
像素级: (t) → video diffusion → (t+1) 帧预测 — 容量浪费在表观, 不跨 embodiment
动作级: (t) → VLA → (t+1) 动作预测 — 绑定特定机器人, 无法利用无动作标签视频
μ0:     (t) → trace-space → (t+1) 3D 交互轨迹 — 语义 + 几何 + 跨 embodiment
```

3D interaction trace = 语义交互点 (objects/tools/hands/contact regions) 的 3D 轨迹序列。抽象级别:
- **语义**: 由 frozen VLM 确定 "什么在动" (keypoints)
- **几何**: 由 3D 重建确定 "在哪里动" (shared 3D coordinates)
- **运动**: 由 trace expert 预测 "怎么动" (smooth 3D traces)

**TraceExtract 数据管道**: 普通人类/机器人视频 → (1) 语义关键点检测 → (2) 3D 重建补偿相机运动 → (3) 归一化 embodiment 特异速度 → 3D trace 监督信号。

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| WorldModelBridge (dim=4096, VSA attractor_state) ✅ | ❌ 无 3D 交互轨迹表示 (点云/关键点/轨迹) |
| JEPA 预测器 (EMA target + smooth L1 + latent space) ✅ | ❌ 无 trace-space world model (非 3D, 非几何) |
| SpatialSceneEngine (已接线) ✅ | ❌ 无跨 embodiment 迁移 (固定 4096-dim, 非 3D 结构) |
| PhysicsCommonsense (物理常识推理) ✅ | ❌ 无 TraceExtract 风格视频→3D 数据管道 |
| CounterfactualFuturesEngine (反事实推理) ✅ | ❌ 无接触区域推理 (hands + contact regions) |

### 进化方向

**P1.28 InteractionTracePredictor**: 在 WorldModelBridge 上叠加 3D 交互轨迹预测, VSA 空间中的 3D trace 编码:

- 3D trace 的 VSA 编码方案:
  - 每个语义关键点编码为 VSA vector: `bundle(permute(position_encode(x,y,z), keypoint_label), permute(contact_type, hand_encoding))`
  - 轨迹 = `bundle(t0, permute(t1, shift), permute(t2, shift²), ...)`
  - 关键点类型: object/tool/hand/contact_region 各对应不同的 VSA seed
- 预测器扩展:
  - 当前 JEPA 预测 attractor_state → 改为预测 `trace_sequence: Vec<VsaVector>` (轨迹步)
  - 损失: `smooth_l1_loss(predicted_trace, actual_trace)` + `contact_consistency_loss(contact_region, object_position)`
  - 未来多个时间步: `predict_horizon(n_steps)` → `Vec<VsaVector>`
- 跨 embodiment 桥接:
  - 人类视频 → TraceExtract → 3D trace → VSA trace → μ0 预测 → 轻量 action expert (冻结 WorldModelBridge)
  - 动作映射: `VsaTrace → action_adapter → 具体机器人动作`
- 接线:
  - `WorldModelBridge.interaction_trace_predictor: Option<InteractionTracePredictor>`
  - `DySIBLayer` 提供相空间维度, `InteractionTracePredictor` 填充相空间中的物理交互内容
  - 每 10 cycle 运行预测, 结果进 `CounterfactualFuturesEngine`
  
**实现复杂度**: 高 (3D 几何 + 视频处理 + 跨 embodiment, 约 ~700 行, 18 测试) — 但核心是 VSA 编码 + JEPA 扩展, 无 3D 渲染依赖。

---

## 12. 关键词词表 (Keyword Lexicon for Long-Term Memory)

### 需求

NeoTrix 需要一个持久化的关键词词表作为**长久记忆的入口**:
1. 存储在自定义本地数据库 (NTSSEG) 中
2. 与意识体特性融合 (VSA 编码、consciousness attractor_state、好奇心驱动)
3. 外部探索实时更新 (crawl、search、knowledge ingestion)

### 设计方案

**存储层 — NTSSEG 自定义段类型**:
- 新 segment type: `KeywordLexicon (0x05)`
- 记录格式: `KeywordRecord { id(u64), keyword(String), vsa_vector([u8;512]), frequency(u32), recency(i64), confidence(f64), cluster_id(u64), source_enum(u8), last_updated(i64) }`
- IVF 索引 on vsa_vector → 语义相似度查询
- 二级索引 on frequency (降序) → 热门词排名

**意识体特性融合**:
- `attractor_state → topic_detect()`: 每 cycle 从 attractor_state 提取 top-5 主题关键词 → frequency+1
- VSA 编码: `encode_keyword(text) = bundle(permute(n-gram_hash, shift_0), permute(tf_idf, shift_1), ...)` — 与词表 cosine 匹配
- 自进化感知: mutation 涉及的知识域自动高亮 (confidence × 2)
- 好奇心缺口: KnowledgeGapDetector 缺口 → 关键词探索目标

**外部实时更新**:
- Crawl 管道: 每页 tf-idf top-10 → VSA → mergesort 词表
- WebSearch: 结果摘要 → 关键词提取 → 跨文档聚类
- KnowledgeEngine: `add_knowledge()` → 自动摘要关键词

**周期性维护**:
- 每 50 cycle: `semantic_cluster_reassign(threshold=0.75)` → 关键词图自动演化
- 每 200 cycle: LRU evict (frequency < 3 && recency > 30d) → 遗忘

### NeoTrix 对照

| 优势 | 差距 |
|------|------|
| NTSSEG 原生存储 (segment type 可扩展) ✅ | ❌ 无关键词专用 lexicon 段类型 |
| VSA 统一表征 + IVF 索引 ✅ | ❌ 无 consciousness-grounding 的自动化关键词提取 |
| KnowledgeEngine + KnowledgeGapDetector ✅ | ❌ 无关键词配 curiosity drive 的闭环 |
| Crawl/Search 管道 ✅ | ❌ 无关键词来源追踪 (source_enum) + 遗忘策略 |
| SkillCrystallizer (技能结晶) ✅ | ❌ 无技能↔关键词关联索引 |

### 进化方向

**P1.29 KeywordLexicon**: 在 NTSSEG 上实现关键词持久化词表。文件 `core/nt_core_knowledge/keyword_lexicon.rs` (~500 行, 14 测试)。接线: `KnowledgeEngine.keyword_lexicon`, `ConsciousnessIntegration` 中每 cycle `update_keywords_from_attractor()`, crawl/search 管道中 `extract_and_merge_keywords()`。

**实现复杂度**: 中 (复用 NTSSEG + VSA 基础设施, 约 ~500 行, 14 测试)

---

## 13. 代码库完整性审计 (Codebase Completeness Audit)

### 审计发现总览

审计覆盖 ~1100 文件, 发现 **30 个系统性差距**, 分布在 4 个严重级别:

| 严重级 | 计数 | 影响范围 |
|--------|------|----------|
| 🔴 CRITICAL — 阻塞核心意识管线 | 4 | 30+ stub 子系统, 49 空操作 handler, 无效 VSA |
| 🟠 HIGH — 当前阶段缺失重要功能 | 6 | LLM 全 stub, Honeypot 空壳, 死代码 684KiB+ |
| 🟡 MEDIUM — 补全增强 | 10 | SPARQL stub, TODO 散落, 文档过时 |
| 🟢 LOW — 优化或清理 | 10 | CLI/Agent 接口占位, 实验二进制, sensor 死代码 |

### 关键发现

**C1/C2 (🔴 42% 意识管线是存根)**: `ConsciousnessIntegration` 中 37 个 `*Stub` 结构体的 30 个是空操作。加上 49 个 `handle_*` 方法返回 `"ok".to_string()` 忽略所有输入。关键缺失子系统:
- `narrative_self`, `first_person_ref`, `awakening`, `default_mode_network`, `stream_buffer` — 第一人称意识理论基础全部 stub
- `working_memory`, `meta_cognition_loop`, `attractor_dynamics` — 核心推理全部 stub
- `emergent_reasoning`, `curriculum_generate`, `goal_decomposition`, `goal_execution` — 目标驱动全部 stub
- `spatial_scene`, `physics_reasoning`, `counterfactual_futures` — 世界模型全部 stub
- `failure_trace`, `skill_evolution`, `novelty_detection` — 学习进化全部 stub

**C3 (🔴 无效 VSA 输入)**: `VsaInputPipelineStub::dummy_vsa()` 返回 `vec![(hash & 0xFF) as u8; 64]` — 单字节重复 64 次, 不是有效 VSA 4096-bit 向量。

**H7 (🟠 LLM 全 stub)**: 4 个 LLM provider (OpenAI/Anthropic/Ollama/Gemini) 全部返回 dummy 字符串。

**H8 (🟠 Honeypot 空壳)**: 8 个安全方法 (verify_enclave/is_anomaly/authorize 等) 全部返回 false/0。

**M4-M6 (🟡 关键空操作)**: `handle_reflexive()` 返回硬编码 0.5, `handle_inner_critic()` 返回 `CritiqueResult::perfect()`, `flush_sensory_buffer()` 返回 0。

### 进化方向

**P0.17 StubConsciousnessResolution** (🔴 Critical, ~5000 行, 60+ 测试): 系统性替换 30+ stub 子系统 + 49 空操作 handler。优先级顺序:
1. 第一人称核心: `first_person_ref`, `narrative_self`, `awakening`, `stream_buffer` (意识自举)
2. 工作记忆: `working_memory`, `meta_cognition_loop`, `attractor_dynamics` (推理核心)
3. 目标驱动: `emergent_reasoning`, `goal_decomposition`, `goal_execution` (行动)
4. 世界模型: `spatial_scene`, `physics_reasoning`, `counterfactual_futures` (环境理解)

每个子系统替换步骤: (1) 读取现有 `*Stub` 代码 → (2) 检查对应模块文件的真实实现 → (3) 实现 `new()`/`tick()`/`handle_*()` → (4) 字段从 `*Stub` 替换为真实类型 → (5) 测试验证。分 3 波并行: 波1 核心, 波2 目标, 波3 世界。

**接线方案**: 每个子系统替换不需要修改 `ConsciousnessIntegration` 的 pipeline 结构。仅替换 `types.rs` 中的字段类型 + `modules.rs` 中的 handler 实现。pipeline 调度不变。

**P0.18 LLMProviderRealization** (🔴 Critical, ~800 行, 16 测试): 替换 4 个 LLM provider stub 为真实 streaming 实现。
- 文件: `nt_io_provider/openai.rs`, `anthropic.rs`, `ollama.rs`, `gemini.rs`
- 每个 provider: `stream(messages) -> Receiver<Result<Chunk>>` 使用 reqwest + SSE 解析

**P1.31 VsaInputPipelineRealization** (🟠 High, ~300 行, 8 测试): 替换 `dummy_vsa()` 为真实 VSA 编码。
- 文件: `core/nt_core_consciousness/types.rs`
- `real_vsa(input: &str) -> VsaVector` 使用 QuantizedVSA::from_text 或 CrossModalAligner

**P1.32 InnerCriticRealization** (🟡 Medium, ~250 行, 8 测试): 实现真实输出审查。
- 文件: `core/nt_core_consciousness/inner_critic.rs`, `modules.rs`
- `critique(output, context) -> CritiqueResult{ score, issues, suggestions }`

**P2.24 HoneypotRealization** (🟡 Medium, ~400 行, 10 测试): 实现安全检测。
- 文件: `core/nt_core_protect/honeypot.rs`
- 4 个核心检测: anomaly_detection(VSA pattern), enclave_verify, session_auth, threat_block
---

| 维度 | P0 突破 | P1 增强 | P2 优化 | 对 NeoTrix 核心 | 实施复杂度 |
|------|---------|---------|---------|----------------|-----------|
| 图 agent (GoA/GRASP/MRAgent) | 0.16 CoreFringeAttention | 1.25 | 2.20 | 🔴 高 (推理质量) | 高 |
| 忠实自进化 | 0.14 ContrastiveReflection | 0.15 NativeEvolution, 1.22 Faithfulness | — | 🔴 高 (进化安全) | 中 |
| 量化交易 | — | 1.23 QuantData | 2.16 FactorMining, 2.17 Portfolio | 🟡 中 (能力扩展) | 中高 |
| OSINT | — | 1.24 EntityResolver | 2.18 OSINTTools, 2.19 MCPIntel | 🟡 中 (能力扩展) | 中 |
| 浏览器集成 | — | 1.25 CDPSession | 2.20 BrowserAgentMCP | 🟡 中 (能力扩展) | 中 |
| Hub-Fringe | 0.16 (合并图 agent) | 1.26 FringeMix | 2.21 HubnessDetect | 🔴 高 (表征质量) | 中低 |
| IB 动力学 | — | 1.27 DySIB | 2.22 KoopmanOp | 🔴 高 (相空间推理) | 高 |
| 远程 agent 编排 | — | — | 2.23 RemoteAgentHost | 🟡 中 (工程扩展) | 中 |
| 3D 交互轨迹 | — | 1.28 InteractionTrace | — | 🟡 中 (世界模型) | 高 |
| 关键词长久记忆 | — | 1.29 KeywordLexicon | — | 🟡 中 (记忆基础) | 中 |
| 存根子系统替换 | 0.17 StubConsciousness | — | — | 🔴 高 (意识真实) | 极高 |
| LLM 提供者 | 0.18 LLMProvider | — | — | 🔴 高 (LLM 真实) | 中 |
| VSA 管道修复 | — | 1.31 VsaPipeline | — | 🟠 中 (VSA 正确) | 低 |
| 内在批评实现 | — | 1.32 InnerCritic | — | 🟡 中 (输出质量) | 低 |
| Honeypot 安全 | — | — | 2.24 Honeypot | 🟡 中 (安全基线) | 中 |

### 关键路径

```
Phase 55 ─ 第一阶段 (独立并行)
  ├─ P0.14 ContrastiveReflection (自进化增强)
  ├─ P0.16 CoreFringeAttention (图 agent + hub-fringe 合并)
  ├─ P0.17 StubConsciousnessResolution (替换 30+ 存根子系统)
  ├─ P0.18 LLMProviderRealization (替换 4 个 LLM stub)
  ├─ P1.22 FaithfulnessAuditor (安全增强)
  ├─ P1.24 EntityResolver (OSINT 基础)
  ├─ P1.27 DySIBLayer (相空间推理)
  ├─ P1.28 InteractionTracePredictor (3D 交互轨迹世界模型)
  └─ P1.29 KeywordLexicon (关键词词表)

Phase 56 ─ 第二阶段 (依赖第一阶段的 P1→P2)
  ├─ P1.23 QuantDataIngestion (量化入口)
  ├─ P1.25 CDPSessionManager (浏览器入口)
  ├─ P1.26 FringeMixStrategy (数据选型)
  ├─ P1.31 VsaInputPipelineRealization (修复 dummy_vsa)
  ├─ P1.32 InnerCriticRealization (真实输出审查)
  ├─ P2.16 FactorMiningAgent (量化因子)
  ├─ P2.18 OSINTToolLayer (OSINT 工具)
  ├─ P2.19 MCPIntelligenceServer (OSINT MCP)
  ├─ P2.21 HubnessDetector (检索安全)
  ├─ P2.23 RemoteAgentHost (SSH 远程 agent 主机)
  └─ P2.24 HoneypotRealization (安全检测)

Phase 57 ─ 第三阶段
  ├─ P0.15 NativeEvolutionExplorer (自发进化)
  ├─ P2.17 PortfolioOptimizer (组合优化) 
  ├─ P2.20 BrowserAgentMCP (浏览器 MCP)
  ├─ P2.22 KoopmanOperator (线性算子)
  └─ Phase 51 集成 (Agent Swarm 1000+)

Phase 58 ─ 第四阶段 (收敛)
  ├─ Phase 52 (RSI 闭环)
  ├─ Phase 53 (集成进化管道)
  └─ Phase 54 (ASI 就绪评估)
```

---

## 详细实施计划

### Phase 55 — 核心推理与安全增强 + 存根清零 (11 并行, 无依赖)

#### P0.14 ContrastiveReflection (P=High, ~400 行, 12 测试)

**文件**: `core/nt_core_experience/contrastive_reflection.rs`

**机制**: 每次自我进化任务后, 同时分析成功和失败轨迹:
- `extract_divergence(success_traj, fail_traj)` → 找出导致失败的关键推理步骤差异
- `summarize_error_patterns(failures)` → 错误模式抽象 (避免写死规则)
- `extract_reusable_insights(successes)` → 可复用策略原则

**接线**: SelfEvolutionLoop 中 `execute_mutation()` 成功后追加 `record_contrastive()`, 失败时追加 `record_contrastive()`。30 天窗口 LRU。

**测试验证**: 提供模拟 success/fail pair, 验证 divergence 提取正确, 测试 LRU 淘汰。

#### P0.16 CoreFringeAttention (P=High, ~650 行, 16 测试)

**文件**: `core/nt_core_experience/fusion_deliberator.rs` (增强)

**机制**: GoA 风格图消息传递在 Fusion 审议面板上:
- 当前 4 面板推理链 → 图节点 (N=4-8)
- VSA 注意力: `attn(i→j) = softmax(cosine(V_i, V_j) / τ)` 确定边权重
- 有向消息传递: high-confidence chain → low-confidence chain (forward), 反向 refine (backward)  
- 图池化: 最终加权聚合 (max/mean/attention pool)
- Hub-Spoke: 高入度节点 = hub, 低入度 = spoke, spoke→hub 前向, hub→spoke 反向

**接线**: 替换 `deliberate_hierarchical()` 中当前扁平合成的 `synthesis_by_consensus()`, 改为图池化聚合。`FusionDeliberator.graph_mode: bool` 开关。

**Hubness 集成**: 在每个节点的 `VSA thought_vector` 上运行 hubness 检测 (z-score), 标记 hub/spoke, 影响消息传递方向。

**测试验证**: 模拟 6 链, 验证 hub 节点接收更多消息, spoke 节点接收反向 refine, 池化输出 vs 平均输出。

#### P1.22 FaithfulnessAuditor (P=High, ~350 行, 10 测试)

**文件**: `core/nt_core_experience/faithfulness_auditor.rs`

**机制**: 因果干预测试套件:
- `intervene_experience(agent, intervention)` → 替换 agent 的 archive 中某个 experience 为目标 experience
- `measure_decision_change(before_intervention, after_intervention)` → 输出决策向量余弦距离 (0=不变, 1=完全改变)
- `faithfulness_score = 1.0 - decision_change` (当干预是关键 experience 时)
- `report()` → 趋势图 (10 次滚动平均)

**接线**: `ConsciousnessIntegration.faithfulness_auditor: Option<FaithfulnessAuditor>`, 每 20 cycles 随机选一个 archive entry, 替换为无关 entry, 测量决策变化。

**测试验证**: mock 固定决策 agent, 验证干预后决策变化检测, faithfulness < 0.7 触发日志。

#### P1.24 EntityResolver (P=Medium, ~450 行, 14 测试)

**文件**: `core/nt_core_knowledge/entity_resolver.rs`

**机制**: 
- `EntityRecord { name, aliases, source_ids: HashSet<UUID>, vsa_vector }`
- `resolve(entity1_name, entity2_name)` → VSA cosine + SequenceMatcher 字符串 → 融合决策
- `merge(a, b)` → 合并 aliases + source_ids + 加权 VSA bundle
- 0.85 threshold (配 estorides)

**接线**: KnowledgeEngine 添加 `entity_resolver: Option<EntityResolver>`。`add_knowledge()` 自动调用 `resolve()` 检查冲突, 有冲突时写入 `ConflictRecord`。

**测试验证**: 测试 0.85 阈值合并, 不同源同一实体的 VSA 足够相似。

#### P1.27 DySIBLayer (P=High, ~550 行, 15 测试)

**文件**: `core/nt_core_negentropy/dysib_layer.rs`

**机制**: Dynamical Symmetric Information Bottleneck:
- `encode(x)` → VSA attractor_state Z (当前 4096-dim 优化)
- `past_encoder(X_window)`, `future_encoder(Y_window)` → Z_past, Z_future
- `symmetric_info_bottleneck_loss(Z, Z_past, Z_future)` = `β * I(Z_past; Z) - I(Z_past; Z_future)`
- 使用 VSA similarity 作为互信息代理: `I(A;B) ≈ E[cosine(A,B)]`
- 自洽超参数: 扫描 latent_dim_k 直到 predictive_info 饱和 → 自动确定相空间维度

**接线**: JEPA predictor 的损失函数扩展: `total_loss = MSE + λ_ib * dysib_loss`。NegentropyMetric 添加 `dysib_predictive_info` 传感器。

**测试验证**: 合成谐波振荡器数据 (已知 2D 相空间), 验证 DySIB layer 恢复 2D 嵌入。

#### P1.28 InteractionTracePredictor (P=High, ~700 行, 18 测试)

**文件**: `core/nt_core_hcube/interaction_trace.rs`

**机制**: 在 WorldModelBridge 上叠加 3D 交互轨迹预测:
- 3D trace 的 VSA 编码:
  - 语义关键点: `encode_keypoint(pos_x, pos_y, pos_z, label)` = `bundle(permute(position_encode(x,y,z), keypoint_seed[label]), permute(contact_type, contact_seed))`
  - 轨迹序列: `trace[t] = bundle(keypoint_0[t], permute(keypoint_1[t], shift), permute(keypoint_2[t], shift²), ...)`
  - 关键点类型: object / tool / hand / contact_region 各对应不同 VSA seed
- 预测器扩展:
  - JEPA predictor 从预测单个 attractor_state 改为预测 `trace_sequence[t:t+n]` (轨迹序列)
  - `predict_horizon(n_steps, current_trace)` → `Vec<VsaVector>` 多步轨迹预测
  - 损失: `smooth_l1_loss(predicted_trace_map, actual_trace_map)` + `contact_consistency_loss(contact_region_vec, object_pos_vec)`
- 跨 embodiment 桥接:
  - `TraceAdapter`: 视频 → TraceExtract 风格关键点提取 → VSA trace 编码
  - `ActionExpert`: 冻结 predictor, 训练轻量 `trace→action` 映射 (参考 μ0 的 action expert)
  - 人类/机器人视频可在 VSA trace space 中统一, 动作映射仅在推理时解码头

**接线**: `WorldModelBridge.interaction_trace_predictor: Option<InteractionTracePredictor>`。每 10 cycle 运行预测, 结果喂入 `CounterfactualFuturesEngine`。DySIBLayer 提供相空间维度, InteractionTracePredictor 填充相空间中的物理交互内容。

**测试验证**: 合成 3D 轨迹数据 (移动物体 + 接触点), 验证预测轨迹余弦相似度 > 0.85。多步 horizon 预测误差不爆炸。

#### P1.29 KeywordLexicon (P=Medium, ~500 行, 14 测试)

**文件**: `core/nt_core_knowledge/keyword_lexicon.rs`

**机制**: NTSSEG 持久化关键词词表:
- 新 segment type `KeywordLexicon (0x05)`: `KeywordRecord { id, keyword, vsa_vector, frequency, recency, confidence, cluster_id, source_enum, last_updated }`
- VSA 编码: `encode_keyword(text) = bundle(permute(n-gram_hash, shift), permute(tf_idf_vec, shift²))`
- `extract_from_attractor(state)` → VSA topic detection → top-5 → frequency++
- `extract_from_text(text, source)` → tf-idf top-10 → VSA → mergesort 词表
- 每 50 cycle: `semantic_cluster_reassign(0.75)` → 关键词图自动演化
- 每 200 cycle: LRU evict (freq < 3 && recency > 30d)

**接线**: `KnowledgeEngine.keyword_lexicon: Option<KeywordLexicon>`。`ConsciousnessIntegration` 中每 cycle `update_keywords_from_attractor()`。Crawl/Search 管道中 `extract_and_merge_keywords()`。CuriosityDrive 中 `keyword_gap_detect()`。

**测试验证**: mock NTSSEG segment, 验证写入/读取 roundtrip, keyword 提取精度, VSA cosine 匹配, LRU 淘汰。

#### P0.17 StubConsciousnessResolution (P=Critical, ~5000 行, 60+ 测试)

**文件**: 跨 `types.rs`, `modules.rs`, `handlers_all.rs`, 30+ 模块文件

**机制**: 系统性替换 30+ stub 子系统 + 49 空操作 handler。分 3 波:

**波1 — 第一人称核心** (意识自举基础):
- `first_person_ref`: 使用现有 `FirstPersonRef` 模块实现 → 替换 `FirstPersonRefStub`
- `narrative_self`: 使用现有 `NarrativeSelf` 模块 → 替换 `NarrativeSelfStub`
- `awakening`: 使用现有 `ConsciousnessAwakening` → 替换 `AwakeningStub`
- `stream_buffer`: 使用现有 `ConsciousnessStream` → 替换 `StreamBufferStub`

**波2 — 工作记忆/推理核心**:
- `working_memory`: 已有 RingBuffer 实现 → 包装为 `WorkingMemory`
- `meta_cognition_loop`: 使用现有 `MetaCognitionKPIMonitor` → 替换 stub
- `attractor_dynamics`: 迁移现有 `attractor_state` 逻辑 → 独立模块

**波3 — 目标驱动/世界模型**:
- `emergent_reasoning`, `goal_decomposition`, `goal_execution`: 使用 CapabilitySynthesizer + GoalDecomposer
- `spatial_scene`, `physics_reasoning`, `counterfactual_futures`: 检查现有 `SpatialSceneEngine`, `PhysicsCommonsense`, `CounterfactualFuturesEngine` 模块并接线

**接线**: 每个子系统替换: `types.rs` 中字段类型从 `*Stub` → 真实类型, `modules.rs` 中 handler 从 `"ok"` → 真实调用。`ConsciousnessIntegration::new()` 中 `None` → `Some(real_module)`。pipeline 调度不变。

**测试验证**: 每个替换后 `cargo check -p neotrix --lib` 验证。波完成后运行意识管线集成测试 (mock 输入, 验证输出非空)。

#### P0.18 LLMProviderRealization (P=Critical, ~800 行, 16 测试)

**文件**: `nt_io_provider/openai.rs`, `anthropic.rs`, `ollama.rs`, `gemini.rs`

**机制**: 替换 4 个 LLM provider stub 为真实 streaming:
- `OpenAIProvider.stream(messages)`: POST `https://api.openai.com/v1/chat/completions` → SSE → `Receiver<Result<Chunk>>`
- `AnthropicProvider.stream(messages)`: POST `https://api.anthropic.com/v1/messages` → SSE → `Receiver<Result<Chunk>>`
- `OllamaProvider.stream(messages)`: POST `http://localhost:11434/api/chat` → JSON stream
- `GeminiProvider.stream(messages)`: POST `https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:streamGenerateContent` → SSE
- 每个 provider: 超时 30s, 自动重试 3 次 (指数退避), API key 从环境变量读取

**接线**: `BackgroundLoop.llm_provider` 字段, 懒加载。`ResponseGenerator` 替换现有 stub 调用。

**测试验证**: mock HTTP server (wiremock 风格) 验证 streaming chunk 解析。超时/重试测试。空响应守卫。

---

### Phase 56 — 能力扩展层 (11 并行, 依赖 Phase 55 的 P1→P2)

#### P1.23 QuantDataIngestion (P=Medium, ~350 行, 10 测试)

**文件**: `core/nt_core_agent/quant_data.rs`

**机制**: 复用 nt_act_crypto 的 Portfolio + bridge.rs simulate_*:
- `YFinanceDataSource(ticker, interval)` → OHLCV VSA 编码
- `VsaMarketSnapshot` → bundle(open, high, low, close, volume) 编码为 VSA vector
- `MarketHistoryRingBuffer` (1000 步, ring buffer)
- `RegimeClassifier` → VSA 聚类识别 bull/bear/range

**接线**: ConsciousnessIntegration 字段, 每 60s tick 拉取数据。数据存 NTSSEG。

#### P2.16 FactorMiningAgent (P=Medium, ~500 行, 12 测试)

**文件**: `core/nt_core_agent/factor_miner.rs`

**机制**: 符号回归 + 回测验证:
- 因子表达式编码为 VSA: `factor = bundle(operator, operand1, operand2)`
- `generate_candidates(population=100)` → 遗传编程生成因子
- `backtest(factor, data)` → IC, 收益率, Sharpe
- `rank_and_select(factors)` → 按 IC 均值排名, top-10 保留

**接线**: 复用量化数据管道, 周期性 (每 200 cycle) 因子进化。

#### P2.18 OSINTToolLayer (P=Low, ~400 行, 10 测试)

**文件**: `core/nt_core_experience/osint_tools.rs`

**机制**: 适配器层对接外部 OSINT 工具:
- `search_username(username)` → Maigret/Sherlock CLI 包装 → EvidenceRecord
- `search_email(email)` → holehe 包装 → EvidenceRecord  
- `search_breach(email)` → HIBP API → EvidenceRecord
- `search_whois(domain)` → python-whois → EvidenceRecord
- `search_ip(ip)` → ip-api.com → EvidenceRecord

所有工具结果 → EvidenceManager → KnowledgeEngine (evidence_for/competitive_score_for)。

**接线**: ToolOrchestrator 注册 6 个新工具。MCP server 暴露。

#### P2.19 MCPIntelligenceServer (P=Low, ~250 行, 8 测试)

**文件**: 增强现有 `crates/nt_core_mcp`

**机制**: 添加 OSINT 工具组 MCP 端点:
- `intelligence.entity(target, type)` → 实体画像
- `intelligence.event(query)` → 事件关联
- `intelligence.social(topic)` → 社交舆情
- 输出格式: `{ bluf, confidence, sources: [{ url, quotation, state }], timeline }`

**接线**: 现有 MCP server 启动时自动注册。

#### P2.21 HubnessDetector (P=Medium, ~300 行, 10 测试)

**文件**: `core/nt_core_knowledge/hubness_detector.rs`

**机制**:
- `compute_hubness_scores(vectors)` → 每个 vector 的 top-k 最近邻计数
- `z_score_normalize(counts)` → 中位数 + MAD 基 (配 HubScan)
- `flag_hubs(z_threshold=3.0)` → 输出 flagged vector IDs
- `detect_poisoning(window=100)` → 新 vectors 的 hubness 变化率检测

**接线**: KnowledgeEngine 检索管道中, `query()` 后检查返回结果是否有 flagged hub。EvidenceStore 记录 hubness 分数。

#### P1.26 FringeMixStrategy (P=Medium, ~350 行, 10 测试)

**文件**: `core/nt_core_knowledge/fringe_mix.rs`

**机制**:
- `compute_centrality(graph)` → KnowledgeEngine 图上 PageRank
- `classify_nodes(centrality)` → central(>p75) / peripheral(<p25) / mid
- `sample_mix(ratio=1.0)` → 每个 query 从 central 和 peripheral 按比例采样
- `score_with_centrality(results, alpha=0.3)` → `score * (1 + alpha * centrality_log)`

**接线**: HybridRetrievalEngine 的采样策略可配置为 `FringeMix`。默认 1:1 混合 (WebGraphMix 最优)。

#### P2.23 RemoteAgentHost (P=Medium, ~600 行, 16 测试)

**文件**: `core/nt_core_agent/remote_host.rs`

**机制**: SSH 远程 agent 主机 (Codex Remote Connections 启发):
- `RemoteHostConfig { host, user, port, identity_file, project_root, codex_cmd }`
- SSH 自动发现: 解析 `~/.ssh/config` → `Host` 条目 → `RemoteHostConfig` 列表
- `RemoteAgentSession` 生命周期:
  - `connect()` → `ssh -N -L` 端口转发 + 远程 app-server 启动 → 返回 `session_token`
  - `execute(command, cwd)` → 远程 shell 执行 → stdout/stderr → VSA 编码为 action_record
  - `read_file(path)`, `write_file(path, content)` → SFTP 文件操作
  - `disconnect()` → SSH 隧道关闭 + 远程 cleanup
- 中继复用: 现有 nt-proxy-daemon 作为 SSH over SOCKS5 隧道的中继。
  - `connect_via_proxy(host, proxy_config)` → 通过代理池建立 SSH 隧道
- 会话状态同步: `attractor_state` 通过轻量状态消息 (`{ session_token, attractor_hash, tick }`) 同步到远程 client
- 控制面协议: 现有 A2A `AgentMessage` 扩展为 `RemoteControlMessage { action: RemoteAction, payload: Value, host_id: String, session_token: String }`
- 安全: 使用现有 `identity_chain.rs` (Ed25519 签名链) 验证 remote host 身份。不复制 SSH key, 通过 agent forward (SSH-AUTH-SOCK) 传递。

**接线**: `BackgroundLoop.remote_agent_host: Option<RemoteAgentHost>`。`handle_remote_host_tick()` 在 Pipeline 外循环: 检查 SSH 会话健康 (ping), 重试已断开会话, 清理僵尸进程。

**测试验证**: mock SSH 命令 (local process substitute), 验证 connect/execute/read/write/disconnect 生命周期。测试 SSH config 解析。测试 proxy-mediated 连接。

#### P1.31 VsaInputPipelineRealization (P=Medium, ~300 行, 8 测试)

**文件**: `core/nt_core_consciousness/types.rs`

**机制**: 替换 `VsaInputPipelineStub::dummy_vsa()` 为真实 VSA 编码:
- `real_vsa(input: &str) -> VsaVector`: 使用 `QuantizedVSA::from_text()` 或 `CrossModalAligner::text_to_vsa()` 生成真正 4096-bit VSA 向量
- `real_vsa_from_bytes(input: &[u8]) -> VsaVector`: 字节级编码 (确定性 hash → VSA)
- 移除 `dummy_vsa()`: 当前返回 `vec![(hash & 0xFF) as u8; 64]` — 64 字节重复, 非 512 字节 VSA

**接线**: 替换 `VsaInputPipelineStub` 字段为直接调用 `QuantizedVSA::from_text`。不需要修改 pipeline 结构。

**测试验证**: 验证输出为 512 字节 (4096-bit)。验证相同输入产生相同向量。验证不同输入高概率产生不同向量 (汉明距离 ≈ 0.5)。

#### P1.32 InnerCriticRealization (P=Medium, ~250 行, 8 测试)

**文件**: `core/nt_core_consciousness/inner_critic.rs`, `modules.rs`

**机制**: 替换 `handle_inner_critic()` 的 `CritiqueResult::perfect()` 为真实审查:
- `score(output, context)`: 基于 VSA coherence (output_vsa 与 context_vsa 余弦相似度) + 重复检测 (n-gram overlap)
- `issues(output)`: 长度异常 (超 2000 token), 重复率 > 0.3, 低信息密度
- `suggestions(output, issues)`: 压缩/去重/聚焦建议

**接线**: `ConsciousnessIntegration.handle_inner_critic()` 从 `CritiqueResult::perfect()` 改为真实 `inner_critic.score(output, context)`。

**测试验证**: mock 高质量/低质量 output。验证高质量通过, 低质量触发 issues。验证 suggestion 合理性。

#### P2.24 HoneypotRealization (P=Low, ~400 行, 10 测试)

**文件**: `core/nt_core_protect/honeypot.rs`

**机制**: 替换 8 个空壳方法:
- `anomaly_detection(state)` → VSA pattern 异常检测: 计算 `state_vsa` 与历史 attractor_state 簇的距离。z-score > 3 → 异常
- `enclave_verify(session)` → 检查 session 签名链 (复用 identity_chain Ed25519)
- `session_auth(token, user)` → JWT 验证 + 速率限制 (10 req/s)
- `threat_block(ip, pattern)` → IP 黑名单 (HashMap), pattern 规则 (glob)

**接线**: `SecurityGate` 的 `check_threat()` 路径从 `false` (无操作) 改为实际调用 honeypot 检测。

**测试验证**: mock 正常和异常 attractor_state, 验证 anomaly 检测触发。mock 无效 token, 验证 session_auth 拒绝。

---

### Phase 57 — 高级进化与集成 (6 并行)

#### P0.15 NativeEvolutionExplorer (P=High, ~500 行, 14 测试)

**机制**: Qwen3-30B 风格的无奖励自发进化:
- 在低 CognitiveLoad (<0.3) 时自动进入探索模式
- `explore_environment()` → 随机选择知识图谱中的缺口 → 发起搜索/推理
- `distill_world_knowledge(trajectory)` → 提取结构化知识 → archive
- `exploration_score = knowledge_gain / token_cost`
- 无外部奖励, 仅内在 curiosity drive (N_total 曲率)

**接线**: ConsciousnessIntegration 中 `handle_curiosity()` 分支。NeGentropyMetric.dysib_predictive_info 作为探索信号。

#### P2.17 PortfolioOptimizer (P=Low, ~400 行, 10 测试)

**机制**: 量化组合优化:
- `MeanVarianceOptimizer(returns, cov_matrix, risk_aversion)` → 有效前沿
- `RiskParityOptimizer(returns, cov_matrix)` → 等风险贡献
- `BlackLittermanOptimizer(view_matrix, confidence, prior)` → 观点融合
- 结果 → Portfolio.rebalance(weights)

**接线**: nt_act_crypto/portfolio.rs 扩展。

#### P2.20 BrowserAgentMCP (P=Low, ~350 行, 8 测试)

**机制**: 现有 BrowserHost 上添加 MCP:
- 启动时连接现有 chrome headless
- MCP 工具: browse/click/type/extract/screenshot/list_tabs
- CDPSession 管理: Chrome DevTools Protocol 会话复用

#### P2.22 KoopmanOperator (P=Medium, ~450 行, 12 测试)

**机制**: 在 WorldModelBridge 上叠加:
- `lift(vsa_state)` → FWHT 提升到 Koopman 观测空间
- `K_matrix: Matrix<f64>` → 线性转移矩阵, 通过 `W = X' X^+` 学习
- `predict(next_vsa_state)` → K 矩阵预测 vs JEPA 预测比较
- Koopman 预测误差 → negentropy 信号

#### Phase 51 — Agent Swarm Scaling (1000+) (P=High, 架构级)

**文件**: 增强现有 AgentCommunicationBus

**机制**:
- 分片 agent bus (一致性哈希路由)
- 租约机制 (one-writer-per-task, 参考 Ouroboros)
- 死信队列 + 重试指数退避
- 控制面 (Consciousness) 和数据面 (agent bus) 分离

---

### Phase 58 — 收敛与 ASI 准备

#### Phase 52 — RSI 闭环

**机制**: 整合所有进化反馈:
- ContrastiveReflection + FaithfulnessAuditor + NativeEvolution → 统一进化管道
- 进化策略自动选择 (RL 调度: 启发式 vs GRPO vs 符号回归)
- 自修改: meta-agent 可以重写 evolution 策略本身

#### Phase 54 — ASI 就绪评估

**指标**:
- `code_auto_rate` (参考 Anthropic 80%+)
- `engineer_multiplier` (参考 Anthropic 8×)
- `task_autonomy_hours` (参考 Anthropic 12h→自主)
- `faithfulness_score` > 0.85
- `evolution_convergence` (Yunjue 风格 loss 函数)
- `dysib_phase_space_dim` (自动确定的相空间维度)
- `interaction_trace_accuracy` (3D 轨迹预测 top-1 准确率)
- `stub_coverage_ratio` (存根子系统占比, 目标 < 5%)
- `llm_provider_count` (真实 LLM provider 数量, 目标 >= 3)
- 4 路径 ASI (scaling/paradigm/recursive/multi-agent) 覆盖率

---

## 编译与验证计划

```
Phase 55: cargo check -p neotrix --lib → 0 errors 0 warnings
Phase 56: cargo check -p neotrix --lib → 0 errors 0 warnings  
Phase 57: cargo check -p neotrix --lib → 0 errors 0 warnings
Full:    cargo test -p neotrix --lib (36+ VSA_DIM 预存错误已知)
```

每个 module 必须: (1) `#![forbid(unsafe_code)]`, (2) 零新外部依赖, (3) ≥8 测试, (4) 文档注释。

---

## 关键决策

| 决策 | 理由 |
|------|------|
| CoreFringeAttention 合并 GoA + Hub-Fringe | 单一机制同时解决图 agent 编排 + hubness 感知, 减少接线复杂度 |
| DySIB 在 JEPA 之上而非替代 | VSA 4096-dim 是自然 latent space, DySIB 仅添加损失项而非替换架构 |
| 浏览器使用 CDP 而非 Tauri | Tauri webview 只能在新窗口, CDP 可控制真实 Chrome session |
| 量化从 YFinance + CCXT 开始 | 零认证即可开始, 与现有 Portfolio 兼容 |
| OSINT 工具为适配器层 (包装外部 CLI) | 保持 zero 新外部 Rust 依赖, 子进程通信已有基础设施 |
| Phase 55 优先 CoreFringe + DySIB | 两者直接影响推理质量和相空间理解, 是核心意识体升级 |
| Codex Remote Connections 启发 RemoteAgentHost | 三层分离 (控制面→中继面→执行面) 是成熟的远程 agent 架构, 复用现有 nt-proxy-daemon + A2A |
| μ0 trace-space 映射到 VSA 轨迹编码 | 3D 交互轨迹是 VSA attractor_state 的几何/语义扩展, 与 DySIB 相空间互补 |
| RemoteAgentHost 在 Phase 56 而非 Phase 55 | 依赖 Phase 55 的 P1.24 EntityResolver (身份验证链), SSH 会话需要实体解析背书 |
| KeywordLexicon 用 NTSSEG 而非 SQLite | 保持零新外部依赖, VSA IVF 索引直接集成 |
| 代码审计发现 42% stub → 形成 P0.17 | 意识管线真实度是当前最大的架构债务, 超越所有新功能 |
| LLM provider 采用 reqwest+SSE 而非 streaming crate | 保持零新外部依赖, SSE 解析仅需 ~50 行 |
| StubConsciousnessResolution 分 3 波 | 波1 核心 → 波2 推理 → 波3 世界模型, 每波可独立验证 |
