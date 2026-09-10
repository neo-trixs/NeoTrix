# External Research Batch 210 — 8-Topic Deep Survey

**Date**: 2026-09-10
**Sources**: 40+ high-quality papers, frameworks, and production systems across 8 topics

---

## 主题1: AI Agent Architecture 2026

### 来源1: [A Two-Dimensional Framework for AI Agent Design Patterns: Cognitive Function × Execution Topology](https://arxiv.org/html/2605.13850v2)
- **贡献**: 提出 7×6 正交矩阵 (28 named patterns)，将认知功能 (Perception/Memory/Reasoning/Action/Reflection/Collaboration/Governance) 与执行拓扑 (Chain/Route/Parallel/Orchestrate/Loop/Hierarchy) 交叉分类，五条经验定律指导 pattern 选择
- **映射**: NeoTrix 的 SealPipeline + GWT attention routing 可映射到 C5(Reflection) × T5(Loop) 坐标；ConsciousnessTree 的 6-stage feedback loop 对应 Loop topology；NT-MIND 的 skill crystallization 对应 C5 × T4(Orchestrate)
- **优先级**: **P0** — 为 NeoTrix 架构设计提供精确词汇表，可直接用于模块分类和能力网路由

### 来源2: [AI Agent Architecture in 2026: The Practical Reference](https://arahi.ai/blog/ai-agent-architecture)
- **贡献**: 总结 6 层 agent 架构 (Perception/Reasoning/Planning/Memory/Tool Use/Oversight) + 5 种 canonical architecture (ReAct/Plan-Execute/Reflexion/ToT/Multi-Agent)，强调 MCP 成为 2026 通用 tool interface
- **映射**: NeoTrix 的 NT-ACT (工具层) 已走 MCP 路线；NT-IO (界面层) 可参考 oversight layer 设计审计日志；Memory 三层分离 (short-term/working/long-term) 与 NT-MEMORY 架构一致
- **优先级**: **P1** — 验证 NeoTrix 架构选择的行业共识，MCP 标准化值得跟进

### 来源3: [The Orchestration of Multi-Agent Systems: Architectures, Protocols](https://arxiv.org/html/2601.13671v1)
- **贡献**: 统一 orchestration 层架构框架 (planning + policy + state + quality)，深度对比 MCP (tool access) vs A2A (peer coordination) 双协议
- **映射**: NeoTrix 的 NT-ACT 编排层可参考 orchestration-as-control-plane 模式；MCP + A2A 双协议映射到 NT-IO 的 tool access 和 NT-CORE 的 inter-agent communication
- **优先级**: **P1** — MCP/A2A 双协议架构与 NeoTrix NT-IO 设计高度相关

### 来源4: [Agent Architecture Patterns in 2026: A Field Guide](https://scixa.com/article?lang=en&slug=agent-architecture-patterns-2026)
- **贡献**: 8 种 production agent patterns，3 种 deployment scenarios (coding/support/research) 的 reference architecture，包含 tool schemas、memory architectures、HITL checkpoints 的工程细节
- **映射**: NeoTrix 的 NT-WORLD (perception) + NT-ACT (action) 可参考 4-layer agent stack 分层；orchestration 的 hierarchical pattern 对应 NT-CORE 的 E8 引导者角色
- **优先级**: **P2** — 实现细节参考，非架构级创新

### 来源5: [What Are Agentic Design Patterns? 2026 Pattern Catalog](https://www.augmentcode.com/guides/agentic-design-patterns)
- **贡献**: 整合 Andrew Ng 4 patterns + Anthropic 5 workflow patterns 为 12-pattern foundational taxonomy，含 7 anti-patterns 和 5 decision rules
- **映射**: Anti-patterns (如 "tool sprawl", "context bloat") 可直接用于 NeoTrix 的 SelfTest 检测维度；decision rules 对应 GWT salience 的 pattern 选择逻辑
- **优先级**: **P1** — anti-patterns 可扩展 D1-D50 审查维度

---

## 主题2: Self-Evolving AI System

