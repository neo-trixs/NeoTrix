# NeoTrix 下一步进化路线图 — 2026-07-04

基于 GitHub Trending 最新趋势 + 现有 KB 缺口 + 架构盲点合成分析。

---

## 一、趋势发现 (GitHub Weekly Trending)

| 项目 | Stars | ★/周 | NeoTrix 相关性 |
|------|-------|-------|--------------|
| **msitarzewski/agency-agents** | 126,845 | +10,483 | **P0: 竞品分析** — 完整 AI Agency (500+ skills), 对比 NeoTrix 的 Subagent/Agent 体系 |
| **DeusData/codebase-memory-mcp** | 25,871 | +10,186 | **P0: 竞品/补强** — MCP 代码库记忆图, 直接对标 NeoTrix GWT+RAG |
| **callesthio/OpenMontage** | 32,825 | +9,213 | P1: Agentic 视频管线 — 12 pipelines/52 tools 架构可借鉴 |
| **usestrix/strix** | 35,534 | +7,567 | P1: AI 安全渗透 — 对标 nt_shield 安全体系 |
| **topoteretes/cognee** | 26,901 | +4,001 | **P0: 直接竞品** — AI 记忆平台, 持久长期记忆, 自托管知识图引擎 |
| **stablyai/orca** | 11,888 | +3,700 | **P0: 竞品分析** — 并行 Agent 开发环境 (ADE) |
| **diegosouzapw/OmniRoute** | 11,113 | +3,631 | **P0: 直接竞品** — 免费 AI 网关 231+ providers, 对标 GatewayV2 |
| **ogulcancelik/herdr** | 11,123 | +3,024 | P1: Rust Agent 多路复用器 — 参考 Rust 原生 Agent 编排 |
| **alibaba/page-agent** | 22,751 | +1,901 | P2: 浏览器 GUI Agent — NT-WORLD browse 增强 |
| **openai/codex-plugin-cc** | 23,690 | +1,296 | P2: Codex↔Claude 互操作 — MCP 生态参考 |

---

## 二、方向一: 对标竞品补齐 (P0, 3周)

| 序号 | 模块 | 对标项目 | NeoTrix 已有 | 差距 |
|------|------|---------|------------|------|
| 1 | **记忆层** | cognee | nt_memory_kb | cognee 有持久 Agent 记忆 + 知识图 + 跨会话检索；NeoTrix 有 SQLite KB 但无 Agent 原生记忆接口 |
| 2 | **网关/Provider** | OmniRoute | GatewayV2 | OmniRoute 有 231+ providers, 50+ 免费, token 压缩 15-95%, 自动回退 + web/pwa |
| 3 | **子代理体系** | agency-agents / orca | nt_core_orch_agent | agency-agents 有 500+ skills + 完整展现层；orca 有并行 Agent ADE + desktop/mobile |
| 4 | **代码记忆** | codebase-memory-mcp | GWT + HyperCube | MCP 服务器实现 158 语言, ms 级查询, 99% token 缩减, 单静态二进制 |
| 5 | **安全** | strix | nt_shield | strix 是开源 AI 渗透测试, 自动发现漏洞 |

### 3周路线图

| 周 | 吸收目标 | Rust 改造 |
|----|---------|----------|
| W1 | 吸收 cognee + codebase-memory-mcp | nt_memory_kb → Agent Memory Interface, MCP codebase search 集成 |
| W2 | 吸收 OmniRoute + herdr | GatewayV2 → 231 providers 适配, Rust agent multiplexer 桥接 |
| W3 | 吸收 agency-agents + orca + strix | nt_core_orch_agent → 异步 + skills 可发现 + 展现层, nt_shield → 渗透检测 |

---

## 三、方向二: 架构盲点补齐 (P0-P1, 4周)

| 盲点 (源自 2026-06-30 合成) | 当前状态 | 目标 |
|---------------------------|---------|------|
| **无上下文压缩管线 (P0-5)** | GWT 上下文无限增长 → 5 层压缩已定义但未接线 | 连接上下文预算 → 裁剪 → 压缩 → 折叠 → 自动压缩 |
| **无边部署/编译管线 (P0-4)** | nt_core_deploy 存在但空壳 | 量化管线 + AOT 编译 + LoRA 适配器 + 硬件检测 |
| **无过程记忆 (P0-6)** | 只有 Episodic, E8 模式从未固化 | ProceduralMemoryStage + skill 固化管线 |
| **E8↔GWT 无梯度流 (P0-8)** | 离散 u8 → 连续 f64, 无可微分路径 | E8 嵌入为 VSA 超向量 + 可微分桥 |
| **无形式化对齐管线 (P0-3)** | RewardCalc 无学习型奖励模型 | DPOStage + GRPO 组采样 + Constit 自批判 |
| **无过程奖励模型 (P0-1)** | PRM 头 E8 有但未接线 | ProcessRewardLearner 已接线 (Cycle 12 完成) |
| **无 SAE 可解释性 (P0-2)** | SAE 模块存在但 E8/GWT 不消费 | SAE Steering + 因果归因 + 特征引导 |

### 4周路线图

| 周 | 焦点 | 模块 |
|----|------|------|
| W1 | 上下文压缩 | nt_core_gwt/context_engine.rs + compaction.rs — 5层管线激活 |
| W2 | 过程记忆 | ProceduralMemoryStage + E8 模式→skill 固化 |
| W3 | 边缘部署 + 量化 | nt_core_deploy: AWQ/GGUF, MLX 绑定, LoRA 适配器 |
| W4 | 对齐管线 | DPOStage + GRPO + ConstitutionalSelfCritiqueStage |

---

## 四、方向三: 资源池深度扩展 (P1, 持续)

| 新类别 | 候选仓库 | 同级别 |
|--------|---------|--------|
| **Agent 框架 & 记忆** | cognee, mem0, memGPT, agency-agents | P0 |
| **MCP 服务器生态** | codebase-memory-mcp, mcp-servers, modelcontextprotocol | P0 |
| **AI 网关** | OmniRoute, one-api, new-api, LiteLLM | P0 |
| **Rust AI 原生** | herdr, candle, burn, mistral.rs | P1 |
| **安全渗透** | strix, burpsuite, nuclei | P1 |
| **正式验证** | lean4, coq, why3, z3 | P1 |
| **边缘部署** | mlx, coremltools, onnxruntime, llama.cpp, GGUF | P1 |
| **对齐 & 奖励** | DPO, GRPO, Constitutional AI, RLHF | P1 |
| **视频/多模态 Agent** | video-use, OpenMontage | P2 |
| **Neuroscience** | Nengo, Brian2, SpiNNaker | P2 |

---

## 五、建议优先级

```
方向一 (竞品补齐) ───────────────────────────── ┐
  吸收 → 对比 → 改造 (W1-W3)                   │
                                                │
方向二 (架构盲点) ───────────────────────────── ┤ 并行
  上下文 → 过程记忆 → 部署 → 对齐 (W1-W4)      │
                                                │
方向三 (资源池) ───────────────────────────── ┘
  持续后台吸收 (1800s cycle)
```

### 本周建议

1. **立即吸收**: cognee (记忆竞品, 与 NeoTrix memory 最直接) + OmniRoute (网关竞品) + codebase-memory-mcp (MCP 代码记忆)
2. **开始改造**: nt_memory_kb → Agent Memory Interface (会话持久化嵌入查询)
3. **下周吸收**: agency-agents + orca + herdr + strix
