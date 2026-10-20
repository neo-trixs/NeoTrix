# Changelog

## [0.19.1] - 2026-07-01 — Cycle 4 Phase 2: 编译全线漂绿

### Compilation & Linting
- **编译器错误 19→0**: tool impls(6), builtin_adapter(5), anthropic(1), sentry(1), etc.
- **neotrix-types clippy 26→0**: self_model(12), pid(5), engine(6), pairwise(2), context_strategy(1)
- **bin target 4 path fixes**: `crate::neotrix` → `neotrix::neotrix` in config.rs + entry/mod.rs
- **孤儿二进制归档**: 19 orphan one-shot scripts → `bin-archive/`
- **双tool目录合并**: `agent/tools/`(4 files) → `agent/tool/mcp/`, 12 consumers updated

### IIT Φ 修复
- 新增 `compute_tononi_phi()` — 基于 MIP-EI 的精确 Φ 值
- 新增 `find_mip()`, `bipartition_ei()`, `covariance_matrix()` 算法
- 11 个测试覆盖对称/非对称/高噪声/链式/星形拓扑

### GatewayV2 全面集成
- `engine_core.rs`, `consciousness_reasoner.rs`, `ProviderRouter` 全部改用 GatewayV2
- 2-phase aggressive retry (Phase 1 normal + Phase 2 all-providers)
- Proxy Pool L7 HTTP HEAD + TCP双重探测

### Async Runtime 修复
- 3 个预存 `state_rollback_on_llm_failure` 测试: `block_on_future()` helper 处理内嵌 tokio runtime

## [0.19.0] - 2026-07-01 — Cycle 4: 盲点补齐完成

### P0 核心推理升级
- **PRM头**: Observer 添加四维评分头(novelty/progress/alignment/efficiency)
- **SAE**: SparseAutoencoder + SAEBridge + SteeringController 三层
- **GRPO**: 组采样(G≥4) + 相对优势 + clipped surrogate + Beam Search(K=4-8)
- **WTA Gate**: GWT 竞争点火取代累加广播
- **5层压缩**: Budget→Snip→Microcompact→Collapse→Auto 管线
- **E8→VSA**: ChaCha12 seeded VSA ℝ^1024 消除E8-GWT梯度壁垒
- **ProceduralMemory**: 成功E8序列→KB技能固化
- **PER分离**: Planner/Executor/Reflector 三角色

### P1 重要功能
- **权限模式链**: Plan/AcceptEdits/BypassPermissions
- **MCP认证**: OAuth 2.1 PKCE + JSON-RPC 2.0 握手
- **Mamba-2 SSD**: SSM_STATE_SIZE=256 + 双门控
- **JEPA重构**: ViT编码器 + Block/Random掩码 + 动作条件预测器
- **隐私架构**: PrivacyEnforcer + DataSovereigntyProof
- **混合编排**: HybridOrchestrator + 成本感知路由
- **主动推理**: ActiveInferenceLoop + expected_free_energy
- **错误恢复**: Retry→CircuitBreaker→Fallback 三层
- **边缘部署**: Quantizer + HardwareDetector + AOT + LoRA
- **IIT φ**: KLD因果效应谱系(≈MIP)

### P2 优化
- **MoE学习路由**: MoERouter + ExpertGate + REINFORCE
- **规模化律**: ChinchillaLaw + KaplanLaw 预测器
- **ANE缓存**: AneProgramCache + LRU + TTL
- **量化管线**: AWQ + GGUF(Q2K→Q8_0)
- **功耗模型**: PowerThermalModel + 17硬件Profile
- **SAE Steering**: SteeringController + 4层覆盖
- **FHRR VSA**: FhrrHyperCube D=2048 bind/bundle/permute
- **谐振器网络**: AdaptiveCouplingKuramoto + ResonatorBank