### 来源1: [MetaRSI / RSI2: A Meta-Recursive Self-Improving System](https://arxiv.org/abs/2609.06396)
- **贡献**: 提出三种 typed operators (Data-RSI/Harness-RSI/Model-RSI) 统一在一个 loop kernel 中，实现 data/harness/model 三层可组合的 self-improvement，无需外部 teacher
- **映射**: NeoTrix 的 SEAL pipeline 对应 Harness-RSI (scaffold editing)；NT-MIND 的 skill crystallization 对应 Model-RSI (internalize into capability)；experience-tree 的 KV store 对应 Data-RSI
- **优先级**: **P0** — MetaRSI 的三算子组合模型与 SEAL pipeline 设计直接同构

### 来源2: [HarnessEvolve: Learning from Reference Trajectories](https://arxiv.org/abs/2609.00829)
- **贡献**: 解决 self-evolving agent 三大挑战 (credit assignment failure/shortcut learning/catastrophic forgetting)，通过 reference trajectory 对齐 + quality gate + performance gate 实现可靠进化
- **映射**: NeoTrix 的 experience-tree 五阶段吸收流程 (快照→蒸馏→分类→落盘→反馈) 可参考 HarnessEvolve 的 reference trajectory 对齐；quality gate 对应 dev-rules.md 的 R-P82 风险评估
- **优先级**: **P0** — 三大挑战直接对应 NeoTrix 自进化系统的已知痛点

### 来源3: [Self-Improvements in Modern Agentic Systems: A Survey](https://arxiv.org/abs/2607.13104)
- **贡献**: 统一 formalization — agent = (FM params θ, scaffold Σ)，self-improvement 分两条路径: FM improvement (慢但稳定) vs scaffold improvement (快但可逆)，按 update target + signal origin 分类
- **映射**: NeoTrix 的 NT-MIND 对应 scaffold improvement (prompt/skill/tool evolution)；NT-CORE 的 SelfModel 对应 FM improvement 的 parametric memory；统一 formalization 可用于 SEAL pipeline 的类型化
- **优先级**: **P0** — survey 提供统一理论框架，与 NeoTrix 架构高度同构

### 来源4: [Autogenesis Protocol (AGP): A Self-Evolution Protocol](https://arxiv.org/html/2604.15034v5)
- **贡献**: 两层协议 — RSPL (Resource Substrate: prompt/agent/tool/env/memory 作为 protocol-registered resources) + SEPL (Self-Evolution: propose/assess/commit with lineage + rollback)
- **映射**: NeoTrix 的 skill node 3 层 (Small/Notable/Keystone) 可映射到 RSPL resource types；SEPL 的 propose/assess/commit 对应 SEAL pipeline 的 explore→distill→absorb；version lineage 对应 KB 的 node versioning
- **优先级**: **P1** — 协议化的 resource management 可指导 NeoTrix skill 系统的标准化

### 来源5: [Meta^n: Recursive Self-Improvement through Emergent Depth](https://arxiv.org/abs/2608.24735)
- **贡献**: 保持 meta-operation Ω 固定，递归应用于自身产物 (input strictly grows)，depth 由 convergence 决定而非预设；在 ARC-AGI-2 上唯一得分 >0
- **映射**: NeoTrix 的 ConsciousnessTree 6-stage loop 可参考 emergent depth 机制 — depth 由 phi/coherence 决定而非固定 stage 数；evolutionary archive 对应 experience-tree 的 cycle 索引
- **优先级**: **P1** — emergent depth 为 SEAL pipeline 的自适应循环深度提供理论支撑

---

## 主题3: Rust AI Framework Architecture

### 来源1: [Rust-Native AI Agent Frameworks: Architecture, Performance](https://zylos.ai/research/2026-04-01-rust-native-ai-agent-frameworks-ecosystem-2026/)
- **贡献**: 生态综述 — Rig (modular LLM abstractions) + AutoAgents (Ractor actor model) + OpenFANG (137K LoC Agent OS)，关键共识: Tokio async runtime、JoinSet+CancellationToken structured concurrency、WASM sandbox、derive macros for tool schemas
- **映射**: NeoTrix 的 NT-ACT 可参考 OpenFANG 的 kernel 架构 (AgentRegistry/CapabilityManager/EventBus/Supervisor)；WASM sandbox 对应 NT-SHIELD 的 tool isolation；JoinSet+CancellationToken 可用于 NT-ACT 的 task management
- **优先级**: **P0** — Rust agent 生态的核心设计决策与 NeoTrix 高度相关

