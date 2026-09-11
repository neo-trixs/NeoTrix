# Model Reverse Engineering — Cycle 342

## 5 Papers

### 1. AgentInfer (arxiv:2512.18337v2)
- **Category**: Agent Co-Design
- **Key Insight**: 4-component framework: AgentCollab + AgentSAM + AgentSched + AgentCompress. 50% token reduction, 1.8-2.5x speedup
- **NeoTrix**: GWT cost-aware routing (A1)

### 2. Latent Action Reparameterization (LAR) (arxiv:2605.18597)
- **Category**: Efficient Agent Action
- **Key Insight**: Learned compact latent action space, multi-step semantic behaviors, reduced decision horizon
- **NeoTrix**: Compressed action space for tool execution (NT-ACT)

### 3. Ring-Linear-2.0 (arxiv:2510.19338v2)
- **Category**: Hybrid Attention
- **Key Insight**: Linear+softmax hybrid, constant KV cache for linear layers, 1/10 cost vs 32B dense
- **NeoTrix**: GWT attention layer optimization

### 4. Governed Memory (arxiv:2603.17787)
- **Category**: Memory Governance
- **Key Insight**: Dual memory model, tiered governance routing, progressive context delivery, 50% token reduction
- **NeoTrix**: Entity-scoped KB isolation (NT-MEMORY)

### 5. Agent-Radar (arxiv:2605.30136)
- **Category**: Attention Steering
- **Key Insight**: Training-free temporal+spatial decay for multi-agent context, up to 7.64 point gains
- **NeoTrix**: PerceptionBridge attention gating (NT-WORLD)

## Cross-Paper Synthesis

Three convergence patterns:
1. **Hybrid attention** (Ring-Linear-2.0, GLIDE) — mix linear and softmax per layer
2. **Memory as control signal** (Governed Memory, Cognee) — KB influences routing, not just storage
3. **Self-evaluation routing** (AgentCollab, AgentInfer) — models decide when to escalate
