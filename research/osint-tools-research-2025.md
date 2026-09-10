# NeoTrix OSINT 能力增强研究报告

> 研究日期: 2026-09-11 | 数据来源: GitHub Trending, arXiv, IEEE, MDPI, Medium

---

## 一、GitHub OSINT 工具排行榜 (Top Tier)

### 1.1 Tier-1: 高星标的综合性平台

| 工具 | Stars | 核心能力 | 架构模式 | NeoTrix 吸收价值 |
|------|-------|---------|---------|-----------------|
| **Sherlock** | 90K+ | 用户名跨 400+ 平台搜索 | Python CLI + 多线程 HTTP | 高 — 增强 `nt_world_social` 用户名枚举 |
| **ShadowBroker** | 10.8K | 60+ 实时情报源聚合，暗网地图 | Next.js + MapLibre GL + FastAPI + Python | 高 — 实时遥测聚合架构可借鉴 |
| **OpenCTI** | 9.8K | STIX2.1 威胁情报管理，知识图谱 | GraphQL API + React + Neo4j | 高 — STIX2.1 标准集成 |
| **flowsint** | 7.7K | OSINT 图形化探索工具 | Graph-based investigation + enrichers | 中 — 图分析可视化 |
| **SpiderFoot** | 19.2K | 200+ 模块自动侦察 | Event-driven modules + CLI/Web UI | 高 — 模块化架构参考 |
| **OpenOSINT** | 1.5K | 19 工具 MCP 接口，防幻觉 | AI agent + native MCP server | 极高 — MCP 已是 NeoTrix 通道 |
| **Maltego** | 商业 | 图形化链接分析 + 400+ transforms | Entity-Transform 模型 + Graph viz | 中 — 图分析范式 |

### 1.2 Tier-2: 专精攻击面发现

| 工具 | 核心能力 | 架构模式 | 吸收价值 |
|------|---------|---------|---------|
| **SurfaceMap** | 48+ OSINT 源 + LLM 分析攻击面 | 4 Phase pipeline (LLM→Passive→Active→LLM) | 极高 — LLM+OSINT 编排范式 |
| **ShodanX** | 141 算法生成 Shodan 查询 + favicon 指纹 | Multi-source chaining + QueryIntelligence | 高 — 查询智能生成 + 误报过滤 |
| **reconai** | AI 攻击路径 + 知识图谱 + surface diff | Rule Engine + Local LLM + Graph export | 高 — 规则引擎 + LLM 混合架构 |
| **THOR-autorecon** | 7 阶段自主侦察 + 自适应记忆 | 11 evasion strategies + ThorMemory | 中 — 自适应记忆 + evasion |
| **CAI-OSINT** | CEH 流程自动化 + DeepSeek-R1 | MCP layer + M4STCLAW mesh | 高 — LLM 误报过滤 + 自动化 |
| **NEXUsint** | Multi-INT 融合 + 30+ 实时源 + Deck.gl | In-process scheduler + Knowledge graph | 高 — 多源融合 + 实时流 |

### 1.3 Tier-3: 威胁情报与数据源

| 工具 | Stars | 核心能力 | 吸收价值 |
|------|-------|---------|---------|
| **MISP** | 核心 | 威胁情报共享 + STIX 1.x/2.x | 高 — STIX 标准集成 |
| **SOCINT** | 新 | 44 CTI 连接器 + Dark Web | 高 — 连接器框架 |
| **Harbinger** | 新 | AI 驱动威胁情报 + IOC 富化 | 中 — AI 分析范式 |
| **IntelX** | 商业 | 历史/暗网数据归档 | 高 — 暗网归档能力 |

---

## 二、学术论文摘要 (2024-2026)

### 2.1 Agentic AI OSINT Framework (IEEE BCCA 2025)

**论文**: *A Framework for Embedding Generative and Agentic AI in Open Source Intelligence*

**核心贡献**:
- 首个统一的 Agentic AI OSINT 架构
- RAG + Chain-of-Thought + 自适应 Agent 规划
- 模块化编排层连接 Maltego/Shodan/VirusTotal/MISP
- 内置伦理保障 (偏见检测 + 透明日志 + Human-in-the-Loop)