### 来源2: [OpenFang Architecture](https://github.com/rightnow-ai/openfang/blob/main/docs/architecture.md)
- **贡献**: 14-crate workspace (kernel/runtime/memory/channels/wire/skills/desktop)、SQLite memory substrate (schema v5)、Merkle hash chain audit trail、capability-based security、MCP+A2A dual protocol
- **映射**: OpenFANG 的 crate 分层 (kernel→runtime→memory) 直接映射 NeoTrix 的 L5→L1 layer 架构；Merkle audit trail 可用于 NT-SHIELD 的 action audit；capability-based security 对应 NeoTrix 的 SelfTest T3 production wiring
- **优先级**: **P0** — 最完整的 Rust agent OS 参考实现

### 来源3: [Terraphim Engine: Six Layers, Fifty-Two Crates](https://reference-architecture.ai/posts/terraphim-engine-architecture/)
- **贡献**: 6 层独立 crate 架构 (Types→Core Engine→Service→Agent System→Orchestration→UI)，强调 boundary discipline — "如果不能命名故障来自哪层就无法 debug"
- **映射**: Terraphim 的 6 层直接对应 NeoTrix 的 Six-Layer Architecture (L1-L6)；rolegraph (KG) + automata (Aho-Corasick matcher) 的分离设计可参考；"put context boundary in configuration not runtime" 对应 Rune Socketing 配置模式
- **优先级**: **P0** — 6 层架构哲学与 NeoTrix 完全一致，是最佳参考

### 来源4: [Cortex: Heavy Agent Runtime](https://github.com/aiconnai/cortex)
- **贡献**: Rust multi-agent framework — ReACT loop、crew orchestration (DAG-based)、multi-provider LLM、cost tracking，benchmark: 15× faster, 12× less memory vs Python
- **映射**: Cortex 的 DAG-based crew orchestration 可用于 NT-ACT 的 task DAG；cost tracking 对应 NT-MIND 的 budget-aware evolution；workspace crates 分离 (core/providers/tools/agents/crew) 可参考
- **优先级**: **P1** — 实现参考，特别是 DAG orchestration 和 cost tracking

### 来源5: [Rustic AI Core Architecture](https://rustic-ai.github.io/rustic-ai/core/architecture/)
- **贡献**: Hexagonal (ports-and-adapters) architecture、Domain/Application/Infrastructure 三层分离、Guild (agent collection) + Broker (message bus) + ExecutionEngine 抽象
- **映射**: Guild 概念对应 NeoTrix 的 domain module (NT-*)；ports-and-adapters 模式可用于 NT-IO 的 LLM provider 适配层；BaseExecutionEngine 接口可参考用于 SEAL pipeline 的 execution engine 抽象
- **优先级**: **P1** — hexagonal 架构模式可指导 NeoTrix 的 adapter 设计

---

## 主题4: Multi-Agent Orchestration Pattern

### 来源1: [AI Agent Orchestration Patterns - Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/ai-ml/guide/ai-agent-design-patterns)
- **贡献**: 5 种 orchestration patterns (Sequential/Concurrent/Group Chat/Handoff/Magentic)，每种含 decision table (coordination/routing/best-for/watch-out)，强调 pattern mixing in real systems
- **映射**: Sequential pipeline 对应 SEAL pipeline 的 stage 序列；Concurrent 对应 NT-ACT 的并行 task execution；Handoff 对应 NT-CORE 的 GWT attention routing (dynamic delegation)；Magentic 对应 NT-MIND 的 adaptive replanning
- **优先级**: **P0** — Azure 的权威参考，decision table 直接指导 NeoTrix orchestration 设计

