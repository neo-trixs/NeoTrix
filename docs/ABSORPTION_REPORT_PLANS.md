# NeoTrix 计划文档吸收报告

**吸收时间**: 2026-09-18 11:58
**吸收Agent**: opencode (mimo-v2.5-free)
**会话ID**: absorption_plans

---

## 吸收统计

| 指标 | 数量 |
|------|------|
| 读取文档数 | 5 |
| 提取关键洞察数 | 25 |
| 写入KB记录数 | 25 |
| 高优先级洞察 | 22 |
| 中优先级洞察 | 3 |

---

## 吸收文档清单

| # | 文档 | 类型 | 核心主题 |
|---|------|------|---------|
| 1 | `docs/2-PLANS/00-INDEX.md` | 索引 | 55份计划文档总览 |
| 2 | `docs/2-PLANS/2026-07-01-desktop-ai-chat-nextgen-design.md` | 架构设计 | 桌面AI聊天四层同心圆架构 |
| 3 | `docs/2-PLANS/2026-07-01-multi-agent-orchestration-design.md` | 架构设计 | L7多Agent图编排引擎 |
| 4 | `docs/2-PLANS/2026-07-01-nt-mind-evolve-design.md` | 进化设计 | 基准驱动进化循环 |
| 5 | `docs/2-PLANS/2026-06-30-unified-llm-proxy-gateway.md` | 网关设计 | 统一LLM Provider网关 |

---

## 关键洞察摘要

### 1. 桌面AI聊天核心设计 (8条)

#### 架构原则
- **四层同心圆架构**: L0核心引擎→L1流式UX→L2知识记忆→L3可扩展层，能力同心圆非分层
- **四条涌现定律**: 流式是体验、缓存是架构、MCP是生态、本地优先是权利

#### 技术选型
- **技术栈**: Tauri 2.x + React 19 + Tailwind v4 + Zustand + TanStack Query + Channel API
- **向量存储**: LanceDB(嵌入式) + ONNX本地嵌入 + CrossEncoder reranker
- **流式IPC**: Channel API替代Event System (50%开销减少)

#### 核心组件
- **三层缓存**: 精确匹配(SHA256) → 语义匹配(HNSW) → Prompt Prefix缓存(KV复用90%折扣)
- **CognitiveStream**: 三阶段认知感知流式(init/streaming/complete)
- **Local LLM Sidecar**: Just-in-Time llama.cpp，统一OpenAI-compatible API

---

### 2. 多Agent编排核心设计 (5条)

#### 架构组件
- **nt_cap_orch_graph**: AgentNode/AgentEdge/AgentReducer图引擎
- **GroundedGate**: 外部接地验证门(Compile/Test/Lint可执行证据)
- **AgentContract**: 类型化契约(结构化工件+数据契约+可执行反馈)

#### 设计原则
- **单agent默认**: 仅特定场景启用多agent(并行化/上下文保护/自主编排)
- **全部未实现**: nt_cap_orch_* 8模块2026-08核实零实现

---

### 3. NT-MIND进化循环核心设计 (4条)

#### 进化循环
- **五阶段循环**: Solve→Observe→Evolve→Gate→Reload
- **三级MutationScope**: Comprehensive(<30%)→Targeted(30-70%)→Minimal(>70%)
- **五种策略**: AdaptiveEvolve/GuidedSynthesis/SkillForge/Recombination/ParameterSearch

#### 安全机制
- **EGL检测**: 演化泛化损失回滚(5%下降触发)
- **收敛条件**: 5连续周期改善<0.01或max_cycles(20)

---

### 4. 统一LLM网关核心设计 (4条)

#### 问题定义
- **现状问题**: 5断开路由器、4桩流式provider、零保护机制
- **GatewayProvider**: 6层中间件管道，零改动现有10+调用点

#### 评分引擎
- **统一评分**: Score = (success²/latency) × cost^β × health^γ
- **免费Provider**: Tier1无密钥→Tier2有密钥免费→Tier3试用/社区

---

## 实施优先级建议

| 优先级 | 模块 | 工作量 | 依赖 |
|--------|------|--------|------|
| **P0** | GroundedGate + AgentContract | 3天 | 无 |
| **P0** | GatewayProvider统一网关 | 11步 | types.rs |
| **P1** | nt_cap_orch_graph图引擎 | 2天 | P0 |
| **P1** | nt_mind_evolve进化循环 | 15天 | SEAL |
| **P2** | 桌面AI Phase 0骨架 | 2周 | Tauri 2.x |

---

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 多Agent模块全部未实现 | 高 | P0从GroundedGate+AgentContract开始 |
| 进化循环依赖SEAL | 中 | PipelineAdapter解耦 |
| 免费Provider稳定性 | 中 | 三层降级策略 |
| Channel API学习曲线 | 低 | 参考Tauri官方文档 |

---

## 下一步行动

1. **立即**: 实现GroundedGate(外部接地验证门)
2. **本周**: 实现AgentContract类型化契约
3. **下周**: 实现GatewayProvider统一网关(11步)
4. **两周内**: 启动nt_mind_evolve BenchmarkSuite(阶段1)

---

**报告生成**: 2026-09-18 11:58 UTC+8
**吸收完成度**: 100%
