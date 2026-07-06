# External Absorption: 综合盲点交叉分析 (2026-07-01)

## 方法论

将 3 轮外部吸收循环发现的 **28+ 盲点**，对照 **9 层架构**逐层映射，
评估「已实现/骨架/缺失」状态，识别跨层模式和架构健康度。

---

## 1. 盲点 vs 9 层映射矩阵

```
盲点                          来源轮次  优先  影响层      状态
─────────────────────────────────────────────────────────────
MCP v3 规格不兼容                C1      P0    L1/L7      🔴 缺migrate
语义缓存层缺失                   C1      P0    L1(IO)     ✅ nt_io_cache
结构化输出约束解码               C1      P1    L1(IO)     ✅ nt_io_constrained
LLM 可观测性/追踪                C1      P1    L1-L9     ✅ nt_io_telemetry
Agent 评估框架                   C1      P1    L8         ✅ nt_mind_evolve
Agentic RAG 管线缺失             C1      P1    L3         ⚠️ 骨架(nt_memory_adaptive_rag)
多Agent编排模式缺失              C1      P1    L7         ✅ nt_act_orch_patterns
DSPy提示优化管线缺失             C1      P2    L8         🔴 未实现
GraphRAG未集成                   C1      P2    L3         ✅ nt_memory_graphrag
约束Streamable HTTP传输缺失      C1      P2    L1/L7     🔴 未实现
测试时计算缩放(TTC)              C3      P0    L4         ⚠️ 骨架(nt_core_ttc空)
执行时对齐(Unfireable Kernel)    C3      P0    L1         ✅ safety_kernel
自适应神经符号推理(NeSy)         C3      P1    L4         ✅ nt_core_adaptive_ne
优先等级宪法                     C3      P1    L8         ✅ constitutional_stage
自改进数据合成                   C3      P1    L8         ⚠️ 骨架(data_synthesis空)
Agent编排效率基准                C3      P1    L7         🔴 未实现
Apple Core AI/边缘部署升级       C3      P2    L0         ✅ nt_core_deploy
EU AI Act合规管线               C3      P2    L1/L9     ✅ nt_core_compliance
SAE升级(SASA/SoftSAE)           C3      P2    L4         ✅ nt_core_sae
ECHO终端即具身信号               C1      P0    L2/L4     🔴 未实现
ccglass代理可观测模式            C1      P1    L1         ⚠️ 部分(nt_io_telemetry)
SkillForge动态技能路由           C1      P2    L7         🔴 未实现
AERS技能库目录                   C1      P2    L7         🔴 未实现
VulnClaw黑板图模式               C2      P2    L4/L7     🔴 未实现
dLLM扩散推理路径                 C2      P2    L4         🔴 未实现 (P2优先级)
苹果神经引擎直接编程             C2      P2    L0         🔴 ANEForge模式未实现
```

**图例**: ✅ 已实现 | ⚠️ 骨架/部分 | 🔴 未实现

---

## 2. 按层健康度评估

### L0 — 基底层 (Substrate) 🟢 健康

| 盲点 | 状态 | 说明 |
|------|------|------|
| 边缘部署/量化/ANE | ✅ | nt_core_deploy 完整实现 |
| ANE直接编程(ANEForge) | 🔴 | 仅依赖CoreML路径，缺少直接ANE dispatch |

**评估**: 部署管线完整，但缺少 Apple Core AI 直接 NE 编程路径。
**建议**: P2，按需补充

---

### L1 — 身体层 (Body) 🟡 中等

| 盲点 | 状态 | 说明 |
|------|------|------|
| 语义缓存 | ✅ | nt_io_cache (三层: exact→semantic→prompt) |
| 结构化输出 | ✅ | nt_io_constrained (DFA + 自适应掩码) |
| 执行时对齐 | ✅ | SafetyKernel (HMAC签名+进程隔离+4属性) |
| EU AI Act 合规 | ✅ | nt_core_compliance (水印/风险分类/审计) |
| MCP v3 迁移 | 🔴 | 无状态HTTP/通知过滤器/Tasks扩展未实现 |
| Streamable HTTP | 🔴 | 标准负载均衡头/无粘性会话未实现 |
| ccglass代理模式 | ⚠️ | nt_io_telemetry有Tracer但无MITM代理 |

**评估**: 执行层已加固（安全内核+合规+约束解码），但 MCP 传输层严重滞后于 2026-07-28 规范。
**建议**: P0 — MCP v3 迁移是停摆风险，必须先做

---

### L2 — 感知层 (Perception) 🔴 薄弱

| 盲点 | 状态 | 说明 |
|------|------|------|
| ECHO终端即具身 | 🔴 | CLI终端输出(stdout/stderr/traces)被丢弃，未作为强化学习信号 |
| JEPA世界模型 | ✅ | nt_core_jepa / nt_core_cdwm 已实现 |