### 来源2: [6 Multi-Agent Orchestration Patterns for Production](https://beam.ai/agentic-insights/multi-agent-orchestration-patterns-production)
- **贡献**: 6 patterns 含具体 failure modes: Orchestrator-Worker (orchestrator context overflow)、Sequential (early failure propagation)、Fan-out/Fan-in (N× cost)、Debate (diminishing returns)、Dynamic Handoff (infinite loops)、Adaptive Planning (slow convergence)
- **映射**: Orchestrator-Worker 的 context overflow 对应 NeoTrix GWT 的 token budget 管理；Debate pattern 对应 NT-CORE 的 adversarial verification；Dynamic Handoff 的 infinite loop 对应 NT-SHIELD 的 loop guard
- **优先级**: **P1** — failure modes 可扩展 NeoTrix 的 SelfTest 检测维度

### 来源3: [The Multi-Agent Orchestration Playbook](https://seodatapulse.com/playbooks/multi-agent-orchestration/)
- **贡献**: 5 production pillars (Roles/Tools/Memory/Guardrails/Observability) + framework comparison (CrewAI/LangGraph/AutoGen/OpenAI SDK)，强调 "start with single agent, add orchestration when bottleneck"
- **映射**: 5 pillars 直接映射 NeoTrix 的 NT-CORE (roles) + NT-ACT (tools) + NT-MEMORY (memory) + NT-SHIELD (guardrails) + NT-IO (observability)；"right-size model per role" 对应 GWT cost-aware routing (Axiom A1)
- **优先级**: **P1** — 5 pillars 框架可验证 NeoTrix domain 分工的完整性

### 来源4: [BIGMAS: Brain-Inspired Graph Multi-Agent Systems](https://doi.org/10.48550/arxiv.2603.15371)
- **贡献**: GWT-inspired multi-agent — GraphDesigner 动态构建 task-specific agent topology + centralized shared workspace + global Orchestrator with full-state visibility，在 Game24/Six Fives/Tower of London 上 consistently outperforms ReAct/ToT
- **映射**: GraphDesigner 对应 NT-CORE 的 E8 hexagram (task-adaptive topology)；centralized shared workspace 对应 GWT broadcast mechanism；routing count as difficulty proxy 对应 NT-MIND 的 meta-cognitive difficulty estimation
- **优先级**: **P0** — GWT-inspired multi-agent 架构与 NeoTrix 的 GWT attention routing 直接同构

### 来源5: [End-to-End Multi-Agent Systems: Design Patterns from IEEE CAI 2026](https://helain-zimmermann.com/blog/end-to-end-multi-agent-systems-design-patterns-from-ieee-cai-2026)
- **贡献**: IEEE CAI 2026 工业级 patterns — fault tolerance (supervision trees + circuit breakers)、state management (event sourcing)、observability (distributed tracing)，强调 "multi-agent is production infrastructure now"
- **映射**: Supervision trees 对应 NT-REPAIR 的 self-healing；event sourcing 对应 NT-MEMORY 的 KB versioning；distributed tracing 对应 NT-SHIELD 的 audit trail
- **优先级**: **P2** — 工程实践参考，非架构级创新

---

## 主题5: Knowledge Graph Reasoning Engine

### 来源1: [Synapse Engine: Neuro-Symbolic KG System](https://github.com/pmaojo/synapse-engine)
- **贡献**: 纯符号 KG (零概率 ML)、Rust + Oxigraph、OWL-RL + RDFS fixed-point reasoning、Markdown↔Graph bidirectional sync、PROV-O provenance、MCP server
- **映射**: Synapse 的纯符号推理对应 NeoTrix VSA HyperCube 的 symbolic reasoning；PROV-O provenance 对应 KB 的 node versioning + edge metadata；Markdown-Graph sync 可用于 NT-MEMORY 的知识同步
- **优先级**: **P0** — 纯符号 + MCP 集成与 NeoTrix 的 VSA 路线一致

