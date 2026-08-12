# Changelog

> This changelog is a wrapper around the git-cliff pipeline. Regenerate the
> `[Unreleased]` body with:
> `git cliff -u` (pipelines defined in `cliff.toml`; used by
> `.github/workflows/release.yml` to emit release notes).

## [Unreleased] — 并行工作批次 + 网络隔离 + 地理瓦片

> git-cliff: `git cliff --unreleased` regenerates the grouped body from
> conventional commits.

### Features
- **nt-core**: CLI 并行批次 — 命令全英文描述 + 动态补全池 + 主题持久化 + shield 写操作 ToolSpec 单一事实源。
- **nt-shield**: 网络隔离默认阻断 — `DeniedProvider` + 本地回环放行 + `NEOTRIX_NETWORK_UNBLOCK` 逃生门。
- **nt-io**: B3 地理瓦片服务 + `/openapi` 端点 — 瓦片路由 + 构建期嵌入 openapi.yaml。
- **nt-mind**: 蜕皮机制融入意识能力网 + C5 自愈养分闭环。
- **nt-core**: 意识核心子代理定义 (`opencode agent nt-core.md`) 与 `status/tick/health/branches` CLI 通道。
- **tauri**: NT-Pack 进程级缓存 + 桌面命令/catalog/gate 演进 + 重启安装更新。
- **frontend**: 代码分包/懒加载 + Globe NT-Pack + CommandPalette/TaskList/LivePreview + 错误监控、更新重启 UX。
- **Capability registry**: 去硬编码读 capability_tree DAG + `neotrix guard` 门禁 + 能力网健康度回流。

### Bug Fixes
- **nt-shield**: `keyvault --features full` 编译修复。
- **nt-core**: 迷雾治理 — 真实 SelfTest 分支健康持久化到 `consciousness/core` 快照。
- **nt-core**: types 测试断言质量 — `unwrap`→`expect` / `len>0`→`is_empty`。

### Chores
- **arch**: 蜕皮归档 — 移除旧躯壳 (frontend-v1/session-log/anchor) 到 `_archive/`，代码树只留最新态。
- **docs/config**: capability registry 重构 + `.gitignore _archive` + progress/docs。

## [Unreleased] — 独立项目进化链路 (G1-G4)

### Project Evolve (G1+G2)
- **`project-evolve` 命令**: 对任意第三方目标项目运行进化链路 (`scan→detect→score→report`), 支持 `--json`/`--autofix`/`--max-rounds`; CLI 插件化入口。
- **EvolutionLoop 目标参数化**: `for_target()`/`scan_project_in()`/`run_cycle_in()`/`autofix_cycle_in()`; `target_dir` 字段 (None=旧行为)。
- **扫描排除非源码目录**: 跳过 `target/.git/.backup/node_modules/_archive` (此前 `.backup/` 被误扫为源码)。

### Autofix 端到端修复
- **修复 `EvolutionLoopProvider::self_diagnose` file=None bug**: `underlying_issue.file` 此前被硬编码为 `None` 丢弃文件路径 → pipeline 对 `AddTestStub`/`SplitLargeFile` 等写空文件路径 `"unknown"` 全部失败。端到端验证 `auto_fixes` 0→1, 测试 stub 真实落盘且编译通过。

### Free_Energy / Phi 接线
- **project-evolve 输出不再恒 0**: `world_fe`/`world_phi` 为 None 时由 `derive_free_energy_phi()` 从项目快照派生 — `free_energy` 接 `ActiveInferenceEngine` (风险密度×精度 + E8 熵), `phi` 接 `IITPhiCalculator` (健康维度共振整合度)。
- 新增单测: 显式值透传 / 派生值有限 / 脏项目自由能高于干净项目。

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