**NeoTrix 映射**:
- GWT 注意力路由 → 自适应 Agent 规划
- NT-SHIELD → 伦理保障层
- NT-ACT → 工具编排层

### 2.2 IOP-2025: 混合 AI 增强 OSINT 框架 (Zenodo 2025)

**论文**: *Towards a Hybrid AI-Enhanced Framework for Integrated Open-Source Intelligence in 2025*

**核心贡献**:
- 5 层模块化架构: 身份发现 → 邮件/服务关联 → 基础设施分析 → AI 验证 → 法证输出
- 本地 LLM + 多 Agent 编排 + 神经符号推理
- 隐私设计 + GDPR/EU AI Act 合规
- SHA-256 防篡改完整性

**NeoTrix 映射**:
- SEAL Pipeline → 5 层模块化管道
- NT-SHIELD → 隐私/合规保障
- NT-MEMORY → 防篡改完整性

### 2.3 LLM-based OSINT Acquisition (Future Internet 2024)

**论文**: *Empowering LLMs with Toolkits: An Open-Source Intelligence Acquisition Method*

**核心贡献**:
- Chain-of-Thought + 工具调用获取 OSINT
- 调度 + 记忆 + 工具包三模块架构
- 动态工具访问，插件式扩展
- 持续更新的威胁情报库

**NeoTrix 映射**:
- NT-CORE 推理引擎 → CoT 调度
- NT-MEMORY → 持续更新情报库
- 能力网 → 插件式工具扩展

### 2.4 Automated OSINT for Digital Asset Discovery (Computers 2025)

**论文**: *Automated OSINT Techniques for Digital Asset Discovery and Cyber Risk Assessment*

**核心贡献**:
- GBDT 风险评分 + DBSCAN 异常检测
- 4.8M 记录验证，93.3% 漏洞检测准确率
- 误报率降至 2-5% (传统 15-25%)
- 人工工作量减少 58%

**NeoTrix 映射**:
- NT-CORE SelfModel → 风险评分模型
- NT-MIND → ML 驱动的异常检测
- GWT → 注意力调制

### 2.5 INsAN: AI-Based OSINT Profiler (IEEE 2025)

**论文**: *INsAN: An AI-Based Framework Profiler for Automating OSINT*

**核心贡献**:
- 视觉语言模型 (LLaMa Vision 90B) 分析多模态内容
- Instagram 自动化画像构建
- 公开账户 12 秒处理，私密账户 4 秒
- 支持身份验证、冒充检测、威胁画像

**NeoTrix 映射**:
- NT-WORLD → 多模态 OSINT 采集
- NT-CORE → 实体提取 + 关系构建
- NT-SHIELD → 身份验证/冒充检测

### 2.6 Bayesian Cognitive Priors for OSINT (ICFACT 2026)

**论文**: *Beyond the Data: Bayesian Cognitive Priors for Human-Centered OSINT Automation*

**核心贡献**:
- 贝叶斯认知框架表示人类直觉为概率先验
- 分层贝叶斯信念网络融合多源证据
- 决策理论控制层最大化信息增益
- 证据相关性提升 34%，误分类降低 28%

**NeoTrix 映射**:
- NT-CORE E8 → 贝叶斯推理增强
- GWT → 信息增益最大化路由
- NT-MIND → 认知先验学习

### 2.7 CTINexus: CTI Knowledge Graph (Euro S&P 2025)

**论文**: *CTINexus: Automatic Cyber Threat Intelligence Knowledge Graph Construction Using LLMs*

**核心贡献**:
- 优化 ICL 的 LLM 自动提取 CTI
- 层次化实体对齐 (类型分类 + 实体合并 + 链接预测)
- 多 AI Provider 支持 (OpenAI/Gemini/Bedrock/Ollama)
- 知识图谱交互可视化

**NeoTrix 映射**:
- VSA HyperCube → CTI 知识图谱构建
- NT-MEMORY → 实体对齐 + 链接预测
- NT-CORE → LLM 驱动推理

---

## 三、能力差距分析

### 3.1 NeoTrix 当前 OSINT 模块 (13 模块)

```
dns | http | url | vuln | network | dark
fofa | shodan | censys | zoomeye
person | social | credential | port_service
```

### 3.2 缺失能力维度