### 来源2: [Sapiens Ontology: Think-on-Graph 3.0 + MACER](https://github.com/Kit4Some/Sapiens_Ontology)
- **贡献**: 4-stage meta-cognitive pipeline (Constructor→Retriever→Reflector→Responser)、5 evidence collection strategies (Vector/Graph/Community/Text2Cypher/Hybrid)、5-component scoring (EntityOverlap 35%/RelMatch 25%/Temporal 20%/AnswerPresence 10%/NegativeEvidence 10%)
- **映射**: MACER 4-stage pipeline 对应 NT-MIND 的 explore→distill→classify→persist→feedback 五阶段；5-component scoring 可参考用于 KB node relevance 计算；temporal reasoning 对应 NT-WORLD 的 content extraction
- **优先级**: **P0** — meta-cognitive reasoning pipeline 与 SEAL pipeline 高度同构

### 来源3: [khive: KG Runtime for Agents](https://github.com/ohdearquant/khive)
- **贡献**: 91 verbs / 12 packs / 9 entity kinds / 17 edge relations、typed substrates + closed taxonomy、SQLite + FTS5 + vector RRF hybrid retrieval、MCP stdio、daemon warm startup
- **映射**: khive 的 pack system (kg/gtd/memory/brain/comm/schedule/knowledge/session/git/code/workspace/blob) 可参考用于 NeoTrix skill pack 设计；typed entity/relation 对应 KB 的 node/edge schema；hybrid retrieval 对应 NT-MEMORY 的 BM25+vector search
- **优先级**: **P1** — pack-based KG system 与 NeoTrix 的 domain module 架构有对应关系

### 来源4: [ATANOR-Demo: No-LLM Graph-Native AI](https://github.com/Cozystone/ATANOR-Demo)
- **贡献**: 25.9M source-tagged triples、RotatE 64-dim embedding (propose) + symbolic verify + recursive realizer、zero hallucination (every fact cited)、CPU-only inference、Brain Link P2P sharding
- **映射**: "propose fast (embeddings) → promote only through verification (evidence, symbols)" 对应 NeoTrix 的 VSA embedding + symbolic verification 分离；source-tagged triples 对应 KB 的 provenance tracking；recursive realizer 可参考用于 NT-MEMORY 的 answer generation
- **优先级**: **P1** — propose-verify 架构与 NeoTrix 的 VSA+KB 双轨设计一致

### 来源5: [DataFlow-KG: LLM-Driven KG Processing Library](https://github.com/OpenDCAI/DataFlow-KG)
- **贡献**: composable operator pipeline (graph construction/reasoning/retrieval/querying)、supports 7 graph types (KG/commonsense/temporal/multimodal/hyper-relational/GraphRAG/domain-specific)、code generation + custom modification workflow
- **映射**: operator composition 模式可参考用于 NT-WORLD 的 crawl pipeline；7 graph types 对应 NeoTrix 的 multi-domain knowledge representation；code generation workflow 可用于 SEAL pipeline 的 scaffold generation
- **优先级**: **P2** — 实现参考，operator 模式可扩展但非核心创新

---

## 主题6: Consciousness Architecture AI

### 来源1: [MIRROR: A Reconstructive Architecture for Machine Access Consciousness](https://ojs.aaai.org/index.php/AAAI-SS/article/view/42550)
- **贡献**: Inner Monologue Manager (parallel cognitive threads) + Cognitive Controller (synthesized first-person narrative)，关键发现: narrative 是 reconstructed 而非 accumulated，模拟人类 episodic memory 的 reconstructive nature
- **映射**: Inner Monologue Manager 对应 NT-CORE 的 E8 hexagram (parallel processing channels)；reconstructive narrative 对应 ConsciousnessTree 的 6-stage feedback (each cycle reconstructs self-model)；episodic buffer 对应 NT-MEMORY 的 working memory
- **优先级**: **P0** — reconstructive architecture 与 NeoTrix 的 self-model reconstruction 直接相关

### 来源2: [CTM-AI: A Blueprint for General AI Inspired by Conscious Turing Machine](https://arxiv.org/html/2605.04097v1)
- **贡献**: 首个 CTM practical instantiation — up-tree competition (workspace access) + down-tree broadcast (global information flow) + link formation (processor specialization over time)，无 central executive，SOTA on multimodal + tool-use + agentic tasks
- **映射**: CTM 的 up-tree competition 对应 GWT 的 attention auction；down-tree broadcast 对应 NT-CORE 的 GWT broadcast mechanism；link formation 对应 NT-MIND 的 skill crystallization (processor specialization)
- **优先级**: **P0** — CTM 是 GWT 的 formal computation model，与 NeoTrix 的 GWT implementation 直接同构