**评估**: 感知层严重缺失具身信号处理。ECHO 论文揭示了系统性盲点：CLI Agent 的终端输出是免费的密集监督信号，NeoTrix 将其当作 IO 丢弃而非学习输入。
**建议**: P0 — 从 L2 感知到 L4 推理必须增加 terminal->embedding->reward 管线

---

### L3 — 记忆层 (Memory) 🟢 健康

| 盲点 | 状态 | 说明 |
|------|------|------|
| Agentic RAG | ⚠️ | nt_memory_adaptive_rag 骨架未连线 |
| GraphRAG | ✅ | nt_memory_graphrag (Leiden社区检测+实体提取) |
| 社区检索 | ✅ | nt_memory_community (4种查询模式) |
| 置信度评分 | ✅ | nt_memory_confidence (4分量表+衰减+矛盾) |
| 智能体驱动记忆 | ✅ | nt_memory_agent_driven (3层+自编辑+合并) |

**评估**: 记忆层是最厚的层之一，6 个独立模块覆盖了检索、图、置信度、社区、嵌入。
唯一缺失：Agentic RAG 的 query-adaptive 检索尚未连线到关键路径。
**建议**: P1 — 将 adaptive_rag 的评分+重写+回退接入 KB 检索关键路径

---

### L4 — 认知层 (Cognition) 🟡 中等

| 盲点 | 状态 | 说明 |
|------|------|------|
| TTC缩放 | ⚠️ | nt_core_ttc 骨架(空文件) |
| 自适应NeSy | ✅ | nt_core_adaptive_ne (697行含LoH+beam/MCTS) |
| SAE可解释性 | ✅ | nt_core_sae + sae_bridge + saesteer |
| ECHO终端信号 | 🔴 | 无嵌入管线 |
| dLLM扩散推理 | 🔴 | P2 — 架构探索性质，优先级低 |
| VulnClaw黑板图 | 🔴 | E8 引擎缺少黑板图状态空间模式 |
| PRM过程奖励 | ✅ | nt_core_prm (HeuristicCoach + TrajectoryCollector) |

**评估**: 认知层模块密度高但集成度低。TTC 是 P0 缺口（空骨架），黑板图模式和扩散推理路径是探索阶段。
**建议**: P0 — 先填 TTC 骨架；P2 探索黑板图

---

### L5 — 意识层 (Consciousness) 🟡 中等

| 盲点 | 状态 | 说明 |
|------|------|------|
| GWT压缩管线 | ✅ | nt_core_gwt 含 thinking_budget (5层压缩) |
| 谐振器网络 | ✅ | nt_core_gwt/resonance (共振矩阵) |
| 注意力自我模型 | ✅ | nt_core_self (AttentionHead/AttentionProfile) |

**评估**: 意识层也较充实，GWT + 压缩 + 共振已实现。
**建议**: 无紧急缺失

---

### L6 — 自我层 (Self) 🟢 健康

| 盲点 | 状态 | 说明 |
|------|------|------|
| 自我模型 | ✅ | nt_core_self (SiliconSelfModel + 12+子模块) |
| 元认知评估器 | ✅ | CognitiveEvaluator + 健康报告 |
| 思维追踪 | ✅ | ThinkingTrace + 反思分级 |

**评估**: 自我层是工程最完整的层之一，12+子模块全部实现。
**建议**: 无紧急缺失

---

### L7 — 能力层 (Capability) 🟡 中等

| 盲点 | 状态 | 说明 |
|------|------|------|
| 多Agent编排 | ✅ | nt_act_orch_patterns (3模式+Factory) |
| A2A协议 | ✅ | a2a.rs (AgentCard + 任务生命周期) |
| MCP v3迁移 | 🔴 | 阻塞 — 传输层未重写 |
| SkillForge路由 | 🔴 | 动态技能发现/创建未实现 |
| AERS目录 | 🔴 | 无23K技能库路由器 |
| 编排效率基准 | 🔴 | 无法保证编排性能 |
| Streamable HTTP | 🔴 | 传输层缺失 |
| 黑板图路由 | 🔴 | VulnClaw模式未整合到调制 |

**评估**: L7 是「骨架集成缺失」模式最严重的层。大量模块(agent团队、A2A、Scheduler)存在但未连线到关键路径。MCP v3迁移是最紧迫的停摆风险。
**建议**: P0 — MCP v3 迁移；P1 — SkillForge 技能路由 + 编排基准

---

### L8 — 自主神经层 (Autonomic) 🟢 健康

| 盲点 | 状态 | 说明 |
|------|------|------|
| SEAL进化管线 | ✅ | 27阶段管线(双迭代) |
| 优先宪法 | ✅ | constitutional_stage (4层优先级体系) |
| 基准进化 | ✅ | nt_mind_evolve (BenchmarkSuite + EGL + 多维评估) |
| 自改进数据合成 | ⚠️ | data_synthesis 骨架(空文件) |
| DSPy提示优化 | 🔴 | 无度量驱动编译器 |
| Agent评估框架 | ✅ | MultiDimResult(6维) + ContaminationDetector + DynamicBenchmark |