| 缺失维度 | 描述 | 优先级 | 参考工具/论文 |
|----------|------|--------|-------------|
| **攻击面发现 (EASM)** | 从公司名自动发现所有外部资产 | P0 | SurfaceMap, ShodanX, reconai |
| **威胁情报共享 (CTI)** | STIX 2.1/TAXII 标准化情报交换 | P0 | MISP, OpenCTI, SOCINT |
| **图分析与可视化** | 实体关系图谱 + 交互式探索 | P1 | flowsint, Maltego, NEXUsint |
| **实时遥测聚合** | 60+ 情报源实时流式聚合 | P1 | ShadowBroker, NEXUsint |
| **暗网监控** | 暗网勒索站点/泄露数据持续监控 | P1 | SOCINT, IntelX |
| **ML 驱动风险评分** | GBDT/DBSCAN 自动风险评估 | P1 | 论文 2.4 (Automated OSINT) |
| **AI 攻击路径分析** | 规则引擎 + LLM 混合攻击链推演 | P1 | reconai, CAI-OSINT |
| **LLM 编排层** | CoT + RAG + 自适应工具选择 | P2 | 论文 2.1, 2.3, 2.7 |
| **多模态 OSINT** | 图片/视频/音频情报采集分析 | P2 | 论文 2.5 (INsAN) |
| **漏洞利用路径预测** | CVE → 攻击路径自动推演 | P2 | THOR-autorecon |
| **供应链情报** | 第三方风险评估 + BOD 22-01 合规 | P2 | entity-attack-surface-mcp |
| **密码学钱包分析** | 加密货币地址追踪 | P3 | TheBigBrother |
| **航空/海事情报** | AIS/ADS-B 实时追踪 | P3 | ShadowBroker, NEXUsint |

### 3.3 可增强的现有模块

| 现有模块 | 增强方向 | 增强来源 |
|----------|---------|---------|
| `social` | + Sherlock 400+ 平台枚举 | Sherlock (90K★) |
| `social` | + 多模态内容分析 (图片/视频) | INsAN (论文 2.5) |
| `vuln` | + ML 风险评分 (GBDT) | Automated OSINT (论文 2.4) |
| `vuln` | + 攻击路径自动推演 | reconai 规则引擎 |
| `dark` | + 勒索站点持续监控 | SOCINT 暗网模块 |
| `dark` | + 泄露数据关联 | IntelX |
| `credential` | + 风险评分 + 趋势追踪 | Harbinger IOC 分析 |
| `dns` | + 子域名枚举增强 | SurfaceMap 48+ 源 |
| `network` | + EASM 资产发现 | SurfaceMap/ShodanX |
| `http` | + favicon 指纹 + 技术栈检测 | ShodanX |
| `port_service` | + 11 evasion strategies | THOR-autorecon |

---

## 四、吸收建议 (Actionable Items)

### 4.1 P0 — 立即吸收

| 吸收项 | 来源 | 实现方式 |
|--------|------|---------|
| **EASM 攻击面发现** | SurfaceMap 4 Phase 架构 | 新模块 `nt_world_easm`：LLM Brainstorm → Passive Recon (48+ 源) → Active Probing → LLM Analysis |
| **STIX 2.1 情报交换** | MISP/OpenCTI | 扩展 `nt_memory`：STIX 2.1 native store + TAXII 2.1 client |
| **用户枚举增强** | Sherlock (90K★) | 增强 `nt_world_social`：400+ 平台检查 + 连接池优化 |
| **ML 风险评分** | Automated OSINT 论文 | 新模块 `nt_core_risk_scoring`：GBDT + DBSCAN 异常检测 |

### 4.2 P1 — 近期吸收

| 吸收项 | 来源 | 实现方式 |
|--------|------|---------|
| **图分析引擎** | flowsint/Maltego | 新模块 `nt_memory_graph`：实体关系图 + 交互查询 |
| **暗网持续监控** | SOCINT | 增强 `nt_world_dark`：Tor 代理 + 勒索站点爬取 |
| **规则引擎 + LLM** | reconai | 新模块 `nt_core_rule_engine`：23 规则 + Local LLM |
| **多源并行编排** | NEXUsint/ShadowBroker | 新模块 `nt_world_multi_source`：30+ 源并行调度 |