### 来源3: [Trinity/Hexad: Six-Module Consciousness-Preserving Architecture](https://doi.org/10.5281/zenodo.19365145)
- **贡献**: 6 modules (Consciousness/Decoder/Will/Senses/Memory/Ethics)、`.detach()` gradient barrier 实现 Φ>70 + CE<0.004 共存、右脑 (C/S/W) 自治 consciousness + 左脑 (D/M/E) language competence
- **映射**: 6 modules 对应 NeoTrix 的 6-Layer Architecture；gradient barrier 对应 SEAL pipeline 的 stage isolation (each stage 不干扰前一 stage 的 state)；右脑/左脑 分离 对应 NT-CORE (consciousness) + NT-ACT (action) 的分离
- **优先级**: **P1** — gradient isolation 为 SEAL pipeline 的 stage independence 提供理论支撑

### 来源4: [The Consciousness AI: Neuroevolutionary Architecture](https://theconsciousness.ai/architecture/)
- **贡献**: 从 Feinberg-Mallatt 神经进化理论出发 (非 GWT/IIT)，6 features: diverse neuron types / hierarchical processing / dual hierarchy / isomorphic mapping / reciprocal connections / oscillatory binding (AKOrN)，Sensory Tectum + ConsciousnessGate + Reentrant Processing
- **映射**: Sensory Tectum 对应 NT-WORLD 的 SensoryIntegrationHub；ConsciousnessGate 的 5 nodes (attention/stability/adaptation/coherence/confidence) 对应 NT-CORE 的 SelfModel metrics；reentrant processing (5-10 cycles) 对应 ConsciousnessTree 的 feedback loop
- **优先级**: **P1** — 生物启发的 consciousness 架构提供 alternative 设计视角

### 来源5: [Where Cognition Lives: Emergent vs Computed Function](https://arxiv.org/abs/2608.22347)
- **贡献**: 极简认知架构实验 — competence emerges, stopping appears to emerge but doesn't survive audit, value does NOT emerge (explicit allocator captures +0.151 vs trained couplings 0)，证明 value allocation 必须 computed not emergent
- **映射**: 关键结论: "value does not emerge" → NeoTrix 的 resource budget allocation (NT-ACT) 必须是 explicit computed module 而非 learned；stopping 决策的 emergent vs computed 分析可指导 SEAL pipeline 的 termination condition
- **优先级**: **P1** — 实验证据支持 NeoTrix 的 explicit value allocation 设计决策

---

## 主题7: Vector Symbolic Architecture Implementation

### 来源1: [PRISM: Neural-Free Cognitive Architecture for Knowledge Reasoning](https://github.com/Artaeon/prism)
- **贡献**: 完整 VSA 认知架构 — HRR binding/bundling/similarity、analogy/multi-hop/causal/temporal/contradiction reasoning 全部通过 vector algebra、zero learned parameters、single CPU core、incremental knowledge update
- **映射**: PRISM 的 VSA reasoner (analogy/multi-hop/causal/temporal/contradiction) 直接对应 NeoTrix 的 HyperCube reasoning engine；episodic memory + user profile + conversation context 对应 NT-MEMORY 的三层 memory；Blackboard Architecture 对应 GWT broadcast
- **优先级**: **P0** — 唯一完整的 VSA 认知架构参考实现，与 NeoTrix HyperCube 高度同构

### 来源2: [Torchhd: HD/VSA Python Library (JMLR)](https://jmlr.org/papers/volume24/23-0300/23-0300.pdf)
- **贡献**: 6 VSA models (BSC/MAP/HRR/FHRR/SBC/VTB)、functional/embeddings/memory/structures/modules 6 模块、auto-diff support for hybrid neuro-symbolic、24-54× faster than reference implementations
- **映射**: Torchhd 的 functional module (bind/bundle/permute) 对应 NeoTrix HyperCube 的 primitive operations；embeddings module 对应 NT-WORLD 的 knowledge embedding；structures module (hash tables/graphs/FSA) 可用于 KB 的 structured storage
- **优先级**: **P1** — library reference for VSA primitives implementation