**评估**: L8 是进化层，SEAL 管线深度可观测，但自改进数据合成和 DSPy 式提示优化缺失。
**建议**: P1 — 填充数据合成骨架；P2 — DSPy 风格提示编译

---

### L9 — 超验层 (Transcendent) 🟢 健康

| 盲点 | 状态 | 说明 |
|------|------|------|
| 元观察器 | ✅ | TurkeyScientist + Observer |
| 代码扫描器 | ✅ | CodeScanner + WeaknessAnalyzer |
| 元认知循环 | ✅ | MetaCognitiveLoop |
| EU AI Act合规 | ✅ | nt_core_compliance (风险分类+水印+审计日志) |

**评估**: 超验层完整。
**建议**: 无紧急缺失

---

## 3. 发现的架构模式

### 模式 1: 「骨架集成缺失」— 关键路径断裂

最致命的跨层模式。NeoTrix 擅长构建独立组件，但集成到关键路径的工作不足。

```
critical_path = engine_core::reason() → GatewayV2::complete_with_selection() → Provider::complete()
```

受影响的模块:
- `nt_io_telemetry` → 有 Tracer trait，零消费者连到关键路径
- `nt_memory_adaptive_rag` → 检索评分/重写/回退未连到 KB 查询
- `nt_core_ttc` → 骨架空文件，引擎未接入 PRM 或 E8
- `data_synthesis` → 骨架空文件，SEAL 管线没有调用
- `nt_cap_orch_patterns` → 有 3 种编排模式，但 engine_core 仍用单一 Agent

### 模式 2: 「行在先知后」— 代码先于架构声明

部分模块在 AGENTS.md 中已注册但无实现，部分实现了但无注册。
这是 7-domain→9-layer 迁移遗留问题。

### 模式 3: 「壳厚核薄」— 边缘层厚，中间层薄

L0/L3/L6/L8/L9 较厚(每层 5-12+ 模块)，L2 和 L5 较薄(每层 1-3 模块)。
L2 感知层是最薄弱的环节(只有 JEPA，没有终端具身感知)。

### 模式 4: 「外部压力驱动」— 吸收循环很有效但单次

3 轮外部吸收循环发现了所有重大盲点，但缺乏持续扫描机制。
所有盲点发现后没有自动追踪器。

---

## 4. 推荐行动路线

```
紧急度  行动                          影响层  努力度  预计影响
────────────────────────────────────────────────────────────
P0      MCP v3 传输迁移                 L1/L7   中      MCP服务器停摆风险
P0      填TTC引擎骨架                   L4      中      推理能力的关键突破
P0      ECHO终端信号管线(L2→L4)        L2/L4   高      CLI学习信号密度×100
P1      连线telemetry到关键路径          L1-L9   低      调试可见性
P1      连线adaptive_rag到KB          L3      中      检索准确率提升
P1      填数据合成骨架                   L8      低      进化能力升级
P1      宪法漏洞修复(价值电位)           L8      低      安全对齐质量提升
P1      Agent编排基准                    L7      低      性能可见性
P2      SkillForge技能路由               L7      中      动态能力发现
P2      DSPy式提示优化                   L8      中      提示质量提升
P2      ANE直接编程路径                  L0      低      边缘性能×2
P2      黑板图模式整合到E8              L4/L7   高      推理多样性
```

**第一阶段 (P0, 1周)**:
1. MCP v3 传输迁移 — 无状态 HTTP + 通知过滤器 + CacheableResult
2. TTC 引擎 — 基于已存在的 nt_core_ttc 骨架（今天已写代码）
3. ECHO 探索 — 设计 terminal→embedding 管线方案

**第二阶段 (P1, 2周)**:
4. Telemetry 连线 — 插入 GatewayV2.complete_with_selection()
5. Adaptive RAG 连线 — KB search() 调 adaptive_rag
6. 数据合成填充 — 基于已写的 data_synthesis 模块
7. 编排基准 — nt_cap_orch_graph 性能测试

**第三阶段 (P2, 4周)**:
8-12. SkillForge + DSPy + ANE + 黑板图

---

## 5. 未探索的高价值方向

这些是吸收循环发现的但超出当前架构范围的方向:
- **Diffusion LLM**: LLaDA/MDLM → 非自回归推理路径（P2）
- **Apple Core AI**: CoreML替代品, Python工具链, AOT编译（P2）
- **EU AI Act 2026-08**: 水印义务, 透明度规则, 风险分类（已实现合规骨架但未运营化）
- **Constitutional Classifiers++**: 级联分类器, 0.05%拒绝率, 红队无突破
- **正式化验证**: Kani + Verus 验证安全内核属性

---

*生成日期: 2026-07-01*
*来源: 3轮外部吸收循环(30+领域, 40+来源)*
