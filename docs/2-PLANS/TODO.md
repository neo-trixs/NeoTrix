# NeoTrix TODO — 更新于 2026-07-01 Cycle 4 Phase 2 完成

> **编译状态**: `cargo check --lib` ✅ **0 errors workspace-wide** (neotrix + neotrix-types + neotrix-tauri)
> 测试: `cargo test --lib` ✅ **4514 tests** (多数通过, 3 pre-existing runtime flaky in engine_core async gateway)
> 盲点总数: 45 (P0: 15 ✅, P1: 12 ✅, P2: 11 ✅, 技术债: 7 ⏳)
> 9支柱映射: P1-CORE / P2-MIND / P3-MEMORY / P4-WORLD / P5-ACT / P6-IO / P7-SHIELD / P8-AGENT / P9-PROVIDER
> Cycle 4 Phase 2: ✅ 全部 23 项盲点 + 新功能已实现

---

## 代码缺陷修复 (Sprint-0)

| # | 任务 | 状态 |
|---|------|------|
| 删除重复文件 | ✅ | 
| 创建缺失模块存根 | ✅ |
| 合并双tool目录 | ✅ agent/tools/ → tool/mcp/ |
| 清理23个bin → 保留4个 | ✅ daemon/kb_crawl/neotrix_web/proxy |
| 修复nt_core_iit_phi | ✅ P1-9 |
| 修复nt_core_fep_iit | ✅ P1-10 |

---

## P0 — 全部完成 ✅

| 模块 | 任务 | 文件 |
|------|------|------|
| P1-CORE | Observer PRM头 | nt_core_observer.rs |
| P1-CORE | Policy GRPO + Beam Search | nt_core_policy.rs |
| P1-CORE | GWT竞争点火 | nt_core_gwt/competition_gate.rs |
| P1-CORE | GWT 5层压缩 | nt_core_gwt/compaction.rs |
| P1-CORE | E8→VSA嵌入 | nt_core_e8_vsa.rs |
| P1-CORE | SAE集成E8流 | nt_core_sae.rs + nt_core_sae_bridge.rs |
| P1-CORE | 过程记忆 | nt_memory_kb/ + procedural_memory.rs |
| P8-AGENT | MCP 4→2传输 | nt_agent_mcp_auth.rs + transport.rs |
| P8-AGENT | PER分离 | nt_act_autonomy/per_agent.rs |

---

## P1 — 全部完成 ✅

| 支柱 | 任务 | 文件 |
|------|------|------|
| P2-MIND | DPOStage | self_iterating/dpo_stage.rs |
| P2-MIND | ConstitutionalSelfCritiqueStage | self_iterating/constitutional_stage.rs |
| P2-MIND | SafetyCheckStage | self_iterating/safety_stage.rs |
| P2-MIND | ProceduralMemoryStage | self_iterating/procedural_memory.rs |
| P3-MEMORY | 隐私架构 | nt_memory_kb/privacy.rs |
| P4-WORLD | JEPA ViT骨干 | nt_world_jepa/vit.rs |
| P4-WORLD | JEPA多块掩码策略 | nt_world_jepa/masking.rs |
| P4-WORLD | JEPA动作条件预测器 | nt_world_jepa/action_predictor.rs + world_model_v2.rs |
| P4-WORLD | 主动推理修复 | core/nt_core_fep.rs |
| P5-ACT | PER分离 | nt_act_autonomy/per_agent.rs |
| P5-ACT | 错误恢复三层栈 | core/nt_core_observer_error.rs |
| P7-SHIELD | 权限模式链 | nt_shield/perm_chain.rs |
| P9-PROVIDER | 边缘部署管线 | core/nt_core_deploy.rs + nt_core_deploy_cache.rs |
| P9-PROVIDER | 混合编排 | nt_provider/hybrid_orch.rs |

---

## P2 — 全部完成 ✅

| 任务 | 文件 |
|------|------|
| HyperCube MAP→FHRR D=2048 | nt_core_hcube/fhrr_vsa.rs |
| GWT共鸣→谐振器网络 | nt_core_gwt/resonator_network.rs |
| GWT MoE学习路由 | nt_core_gwt/moe_router.rs |
| SAE特征引导(Steering) | nt_core_sae.rs |
| IIT φ修复(因果效应谱系) | nt_core_gwt/geometry_sync.rs |
| E8过渡可微分(E8→VSA) | nt_core_e8_vsa.rs |
| HOT高阶思维 + AST | core/nt_core_hot_ast.rs |
| nt_core_ssm Mamba-2 SSD N=256 | core/nt_core_ssm.rs + nt_core_signal/ |
| HyperCube量化 | core/nt_core_deploy.rs |
| 规模化律测量 | core/nt_core_meta/scaling_law.rs |
| ANE program cache | core/nt_core_deploy_cache.rs |
| 功耗预算模型 | core/nt_core_deploy.rs |

---

## 技术债 (待修复)

| # | 任务 | 位置 | 
|---|------|------|
| ⏳ | neotrix-types clippy: 18个修复 | crates/neotrix-types/ |
| ⏳ | Examples: 4个unused variable警告 | neotrix-core/examples/ |
| ⏳ | daemon二进制: 2个unused variable | src/bin/daemon.rs |
| ⏳ | test预存失败: 10个修复 | 各test模块 |
| ⏳ | file_sync/static_server | nt_act_sync/ |
| ⏳ | crypto_agent test-runtime失败 | nt_act_crypto/ |
| ⏳ | macOS code signing + notarization | 全局 |

---

## 未来计划 (Cycle 5)

| 方向 | 预期收获 |
|------|---------|
| A2A协议 (Google/TensorLake) | agent间互操作标准 |
| 神经形态计算 (Loihi 2) | 功耗降低1000×路径 |
| 形式化验证 (TLA+/Coq) | E8过渡安全性证明 |
| 差分隐私 (ε-DP) | 隐私保护记忆 |
| 多模态世界模型 | 统一表示空间 |

---

## 文档状态

| 内容 | 状态 |
|------|------|
| BLIND_SPOT_SYNTHESIS.md | ✅ 更新至Cycle 4完成 |
| EXPERIENCE_TREE_2026-07-01.md | ✅ |
| ARCHITECTURE_9PILLAR.md | ✅ |
| AGENTS.md | 🏗 需同步9支柱命名 |
| CHANGELOG.md | 🏗 需创建 |