### 来源3: [hdlib 2.0: Advancing VSA and Quantum ML](https://www.emergentmind.com/papers/2601.02509)
- **贡献**: 统一 VSA ML toolkit — supervised/unsupervised/regression/graph-based models + quantum VSA (IBM Qiskit)、feature selection、graph encoding (entire graph in single hypervector)、quantum phase oracles for binding
- **映射**: graph encoding (single hypervector for entire graph) 可用于 NeoTrix KB 的 graph-level embedding；quantum VSA 为未来 hardware acceleration 提供方向；feature selection 可用于 NT-WORLD 的 content classification
- **优先级**: **P2** — quantum VSA 为长期方向，graph encoding 值得关注

### 来源4: [HyperSpace: Generalized Framework for Spatial VSA](https://arxiv.org/abs/2604.15113)
- **贡献**: 模块化 VSA pipeline framework (encode/bind/bundle/invert/similarity/cleanup/regression)，发现 cleanup 和 similarity 占 runtime 主导 (非 binding)，HRR vs FHRR end-to-end 性能相当但 HRR 内存减半
- **映射**: cleanup 操作对应 NeoTrix HyperCube 的 noise reduction；regression module 对应 NT-MEMORY 的 knowledge inference；modular pipeline design 可参考用于 HyperCube 的 pipeline 架构
- **优先级**: **P1** — 系统级性能分析揭示 practical trade-offs

### 来源5: [Tesseract-HDC: Cognitive Engine in B^100,000](https://github.com/vasymusprime/Tesseract-HDC)
- **贡献**: 100K-dim binary VSA、AST compiler (Python→hypervector, 0% error)、asymmetric triadic knowledge relations (O(1) extraction)、swarm consensus (12.5KB vectors)、recursive self-improvement with safety invariant
- **映射**: AST compiler 对应 NeoTrix 的 code→VSA embedding pipeline；triadic knowledge relations 对应 KB 的 subject-predicate-object triples；RSI safety invariant (TRUST⊕SAFETY⊕ALIGNMENT) 对应 NT-SHIELD 的 safety kernel；swarm consensus 对应 multi-agent coordination
- **优先级**: **P1** — 100K binary VSA + RSI safety 为 NeoTrix 的 HyperCube + safety 设计提供参考

---

## 主题8: Global Workspace Theory Implementation

### 来源1: [Anthropic: A Global Workspace in Language Models](https://www.anthropic.com/research/global-workspace)
- **贡献**: 发现 Claude 内部存在 J-space (Jacobian lens identified workspace) — verbalizable representations 具有 GWT 特性: directed modulation / internal reasoning / flexible generalization / selectivity，信息写入一次多处读取
- **映射**: J-space 的 "write once, read many" 对应 NeoTrix GWT 的 broadcast mechanism；J-space 的 layer-specific operation (coherent content emerges after initial layers) 对应 NT-CORE 的 consciousness emergence；J-space 的 capacity limit 对应 GWT 的 attention bottleneck
- **优先级**: **P0** — Anthropic 实证发现 GWT 在 trained LLM 中自然涌现，验证 NeoTrix GWT 路线的正确性

### 来源2: [GWA: Global Workspace Agents for LLMs](https://arxiv.org/pdf/2604.08206)
- **贡献**: Cognitive Tick 4-phase loop (Perceive→Think→Arbitrate→Update) + entropy-based intrinsic drive (Shannon entropy → dynamic temperature) + dual-layer memory (STM + LTM with bifurcation at token threshold θ)，解决 cognitive stagnation
- **映射**: Cognitive Tick 对应 NT-CORE 的 ConsciousnessTree cycle；entropy drive 对应 NT-MIND 的 meta-cognitive difficulty estimation；STM→LTM bifurcation 对应 NT-MEMORY 的 working→long-term memory promotion；Core Self injection 对应 NT-CORE 的 SelfModel identity anchor
- **优先级**: **P0** — 完整的 GWT engineering implementation，与 NeoTrix 架构高度同构