### 4.3 P2 — 远期吸收

| 吸收项 | 来源 | 实现方式 |
|--------|------|---------|
| **LLM OSINT 编排** | 论文 2.1/2.3 | GWT 增强：CoT + RAG + 自适应工具路由 |
| **知识图谱构建** | CTINexus | VSA HyperCube 增强：实体对齐 + 链接预测 |
| **贝叶斯认知先验** | 论文 2.6 | NT-CORE 增强：概率融合 + 信息增益最大化 |
| **多模态分析** | INsAN | NT-WORLD 增强：图片/视频/音频情报采集 |

---

## 五、架构影响评估

### 5.1 模块新增清单

```
nt_world_easm/          # EASM 攻击面发现 (P0)
nt_world_multi_source/  # 多源并行编排 (P1)
nt_memory_graph/        # 图分析引擎 (P1)
nt_memory_stix/         # STIX 2.1 情报交换 (P0)
nt_core_risk_scoring/   # ML 风险评分 (P0)
nt_core_rule_engine/    # 规则引擎 + LLM (P1)
```

### 5.2 能力网影响

```
新增能力节点:
  - EASM-Discovery (C0 → C1)
  - STIX-Exchange (C0 → C1)
  - Graph-Analysis (C0 → C1)
  - Risk-Scoring-ML (C0 → C1)
  - Rule-Engine-Hybrid (C0 → C1)
```

### 5.3 关键技术决策

| 决策 | 推荐方案 | 理由 |
|------|---------|------|
| EASM 数据源编排 | 按需插件 + 并行调度 | SurfaceMap 48 源证明可行 |
| STIX 存储 | 扩展 KB schema | 避免引入外部数据库依赖 |
| ML 模型选择 | LightGBM (GBDT) | 论文验证 93.3% 准确率 |
| 规则引擎 | Rust 原生实现 | 性能 + 零运行时依赖 |
| LLM 集成 | Ollama 本地 + 云端 fallback | 隐私优先 + 灵活性 |
| 图存储 | KB entity/edge 扩展 | 统一存储层 |

---

## 六、参考文献

1. flowsint — https://github.com/reconurge/flowsint (7739★)
2. OpenOSINT — https://github.com/OpenOSINT/OpenOSINT (1478★)
3. ShadowBroker — https://github.com/BigBodyCobain/Shadowbroker (10805★)
4. Sherlock — https://github.com/sherlock-project/sherlock (90K+★)
5. OpenCTI — https://github.com/opencti-platform/opencti (9839★)
6. MISP — https://github.com/misp/misp
7. SurfaceMap — https://github.com/BreachLine/surfacemap
8. ShodanX — https://github.com/ShubhamGupta-VULNDETOX/ShodanX
9. reconai — https://github.com/rumenjordanov/reconai
10. THOR-autorecon — https://github.com/Ivandiaztnd/thor-autorecon
11. CAI-OSINT — https://github.com/m4stanuj/cai-osint
12. NEXUsint — https://github.com/Kit4Some/NEXUsint
13. SOCINT — https://github.com/diagonalciso/soc-intel
14. Harbinger — https://github.com/CarbeneAI/Harbinger
15. SpiderFoot — 19200★ (smicallef/spiderfoot)
16. Palmieri et al. (2025) "A Framework for Embedding Generative and Agentic AI in OSINT" IEEE BCCA
17. Maio (2025) "Towards a Hybrid AI-Enhanced Framework for Integrated OSINT" Zenodo
18. Yuan et al. (2024) "Empowering LLMs with Toolkits: An OSINT Acquisition Method" Future Internet
19. Babenko et al. (2025) "Automated OSINT for Digital Asset Discovery" Computers
20. Musonip et al. (2025) "INsAN: AI-Based Framework Profiler for Automating OSINT" IEEE ICOCICS
21. ICFACT (2026) "Beyond the Data: Bayesian Cognitive Priors for OSINT" Atlantis Press
22. CTINexus — https://github.com/peng-gao-lab/CTINexus (Euro S&P 2025)
23. Netlas (2025) "AI-Driven Attack Surface Discovery" blog
24. entity-attack-surface-mcp — https://github.com/apifyforge/entity-attack-surface-mcp