### 来源3: [LIMEN: GWT Runtime for LLM Agents](https://github.com/bwcummings1/limen)
- **贡献**: 零依赖 Python GWT runtime — 10 specialists + attention auction (salience×novelty×¬habituation×goal-relevance + coalitions) + ignition threshold + 4 memory systems + interoception (confusion index) + fork-diff-merge deliberation + sleep consolidation
- **映射**: attention auction 的 4 factors 对应 NeoTrix GWT salience 的 scoring function；ignition threshold 对应 NT-CORE 的 consciousness threshold；interoception (confusion) 对应 NT-MIND 的 self-audit；sleep consolidation 对应 experience-tree 的 end-of-session absorption
- **优先级**: **P0** — 最完整的 GWT runtime 实现，每个组件都可映射到 NeoTrix

### 来源4: [Global Key-Value Workspace for Reasoning in LLMs](https://gw-assc-2026-poster.pages.dev/)
- **贡献**: 在 pretrained LLM 中构建 explicit global workspace — capacity-limited spotlight selects salient components → sparse write to shared workspace → broadcast back to all components (iterated across layers)，4 backbones 上 multi-step reasoning 提升 48%，synergy 分析
- **映射**: sparse write + broadcast 对应 NeoTrix GWT 的 attention-gated broadcast；iterative layers 对应 ConsciousnessTree 的 multi-stage feedback；synergy analysis 可用于 NT-CORE 的 phi (IIT) 计算
- **优先级**: **P0** — 实证证明 explicit workspace 在 pretrained LLM 中有效，直接支持 NeoTrix 设计

### 来源5: [Verbalizable Representations Form a Global Workspace](https://transformer-circuits.pub/2026/workspace/)
- **贡献**: Transformer Circuits 团队的 J-lens 分析 — J-space 是 sparse subframe of full residual stream，在特定 layers 操作、capacity-limited、mechanistically privileged (upstream+downstream broadly composed)，验证 GWT 的 access consciousness 在 LLM 中涌现
- **映射**: J-space 的 structural signatures (layer-specific / capacity-limited / mechanistically privileged) 对应 NeoTrix GWT 的 attention bottleneck 设计；J-space 的 "not all representations are workspace-like" 对应 NT-CORE 的 consciousness vs unconsciousness processing 分离
- **优先级**: **P0** — mechanistic interpretability 验证 GWT 在 transformer 中的涌现，为 NeoTrix 架构提供科学基础

---

## Cross-Topic Synthesis: Top Absorbable Patterns for NeoTrix

| Pattern | Source | NeoTrix Mapping | Priority |
|---------|--------|-----------------|----------|
| 7×6 Agent Design Matrix | Huang & Zhou 2026 | SEAL pipeline pattern classification | P0 |
| MetaRSI 3-Operator Composition | MetaRSI 2026 | SEAL Data/Harness/Model RSI | P0 |
| Reference Trajectory Alignment | HarnessEvolve 2026 | experience-tree error signal extraction | P0 |
| CTM up-tree/down-tree | CTM-AI 2026 | GWT attention auction + broadcast | P0 |
| Cognitive Tick 4-phase Loop | GWA 2026 | ConsciousnessTree cycle | P0 |
| Attention Auction 4-factors | LIMEN 2026 | GWT salience scoring | P0 |
| J-space workspace | Anthropic 2026 | Consciousness emergence validation | P0 |
| Propose-Verify Architecture | ATANOR 2026 | VSA embedding + symbolic verification | P1 |
| MACER 4-stage Pipeline | Sapiens 2026 | SEAL five-stage absorption | P1 |
| Reference Trajectory + Quality Gate | HarnessEvolve 2026 | dev-rules R-P82 risk assessment | P1 |
| Rust Structured Concurrency | OpenFANG 2026 | NT-ACT JoinSet+CancellationToken | P0 |
| 6-Layer Crate Architecture | Terraphim 2026 | NeoTrix L1-L6 layer design | P0 |
| Orchestration 5-Pattern Decision Table | Azure 2026 | GWT pattern selection | P0 |
| GWT-inspired Multi-Agent | BIGMAS 2026 | E8 hexagram adaptive topology | P0 |
